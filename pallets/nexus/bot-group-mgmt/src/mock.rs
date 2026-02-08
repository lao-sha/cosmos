use crate as pallet_bot_group_mgmt;
use frame_support::{
	derive_impl,
	parameter_types,
	traits::ConstU32,
};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system,
		BotGroupMgmt: pallet_bot_group_mgmt,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
}

impl pallet_bot_group_mgmt::Config for Test {
	type MaxLogsPerCommunity = ConstU32<1000>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();
	t.into()
}
