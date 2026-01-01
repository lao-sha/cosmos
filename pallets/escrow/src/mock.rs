use crate as pallet_escrow;
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64},
    PalletId,
};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        Escrow: pallet_escrow,
    }
);

// System配置
parameter_types! {
    pub const SS58Prefix: u16 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = sp_core::H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = ();
    type ExtensionsWeightInfo = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

// Balances配置
parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type DoneSlashHandler = ();
}

// Timestamp配置
parameter_types! {
    pub const MinimumPeriod: u64 = 3;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

// Escrow参数配置
parameter_types! {
    pub const EscrowPalletId: PalletId = PalletId(*b"py/escro");
    pub const MaxExpiringPerBlock: u32 = 100;
}

// Mock ExpiryPolicy - 可灵活配置过期动作
pub struct MockExpiryPolicy;
impl pallet_escrow::ExpiryPolicy<u64, u64> for MockExpiryPolicy {
    /// 函数级详细中文注释：返回过期动作
    /// 默认返回Noop，测试中可通过修改此处模拟不同场景
    fn on_expire(_id: u64) -> Result<pallet_escrow::ExpiryAction<u64>, sp_runtime::DispatchError> {
        Ok(pallet_escrow::ExpiryAction::Noop)
    }
    
    /// 函数级详细中文注释：返回当前块号
    fn now() -> u64 {
        System::block_number()
    }
}

// Escrow配置
impl pallet_escrow::pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type EscrowPalletId = EscrowPalletId;
    type AuthorizedOrigin = frame_system::EnsureRoot<u64>;
    type AdminOrigin = frame_system::EnsureRoot<u64>;
    type MaxExpiringPerBlock = MaxExpiringPerBlock;
    type ExpiryPolicy = MockExpiryPolicy;
}

/// 函数级详细中文注释：构建测试环境
/// 初始化3个测试账户，每个账户余额100,000
/// 同时给托管账户初始余额，避免ExistenceRequirement::KeepAlive问题
pub fn new_test_ext() -> sp_io::TestExternalities {
    use sp_runtime::traits::AccountIdConversion;
    
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    
    // 获取托管pallet账户
    let escrow_account: u64 = EscrowPalletId::get().into_account_truncating();
    
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 100000),
            (2, 100000),
            (3, 100000),
            (escrow_account, 1000), // 给托管账户初始余额
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}

