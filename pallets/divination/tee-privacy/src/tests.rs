//! # TEE 隐私计算模块 - 单元测试

use crate::{mock::*, types::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

// ============================================================================
// TEE 节点注册测试
// ============================================================================

#[test]
fn register_tee_node_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let enclave_pubkey = [42u8; 32];
        let attestation = create_test_attestation();

        // 注册 TEE 节点
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(account),
            enclave_pubkey,
            attestation.clone(),
        ));

        // 验证存储
        let node = TeePrivacy::tee_nodes(account).expect("Node should exist");
        assert_eq!(node.enclave_pubkey, enclave_pubkey);
        assert_eq!(node.status, TeeNodeStatus::Active);
        assert_eq!(node.attestation.mr_enclave, attestation.mr_enclave);

        // 验证活跃节点列表
        let active_nodes = TeePrivacy::active_nodes();
        assert!(active_nodes.contains(&account));

        // 验证节点计数
        assert_eq!(TeePrivacy::node_count(), 1);

        // 验证事件
        System::assert_last_event(
            Event::TeeNodeRegistered {
                account,
                enclave_pubkey,
                tee_type: TeeType::IntelSgx,
            }
            .into(),
        );
    });
}

#[test]
fn register_tee_node_fails_if_already_registered() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let enclave_pubkey = [42u8; 32];
        let attestation = create_test_attestation();

        // 首次注册成功
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(account),
            enclave_pubkey,
            attestation.clone(),
        ));

        // 再次注册应该失败
        assert_noop!(
            TeePrivacy::register_tee_node(
                RuntimeOrigin::signed(account),
                enclave_pubkey,
                attestation,
            ),
            Error::<Test>::NodeAlreadyRegistered
        );
    });
}

#[test]
fn register_tee_node_fails_with_invalid_pubkey() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let invalid_pubkey = [0u8; 32]; // 全零无效
        let attestation = create_test_attestation();

        assert_noop!(
            TeePrivacy::register_tee_node(
                RuntimeOrigin::signed(account),
                invalid_pubkey,
                attestation,
            ),
            Error::<Test>::InvalidEnclavePubkey
        );
    });
}

#[test]
fn register_tee_node_fails_with_invalid_mr_enclave() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let enclave_pubkey = [42u8; 32];
        let attestation = create_invalid_attestation(); // MRENCLAVE 不在允许列表

        assert_noop!(
            TeePrivacy::register_tee_node(
                RuntimeOrigin::signed(account),
                enclave_pubkey,
                attestation,
            ),
            Error::<Test>::MrEnclaveNotAllowed
        );
    });
}

// ============================================================================
// 认证更新测试
// ============================================================================

#[test]
fn update_attestation_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let enclave_pubkey = [42u8; 32];
        let attestation = create_test_attestation();

        // 先注册节点
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(account),
            enclave_pubkey,
            attestation.clone(),
        ));

        // 创建新的认证报告 - 使用相同时间戳（模拟同时刻更新）
        let mut new_attestation = attestation;
        new_attestation.mr_enclave = [2u8; 32]; // 使用另一个允许的 MRENCLAVE

        // 更新认证
        assert_ok!(TeePrivacy::update_attestation(
            RuntimeOrigin::signed(account),
            new_attestation.clone(),
        ));

        // 验证更新
        let node = TeePrivacy::tee_nodes(account).expect("Node should exist");
        assert_eq!(node.attestation.mr_enclave, [2u8; 32]);

        // 验证事件
        System::assert_last_event(
            Event::AttestationUpdated {
                account,
                mr_enclave: [2u8; 32],
                timestamp: new_attestation.timestamp,
            }
            .into(),
        );
    });
}

#[test]
fn update_attestation_fails_if_not_registered() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let attestation = create_test_attestation();

        assert_noop!(
            TeePrivacy::update_attestation(RuntimeOrigin::signed(account), attestation,),
            Error::<Test>::NodeNotRegistered
        );
    });
}

// ============================================================================
// 节点状态更新测试
// ============================================================================

#[test]
fn update_node_status_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let enclave_pubkey = [42u8; 32];
        let attestation = create_test_attestation();

        // 先注册节点
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(account),
            enclave_pubkey,
            attestation,
        ));

        // 暂停节点
        assert_ok!(TeePrivacy::update_node_status(
            RuntimeOrigin::signed(account),
            TeeNodeStatus::Suspended,
        ));

        // 验证状态
        let node = TeePrivacy::tee_nodes(account).expect("Node should exist");
        assert_eq!(node.status, TeeNodeStatus::Suspended);

        // 验证从活跃列表移除
        let active_nodes = TeePrivacy::active_nodes();
        assert!(!active_nodes.contains(&account));

        // 恢复活跃
        assert_ok!(TeePrivacy::update_node_status(
            RuntimeOrigin::signed(account),
            TeeNodeStatus::Active,
        ));

        // 验证重新加入活跃列表
        let active_nodes = TeePrivacy::active_nodes();
        assert!(active_nodes.contains(&account));
    });
}

