//! # 商城店铺代币模块 (pallet-sharemall-token)
//!
//! ## 概述
//!
//! 本模块作为 pallet-assets 的桥接层，为每个店铺提供代币功能：
//! - 店铺代币创建和配置
//! - 购物返积分
//! - 积分抵扣
//! - 积分转让
//!
//! ## 架构
//!
//! ```text
//! pallet-sharemall-token (桥接层)
//!         │
//!         │ fungibles::* traits
//!         ▼
//! pallet-assets (底层资产模块)
//! ```
//!
//! ## 版本历史
//!
//! - v0.1.0 (2026-01-31): 初始版本

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
    use pallet_sharemall_common::ShopProvider;
    use sp_runtime::traits::{AtLeast32BitUnsigned, Saturating, Zero};

    /// 店铺代币配置
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct ShopTokenConfig<Balance, BlockNumber> {
        /// 是否已启用代币
        pub enabled: bool,
        /// 购物返积分比例（基点，500 = 5%）
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
    }

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
            let config = ShopTokenConfig {
                enabled: true,
                reward_rate,
                exchange_rate,
                min_redeem: Zero::zero(),
                max_redeem_per_order: Zero::zero(),
                transferable: true,
                created_at: now,
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

use pallet_sharemall_common::ShopTokenProvider;

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
