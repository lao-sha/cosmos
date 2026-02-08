use crate as pallet_bot_consensus;
use frame_support::{
	derive_impl,
	parameter_types,
	traits::{ConstU32, ConstU128},
};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system,
		Balances: pallet_balances,
		BotConsensus: pallet_bot_consensus,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
	type AccountData = pallet_balances::AccountData<u128>;
}

impl pallet_balances::Config for Test {
	type Balance = u128;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ConstU32<0>;
	type DoneSlashHandler = ();
}

parameter_types! {
	pub const MinStake: u128 = 1000;
	pub const ExitCooldownPeriod: u64 = 10;
	pub const MaxNodes: u32 = 100;
	pub const SlashPercentage: u32 = 10;
	pub const ReporterRewardPercentage: u32 = 50;
	pub const SuspendThreshold: u16 = 2000;
	pub const ForceExitThreshold: u16 = 1000;
}

impl pallet_bot_consensus::Config for Test {
	type Currency = Balances;
	type MinStake = MinStake;
	type ExitCooldownPeriod = ExitCooldownPeriod;
	type MaxNodes = MaxNodes;
	type SlashPercentage = SlashPercentage;
	type ReporterRewardPercentage = ReporterRewardPercentage;
	type SuspendThreshold = SuspendThreshold;
	type ForceExitThreshold = ForceExitThreshold;
}

/// 构建测试外部环境
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(1, 100_000),
			(2, 100_000),
			(3, 100_000),
			(4, 100_000),
			(5, 100_000),
		],
		dev_accounts: None,
	}
	.assimilate_storage(&mut t)
	.unwrap();

	t.into()
}

/// 创建一个 NodeId
pub fn node_id(id: u8) -> crate::NodeId {
	let v: sp_std::vec::Vec<u8> = sp_std::vec![id; 4];
	v.try_into().expect("4 bytes fits in 32")
}
