//! # Buyer Quota Module (买家额度管理模块)
//!
//! ## 函数级详细中文注释：强化信用额度系统（方案C+实现）
//!
//! ### 核心功能
//! - 渐进式额度计算（基于信用分和订单历史）
//! - 首购限制机制（10 USD起步）
//! - 并发订单数量控制
//! - 额度占用/释放管理
//! - 违约惩罚和额度减少
//! - 信用恢复和额度提升
//!
//! ### 设计理念
//! 完全替代押金机制，通过信用额度控制买家行为，解决DUST押金的逻辑矛盾。

use codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

// ===== 数据结构 =====

/// 函数级详细中文注释：买家信用额度配置
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct BuyerQuotaProfile<T: frame_system::Config> {
    /// 信用分（500-1000）
    pub credit_score: u16,

    /// 总完成订单数
    pub total_orders: u32,

    /// 当前可用额度（USD，精度10^6）
    pub available_quota: u64,

    /// 最大额度上限（根据信用分计算，USD，精度10^6）
    pub max_quota: u64,

    /// 已占用额度（进行中的订单，USD，精度10^6）
    pub occupied_quota: u64,

    /// 当前并发订单数
    pub active_orders: u32,

    /// 最大并发订单数（根据订单历史计算）
    pub max_concurrent_orders: u32,

    /// 上次违约时间
    pub last_violation_at: frame_system::pallet_prelude::BlockNumberFor<T>,

    /// 连续无违约订单数
    pub consecutive_good_orders: u32,

    /// 总违约次数
    pub total_violations: u32,

    /// 警告次数
    pub warnings: u32,

    /// 是否被暂停服务
    pub is_suspended: bool,

    /// 暂停解除时间（如果被暂停）
    pub suspension_until: Option<frame_system::pallet_prelude::BlockNumberFor<T>>,

    /// 是否被永久拉黑
    pub is_blacklisted: bool,
}

impl<T: frame_system::Config> Default for BuyerQuotaProfile<T> {
    fn default() -> Self {
        Self {
            credit_score: 500,  // 新用户初始分500
            total_orders: 0,
            available_quota: 10_000_000,  // 首购10 USD
            max_quota: 10_000_000,
            occupied_quota: 0,
            active_orders: 0,
            max_concurrent_orders: 1,  // 首购仅1笔并发
            last_violation_at: 0u32.into(),
            consecutive_good_orders: 0,
            total_violations: 0,
            warnings: 0,
            is_suspended: false,
            suspension_until: None,
            is_blacklisted: false,
        }
    }
}

/// 函数级详细中文注释：违约类型枚举
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ViolationType {
    /// 订单超时未付款
    OrderTimeout {
        order_id: u64,
        timeout_minutes: u32,
    },
    /// 争议败诉
    DisputeLoss {
        dispute_id: u64,
        loss_amount_usd: u64,
    },
    /// 恶意行为（多次违约）
    MaliciousBehavior {
        violation_count: u32,
    },
}

/// 函数级详细中文注释：违约惩罚记录
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct ViolationRecord<T: frame_system::Config> {
    /// 违约类型
    pub violation_type: ViolationType,

    /// 违约时间
    pub occurred_at: frame_system::pallet_prelude::BlockNumberFor<T>,

    /// 信用分扣除
    pub score_penalty: u16,

    /// 额度减少百分比（bps）
    pub quota_reduction_bps: u16,

    /// 惩罚持续天数
    pub penalty_duration_days: u32,

    /// 是否导致暂停
    pub caused_suspension: bool,
}

// ===== 额度计算函数 =====

