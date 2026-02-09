use crate::{mock::*, Error, NodeStatus, NodeId, SubscriptionTier, SubscriptionStatus};
use frame_support::{assert_ok, assert_noop, traits::Hooks};

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

// ═══════════════════════════════════════════════════════════════
// 订阅管理测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn subscribe_works() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		register_mock_bot(bh, 1);

		assert_ok!(BotConsensus::subscribe(
			RuntimeOrigin::signed(1),
			bh,
			SubscriptionTier::Basic,
			100, // 10 Era 费用
		));

		let sub = BotConsensus::subscriptions(&bh).unwrap();
		assert_eq!(sub.owner, 1);
		assert_eq!(sub.tier, SubscriptionTier::Basic);
		assert_eq!(sub.fee_per_era, 10);
		assert_eq!(sub.status, SubscriptionStatus::Active);
		assert_eq!(BotConsensus::subscription_escrow(&bh), 100);
		assert_eq!(Balances::reserved_balance(1), 100);
	});
}

#[test]
fn subscribe_fails_bot_not_registered() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(99);
		assert_noop!(
			BotConsensus::subscribe(RuntimeOrigin::signed(1), bh, SubscriptionTier::Basic, 100),
			Error::<Test>::BotNotRegistered
		);
	});
}

#[test]
fn subscribe_fails_not_owner() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		register_mock_bot(bh, 1);

		assert_noop!(
			BotConsensus::subscribe(RuntimeOrigin::signed(2), bh, SubscriptionTier::Basic, 100),
			Error::<Test>::NotBotOwner
		);
	});
}

#[test]
fn subscribe_fails_duplicate() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		register_mock_bot(bh, 1);

		assert_ok!(BotConsensus::subscribe(
			RuntimeOrigin::signed(1), bh, SubscriptionTier::Basic, 100,
		));
		assert_noop!(
			BotConsensus::subscribe(RuntimeOrigin::signed(1), bh, SubscriptionTier::Pro, 100),
			Error::<Test>::SubscriptionAlreadyExists
		);
	});
}

#[test]
fn subscribe_fails_insufficient_deposit() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		register_mock_bot(bh, 1);

		// Basic = 10/Era, deposit 5 < 10
		assert_noop!(
			BotConsensus::subscribe(RuntimeOrigin::signed(1), bh, SubscriptionTier::Basic, 5),
			Error::<Test>::InsufficientDeposit
		);
	});
}

#[test]
fn subscribe_pro_and_enterprise_fee() {
	new_test_ext().execute_with(|| {
		let bh1 = bot_hash(1);
		let bh2 = bot_hash(2);
		register_mock_bot(bh1, 1);
		register_mock_bot(bh2, 2);

		assert_ok!(BotConsensus::subscribe(
			RuntimeOrigin::signed(1), bh1, SubscriptionTier::Pro, 300,
		));
		assert_eq!(BotConsensus::subscriptions(&bh1).unwrap().fee_per_era, 30);

		assert_ok!(BotConsensus::subscribe(
			RuntimeOrigin::signed(2), bh2, SubscriptionTier::Enterprise, 1000,
		));
		assert_eq!(BotConsensus::subscriptions(&bh2).unwrap().fee_per_era, 100);
	});
}

#[test]
fn deposit_subscription_works() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		register_mock_bot(bh, 1);

		assert_ok!(BotConsensus::subscribe(
			RuntimeOrigin::signed(1), bh, SubscriptionTier::Basic, 50,
		));
		assert_eq!(BotConsensus::subscription_escrow(&bh), 50);

		assert_ok!(BotConsensus::deposit_subscription(
			RuntimeOrigin::signed(1), bh, 30,
		));
		assert_eq!(BotConsensus::subscription_escrow(&bh), 80);
		assert_eq!(Balances::reserved_balance(1), 80);
	});
}

#[test]
fn deposit_subscription_fails_cancelled() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		register_mock_bot(bh, 1);

		assert_ok!(BotConsensus::subscribe(
			RuntimeOrigin::signed(1), bh, SubscriptionTier::Basic, 50,
		));
		assert_ok!(BotConsensus::cancel_subscription(RuntimeOrigin::signed(1), bh));

		assert_noop!(
			BotConsensus::deposit_subscription(RuntimeOrigin::signed(1), bh, 30),
			Error::<Test>::SubscriptionAlreadyCancelled
		);
	});
}

