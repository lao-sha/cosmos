//! # TEE 隐私计算模块 - 集成测试
//!
//! 本模块提供端到端的集成测试，验证完整的 TEE 隐私计算流程。

#[cfg(test)]
mod integration_tests {
    use crate::mock::*;
    use crate::types::*;
    use crate::*;
    use frame_support::{assert_noop, assert_ok};

    // ============================================================================
    // 辅助函数
    // ============================================================================

    fn create_test_attestation() -> TeeAttestation {
        // 使用 mock 中已允许的 mr_signer
        let mr_signer = [10u8; 32]; // 在 mock 的允许列表中

        TeeAttestation {
            tee_type: TeeType::IntelSgx,
            mr_enclave: [1u8; 32], // 将由调用者修改
            mr_signer,
            isv_prod_id: 1,
            isv_svn: 1,
            report_data: [0u8; 64],
            ias_signature: BoundedVec::try_from(vec![0u8; 64]).unwrap(),
            timestamp: 1704067200, // 与 mock 时间戳匹配
        }
    }

    fn setup_tee_node(account: u64) -> [u8; 32] {
        let pubkey = [account as u8; 32];
        let mut attestation = create_test_attestation();
        // 使用不同的 mr_enclave 以避免冲突
        attestation.mr_enclave[0] = account as u8;

        // 添加允许的 MR_ENCLAVE（忽略已存在的错误）
        let _ = TeePrivacy::add_allowed_mr_enclave(
            RuntimeOrigin::root(),
            attestation.mr_enclave
        );

        // 注册节点
        assert_ok!(TeePrivacy::register_tee_node(
            RuntimeOrigin::signed(account),
            pubkey,
            attestation
        ));

        // 激活节点
        assert_ok!(TeePrivacy::update_node_status(
            RuntimeOrigin::signed(account),
            TeeNodeStatus::Active
        ));

        pubkey
    }

    fn setup_staked_node(account: u64, stake_amount: u128) -> [u8; 32] {
        let pubkey = setup_tee_node(account);

        // 质押
        assert_ok!(TeePrivacy::stake(
            RuntimeOrigin::signed(account),
            stake_amount
        ));

        pubkey
    }

    // ============================================================================
    // 集成测试：完整的 TEE 计算流程
    // ============================================================================

    #[test]
    fn integration_full_compute_flow() {
        new_test_ext().execute_with(|| {
            let node_account = 1u64;
            let user_account = 2u64;

            // 1. 设置 TEE 节点
            let _pubkey = setup_staked_node(node_account, 1000);

            // 2. 用户提交计算请求
            let input_hash = [0xAB; 32];
            let compute_type_id = 0u8; // BaZi

            assert_ok!(TeePrivacy::submit_compute_request(
                RuntimeOrigin::signed(user_account),
                compute_type_id,
                input_hash,
                None // 自动分配节点
            ));

            // 验证请求已创建
            let request = ComputeRequests::<Test>::get(0).expect("Request should exist");
            assert_eq!(request.requester, user_account);
            assert_eq!(request.compute_type_id, compute_type_id);
            assert_eq!(request.input_hash, input_hash);
            // 请求状态可能是 Pending 或 Processing，取决于是否有节点被分配
            assert!(request.status == RequestStatus::Pending || request.status == RequestStatus::Processing);

            // 3. TEE 节点提交计算结果
            let output_hash = [0xCD; 32];
            let enclave_signature = [0xEF; 64];

            assert_ok!(TeePrivacy::submit_compute_result(
                RuntimeOrigin::signed(node_account),
                0, // request_id
                output_hash,
                enclave_signature
            ));

            // 4. 验证结果已存储
            let result = ComputeResults::<Test>::get(0).expect("Result should exist");
            assert_eq!(result.executor, node_account);
            assert_eq!(result.output_hash, output_hash);
            assert_eq!(result.enclave_signature, enclave_signature);

            // 5. 验证请求状态已更新
            let updated_request = ComputeRequests::<Test>::get(0).expect("Request should exist");
            assert_eq!(updated_request.status, RequestStatus::Completed);

            // 6. 验证节点统计已更新
            let stats = NodeStats::<Test>::get(node_account).expect("Stats should exist");
            assert_eq!(stats.completed_requests, 1);
        });
    }

