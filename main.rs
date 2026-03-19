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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Why: 各来源统一走 news_loop，main 仅负责装配依赖与启动顺序，避免入口继续膨胀。
    let client = reqwest::Client::new();
    let (sender, receiver) = newsflow::build_news_queue();
    let _sender_task = tokio::spawn(async move {
        if let Err(err) = newsflow::run_global_sender(receiver).await {
            println!("global sender failed: {err}");
        }
    });
    // let _pan_task = tokio::spawn(news_loop::run_source_loop(
    //     news_loop::pannews_source(),
    //     client.clone(),
    //     sender.clone(),
    // ));
    let _beats_task = tokio::spawn(news_loop::run_source_loop(
        news_loop::blockbeats_source(),
        client.clone(),
        sender.clone(),
    ));
    // let _odaily_task = tokio::spawn(news_loop::run_source_loop(
    //     news_loop::odaily_source(),
    //     client.clone(),
    //     sender.clone(),
    // ));
    // let _bwe_task = tokio::spawn(news_loop::run_source_loop(
    //     news_loop::bwenews_source(),
    //     client,
    //     sender,
    // ));
    webpage::serve_index_html("0.0.0.0:80", "index/index.html").await?;
    Ok(())
}
