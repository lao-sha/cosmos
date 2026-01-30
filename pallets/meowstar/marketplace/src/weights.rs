//! Weights for pallet-meowstar-marketplace

use frame_support::{traits::Get, weights::Weight};

pub trait WeightInfo {
    fn list_fixed_price() -> Weight;
    fn list_auction() -> Weight;
    fn cancel_listing() -> Weight;
    fn buy() -> Weight;
    fn place_bid() -> Weight;
    fn end_auction() -> Weight;
}

pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn list_fixed_price() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    fn list_auction() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    fn cancel_listing() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn buy() -> Weight {
        Weight::from_parts(80_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(6))
    }

    fn place_bid() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn end_auction() -> Weight {
        Weight::from_parts(90_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(6))
    }
}

impl WeightInfo for () {
    fn list_fixed_price() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn list_auction() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn cancel_listing() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn buy() -> Weight { Weight::from_parts(80_000_000, 0) }
    fn place_bid() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn end_auction() -> Weight { Weight::from_parts(90_000_000, 0) }
}
