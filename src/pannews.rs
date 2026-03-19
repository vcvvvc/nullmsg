use serde::Deserialize;

use crate::newsflow::UnifiedNewsItem;

pub type PanNewsResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

// Why: 先集中定义接口常量，避免后续抓取/测试代码重复硬编码 URL，降低与 Python 基线的偏移风险。
pub const PANNEWS_FLASH_API: &str =
    "https://universal-api.panewslab.com/articles?type=NEWS&isShowInList=true&take=20&skip=0";
const PANNEWS_ARTICLE_PATH_PREFIX: &str = "https://www.panewslab.com/zh/articles/";
const PANNEWS_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:142.0) Gecko/20100101 Firefox/142.0";
const PANNEWS_ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
const PANNEWS_ACCEPT_LANGUAGE: &str = "zh-CN,zh;q=0.8,zh-TW;q=0.7,zh-HK;q=0.5,en-US;q=0.3,en;q=0.2";
const PANNEWS_POLL_WITH_NEWS_SECS: u64 = 600;
const PANNEWS_POLL_IDLE_SECS: u64 = 300;

#[derive(Debug, Deserialize)]
struct PanNewsRawItem {
    id: String,
    title: String,
    #[serde(alias = "desc", alias = "content", alias = "description")]
    description: String,
}

impl PanNewsRawItem {
    // Why: 把字段清洗规则集中在一个出口，避免发送链路中重复处理 `\r\n` 和链接拼接导致行为漂移。
    fn into_news_item(self) -> UnifiedNewsItem {
        UnifiedNewsItem {
            url: format!("{PANNEWS_ARTICLE_PATH_PREFIX}{}", self.id),
            id: self.id,
            title: self.title,
            body: self.description.replace("\r\n", "\n"),
        }
    }
}

#[derive(Debug, Default)]
pub struct PanNewsCrawler {
    top_id: Option<String>,
    next_poll_secs: u64,
}

impl PanNewsCrawler {
    pub fn new() -> Self {
        // Why: 与 Python 基线保持同一初始游标语义，避免首次启动行为偏移。
        Self {
            top_id: Some("1".to_string()),
            next_poll_secs: PANNEWS_POLL_WITH_NEWS_SECS,
        }
    }

    pub fn top_id(&self) -> Option<&str> {
        self.top_id.as_deref()
    }

    pub fn next_poll_secs(&self) -> u64 {
        self.next_poll_secs
    }

    // Why: 先把“拉取+反序列化”封装成独立最小单元，后续流程层只关心去重与发送，降低耦合。
    async fn fetch_payload(&self, client: &reqwest::Client) -> PanNewsResult<serde_json::Value> {
        // Why: 对齐 Python 端请求头可降低接口侧的反爬分歧，减少同一代码在不同环境下响应漂移。
        let response = client
            .get(PANNEWS_FLASH_API)
            .header(reqwest::header::USER_AGENT, PANNEWS_USER_AGENT)
            .header(reqwest::header::ACCEPT, PANNEWS_ACCEPT)
            .header(reqwest::header::ACCEPT_LANGUAGE, PANNEWS_ACCEPT_LANGUAGE)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(
                std::io::Error::other(format!("pannews status {}", response.status())).into(),
            );
        }
        Ok(response.json::<serde_json::Value>().await?)
    }

    // Why: 新接口返回形状不稳定时，统一在此做列表抽取，降低主流程分支噪音。
    fn extract_raw_list(payload: serde_json::Value) -> Vec<serde_json::Value> {
        if let Some(list) = payload.as_array() {
            return list.to_vec();
        }
        if let Some(list) = payload.get("data").and_then(|v| v.as_array()) {
            return list.to_vec();
        }
        if let Some(list) = payload.get("items").and_then(|v| v.as_array()) {
            return list.to_vec();
        }
        if let Some(list) = payload.get("list").and_then(|v| v.as_array()) {
            return list.to_vec();
        }
        Vec::new()
    }

    // Why: 用宽松映射容忍上游字段命名变体，避免接口小改动导致整批抓取失败。
    fn map_raw_item(raw: serde_json::Value) -> Option<PanNewsRawItem> {
        let id = raw
            .get("id")
            .or_else(|| raw.get("_id"))
            .or_else(|| raw.get("articleId"))
            .and_then(|v| v.as_str())
            .map(str::to_string)?;
        let title = raw
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let description = raw
            .get("desc")
            .or_else(|| raw.get("description"))
            .or_else(|| raw.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        Some(PanNewsRawItem {
            id,
            title,
            description,
        })
    }

    // Why: 给流程层提供单入口，避免在 newsflow 中重复感知 PanNews 的 JSON 层级细节。
    pub async fn fetch_new_items(
        &mut self,
        client: &reqwest::Client,
    ) -> PanNewsResult<Vec<UnifiedNewsItem>> {
        let payload = self.fetch_payload(client).await?;
        let raw_list = Self::extract_raw_list(payload);
        let items = raw_list
            .into_iter()
            .filter_map(Self::map_raw_item)
            .collect::<Vec<_>>();
        Ok(self.collect_new_items(items))
    }

    // Why: 把“按上次游标去重并更新游标”的规则固定在爬虫内部，避免流程层重复写 break/更新逻辑。
    fn collect_new_items(&mut self, raw_list: Vec<PanNewsRawItem>) -> Vec<UnifiedNewsItem> {
        if raw_list.is_empty() {
            self.next_poll_secs = PANNEWS_POLL_IDLE_SECS;
            return Vec::new();
        }

        let latest_id = raw_list[0].id.clone();
        let mut items = Vec::new();
        for raw in raw_list {
            if self.top_id.as_deref() == Some(raw.id.as_str()) {
                break;
            }
            items.push(raw.into_news_item());
        }

        self.next_poll_secs = if items.is_empty() {
            PANNEWS_POLL_IDLE_SECS
        } else {
            PANNEWS_POLL_WITH_NEWS_SECS
        };
        self.top_id = Some(latest_id);
        items
    }
}
