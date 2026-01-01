//! # 统一隐私授权模块 - 测试 Mock
//!
//! 提供测试用的 Runtime 配置。

use crate as pallet_divination_privacy;
use frame_support::{derive_impl, parameter_types};
use frame_system as system;
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

// 配置 mock runtime
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Privacy: pallet_divination_privacy,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountData = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const MaxEncryptedDataLen: u32 = 512;
    pub const MaxEncryptedKeyLen: u32 = 128;
    pub const MaxGranteesPerRecord: u32 = 20;
    pub const MaxRecordsPerUser: u32 = 1000;
    pub const MaxProvidersPerType: u32 = 1000;
    pub const MaxGrantsPerProvider: u32 = 500;
    pub const MaxAuthorizationsPerBounty: u32 = 100;
}

impl pallet_divination_privacy::Config for Test {
    type MaxEncryptedDataLen = MaxEncryptedDataLen;
    type MaxEncryptedKeyLen = MaxEncryptedKeyLen;
    type MaxGranteesPerRecord = MaxGranteesPerRecord;
    type MaxRecordsPerUser = MaxRecordsPerUser;
    type MaxProvidersPerType = MaxProvidersPerType;
    type MaxGrantsPerProvider = MaxGrantsPerProvider;
    type MaxAuthorizationsPerBounty = MaxAuthorizationsPerBounty;
    type EventHandler = ();
    type WeightInfo = ();
}

/// 构建测试外部性
pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

/// 测试账户
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const MASTER: u64 = 10;
#[allow(dead_code)]
pub const AI_SERVICE: u64 = 20;

/// 生成测试用的公钥
pub fn test_public_key(seed: u8) -> [u8; 32] {
    let mut key = [0u8; 32];
    key[0] = seed;
    key[31] = seed;
    key
}

/// 生成测试用的加密数据
pub fn test_encrypted_data(len: usize) -> Vec<u8> {
    vec![0xAB; len]
}

/// 生成测试用的 nonce
pub fn test_nonce() -> [u8; 24] {
    [0x12; 24]
}

/// 生成测试用的 auth_tag
pub fn test_auth_tag() -> [u8; 16] {
    [0x34; 16]
}

/// 生成测试用的数据哈希
pub fn test_data_hash() -> [u8; 32] {
    [0x56; 32]
}

/// 生成测试用的加密密钥
pub fn test_encrypted_key(seed: u8) -> Vec<u8> {
    vec![seed; 64]
}
