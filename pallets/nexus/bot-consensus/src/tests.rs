use crate::{mock::*, Error, NodeStatus, NodeId};
use frame_support::{assert_ok, assert_noop};

fn make_node_id(id: u8) -> NodeId {
	let v: sp_std::vec::Vec<u8> = sp_std::vec![id; 4];
	v.try_into().expect("4 bytes fits in 32")
}

#[test]
fn register_node_works() {
	new_test_ext().execute_with(|| {
		let nid = make_node_id(1);
		let pubkey = [1u8; 32];
		let endpoint = [2u8; 32];

		assert_ok!(BotConsensus::register_node(
			RuntimeOrigin::signed(1),
			nid.clone(),
			pubkey,
			endpoint,
		));

		// 节点存在
		let node = BotConsensus::nodes(&nid).expect("node should exist");
		assert_eq!(node.operator, 1);
		assert_eq!(node.node_public_key, pubkey);
		assert_eq!(node.endpoint_hash, endpoint);
		assert_eq!(node.status, NodeStatus::Probation);
		assert_eq!(node.reputation, 5000);
		assert_eq!(node.stake, 1000);

		// 在活跃列表中
		let active = BotConsensus::active_node_list();
		assert!(active.contains(&nid));

		// 运营者节点列表
		let op_nodes = BotConsensus::operator_nodes(1u64);
		assert!(op_nodes.contains(&nid));

		// 质押已锁定
		assert_eq!(Balances::reserved_balance(1), 1000);
	});
}

#[test]
fn register_node_fails_duplicate() {
	new_test_ext().execute_with(|| {
		let nid = make_node_id(1);
		assert_ok!(BotConsensus::register_node(
			RuntimeOrigin::signed(1),
			nid.clone(),
			[1u8; 32],
			[2u8; 32],
		));

		assert_noop!(
			BotConsensus::register_node(
				RuntimeOrigin::signed(2),
				nid,
				[3u8; 32],
				[4u8; 32],
			),
			Error::<Test>::NodeAlreadyExists
		);
	});
}

#[test]
fn register_node_fails_insufficient_balance() {
	new_test_ext().execute_with(|| {
		// 账户 10 没有余额
		let nid = make_node_id(1);
		assert_noop!(
			BotConsensus::register_node(
				RuntimeOrigin::signed(10),
				nid,
				[1u8; 32],
				[2u8; 32],
			),
			Error::<Test>::InsufficientStake
		);
	});
}

#[test]
fn request_exit_works() {
	new_test_ext().execute_with(|| {
		let nid = make_node_id(1);
		assert_ok!(BotConsensus::register_node(
			RuntimeOrigin::signed(1),
			nid.clone(),
			[1u8; 32],
			[2u8; 32],
		));

		assert_ok!(BotConsensus::request_exit(
			RuntimeOrigin::signed(1),
			nid.clone(),
		));

		// 状态变为 Exiting
		let node = BotConsensus::nodes(&nid).unwrap();
		assert_eq!(node.status, NodeStatus::Exiting);

		// 从活跃列表移除
		let active = BotConsensus::active_node_list();
		assert!(!active.contains(&nid));

		// 质押仍锁定
		assert_eq!(Balances::reserved_balance(1), 1000);
	});
}

#[test]
fn request_exit_fails_not_operator() {
	new_test_ext().execute_with(|| {
		let nid = make_node_id(1);
		assert_ok!(BotConsensus::register_node(
			RuntimeOrigin::signed(1),
			nid.clone(),
			[1u8; 32],
			[2u8; 32],
		));

		assert_noop!(
			BotConsensus::request_exit(RuntimeOrigin::signed(2), nid),
			Error::<Test>::NotNodeOperator
		);
	});
}

#[test]
fn finalize_exit_works() {
	new_test_ext().execute_with(|| {
		let nid = make_node_id(1);
		assert_ok!(BotConsensus::register_node(
			RuntimeOrigin::signed(1),
			nid.clone(),
			[1u8; 32],
			[2u8; 32],
		));

		assert_ok!(BotConsensus::request_exit(
			RuntimeOrigin::signed(1),
			nid.clone(),
		));

		// 冷却期未结束，应失败
		assert_noop!(
			BotConsensus::finalize_exit(RuntimeOrigin::signed(1), nid.clone()),
			Error::<Test>::CooldownNotExpired
		);

		// 推进区块到冷却期结束
		System::set_block_number(11);

		assert_ok!(BotConsensus::finalize_exit(
			RuntimeOrigin::signed(1),
			nid.clone(),
		));

		// 节点已移除
		assert!(BotConsensus::nodes(&nid).is_none());

		// 质押已释放
		assert_eq!(Balances::reserved_balance(1), 0);
		assert_eq!(Balances::free_balance(1), 100_000);

		// 运营者节点列表已清理
		let op_nodes = BotConsensus::operator_nodes(1u64);
		assert!(!op_nodes.contains(&nid));
	});
}