#[test]
fn update_node_status_fails_with_invalid_status() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let enclave_pubkey = [42u8; 32];
        let attestation = create_test_attestation();

        // 先注册节点
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(account),
            enclave_pubkey,
            attestation,
        ));

        // 尝试设置为 Pending 状态（不允许）
        assert_noop!(
            TeePrivacy::update_node_status(RuntimeOrigin::signed(account), TeeNodeStatus::Pending,),
            Error::<Test>::InvalidNodeStatus
        );

        // 尝试设置为 Deregistered 状态（不允许）
        assert_noop!(
            TeePrivacy::update_node_status(
                RuntimeOrigin::signed(account),
                TeeNodeStatus::Deregistered,
            ),
            Error::<Test>::InvalidNodeStatus
        );
    });
}

// ============================================================================
// 节点注销测试
// ============================================================================

#[test]
fn deregister_tee_node_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let enclave_pubkey = [42u8; 32];
        let attestation = create_test_attestation();

        // 先注册节点
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(account),
            enclave_pubkey,
            attestation,
        ));

        assert_eq!(TeePrivacy::node_count(), 1);

        // 注销节点
        assert_ok!(TeePrivacy::deregister_tee_node(RuntimeOrigin::signed(
            account
        ),));

        // 验证状态变更
        let node = TeePrivacy::tee_nodes(account).expect("Node should still exist");
        assert_eq!(node.status, TeeNodeStatus::Deregistered);

        // 验证从活跃列表移除
        let active_nodes = TeePrivacy::active_nodes();
        assert!(!active_nodes.contains(&account));

        // 验证节点计数减少
        assert_eq!(TeePrivacy::node_count(), 0);

        // 验证事件
        System::assert_last_event(Event::TeeNodeDeregistered { account }.into());
    });
}

#[test]
fn deregister_tee_node_fails_if_not_registered() {
    new_test_ext().execute_with(|| {
        let account = 1u64;

        assert_noop!(
            TeePrivacy::deregister_tee_node(RuntimeOrigin::signed(account),),
            Error::<Test>::NodeNotRegistered
        );
    });
}

#[test]
fn cannot_reactivate_deregistered_node() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let enclave_pubkey = [42u8; 32];
        let attestation = create_test_attestation();

        // 注册并注销节点
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(account),
            enclave_pubkey,
            attestation,
        ));

        assert_ok!(TeePrivacy::deregister_tee_node(RuntimeOrigin::signed(
            account
        ),));

        // 尝试重新激活应该失败
        assert_noop!(
            TeePrivacy::update_node_status(RuntimeOrigin::signed(account), TeeNodeStatus::Active,),
            Error::<Test>::InvalidNodeStatus
        );
    });
}

// ============================================================================
// 辅助函数测试
// ============================================================================

#[test]
fn is_node_active_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let enclave_pubkey = [42u8; 32];
        let attestation = create_test_attestation();

        // 未注册节点
        assert!(!TeePrivacy::is_node_active(&account));

        // 注册后应该活跃
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(account),
            enclave_pubkey,
            attestation,
        ));
        assert!(TeePrivacy::is_node_active(&account));

        // 暂停后不活跃
        assert_ok!(TeePrivacy::update_node_status(
            RuntimeOrigin::signed(account),
            TeeNodeStatus::Suspended,
        ));
        assert!(!TeePrivacy::is_node_active(&account));
    });
}

#[test]
fn active_node_count_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(TeePrivacy::active_node_count(), 0);

        // 注册第一个节点
        let attestation1 = create_test_attestation();
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(1),
            [1u8; 32],
            attestation1,
        ));
        assert_eq!(TeePrivacy::active_node_count(), 1);

        // 注册第二个节点
        let attestation2 = create_test_attestation();
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(2),
            [2u8; 32],
            attestation2,
        ));
        assert_eq!(TeePrivacy::active_node_count(), 2);

        // 暂停一个节点
        assert_ok!(TeePrivacy::update_node_status(
            RuntimeOrigin::signed(1),
            TeeNodeStatus::Suspended,
        ));
        assert_eq!(TeePrivacy::active_node_count(), 1);
    });
}

#[test]
fn select_random_node_works() {
    new_test_ext().execute_with(|| {
        // 没有节点时返回 None
        assert!(TeePrivacy::select_random_node().is_none());

        // 注册节点后应该能选择
        let attestation = create_test_attestation();
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(1),
            [1u8; 32],
            attestation,
        ));

        let selected = TeePrivacy::select_random_node();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap(), 1);
    });
}

#[test]
fn get_enclave_pubkey_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let enclave_pubkey = [42u8; 32];
        let attestation = create_test_attestation();

        // 未注册时返回 None
        assert!(TeePrivacy::get_enclave_pubkey(&account).is_none());

        // 注册后返回公钥
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(account),
            enclave_pubkey,
            attestation,
        ));

        let pubkey = TeePrivacy::get_enclave_pubkey(&account);
        assert_eq!(pubkey, Some(enclave_pubkey));
    });
}

// ============================================================================
// 多节点场景测试
// ============================================================================

