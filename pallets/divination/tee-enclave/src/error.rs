//! # 错误类型定义

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Enclave 错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnclaveError {
    // ==================== 密钥错误 ====================
    /// 密钥未初始化
    KeyNotInitialized,
    /// 密钥生成失败
    KeyGenerationFailed,
    /// 无效的公钥
    InvalidPublicKey,
    /// 密钥派生失败
    KeyDerivationFailed,
    /// 密封失败
    SealingFailed,
    /// 解封失败
    UnsealingFailed,

    // ==================== 加密错误 ====================
    /// 加密失败
    EncryptionFailed,
    /// 解密失败
    DecryptionFailed,
    /// 无效的 Nonce
    InvalidNonce,
    /// 认证标签验证失败
    AuthTagVerificationFailed,

    // ==================== 签名错误 ====================
    /// 签名失败
    SigningFailed,
    /// 签名验证失败
    SignatureVerificationFailed,
    /// 无效的签名
    InvalidSignature,

    // ==================== 计算错误 ====================
    /// 无效的计算类型
    InvalidComputeType,
    /// 无效的输入数据
    InvalidInputData,
    /// 计算失败
    ComputationFailed,
    /// 不支持的计算类型
    UnsupportedComputeType,

    // ==================== 序列化错误 ====================
    /// 序列化失败
    SerializationFailed,
    /// 反序列化失败
    DeserializationFailed,

    // ==================== 运行时错误 ====================
    /// 运行时未初始化
    RuntimeNotInitialized,
    /// 请求处理失败
    RequestProcessingFailed,
    /// 内存不足
    OutOfMemory,

    // ==================== 其他错误 ====================
    /// 内部错误
    InternalError(String),
}

impl core::fmt::Display for EnclaveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::KeyNotInitialized => write!(f, "Key not initialized"),
            Self::KeyGenerationFailed => write!(f, "Key generation failed"),
            Self::InvalidPublicKey => write!(f, "Invalid public key"),
            Self::KeyDerivationFailed => write!(f, "Key derivation failed"),
            Self::SealingFailed => write!(f, "Sealing failed"),
            Self::UnsealingFailed => write!(f, "Unsealing failed"),
            Self::EncryptionFailed => write!(f, "Encryption failed"),
            Self::DecryptionFailed => write!(f, "Decryption failed"),
            Self::InvalidNonce => write!(f, "Invalid nonce"),
            Self::AuthTagVerificationFailed => write!(f, "Auth tag verification failed"),
            Self::SigningFailed => write!(f, "Signing failed"),
            Self::SignatureVerificationFailed => write!(f, "Signature verification failed"),
            Self::InvalidSignature => write!(f, "Invalid signature"),
            Self::InvalidComputeType => write!(f, "Invalid compute type"),
            Self::InvalidInputData => write!(f, "Invalid input data"),
            Self::ComputationFailed => write!(f, "Computation failed"),
            Self::UnsupportedComputeType => write!(f, "Unsupported compute type"),
            Self::SerializationFailed => write!(f, "Serialization failed"),
            Self::DeserializationFailed => write!(f, "Deserialization failed"),
            Self::RuntimeNotInitialized => write!(f, "Runtime not initialized"),
            Self::RequestProcessingFailed => write!(f, "Request processing failed"),
            Self::OutOfMemory => write!(f, "Out of memory"),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EnclaveError {}

/// Enclave 结果类型
pub type EnclaveResult<T> = Result<T, EnclaveError>;
