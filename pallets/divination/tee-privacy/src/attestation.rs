//! # 远程认证验证模块
//!
//! 实现各种 TEE 平台的远程认证验证逻辑：
//! - Intel SGX EPID 认证
//! - Intel SGX DCAP 认证
//! - ARM TrustZone 认证
//! - AMD SEV 认证
//!
//! ## 验证流程
//!
//! 1. 解析认证报告格式
//! 2. 验证报告签名
//! 3. 检查 MRENCLAVE/MRSIGNER
//! 4. 验证报告数据（Enclave 公钥哈希等）
//! 5. 检查报告时效性

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use crate::types::{TeeAttestation, TeeType};

// ============================================================================
// 认证验证结果
// ============================================================================

/// 详细的认证验证结果
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub struct AttestationVerificationResult {
    /// 总体是否有效
    pub is_valid: bool,
    /// 签名验证结果
    pub signature_valid: bool,
    /// MRENCLAVE 是否在白名单中
    pub mr_enclave_trusted: bool,
    /// MRSIGNER 是否在白名单中
    pub mr_signer_trusted: bool,
    /// 报告数据验证结果
    pub report_data_valid: bool,
    /// 是否过期
    pub is_expired: bool,
    /// 安全版本是否足够
    pub svn_sufficient: bool,
    /// 错误代码（0 表示无错误）
    pub error_code: u32,
    /// 错误描述
    pub error_message: Option<Vec<u8>>,
}

impl Default for AttestationVerificationResult {
    fn default() -> Self {
        Self {
            is_valid: false,
            signature_valid: false,
            mr_enclave_trusted: false,
            mr_signer_trusted: false,
            report_data_valid: false,
            is_expired: true,
            svn_sufficient: false,
            error_code: 0,
            error_message: None,
        }
    }
}

/// 验证错误代码
pub mod error_codes {
    pub const SUCCESS: u32 = 0;
    pub const INVALID_SIGNATURE: u32 = 1;
    pub const UNTRUSTED_MRENCLAVE: u32 = 2;
    pub const UNTRUSTED_MRSIGNER: u32 = 3;
    pub const INVALID_REPORT_DATA: u32 = 4;
    pub const REPORT_EXPIRED: u32 = 5;
    pub const INSUFFICIENT_SVN: u32 = 6;
    pub const UNSUPPORTED_TEE_TYPE: u32 = 7;
    pub const MALFORMED_REPORT: u32 = 8;
    pub const QUOTE_STATUS_INVALID: u32 = 9;
    pub const CERTIFICATE_CHAIN_INVALID: u32 = 10;
}

// ============================================================================
// Intel SGX EPID 认证
// ============================================================================

/// EPID Quote 状态
#[derive(Clone, Copy, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub enum EpidQuoteStatus {
    /// 有效
    Ok = 0,
    /// 签名无效
    SignatureInvalid = 1,
    /// 组已撤销
    GroupRevoked = 2,
    /// 签名已撤销
    SignatureRevoked = 3,
    /// 密钥已撤销
    KeyRevoked = 4,
    /// SIGRL 版本不匹配
    SigrlVersionMismatch = 5,
    /// 组超出日期
    GroupOutOfDate = 6,
    /// 配置需要
    ConfigurationNeeded = 7,
    /// SW 加固需要
    SwHardeningNeeded = 8,
    /// 配置和 SW 加固需要
    ConfigurationAndSwHardeningNeeded = 9,
    /// 未知状态
    Unknown = 255,
}

impl Default for EpidQuoteStatus {
    fn default() -> Self {
        EpidQuoteStatus::Unknown
    }
}

impl From<u8> for EpidQuoteStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => EpidQuoteStatus::Ok,
            1 => EpidQuoteStatus::SignatureInvalid,
            2 => EpidQuoteStatus::GroupRevoked,
            3 => EpidQuoteStatus::SignatureRevoked,
            4 => EpidQuoteStatus::KeyRevoked,
            5 => EpidQuoteStatus::SigrlVersionMismatch,
            6 => EpidQuoteStatus::GroupOutOfDate,
            7 => EpidQuoteStatus::ConfigurationNeeded,
            8 => EpidQuoteStatus::SwHardeningNeeded,
            9 => EpidQuoteStatus::ConfigurationAndSwHardeningNeeded,
            _ => EpidQuoteStatus::Unknown,
        }
    }
}

