//! Weight information for the membership pallet.

use frame_support::weights::Weight;

/// Weight functions for the membership pallet.
pub trait WeightInfo {
    fn subscribe() -> Weight;
    fn upgrade_tier() -> Weight;
    fn cancel_subscription() -> Weight;
    fn check_in() -> Weight;
    fn update_profile() -> Weight;
    fn clear_sensitive_data() -> Weight;
    fn apply_provider() -> Weight;
    fn verify_provider() -> Weight;
    fn use_free_ai() -> Weight;
}

/// Default weights for testing.
impl WeightInfo for () {
    fn subscribe() -> Weight {
        Weight::from_parts(50_000_000, 0)
    }

    fn upgrade_tier() -> Weight {
        Weight::from_parts(40_000_000, 0)
    }

    fn cancel_subscription() -> Weight {
        Weight::from_parts(20_000_000, 0)
    }

    fn check_in() -> Weight {
        Weight::from_parts(80_000_000, 0)
    }

    fn update_profile() -> Weight {
        Weight::from_parts(30_000_000, 0)
    }

    fn clear_sensitive_data() -> Weight {
        Weight::from_parts(20_000_000, 0)
    }

    fn apply_provider() -> Weight {
        Weight::from_parts(25_000_000, 0)
    }

    fn verify_provider() -> Weight {
        Weight::from_parts(20_000_000, 0)
    }

    fn use_free_ai() -> Weight {
        Weight::from_parts(15_000_000, 0)
    }
}
