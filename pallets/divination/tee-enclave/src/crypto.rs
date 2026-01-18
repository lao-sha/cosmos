//! # 加密模块
//!
//! 提供 Enclave 内的加密功能：
//! - AES-256-GCM 加密/解密
//! - HKDF 密钥派生
//! - 安全随机数生成

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use hkdf::Hkdf;
use rand_core::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};

use crate::error::{EnclaveError, EnclaveResult};
use crate::types::{EncryptedRequest, EncryptedResponse};

/// AES-256-GCM Nonce 大小
pub const NONCE_SIZE: usize = 12;

/// AES-256-GCM 认证标签大小
pub const AUTH_TAG_SIZE: usize = 16;

/// AES-256 密钥大小
pub const KEY_SIZE: usize = 32;

/// 加密上下文
///
/// 管理与特定会话相关的加密操作
pub struct CryptoContext {
    /// 会话密钥 (从 ECDH 共享密钥派生)
    session_key: [u8; KEY_SIZE],
}

impl CryptoContext {
    /// 从共享密钥创建加密上下文
    ///
    /// 使用 HKDF 从 X25519 共享密钥派生会话密钥
    pub fn from_shared_secret(shared_secret: &[u8; 32], info: &[u8]) -> EnclaveResult<Self> {
        let session_key = derive_session_key(shared_secret, info)?;
        Ok(Self { session_key })
    }

    /// 加密数据
    pub fn encrypt<R: RngCore + CryptoRng>(
        &self,
        plaintext: &[u8],
        rng: &mut R,
    ) -> EnclaveResult<EncryptedData> {
        // 生成随机 nonce
        let mut nonce = [0u8; NONCE_SIZE];
        rng.fill_bytes(&mut nonce);

        // 加密
        let (ciphertext, auth_tag) = aes_gcm_encrypt(&self.session_key, &nonce, plaintext, &[])?;

        Ok(EncryptedData {
            ciphertext,
            nonce,
            auth_tag,
        })
    }

    /// 解密数据
    pub fn decrypt(&self, encrypted: &EncryptedData) -> EnclaveResult<Vec<u8>> {
        aes_gcm_decrypt(
            &self.session_key,
            &encrypted.nonce,
            &encrypted.ciphertext,
            &encrypted.auth_tag,
            &[],
        )
    }

    /// 加密并生成响应
    pub fn encrypt_response<R: RngCore + CryptoRng>(
        &self,
        plaintext: &[u8],
        output_hash: [u8; 32],
        enclave_signature: [u8; 64],
        rng: &mut R,
    ) -> EnclaveResult<EncryptedResponse> {
        let encrypted = self.encrypt(plaintext, rng)?;

        Ok(EncryptedResponse {
            ciphertext: encrypted.ciphertext,
            nonce: encrypted.nonce,
            auth_tag: encrypted.auth_tag,
            output_hash,
            enclave_signature,
        })
    }

    /// 从加密请求解密
    pub fn decrypt_request(&self, request: &EncryptedRequest) -> EnclaveResult<Vec<u8>> {
        aes_gcm_decrypt(
            &self.session_key,
            &request.nonce,
            &request.ciphertext,
            &request.auth_tag,
            &[],
        )
    }
}

impl Drop for CryptoContext {
    fn drop(&mut self) {
        // 清零会话密钥
        self.session_key.iter_mut().for_each(|b| *b = 0);
    }
}

/// 加密数据结构
#[derive(Debug, Clone)]
pub struct EncryptedData {
    /// 密文
    pub ciphertext: Vec<u8>,
    /// Nonce (12 字节)
    pub nonce: [u8; NONCE_SIZE],
    /// 认证标签 (16 字节)
    pub auth_tag: [u8; AUTH_TAG_SIZE],
}

/// 使用 HKDF 从共享密钥派生会话密钥
///
/// # 参数
/// - `shared_secret`: X25519 ECDH 共享密钥
/// - `info`: 上下文信息 (用于密钥分离)
///
/// # 返回
/// 派生的 256 位会话密钥
pub fn derive_session_key(shared_secret: &[u8; 32], info: &[u8]) -> EnclaveResult<[u8; KEY_SIZE]> {
    // 使用空 salt (HKDF 会使用默认的零字节 salt)
    let hkdf = Hkdf::<Sha256>::new(None, shared_secret);

    let mut session_key = [0u8; KEY_SIZE];
    hkdf.expand(info, &mut session_key)
        .map_err(|_| EnclaveError::KeyDerivationFailed)?;

    Ok(session_key)
}

