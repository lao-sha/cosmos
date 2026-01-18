//! # Enclave 运行时
//!
//! 管理 Enclave 的完整生命周期和请求处理流程

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use rand_core::{CryptoRng, RngCore};
use x25519_dalek::PublicKey as X25519PublicKey;

use crate::crypto::{derive_session_key, sha256_hash, CryptoContext};
use crate::divination::DivinationEngine;
use crate::error::{EnclaveError, EnclaveResult};
use crate::keys::KeyManager;
use crate::types::{
    ComputationProof, ComputeInput, ComputeOutput, ComputeRequest, ComputeResponse, ComputeTypeId,
    EncryptedRequest, EncryptedResponse,
};

/// Enclave 运行时状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    /// 未初始化
    Uninitialized,
    /// 已初始化，等待请求
    Ready,
    /// 正在处理请求
    Processing,
    /// 错误状态
    Error,
}

/// Enclave 运行时
///
/// 负责：
/// 1. 密钥管理
/// 2. 请求解密
/// 3. 计算调度
/// 4. 响应加密和签名
pub struct EnclaveRuntime {
    /// 密钥管理器
    key_manager: KeyManager,
    /// 占卜计算引擎
    divination_engine: DivinationEngine,
    /// 运行时状态
    state: RuntimeState,
    /// 已处理请求计数
    request_count: u64,
}

impl EnclaveRuntime {
    /// 创建新的 Enclave 运行时（未初始化）
    pub fn new() -> Self {
        Self {
            key_manager: KeyManager::new(),
            divination_engine: DivinationEngine::new(),
            state: RuntimeState::Uninitialized,
            request_count: 0,
        }
    }

    /// 初始化运行时
    pub fn initialize<R: RngCore + CryptoRng>(&mut self, rng: &mut R) -> EnclaveResult<()> {
        if self.state != RuntimeState::Uninitialized {
            return Err(EnclaveError::InternalError(String::from(
                "Runtime already initialized",
            )));
        }

        // 初始化密钥
        self.key_manager.initialize(rng)?;

        self.state = RuntimeState::Ready;
        Ok(())
    }

    /// 从密封数据恢复运行时
    pub fn restore_from_sealed(
        &mut self,
        sealed_x25519: &[u8; 32],
        sealed_ed25519: &[u8; 32],
    ) -> EnclaveResult<()> {
        if self.state != RuntimeState::Uninitialized {
            return Err(EnclaveError::InternalError(String::from(
                "Runtime already initialized",
            )));
        }

        let sealed_data = crate::keys::SealedKeyData {
            x25519_secret: *sealed_x25519,
            ed25519_secret: *sealed_ed25519,
        };

        self.key_manager.restore_from_sealed(&sealed_data)?;

        self.state = RuntimeState::Ready;
        Ok(())
    }

    /// 获取 Enclave 公钥信息
    pub fn get_public_keys(&self) -> EnclaveResult<EnclavePublicKeys> {
        Ok(EnclavePublicKeys {
            x25519_pubkey: self.key_manager.get_x25519_public_key()?,
            ed25519_pubkey: self.key_manager.get_ed25519_public_key()?,
        })
    }

    /// 获取密封数据（用于持久化）
    pub fn get_sealed_data(&self) -> EnclaveResult<crate::keys::SealedKeyData> {
        self.key_manager.get_sealed_data()
    }

    /// 处理加密请求
    ///
    /// 完整流程:
    /// 1. 从临时公钥计算共享密钥
    /// 2. 派生会话密钥
    /// 3. 解密请求
    /// 4. 执行计算
    /// 5. 加密响应
    /// 6. 签名计算证明
    pub fn process_encrypted_request<R: RngCore + CryptoRng>(
        &mut self,
        encrypted_request: &EncryptedRequest,
        timestamp: u64,
        rng: &mut R,
    ) -> EnclaveResult<ProcessedResult> {
        if self.state != RuntimeState::Ready {
            return Err(EnclaveError::RuntimeNotInitialized);
        }

        self.state = RuntimeState::Processing;

        let result = self.do_process_request(encrypted_request, timestamp, rng);

        self.state = RuntimeState::Ready;
        self.request_count += 1;

        result
    }