/// EPID 认证报告结构
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug, Default)]
pub struct EpidReport {
    /// Quote 状态
    pub quote_status: EpidQuoteStatus,
    /// Quote 主体
    pub quote_body: EpidQuoteBody,
    /// IAS 报告 ID
    pub report_id: [u8; 32],
    /// IAS 时间戳
    pub timestamp: u64,
    /// 平台信息 Blob（可选）
    pub platform_info_blob: Option<Vec<u8>>,
    /// 咨询 ID 列表
    pub advisory_ids: Vec<Vec<u8>>,
}

/// EPID Quote 主体
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug, Default)]
pub struct EpidQuoteBody {
    /// Quote 版本
    pub version: u16,
    /// 签名类型 (0=未链接, 1=已链接)
    pub sign_type: u16,
    /// EPID 组 ID
    pub epid_group_id: [u8; 4],
    /// QE SVN
    pub qe_svn: u16,
    /// PCE SVN
    pub pce_svn: u16,
    /// 扩展 EPID 组 ID
    pub xeid: u32,
    /// 基本名称
    pub basename: [u8; 32],
    /// 报告主体
    pub report_body: SgxReportBody,
}

/// SGX 报告主体（EPID 和 DCAP 共用）
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub struct SgxReportBody {
    /// CPU SVN
    pub cpu_svn: [u8; 16],
    /// MISCSELECT
    pub misc_select: u32,
    /// 属性
    pub attributes: [u8; 16],
    /// MRENCLAVE
    pub mr_enclave: [u8; 32],
    /// MRSIGNER
    pub mr_signer: [u8; 32],
    /// ISV Product ID
    pub isv_prod_id: u16,
    /// ISV SVN
    pub isv_svn: u16,
    /// 报告数据
    pub report_data: [u8; 64],
}

impl Default for SgxReportBody {
    fn default() -> Self {
        Self {
            cpu_svn: [0u8; 16],
            misc_select: 0,
            attributes: [0u8; 16],
            mr_enclave: [0u8; 32],
            mr_signer: [0u8; 32],
            isv_prod_id: 0,
            isv_svn: 0,
            report_data: [0u8; 64],
        }
    }
}

/// EPID 认证验证器
pub struct EpidVerifier;

impl EpidVerifier {
    /// 验证 EPID 认证报告
    ///
    /// # 参数
    /// - `attestation`: TEE 认证报告
    /// - `trusted_mr_enclaves`: 可信的 MRENCLAVE 列表
    /// - `trusted_mr_signers`: 可信的 MRSIGNER 列表
    /// - `min_isv_svn`: 最小 ISV SVN 版本
    /// - `current_timestamp`: 当前时间戳
    /// - `max_report_age`: 报告最大有效期（秒）
    /// - `expected_report_data`: 预期的报告数据（通常是公钥哈希）
    pub fn verify(
        attestation: &TeeAttestation,
        trusted_mr_enclaves: &[[u8; 32]],
        trusted_mr_signers: &[[u8; 32]],
        min_isv_svn: u16,
        current_timestamp: u64,
        max_report_age: u64,
        expected_report_data: Option<&[u8; 64]>,
    ) -> AttestationVerificationResult {
        let mut result = AttestationVerificationResult::default();

        // 1. 检查 TEE 类型
        if attestation.tee_type != TeeType::IntelSgx {
            result.error_code = error_codes::UNSUPPORTED_TEE_TYPE;
            result.error_message = Some(b"Expected Intel SGX EPID attestation".to_vec());
            return result;
        }

        // 2. 验证 IAS 签名
        // 注意：在生产环境中，这里需要验证 Intel 的 RSA 签名
        // 目前使用简化验证（检查签名非空）
        if attestation.ias_signature.is_empty() {
            result.error_code = error_codes::INVALID_SIGNATURE;
            result.error_message = Some(b"IAS signature is empty".to_vec());
            return result;
        }
        result.signature_valid = Self::verify_ias_signature(attestation);

        if !result.signature_valid {
            result.error_code = error_codes::INVALID_SIGNATURE;
            result.error_message = Some(b"IAS signature verification failed".to_vec());
            return result;
        }

        // 3. 检查 MRENCLAVE
        result.mr_enclave_trusted = trusted_mr_enclaves.is_empty()
            || trusted_mr_enclaves.contains(&attestation.mr_enclave);

        if !result.mr_enclave_trusted {
            result.error_code = error_codes::UNTRUSTED_MRENCLAVE;
            result.error_message = Some(b"MRENCLAVE not in trusted list".to_vec());
            return result;
        }

        // 4. 检查 MRSIGNER
        result.mr_signer_trusted = trusted_mr_signers.is_empty()
            || trusted_mr_signers.contains(&attestation.mr_signer);

        if !result.mr_signer_trusted {
            result.error_code = error_codes::UNTRUSTED_MRSIGNER;
            result.error_message = Some(b"MRSIGNER not in trusted list".to_vec());
            return result;
        }

        // 5. 检查报告数据
        if let Some(expected) = expected_report_data {
            result.report_data_valid = attestation.report_data == *expected;
            if !result.report_data_valid {
                result.error_code = error_codes::INVALID_REPORT_DATA;
                result.error_message = Some(b"Report data mismatch".to_vec());
                return result;
            }
        } else {
            result.report_data_valid = true;
        }

        // 6. 检查过期
        if current_timestamp > attestation.timestamp {
            let age = current_timestamp - attestation.timestamp;
            result.is_expired = age > max_report_age;
        } else {
            result.is_expired = false;
        }

        if result.is_expired {
            result.error_code = error_codes::REPORT_EXPIRED;
            result.error_message = Some(b"Attestation report expired".to_vec());
            return result;
        }

        // 7. 检查 SVN
        result.svn_sufficient = attestation.isv_svn >= min_isv_svn;

        if !result.svn_sufficient {
            result.error_code = error_codes::INSUFFICIENT_SVN;
            result.error_message = Some(b"ISV SVN too low".to_vec());
            return result;
        }

        // 所有检查通过
        result.is_valid = true;
        result.error_code = error_codes::SUCCESS;
        result
    }

