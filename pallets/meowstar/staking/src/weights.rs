//! Weights for pallet-meowstar-staking

use frame_support::{traits::Get, weights::Weight};

pub trait WeightInfo {
    fn stake() -> Weight;
    fn unstake() -> Weight;
    fn claim_rewards() -> Weight;
    fn fund_reward_pool() -> Weight;
}

pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn stake() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    fn unstake() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    fn claim_rewards() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn fund_reward_pool() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}

impl WeightInfo for () {
    fn stake() -> Weight {
        Weight::from_parts(50_000_000, 0)
    }

    fn unstake() -> Weight {
        Weight::from_parts(60_000_000, 0)
    }

    fn claim_rewards() -> Weight {
        Weight::from_parts(40_000_000, 0)
    }

    fn fund_reward_pool() -> Weight {
        Weight::from_parts(30_000_000, 0)
    }
}
