//! # 玄学公共模块 Trait 定义
//!
//! 本模块定义了各玄学系统与公共服务模块之间的接口。
//!
//! ## 核心 Trait
//!
//! - `DivinationProvider` - 占卜结果提供者，用于 NFT 和 AI 模块查询占卜数据
//! - `InterpretationContextGenerator` - AI 解读上下文生成器
//! - `StorageDepositManager` - 存储押金管理器，处理押金锁定、释放和返还
//! - `Deletable` - 可删除数据 Trait，定义数据删除的权限检查和执行逻辑

use crate::deposit::PrivacyMode;
use crate::types::{DivinationType, InterpretationType, RarityInput};
use frame_support::dispatch::DispatchResult;
use sp_std::vec::Vec;

/// 占卜结果提供者 Trait
///
/// 各玄学系统（梅花、八字等）需要在 Runtime 中实现此 trait，
/// 以便公共服务模块（NFT、AI、Market）能够查询占卜结果数据。
///
/// # 实现说明
///
/// 在 Runtime 中为每个玄学系统实现此 trait，然后使用组合模式
/// 创建一个统一的 Provider 供公共模块使用。
///
/// # 示例
///
/// ```ignore
/// pub struct MeihuaDivinationProvider;
///
/// impl DivinationProvider<AccountId> for MeihuaDivinationProvider {
///     fn result_exists(divination_type: DivinationType, result_id: u64) -> bool {
///         match divination_type {
///             DivinationType::Meihua => Meihua::hexagrams(result_id).is_some(),
///             _ => false,
///         }
///     }
///     // ... 其他方法
/// }
/// ```
pub trait DivinationProvider<AccountId: PartialEq> {
    /// 检查占卜结果是否存在
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID（如卦象 ID、命盘 ID）
    ///
    /// # 返回
    /// 结果是否存在
    fn result_exists(divination_type: DivinationType, result_id: u64) -> bool;

    /// 获取占卜结果的创建者账户
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    ///
    /// # 返回
    /// 创建者账户，如果不存在则返回 None
    fn result_creator(divination_type: DivinationType, result_id: u64) -> Option<AccountId>;

    /// 获取稀有度计算数据
    ///
    /// 各玄学系统将自身结果的特征转换为 `RarityInput`，
    /// 由统一的算法计算 NFT 稀有度。
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    ///
    /// # 返回
    /// 稀有度计算输入数据
    fn rarity_data(divination_type: DivinationType, result_id: u64) -> Option<RarityInput>;

    /// 获取占卜结果摘要
    ///
    /// 返回结果的序列化摘要，用于 AI 解读的输入上下文。
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    ///
    /// # 返回
    /// 结果摘要的字节序列（通常为 JSON 或 SCALE 编码）
    fn result_summary(divination_type: DivinationType, result_id: u64) -> Option<Vec<u8>>;

    /// 检查占卜结果是否可以铸造为 NFT
    ///
    /// 通常需要检查：
    /// 1. 结果存在
    /// 2. 未被铸造过
    /// 3. 满足铸造条件（如状态为活跃）
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    ///
    /// # 返回
    /// 是否可以铸造为 NFT
    fn is_nftable(divination_type: DivinationType, result_id: u64) -> bool;

    /// 标记占卜结果已被铸造为 NFT
    ///
    /// NFT 模块在成功铸造后调用此方法，
    /// 各玄学系统应记录该结果已被 NFT 化。
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    fn mark_as_nfted(divination_type: DivinationType, result_id: u64);

    /// 获取占卜结果的创建时间（区块号）
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    ///
    /// # 返回
    /// 创建时的区块号
    fn result_created_at(divination_type: DivinationType, result_id: u64) -> Option<u32> {
        // 默认实现返回 None，各系统可覆盖
        let _ = (divination_type, result_id);
        None
    }

    /// 检查账户是否拥有该占卜结果
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    /// - `account`: 账户
    ///
    /// # 返回
    /// 是否为所有者
    fn is_owner(
        divination_type: DivinationType,
        result_id: u64,
        account: &AccountId,
    ) -> bool {
        Self::result_creator(divination_type, result_id)
            .map(|creator| &creator == account)
            .unwrap_or(false)
    }
}

