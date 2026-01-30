//! # Mock Runtime for Referral Pallet Testing
//!
//! 函数级详细中文注释：提供 Referral Pallet 的测试运行时环境

use crate as pallet_referral;
use frame_support::{
    parameter_types,
    traits::ConstU32,
};
use sp_runtime::{
    BuildStorage,
    traits::{BlakeTwo256, IdentityLookup},
};

type Block = frame_system::mocking::MockBlock<Test>;

// 函数级中文注释：构建测试运行时
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Referral: pallet_referral,
    }
);

// ========================================
// System 配置
// ========================================

parameter_types! {
    pub const BlockHashCount: u64 = 250;
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
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
    type ExtensionsWeightInfo = ();
}

// ========================================
// Balances 配置
// ========================================

parameter_types! {
    pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
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

// ========================================
// Referral 配置参数
// ========================================

parameter_types! {
    /// 推荐码最大长度
    pub const MaxCodeLen: u32 = 32;
    
    /// 推荐链最大搜索深度
    pub const MaxSearchHops: u32 = 20;

    /// 🆕 单个账户最大下线数量
    pub const MaxDownlines: u32 = 1000;
}

// ========================================
// Mock MembershipProvider
// ========================================

/// 函数级中文注释：模拟会员信息提供者
///
/// 测试环境简化规则：
/// - 账户 ID > 0 且 <= 900 的为有效会员
/// - 账户 ID > 900 的为无效会员（用于测试失败场景）
pub struct MockMembershipProvider;

impl pallet_referral::MembershipProvider<u64> for MockMembershipProvider {
    fn is_valid_member(who: &u64) -> bool {
        *who > 0 && *who <= 900
    }
}

// ========================================
// Referral Pallet 配置
// ========================================

impl pallet_referral::Config for Test {
    type MembershipProvider = MockMembershipProvider;
    type MaxCodeLen = MaxCodeLen;
    type MaxSearchHops = MaxSearchHops;
    type MaxDownlines = MaxDownlines;
    type WeightInfo = ();
}

// ========================================
// 测试辅助函数
// ========================================

/// 函数级中文注释：创建测试环境
///
/// 初始化测试环境，并为测试账户分配初始余额。
///
/// **测试账户**：
/// - Alice (1): 10,000 COS
/// - Bob (2): 10,000 COS
/// - Charlie (3): 10,000 COS
/// - Dave (4): 10,000 COS
/// - Eve (5): 10,000 COS
/// - NonMember (901): 无效会员
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 10_000_000_000_000_000),   // Alice: 10,000 COS
            (2, 10_000_000_000_000_000),   // Bob: 10,000 COS
            (3, 10_000_000_000_000_000),   // Charlie: 10,000 COS
            (4, 10_000_000_000_000_000),   // Dave: 10,000 COS
            (5, 10_000_000_000_000_000),   // Eve: 10,000 COS
            (901, 10_000_000_000_000_000), // NonMember: 非会员
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}

/// 函数级中文注释：为账户设置推荐码（测试辅助）
pub fn setup_code_for_account(account: u64, code: &[u8]) {
    use frame_support::BoundedVec;
    let bounded_code: BoundedVec<u8, MaxCodeLen> = code.to_vec().try_into().unwrap();
    pallet_referral::AccountByCode::<Test>::insert(&bounded_code, account);
    pallet_referral::CodeByAccount::<Test>::insert(account, &bounded_code);
}