    /// 验证 IAS 签名
    ///
    /// 注意：生产环境需要实现完整的 RSA 签名验证
    fn verify_ias_signature(attestation: &TeeAttestation) -> bool {
        // 简化验证：检查签名格式
        // 实际实现需要：
        // 1. 获取 Intel 的报告签名证书
        // 2. 验证证书链
        // 3. 使用公钥验证报告的 RSA-SHA256 签名

        // 检查签名长度（RSA-2048 签名为 256 字节）
        if attestation.ias_signature.len() < 256 {
            // 开发环境可能使用测试签名
            return attestation.ias_signature.len() >= 64;
        }

        true
    }

    /// 解析 IAS 响应中的 Quote 状态
    pub fn parse_quote_status(status_str: &[u8]) -> EpidQuoteStatus {
        match status_str {
            b"OK" => EpidQuoteStatus::Ok,
            b"SIGNATURE_INVALID" => EpidQuoteStatus::SignatureInvalid,
            b"GROUP_REVOKED" => EpidQuoteStatus::GroupRevoked,
            b"SIGNATURE_REVOKED" => EpidQuoteStatus::SignatureRevoked,
            b"KEY_REVOKED" => EpidQuoteStatus::KeyRevoked,
            b"SIGRL_VERSION_MISMATCH" => EpidQuoteStatus::SigrlVersionMismatch,
            b"GROUP_OUT_OF_DATE" => EpidQuoteStatus::GroupOutOfDate,
            b"CONFIGURATION_NEEDED" => EpidQuoteStatus::ConfigurationNeeded,
            b"SW_HARDENING_NEEDED" => EpidQuoteStatus::SwHardeningNeeded,
            b"CONFIGURATION_AND_SW_HARDENING_NEEDED" => {
                EpidQuoteStatus::ConfigurationAndSwHardeningNeeded
            }
            _ => EpidQuoteStatus::Unknown,
        }
    }
}

// ============================================================================
// Intel SGX DCAP 认证
// ============================================================================

/// DCAP Quote 版本
#[derive(Clone, Copy, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub enum DcapQuoteVersion {
    V3 = 3,
    V4 = 4,
}

impl Default for DcapQuoteVersion {
    fn default() -> Self {
        DcapQuoteVersion::V3
    }
}

