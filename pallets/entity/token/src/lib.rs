//! # 实体通证模块 (pallet-entity-token)
//!
//! ## 概述
//!
//! 本模块作为 pallet-assets 的桥接层，为每个实体提供通证功能：
//! - 实体通证创建和配置
//! - 多种通证类型（积分、治理、股权、会员等）
//! - 购物/参与奖励
//! - 积分/通证兑换
//! - 通证转让
//! - 分红功能
//!
//! ## 架构
//!
//! ```text
//! pallet-entity-token (桥接层)
//!         │
//!         │ fungibles::* traits
//!         ▼
//! pallet-assets (底层资产模块)
//! ```
//!
//! ## 版本历史
//!
//! - v0.1.0 (2026-01-31): 初始版本
//! - v0.2.0 (2026-02-03): Phase 2 扩展，支持多种通证类型和分红

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;

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
        traits::fungibles::{Create, Inspect, Mutate, metadata::Mutate as MetadataMutate},
        BoundedVec,
    };
    use frame_system::pallet_prelude::*;
    use pallet_entity_common::{DividendConfig, ShopProvider, TokenType};
    use sp_runtime::traits::{AtLeast32BitUnsigned, Saturating, Zero};

    /// 实体通证配置（原 ShopTokenConfig，Phase 2 扩展）
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct EntityTokenConfig<Balance, BlockNumber> {
        /// 是否已启用通证
        pub enabled: bool,
        /// 购物/参与返积分比例（基点，500 = 5%）
        pub reward_rate: u16,
        /// 积分兑换比例（基点，1000 = 10%，即 10 积分 = 1 元折扣）
        pub exchange_rate: u16,
        /// 最低兑换门槛
        pub min_redeem: Balance,
        /// 单笔最大兑换（0 = 无限制）
        pub max_redeem_per_order: Balance,
        /// 是否允许用户间转让
        pub transferable: bool,
        /// 创建时间
        pub created_at: BlockNumber,
        // ========== Phase 2 新增字段 ==========
        /// 通证类型（默认 Points）
        pub token_type: TokenType,
        /// 最大供应量（0 = 无限制）
        pub max_supply: Balance,
        /// 分红配置
        pub dividend_config: DividendConfig<Balance, BlockNumber>,
    }

    /// 向后兼容：ShopTokenConfig 类型别名
    pub type ShopTokenConfig<Balance, BlockNumber> = EntityTokenConfig<Balance, BlockNumber>;

    /// 配置类型别名
    pub type ShopTokenConfigOf<T> = ShopTokenConfig<
        <T as Config>::AssetBalance,
        BlockNumberFor<T>,
    >;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 运行时事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 资产 ID 类型
        type AssetId: Member + Parameter + Copy + MaxEncodedLen + From<u64> + Into<u64>;

        /// 资产余额类型
        type AssetBalance: Member
            + Parameter
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + MaxEncodedLen
            + From<u128>
            + Into<u128>;

        /// 资产创建接口
        type Assets: Create<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
            + Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
            + Mutate<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
            + MetadataMutate<Self::AccountId, AssetId = Self::AssetId>;

        /// 店铺查询接口
        type ShopProvider: ShopProvider<Self::AccountId>;

        /// 店铺代币 ID 偏移量（避免与其他资产冲突）
        #[pallet::constant]
        type ShopTokenOffset: Get<u64>;

        /// 代币名称最大长度
        #[pallet::constant]
        type MaxTokenNameLength: Get<u32>;

        /// 代币符号最大长度
        #[pallet::constant]
        type MaxTokenSymbolLength: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ==================== 存储项 ====================

    /// 店铺代币配置存储
    #[pallet::storage]
    #[pallet::getter(fn shop_token_configs)]
    pub type ShopTokenConfigs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // shop_id
        ShopTokenConfigOf<T>,
    >;

    /// 店铺代币元数据（名称、符号）
    #[pallet::storage]
    #[pallet::getter(fn shop_token_metadata)]
    pub type ShopTokenMetadata<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // shop_id
        (
            BoundedVec<u8, T::MaxTokenNameLength>,   // name
            BoundedVec<u8, T::MaxTokenSymbolLength>, // symbol
            u8,                                      // decimals
        ),
    >;

    /// 统计：已创建的店铺代币数量
    #[pallet::storage]
    #[pallet::getter(fn total_shop_tokens)]
    pub type TotalShopTokens<T: Config> = StorageValue<_, u64, ValueQuery>;

    // ========== Phase 4 新增存储项 ==========

    /// 锁仓记录 (entity_id, holder) -> (locked_amount, unlock_at)
    #[pallet::storage]
    #[pallet::getter(fn locked_tokens)]
    pub type LockedTokens<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,  // entity_id
        Blake2_128Concat,
        T::AccountId,
        (T::AssetBalance, BlockNumberFor<T>),  // (amount, unlock_at)
    >;

    /// 待领取分红 (entity_id, holder) -> amount
    #[pallet::storage]
    #[pallet::getter(fn pending_dividends)]
    pub type PendingDividends<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,  // entity_id
        Blake2_128Concat,
        T::AccountId,
        T::AssetBalance,
        ValueQuery,
    >;

    /// 已领取分红总额 (entity_id, holder) -> total_claimed
    #[pallet::storage]
    #[pallet::getter(fn claimed_dividends)]
    pub type ClaimedDividends<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        T::AssetBalance,
        ValueQuery,
    >;

    // ==================== 事件 ====================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 店铺代币已创建
        ShopTokenCreated {
            shop_id: u64,
            asset_id: T::AssetId,
            name: Vec<u8>,
            symbol: Vec<u8>,
        },
        /// 店铺代币配置已更新
        TokenConfigUpdated { shop_id: u64 },
        /// 购物奖励已发放
        RewardIssued {
            shop_id: u64,
            buyer: T::AccountId,
            amount: T::AssetBalance,
        },
        /// 积分已兑换
        TokensRedeemed {
            shop_id: u64,
            buyer: T::AccountId,
            tokens: T::AssetBalance,
            discount: T::AssetBalance,
        },
        /// 积分已转让
        TokensTransferred {
            shop_id: u64,
            from: T::AccountId,
            to: T::AccountId,
            amount: T::AssetBalance,
        },
        /// 代币已铸造
        TokensMinted {
            shop_id: u64,
            to: T::AccountId,
            amount: T::AssetBalance,
        },
        /// 代币已销毁
        TokensBurned {
            shop_id: u64,
            from: T::AccountId,
            amount: T::AssetBalance,
        },
        // ========== Phase 4 新增事件 ==========
        /// 分红已配置
        DividendConfigured {
            entity_id: u64,
            enabled: bool,
            min_period: BlockNumberFor<T>,
        },
        /// 分红已分发
        DividendDistributed {
            entity_id: u64,
            total_amount: T::AssetBalance,
            recipients_count: u32,
        },
        /// 分红已领取
        DividendClaimed {
            entity_id: u64,
            holder: T::AccountId,
            amount: T::AssetBalance,
        },
        /// 代币已锁仓
        TokensLocked {
            entity_id: u64,
            holder: T::AccountId,
            amount: T::AssetBalance,
            unlock_at: BlockNumberFor<T>,
        },
        /// 代币已解锁
        TokensUnlocked {
            entity_id: u64,
            holder: T::AccountId,
            amount: T::AssetBalance,
        },
        /// 通证类型已变更
        TokenTypeChanged {
            entity_id: u64,
            old_type: TokenType,
            new_type: TokenType,
        },
    }

    // ==================== 错误 ====================

    #[pallet::error]
    pub enum Error<T> {
        /// 店铺不存在
        ShopNotFound,
        /// 不是店主
        NotShopOwner,
        /// 店铺代币未启用
        TokenNotEnabled,
        /// 代币已存在
        TokenAlreadyExists,
        /// 余额不足
        InsufficientBalance,
        /// 低于最低兑换门槛
        BelowMinRedeem,
        /// 超过单笔最大兑换
        ExceedsMaxRedeem,
        /// 不允许转让
        TransferNotAllowed,
        /// 名称过长
        NameTooLong,
        /// 符号过长
        SymbolTooLong,
        /// 资产创建失败
        AssetCreationFailed,
        /// 无效的奖励率
        InvalidRewardRate,
        /// 无效的兑换率
        InvalidExchangeRate,
        // ========== Phase 4 新增错误 ==========
        /// 分红未启用
        DividendNotEnabled,
        /// 分红周期未到
        DividendPeriodNotReached,
        /// 无可领取分红
        NoDividendToClaim,
        /// 代币已锁仓
        TokensAreLocked,
        /// 无锁仓代币
        NoLockedTokens,
        /// 解锁时间未到
        UnlockTimeNotReached,
        /// 超过最大供应量
        ExceedsMaxSupply,
        /// 通证类型不支持此操作
        TokenTypeNotSupported,
        /// 不允许该通证类型
        TokenTypeNotAllowed,
    }

    // ==================== Extrinsics ====================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 为店铺创建代币
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `name`: 代币名称
        /// - `symbol`: 代币符号
        /// - `decimals`: 小数位数
        /// - `reward_rate`: 购物返积分比例（基点）
        /// - `exchange_rate`: 积分兑换比例（基点）
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(80_000, 0))]
        pub fn create_shop_token(
            origin: OriginFor<T>,
            shop_id: u64,
            name: Vec<u8>,
            symbol: Vec<u8>,
            decimals: u8,
            reward_rate: u16,
            exchange_rate: u16,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证店铺存在且调用者是店主
            ensure!(T::ShopProvider::shop_exists(shop_id), Error::<T>::ShopNotFound);
            let owner = T::ShopProvider::shop_owner(shop_id).ok_or(Error::<T>::ShopNotFound)?;
            ensure!(owner == who, Error::<T>::NotShopOwner);

            // 检查代币是否已存在
            ensure!(!ShopTokenConfigs::<T>::contains_key(shop_id), Error::<T>::TokenAlreadyExists);

            // 验证参数
            ensure!(reward_rate <= 10000, Error::<T>::InvalidRewardRate);
            ensure!(exchange_rate <= 10000, Error::<T>::InvalidExchangeRate);

            // 转换名称和符号
            let name_bounded: BoundedVec<u8, T::MaxTokenNameLength> =
                name.clone().try_into().map_err(|_| Error::<T>::NameTooLong)?;
            let symbol_bounded: BoundedVec<u8, T::MaxTokenSymbolLength> =
                symbol.clone().try_into().map_err(|_| Error::<T>::SymbolTooLong)?;

            // 计算资产 ID
            let asset_id = Self::shop_to_asset_id(shop_id);

            // 通过 pallet-assets 创建资产
            T::Assets::create(asset_id, who.clone(), true, 1u32.into())
                .map_err(|_| Error::<T>::AssetCreationFailed)?;

            // 设置元数据
            T::Assets::set(asset_id, &who, name.clone(), symbol.clone(), decimals)
                .map_err(|_| Error::<T>::AssetCreationFailed)?;

            // 保存配置
            let now = <frame_system::Pallet<T>>::block_number();
            let config = EntityTokenConfig {
                enabled: true,
                reward_rate,
                exchange_rate,
                min_redeem: Zero::zero(),
                max_redeem_per_order: Zero::zero(),
                transferable: true,
                created_at: now,
                // Phase 2 新增字段（默认值）
                token_type: TokenType::Points,
                max_supply: Zero::zero(),  // 0 = 无限制
                dividend_config: DividendConfig {
                    enabled: false,
                    min_period: Zero::zero(),
                    last_distribution: Zero::zero(),
                    accumulated: Zero::zero(),
                },
            };
            ShopTokenConfigs::<T>::insert(shop_id, config);
            ShopTokenMetadata::<T>::insert(shop_id, (name_bounded, symbol_bounded, decimals));
            TotalShopTokens::<T>::mutate(|n| *n = n.saturating_add(1));

            Self::deposit_event(Event::ShopTokenCreated {
                shop_id,
                asset_id,
                name,
                symbol,
            });

            Ok(())
        }

        /// 更新代币配置
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn update_token_config(
            origin: OriginFor<T>,
            shop_id: u64,
            reward_rate: Option<u16>,
            exchange_rate: Option<u16>,
            min_redeem: Option<T::AssetBalance>,
            max_redeem_per_order: Option<T::AssetBalance>,
            transferable: Option<bool>,
            enabled: Option<bool>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证店主
            let owner = T::ShopProvider::shop_owner(shop_id).ok_or(Error::<T>::ShopNotFound)?;
            ensure!(owner == who, Error::<T>::NotShopOwner);

            ShopTokenConfigs::<T>::try_mutate(shop_id, |maybe_config| -> DispatchResult {
                let config = maybe_config.as_mut().ok_or(Error::<T>::TokenNotEnabled)?;

                if let Some(rate) = reward_rate {
                    ensure!(rate <= 10000, Error::<T>::InvalidRewardRate);
                    config.reward_rate = rate;
                }
                if let Some(rate) = exchange_rate {
                    ensure!(rate <= 10000, Error::<T>::InvalidExchangeRate);
                    config.exchange_rate = rate;
                }
                if let Some(min) = min_redeem {
                    config.min_redeem = min;
                }
                if let Some(max) = max_redeem_per_order {
                    config.max_redeem_per_order = max;
                }
                if let Some(t) = transferable {
                    config.transferable = t;
                }
                if let Some(e) = enabled {
                    config.enabled = e;
                }

                Ok(())
            })?;

            Self::deposit_event(Event::TokenConfigUpdated { shop_id });
            Ok(())
        }

        /// 店主铸造代币（用于活动奖励等）
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(40_000, 0))]
        pub fn mint_tokens(
            origin: OriginFor<T>,
            shop_id: u64,
            to: T::AccountId,
            amount: T::AssetBalance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证店主
            let owner = T::ShopProvider::shop_owner(shop_id).ok_or(Error::<T>::ShopNotFound)?;
            ensure!(owner == who, Error::<T>::NotShopOwner);

            // 检查代币是否启用
            let config = ShopTokenConfigs::<T>::get(shop_id).ok_or(Error::<T>::TokenNotEnabled)?;
            ensure!(config.enabled, Error::<T>::TokenNotEnabled);

            // 铸造代币
            let asset_id = Self::shop_to_asset_id(shop_id);
            T::Assets::mint_into(asset_id, &to, amount)?;

            Self::deposit_event(Event::TokensMinted {
                shop_id,
                to,
                amount,
            });

            Ok(())
        }

        /// 用户转让积分
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(40_000, 0))]
        pub fn transfer_tokens(
            origin: OriginFor<T>,
            shop_id: u64,
            to: T::AccountId,
            amount: T::AssetBalance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查代币配置
            let config = ShopTokenConfigs::<T>::get(shop_id).ok_or(Error::<T>::TokenNotEnabled)?;
            ensure!(config.enabled, Error::<T>::TokenNotEnabled);
            ensure!(config.transferable, Error::<T>::TransferNotAllowed);

            // 检查余额
            let asset_id = Self::shop_to_asset_id(shop_id);
            let balance = T::Assets::balance(asset_id, &who);
            ensure!(balance >= amount, Error::<T>::InsufficientBalance);

            // 转账
            T::Assets::transfer(asset_id, &who, &to, amount, frame_support::traits::tokens::Preservation::Preserve)?;

            Self::deposit_event(Event::TokensTransferred {
                shop_id,
                from: who,
                to,
                amount,
            });

            Ok(())
        }

        // ==================== Phase 4 新增 Extrinsics ====================

        /// 配置分红
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn configure_dividend(
            origin: OriginFor<T>,
            entity_id: u64,
            enabled: bool,
            min_period: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证所有者
            let owner = T::ShopProvider::shop_owner(entity_id).ok_or(Error::<T>::ShopNotFound)?;
            ensure!(owner == who, Error::<T>::NotShopOwner);

            // 检查代币是否存在
            ShopTokenConfigs::<T>::try_mutate(entity_id, |maybe_config| -> DispatchResult {
                let config = maybe_config.as_mut().ok_or(Error::<T>::TokenNotEnabled)?;
                
                // 检查通证类型是否支持分红
                ensure!(config.token_type.has_dividend_rights(), Error::<T>::TokenTypeNotSupported);
                
                config.dividend_config.enabled = enabled;
                config.dividend_config.min_period = min_period;
                
                Ok(())
            })?;

            Self::deposit_event(Event::DividendConfigured {
                entity_id,
                enabled,
                min_period,
            });
            Ok(())
        }

        /// 分发分红（按持有比例分配）
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(100_000, 0))]
        pub fn distribute_dividend(
            origin: OriginFor<T>,
            entity_id: u64,
            total_amount: T::AssetBalance,
            recipients: Vec<(T::AccountId, T::AssetBalance)>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证所有者
            let owner = T::ShopProvider::shop_owner(entity_id).ok_or(Error::<T>::ShopNotFound)?;
            ensure!(owner == who, Error::<T>::NotShopOwner);

            // 检查分红配置
            let config = ShopTokenConfigs::<T>::get(entity_id).ok_or(Error::<T>::TokenNotEnabled)?;
            ensure!(config.dividend_config.enabled, Error::<T>::DividendNotEnabled);

            let now = <frame_system::Pallet<T>>::block_number();
            let last = config.dividend_config.last_distribution;
            let min_period = config.dividend_config.min_period;
            
            // 检查分红周期（首次分红跳过检查）
            if !last.is_zero() {
                ensure!(now >= last + min_period, Error::<T>::DividendPeriodNotReached);
            }

            // 分配分红到待领取
            let mut count = 0u32;
            for (holder, amount) in recipients.iter() {
                if !amount.is_zero() {
                    PendingDividends::<T>::mutate(entity_id, holder, |pending| {
                        *pending = pending.saturating_add(*amount);
                    });
                    count = count.saturating_add(1);
                }
            }

            // 更新上次分红时间
            ShopTokenConfigs::<T>::mutate(entity_id, |maybe_config| {
                if let Some(config) = maybe_config {
                    config.dividend_config.last_distribution = now;
                }
            });

            Self::deposit_event(Event::DividendDistributed {
                entity_id,
                total_amount,
                recipients_count: count,
            });
            Ok(())
        }

        /// 领取分红
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(40_000, 0))]
        pub fn claim_dividend(
            origin: OriginFor<T>,
            entity_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let pending = PendingDividends::<T>::get(entity_id, &who);
            ensure!(!pending.is_zero(), Error::<T>::NoDividendToClaim);

            // 清空待领取
            PendingDividends::<T>::remove(entity_id, &who);

            // 更新已领取总额
            ClaimedDividends::<T>::mutate(entity_id, &who, |claimed| {
                *claimed = claimed.saturating_add(pending);
            });

            // 铸造分红代币给持有人（或从国库转出，这里简化为铸造）
            let asset_id = Self::shop_to_asset_id(entity_id);
            T::Assets::mint_into(asset_id, &who, pending)?;

            Self::deposit_event(Event::DividendClaimed {
                entity_id,
                holder: who,
                amount: pending,
            });
            Ok(())
        }

        /// 锁仓代币
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(40_000, 0))]
        pub fn lock_tokens(
            origin: OriginFor<T>,
            entity_id: u64,
            amount: T::AssetBalance,
            lock_duration: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查余额
            let asset_id = Self::shop_to_asset_id(entity_id);
            let balance = T::Assets::balance(asset_id, &who);
            
            // 检查现有锁仓
            let existing_locked = LockedTokens::<T>::get(entity_id, &who)
                .map(|(amt, _)| amt)
                .unwrap_or_else(Zero::zero);
            
            let available = balance.saturating_sub(existing_locked);
            ensure!(available >= amount, Error::<T>::InsufficientBalance);

            let now = <frame_system::Pallet<T>>::block_number();
            let unlock_at = now.saturating_add(lock_duration);

            // 更新锁仓记录（合并或创建）
            LockedTokens::<T>::mutate(entity_id, &who, |maybe_locked| {
                match maybe_locked {
                    Some((locked_amt, existing_unlock)) => {
                        // 合并锁仓，使用较晚的解锁时间
                        *locked_amt = locked_amt.saturating_add(amount);
                        if unlock_at > *existing_unlock {
                            *existing_unlock = unlock_at;
                        }
                    }
                    None => {
                        *maybe_locked = Some((amount, unlock_at));
                    }
                }
            });

            Self::deposit_event(Event::TokensLocked {
                entity_id,
                holder: who,
                amount,
                unlock_at,
            });
            Ok(())
        }

        /// 解锁代币
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn unlock_tokens(
            origin: OriginFor<T>,
            entity_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let (locked_amount, unlock_at) = LockedTokens::<T>::get(entity_id, &who)
                .ok_or(Error::<T>::NoLockedTokens)?;

            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(now >= unlock_at, Error::<T>::UnlockTimeNotReached);

            // 移除锁仓记录
            LockedTokens::<T>::remove(entity_id, &who);

            Self::deposit_event(Event::TokensUnlocked {
                entity_id,
                holder: who,
                amount: locked_amount,
            });
            Ok(())
        }

        /// 变更通证类型（需所有者操作）
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn change_token_type(
            origin: OriginFor<T>,
            entity_id: u64,
            new_type: TokenType,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证所有者
            let owner = T::ShopProvider::shop_owner(entity_id).ok_or(Error::<T>::ShopNotFound)?;
            ensure!(owner == who, Error::<T>::NotShopOwner);

            let old_type = ShopTokenConfigs::<T>::try_mutate(entity_id, |maybe_config| -> Result<TokenType, DispatchError> {
                let config = maybe_config.as_mut().ok_or(Error::<T>::TokenNotEnabled)?;
                let old = config.token_type;
                config.token_type = new_type;
                
                // 根据新类型更新可转让性
                config.transferable = new_type.is_transferable_by_default();
                
                Ok(old)
            })?;

            Self::deposit_event(Event::TokenTypeChanged {
                entity_id,
                old_type,
                new_type,
            });
            Ok(())
        }

        /// 设置最大供应量
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(25_000, 0))]
        pub fn set_max_supply(
            origin: OriginFor<T>,
            entity_id: u64,
            max_supply: T::AssetBalance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证所有者
            let owner = T::ShopProvider::shop_owner(entity_id).ok_or(Error::<T>::ShopNotFound)?;
            ensure!(owner == who, Error::<T>::NotShopOwner);

            ShopTokenConfigs::<T>::try_mutate(entity_id, |maybe_config| -> DispatchResult {
                let config = maybe_config.as_mut().ok_or(Error::<T>::TokenNotEnabled)?;
                
                // 检查当前供应量是否超过新的最大值
                let current_supply = Self::get_total_supply(entity_id);
                if !max_supply.is_zero() {
                    ensure!(current_supply <= max_supply, Error::<T>::ExceedsMaxSupply);
                }
                
                config.max_supply = max_supply;
                Ok(())
            })?;

            Self::deposit_event(Event::TokenConfigUpdated { shop_id: entity_id });
            Ok(())
        }
    }

    // ==================== 内部函数 ====================

    impl<T: Config> Pallet<T> {
        /// 店铺 ID 转资产 ID
        pub fn shop_to_asset_id(shop_id: u64) -> T::AssetId {
            (T::ShopTokenOffset::get() + shop_id).into()
        }

        /// 资产 ID 转店铺 ID
        pub fn asset_to_shop_id(asset_id: T::AssetId) -> Option<u64> {
            let id: u64 = asset_id.into();
            let offset = T::ShopTokenOffset::get();
            if id >= offset {
                Some(id - offset)
            } else {
                None
            }
        }

        /// 获取用户在某店铺的积分余额
        pub fn token_balance(shop_id: u64, holder: &T::AccountId) -> T::AssetBalance {
            let asset_id = Self::shop_to_asset_id(shop_id);
            T::Assets::balance(asset_id, holder)
        }

        /// 购物奖励（由 order 模块调用）
        pub fn reward_on_purchase(
            shop_id: u64,
            buyer: &T::AccountId,
            purchase_amount: T::AssetBalance,
        ) -> Result<T::AssetBalance, DispatchError> {
            let config = match ShopTokenConfigs::<T>::get(shop_id) {
                Some(c) if c.enabled && c.reward_rate > 0 => c,
                _ => return Ok(Zero::zero()),
            };

            // 计算奖励：purchase_amount * reward_rate / 10000
            let reward = purchase_amount
                .saturating_mul(config.reward_rate.into())
                / 10000u32.into();

            if reward.is_zero() {
                return Ok(Zero::zero());
            }

            // 铸造代币给买家
            let asset_id = Self::shop_to_asset_id(shop_id);
            T::Assets::mint_into(asset_id, buyer, reward)?;

            Self::deposit_event(Event::RewardIssued {
                shop_id,
                buyer: buyer.clone(),
                amount: reward,
            });

            Ok(reward)
        }

        /// 积分兑换折扣（由 order 模块调用）
        pub fn redeem_for_discount(
            shop_id: u64,
            buyer: &T::AccountId,
            tokens_to_use: T::AssetBalance,
        ) -> Result<T::AssetBalance, DispatchError> {
            let config = ShopTokenConfigs::<T>::get(shop_id)
                .ok_or(Error::<T>::TokenNotEnabled)?;

            ensure!(config.enabled, Error::<T>::TokenNotEnabled);
            ensure!(tokens_to_use >= config.min_redeem, Error::<T>::BelowMinRedeem);
            ensure!(
                config.max_redeem_per_order.is_zero() || tokens_to_use <= config.max_redeem_per_order,
                Error::<T>::ExceedsMaxRedeem
            );

            // 检查余额
            let asset_id = Self::shop_to_asset_id(shop_id);
            let balance = T::Assets::balance(asset_id, buyer);
            ensure!(balance >= tokens_to_use, Error::<T>::InsufficientBalance);

            // 计算折扣：tokens * exchange_rate / 10000
            let discount = tokens_to_use
                .saturating_mul(config.exchange_rate.into())
                / 10000u32.into();

            // 销毁积分
            T::Assets::burn_from(
                asset_id,
                buyer,
                tokens_to_use,
                frame_support::traits::tokens::Preservation::Expendable,
                frame_support::traits::tokens::Precision::Exact,
                frame_support::traits::tokens::Fortitude::Polite,
            )?;

            Self::deposit_event(Event::TokensRedeemed {
                shop_id,
                buyer: buyer.clone(),
                tokens: tokens_to_use,
                discount,
            });

            Ok(discount)
        }
    }
}

