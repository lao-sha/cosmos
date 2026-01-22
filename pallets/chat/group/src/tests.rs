//! 群聊举报系统单元测试

use crate::{mock::*, Error, Event, types};
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;

// 类型别名，方便使用
type ChatGroup = crate::Pallet<Test>;
type RuntimeOrigin = <Test as frame_system::Config>::RuntimeOrigin;

// ============ 基础群组功能测试 ============

#[test]
fn create_group_works() {
    new_test_ext().execute_with(|| {
        // 创建群组
        assert_ok!(ChatGroup::create_group(
            RuntimeOrigin::signed(ALICE),
            b"Test Group".to_vec(),
            Some(b"Test Description".to_vec()),
            1, // Business encryption
            true,
        ));

        // 验证群组创建（注意：群组ID是随机生成的，我们需要从事件中获取）
        let events = System::events();
        assert!(!events.is_empty());
    });
}

// ============ 举报功能测试 ============

#[test]
fn report_group_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // 假设群组ID（实际应该从创建群组后获取）
        let group_id = 10_000_000_000u64;

        // BOB 举报群组
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::IllegalContent,
            b"QmTestEvidence".to_vec(),
            b"This group contains illegal content".to_vec(),
            false, // 非匿名
        ));

        // 检查举报记录
        let report = ChatGroup::group_reports(0).unwrap();
        assert_eq!(report.reporter, BOB);
        assert_eq!(report.group_id, group_id);
        assert_eq!(report.status, types::ReportStatus::Pending);
        assert_eq!(report.is_anonymous, false);
    });
}

#[test]
fn anonymous_report_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 匿名举报
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::Spam,
            b"QmTest".to_vec(),
            b"Spam content".to_vec(),
            true, // 匿名
        ));

        let report = ChatGroup::group_reports(0).unwrap();
        assert_eq!(report.is_anonymous, true);
    });
}

#[test]
fn report_message_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;
        let message_id = 1u64;

        // 举报消息
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Message(message_id),
            types::GroupReportType::Harassment,
            b"QmTest".to_vec(),
            b"Harassment message".to_vec(),
            false,
        ));

        let report = ChatGroup::group_reports(0).unwrap();
        assert_eq!(report.target, types::GroupReportTarget::Message(message_id));
    });
}

#[test]
fn report_cooldown_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 第一次举报成功
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::Spam,
            b"QmTest1".to_vec(),
            b"Test1".to_vec(),
            false,
        ));

        // 立即再次举报失败（冷却期未过）
        assert_noop!(
            ChatGroup::report_group(
                RuntimeOrigin::signed(BOB),
                group_id,
                types::GroupReportTarget::Group,
                types::GroupReportType::Spam,
                b"QmTest2".to_vec(),
                b"Test2".to_vec(),
                false,
            ),
            Error::<Test>::ReportCooldownNotExpired
        );

        // 等待冷却期过后（10个区块）
        System::set_block_number(12);

        // 现在可以再次举报
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::Harassment,
            b"QmTest3".to_vec(),
            b"Test3".to_vec(),
            false,
        ));
    });
}

#[test]
fn evidence_cid_too_long_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 证据CID太长（超过128字节）
        let long_cid = vec![b'Q'; 129];

        assert_noop!(
            ChatGroup::report_group(
                RuntimeOrigin::signed(BOB),
                group_id,
                types::GroupReportTarget::Group,
                types::GroupReportType::Spam,
                long_cid,
                b"Test".to_vec(),
                false,
            ),
            Error::<Test>::EvidenceCidTooLong
        );
    });
}

#[test]
fn description_too_long_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 描述太长（超过512字节）
        let long_desc = vec![b'A'; 513];

        assert_noop!(
            ChatGroup::report_group(
                RuntimeOrigin::signed(BOB),
                group_id,
                types::GroupReportTarget::Group,
                types::GroupReportType::Spam,
                b"QmTest".to_vec(),
                long_desc,
                false,
            ),
            Error::<Test>::DescriptionTooLong
        );
    });
}

// ============ 撤回举报测试 ============

#[test]
fn withdraw_report_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 创建举报
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::Spam,
            b"QmTest".to_vec(),
            b"Test".to_vec(),
            false,
        ));

        // 在撤回窗口期内撤回
        System::set_block_number(3);
        assert_ok!(ChatGroup::withdraw_group_report(
            RuntimeOrigin::signed(BOB),
            0,
        ));

        // 检查状态
        let report = ChatGroup::group_reports(0).unwrap();
        assert_eq!(report.status, types::ReportStatus::Withdrawn);
    });
}

