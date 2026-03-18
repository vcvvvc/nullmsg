use base64::{Engine as _, engine::general_purpose};

use aes::Aes128;
use cbc::Encryptor;
use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
type Aes128CbcEnc = Encryptor<Aes128>;

pub fn encrypt(text: &str, key: &str, iv: &str) -> String {
    let encryptor = Aes128CbcEnc::new(
        key.as_bytes().into(),
        iv.as_bytes().into(),
    );
    let pt = text.as_bytes();
    let pt_len = pt.len();
    let mut buf = vec![0u8; pt_len + 16];
    buf[..pt_len].copy_from_slice(pt);
    let encrypted = encryptor.encrypt_padded_mut::<Pkcs7>(&mut buf, pt_len).unwrap();
    general_purpose::STANDARD.encode(encrypted)
}
