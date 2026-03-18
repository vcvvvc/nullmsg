use base64::{Engine as _, engine::general_purpose};

use aes::Aes256;
use aes::cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use cbc::Encryptor;
type Aes256CbcEnc = Encryptor<Aes256>;

pub fn encrypt(text: &str, key: &str, iv: &str) -> String {
    let encryptor = Aes256CbcEnc::new(key.as_bytes().into(), iv.as_bytes().into());
    let pt = text.as_bytes();
    let pt_len = pt.len();
    let mut buf = vec![0u8; pt_len + 16];
    buf[..pt_len].copy_from_slice(pt);
    let encrypted = encryptor
        .encrypt_padded_mut::<Pkcs7>(&mut buf, pt_len)
        .unwrap();
    general_purpose::STANDARD.encode(encrypted)
}
