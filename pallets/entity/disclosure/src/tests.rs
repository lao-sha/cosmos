//! 财务披露模块测试

use crate::{mock::*, pallet::*};
use frame_support::{assert_noop, assert_ok};

// ==================== configure_disclosure ====================

#[test]
fn configure_disclosure_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(EntityDisclosure::configure_disclosure(
            RuntimeOrigin::signed(OWNER),
            ENTITY_ID,
            DisclosureLevel::Standard,
            true,
            50u64,
            100u64,
        ));

        let config = DisclosureConfigs::<Test>::get(ENTITY_ID).unwrap();
        assert_eq!(config.level, DisclosureLevel::Standard);
        assert!(config.insider_trading_control);
        assert_eq!(config.blackout_period_before, 50);
        assert_eq!(config.blackout_period_after, 100);
        // next_required = now(1) + StandardInterval(500) = 501
        assert_eq!(config.next_required_disclosure, 501);
        assert_eq!(config.violation_count, 0);
    });
}

#[test]
fn configure_disclosure_fails_not_owner() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            EntityDisclosure::configure_disclosure(
                RuntimeOrigin::signed(ALICE), ENTITY_ID,
                DisclosureLevel::Basic, false, 0u64, 0u64,
            ),
            Error::<Test>::NotAdmin
        );
    });
}

#[test]
fn configure_disclosure_fails_entity_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            EntityDisclosure::configure_disclosure(
                RuntimeOrigin::signed(OWNER), 999,
                DisclosureLevel::Basic, false, 0u64, 0u64,
            ),
            Error::<Test>::EntityNotFound
        );
    });
}

// ==================== publish_disclosure ====================

#[test]
fn publish_disclosure_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(EntityDisclosure::publish_disclosure(
            RuntimeOrigin::signed(OWNER),
            ENTITY_ID,
            DisclosureType::AnnualReport,
            b"QmTest123".to_vec(),
            Some(b"QmSummary".to_vec()),
        ));

        let disclosure = Disclosures::<Test>::get(0).unwrap();
        assert_eq!(disclosure.entity_id, ENTITY_ID);
        assert_eq!(disclosure.disclosure_type, DisclosureType::AnnualReport);
        assert_eq!(disclosure.status, DisclosureStatus::Published);
        assert_eq!(disclosure.discloser, OWNER);
        assert_eq!(disclosure.previous_id, None);

        // 历史索引已更新
        let history = EntityDisclosures::<Test>::get(ENTITY_ID);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0], 0);

        // NextDisclosureId 已递增
        assert_eq!(NextDisclosureId::<Test>::get(), 1);
    });
}

#[test]
fn publish_disclosure_auto_blackout() {
    new_test_ext().execute_with(|| {
        // 先配置：启用内幕控制 + blackout_after=50
        assert_ok!(EntityDisclosure::configure_disclosure(
            RuntimeOrigin::signed(OWNER), ENTITY_ID,
            DisclosureLevel::Standard, true, 0u64, 50u64,
        ));

        assert_ok!(EntityDisclosure::publish_disclosure(
            RuntimeOrigin::signed(OWNER), ENTITY_ID,
            DisclosureType::QuarterlyReport, b"QmReport".to_vec(), None,
        ));

        // 应自动进入黑窗口期
        assert!(Pallet::<Test>::is_in_blackout(ENTITY_ID));
        let (start, end) = BlackoutPeriods::<Test>::get(ENTITY_ID).unwrap();
        assert_eq!(start, 1);
        assert_eq!(end, 51);
    });
}

#[test]
fn publish_disclosure_no_blackout_when_disabled() {
    new_test_ext().execute_with(|| {
        // 配置：不启用内幕控制
        assert_ok!(EntityDisclosure::configure_disclosure(
            RuntimeOrigin::signed(OWNER), ENTITY_ID,
            DisclosureLevel::Basic, false, 0u64, 50u64,
        ));

        assert_ok!(EntityDisclosure::publish_disclosure(
            RuntimeOrigin::signed(OWNER), ENTITY_ID,
            DisclosureType::AnnualReport, b"QmReport".to_vec(), None,
        ));

        // 不应进入黑窗口
        assert!(!Pallet::<Test>::is_in_blackout(ENTITY_ID));
    });
}

