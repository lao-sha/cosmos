use crate as pallet_bot_registry;
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
		BotRegistry: pallet_bot_registry,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
}

parameter_types! {
	pub const MaxBotsPerOwner: u32 = 20;
	pub const MaxPlatformsPerCommunity: u32 = 5;
	pub const MaxPlatformBindingsPerUser: u32 = 5;
}

impl pallet_bot_registry::Config for Test {
	type MaxBotsPerOwner = MaxBotsPerOwner;
	type MaxPlatformsPerCommunity = MaxPlatformsPerCommunity;
	type MaxPlatformBindingsPerUser = MaxPlatformBindingsPerUser;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();
	t.into()
}
