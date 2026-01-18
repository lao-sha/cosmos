#![cfg_attr(not(feature = "std"), no_std)]

//! # Swap Pallet (做市商兑换模块)
//!
//! ## 概述
//!
//! 本模块负责 DUST → USDT 做市商兑换服务，包括：
//! - 做市商兑换（市场化服务）
//! - OCW 自动验证
//! - 超时退款机制
//!
//! ## 版本历史
//!
//! - v0.1.0 (2025-11-03): 从 pallet-trading 拆分而来
//! - v0.2.0 (2026-01-18): 移除官方桥接功能，仅保留做市商兑换
//! - v0.3.0 (2026-01-18): 重命名 bridge → swap

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
    
    // 🆕 v0.4.0: 从 pallet-trading-common 导入公共类型和 Trait
    use pallet_trading_common::{
        TronAddress,
        PricingProvider,
        MakerInterface,
        MakerCreditInterface,
    };
    // MakerApplicationInfo 通过 MakerInterface::get_maker_application 返回
    
    /// 函数级详细中文注释：Balance 类型别名
    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    
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
    
    /// 🆕 2026-01-18: 兑换时间信息结构（供 RPC 查询使用）
    #[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct SwapTimeInfo<T: Config> {
        /// 兑换ID
        pub swap_id: u64,
        /// 做市商ID
        pub maker_id: u64,
        /// 用户账户
        pub user: T::AccountId,
        /// DUST 数量
        pub dust_amount: BalanceOf<T>,
        /// USDT 金额
        pub usdt_amount: u64,
        /// 创建区块
        pub created_at_block: u64,
        /// 创建时间（预估 Unix 秒）
        pub created_at_timestamp: u64,
        /// 超时区块
        pub timeout_at_block: u64,
        /// 超时时间（预估 Unix 秒）
        pub timeout_at_timestamp: u64,
        /// 剩余秒数（0表示已超时）
        pub remaining_seconds: u64,
        /// 可读剩余时间（如 "45m", "1h 30m"）
        pub remaining_readable: sp_std::vec::Vec<u8>,
        /// 兑换状态（0-4）
        pub status: u8,
        /// 是否已超时
        pub is_timeout: bool,
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
        
        /// Credit Pallet 接口（用于记录做市商信用分）
        /// 🆕 2026-01-18: 统一使用 pallet_trading_common::MakerCreditInterface
        type Credit: pallet_trading_common::MakerCreditInterface;
        
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
        /// 🆕 2026-01-18: 兑换超时（自动退款）
        SwapTimeout {
            swap_id: u64,
            user: T::AccountId,
            maker_id: u64,
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
        /// 🆕 2026-01-18: 尚未超时
        NotYetTimeout,
    }
    
    // ===== Extrinsics =====
    
    #[pallet::call]
    impl<T: Config> Pallet<T> {
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
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::maker_swap())]
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
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::mark_swap_complete())]
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
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::report_swap())]
        pub fn report_swap(
            origin: OriginFor<T>,
            swap_id: u64,
        ) -> DispatchResult {
            let user = ensure_signed(origin)?;
            Self::do_report_swap(&user, swap_id)
        }
        
    }
    
    // ===== 内部实现 =====
    
    impl<T: Config> Pallet<T> {
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
        
        // ===== 🆕 2026-01-18: 可读时间查询接口 =====
        
        /// 函数级详细中文注释：获取兑换详情（含可读时间）
        /// 
        /// ## 功能说明
        /// 为前端提供人可读的时间信息
        /// - 区块号自动转换为预估时间戳
        /// - 计算剩余时间
        /// - 提供可读格式（如 "45m"）
        pub fn get_swap_with_time(swap_id: u64) -> Option<SwapTimeInfo<T>> {
            let record = MakerSwaps::<T>::get(swap_id)?;
            let current_block = frame_system::Pallet::<T>::block_number();
            let current_block_u64: u64 = current_block.saturated_into();
            let created_at_u64: u64 = record.created_at.saturated_into();
            let timeout_at_u64: u64 = record.timeout_at.saturated_into();
            
            // 使用当前时间戳（假设 pallet_timestamp 可用）
            // 这里使用区块号估算
            let now_estimate = current_block_u64 * pallet_trading_common::DEFAULT_BLOCK_TIME_SECS;
            
            let created_at_timestamp = pallet_trading_common::estimate_timestamp_from_block(
                created_at_u64,
                current_block_u64,
                now_estimate,
            );
            
            let timeout_at_timestamp = pallet_trading_common::estimate_timestamp_from_block(
                timeout_at_u64,
                current_block_u64,
                now_estimate,
            );
            
            let remaining_seconds = pallet_trading_common::estimate_remaining_seconds(
                timeout_at_u64,
                current_block_u64,
            );
            
            let is_timeout = current_block >= record.timeout_at 
                && record.status == SwapStatus::Pending;
            
            Some(SwapTimeInfo {
                swap_id,
                maker_id: record.maker_id,
                user: record.user.clone(),
                dust_amount: record.dust_amount,
                usdt_amount: record.usdt_amount,
                created_at_block: created_at_u64,
                created_at_timestamp,
                timeout_at_block: timeout_at_u64,
                timeout_at_timestamp,
                remaining_seconds,
                remaining_readable: pallet_trading_common::format_duration(remaining_seconds),
                status: Self::status_to_u8(&record.status),
                is_timeout,
            })
        }
        
        /// 函数级详细中文注释：批量获取用户兑换（含可读时间）
        pub fn get_user_swaps_with_time(who: &T::AccountId) -> sp_std::vec::Vec<SwapTimeInfo<T>> {
            UserSwaps::<T>::get(who)
                .iter()
                .filter_map(|&swap_id| Self::get_swap_with_time(swap_id))
                .collect()
        }
        
        /// 内部函数：状态转换为 u8
        fn status_to_u8(status: &SwapStatus) -> u8 {
            match status {
                SwapStatus::Pending => 0,
                SwapStatus::Completed => 1,
                SwapStatus::UserReported => 2,
                SwapStatus::Arbitrating => 3,
                SwapStatus::ArbitrationApproved => 4,
                SwapStatus::ArbitrationRejected => 5,
                SwapStatus::Refunded => 6,
            }
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
    
    // ===== 🆕 2026-01-18: 自动超时处理（使用 on_initialize 替代 OCW）=====
    
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// 函数级详细中文注释：区块初始化时检查超时兑换
        /// 
        /// ## 功能说明
        /// - 每 50 个区块检查一次（约 5 分钟）
        /// - 扫描最近 100 个兑换
        /// - 每次最多处理 10 个超时兑换
        /// 
        /// ## 🆕 2026-01-18 修复
        /// - 原 OCW 方式直接修改状态无效
        /// - 改为 on_initialize 在链上直接处理
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            // 每 50 个区块检查一次
            let check_interval: u32 = 50;
            let now_u32: u32 = now.saturated_into();
            
            if now_u32 % check_interval != 0 {
                return Weight::zero();
            }
            
            Self::process_timeout_swaps(now)
        }
    }
    
    impl<T: Config> Pallet<T> {
        /// 🆕 2026-01-18: 处理超时兑换
        /// 
        /// ## 功能说明
        /// - 扫描 Pending 状态的兑换
        /// - 找出超时的订单并自动退款
        /// - 每次最多处理 10 个
        fn process_timeout_swaps(current_block: BlockNumberFor<T>) -> Weight {
            let next_id = NextSwapId::<T>::get();
            let start_id = if next_id > 100 { next_id - 100 } else { 0 };
            
            let max_per_block = 10u32;
            let mut processed_count = 0u32;
            
            for swap_id in start_id..next_id {
                if processed_count >= max_per_block {
                    break;
                }
                
                if let Some(record) = MakerSwaps::<T>::get(swap_id) {
                    // 只处理 Pending 状态的订单
                    if record.status != SwapStatus::Pending {
                        continue;
                    }
                    
                    // 检查是否超时
                    if current_block >= record.timeout_at {
                        // 执行超时处理
                        if Self::do_process_timeout(swap_id).is_ok() {
                            processed_count += 1;
                        }
                    }
                }
            }
            
            // 返回消耗的权重
            Weight::from_parts((processed_count as u64) * 100_000 + 10_000, 0)
        }
        
        /// 🆕 2026-01-18: 执行单个兑换的超时处理
        /// 
        /// ## 功能说明
        /// 1. 验证超时条件
        /// 2. 退款给用户
        /// 3. 记录做市商超时
        /// 4. 更新兑换状态
        fn do_process_timeout(swap_id: u64) -> DispatchResult {
            // 1. 获取兑换记录
            let mut record = MakerSwaps::<T>::get(swap_id)
                .ok_or(Error::<T>::SwapNotFound)?;
            
            // 2. 验证状态
            ensure!(
                record.status == SwapStatus::Pending,
                Error::<T>::InvalidStatus
            );
            
            // 3. 验证已超时
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block >= record.timeout_at,
                Error::<T>::NotYetTimeout
            );
            
            // 4. 退款给用户
            T::Escrow::refund_all(swap_id, &record.user)?;
            
            // 5. 记录超时到信用分
            let _ = T::Credit::record_maker_order_timeout(
                record.maker_id,
                swap_id,
            );
            
            // 6. 更新状态
            record.status = SwapStatus::Refunded;
            MakerSwaps::<T>::insert(swap_id, record.clone());
            
            // 7. 发出事件
            Self::deposit_event(Event::SwapTimeout {
                swap_id,
                user: record.user,
                maker_id: record.maker_id,
            });
            
            Ok(())
        }
    }
}
