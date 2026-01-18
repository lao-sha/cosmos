//! 直播间模块权重定义

use frame_support::weights::Weight;

/// 权重信息 trait
pub trait WeightInfo {
    fn create_room() -> Weight;
    fn start_live() -> Weight;
    fn pause_live() -> Weight;
    fn resume_live() -> Weight;
    fn end_live() -> Weight;
    fn update_room() -> Weight;
    fn buy_ticket() -> Weight;
    fn send_gift() -> Weight;
    fn withdraw_earnings() -> Weight;
    fn sync_live_stats() -> Weight;
    fn kick_viewer() -> Weight;
    fn remove_from_blacklist() -> Weight;
    fn ban_room() -> Weight;
    fn start_co_host() -> Weight;
    fn end_co_host() -> Weight;
    fn create_gift() -> Weight;
    fn update_gift() -> Weight;
}

/// 默认权重实现 (用于测试和开发)
impl WeightInfo for () {
    fn create_room() -> Weight {
        Weight::from_parts(50_000_000, 0)
    }

    fn start_live() -> Weight {
        Weight::from_parts(30_000_000, 0)
    }

    fn pause_live() -> Weight {
        Weight::from_parts(25_000_000, 0)
    }

    fn resume_live() -> Weight {
        Weight::from_parts(25_000_000, 0)
    }

    fn end_live() -> Weight {
        Weight::from_parts(40_000_000, 0)
    }

    fn update_room() -> Weight {
        Weight::from_parts(35_000_000, 0)
    }

    fn buy_ticket() -> Weight {
        Weight::from_parts(45_000_000, 0)
    }

    fn send_gift() -> Weight {
        Weight::from_parts(60_000_000, 0)
    }

    fn withdraw_earnings() -> Weight {
        Weight::from_parts(35_000_000, 0)
    }

    fn sync_live_stats() -> Weight {
        Weight::from_parts(25_000_000, 0)
    }

    fn kick_viewer() -> Weight {
        Weight::from_parts(30_000_000, 0)
    }

    fn remove_from_blacklist() -> Weight {
        Weight::from_parts(30_000_000, 0)
    }

    fn ban_room() -> Weight {
        Weight::from_parts(40_000_000, 0)
    }

    fn start_co_host() -> Weight {
        Weight::from_parts(35_000_000, 0)
    }

    fn end_co_host() -> Weight {
        Weight::from_parts(35_000_000, 0)
    }

    fn create_gift() -> Weight {
        Weight::from_parts(40_000_000, 0)
    }

    fn update_gift() -> Weight {
        Weight::from_parts(25_000_000, 0)
    }
}