#[test]
fn multiple_nodes_registration() {
    new_test_ext().execute_with(|| {
        let attestation = create_test_attestation();

        // 注册多个节点
        for i in 1..=5u64 {
            let pubkey = [i as u8; 32];
            assert_ok!(TeePrivacy::register_tee_node(
                RuntimeOrigin::signed(i),
                pubkey,
                attestation.clone(),
            ));
        }

        // 验证节点计数
        assert_eq!(TeePrivacy::node_count(), 5);
        assert_eq!(TeePrivacy::active_node_count(), 5);

        // 验证所有节点都在活跃列表中
        let active_nodes = TeePrivacy::active_nodes();
        for i in 1..=5u64 {
            assert!(active_nodes.contains(&i));
        }
    });
}

// ============================================================================
// Phase 2: 质押测试
// ============================================================================

#[test]
fn stake_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let enclave_pubkey = [42u8; 32];
        let attestation = create_test_attestation();

        // 先注册节点
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(account),
            enclave_pubkey,
            attestation,
        ));

        let initial_balance = Balances::free_balance(account);

        // 质押
        assert_ok!(TeePrivacy::stake(
            RuntimeOrigin::signed(account),
            MinimumStake::get(),
        ));

        // 验证余额变化
        assert_eq!(
            Balances::free_balance(account),
            initial_balance - MinimumStake::get()
        );
        assert_eq!(
            Balances::reserved_balance(account),
            MinimumStake::get()
        );

        // 验证质押信息
        let stake_info = TeePrivacy::node_stakes(account).expect("Stake should exist");
        assert_eq!(stake_info.amount, MinimumStake::get());
        assert!(!stake_info.is_unbonding);

        // 验证事件
        System::assert_last_event(
            Event::Staked {
                account,
                amount: MinimumStake::get(),
            }
            .into(),
        );
    });
}

#[test]
fn stake_fails_if_not_registered() {
    new_test_ext().execute_with(|| {
        let account = 1u64;

        assert_noop!(
            TeePrivacy::stake(RuntimeOrigin::signed(account), MinimumStake::get(),),
            Error::<Test>::NodeNotRegistered
        );
    });
}

#[test]
fn stake_fails_if_amount_too_low() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let enclave_pubkey = [42u8; 32];
        let attestation = create_test_attestation();

        // 先注册节点
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(account),
            enclave_pubkey,
            attestation,
        ));

        // 质押金额低于最低要求
        assert_noop!(
            TeePrivacy::stake(RuntimeOrigin::signed(account), MinimumStake::get() - 1,),
            Error::<Test>::InsufficientStake
        );
    });
}

#[test]
fn request_unstake_works() {
    new_test_ext().execute_with(|| {
        register_and_stake_node(1);

        // 申请解除质押
        assert_ok!(TeePrivacy::request_unstake(RuntimeOrigin::signed(1),));

        // 验证质押状态
        let stake_info = TeePrivacy::node_stakes(1).expect("Stake should exist");
        assert!(stake_info.is_unbonding);
        assert!(stake_info.unlock_at.is_some());
    });
}

#[test]
fn request_unstake_fails_if_not_staked() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let enclave_pubkey = [42u8; 32];
        let attestation = create_test_attestation();

        // 只注册不质押
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(account),
            enclave_pubkey,
            attestation,
        ));

        assert_noop!(
            TeePrivacy::request_unstake(RuntimeOrigin::signed(account),),
            Error::<Test>::NotStaked
        );
    });
}

#[test]
fn withdraw_unstaked_works() {
    new_test_ext().execute_with(|| {
        register_and_stake_node(1);

        let initial_free = Balances::free_balance(1);
        let stake_amount = TeePrivacy::node_stakes(1).unwrap().amount;

        // 申请解除质押
        assert_ok!(TeePrivacy::request_unstake(RuntimeOrigin::signed(1),));

        // 获取解锁区块
        let unlock_at = TeePrivacy::node_stakes(1).unwrap().unlock_at.unwrap();

        // 推进到解锁区块
        run_to_block(unlock_at);

        // 提取
        assert_ok!(TeePrivacy::withdraw_unstaked(RuntimeOrigin::signed(1),));

        // 验证余额恢复
        assert_eq!(Balances::free_balance(1), initial_free + stake_amount);
        assert_eq!(Balances::reserved_balance(1), 0);

        // 验证质押记录已删除
        assert!(TeePrivacy::node_stakes(1).is_none());
    });
}

#[test]
fn withdraw_unstaked_fails_before_unlock() {
    new_test_ext().execute_with(|| {
        register_and_stake_node(1);

        // 申请解除质押
        assert_ok!(TeePrivacy::request_unstake(RuntimeOrigin::signed(1),));

        // 不推进区块，直接尝试提取
        assert_noop!(
            TeePrivacy::withdraw_unstaked(RuntimeOrigin::signed(1),),
            Error::<Test>::UnlockTimeNotReached
        );
    });
}

// ============================================================================
// Phase 2: 计算请求测试
// ============================================================================