/// 空实现的占卜结果提供者
///
/// 用于测试或默认配置。
pub struct NullDivinationProvider;

impl<AccountId: PartialEq> DivinationProvider<AccountId> for NullDivinationProvider {
    fn result_exists(_: DivinationType, _: u64) -> bool {
        false
    }

    fn result_creator(_: DivinationType, _: u64) -> Option<AccountId> {
        None
    }

    fn rarity_data(_: DivinationType, _: u64) -> Option<RarityInput> {
        None
    }

    fn result_summary(_: DivinationType, _: u64) -> Option<Vec<u8>> {
        None
    }

    fn is_nftable(_: DivinationType, _: u64) -> bool {
        false
    }

    fn mark_as_nfted(_: DivinationType, _: u64) {}
}

/// AI 解读上下文生成器 Trait
///
/// 各玄学系统实现此 trait 以生成专用的 AI 解读上下文。
/// 不同的占卜类型需要不同的上下文格式以获得最佳解读效果。
///
/// # 示例
///
/// ```ignore
/// pub struct MeihuaContextGenerator;
///
/// impl InterpretationContextGenerator for MeihuaContextGenerator {
///     fn generate_context(
///         divination_type: DivinationType,
///         result_id: u64,
///         interpretation_type: InterpretationType,
///     ) -> Option<Vec<u8>> {
///         if divination_type != DivinationType::Meihua {
///             return None;
///         }
///
///         let hexagram = Meihua::hexagrams(result_id)?;
///         let context = serde_json::json!({
///             "system": "meihua",
///             "ben_gua": format_hexagram(&hexagram.ben_gua),
///             "bian_gua": format_hexagram(&hexagram.bian_gua),
///             "dong_yao": hexagram.dong_yao,
///             "interpretation_type": interpretation_type.name(),
///         });
///         Some(context.to_string().into_bytes())
///     }
/// }
/// ```
pub trait InterpretationContextGenerator {
    /// 生成 AI 解读的上下文
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    /// - `interpretation_type`: 解读类型
    ///
    /// # 返回
    /// 上下文 JSON 字符串的字节序列
    fn generate_context(
        divination_type: DivinationType,
        result_id: u64,
        interpretation_type: InterpretationType,
    ) -> Option<Vec<u8>>;

    /// 获取该占卜类型支持的解读类型列表
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    ///
    /// # 返回
    /// 支持的解读类型列表
    fn supported_interpretation_types(divination_type: DivinationType) -> Vec<InterpretationType> {
        match divination_type {
            DivinationType::Meihua | DivinationType::Bazi => {
                sp_std::vec![
                    InterpretationType::Basic,
                    InterpretationType::Detailed,
                    InterpretationType::Professional,
                    InterpretationType::Career,
                    InterpretationType::Relationship,
                    InterpretationType::Health,
                    InterpretationType::Wealth,
                    InterpretationType::Education,
                    InterpretationType::Annual,
                ]
            }
            _ => {
                sp_std::vec![
                    InterpretationType::Basic,
                    InterpretationType::Detailed,
                ]
            }
        }
    }

    /// 获取解读所需的预估 token 数量
    ///
    /// 用于预估 AI 调用成本。
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `interpretation_type`: 解读类型
    ///
    /// # 返回
    /// 预估的输出 token 数量
    fn estimated_tokens(
        divination_type: DivinationType,
        interpretation_type: InterpretationType,
    ) -> u32 {
        let _ = divination_type;
        match interpretation_type {
            InterpretationType::Basic => 500,
            InterpretationType::Detailed => 1500,
            InterpretationType::Professional => 3000,
            _ => 1000,
        }
    }
}

/// 空实现的上下文生成器
pub struct NullContextGenerator;

impl InterpretationContextGenerator for NullContextGenerator {
    fn generate_context(
        _: DivinationType,
        _: u64,
        _: InterpretationType,
    ) -> Option<Vec<u8>> {
        None
    }
}

