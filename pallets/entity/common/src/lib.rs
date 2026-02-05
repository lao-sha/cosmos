//! # 实体公共模块 (pallet-entity-common)
//!
//! 定义实体各子模块共享的类型和 Trait 接口

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

// ============================================================================
// 实体类型枚举 (Phase 2 新增)
// ============================================================================

/// 实体类型
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum EntityType {
    /// 商户（原 Shop，默认类型）
    #[default]
    Merchant,
    /// 企业
    Enterprise,
    /// 去中心化自治组织
    DAO,
    /// 社区
    Community,
    /// 项目方
    Project,
    /// 服务提供商
    ServiceProvider,
    /// 基金
    Fund,
    /// 自定义类型
    Custom(u8),
}

/// 治理模式
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum GovernanceMode {
    /// 无治理（管理员全权控制）
    #[default]
    None,
    /// 咨询型（提案不自动执行，仅收集意见）
    Advisory,
    /// 双轨制（管理员可快速执行，重大决策需投票）
    DualTrack,
    /// 委员会（委员会成员投票决策）
    Committee,
    /// 完全 DAO（所有决策需投票）
    FullDAO,
    /// 分层治理（不同级别决策不同阈值）
    Tiered,
}

// ============================================================================
// 实体相关类型
// ============================================================================

/// 实体状态（原 ShopStatus）
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum EntityStatus {
    /// 待审核
    #[default]
    Pending,
    /// 正常运营
    Active,
    /// 暂停运营（管理员主动）
    Suspended,
    /// 被封禁（治理处罚）
    Banned,
    /// 已关闭
    Closed,
}

/// 向后兼容：ShopStatus 别名
pub type ShopStatus = EntityStatus;

// ============================================================================
// 通证类型枚举 (Phase 2 新增)
// ============================================================================

/// 通证类型
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum TokenType {
    /// 积分（原默认类型，消费奖励）
    #[default]
    Points,
    /// 治理代币（投票权）
    Governance,
    /// 股权代币（分红权）
    Equity,
    /// 会员代币（会员资格）
    Membership,
    /// 份额代币（基金份额）
    Share,
    /// 债券代币（固定收益）
    Bond,
    /// 混合型（多种权益）
    Hybrid(u8),
}

impl TokenType {
    /// 是否具有投票权
    pub fn has_voting_power(&self) -> bool {
        matches!(self, Self::Governance | Self::Equity | Self::Hybrid(_))
    }
    
    /// 是否具有分红权
    pub fn has_dividend_rights(&self) -> bool {
        matches!(self, Self::Equity | Self::Share | Self::Hybrid(_))
    }
    
    /// 是否可转让（默认可转让）
    pub fn is_transferable_by_default(&self) -> bool {
        !matches!(self, Self::Membership)
    }
    
    /// 获取默认要求的 KYC 级别
    /// 返回 (持有者 KYC, 接收方 KYC)
    pub fn required_kyc_level(&self) -> (u8, u8) {
        match self {
            Self::Points => (0, 0),           // None, None
            Self::Membership => (1, 1),       // Basic, Basic
            Self::Governance => (2, 2),       // Standard, Standard
            Self::Share | Self::Bond => (2, 2), // Standard, Standard
            Self::Equity => (3, 3),           // Enhanced, Enhanced
            Self::Hybrid(_) => (2, 2),        // Standard, Standard (默认)
        }
    }
    
    /// 是否为证券类型（需要严格合规）
    pub fn is_security(&self) -> bool {
        matches!(self, Self::Equity | Self::Share | Self::Bond)
    }
    
    /// 是否需要强制披露
    pub fn requires_disclosure(&self) -> bool {
        matches!(self, Self::Equity | Self::Share | Self::Bond)
    }
    
    /// 默认转账限制模式
    /// 0 = None, 1 = Whitelist, 2 = Blacklist, 3 = KycRequired, 4 = MembersOnly
    pub fn default_transfer_restriction(&self) -> u8 {
        match self {
            Self::Points => 0,       // None
            Self::Membership => 4,   // MembersOnly
            Self::Governance => 3,   // KycRequired
            Self::Share | Self::Bond => 3, // KycRequired
            Self::Equity => 1,       // Whitelist
            Self::Hybrid(_) => 0,    // None (可配置)
        }
    }
}