/// DCAP TCB 状态
#[derive(Clone, Copy, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub enum TcbStatus {
    /// 最新
    UpToDate = 0,
    /// SW 加固需要
    SwHardeningNeeded = 1,
    /// 配置需要
    ConfigurationNeeded = 2,
    /// 配置和 SW 加固需要
    ConfigurationAndSwHardeningNeeded = 3,
    /// 过时
    OutOfDate = 4,
    /// 过时且配置需要
    OutOfDateConfigurationNeeded = 5,
    /// 已撤销
    Revoked = 6,
    /// 未知
    Unknown = 255,
}

impl Default for TcbStatus {
    fn default() -> Self {
        TcbStatus::Unknown
    }
}

/// DCAP 认证报告
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug, Default)]
pub struct DcapReport {
    /// Quote 版本
    pub version: DcapQuoteVersion,
    /// 认证密钥类型
    pub attestation_key_type: u16,
    /// TEE 类型
    pub tee_type: u32,
    /// QE 供应商 ID
    pub qe_vendor_id: [u8; 16],
    /// 用户数据
    pub user_data: [u8; 20],
    /// 报告主体
    pub report_body: SgxReportBody,
    /// TCB 状态
    pub tcb_status: TcbStatus,
    /// 认证数据
    pub cert_data: Vec<u8>,
}

/// DCAP 认证验证器
pub struct DcapVerifier;

impl DcapVerifier {
    /// 验证 DCAP 认证报告
    pub fn verify(
        attestation: &TeeAttestation,
        trusted_mr_enclaves: &[[u8; 32]],
        trusted_mr_signers: &[[u8; 32]],
        min_isv_svn: u16,
        current_timestamp: u64,
        max_report_age: u64,
        expected_report_data: Option<&[u8; 64]>,
    ) -> AttestationVerificationResult {
        let mut result = AttestationVerificationResult::default();

        // 1. 检查 TEE 类型
        if attestation.tee_type != TeeType::IntelSgxDcap {
            result.error_code = error_codes::UNSUPPORTED_TEE_TYPE;
            result.error_message = Some(b"Expected Intel SGX DCAP attestation".to_vec());
            return result;
        }

        // 2. 验证 ECDSA 签名和证书链
        result.signature_valid = Self::verify_dcap_signature(attestation);

        if !result.signature_valid {
            result.error_code = error_codes::INVALID_SIGNATURE;
            result.error_message = Some(b"DCAP signature verification failed".to_vec());
            return result;
        }

        // 3. 检查 MRENCLAVE
        result.mr_enclave_trusted = trusted_mr_enclaves.is_empty()
            || trusted_mr_enclaves.contains(&attestation.mr_enclave);

        if !result.mr_enclave_trusted {
            result.error_code = error_codes::UNTRUSTED_MRENCLAVE;
            result.error_message = Some(b"MRENCLAVE not in trusted list".to_vec());
            return result;
        }

        // 4. 检查 MRSIGNER
        result.mr_signer_trusted = trusted_mr_signers.is_empty()
            || trusted_mr_signers.contains(&attestation.mr_signer);

        if !result.mr_signer_trusted {
            result.error_code = error_codes::UNTRUSTED_MRSIGNER;
            result.error_message = Some(b"MRSIGNER not in trusted list".to_vec());
            return result;
        }

        // 5. 检查报告数据
        if let Some(expected) = expected_report_data {
            result.report_data_valid = attestation.report_data == *expected;
            if !result.report_data_valid {
                result.error_code = error_codes::INVALID_REPORT_DATA;
                result.error_message = Some(b"Report data mismatch".to_vec());
                return result;
            }
        } else {
            result.report_data_valid = true;
        }

        // 6. 检查过期
        if current_timestamp > attestation.timestamp {
            let age = current_timestamp - attestation.timestamp;
            result.is_expired = age > max_report_age;
        } else {
            result.is_expired = false;
        }

        if result.is_expired {
            result.error_code = error_codes::REPORT_EXPIRED;
            result.error_message = Some(b"Attestation report expired".to_vec());
            return result;
        }

        // 7. 检查 SVN
        result.svn_sufficient = attestation.isv_svn >= min_isv_svn;

        if !result.svn_sufficient {
            result.error_code = error_codes::INSUFFICIENT_SVN;
            result.error_message = Some(b"ISV SVN too low".to_vec());
            return result;
        }

        // 所有检查通过
        result.is_valid = true;
        result.error_code = error_codes::SUCCESS;
        result
    }