#[test]
fn cancel_subscription_works() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		register_mock_bot(bh, 1);

		let free_before = Balances::free_balance(1);
		assert_ok!(BotConsensus::subscribe(
			RuntimeOrigin::signed(1), bh, SubscriptionTier::Basic, 100,
		));
		assert_eq!(Balances::free_balance(1), free_before - 100);

		assert_ok!(BotConsensus::cancel_subscription(RuntimeOrigin::signed(1), bh));

		let sub = BotConsensus::subscriptions(&bh).unwrap();
		assert_eq!(sub.status, SubscriptionStatus::Cancelled);
		assert_eq!(BotConsensus::subscription_escrow(&bh), 0);
		// 退还到 free
		assert_eq!(Balances::free_balance(1), free_before);
	});
}

#[test]
fn cancel_subscription_fails_double_cancel() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		register_mock_bot(bh, 1);
		assert_ok!(BotConsensus::subscribe(
			RuntimeOrigin::signed(1), bh, SubscriptionTier::Basic, 50,
		));
		assert_ok!(BotConsensus::cancel_subscription(RuntimeOrigin::signed(1), bh));
		assert_noop!(
			BotConsensus::cancel_subscription(RuntimeOrigin::signed(1), bh),
			Error::<Test>::SubscriptionAlreadyCancelled
		);
	});
}

#[test]
fn change_tier_works() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		register_mock_bot(bh, 1);

		assert_ok!(BotConsensus::subscribe(
			RuntimeOrigin::signed(1), bh, SubscriptionTier::Basic, 100,
		));

		assert_ok!(BotConsensus::change_tier(
			RuntimeOrigin::signed(1), bh, SubscriptionTier::Pro,
		));

		let sub = BotConsensus::subscriptions(&bh).unwrap();
		assert_eq!(sub.tier, SubscriptionTier::Pro);
		assert_eq!(sub.fee_per_era, 30);
	});
}

#[test]
fn change_tier_fails_same() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		register_mock_bot(bh, 1);
		assert_ok!(BotConsensus::subscribe(
			RuntimeOrigin::signed(1), bh, SubscriptionTier::Basic, 100,
		));

		assert_noop!(
			BotConsensus::change_tier(RuntimeOrigin::signed(1), bh, SubscriptionTier::Basic),
			Error::<Test>::SameTier
		);
	});
}

#[test]
fn claim_rewards_works() {
	new_test_ext().execute_with(|| {
		let nid = make_node_id(1);
		register_and_activate(1, &nid);

		// 手动写入待领取奖励
		crate::pallet::NodePendingRewards::<Test>::insert(&nid, 500u128);

		let free_before = Balances::free_balance(1);
		assert_ok!(BotConsensus::claim_rewards(RuntimeOrigin::signed(1), nid.clone()));

		// 奖励已到账
		assert_eq!(Balances::free_balance(1), free_before + 500);
		assert_eq!(BotConsensus::node_pending_rewards(&nid), 0);
	});
}

#[test]
fn claim_rewards_fails_no_pending() {
	new_test_ext().execute_with(|| {
		let nid = make_node_id(1);
		register_and_activate(1, &nid);

		assert_noop!(
			BotConsensus::claim_rewards(RuntimeOrigin::signed(1), nid),
			Error::<Test>::NoPendingRewards
		);
	});
}

#[test]
fn claim_rewards_fails_not_operator() {
	new_test_ext().execute_with(|| {
		let nid = make_node_id(1);
		register_and_activate(1, &nid);
		crate::pallet::NodePendingRewards::<Test>::insert(&nid, 500u128);

		assert_noop!(
			BotConsensus::claim_rewards(RuntimeOrigin::signed(2), nid),
			Error::<Test>::NotNodeOperator
		);
	});
}

