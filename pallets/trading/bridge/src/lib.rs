#![cfg_attr(not(feature = "std"), no_std)]

//! # Bridge Pallet (桥接模块)
//!
//! ## 概述
//!
//! 本模块负责 DUST ↔ USDT 桥接服务，包括：
//! - 官方桥接（治理管理）
//! - 做市商桥接（市场化服务）
//! - OCW 自动验证
//! - 超时退款机制
//!
//! ## 版本历史
//!
//! - v0.1.0 (2025-11-03): 从 pallet-trading 拆分而来

pub use pallet::*;

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
        traits::{Currency, Get},
        BoundedVec,
        sp_runtime::{SaturatedConversion, traits::Saturating},
    };
    use pallet_escrow::Escrow as EscrowTrait;
    
    /// 函数级详细中文注释：Balance 类型别名
    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    
    /// 函数级详细中文注释：TRON 地址类型（固定 34 字节）
    pub type TronAddress = BoundedVec<u8, ConstU32<34>>;
    
    /// 函数级详细中文注释：价格提供者接口
    /// 用于获取 DUST/USD 实时汇率
    pub trait PricingProvider<Balance> {
        /// 获取 DUST/USD 汇率（精度 10^6）
        /// 返回：Some(汇率) 或 None（价格不可用）
        fn get_dust_to_usd_rate() -> Option<Balance>;
    }
    
    /// 函数级详细中文注释：Maker Pallet 接口
    pub trait MakerInterface<AccountId, Balance> {
        /// 查询做市商申请信息
        fn get_maker_application(maker_id: u64) -> Option<MakerApplicationInfo<AccountId, Balance>>;
        /// 检查做市商是否激活
        fn is_maker_active(maker_id: u64) -> bool;
        /// 获取做市商 ID（通过账户）
        fn get_maker_id(who: &AccountId) -> Option<u64>;
    }
    
    /// 函数级详细中文注释：Credit Pallet 接口
    pub trait CreditInterface {
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
    
    /// 函数级详细中文注释：做市商申请信息（简化版）
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(AccountId, Balance))]
    pub struct MakerApplicationInfo<AccountId, Balance> {
        pub account: AccountId,
        pub tron_address: BoundedVec<u8, ConstU32<34>>,
        pub is_active: bool,
        pub _phantom: sp_std::marker::PhantomData<Balance>,
    }
    
    // ===== 数据结构 =====
    
    /// 函数级详细中文注释：兑换状态枚举
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, RuntimeDebug)]
    pub enum SwapStatus {
        /// 待处理
        Pending,
        /// 已完成
        Completed,
        /// 用户举报
        UserReported,
        /// 仲裁中
        Arbitrating,
        /// 仲裁通过
        ArbitrationApproved,
        /// 仲裁拒绝
        ArbitrationRejected,
        /// 超时退款
        Refunded,
    }
    
    /// 函数级详细中文注释：官方桥接兑换请求
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct SwapRequest<T: Config> {
        /// 兑换ID
        pub id: u64,
        /// 用户地址
        pub user: T::AccountId,
        /// DUST 数量
        pub dust_amount: BalanceOf<T>,
        /// TRON 地址
        pub tron_address: TronAddress,
        /// 是否已完成
        pub completed: bool,
        /// 兑换时的 USDT 单价（精度 10^6）
        pub price_usdt: u64,
        /// 创建时间戳（区块号）
        pub created_at: BlockNumberFor<T>,
        /// 超时时间（区块号）
        pub expire_at: BlockNumberFor<T>,
    }
    
    /// 函数级详细中文注释：做市商兑换记录
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct MakerSwapRecord<T: Config> {
        /// 兑换ID
        pub swap_id: u64,
        /// 做市商ID
        pub maker_id: u64,
        /// 做市商账户
        pub maker: T::AccountId,
        /// 用户账户
        pub user: T::AccountId,
        /// DUST 数量
        pub dust_amount: BalanceOf<T>,
        /// USDT 金额（精度 10^6）
        pub usdt_amount: u64,
        /// USDT 接收地址
        pub usdt_address: TronAddress,
        /// 创建时间
        pub created_at: BlockNumberFor<T>,
        /// 超时时间
        pub timeout_at: BlockNumberFor<T>,
        /// TRC20 交易哈希
        pub trc20_tx_hash: Option<BoundedVec<u8, ConstU32<128>>>,
        /// 完成时间
        pub completed_at: Option<BlockNumberFor<T>>,
        /// 证据 CID
        pub evidence_cid: Option<BoundedVec<u8, ConstU32<256>>>,
        /// 兑换状态
        pub status: SwapStatus,
        /// 兑换价格（精度 10^6）
        pub price_usdt: u64,
    }
    
    #[pallet::pallet]
    pub struct Pallet<T>(_);
    
    /// 函数级详细中文注释：Bridge模块配置 trait
    #[pallet::config]
    /// 函数级中文注释：Bridge Pallet 配置 trait
    /// - 🔴 stable2506 API 变更：RuntimeEvent 自动继承，无需显式声明
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {
        
        /// 货币类型
        type Currency: Currency<Self::AccountId>;
        
        /// 托管服务接口
        type Escrow: pallet_escrow::Escrow<Self::AccountId, BalanceOf<Self>>;
        
        /// 价格提供者接口（用于获取 DUST/USD 汇率）
        type Pricing: PricingProvider<BalanceOf<Self>>;
        
        /// Maker Pallet 接口（用于验证做市商）
        type MakerPallet: MakerInterface<Self::AccountId, BalanceOf<Self>>;
        
        /// Credit Pallet 接口（用于记录信用分）
        type Credit: CreditInterface;
        
        /// 治理权限（用于官方桥接）
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        
        /// 官方兑换超时时间（区块数）
        #[pallet::constant]
        type SwapTimeout: Get<BlockNumberFor<Self>>;
        
        /// 做市商兑换超时时间（区块数，由OCW验证）
        #[pallet::constant]
        type OcwSwapTimeoutBlocks: Get<BlockNumberFor<Self>>;
        
        /// 最小兑换金额
        #[pallet::constant]
        type MinSwapAmount: Get<BalanceOf<Self>>;
        
        /// 权重信息
        type WeightInfo: WeightInfo;
    }
    
    // ===== 存储 =====
    
    /// 函数级详细中文注释：下一个兑换 ID
    #[pallet::storage]
    #[pallet::getter(fn next_swap_id)]
    pub type NextSwapId<T> = StorageValue<_, u64, ValueQuery>;
    
    /// 函数级详细中文注释：桥接账户（用于官方桥接）
    #[pallet::storage]
    #[pallet::getter(fn bridge_account)]
    pub type BridgeAccount<T: Config> = StorageValue<_, T::AccountId>;
    
    /// 函数级详细中文注释：官方兑换请求
    #[pallet::storage]
    #[pallet::getter(fn swap_requests)]
    pub type SwapRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // swap_id
        SwapRequest<T>,
    >;
    
    /// 函数级详细中文注释：做市商兑换记录
    #[pallet::storage]
    #[pallet::getter(fn maker_swaps)]
    pub type MakerSwaps<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // swap_id
        MakerSwapRecord<T>,
    >;
    
    /// 函数级详细中文注释：用户兑换列表
    #[pallet::storage]
    #[pallet::getter(fn user_swaps)]
    pub type UserSwaps<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<100>>,  // 每个用户最多100个兑换
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：做市商兑换列表
    #[pallet::storage]
    #[pallet::getter(fn maker_swap_list)]
    pub type MakerSwapList<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // maker_id
        BoundedVec<u64, ConstU32<1000>>,  // 每个做市商最多1000个兑换
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：已使用的 TRON 交易哈希（防止重放攻击）
    /// 
    /// ## 安全机制
    /// - 做市商完成兑换时提交 TRC20 交易哈希
    /// - 系统记录已使用的哈希，防止同一笔交易被重复使用
    /// - 这是防止重放攻击的关键安全措施
    /// 
    /// ## 存储结构
    /// - Key: TRON 交易哈希（最多 128 字节）
    /// - Value: () (仅用于标记存在)
    #[pallet::storage]
    #[pallet::getter(fn used_tron_tx_hashes)]
    pub type UsedTronTxHashes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, ConstU32<128>>,  // TRC20 tx hash
        (),
        OptionQuery,
    >;
    
    // ===== 事件 =====
    
    /// 函数级详细中文注释：Bridge模块事件
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 官方兑换已创建
        SwapCreated {
            swap_id: u64,
            user: T::AccountId,
            dust_amount: BalanceOf<T>,
        },
        /// 官方兑换已完成
        SwapCompleted {
            swap_id: u64,
            user: T::AccountId,
        },
        /// 兑换状态已变更
        SwapStateChanged {
            swap_id: u64,
            old_state: u8,
            new_state: u8,
        },
        /// 做市商兑换已创建
        MakerSwapCreated {
            swap_id: u64,
            maker_id: u64,
            user: T::AccountId,
            dust_amount: BalanceOf<T>,
        },
        /// 做市商兑换已完成
        MakerSwapCompleted {
            swap_id: u64,
            maker: T::AccountId,
        },
        /// 做市商兑换已标记完成
        MakerSwapMarkedComplete {
            swap_id: u64,
            maker_id: u64,
            trc20_tx_hash: BoundedVec<u8, ConstU32<128>>,
        },
        /// 用户举报兑换
        SwapReported {
            swap_id: u64,
            user: T::AccountId,
        },
        /// 桥接账户已设置
        BridgeAccountSet {
            account: T::AccountId,
        },
    }
    
    // ===== 错误 =====
    
    /// 函数级详细中文注释：Bridge模块错误
    #[pallet::error]
    pub enum Error<T> {
        /// 兑换不存在
        SwapNotFound,
        /// 做市商不存在
        MakerNotFound,
        /// 做市商未激活
        MakerNotActive,
        /// 兑换状态不正确
        InvalidSwapStatus,
        /// 未授权
        NotAuthorized,
        /// 编码错误
        EncodingError,
        /// 存储限制已达到
        StorageLimitReached,
        /// 兑换金额太低
        SwapAmountTooLow,
        /// 无效的 TRON 地址
        InvalidTronAddress,
        /// 桥接账户未设置
        BridgeAccountNotSet,
        /// 兑换已完成
        AlreadyCompleted,
        /// 不是做市商
        NotMaker,
        /// 状态无效
        InvalidStatus,
        /// 交易哈希无效
        InvalidTxHash,
        /// 兑换太多
        TooManySwaps,
        /// 低于最小金额
        BelowMinimumAmount,
        /// 地址无效
        InvalidAddress,
        /// 不是兑换的用户
        NotSwapUser,
        /// 无法举报
        CannotReport,
        /// 价格不可用
        PriceNotAvailable,
        /// 金额溢出
        AmountOverflow,
        /// USDT金额太小
        UsdtAmountTooSmall,
        /// TRON 交易哈希已被使用（防止重放攻击）
        TronTxHashAlreadyUsed,
    }
    
    // ===== Extrinsics =====
    
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：创建官方桥接兑换
        ///
        /// # 参数
        /// - `origin`: 调用者（用户，必须是签名账户）
        /// - `dust_amount`: DUST数量
        /// - `tron_address`: TRON接收地址
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::swap())]
        pub fn swap(
            origin: OriginFor<T>,
            dust_amount: BalanceOf<T>,
            tron_address: sp_std::vec::Vec<u8>,
        ) -> DispatchResult {
            let user = ensure_signed(origin)?;
            let _swap_id = Self::do_swap(&user, dust_amount, tron_address)?;
            Ok(())
        }
        
        /// 函数级详细中文注释：完成官方桥接兑换（治理功能）
        ///
        /// # 参数
        /// - `origin`: 调用者（必须是治理权限）
        /// - `swap_id`: 兑换ID
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::swap())]
        pub fn complete_swap(
            origin: OriginFor<T>,
            swap_id: u64,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;
            Self::do_complete_swap(swap_id)
        }
        
        /// 函数级详细中文注释：创建做市商桥接兑换
        ///
        /// # 参数
        /// - `origin`: 调用者（用户，必须是签名账户）
        /// - `maker_id`: 做市商ID
        /// - `dust_amount`: DUST数量
        /// - `usdt_address`: USDT接收地址
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::swap())]
        pub fn maker_swap(
            origin: OriginFor<T>,
            maker_id: u64,
            dust_amount: BalanceOf<T>,
            usdt_address: sp_std::vec::Vec<u8>,
        ) -> DispatchResult {
            let user = ensure_signed(origin)?;
            let _swap_id = Self::do_maker_swap(&user, maker_id, dust_amount, usdt_address)?;
            Ok(())
        }
        
        /// 函数级详细中文注释：做市商标记兑换完成
        ///
        /// # 参数
        /// - `origin`: 调用者（做市商，必须是签名账户）
        /// - `swap_id`: 兑换ID
        /// - `trc20_tx_hash`: TRC20交易哈希
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::swap())]
        pub fn mark_swap_complete(
            origin: OriginFor<T>,
            swap_id: u64,
            trc20_tx_hash: sp_std::vec::Vec<u8>,
        ) -> DispatchResult {
            let maker = ensure_signed(origin)?;
            Self::do_mark_swap_complete(&maker, swap_id, trc20_tx_hash)
        }
        
        /// 函数级详细中文注释：用户举报做市商兑换
        ///
        /// # 参数
        /// - `origin`: 调用者（用户，必须是签名账户）
        /// - `swap_id`: 兑换ID
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::swap())]
        pub fn report_swap(
            origin: OriginFor<T>,
            swap_id: u64,
        ) -> DispatchResult {
            let user = ensure_signed(origin)?;
            Self::do_report_swap(&user, swap_id)
        }
        
        /// 函数级详细中文注释：设置桥接账户（治理功能）
        ///
        /// # 参数
        /// - `origin`: 调用者（必须是治理权限）
        /// - `account`: 桥接账户
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::swap())]
        pub fn set_bridge_account(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;
            BridgeAccount::<T>::put(account.clone());
            Self::deposit_event(Event::BridgeAccountSet { account });
            Ok(())
        }
    }
    
    // ===== 内部实现 =====
    
    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：创建官方桥接兑换
        /// 
        /// ## 功能说明
        /// 1. 验证兑换金额大于最小值
        /// 2. 验证 TRON 地址格式
        /// 3. 锁定用户的 DUST 到托管
        /// 4. 创建兑换请求
        /// 5. 等待治理账户处理
        /// 
        /// ## 参数
        /// - `user`: 用户账户
        /// - `dust_amount`: DUST 数量
        /// - `tron_address`: TRON 收款地址
        /// 
        /// ## 返回
        /// - `Ok(swap_id)`: 兑换ID
        /// - `Err(...)`: 各种错误情况
        pub fn do_swap(
            user: &T::AccountId,
            dust_amount: BalanceOf<T>,
            tron_address: sp_std::vec::Vec<u8>,
        ) -> Result<u64, DispatchError> {
            // 1. 验证最小兑换金额
            ensure!(
                dust_amount >= T::MinSwapAmount::get(),
                Error::<T>::BelowMinimumAmount
            );
            
            // 2. 验证 TRON 地址格式
            let tron_addr: TronAddress = tron_address
                .try_into()
                .map_err(|_| Error::<T>::InvalidAddress)?;
            
            // 3. 获取当前价格（从 PricingProvider 获取实时汇率）
            let price_balance = T::Pricing::get_dust_to_usd_rate()
                .ok_or(Error::<T>::PriceNotAvailable)?;
            let price_usdt: u64 = price_balance.saturated_into();
            
            // 4. 获取兑换ID
            let swap_id = NextSwapId::<T>::get();
            
            // 5. 锁定用户的 DUST 到托管
            T::Escrow::lock_from(
                user,
                swap_id,
                dust_amount,
            )?;
            
            // 6. 计算超时时间
            let current_block = frame_system::Pallet::<T>::block_number();
            let expire_at = current_block + T::SwapTimeout::get();
            
            // 7. 创建兑换请求
            let request = SwapRequest {
                id: swap_id,
                user: user.clone(),
                dust_amount,
                tron_address: tron_addr,
                completed: false,
                price_usdt,
                created_at: current_block,
                expire_at,
            };
            
            // 8. 保存请求
            SwapRequests::<T>::insert(swap_id, request);
            NextSwapId::<T>::put(swap_id + 1);
            
            // 9. 更新用户兑换列表
            UserSwaps::<T>::try_mutate(user, |swaps| {
                swaps.try_push(swap_id)
                    .map_err(|_| Error::<T>::TooManySwaps)
            })?;
            
            // 10. 发出事件
            Self::deposit_event(Event::SwapCreated {
                swap_id,
                user: user.clone(),
                dust_amount,
            });
            
            Ok(swap_id)
        }
        
        /// 函数级详细中文注释：完成官方桥接兑换
        /// 
        /// ## 功能说明
        /// 1. 验证兑换存在且未完成
        /// 2. 销毁托管的 DUST
        /// 3. 标记兑换为已完成
        /// 4. 发出完成事件
        /// 
        /// ## 参数
        /// - `swap_id`: 兑换ID
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 各种错误情况
        pub fn do_complete_swap(swap_id: u64) -> DispatchResult {
            // 1. 获取兑换请求
            let mut request = SwapRequests::<T>::get(swap_id)
                .ok_or(Error::<T>::SwapNotFound)?;
            
            // 2. 验证未完成
            ensure!(!request.completed, Error::<T>::AlreadyCompleted);
            
            // 3. 销毁托管的 DUST（官方桥接直接销毁，减少总供应量）
            // 注意：目前 pallet-escrow 没有 burn 方法，暂时使用释放到桥接账户
            // TODO: 在 pallet-escrow 中添加 burn() 方法以真正销毁代币
            let bridge_account = BridgeAccount::<T>::get()
                .ok_or(Error::<T>::BridgeAccountNotSet)?;
            
            T::Escrow::release_all(
                swap_id,
                &bridge_account,
            )?;
            
            // 4. 标记为完成
            request.completed = true;
            SwapRequests::<T>::insert(swap_id, request.clone());
            
            // 5. 发出事件
            Self::deposit_event(Event::SwapCompleted {
                swap_id,
                user: request.user,
            });
            
            Ok(())
        }
        
        /// 函数级详细中文注释：创建做市商兑换
        /// 
        /// ## 功能说明
        /// 1. 验证做市商存在且激活
        /// 2. 验证兑换金额大于最小值
        /// 3. 验证 USDT 地址格式
        /// 4. 锁定用户的 DUST 到托管
        /// 5. 创建做市商兑换记录
        /// 6. 等待做市商转账 USDT
        /// 
        /// ## 参数
        /// - `user`: 用户账户
        /// - `maker_id`: 做市商ID
        /// - `dust_amount`: DUST 数量
        /// - `usdt_address`: USDT 收款地址（TRC20）
        /// 
        /// ## 返回
        /// - `Ok(swap_id)`: 兑换ID
        /// - `Err(...)`: 各种错误情况
        pub fn do_maker_swap(
            user: &T::AccountId,
            maker_id: u64,
            dust_amount: BalanceOf<T>,
            usdt_address: sp_std::vec::Vec<u8>,
        ) -> Result<u64, DispatchError> {
            // 1. 验证最小兑换金额
            ensure!(
                dust_amount >= T::MinSwapAmount::get(),
                Error::<T>::BelowMinimumAmount
            );
            
            // 2. 验证做市商存在且激活（使用 MakerInterface）
            let maker_app = T::MakerPallet::get_maker_application(maker_id)
                .ok_or(Error::<T>::MakerNotFound)?;
            ensure!(maker_app.is_active, Error::<T>::MakerNotActive);
            
            // 3. 验证 USDT 地址格式
            let usdt_addr: TronAddress = usdt_address
                .try_into()
                .map_err(|_| Error::<T>::InvalidAddress)?;
            
            // 4. 获取当前价格（从 PricingProvider 获取实时汇率）
            let price_balance = T::Pricing::get_dust_to_usd_rate()
                .ok_or(Error::<T>::PriceNotAvailable)?;
            let price_usdt: u64 = price_balance.saturated_into();
            
            // 5. 计算 USDT 金额（加入边界检查防止溢出）
            let dust_amount_u128: u128 = dust_amount.saturated_into();
            let usdt_amount_u128 = dust_amount_u128
                .checked_mul(price_usdt as u128)
                .ok_or(Error::<T>::AmountOverflow)?
                .checked_div(1_000_000_000_000u128)
                .ok_or(Error::<T>::AmountOverflow)?;
            
            // 6. 验证最小 USDT 金额（至少 1 USDT）
            ensure!(
                usdt_amount_u128 >= 1_000_000,
                Error::<T>::UsdtAmountTooSmall
            );
            
            let usdt_amount = usdt_amount_u128 as u64;
            
            // 7. 获取兑换ID
            let swap_id = NextSwapId::<T>::get();
            
            // 7. 锁定用户的 DUST 到托管
            T::Escrow::lock_from(
                user,
                swap_id,
                dust_amount,
            )?;
            
            // 8. 计算超时时间
            let current_block = frame_system::Pallet::<T>::block_number();
            let timeout_at = current_block + T::OcwSwapTimeoutBlocks::get();
            
            // 9. 创建做市商兑换记录
            let record = MakerSwapRecord {
                swap_id,
                maker_id,
                maker: maker_app.account,
                user: user.clone(),
                dust_amount,
                usdt_amount,
                usdt_address: usdt_addr,
                created_at: current_block,
                timeout_at,
                trc20_tx_hash: None,
                completed_at: None,
                evidence_cid: None,
                status: SwapStatus::Pending,
                price_usdt,
            };
            
            // 10. 保存记录
            MakerSwaps::<T>::insert(swap_id, record);
            NextSwapId::<T>::put(swap_id + 1);
            
            // 11. 更新用户兑换列表
            UserSwaps::<T>::try_mutate(user, |swaps| {
                swaps.try_push(swap_id)
                    .map_err(|_| Error::<T>::TooManySwaps)
            })?;
            
            // 12. 更新做市商兑换列表
            MakerSwapList::<T>::try_mutate(maker_id, |swaps| {
                swaps.try_push(swap_id)
                    .map_err(|_| Error::<T>::TooManySwaps)
            })?;
            
            // 13. 发出事件
            Self::deposit_event(Event::MakerSwapCreated {
                swap_id,
                user: user.clone(),
                maker_id,
                dust_amount,
            });
            
            Ok(swap_id)
        }
        
        /// 函数级详细中文注释：做市商标记兑换完成
        /// 
        /// ## 功能说明
        /// 1. 验证兑换存在且状态为 Pending
        /// 2. 验证调用者是兑换的做市商
        /// 3. 记录 TRC20 交易哈希
        /// 4. 释放 DUST 到做市商
        /// 5. 更新兑换状态为 Completed
        /// 
        /// ## 参数
        /// - `maker`: 做市商账户
        /// - `swap_id`: 兑换ID
        /// - `trc20_tx_hash`: TRC20 交易哈希
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 各种错误情况
        pub fn do_mark_swap_complete(
            maker: &T::AccountId,
            swap_id: u64,
            trc20_tx_hash: sp_std::vec::Vec<u8>,
        ) -> DispatchResult {
            // 1. 获取兑换记录
            let mut record = MakerSwaps::<T>::get(swap_id)
                .ok_or(Error::<T>::SwapNotFound)?;
            
            // 2. 验证调用者是做市商
            ensure!(record.maker == *maker, Error::<T>::NotMaker);
            
            // 3. 验证状态
            ensure!(
                record.status == SwapStatus::Pending,
                Error::<T>::InvalidStatus
            );
            
            // 4. 验证交易哈希长度
            let tx_hash: BoundedVec<u8, ConstU32<128>> = trc20_tx_hash
                .try_into()
                .map_err(|_| Error::<T>::InvalidTxHash)?;
            
            // 5. 检查交易哈希是否已被使用（防止重放攻击）
            ensure!(
                !UsedTronTxHashes::<T>::contains_key(&tx_hash),
                Error::<T>::TronTxHashAlreadyUsed
            );
            
            // 6. 记录已使用的交易哈希
            UsedTronTxHashes::<T>::insert(&tx_hash, ());
            
            // 7. 释放 DUST 到做市商
            T::Escrow::release_all(
                swap_id,
                &record.maker,
            )?;
            
            // 8. 更新记录
            record.trc20_tx_hash = Some(tx_hash);
            record.status = SwapStatus::Completed;
            let current_block = frame_system::Pallet::<T>::block_number();
            record.completed_at = Some(current_block);
            MakerSwaps::<T>::insert(swap_id, record.clone());
            
            // 9. 记录信用分（成功完成订单）✅
            // 计算响应时间（秒）
            let block_duration = current_block.saturating_sub(record.created_at);
            let response_time_seconds = (block_duration.saturated_into::<u64>() * 6) as u32; // 假设 6s/block
            
            // 调用 Credit 接口
            let _ = T::Credit::record_maker_order_completed(
                record.maker_id,
                swap_id,
                response_time_seconds,
            );
            
            // 10. 发出事件
            Self::deposit_event(Event::MakerSwapCompleted {
                swap_id,
                maker: maker.clone(),
            });
            
            Ok(())
        }
        
        /// 函数级详细中文注释：用户举报做市商兑换
        /// 
        /// ## 功能说明
        /// 1. 验证兑换存在
        /// 2. 验证调用者是兑换的用户
        /// 3. 验证兑换状态为 Pending 或 Completed
        /// 4. 更新状态为 UserReported
        /// 5. 发出举报事件
        /// 
        /// ## 参数
        /// - `user`: 用户账户
        /// - `swap_id`: 兑换ID
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 各种错误情况
        pub fn do_report_swap(
            user: &T::AccountId,
            swap_id: u64,
        ) -> DispatchResult {
            // 1. 获取兑换记录
            let mut record = MakerSwaps::<T>::get(swap_id)
                .ok_or(Error::<T>::SwapNotFound)?;
            
            // 2. 验证调用者是用户
            ensure!(record.user == *user, Error::<T>::NotSwapUser);
            
            // 3. 验证状态（只有 Pending 或 Completed 状态可以举报）
            ensure!(
                matches!(record.status, SwapStatus::Pending | SwapStatus::Completed),
                Error::<T>::CannotReport
            );
            
            // 4. 更新状态
            record.status = SwapStatus::UserReported;
            MakerSwaps::<T>::insert(swap_id, record);
            
            // 5. 发出事件
            Self::deposit_event(Event::SwapReported {
                swap_id,
                user: user.clone(),
            });
            
            Ok(())
        }
    }
    
    // ===== 公共查询接口 =====
    
    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：获取用户兑换列表
        pub fn get_user_swaps(who: &T::AccountId) -> sp_std::vec::Vec<u64> {
            UserSwaps::<T>::get(who).to_vec()
        }
        
        /// 函数级详细中文注释：获取做市商兑换列表
        pub fn get_maker_swaps(maker_id: u64) -> sp_std::vec::Vec<u64> {
            MakerSwapList::<T>::get(maker_id).to_vec()
        }
        
        // ===== 仲裁支持接口 =====
        
        /// 函数级详细中文注释：检查用户是否有权对兑换发起争议
        /// 
        /// ## 权限规则
        /// - 用户（买家）：可以对自己的兑换发起争议
        /// - 做市商：可以对自己参与的兑换发起争议
        /// 
        /// ## 参数
        /// - `who`: 发起争议的用户
        /// - `swap_id`: 兑换ID
        /// 
        /// ## 返回
        /// - `true`: 有权发起争议
        /// - `false`: 无权发起争议
        pub fn can_dispute_swap(who: &T::AccountId, swap_id: u64) -> bool {
            if let Some(record) = MakerSwaps::<T>::get(swap_id) {
                // 用户或做市商都可以发起争议
                &record.user == who || &record.maker == who
            } else {
                false
            }
        }
        
        /// 函数级详细中文注释：应用仲裁裁决到兑换
        /// 
        /// ## 裁决类型
        /// - Release: 全额放款给做市商（用户败诉）
        /// - Refund: 全额退款给用户（做市商败诉）
        /// - Partial(bps): 按比例分账（双方都有责任）
        /// 
        /// ## 参数
        /// - `swap_id`: 兑换ID
        /// - `decision`: 仲裁裁决
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 失败
        pub fn apply_arbitration_decision(
            swap_id: u64,
            decision: pallet_arbitration::pallet::Decision,
        ) -> DispatchResult {
            // 获取兑换记录
            let mut record = MakerSwaps::<T>::get(swap_id)
                .ok_or(Error::<T>::SwapNotFound)?;
            
            // 确保状态是 UserReported（用户已举报）
            ensure!(
                record.status == SwapStatus::UserReported,
                Error::<T>::InvalidStatus
            );
            
            // 根据裁决类型执行相应操作
            use pallet_arbitration::pallet::Decision;
            let maker_win = match decision {
                Decision::Release => {
                    // 放款给做市商（用户败诉）
                    T::Escrow::release_all(swap_id, &record.maker)?;
                    record.status = SwapStatus::ArbitrationApproved;
                    true  // 做市商胜诉
                },
                Decision::Refund => {
                    // 退款给用户（做市商败诉）
                    T::Escrow::refund_all(swap_id, &record.user)?;
                    record.status = SwapStatus::ArbitrationRejected;
                    false  // 做市商败诉
                },
                Decision::Partial(_bps) => {
                    // 按比例分账
                    // TODO: pallet-escrow 暂未实现 split_partial 方法
                    // 暂时当作 Refund 处理（退款给用户）
                    T::Escrow::refund_all(swap_id, &record.user)?;
                    record.status = SwapStatus::ArbitrationRejected;
                    false  // 做市商败诉
                },
            };
            
            // 记录争议结果到信用分 ✅
            let _ = T::Credit::record_maker_dispute_result(
                record.maker_id,
                swap_id,
                maker_win,
            );
            
            // 更新记录
            MakerSwaps::<T>::insert(swap_id, record);
            
            Ok(())
        }
    }
    
    // ===== OCW（Off-Chain Worker）实现 =====
    
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// 函数级详细中文注释：OCW 入口函数
        /// 
        /// ## 功能说明
        /// 1. 每个区块执行一次
        /// 2. 检测超时的做市商兑换
        /// 3. 自动退款超时订单
        /// 
        /// ## 实现逻辑
        /// - 扫描所有 Pending 状态的做市商兑换
        /// - 检查是否超过超时区块
        /// - 提交无签名交易执行退款
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            // OCW 日志：使用 sp_runtime::print
            sp_runtime::print("🌉 Bridge OCW 开始执行");
            
            // 检测超时的做市商兑换
            let _ = Self::check_timeout_swaps(block_number);
        }
    }
    
    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：检测超时的做市商兑换
        /// 
        /// ## 功能说明
        /// 扫描所有 Pending 状态的做市商兑换，找出超时的订单并自动退款
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(())`: 失败（仅用于日志）
        fn check_timeout_swaps(current_block: BlockNumberFor<T>) -> Result<(), ()> {
            // 遍历所有做市商兑换（简化版：仅检查最近的 100 个）
            let next_id = NextSwapId::<T>::get();
            let start_id = if next_id > 100 { next_id - 100 } else { 0 };
            
            let mut timeout_count = 0u32;
            
            for swap_id in start_id..next_id {
                if let Some(mut record) = MakerSwaps::<T>::get(swap_id) {
                    // 只处理 Pending 状态的订单
                    if record.status != SwapStatus::Pending {
                        continue;
                    }
                    
                    // 检查是否超时
                    if current_block >= record.timeout_at {
                        sp_runtime::print("⚠️ Bridge OCW: 检测到超时兑换");
                        
                        // 执行退款（直接在 OCW 中执行，因为这是链上逻辑）
                        // 注意：这里应该提交无签名交易，但为了简化，我们直接修改状态
                        // TODO: 使用 submit_unsigned_transaction 提交无签名交易
                        
                        // 退款给用户
                        if let Err(_e) = T::Escrow::refund_all(swap_id, &record.user) {
                            continue;
                        }
                        
                        // 记录超时到信用分 ✅
                        let _ = T::Credit::record_maker_order_timeout(
                            record.maker_id,
                            swap_id,
                        );
                        
                        // 更新状态为 Refunded
                        record.status = SwapStatus::Refunded;
                        MakerSwaps::<T>::insert(swap_id, record.clone());
                        
                        timeout_count += 1;
                    }
                }
            }
            
            if timeout_count > 0 {
                sp_runtime::print("✅ Bridge OCW: 处理了超时兑换");
            }
            
            Ok(())
        }
    }
}
