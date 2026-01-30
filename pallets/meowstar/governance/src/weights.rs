//! Weights for pallet-meowstar-governance

use frame_support::{traits::Get, weights::Weight};

pub trait WeightInfo {
    fn create_proposal() -> Weight;
    fn vote() -> Weight;
    fn finalize_proposal() -> Weight;
    fn execute_proposal() -> Weight;
    fn emergency_pause() -> Weight;
    fn emergency_unpause() -> Weight;
    fn fund_treasury() -> Weight;
}

pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_proposal() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn vote() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn finalize_proposal() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn execute_proposal() -> Weight {
        Weight::from_parts(70_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn emergency_pause() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn emergency_unpause() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn fund_treasury() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}

impl WeightInfo for () {
    fn create_proposal() -> Weight { Weight::from_parts(60_000_000, 0) }
    fn vote() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn finalize_proposal() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn execute_proposal() -> Weight { Weight::from_parts(70_000_000, 0) }
    fn emergency_pause() -> Weight { Weight::from_parts(20_000_000, 0) }
    fn emergency_unpause() -> Weight { Weight::from_parts(20_000_000, 0) }
    fn fund_treasury() -> Weight { Weight::from_parts(30_000_000, 0) }
}
