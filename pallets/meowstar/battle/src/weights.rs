//! Weights for pallet-meowstar-battle

use frame_support::{traits::Get, weights::Weight};

pub trait WeightInfo {
    fn start_pve_battle() -> Weight;
    fn join_pvp_queue() -> Weight;
    fn submit_action() -> Weight;
    fn leave_queue() -> Weight;
    fn surrender() -> Weight;
}

/// Default weights for the pallet
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn start_pve_battle() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn join_pvp_queue() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn submit_action() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn leave_queue() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn surrender() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }
}

impl WeightInfo for () {
    fn start_pve_battle() -> Weight {
        Weight::from_parts(60_000_000, 0)
    }

    fn join_pvp_queue() -> Weight {
        Weight::from_parts(40_000_000, 0)
    }

    fn submit_action() -> Weight {
        Weight::from_parts(50_000_000, 0)
    }

    fn leave_queue() -> Weight {
        Weight::from_parts(20_000_000, 0)
    }

    fn surrender() -> Weight {
        Weight::from_parts(40_000_000, 0)
    }
}