    /// 验证 DCAP ECDSA 签名
    fn verify_dcap_signature(attestation: &TeeAttestation) -> bool {
        // DCAP 使用 ECDSA-256 签名
        // 实际实现需要：
        // 1. 解析 PCK 证书链
        // 2. 验证证书链到 Intel 根证书
        // 3. 验证 Quote 签名

        // 简化验证
        !attestation.ias_signature.is_empty()
    }

    /// 解析 TCB 状态
    pub fn parse_tcb_status(status: u8) -> TcbStatus {
        match status {
            0 => TcbStatus::UpToDate,
            1 => TcbStatus::SwHardeningNeeded,
            2 => TcbStatus::ConfigurationNeeded,
            3 => TcbStatus::ConfigurationAndSwHardeningNeeded,
            4 => TcbStatus::OutOfDate,
            5 => TcbStatus::OutOfDateConfigurationNeeded,
            6 => TcbStatus::Revoked,
            _ => TcbStatus::Unknown,
        }
    }
}

// ============================================================================
// ARM TrustZone 认证
// ============================================================================

/// TrustZone 认证令牌类型
#[derive(Clone, Copy, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub enum TrustZoneTokenType {
    /// PSA 初始认证令牌
    PsaInitialAttestation = 0,
    /// CCA 平台令牌
    CcaPlatformToken = 1,
    /// CCA 领域令牌
    CcaRealmToken = 2,
}

impl Default for TrustZoneTokenType {
    fn default() -> Self {
        TrustZoneTokenType::PsaInitialAttestation
    }
}

/// TrustZone 认证报告
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub struct TrustZoneReport {
    /// 令牌类型
    pub token_type: TrustZoneTokenType,
    /// 实现 ID
    pub implementation_id: [u8; 32],
    /// 实例 ID
    pub instance_id: [u8; 33],
    /// 安全生命周期
    pub security_lifecycle: u32,
    /// 引导种子
    pub boot_seed: [u8; 32],
    /// 软件组件列表哈希
    pub sw_components_hash: [u8; 32],
    /// 无软件组件标志
    pub no_sw_components: bool,
    /// 挑战值/nonce
    pub challenge: [u8; 64],
    /// 签名
    pub signature: Vec<u8>,
}

impl Default for TrustZoneReport {
    fn default() -> Self {
        Self {
            token_type: TrustZoneTokenType::default(),
            implementation_id: [0u8; 32],
            instance_id: [0u8; 33],
            security_lifecycle: 0,
            boot_seed: [0u8; 32],
            sw_components_hash: [0u8; 32],
            no_sw_components: false,
            challenge: [0u8; 64],
            signature: Vec::new(),
        }
    }
}

/// TrustZone 认证验证器
pub struct TrustZoneVerifier;

impl TrustZoneVerifier {
    /// 验证 TrustZone 认证报告
    pub fn verify(
        attestation: &TeeAttestation,
        trusted_implementations: &[[u8; 32]],
        current_timestamp: u64,
        max_report_age: u64,
        expected_report_data: Option<&[u8; 64]>,
    ) -> AttestationVerificationResult {
        let mut result = AttestationVerificationResult::default();

        // 1. 检查 TEE 类型
        if attestation.tee_type != TeeType::ArmTrustZone {
            result.error_code = error_codes::UNSUPPORTED_TEE_TYPE;
            result.error_message = Some(b"Expected ARM TrustZone attestation".to_vec());
            return result;
        }

        // 2. 验证令牌签名
        result.signature_valid = Self::verify_token_signature(attestation);

        if !result.signature_valid {
            result.error_code = error_codes::INVALID_SIGNATURE;
            result.error_message = Some(b"Token signature verification failed".to_vec());
            return result;
        }

        // 3. 对于 TrustZone，使用 mr_enclave 存储实现 ID
        result.mr_enclave_trusted = trusted_implementations.is_empty()
            || trusted_implementations.contains(&attestation.mr_enclave);

        if !result.mr_enclave_trusted {
            result.error_code = error_codes::UNTRUSTED_MRENCLAVE;
            result.error_message = Some(b"Implementation ID not in trusted list".to_vec());
            return result;
        }

        // 4. 检查报告数据（挑战值）
        if let Some(expected) = expected_report_data {
            result.report_data_valid = attestation.report_data == *expected;
            if !result.report_data_valid {
                result.error_code = error_codes::INVALID_REPORT_DATA;
                result.error_message = Some(b"Challenge mismatch".to_vec());
                return result;
            }
        } else {
            result.report_data_valid = true;
        }

        // 5. 检查过期
        if current_timestamp > attestation.timestamp {
            let age = current_timestamp - attestation.timestamp;
            result.is_expired = age > max_report_age;
        } else {
            result.is_expired = false;
        }

        if result.is_expired {
            result.error_code = error_codes::REPORT_EXPIRED;
            result.error_message = Some(b"Attestation report expired".to_vec());
            return result;
        }

        // TrustZone 不使用 SVN 概念
        result.svn_sufficient = true;
        result.mr_signer_trusted = true;

        // 所有检查通过
        result.is_valid = true;
        result.error_code = error_codes::SUCCESS;
        result
    }