/// 转账限制模式
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum TransferRestrictionMode {
    /// 无限制（默认）
    #[default]
    None,
    /// 白名单模式 - 只能转给白名单地址
    Whitelist,
    /// 黑名单模式 - 禁止转给黑名单地址
    Blacklist,
    /// KYC 模式 - 接收方需满足 KYC 要求
    KycRequired,
    /// 闭环模式 - 只能在实体成员间转账
    MembersOnly,
}

impl TransferRestrictionMode {
    /// 从 u8 转换
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Whitelist,
            2 => Self::Blacklist,
            3 => Self::KycRequired,
            4 => Self::MembersOnly,
            _ => Self::None,
        }
    }
}

/// 分红配置
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct DividendConfig<Balance, BlockNumber> {
    /// 是否启用分红
    pub enabled: bool,
    /// 最小分红周期（区块数）
    pub min_period: BlockNumber,
    /// 上次分红时间
    pub last_distribution: BlockNumber,
    /// 累计待分配金额
    pub accumulated: Balance,
}

// ============================================================================
// 服务/商品相关类型
// ============================================================================

/// 商品状态
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum ProductStatus {
    /// 草稿（未上架）
    #[default]
    Draft,
    /// 在售
    OnSale,
    /// 售罄
    SoldOut,
    /// 已下架
    OffShelf,
}

/// 商品类别
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum ProductCategory {
    /// 数字商品（虚拟物品）
    Digital,
    /// 实物商品
    #[default]
    Physical,
    /// 服务类
    Service,
    /// 其他
    Other,
}

// ============================================================================
// 会员相关类型
// ============================================================================

/// 会员等级
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum MemberLevel {
    #[default]
    Normal,     // 普通会员
    Silver,     // 银卡会员
    Gold,       // 金卡会员
    Platinum,   // 白金会员
    Diamond,    // 钻石会员
}

// ============================================================================
// 订单相关类型
// ============================================================================

/// 订单状态
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum MallOrderStatus {
    /// 已创建，待支付
    #[default]
    Created,
    /// 已支付，待发货
    Paid,
    /// 已发货，待收货
    Shipped,
    /// 已完成
    Completed,
    /// 已取消（买家取消）
    Cancelled,
    /// 争议中
    Disputed,
    /// 已退款
    Refunded,
    /// 已过期（支付超时）
    Expired,
}

// ============================================================================
// 跨模块 Trait 接口
// ============================================================================

/// 店铺查询接口
/// 
/// 供 product 模块查询店铺信息
pub trait ShopProvider<AccountId> {
    /// 检查店铺是否存在
    fn shop_exists(shop_id: u64) -> bool;
    
    /// 检查店铺是否激活
    fn is_shop_active(shop_id: u64) -> bool;
    
    /// 获取店铺所有者
    fn shop_owner(shop_id: u64) -> Option<AccountId>;
    
    /// 获取店铺派生账户
    fn shop_account(shop_id: u64) -> AccountId;
    
    /// 更新店铺统计（销售额、订单数）
    fn update_shop_stats(shop_id: u64, sales_amount: u128, order_count: u32) -> Result<(), DispatchError>;
    
    /// 更新店铺评分
    fn update_shop_rating(shop_id: u64, rating: u8) -> Result<(), DispatchError>;
    
    // ==================== 治理调用接口 ====================
    
    /// 暂停店铺（治理调用）
    fn pause_shop(shop_id: u64) -> Result<(), DispatchError> {
        let _ = shop_id;
        Ok(()) // 默认空实现
    }
    
    /// 恢复店铺（治理调用）
    fn resume_shop(shop_id: u64) -> Result<(), DispatchError> {
        let _ = shop_id;
        Ok(())
    }
}

/// 商品查询接口
/// 
/// 供 order 模块查询和更新商品信息
pub trait ProductProvider<AccountId, Balance> {
    /// 检查商品是否存在
    fn product_exists(product_id: u64) -> bool;
    
    /// 检查商品是否在售
    fn is_product_on_sale(product_id: u64) -> bool;
    
