//! # TEE 隐私计算模块 - 测试 Mock

use crate as pallet_tee_privacy;
use frame_support::{
    derive_impl,
    parameter_types,
    traits::ConstU32,
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// 配置测试运行时
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,
        TeePrivacy: pallet_tee_privacy,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type RuntimeTask = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 1;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ConstU32<0>;
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type DoneSlashHandler = ();
}

parameter_types! {
    pub const MaxNodes: u32 = 100;
    pub const MaxPendingRequests: u32 = 1000;
    pub const AttestationValidity: u32 = 14400;
    pub const MaxAllowedMrEnclaves: u32 = 10;
    pub const RequestTimeout: u32 = 100;
    pub const MinimumStake: u128 = 1000;
    pub const BaseComputeFee: u128 = 100;
    pub const SlashRatio: u32 = 100; // 10%
    pub const MaxBatchSize: u32 = 10;
}

impl pallet_tee_privacy::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxNodes = MaxNodes;
    type MaxPendingRequests = MaxPendingRequests;
    type AttestationValidity = AttestationValidity;
    type MaxAllowedMrEnclaves = MaxAllowedMrEnclaves;
    type RequestTimeout = RequestTimeout;
    type MinimumStake = MinimumStake;
    type BaseComputeFee = BaseComputeFee;
    type SlashRatio = SlashRatio;
    type MaxBatchSize = MaxBatchSize;
    type WeightInfo = ();
}

/// 构建测试外部化环境
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    // 添加初始余额
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 100_000),
            (2, 100_000),
            (3, 100_000),
            (4, 100_000),
            (5, 100_000),
            (10, 1_000_000), // 用于请求者
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    // 添加初始允许的 MRENCLAVE
    pallet_tee_privacy::GenesisConfig::<Test> {
        allowed_mr_enclaves: vec![[1u8; 32], [2u8; 32]],
        allowed_mr_signers: vec![[10u8; 32]],
        _phantom: Default::default(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        // 设置时间戳为当前时间（模拟）- 单位：秒
        pallet_timestamp::Now::<Test>::put(1704067200u64); // 2024-01-01 00:00:00 UTC (秒)
    });
    ext
}

/// 创建测试用的认证报告
pub fn create_test_attestation() -> crate::types::TeeAttestation {
    crate::types::TeeAttestation {
        tee_type: crate::types::TeeType::IntelSgx,
        mr_enclave: [1u8; 32], // 在允许列表中
        mr_signer: [10u8; 32], // 在允许列表中
        isv_prod_id: 1,
        isv_svn: 1,
        report_data: [0u8; 64],
        ias_signature: Default::default(),
        timestamp: 1704067200, // 与 mock 时间戳匹配
    }
}

/// 创建无效的认证报告（MRENCLAVE 不在允许列表中）
pub fn create_invalid_attestation() -> crate::types::TeeAttestation {
    crate::types::TeeAttestation {
        tee_type: crate::types::TeeType::IntelSgx,
        mr_enclave: [99u8; 32], // 不在允许列表中
        mr_signer: [10u8; 32],
        isv_prod_id: 1,
        isv_svn: 1,
        report_data: [0u8; 64],
        ias_signature: Default::default(),
        timestamp: 1704067200,
    }
}

/// 注册并质押 TEE 节点的辅助函数
pub fn register_and_stake_node(account: u64) {
    let enclave_pubkey = [account as u8; 32];
    let attestation = create_test_attestation();

    TeePrivacy::register_tee_node(
        RuntimeOrigin::signed(account),
        enclave_pubkey,
        attestation,
    ).unwrap();

    TeePrivacy::stake(
        RuntimeOrigin::signed(account),
        MinimumStake::get(),
    ).unwrap();
}

/// 推进区块
pub fn run_to_block(n: u64) {
    use frame_support::traits::Hooks;
    while System::block_number() < n {
        let current = System::block_number();
        TeePrivacy::on_finalize(current);
        System::set_block_number(current + 1);
        <frame_system::Pallet<Test> as Hooks<u64>>::on_initialize(current + 1);
    }
}