    /// 验证令牌签名
    fn verify_token_signature(attestation: &TeeAttestation) -> bool {
        // PSA 令牌使用 COSE 签名格式
        // 实际实现需要解析 COSE 结构并验证签名

        !attestation.ias_signature.is_empty()
    }
}

// ============================================================================
// AMD SEV 认证
// ============================================================================

/// SEV 报告类型
#[derive(Clone, Copy, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub enum SevReportType {
    /// SEV
    Sev = 0,
    /// SEV-ES
    SevEs = 1,
    /// SEV-SNP
    SevSnp = 2,
}

impl Default for SevReportType {
    fn default() -> Self {
        SevReportType::SevSnp
    }
}

/// AMD SEV 认证报告
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub struct SevReport {
    /// 报告类型
    pub report_type: SevReportType,
    /// 报告版本
    pub version: u32,
    /// 客户 ID
    pub guest_svn: u32,
    /// 策略
    pub policy: u64,
    /// 度量值
    pub measurement: [u8; 48],
    /// 主机数据
    pub host_data: [u8; 32],
    /// 报告数据
    pub report_data: [u8; 64],
    /// 芯片 ID
    pub chip_id: [u8; 64],
    /// 签名
    pub signature: Vec<u8>,
}

impl Default for SevReport {
    fn default() -> Self {
        Self {
            report_type: SevReportType::default(),
            version: 0,
            guest_svn: 0,
            policy: 0,
            measurement: [0u8; 48],
            host_data: [0u8; 32],
            report_data: [0u8; 64],
            chip_id: [0u8; 64],
            signature: Vec::new(),
        }
    }
}

/// AMD SEV 认证验证器
pub struct SevVerifier;

impl SevVerifier {
    /// 验证 SEV 认证报告
    pub fn verify(
        attestation: &TeeAttestation,
        trusted_measurements: &[[u8; 32]],
        current_timestamp: u64,
        max_report_age: u64,
        expected_report_data: Option<&[u8; 64]>,
    ) -> AttestationVerificationResult {
        let mut result = AttestationVerificationResult::default();

        // 1. 检查 TEE 类型
        if attestation.tee_type != TeeType::AmdSev {
            result.error_code = error_codes::UNSUPPORTED_TEE_TYPE;
            result.error_message = Some(b"Expected AMD SEV attestation".to_vec());
            return result;
        }

        // 2. 验证报告签名
        result.signature_valid = Self::verify_report_signature(attestation);

        if !result.signature_valid {
            result.error_code = error_codes::INVALID_SIGNATURE;
            result.error_message = Some(b"SEV report signature verification failed".to_vec());
            return result;
        }

        // 3. 对于 SEV，使用 mr_enclave 存储度量值的前 32 字节
        result.mr_enclave_trusted = trusted_measurements.is_empty()
            || trusted_measurements.contains(&attestation.mr_enclave);

        if !result.mr_enclave_trusted {
            result.error_code = error_codes::UNTRUSTED_MRENCLAVE;
            result.error_message = Some(b"Measurement not in trusted list".to_vec());
            return result;
        }

        // 4. 检查报告数据
        if let Some(expected) = expected_report_data {
            result.report_data_valid = attestation.report_data == *expected;
            if !result.report_data_valid {
                result.error_code = error_codes::INVALID_REPORT_DATA;
                result.error_message = Some(b"Report data mismatch".to_vec());
                return result;
            }
        } else {
            result.report_data_valid = true;
        }

        // 5. 检查过期
        if current_timestamp > attestation.timestamp {
            let age = current_timestamp - attestation.timestamp;
            result.is_expired = age > max_report_age;
        } else {
            result.is_expired = false;
        }

        if result.is_expired {
            result.error_code = error_codes::REPORT_EXPIRED;
            result.error_message = Some(b"Attestation report expired".to_vec());
            return result;
        }

        // SEV 使用 guest_svn 作为版本
        result.svn_sufficient = true;
        result.mr_signer_trusted = true;

        // 所有检查通过
        result.is_valid = true;
        result.error_code = error_codes::SUCCESS;
        result
    }