#[test]
fn submit_confirmations_works() {
	new_test_ext().execute_with(|| {
		// 先注册节点
		let nid = make_node_id(1);
		assert_ok!(BotConsensus::register_node(
			RuntimeOrigin::signed(1),
			nid.clone(),
			[1u8; 32],
			[2u8; 32],
		));

		let msg_id = [10u8; 32];
		let msg_hash = [20u8; 32];

		assert_ok!(BotConsensus::submit_confirmations(
			RuntimeOrigin::signed(1),
			vec![(msg_id, 2, 42, msg_hash, vec![nid.clone()])],
		));

		// 确认记录存在
		let conf = BotConsensus::message_confirmations(msg_id).unwrap();
		assert_eq!(conf.owner, 2);
		assert_eq!(conf.sequence, 42);
		assert_eq!(conf.msg_hash, msg_hash);
		assert!(conf.confirmed_by.contains(&nid));
	});
}

#[test]
fn submit_confirmations_skips_duplicates() {
	new_test_ext().execute_with(|| {
		let nid = make_node_id(1);
		assert_ok!(BotConsensus::register_node(
			RuntimeOrigin::signed(1),
			nid.clone(),
			[1u8; 32],
			[2u8; 32],
		));

		let msg_id = [10u8; 32];

		// 第一次提交
		assert_ok!(BotConsensus::submit_confirmations(
			RuntimeOrigin::signed(1),
			vec![(msg_id, 2, 1, [20u8; 32], vec![nid.clone()])],
		));

		// 重复提交同一 msg_id — 不报错，静默跳过
		assert_ok!(BotConsensus::submit_confirmations(
			RuntimeOrigin::signed(1),
			vec![(msg_id, 2, 1, [30u8; 32], vec![nid.clone()])],
		));

		// 仍然是第一次的 hash
		let conf = BotConsensus::message_confirmations(msg_id).unwrap();
		assert_eq!(conf.msg_hash, [20u8; 32]);
	});
}

#[test]
fn report_equivocation_works() {
	new_test_ext().execute_with(|| {
		let owner = 2u64;
		let sequence = 42u64;

		assert_ok!(BotConsensus::report_equivocation(
			RuntimeOrigin::signed(1),
			owner,
			sequence,
			[10u8; 32],
			[11u8; 64],
			[20u8; 32],
			[21u8; 64],
		));

		// 记录存在
		let record = BotConsensus::equivocation_records(owner, sequence).unwrap();
		assert_eq!(record.reporter, 1);
		assert_eq!(record.msg_hash_a, [10u8; 32]);
		assert_eq!(record.msg_hash_b, [20u8; 32]);
		assert!(!record.resolved);
	});
}

#[test]
fn report_equivocation_fails_same_hash() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			BotConsensus::report_equivocation(
				RuntimeOrigin::signed(1),
				2,
				42,
				[10u8; 32],
				[11u8; 64],
				[10u8; 32],  // 相同的 hash
				[21u8; 64],
			),
			Error::<Test>::InvalidEquivocationEvidence
		);
	});
}

#[test]
fn report_equivocation_fails_duplicate() {
	new_test_ext().execute_with(|| {
		assert_ok!(BotConsensus::report_equivocation(
			RuntimeOrigin::signed(1),
			2, 42,
			[10u8; 32], [11u8; 64],
			[20u8; 32], [21u8; 64],
		));

		assert_noop!(
			BotConsensus::report_equivocation(
				RuntimeOrigin::signed(3),
				2, 42,
				[30u8; 32], [31u8; 64],
				[40u8; 32], [41u8; 64],
			),
			Error::<Test>::EquivocationAlreadyReported
		);
	});
}

#[test]
fn report_node_offline_reduces_reputation() {
	new_test_ext().execute_with(|| {
		let nid = make_node_id(1);
		assert_ok!(BotConsensus::register_node(
			RuntimeOrigin::signed(1),
			nid.clone(),
			[1u8; 32],
			[2u8; 32],
		));

		// 初始信誉 5000
		assert_eq!(BotConsensus::nodes(&nid).unwrap().reputation, 5000);

		// 举报离线，3 条证据 → -30
		assert_ok!(BotConsensus::report_node_offline(
			RuntimeOrigin::signed(2),
			nid.clone(),
			vec![[10u8; 32], [11u8; 32], [12u8; 32]],
		));

		let node = BotConsensus::nodes(&nid).unwrap();
		assert_eq!(node.reputation, 4970);
		assert_eq!(node.messages_missed, 3);
	});
}