#[test]
fn publish_disclosure_fails_not_owner() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            EntityDisclosure::publish_disclosure(
                RuntimeOrigin::signed(ALICE), ENTITY_ID,
                DisclosureType::AnnualReport, b"Qm".to_vec(), None,
            ),
            Error::<Test>::NotAdmin
        );
    });
}

#[test]
fn publish_disclosure_fails_cid_too_long() {
    new_test_ext().execute_with(|| {
        let long_cid = vec![0u8; 65]; // MaxCidLength = 64
        assert_noop!(
            EntityDisclosure::publish_disclosure(
                RuntimeOrigin::signed(OWNER), ENTITY_ID,
                DisclosureType::AnnualReport, long_cid, None,
            ),
            Error::<Test>::CidTooLong
        );
    });
}

#[test]
fn publish_disclosure_fails_history_full() {
    new_test_ext().execute_with(|| {
        // MaxDisclosureHistory = 10
        for _ in 0..10 {
            assert_ok!(EntityDisclosure::publish_disclosure(
                RuntimeOrigin::signed(OWNER), ENTITY_ID,
                DisclosureType::Other, b"QmCid".to_vec(), None,
            ));
        }
        // 11th should fail
        assert_noop!(
            EntityDisclosure::publish_disclosure(
                RuntimeOrigin::signed(OWNER), ENTITY_ID,
                DisclosureType::Other, b"QmCid".to_vec(), None,
            ),
            Error::<Test>::HistoryFull
        );
    });
}

// ==================== withdraw_disclosure ====================

#[test]
fn withdraw_disclosure_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(EntityDisclosure::publish_disclosure(
            RuntimeOrigin::signed(OWNER), ENTITY_ID,
            DisclosureType::AnnualReport, b"QmCid".to_vec(), None,
        ));

        assert_ok!(EntityDisclosure::withdraw_disclosure(
            RuntimeOrigin::signed(OWNER), 0,
        ));

        let record = Disclosures::<Test>::get(0).unwrap();
        assert_eq!(record.status, DisclosureStatus::Withdrawn);
    });
}

#[test]
fn withdraw_disclosure_fails_not_published() {
    new_test_ext().execute_with(|| {
        assert_ok!(EntityDisclosure::publish_disclosure(
            RuntimeOrigin::signed(OWNER), ENTITY_ID,
            DisclosureType::AnnualReport, b"QmCid".to_vec(), None,
        ));
        assert_ok!(EntityDisclosure::withdraw_disclosure(RuntimeOrigin::signed(OWNER), 0));

        // 再次撤回已撤回的应失败
        assert_noop!(
            EntityDisclosure::withdraw_disclosure(RuntimeOrigin::signed(OWNER), 0),
            Error::<Test>::InvalidDisclosureStatus
        );
    });
}

#[test]
fn withdraw_disclosure_fails_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            EntityDisclosure::withdraw_disclosure(RuntimeOrigin::signed(OWNER), 999),
            Error::<Test>::DisclosureNotFound
        );
    });
}

// ==================== correct_disclosure ====================

#[test]
fn correct_disclosure_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(EntityDisclosure::publish_disclosure(
            RuntimeOrigin::signed(OWNER), ENTITY_ID,
            DisclosureType::AnnualReport, b"QmOld".to_vec(), None,
        ));

        assert_ok!(EntityDisclosure::correct_disclosure(
            RuntimeOrigin::signed(OWNER), 0,
            b"QmNew".to_vec(), None,
        ));

        // 旧记录标记为 Corrected
        let old = Disclosures::<Test>::get(0).unwrap();
        assert_eq!(old.status, DisclosureStatus::Corrected);

        // 新记录指向旧记录
        let new_rec = Disclosures::<Test>::get(1).unwrap();
        assert_eq!(new_rec.status, DisclosureStatus::Published);
        assert_eq!(new_rec.previous_id, Some(0));
    });
}

#[test]
fn correct_disclosure_fails_withdrawn() {
    new_test_ext().execute_with(|| {
        assert_ok!(EntityDisclosure::publish_disclosure(
            RuntimeOrigin::signed(OWNER), ENTITY_ID,
            DisclosureType::AnnualReport, b"QmOld".to_vec(), None,
        ));
        assert_ok!(EntityDisclosure::withdraw_disclosure(RuntimeOrigin::signed(OWNER), 0));

        // H2: 不能更正已撤回的记录
        assert_noop!(
            EntityDisclosure::correct_disclosure(
                RuntimeOrigin::signed(OWNER), 0, b"QmNew".to_vec(), None,
            ),
            Error::<Test>::InvalidDisclosureStatus
        );
    });
}

