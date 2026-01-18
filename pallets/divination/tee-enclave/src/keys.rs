//! # 密钥管理模块
//!
//! 管理 Enclave 内的密钥对，包括：
//! - X25519 密钥交换密钥对
//! - Ed25519 签名密钥对
//! - 密钥密封/解封

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand_core::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

use crate::error::{EnclaveError, EnclaveResult};

/// 密钥管理器
///
/// 管理 Enclave 内所有密钥
pub struct KeyManager {
    /// X25519 密钥交换私钥
    x25519_secret: Option<StaticSecret>,
    /// X25519 公钥
    x25519_public: Option<X25519PublicKey>,
    /// Ed25519 签名私钥
    ed25519_signing: Option<SigningKey>,
    /// Ed25519 验证公钥
    ed25519_verifying: Option<VerifyingKey>,
    /// 是否已初始化
    initialized: bool,
}

impl KeyManager {
    /// 创建新的密钥管理器（未初始化）
    pub fn new() -> Self {
        Self {
            x25519_secret: None,
            x25519_public: None,
            ed25519_signing: None,
            ed25519_verifying: None,
            initialized: false,
        }
    }

    /// 初始化密钥（生成新密钥对）
    pub fn initialize<R: RngCore + CryptoRng>(&mut self, rng: &mut R) -> EnclaveResult<()> {
        // 生成 X25519 密钥对
        let x25519_secret = StaticSecret::random_from_rng(&mut *rng);
        let x25519_public = X25519PublicKey::from(&x25519_secret);

        // 生成 Ed25519 密钥对
        let mut ed25519_seed = [0u8; 32];
        rng.fill_bytes(&mut ed25519_seed);
        let ed25519_signing = SigningKey::from_bytes(&ed25519_seed);
        let ed25519_verifying = ed25519_signing.verifying_key();

        self.x25519_secret = Some(x25519_secret);
        self.x25519_public = Some(x25519_public);
        self.ed25519_signing = Some(ed25519_signing);
        self.ed25519_verifying = Some(ed25519_verifying);
        self.initialized = true;

        Ok(())
    }

    /// 从密封数据恢复密钥
    pub fn restore_from_sealed(&mut self, sealed_data: &SealedKeyData) -> EnclaveResult<()> {
        // 恢复 X25519 密钥
        let x25519_secret = StaticSecret::from(sealed_data.x25519_secret);
        let x25519_public = X25519PublicKey::from(&x25519_secret);

        // 恢复 Ed25519 密钥
        let ed25519_signing = SigningKey::from_bytes(&sealed_data.ed25519_secret);
        let ed25519_verifying = ed25519_signing.verifying_key();

        self.x25519_secret = Some(x25519_secret);
        self.x25519_public = Some(x25519_public);
        self.ed25519_signing = Some(ed25519_signing);
        self.ed25519_verifying = Some(ed25519_verifying);
        self.initialized = true;

        Ok(())
    }

    /// 获取密封数据（用于持久化）
    pub fn get_sealed_data(&self) -> EnclaveResult<SealedKeyData> {
        if !self.initialized {
            return Err(EnclaveError::KeyNotInitialized);
        }

        let x25519_secret = self
            .x25519_secret
            .as_ref()
            .ok_or(EnclaveError::KeyNotInitialized)?;
        let ed25519_signing = self
            .ed25519_signing
            .as_ref()
            .ok_or(EnclaveError::KeyNotInitialized)?;

        Ok(SealedKeyData {
            x25519_secret: x25519_secret.to_bytes(),
            ed25519_secret: ed25519_signing.to_bytes(),
        })
    }

    /// 获取 X25519 公钥（用于密钥交换）
    pub fn get_x25519_public_key(&self) -> EnclaveResult<[u8; 32]> {
        self.x25519_public
            .as_ref()
            .map(|pk| pk.to_bytes())
            .ok_or(EnclaveError::KeyNotInitialized)
    }

    /// 获取 Ed25519 公钥（用于签名验证）
    pub fn get_ed25519_public_key(&self) -> EnclaveResult<[u8; 32]> {
        self.ed25519_verifying
            .as_ref()
            .map(|vk| vk.to_bytes())
            .ok_or(EnclaveError::KeyNotInitialized)
    }

    /// 使用 X25519 执行密钥交换，返回共享密钥
    pub fn derive_shared_secret(&self, peer_public_key: &[u8; 32]) -> EnclaveResult<[u8; 32]> {
        let secret = self
            .x25519_secret
            .as_ref()
            .ok_or(EnclaveError::KeyNotInitialized)?;

        let peer_public = X25519PublicKey::from(*peer_public_key);
        let shared_secret = secret.diffie_hellman(&peer_public);

        Ok(shared_secret.to_bytes())
    }

    /// 使用 Ed25519 签名数据
    pub fn sign(&self, message: &[u8]) -> EnclaveResult<[u8; 64]> {
        let signing_key = self
            .ed25519_signing
            .as_ref()
            .ok_or(EnclaveError::KeyNotInitialized)?;

        let signature = signing_key.sign(message);
        Ok(signature.to_bytes())
    }

