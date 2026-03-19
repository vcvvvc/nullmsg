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
        let (mut socket, _) = listener.accept().await?;
        let mut req_buf = [0u8; 512];
        let _ = socket.read(&mut req_buf).await;
        socket.write_all(response.as_bytes()).await?;
        socket.shutdown().await?;
    }
}
