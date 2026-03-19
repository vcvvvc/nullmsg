use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::blockbeats;
use crate::bwenews;
use crate::newsflow::{self, NewsSender, UnifiedNewsItem};
use crate::odaily;
use crate::pannews;

pub const DEFAULT_FETCH_INTERVAL_SECS: u64 = 600;

pub struct FetchOutcome {
    pub items: Vec<UnifiedNewsItem>,
    pub sleep_secs: u64,
}

pub type FetchResult = Result<FetchOutcome, Box<dyn std::error::Error + Send + Sync>>;
pub type FetchFuture = Pin<Box<dyn Future<Output = FetchResult> + Send>>;
pub type FetchFn = Arc<dyn Fn(reqwest::Client) -> FetchFuture + Send + Sync>;

pub struct NewsSourceLoop {
    source: &'static str,
    source_label: &'static str,
    fetch: FetchFn,
}

impl NewsSourceLoop {
    // Why: 把“来源标识 + 抓取函数”打包成一个最小单元，main 只负责拼装而不承载循环细节。
    pub fn new(
        source: &'static str,
        source_label: &'static str,
        fetch: impl Fn(reqwest::Client) -> FetchFuture + Send + Sync + 'static,
    ) -> Self {
        Self {
            source,
            source_label,
            fetch: Arc::new(fetch),
        }
    }
}

// Why: 来源专用状态封装在循环模块内，main 只保留装配语义，避免入口被 Arc/Mutex 细节污染。
pub fn pannews_source() -> NewsSourceLoop {
    let crawler = Arc::new(tokio::sync::Mutex::new(pannews::PanNewsCrawler::new()));
    NewsSourceLoop::new("PanNews", "pannews", {
        let crawler = Arc::clone(&crawler);
        move |client: reqwest::Client| {
            let crawler = Arc::clone(&crawler);
            Box::pin(async move {
                let mut crawler = crawler.lock().await;
                let items = crawler.fetch_new_items(&client).await?;
                Ok(FetchOutcome {
                    items,
                    sleep_secs: crawler.next_poll_secs(),
                })
            })
        }
    })
}

// Why: 与 PanNews 走同一装配模式，后续新增来源只增工厂函数，不扩散 main 的结构复杂度。
pub fn blockbeats_source() -> NewsSourceLoop {
    let crawler = Arc::new(tokio::sync::Mutex::new(blockbeats::BlockBeatsCrawler::new()));
    NewsSourceLoop::new("BlockBeats", "blockbeats", {
        let crawler = Arc::clone(&crawler);
        move |client: reqwest::Client| {
            let crawler = Arc::clone(&crawler);
            Box::pin(async move {
                let mut crawler = crawler.lock().await;
                let items = crawler.fetch_new_items(&client).await?;
                Ok(FetchOutcome {
                    items,
                    sleep_secs: crawler.next_poll_secs(),
                })
            })
        }
    })
}

// Why: Odaily 走同一来源工厂模板，新增来源时只新增工厂函数，保持循环层稳定。
pub fn odaily_source() -> NewsSourceLoop {
    let crawler = Arc::new(tokio::sync::Mutex::new(odaily::ODailyCrawler::new()));
    NewsSourceLoop::new("oDaily", "odaily", {
        let crawler = Arc::clone(&crawler);
        move |client: reqwest::Client| {
            let crawler = Arc::clone(&crawler);
            Box::pin(async move {
                let mut crawler = crawler.lock().await;
                let items = crawler.fetch_new_items(&client).await?;
                Ok(FetchOutcome {
                    items,
                    sleep_secs: crawler.next_poll_secs(),
                })
            })
        }
    })
}

// Why: BWE RSS 源和 JSON 源统一在同一循环协议下运行，发送层无需感知来源差异。
pub fn bwenews_source() -> NewsSourceLoop {
    let crawler = Arc::new(tokio::sync::Mutex::new(bwenews::BweNewsCrawler::new()));
    NewsSourceLoop::new("BWEnews", "bwenews", {
        let crawler = Arc::clone(&crawler);
        move |client: reqwest::Client| {
            let crawler = Arc::clone(&crawler);
            Box::pin(async move {
                let mut crawler = crawler.lock().await;
                let items = crawler.fetch_new_items(&client).await?;
                Ok(FetchOutcome {
                    items,
                    sleep_secs: crawler.next_poll_secs(),
                })
            })
        }
    })
}

// Why: 统一承载“抓取 -> 发送 -> 固定间隔休眠”主干，避免每个新闻源复制一份循环模板。
pub async fn run_source_loop(
    source_loop: NewsSourceLoop,
    client: reqwest::Client,
    sender: NewsSender,
) {
    loop {
        let mut sleep_secs = DEFAULT_FETCH_INTERVAL_SECS;
        let fetch_result = match (source_loop.fetch)(client.clone()).await {
            Ok(outcome) => {
                sleep_secs = outcome.sleep_secs.max(1);
                Ok(outcome.items)
            }
            Err(err) => Err(err),
        };
        if let Err(err) = newsflow::process_source_once(
            source_loop.source,
            source_loop.source_label,
            fetch_result,
            &sender,
        )
        .await
        {
            println!("{} process failed: {}", source_loop.source_label, err);
        }
        tokio::time::sleep(std::time::Duration::from_secs(sleep_secs)).await;
    }
}
