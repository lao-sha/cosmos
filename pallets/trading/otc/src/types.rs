//! OTC订单KYC认证相关类型定义

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

/// KYC配置结构
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct KycConfig<BlockNumber> {
    /// 是否启用KYC要求
    pub enabled: bool,
    /// 创建OTC订单的最低认证等级（数值表示：0=Unknown, 1=FeePaid, 2=Reasonable, 3=KnownGood）
    pub min_judgment_priority: u8,
    /// 配置生效的区块高度
    pub effective_block: BlockNumber,
    /// 最后更新时间
    pub updated_at: BlockNumber,
}

/// KYC验证结果
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, RuntimeDebug)]
pub enum KycVerificationResult {
    /// 验证通过
    Passed,
    /// 验证失败
    Failed(KycFailureReason),
    /// 豁免：用户在豁免列表中
    Exempted,
    /// 跳过：KYC未启用
    Skipped,
}

/// KYC验证失败原因
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, RuntimeDebug, MaxEncodedLen)]
pub enum KycFailureReason {
    /// 未设置身份信息
    IdentityNotSet,
    /// 没有有效的身份判断
    NoValidJudgement,
    /// 认证等级不足（不存储具体等级，以避免MaxEncodedLen问题）
    InsufficientLevel,
    /// 身份认证质量问题
    QualityIssue,
}

impl KycFailureReason {
    /// 转换为原因代码
    pub fn to_code(&self) -> u8 {
        match self {
            KycFailureReason::IdentityNotSet => 0,
            KycFailureReason::NoValidJudgement => 1,
            KycFailureReason::InsufficientLevel => 2,
            KycFailureReason::QualityIssue => 3,
        }
    }
}