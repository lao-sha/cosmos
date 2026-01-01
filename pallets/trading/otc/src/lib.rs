//! # OTC Order Pallet (场外交易订单模块 - 集成KYC认证)
//!
//! ## 概述
//!
//! 本模块负责 OTC（场外交易）订单的完整生命周期管理，包括：
//! - 订单创建与管理
//! - 首购订单特殊逻辑（固定USD价值，动态DUST数量）
//! - 订单状态流转（创建→付款→释放→完成）
//! - 订单争议与仲裁
//! - 自动清理过期订单
//! - **🆕 KYC身份认证要求（基于pallet-identity）**
//!
//! ## KYC认证功能
//!
//! - 委员会可以启用/禁用KYC要求
//! - 支持不同的认证等级要求（Reasonable/KnownGood等）
//! - 紧急豁免账户机制
//! - 只有通过KYC认证的用户才能创建OTC订单
//!
//! ## 版本历史
//!
//! - v0.1.0 (2025-11-03): 从 pallet-trading 拆分而来
//! - v0.2.0 (2025-11-13): 集成KYC认证功能
//! - v0.3.0 (2025-11-28): 集成聊天权限系统

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;

mod types;
mod kyc;

// 选择性导出 types 中的类型（避免 KycConfig 冲突）
pub use types::{KycVerificationResult, KycFailureReason};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use frame_support::{
        traits::{Currency, Get, UnixTime},
        BoundedVec,
        sp_runtime::SaturatedConversion,
    };
    use sp_core::H256;
    use pallet_escrow::Escrow as EscrowTrait;
    use pallet_chat_permission::SceneAuthorizationManager;

    /// 函数级详细中文注释：Balance 类型别名
    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    
    /// 函数级详细中文注释：时间戳类型别名（毫秒）
    pub type MomentOf = u64;
    
    /// 函数级详细中文注释：TRON 地址类型（固定 34 字节）
    pub type TronAddress = BoundedVec<u8, ConstU32<34>>;
    
    /// 函数级详细中文注释：做市商信用接口
    /// 用于记录做市商的订单完成、超时和争议结果
    pub trait MakerCreditInterface {
        /// 记录做市商订单完成（提升信用分）
        fn record_maker_order_completed(
            maker_id: u64,
            order_id: u64,
            response_time_seconds: u32,
        ) -> DispatchResult;
        /// 记录做市商订单超时（降低信用分）
        fn record_maker_order_timeout(
            maker_id: u64,
            order_id: u64,
        ) -> DispatchResult;
        /// 记录做市商争议结果（根据结果调整信用分）
        fn record_maker_dispute_result(
            maker_id: u64,
            order_id: u64,
            maker_win: bool,
        ) -> DispatchResult;
    }
    
    // ===== 数据结构 =====
    
    /// 函数级详细中文注释：订单状态枚举
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum OrderState {
        /// 已创建，等待买家付款
        Created,
        /// 买家已标记付款或做市商已确认
        PaidOrCommitted,
        /// DUST已释放
        Released,
        /// 已退款
        Refunded,
        /// 已取消
        Canceled,
        /// 争议中
        Disputed,
        /// 已关闭
        Closed,
        /// 已过期（1小时未支付，自动取消）
        Expired,
    }
    
    /// 函数级详细中文注释：OTC订单结构
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Order<T: Config> {
        /// 做市商ID
        pub maker_id: u64,
        /// 做市商账户
        pub maker: T::AccountId,
        /// 买家账户
        pub taker: T::AccountId,
        /// 单价（USDT/DUST，精度10^6）
        pub price: BalanceOf<T>,
        /// 数量（DUST数量）
        pub qty: BalanceOf<T>,
        /// 总金额（USDT金额）
        pub amount: BalanceOf<T>,
        /// 创建时间
        pub created_at: MomentOf,
        /// 超时时间
        pub expire_at: MomentOf,
        /// 证据窗口截止时间
        pub evidence_until: MomentOf,
        /// 做市商TRON收款地址
        pub maker_tron_address: TronAddress,
        /// 支付承诺哈希（买家提供）
        pub payment_commit: H256,
        /// 联系方式承诺哈希（买家提供）
        pub contact_commit: H256,
        /// 订单状态
        pub state: OrderState,
        /// EPAY交易号（可选）
        pub epay_trade_no: Option<BoundedVec<u8, ConstU32<64>>>,
        /// 订单完成时间
        pub completed_at: Option<MomentOf>,
        /// 是否为首购订单
        pub is_first_purchase: bool,
    }
    
    #[pallet::pallet]
    pub struct Pallet<T>(_);
    
    /// 函数级详细中文注释：OTC订单模块配置 trait
    #[pallet::config]
    /// 函数级中文注释：OtcOrder Pallet 配置 trait
    /// - 🔴 stable2506 API 变更：RuntimeEvent 自动继承，无需显式声明
    /// - 🆕 集成KYC认证配置（不再继承 pallet_identity::Config，使用数值表示等级）
    /// - 🆕 2025-11-28: 集成聊天权限系统
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {

        /// 货币类型
        type Currency: Currency<Self::AccountId>;

        /// Timestamp（用于获取当前时间）
        type Timestamp: UnixTime;

        /// 托管服务接口（注意：Escrow 使用 order_id 作为托管 ID）
        type Escrow: pallet_escrow::Escrow<Self::AccountId, BalanceOf<Self>>;

        /// 买家信用记录接口（同时支持额度管理）
        type Credit: pallet_trading_credit::BuyerCreditInterface<Self::AccountId>
            + pallet_trading_credit::quota::BuyerQuotaInterface<Self::AccountId>;

        /// 做市商信用记录接口
        type MakerCredit: MakerCreditInterface;

        /// 定价服务接口
        type Pricing: PricingProvider<BalanceOf<Self>>;

        /// Maker Pallet 类型（用于跨 pallet 调用）
        type MakerPallet: MakerInterface<Self::AccountId, BalanceOf<Self>>;

        /// 🆕 委员会起源（用于KYC配置管理）
        type CommitteeOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// 🆕 Identity Provider（用于KYC验证）
        type IdentityProvider: IdentityVerificationProvider<Self::AccountId>;

        /// 🆕 2025-11-28: 聊天权限管理器
        /// 用于在订单创建时自动授予买卖双方聊天权限
        type ChatPermission: pallet_chat_permission::SceneAuthorizationManager<
            Self::AccountId,
            BlockNumberFor<Self>,
        >;

        /// 订单超时时间（默认 1 小时，毫秒）
        #[pallet::constant]
        type OrderTimeout: Get<u64>;

        /// 证据窗口时间（默认 24 小时，毫秒）
        #[pallet::constant]
        type EvidenceWindow: Get<u64>;

        /// 首购订单USD固定价值（精度 10^6，10_000_000 = 10 USD）
        #[pallet::constant]
        type FirstPurchaseUsdValue: Get<u128>;

        /// 首购订单最小DUST数量（防止汇率异常）
        #[pallet::constant]
        type MinFirstPurchaseDustAmount: Get<BalanceOf<Self>>;

        /// 首购订单最大DUST数量（防止汇率异常）
        #[pallet::constant]
        type MaxFirstPurchaseDustAmount: Get<BalanceOf<Self>>;

        /// OTC订单最大USD金额（200 USD，精度10^6）
        #[pallet::constant]
        type MaxOrderUsdAmount: Get<u64>;

        /// OTC订单最小USD金额（20 USD，精度10^6，首购除外）
        #[pallet::constant]
        type MinOrderUsdAmount: Get<u64>;

        /// 首购订单固定USD金额（10 USD，精度10^6）
        #[pallet::constant]
        type FirstPurchaseUsdAmount: Get<u64>;

        /// 金额验证容差（1%，用于处理价格微小波动）
        #[pallet::constant]
        type AmountValidationTolerance: Get<u16>;

        /// 每个做市商最多同时接收的首购订单数量（默认 5）
        #[pallet::constant]
        type MaxFirstPurchaseOrdersPerMaker: Get<u32>;

        /// 权重信息
        type WeightInfo: WeightInfo;
    }
    
    /// 函数级详细中文注释：定价服务 trait
    pub trait PricingProvider<Balance> {
        /// 获取 DUST/USD 汇率（精度 10^6）
        fn get_dust_to_usd_rate() -> Option<Balance>;
    }
    
    /// 函数级详细中文注释：Maker Pallet 接口
    pub trait MakerInterface<AccountId, Balance> {
        /// 查询做市商申请信息
        fn get_maker_application(maker_id: u64) -> Option<MakerApplicationInfo<AccountId, Balance>>;
        /// 检查做市商是否激活
        fn is_maker_active(maker_id: u64) -> bool;
    }
    
    /// 函数级详细中文注释：做市商申请信息（简化版）
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(AccountId, Balance))]
    pub struct MakerApplicationInfo<AccountId, Balance> {
        pub account: AccountId,
        pub tron_address: BoundedVec<u8, ConstU32<34>>,
        pub is_active: bool,
        pub _phantom: sp_std::marker::PhantomData<Balance>,
    }

    /// 函数级详细中文注释：Identity 验证 Provider trait
    /// 用于查询账户的身份认证状态，避免直接依赖 pallet_identity::Config
    pub trait IdentityVerificationProvider<AccountId> {
        /// 获取账户的最高身份认证等级（数值）
        /// 返回 None 表示未设置身份信息
        /// 返回值：0=Unknown, 1=FeePaid, 2=Reasonable, 3=KnownGood
        fn get_highest_judgement_priority(who: &AccountId) -> Option<u8>;

        /// 检查账户的身份认证是否有问题
        fn has_problematic_judgement(who: &AccountId) -> bool;
    }

    /// 临时实现（用于编译通过）
    impl<AccountId> IdentityVerificationProvider<AccountId> for () {
        fn get_highest_judgement_priority(_who: &AccountId) -> Option<u8> {
            None
        }

        fn has_problematic_judgement(_who: &AccountId) -> bool {
            false
        }
    }
    
    #[allow(dead_code)]
    impl<Balance> PricingProvider<Balance> for () {
        fn get_dust_to_usd_rate() -> Option<Balance> {
            None
        }
    }
    
    // ===== 存储 =====
    
    /// 函数级详细中文注释：下一个订单 ID
    #[pallet::storage]
    #[pallet::getter(fn next_order_id)]
    pub type NextOrderId<T> = StorageValue<_, u64, ValueQuery>;
    
    /// 函数级详细中文注释：订单记录
    #[pallet::storage]
    #[pallet::getter(fn orders)]
    pub type Orders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // order_id
        Order<T>,
    >;
    
    /// 函数级详细中文注释：买家订单列表
    #[pallet::storage]
    #[pallet::getter(fn buyer_orders)]
    pub type BuyerOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<100>>,  // 每个买家最多100个订单
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：做市商订单列表
    #[pallet::storage]
    #[pallet::getter(fn maker_orders)]
    pub type MakerOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // maker_id
        BoundedVec<u64, ConstU32<1000>>,  // 每个做市商最多1000个订单
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：买家是否已首购
    #[pallet::storage]
    #[pallet::getter(fn has_first_purchased)]
    pub type HasFirstPurchased<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        bool,
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：做市商首购订单计数
    #[pallet::storage]
    #[pallet::getter(fn maker_first_purchase_count)]
    pub type MakerFirstPurchaseCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // maker_id
        u32,
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：做市商首购订单列表
    #[pallet::storage]
    #[pallet::getter(fn maker_first_purchase_orders)]
    pub type MakerFirstPurchaseOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // maker_id
        BoundedVec<u64, ConstU32<10>>,  // 最多10个首购订单
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：TRON 交易哈希使用记录（防重放）
    #[pallet::storage]
    #[pallet::getter(fn tron_tx_used)]
    pub type TronTxUsed<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256,  // tx_hash
        BlockNumberFor<T>,  // recorded_at
    >;
    
    /// 函数级详细中文注释：TRON 交易哈希队列（用于清理）
    #[pallet::storage]
    #[pallet::getter(fn tron_tx_queue)]
    pub type TronTxQueue<T: Config> = StorageValue<
        _,
        BoundedVec<(H256, BlockNumberFor<T>), ConstU32<10000>>,
        ValueQuery,
    >;

    // ===== KYC存储 =====

    /// 函数级详细中文注释：KYC配置存储
    #[pallet::storage]
    #[pallet::getter(fn kyc_config)]
    pub type KycConfig<T: Config> = StorageValue<
        _,
        crate::types::KycConfig<BlockNumberFor<T>>,
        ValueQuery,
    >;

    /// 函数级详细中文注释：KYC豁免账户列表
    #[pallet::storage]
    #[pallet::getter(fn kyc_exempt_accounts)]
    pub type KycExemptAccounts<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        (),
        OptionQuery,
    >;

    // ===== Genesis配置 =====

    /// 函数级详细中文注释：Genesis配置结构
    ///
    /// 🔒 暂时禁用创世配置，避免 serde 编译问题，待后续重构
    // TODO: 重构 GenesisConfig 以支持正确的 serde 序列化
    /*
    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// 初始KYC配置
        pub kyc_config: crate::types::KycConfig<BlockNumberFor<T>>,
        /// 初始豁免账户列表
        pub exempt_accounts: Vec<T::AccountId>,
    }

    /// 函数级详细中文注释：Genesis构建实现
    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            KycConfig::<T>::put(&self.kyc_config);

            for account in &self.exempt_accounts {
                KycExemptAccounts::<T>::insert(account, ());
            }
        }
    }
    */

    // ===== 事件 =====
    
    /// 函数级详细中文注释：OTC订单模块事件
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 订单已创建
        OrderCreated {
            order_id: u64,
            maker_id: u64,
            buyer: T::AccountId,
            dust_amount: BalanceOf<T>,
            is_first_purchase: bool,
        },
        /// 订单状态已变更
        OrderStateChanged {
            order_id: u64,
            old_state: u8,
            new_state: u8,
            actor: Option<T::AccountId>,
        },
        /// 首购订单已创建
        FirstPurchaseOrderCreated {
            order_id: u64,
            buyer: T::AccountId,
            maker_id: u64,
            usd_value: u128,
            dust_amount: BalanceOf<T>,
        },
        /// TRON 交易哈希已记录
        TronTxHashRecorded {
            tx_hash: H256,
        },
        /// TRON 交易哈希已清理
        TronTxHashCleaned {
            count: u32,
        },

        // ===== KYC相关事件 =====

        /// KYC要求已启用
        /// 等级优先级：0=Unknown, 1=FeePaid, 2=Reasonable, 3=KnownGood
        KycEnabled {
            min_judgment_priority: u8,
        },
        /// KYC要求已禁用
        KycDisabled,
        /// KYC最低等级已更新
        /// 等级优先级：0=Unknown, 1=FeePaid, 2=Reasonable, 3=KnownGood
        KycLevelUpdated {
            new_priority: u8,
        },
        /// 账户被添加到KYC豁免列表
        AccountExemptedFromKyc {
            account: T::AccountId,
        },
        /// 账户从KYC豁免列表中移除
        AccountRemovedFromKycExemption {
            account: T::AccountId,
        },
        /// KYC验证失败
        /// 原因代码：0=IdentityNotSet, 1=NoValidJudgement, 2=InsufficientLevel, 3=QualityIssue
        KycVerificationFailed {
            account: T::AccountId,
            reason_code: u8,
        },
    }
    
    // ===== 错误 =====
    
    /// 函数级详细中文注释：OTC订单模块错误
    #[pallet::error]
    pub enum Error<T> {
        /// 订单不存在
        OrderNotFound,
        /// 做市商不存在
        MakerNotFound,
        /// 做市商未激活
        MakerNotActive,
        /// 订单状态不正确
        InvalidOrderStatus,
        /// 未授权
        NotAuthorized,
        /// 编码错误
        EncodingError,
        /// 存储限制已达到
        StorageLimitReached,
        /// 订单太多
        TooManyOrders,
        /// 已经首购过
        AlreadyFirstPurchased,
        /// 首购配额已用完
        FirstPurchaseQuotaExhausted,
        /// 做市商余额不足
        MakerInsufficientBalance,
        /// 定价不可用
        PricingUnavailable,
        /// 价格无效
        InvalidPrice,
        /// 计算溢出
        CalculationOverflow,
        /// TRON交易哈希已使用
        TronTxHashAlreadyUsed,

        /// 订单金额超过限制
        OrderAmountExceedsLimit,

        /// 订单金额太小
        OrderAmountTooSmall,

        /// 金额计算溢出
        AmountCalculationOverflow,

        /// 定价服务不可用
        PricingServiceUnavailable,

        // ===== KYC相关错误 =====

        /// 未设置身份信息
        IdentityNotSet,
        /// 没有有效的身份判断
        NoValidJudgement,
        /// KYC认证等级不足
        InsufficientKycLevel,
        /// 身份认证质量问题
        IdentityQualityIssue,
        /// 账户已在豁免列表中
        AccountAlreadyExempted,
        /// 账户不在豁免列表中
        AccountNotExempted,
    }
    
    // ===== Extrinsics =====
    
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：创建OTC订单
        ///
        /// # 参数
        /// - `origin`: 调用者（买家，必须是签名账户）
        /// - `maker_id`: 做市商ID
        /// - `dust_amount`: DUST数量
        /// - `payment_commit`: 支付承诺哈希
        /// - `contact_commit`: 联系方式承诺哈希
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::create_order())]
        pub fn create_order(
            origin: OriginFor<T>,
            maker_id: u64,
            dust_amount: BalanceOf<T>,
            payment_commit: H256,
            contact_commit: H256,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;
            let _order_id = Self::do_create_order(
                &buyer,
                maker_id,
                dust_amount,
                payment_commit,
                contact_commit,
            )?;
            Ok(())
        }
        
        /// 函数级详细中文注释：创建首购订单
        ///
        /// # 参数
        /// - `origin`: 调用者（买家，必须是签名账户）
        /// - `maker_id`: 做市商ID
        /// - `payment_commit`: 支付承诺哈希
        /// - `contact_commit`: 联系方式承诺哈希
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::create_order())]
        pub fn create_first_purchase(
            origin: OriginFor<T>,
            maker_id: u64,
            payment_commit: H256,
            contact_commit: H256,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;
            let _order_id = Self::do_create_first_purchase(
                &buyer,
                maker_id,
                payment_commit,
                contact_commit,
            )?;
            Ok(())
        }
        
        /// 函数级详细中文注释：买家标记已付款
        ///
        /// # 参数
        /// - `origin`: 调用者（买家，必须是签名账户）
        /// - `order_id`: 订单ID
        /// - `tron_tx_hash`: TRON交易哈希（可选）
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::create_order())]
        pub fn mark_paid(
            origin: OriginFor<T>,
            order_id: u64,
            tron_tx_hash: Option<sp_std::vec::Vec<u8>>,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;
            Self::do_mark_paid(&buyer, order_id, tron_tx_hash)
        }
        
        /// 函数级详细中文注释：做市商释放DUST
        ///
        /// # 参数
        /// - `origin`: 调用者（做市商，必须是签名账户）
        /// - `order_id`: 订单ID
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::create_order())]
        pub fn release_dust(
            origin: OriginFor<T>,
            order_id: u64,
        ) -> DispatchResult {
            let maker = ensure_signed(origin)?;
            Self::do_release_dust(&maker, order_id)
        }
        
        /// 函数级详细中文注释：取消订单
        ///
        /// # 参数
        /// - `origin`: 调用者（买家或做市商，必须是签名账户）
        /// - `order_id`: 订单ID
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::create_order())]
        pub fn cancel_order(
            origin: OriginFor<T>,
            order_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_cancel_order(&who, order_id)
        }
        
        /// 函数级详细中文注释：发起订单争议
        ///
        /// # 参数
        /// - `origin`: 调用者（买家或做市商，必须是签名账户）
        /// - `order_id`: 订单ID
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::create_order())]
        pub fn dispute_order(
            origin: OriginFor<T>,
            order_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_dispute_order(&who, order_id)
        }

        // ===== KYC管理函数 =====

        /// 函数级详细中文注释：启用KYC要求
        ///
        /// # 参数
        /// - `origin`: 调用者（委员会起源）
        /// - `min_judgment_priority`: 最低认证等级（数值：0=Unknown, 1=FeePaid, 2=Reasonable, 3=KnownGood）
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::enable_kyc_requirement())]
        pub fn enable_kyc_requirement(
            origin: OriginFor<T>,
            min_judgment_priority: u8,
        ) -> DispatchResult {
            T::CommitteeOrigin::ensure_origin(origin)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            let config = crate::types::KycConfig {
                enabled: true,
                min_judgment_priority,
                effective_block: current_block,
                updated_at: current_block,
            };

            KycConfig::<T>::put(config);

            Self::deposit_event(Event::KycEnabled { min_judgment_priority });
            Ok(())
        }

        /// 函数级详细中文注释：禁用KYC要求
        ///
        /// # 参数
        /// - `origin`: 调用者（委员会起源）
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::disable_kyc_requirement())]
        pub fn disable_kyc_requirement(origin: OriginFor<T>) -> DispatchResult {
            T::CommitteeOrigin::ensure_origin(origin)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            KycConfig::<T>::mutate(|config| {
                config.enabled = false;
                config.effective_block = current_block;
                config.updated_at = current_block;
            });

            Self::deposit_event(Event::KycDisabled);
            Ok(())
        }

        /// 函数级详细中文注释：更新最低认证等级
        ///
        /// # 参数
        /// - `origin`: 调用者（委员会起源）
        /// - `new_priority`: 新的最低认证等级（数值：0=Unknown, 1=FeePaid, 2=Reasonable, 3=KnownGood）
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(8)]
        #[pallet::weight(<T as Config>::WeightInfo::update_min_judgment_level())]
        pub fn update_min_judgment_level(
            origin: OriginFor<T>,
            new_priority: u8,
        ) -> DispatchResult {
            T::CommitteeOrigin::ensure_origin(origin)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            KycConfig::<T>::mutate(|config| {
                config.min_judgment_priority = new_priority;
                config.effective_block = current_block;
                config.updated_at = current_block;
            });

            Self::deposit_event(Event::KycLevelUpdated { new_priority });
            Ok(())
        }

        /// 函数级详细中文注释：将账户添加到KYC豁免列表
        ///
        /// # 参数
        /// - `origin`: 调用者（委员会起源）
        /// - `account`: 要豁免的账户
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::exempt_account_from_kyc())]
        pub fn exempt_account_from_kyc(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResult {
            T::CommitteeOrigin::ensure_origin(origin)?;

            ensure!(
                !KycExemptAccounts::<T>::contains_key(&account),
                Error::<T>::AccountAlreadyExempted
            );

            KycExemptAccounts::<T>::insert(&account, ());

            Self::deposit_event(Event::AccountExemptedFromKyc { account });
            Ok(())
        }

        /// 函数级详细中文注释：从KYC豁免列表移除账户
        ///
        /// # 参数
        /// - `origin`: 调用者（委员会起源）
        /// - `account`: 要移除豁免的账户
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_kyc_exemption())]
        pub fn remove_kyc_exemption(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResult {
            T::CommitteeOrigin::ensure_origin(origin)?;

            ensure!(
                KycExemptAccounts::<T>::contains_key(&account),
                Error::<T>::AccountNotExempted
            );

            KycExemptAccounts::<T>::remove(&account);

            Self::deposit_event(Event::AccountRemovedFromKycExemption { account });
            Ok(())
        }
    }
    
    // ===== 内部实现 =====
    
    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：创建OTC订单
        /// 
        /// ## 功能说明
        /// 1. 验证做市商存在且激活
        /// 2. 获取当前DUST/USD价格
        /// 3. 计算订单总金额
        /// 4. 将做市商的DUST锁定到托管
        /// 5. 创建订单记录
        /// 6. 更新买家和做市商的订单列表
        /// 7. 发出订单创建事件
        /// 
        /// ## 参数
        /// - `buyer`: 买家账户
        /// - `maker_id`: 做市商ID
        /// - `dust_amount`: 购买的DUST数量
        /// - `payment_commit`: 支付承诺哈希
        /// - `contact_commit`: 联系方式承诺哈希
        /// 
        /// ## 返回
        /// - `Ok(order_id)`: 订单ID
        /// - `Err(...)`: 各种错误情况
        pub fn do_create_order(
            buyer: &T::AccountId,
            maker_id: u64,
            dust_amount: BalanceOf<T>,
            payment_commit: H256,
            contact_commit: H256,
        ) -> Result<u64, DispatchError> {
            use pallet_trading_credit::quota::BuyerQuotaInterface;

            // 🆕 Step 0: KYC验证检查
            Self::enforce_kyc_requirement(buyer)?;

            // 1. 验证订单金额（新增）
            let _usd_amount = Self::validate_order_amount(dust_amount, false)?;

            // 2. 查询做市商信息
            let maker_app = T::MakerPallet::get_maker_application(maker_id)
                .ok_or(Error::<T>::MakerNotFound)?;
            
            // 2. 验证做市商状态
            ensure!(maker_app.is_active, Error::<T>::MakerNotActive);
            
            // 3. 获取当前DUST/USD价格
            let price = T::Pricing::get_dust_to_usd_rate()
                .ok_or(Error::<T>::PricingUnavailable)?;
            
            // 4. 计算总金额（USDT）= dust_amount * price
            let amount = dust_amount
                .checked_mul(&price)
                .ok_or(Error::<T>::CalculationOverflow)?;

            // 🆕 方案C+：买家额度检查和占用
            // 5. 计算订单USD金额（精度10^6）
            let amount_usd: u64 = Self::calculate_usd_amount_from_dust(dust_amount, price)?;

            // 6. 检查并占用买家额度
            T::Credit::occupy_quota(buyer, amount_usd)?;

            // 7. 获取做市商的TRON收款地址
            let maker_tron_address = maker_app.tron_address
                .try_into()
                .map_err(|_| Error::<T>::EncodingError)?;

            // 8. 获取订单ID（提前）
            let order_id = NextOrderId::<T>::get();

            // 9. 将做市商的DUST锁定到托管（使用 order_id 作为托管 ID）
            T::Escrow::lock_from(
                &maker_app.account,
                order_id,
                dust_amount,
            )?;

            // 10. 获取当前时间并计算超时时间
            let now = T::Timestamp::now().as_secs().saturated_into::<u64>();
            let expire_at = now
                .checked_add(T::OrderTimeout::get())
                .ok_or(Error::<T>::CalculationOverflow)?;
            let evidence_until = now
                .checked_add(T::EvidenceWindow::get())
                .ok_or(Error::<T>::CalculationOverflow)?;

            // 11. 创建订单记录
            let order = Order {
                maker_id,
                maker: maker_app.account.clone(),
                taker: buyer.clone(),
                price,
                qty: dust_amount,
                amount,
                created_at: now,
                expire_at,
                evidence_until,
                maker_tron_address,
                payment_commit,
                contact_commit,
                state: OrderState::Created,
                epay_trade_no: None,
                completed_at: None,
                is_first_purchase: false,
            };

            // 12. 保存订单
            Orders::<T>::insert(order_id, order);
            NextOrderId::<T>::put(order_id + 1);

            // 13. 更新买家订单列表
            BuyerOrders::<T>::try_mutate(buyer, |orders| {
                orders.try_push(order_id)
                    .map_err(|_| Error::<T>::TooManyOrders)
            })?;

            // 14. 更新做市商订单列表
            MakerOrders::<T>::try_mutate(maker_id, |orders| {
                orders.try_push(order_id)
                    .map_err(|_| Error::<T>::TooManyOrders)
            })?;

            // 15. 发出事件
            Self::deposit_event(Event::OrderCreated {
                order_id,
                maker_id,
                buyer: buyer.clone(),
                dust_amount,
                is_first_purchase: false,
            });

            // 16. 🆕 2025-11-28: 授予买卖双方聊天权限
            // 订单创建后，买家和做市商之间自动获得基于订单场景的聊天权限
            // 有效期：30天（30 * 24 * 60 * 10 个区块，假设 6 秒/区块）
            let chat_duration = 30u32 * 24 * 60 * 10; // 30天
            let order_metadata = sp_std::vec::Vec::from(
                alloc::format!("OTC订单#{}", order_id).as_bytes()
            );
            let _ = T::ChatPermission::grant_bidirectional_scene_authorization(
                *b"otc_ordr",
                buyer,
                &maker_app.account,
                pallet_chat_permission::SceneType::Order,
                pallet_chat_permission::SceneId::Numeric(order_id),
                Some(chat_duration.into()),
                order_metadata,
            );

            Ok(order_id)
        }
        
        /// 函数级详细中文注释：创建首购订单
        /// 
        /// ## 功能说明
        /// 1. 验证买家未进行过首购
        /// 2. 验证做市商首购配额未用完
        /// 3. 获取当前DUST/USD价格
        /// 4. 根据固定USD价值计算DUST数量
        /// 5. 验证DUST数量在合理范围内
        /// 6. 创建首购订单
        /// 
        /// ## 参数
        /// - `buyer`: 买家账户
        /// - `maker_id`: 做市商ID
        /// - `payment_commit`: 支付承诺哈希
        /// - `contact_commit`: 联系方式承诺哈希
        /// 
        /// ## 返回
        /// - `Ok(order_id)`: 订单ID
        /// - `Err(...)`: 各种错误情况
        pub fn do_create_first_purchase(
            buyer: &T::AccountId,
            maker_id: u64,
            payment_commit: H256,
            contact_commit: H256,
        ) -> Result<u64, DispatchError> {
            // 🆕 Step 0: KYC验证检查
            Self::enforce_kyc_requirement(buyer)?;

            // 1. 检查买家是否已首购
            ensure!(
                !HasFirstPurchased::<T>::get(buyer),
                Error::<T>::AlreadyFirstPurchased
            );
            
            // 2. 查询做市商信息
            let maker_app = T::MakerPallet::get_maker_application(maker_id)
                .ok_or(Error::<T>::MakerNotFound)?;
            
            // 3. 验证做市商状态
            ensure!(maker_app.is_active, Error::<T>::MakerNotActive);
            
            // 4. 检查做市商首购配额
            let current_count = MakerFirstPurchaseCount::<T>::get(maker_id);
            ensure!(
                current_count < T::MaxFirstPurchaseOrdersPerMaker::get(),
                Error::<T>::FirstPurchaseQuotaExhausted
            );
            
            // 5. 获取当前DUST/USD价格
            let price = T::Pricing::get_dust_to_usd_rate()
                .ok_or(Error::<T>::PricingUnavailable)?;
            
            // 6. 计算DUST数量
            // USD价值 / 价格 = DUST数量
            // 注意：price 是 USDT/DUST，所以需要除法
            let usd_value = T::FirstPurchaseUsdValue::get();
            let price_u128 = TryInto::<u128>::try_into(price)
                .map_err(|_| Error::<T>::CalculationOverflow)?;
            
            ensure!(price_u128 > 0, Error::<T>::InvalidPrice);
            
            // dust_amount = usd_value * 10^12 / price (考虑精度)
            let dust_amount_u128 = usd_value
                .checked_mul(1_000_000_000_000) // 10^12 (DUST精度)
                .and_then(|v| v.checked_div(price_u128))
                .ok_or(Error::<T>::CalculationOverflow)?;
            
            let dust_amount: BalanceOf<T> = TryInto::<u128>::try_into(dust_amount_u128)
                .ok()
                .and_then(|v| TryInto::<BalanceOf<T>>::try_into(v).ok())
                .ok_or(Error::<T>::CalculationOverflow)?;
            
            // 7. 验证DUST数量在合理范围内
            ensure!(
                dust_amount >= T::MinFirstPurchaseDustAmount::get(),
                Error::<T>::InvalidPrice
            );
            ensure!(
                dust_amount <= T::MaxFirstPurchaseDustAmount::get(),
                Error::<T>::InvalidPrice
            );
            
            // 8. 验证做市商余额
            let maker_balance = <T as Config>::Currency::free_balance(&maker_app.account);
            ensure!(
                maker_balance >= dust_amount,
                Error::<T>::MakerInsufficientBalance
            );
            
            // 9. 获取做市商的TRON收款地址
            let maker_tron_address = maker_app.tron_address
                .try_into()
                .map_err(|_| Error::<T>::EncodingError)?;
            
            // 10. 获取订单ID（提前）
            let order_id = NextOrderId::<T>::get();
            
            // 11. 将做市商的DUST锁定到托管（使用 order_id 作为托管 ID）
            T::Escrow::lock_from(
                &maker_app.account,
                order_id,
                dust_amount,
            )?;
            
            // 12. 获取当前时间并计算超时时间
            let now = T::Timestamp::now().as_secs().saturated_into::<u64>();
            let expire_at = now
                .checked_add(T::OrderTimeout::get())
                .ok_or(Error::<T>::CalculationOverflow)?;
            let evidence_until = now
                .checked_add(T::EvidenceWindow::get())
                .ok_or(Error::<T>::CalculationOverflow)?;
            
            // 13. 创建订单记录
            let amount = usd_value
                .try_into()
                .map_err(|_| Error::<T>::CalculationOverflow)?;
            
            let order = Order {
                maker_id,
                maker: maker_app.account.clone(),
                taker: buyer.clone(),
                price,
                qty: dust_amount,
                amount,
                created_at: now,
                expire_at,
                evidence_until,
                maker_tron_address,
                payment_commit,
                contact_commit,
                state: OrderState::Created,
                epay_trade_no: None,
                completed_at: None,
                is_first_purchase: true,
            };
            
            // 14. 保存订单
            Orders::<T>::insert(order_id, order);
            NextOrderId::<T>::put(order_id + 1);
            
            // 15. 更新买家订单列表
            BuyerOrders::<T>::try_mutate(buyer, |orders| {
                orders.try_push(order_id)
                    .map_err(|_| Error::<T>::TooManyOrders)
            })?;
            
            // 16. 更新做市商订单列表
            MakerOrders::<T>::try_mutate(maker_id, |orders| {
                orders.try_push(order_id)
                    .map_err(|_| Error::<T>::TooManyOrders)
            })?;
            
            // 17. 更新做市商首购计数和列表
            MakerFirstPurchaseCount::<T>::mutate(maker_id, |count| {
                *count = count.saturating_add(1);
            });
            
            MakerFirstPurchaseOrders::<T>::try_mutate(maker_id, |orders| {
                orders.try_push(order_id)
                    .map_err(|_| Error::<T>::StorageLimitReached)
            })?;
            
            // 18. 发出事件
            Self::deposit_event(Event::FirstPurchaseOrderCreated {
                order_id,
                buyer: buyer.clone(),
                maker_id,
                usd_value,
                dust_amount,
            });

            // 19. 🆕 2025-11-28: 授予买卖双方聊天权限
            // 首购订单创建后，买家和做市商之间自动获得基于订单场景的聊天权限
            // 有效期：30天（30 * 24 * 60 * 10 个区块，假设 6 秒/区块）
            let chat_duration = 30u32 * 24 * 60 * 10; // 30天
            let order_metadata = sp_std::vec::Vec::from(
                alloc::format!("首购订单#{}", order_id).as_bytes()
            );
            let _ = T::ChatPermission::grant_bidirectional_scene_authorization(
                *b"otc_ordr",
                buyer,
                &maker_app.account,
                pallet_chat_permission::SceneType::Order,
                pallet_chat_permission::SceneId::Numeric(order_id),
                Some(chat_duration.into()),
                order_metadata,
            );

            Ok(order_id)
        }
        
        /// 函数级详细中文注释：买家标记已付款
        /// 
        /// ## 功能说明
        /// 1. 验证订单存在且状态为 Created
        /// 2. 验证调用者是订单买家
        /// 3. 如提供TRON交易哈希，验证未被使用
        /// 4. 更新订单状态为 PaidOrCommitted
        /// 5. 记录TRON交易哈希（如有）
        /// 6. 发出状态变更事件
        /// 
        /// ## 参数
        /// - `buyer`: 买家账户
        /// - `order_id`: 订单ID
        /// - `tron_tx_hash`: TRON交易哈希（可选）
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 各种错误情况
        pub fn do_mark_paid(
            buyer: &T::AccountId,
            order_id: u64,
            tron_tx_hash: Option<sp_std::vec::Vec<u8>>,
        ) -> DispatchResult {
            // 1. 获取订单
            let mut order = Orders::<T>::get(order_id)
                .ok_or(Error::<T>::OrderNotFound)?;
            
            // 2. 验证订单状态
            ensure!(
                matches!(order.state, OrderState::Created),
                Error::<T>::InvalidOrderStatus
            );
            
            // 3. 验证调用者是买家
            ensure!(order.taker == *buyer, Error::<T>::NotAuthorized);
            
            // 4. 如提供TRON交易哈希，验证并记录
            if let Some(tx_hash_vec) = tron_tx_hash {
                // 将 Vec<u8> 转换为 H256
                ensure!(tx_hash_vec.len() == 32, Error::<T>::EncodingError);
                let mut hash_bytes = [0u8; 32];
                hash_bytes.copy_from_slice(&tx_hash_vec);
                let tx_hash = H256::from(hash_bytes);
                
                // 检查是否已使用
                ensure!(
                    !TronTxUsed::<T>::contains_key(tx_hash),
                    Error::<T>::TronTxHashAlreadyUsed
                );
                
                // 记录使用
                let current_block = frame_system::Pallet::<T>::block_number();
                TronTxUsed::<T>::insert(tx_hash, current_block);
                
                // 添加到清理队列
                TronTxQueue::<T>::try_mutate(|queue| {
                    queue.try_push((tx_hash, current_block))
                        .map_err(|_| Error::<T>::StorageLimitReached)
                })?;
                
                Self::deposit_event(Event::TronTxHashRecorded { tx_hash });
            }
            
            // 5. 更新订单状态
            let old_state = order.state.clone();
            order.state = OrderState::PaidOrCommitted;
            Orders::<T>::insert(order_id, order);
            
            // 6. 发出事件
            Self::deposit_event(Event::OrderStateChanged {
                order_id,
                old_state: Self::state_to_u8(&old_state),
                new_state: Self::state_to_u8(&OrderState::PaidOrCommitted),
                actor: Some(buyer.clone()),
            });
            
            Ok(())
        }
        
        /// 函数级详细中文注释：做市商释放DUST
        /// 
        /// ## 功能说明
        /// 1. 验证订单存在且状态为 PaidOrCommitted
        /// 2. 验证调用者是订单做市商
        /// 3. 从托管释放DUST到买家
        /// 4. 更新订单状态为 Released
        /// 5. 更新信用记录
        /// 6. 更新首购状态（如是首购订单）
        /// 7. 发出状态变更事件
        /// 
        /// ## 参数
        /// - `maker`: 做市商账户
        /// - `order_id`: 订单ID
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 各种错误情况
        pub fn do_release_dust(
            maker: &T::AccountId,
            order_id: u64,
        ) -> DispatchResult {
            use pallet_trading_credit::quota::BuyerQuotaInterface;

            // 1. 获取订单
            let mut order = Orders::<T>::get(order_id)
                .ok_or(Error::<T>::OrderNotFound)?;
            
            // 2. 验证订单状态
            ensure!(
                matches!(order.state, OrderState::PaidOrCommitted),
                Error::<T>::InvalidOrderStatus
            );
            
            // 3. 验证调用者是做市商
            ensure!(order.maker == *maker, Error::<T>::NotAuthorized);
            
            // 4. 从托管释放DUST到买家（使用 order_id 作为托管 ID）
            T::Escrow::release_all(order_id, &order.taker)?;
            
            // 5. 更新订单状态
            let old_state = order.state.clone();
            order.state = OrderState::Released;
            let now = T::Timestamp::now().as_secs().saturated_into::<u64>();
            order.completed_at = Some(now);
            Orders::<T>::insert(order_id, order.clone());
            
            // 6. 记录做市商订单完成到信用分 ✅
            let response_time_seconds = now.saturating_sub(order.created_at) as u32;
            let _ = T::MakerCredit::record_maker_order_completed(
                order.maker_id,
                order_id,
                response_time_seconds,
            );

            // 🆕 方案C+：买家额度管理
            // 7. 释放买家占用的额度
            let amount_usd: u64 = Self::calculate_usd_amount_from_dust(order.qty, order.price)?;
            let _ = T::Credit::release_quota(&order.taker, amount_usd);

            // 8. 记录订单完成，提升买家信用分
            let _ = T::Credit::record_order_completed(&order.taker, order_id);

            // 9. 如是首购订单，更新首购状态
            if order.is_first_purchase {
                HasFirstPurchased::<T>::insert(&order.taker, true);

                // 减少做市商首购订单计数
                MakerFirstPurchaseCount::<T>::mutate(order.maker_id, |count| {
                    *count = count.saturating_sub(1);
                });
            }

            // 10. 发出事件
            Self::deposit_event(Event::OrderStateChanged {
                order_id,
                old_state: Self::state_to_u8(&old_state),
                new_state: Self::state_to_u8(&OrderState::Released),
                actor: Some(maker.clone()),
            });
            
            Ok(())
        }
        
        /// 函数级详细中文注释：取消订单
        /// 
        /// ## 功能说明
        /// 1. 验证订单存在
        /// 2. 验证调用者权限（买家或做市商）
        /// 3. 验证订单状态可以取消
        /// 4. 从托管退还DUST给做市商
        /// 5. 更新订单状态为 Canceled
        /// 6. 发出状态变更事件
        /// 
        /// ## 参数
        /// - `who`: 调用者账户（买家或做市商）
        /// - `order_id`: 订单ID
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 各种错误情况
        pub fn do_cancel_order(
            who: &T::AccountId,
            order_id: u64,
        ) -> DispatchResult {
            use pallet_trading_credit::quota::BuyerQuotaInterface;

            // 1. 获取订单
            let mut order = Orders::<T>::get(order_id)
                .ok_or(Error::<T>::OrderNotFound)?;
            
            // 2. 验证调用者是买家或做市商
            ensure!(
                order.taker == *who || order.maker == *who,
                Error::<T>::NotAuthorized
            );
            
            // 3. 验证订单状态（只有 Created 和 Expired 状态可以取消）
            ensure!(
                matches!(order.state, OrderState::Created | OrderState::Expired),
                Error::<T>::InvalidOrderStatus
            );
            
            // 4. 从托管退还DUST给做市商（使用 order_id 作为托管 ID）
            T::Escrow::refund_all(order_id, &order.maker)?;
            
            // 5. 更新订单状态
            let old_state = order.state.clone();
            order.state = OrderState::Canceled;
            let now = T::Timestamp::now().as_secs().saturated_into::<u64>();
            order.completed_at = Some(now);
            Orders::<T>::insert(order_id, order.clone());

            // 🆕 方案C+：买家额度管理
            // 6. 释放买家占用的额度
            let amount_usd: u64 = Self::calculate_usd_amount_from_dust(order.qty, order.price)?;
            let _ = T::Credit::release_quota(&order.taker, amount_usd);

            // 7. 记录订单取消（轻度降低信用）
            let _ = T::Credit::record_order_cancelled(&order.taker, order_id);

            // 8. 如是首购订单，减少做市商首购计数
            if order.is_first_purchase {
                MakerFirstPurchaseCount::<T>::mutate(order.maker_id, |count| {
                    *count = count.saturating_sub(1);
                });
            }

            // 9. 发出事件
            Self::deposit_event(Event::OrderStateChanged {
                order_id,
                old_state: Self::state_to_u8(&old_state),
                new_state: Self::state_to_u8(&OrderState::Canceled),
                actor: Some(who.clone()),
            });
            
            Ok(())
        }
        
        /// 函数级详细中文注释：发起订单争议
        /// 
        /// ## 功能说明
        /// 1. 验证订单存在
        /// 2. 验证调用者权限（买家或做市商）
        /// 3. 验证订单状态可以争议
        /// 4. 更新订单状态为 Disputed
        /// 5. 发出状态变更事件
        /// 
        /// ## 参数
        /// - `who`: 调用者账户（买家或做市商）
        /// - `order_id`: 订单ID
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 各种错误情况
        pub fn do_dispute_order(
            who: &T::AccountId,
            order_id: u64,
        ) -> DispatchResult {
            // 1. 获取订单
            let mut order = Orders::<T>::get(order_id)
                .ok_or(Error::<T>::OrderNotFound)?;
            
            // 2. 验证调用者是买家或做市商
            ensure!(
                order.taker == *who || order.maker == *who,
                Error::<T>::NotAuthorized
            );
            
            // 3. 验证订单状态（只有 PaidOrCommitted 状态可以发起争议）
            ensure!(
                matches!(order.state, OrderState::PaidOrCommitted),
                Error::<T>::InvalidOrderStatus
            );
            
            // 4. 更新订单状态
            let old_state = order.state.clone();
            order.state = OrderState::Disputed;
            Orders::<T>::insert(order_id, order);
            
            // 5. 发出事件
            Self::deposit_event(Event::OrderStateChanged {
                order_id,
                old_state: Self::state_to_u8(&old_state),
                new_state: Self::state_to_u8(&OrderState::Disputed),
                actor: Some(who.clone()),
            });
            
            Ok(())
        }
    }
    
    // ===== 公共查询接口 =====
    
    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：检查买家是否已首购
        pub fn has_user_first_purchased(who: &T::AccountId) -> bool {
            HasFirstPurchased::<T>::get(who)
        }
        
        /// 函数级详细中文注释：获取做市商首购订单数量
        pub fn get_maker_first_purchase_count(maker_id: u64) -> u32 {
            MakerFirstPurchaseCount::<T>::get(maker_id)
        }
        
        /// 函数级详细中文注释：将订单状态转换为 u8（用于事件）
        fn state_to_u8(state: &OrderState) -> u8 {
            match state {
                OrderState::Created => 0,
                OrderState::PaidOrCommitted => 1,
                OrderState::Released => 2,
                OrderState::Refunded => 3,
                OrderState::Canceled => 4,
                OrderState::Disputed => 5,
                OrderState::Closed => 6,
                OrderState::Expired => 7,
            }
        }
        
        // ===== 仲裁支持接口 =====
        
        /// 函数级详细中文注释：检查用户是否有权对订单发起争议
        /// 
        /// ## 权限规则
        /// - 买家（taker）：可以对自己的订单发起争议
        /// - 做市商（maker）：可以对自己参与的订单发起争议
        /// 
        /// ## 参数
        /// - `who`: 发起争议的用户
        /// - `order_id`: 订单ID
        /// 
        /// ## 返回
        /// - `true`: 有权发起争议
        /// - `false`: 无权发起争议
        pub fn can_dispute_order(who: &T::AccountId, order_id: u64) -> bool {
            if let Some(order) = Orders::<T>::get(order_id) {
                // 买家或做市商都可以发起争议
                &order.taker == who || &order.maker == who
            } else {
                false
            }
        }
        
        /// 函数级详细中文注释：应用仲裁裁决到订单
        /// 
        /// ## 裁决类型
        /// - Release: 全额放款给做市商（买家败诉）
        /// - Refund: 全额退款给买家（做市商败诉）
        /// - Partial(bps): 按比例分账（双方都有责任）
        /// 
        /// ## 参数
        /// - `order_id`: 订单ID
        /// - `decision`: 仲裁裁决
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 失败
        pub fn apply_arbitration_decision(
            order_id: u64,
            decision: pallet_arbitration::pallet::Decision,
        ) -> DispatchResult {
            // 获取订单记录
            let mut order = Orders::<T>::get(order_id)
                .ok_or(Error::<T>::OrderNotFound)?;
            
            // 确保状态是 Disputed（争议中）
            ensure!(
                order.state == OrderState::Disputed,
                Error::<T>::InvalidOrderStatus
            );
            
            // 根据裁决类型执行相应操作
            use pallet_arbitration::pallet::Decision;
            let maker_win = match decision {
                Decision::Release => {
                    // 放款给做市商（买家败诉）
                    T::Escrow::release_all(order_id, &order.maker)?;
                    order.state = OrderState::Released;
                    true  // 做市商胜诉
                },
                Decision::Refund => {
                    // 退款给买家（做市商败诉）
                    T::Escrow::refund_all(order_id, &order.taker)?;
                    order.state = OrderState::Refunded;
                    false  // 做市商败诉
                },
                Decision::Partial(_bps) => {
                    // 按比例分账
                    // TODO: pallet-escrow 暂未实现 split_partial 方法
                    // 暂时当作 Refund 处理（退款给买家）
                    T::Escrow::refund_all(order_id, &order.taker)?;
                    order.state = OrderState::Refunded;
                    false  // 做市商败诉
                },
            };
            
            // 记录争议结果到信用分 ✅
            let _ = T::MakerCredit::record_maker_dispute_result(
                order.maker_id,
                order_id,
                maker_win,
            );
            
            // 更新订单
            order.completed_at = Some(T::Timestamp::now().as_secs());
            Orders::<T>::insert(order_id, order);
            
            Ok(())
        }

        // ===== 新增：订单金额验证逻辑 =====

        /// 函数级详细中文注释：验证订单金额是否符合限制
        ///
        /// # 参数
        /// - dust_amount: 购买的DUST数量
        /// - is_first_purchase: 是否为首购订单
        ///
        /// # 返回
        /// - Ok(usd_amount): 验证通过，返回对应的USD金额
        /// - Err(DispatchError): 验证失败
        pub fn validate_order_amount(
            dust_amount: BalanceOf<T>,
            is_first_purchase: bool,
        ) -> Result<u64, DispatchError> {
            // 首购订单使用固定价格，无需验证限额
            if is_first_purchase {
                return Ok(T::FirstPurchaseUsdAmount::get());
            }

            // 获取当前DUST/USD价格
            let dust_to_usd_rate = T::Pricing::get_dust_to_usd_rate()
                .ok_or(Error::<T>::PricingServiceUnavailable)?;

            // 计算订单的USD金额
            let usd_amount = Self::calculate_usd_amount_from_dust(
                dust_amount,
                dust_to_usd_rate,
            )?;

            // 验证最小金额（至少20 USD，首购除外）
            ensure!(
                usd_amount >= T::MinOrderUsdAmount::get(),
                Error::<T>::OrderAmountTooSmall
            );

            // 验证是否超过最大限制
            let max_amount = T::MaxOrderUsdAmount::get();
            ensure!(
                usd_amount <= max_amount,
                Error::<T>::OrderAmountExceedsLimit
            );

            Ok(usd_amount)
        }

        /// 函数级详细中文注释：计算DUST对应的USD金额
        ///
        /// # 参数
        /// - dust_amount: DUST数量
        /// - dust_to_usd_rate: DUST/USD汇率
        ///
        /// # 返回
        /// - Ok(u64): USD金额（精度10^6）
        /// - Err(DispatchError): 计算错误
        fn calculate_usd_amount_from_dust(
            dust_amount: BalanceOf<T>,
            dust_to_usd_rate: BalanceOf<T>,
        ) -> Result<u64, DispatchError> {
            // 转换为u128进行高精度计算
            let dust_u128: u128 = dust_amount.saturated_into();
            let rate_u128: u128 = dust_to_usd_rate.saturated_into();

            // 计算USD金额 = DUST数量 × DUST/USD汇率 ÷ DUST精度
            // DUST精度为10^12，USD精度为10^6
            let usd_u128 = dust_u128
                .checked_mul(rate_u128)
                .ok_or(Error::<T>::AmountCalculationOverflow)?
                .checked_div(1_000_000_000_000u128) // 除以DUST精度10^12
                .ok_or(Error::<T>::AmountCalculationOverflow)?;

            // 验证结果是否在u64范围内
            let usd_amount: u64 = usd_u128
                .try_into()
                .map_err(|_| Error::<T>::AmountCalculationOverflow)?;

            Ok(usd_amount)
        }

        /// 函数级详细中文注释：计算指定USD金额对应的最大DUST数量
        ///
        /// # 参数
        /// - usd_amount: USD金额（精度10^6）
        ///
        /// # 返回
        /// - Ok(BalanceOf<T>): 对应的DUST数量
        /// - Err(DispatchError): 计算错误
        pub fn calculate_max_dust_for_usd_amount(
            usd_amount: u64,
        ) -> Result<BalanceOf<T>, DispatchError> {
            // 获取当前DUST/USD价格
            let dust_to_usd_rate = T::Pricing::get_dust_to_usd_rate()
                .ok_or(Error::<T>::PricingServiceUnavailable)?;

            // 计算DUST数量 = USD金额 × DUST精度 ÷ DUST/USD汇率
            let usd_u128 = usd_amount as u128;
            let rate_u128: u128 = dust_to_usd_rate.saturated_into();

            let dust_u128 = usd_u128
                .checked_mul(1_000_000_000_000u128) // 乘以DUST精度10^12
                .ok_or(Error::<T>::AmountCalculationOverflow)?
                .checked_div(rate_u128)
                .ok_or(Error::<T>::AmountCalculationOverflow)?;

            // 转换为BalanceOf<T>
            let dust_amount: BalanceOf<T> = dust_u128
                .try_into()
                .map_err(|_| Error::<T>::AmountCalculationOverflow)?;

            Ok(dust_amount)
        }

        /// 函数级详细中文注释：查询当前最大可购买DUST数量
        ///
        /// # 返回
        /// - Ok(BalanceOf<T>): 当前价格下最大可购买的DUST数量
        /// - Err(DispatchError): 查询失败
        pub fn get_max_purchasable_dust() -> Result<BalanceOf<T>, DispatchError> {
            Self::calculate_max_dust_for_usd_amount(T::MaxOrderUsdAmount::get())
        }

        /// 函数级详细中文注释：查询指定DUST数量对应的USD金额
        ///
        /// # 参数
        /// - dust_amount: DUST数量
        ///
        /// # 返回
        /// - Ok(u64): 对应的USD金额
        /// - Err(DispatchError): 查询失败
        pub fn get_usd_amount_for_dust(
            dust_amount: BalanceOf<T>
        ) -> Result<u64, DispatchError> {
            let dust_to_usd_rate = T::Pricing::get_dust_to_usd_rate()
                .ok_or(Error::<T>::PricingServiceUnavailable)?;

            Self::calculate_usd_amount_from_dust(dust_amount, dust_to_usd_rate)
        }

        /// 函数级详细中文注释：检查指定DUST数量是否符合订单限制
        ///
        /// # 参数
        /// - dust_amount: 要检查的DUST数量
        ///
        /// # 返回
        /// - true: 符合限制
        /// - false: 超过限制
        pub fn is_dust_amount_valid(dust_amount: BalanceOf<T>) -> bool {
            Self::validate_order_amount(dust_amount, false).is_ok()
        }
    }
}