// ==================== add_insider / remove_insider ====================

#[test]
fn add_insider_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(EntityDisclosure::add_insider(
            RuntimeOrigin::signed(OWNER), ENTITY_ID, ALICE, InsiderRole::Admin,
        ));
        assert!(Pallet::<Test>::is_insider(ENTITY_ID, &ALICE));
    });
}

#[test]
fn add_insider_fails_duplicate() {
    new_test_ext().execute_with(|| {
        assert_ok!(EntityDisclosure::add_insider(
            RuntimeOrigin::signed(OWNER), ENTITY_ID, ALICE, InsiderRole::Admin,
        ));
        assert_noop!(
            EntityDisclosure::add_insider(
                RuntimeOrigin::signed(OWNER), ENTITY_ID, ALICE, InsiderRole::Auditor,
            ),
            Error::<Test>::InsiderExists
        );
    });
}

#[test]
fn add_insider_reactivates_removed() {
    new_test_ext().execute_with(|| {
        // H3: 添加后移除再添加，应重用旧记录
        assert_ok!(EntityDisclosure::add_insider(
            RuntimeOrigin::signed(OWNER), ENTITY_ID, ALICE, InsiderRole::Admin,
        ));
        assert_ok!(EntityDisclosure::remove_insider(
            RuntimeOrigin::signed(OWNER), ENTITY_ID, ALICE,
        ));
        assert!(!Pallet::<Test>::is_insider(ENTITY_ID, &ALICE));

        // 重新添加（不同角色）
        assert_ok!(EntityDisclosure::add_insider(
            RuntimeOrigin::signed(OWNER), ENTITY_ID, ALICE, InsiderRole::Auditor,
        ));
        assert!(Pallet::<Test>::is_insider(ENTITY_ID, &ALICE));

        // BoundedVec 长度不应增加
        let insiders = Insiders::<Test>::get(ENTITY_ID);
        assert_eq!(insiders.len(), 1);
    });
}

#[test]
fn add_insider_fails_full() {
    new_test_ext().execute_with(|| {
        // MaxInsiders = 5
        for i in 10..15 {
            assert_ok!(EntityDisclosure::add_insider(
                RuntimeOrigin::signed(OWNER), ENTITY_ID, i, InsiderRole::Advisor,
            ));
        }
        assert_noop!(
            EntityDisclosure::add_insider(
                RuntimeOrigin::signed(OWNER), ENTITY_ID, 99, InsiderRole::Advisor,
            ),
            Error::<Test>::InsidersFull
        );
    });
}

#[test]
fn remove_insider_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(EntityDisclosure::add_insider(
            RuntimeOrigin::signed(OWNER), ENTITY_ID, ALICE, InsiderRole::Admin,
        ));
        assert_ok!(EntityDisclosure::remove_insider(
            RuntimeOrigin::signed(OWNER), ENTITY_ID, ALICE,
        ));
        assert!(!Pallet::<Test>::is_insider(ENTITY_ID, &ALICE));
    });
}

#[test]
fn remove_insider_fails_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            EntityDisclosure::remove_insider(
                RuntimeOrigin::signed(OWNER), ENTITY_ID, ALICE,
            ),
            Error::<Test>::InsiderNotFound
        );
    });
}

// ==================== blackout ====================

#[test]
fn blackout_period_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(EntityDisclosure::start_blackout(
            RuntimeOrigin::signed(OWNER), ENTITY_ID, 100u64,
        ));
        assert!(Pallet::<Test>::is_in_blackout(ENTITY_ID));

        assert_ok!(EntityDisclosure::end_blackout(
            RuntimeOrigin::signed(OWNER), ENTITY_ID,
        ));
        assert!(!Pallet::<Test>::is_in_blackout(ENTITY_ID));
    });
}

#[test]
fn blackout_expires_naturally() {
    new_test_ext().execute_with(|| {
        assert_ok!(EntityDisclosure::start_blackout(
            RuntimeOrigin::signed(OWNER), ENTITY_ID, 50u64,
        ));
        assert!(Pallet::<Test>::is_in_blackout(ENTITY_ID));

        advance_blocks(51);
        assert!(!Pallet::<Test>::is_in_blackout(ENTITY_ID));
    });
}

