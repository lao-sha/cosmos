//! # 存储押金模块
//!
//! 本模块提供所有玄学系统通用的存储押金计算和管理功能。
//!
//! ## 概述
//!
//! 存储押金机制用于：
//! - 防止存储滥用
//! - 激励用户及时清理不需要的数据
//! - 确保链上存储的可持续性
//!
//! ## 计算公式
//!
//! ```text
//! 存储押金 = 基础费率 × 数据大小系数 × 隐私模式系数
//! ```
//!
//! ## 返还规则
//!
//! | 删除时机 | 返还比例 |
//! |---------|---------|
//! | 30天内删除 | 100% |
//! | 30天后删除 | 90% |
//!
//! 扣除的 10% 进入国库。

use codec::{Decode, Encode, MaxEncodedLen};
use core::ops::Div;
use scale_info::TypeInfo;
use sp_runtime::traits::{Saturating, Zero};

/// 隐私模式
///
/// 不同隐私模式的数据存储成本不同
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum PrivacyMode {
    /// 公开模式 - 所有数据明文存储
    #[default]
    Public = 0,
    /// 部分加密 - 计算数据明文，敏感数据加密
    Partial = 1,
    /// 完全加密 - 所有数据加密存储
    Private = 2,
}

impl PrivacyMode {
    /// 获取隐私模式名称
    pub fn name(&self) -> &'static str {
        match self {
            PrivacyMode::Public => "公开",
            PrivacyMode::Partial => "部分加密",
            PrivacyMode::Private => "完全加密",
        }
    }

    /// 获取隐私模式系数（百分比，100 = 1.0x）
    ///
    /// - Public: 1.0x (100)
    /// - Partial: 1.2x (120)
    /// - Private: 1.5x (150)
    pub fn multiplier(&self) -> u32 {
        match self {
            PrivacyMode::Public => 100,
            PrivacyMode::Partial => 120,
            PrivacyMode::Private => 150,
        }
    }

    /// 从 u8 转换
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(PrivacyMode::Public),
            1 => Some(PrivacyMode::Partial),
            2 => Some(PrivacyMode::Private),
            _ => None,
        }
    }
}

/// 押金配置
///
/// 定义存储押金的基础参数
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct DepositConfig<Balance> {
    /// 每 KB 基础费率
    pub base_rate_per_kb: Balance,
    /// 最小押金
    pub minimum_deposit: Balance,
    /// 最大押金
    pub maximum_deposit: Balance,
}

impl<Balance: Clone + Saturating + PartialOrd + From<u32>> DepositConfig<Balance> {
    /// 创建测试配置
    ///
    /// 使用较小的测试值，实际生产值应在 Runtime 中配置。
    ///
    /// 测试值：
    /// - 基础费率: 100 (0.0000001 DUST per KB)
    /// - 最小押金: 10 (0.00000001 DUST)
    /// - 最大押金: 100_000_000 (0.1 DUST)
    ///
    /// 生产建议值（需要在 Runtime 中使用 u128）：
    /// - 基础费率: 10_000_000_000 (0.01 DUST per KB)
    /// - 最小押金: 1_000_000_000 (0.001 DUST)
    /// - 最大押金: 100_000_000_000_000 (100 DUST)
    pub fn test_config() -> Self {
        Self {
            base_rate_per_kb: Balance::from(100u32),
            minimum_deposit: Balance::from(10u32),
            maximum_deposit: Balance::from(100_000_000u32),
        }
    }

    /// 创建自定义配置
    ///
    /// # 参数
    /// - `base_rate`: 每 KB 基础费率
    /// - `min_deposit`: 最小押金
    /// - `max_deposit`: 最大押金
    pub fn new(base_rate: Balance, min_deposit: Balance, max_deposit: Balance) -> Self {
        Self {
            base_rate_per_kb: base_rate,
            minimum_deposit: min_deposit,
            maximum_deposit: max_deposit,
        }
    }
}

