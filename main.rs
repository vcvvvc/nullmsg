#[path = "src/blockbeats.rs"]
mod blockbeats;
#[path = "src/bwenews.rs"]
mod bwenews;
#[path = "src/crypto.rs"]
mod crypto;
#[path = "src/news_loop.rs"]
mod news_loop;
#[path = "src/newsflow.rs"]
mod newsflow;
#[path = "src/odaily.rs"]
mod odaily;
#[path = "src/pannews.rs"]
mod pannews;
#[path = "src/pusher.rs"]
mod pusher;
#[path = "src/webpage.rs"]
mod webpage;

const SOURCE_RESTART_DELAY_SECS: u64 = 60;

fn spawn_source_supervisor(
    source_label: &'static str,
    source_factory: fn() -> news_loop::NewsSourceLoop,
    client: reqwest::Client,
    sender: newsflow::NewsSender,
) -> tokio::task::JoinHandle<()> {
    // Why: 来源任务异常退出后自动拉起，避免长跑进程出现“某一路静默死亡”。
    tokio::spawn(async move {
        loop {
            let source_loop = source_factory();
            let join = tokio::spawn(news_loop::run_source_loop(
                source_loop,
                client.clone(),
                sender.clone(),
            ));
            match join.await {
                Ok(()) => println!("source loop exited [{source_label}]"),
                Err(err) => println!("source loop crashed [{source_label}]: {err}"),
            }
            tokio::time::sleep(std::time::Duration::from_secs(SOURCE_RESTART_DELAY_SECS)).await;
        }
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Why: 各来源统一走 news_loop，main 仅负责装配依赖与启动顺序，避免入口继续膨胀。
    let client = reqwest::Client::new();
    let (sender, receiver) = newsflow::build_news_queue();
    let _sender_task = tokio::spawn(async move {
        newsflow::run_global_sender(receiver).await;
    });
    let _pan_task = spawn_source_supervisor(
        "pannews",
        news_loop::pannews_source,
        client.clone(),
        sender.clone(),
    );
    let _beats_task = spawn_source_supervisor(
        "blockbeats",
        news_loop::blockbeats_source,
        client.clone(),
        sender.clone(),
    );
    let _odaily_task = spawn_source_supervisor(
        "odaily",
        news_loop::odaily_source,
        client.clone(),
        sender.clone(),
    );
    let _bwe_task = spawn_source_supervisor("bwenews", news_loop::bwenews_source, client, sender);
    webpage::serve_index_html("0.0.0.0:10000", "index/index.html").await?;
    Ok(())
}