    /// 内部请求处理实现
    fn do_process_request<R: RngCore + CryptoRng>(
        &self,
        encrypted_request: &EncryptedRequest,
        timestamp: u64,
        rng: &mut R,
    ) -> EnclaveResult<ProcessedResult> {
        // 1. 计算共享密钥
        let shared_secret = self
            .key_manager
            .derive_shared_secret(&encrypted_request.ephemeral_pubkey)?;

        // 2. 创建加密上下文
        let ctx = CryptoContext::from_shared_secret(&shared_secret, b"stardust-enclave-v1")?;

        // 3. 解密请求
        let plaintext = ctx.decrypt_request(encrypted_request)?;

        // 4. 反序列化请求
        let request: ComputeRequest = self.deserialize_request(&plaintext)?;

        // 5. 计算输入哈希
        let input_hash = sha256_hash(&plaintext);

        // 6. 执行计算
        let output = self.execute_computation(&request)?;

        // 7. 序列化输出
        let output_bytes = self.serialize_output(&output)?;

        // 8. 计算输出哈希
        let output_hash = sha256_hash(&output_bytes);

        // 9. 签名计算证明
        let signature = self
            .key_manager
            .sign_computation(&input_hash, &output_hash, timestamp)?;

        // 10. 加密响应
        let encrypted_response =
            ctx.encrypt_response(&output_bytes, output_hash, signature, rng)?;

        // 11. 构造计算证明
        let proof = ComputationProof {
            input_hash,
            output_hash,
            enclave_signature: signature,
            timestamp,
        };

        // 12. 构造完整响应
        let response = ComputeResponse {
            request_id: request.request_id,
            compute_type: request.compute_type,
            output,
            completed_at: timestamp,
        };

        Ok(ProcessedResult {
            response,
            encrypted_response,
            proof,
            session_key: derive_session_key(&shared_secret, b"stardust-enclave-v1")?,
        })
    }

    /// 反序列化请求
    fn deserialize_request(&self, data: &[u8]) -> EnclaveResult<ComputeRequest> {
        #[cfg(feature = "std")]
        {
            serde_json::from_slice(data).map_err(|_| EnclaveError::DeserializationFailed)
        }

        #[cfg(not(feature = "std"))]
        {
            // no_std 环境下使用简化的反序列化
            // 实际实现中可以使用 postcard 或其他 no_std 兼容的序列化库
            Err(EnclaveError::DeserializationFailed)
        }
    }

    /// 序列化输出
    fn serialize_output(&self, output: &ComputeOutput) -> EnclaveResult<Vec<u8>> {
        #[cfg(feature = "std")]
        {
            serde_json::to_vec(output).map_err(|_| EnclaveError::SerializationFailed)
        }

        #[cfg(not(feature = "std"))]
        {
            // no_std 环境下使用简化的序列化
            Err(EnclaveError::SerializationFailed)
        }
    }

    /// 执行计算
    fn execute_computation(&self, request: &ComputeRequest) -> EnclaveResult<ComputeOutput> {
        // 验证输入与计算类型匹配
        self.validate_input_type(&request.compute_type, &request.input)?;

        // 调用占卜引擎执行计算
        self.divination_engine.compute(&request.input)
    }

    /// 验证输入类型与计算类型匹配
    fn validate_input_type(
        &self,
        compute_type: &ComputeTypeId,
        input: &ComputeInput,
    ) -> EnclaveResult<()> {
        let valid = match (compute_type, input) {
            (ComputeTypeId::BaZi, ComputeInput::BaZi(_)) => true,
            (ComputeTypeId::MeiHua, ComputeInput::MeiHua(_)) => true,
            (ComputeTypeId::QiMen, ComputeInput::QiMen(_)) => true,
            (ComputeTypeId::LiuYao, ComputeInput::LiuYao(_)) => true,
            (ComputeTypeId::ZiWei, ComputeInput::ZiWei(_)) => true,
            (ComputeTypeId::Tarot, ComputeInput::Tarot(_)) => true,
            (ComputeTypeId::DaLiuRen, ComputeInput::DaLiuRen(_)) => true,
            (ComputeTypeId::XiaoLiuRen, ComputeInput::XiaoLiuRen(_)) => true,
            _ => false,
        };

        if valid {
            Ok(())
        } else {
            Err(EnclaveError::InvalidInputData)
        }
    }

