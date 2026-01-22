//! 群聊模块测试 Mock

use crate as pallet_chat_group;
use frame_support::{
    derive_impl,
    parameter_types,
    traits::{ConstU32, ConstU64, ConstU128, Randomness as RandomnessTrait},
    PalletId,
};
use frame_system::EnsureRoot;
use sp_runtime::{BuildStorage, traits::Hash};
use sp_core::H256;

type Block = frame_system::mocking::MockBlock<Test>;

// 简单的测试用随机数生成器
pub struct TestRandomness;
impl RandomnessTrait<H256, BlockNumberFor<Test>> for TestRandomness {
    fn random(subject: &[u8]) -> (H256, BlockNumberFor<Test>) {
        let block_number = frame_system::Pallet::<Test>::block_number();
        let hash = sp_io::hashing::blake2_256(subject);
        (H256::from(hash), block_number)
    }
}

// 配置测试运行时
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Timestamp: pallet_timestamp,
        ChatGroup: pallet_chat_group,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<1>;
    type WeightInfo = ();
}

parameter_types! {
    pub const GroupPalletId: PalletId = PalletId(*b"py/group");
}

impl pallet_chat_group::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Randomness = TestRandomness;
    type TimeProvider = Timestamp;
    type MaxGroupNameLen = ConstU32<64>;
    type MaxGroupDescriptionLen = ConstU32<512>;
    type MaxGroupMembers = ConstU32<500>;
    type MaxGroupsPerUser = ConstU32<100>;
    type MaxMessageLen = ConstU32<2048>;
    type MaxGroupMessageHistory = ConstU32<1000>;
    type MaxCidLen = ConstU32<128>;
    type MaxKeyLen = ConstU32<256>;
    type PalletId = GroupPalletId;
    type MessageRateLimit = ConstU32<60>;
    type GroupCreationCooldown = ConstU64<10>;
    type WeightInfo = ();

    // 举报系统配置
    type MinReportDeposit = ConstU128<10_000_000_000_000>; // 10 DUST
    type ReportTimeout = ConstU64<100>; // 100 blocks
    type ReportCooldownPeriod = ConstU64<10>; // 10 blocks
    type ReportWithdrawWindow = ConstU64<5>; // 5 blocks
    type ContentCommittee = EnsureRoot<u64>;
    type GovernanceOrigin = EnsureRoot<u64>;
}

/// 构建测试外部环境
pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        // Timestamp 会自动设置，不需要手动设置
    });
    ext
}

/// 测试账户
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DAVE: u64 = 4;
pub const EVE: u64 = 5;
