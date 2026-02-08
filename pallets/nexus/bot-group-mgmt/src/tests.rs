use crate::{mock::*, ActionType};
use frame_support::assert_ok;

fn community(id: u8) -> [u8; 32] {
	let mut h = [0u8; 32];
	h[0] = id;
	h
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
fn different_communities_independent_logs() {
	new_test_ext().execute_with(|| {
		let ch1 = community(1);
		let ch2 = community(2);

		assert_ok!(BotGroupMgmt::log_action(
			RuntimeOrigin::signed(1),
			ch1, ActionType::Ban,
			[10u8; 32], [20u8; 32], 3, 1, [30u8; 32],
		));
		assert_ok!(BotGroupMgmt::log_action(
			RuntimeOrigin::signed(1),
			ch2, ActionType::Mute,
			[11u8; 32], [21u8; 32], 2, 1, [31u8; 32],
		));

		assert_eq!(BotGroupMgmt::log_count(&ch1), 1);
		assert_eq!(BotGroupMgmt::log_count(&ch2), 1);

		let log1 = BotGroupMgmt::action_logs(ch1, 0).unwrap();
		let log2 = BotGroupMgmt::action_logs(ch2, 0).unwrap();
		assert_eq!(log1.action_type, ActionType::Ban);
		assert_eq!(log2.action_type, ActionType::Mute);
	});
}

#[test]
fn action_type_variants_snapshot() {
	let types = vec![
		ActionType::Ban,
		ActionType::Unban,
		ActionType::Mute,
		ActionType::Unmute,
		ActionType::DeleteMessage,
		ActionType::PinMessage,
		ActionType::UnpinMessage,
		ActionType::ApproveJoin,
		ActionType::DeclineJoin,
		ActionType::SendMessage,
		ActionType::SetPermissions,
	];
	assert_eq!(types.len(), 11);
}