/// 函数级详细中文注释：根据信用分和订单历史计算最大额度
///
/// # 参数
/// - `credit_score`: 信用分（500-1000）
/// - `total_orders`: 总完成订单数
///
/// # 返回
/// 最大额度（USD，精度10^6）
pub fn calculate_max_quota(credit_score: u16, total_orders: u32) -> u64 {
    // 基础额度（根据信用分）
    let base_quota: u64 = match credit_score {
        900..=1000 => 5000_000_000,  // 5000 USD
        800..=899  => 2000_000_000,  // 2000 USD
        700..=799  => 1000_000_000,  // 1000 USD
        600..=699  =>  500_000_000,  // 500 USD
        500..=599  =>  200_000_000,  // 200 USD
        _          =>  100_000_000,  // 100 USD（低信用）
    };

    // 新用户首购限制
    if total_orders == 0 {
        return 10_000_000; // 首购仅10 USD
    }

    // 根据订单历史动态调整（每10单增加50 USD）
    let history_boost = (total_orders / 10) as u64 * 50_000_000;

    // 计算最终额度，上限10000 USD
    base_quota
        .saturating_add(history_boost)
        .min(10000_000_000)
}

/// 函数级详细中文注释：计算最大并发订单数
///
/// # 参数
/// - `total_orders`: 总完成订单数
///
/// # 返回
/// 最大并发订单数
pub fn calculate_max_concurrent(total_orders: u32) -> u32 {
    match total_orders {
        0..=2   => 1,  // 前3单：仅1笔并发
        3..=9   => 2,  // 3-9单：2笔并发
        10..=49 => 3,  // 10-49单：3笔并发
        _       => 5,  // 50单以上：5笔并发
    }
}

/// 函数级详细中文注释：计算违约惩罚参数
///
/// # 参数
/// - `violation_type`: 违约类型
/// - `total_violations`: 历史违约次数
///
/// # 返回
/// (信用分扣除, 额度减少百分比bps, 惩罚持续天数, 是否暂停)
pub fn calculate_violation_penalty(
    violation_type: &ViolationType,
    total_violations: u32,
) -> (u16, u16, u32, bool) {
    match violation_type {
        ViolationType::OrderTimeout { .. } => {
            // 订单超时：-20分，额度减半7天
            let score_penalty = 20;
            let quota_reduction_bps = 5000;  // 50%
            let duration_days = 7;
            let suspend = total_violations >= 3;  // 3次超时暂停服务

            (score_penalty, quota_reduction_bps, duration_days, suspend)
        },

        ViolationType::DisputeLoss { .. } => {
            // 争议败诉：-50分，暂停30天
            let score_penalty = 50;
            let quota_reduction_bps = 10000;  // 100%（暂停期间）
            let duration_days = 30;
            let suspend = true;

            (score_penalty, quota_reduction_bps, duration_days, suspend)
        },

        ViolationType::MaliciousBehavior { violation_count } => {
            // 恶意行为：根据次数递增
            if *violation_count >= 3 {
                // 3次以上：永久拉黑
                (100, 10000, u32::MAX, true)
            } else {
                // 1-2次：严厉警告
                (30, 7000, 14, true)
            }
        },
    }
}

/// 函数级详细中文注释：检查是否可以恢复信用分
///
/// # 参数
/// - `profile`: 买家额度配置
/// - `current_block`: 当前区块号
/// - `blocks_per_day`: 每日区块数
///
/// # 返回
/// (是否可恢复, 恢复分数)
pub fn can_recover_credit<T: frame_system::Config>(
    profile: &BuyerQuotaProfile<T>,
    current_block: frame_system::pallet_prelude::BlockNumberFor<T>,
    blocks_per_day: frame_system::pallet_prelude::BlockNumberFor<T>,
) -> (bool, u16) {
    use sp_runtime::traits::{CheckedDiv, CheckedSub, UniqueSaturatedInto};

    // 黑名单用户不可恢复
    if profile.is_blacklisted {
        return (false, 0);
    }

    // 计算距离上次违约的天数
    let blocks_since_violation = current_block
        .checked_sub(&profile.last_violation_at)
        .unwrap_or_else(|| 0u32.into());

    let days_since_violation = blocks_since_violation
        .checked_div(&blocks_per_day)
        .unwrap_or_else(|| 0u32.into());

    // 转换为u32进行比较
    let days_u32: u32 = days_since_violation.unique_saturated_into();

    // 恢复条件：30天内无违约
    if days_u32 >= 30 {
        return (true, 10);  // 每30天恢复10分
    }

    // 奖励机制：连续10单无问题
    if profile.consecutive_good_orders >= 10 {
        return (true, 5);  // 连续10单奖励5分
    }

    (false, 0)
}