// ==================== 公共查询函数 ====================

impl<T: Config> Pallet<T> {
    /// 获取用户在店铺的代币余额（公共接口）
    pub fn get_balance(shop_id: u64, holder: &T::AccountId) -> T::AssetBalance {
        Self::token_balance(shop_id, holder)
    }

    /// 获取店铺代币总供应量
    pub fn get_total_supply(shop_id: u64) -> T::AssetBalance {
        use frame_support::traits::fungibles::Inspect;
        let asset_id = Self::shop_to_asset_id(shop_id);
        T::Assets::total_issuance(asset_id)
    }

    /// 检查店铺代币是否启用
    pub fn is_token_enabled(shop_id: u64) -> bool {
        ShopTokenConfigs::<T>::get(shop_id)
            .map(|c| c.enabled)
            .unwrap_or(false)
    }
}

// ==================== ShopTokenProvider 实现 ====================

use pallet_entity_common::ShopTokenProvider;

impl<T: Config> ShopTokenProvider<T::AccountId, T::AssetBalance> for Pallet<T> {
    fn is_token_enabled(shop_id: u64) -> bool {
        Pallet::<T>::is_token_enabled(shop_id)
    }

    fn token_balance(shop_id: u64, holder: &T::AccountId) -> T::AssetBalance {
        Pallet::<T>::get_balance(shop_id, holder)
    }

