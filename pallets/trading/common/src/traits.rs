//! # 公共 Trait 定义
//!
//! 本模块定义 Trading 相关的公共接口，供多个 pallet 共享。
//!
//! ## 版本历史
//! - v0.1.0 (2026-01-18): 初始版本，从 OTC/Swap/Maker 模块提取

use sp_runtime::DispatchResult;
use crate::types::MakerApplicationInfo;

/// 函数级详细中文注释：定价服务接口
///
/// ## 说明
/// 提供 DUST/USD 实时汇率查询功能
///
/// ## 使用者
/// - `pallet-trading-otc`: 计算订单金额
/// - `pallet-trading-swap`: 计算兑换金额
/// - `pallet-trading-maker`: 计算押金价值
///
/// ## 实现者
/// - `pallet-trading-pricing`: 提供聚合价格
pub trait PricingProvider<Balance> {
    /// 获取 DUST/USD 汇率（精度 10^6）
    ///
    /// ## 返回
    /// - `Some(rate)`: 当前汇率（如 1_000_000 表示 1 DUST = 1 USD）
    /// - `None`: 价格不可用（冷启动期或无数据）
    fn get_dust_to_usd_rate() -> Option<Balance>;
}

/// 函数级详细中文注释：Maker Pallet 接口
///
/// ## 说明
/// 提供做市商信息查询功能
///
/// ## 使用者
/// - `pallet-trading-otc`: 验证做市商和获取收款地址
/// - `pallet-trading-swap`: 验证做市商状态
///
/// ## 实现者
/// - `pallet-trading-maker`: 提供做市商管理
pub trait MakerInterface<AccountId, Balance> {
    /// 查询做市商申请信息
    ///
    /// ## 参数
    /// - `maker_id`: 做市商ID
    ///
    /// ## 返回
    /// - `Some(info)`: 做市商信息
    /// - `None`: 做市商不存在
    fn get_maker_application(maker_id: u64) -> Option<MakerApplicationInfo<AccountId, Balance>>;
    
    /// 检查做市商是否激活
    ///
    /// ## 参数
    /// - `maker_id`: 做市商ID
    ///
    /// ## 返回
    /// - `true`: 激活状态
    /// - `false`: 未激活或不存在
    fn is_maker_active(maker_id: u64) -> bool;
    
    /// 获取做市商 ID（通过账户）
    ///
    /// ## 参数
    /// - `who`: 账户地址
    ///
    /// ## 返回
    /// - `Some(maker_id)`: 做市商ID
    /// - `None`: 该账户不是做市商
    fn get_maker_id(who: &AccountId) -> Option<u64>;
}

/// 函数级详细中文注释：做市商信用接口
///
/// ## 说明
/// 提供做市商信用分管理功能
///
/// ## 使用者
/// - `pallet-trading-otc`: 订单完成/超时/争议时调用
/// - `pallet-trading-swap`: 兑换完成/超时/争议时调用
///
/// ## 实现者
/// - `pallet-trading-credit`: 提供信用分管理
pub trait MakerCreditInterface {
    /// 记录做市商订单完成（提升信用分）
    ///
    /// ## 参数
    /// - `maker_id`: 做市商ID
    /// - `order_id`: 订单ID
    /// - `response_time_seconds`: 响应时间（秒）
    fn record_maker_order_completed(
        maker_id: u64,
        order_id: u64,
        response_time_seconds: u32,
    ) -> DispatchResult;
    
    /// 记录做市商订单超时（降低信用分）
    ///
    /// ## 参数
    /// - `maker_id`: 做市商ID
    /// - `order_id`: 订单ID
    fn record_maker_order_timeout(
        maker_id: u64,
        order_id: u64,
    ) -> DispatchResult;
    
    /// 记录做市商争议结果
    ///
    /// ## 参数
    /// - `maker_id`: 做市商ID
    /// - `order_id`: 订单ID
    /// - `maker_win`: true = 做市商胜诉
    fn record_maker_dispute_result(
        maker_id: u64,
        order_id: u64,
        maker_win: bool,
    ) -> DispatchResult;
}

// ===== 默认实现（用于测试和 Mock）=====

/// PricingProvider 的空实现
impl<Balance> PricingProvider<Balance> for () {
    fn get_dust_to_usd_rate() -> Option<Balance> {
        None
    }
}

/// MakerInterface 的空实现
impl<AccountId, Balance> MakerInterface<AccountId, Balance> for () {
    fn get_maker_application(_maker_id: u64) -> Option<MakerApplicationInfo<AccountId, Balance>> {
        None
    }
    
    fn is_maker_active(_maker_id: u64) -> bool {
        false
    }
    
    fn get_maker_id(_who: &AccountId) -> Option<u64> {
        None
    }
}

/// MakerCreditInterface 的空实现
impl MakerCreditInterface for () {
    fn record_maker_order_completed(
        _maker_id: u64,
        _order_id: u64,
        _response_time_seconds: u32,
    ) -> DispatchResult {
        Ok(())
    }
    
    fn record_maker_order_timeout(
        _maker_id: u64,
        _order_id: u64,
    ) -> DispatchResult {
        Ok(())
    }
    
    fn record_maker_dispute_result(
        _maker_id: u64,
        _order_id: u64,
        _maker_win: bool,
    ) -> DispatchResult {
        Ok(())
    }
}