/// NFT 元数据生成器 Trait
///
/// 用于生成 NFT 的元数据（符合 ERC721 Metadata 标准）。
pub trait NftMetadataGenerator {
    /// 生成 NFT 元数据 JSON
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    /// - `name`: NFT 名称
    ///
    /// # 返回
    /// 元数据 JSON 字符串的字节序列
    fn generate_metadata(
        divination_type: DivinationType,
        result_id: u64,
        name: &[u8],
    ) -> Option<Vec<u8>>;

    /// 生成 NFT 图片描述
    ///
    /// 用于 AI 图片生成的描述文本。
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    ///
    /// # 返回
    /// 图片生成描述
    fn generate_image_prompt(
        divination_type: DivinationType,
        result_id: u64,
    ) -> Option<Vec<u8>>;
}

/// 空实现的元数据生成器
pub struct NullMetadataGenerator;

impl NftMetadataGenerator for NullMetadataGenerator {
    fn generate_metadata(_: DivinationType, _: u64, _: &[u8]) -> Option<Vec<u8>> {
        None
    }

    fn generate_image_prompt(_: DivinationType, _: u64) -> Option<Vec<u8>> {
        None
    }
}

// ================================
// 存储押金管理 Trait
// ================================

/// 存储押金管理器 Trait
///
/// 定义了存储押金的计算、锁定、释放和返还逻辑。
/// 各 pallet 通过实现此 trait 来管理用户存储押金。
///
/// # 设计原则
///
/// 1. **公平性**: 押金与存储空间成正比
/// 2. **激励性**: 早删除返还比例更高
/// 3. **可持续性**: 押金覆盖长期存储成本
///
/// # 返还规则
///
/// | 删除时机 | 返还比例 |
/// |---------|---------|
/// | 30天内删除 | 100% |
/// | 30天后删除 | 90% |
///
/// # 使用示例
///
/// ```ignore
/// // 创建数据时锁定押金
/// let deposit = T::DepositManager::calculate_deposit(data_size, PrivacyMode::Partial);
/// T::DepositManager::reserve_deposit(&who, deposit)?;
///
/// // 删除数据时返还押金
/// let refund = T::DepositManager::unreserve_deposit(&who, deposit, duration);
/// ```
pub trait StorageDepositManager<AccountId, Balance, BlockNumber> {
    /// 计算存储押金
    ///
    /// # 公式
    /// ```text
    /// 押金 = 基础费率 × ceil(数据大小 / 1024) × 隐私模式系数 / 100
    /// ```
    ///
    /// # 参数
    /// - `data_size`: 数据大小（字节）
    /// - `privacy_mode`: 隐私模式（影响押金系数）
    ///
    /// # 返回
    /// 计算后的押金金额（已限制在最小/最大范围内）
    fn calculate_deposit(data_size: u32, privacy_mode: PrivacyMode) -> Balance;

    /// 锁定存储押金
    ///
    /// 从用户的可用余额中锁定指定金额作为存储押金。
    /// 使用 Substrate 的 `reserve` 机制实现。
    ///
    /// # 参数
    /// - `who`: 用户账户
    /// - `amount`: 押金金额
    ///
    /// # 返回
    /// - `Ok(())`: 锁定成功
    /// - `Err`: 余额不足或其他错误
    fn reserve_deposit(who: &AccountId, amount: Balance) -> DispatchResult;

    /// 释放存储押金并返还
    ///
    /// 根据存储时长计算返还比例，将押金返还给用户。
    /// 超过 30 天的部分会扣除 10% 作为存储成本进入国库。
    ///
    /// # 参数
    /// - `who`: 用户账户
    /// - `amount`: 原始押金金额
    /// - `created_at`: 数据创建时间（区块号）
    /// - `current_block`: 当前区块号
    ///
    /// # 返回
    /// 实际返还给用户的金额
    fn unreserve_deposit(
        who: &AccountId,
        amount: Balance,
        created_at: BlockNumber,
        current_block: BlockNumber,
    ) -> Balance;

