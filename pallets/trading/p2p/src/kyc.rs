//! # P2P KYC 验证模块
//!
//! 从 pallet-trading-otc/src/kyc.rs 迁移而来。
//!
//! ## 设计
//! - KYC 核心逻辑（enforce_kyc_requirement）已内联在 lib.rs 的 Helpers 区段
//! - 本模块提供独立的 verify_kyc 查询接口，供 RPC 或外部调用

use crate::pallet::{Config, Pallet, KycConfigStore, KycExemptAccounts, IdentityVerificationProvider};
use crate::types::*;

impl<T: Config> Pallet<T> {
    /// 查询用户 KYC 验证结果（不产生事件，纯查询）
    pub fn verify_kyc(who: &T::AccountId) -> KycVerificationResult {
        let config = KycConfigStore::<T>::get();
        if !config.enabled {
            return KycVerificationResult::Skipped;
        }
        if KycExemptAccounts::<T>::contains_key(who) {
            return KycVerificationResult::Exempted;
        }
        match Self::check_identity_judgement(who, config.min_judgment_priority) {
            Ok(()) => KycVerificationResult::Passed,
            Err(reason) => KycVerificationResult::Failed(reason),
        }
    }

    /// 检查身份认证判断是否满足要求
    fn check_identity_judgement(
        who: &T::AccountId,
        min_priority: u8,
    ) -> Result<(), KycFailureReason> {
        let highest_priority = T::IdentityProvider::get_highest_judgement_priority(who)
            .ok_or(KycFailureReason::IdentityNotSet)?;
        if T::IdentityProvider::has_problematic_judgement(who) {
            return Err(KycFailureReason::QualityIssue);
        }
        if highest_priority >= min_priority {
            Ok(())
        } else {
            Err(KycFailureReason::InsufficientLevel)
        }
    }

    /// 检查账户是否为 KYC 豁免账户
    pub fn is_kyc_exempt(who: &T::AccountId) -> bool {
        KycExemptAccounts::<T>::contains_key(who)
    }
}
