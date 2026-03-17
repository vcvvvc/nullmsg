struct Config {
    server_host: String,
    server_key: String,
    aes_key: String,
    aes_iv: String,
}

fn parse_config(content: &str) -> Config {
    let mut server_host = String::new();
    let mut server_key  = String::new();
    let mut aes_key     = String::new();
    let mut aes_iv      = String::new();

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim();
            let val = line[pos + 1 ..].trim().trim_matches('"');

            match key {
                "server_host" => server_host = val.to_string(),
                "server_key" => server_key = val.to_string(),
                "aes_key" => aes_key = val.to_string(),
                "aes_iv" => aes_iv = val.to_string(),
                _             => {}  // _ 是通配符，匹配所有其他情况，这里忽略

            }

        }
    }

    Config { server_host, server_key, aes_key, aes_iv }

}


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let stext =  std::fs::read_to_string("./config.toml")?;
    // println!("{}", stext);

    let cfg = parse_config(&stext);
    println!("aes_iv = {}", cfg.aes_iv);


    Ok(())
}