#[test]
fn submit_compute_request_works() {
    new_test_ext().execute_with(|| {
        // 注册并质押节点
        register_and_stake_node(1);

        // 用户提交请求
        let requester = 10u64;
        let input_hash = [42u8; 32];

        assert_ok!(TeePrivacy::submit_compute_request(
            RuntimeOrigin::signed(requester),
            0, // BaZi
            input_hash,
            None, // 自动分配节点
        ));

        // 验证请求存储
        let request = TeePrivacy::compute_requests(0).expect("Request should exist");
        assert_eq!(request.requester, requester);
        assert_eq!(request.input_hash, input_hash);
        assert_eq!(request.compute_type_id, 0);
        assert_eq!(request.status, RequestStatus::Processing);
        assert_eq!(request.assigned_node, Some(1));

        // 验证待处理列表
        let pending = TeePrivacy::pending_requests();
        assert!(pending.contains(&0));

        // 验证节点当前请求
        assert_eq!(TeePrivacy::node_current_request(1), Some(0));
    });
}

#[test]
fn submit_compute_request_fails_without_nodes() {
    new_test_ext().execute_with(|| {
        let requester = 10u64;
        let input_hash = [42u8; 32];

        assert_noop!(
            TeePrivacy::submit_compute_request(
                RuntimeOrigin::signed(requester),
                0,
                input_hash,
                None,
            ),
            Error::<Test>::NoAvailableNodes
        );
    });
}

#[test]
fn submit_compute_result_works() {
    new_test_ext().execute_with(|| {
        // 注册并质押节点
        register_and_stake_node(1);

        // 用户提交请求
        let requester = 10u64;
        let input_hash = [42u8; 32];

        assert_ok!(TeePrivacy::submit_compute_request(
            RuntimeOrigin::signed(requester),
            0,
            input_hash,
            Some(1),
        ));

        // 节点提交结果
        let output_hash = [99u8; 32];
        let enclave_signature = [0u8; 64];

        assert_ok!(TeePrivacy::submit_compute_result(
            RuntimeOrigin::signed(1),
            0,
            output_hash,
            enclave_signature,
        ));

        // 验证请求状态
        let request = TeePrivacy::compute_requests(0).expect("Request should exist");
        assert_eq!(request.status, RequestStatus::Completed);

        // 验证结果存储
        let result = TeePrivacy::compute_results(0).expect("Result should exist");
        assert_eq!(result.output_hash, output_hash);
        assert_eq!(result.executor, 1);

        // 验证待处理列表已清除
        let pending = TeePrivacy::pending_requests();
        assert!(!pending.contains(&0));

        // 验证节点当前请求已清除
        assert!(TeePrivacy::node_current_request(1).is_none());

        // 验证节点统计
        let stats = TeePrivacy::node_stats(1).expect("Stats should exist");
        assert_eq!(stats.completed_requests, 1);
    });
}

#[test]
fn submit_compute_result_fails_if_not_assigned() {
    new_test_ext().execute_with(|| {
        // 注册两个节点
        register_and_stake_node(1);
        register_and_stake_node(2);

        // 用户提交请求，指定节点1
        assert_ok!(TeePrivacy::submit_compute_request(
            RuntimeOrigin::signed(10),
            0,
            [42u8; 32],
            Some(1),
        ));

        // 节点2尝试提交结果
        assert_noop!(
            TeePrivacy::submit_compute_result(
                RuntimeOrigin::signed(2),
                0,
                [99u8; 32],
                [0u8; 64],
            ),
            Error::<Test>::NotAssignedNode
        );
    });
}

#[test]
fn cancel_compute_request_works() {
    new_test_ext().execute_with(|| {
        register_and_stake_node(1);

        // 用户提交请求
        let requester = 10u64;
        assert_ok!(TeePrivacy::submit_compute_request(
            RuntimeOrigin::signed(requester),
            0,
            [42u8; 32],
            Some(1),
        ));

        // 用户取消请求
        assert_ok!(TeePrivacy::cancel_compute_request(
            RuntimeOrigin::signed(requester),
            0,
        ));

        // 验证请求状态
        let request = TeePrivacy::compute_requests(0).expect("Request should exist");
        assert_eq!(request.status, RequestStatus::Failed);

        // 验证待处理列表已清除
        let pending = TeePrivacy::pending_requests();
        assert!(!pending.contains(&0));
    });
}

#[test]
fn cancel_compute_request_fails_if_not_requester() {
    new_test_ext().execute_with(|| {
        register_and_stake_node(1);

        // 用户10提交请求
        assert_ok!(TeePrivacy::submit_compute_request(
            RuntimeOrigin::signed(10),
            0,
            [42u8; 32],
            Some(1),
        ));

        // 用户2尝试取消
        assert_noop!(
            TeePrivacy::cancel_compute_request(RuntimeOrigin::signed(2), 0,),
            Error::<Test>::NotRequester
        );
    });
}

// ============================================================================
// Phase 2: 超时与故障转移测试
// ============================================================================

