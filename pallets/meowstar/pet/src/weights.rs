//! Weights for pallet-meowstar-pet

use frame_support::{traits::Get, weights::Weight};

pub trait WeightInfo {
    fn hatch_pet() -> Weight;
    fn level_up() -> Weight;
    fn evolve() -> Weight;
    fn transfer() -> Weight;
    fn rename() -> Weight;
}

/// Default weights for the pallet
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn hatch_pet() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    fn level_up() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn evolve() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn transfer() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn rename() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

impl WeightInfo for () {
    fn hatch_pet() -> Weight {
        Weight::from_parts(50_000_000, 0)
    }

    fn level_up() -> Weight {
        Weight::from_parts(30_000_000, 0)
    }

    fn evolve() -> Weight {
        Weight::from_parts(40_000_000, 0)
    }

    fn transfer() -> Weight {
        Weight::from_parts(35_000_000, 0)
    }

    fn rename() -> Weight {
        Weight::from_parts(20_000_000, 0)
    }
}