// ═══════════════════════════════════════════════════════════════
// compute_node_weight 测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn compute_node_weight_new_node() {
	new_test_ext().execute_with(|| {
		let nid = make_node_id(1);
		register_and_activate(1, &nid);

		let node = BotConsensus::nodes(&nid).unwrap();
		let stats = BotConsensus::leader_stats(&nid);

		let w = BotConsensus::compute_node_weight(&node, &stats);
		// rep=5000, uptime=5000(default), leader_bonus=10000
		// 5000*5000*10000/10^8 = 2500
		assert_eq!(w, 2500);
	});
}

#[test]
fn compute_node_weight_high_performance() {
	new_test_ext().execute_with(|| {
		let nid = make_node_id(1);
		register_and_activate(1, &nid);

		// 模拟高性能节点: rep=10000, 100% uptime, 100% leader success
		crate::pallet::Nodes::<Test>::mutate(&nid, |maybe_node| {
			if let Some(n) = maybe_node {
				n.reputation = 10000;
				n.messages_confirmed = 100;
				n.messages_missed = 0;
			}
		});
		crate::pallet::LeaderStatsStore::<Test>::insert(&nid, crate::LeaderStats {
			total_leads: 10,
			successful: 10,
			timeout: 0,
			failed: 0,
			consecutive_timeouts: 0,
		});

		let node = BotConsensus::nodes(&nid).unwrap();
		let stats = BotConsensus::leader_stats(&nid);
		let w = BotConsensus::compute_node_weight(&node, &stats);
		// rep=10000, uptime=10000, leader=15000
		// 10000*10000*15000/10^8 = 15000
		assert_eq!(w, 15000);
	});
}

// ═══════════════════════════════════════════════════════════════
// on_era_end 测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn era_end_inflation_only() {
	new_test_ext().execute_with(|| {
		// 注册并激活一个节点
		let nid = make_node_id(1);
		register_and_activate(1, &nid);

		// 触发 Era 0 结束
		System::set_block_number(10);
		BotConsensus::on_initialize(10);

		// Era 应该推进
		assert_eq!(BotConsensus::current_era(), 1);

		// 奖励信息应该记录
		let info = BotConsensus::era_rewards(0).unwrap();
		assert_eq!(info.subscription_income, 0);
		assert_eq!(info.inflation_mint, 100);
		assert_eq!(info.node_count, 1);

		// 节点应有待领取奖励 (100 inflation → 节点池)
		// MaxRewardShare = 30%, 所以单节点最多拿 30
		let pending = BotConsensus::node_pending_rewards(&nid);
		assert!(pending > 0);
		assert_eq!(pending, 30); // capped at 30% of 100
	});
}

#[test]
fn era_end_subscription_deduction() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		register_mock_bot(bh, 1);

		let nid = make_node_id(2);
		register_and_activate(2, &nid);

		// 创建订阅
		assert_ok!(BotConsensus::subscribe(
			RuntimeOrigin::signed(1), bh, SubscriptionTier::Basic, 50,
		));

		// 触发 Era
		System::set_block_number(10);
		BotConsensus::on_initialize(10);

		// 扣费 10
		assert_eq!(BotConsensus::subscription_escrow(&bh), 40);

		let info = BotConsensus::era_rewards(0).unwrap();
		assert_eq!(info.subscription_income, 10);
		// 节点池 = 10*80/100 + 100(inflation) = 8 + 100 = 108
		assert!(info.total_distributed > 0);
	});
}

#[test]
fn era_end_past_due_then_suspend() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		register_mock_bot(bh, 1);

		let nid = make_node_id(2);
		register_and_activate(2, &nid);

		// 订阅刚好够 1 Era (Basic = 10/Era)
		assert_ok!(BotConsensus::subscribe(
			RuntimeOrigin::signed(1), bh, SubscriptionTier::Basic, 10,
		));

		// Era 0: 扣费成功, escrow = 0
		System::set_block_number(10);
		BotConsensus::on_initialize(10);
		assert_eq!(BotConsensus::subscription_escrow(&bh), 0);
		assert_eq!(BotConsensus::subscriptions(&bh).unwrap().status, SubscriptionStatus::Active);

		// Era 1: 余额不足 → PastDue
		System::set_block_number(20);
		BotConsensus::on_initialize(20);
		assert_eq!(BotConsensus::subscriptions(&bh).unwrap().status, SubscriptionStatus::PastDue);

		// Era 2: 仍然不足 → Suspended
		System::set_block_number(30);
		BotConsensus::on_initialize(30);
		assert_eq!(BotConsensus::subscriptions(&bh).unwrap().status, SubscriptionStatus::Suspended);
	});
}

