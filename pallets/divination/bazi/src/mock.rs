//! # 测试模拟环境
//!
//! 为单元测试提供模拟的运行时环境

use crate as pallet_bazi_chart;
use frame_support::{
	derive_impl,
	parameter_types,
	traits::ConstU32,
};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

// 配置测试运行时
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		BaziChart: pallet_bazi_chart,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
	type AccountData = pallet_balances::AccountData<u128>;
}

// 配置 Balances pallet
parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type Balance = u128;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
}

// 存储押金配置参数
parameter_types! {
	/// 每 KB 存储押金（测试值：100）
	pub const StorageDepositPerKb: u128 = 100;
	/// 最小存储押金（测试值：10）
	pub const MinStorageDeposit: u128 = 10;
	/// 最大存储押金（测试值：100_000_000）
	pub const MaxStorageDeposit: u128 = 100_000_000;
}

impl pallet_bazi_chart::Config for Test {
	type WeightInfo = ();
	type MaxChartsPerAccount = ConstU32<10>;
	type MaxDaYunSteps = ConstU32<12>;
	type MaxCangGan = ConstU32<3>;
	// 存储押金相关配置
	type Currency = Balances;
	type StorageDepositPerKb = StorageDepositPerKb;
	type MinStorageDeposit = MinStorageDeposit;
	type MaxStorageDeposit = MaxStorageDeposit;
}

// 构建测试用的存储
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	// 为测试账户配置初始余额
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(1, 1_000_000_000),  // 账户 1 有 10 亿余额
			(2, 1_000_000_000),  // 账户 2 有 10 亿余额
		],
		..Default::default()
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
