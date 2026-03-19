use std::io::{Cursor, Error as IoError, ErrorKind};
use std::time::Duration;

use crate::newsflow::UnifiedNewsItem;

pub type BweNewsResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const BWE_RSS_URL: &str = "https://rss-public.bwe-ws.com/";
const BWE_POLL_WITH_NEWS_SECS: u64 = 120;
const BWE_POLL_IDLE_SECS: u64 = 30;
const BWE_POLL_EMPTY_SECS: u64 = 300;

#[derive(Debug, Default)]
pub struct BweNewsCrawler {
    top_id: Option<String>,
    next_poll_secs: u64,
}

impl BweNewsCrawler {
    pub fn new() -> Self {
        Self {
            top_id: None,
            next_poll_secs: BWE_POLL_WITH_NEWS_SECS,
        }
    }

    pub fn next_poll_secs(&self) -> u64 {
        self.next_poll_secs
    }

    // Why: RSS 源与 JSON 源解析方式不同，单独封装拉取+解析可隔离依赖与错误边界。
    async fn fetch_feed(&self, client: &reqwest::Client) -> BweNewsResult<rss::Channel> {
        let bytes = client
            .get(BWE_RSS_URL)
            .header(
                reqwest::header::USER_AGENT,
                "Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1",
            )
            .timeout(Duration::from_secs(30))
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        Ok(rss::Channel::read_from(Cursor::new(bytes))?)
    }

    // Why: 复用 Python 的“最新 ID 对比 + 倒序发送新消息”语义，保证迁移后行为一致。
    fn collect_new_items(&mut self, entries: &[rss::Item]) -> BweNewsResult<Vec<UnifiedNewsItem>> {
        if entries.is_empty() {
            self.next_poll_secs = BWE_POLL_EMPTY_SECS;
            return Ok(Vec::new());
        }

        let latest = entries
            .last()
            .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "empty bwe feed entries"))?;
        let latest_id = entry_id(latest).ok_or_else(|| {
            IoError::new(ErrorKind::InvalidData, "missing id/link in latest bwe item")
        })?;

        if self.top_id.as_deref() == Some(latest_id.as_str()) {
            self.next_poll_secs = BWE_POLL_IDLE_SECS;
            return Ok(Vec::new());
        }

        let mut items = Vec::new();
        for entry in entries.iter().rev() {
            let id = match entry_id(entry) {
                Some(v) => v,
                None => continue,
            };
            if self.top_id.as_deref() == Some(id.as_str()) {
                break;
            }

            let raw_title = entry.title().unwrap_or_default();
            let (title, body) =
                split_bwe_title_and_body(raw_title, entry.description().unwrap_or_default());
            let url = entry
                .link()
                .unwrap_or_default()
                .trim_start_matches("https://")
                .to_string();
            items.push(UnifiedNewsItem {
                id,
                title,
                body,
                url,
            });
        }

        self.next_poll_secs = if items.is_empty() {
            BWE_POLL_IDLE_SECS
        } else {
            BWE_POLL_WITH_NEWS_SECS
        };
        self.top_id = Some(latest_id);
        Ok(items)
    }

    pub async fn fetch_new_items(
        &mut self,
        client: &reqwest::Client,
    ) -> BweNewsResult<Vec<UnifiedNewsItem>> {
        let feed = self.fetch_feed(client).await?;
        self.collect_new_items(feed.items())
    }
}

fn entry_id(entry: &rss::Item) -> Option<String> {
    if let Some(guid) = entry.guid().map(|v| v.value().to_string()) {
        return Some(guid);
    }
    entry.link().map(str::to_string)
}

fn split_bwe_title_and_body(raw_title: &str, fallback_desc: &str) -> (String, String) {
    let (title_part, content_part) = if let Some((left, right)) = raw_title.split_once("<br/>") {
        (left, right)
    } else if let Some((left, right)) = raw_title.split_once("<br>") {
        (left, right)
    } else {
        (raw_title, fallback_desc)
    };
    let body = content_part
        .replace("<br/>", "\n")
        .replace("<br>", "\n")
        .replace('—', " ")
        .trim()
        .to_string();
    (title_part.trim().to_string(), body)
}