    /// 获取运行时状态
    pub fn state(&self) -> RuntimeState {
        self.state
    }

    /// 获取已处理请求数
    pub fn request_count(&self) -> u64 {
        self.request_count
    }

    /// 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.state != RuntimeState::Uninitialized
    }

    /// 直接处理请求（用于测试，跳过加密）
    #[cfg(feature = "std")]
    pub fn process_request_direct(
        &self,
        request: &ComputeRequest,
        timestamp: u64,
    ) -> EnclaveResult<(ComputeResponse, ComputationProof)> {
        // 序列化请求计算输入哈希
        let request_bytes =
            serde_json::to_vec(request).map_err(|_| EnclaveError::SerializationFailed)?;
        let input_hash = sha256_hash(&request_bytes);

        // 执行计算
        let output = self.execute_computation(request)?;

        // 序列化输出计算哈希
        let output_bytes = self.serialize_output(&output)?;
        let output_hash = sha256_hash(&output_bytes);

        // 签名
        let signature = self
            .key_manager
            .sign_computation(&input_hash, &output_hash, timestamp)?;

        let response = ComputeResponse {
            request_id: request.request_id,
            compute_type: request.compute_type,
            output,
            completed_at: timestamp,
        };

        let proof = ComputationProof {
            input_hash,
            output_hash,
            enclave_signature: signature,
            timestamp,
        };

        Ok((response, proof))
    }

    /// 验证计算证明
    pub fn verify_proof(
        &self,
        proof: &ComputationProof,
        enclave_pubkey: &[u8; 32],
    ) -> EnclaveResult<bool> {
        // 重构待验证消息
        let mut message = Vec::with_capacity(72);
        message.extend_from_slice(&proof.input_hash);
        message.extend_from_slice(&proof.output_hash);
        message.extend_from_slice(&proof.timestamp.to_le_bytes());

        // 验证签名
        use ed25519_dalek::{Signature, VerifyingKey};

        let verifying_key = VerifyingKey::from_bytes(enclave_pubkey)
            .map_err(|_| EnclaveError::InvalidPublicKey)?;

        let signature = Signature::from_bytes(&proof.enclave_signature);

        Ok(verifying_key.verify_strict(&message, &signature).is_ok())
    }
}

impl Default for EnclaveRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Enclave 公钥信息
#[derive(Debug, Clone)]
pub struct EnclavePublicKeys {
    /// X25519 密钥交换公钥
    pub x25519_pubkey: [u8; 32],
    /// Ed25519 签名验证公钥
    pub ed25519_pubkey: [u8; 32],
}

/// 处理结果
#[derive(Debug)]
pub struct ProcessedResult {
    /// 明文响应（用于日志等）
    pub response: ComputeResponse,
    /// 加密响应（发送给客户端）
    pub encrypted_response: EncryptedResponse,
    /// 计算证明（提交到链上）
    pub proof: ComputationProof,
    /// 会话密钥（客户端解密用）
    pub session_key: [u8; 32],
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_runtime_lifecycle() {
        let mut rng = rand::thread_rng();
        let mut runtime = EnclaveRuntime::new();

        assert_eq!(runtime.state(), RuntimeState::Uninitialized);
        assert!(!runtime.is_initialized());

        runtime.initialize(&mut rng).unwrap();

        assert_eq!(runtime.state(), RuntimeState::Ready);
        assert!(runtime.is_initialized());
    }

    #[test]
    fn test_get_public_keys() {
        let mut rng = rand::thread_rng();
        let mut runtime = EnclaveRuntime::new();
        runtime.initialize(&mut rng).unwrap();

        let keys = runtime.get_public_keys().unwrap();
        assert_eq!(keys.x25519_pubkey.len(), 32);
        assert_eq!(keys.ed25519_pubkey.len(), 32);
    }

