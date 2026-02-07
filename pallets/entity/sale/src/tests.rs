//! 代币发售模块测试

use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_sale_round_works() {
    new_test_ext().execute_with(|| {
        let entity_id = 1u64;
        let total_supply = 1_000_000u128;

        assert_ok!(EntityTokenSale::create_sale_round(
            RuntimeOrigin::signed(CREATOR),
            entity_id,
            SaleMode::FixedPrice,
            total_supply,
            10u64.into(),  // start_block
            100u64.into(), // end_block
            false,         // kyc_required
            0,             // min_kyc_level
        ));

        let round = SaleRounds::<Test>::get(0).unwrap();
        assert_eq!(round.entity_id, entity_id);
        assert_eq!(round.total_supply, total_supply);
        assert_eq!(round.status, RoundStatus::NotStarted);
    });
}

#[test]
fn add_payment_option_works() {
    new_test_ext().execute_with(|| {
        // 创建轮次
        assert_ok!(EntityTokenSale::create_sale_round(
            RuntimeOrigin::signed(CREATOR),
            1,
            SaleMode::FixedPrice,
            1_000_000u128,
            10u64.into(),
            100u64.into(),
            false,
            0,
        ));

        // 添加支付选项
        assert_ok!(EntityTokenSale::add_payment_option(
            RuntimeOrigin::signed(CREATOR),
            0,               // round_id
            None,            // asset_id (native)
            100u128,         // price
            10u128,          // min_purchase
            10_000u128,      // max_purchase
        ));

        let round = SaleRounds::<Test>::get(0).unwrap();
        assert_eq!(round.payment_options.len(), 1);
    });
}

#[test]
fn set_vesting_config_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(EntityTokenSale::create_sale_round(
            RuntimeOrigin::signed(CREATOR),
            1,
            SaleMode::FixedPrice,
            1_000_000u128,
            10u64.into(),
            100u64.into(),
            false,
            0,
        ));

        assert_ok!(EntityTokenSale::set_vesting_config(
            RuntimeOrigin::signed(CREATOR),
            0,                          // round_id
            VestingType::Linear,        // vesting_type
            1000,                       // initial_unlock_bps (10%)
            100u64.into(),              // cliff_duration
            1000u64.into(),             // total_duration
            100u64.into(),              // unlock_interval
        ));

        let round = SaleRounds::<Test>::get(0).unwrap();
        assert_eq!(round.vesting_config.vesting_type, VestingType::Linear);
        assert_eq!(round.vesting_config.initial_unlock_bps, 1000);
    });
}

#[test]
fn start_and_end_sale_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(EntityTokenSale::create_sale_round(
            RuntimeOrigin::signed(CREATOR),
            1,
            SaleMode::FixedPrice,
            1_000_000u128,
            10u64.into(),
            100u64.into(),
            false,
            0,
        ));

        // 开始发售
        assert_ok!(EntityTokenSale::start_sale(
            RuntimeOrigin::signed(CREATOR),
            0,
        ));

        let round = SaleRounds::<Test>::get(0).unwrap();
        assert_eq!(round.status, RoundStatus::Active);

        // 结束发售
        assert_ok!(EntityTokenSale::end_sale(
            RuntimeOrigin::signed(CREATOR),
            0,
        ));

        let round = SaleRounds::<Test>::get(0).unwrap();
        assert_eq!(round.status, RoundStatus::Ended);
    });
}

#[test]
fn calculate_initial_unlock_works() {
    new_test_ext().execute_with(|| {
        let vesting = VestingConfig {
            vesting_type: VestingType::Linear,
            initial_unlock_bps: 2000, // 20%
            cliff_duration: 100u64,
            total_duration: 1000u64,
            unlock_interval: 100u64,
        };

        let total = 1_000_000u128;
        let initial = Pallet::<Test>::calculate_initial_unlock(&vesting, total);
        assert_eq!(initial, 200_000u128); // 20% of 1M
    });
}
