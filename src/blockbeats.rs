use std::io::{Error as IoError, ErrorKind};
use std::time::Duration;

use crate::newsflow::UnifiedNewsItem;

pub type BlockBeatsResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub const BLOCKBEATS_URL: &str =
    "https://api-pro.theblockbeats.info/v1/newsflash?page=1&size=10&lang=cn";
const BLOCKBEATS_FLASH_LINK_PREFIX: &str = "www.theblockbeats.info/flash/";
const BLOCKBEATS_POLL_WITH_NEWS_SECS: u64 = 600;
const BLOCKBEATS_POLL_IDLE_SECS: u64 = 300;

#[derive(Debug, Default)]
pub struct BlockBeatsCrawler {
    topid: Option<String>,
    next_poll_secs: u64,
}

impl BlockBeatsCrawler {
    pub fn new() -> Self {
        Self {
            topid: None,
            next_poll_secs: BLOCKBEATS_POLL_WITH_NEWS_SECS,
        }
    }

    pub fn topid(&self) -> Option<&str> {
        self.topid.as_deref()
    }

    pub fn next_poll_secs(&self) -> u64 {
        self.next_poll_secs
    }

    // What: 调用 Pro API 并归一化为现有快讯列表结构。
    // Why: 保持下游增量去重逻辑不变，避免来源协议升级扩散到发送链路。
    async fn fetch_payload(&self, client: &reqwest::Client) -> BlockBeatsResult<serde_json::Value> {
        let config: toml::Value = toml::from_str(&std::fs::read_to_string("./config.toml")?)?;
        let api_key = config
            .get("blockbeats_api_key")
            .and_then(toml::Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "blockbeats_api_key is missing"))?;
        let payload = client
            .get(BLOCKBEATS_URL)
            .header("api-key", api_key)
            .timeout(Duration::from_secs(30))
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;
        let list = payload
            .pointer("/data/data")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "blockbeats Pro data is missing"))?
            .iter()
            .map(|item| {
                let mut item = item.clone();
                item["article_id"] = item.get("id").cloned().unwrap_or_default();
                item
            })
            .collect::<Vec<_>>();
        Ok(serde_json::json!({"data": {"list": list}}))
    }

    // Why: 把“按 topid 截断并更新游标”固定在爬虫内，流程层无需再感知来源字段细节。
    fn collect_new_items(
        &mut self,
        raw_list: Vec<serde_json::Value>,
    ) -> BlockBeatsResult<Vec<UnifiedNewsItem>> {
        if raw_list.is_empty() {
            self.next_poll_secs = BLOCKBEATS_POLL_IDLE_SECS;
            return Ok(Vec::new());
        }

        let latest_id = raw_list[0]
            .get("article_id")
            .and_then(|value| match value {
                serde_json::Value::Number(v) => Some(v.to_string()),
                serde_json::Value::String(v) => Some(v.clone()),
                _ => None,
            })
            .ok_or_else(|| {
                IoError::new(
                    ErrorKind::InvalidData,
                    "invalid article_id type in first item",
                )
            })?;

        let mut news = Vec::new();
        for item in raw_list {
            let article_id = item
                .get("article_id")
                .and_then(|value| match value {
                    serde_json::Value::Number(v) => Some(v.to_string()),
                    serde_json::Value::String(v) => Some(v.clone()),
                    _ => None,
                })
                .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "invalid article_id type"))?;
            if self.topid.as_deref() == Some(article_id.as_str()) {
                break;
            }

            // Why: 抓取层直接转成统一新闻模型，流程层不再感知来源字段命名差异。
            news.push(UnifiedNewsItem {
                id: article_id.clone(),
                url: format!("{BLOCKBEATS_FLASH_LINK_PREFIX}{article_id}"),
                body: strip_html_tags(
                    item.get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default(),
                ),
                title: item
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
            });
        }

        self.next_poll_secs = if news.is_empty() {
            BLOCKBEATS_POLL_IDLE_SECS
        } else {
            BLOCKBEATS_POLL_WITH_NEWS_SECS
        };
        self.topid = Some(latest_id);
        Ok(news)
    }

    pub async fn fetch_new_items(
        &mut self,
        client: &reqwest::Client,
    ) -> BlockBeatsResult<Vec<UnifiedNewsItem>> {
        let payload = self.fetch_payload(client).await?;
        let raw_list = payload
            .get("data")
            .and_then(|v| v.get("list"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                IoError::new(
                    ErrorKind::InvalidData,
                    "blockbeats response data.list is missing or invalid",
                )
            })?
            .clone();
        self.collect_new_items(raw_list)
    }
}

// Why: 对齐 Python 基线的正则清洗语义，推送正文只保留可读文本，避免 HTML 标签污染通知。
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
