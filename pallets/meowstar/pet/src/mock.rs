//! Mock runtime for pallet-meowstar-pet tests

use crate as pallet_meowstar_pet;
use frame_support::{
    derive_impl,
    parameter_types,
    traits::{ConstU32, ConstU64, ConstU128},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const POOR_ACCOUNT: u64 = 99;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        MeowstarPet: pallet_meowstar_pet,
    }
);

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
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
}

/// 简单的测试用随机数实现
pub struct TestRandomness;
impl frame_support::traits::Randomness<H256, u64> for TestRandomness {
    fn random(subject: &[u8]) -> (H256, u64) {
        let block_number = System::block_number();
        let hash = BlakeTwo256::hash(subject);
        (hash, block_number)
    }
}

parameter_types! {
    pub const HatchingFee: u128 = 100;
    pub const LevelUpBaseFee: u128 = 10;
    pub const EvolutionFeeMultiplier: u32 = 50;
    pub const MaxPetsPerAccount: u32 = 10;
}

impl pallet_meowstar_pet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Randomness = TestRandomness;
    type MaxPetsPerAccount = MaxPetsPerAccount;
    type HatchingFee = HatchingFee;
    type LevelUpBaseFee = LevelUpBaseFee;
    type EvolutionFeeMultiplier = EvolutionFeeMultiplier;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 10_000),
            (BOB, 10_000),
            (CHARLIE, 10_000),
            (POOR_ACCOUNT, 1),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
