use base64::{Engine as _, engine::general_purpose};
use std::error::Error as StdError;
use std::fmt::{Display, Formatter};

use aes::Aes256;
use aes::cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use cbc::Encryptor;

type Aes256CbcEnc = Encryptor<Aes256>;
// 将协议约束常量化，避免散落魔法数字导致维护时“改一处漏一处”。
const AES_KEY_LEN: usize = 32;
const AES_IV_LEN: usize = 16;
const AES_BLOCK_LEN: usize = 16;

#[derive(Debug)]
pub enum EncryptError {
    InvalidLength {
        field: &'static str,
        expected: usize,
        got: usize,
    },
    CipherInit,
    Encrypt,
}

impl Display for EncryptError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength {
                field,
                expected,
                got,
            } => {
                write!(f, "invalid {field} length: expected {expected}, got {got}")
            }
            Self::CipherInit => write!(f, "failed to initialize aes-cbc cipher"),
            Self::Encrypt => write!(f, "failed to encrypt payload"),
        }
    }
}

impl StdError for EncryptError {}

fn ensure_exact_len(
    field: &'static str,
    bytes: &[u8],
    expected: usize,
) -> Result<(), EncryptError> {
    // 统一长度校验入口，避免 key/iv 分叉为两套几乎相同的分支逻辑。
    if bytes.len() != expected {
        return Err(EncryptError::InvalidLength {
            field,
            expected,
            got: bytes.len(),
        });
    }
    Ok(())
}

pub fn encrypt(text: &str, key: &str, iv: &str) -> Result<String, EncryptError> {
    let key_bytes = key.as_bytes();
    let iv_bytes = iv.as_bytes();
    ensure_exact_len("aes_key", key_bytes, AES_KEY_LEN)?;
    ensure_exact_len("aes_iv", iv_bytes, AES_IV_LEN)?;

    let encryptor =
        Aes256CbcEnc::new_from_slices(key_bytes, iv_bytes).map_err(|_| EncryptError::CipherInit)?;
    let pt = text.as_bytes();
    let pt_len = pt.len();
    // 预留一个块大小空间，让 PKCS7 在原地补齐与加密，避免额外临时分配。
    let mut buf = vec![0u8; pt_len + AES_BLOCK_LEN];
    buf[..pt_len].copy_from_slice(pt);
    let encrypted = encryptor
        .encrypt_padded_mut::<Pkcs7>(&mut buf, pt_len)
        .map_err(|_| EncryptError::Encrypt)?;
    Ok(general_purpose::STANDARD.encode(encrypted))
}
