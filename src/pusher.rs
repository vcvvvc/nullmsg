use crate::crypto;
use std::time::Duration;

const DEFAULT_PUSH_HOST: &str = "https://bark-test-cje9.onrender.com";
const PUSH_PATH: &str = "/quicknews";
const PUSH_TIMEOUT_SECS: u64 = 30;

pub type PushResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone)]
pub struct PushNews {
    pub title: String,
    pub body: String,
    pub url: String,
    pub group: String,
}

#[derive(serde::Serialize)]
struct EncryptPayload<'a> {
    title: &'a str,
    body: &'a str,
    url: &'a str,
    sound: &'a str,
    group: &'a str,
}

#[derive(serde::Deserialize)]
struct PusherConfig {
    server_host: String,
    server_key: String,
    aes_key: String,
    aes_iv: String,
}

pub struct Pusher {
    client: reqwest::Client,
    send_url: String,
    aes_key: String,
    aes_iv: String,
}

impl Pusher {
    pub fn from_config_file(path: &str) -> PushResult<Self> {
        // 配置读取下沉到发送模块，避免 aes_key/aes_iv 在调用链上层层透传造成耦合扩散。
        let raw = std::fs::read_to_string(path)?;
        let cfg: PusherConfig = toml::from_str(&raw)?;
        Self::new(&cfg.server_host, &cfg.server_key, &cfg.aes_key, &cfg.aes_iv)
    }

    fn new(server_host: &str, _server_key: &str, aes_key: &str, aes_iv: &str) -> PushResult<Self> {
        // 与 Python 版对齐：host 为空时给默认值，URL 固定拼接 /quicknews，避免路径散落在业务代码里。
        let host = normalize_host(server_host);
        let send_url = format!("{}{PUSH_PATH}", host.trim_end_matches('/'));
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(PUSH_TIMEOUT_SECS))
            .build()?;

        Ok(Self {
            client,
            send_url,
            aes_key: aes_key.to_string(),
            aes_iv: aes_iv.to_string(),
        })
    }

    fn encrypt_news(&self, news: &PushNews) -> PushResult<String> {
        let payload = EncryptPayload {
            title: &news.title,
            body: &news.body,
            url: &news.url,
            sound: "healthnotification",
            group: &news.group,
        };
        let json_text = serde_json::to_string(&payload)?;
        Ok(crypto::encrypt(&json_text, &self.aes_key, &self.aes_iv)?)
    }

    pub async fn send_news(&self, news: &PushNews) -> PushResult<()> {
        let ciphertext = self.encrypt_news(news)?;
        // ciphertext 是 Base64；若直接放入 form，`+` 会被当作空格，服务端拿到的密文将发生位级漂移。
        let body = format!("ciphertext={}", percent_encode_form_value(&ciphertext));
        let mut last_error: Option<String> = None;
        // 与 Python 基线对齐：发送失败时最多重试 3 次，避免单次瞬时网络抖动直接丢消息。
        for attempt in 1..=3 {
            let result = self
                .client
                .post(&self.send_url)
                .header(
                    reqwest::header::USER_AGENT,
                    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15",
                )
                .header(
                    reqwest::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .body(body.clone())
                .send()
                .await;
            match result.and_then(|resp| resp.error_for_status()) {
                Ok(_) => return Ok(()),
                Err(err) => last_error = Some(err.to_string()),
            }
            if attempt < 3 {
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
        Err(std::io::Error::other(format!(
            "push failed after 3 attempts: {}",
            last_error.unwrap_or_else(|| "unknown error".to_string())
        ))
        .into())
    }
}

// 我的host自带https 不需要额外处理。
fn normalize_host(server_host: &str) -> String {
    if server_host.trim().is_empty() {
        DEFAULT_PUSH_HOST
    } else {
        server_host.trim()
    }
    .to_string()
}

fn percent_encode_form_value(input: &str) -> String {
    // 手工构造 x-www-form-urlencoded 请求体时，必须对 value 做百分号编码，避免 `+ / =` 被表单协议重解释。
    fn to_hex_upper(n: u8) -> char {
        match n {
            0..=9 => (b'0' + n) as char,
            _ => (b'A' + (n - 10)) as char,
        }
    }

    let mut out = String::with_capacity(input.len());
    for b in input.bytes() {
        let is_unreserved = matches!(
            b,
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~'
        );
        if is_unreserved {
            out.push(b as char);
            continue;
        }
        out.push('%');
        out.push(to_hex_upper((b >> 4) & 0x0F));
        out.push(to_hex_upper(b & 0x0F));
    }
    out
}
