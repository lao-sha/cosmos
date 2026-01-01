//! KYC验证相关逻辑实现

use crate::{Config, Error, Event, types::*};
use crate::pallet::IdentityVerificationProvider;
use frame_support::pallet_prelude::*;

impl<T: Config> crate::Pallet<T> {
    /// 检查用户是否满足KYC要求
    pub fn verify_kyc(who: &T::AccountId) -> KycVerificationResult {
        // 获取当前KYC配置
        let config = crate::pallet::KycConfig::<T>::get();

        // 如果KYC未启用，直接跳过
        if !config.enabled {
            return KycVerificationResult::Skipped;
        }

        // 检查是否为豁免账户
        if Self::is_kyc_exempt(who) {
            return KycVerificationResult::Exempted;
        }

        // 验证身份认证状态
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
        // 使用 IdentityProvider 获取身份信息
        let highest_priority = T::IdentityProvider::get_highest_judgement_priority(who)
            .ok_or(KycFailureReason::IdentityNotSet)?;

        // 检查是否为问题判断
        if T::IdentityProvider::has_problematic_judgement(who) {
            return Err(KycFailureReason::QualityIssue);
        }

        // 检查等级是否足够
        if highest_priority >= min_priority {
            Ok(())
        } else {
            Err(KycFailureReason::InsufficientLevel)
        }
    }

    /// 检查账户是否为KYC豁免账户
    pub fn is_kyc_exempt(who: &T::AccountId) -> bool {
        crate::KycExemptAccounts::<T>::contains_key(who)
    }

    /// 强制执行KYC检查（创建订单时使用）
    pub fn enforce_kyc_requirement(who: &T::AccountId) -> DispatchResult {
        match Self::verify_kyc(who) {
            KycVerificationResult::Passed |
            KycVerificationResult::Exempted |
            KycVerificationResult::Skipped => Ok(()),

            KycVerificationResult::Failed(reason) => {
                // 发出KYC验证失败事件
                Self::deposit_event(Event::KycVerificationFailed {
                    account: who.clone(),
                    reason_code: reason.to_code(),
                });

                // 返回对应的错误
                match reason {
                    KycFailureReason::IdentityNotSet =>
                        Err(Error::<T>::IdentityNotSet.into()),
                    KycFailureReason::NoValidJudgement =>
                        Err(Error::<T>::NoValidJudgement.into()),
                    KycFailureReason::InsufficientLevel =>
                        Err(Error::<T>::InsufficientKycLevel.into()),
                    KycFailureReason::QualityIssue =>
                        Err(Error::<T>::IdentityQualityIssue.into()),
                }
            }
        }
    }
}