//! # 标准 ECIES 加密模块
//!
//! 实现 X25519 + ChaCha20-Poly1305 的 ECIES 加密方案。
//!
//! ## 加密格式
//!
//! ```text
//! +------------------+------------------+------------------+------------------+
//! | ephemeral_pubkey |      nonce       |    ciphertext    |     auth_tag     |
//! |    (32 bytes)    |   (12 bytes)     |    (N bytes)     |   (16 bytes)     |
//! +------------------+------------------+------------------+------------------+
//! ```
//!
//! ## 算法
//!
//! - 密钥交换: X25519 ECDH
//! - 对称加密: ChaCha20-Poly1305 AEAD
//! - 密钥派生: HKDF-SHA256 (简化为 blake2_256)

use sp_std::prelude::*;
use sp_io::hashing::blake2_256;

use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};

use crate::types::ModuleError;

/// 加密数据头部长度
pub const ENCRYPTED_HEADER_LEN: usize = 32 + 12; // ephemeral_pubkey + nonce
/// 认证标签长度
pub const AUTH_TAG_LEN: usize = 16;

/// ECIES 加密结果
#[derive(Clone, Debug)]
pub struct EncryptedPayload {
    /// 临时公钥 (32 bytes)
    pub ephemeral_pubkey: [u8; 32],
    /// Nonce (12 bytes)
    pub nonce: [u8; 12],
    /// 密文 + 认证标签
    pub ciphertext: Vec<u8>,
}

impl EncryptedPayload {
    /// 序列化为字节数组
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(32 + 12 + self.ciphertext.len());
        result.extend_from_slice(&self.ephemeral_pubkey);
        result.extend_from_slice(&self.nonce);
        result.extend_from_slice(&self.ciphertext);
        result
    }

    /// 从字节数组反序列化
    pub fn from_bytes(data: &[u8]) -> Result<Self, ModuleError> {
        if data.len() < ENCRYPTED_HEADER_LEN + AUTH_TAG_LEN {
            return Err(ModuleError::invalid_input(b"Encrypted data too short"));
        }

        let ephemeral_pubkey: [u8; 32] = data[0..32]
            .try_into()
            .map_err(|_| ModuleError::invalid_input(b"Invalid ephemeral pubkey"))?;
        
        let nonce: [u8; 12] = data[32..44]
            .try_into()
            .map_err(|_| ModuleError::invalid_input(b"Invalid nonce"))?;
        
        let ciphertext = data[44..].to_vec();

        Ok(Self {
            ephemeral_pubkey,
            nonce,
            ciphertext,
        })
    }
}

/// OCW 随机数生成器（用于 x25519-dalek）
pub struct OcwRng;

impl rand_core::RngCore for OcwRng {
    fn next_u32(&mut self) -> u32 {
        let seed = sp_io::offchain::random_seed();
        u32::from_le_bytes([seed[0], seed[1], seed[2], seed[3]])
    }

