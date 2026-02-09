use crate as pallet_bot_consensus;
use frame_support::{
	derive_impl,
	parameter_types,
	traits::{ConstU32, ConstU128},
};
use sp_runtime::{BuildStorage, Perbill};
use std::cell::RefCell;
use std::collections::HashMap;

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

// ═══════════════════════════════════════════════════════════════
// Mock Bot Registry
// ═══════════════════════════════════════════════════════════════

thread_local! {
	/// bot_id_hash → owner_account_id
	static BOTS: RefCell<HashMap<[u8; 32], u64>> = RefCell::new(HashMap::new());
}

/// 注册一个 Mock Bot（测试辅助）
pub fn register_mock_bot(bot_id_hash: [u8; 32], owner: u64) {
	BOTS.with(|b| b.borrow_mut().insert(bot_id_hash, owner));
}

pub struct MockBotRegistry;

impl pallet_bot_consensus::BotRegistryProvider<u64> for MockBotRegistry {
	fn bot_exists(bot_id_hash: &[u8; 32]) -> bool {
		BOTS.with(|b| b.borrow().contains_key(bot_id_hash))
	}

	fn is_bot_owner(bot_id_hash: &[u8; 32], who: &u64) -> bool {
		BOTS.with(|b| {
			b.borrow().get(bot_id_hash).map(|o| o == who).unwrap_or(false)
		})
	}
}

parameter_types! {
	pub const MinStake: u128 = 1000;
	pub const ExitCooldownPeriod: u64 = 10;
	pub const MaxNodes: u32 = 100;
	pub const SlashPercentage: u32 = 10;
	pub const ReporterRewardPercentage: u32 = 50;
	pub const SuspendThreshold: u16 = 2000;
	pub const ForceExitThreshold: u16 = 1000;
	// 奖励参数
	pub const EraLength: u64 = 10;                                   // 10 blocks per Era (测试用)
	pub const InflationPerEra: u128 = 100;                           // 100 units/Era
	pub const MinUptimeForReward: Perbill = Perbill::from_percent(80);
	pub const MaxRewardShare: Perbill = Perbill::from_percent(30);
	pub const BasicFeePerEra: u128 = 10;                             // 10 units/Era
	pub const ProFeePerEra: u128 = 30;                               // 30 units/Era
	pub const EnterpriseFeePerEra: u128 = 100;                       // 100 units/Era
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
	type EraLength = EraLength;
	type InflationPerEra = InflationPerEra;
	type MinUptimeForReward = MinUptimeForReward;
	type MaxRewardShare = MaxRewardShare;
	type BasicFeePerEra = BasicFeePerEra;
	type ProFeePerEra = ProFeePerEra;
	type EnterpriseFeePerEra = EnterpriseFeePerEra;
	type BotRegistry = MockBotRegistry;
}

/// 构建测试外部环境
pub fn new_test_ext() -> sp_io::TestExternalities {
	// 清理 thread_local 状态
	BOTS.with(|b| b.borrow_mut().clear());

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

/// 创建一个 bot_id_hash
pub fn bot_hash(id: u8) -> [u8; 32] {
	[id; 32]
}

/// 注册节点并激活
pub fn register_and_activate(operator: u64, nid: &crate::NodeId) {
	assert!(BotConsensus::register_node(
		RuntimeOrigin::signed(operator),
		nid.clone(),
		[operator as u8; 32],
		[operator as u8 + 10; 32],
	).is_ok());
	assert!(BotConsensus::activate_node(
		RuntimeOrigin::signed(operator),
		nid.clone(),
	).is_ok());
}