    fn reward_on_purchase(
        shop_id: u64,
        buyer: &T::AccountId,
        purchase_amount: T::AssetBalance,
    ) -> Result<T::AssetBalance, sp_runtime::DispatchError> {
        Pallet::<T>::reward_on_purchase(shop_id, buyer, purchase_amount)
    }

    fn redeem_for_discount(
        shop_id: u64,
        buyer: &T::AccountId,
        tokens: T::AssetBalance,
    ) -> Result<T::AssetBalance, sp_runtime::DispatchError> {
        Pallet::<T>::redeem_for_discount(shop_id, buyer, tokens)
    }

    fn transfer(
        shop_id: u64,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: T::AssetBalance,
    ) -> Result<(), sp_runtime::DispatchError> {
        use frame_support::traits::fungibles::Mutate;
        let asset_id = Pallet::<T>::shop_to_asset_id(shop_id);
        T::Assets::transfer(
            asset_id,
            from,
            to,
            amount,
            frame_support::traits::tokens::Preservation::Preserve,
        )?;
        Ok(())
    }

    fn reserve(
        shop_id: u64,
        who: &T::AccountId,
        amount: T::AssetBalance,
    ) -> Result<(), sp_runtime::DispatchError> {
        use frame_support::traits::fungibles::Inspect;
        let asset_id = Pallet::<T>::shop_to_asset_id(shop_id);
        // 注意：pallet-assets 不直接支持 hold，使用转移到模块账户的方式模拟
        // 这里简化实现：直接检查余额是否足够
        let balance = T::Assets::balance(asset_id, who);
        if balance < amount {
            return Err(sp_runtime::DispatchError::Other("InsufficientBalance"));
        }
        Ok(())
    }

    fn unreserve(
        _shop_id: u64,
        _who: &T::AccountId,
        amount: T::AssetBalance,
    ) -> T::AssetBalance {
        // 简化实现：返回解锁的金额
        amount
    }

    fn repatriate_reserved(
        shop_id: u64,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: T::AssetBalance,
    ) -> Result<T::AssetBalance, sp_runtime::DispatchError> {
        // 直接转账（简化实现）
        Self::transfer(shop_id, from, to, amount)?;
        Ok(amount)
    }
}