#[test]
fn request_timeout_triggers_failover() {
    new_test_ext().execute_with(|| {
        // 注册两个节点
        register_and_stake_node(1);
        register_and_stake_node(2);

        // 用户提交请求
        assert_ok!(TeePrivacy::submit_compute_request(
            RuntimeOrigin::signed(10),
            0,
            [42u8; 32],
            Some(1),
        ));

        // 验证请求分配给节点1
        let request = TeePrivacy::compute_requests(0).unwrap();
        assert_eq!(request.assigned_node, Some(1));
        let timeout_at = request.timeout_at;

        // 推进到超时区块+1 (需要过超时区块才会触发on_finalize处理)
        run_to_block(timeout_at + 1);

        // 验证请求已故障转移到节点2
        let request = TeePrivacy::compute_requests(0).unwrap();
        assert_eq!(request.assigned_node, Some(2));
        assert_eq!(request.failover_count, 1);

        // 验证节点1的统计
        let stats = TeePrivacy::node_stats(1).expect("Stats should exist");
        assert_eq!(stats.timeout_requests, 1);
    });
}

#[test]
fn request_fails_after_max_failovers() {
    new_test_ext().execute_with(|| {
        // 只注册一个节点
        register_and_stake_node(1);

        // 用户提交请求
        assert_ok!(TeePrivacy::submit_compute_request(
            RuntimeOrigin::signed(10),
            0,
            [42u8; 32],
            Some(1),
        ));

        // 获取超时区块
        let request = TeePrivacy::compute_requests(0).unwrap();
        let mut timeout_at = request.timeout_at;

        // 模拟多次超时（因为只有一个节点，会一直分配到同一个节点）
        for _i in 0..5 {
            run_to_block(timeout_at + 1);
            if let Some(req) = TeePrivacy::compute_requests(0) {
                if req.status == RequestStatus::Failed {
                    // 已失败，停止循环
                    break;
                }
                timeout_at = req.timeout_at;
            }
        }

        // 验证请求最终失败
        let request = TeePrivacy::compute_requests(0).unwrap();
        assert_eq!(request.status, RequestStatus::Failed);
        assert!(request.failure_reason.is_some());
    });
}

#[test]
fn slash_node_on_timeout() {
    new_test_ext().execute_with(|| {
        // 注册两个节点
        register_and_stake_node(1);
        register_and_stake_node(2);

        let initial_stake = TeePrivacy::node_stakes(1).unwrap().amount;

        // 用户提交请求
        assert_ok!(TeePrivacy::submit_compute_request(
            RuntimeOrigin::signed(10),
            0,
            [42u8; 32],
            Some(1),
        ));

        // 获取超时区块
        let request = TeePrivacy::compute_requests(0).unwrap();
        let timeout_at = request.timeout_at;

        // 推进到超时区块+1
        run_to_block(timeout_at + 1);

        // 验证节点1被惩罚
        let stake = TeePrivacy::node_stakes(1).unwrap();
        assert!(stake.amount < initial_stake);

        // 验证统计更新
        let stats = TeePrivacy::node_stats(1).unwrap();
        assert_eq!(stats.slash_count, 1);
        assert!(stats.total_slashed > 0);
    });
}

// ============================================================================
// Phase 5: 批处理测试
// ============================================================================

#[test]
fn submit_batch_compute_requests_works() {
    new_test_ext().execute_with(|| {
        // 注册并质押节点
        register_and_stake_node(1);

        // 用户提交批量请求
        let requester = 10u64;
        let requests = vec![
            BatchRequestItem {
                compute_type_id: 0, // BaZi
                input_hash: [1u8; 32],
            },
            BatchRequestItem {
                compute_type_id: 1, // MeiHua
                input_hash: [2u8; 32],
            },
            BatchRequestItem {
                compute_type_id: 2, // QiMen
                input_hash: [3u8; 32],
            },
        ];

        assert_ok!(TeePrivacy::submit_batch_compute_requests(
            RuntimeOrigin::signed(requester),
            requests,
        ));

        // 验证所有请求都已创建
        for i in 0..3u64 {
            let request = TeePrivacy::compute_requests(i).expect("Request should exist");
            assert_eq!(request.requester, requester);
            assert_eq!(request.compute_type_id, i as u8);
            assert_eq!(request.status, RequestStatus::Processing);
            assert_eq!(request.assigned_node, Some(1)); // 所有请求分配给同一节点
        }

        // 验证待处理列表
        let pending = TeePrivacy::pending_requests();
        assert!(pending.contains(&0));
        assert!(pending.contains(&1));
        assert!(pending.contains(&2));

        // 验证下一个请求 ID 更新
        assert_eq!(TeePrivacy::next_request_id(), 3);
    });
}

#[test]
fn submit_batch_compute_requests_fails_if_empty() {
    new_test_ext().execute_with(|| {
        register_and_stake_node(1);

        // 提交空批次
        assert_noop!(
            TeePrivacy::submit_batch_compute_requests(
                RuntimeOrigin::signed(10),
                vec![],
            ),
            Error::<Test>::EmptyBatch
        );
    });
}

#[test]
fn submit_batch_compute_requests_fails_if_exceeds_limit() {
    new_test_ext().execute_with(|| {
        register_and_stake_node(1);

        // 创建超过限制的批次
        let requests: Vec<BatchRequestItem> = (0..15)
            .map(|i| BatchRequestItem {
                compute_type_id: i as u8,
                input_hash: [i as u8; 32],
            })
            .collect();

        assert_noop!(
            TeePrivacy::submit_batch_compute_requests(
                RuntimeOrigin::signed(10),
                requests,
            ),
            Error::<Test>::BatchSizeExceeded
        );
    });
}

