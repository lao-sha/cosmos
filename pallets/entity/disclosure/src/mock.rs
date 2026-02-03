//! 财务披露模块测试 mock

use crate as pallet_entity_disclosure;
use frame_support::{
    derive_impl,
    parameter_types,
    traits::ConstU32,
};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

pub const OWNER: u64 = 1;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        EntityDisclosure: pallet_entity_disclosure,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
}

parameter_types! {
    pub const BasicDisclosureInterval: u64 = 1000;
    pub const StandardDisclosureInterval: u64 = 500;
    pub const EnhancedDisclosureInterval: u64 = 100;
}

/// Mock ShopProvider
pub struct MockShopProvider;
impl pallet_entity_common::ShopProvider<u64> for MockShopProvider {
    fn shop_exists(shop_id: u64) -> bool {
        shop_id > 0
    }
    fn shop_owner(shop_id: u64) -> Option<u64> {
        if shop_id > 0 { Some(OWNER) } else { None }
    }
    fn is_shop_active(_shop_id: u64) -> bool {
        true
    }
}

impl pallet_entity_disclosure::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type EntityProvider = MockShopProvider;
    type MaxCidLength = ConstU32<64>;
    type MaxInsiders = ConstU32<50>;
    type MaxDisclosureHistory = ConstU32<100>;
    type BasicDisclosureInterval = BasicDisclosureInterval;
    type StandardDisclosureInterval = StandardDisclosureInterval;
    type EnhancedDisclosureInterval = EnhancedDisclosureInterval;
    type MajorHolderThreshold = ConstU32<500>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    t.into()
}