    /// 签名计算结果（包含输入输出哈希）
    pub fn sign_computation(
        &self,
        input_hash: &[u8; 32],
        output_hash: &[u8; 32],
        timestamp: u64,
    ) -> EnclaveResult<[u8; 64]> {
        // 构造待签名消息: input_hash || output_hash || timestamp
        let mut message = Vec::with_capacity(72);
        message.extend_from_slice(input_hash);
        message.extend_from_slice(output_hash);
        message.extend_from_slice(&timestamp.to_le_bytes());

        self.sign(&message)
    }

    /// 验证签名
    pub fn verify(&self, message: &[u8], signature: &[u8; 64]) -> EnclaveResult<bool> {
        let verifying_key = self
            .ed25519_verifying
            .as_ref()
            .ok_or(EnclaveError::KeyNotInitialized)?;

        let sig = ed25519_dalek::Signature::from_bytes(signature);
        Ok(verifying_key.verify_strict(message, &sig).is_ok())
    }

    /// 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 密封的密钥数据（用于持久化存储）
///
/// 在实际 SGX 环境中，这些数据会使用 SGX Sealing 加密
#[derive(Clone)]
pub struct SealedKeyData {
    /// X25519 私钥
    pub x25519_secret: [u8; 32],
    /// Ed25519 私钥
    pub ed25519_secret: [u8; 32],
}

impl SealedKeyData {
    /// 计算密钥数据的哈希（用于报告中的 report_data）
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.x25519_secret);
        hasher.update(&self.ed25519_secret);
        hasher.finalize().into()
    }
}

// 清零敏感数据
impl Drop for SealedKeyData {
    fn drop(&mut self) {
        // 用零覆盖密钥材料
        self.x25519_secret.iter_mut().for_each(|b| *b = 0);
        self.ed25519_secret.iter_mut().for_each(|b| *b = 0);
    }
}

/// 从公钥计算其哈希（用于认证报告中的 report_data）
pub fn compute_pubkey_hash(x25519_pubkey: &[u8; 32], ed25519_pubkey: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(x25519_pubkey);
    hasher.update(ed25519_pubkey);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_manager_initialize() {
        let mut rng = rand::thread_rng();
        let mut km = KeyManager::new();

        assert!(!km.is_initialized());

        km.initialize(&mut rng).unwrap();

        assert!(km.is_initialized());
        assert!(km.get_x25519_public_key().is_ok());
        assert!(km.get_ed25519_public_key().is_ok());
    }

    #[test]
    fn test_sign_and_verify() {
        let mut rng = rand::thread_rng();
        let mut km = KeyManager::new();
        km.initialize(&mut rng).unwrap();

        let message = b"Hello, TEE!";
        let signature = km.sign(message).unwrap();

        assert!(km.verify(message, &signature).unwrap());

        // 验证错误消息
        let wrong_message = b"Wrong message";
        assert!(!km.verify(wrong_message, &signature).unwrap());
    }

    #[test]
    fn test_key_exchange() {
        let mut rng = rand::thread_rng();

        // Alice
        let mut alice = KeyManager::new();
        alice.initialize(&mut rng).unwrap();
        let alice_public = alice.get_x25519_public_key().unwrap();

        // Bob
        let mut bob = KeyManager::new();
        bob.initialize(&mut rng).unwrap();
        let bob_public = bob.get_x25519_public_key().unwrap();

        // 密钥交换
        let alice_shared = alice.derive_shared_secret(&bob_public).unwrap();
        let bob_shared = bob.derive_shared_secret(&alice_public).unwrap();

        // 共享密钥应该相同
        assert_eq!(alice_shared, bob_shared);
    }

    #[test]
    fn test_sealed_data() {
        let mut rng = rand::thread_rng();
        let mut km1 = KeyManager::new();
        km1.initialize(&mut rng).unwrap();

        // 获取密封数据
        let sealed = km1.get_sealed_data().unwrap();

        // 从密封数据恢复
        let mut km2 = KeyManager::new();
        km2.restore_from_sealed(&sealed).unwrap();

        // 公钥应该相同
        assert_eq!(
            km1.get_x25519_public_key().unwrap(),
            km2.get_x25519_public_key().unwrap()
        );
        assert_eq!(
            km1.get_ed25519_public_key().unwrap(),
            km2.get_ed25519_public_key().unwrap()
        );
    }

    #[test]
    fn test_sign_computation() {
        let mut rng = rand::thread_rng();
        let mut km = KeyManager::new();
        km.initialize(&mut rng).unwrap();

        let input_hash = [1u8; 32];
        let output_hash = [2u8; 32];
        let timestamp = 1704067200u64;

        let signature = km
            .sign_computation(&input_hash, &output_hash, timestamp)
            .unwrap();

        // 验证签名
        let mut message = Vec::new();
        message.extend_from_slice(&input_hash);
        message.extend_from_slice(&output_hash);
        message.extend_from_slice(&timestamp.to_le_bytes());

        assert!(km.verify(&message, &signature).unwrap());
    }
}
