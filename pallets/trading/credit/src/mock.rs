//! # Mock Runtime for Credit Pallet

use crate as pallet_credit;
use frame_support::{
    parameter_types,
    traits::ConstU32,
};
use sp_runtime::{
    BuildStorage,
    traits::{BlakeTwo256, IdentityLookup},
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,
        Credit: pallet_credit,
    }
);

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
    type AccountData = pallet_balances::AccountData<u64>;
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
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type MaxReserves = ();
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
    type MaxHolds = ();
}

parameter_types! {
    pub const InitialBuyerCreditScore: u16 = 500;
    pub const OrderCompletedBonus: u16 = 10;
    pub const OrderDefaultPenalty: u16 = 50;
    pub const MaxReferrers: u32 = 10;
    pub const MaxEndorsers: u32 = 100;
    pub const InitialMakerCreditScore: u16 = 900;
    pub const MakerOrderCompletedBonus: u16 = 2;
    pub const MakerOrderTimeoutPenalty: u16 = 30;
    pub const MakerDisputeLossPenalty: u16 = 50;
    pub const MakerSuspensionThreshold: u16 = 750;
    pub const MakerWarningThreshold: u16 = 800;
}

impl pallet_credit::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InitialBuyerCreditScore = InitialBuyerCreditScore;
    type OrderCompletedBonus = OrderCompletedBonus;
    type OrderDefaultPenalty = OrderDefaultPenalty;
    type MaxReferrers = MaxReferrers;
    type MaxEndorsers = MaxEndorsers;
    type InitialMakerCreditScore = InitialMakerCreditScore;
    type MakerOrderCompletedBonus = MakerOrderCompletedBonus;
    type MakerOrderTimeoutPenalty = MakerOrderTimeoutPenalty;
    type MakerDisputeLossPenalty = MakerDisputeLossPenalty;
    type MakerSuspensionThreshold = MakerSuspensionThreshold;
    type MakerWarningThreshold = MakerWarningThreshold;
    type CreditWeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    t.into()
}