    /// 获取用户已锁定的押金总额
    ///
    /// # 参数
    /// - `who`: 用户账户
    ///
    /// # 返回
    /// 该用户当前被锁定的押金总额
    fn total_reserved(who: &AccountId) -> Balance;

    /// 估算特定占卜类型和隐私模式的押金
    ///
    /// 用于前端预估费用显示。
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型索引
    /// - `privacy_mode`: 隐私模式
    ///
    /// # 返回
    /// 估算的押金金额
    fn estimate_deposit(divination_type: u8, privacy_mode: PrivacyMode) -> Balance {
        let data_size = crate::deposit::estimate_data_size(divination_type, privacy_mode);
        Self::calculate_deposit(data_size, privacy_mode)
    }
}

/// 空实现的存储押金管理器
///
/// 用于测试或不需要押金机制的场景。
pub struct NullStorageDepositManager;

impl<AccountId, Balance: Default, BlockNumber> StorageDepositManager<AccountId, Balance, BlockNumber>
    for NullStorageDepositManager
{
    fn calculate_deposit(_data_size: u32, _privacy_mode: PrivacyMode) -> Balance {
        Balance::default()
    }

    fn reserve_deposit(_who: &AccountId, _amount: Balance) -> DispatchResult {
        Ok(())
    }

    fn unreserve_deposit(
        _who: &AccountId,
        _amount: Balance,
        _created_at: BlockNumber,
        _current_block: BlockNumber,
    ) -> Balance {
        Balance::default()
    }

    fn total_reserved(_who: &AccountId) -> Balance {
        Balance::default()
    }
}

// ================================
// 可删除数据 Trait
// ================================

/// 可删除数据 Trait
///
/// 定义了数据删除的权限检查和执行逻辑。
/// 各占卜模块实现此 trait 以支持统一的删除机制。
///
/// # 设计原则
///
/// 1. **权限控制**: 只有所有者可以删除数据
/// 2. **关联清理**: 删除时自动清理所有关联数据
/// 3. **押金返还**: 删除时触发押金返还
///
/// # 使用示例
///
/// ```ignore
/// // 检查是否可删除
/// if T::Deletable::can_delete(&who, chart_id) {
///     // 获取关联数据列表
///     let related = T::Deletable::get_related_data_ids(chart_id);
///
///     // 执行删除
///     T::Deletable::do_delete(&who, chart_id)?;
/// }
/// ```
pub trait Deletable<AccountId, ChartId> {
    /// 检查是否可以删除
    ///
    /// 验证删除权限和前置条件：
    /// 1. 调用者是数据所有者
    /// 2. 数据存在
    /// 3. 没有锁定（如已 NFT 化则不可删）
    ///
    /// # 参数
    /// - `who`: 调用者账户
    /// - `chart_id`: 数据 ID
    ///
    /// # 返回
    /// - `true`: 可以删除
    /// - `false`: 不可删除
    fn can_delete(who: &AccountId, chart_id: ChartId) -> bool;

    /// 执行删除
    ///
    /// 删除数据及其所有关联存储项。
    /// 调用前应先使用 `can_delete` 检查权限。
    ///
    /// # 删除流程
    /// 1. 验证权限
    /// 2. 获取押金信息
    /// 3. 删除主数据
    /// 4. 删除关联数据（缓存、加密数据等）
    /// 5. 更新用户列表索引
    /// 6. 返还押金
    ///
    /// # 参数
    /// - `who`: 调用者账户
    /// - `chart_id`: 数据 ID
    ///
    /// # 返回
    /// - `Ok(())`: 删除成功
    /// - `Err`: 删除失败（权限不足、数据不存在等）
    fn do_delete(who: &AccountId, chart_id: ChartId) -> DispatchResult;

    /// 获取关联数据 ID 列表
    ///
    /// 返回需要一起删除的关联数据 ID。
    /// 用于前端显示删除影响范围。
    ///
    /// # 参数
    /// - `chart_id`: 主数据 ID
    ///
    /// # 返回
    /// 关联数据 ID 列表（如解读缓存 ID、加密数据 ID 等）
    fn get_related_data_ids(chart_id: ChartId) -> Vec<ChartId>;

