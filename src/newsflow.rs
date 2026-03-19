use crate::pusher::{PushNews, Pusher};
use std::fmt::Display;
use tokio::sync::mpsc;

const SEND_INTERVAL_SECS: u64 = 1;
const SEND_QUEUE_SIZE: usize = 1024;

#[derive(Debug)]
pub struct UnifiedNewsItem {
    pub id: String,
    pub title: String,
    pub body: String,
    pub url: String,
}

pub type NewsSender = mpsc::Sender<PushNews>;
pub type NewsReceiver = mpsc::Receiver<PushNews>;

pub fn build_news_queue() -> (NewsSender, NewsReceiver) {
    mpsc::channel(SEND_QUEUE_SIZE)
}

pub async fn run_global_sender(
    mut receiver: NewsReceiver,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Why: 全局只保留一个发送器实例，确保所有来源共享同一节流窗口，避免并发推送打爆通知端。
    let pusher = Pusher::from_config_file("./config.toml")?;
    while let Some(payload) = receiver.recv().await {
        match pusher.send_news(&payload).await {
            Ok(()) => println!("push ok [{}]: {}", payload.group, payload.title),
            Err(err) => println!(
                "push failed [{}]: {} | {}",
                payload.group, payload.title, err
            ),
        }
        tokio::time::sleep(std::time::Duration::from_secs(SEND_INTERVAL_SECS)).await;
    }
    Ok(())
}

pub async fn process_news_batch<I>(source: &str, items: I, sender: &NewsSender)
where
    I: IntoIterator<Item = UnifiedNewsItem>,
{
    // Why: 抓取线程只负责入队，发送线程统一消费，保证全局发送节流与来源解耦。
    for item in items {
        let payload = map_news(source, &item);
        if let Err(err) = sender.send(payload).await {
            println!("enqueue failed [{}]: {}", source, err);
            break;
        }
    }
}

pub async fn process_source_once<E>(
    source: &str,
    source_label: &str,
    fetch_result: Result<Vec<UnifiedNewsItem>, E>,
    sender: &NewsSender,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    E: Display,
{
    // Why: 统一“抓取结果 -> 预览日志 -> 入全局队列”主干，来源线程不再直接触发网络发送。
    process_source_result(source, source_label, fetch_result, sender).await;
    Ok(())
}

async fn process_source_result<E>(
    source: &str,
    source_label: &str,
    fetch_result: Result<Vec<UnifiedNewsItem>, E>,
    sender: &NewsSender,
) where
    E: Display,
{
    // Why: 把“抓取结果处理 + 预览日志 + 入队”收敛成模板，避免每个新闻源重复一套样板代码。
    match fetch_result {
        Ok(news_list) => {
            log_source_preview(source_label, &news_list);
            process_news_batch(source, news_list, sender).await;
        }
        Err(err) => println!("{source_label} fetch failed: {err}"),
    }
}

fn log_source_preview(source_label: &str, news_list: &[UnifiedNewsItem]) {
    println!("{source_label} fetched: {} items", news_list.len());
    for item in news_list.iter() {
        println!(
            "{source_label} item: {} | {} | {}",
            item.id, item.title, item.url
        );
    }
}

fn map_news(group: &str, news: &UnifiedNewsItem) -> PushNews {
    // Why: 只保留一个映射出口，避免按来源复制 map_xxx 函数导致协议字段分叉。
    PushNews {
        title: news.title.clone(),
        body: news.body.clone(),
        url: news.url.clone(),
        group: group.to_string(),
    }
}
