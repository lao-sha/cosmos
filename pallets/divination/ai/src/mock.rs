//! Mock runtime for testing pallet-divination-ai

use crate as pallet_divination_ai;
use frame_support::{
    derive_impl,
    parameter_types,
    traits::{ConstU32, ConstU64},
};
use pallet_divination_common::{DivinationProvider, DivinationType, RarityInput};
use sp_runtime::BuildStorage;
use sp_std::vec::Vec;

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        DivinationAiPallet: pallet_divination_ai,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountData = pallet_balances::AccountData<u64>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = System;
    type Balance = u64;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<1>;
    type WeightInfo = ();
}

/// Mock DivinationProvider for testing
pub struct MockDivinationProvider;

// 用于测试的模拟数据
thread_local! {
    static MOCK_RESULTS: std::cell::RefCell<std::collections::HashMap<(DivinationType, u64), MockResult>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

#[derive(Clone)]
pub struct MockResult {
    pub creator: u64,
    pub rarity_input: RarityInput,
}

impl MockDivinationProvider {
    /// 添加模拟的占卜结果
    pub fn add_result(
        divination_type: DivinationType,
        result_id: u64,
        creator: u64,
        rarity_input: RarityInput,
    ) {
        MOCK_RESULTS.with(|r| {
            r.borrow_mut().insert(
                (divination_type, result_id),
                MockResult {
                    creator,
                    rarity_input,
                },
            );
        });
    }

    /// 清除所有模拟数据
    pub fn clear() {
        MOCK_RESULTS.with(|r| r.borrow_mut().clear());
    }
}

impl DivinationProvider<u64> for MockDivinationProvider {
    fn result_exists(divination_type: DivinationType, result_id: u64) -> bool {
        MOCK_RESULTS.with(|r| r.borrow().contains_key(&(divination_type, result_id)))
    }

    fn result_creator(divination_type: DivinationType, result_id: u64) -> Option<u64> {
        MOCK_RESULTS.with(|r| {
            r.borrow()
                .get(&(divination_type, result_id))
                .map(|m| m.creator)
        })
    }

    fn rarity_data(divination_type: DivinationType, result_id: u64) -> Option<RarityInput> {
        MOCK_RESULTS.with(|r| {
            r.borrow()
                .get(&(divination_type, result_id))
                .map(|m| m.rarity_input.clone())
        })
    }

    fn result_summary(_divination_type: DivinationType, _result_id: u64) -> Option<Vec<u8>> {
        Some(b"mock summary".to_vec())
    }

    fn is_nftable(_divination_type: DivinationType, _result_id: u64) -> bool {
        true
    }

    fn mark_as_nfted(_divination_type: DivinationType, _result_id: u64) {
        // no-op
    }
}

parameter_types! {
    pub const TreasuryAccount: u64 = 999;
}

impl pallet_divination_ai::Config for Test {
    type AiCurrency = Balances;
    type DivinationProvider = MockDivinationProvider;
    type BaseInterpretationFee = ConstU64<1_000_000_000_000>; // 1 UNIT
    type MinOracleStake = ConstU64<10_000_000_000_000>; // 10 UNIT
    type DisputeDeposit = ConstU64<500_000_000_000>; // 0.5 UNIT
    type RequestTimeout = ConstU64<100>;
    type ProcessingTimeout = ConstU64<50>;
    type DisputePeriod = ConstU64<200>;
    type MaxCidLength = ConstU32<128>;
    type MaxOracles = ConstU32<100>;
    type TreasuryAccount = TreasuryAccount;
    type ArbitratorOrigin = frame_system::EnsureRoot<u64>;
    type GovernanceOrigin = frame_system::EnsureRoot<u64>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 100_000_000_000_000), // Alice: 100 UNIT
            (2, 100_000_000_000_000), // Bob: 100 UNIT
            (3, 100_000_000_000_000), // Charlie: 100 UNIT
            (4, 100_000_000_000_000), // Oracle: 100 UNIT
            (999, 1_000_000_000),     // Treasury: minimal balance
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        MockDivinationProvider::clear();
    });
    ext
}
