//! Tests for pallet-divination-ai

use crate::{mock::*, Error, Event};
use crate::types::{DisputeResolution, DisputeStatus, FeeDistribution};
use frame_support::{assert_noop, assert_ok};
use pallet_divination_common::{DivinationType, InterpretationStatus, InterpretationType, RarityInput};

// ==================== 预言机注册测试 ====================

#[test]
fn register_oracle_works() {
    new_test_ext().execute_with(|| {
        // Oracle (账户 4) 注册
        assert_ok!(DivinationAiPallet::register_oracle(
            RuntimeOrigin::signed(4),
            b"Test Oracle".to_vec(),
            0b11,  // 支持 Meihua 和 Bazi
            0b111, // 支持 Basic, Detailed, Professional
        ));

        // 验证预言机已注册
        let oracle = DivinationAiPallet::oracles(4).expect("Oracle should exist");
        assert_eq!(oracle.account, 4);
        assert!(oracle.is_active);
        assert_eq!(oracle.supported_divination_types, 0b11);
        assert_eq!(oracle.supported_interpretation_types, 0b111);

        // 验证活跃列表
        let active = DivinationAiPallet::active_oracles();
        assert!(active.contains(&4));

        // 验证事件
        System::assert_has_event(
            Event::OracleRegistered {
                oracle: 4,
                stake: 10_000_000_000_000,
            }
            .into(),
        );
    });
}

#[test]
fn register_oracle_fails_if_already_registered() {
    new_test_ext().execute_with(|| {
        assert_ok!(DivinationAiPallet::register_oracle(
            RuntimeOrigin::signed(4),
            b"Oracle".to_vec(),
            0b11,
            0b111,
        ));

        assert_noop!(
            DivinationAiPallet::register_oracle(
                RuntimeOrigin::signed(4),
                b"Oracle 2".to_vec(),
                0b11,
                0b111,
            ),
            Error::<Test>::OracleAlreadyRegistered
        );
    });
}

// ==================== 解读请求测试 ====================

#[test]
fn request_interpretation_works() {
    new_test_ext().execute_with(|| {
        // 添加模拟占卜结果
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        // Alice 请求解读
        assert_ok!(DivinationAiPallet::request_interpretation(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            InterpretationType::Basic,
            None,
        ));

        // 验证请求已创建
        let request = DivinationAiPallet::requests(0).expect("Request should exist");
        assert_eq!(request.divination_type, DivinationType::Meihua);
        assert_eq!(request.result_id, 1);
        assert_eq!(request.requester, 1);
        assert_eq!(request.interpretation_type, InterpretationType::Basic);
        assert_eq!(request.status, InterpretationStatus::Pending);

        // 验证统计
        let stats = DivinationAiPallet::stats();
        assert_eq!(stats.total_requests, 1);

        // 验证事件
        System::assert_has_event(
            Event::InterpretationRequested {
                request_id: 0,
                divination_type: DivinationType::Meihua,
                result_id: 1,
                requester: 1,
                interpretation_type: InterpretationType::Basic,
                fee: 1_000_000_000_000,
            }
            .into(),
        );
    });
}

#[test]
fn request_interpretation_fails_if_result_not_found() {
    new_test_ext().execute_with(|| {
        // 没有添加模拟数据
        assert_noop!(
            DivinationAiPallet::request_interpretation(
                RuntimeOrigin::signed(1),
                DivinationType::Meihua,
                999,
                InterpretationType::Basic,
                None,
            ),
            Error::<Test>::DivinationResultNotFound
        );
    });
}

// ==================== 接受请求测试 ====================

