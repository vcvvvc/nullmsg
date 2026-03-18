#[path = "src/crypto.rs"]
mod crypto;

#[derive(serde::Deserialize)]
struct Config {
    server_host: String,
    server_key: String,
    aes_key: String,
    aes_iv: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stext = std::fs::read_to_string("./config.toml")?;

    let cfg: Config = toml::from_str(&stext)?;
    // 先用有限 tick 验证异步调度链路可用，避免在功能尚未接入前引入常驻循环干扰后续调试。
    let mut ticker = tokio::time::interval(std::time::Duration::from_millis(500));
    for seq in 1..=2 {
        ticker.tick().await;
        println!("interval tick {}", seq);
    }

    let encoded = crypto::encrypt("这是密码", &cfg.aes_key, &cfg.aes_iv)?;
    println!("encrypt = {}", encoded);

    Ok(())
}
