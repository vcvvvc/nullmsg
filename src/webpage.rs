use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

type WebResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn serve_index_html(listen_addr: &str, html_file: &str) -> WebResult<()> {
    // Why: 只保留最小页面托管能力，固定返回 index/index.html，减少分支与辅助函数。
    let listener = TcpListener::bind(listen_addr).await?;
    println!("html server listening on http://{listen_addr} (file: {html_file})");
    let body = std::fs::read_to_string(html_file).unwrap_or_else(|_| "hello world".to_string());
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );

    loop {
        let (mut socket, _) = match listener.accept().await {
            Ok(pair) => pair,
            Err(err) => {
                println!("web accept failed: {err}");
                continue;
            }
        };
        let mut req_buf = [0u8; 512];
        if let Err(err) = socket.read(&mut req_buf).await {
            println!("web read failed: {err}");
            continue;
        }
        if let Err(err) = socket.write_all(response.as_bytes()).await {
            println!("web write failed: {err}");
            continue;
        }
        if let Err(err) = socket.shutdown().await {
            println!("web shutdown failed: {err}");
        }
    }
}
