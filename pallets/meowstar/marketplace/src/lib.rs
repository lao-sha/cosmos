//! # Meowstar Marketplace Pallet
//!
//! 喵星宇宙 NFT 市场系统 Pallet
//!
//! ## 功能
//! - 宠物挂单出售
//! - 拍卖系统
//! - 交易手续费
//! - 价格历史

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency, ExistenceRequirement},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Saturating;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// 挂单类型
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default, codec::DecodeWithMemTracking)]
    pub enum ListingType {
        /// 固定价格
        #[default]
        FixedPrice,
        /// 拍卖
        Auction,
    }

    /// 挂单状态
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default, codec::DecodeWithMemTracking)]
    pub enum ListingStatus {
        /// 活跃
        #[default]
        Active,
        /// 已售出
        Sold,
        /// 已取消
        Cancelled,
        /// 已过期
        Expired,
    }

    /// 挂单信息
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Listing<T: Config> {
        /// 挂单ID
        pub id: u64,
        /// 卖家
        pub seller: T::AccountId,
        /// 宠物ID
        pub pet_id: u64,
        /// 挂单类型
        pub listing_type: ListingType,
        /// 价格 (固定价格) 或 起拍价 (拍卖)
        pub price: BalanceOf<T>,
        /// 当前最高出价 (拍卖)
        pub highest_bid: BalanceOf<T>,
        /// 当前最高出价者 (拍卖)
        pub highest_bidder: Option<T::AccountId>,
        /// 状态
        pub status: ListingStatus,
        /// 创建区块
        pub created_at: BlockNumberFor<T>,
        /// 过期区块
        pub expires_at: BlockNumberFor<T>,
    }

    /// 出价记录
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Bid<T: Config> {
        /// 出价者
        pub bidder: T::AccountId,
        /// 出价金额
        pub amount: BalanceOf<T>,
        /// 出价区块
        pub block: BlockNumberFor<T>,
    }

    /// 交易记录
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct TradeRecord<T: Config> {
        /// 宠物ID
        pub pet_id: u64,
        /// 卖家
        pub seller: T::AccountId,
        /// 买家
        pub buyer: T::AccountId,
        /// 成交价格
        pub price: BalanceOf<T>,
        /// 手续费
        pub fee: BalanceOf<T>,
        /// 交易区块
        pub block: BlockNumberFor<T>,
    }

    /// 市场统计
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    pub struct MarketStats<Balance> {
        /// 总交易量
        pub total_volume: Balance,
        /// 总交易次数
        pub total_trades: u64,
        /// 总手续费收入
        pub total_fees: Balance,
        /// 当前挂单数量
        pub active_listings: u32,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_meowstar_pet::Config {
        /// 事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 货币类型
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 交易手续费率 (百分比 * 100, 如 250 = 2.5%)
        #[pallet::constant]
        type TradeFeeRate: Get<u32>;

        /// 最小挂单价格
        #[pallet::constant]
        type MinListingPrice: Get<BalanceOf<Self>>;

        /// 默认挂单有效期 (区块数)
        #[pallet::constant]
        type DefaultListingDuration: Get<BlockNumberFor<Self>>;

        /// 拍卖最小加价比例 (百分比 * 100)
        #[pallet::constant]
        type MinBidIncrement: Get<u32>;

        /// 手续费接收账户
        type FeeReceiver: Get<Self::AccountId>;

        /// 权重信息
        type MarketWeightInfo: WeightInfo;
    }

    /// 挂单存储
    #[pallet::storage]
    #[pallet::getter(fn listings)]
    pub type Listings<T: Config> = StorageMap<_, Blake2_128Concat, u64, Listing<T>, OptionQuery>;

    /// 下一个挂单ID
    #[pallet::storage]
    #[pallet::getter(fn next_listing_id)]
    pub type NextListingId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// 宠物到挂单的映射
    #[pallet::storage]
    #[pallet::getter(fn pet_listing)]
    pub type PetListing<T: Config> = StorageMap<_, Blake2_128Concat, u64, u64, OptionQuery>;

    /// 交易历史
    #[pallet::storage]
    #[pallet::getter(fn trade_history)]
    pub type TradeHistory<T: Config> = StorageMap<_, Blake2_128Concat, u64, TradeRecord<T>, OptionQuery>;

    /// 下一个交易ID
    #[pallet::storage]
    #[pallet::getter(fn next_trade_id)]
    pub type NextTradeId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// 市场统计
    #[pallet::storage]
    #[pallet::getter(fn market_stats)]
    pub type Stats<T: Config> = StorageValue<_, MarketStats<BalanceOf<T>>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 宠物已挂单
        Listed {
            listing_id: u64,
            seller: T::AccountId,
            pet_id: u64,
            listing_type: ListingType,
            price: BalanceOf<T>,
        },
        /// 挂单已取消
        ListingCancelled {
            listing_id: u64,
            pet_id: u64,
        },
        /// 宠物已售出
        Sold {
            listing_id: u64,
            pet_id: u64,
            seller: T::AccountId,
            buyer: T::AccountId,
            price: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 拍卖出价
        BidPlaced {
            listing_id: u64,
            bidder: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 拍卖结束
        AuctionEnded {
            listing_id: u64,
            pet_id: u64,
            winner: Option<T::AccountId>,
            final_price: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 宠物不存在
        PetNotFound,
        /// 不是宠物所有者
        NotPetOwner,
        /// 宠物已挂单
        PetAlreadyListed,
        /// 挂单不存在
        ListingNotFound,
        /// 挂单不活跃
        ListingNotActive,
        /// 价格过低
        PriceTooLow,
        /// 不能购买自己的宠物
        CannotBuyOwnPet,
        /// 余额不足
        InsufficientBalance,
        /// 出价过低
        BidTooLow,
        /// 拍卖未结束
        AuctionNotEnded,
        /// 不是拍卖类型
        NotAuction,
        /// 挂单已过期
        ListingExpired,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 挂单出售 (固定价格)
        #[pallet::call_index(0)]
        #[pallet::weight(T::MarketWeightInfo::list_fixed_price())]
        pub fn list_fixed_price(
            origin: OriginFor<T>,
            pet_id: u64,
            price: BalanceOf<T>,
            duration: Option<BlockNumberFor<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查价格
            ensure!(price >= T::MinListingPrice::get(), Error::<T>::PriceTooLow);

            // 检查宠物所有权
            let pet = pallet_meowstar_pet::Pallet::<T>::get_pet(pet_id)
                .ok_or(Error::<T>::PetNotFound)?;
            ensure!(pet.owner == who, Error::<T>::NotPetOwner);

            // 检查是否已挂单
            ensure!(
                !PetListing::<T>::contains_key(pet_id),
                Error::<T>::PetAlreadyListed
            );

            // 创建挂单
            let listing_id = NextListingId::<T>::get();
            let current_block = frame_system::Pallet::<T>::block_number();
            let expires_at = current_block.saturating_add(
                duration.unwrap_or(T::DefaultListingDuration::get())
            );

            let listing = Listing {
                id: listing_id,
                seller: who.clone(),
                pet_id,
                listing_type: ListingType::FixedPrice,
                price,
                highest_bid: 0u32.into(),
                highest_bidder: None,
                status: ListingStatus::Active,
                created_at: current_block,
                expires_at,
            };

            Listings::<T>::insert(listing_id, listing);
            PetListing::<T>::insert(pet_id, listing_id);
            NextListingId::<T>::put(listing_id.saturating_add(1));

            Stats::<T>::mutate(|s| s.active_listings = s.active_listings.saturating_add(1));

            Self::deposit_event(Event::Listed {
                listing_id,
                seller: who,
                pet_id,
                listing_type: ListingType::FixedPrice,
                price,
            });

            Ok(())
        }

        /// 挂单拍卖
        #[pallet::call_index(1)]
        #[pallet::weight(T::MarketWeightInfo::list_auction())]
        pub fn list_auction(
            origin: OriginFor<T>,
            pet_id: u64,
            starting_price: BalanceOf<T>,
            duration: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查价格
            ensure!(starting_price >= T::MinListingPrice::get(), Error::<T>::PriceTooLow);

            // 检查宠物所有权
            let pet = pallet_meowstar_pet::Pallet::<T>::get_pet(pet_id)
                .ok_or(Error::<T>::PetNotFound)?;
            ensure!(pet.owner == who, Error::<T>::NotPetOwner);

            // 检查是否已挂单
            ensure!(
                !PetListing::<T>::contains_key(pet_id),
                Error::<T>::PetAlreadyListed
            );

            // 创建挂单
            let listing_id = NextListingId::<T>::get();
            let current_block = frame_system::Pallet::<T>::block_number();
            let expires_at = current_block.saturating_add(duration);

            let listing = Listing {
                id: listing_id,
                seller: who.clone(),
                pet_id,
                listing_type: ListingType::Auction,
                price: starting_price,
                highest_bid: 0u32.into(),
                highest_bidder: None,
                status: ListingStatus::Active,
                created_at: current_block,
                expires_at,
            };

            Listings::<T>::insert(listing_id, listing);
            PetListing::<T>::insert(pet_id, listing_id);
            NextListingId::<T>::put(listing_id.saturating_add(1));

            Stats::<T>::mutate(|s| s.active_listings = s.active_listings.saturating_add(1));

            Self::deposit_event(Event::Listed {
                listing_id,
                seller: who,
                pet_id,
                listing_type: ListingType::Auction,
                price: starting_price,
            });

            Ok(())
        }

        /// 取消挂单
        #[pallet::call_index(2)]
        #[pallet::weight(T::MarketWeightInfo::cancel_listing())]
        pub fn cancel_listing(origin: OriginFor<T>, listing_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut listing = Listings::<T>::get(listing_id)
                .ok_or(Error::<T>::ListingNotFound)?;
            ensure!(listing.seller == who, Error::<T>::NotPetOwner);
            ensure!(listing.status == ListingStatus::Active, Error::<T>::ListingNotActive);

            // 如果是拍卖且有出价，退还最高出价
            if listing.listing_type == ListingType::Auction {
                if let Some(bidder) = &listing.highest_bidder {
                    <T as pallet::Config>::Currency::unreserve(bidder, listing.highest_bid);
                }
            }

            listing.status = ListingStatus::Cancelled;
            Listings::<T>::insert(listing_id, listing.clone());
            PetListing::<T>::remove(listing.pet_id);

            Stats::<T>::mutate(|s| s.active_listings = s.active_listings.saturating_sub(1));

            Self::deposit_event(Event::ListingCancelled {
                listing_id,
                pet_id: listing.pet_id,
            });

            Ok(())
        }

        /// 购买 (固定价格)
        #[pallet::call_index(3)]
        #[pallet::weight(T::MarketWeightInfo::buy())]
        pub fn buy(origin: OriginFor<T>, listing_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut listing = Listings::<T>::get(listing_id)
                .ok_or(Error::<T>::ListingNotFound)?;
            ensure!(listing.status == ListingStatus::Active, Error::<T>::ListingNotActive);
            ensure!(listing.listing_type == ListingType::FixedPrice, Error::<T>::NotAuction);
            ensure!(listing.seller != who, Error::<T>::CannotBuyOwnPet);

            // 检查是否过期
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(current_block <= listing.expires_at, Error::<T>::ListingExpired);

            // 计算手续费
            let fee = Self::calculate_fee(listing.price);
            let seller_amount = listing.price.saturating_sub(fee);

            // 转账
            <T as pallet::Config>::Currency::transfer(
                &who,
                &listing.seller,
                seller_amount,
                ExistenceRequirement::KeepAlive,
            )?;
            <T as pallet::Config>::Currency::transfer(
                &who,
                &T::FeeReceiver::get(),
                fee,
                ExistenceRequirement::KeepAlive,
            )?;

            // 转移宠物
            pallet_meowstar_pet::Pallet::<T>::do_transfer(
                listing.pet_id,
                &listing.seller,
                &who,
            )?;

            // 更新挂单状态
            listing.status = ListingStatus::Sold;
            Listings::<T>::insert(listing_id, listing.clone());
            PetListing::<T>::remove(listing.pet_id);

            // 记录交易
            Self::record_trade(
                listing.pet_id,
                listing.seller.clone(),
                who.clone(),
                listing.price,
                fee,
            );

            Self::deposit_event(Event::Sold {
                listing_id,
                pet_id: listing.pet_id,
                seller: listing.seller,
                buyer: who,
                price: listing.price,
                fee,
            });

            Ok(())
        }

        /// 出价 (拍卖)
        #[pallet::call_index(4)]
        #[pallet::weight(T::MarketWeightInfo::place_bid())]
        pub fn place_bid(
            origin: OriginFor<T>,
            listing_id: u64,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut listing = Listings::<T>::get(listing_id)
                .ok_or(Error::<T>::ListingNotFound)?;
            ensure!(listing.status == ListingStatus::Active, Error::<T>::ListingNotActive);
            ensure!(listing.listing_type == ListingType::Auction, Error::<T>::NotAuction);
            ensure!(listing.seller != who, Error::<T>::CannotBuyOwnPet);

            // 检查是否过期
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(current_block <= listing.expires_at, Error::<T>::ListingExpired);

            // 检查出价金额
            let min_bid = if listing.highest_bid > 0u32.into() {
                let increment_rate = T::MinBidIncrement::get() as u128;
                let current_bid: u128 = listing.highest_bid.try_into().unwrap_or(0);
                let min_increment = current_bid.saturating_mul(increment_rate) / 10000;
                (current_bid.saturating_add(min_increment)).try_into().unwrap_or(listing.highest_bid)
            } else {
                listing.price
            };
            ensure!(amount >= min_bid, Error::<T>::BidTooLow);

            // 退还之前的最高出价
            if let Some(prev_bidder) = &listing.highest_bidder {
                <T as pallet::Config>::Currency::unreserve(prev_bidder, listing.highest_bid);
            }

            // 锁定新出价
            <T as pallet::Config>::Currency::reserve(&who, amount)?;

            // 更新挂单
            listing.highest_bid = amount;
            listing.highest_bidder = Some(who.clone());
            Listings::<T>::insert(listing_id, listing);

            Self::deposit_event(Event::BidPlaced {
                listing_id,
                bidder: who,
                amount,
            });

            Ok(())
        }

        /// 结束拍卖
        #[pallet::call_index(5)]
        #[pallet::weight(T::MarketWeightInfo::end_auction())]
        pub fn end_auction(origin: OriginFor<T>, listing_id: u64) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let mut listing = Listings::<T>::get(listing_id)
                .ok_or(Error::<T>::ListingNotFound)?;
            ensure!(listing.status == ListingStatus::Active, Error::<T>::ListingNotActive);
            ensure!(listing.listing_type == ListingType::Auction, Error::<T>::NotAuction);

            // 检查是否已过期
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(current_block > listing.expires_at, Error::<T>::AuctionNotEnded);

            if let Some(winner) = listing.highest_bidder.clone() {
                // 有出价者，完成交易
                let fee = Self::calculate_fee(listing.highest_bid);
                let seller_amount = listing.highest_bid.saturating_sub(fee);

                // 解锁并转账
                <T as pallet::Config>::Currency::unreserve(&winner, listing.highest_bid);
                <T as pallet::Config>::Currency::transfer(
                    &winner,
                    &listing.seller,
                    seller_amount,
                    ExistenceRequirement::KeepAlive,
                )?;
                <T as pallet::Config>::Currency::transfer(
                    &winner,
                    &T::FeeReceiver::get(),
                    fee,
                    ExistenceRequirement::KeepAlive,
                )?;

                // 转移宠物
                pallet_meowstar_pet::Pallet::<T>::do_transfer(
                    listing.pet_id,
                    &listing.seller,
                    &winner,
                )?;

                listing.status = ListingStatus::Sold;

                // 记录交易
                Self::record_trade(
                    listing.pet_id,
                    listing.seller.clone(),
                    winner.clone(),
                    listing.highest_bid,
                    fee,
                );

                Self::deposit_event(Event::AuctionEnded {
                    listing_id,
                    pet_id: listing.pet_id,
                    winner: Some(winner),
                    final_price: listing.highest_bid,
                });
            } else {
                // 无出价者，拍卖流拍
                listing.status = ListingStatus::Expired;

                Self::deposit_event(Event::AuctionEnded {
                    listing_id,
                    pet_id: listing.pet_id,
                    winner: None,
                    final_price: 0u32.into(),
                });
            }

            Listings::<T>::insert(listing_id, listing.clone());
            PetListing::<T>::remove(listing.pet_id);
            Stats::<T>::mutate(|s| s.active_listings = s.active_listings.saturating_sub(1));

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// 计算手续费
        fn calculate_fee(price: BalanceOf<T>) -> BalanceOf<T> {
            let price_u128: u128 = price.try_into().unwrap_or(0);
            let fee_rate = T::TradeFeeRate::get() as u128;
            let fee = price_u128.saturating_mul(fee_rate) / 10000;
            fee.try_into().unwrap_or(0u32.into())
        }

        /// 记录交易
        fn record_trade(
            pet_id: u64,
            seller: T::AccountId,
            buyer: T::AccountId,
            price: BalanceOf<T>,
            fee: BalanceOf<T>,
        ) {
            let trade_id = NextTradeId::<T>::get();
            let current_block = frame_system::Pallet::<T>::block_number();

            let record = TradeRecord {
                pet_id,
                seller,
                buyer,
                price,
                fee,
                block: current_block,
            };

            TradeHistory::<T>::insert(trade_id, record);
            NextTradeId::<T>::put(trade_id.saturating_add(1));

            Stats::<T>::mutate(|s| {
                s.total_volume = s.total_volume.saturating_add(price);
                s.total_trades = s.total_trades.saturating_add(1);
                s.total_fees = s.total_fees.saturating_add(fee);
                s.active_listings = s.active_listings.saturating_sub(1);
            });
        }

        /// 获取宠物的挂单信息
        pub fn get_pet_listing(pet_id: u64) -> Option<Listing<T>> {
            PetListing::<T>::get(pet_id)
                .and_then(|listing_id| Listings::<T>::get(listing_id))
        }
    }
}
