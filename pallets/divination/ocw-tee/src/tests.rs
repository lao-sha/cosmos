//! # OCW + TEE 通用模块测试

use crate::types::*;

#[test]
fn test_privacy_mode_conversion() {
    assert_eq!(PrivacyMode::from_u8(0), Some(PrivacyMode::Public));
    assert_eq!(PrivacyMode::from_u8(1), Some(PrivacyMode::Encrypted));
    assert_eq!(PrivacyMode::from_u8(2), Some(PrivacyMode::Private));
    assert_eq!(PrivacyMode::from_u8(3), None);
}

#[test]
fn test_privacy_mode_properties() {
    // Public
    assert!(!PrivacyMode::Public.requires_tee());
    assert!(PrivacyMode::Public.stores_index_on_chain());

    // Encrypted
    assert!(PrivacyMode::Encrypted.requires_tee());
    assert!(PrivacyMode::Encrypted.stores_index_on_chain());

    // Private
    assert!(PrivacyMode::Private.requires_tee());
    assert!(!PrivacyMode::Private.stores_index_on_chain());
}

#[test]
fn test_request_status() {
    assert!(!RequestStatus::Pending.is_final());
    assert!(!RequestStatus::Processing.is_final());
    assert!(RequestStatus::Completed.is_final());
    assert!(!RequestStatus::Failed.is_final());
    assert!(RequestStatus::Timeout.is_final());

    assert!(!RequestStatus::Pending.is_retryable());
    assert!(RequestStatus::Failed.is_retryable());
}

#[test]
fn test_divination_type_endpoints() {
    assert_eq!(DivinationType::BaZi.tee_endpoint(), "/compute/bazi");
    assert_eq!(DivinationType::QiMen.tee_endpoint(), "/compute/qimen");
    assert_eq!(DivinationType::Meihua.tee_endpoint(), "/compute/meihua");
    assert_eq!(DivinationType::LiuYao.tee_endpoint(), "/compute/liuyao");
    assert_eq!(DivinationType::ZiWei.tee_endpoint(), "/compute/ziwei");
    assert_eq!(DivinationType::DaLiuRen.tee_endpoint(), "/compute/daliuren");
    assert_eq!(DivinationType::XiaoLiuRen.tee_endpoint(), "/compute/xiaoliuren");
    assert_eq!(DivinationType::Tarot.tee_endpoint(), "/compute/tarot");
}

#[test]
fn test_divination_type_timeouts() {
    // 简单模块超时较短
    assert_eq!(DivinationType::XiaoLiuRen.recommended_timeout(), 60);
    assert_eq!(DivinationType::Tarot.recommended_timeout(), 60);
    assert_eq!(DivinationType::Meihua.recommended_timeout(), 80);

    // 中等复杂度
    assert_eq!(DivinationType::BaZi.recommended_timeout(), 100);
    assert_eq!(DivinationType::LiuYao.recommended_timeout(), 100);

    // 复杂模块超时较长
    assert_eq!(DivinationType::QiMen.recommended_timeout(), 150);
    assert_eq!(DivinationType::ZiWei.recommended_timeout(), 150);
    assert_eq!(DivinationType::DaLiuRen.recommended_timeout(), 150);
}

#[test]
fn test_divination_type_input_sizes() {
    // 简单输入
    assert_eq!(DivinationType::Meihua.max_input_size(), 128);
    assert_eq!(DivinationType::XiaoLiuRen.max_input_size(), 128);

    // 中等输入
    assert_eq!(DivinationType::BaZi.max_input_size(), 256);
    assert_eq!(DivinationType::LiuYao.max_input_size(), 256);

    // 复杂输入（包含占问事宜）
    assert_eq!(DivinationType::QiMen.max_input_size(), 512);
}

#[test]
fn test_module_error_creation() {
    let err = ModuleError::invalid_input(b"test error");
    match err {
        ModuleError::InvalidInput(msg) => {
            assert_eq!(msg.as_slice(), b"test error");
        }
        _ => panic!("Expected InvalidInput"),
    }

    let err = ModuleError::computation_failed(b"compute error");
    match err {
        ModuleError::ComputationFailed(msg) => {
            assert_eq!(msg.as_slice(), b"compute error");
        }
        _ => panic!("Expected ComputationFailed"),
    }
}

#[test]
fn test_computation_proof_verify() {
    // 空证明应该失败
    let empty_proof = ComputationProof {
        mrenclave: [0u8; 32],
        input_hash: [0u8; 32],
        output_hash: [0u8; 32],
        timestamp: 0,
        signature: [0u8; 64],
    };
    assert!(!empty_proof.verify());

    // 非空证明应该通过（简化验证）
    let valid_proof = ComputationProof {
        mrenclave: [1u8; 32],
        input_hash: [2u8; 32],
        output_hash: [3u8; 32],
        timestamp: 1234567890,
        signature: [4u8; 64],
    };
    assert!(valid_proof.verify());
}