/// 押金记录
///
/// 记录用户为特定数据支付的押金信息
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct DepositRecord<Balance, BlockNumber> {
    /// 押金金额
    pub amount: Balance,
    /// 创建区块号
    pub created_at: BlockNumber,
    /// 数据大小（字节）
    pub data_size: u32,
    /// 隐私模式
    pub privacy_mode: PrivacyMode,
}

/// 30天的区块数量（假设 6 秒一个区块）
///
/// 30 天 = 30 × 24 × 60 × 60 / 6 = 432_000 区块
pub const BLOCKS_PER_30_DAYS: u32 = 432_000;

/// 计算存储押金
///
/// # 公式
///
/// ```text
/// 押金 = 基础费率 × ceil(数据大小 / 1024) × 隐私模式系数 / 100
/// ```
///
/// # 参数
///
/// - `data_size_bytes`: 数据大小（字节）
/// - `privacy_mode`: 隐私模式
/// - `config`: 押金配置
///
/// # 返回
///
/// 计算后的押金金额（已限制在最小/最大范围内）
///
/// # 示例
///
/// ```ignore
/// let config = DepositConfig::default_config();
/// let deposit = calculate_storage_deposit::<u128>(
///     2048,  // 2 KB
///     PrivacyMode::Partial,
///     &config,
/// );
/// // deposit = 0.01 × 2 × 1.2 = 0.024 DUST
/// ```
pub fn calculate_storage_deposit<Balance>(
    data_size_bytes: u32,
    privacy_mode: PrivacyMode,
    config: &DepositConfig<Balance>,
) -> Balance
where
    Balance: Clone + Saturating + PartialOrd + From<u32> + Copy + Div<Output = Balance>,
{
    // 计算 KB 数（向上取整）
    let size_kb = (data_size_bytes.saturating_add(1023)) / 1024;
    let size_kb = if size_kb == 0 { 1 } else { size_kb };

    // 获取隐私模式系数
    let multiplier = privacy_mode.multiplier();

    // 计算押金
    // deposit = base_rate × size_kb × multiplier / 100
    let deposit = config
        .base_rate_per_kb
        .saturating_mul(Balance::from(size_kb))
        .saturating_mul(Balance::from(multiplier))
        / Balance::from(100u32);

    // 限制在最小/最大范围内
    if deposit < config.minimum_deposit {
        config.minimum_deposit.clone()
    } else if deposit > config.maximum_deposit {
        config.maximum_deposit.clone()
    } else {
        deposit
    }
}

/// 计算返还金额
///
/// # 返还规则
///
/// | 删除时机 | 返还比例 |
/// |---------|---------|
/// | 30天内删除 | 100% |
/// | 30天后删除 | 90% |
///
/// # 参数
///
/// - `deposit_amount`: 原始押金金额
/// - `created_at`: 创建区块号
/// - `current_block`: 当前区块号
///
/// # 返回
///
/// (返还金额, 国库金额)
///
/// # 示例
///
/// ```ignore
/// let (refund, treasury) = calculate_refund_amount(
///     1_000_000_000_000u128,  // 1 DUST
///     100,                    // 创建于区块 100
///     500_000,                // 当前区块 500000（超过30天）
/// );
/// // refund = 0.9 DUST, treasury = 0.1 DUST
/// ```
pub fn calculate_refund_amount<Balance, BlockNumber>(
    deposit_amount: Balance,
    created_at: BlockNumber,
    current_block: BlockNumber,
) -> (Balance, Balance)
where
    Balance: Clone + Saturating + PartialOrd + From<u32> + Copy + Zero + Div<Output = Balance>,
    BlockNumber: Clone + Saturating + PartialOrd + From<u32> + Copy,
{
    // 计算存在时长（区块数）
    let duration = if current_block > created_at {
        current_block.saturating_sub(created_at)
    } else {
        BlockNumber::from(0u32)
    };

    let threshold = BlockNumber::from(BLOCKS_PER_30_DAYS);

    if duration < threshold {
        // 30天内删除：100% 返还
        (deposit_amount, Balance::zero())
    } else {
        // 30天后删除：90% 返还，10% 进入国库
        let treasury_amount = deposit_amount.saturating_mul(Balance::from(10u32))
            / Balance::from(100u32);
        let refund_amount = deposit_amount.saturating_sub(treasury_amount);
        (refund_amount, treasury_amount)
    }
}