    /// 获取商品所属店铺
    fn product_shop_id(product_id: u64) -> Option<u64>;
    
    /// 获取商品价格
    fn product_price(product_id: u64) -> Option<Balance>;
    
    /// 获取商品库存
    fn product_stock(product_id: u64) -> Option<u32>;
    
    /// 获取商品类别
    fn product_category(product_id: u64) -> Option<ProductCategory>;
    
    /// 扣减库存
    fn deduct_stock(product_id: u64, quantity: u32) -> Result<(), DispatchError>;
    
    /// 恢复库存
    fn restore_stock(product_id: u64, quantity: u32) -> Result<(), DispatchError>;
    
    /// 增加销量
    fn add_sold_count(product_id: u64, quantity: u32) -> Result<(), DispatchError>;
    
    // ==================== 治理调用接口 ====================
    
    /// 更新商品价格（治理调用）
    fn update_price(product_id: u64, new_price: Balance) -> Result<(), DispatchError> {
        let _ = (product_id, new_price);
        Ok(())
    }
    
    /// 下架商品（治理调用）
    fn delist_product(product_id: u64) -> Result<(), DispatchError> {
        let _ = product_id;
        Ok(())
    }
    
    /// 调整库存（治理调用）
    fn set_inventory(product_id: u64, new_inventory: u32) -> Result<(), DispatchError> {
        let _ = (product_id, new_inventory);
        Ok(())
    }
}

/// 订单查询接口
/// 
/// 供 review 模块查询订单信息
pub trait OrderProvider<AccountId, Balance> {
    /// 检查订单是否存在
    fn order_exists(order_id: u64) -> bool;
    
    /// 获取订单买家
    fn order_buyer(order_id: u64) -> Option<AccountId>;
    
    /// 获取订单店铺
    fn order_shop_id(order_id: u64) -> Option<u64>;
    
    /// 检查订单是否已完成
    fn is_order_completed(order_id: u64) -> bool;
}

// ============================================================================
// 空实现（用于测试）
// ============================================================================

/// 空店铺提供者（测试用）
pub struct NullShopProvider;

impl<AccountId: Default> ShopProvider<AccountId> for NullShopProvider {
    fn shop_exists(_shop_id: u64) -> bool { false }
    fn is_shop_active(_shop_id: u64) -> bool { false }
    fn shop_owner(_shop_id: u64) -> Option<AccountId> { None }
    fn shop_account(_shop_id: u64) -> AccountId { AccountId::default() }
    fn update_shop_stats(_shop_id: u64, _sales_amount: u128, _order_count: u32) -> Result<(), DispatchError> { Ok(()) }
    fn update_shop_rating(_shop_id: u64, _rating: u8) -> Result<(), DispatchError> { Ok(()) }
}

/// 空商品提供者（测试用）
pub struct NullProductProvider;

impl<AccountId, Balance> ProductProvider<AccountId, Balance> for NullProductProvider {
    fn product_exists(_product_id: u64) -> bool { false }
    fn is_product_on_sale(_product_id: u64) -> bool { false }
    fn product_shop_id(_product_id: u64) -> Option<u64> { None }
    fn product_price(_product_id: u64) -> Option<Balance> { None }
    fn product_stock(_product_id: u64) -> Option<u32> { None }
    fn product_category(_product_id: u64) -> Option<ProductCategory> { None }
    fn deduct_stock(_product_id: u64, _quantity: u32) -> Result<(), DispatchError> { Ok(()) }
    fn restore_stock(_product_id: u64, _quantity: u32) -> Result<(), DispatchError> { Ok(()) }
    fn add_sold_count(_product_id: u64, _quantity: u32) -> Result<(), DispatchError> { Ok(()) }
}

/// 空订单提供者（测试用）
pub struct NullOrderProvider;

impl<AccountId, Balance> OrderProvider<AccountId, Balance> for NullOrderProvider {
    fn order_exists(_order_id: u64) -> bool { false }
    fn order_buyer(_order_id: u64) -> Option<AccountId> { None }
    fn order_shop_id(_order_id: u64) -> Option<u64> { None }
    fn is_order_completed(_order_id: u64) -> bool { false }
}

