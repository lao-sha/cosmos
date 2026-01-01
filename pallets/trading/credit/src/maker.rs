//! # Maker Credit Module (做市商信用模块)
//!
//! ## 函数级详细中文注释：做市商信用风控管理
//!
//! ### 核心功能
//! - 信用评分体系（800-1000分）
//! - 履约率追踪
//! - 违约惩罚
//! - 动态保证金
//! - 服务质量评价
//! - 自动降级/禁用

use codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use frame_support::pallet_prelude::*;

// ===== 数据结构 =====

/// 函数级详细中文注释：服务状态枚举
#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ServiceStatus {
    /// 正常服务
    Active,
    /// 警告状态（750-799分）
    Warning,
    /// 暂停服务（< 750分）
    Suspended,
}

impl Default for ServiceStatus {
    fn default() -> Self {
        ServiceStatus::Active
    }
}

/// 函数级详细中文注释：违约类型枚举
#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum DefaultType {
    /// 超时未释放
    Timeout,
    /// 恶意取消
    Cancellation,
    /// 争议败诉
    DisputeLoss,
    /// 保证金不足
    InsufficientFund,
}

/// 函数级详细中文注释：信用等级枚举
#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum CreditLevel {
    /// 钻石（950-1000分）
    Diamond,
    /// 白金（900-949分）
    Platinum,
    /// 黄金（850-899分）
    Gold,
    /// 白银（820-849分）
    Silver,
    /// 青铜（800-819分）
    Bronze,
}

impl Default for CreditLevel {
    fn default() -> Self {
        Self::Bronze
    }
}

impl CreditLevel {
    /// 函数级详细中文注释：根据信用分确定等级
    pub fn from_credit_score(score: u16) -> Self {
        match score {
            950..=1000 => CreditLevel::Diamond,
            900..=949 => CreditLevel::Platinum,
            850..=899 => CreditLevel::Gold,
            820..=849 => CreditLevel::Silver,
            _ => CreditLevel::Bronze,
        }
    }
    
    /// 函数级详细中文注释：获取等级对应的保证金折扣系数（百分比）
    pub fn get_deposit_discount(&self) -> u8 {
        match self {
            CreditLevel::Diamond => 50,   // 0.5x（减50%）
            CreditLevel::Platinum => 70,  // 0.7x（减30%）
            CreditLevel::Gold => 80,      // 0.8x（减20%）
            CreditLevel::Silver => 90,    // 0.9x（减10%）
            CreditLevel::Bronze => 100,   // 1.0x（无折扣）
        }
    }
}

/// 函数级详细中文注释：信用记录结构
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(BlockNumber))]
pub struct CreditRecord<BlockNumber> {
    /// 当前信用分（800-1000）
    pub credit_score: u16,
    /// 信用等级
    pub level: CreditLevel,
    /// 服务状态
    pub status: ServiceStatus,

    // === 履约数据 ===
    /// 总订单数
    pub total_orders: u32,
    /// 完成订单数
    pub completed_orders: u32,
    /// 超时订单数
    pub timeout_orders: u32,
    /// 取消订单数（做市商责任）
    pub cancelled_orders: u32,
    /// 及时释放订单数（< 24h）
    pub timely_release_orders: u32,

    // === 服务质量 ===
    /// 买家评分总和
    pub rating_sum: u32,
    /// 评分次数
    pub rating_count: u32,
    /// 平均响应时间（秒）
    pub avg_response_time: u32,

    // === 违约记录 ===
    /// 违约次数
    pub default_count: u16,
    /// 争议失败次数
    pub dispute_loss_count: u16,
    /// 最后一次违约区块
    pub last_default_block: Option<BlockNumber>,

    // === 活跃度 ===
    /// 最后一次订单区块
    pub last_order_block: BlockNumber,
    /// 连续服务天数
    pub consecutive_days: u16,
}

/// 函数级详细中文注释：评价标签枚举
#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RatingTag {
    FastRelease,       // 快速释放
    GoodCommunication, // 沟通良好
    FairPrice,         // 价格合理
    SlowRelease,       // 释放慢
    PoorCommunication, // 沟通差
    Unresponsive,      // 不回应
}

/// 函数级详细中文注释：评价记录结构
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(AccountId))]
pub struct Rating<AccountId> {
    /// 买家账户
    pub buyer: AccountId,
    /// 评分（1-5星）
    pub stars: u8,
    /// 评价标签代码（最多5个）
    /// 标签代码: 0=FastRelease, 1=GoodCommunication, 2=FairPrice,
    ///          3=SlowRelease, 4=PoorCommunication, 5=Unresponsive
    pub tags_codes: BoundedVec<u8, ConstU32<5>>,
    /// 评价时间（区块号）
    pub rated_at: u32,
}

/// 函数级详细中文注释：违约记录结构
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct DefaultRecord<BlockNumber> {
    /// 违约类型
    pub default_type: DefaultType,
    /// 违约区块
    pub block: BlockNumber,
    /// 惩罚分数
    pub penalty_score: u16,
    /// 是否已恢复
    pub recovered: bool,
}