/// AES-256-GCM 加密
///
/// # 参数
/// - `key`: 256 位密钥
/// - `nonce`: 96 位 nonce
/// - `plaintext`: 明文
/// - `aad`: 附加认证数据 (可选)
///
/// # 返回
/// (密文, 认证标签)
pub fn aes_gcm_encrypt(
    key: &[u8; KEY_SIZE],
    nonce: &[u8; NONCE_SIZE],
    plaintext: &[u8],
    aad: &[u8],
) -> EnclaveResult<(Vec<u8>, [u8; AUTH_TAG_SIZE])> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| EnclaveError::EncryptionFailed)?;

    let nonce = Nonce::from_slice(nonce);

    // aes-gcm crate 将认证标签附加到密文末尾
    let ciphertext_with_tag = if aad.is_empty() {
        cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| EnclaveError::EncryptionFailed)?
    } else {
        cipher
            .encrypt(
                nonce,
                aes_gcm::aead::Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .map_err(|_| EnclaveError::EncryptionFailed)?
    };

    // 分离密文和认证标签
    let tag_start = ciphertext_with_tag.len() - AUTH_TAG_SIZE;
    let ciphertext = ciphertext_with_tag[..tag_start].to_vec();
    let mut auth_tag = [0u8; AUTH_TAG_SIZE];
    auth_tag.copy_from_slice(&ciphertext_with_tag[tag_start..]);

    Ok((ciphertext, auth_tag))
}

/// AES-256-GCM 解密
///
/// # 参数
/// - `key`: 256 位密钥
/// - `nonce`: 96 位 nonce
/// - `ciphertext`: 密文
/// - `auth_tag`: 认证标签
/// - `aad`: 附加认证数据 (可选)
///
/// # 返回
/// 明文
pub fn aes_gcm_decrypt(
    key: &[u8; KEY_SIZE],
    nonce: &[u8; NONCE_SIZE],
    ciphertext: &[u8],
    auth_tag: &[u8; AUTH_TAG_SIZE],
    aad: &[u8],
) -> EnclaveResult<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| EnclaveError::DecryptionFailed)?;

    let nonce = Nonce::from_slice(nonce);

    // 重新组合密文和认证标签
    let mut ciphertext_with_tag = ciphertext.to_vec();
    ciphertext_with_tag.extend_from_slice(auth_tag);

    let plaintext = if aad.is_empty() {
        cipher
            .decrypt(nonce, ciphertext_with_tag.as_slice())
            .map_err(|_| EnclaveError::AuthTagVerificationFailed)?
    } else {
        cipher
            .decrypt(
                nonce,
                aes_gcm::aead::Payload {
                    msg: &ciphertext_with_tag,
                    aad,
                },
            )
            .map_err(|_| EnclaveError::AuthTagVerificationFailed)?
    };

    Ok(plaintext)
}

/// 计算 SHA-256 哈希
pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// 计算多个数据块的哈希
pub fn sha256_hash_multi(data_blocks: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for block in data_blocks {
        hasher.update(block);
    }
    hasher.finalize().into()
}

/// 用于客户端的加密助手
///
/// 客户端使用此模块加密发送给 Enclave 的请求
pub mod client {
    use super::*;
    use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

    /// 为发送给 Enclave 的请求生成加密数据
    ///
    /// # 参数
    /// - `enclave_pubkey`: Enclave 的 X25519 公钥
    /// - `plaintext`: 要加密的数据
    /// - `rng`: 随机数生成器
    ///
    /// # 返回
    /// (加密数据, 临时公钥)
    pub fn encrypt_for_enclave<R: RngCore + CryptoRng>(
        enclave_pubkey: &[u8; 32],
        plaintext: &[u8],
        rng: &mut R,
    ) -> EnclaveResult<EncryptedRequest> {
        // 生成临时密钥对
        let ephemeral_secret = StaticSecret::random_from_rng(&mut *rng);
        let ephemeral_public = X25519PublicKey::from(&ephemeral_secret);

        // 计算共享密钥
        let enclave_public = X25519PublicKey::from(*enclave_pubkey);
        let shared_secret = ephemeral_secret.diffie_hellman(&enclave_public);

        // 派生会话密钥
        let session_key = derive_session_key(shared_secret.as_bytes(), b"stardust-enclave-v1")?;

        // 生成随机 nonce
        let mut nonce = [0u8; NONCE_SIZE];
        rng.fill_bytes(&mut nonce);

        // 加密
        let (ciphertext, auth_tag) = aes_gcm_encrypt(&session_key, &nonce, plaintext, &[])?;

        Ok(EncryptedRequest {
            ciphertext,
            ephemeral_pubkey: ephemeral_public.to_bytes(),
            nonce,
            auth_tag,
        })
    }

