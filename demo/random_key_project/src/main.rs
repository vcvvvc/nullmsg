use std::io::{Error, ErrorKind};

fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

fn main() -> Result<(), Error> {
    // Why: 与 Python demo 脚本保持同一输出语义，便于替换与核对。
    let mut key = [0_u8; 16];
    let mut iv = [0_u8; 16];
    getrandom::fill(&mut key).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    getrandom::fill(&mut iv).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    let key_hex = to_hex(&key);
    let iv_hex = to_hex(&iv);
    let iv_short = &iv_hex[..16];

    println!("key: {key_hex}");
    println!("iv: {iv_short}");
    Ok(())
}