    #[test]
    fn integration_batch_compute_flow() {
        new_test_ext().execute_with(|| {
            let node_account = 1u64;
            let user_account = 2u64;

            // 1. 设置 TEE 节点
            setup_staked_node(node_account, 1000);

            // 2. 批量提交计算请求
            let batch_requests = vec![
                BatchRequestItem {
                    compute_type_id: 0,
                    input_hash: [0x01; 32],
                },
                BatchRequestItem {
                    compute_type_id: 1,
                    input_hash: [0x02; 32],
                },
                BatchRequestItem {
                    compute_type_id: 2,
                    input_hash: [0x03; 32],
                },
            ];

            assert_ok!(TeePrivacy::submit_batch_compute_requests(
                RuntimeOrigin::signed(user_account),
                batch_requests
            ));

            // 验证所有请求已创建
            assert!(ComputeRequests::<Test>::contains_key(0));
            assert!(ComputeRequests::<Test>::contains_key(1));
            assert!(ComputeRequests::<Test>::contains_key(2));

            // 3. 批量提交结果
            let batch_results = vec![
                BatchResultItem {
                    request_id: 0,
                    output_hash: [0xA1; 32],
                    enclave_signature: [0xB1; 64],
                },
                BatchResultItem {
                    request_id: 1,
                    output_hash: [0xA2; 32],
                    enclave_signature: [0xB2; 64],
                },
                BatchResultItem {
                    request_id: 2,
                    output_hash: [0xA3; 32],
                    enclave_signature: [0xB3; 64],
                },
            ];

            assert_ok!(TeePrivacy::submit_batch_compute_results(
                RuntimeOrigin::signed(node_account),
                batch_results
            ));

            // 验证所有结果已存储
            assert!(ComputeResults::<Test>::contains_key(0));
            assert!(ComputeResults::<Test>::contains_key(1));
            assert!(ComputeResults::<Test>::contains_key(2));

            // 验证节点统计
            let stats = NodeStats::<Test>::get(node_account).expect("Stats should exist");
            assert_eq!(stats.completed_requests, 3);
        });
    }

    #[test]
    fn integration_staking_lifecycle() {
        new_test_ext().execute_with(|| {
            let node = 1u64;
            let initial_balance = Balances::free_balance(node);

            // 1. 注册并激活节点
            setup_tee_node(node);

            // 2. 质押
            let stake_amount = 1000u128;
            assert_ok!(TeePrivacy::stake(RuntimeOrigin::signed(node), stake_amount));

            // 验证质押信息
            let stake_info = NodeStakes::<Test>::get(node).expect("Stake should exist");
            assert_eq!(stake_info.amount, stake_amount);
            assert!(!stake_info.is_unbonding);

            // 验证余额减少
            assert_eq!(
                Balances::free_balance(node),
                initial_balance - stake_amount
            );

            // 3. 申请解除质押
            assert_ok!(TeePrivacy::request_unstake(RuntimeOrigin::signed(node)));

            let stake_info = NodeStakes::<Test>::get(node).expect("Stake should exist");
            assert!(stake_info.is_unbonding);
            assert!(stake_info.unlock_at.is_some());

            // 4. 推进到解锁区块
            let unlock_at = stake_info.unlock_at.unwrap();
            System::set_block_number(unlock_at + 1);

            // 5. 提取质押
            assert_ok!(TeePrivacy::withdraw_unstaked(RuntimeOrigin::signed(node)));

            // 验证质押已清除
            assert!(!NodeStakes::<Test>::contains_key(node));

            // 验证余额恢复
            assert_eq!(Balances::free_balance(node), initial_balance);
        });
    }

    #[test]
    fn integration_multiple_nodes_registration() {
        new_test_ext().execute_with(|| {
            // 设置多个节点
            for i in 1..=5 {
                let mut attestation = create_test_attestation();
                attestation.mr_enclave[0] = i as u8;

                let _ = TeePrivacy::add_allowed_mr_enclave(
                    RuntimeOrigin::root(),
                    attestation.mr_enclave
                );

                assert_ok!(TeePrivacy::register_tee_node(
                    RuntimeOrigin::signed(i),
                    [i as u8; 32],
                    attestation
                ));

                assert_ok!(TeePrivacy::update_node_status(
                    RuntimeOrigin::signed(i),
                    TeeNodeStatus::Active
                ));

                assert_ok!(TeePrivacy::stake(RuntimeOrigin::signed(i), 1000));
            }

            // 验证所有节点都在活跃列表中
            assert_eq!(TeePrivacy::active_node_count(), 5);
        });
    }