    /// 获取押金记录
    ///
    /// 返回数据创建时锁定的押金信息，用于计算返还金额。
    ///
    /// # 参数
    /// - `chart_id`: 数据 ID
    ///
    /// # 返回
    /// `(押金金额, 创建区块号)` 或 None（无押金记录）
    fn get_deposit_info(chart_id: ChartId) -> Option<(u128, u32)>;

    /// 批量删除
    ///
    /// 删除多条数据，适用于用户清理所有数据的场景。
    /// 默认实现为循环调用 `do_delete`。
    ///
    /// # 参数
    /// - `who`: 调用者账户
    /// - `chart_ids`: 要删除的数据 ID 列表
    ///
    /// # 返回
    /// - `Ok(deleted_count)`: 成功删除的数量
    /// - `Err`: 批量删除过程中的错误
    fn batch_delete(who: &AccountId, chart_ids: Vec<ChartId>) -> Result<u32, &'static str>
    where
        ChartId: Copy,
    {
        let mut deleted = 0u32;
        for chart_id in chart_ids {
            if Self::can_delete(who, chart_id) {
                if Self::do_delete(who, chart_id).is_ok() {
                    deleted = deleted.saturating_add(1);
                }
            }
        }
        Ok(deleted)
    }
}

/// 空实现的可删除 Trait
///
/// 用于测试或不需要删除功能的场景。
pub struct NullDeletable;

/// 命盘级联删除器 Trait
///
/// 当用户删除命盘时，需要级联删除关联的订单、解读、评价等数据，
/// 并取消相关的 IPFS Pin。
///
/// # 实现说明
///
/// - `pallet-divination-market` 实现此 trait 以删除订单相关数据
/// - `pallet-divination-privacy` 实现此 trait 以撤销授权
///
/// # 示例
///
/// ```ignore
/// impl<T: Config> ChartCascadeDeleter<T::AccountId> for Pallet<T> {
///     fn cascade_delete_for_chart(
///         owner: &T::AccountId,
///         divination_type: DivinationType,
///         chart_id: u64,
///     ) -> Result<CascadeDeleteResult, DispatchError> {
///         // 1. 查找关联订单
///         // 2. 收集 CID 并 Unpin
///         // 3. 删除订单记录
///         Ok(CascadeDeleteResult { ... })
///     }
/// }
/// ```
pub trait ChartCascadeDeleter<AccountId> {
    /// 级联删除命盘关联数据
    ///
    /// # 参数
    /// - `owner`: 命盘所有者
    /// - `divination_type`: 占卜类型
    /// - `chart_id`: 命盘 ID
    ///
    /// # 返回
    /// - `Ok(CascadeDeleteResult)`: 删除结果统计
    /// - `Err`: 删除失败
    fn cascade_delete_for_chart(
        owner: &AccountId,
        divination_type: DivinationType,
        chart_id: u64,
    ) -> Result<CascadeDeleteResult, sp_runtime::DispatchError>;
}

/// 级联删除结果
#[derive(Clone, Debug, Default)]
pub struct CascadeDeleteResult {
    /// 删除的订单数量
    pub orders_deleted: u32,
    /// 取消 Pin 的 CID 数量
    pub cids_unpinned: u32,
    /// 撤销的授权数量
    pub grants_revoked: u32,
}

/// 空实现的级联删除器
pub struct NullChartCascadeDeleter;

impl<AccountId> ChartCascadeDeleter<AccountId> for NullChartCascadeDeleter {
    fn cascade_delete_for_chart(
        _owner: &AccountId,
        _divination_type: DivinationType,
        _chart_id: u64,
    ) -> Result<CascadeDeleteResult, sp_runtime::DispatchError> {
        Ok(CascadeDeleteResult::default())
    }
}

impl<AccountId, ChartId: Default> Deletable<AccountId, ChartId> for NullDeletable {
    fn can_delete(_who: &AccountId, _chart_id: ChartId) -> bool {
        false
    }