#[test]
fn accept_request_works() {
    new_test_ext().execute_with(|| {
        // 设置
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        // 注册预言机
        assert_ok!(DivinationAiPallet::register_oracle(
            RuntimeOrigin::signed(4),
            b"Oracle".to_vec(),
            0b11,
            0b111,
        ));

        // 创建请求
        assert_ok!(DivinationAiPallet::request_interpretation(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            InterpretationType::Basic,
            None,
        ));

        // 预言机接受请求
        assert_ok!(DivinationAiPallet::accept_request(RuntimeOrigin::signed(4), 0));

        // 验证请求状态
        let request = DivinationAiPallet::requests(0).unwrap();
        assert_eq!(request.status, InterpretationStatus::Processing);
        assert_eq!(request.oracle_node, Some(4));

        // 验证事件
        System::assert_has_event(
            Event::RequestAccepted {
                request_id: 0,
                oracle: 4,
            }
            .into(),
        );
    });
}

#[test]
fn accept_request_fails_if_oracle_not_active() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        // 注册并暂停预言机
        assert_ok!(DivinationAiPallet::register_oracle(
            RuntimeOrigin::signed(4),
            b"Oracle".to_vec(),
            0b11,
            0b111,
        ));
        assert_ok!(DivinationAiPallet::pause_oracle(RuntimeOrigin::signed(4)));

        // 创建请求
        assert_ok!(DivinationAiPallet::request_interpretation(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            InterpretationType::Basic,
            None,
        ));

        // 应该失败
        assert_noop!(
            DivinationAiPallet::accept_request(RuntimeOrigin::signed(4), 0),
            Error::<Test>::OracleNotActive
        );
    });
}

// ==================== 提交结果测试 ====================

#[test]
fn submit_result_works() {
    new_test_ext().execute_with(|| {
        // 设置
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationAiPallet::register_oracle(
            RuntimeOrigin::signed(4),
            b"Oracle".to_vec(),
            0b11,
            0b111,
        ));

        assert_ok!(DivinationAiPallet::request_interpretation(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            InterpretationType::Basic,
            None,
        ));

        assert_ok!(DivinationAiPallet::accept_request(RuntimeOrigin::signed(4), 0));

        // 提交结果
        assert_ok!(DivinationAiPallet::submit_result(
            RuntimeOrigin::signed(4),
            0,
            b"QmContentCid123".to_vec(),
            Some(b"QmSummaryCid".to_vec()),
            b"gpt-4".to_vec(),
            b"zh-CN".to_vec(),
        ));

        // 验证结果
        let result = DivinationAiPallet::results(0).expect("Result should exist");
        assert_eq!(result.oracle, 4);
        assert!(result.user_rating.is_none());

        // 验证请求状态
        let request = DivinationAiPallet::requests(0).unwrap();
        assert_eq!(request.status, InterpretationStatus::Completed);

        // 验证统计
        let stats = DivinationAiPallet::stats();
        assert_eq!(stats.completed_requests, 1);
    });
}

// ==================== 评分测试 ====================

#[test]
fn rate_result_works() {
    new_test_ext().execute_with(|| {
        // 完整流程
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationAiPallet::register_oracle(
            RuntimeOrigin::signed(4),
            b"Oracle".to_vec(),
            0b11,
            0b111,
        ));

        assert_ok!(DivinationAiPallet::request_interpretation(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            InterpretationType::Basic,
            None,
        ));

        assert_ok!(DivinationAiPallet::accept_request(RuntimeOrigin::signed(4), 0));

        assert_ok!(DivinationAiPallet::submit_result(
            RuntimeOrigin::signed(4),
            0,
            b"QmCid".to_vec(),
            None,
            b"gpt".to_vec(),
            b"zh".to_vec(),
        ));

        // 用户评分
        assert_ok!(DivinationAiPallet::rate_result(RuntimeOrigin::signed(1), 0, 5));

        // 验证评分
        let result = DivinationAiPallet::results(0).unwrap();
        assert_eq!(result.user_rating, Some(5));

        // 验证事件
        System::assert_has_event(
            Event::ResultRated {
                request_id: 0,
                user: 1,
                rating: 5,
            }
            .into(),
        );
    });
}