#[test]
fn submit_batch_compute_requests_fails_without_nodes() {
    new_test_ext().execute_with(|| {
        // 不注册任何节点
        let requests = vec![
            BatchRequestItem {
                compute_type_id: 0,
                input_hash: [1u8; 32],
            },
        ];

        assert_noop!(
            TeePrivacy::submit_batch_compute_requests(
                RuntimeOrigin::signed(10),
                requests,
            ),
            Error::<Test>::NoAvailableNodes
        );
    });
}

#[test]
fn submit_batch_compute_results_works() {
    new_test_ext().execute_with(|| {
        // 注册并质押节点
        register_and_stake_node(1);

        // 用户提交批量请求
        let requester = 10u64;
        let requests = vec![
            BatchRequestItem {
                compute_type_id: 0,
                input_hash: [1u8; 32],
            },
            BatchRequestItem {
                compute_type_id: 1,
                input_hash: [2u8; 32],
            },
        ];

        assert_ok!(TeePrivacy::submit_batch_compute_requests(
            RuntimeOrigin::signed(requester),
            requests,
        ));

        // 节点提交批量结果
        let results = vec![
            BatchResultItem {
                request_id: 0,
                output_hash: [10u8; 32],
                enclave_signature: [0u8; 64],
            },
            BatchResultItem {
                request_id: 1,
                output_hash: [20u8; 32],
                enclave_signature: [0u8; 64],
            },
        ];

        assert_ok!(TeePrivacy::submit_batch_compute_results(
            RuntimeOrigin::signed(1),
            results,
        ));

        // 验证所有请求都已完成
        for i in 0..2u64 {
            let request = TeePrivacy::compute_requests(i).expect("Request should exist");
            assert_eq!(request.status, RequestStatus::Completed);

            let result = TeePrivacy::compute_results(i).expect("Result should exist");
            assert_eq!(result.executor, 1);
        }

        // 验证待处理列表已清空
        let pending = TeePrivacy::pending_requests();
        assert!(!pending.contains(&0));
        assert!(!pending.contains(&1));

        // 验证节点统计
        let stats = TeePrivacy::node_stats(1).expect("Stats should exist");
        assert_eq!(stats.completed_requests, 2);
    });
}

#[test]
fn submit_batch_compute_results_fails_if_empty() {
    new_test_ext().execute_with(|| {
        register_and_stake_node(1);

        // 提交空批次
        assert_noop!(
            TeePrivacy::submit_batch_compute_results(
                RuntimeOrigin::signed(1),
                vec![],
            ),
            Error::<Test>::EmptyBatch
        );
    });
}

#[test]
fn submit_batch_compute_results_fails_if_not_assigned() {
    new_test_ext().execute_with(|| {
        // 注册两个节点
        register_and_stake_node(1);
        register_and_stake_node(2);

        // 使用单个请求 API 并明确指定节点1
        assert_ok!(TeePrivacy::submit_compute_request(
            RuntimeOrigin::signed(10),
            0,
            [1u8; 32],
            Some(1), // 明确指定节点1
        ));

        // 节点2尝试提交结果（未分配给它）
        let results = vec![
            BatchResultItem {
                request_id: 0,
                output_hash: [10u8; 32],
                enclave_signature: [0u8; 64],
            },
        ];

        // 批处理会跳过无效请求，但不会失败
        assert_ok!(TeePrivacy::submit_batch_compute_results(
            RuntimeOrigin::signed(2),
            results,
        ));

        // 验证请求状态仍为 Processing（未被节点2修改）
        let request = TeePrivacy::compute_requests(0).unwrap();
        assert_eq!(request.status, RequestStatus::Processing);
    });
}

#[test]
fn submit_batch_compute_results_partial_success() {
    new_test_ext().execute_with(|| {
        // 注册并质押节点
        register_and_stake_node(1);

        // 提交两个请求
        let requests = vec![
            BatchRequestItem {
                compute_type_id: 0,
                input_hash: [1u8; 32],
            },
            BatchRequestItem {
                compute_type_id: 1,
                input_hash: [2u8; 32],
            },
        ];

        assert_ok!(TeePrivacy::submit_batch_compute_requests(
            RuntimeOrigin::signed(10),
            requests,
        ));

        // 提交结果：一个有效，一个无效（请求ID不存在）
        let results = vec![
            BatchResultItem {
                request_id: 0,
                output_hash: [10u8; 32],
                enclave_signature: [0u8; 64],
            },
            BatchResultItem {
                request_id: 999, // 不存在的请求
                output_hash: [20u8; 32],
                enclave_signature: [0u8; 64],
            },
        ];

        // 批处理应该成功（跳过无效项）
        assert_ok!(TeePrivacy::submit_batch_compute_results(
            RuntimeOrigin::signed(1),
            results,
        ));

        // 验证第一个请求已完成
        let request0 = TeePrivacy::compute_requests(0).unwrap();
        assert_eq!(request0.status, RequestStatus::Completed);

        // 验证第二个请求仍在处理中
        let request1 = TeePrivacy::compute_requests(1).unwrap();
        assert_eq!(request1.status, RequestStatus::Processing);

        // 验证节点统计只计算一次成功
        let stats = TeePrivacy::node_stats(1).unwrap();
        assert_eq!(stats.completed_requests, 1);
    });
}