    fn do_delete(_who: &AccountId, _chart_id: ChartId) -> DispatchResult {
        Err(sp_runtime::DispatchError::Other("Not implemented"))
    }

    fn get_related_data_ids(_chart_id: ChartId) -> Vec<ChartId> {
        Vec::new()
    }

    fn get_deposit_info(_chart_id: ChartId) -> Option<(u128, u32)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_provider() {
        type Provider = NullDivinationProvider;
        assert!(!<Provider as DivinationProvider<u64>>::result_exists(DivinationType::Meihua, 1));
        assert!(<Provider as DivinationProvider<u64>>::result_creator(DivinationType::Meihua, 1).is_none());
        assert!(<Provider as DivinationProvider<u64>>::rarity_data(DivinationType::Meihua, 1).is_none());
        assert!(!<Provider as DivinationProvider<u64>>::is_nftable(DivinationType::Meihua, 1));
    }

    #[test]
    fn test_null_context_generator() {
        assert!(NullContextGenerator::generate_context(
            DivinationType::Meihua,
            1,
            InterpretationType::Basic
        ).is_none());
    }

    #[test]
    fn test_supported_interpretation_types() {
        let meihua_types = NullContextGenerator::supported_interpretation_types(DivinationType::Meihua);
        assert_eq!(meihua_types.len(), 9);
        assert!(meihua_types.contains(&InterpretationType::Professional));

        let liuyao_types = NullContextGenerator::supported_interpretation_types(DivinationType::Liuyao);
        assert_eq!(liuyao_types.len(), 2);
        assert!(!liuyao_types.contains(&InterpretationType::Professional));
    }

    #[test]
    fn test_estimated_tokens() {
        assert_eq!(
            NullContextGenerator::estimated_tokens(DivinationType::Meihua, InterpretationType::Basic),
            500
        );
        assert_eq!(
            NullContextGenerator::estimated_tokens(DivinationType::Bazi, InterpretationType::Professional),
            3000
        );
    }

    #[test]
    fn test_null_storage_deposit_manager() {
        type Manager = NullStorageDepositManager;

        // 测试 calculate_deposit 返回默认值
        let deposit = <Manager as StorageDepositManager<u64, u128, u32>>::calculate_deposit(
            1024,
            PrivacyMode::Public,
        );
        assert_eq!(deposit, 0);

        // 测试 reserve_deposit 总是成功
        let result = <Manager as StorageDepositManager<u64, u128, u32>>::reserve_deposit(
            &1u64,
            1000u128,
        );
        assert!(result.is_ok());

        // 测试 unreserve_deposit 返回默认值
        let refund = <Manager as StorageDepositManager<u64, u128, u32>>::unreserve_deposit(
            &1u64,
            1000u128,
            0u32,
            100u32,
        );
        assert_eq!(refund, 0);

        // 测试 total_reserved 返回默认值
        let reserved = <Manager as StorageDepositManager<u64, u128, u32>>::total_reserved(&1u64);
        assert_eq!(reserved, 0);
    }

    #[test]
    fn test_null_deletable() {
        type Del = NullDeletable;

        // 测试 can_delete 总是返回 false
        let can = <Del as Deletable<u64, u64>>::can_delete(&1u64, 1u64);
        assert!(!can);

        // 测试 do_delete 总是失败
        let result = <Del as Deletable<u64, u64>>::do_delete(&1u64, 1u64);
        assert!(result.is_err());

        // 测试 get_related_data_ids 返回空列表
        let related = <Del as Deletable<u64, u64>>::get_related_data_ids(1u64);
        assert!(related.is_empty());

        // 测试 get_deposit_info 返回 None
        let info = <Del as Deletable<u64, u64>>::get_deposit_info(1u64);
        assert!(info.is_none());

        // 测试 batch_delete 返回 0
        let deleted = <Del as Deletable<u64, u64>>::batch_delete(&1u64, vec![1u64, 2u64, 3u64]);
        assert_eq!(deleted, Ok(0));
    }
}