#[test]
fn rate_result_fails_if_invalid_rating() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationAiPallet::register_oracle(
            RuntimeOrigin::signed(4),
            b"Oracle".to_vec(),
            0b11,
            0b111,
        ));

        assert_ok!(DivinationAiPallet::request_interpretation(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            InterpretationType::Basic,
            None,
        ));

        assert_ok!(DivinationAiPallet::accept_request(RuntimeOrigin::signed(4), 0));

        assert_ok!(DivinationAiPallet::submit_result(
            RuntimeOrigin::signed(4),
            0,
            b"QmCid".to_vec(),
            None,
            b"gpt".to_vec(),
            b"zh".to_vec(),
        ));

        // 评分范围无效
        assert_noop!(
            DivinationAiPallet::rate_result(RuntimeOrigin::signed(1), 0, 0),
            Error::<Test>::InvalidRating
        );

        assert_noop!(
            DivinationAiPallet::rate_result(RuntimeOrigin::signed(1), 0, 6),
            Error::<Test>::InvalidRating
        );
    });
}

// ==================== 报告失败测试 ====================

#[test]
fn report_failure_works() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationAiPallet::register_oracle(
            RuntimeOrigin::signed(4),
            b"Oracle".to_vec(),
            0b11,
            0b111,
        ));

        assert_ok!(DivinationAiPallet::request_interpretation(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            InterpretationType::Basic,
            None,
        ));

        assert_ok!(DivinationAiPallet::accept_request(RuntimeOrigin::signed(4), 0));

        // 报告失败
        assert_ok!(DivinationAiPallet::report_failure(
            RuntimeOrigin::signed(4),
            0,
            b"Model error".to_vec(),
        ));

        // 验证请求状态
        let request = DivinationAiPallet::requests(0).unwrap();
        assert_eq!(request.status, InterpretationStatus::Failed);

        // 验证统计
        let stats = DivinationAiPallet::stats();
        assert_eq!(stats.failed_requests, 1);
    });
}

// ==================== 争议测试 ====================

#[test]
fn create_and_resolve_dispute_works() {
    new_test_ext().execute_with(|| {
        // 完整流程
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationAiPallet::register_oracle(
            RuntimeOrigin::signed(4),
            b"Oracle".to_vec(),
            0b11,
            0b111,
        ));

        assert_ok!(DivinationAiPallet::request_interpretation(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            InterpretationType::Basic,
            None,
        ));

        assert_ok!(DivinationAiPallet::accept_request(RuntimeOrigin::signed(4), 0));

        assert_ok!(DivinationAiPallet::submit_result(
            RuntimeOrigin::signed(4),
            0,
            b"QmCid".to_vec(),
            None,
            b"gpt".to_vec(),
            b"zh".to_vec(),
        ));

        // 创建争议
        assert_ok!(DivinationAiPallet::create_dispute(
            RuntimeOrigin::signed(1),
            0,
            [0u8; 32],
        ));

        // 验证争议
        let dispute = DivinationAiPallet::disputes(0).expect("Dispute should exist");
        assert_eq!(dispute.request_id, 0);
        assert_eq!(dispute.disputer, 1);
        assert_eq!(dispute.status, DisputeStatus::Pending);

        // 解决争议（用户胜诉）
        assert_ok!(DivinationAiPallet::resolve_dispute(
            RuntimeOrigin::root(),
            0,
            DisputeResolution::UserWins,
        ));

        // 验证争议状态
        let dispute = DivinationAiPallet::disputes(0).unwrap();
        assert_eq!(dispute.status, DisputeStatus::Resolved);
        assert_eq!(dispute.resolution, Some(DisputeResolution::UserWins));

        // 验证统计
        let stats = DivinationAiPallet::stats();
        assert_eq!(stats.total_disputes, 1);
        assert_eq!(stats.disputes_user_wins, 1);
    });
}

// ==================== 预言机管理测试 ====================