/// 估算数据大小
///
/// 根据占卜类型和隐私模式估算存储数据大小
///
/// # 参数
///
/// - `divination_type`: 占卜类型索引
/// - `privacy_mode`: 隐私模式
///
/// # 返回
///
/// 估算的数据大小（字节）
pub fn estimate_data_size(divination_type: u8, privacy_mode: PrivacyMode) -> u32 {
    // 基础大小（根据占卜类型）
    let base_size = match divination_type {
        0 => 400,   // Meihua: 300-1000 bytes
        1 => 1500,  // Bazi: 500-2000 bytes
        2 => 800,   // Liuyao: 350-1300 bytes
        3 => 1000,  // Qimen: 400-1500 bytes
        4 => 2000,  // Ziwei: 800-3000 bytes
        5 => 1000,  // Taiyi: 预留
        6 => 800,   // Daliuren: 300-1200 bytes
        7 => 500,   // XiaoLiuRen: 200-800 bytes
        8 => 800,   // Tarot: 400-1200 bytes
        _ => 500,   // 默认
    };

    // 隐私模式会增加额外开销
    let overhead = match privacy_mode {
        PrivacyMode::Public => 0,
        PrivacyMode::Partial => 200,  // 加密数据额外开销
        PrivacyMode::Private => 400,  // 完全加密额外开销
    };

    base_size + overhead
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestBalance = u128;
    type TestBlockNumber = u32;

    fn test_config() -> DepositConfig<TestBalance> {
        DepositConfig {
            base_rate_per_kb: 10_000_000_000,     // 0.01 DUST per KB
            minimum_deposit: 1_000_000_000,       // 0.001 DUST
            maximum_deposit: 100_000_000_000_000, // 100 DUST
        }
    }

    #[test]
    fn test_privacy_mode_multiplier() {
        assert_eq!(PrivacyMode::Public.multiplier(), 100);
        assert_eq!(PrivacyMode::Partial.multiplier(), 120);
        assert_eq!(PrivacyMode::Private.multiplier(), 150);
    }

    #[test]
    fn test_privacy_mode_from_u8() {
        assert_eq!(PrivacyMode::from_u8(0), Some(PrivacyMode::Public));
        assert_eq!(PrivacyMode::from_u8(1), Some(PrivacyMode::Partial));
        assert_eq!(PrivacyMode::from_u8(2), Some(PrivacyMode::Private));
        assert_eq!(PrivacyMode::from_u8(3), None);
    }

    #[test]
    fn test_calculate_storage_deposit_public() {
        let config = test_config();

        // 1 KB, Public mode
        // deposit = 0.01 × 1 × 1.0 = 0.01 DUST
        let deposit = calculate_storage_deposit::<TestBalance>(
            1024,
            PrivacyMode::Public,
            &config,
        );
        assert_eq!(deposit, 10_000_000_000);

        // 2 KB, Public mode
        // deposit = 0.01 × 2 × 1.0 = 0.02 DUST
        let deposit = calculate_storage_deposit::<TestBalance>(
            2048,
            PrivacyMode::Public,
            &config,
        );
        assert_eq!(deposit, 20_000_000_000);
    }

    #[test]
    fn test_calculate_storage_deposit_partial() {
        let config = test_config();

        // 1 KB, Partial mode
        // deposit = 0.01 × 1 × 1.2 = 0.012 DUST
        let deposit = calculate_storage_deposit::<TestBalance>(
            1024,
            PrivacyMode::Partial,
            &config,
        );
        assert_eq!(deposit, 12_000_000_000);

        // 2 KB, Partial mode
        // deposit = 0.01 × 2 × 1.2 = 0.024 DUST
        let deposit = calculate_storage_deposit::<TestBalance>(
            2048,
            PrivacyMode::Partial,
            &config,
        );
        assert_eq!(deposit, 24_000_000_000);
    }

    #[test]
    fn test_calculate_storage_deposit_private() {
        let config = test_config();

        // 1 KB, Private mode
        // deposit = 0.01 × 1 × 1.5 = 0.015 DUST
        let deposit = calculate_storage_deposit::<TestBalance>(
            1024,
            PrivacyMode::Private,
            &config,
        );
        assert_eq!(deposit, 15_000_000_000);
    }

    #[test]
    fn test_calculate_storage_deposit_minimum() {
        let config = test_config();

        // 100 bytes, Public mode → should be minimum deposit
        let deposit = calculate_storage_deposit::<TestBalance>(
            100,
            PrivacyMode::Public,
            &config,
        );
        // 100 bytes → 1 KB (向上取整)
        // deposit = 0.01 × 1 × 1.0 = 0.01 DUST = 10_000_000_000
        // 大于最小押金 1_000_000_000
        assert_eq!(deposit, 10_000_000_000);
    }

    #[test]
    fn test_calculate_storage_deposit_size_ceiling() {
        let config = test_config();

        // 1025 bytes → 2 KB (向上取整)
        let deposit = calculate_storage_deposit::<TestBalance>(
            1025,
            PrivacyMode::Public,
            &config,
        );
        // deposit = 0.01 × 2 × 1.0 = 0.02 DUST
        assert_eq!(deposit, 20_000_000_000);
    }

    #[test]
    fn test_calculate_refund_within_30_days() {
        let deposit_amount: TestBalance = 1_000_000_000_000; // 1 DUST
        let created_at: TestBlockNumber = 100;
        let current_block: TestBlockNumber = 100 + BLOCKS_PER_30_DAYS - 1; // 刚好30天内

        let (refund, treasury) = calculate_refund_amount(
            deposit_amount,
            created_at,
            current_block,
        );

        // 30天内删除：100% 返还
        assert_eq!(refund, 1_000_000_000_000);
        assert_eq!(treasury, 0);
    }

    #[test]
    fn test_calculate_refund_after_30_days() {
        let deposit_amount: TestBalance = 1_000_000_000_000; // 1 DUST
        let created_at: TestBlockNumber = 100;
        let current_block: TestBlockNumber = 100 + BLOCKS_PER_30_DAYS + 1; // 超过30天

        let (refund, treasury) = calculate_refund_amount(
            deposit_amount,
            created_at,
            current_block,
        );

        // 30天后删除：90% 返还，10% 进入国库
        assert_eq!(refund, 900_000_000_000);
        assert_eq!(treasury, 100_000_000_000);
    }

    #[test]
    fn test_calculate_refund_edge_case_exactly_30_days() {
        let deposit_amount: TestBalance = 1_000_000_000_000; // 1 DUST
        let created_at: TestBlockNumber = 100;
        let current_block: TestBlockNumber = 100 + BLOCKS_PER_30_DAYS; // 刚好30天

        let (refund, treasury) = calculate_refund_amount(
            deposit_amount,
            created_at,
            current_block,
        );

        // 刚好30天：90% 返还（不包含30天内）
        assert_eq!(refund, 900_000_000_000);
        assert_eq!(treasury, 100_000_000_000);
    }

    #[test]
    fn test_estimate_data_size() {
        // 测试不同占卜类型的估算大小
        assert_eq!(estimate_data_size(0, PrivacyMode::Public), 400);   // Meihua
        assert_eq!(estimate_data_size(1, PrivacyMode::Public), 1500);  // Bazi
        assert_eq!(estimate_data_size(4, PrivacyMode::Public), 2000);  // Ziwei

        // 测试隐私模式额外开销
        assert_eq!(estimate_data_size(0, PrivacyMode::Partial), 600);  // Meihua + 200
        assert_eq!(estimate_data_size(0, PrivacyMode::Private), 800);  // Meihua + 400
    }
}