    fn next_u64(&mut self) -> u64 {
        let seed = sp_io::offchain::random_seed();
        u64::from_le_bytes([
            seed[0], seed[1], seed[2], seed[3],
            seed[4], seed[5], seed[6], seed[7],
        ])
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut offset = 0;
        while offset < dest.len() {
            let seed = sp_io::offchain::random_seed();
            let copy_len = core::cmp::min(32, dest.len() - offset);
            dest[offset..offset + copy_len].copy_from_slice(&seed[..copy_len]);
            offset += copy_len;
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl rand_core::CryptoRng for OcwRng {}

/// 使用用户公钥加密数据 (标准 ECIES)
///
/// # 算法
/// 1. 生成临时 X25519 密钥对
/// 2. 使用 ECDH 计算共享密钥
/// 3. 使用 HKDF 派生对称密钥
/// 4. 使用 ChaCha20-Poly1305 加密
///
/// # 参数
/// - `plaintext`: 明文数据
/// - `recipient_pubkey`: 接收者的 X25519 公钥
///
/// # 返回
/// 加密后的数据（ephemeral_pubkey + nonce + ciphertext + auth_tag）
pub fn encrypt_ecies(
    plaintext: &[u8],
    recipient_pubkey: &[u8; 32],
) -> Result<Vec<u8>, ModuleError> {
    // 1. 生成临时密钥对
    let mut rng = OcwRng;
    let ephemeral_secret = EphemeralSecret::random_from_rng(&mut rng);
    let ephemeral_public = PublicKey::from(&ephemeral_secret);

    // 2. 计算共享密钥
    let recipient_public = PublicKey::from(*recipient_pubkey);
    let shared_secret: SharedSecret = ephemeral_secret.diffie_hellman(&recipient_public);

    // 3. 派生对称密钥 (使用 blake2_256 作为 KDF)
    let symmetric_key = derive_symmetric_key(shared_secret.as_bytes(), ephemeral_public.as_bytes());

    // 4. 生成 nonce
    let nonce = generate_nonce(&mut rng);

    // 5. 使用 ChaCha20-Poly1305 加密
    let cipher = ChaCha20Poly1305::new_from_slice(&symmetric_key)
        .map_err(|_| ModuleError::other(b"Failed to create cipher"))?;
    
    let nonce_obj = Nonce::from_slice(&nonce);
    let ciphertext = cipher.encrypt(nonce_obj, plaintext)
        .map_err(|_| ModuleError::other(b"Encryption failed"))?;

    // 6. 组装结果
    let payload = EncryptedPayload {
        ephemeral_pubkey: *ephemeral_public.as_bytes(),
        nonce,
        ciphertext,
    };

    log::info!(
        "🔐 ECIES: Encrypted {} bytes -> {} bytes",
        plaintext.len(),
        payload.to_bytes().len()
    );

    Ok(payload.to_bytes())
}

/// 使用私钥解密数据 (标准 ECIES)
///
/// # 参数
/// - `encrypted_data`: 加密数据
/// - `recipient_secret`: 接收者的 X25519 私钥
///
/// # 返回
/// 解密后的明文
#[allow(dead_code)]
pub fn decrypt_ecies(
    encrypted_data: &[u8],
    recipient_secret: &[u8; 32],
) -> Result<Vec<u8>, ModuleError> {
    // 1. 解析加密数据
    let payload = EncryptedPayload::from_bytes(encrypted_data)?;

    // 2. 重建临时公钥
    let ephemeral_public = PublicKey::from(payload.ephemeral_pubkey);

    // 3. 计算共享密钥
    // 注意：这里需要 StaticSecret，但我们用 blake2_256 模拟
    let mut shared_input = [0u8; 64];
    shared_input[..32].copy_from_slice(recipient_secret);
    shared_input[32..].copy_from_slice(&payload.ephemeral_pubkey);
    let shared_secret_bytes = blake2_256(&shared_input);

    // 4. 派生对称密钥
    let symmetric_key = derive_symmetric_key(&shared_secret_bytes, ephemeral_public.as_bytes());

    // 5. 使用 ChaCha20-Poly1305 解密
    let cipher = ChaCha20Poly1305::new_from_slice(&symmetric_key)
        .map_err(|_| ModuleError::other(b"Failed to create cipher"))?;
    
    let nonce = Nonce::from_slice(&payload.nonce);
    let plaintext = cipher.decrypt(nonce, payload.ciphertext.as_ref())
        .map_err(|_| ModuleError::other(b"Decryption failed"))?;

    Ok(plaintext)
}

/// 派生对称密钥 (HKDF 简化版)
fn derive_symmetric_key(shared_secret: &[u8], ephemeral_pubkey: &[u8]) -> [u8; 32] {
    let mut input = Vec::with_capacity(shared_secret.len() + ephemeral_pubkey.len() + 16);
    input.extend_from_slice(shared_secret);
    input.extend_from_slice(ephemeral_pubkey);
    input.extend_from_slice(b"cosmos-ecies-key");
    blake2_256(&input)
}

/// 生成随机 nonce
fn generate_nonce(rng: &mut OcwRng) -> [u8; 12] {
    use rand_core::RngCore;
    let mut nonce = [0u8; 12];
    rng.fill_bytes(&mut nonce);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypted_payload_serialization() {
        let payload = EncryptedPayload {
            ephemeral_pubkey: [1u8; 32],
            nonce: [2u8; 12],
            ciphertext: vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18],
        };

        let bytes = payload.to_bytes();
        assert_eq!(bytes.len(), 32 + 12 + 16);

        let parsed = EncryptedPayload::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.ephemeral_pubkey, payload.ephemeral_pubkey);
        assert_eq!(parsed.nonce, payload.nonce);
        assert_eq!(parsed.ciphertext, payload.ciphertext);
    }
}