    // ============================================================================
    // 安全测试
    // ============================================================================

    #[test]
    fn security_unauthorized_result_submission() {
        new_test_ext().execute_with(|| {
            let authorized_node = 1u64;
            let unauthorized_node = 2u64;
            let user = 3u64;

            // 设置授权节点
            setup_staked_node(authorized_node, 1000);

            // 设置未授权节点
            let mut attestation = create_test_attestation();
            attestation.mr_enclave[0] = 2;

            let _ = TeePrivacy::add_allowed_mr_enclave(
                RuntimeOrigin::root(),
                attestation.mr_enclave
            );

            assert_ok!(TeePrivacy::register_tee_node(
                RuntimeOrigin::signed(unauthorized_node),
                [2u8; 32],
                attestation
            ));

            // 提交请求并指定授权节点
            assert_ok!(TeePrivacy::submit_compute_request(
                RuntimeOrigin::signed(user),
                0,
                [0x00; 32],
                Some(authorized_node)
            ));

            // 未授权节点尝试提交结果应该失败
            assert_noop!(
                TeePrivacy::submit_compute_result(
                    RuntimeOrigin::signed(unauthorized_node),
                    0,
                    [0xFF; 32],
                    [0xEE; 64]
                ),
                Error::<Test>::NotAssignedNode
            );
        });
    }

    #[test]
    fn security_invalid_attestation() {
        new_test_ext().execute_with(|| {
            let node = 1u64;

            // 创建无效的认证报告（MR_ENCLAVE 不在允许列表中）
            let invalid_attestation = TeeAttestation {
                tee_type: TeeType::IntelSgx,
                mr_enclave: [0xFF; 32], // 不在允许列表中
                mr_signer: [10u8; 32],  // 使用有效的 mr_signer
                isv_prod_id: 1,
                isv_svn: 1,
                report_data: [0u8; 64],
                ias_signature: BoundedVec::try_from(vec![0u8; 64]).unwrap(),
                timestamp: 1704067200,
            };

            // 注册应该失败
            assert_noop!(
                TeePrivacy::register_tee_node(
                    RuntimeOrigin::signed(node),
                    [1u8; 32],
                    invalid_attestation
                ),
                Error::<Test>::MrEnclaveNotAllowed
            );
        });
    }

    #[test]
    fn security_double_registration() {
        new_test_ext().execute_with(|| {
            let node = 1u64;

            // 第一次注册
            setup_tee_node(node);

            // 第二次注册应该失败（使用相同的 mr_enclave）
            let mut attestation = create_test_attestation();
            attestation.mr_enclave[0] = node as u8; // 使用相同的 mr_enclave

            assert_noop!(
                TeePrivacy::register_tee_node(
                    RuntimeOrigin::signed(node),
                    [1u8; 32],
                    attestation
                ),
                Error::<Test>::NodeAlreadyRegistered
            );
        });
    }

    #[test]
    fn security_request_cancellation() {
        new_test_ext().execute_with(|| {
            let node = 1u64;
            let user1 = 2u64;
            let user2 = 3u64;

            setup_staked_node(node, 1000);

            // user1 提交请求
            assert_ok!(TeePrivacy::submit_compute_request(
                RuntimeOrigin::signed(user1),
                0,
                [0x00; 32],
                None
            ));

            // user2 尝试取消 user1 的请求应该失败
            assert_noop!(
                TeePrivacy::cancel_compute_request(RuntimeOrigin::signed(user2), 0),
                Error::<Test>::NotRequester
            );

            // user1 可以取消自己的请求
            assert_ok!(TeePrivacy::cancel_compute_request(
                RuntimeOrigin::signed(user1),
                0
            ));
        });
    }

    #[test]
    fn security_admin_only_functions() {
        new_test_ext().execute_with(|| {
            let non_admin = 1u64;

            // 非管理员尝试添加 MR_ENCLAVE 应该失败
            assert_noop!(
                TeePrivacy::add_allowed_mr_enclave(RuntimeOrigin::signed(non_admin), [1u8; 32]),
                sp_runtime::DispatchError::BadOrigin
            );

            // 非管理员尝试设置审计状态应该失败
            assert_noop!(
                TeePrivacy::set_audit_enabled(RuntimeOrigin::signed(non_admin), true),
                sp_runtime::DispatchError::BadOrigin
            );
        });
    }
}