#[test]
fn era_end_cancelled_sub_skipped() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		register_mock_bot(bh, 1);
		let nid = make_node_id(2);
		register_and_activate(2, &nid);

		assert_ok!(BotConsensus::subscribe(
			RuntimeOrigin::signed(1), bh, SubscriptionTier::Basic, 100,
		));
		assert_ok!(BotConsensus::cancel_subscription(RuntimeOrigin::signed(1), bh));

		System::set_block_number(10);
		BotConsensus::on_initialize(10);

		// 不应有订阅收入
		let info = BotConsensus::era_rewards(0).unwrap();
		assert_eq!(info.subscription_income, 0);
	});
}

#[test]
fn era_end_multi_node_weighted_distribution() {
	new_test_ext().execute_with(|| {
		// 注册 2 个节点
		let nid1 = make_node_id(1);
		let nid2 = make_node_id(2);
		register_and_activate(1, &nid1);
		register_and_activate(2, &nid2);

		// 节点1: 高信誉 (10000)
		crate::pallet::Nodes::<Test>::mutate(&nid1, |n| {
			if let Some(n) = n { n.reputation = 10000; }
		});
		// 节点2: 默认信誉 (5000)

		System::set_block_number(10);
		BotConsensus::on_initialize(10);

		let r1 = BotConsensus::node_pending_rewards(&nid1);
		let r2 = BotConsensus::node_pending_rewards(&nid2);

		// 两个节点都 capped at 30% = 30 each (权重不同但都被 cap)
		// 或者低权重节点未被 cap
		assert!(r1 > 0 && r2 > 0, "both should get rewards: r1={}, r2={}", r1, r2);
		// 总和不超过总池
		assert!(r1 + r2 <= 100);
	});
}

#[test]
fn era_end_max_reward_share_cap() {
	new_test_ext().execute_with(|| {
		// 注册 2 个节点，一个极高权重，一个极低
		let nid1 = make_node_id(1);
		let nid2 = make_node_id(2);
		register_and_activate(1, &nid1);
		register_and_activate(2, &nid2);

		// 节点1: max perf
		crate::pallet::Nodes::<Test>::mutate(&nid1, |n| {
			if let Some(n) = n {
				n.reputation = 10000;
				n.messages_confirmed = 100;
			}
		});
		crate::pallet::LeaderStatsStore::<Test>::insert(&nid1, crate::LeaderStats {
			total_leads: 10, successful: 10, timeout: 0, failed: 0, consecutive_timeouts: 0,
		});

		// 节点2: 最低信誉仍 Active
		crate::pallet::Nodes::<Test>::mutate(&nid2, |n| {
			if let Some(n) = n { n.reputation = 2001; }
		});

		System::set_block_number(10);
		BotConsensus::on_initialize(10);

		let r1 = BotConsensus::node_pending_rewards(&nid1);
		// MaxRewardShare = 30%, 总池 = 100 inflation → max = 30
		assert!(r1 <= 30, "node1 reward {} should be capped at 30", r1);
	});
}

#[test]
fn deposit_reactivates_past_due_subscription() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		register_mock_bot(bh, 1);
		let nid = make_node_id(2);
		register_and_activate(2, &nid);

		// 刚好 1 Era
		assert_ok!(BotConsensus::subscribe(
			RuntimeOrigin::signed(1), bh, SubscriptionTier::Basic, 10,
		));

		// Era 0 → 扣完, Era 1 → PastDue
		System::set_block_number(10);
		BotConsensus::on_initialize(10);
		System::set_block_number(20);
		BotConsensus::on_initialize(20);
		assert_eq!(BotConsensus::subscriptions(&bh).unwrap().status, SubscriptionStatus::PastDue);

		// 充值后恢复
		assert_ok!(BotConsensus::deposit_subscription(RuntimeOrigin::signed(1), bh, 100));
		assert_eq!(BotConsensus::subscriptions(&bh).unwrap().status, SubscriptionStatus::Active);
	});
}