#[test]
fn pause_and_resume_oracle_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(DivinationAiPallet::register_oracle(
            RuntimeOrigin::signed(4),
            b"Oracle".to_vec(),
            0b11,
            0b111,
        ));

        // 暂停
        assert_ok!(DivinationAiPallet::pause_oracle(RuntimeOrigin::signed(4)));

        let oracle = DivinationAiPallet::oracles(4).unwrap();
        assert!(!oracle.is_active);
        assert!(!DivinationAiPallet::active_oracles().contains(&4));

        // 恢复
        assert_ok!(DivinationAiPallet::resume_oracle(RuntimeOrigin::signed(4)));

        let oracle = DivinationAiPallet::oracles(4).unwrap();
        assert!(oracle.is_active);
        assert!(DivinationAiPallet::active_oracles().contains(&4));
    });
}

#[test]
fn unregister_oracle_works() {
    new_test_ext().execute_with(|| {
        let balance_before = Balances::free_balance(4);

        assert_ok!(DivinationAiPallet::register_oracle(
            RuntimeOrigin::signed(4),
            b"Oracle".to_vec(),
            0b11,
            0b111,
        ));

        // 注销
        assert_ok!(DivinationAiPallet::unregister_oracle(RuntimeOrigin::signed(4)));

        // 验证预言机已移除
        assert!(DivinationAiPallet::oracles(4).is_none());
        assert!(!DivinationAiPallet::active_oracles().contains(&4));

        // 验证质押已退还
        let balance_after = Balances::free_balance(4);
        assert_eq!(balance_after, balance_before);
    });
}

// ==================== 多类型测试 ====================

#[test]
fn multiple_divination_types_work() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());
        MockDivinationProvider::add_result(DivinationType::Bazi, 1, 1, RarityInput::common());

        assert_ok!(DivinationAiPallet::register_oracle(
            RuntimeOrigin::signed(4),
            b"Oracle".to_vec(),
            0b11,  // 支持 Meihua 和 Bazi
            0b111, // 支持 Basic, Detailed, Professional
        ));

        // 梅花请求
        assert_ok!(DivinationAiPallet::request_interpretation(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            InterpretationType::Basic,
            None,
        ));

        // 八字请求
        assert_ok!(DivinationAiPallet::request_interpretation(
            RuntimeOrigin::signed(2),
            DivinationType::Bazi,
            1,
            InterpretationType::Detailed,
            None,
        ));

        // 验证类型统计
        let meihua_stats = DivinationAiPallet::type_stats(DivinationType::Meihua);
        assert_eq!(meihua_stats.request_count, 1);

        let bazi_stats = DivinationAiPallet::type_stats(DivinationType::Bazi);
        assert_eq!(bazi_stats.request_count, 1);
    });
}

// ==================== 费用分配测试 ====================

#[test]
fn update_fee_distribution_works() {
    new_test_ext().execute_with(|| {
        let new_distribution = FeeDistribution {
            oracle_share: 8000,
            treasury_share: 1500,
            burn_share: 300,
            staking_pool_share: 200,
        };

        assert_ok!(DivinationAiPallet::update_fee_distribution(
            RuntimeOrigin::root(),
            new_distribution.clone(),
        ));

        let stored = DivinationAiPallet::fee_distribution();
        assert_eq!(stored, new_distribution);
    });
}

#[test]
fn fee_distribution_validation_works() {
    new_test_ext().execute_with(|| {
        // 总和不等于 10000 应该失败
        let invalid_distribution = FeeDistribution {
            oracle_share: 5000,
            treasury_share: 1000,
            burn_share: 100,
            staking_pool_share: 100,
        };

        assert_noop!(
            DivinationAiPallet::update_fee_distribution(
                RuntimeOrigin::root(),
                invalid_distribution,
            ),
            Error::<Test>::InvalidRating // 使用现有错误类型
        );
    });
}