#[test]
fn low_reputation_triggers_suspend() {
	new_test_ext().execute_with(|| {
		let nid = make_node_id(1);
		assert_ok!(BotConsensus::register_node(
			RuntimeOrigin::signed(1),
			nid.clone(),
			[1u8; 32],
			[2u8; 32],
		));

		// 先激活节点
		assert_ok!(BotConsensus::activate_node(
			RuntimeOrigin::signed(1),
			nid.clone(),
		));
		assert_eq!(BotConsensus::nodes(&nid).unwrap().status, NodeStatus::Active);

		// 大量离线举报使信誉降至 SuspendThreshold (2000) 以下
		// 需要 301 条证据 (5000 - 301*10 = 1990 < 2000)
		let evidence: Vec<[u8; 32]> = (0..301u16).map(|i| {
			let mut e = [0u8; 32];
			e[0] = (i & 0xff) as u8;
			e[1] = (i >> 8) as u8;
			e
		}).collect();

		assert_ok!(BotConsensus::report_node_offline(
			RuntimeOrigin::signed(2),
			nid.clone(),
			evidence,
		));

		let node = BotConsensus::nodes(&nid).unwrap();
		assert_eq!(node.status, NodeStatus::Suspended);

		// 从活跃列表移除
		let active = BotConsensus::active_node_list();
		assert!(!active.contains(&nid));
	});
}

#[test]
fn leader_timeout_tracking_works() {
	new_test_ext().execute_with(|| {
		let nid = make_node_id(1);
		assert_ok!(BotConsensus::register_node(
			RuntimeOrigin::signed(1),
			nid.clone(),
			[1u8; 32],
			[2u8; 32],
		));

		// 报告 3 次超时
		for _ in 0..3 {
			assert_ok!(BotConsensus::report_leader_timeout(
				RuntimeOrigin::signed(2),
				nid.clone(),
			));
		}

		let stats = BotConsensus::leader_stats(&nid);
		assert_eq!(stats.total_leads, 3);
		assert_eq!(stats.timeout, 3);
		assert_eq!(stats.consecutive_timeouts, 3);

		// 第 3 次时触发信誉扣减 -100
		let node = BotConsensus::nodes(&nid).unwrap();
		assert_eq!(node.reputation, 4900); // 5000 - 100
	});
}

#[test]
fn leader_success_resets_consecutive_timeouts() {
	new_test_ext().execute_with(|| {
		let nid = make_node_id(1);
		assert_ok!(BotConsensus::register_node(
			RuntimeOrigin::signed(1),
			nid.clone(),
			[1u8; 32],
			[2u8; 32],
		));

		// 2 次超时
		assert_ok!(BotConsensus::report_leader_timeout(RuntimeOrigin::signed(2), nid.clone()));
		assert_ok!(BotConsensus::report_leader_timeout(RuntimeOrigin::signed(2), nid.clone()));
		assert_eq!(BotConsensus::leader_stats(&nid).consecutive_timeouts, 2);

		// 1 次成功 → 重置
		assert_ok!(BotConsensus::report_leader_success(RuntimeOrigin::signed(2), nid.clone()));

		let stats = BotConsensus::leader_stats(&nid);
		assert_eq!(stats.consecutive_timeouts, 0);
		assert_eq!(stats.successful, 1);
		assert_eq!(stats.total_leads, 3);
	});
}

#[test]
fn activate_node_works() {
	new_test_ext().execute_with(|| {
		let nid = make_node_id(1);
		assert_ok!(BotConsensus::register_node(
			RuntimeOrigin::signed(1),
			nid.clone(),
			[1u8; 32],
			[2u8; 32],
		));

		// 初始状态是 Probation
		assert_eq!(BotConsensus::nodes(&nid).unwrap().status, NodeStatus::Probation);

		assert_ok!(BotConsensus::activate_node(
			RuntimeOrigin::signed(1),
			nid.clone(),
		));

		assert_eq!(BotConsensus::nodes(&nid).unwrap().status, NodeStatus::Active);
	});
}

#[test]
fn multiple_nodes_register() {
	new_test_ext().execute_with(|| {
		// 3 个不同运营者注册节点
		for i in 1u8..=3 {
			let nid = make_node_id(i);
			assert_ok!(BotConsensus::register_node(
				RuntimeOrigin::signed(i as u64),
				nid,
				[i; 32],
				[i + 10; 32],
			));
		}

		assert_eq!(BotConsensus::active_node_count(), 3);

		let active = BotConsensus::active_node_list();
		assert_eq!(active.len(), 3);
	});
}