// ============================================================================
// Phase 5: Enclave 升级机制测试
// ============================================================================

#[test]
fn add_allowed_mr_enclave_works() {
    new_test_ext().execute_with(|| {
        let new_mr_enclave = [42u8; 32];

        // 验证初始不在列表中
        let initial_list = TeePrivacy::allowed_mr_enclaves();
        assert!(!initial_list.contains(&new_mr_enclave));

        // Root 添加新 MRENCLAVE
        assert_ok!(TeePrivacy::add_allowed_mr_enclave(
            RuntimeOrigin::root(),
            new_mr_enclave,
        ));

        // 验证已添加
        let updated_list = TeePrivacy::allowed_mr_enclaves();
        assert!(updated_list.contains(&new_mr_enclave));

        // 验证事件
        System::assert_last_event(Event::MrEnclaveAllowed { mr_enclave: new_mr_enclave }.into());
    });
}

#[test]
fn add_allowed_mr_enclave_fails_if_not_root() {
    new_test_ext().execute_with(|| {
        let new_mr_enclave = [42u8; 32];

        // 非 Root 用户无法添加
        assert_noop!(
            TeePrivacy::add_allowed_mr_enclave(
                RuntimeOrigin::signed(1),
                new_mr_enclave,
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn add_allowed_mr_enclave_fails_if_exists() {
    new_test_ext().execute_with(|| {
        // [1u8; 32] 已在初始列表中
        let existing_mr_enclave = [1u8; 32];

        assert_noop!(
            TeePrivacy::add_allowed_mr_enclave(
                RuntimeOrigin::root(),
                existing_mr_enclave,
            ),
            Error::<Test>::EntryAlreadyExists
        );
    });
}

#[test]
fn remove_allowed_mr_enclave_works() {
    new_test_ext().execute_with(|| {
        // [1u8; 32] 在初始列表中
        let existing_mr_enclave = [1u8; 32];

        // 验证初始在列表中
        let initial_list = TeePrivacy::allowed_mr_enclaves();
        assert!(initial_list.contains(&existing_mr_enclave));

        // Root 移除 MRENCLAVE
        assert_ok!(TeePrivacy::remove_allowed_mr_enclave(
            RuntimeOrigin::root(),
            existing_mr_enclave,
        ));

        // 验证已移除
        let updated_list = TeePrivacy::allowed_mr_enclaves();
        assert!(!updated_list.contains(&existing_mr_enclave));

        // 验证事件
        System::assert_last_event(Event::MrEnclaveDisallowed { mr_enclave: existing_mr_enclave }.into());
    });
}

#[test]
fn remove_allowed_mr_enclave_fails_if_not_exists() {
    new_test_ext().execute_with(|| {
        let non_existent = [99u8; 32];

        assert_noop!(
            TeePrivacy::remove_allowed_mr_enclave(
                RuntimeOrigin::root(),
                non_existent,
            ),
            Error::<Test>::EntryNotFound
        );
    });
}

#[test]
fn add_allowed_mr_signer_works() {
    new_test_ext().execute_with(|| {
        let new_mr_signer = [42u8; 32];

        // 验证初始不在列表中
        let initial_list = TeePrivacy::allowed_mr_signers();
        assert!(!initial_list.contains(&new_mr_signer));

        // Root 添加新 MRSIGNER
        assert_ok!(TeePrivacy::add_allowed_mr_signer(
            RuntimeOrigin::root(),
            new_mr_signer,
        ));

        // 验证已添加
        let updated_list = TeePrivacy::allowed_mr_signers();
        assert!(updated_list.contains(&new_mr_signer));

        // 验证事件
        System::assert_last_event(Event::MrSignerAllowed { mr_signer: new_mr_signer }.into());
    });
}

#[test]
fn remove_allowed_mr_signer_works() {
    new_test_ext().execute_with(|| {
        // [10u8; 32] 在初始列表中
        let existing_mr_signer = [10u8; 32];

        // Root 移除 MRSIGNER
        assert_ok!(TeePrivacy::remove_allowed_mr_signer(
            RuntimeOrigin::root(),
            existing_mr_signer,
        ));

        // 验证已移除
        let updated_list = TeePrivacy::allowed_mr_signers();
        assert!(!updated_list.contains(&existing_mr_signer));

        // 验证事件
        System::assert_last_event(Event::MrSignerDisallowed { mr_signer: existing_mr_signer }.into());
    });
}

#[test]
fn enclave_upgrade_scenario() {
    new_test_ext().execute_with(|| {
        // 场景：升级 Enclave 代码
        // 1. 添加新版本 MRENCLAVE
        // 2. 验证新 Enclave 可以注册
        // 3. 移除旧版本 MRENCLAVE
        // 4. 验证旧 Enclave 无法注册

        let old_mr_enclave = [1u8; 32]; // 已在初始列表
        let new_mr_enclave = [100u8; 32]; // 新版本

        // 1. 添加新版本
        assert_ok!(TeePrivacy::add_allowed_mr_enclave(
            RuntimeOrigin::root(),
            new_mr_enclave,
        ));

        // 2. 创建使用新版本的认证
        let mut new_attestation = create_test_attestation();
        new_attestation.mr_enclave = new_mr_enclave;

        // 注册使用新版本的节点
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(1),
            [1u8; 32],
            new_attestation,
        ));

        // 3. 移除旧版本
        assert_ok!(TeePrivacy::remove_allowed_mr_enclave(
            RuntimeOrigin::root(),
            old_mr_enclave,
        ));

        // 4. 使用旧版本的节点无法注册
        let old_attestation = create_test_attestation(); // mr_enclave = [1u8; 32]

        assert_noop!(
            TeePrivacy::register_tee_node(
                RuntimeOrigin::signed(2),
                [2u8; 32],
                old_attestation,
            ),
            Error::<Test>::MrEnclaveNotAllowed
        );
    });
}

// ============================================================================
// Phase 5: 审计日志测试
// ============================================================================

#[test]
fn set_audit_enabled_works() {
    new_test_ext().execute_with(|| {
        // 初始状态为禁用
        assert!(!TeePrivacy::audit_enabled());

        // Root 启用审计
        assert_ok!(TeePrivacy::set_audit_enabled(
            RuntimeOrigin::root(),
            true,
        ));

        // 验证已启用
        assert!(TeePrivacy::audit_enabled());

        // 验证事件
        System::assert_last_event(Event::AuditStatusChanged { enabled: true }.into());

        // 禁用审计
        assert_ok!(TeePrivacy::set_audit_enabled(
            RuntimeOrigin::root(),
            false,
        ));

        // 验证已禁用
        assert!(!TeePrivacy::audit_enabled());
    });
}

#[test]
fn set_audit_enabled_fails_if_not_root() {
    new_test_ext().execute_with(|| {
        // 非 Root 用户无法更改
        assert_noop!(
            TeePrivacy::set_audit_enabled(
                RuntimeOrigin::signed(1),
                true,
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn audit_log_records_when_enabled() {
    new_test_ext().execute_with(|| {
        // 启用审计
        assert_ok!(TeePrivacy::set_audit_enabled(
            RuntimeOrigin::root(),
            true,
        ));

        // 手动记录审计日志
        TeePrivacy::log_audit_event(
            AuditEventType::NodeRegistered,
            &1u64,
            [0u8; 32],
            true,
        );

        // 验证日志已记录
        let log = TeePrivacy::audit_logs(0).expect("Log should exist");
        assert_eq!(log.event_type, AuditEventType::NodeRegistered);
        assert_eq!(log.account, 1u64);
        assert!(log.success);

        // 验证账户索引
        let account_logs = TeePrivacy::account_audit_logs(1u64);
        assert!(account_logs.contains(&0));
    });
}

#[test]
fn audit_log_skipped_when_disabled() {
    new_test_ext().execute_with(|| {
        // 审计默认禁用
        assert!(!TeePrivacy::audit_enabled());

        // 尝试记录日志
        TeePrivacy::log_audit_event(
            AuditEventType::NodeRegistered,
            &1u64,
            [0u8; 32],
            true,
        );

        // 验证日志未记录
        assert!(TeePrivacy::audit_logs(0).is_none());
    });
}

#[test]
fn get_account_audit_logs_works() {
    new_test_ext().execute_with(|| {
        // 启用审计
        assert_ok!(TeePrivacy::set_audit_enabled(
            RuntimeOrigin::root(),
            true,
        ));

        let account = 1u64;

        // 记录多条日志
        TeePrivacy::log_audit_event(
            AuditEventType::NodeRegistered,
            &account,
            [1u8; 32],
            true,
        );
        TeePrivacy::log_audit_event(
            AuditEventType::AttestationUpdated,
            &account,
            [2u8; 32],
            true,
        );
        TeePrivacy::log_audit_event(
            AuditEventType::Staked,
            &account,
            [3u8; 32],
            true,
        );

        // 获取账户日志
        let logs = TeePrivacy::get_account_audit_logs(&account);
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0].event_type, AuditEventType::NodeRegistered);
        assert_eq!(logs[1].event_type, AuditEventType::AttestationUpdated);
        assert_eq!(logs[2].event_type, AuditEventType::Staked);
    });
}

#[test]
fn get_audit_logs_range_works() {
    new_test_ext().execute_with(|| {
        // 启用审计
        assert_ok!(TeePrivacy::set_audit_enabled(
            RuntimeOrigin::root(),
            true,
        ));

        // 记录多条日志
        for i in 0..5u64 {
            TeePrivacy::log_audit_event(
                AuditEventType::ComputeRequestSubmitted,
                &i,
                [i as u8; 32],
                true,
            );
        }

        // 获取范围日志
        let logs = TeePrivacy::get_audit_logs_range(1, 3);
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0].id, 1);
        assert_eq!(logs[1].id, 2);
        assert_eq!(logs[2].id, 3);
    });
}