    /// 验证 SEV 报告签名
    fn verify_report_signature(attestation: &TeeAttestation) -> bool {
        // SEV-SNP 使用 VCEK 证书签名
        // 实际实现需要验证证书链和 ECDSA 签名

        !attestation.ias_signature.is_empty()
    }
}

// ============================================================================
// 统一验证接口
// ============================================================================

/// 统一的认证验证器
pub struct AttestationVerifier;

impl AttestationVerifier {
    /// 根据 TEE 类型自动选择验证器
    pub fn verify(
        attestation: &TeeAttestation,
        trusted_mr_enclaves: &[[u8; 32]],
        trusted_mr_signers: &[[u8; 32]],
        min_isv_svn: u16,
        current_timestamp: u64,
        max_report_age: u64,
        expected_report_data: Option<&[u8; 64]>,
    ) -> AttestationVerificationResult {
        match attestation.tee_type {
            TeeType::IntelSgx => EpidVerifier::verify(
                attestation,
                trusted_mr_enclaves,
                trusted_mr_signers,
                min_isv_svn,
                current_timestamp,
                max_report_age,
                expected_report_data,
            ),
            TeeType::IntelSgxDcap => DcapVerifier::verify(
                attestation,
                trusted_mr_enclaves,
                trusted_mr_signers,
                min_isv_svn,
                current_timestamp,
                max_report_age,
                expected_report_data,
            ),
            TeeType::ArmTrustZone => TrustZoneVerifier::verify(
                attestation,
                trusted_mr_enclaves,
                current_timestamp,
                max_report_age,
                expected_report_data,
            ),
            TeeType::AmdSev => SevVerifier::verify(
                attestation,
                trusted_mr_enclaves,
                current_timestamp,
                max_report_age,
                expected_report_data,
            ),
            TeeType::RiscVKeystone => {
                // Keystone 支持待实现
                let mut result = AttestationVerificationResult::default();
                result.error_code = error_codes::UNSUPPORTED_TEE_TYPE;
                result.error_message = Some(b"RISC-V Keystone not yet supported".to_vec());
                result
            }
        }
    }

