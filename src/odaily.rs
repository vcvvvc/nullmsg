use std::io::{Error as IoError, ErrorKind};
use std::time::Duration;

use crate::newsflow::UnifiedNewsItem;

pub type ODailyResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const ODAILY_API: &str = "https://h5-api.odaily.news/newsflash/page?isImport=false&page=1&size=20";
const ODAILY_REFERER: &str = "https://m.odaily.news";
const ODAILY_LINK_PREFIX: &str = "www.odaily.news/newsflash/";
const ODAILY_POLL_WITH_NEWS_SECS: u64 = 600;
const ODAILY_POLL_IDLE_SECS: u64 = 300;

#[derive(Debug, Default)]
pub struct ODailyCrawler {
    top_id: Option<String>,
    next_poll_secs: u64,
}

impl ODailyCrawler {
    pub fn new() -> Self {
        Self {
            top_id: Some("0".to_string()),
            next_poll_secs: ODAILY_POLL_WITH_NEWS_SECS,
        }
    }

    pub fn next_poll_secs(&self) -> u64 {
        self.next_poll_secs
    }

    // Why: 与 Python 基线保持同一请求入口与请求头，避免因接口策略差异导致抓取行为漂移。
    async fn fetch_payload(&self, client: &reqwest::Client) -> ODailyResult<serde_json::Value> {
        Ok(client
            .get(ODAILY_API)
            .header(
                reqwest::header::USER_AGENT,
                "Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1",
            )
            .header(reqwest::header::REFERER, ODAILY_REFERER)
            .timeout(Duration::from_secs(30))
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?)
    }

    // Why: 把“按游标截断 + 文本清洗 + 下轮间隔更新”固定在来源内部，流程层只消费统一模型。
    fn collect_new_items(
        &mut self,
        raw_list: Vec<serde_json::Value>,
    ) -> ODailyResult<Vec<UnifiedNewsItem>> {
        if raw_list.is_empty() {
            self.next_poll_secs = ODAILY_POLL_IDLE_SECS;
            return Ok(Vec::new());
        }

        let latest_id = parse_odaily_id(&raw_list[0]).ok_or_else(|| {
            IoError::new(ErrorKind::InvalidData, "invalid id in first odaily item")
        })?;

        let mut items = Vec::new();
        for raw in raw_list {
            let news_id = parse_odaily_id(&raw)
                .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "invalid odaily id"))?;
            if self.top_id.as_deref() == Some(news_id.as_str()) {
                break;
            }

            let title = raw
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let body_html = raw
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            items.push(UnifiedNewsItem {
                id: news_id.clone(),
                title,
                body: strip_html_tags(body_html),
                url: format!("{ODAILY_LINK_PREFIX}{news_id}"),
            });
        }

        self.next_poll_secs = if items.is_empty() {
            ODAILY_POLL_IDLE_SECS
        } else {
            ODAILY_POLL_WITH_NEWS_SECS
        };
        self.top_id = Some(latest_id);
        Ok(items)
    }

    pub async fn fetch_new_items(
        &mut self,
        client: &reqwest::Client,
    ) -> ODailyResult<Vec<UnifiedNewsItem>> {
        let payload = self.fetch_payload(client).await?;
        let raw_list = payload
            .get("data")
            .and_then(|v| v.get("list"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                IoError::new(
                    ErrorKind::InvalidData,
                    "odaily response data.list is missing or invalid",
                )
            })?
            .clone();
        self.collect_new_items(raw_list)
    }
}

fn parse_odaily_id(item: &serde_json::Value) -> Option<String> {
    let value = item.get("id")?;
    match value {
        serde_json::Value::Number(v) => Some(v.to_string()),
        serde_json::Value::String(v) => Some(v.clone()),
        _ => None,
    }
}

fn strip_html_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    output
}