// ===== 额度管理接口 Trait =====

/// 函数级详细中文注释：买家额度管理接口（供其他Pallet调用）
///
/// 此trait提供了OTC订单等模块所需的额度管理功能
pub trait BuyerQuotaInterface<AccountId> {
    /// 获取可用额度
    fn get_available_quota(buyer: &AccountId) -> Result<u64, sp_runtime::DispatchError>;

    /// 占用额度（创建订单时）
    fn occupy_quota(buyer: &AccountId, amount_usd: u64) -> sp_runtime::DispatchResult;

    /// 释放额度（订单完成/取消时）
    fn release_quota(buyer: &AccountId, amount_usd: u64) -> sp_runtime::DispatchResult;

    /// 检查并发订单数是否超限
    fn check_concurrent_limit(buyer: &AccountId) -> Result<bool, sp_runtime::DispatchError>;

    /// 记录订单完成（提升信用）
    fn record_order_completed(buyer: &AccountId, order_id: u64) -> sp_runtime::DispatchResult;

    /// 记录订单取消（降低信用）
    fn record_order_cancelled(buyer: &AccountId, order_id: u64) -> sp_runtime::DispatchResult;

    /// 记录违约行为（降低信用+减少额度）
    fn record_violation(
        buyer: &AccountId,
        violation_type: ViolationType,
    ) -> sp_runtime::DispatchResult;

    /// 检查是否被暂停服务
    fn is_suspended(buyer: &AccountId) -> Result<bool, sp_runtime::DispatchError>;

    /// 检查是否被拉黑
    fn is_blacklisted(buyer: &AccountId) -> Result<bool, sp_runtime::DispatchError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_max_quota() {
        // 新用户首购限制
        assert_eq!(calculate_max_quota(800, 0), 10_000_000); // 10 USD

        // 完成3单后
        assert_eq!(calculate_max_quota(800, 3), 2000_000_000); // 2000 USD

        // 高信用用户
        assert_eq!(calculate_max_quota(950, 50), 5250_000_000); // 5000 + 250 = 5250 USD

        // 低信用用户
        assert_eq!(calculate_max_quota(450, 10), 100_000_000); // 100 USD
    }

    #[test]
    fn test_calculate_max_concurrent() {
        assert_eq!(calculate_max_concurrent(0), 1);   // 首购：1笔
        assert_eq!(calculate_max_concurrent(5), 2);   // 5单：2笔
        assert_eq!(calculate_max_concurrent(20), 3);  // 20单：3笔
        assert_eq!(calculate_max_concurrent(100), 5); // 100单：5笔
    }

    #[test]
    fn test_calculate_violation_penalty() {
        // 首次订单超时
        let (score, quota_bps, days, suspend) = calculate_violation_penalty(
            &ViolationType::OrderTimeout { order_id: 1, timeout_minutes: 120 },
            0,
        );
        assert_eq!(score, 20);
        assert_eq!(quota_bps, 5000);  // 50%
        assert_eq!(days, 7);
        assert_eq!(suspend, false);

        // 第3次超时
        let (_, _, _, suspend) = calculate_violation_penalty(
            &ViolationType::OrderTimeout { order_id: 3, timeout_minutes: 120 },
            3,
        );
        assert_eq!(suspend, true);  // 暂停服务

        // 争议败诉
        let (score, quota_bps, days, suspend) = calculate_violation_penalty(
            &ViolationType::DisputeLoss { dispute_id: 1, loss_amount_usd: 100_000_000 },
            0,
        );
        assert_eq!(score, 50);
        assert_eq!(quota_bps, 10000);  // 100%
        assert_eq!(days, 30);
        assert_eq!(suspend, true);

        // 恶意行为（3次以上）
        let (score, quota_bps, days, suspend) = calculate_violation_penalty(
            &ViolationType::MaliciousBehavior { violation_count: 3 },
            0,
        );
        assert_eq!(score, 100);
        assert_eq!(quota_bps, 10000);
        assert_eq!(days, u32::MAX);  // 永久
        assert_eq!(suspend, true);
    }
}
