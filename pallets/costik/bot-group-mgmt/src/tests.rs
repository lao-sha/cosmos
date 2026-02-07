use crate::{mock::*, Error, ActionType, JoinApprovalPolicy};
use frame_support::{assert_ok, assert_noop};

fn community(id: u8) -> [u8; 32] {
	let mut h = [0u8; 32];
	h[0] = id;
	h
}

#[test]
fn set_group_rules_works() {
	new_test_ext().execute_with(|| {
		let ch = community(1);

		assert_ok!(BotGroupMgmt::set_group_rules(
			RuntimeOrigin::signed(1),
			ch,
			JoinApprovalPolicy::AutoApprove,
			30,    // rate limit
			100,   // auto mute duration
			true,  // filter links
			false, // restrict mentions
		));

		let rules = BotGroupMgmt::group_rules(ch).expect("rules should exist");
		assert_eq!(rules.admin, 1);
		assert_eq!(rules.join_policy, JoinApprovalPolicy::AutoApprove);
		assert_eq!(rules.rate_limit_per_minute, 30);
		assert_eq!(rules.filter_links, true);
		assert_eq!(rules.version, 1);
	});
}

#[test]
fn update_group_rules_increments_version() {
	new_test_ext().execute_with(|| {
		let ch = community(1);

		assert_ok!(BotGroupMgmt::set_group_rules(
			RuntimeOrigin::signed(1),
			ch, JoinApprovalPolicy::AutoApprove,
			10, 0, false, false,
		));
		assert_eq!(BotGroupMgmt::group_rules(ch).unwrap().version, 1);

		// 更新规则
		assert_ok!(BotGroupMgmt::set_group_rules(
			RuntimeOrigin::signed(1),
			ch, JoinApprovalPolicy::ManualApproval,
			20, 50, true, true,
		));

		let rules = BotGroupMgmt::group_rules(ch).unwrap();
		assert_eq!(rules.version, 2);
		assert_eq!(rules.join_policy, JoinApprovalPolicy::ManualApproval);
		assert_eq!(rules.rate_limit_per_minute, 20);
	});
}

#[test]
fn update_rules_fails_not_admin() {
	new_test_ext().execute_with(|| {
		let ch = community(1);

		assert_ok!(BotGroupMgmt::set_group_rules(
			RuntimeOrigin::signed(1),
			ch, JoinApprovalPolicy::AutoApprove,
			10, 0, false, false,
		));

		// 账户 2 不是管理员
		assert_noop!(
			BotGroupMgmt::set_group_rules(
				RuntimeOrigin::signed(2),
				ch, JoinApprovalPolicy::ManualApproval,
				20, 0, false, false,
			),
			Error::<Test>::NotAdmin
		);
	});
}

#[test]
fn remove_group_rules_works() {
	new_test_ext().execute_with(|| {
		let ch = community(1);

		assert_ok!(BotGroupMgmt::set_group_rules(
			RuntimeOrigin::signed(1),
			ch, JoinApprovalPolicy::AutoApprove,
			10, 0, false, false,
		));

		assert_ok!(BotGroupMgmt::remove_group_rules(
			RuntimeOrigin::signed(1), ch,
		));

		assert!(BotGroupMgmt::group_rules(ch).is_none());
	});
}

#[test]
fn remove_rules_fails_not_admin() {
	new_test_ext().execute_with(|| {
		let ch = community(1);

		assert_ok!(BotGroupMgmt::set_group_rules(
			RuntimeOrigin::signed(1),
			ch, JoinApprovalPolicy::AutoApprove,
			10, 0, false, false,
		));

		assert_noop!(
			BotGroupMgmt::remove_group_rules(RuntimeOrigin::signed(2), ch),
			Error::<Test>::NotAdmin
		);
	});
}

#[test]
fn log_action_works() {
	new_test_ext().execute_with(|| {
		let ch = community(1);

		assert_ok!(BotGroupMgmt::log_action(
			RuntimeOrigin::signed(1),
			ch,
			ActionType::Ban,
			[10u8; 32],  // target user
			[20u8; 32],  // executor node
			3,           // consensus count
			42,          // sequence
			[30u8; 32],  // msg hash
		));

		// 日志存在
		let log = BotGroupMgmt::action_logs(ch, 0).expect("log should exist");
		assert_eq!(log.action_type, ActionType::Ban);
		assert_eq!(log.consensus_count, 3);
		assert_eq!(log.sequence, 42);

		// 计数
		assert_eq!(BotGroupMgmt::log_count(&ch), 1);
	});
}

#[test]
fn multiple_logs_increment_index() {
	new_test_ext().execute_with(|| {
		let ch = community(1);

		for i in 0..5u64 {
			assert_ok!(BotGroupMgmt::log_action(
				RuntimeOrigin::signed(1),
				ch,
				ActionType::Mute,
				[i as u8; 32],
				[20u8; 32],
				3,
				i,
				[30u8; 32],
			));
		}

		assert_eq!(BotGroupMgmt::log_count(&ch), 5);

		// 验证每个日志
		for i in 0..5u64 {
			let log = BotGroupMgmt::action_logs(ch, i).unwrap();
			assert_eq!(log.sequence, i);
		}
	});
}

#[test]
fn get_join_policy_default() {
	new_test_ext().execute_with(|| {
		// 无规则时返回默认
		let policy = BotGroupMgmt::get_join_policy(&community(99));
		assert_eq!(policy, JoinApprovalPolicy::AutoApprove);
	});
}

#[test]
fn different_communities_independent() {
	new_test_ext().execute_with(|| {
		let ch1 = community(1);
		let ch2 = community(2);

		assert_ok!(BotGroupMgmt::set_group_rules(
			RuntimeOrigin::signed(1),
			ch1, JoinApprovalPolicy::ManualApproval,
			10, 0, false, false,
		));
		assert_ok!(BotGroupMgmt::set_group_rules(
			RuntimeOrigin::signed(2),
			ch2, JoinApprovalPolicy::BalanceThreshold,
			20, 0, true, true,
		));

		assert_eq!(
			BotGroupMgmt::group_rules(ch1).unwrap().join_policy,
			JoinApprovalPolicy::ManualApproval
		);
		assert_eq!(
			BotGroupMgmt::group_rules(ch2).unwrap().join_policy,
			JoinApprovalPolicy::BalanceThreshold
		);
	});
}