// ==================== can_insider_trade ====================

#[test]
fn can_insider_trade_non_insider_always_true() {
    new_test_ext().execute_with(|| {
        // 非内幕人员始终可交易（即使在黑窗口期）
        assert_ok!(EntityDisclosure::configure_disclosure(
            RuntimeOrigin::signed(OWNER), ENTITY_ID,
            DisclosureLevel::Standard, true, 0u64, 100u64,
        ));
        assert_ok!(EntityDisclosure::start_blackout(
            RuntimeOrigin::signed(OWNER), ENTITY_ID, 100u64,
        ));
        assert!(Pallet::<Test>::can_insider_trade(ENTITY_ID, &ALICE));
    });
}

#[test]
fn can_insider_trade_blocked_during_blackout() {
    new_test_ext().execute_with(|| {
        assert_ok!(EntityDisclosure::configure_disclosure(
            RuntimeOrigin::signed(OWNER), ENTITY_ID,
            DisclosureLevel::Standard, true, 0u64, 100u64,
        ));
        assert_ok!(EntityDisclosure::add_insider(
            RuntimeOrigin::signed(OWNER), ENTITY_ID, ALICE, InsiderRole::Admin,
        ));
        assert_ok!(EntityDisclosure::start_blackout(
            RuntimeOrigin::signed(OWNER), ENTITY_ID, 100u64,
        ));

        // 内幕人员在黑窗口期内不可交易
        assert!(!Pallet::<Test>::can_insider_trade(ENTITY_ID, &ALICE));

        // 黑窗口结束后可交易
        advance_blocks(101);
        assert!(Pallet::<Test>::can_insider_trade(ENTITY_ID, &ALICE));
    });
}

#[test]
fn can_insider_trade_allowed_when_control_disabled() {
    new_test_ext().execute_with(|| {
        // 未启用内幕控制，即使在黑窗口期也可交易
        assert_ok!(EntityDisclosure::configure_disclosure(
            RuntimeOrigin::signed(OWNER), ENTITY_ID,
            DisclosureLevel::Standard, false, 0u64, 0u64,
        ));
        assert_ok!(EntityDisclosure::add_insider(
            RuntimeOrigin::signed(OWNER), ENTITY_ID, ALICE, InsiderRole::Admin,
        ));
        assert_ok!(EntityDisclosure::start_blackout(
            RuntimeOrigin::signed(OWNER), ENTITY_ID, 100u64,
        ));

        assert!(Pallet::<Test>::can_insider_trade(ENTITY_ID, &ALICE));
    });
}

// ==================== helper 函数 ====================

#[test]
fn calculate_next_disclosure_intervals() {
    new_test_ext().execute_with(|| {
        let now = 100u64;
        assert_eq!(Pallet::<Test>::calculate_next_disclosure(DisclosureLevel::Basic, now), 1100);
        assert_eq!(Pallet::<Test>::calculate_next_disclosure(DisclosureLevel::Standard, now), 600);
        assert_eq!(Pallet::<Test>::calculate_next_disclosure(DisclosureLevel::Enhanced, now), 200);
        assert_eq!(Pallet::<Test>::calculate_next_disclosure(DisclosureLevel::Full, now), 100); // +0
    });
}

#[test]
fn is_disclosure_overdue_works() {
    new_test_ext().execute_with(|| {
        // 无配置时不逾期
        assert!(!Pallet::<Test>::is_disclosure_overdue(ENTITY_ID));

        assert_ok!(EntityDisclosure::configure_disclosure(
            RuntimeOrigin::signed(OWNER), ENTITY_ID,
            DisclosureLevel::Enhanced, false, 0u64, 0u64,
        ));
        // 刚配置，next_required = 1+100=101
        assert!(!Pallet::<Test>::is_disclosure_overdue(ENTITY_ID));

        advance_blocks(101);
        assert!(Pallet::<Test>::is_disclosure_overdue(ENTITY_ID));
    });
}

#[test]
fn get_disclosure_level_default() {
    new_test_ext().execute_with(|| {
        assert_eq!(Pallet::<Test>::get_disclosure_level(ENTITY_ID), DisclosureLevel::Basic);
    });
}
