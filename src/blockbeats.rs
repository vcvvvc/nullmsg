use std::io::{Error as IoError, ErrorKind};
use std::time::Duration;

use crate::newsflow::UnifiedNewsItem;

pub type BlockBeatsResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub const BLOCKBEATS_URL: &str = "https://api.blockbeats.cn/v2/newsflash/list?page=1&limit=10";
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

    // Why: 把“请求+反序列化”单独封装，主流程只保留增量去重语义，降低阅读复杂度。
    async fn fetch_payload(&self, client: &reqwest::Client) -> BlockBeatsResult<serde_json::Value> {
        Ok(client
            .get(BLOCKBEATS_URL)
            .header(
                reqwest::header::USER_AGENT,
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36",
            )
            .header(reqwest::header::HOST, "api.blockbeats.cn")
            .timeout(Duration::from_secs(30))
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?)
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
