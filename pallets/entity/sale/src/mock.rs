//! 代币发售模块测试 mock

use crate as pallet_entity_sale;
use frame_support::{derive_impl, traits::ConstU32};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

pub const CREATOR: u64 = 1;
pub const BUYER: u64 = 2;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        EntitySale: pallet_entity_sale,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
}

impl pallet_entity_sale::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type AssetId = u64;
    type MaxPaymentOptions = ConstU32<5>;
    type MaxWhitelistSize = ConstU32<100>;
    type MaxRoundsHistory = ConstU32<50>;
    type MaxSubscriptionsPerRound = ConstU32<1000>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    t.into()
}