    /// 快速验证（仅检查基本有效性）
    pub fn quick_verify(
        attestation: &TeeAttestation,
        current_timestamp: u64,
        max_report_age: u64,
    ) -> bool {
        // 检查签名非空
        if attestation.ias_signature.is_empty() {
            return false;
        }

        // 检查过期
        if current_timestamp > attestation.timestamp {
            let age = current_timestamp - attestation.timestamp;
            if age > max_report_age {
                return false;
            }
        }

        true
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::BoundedVec;

    fn create_test_attestation(tee_type: TeeType) -> TeeAttestation {
        TeeAttestation {
            tee_type,
            mr_enclave: [1u8; 32],
            mr_signer: [2u8; 32],
            isv_prod_id: 1,
            isv_svn: 1,
            report_data: [0u8; 64],
            ias_signature: BoundedVec::try_from(vec![0u8; 256]).unwrap(),
            timestamp: 1704067200, // 2024-01-01
        }
    }

    #[test]
    fn test_epid_verification_success() {
        let attestation = create_test_attestation(TeeType::IntelSgx);
        let trusted_enclaves = [[1u8; 32]];
        let trusted_signers = [[2u8; 32]];

        let result = EpidVerifier::verify(
            &attestation,
            &trusted_enclaves,
            &trusted_signers,
            1,                // min_svn
            1704067200 + 100, // current_timestamp
            86400,            // max_age = 1 day
            None,
        );

        assert!(result.is_valid);
        assert!(result.signature_valid);
        assert!(result.mr_enclave_trusted);
        assert!(result.mr_signer_trusted);
        assert!(!result.is_expired);
    }

    #[test]
    fn test_epid_verification_expired() {
        let attestation = create_test_attestation(TeeType::IntelSgx);

        let result = EpidVerifier::verify(
            &attestation,
            &[],
            &[],
            1,
            1704067200 + 100000, // 过期
            86400,
            None,
        );

        assert!(!result.is_valid);
        assert!(result.is_expired);
        assert_eq!(result.error_code, error_codes::REPORT_EXPIRED);
    }

    #[test]
    fn test_epid_verification_untrusted_mrenclave() {
        let attestation = create_test_attestation(TeeType::IntelSgx);
        let trusted_enclaves = [[99u8; 32]]; // 不匹配

        let result = EpidVerifier::verify(
            &attestation,
            &trusted_enclaves,
            &[],
            1,
            1704067200 + 100,
            86400,
            None,
        );

        assert!(!result.is_valid);
        assert!(!result.mr_enclave_trusted);
        assert_eq!(result.error_code, error_codes::UNTRUSTED_MRENCLAVE);
    }

    #[test]
    fn test_dcap_verification() {
        let attestation = create_test_attestation(TeeType::IntelSgxDcap);

        let result = DcapVerifier::verify(
            &attestation,
            &[],
            &[],
            1,
            1704067200 + 100,
            86400,
            None,
        );

        assert!(result.is_valid);
    }

    #[test]
    fn test_trustzone_verification() {
        let attestation = create_test_attestation(TeeType::ArmTrustZone);

        let result = TrustZoneVerifier::verify(
            &attestation,
            &[],
            1704067200 + 100,
            86400,
            None,
        );

        assert!(result.is_valid);
    }

    #[test]
    fn test_sev_verification() {
        let attestation = create_test_attestation(TeeType::AmdSev);

        let result = SevVerifier::verify(
            &attestation,
            &[],
            1704067200 + 100,
            86400,
            None,
        );

        assert!(result.is_valid);
    }

    #[test]
    fn test_unified_verifier() {
        // 测试 SGX EPID
        let epid_att = create_test_attestation(TeeType::IntelSgx);
        let result = AttestationVerifier::verify(&epid_att, &[], &[], 1, 1704067200 + 100, 86400, None);
        assert!(result.is_valid);

        // 测试 SGX DCAP
        let dcap_att = create_test_attestation(TeeType::IntelSgxDcap);
        let result = AttestationVerifier::verify(&dcap_att, &[], &[], 1, 1704067200 + 100, 86400, None);
        assert!(result.is_valid);

        // 测试 TrustZone
        let tz_att = create_test_attestation(TeeType::ArmTrustZone);
        let result = AttestationVerifier::verify(&tz_att, &[], &[], 1, 1704067200 + 100, 86400, None);
        assert!(result.is_valid);

        // 测试 SEV
        let sev_att = create_test_attestation(TeeType::AmdSev);
        let result = AttestationVerifier::verify(&sev_att, &[], &[], 1, 1704067200 + 100, 86400, None);
        assert!(result.is_valid);
    }

    #[test]
    fn test_quick_verify() {
        let attestation = create_test_attestation(TeeType::IntelSgx);

        // 有效
        assert!(AttestationVerifier::quick_verify(&attestation, 1704067200 + 100, 86400));

        // 过期
        assert!(!AttestationVerifier::quick_verify(&attestation, 1704067200 + 100000, 86400));

        // 空签名
        let mut empty_sig = attestation.clone();
        empty_sig.ias_signature = BoundedVec::new();
        assert!(!AttestationVerifier::quick_verify(&empty_sig, 1704067200 + 100, 86400));
    }

    #[test]
    fn test_quote_status_parsing() {
        assert_eq!(EpidVerifier::parse_quote_status(b"OK"), EpidQuoteStatus::Ok);
        assert_eq!(
            EpidVerifier::parse_quote_status(b"GROUP_OUT_OF_DATE"),
            EpidQuoteStatus::GroupOutOfDate
        );
        assert_eq!(
            EpidVerifier::parse_quote_status(b"UNKNOWN_STATUS"),
            EpidQuoteStatus::Unknown
        );
    }
}
