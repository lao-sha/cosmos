//! # 商城模块数据类型定义
//!
//! 定义店铺、商品、订单等核心数据结构

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::*, BoundedVec};
use scale_info::TypeInfo;

// ============================================================================
// 店铺相关类型
// ============================================================================

/// 店铺状态
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum ShopStatus {
    /// 待审核
    #[default]
    Pending,
    /// 正常营业
    Active,
    /// 暂停营业（店主主动）
    Suspended,
    /// 被封禁（治理处罚）
    Banned,
    /// 已关闭
    Closed,
}

/// 店铺信息
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
#[scale_info(skip_type_params(MaxNameLen, MaxCidLen))]
pub struct Shop<AccountId, Balance, BlockNumber, MaxNameLen: Get<u32>, MaxCidLen: Get<u32>> {
    /// 店铺 ID
    pub id: u64,
    /// 店主账户
    pub owner: AccountId,
    /// 店铺名称
    pub name: BoundedVec<u8, MaxNameLen>,
    /// 店铺 Logo IPFS CID
    pub logo_cid: Option<BoundedVec<u8, MaxCidLen>>,
    /// 店铺描述 IPFS CID
    pub description_cid: Option<BoundedVec<u8, MaxCidLen>>,
    /// 保证金金额
    pub deposit: Balance,
    /// 店铺状态
    pub status: ShopStatus,
    /// 商品数量
    pub product_count: u32,
    /// 累计销售额
    pub total_sales: Balance,
    /// 累计订单数
    pub total_orders: u32,
    /// 店铺评分 (0-500，代表 0.0-5.0)
    pub rating: u16,
    /// 评价数量
    pub rating_count: u32,
    /// 创建时间
    pub created_at: BlockNumber,
}

// ============================================================================
// 商品相关类型
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
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
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

/// 商品信息
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
#[scale_info(skip_type_params(MaxCidLen))]
pub struct Product<Balance, BlockNumber, MaxCidLen: Get<u32>> {
    /// 商品 ID
    pub id: u64,
    /// 所属店铺 ID
    pub shop_id: u64,
    /// 商品名称 IPFS CID
    pub name_cid: BoundedVec<u8, MaxCidLen>,
    /// 商品图片 IPFS CID（JSON 数组）
    pub images_cid: BoundedVec<u8, MaxCidLen>,
    /// 商品详情 IPFS CID（富文本/Markdown）
    pub detail_cid: BoundedVec<u8, MaxCidLen>,
    /// 单价（COS）
    pub price: Balance,
    /// 库存数量（0 表示无限）
    pub stock: u32,
    /// 已售数量
    pub sold_count: u32,
    /// 商品状态
    pub status: ProductStatus,
    /// 商品类别
    pub category: ProductCategory,
    /// 创建时间
    pub created_at: BlockNumber,
    /// 更新时间
    pub updated_at: BlockNumber,
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

/// 订单信息
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
#[scale_info(skip_type_params(MaxCidLen))]
pub struct MallOrder<AccountId, Balance, BlockNumber, MaxCidLen: Get<u32>> {
    /// 订单 ID
    pub id: u64,
    /// 店铺 ID
    pub shop_id: u64,
    /// 商品 ID
    pub product_id: u64,
    /// 买家账户
    pub buyer: AccountId,
    /// 卖家账户（店主）
    pub seller: AccountId,
    /// 购买数量
    pub quantity: u32,
    /// 单价
    pub unit_price: Balance,
    /// 总金额
    pub total_amount: Balance,
    /// 平台费
    pub platform_fee: Balance,
    /// 收货地址 IPFS CID（加密存储）
    pub shipping_cid: Option<BoundedVec<u8, MaxCidLen>>,
    /// 物流信息 IPFS CID
    pub tracking_cid: Option<BoundedVec<u8, MaxCidLen>>,
    /// 订单状态
    pub status: MallOrderStatus,
    /// 创建时间
    pub created_at: BlockNumber,
    /// 支付时间
    pub paid_at: Option<BlockNumber>,
    /// 发货时间
    pub shipped_at: Option<BlockNumber>,
    /// 完成时间
    pub completed_at: Option<BlockNumber>,
    /// 托管 ID（对应 pallet-escrow）
    pub escrow_id: u64,
}

/// 订单评价
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
#[scale_info(skip_type_params(MaxCidLen))]
pub struct MallReview<AccountId, BlockNumber, MaxCidLen: Get<u32>> {
    /// 订单 ID
    pub order_id: u64,
    /// 评价者
    pub reviewer: AccountId,
    /// 评分 (1-5)
    pub rating: u8,
    /// 评价内容 IPFS CID
    pub content_cid: Option<BoundedVec<u8, MaxCidLen>>,
    /// 评价时间
    pub created_at: BlockNumber,
}

// ============================================================================
// 统计类型
// ============================================================================

/// 商城统计
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct MallStatistics<Balance: Default> {
    /// 总店铺数
    pub total_shops: u64,
    /// 活跃店铺数
    pub active_shops: u64,
    /// 总商品数
    pub total_products: u64,
    /// 在售商品数
    pub on_sale_products: u64,
    /// 总订单数
    pub total_orders: u64,
    /// 已完成订单数
    pub completed_orders: u64,
    /// 总交易额
    pub total_volume: Balance,
    /// 总平台费收入
    pub total_platform_fees: Balance,
}
