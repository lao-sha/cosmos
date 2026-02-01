//! # 多店铺商城模块 (pallet-sharemall)
//!
//! ## 概述
//!
//! 本模块提供去中心化的多店铺商城功能，包括：
//! - 店铺管理：创建、更新、暂停、关闭店铺
//! - 商品管理：上架、下架、库存管理
//! - 订单流程：下单、支付、发货、确认收货
//! - 评价系统：订单完成后评价
//! - 争议处理：集成仲裁系统
//!
//! ## 版本历史
//!
//! - v0.1.0 (2026-01-31): 初始版本

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;

mod types;
pub use types::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use alloc::vec::Vec;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
        BoundedVec,
    };
    use frame_system::pallet_prelude::*;
    use pallet_escrow::pallet::Escrow as EscrowTrait;
    use sp_runtime::{
        traits::{Saturating, Zero},
        SaturatedConversion,
    };

    /// 货币余额类型别名
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// 店铺类型别名
    pub type ShopOf<T> = Shop<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
        <T as Config>::MaxShopNameLength,
        <T as Config>::MaxCidLength,
    >;

    /// 商品类型别名
    pub type ProductOf<T> = Product<
        BalanceOf<T>,
        BlockNumberFor<T>,
        <T as Config>::MaxCidLength,
    >;

    /// 订单类型别名
    pub type MallOrderOf<T> = MallOrder<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
        <T as Config>::MaxCidLength,
    >;

    /// 评价类型别名
    pub type MallReviewOf<T> = MallReview<
        <T as frame_system::Config>::AccountId,
        BlockNumberFor<T>,
        <T as Config>::MaxCidLength,
    >;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 运行时事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 货币类型
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 托管接口
        type Escrow: EscrowTrait<Self::AccountId, BalanceOf<Self>>;

        /// 平台账户
        #[pallet::constant]
        type PlatformAccount: Get<Self::AccountId>;

        /// 最低店铺保证金
        #[pallet::constant]
        type MinShopDeposit: Get<BalanceOf<Self>>;

        /// 平台费率（基点，200 = 2%）
        #[pallet::constant]
        type PlatformFeeRate: Get<u16>;

        /// 支付超时（区块数）
        #[pallet::constant]
        type PaymentTimeout: Get<BlockNumberFor<Self>>;

        /// 发货超时（区块数）
        #[pallet::constant]
        type ShipTimeout: Get<BlockNumberFor<Self>>;

        /// 确认收货超时（区块数）
        #[pallet::constant]
        type ConfirmTimeout: Get<BlockNumberFor<Self>>;

        /// 每店铺最大商品数
        #[pallet::constant]
        type MaxProductsPerShop: Get<u32>;

        /// 店铺名称最大长度
        #[pallet::constant]
        type MaxShopNameLength: Get<u32>;

        /// CID 最大长度
        #[pallet::constant]
        type MaxCidLength: Get<u32>;

        /// 治理 Origin
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ==================== 存储项 ====================

    /// 下一个店铺 ID
    #[pallet::storage]
    #[pallet::getter(fn next_shop_id)]
    pub type NextShopId<T> = StorageValue<_, u64, ValueQuery>;

    /// 店铺存储
    #[pallet::storage]
    #[pallet::getter(fn shops)]
    pub type Shops<T: Config> = StorageMap<_, Blake2_128Concat, u64, ShopOf<T>>;

    /// 用户店铺索引（一个用户只能有一个店铺）
    #[pallet::storage]
    #[pallet::getter(fn user_shop)]
    pub type UserShop<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64>;

    /// 下一个商品 ID
    #[pallet::storage]
    #[pallet::getter(fn next_product_id)]
    pub type NextProductId<T> = StorageValue<_, u64, ValueQuery>;

    /// 商品存储
    #[pallet::storage]
    #[pallet::getter(fn products)]
    pub type Products<T: Config> = StorageMap<_, Blake2_128Concat, u64, ProductOf<T>>;

    /// 店铺商品索引
    #[pallet::storage]
    #[pallet::getter(fn shop_products)]
    pub type ShopProducts<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        BoundedVec<u64, T::MaxProductsPerShop>,
        ValueQuery,
    >;

    /// 下一个订单 ID
    #[pallet::storage]
    #[pallet::getter(fn next_order_id)]
    pub type NextOrderId<T> = StorageValue<_, u64, ValueQuery>;

    /// 订单存储
    #[pallet::storage]
    #[pallet::getter(fn orders)]
    pub type Orders<T: Config> = StorageMap<_, Blake2_128Concat, u64, MallOrderOf<T>>;

    /// 买家订单索引
    #[pallet::storage]
    #[pallet::getter(fn buyer_orders)]
    pub type BuyerOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<1000>>,
        ValueQuery,
    >;

    /// 店铺订单索引
    #[pallet::storage]
    #[pallet::getter(fn shop_orders)]
    pub type ShopOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        BoundedVec<u64, ConstU32<10000>>,
        ValueQuery,
    >;

    /// 订单评价存储
    #[pallet::storage]
    #[pallet::getter(fn reviews)]
    pub type Reviews<T: Config> = StorageMap<_, Blake2_128Concat, u64, MallReviewOf<T>>;

    /// 商城统计
    #[pallet::storage]
    #[pallet::getter(fn mall_stats)]
    pub type MallStats<T: Config> = StorageValue<_, MallStatistics<BalanceOf<T>>, ValueQuery>;

    // ==================== 事件 ====================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // ========== 店铺事件 ==========
        /// 店铺已创建
        ShopCreated {
            shop_id: u64,
            owner: T::AccountId,
            deposit: BalanceOf<T>,
        },
        /// 店铺已更新
        ShopUpdated { shop_id: u64 },
        /// 店铺状态已变更
        ShopStatusChanged { shop_id: u64, status: ShopStatus },
        /// 店铺已关闭
        ShopClosed {
            shop_id: u64,
            deposit_refunded: BalanceOf<T>,
        },

        // ========== 商品事件 ==========
        /// 商品已创建
        ProductCreated { product_id: u64, shop_id: u64 },
        /// 商品已更新
        ProductUpdated { product_id: u64 },
        /// 商品状态已变更
        ProductStatusChanged {
            product_id: u64,
            status: ProductStatus,
        },
        /// 商品已删除
        ProductDeleted { product_id: u64 },

        // ========== 订单事件 ==========
        /// 订单已创建
        OrderCreated {
            order_id: u64,
            buyer: T::AccountId,
            seller: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 订单已支付
        OrderPaid { order_id: u64, escrow_id: u64 },
        /// 订单已发货
        OrderShipped { order_id: u64 },
        /// 订单已完成
        OrderCompleted {
            order_id: u64,
            seller_received: BalanceOf<T>,
        },
        /// 订单已取消
        OrderCancelled { order_id: u64 },
        /// 订单已退款
        OrderRefunded {
            order_id: u64,
            amount: BalanceOf<T>,
        },
        /// 订单进入争议
        OrderDisputed { order_id: u64 },

        // ========== 评价事件 ==========
        /// 评价已提交
        ReviewSubmitted { order_id: u64, rating: u8 },
    }

    // ==================== 错误 ====================

    #[pallet::error]
    pub enum Error<T> {
        // ========== 店铺错误 ==========
        /// 店铺不存在
        ShopNotFound,
        /// 用户已有店铺
        ShopAlreadyExists,
        /// 不是店主
        NotShopOwner,
        /// 店铺未激活
        ShopNotActive,
        /// 店铺有进行中的订单
        ShopHasPendingOrders,
        /// 保证金不足
        InsufficientDeposit,

        // ========== 商品错误 ==========
        /// 商品不存在
        ProductNotFound,
        /// 商品不在售
        ProductNotOnSale,
        /// 库存不足
        InsufficientStock,
        /// 达到最大商品数
        MaxProductsReached,
        /// 商品有进行中的订单
        ProductHasPendingOrders,

        // ========== 订单错误 ==========
        /// 订单不存在
        OrderNotFound,
        /// 不是订单买家
        NotOrderBuyer,
        /// 不是订单卖家
        NotOrderSeller,
        /// 无效的订单状态
        InvalidOrderStatus,
        /// 订单已过期
        OrderExpired,
        /// 无法取消订单
        CannotCancelOrder,
        /// 已评价过
        AlreadyReviewed,
        /// 无效的评分
        InvalidRating,
        /// 不能购买自己店铺的商品
        CannotBuyOwnProduct,

        // ========== 通用错误 ==========
        /// 余额不足
        InsufficientBalance,
        /// 数值溢出
        Overflow,
        /// 名称过长
        NameTooLong,
        /// CID 过长
        CidTooLong,
    }

    // ==================== Hooks ====================

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// 空闲时处理超时订单
        fn on_idle(_now: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
            let base_weight = Weight::from_parts(20_000, 0);
            if remaining_weight.ref_time() < base_weight.ref_time() * 5 {
                return Weight::zero();
            }

            Self::process_expired_orders(5)
        }
    }

    // ==================== Extrinsics ====================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // ==================== 店铺管理 ====================

        /// 创建店铺
        ///
        /// 用户缴纳保证金后创建店铺，初始状态为 Pending
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(50_000, 0))]
        pub fn create_shop(
            origin: OriginFor<T>,
            name: Vec<u8>,
            logo_cid: Option<Vec<u8>>,
            description_cid: Option<Vec<u8>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查用户是否已有店铺
            ensure!(!UserShop::<T>::contains_key(&who), Error::<T>::ShopAlreadyExists);

            // 转换名称
            let name: BoundedVec<u8, T::MaxShopNameLength> =
                name.try_into().map_err(|_| Error::<T>::NameTooLong)?;

            // 转换 CID
            let logo_cid: Option<BoundedVec<u8, T::MaxCidLength>> = logo_cid
                .map(|c| c.try_into().map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;
            let description_cid: Option<BoundedVec<u8, T::MaxCidLength>> = description_cid
                .map(|c| c.try_into().map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;

            // 锁定保证金
            let deposit = T::MinShopDeposit::get();
            T::Currency::reserve(&who, deposit)?;

            // 创建店铺
            let shop_id = NextShopId::<T>::get();
            let now = <frame_system::Pallet<T>>::block_number();

            let shop = Shop {
                id: shop_id,
                owner: who.clone(),
                name,
                logo_cid,
                description_cid,
                deposit,
                status: ShopStatus::Pending,
                product_count: 0,
                total_sales: Zero::zero(),
                total_orders: 0,
                rating: 0,
                rating_count: 0,
                created_at: now,
            };

            // 存储
            Shops::<T>::insert(shop_id, shop);
            UserShop::<T>::insert(&who, shop_id);
            NextShopId::<T>::put(shop_id.saturating_add(1));

            // 更新统计
            MallStats::<T>::mutate(|stats| {
                stats.total_shops = stats.total_shops.saturating_add(1);
            });

            Self::deposit_event(Event::ShopCreated {
                shop_id,
                owner: who,
                deposit,
            });

            Ok(())
        }

        /// 更新店铺信息
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn update_shop(
            origin: OriginFor<T>,
            shop_id: u64,
            name: Option<Vec<u8>>,
            logo_cid: Option<Vec<u8>>,
            description_cid: Option<Vec<u8>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Shops::<T>::try_mutate(shop_id, |maybe_shop| -> DispatchResult {
                let shop = maybe_shop.as_mut().ok_or(Error::<T>::ShopNotFound)?;
                ensure!(shop.owner == who, Error::<T>::NotShopOwner);

                if let Some(n) = name {
                    shop.name = n.try_into().map_err(|_| Error::<T>::NameTooLong)?;
                }
                if let Some(c) = logo_cid {
                    shop.logo_cid = Some(c.try_into().map_err(|_| Error::<T>::CidTooLong)?);
                }
                if let Some(c) = description_cid {
                    shop.description_cid = Some(c.try_into().map_err(|_| Error::<T>::CidTooLong)?);
                }

                Ok(())
            })?;

            Self::deposit_event(Event::ShopUpdated { shop_id });
            Ok(())
        }

        /// 暂停店铺营业
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn suspend_shop(origin: OriginFor<T>, shop_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Shops::<T>::try_mutate(shop_id, |maybe_shop| -> DispatchResult {
                let shop = maybe_shop.as_mut().ok_or(Error::<T>::ShopNotFound)?;
                ensure!(shop.owner == who, Error::<T>::NotShopOwner);
                ensure!(shop.status == ShopStatus::Active, Error::<T>::InvalidOrderStatus);

                shop.status = ShopStatus::Suspended;
                Ok(())
            })?;

            // 更新统计
            MallStats::<T>::mutate(|stats| {
                stats.active_shops = stats.active_shops.saturating_sub(1);
            });

            Self::deposit_event(Event::ShopStatusChanged {
                shop_id,
                status: ShopStatus::Suspended,
            });
            Ok(())
        }

        /// 恢复店铺营业
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn resume_shop(origin: OriginFor<T>, shop_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Shops::<T>::try_mutate(shop_id, |maybe_shop| -> DispatchResult {
                let shop = maybe_shop.as_mut().ok_or(Error::<T>::ShopNotFound)?;
                ensure!(shop.owner == who, Error::<T>::NotShopOwner);
                ensure!(shop.status == ShopStatus::Suspended, Error::<T>::InvalidOrderStatus);

                shop.status = ShopStatus::Active;
                Ok(())
            })?;

            // 更新统计
            MallStats::<T>::mutate(|stats| {
                stats.active_shops = stats.active_shops.saturating_add(1);
            });

            Self::deposit_event(Event::ShopStatusChanged {
                shop_id,
                status: ShopStatus::Active,
            });
            Ok(())
        }

        /// 关闭店铺
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(40_000, 0))]
        pub fn close_shop(origin: OriginFor<T>, shop_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let shop = Shops::<T>::get(shop_id).ok_or(Error::<T>::ShopNotFound)?;
            ensure!(shop.owner == who, Error::<T>::NotShopOwner);

            // 检查是否有进行中的订单
            let shop_order_ids = ShopOrders::<T>::get(shop_id);
            for order_id in shop_order_ids.iter() {
                if let Some(order) = Orders::<T>::get(order_id) {
                    let is_pending = matches!(
                        order.status,
                        MallOrderStatus::Created | MallOrderStatus::Paid | MallOrderStatus::Shipped
                    );
                    ensure!(!is_pending, Error::<T>::ShopHasPendingOrders);
                }
            }

            // 退还保证金
            let deposit = shop.deposit;
            T::Currency::unreserve(&who, deposit);

            // 更新店铺状态
            Shops::<T>::mutate(shop_id, |maybe_shop| {
                if let Some(s) = maybe_shop {
                    s.status = ShopStatus::Closed;
                }
            });

            // 更新统计
            MallStats::<T>::mutate(|stats| {
                if shop.status == ShopStatus::Active {
                    stats.active_shops = stats.active_shops.saturating_sub(1);
                }
            });

            Self::deposit_event(Event::ShopClosed {
                shop_id,
                deposit_refunded: deposit,
            });
            Ok(())
        }

        /// 审核通过店铺（治理）
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn approve_shop(origin: OriginFor<T>, shop_id: u64) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            Shops::<T>::try_mutate(shop_id, |maybe_shop| -> DispatchResult {
                let shop = maybe_shop.as_mut().ok_or(Error::<T>::ShopNotFound)?;
                ensure!(shop.status == ShopStatus::Pending, Error::<T>::InvalidOrderStatus);

                shop.status = ShopStatus::Active;
                Ok(())
            })?;

            // 更新统计
            MallStats::<T>::mutate(|stats| {
                stats.active_shops = stats.active_shops.saturating_add(1);
            });

            Self::deposit_event(Event::ShopStatusChanged {
                shop_id,
                status: ShopStatus::Active,
            });
            Ok(())
        }

        /// 封禁店铺（治理）
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn ban_shop(origin: OriginFor<T>, shop_id: u64) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            let shop = Shops::<T>::get(shop_id).ok_or(Error::<T>::ShopNotFound)?;

            Shops::<T>::mutate(shop_id, |maybe_shop| {
                if let Some(s) = maybe_shop {
                    s.status = ShopStatus::Banned;
                }
            });

            // 更新统计
            if shop.status == ShopStatus::Active {
                MallStats::<T>::mutate(|stats| {
                    stats.active_shops = stats.active_shops.saturating_sub(1);
                });
            }

            Self::deposit_event(Event::ShopStatusChanged {
                shop_id,
                status: ShopStatus::Banned,
            });
            Ok(())
        }

        // ==================== 商品管理 ====================

        /// 创建商品
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(40_000, 0))]
        pub fn create_product(
            origin: OriginFor<T>,
            shop_id: u64,
            name_cid: Vec<u8>,
            images_cid: Vec<u8>,
            detail_cid: Vec<u8>,
            price: BalanceOf<T>,
            stock: u32,
            category: ProductCategory,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证店铺
            let shop = Shops::<T>::get(shop_id).ok_or(Error::<T>::ShopNotFound)?;
            ensure!(shop.owner == who, Error::<T>::NotShopOwner);
            ensure!(shop.status == ShopStatus::Active, Error::<T>::ShopNotActive);

            // 检查商品数量限制
            let product_ids = ShopProducts::<T>::get(shop_id);
            ensure!(
                product_ids.len() < T::MaxProductsPerShop::get() as usize,
                Error::<T>::MaxProductsReached
            );

            // 转换 CID
            let name_cid: BoundedVec<u8, T::MaxCidLength> =
                name_cid.try_into().map_err(|_| Error::<T>::CidTooLong)?;
            let images_cid: BoundedVec<u8, T::MaxCidLength> =
                images_cid.try_into().map_err(|_| Error::<T>::CidTooLong)?;
            let detail_cid: BoundedVec<u8, T::MaxCidLength> =
                detail_cid.try_into().map_err(|_| Error::<T>::CidTooLong)?;

            // 创建商品
            let product_id = NextProductId::<T>::get();
            let now = <frame_system::Pallet<T>>::block_number();

            let product = Product {
                id: product_id,
                shop_id,
                name_cid,
                images_cid,
                detail_cid,
                price,
                stock,
                sold_count: 0,
                status: ProductStatus::Draft,
                category,
                created_at: now,
                updated_at: now,
            };

            // 存储
            Products::<T>::insert(product_id, product);
            ShopProducts::<T>::try_mutate(shop_id, |ids| ids.try_push(product_id))
                .map_err(|_| Error::<T>::MaxProductsReached)?;
            NextProductId::<T>::put(product_id.saturating_add(1));

            // 更新店铺商品数
            Shops::<T>::mutate(shop_id, |maybe_shop| {
                if let Some(s) = maybe_shop {
                    s.product_count = s.product_count.saturating_add(1);
                }
            });

            // 更新统计
            MallStats::<T>::mutate(|stats| {
                stats.total_products = stats.total_products.saturating_add(1);
            });

            Self::deposit_event(Event::ProductCreated { product_id, shop_id });
            Ok(())
        }

        /// 更新商品信息
        #[pallet::call_index(11)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn update_product(
            origin: OriginFor<T>,
            product_id: u64,
            name_cid: Option<Vec<u8>>,
            images_cid: Option<Vec<u8>>,
            detail_cid: Option<Vec<u8>>,
            price: Option<BalanceOf<T>>,
            stock: Option<u32>,
            category: Option<ProductCategory>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Products::<T>::try_mutate(product_id, |maybe_product| -> DispatchResult {
                let product = maybe_product.as_mut().ok_or(Error::<T>::ProductNotFound)?;

                // 验证店主
                let shop = Shops::<T>::get(product.shop_id).ok_or(Error::<T>::ShopNotFound)?;
                ensure!(shop.owner == who, Error::<T>::NotShopOwner);

                // 更新字段
                if let Some(c) = name_cid {
                    product.name_cid = c.try_into().map_err(|_| Error::<T>::CidTooLong)?;
                }
                if let Some(c) = images_cid {
                    product.images_cid = c.try_into().map_err(|_| Error::<T>::CidTooLong)?;
                }
                if let Some(c) = detail_cid {
                    product.detail_cid = c.try_into().map_err(|_| Error::<T>::CidTooLong)?;
                }
                if let Some(p) = price {
                    product.price = p;
                }
                if let Some(s) = stock {
                    product.stock = s;
                    // 如果库存恢复且状态为售罄，自动改为在售
                    if s > 0 && product.status == ProductStatus::SoldOut {
                        product.status = ProductStatus::OnSale;
                    }
                }
                if let Some(c) = category {
                    product.category = c;
                }

                product.updated_at = <frame_system::Pallet<T>>::block_number();
                Ok(())
            })?;

            Self::deposit_event(Event::ProductUpdated { product_id });
            Ok(())
        }

        /// 上架商品
        #[pallet::call_index(12)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn publish_product(origin: OriginFor<T>, product_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Products::<T>::try_mutate(product_id, |maybe_product| -> DispatchResult {
                let product = maybe_product.as_mut().ok_or(Error::<T>::ProductNotFound)?;

                let shop = Shops::<T>::get(product.shop_id).ok_or(Error::<T>::ShopNotFound)?;
                ensure!(shop.owner == who, Error::<T>::NotShopOwner);
                ensure!(shop.status == ShopStatus::Active, Error::<T>::ShopNotActive);

                product.status = ProductStatus::OnSale;
                product.updated_at = <frame_system::Pallet<T>>::block_number();
                Ok(())
            })?;

            // 更新统计
            MallStats::<T>::mutate(|stats| {
                stats.on_sale_products = stats.on_sale_products.saturating_add(1);
            });

            Self::deposit_event(Event::ProductStatusChanged {
                product_id,
                status: ProductStatus::OnSale,
            });
            Ok(())
        }

        /// 下架商品
        #[pallet::call_index(13)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn unpublish_product(origin: OriginFor<T>, product_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Products::<T>::try_mutate(product_id, |maybe_product| -> DispatchResult {
                let product = maybe_product.as_mut().ok_or(Error::<T>::ProductNotFound)?;

                let shop = Shops::<T>::get(product.shop_id).ok_or(Error::<T>::ShopNotFound)?;
                ensure!(shop.owner == who, Error::<T>::NotShopOwner);

                product.status = ProductStatus::OffShelf;
                product.updated_at = <frame_system::Pallet<T>>::block_number();
                Ok(())
            })?;

            // 更新统计
            MallStats::<T>::mutate(|stats| {
                stats.on_sale_products = stats.on_sale_products.saturating_sub(1);
            });

            Self::deposit_event(Event::ProductStatusChanged {
                product_id,
                status: ProductStatus::OffShelf,
            });
            Ok(())
        }

        // ==================== 订单流程 ====================

        /// 下单并支付
        #[pallet::call_index(20)]
        #[pallet::weight(Weight::from_parts(60_000, 0))]
        pub fn place_order(
            origin: OriginFor<T>,
            product_id: u64,
            quantity: u32,
            shipping_cid: Option<Vec<u8>>,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;

            // 获取商品信息
            let mut product = Products::<T>::get(product_id).ok_or(Error::<T>::ProductNotFound)?;
            ensure!(product.status == ProductStatus::OnSale, Error::<T>::ProductNotOnSale);

            // 检查库存
            if product.stock > 0 {
                ensure!(product.stock >= quantity, Error::<T>::InsufficientStock);
            }

            // 获取店铺信息
            let shop = Shops::<T>::get(product.shop_id).ok_or(Error::<T>::ShopNotFound)?;
            ensure!(shop.status == ShopStatus::Active, Error::<T>::ShopNotActive);
            ensure!(shop.owner != buyer, Error::<T>::CannotBuyOwnProduct);

            // 计算金额
            let total_amount = product.price.saturating_mul(quantity.into());
            let platform_fee = total_amount
                .saturating_mul(T::PlatformFeeRate::get().into())
                / 10000u32.into();

            // 转换 shipping_cid
            let shipping_cid: Option<BoundedVec<u8, T::MaxCidLength>> = shipping_cid
                .map(|c| c.try_into().map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;

            // 创建订单
            let order_id = NextOrderId::<T>::get();
            let now = <frame_system::Pallet<T>>::block_number();

            // 锁定资金到托管
            T::Escrow::lock_from(&buyer, order_id, total_amount)?;

            let order = MallOrder {
                id: order_id,
                shop_id: product.shop_id,
                product_id,
                buyer: buyer.clone(),
                seller: shop.owner.clone(),
                quantity,
                unit_price: product.price,
                total_amount,
                platform_fee,
                shipping_cid,
                tracking_cid: None,
                status: MallOrderStatus::Paid,
                created_at: now,
                paid_at: Some(now),
                shipped_at: None,
                completed_at: None,
                escrow_id: order_id,
            };

            // 更新库存
            if product.stock > 0 {
                product.stock = product.stock.saturating_sub(quantity);
                if product.stock == 0 {
                    product.status = ProductStatus::SoldOut;
                }
            }
            product.sold_count = product.sold_count.saturating_add(quantity);
            Products::<T>::insert(product_id, product);

            // 存储订单
            Orders::<T>::insert(order_id, order);
            BuyerOrders::<T>::try_mutate(&buyer, |ids| ids.try_push(order_id))
                .map_err(|_| Error::<T>::Overflow)?;
            ShopOrders::<T>::try_mutate(shop.id, |ids| ids.try_push(order_id))
                .map_err(|_| Error::<T>::Overflow)?;
            NextOrderId::<T>::put(order_id.saturating_add(1));

            // 更新统计
            MallStats::<T>::mutate(|stats| {
                stats.total_orders = stats.total_orders.saturating_add(1);
            });

            Self::deposit_event(Event::OrderCreated {
                order_id,
                buyer,
                seller: shop.owner,
                amount: total_amount,
            });
            Self::deposit_event(Event::OrderPaid {
                order_id,
                escrow_id: order_id,
            });

            Ok(())
        }

        /// 取消订单
        #[pallet::call_index(21)]
        #[pallet::weight(Weight::from_parts(40_000, 0))]
        pub fn cancel_order(origin: OriginFor<T>, order_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            ensure!(order.buyer == who, Error::<T>::NotOrderBuyer);
            ensure!(
                order.status == MallOrderStatus::Created || order.status == MallOrderStatus::Paid,
                Error::<T>::CannotCancelOrder
            );

            // 退款
            T::Escrow::refund_all(order_id, &order.buyer)?;

            // 恢复库存
            Products::<T>::mutate(order.product_id, |maybe_product| {
                if let Some(p) = maybe_product {
                    if p.stock > 0 || p.status == ProductStatus::SoldOut {
                        p.stock = p.stock.saturating_add(order.quantity);
                        if p.status == ProductStatus::SoldOut {
                            p.status = ProductStatus::OnSale;
                        }
                    }
                    p.sold_count = p.sold_count.saturating_sub(order.quantity);
                }
            });

            // 更新订单状态
            Orders::<T>::mutate(order_id, |maybe_order| {
                if let Some(o) = maybe_order {
                    o.status = MallOrderStatus::Cancelled;
                }
            });

            Self::deposit_event(Event::OrderCancelled { order_id });
            Ok(())
        }

        /// 发货
        #[pallet::call_index(22)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn ship_order(
            origin: OriginFor<T>,
            order_id: u64,
            tracking_cid: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Orders::<T>::try_mutate(order_id, |maybe_order| -> DispatchResult {
                let order = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?;
                ensure!(order.seller == who, Error::<T>::NotOrderSeller);
                ensure!(order.status == MallOrderStatus::Paid, Error::<T>::InvalidOrderStatus);

                order.tracking_cid = Some(
                    tracking_cid.try_into().map_err(|_| Error::<T>::CidTooLong)?
                );
                order.status = MallOrderStatus::Shipped;
                order.shipped_at = Some(<frame_system::Pallet<T>>::block_number());
                Ok(())
            })?;

            Self::deposit_event(Event::OrderShipped { order_id });
            Ok(())
        }

        /// 确认收货
        #[pallet::call_index(23)]
        #[pallet::weight(Weight::from_parts(50_000, 0))]
        pub fn confirm_receipt(origin: OriginFor<T>, order_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            ensure!(order.buyer == who, Error::<T>::NotOrderBuyer);
            ensure!(order.status == MallOrderStatus::Shipped, Error::<T>::InvalidOrderStatus);

            Self::do_complete_order(order_id, &order)
        }

        /// 申请退款
        #[pallet::call_index(24)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn request_refund(
            origin: OriginFor<T>,
            order_id: u64,
            _reason_cid: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Orders::<T>::try_mutate(order_id, |maybe_order| -> DispatchResult {
                let order = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?;
                ensure!(order.buyer == who, Error::<T>::NotOrderBuyer);
                ensure!(
                    order.status == MallOrderStatus::Paid || order.status == MallOrderStatus::Shipped,
                    Error::<T>::InvalidOrderStatus
                );

                order.status = MallOrderStatus::Disputed;
                Ok(())
            })?;

            Self::deposit_event(Event::OrderDisputed { order_id });
            Ok(())
        }

        /// 同意退款（卖家）
        #[pallet::call_index(25)]
        #[pallet::weight(Weight::from_parts(40_000, 0))]
        pub fn approve_refund(origin: OriginFor<T>, order_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            ensure!(order.seller == who, Error::<T>::NotOrderSeller);
            ensure!(order.status == MallOrderStatus::Disputed, Error::<T>::InvalidOrderStatus);

            // 退款给买家
            T::Escrow::refund_all(order_id, &order.buyer)?;

            // 恢复库存
            Products::<T>::mutate(order.product_id, |maybe_product| {
                if let Some(p) = maybe_product {
                    if p.stock > 0 || p.status == ProductStatus::SoldOut {
                        p.stock = p.stock.saturating_add(order.quantity);
                        if p.status == ProductStatus::SoldOut {
                            p.status = ProductStatus::OnSale;
                        }
                    }
                    p.sold_count = p.sold_count.saturating_sub(order.quantity);
                }
            });

            // 更新订单状态
            Orders::<T>::mutate(order_id, |maybe_order| {
                if let Some(o) = maybe_order {
                    o.status = MallOrderStatus::Refunded;
                }
            });

            Self::deposit_event(Event::OrderRefunded {
                order_id,
                amount: order.total_amount,
            });
            Ok(())
        }

        /// 提交评价
        #[pallet::call_index(30)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn submit_review(
            origin: OriginFor<T>,
            order_id: u64,
            rating: u8,
            content_cid: Option<Vec<u8>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证评分范围
            ensure!(rating >= 1 && rating <= 5, Error::<T>::InvalidRating);

            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            ensure!(order.buyer == who, Error::<T>::NotOrderBuyer);
            ensure!(order.status == MallOrderStatus::Completed, Error::<T>::InvalidOrderStatus);
            ensure!(!Reviews::<T>::contains_key(order_id), Error::<T>::AlreadyReviewed);

            // 转换 CID
            let content_cid: Option<BoundedVec<u8, T::MaxCidLength>> = content_cid
                .map(|c| c.try_into().map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;

            let now = <frame_system::Pallet<T>>::block_number();

            let review = MallReview {
                order_id,
                reviewer: who,
                rating,
                content_cid,
                created_at: now,
            };

            Reviews::<T>::insert(order_id, review);

            // 更新店铺评分
            Shops::<T>::mutate(order.shop_id, |maybe_shop| {
                if let Some(shop) = maybe_shop {
                    let total_rating = (shop.rating as u32)
                        .saturating_mul(shop.rating_count)
                        .saturating_add((rating as u32) * 100);
                    shop.rating_count = shop.rating_count.saturating_add(1);
                    shop.rating = (total_rating / shop.rating_count) as u16;
                }
            });

            Self::deposit_event(Event::ReviewSubmitted { order_id, rating });
            Ok(())
        }
    }

    // ==================== 内部函数 ====================

    impl<T: Config> Pallet<T> {
        /// 完成订单（释放资金）
        fn do_complete_order(order_id: u64, order: &MallOrderOf<T>) -> DispatchResult {
            // 计算卖家收入（扣除平台费）
            let seller_amount = order.total_amount.saturating_sub(order.platform_fee);

            // 释放资金给卖家
            T::Escrow::transfer_from_escrow(order_id, &order.seller, seller_amount)?;

            // 平台费转给平台账户
            if !order.platform_fee.is_zero() {
                T::Escrow::transfer_from_escrow(order_id, &T::PlatformAccount::get(), order.platform_fee)?;
            }

            let now = <frame_system::Pallet<T>>::block_number();

            // 更新订单状态
            Orders::<T>::mutate(order_id, |maybe_order| {
                if let Some(o) = maybe_order {
                    o.status = MallOrderStatus::Completed;
                    o.completed_at = Some(now);
                }
            });

            // 更新店铺统计
            Shops::<T>::mutate(order.shop_id, |maybe_shop| {
                if let Some(shop) = maybe_shop {
                    shop.total_sales = shop.total_sales.saturating_add(seller_amount);
                    shop.total_orders = shop.total_orders.saturating_add(1);
                }
            });

            // 更新全局统计
            MallStats::<T>::mutate(|stats| {
                stats.completed_orders = stats.completed_orders.saturating_add(1);
                stats.total_volume = stats.total_volume.saturating_add(order.total_amount);
                stats.total_platform_fees = stats.total_platform_fees.saturating_add(order.platform_fee);
            });

            Self::deposit_event(Event::OrderCompleted {
                order_id,
                seller_received: seller_amount,
            });

            Ok(())
        }

        /// 处理过期订单
        fn process_expired_orders(max_count: u32) -> Weight {
            let now = <frame_system::Pallet<T>>::block_number();
            let mut processed = 0u32;

            // 遍历订单检查超时
            let next_id = NextOrderId::<T>::get();
            for order_id in 0..next_id {
                if processed >= max_count {
                    break;
                }

                if let Some(order) = Orders::<T>::get(order_id) {
                    match order.status {
                        // 发货超时：自动退款
                        MallOrderStatus::Paid => {
                            if let Some(paid_at) = order.paid_at {
                                if now > paid_at.saturating_add(T::ShipTimeout::get()) {
                                    let _ = T::Escrow::refund_all(order_id, &order.buyer);
                                    Orders::<T>::mutate(order_id, |o| {
                                        if let Some(ord) = o {
                                            ord.status = MallOrderStatus::Refunded;
                                        }
                                    });
                                    processed = processed.saturating_add(1);
                                }
                            }
                        }
                        // 确认超时：自动确认
                        MallOrderStatus::Shipped => {
                            if let Some(shipped_at) = order.shipped_at {
                                if now > shipped_at.saturating_add(T::ConfirmTimeout::get()) {
                                    let _ = Self::do_complete_order(order_id, &order);
                                    processed = processed.saturating_add(1);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            Weight::from_parts(30_000 * processed as u64, 0)
        }
    }
}
