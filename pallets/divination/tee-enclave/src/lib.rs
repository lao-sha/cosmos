//! # TEE Enclave 应用
//!
//! 本模块实现运行在可信执行环境 (TEE) 内的占卜计算引擎。
//!
//! ## 架构概述
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                          TEE Enclave                                     │
//! │  ┌─────────────────┐ ┌─────────────────┐ ┌────────────────────────────┐ │
//! │  │   KeyManager    │ │    Crypto       │ │    DivinationEngine       │ │
//! │  │                 │ │                 │ │                            │ │
//! │  │ - ECDH 密钥对   │ │ - AES-256-GCM   │ │ - 八字计算                 │ │
//! │  │ - Ed25519 签名  │ │ - X25519 密钥   │ │ - 梅花易数                 │ │
//! │  │ - 密钥密封      │ │ - SHA-256 哈希  │ │ - 奇门遁甲                 │ │
//! │  └─────────────────┘ └─────────────────┘ │ - 六爻/紫微/塔罗等         │ │
//! │                                          └────────────────────────────┘ │
//! │  ┌─────────────────────────────────────────────────────────────────┐   │
//! │  │                     EnclaveRuntime                               │   │
//! │  │  - 请求处理循环                                                   │   │
//! │  │  - 解密输入 -> 计算 -> 加密输出 -> 签名                            │   │
//! │  └─────────────────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## 安全特性
//!
//! 1. **密钥隔离**: 所有密钥仅存在于 Enclave 内存中
//! 2. **密钥密封**: 使用 SGX Sealing 持久化密钥
//! 3. **远程认证**: 支持 EPID/DCAP 认证
//! 4. **计算证明**: 所有输出都附带 Enclave 签名
//!
//! ## 使用示例
//!
//! ```ignore
//! use tee_enclave::{EnclaveRuntime, ComputeRequest};
//!
//! // 初始化 Enclave 运行时
//! let runtime = EnclaveRuntime::new()?;
//!
//! // 处理计算请求
//! let result = runtime.process_request(request)?;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

pub mod crypto;
pub mod divination;
pub mod error;
pub mod keys;
pub mod runtime;
pub mod types;

pub use crypto::*;
pub use divination::*;
pub use error::*;
pub use keys::*;
pub use runtime::*;
pub use types::*;

/// Enclave 版本
pub const ENCLAVE_VERSION: &str = "0.1.0";

/// 支持的计算类型数量
pub const SUPPORTED_COMPUTE_TYPES: usize = 8;