#[test]
fn withdraw_fails_after_window() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 创建举报
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::Spam,
            b"QmTest".to_vec(),
            b"Test".to_vec(),
            false,
        ));

        // 超过撤回窗口期（5个区块）
        System::set_block_number(7);

        // 撤回失败
        assert_noop!(
            ChatGroup::withdraw_group_report(
                RuntimeOrigin::signed(BOB),
                0,
            ),
            Error::<Test>::WithdrawWindowExpired
        );
    });
}

#[test]
fn withdraw_fails_if_not_reporter() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // BOB 创建举报
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::Spam,
            b"QmTest".to_vec(),
            b"Test".to_vec(),
            false,
        ));

        // CHARLIE 尝试撤回（失败）
        assert_noop!(
            ChatGroup::withdraw_group_report(
                RuntimeOrigin::signed(CHARLIE),
                0,
            ),
            Error::<Test>::NotReporter
        );
    });
}

#[test]
fn withdraw_fails_if_already_resolved() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 创建举报
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::Spam,
            b"QmTest".to_vec(),
            b"Test".to_vec(),
            false,
        ));

        // 审核举报
        assert_ok!(ChatGroup::resolve_group_report(
            RuntimeOrigin::root(),
            0,
            types::ReportStatus::Rejected,
            None,
        ));

        // 尝试撤回已处理的举报（失败）
        assert_noop!(
            ChatGroup::withdraw_group_report(
                RuntimeOrigin::signed(BOB),
                0,
            ),
            Error::<Test>::ReportAlreadyResolved
        );
    });
}

// ============ 审核举报测试 ============

#[test]
fn resolve_report_upheld_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 创建举报
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::IllegalContent,
            b"QmTest".to_vec(),
            b"Test".to_vec(),
            false,
        ));

        // 委员会审核：举报成立
        assert_ok!(ChatGroup::resolve_group_report(
            RuntimeOrigin::root(),
            0,
            types::ReportStatus::Upheld,
            Some(b"Confirmed illegal content".to_vec()),
        ));

        // 检查举报状态
        let report = ChatGroup::group_reports(0).unwrap();
        assert_eq!(report.status, types::ReportStatus::Upheld);
        assert!(report.resolved_at.is_some());

        // 检查群组被封禁
        assert!(ChatGroup::group_ban_records(group_id).is_some());
    });
}

#[test]
fn resolve_report_rejected_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 创建举报
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::Spam,
            b"QmTest".to_vec(),
            b"Test".to_vec(),
            false,
        ));

        // 委员会审核：举报驳回
        assert_ok!(ChatGroup::resolve_group_report(
            RuntimeOrigin::root(),
            0,
            types::ReportStatus::Rejected,
            None,
        ));

        // 检查举报状态
        let report = ChatGroup::group_reports(0).unwrap();
        assert_eq!(report.status, types::ReportStatus::Rejected);

        // 检查群组未被封禁
        assert!(ChatGroup::group_ban_records(group_id).is_none());
    });
}

#[test]
fn resolve_report_malicious_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 创建举报
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::Spam,
            b"QmTest".to_vec(),
            b"Test".to_vec(),
            false,
        ));

        // 委员会审核：恶意举报
        assert_ok!(ChatGroup::resolve_group_report(
            RuntimeOrigin::root(),
            0,
            types::ReportStatus::Malicious,
            None,
        ));

        // 检查举报状态
        let report = ChatGroup::group_reports(0).unwrap();
        assert_eq!(report.status, types::ReportStatus::Malicious);
    });
}

// ============ 过期举报测试 ============

#[test]
fn expire_report_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 创建举报
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::Spam,
            b"QmTest".to_vec(),
            b"Test".to_vec(),
            false,
        ));

        // 超过超时时间（100个区块）
        System::set_block_number(102);

        // 任何人都可以触发过期
        assert_ok!(ChatGroup::expire_group_report(
            RuntimeOrigin::signed(CHARLIE),
            0,
        ));

        // 检查状态
        let report = ChatGroup::group_reports(0).unwrap();
        assert_eq!(report.status, types::ReportStatus::Expired);
    });
}

#[test]
fn expire_fails_before_timeout() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 创建举报
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::Spam,
            b"QmTest".to_vec(),
            b"Test".to_vec(),
            false,
        ));

        // 未超时
        System::set_block_number(50);

        // 过期失败
        assert_noop!(
            ChatGroup::expire_group_report(
                RuntimeOrigin::signed(CHARLIE),
                0,
            ),
            Error::<Test>::ReportNotExpired
        );
    });
}