    #[test]
    fn test_sealed_data_roundtrip() {
        let mut rng = rand::thread_rng();

        // 创建并初始化第一个运行时
        let mut runtime1 = EnclaveRuntime::new();
        runtime1.initialize(&mut rng).unwrap();
        let keys1 = runtime1.get_public_keys().unwrap();

        // 获取密封数据
        let sealed = runtime1.get_sealed_data().unwrap();

        // 使用密封数据恢复第二个运行时
        let mut runtime2 = EnclaveRuntime::new();
        runtime2
            .restore_from_sealed(&sealed.x25519_secret, &sealed.ed25519_secret)
            .unwrap();
        let keys2 = runtime2.get_public_keys().unwrap();

        // 公钥应该相同
        assert_eq!(keys1.x25519_pubkey, keys2.x25519_pubkey);
        assert_eq!(keys1.ed25519_pubkey, keys2.ed25519_pubkey);
    }

    #[test]
    fn test_process_request_direct() {
        let mut rng = rand::thread_rng();
        let mut runtime = EnclaveRuntime::new();
        runtime.initialize(&mut rng).unwrap();

        let request = ComputeRequest {
            request_id: 1,
            compute_type: ComputeTypeId::XiaoLiuRen,
            input: ComputeInput::XiaoLiuRen(XiaoLiuRenInput {
                month: 5,
                day: 15,
                hour: 10,
            }),
            timestamp: 1704067200,
        };

        let (response, proof) = runtime
            .process_request_direct(&request, 1704067200)
            .unwrap();

        assert_eq!(response.request_id, 1);
        assert_eq!(response.compute_type, ComputeTypeId::XiaoLiuRen);

        // 验证证明签名
        let keys = runtime.get_public_keys().unwrap();
        assert!(runtime.verify_proof(&proof, &keys.ed25519_pubkey).unwrap());
    }

    #[test]
    fn test_validate_input_type() {
        let mut rng = rand::thread_rng();
        let mut runtime = EnclaveRuntime::new();
        runtime.initialize(&mut rng).unwrap();

        // 正确的类型匹配
        assert!(runtime
            .validate_input_type(
                &ComputeTypeId::BaZi,
                &ComputeInput::BaZi(BaZiInput {
                    year: 1990,
                    month: 1,
                    day: 1,
                    hour: 12,
                    gender: Gender::Male,
                    longitude: None,
                })
            )
            .is_ok());

        // 错误的类型匹配
        assert!(runtime
            .validate_input_type(
                &ComputeTypeId::BaZi,
                &ComputeInput::XiaoLiuRen(XiaoLiuRenInput {
                    month: 5,
                    day: 15,
                    hour: 10,
                })
            )
            .is_err());
    }

    #[test]
    fn test_verify_proof() {
        let mut rng = rand::thread_rng();
        let mut runtime = EnclaveRuntime::new();
        runtime.initialize(&mut rng).unwrap();

        let input_hash = [1u8; 32];
        let output_hash = [2u8; 32];
        let timestamp = 1704067200u64;

        // 使用密钥管理器签名
        let sealed = runtime.get_sealed_data().unwrap();
        let mut key_manager = KeyManager::new();
        key_manager.restore_from_sealed(&crate::keys::SealedKeyData {
            x25519_secret: sealed.x25519_secret,
            ed25519_secret: sealed.ed25519_secret,
        }).unwrap();

        let signature = key_manager
            .sign_computation(&input_hash, &output_hash, timestamp)
            .unwrap();

        let proof = ComputationProof {
            input_hash,
            output_hash,
            enclave_signature: signature,
            timestamp,
        };

        let keys = runtime.get_public_keys().unwrap();

        // 验证应该成功
        assert!(runtime.verify_proof(&proof, &keys.ed25519_pubkey).unwrap());

        // 篡改的证明应该失败
        let mut tampered_proof = proof.clone();
        tampered_proof.input_hash[0] ^= 0xFF;
        assert!(!runtime
            .verify_proof(&tampered_proof, &keys.ed25519_pubkey)
            .unwrap());
    }
}