    /// 解密来自 Enclave 的响应
    ///
    /// # 参数
    /// - `response`: Enclave 返回的加密响应
    /// - `session_key`: 与 Enclave 协商的会话密钥
    pub fn decrypt_response(
        response: &EncryptedResponse,
        session_key: &[u8; KEY_SIZE],
    ) -> EnclaveResult<Vec<u8>> {
        aes_gcm_decrypt(
            session_key,
            &response.nonce,
            &response.ciphertext,
            &response.auth_tag,
            &[],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_encrypt_decrypt() {
        let key = [0u8; 32];
        let nonce = [0u8; 12];
        let plaintext = b"Hello, TEE!";

        let (ciphertext, auth_tag) = aes_gcm_encrypt(&key, &nonce, plaintext, &[]).unwrap();

        let decrypted = aes_gcm_decrypt(&key, &nonce, &ciphertext, &auth_tag, &[]).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes_gcm_with_aad() {
        let key = [1u8; 32];
        let nonce = [2u8; 12];
        let plaintext = b"Secret data";
        let aad = b"Associated data";

        let (ciphertext, auth_tag) = aes_gcm_encrypt(&key, &nonce, plaintext, aad).unwrap();

        // 正确的 AAD
        let decrypted = aes_gcm_decrypt(&key, &nonce, &ciphertext, &auth_tag, aad).unwrap();
        assert_eq!(decrypted, plaintext);

        // 错误的 AAD 应该失败
        let result = aes_gcm_decrypt(&key, &nonce, &ciphertext, &auth_tag, b"wrong aad");
        assert!(result.is_err());
    }

    #[test]
    fn test_derive_session_key() {
        let shared_secret = [42u8; 32];
        let info1 = b"context-1";
        let info2 = b"context-2";

        let key1 = derive_session_key(&shared_secret, info1).unwrap();
        let key2 = derive_session_key(&shared_secret, info2).unwrap();

        // 不同的 info 应该产生不同的密钥
        assert_ne!(key1, key2);

        // 相同的输入应该产生相同的密钥
        let key1_again = derive_session_key(&shared_secret, info1).unwrap();
        assert_eq!(key1, key1_again);
    }

    #[test]
    fn test_sha256_hash() {
        let data = b"test data";
        let hash1 = sha256_hash(data);
        let hash2 = sha256_hash(data);

        // 相同数据应该产生相同哈希
        assert_eq!(hash1, hash2);

        // 不同数据应该产生不同哈希
        let hash3 = sha256_hash(b"different data");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_sha256_hash_multi() {
        let data1 = b"hello";
        let data2 = b"world";

        // 多块哈希
        let hash_multi = sha256_hash_multi(&[data1, data2]);

        // 应该等于连接后的哈希
        let mut combined = Vec::new();
        combined.extend_from_slice(data1);
        combined.extend_from_slice(data2);
        let hash_combined = sha256_hash(&combined);

        assert_eq!(hash_multi, hash_combined);
    }

    #[test]
    fn test_crypto_context() {
        let mut rng = rand::thread_rng();

        // 模拟共享密钥
        let shared_secret = [99u8; 32];

        let ctx = CryptoContext::from_shared_secret(&shared_secret, b"test").unwrap();

        let plaintext = b"Confidential divination data";
        let encrypted = ctx.encrypt(plaintext, &mut rng).unwrap();
        let decrypted = ctx.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_client_encrypt_for_enclave() {
        use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

        let mut rng = rand::thread_rng();

        // 模拟 Enclave 密钥对
        let enclave_secret = StaticSecret::random_from_rng(&mut rng);
        let enclave_public = X25519PublicKey::from(&enclave_secret);

        // 客户端加密
        let plaintext = b"Request from client";
        let encrypted =
            client::encrypt_for_enclave(&enclave_public.to_bytes(), plaintext, &mut rng).unwrap();

        // Enclave 解密
        let ephemeral_public = X25519PublicKey::from(encrypted.ephemeral_pubkey);
        let shared_secret = enclave_secret.diffie_hellman(&ephemeral_public);
        let session_key =
            derive_session_key(shared_secret.as_bytes(), b"stardust-enclave-v1").unwrap();

        let decrypted = aes_gcm_decrypt(
            &session_key,
            &encrypted.nonce,
            &encrypted.ciphertext,
            &encrypted.auth_tag,
            &[],
        )
        .unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_tampered_ciphertext() {
        let key = [0u8; 32];
        let nonce = [0u8; 12];
        let plaintext = b"Important data";

        let (mut ciphertext, auth_tag) = aes_gcm_encrypt(&key, &nonce, plaintext, &[]).unwrap();

        // 篡改密文
        if !ciphertext.is_empty() {
            ciphertext[0] ^= 0xFF;
        }

        // 解密应该失败
        let result = aes_gcm_decrypt(&key, &nonce, &ciphertext, &auth_tag, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_auth_tag() {
        let key = [0u8; 32];
        let nonce = [0u8; 12];
        let plaintext = b"Important data";

        let (ciphertext, mut auth_tag) = aes_gcm_encrypt(&key, &nonce, plaintext, &[]).unwrap();

        // 篡改认证标签
        auth_tag[0] ^= 0xFF;

        // 解密应该失败
        let result = aes_gcm_decrypt(&key, &nonce, &ciphertext, &auth_tag, &[]);
        assert!(result.is_err());
    }
}
