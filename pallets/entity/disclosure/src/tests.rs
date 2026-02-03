//! 财务披露模块测试

use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};

#[test]
fn configure_disclosure_works() {
    new_test_ext().execute_with(|| {
        let entity_id = 1u64;
        
        // 配置披露设置
        assert_ok!(EntityDisclosure::configure_disclosure(
            RuntimeOrigin::signed(OWNER),
            entity_id,
            DisclosureLevel::Standard,
            true,  // insider_trading_control
            100u64.into(),  // blackout_before
            100u64.into(),  // blackout_after
        ));

        // 验证配置
        let config = DisclosureConfigs::<Test>::get(entity_id).unwrap();
        assert_eq!(config.level, DisclosureLevel::Standard);
        assert!(config.insider_trading_control);
    });
}

#[test]
fn publish_disclosure_works() {
    new_test_ext().execute_with(|| {
        let entity_id = 1u64;
        
        // 发布披露
        assert_ok!(EntityDisclosure::publish_disclosure(
            RuntimeOrigin::signed(OWNER),
            entity_id,
            DisclosureType::AnnualReport,
            b"QmTest123".to_vec(),
            Some(b"QmSummary".to_vec()),
        ));

        // 验证披露记录
        let disclosure = Disclosures::<Test>::get(0).unwrap();
        assert_eq!(disclosure.entity_id, entity_id);
        assert_eq!(disclosure.disclosure_type, DisclosureType::AnnualReport);
        assert_eq!(disclosure.status, DisclosureStatus::Published);
    });
}

#[test]
fn add_insider_works() {
    new_test_ext().execute_with(|| {
        let entity_id = 1u64;
        let insider = 2u64;
        
        assert_ok!(EntityDisclosure::add_insider(
            RuntimeOrigin::signed(OWNER),
            entity_id,
            insider,
            InsiderRole::Admin,
        ));

        // 验证内幕人员
        assert!(Pallet::<Test>::is_insider(entity_id, &insider));
    });
}

#[test]
fn blackout_period_works() {
    new_test_ext().execute_with(|| {
        let entity_id = 1u64;
        
        // 开始黑窗口期
        assert_ok!(EntityDisclosure::start_blackout(
            RuntimeOrigin::signed(OWNER),
            entity_id,
            100u64.into(),
        ));

        // 验证黑窗口期
        assert!(Pallet::<Test>::is_in_blackout(entity_id));
        
        // 结束黑窗口期
        assert_ok!(EntityDisclosure::end_blackout(
            RuntimeOrigin::signed(OWNER),
            entity_id,
        ));
        
        assert!(!Pallet::<Test>::is_in_blackout(entity_id));
    });
}
