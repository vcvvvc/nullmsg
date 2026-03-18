use std::time::Duration;
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
    // println!("{}", stext);

    let cfg: Config = toml::from_str(&stext)?;

    println!("server_host = {}", cfg.server_host);
    println!("server_key  = {}", cfg.server_key);
    println!("aes_key     = {}", cfg.aes_key);
    println!("aes_iv      = {}", cfg.aes_iv);

    let encoded = crypto::encrypt("这是密码", &cfg.aes_key, &cfg.aes_iv);
    println!("encrypt = {}", encoded);

    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("tokio 异步运行正常");

    Ok(())
}