#[test]
fn expire_fails_if_already_resolved() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 创建举报
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::Spam,
            b"QmTest".to_vec(),
            b"Test".to_vec(),
            false,
        ));

        // 审核举报
        assert_ok!(ChatGroup::resolve_group_report(
            RuntimeOrigin::root(),
            0,
            types::ReportStatus::Rejected,
            None,
        ));

        // 超过超时时间
        System::set_block_number(102);

        // 尝试过期已处理的举报（失败）
        assert_noop!(
            ChatGroup::expire_group_report(
                RuntimeOrigin::signed(CHARLIE),
                0,
            ),
            Error::<Test>::ReportAlreadyResolved
        );
    });
}

// ============ 申诉测试 ============

#[test]
fn appeal_ban_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 创建举报并审核通过（封禁群组）
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::IllegalContent,
            b"QmTest".to_vec(),
            b"Test".to_vec(),
            false,
        ));

        assert_ok!(ChatGroup::resolve_group_report(
            RuntimeOrigin::root(),
            0,
            types::ReportStatus::Upheld,
            None,
        ));

        // 群组创建者申诉
        assert_ok!(ChatGroup::appeal_group_ban(
            RuntimeOrigin::signed(ALICE),
            group_id,
            b"QmAppealEvidence".to_vec(),
            b"This is a false report".to_vec(),
        ));

        // 检查申诉状态
        let ban_record = ChatGroup::group_ban_records(group_id).unwrap();
        assert_eq!(ban_record.is_appealed, true);
    });
}

#[test]
fn appeal_fails_if_not_banned() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 尝试申诉未被封禁的群组
        assert_noop!(
            ChatGroup::appeal_group_ban(
                RuntimeOrigin::signed(ALICE),
                group_id,
                b"QmAppeal".to_vec(),
                b"Appeal".to_vec(),
            ),
            Error::<Test>::GroupNotBanned
        );
    });
}

#[test]
fn appeal_fails_if_already_appealed() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 创建举报、审核、申诉
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::Spam,
            b"QmTest".to_vec(),
            b"Test".to_vec(),
            false,
        ));

        assert_ok!(ChatGroup::resolve_group_report(
            RuntimeOrigin::root(),
            0,
            types::ReportStatus::Upheld,
            None,
        ));

        assert_ok!(ChatGroup::appeal_group_ban(
            RuntimeOrigin::signed(ALICE),
            group_id,
            b"QmAppeal".to_vec(),
            b"Appeal".to_vec(),
        ));

        // 尝试再次申诉
        assert_noop!(
            ChatGroup::appeal_group_ban(
                RuntimeOrigin::signed(ALICE),
                group_id,
                b"QmAppeal2".to_vec(),
                b"Appeal2".to_vec(),
            ),
            Error::<Test>::AlreadyAppealed
        );
    });
}

#[test]
fn resolve_appeal_upheld_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 创建举报、审核、申诉
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::Spam,
            b"QmTest".to_vec(),
            b"Test".to_vec(),
            false,
        ));

        assert_ok!(ChatGroup::resolve_group_report(
            RuntimeOrigin::root(),
            0,
            types::ReportStatus::Upheld,
            None,
        ));

        assert_ok!(ChatGroup::appeal_group_ban(
            RuntimeOrigin::signed(ALICE),
            group_id,
            b"QmAppeal".to_vec(),
            b"Appeal reason".to_vec(),
        ));

        // 治理审核：申诉成立
        assert_ok!(ChatGroup::resolve_group_ban_appeal(
            RuntimeOrigin::root(),
            group_id,
            types::AppealResult::Upheld,
        ));

        // 检查封禁已解除
        assert!(ChatGroup::group_ban_records(group_id).is_none());
    });
}

#[test]
fn resolve_appeal_rejected_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let group_id = 10_000_000_000u64;

        // 创建举报、审核、申诉
        assert_ok!(ChatGroup::report_group(
            RuntimeOrigin::signed(BOB),
            group_id,
            types::GroupReportTarget::Group,
            types::GroupReportType::IllegalContent,
            b"QmTest".to_vec(),
            b"Test".to_vec(),
            false,
        ));

        assert_ok!(ChatGroup::resolve_group_report(
            RuntimeOrigin::root(),
            0,
            types::ReportStatus::Upheld,
            None,
        ));

        assert_ok!(ChatGroup::appeal_group_ban(
            RuntimeOrigin::signed(ALICE),
            group_id,
            b"QmAppeal".to_vec(),
            b"Appeal".to_vec(),
        ));

        // 治理审核：申诉驳回
        assert_ok!(ChatGroup::resolve_group_ban_appeal(
            RuntimeOrigin::root(),
            group_id,
            types::AppealResult::Rejected,
        ));

        // 检查封禁仍然存在
        let ban_record = ChatGroup::group_ban_records(group_id).unwrap();
        assert_eq!(ban_record.appeal_result, Some(types::AppealResult::Rejected));
    });
}
