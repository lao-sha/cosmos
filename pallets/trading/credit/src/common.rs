//! # Common Module (公共模块)
//!
//! ## 函数级详细中文注释：信用系统公共工具
//!
//! ### 功能
//! - 信用分计算工具
//! - 风险评估函数
//! - 数据验证和校验

use sp_std::cmp::{min, max};

// ===== 买家信用相关函数 =====

/// 函数级详细中文注释：计算买家信用等级
pub fn calculate_buyer_level(completed_orders: u32) -> crate::buyer::CreditLevel {
    crate::buyer::CreditLevel::from_completed_orders(completed_orders)
}

/// 函数级详细中文注释：计算新用户层级
pub fn calculate_new_user_tier(risk_score: u16) -> crate::buyer::NewUserTier {
    crate::buyer::NewUserTier::from_risk_score(risk_score)
}

// ===== 做市商信用相关函数 =====

/// 函数级详细中文注释：计算做市商信用等级
pub fn calculate_maker_level(credit_score: u16) -> crate::maker::CreditLevel {
    crate::maker::CreditLevel::from_credit_score(credit_score)
}

/// 函数级详细中文注释：计算做市商服务状态
pub fn calculate_maker_status(credit_score: u16) -> crate::maker::ServiceStatus {
    if credit_score < 750 {
        crate::maker::ServiceStatus::Suspended
    } else if credit_score < 800 {
        crate::maker::ServiceStatus::Warning
    } else {
        crate::maker::ServiceStatus::Active
    }
}

/// 函数级详细中文注释：计算做市商动态保证金折扣
pub fn calculate_deposit_discount(credit_score: u16) -> u8 {
    let level = calculate_maker_level(credit_score);
    level.get_deposit_discount()
}

// ===== 通用工具函数 =====

/// 函数级详细中文注释：安全的信用分加法（防止溢出）
pub fn safe_credit_add(current: u16, delta: u16, max_value: u16) -> u16 {
    min(current.saturating_add(delta), max_value)
}

/// 函数级详细中文注释：安全的信用分减法（防止下溢）
pub fn safe_credit_sub(current: u16, delta: u16, min_value: u16) -> u16 {
    max(current.saturating_sub(delta), min_value)
}

/// 函数级详细中文注释：计算加权平均值
pub fn calculate_weighted_average(sum: u32, count: u32) -> u32 {
    if count == 0 {
        0
    } else {
        sum / count
    }
}

/// 函数级详细中文注释：判断信用分是否需要警告
pub fn is_credit_warning(score: u16, warning_threshold: u16) -> bool {
    score <= warning_threshold
}

/// 函数级详细中文注释：判断服务是否应该暂停
pub fn should_suspend_service(score: u16, suspension_threshold: u16) -> bool {
    score < suspension_threshold
}

/// 函数级详细中文注释：计算履约率（百分比）
pub fn calculate_completion_rate(completed: u32, total: u32) -> u8 {
    if total == 0 {
        100 // 没有订单时默认100%
    } else {
        let rate = (completed as u64 * 100) / (total as u64);
        min(rate, 100) as u8
    }
}

/// 函数级详细中文注释：计算违约率（百分比）
pub fn calculate_default_rate(defaulted: u32, total: u32) -> u8 {
    if total == 0 {
        0
    } else {
        let rate = (defaulted as u64 * 100) / (total as u64);
        min(rate, 100) as u8
    }
}