// ============================================================================
// 店铺代币接口
// ============================================================================

/// 店铺代币接口
/// 
/// 供 order 模块调用，实现购物返积分和积分抵扣
pub trait ShopTokenProvider<AccountId, Balance> {
    /// 检查店铺是否启用代币
    fn is_token_enabled(shop_id: u64) -> bool;
    
    /// 获取用户积分余额
    fn token_balance(shop_id: u64, holder: &AccountId) -> Balance;
    
    /// 购物奖励（订单完成时调用）
    fn reward_on_purchase(
        shop_id: u64,
        buyer: &AccountId,
        purchase_amount: Balance,
    ) -> Result<Balance, DispatchError>;
    
    /// 积分兑换折扣（下单时调用）
    fn redeem_for_discount(
        shop_id: u64,
        buyer: &AccountId,
        tokens: Balance,
    ) -> Result<Balance, DispatchError>;
    
    /// 转移代币（P2P 交易市场使用）
    fn transfer(
        shop_id: u64,
        from: &AccountId,
        to: &AccountId,
        amount: Balance,
    ) -> Result<(), DispatchError>;
    
    /// 锁定代币（挂单时使用）
    fn reserve(
        shop_id: u64,
        who: &AccountId,
        amount: Balance,
    ) -> Result<(), DispatchError>;
    
    /// 解锁代币（取消订单时使用）
    fn unreserve(
        shop_id: u64,
        who: &AccountId,
        amount: Balance,
    ) -> Balance;
    
    /// 从锁定中转移（成交时使用）
    fn repatriate_reserved(
        shop_id: u64,
        from: &AccountId,
        to: &AccountId,
        amount: Balance,
    ) -> Result<Balance, DispatchError>;
    
    /// Phase 8: 获取代币类型
    fn get_token_type(shop_id: u64) -> TokenType;
    
    /// Phase 8: 获取代币总供应量
    fn total_supply(shop_id: u64) -> Balance;
}

/// 空店铺代币提供者（测试用或未启用代币时）
pub struct NullShopTokenProvider;

// ============================================================================
// 定价接口
// ============================================================================

/// COS/USDT 价格查询接口
/// 
/// 供 shop 模块计算 USDT 等值的 COS 押金
pub trait PricingProvider {
    /// 获取 COS/USDT 加权平均价格
    /// 
    /// # 返回
    /// - `u64`: 价格（精度 10^6，即 1,000,000 = 1 USDT/COS）
    /// - 返回 0 表示价格不可用
    fn get_cos_usdt_price() -> u64;
}

/// 空定价提供者（测试用）
pub struct NullPricingProvider;

impl PricingProvider for NullPricingProvider {
    fn get_cos_usdt_price() -> u64 {
        // 默认价格：0.000001 USDT/COS（精度 10^6 = 1）
        1
    }
}

impl<AccountId, Balance: Default> ShopTokenProvider<AccountId, Balance> for NullShopTokenProvider {
    fn is_token_enabled(_shop_id: u64) -> bool { false }
    fn token_balance(_shop_id: u64, _holder: &AccountId) -> Balance { Default::default() }
    fn reward_on_purchase(_: u64, _: &AccountId, _: Balance) -> Result<Balance, DispatchError> { 
        Ok(Default::default()) 
    }
    fn redeem_for_discount(_: u64, _: &AccountId, _: Balance) -> Result<Balance, DispatchError> { 
        Ok(Default::default()) 
    }
    fn transfer(_: u64, _: &AccountId, _: &AccountId, _: Balance) -> Result<(), DispatchError> {
        Ok(())
    }
    fn reserve(_: u64, _: &AccountId, _: Balance) -> Result<(), DispatchError> {
        Ok(())
    }
    fn unreserve(_: u64, _: &AccountId, _: Balance) -> Balance {
        Default::default()
    }
    fn repatriate_reserved(_: u64, _: &AccountId, _: &AccountId, _: Balance) -> Result<Balance, DispatchError> {
        Ok(Default::default())
    }
    fn get_token_type(_shop_id: u64) -> TokenType {
        TokenType::default()
    }
    fn total_supply(_shop_id: u64) -> Balance {
        Default::default()
    }
}
