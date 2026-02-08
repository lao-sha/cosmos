//! # 公共类型定义
//!
//! 本模块定义 Trading 相关的公共类型，供多个 pallet 共享。
//!
//! ## 版本历史
//! - v0.1.0 (2026-01-18): 初始版本，从 OTC/Swap/Maker 模块提取

use codec::{Encode, Decode};
use scale_info::TypeInfo;
use frame_support::{BoundedVec, pallet_prelude::ConstU32};

/// 函数级详细中文注释：TRON 地址类型（固定 34 字节）
///
/// ## 说明
/// - TRC20 地址以 'T' 开头，长度固定为 34 字符
/// - 用于 OTC 订单收款地址和 Swap 兑换地址
///
/// ## 使用者
/// - `pallet-trading-p2p`: 做市商收款地址 / 用户 USDT 接收地址
/// - `pallet-trading-maker`: 做市商注册地址
pub type TronAddress = BoundedVec<u8, ConstU32<34>>;

/// 函数级详细中文注释：时间戳类型（Unix 秒）
///
/// ## 说明
/// - 用于 OTC 订单的时间字段
/// - 精度为秒（非毫秒）
pub type MomentOf = u64;

/// 函数级详细中文注释：IPFS CID 类型（最大 64 字节）
///
/// ## 说明
/// - 用于存储 IPFS 内容标识符
/// - 如做市商的公开/私密资料
pub type Cid = BoundedVec<u8, ConstU32<64>>;

/// 函数级详细中文注释：交易哈希类型（最大 128 字节）
///
/// ## 说明
/// - 用于存储 TRON TRC20 交易哈希
/// - Swap 模块中使用
pub type TxHash = BoundedVec<u8, ConstU32<128>>;

/// 函数级详细中文注释：做市商申请信息（简化版，用于跨模块传递）
///
/// ## 说明
/// - 由 Maker 模块提供给 OTC/Swap 模块查询使用
/// - 仅包含必要的字段，避免暴露完整的内部结构
#[derive(Clone, Encode, Decode, TypeInfo)]
#[scale_info(skip_type_params(AccountId, Balance))]
pub struct MakerApplicationInfo<AccountId, Balance> {
    /// 做市商账户
    pub account: AccountId,
    /// TRON 收款地址
    pub tron_address: TronAddress,
    /// 是否激活
    pub is_active: bool,
    /// 幽灵数据（类型占位）
    pub _phantom: sp_std::marker::PhantomData<Balance>,
}
