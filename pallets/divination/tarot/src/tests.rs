//! 塔罗牌 Pallet 单元测试
//!
//! 测试各种占卜功能的正确性

use crate::{mock::*, types::*, Error, Event};
use frame_support::{assert_noop, assert_ok, BoundedVec, pallet_prelude::ConstU32};
use sp_runtime::traits::BadOrigin;

/// 测试随机抽牌占卜
#[test]
fn test_divine_random_works() {
    new_test_ext().execute_with(|| {
        let question_hash = [1u8; 32];

        // Alice 进行随机占卜（单张牌）
        assert_ok!(Tarot::divine_random(
            RuntimeOrigin::signed(ALICE),
            SpreadType::SingleCard,
            question_hash,
            PrivacyMode::Private
        ));

        // 检查占卜记录是否创建
        assert!(Tarot::readings(0).is_some());
        let reading = Tarot::readings(0).unwrap();
        assert_eq!(reading.diviner, ALICE);
        assert_eq!(reading.spread_type, SpreadType::SingleCard);
        assert_eq!(reading.method, DivinationMethod::Random);
        assert_eq!(reading.cards.len(), 1);
        assert_eq!(reading.privacy_mode, PrivacyMode::Private);

        // 检查用户占卜索引
        let user_readings = Tarot::user_readings(ALICE);
        assert_eq!(user_readings.len(), 1);
        assert_eq!(user_readings[0], 0);

        // 检查下一个 ID 已递增
        assert_eq!(Tarot::next_reading_id(), 1);

        // 检查事件
        System::assert_last_event(
            Event::ReadingCreated {
                reading_id: 0,
                diviner: ALICE,
                spread_type: SpreadType::SingleCard,
                method: DivinationMethod::Random,
            }
            .into(),
        );
    });
}

/// 测试三张牌占卜
#[test]
fn test_divine_three_card_spread() {
    new_test_ext().execute_with(|| {
        let question_hash = [2u8; 32];

        // Alice 进行三张牌占卜（时间线）
        assert_ok!(Tarot::divine_random(
            RuntimeOrigin::signed(ALICE),
            SpreadType::ThreeCardTime,
            question_hash,
            PrivacyMode::Public // 公开
        ));

        let reading = Tarot::readings(0).unwrap();
        assert_eq!(reading.cards.len(), 3);
        assert_eq!(reading.privacy_mode, PrivacyMode::Public);

        // 检查公开列表
        let public_readings = Tarot::public_readings();
        assert_eq!(public_readings.len(), 1);
        assert_eq!(public_readings[0], 0);
    });
}

/// 测试凯尔特十字牌阵（10张牌）
#[test]
fn test_divine_celtic_cross() {
    new_test_ext().execute_with(|| {
        let question_hash = [3u8; 32];

        assert_ok!(Tarot::divine_random(
            RuntimeOrigin::signed(ALICE),
            SpreadType::CelticCross,
            question_hash,
            PrivacyMode::Private
        ));

        let reading = Tarot::readings(0).unwrap();
        assert_eq!(reading.cards.len(), 10);

        // 检查所有牌都不重复
        let mut card_ids: Vec<u8> = reading.cards.iter().map(|c| c.card.id).collect();
        card_ids.sort();
        card_ids.dedup();
        assert_eq!(card_ids.len(), 10);
    });
}

/// 测试时间起卦占卜
#[test]
fn test_divine_by_time_works() {
    new_test_ext().execute_with(|| {
        let question_hash = [4u8; 32];

        assert_ok!(Tarot::divine_by_time(
            RuntimeOrigin::signed(BOB),
            SpreadType::SingleCard,
            question_hash,
            PrivacyMode::Private
        ));

        let reading = Tarot::readings(0).unwrap();
        assert_eq!(reading.diviner, BOB);
        assert_eq!(reading.method, DivinationMethod::ByTime);
    });
}

/// 测试数字起卦占卜
#[test]
fn test_divine_by_numbers_works() {
    new_test_ext().execute_with(|| {
        let question_hash = [5u8; 32];
        let numbers: BoundedVec<u16, ConstU32<16>> =
            vec![7u16, 13, 42].try_into().unwrap();

        assert_ok!(Tarot::divine_by_numbers(
            RuntimeOrigin::signed(CHARLIE),
            numbers,
            SpreadType::ThreeCardSituation,
            question_hash,
            PrivacyMode::Private
        ));

        let reading = Tarot::readings(0).unwrap();
        assert_eq!(reading.diviner, CHARLIE);
        assert_eq!(reading.method, DivinationMethod::ByNumbers);
        assert_eq!(reading.cards.len(), 3);
    });
}

/// 测试数字起卦参数缺失
#[test]
fn test_divine_by_numbers_missing_params() {
    new_test_ext().execute_with(|| {
        let question_hash = [6u8; 32];
        let empty_numbers: BoundedVec<u16, ConstU32<16>> = vec![].try_into().unwrap();

        assert_noop!(
            Tarot::divine_by_numbers(
                RuntimeOrigin::signed(ALICE),
                empty_numbers,
                SpreadType::SingleCard,
                question_hash,
                PrivacyMode::Private
            ),
            Error::<Test>::MissingNumberParams
        );
    });
}

/// 测试手动指定占卜
#[test]
fn test_divine_manual_works() {
    new_test_ext().execute_with(|| {
        let question_hash = [7u8; 32];
        // 指定三张牌：愚者(0)正位, 魔术师(1)逆位, 女祭司(2)正位
        let cards: BoundedVec<(u8, bool), ConstU32<12>> =
            vec![(0, false), (1, true), (2, false)].try_into().unwrap();

        assert_ok!(Tarot::divine_manual(
            RuntimeOrigin::signed(ALICE),
            cards,
            SpreadType::ThreeCardTime,
            question_hash,
            PrivacyMode::Private
        ));

        let reading = Tarot::readings(0).unwrap();
        assert_eq!(reading.method, DivinationMethod::Manual);
        assert_eq!(reading.cards.len(), 3);

        // 验证指定的牌
        assert_eq!(reading.cards[0].card.id, 0);
        assert_eq!(reading.cards[0].position, CardPosition::Upright);
        assert_eq!(reading.cards[1].card.id, 1);
        assert_eq!(reading.cards[1].position, CardPosition::Reversed);
        assert_eq!(reading.cards[2].card.id, 2);
        assert_eq!(reading.cards[2].position, CardPosition::Upright);
    });
}

/// 测试手动指定牌数不匹配
#[test]
fn test_divine_manual_card_count_mismatch() {
    new_test_ext().execute_with(|| {
        let question_hash = [8u8; 32];
        // 指定2张牌但使用三张牌牌阵
        let cards: BoundedVec<(u8, bool), ConstU32<12>> =
            vec![(0, false), (1, false)].try_into().unwrap();

        assert_noop!(
            Tarot::divine_manual(
                RuntimeOrigin::signed(ALICE),
                cards,
                SpreadType::ThreeCardTime, // 需要3张
                question_hash,
                PrivacyMode::Private
            ),
            Error::<Test>::CardCountMismatch
        );
    });
}

/// 测试手动指定无效牌ID
#[test]
fn test_divine_manual_invalid_card_id() {
    new_test_ext().execute_with(|| {
        let question_hash = [9u8; 32];
        // 指定无效牌ID（78超出范围）
        let cards: BoundedVec<(u8, bool), ConstU32<12>> =
            vec![(0, false), (78, false), (2, false)].try_into().unwrap();

        assert_noop!(
            Tarot::divine_manual(
                RuntimeOrigin::signed(ALICE),
                cards,
                SpreadType::ThreeCardTime,
                question_hash,
                PrivacyMode::Private
            ),
            Error::<Test>::InvalidCardId
        );
    });
}

/// 测试手动指定重复牌
#[test]
fn test_divine_manual_duplicate_cards() {
    new_test_ext().execute_with(|| {
        let question_hash = [10u8; 32];
        // 指定重复的牌
        let cards: BoundedVec<(u8, bool), ConstU32<12>> =
            vec![(0, false), (0, true), (2, false)].try_into().unwrap();

        assert_noop!(
            Tarot::divine_manual(
                RuntimeOrigin::signed(ALICE),
                cards,
                SpreadType::ThreeCardTime,
                question_hash,
                PrivacyMode::Private
            ),
            Error::<Test>::InvalidCardId
        );
    });
}

/// 测试每日占卜次数限制
#[test]
fn test_daily_limit() {
    new_test_ext().execute_with(|| {
        let question_hash = [11u8; 32];

        // 进行10次占卜（达到每日上限）
        for _ in 0..10 {
            assert_ok!(Tarot::divine_random(
                RuntimeOrigin::signed(ALICE),
                SpreadType::SingleCard,
                question_hash,
                PrivacyMode::Private
            ));
        }

        // 第11次应该失败
        assert_noop!(
            Tarot::divine_random(
                RuntimeOrigin::signed(ALICE),
                SpreadType::SingleCard,
                question_hash,
                PrivacyMode::Private
            ),
            Error::<Test>::DailyLimitExceeded
        );

        // Bob 应该仍可以占卜
        assert_ok!(Tarot::divine_random(
            RuntimeOrigin::signed(BOB),
            SpreadType::SingleCard,
            question_hash,
            PrivacyMode::Private
        ));
    });
}

/// 测试更改隐私模式
#[test]
fn test_set_reading_privacy_mode() {
    new_test_ext().execute_with(|| {
        let question_hash = [12u8; 32];

        // 创建私密占卜
        assert_ok!(Tarot::divine_random(
            RuntimeOrigin::signed(ALICE),
            SpreadType::SingleCard,
            question_hash,
            PrivacyMode::Private
        ));

        // 公开列表应为空
        assert!(Tarot::public_readings().is_empty());

        // 设为公开
        assert_ok!(Tarot::set_reading_privacy_mode(
            RuntimeOrigin::signed(ALICE),
            0,
            PrivacyMode::Public
        ));

        // 检查更新
        let reading = Tarot::readings(0).unwrap();
        assert_eq!(reading.privacy_mode, PrivacyMode::Public);
        assert_eq!(Tarot::public_readings().len(), 1);

        // 再设为私密
        assert_ok!(Tarot::set_reading_privacy_mode(
            RuntimeOrigin::signed(ALICE),
            0,
            PrivacyMode::Private
        ));

        let reading = Tarot::readings(0).unwrap();
        assert_eq!(reading.privacy_mode, PrivacyMode::Private);
        assert!(Tarot::public_readings().is_empty());
    });
}

/// 测试非所有者无法更改隐私模式
#[test]
fn test_set_privacy_mode_not_owner() {
    new_test_ext().execute_with(|| {
        let question_hash = [13u8; 32];

        // Alice 创建占卜
        assert_ok!(Tarot::divine_random(
            RuntimeOrigin::signed(ALICE),
            SpreadType::SingleCard,
            question_hash,
            PrivacyMode::Private
        ));

        // Bob 尝试更改（应失败）
        assert_noop!(
            Tarot::set_reading_privacy_mode(RuntimeOrigin::signed(BOB), 0, PrivacyMode::Public),
            Error::<Test>::NotOwner
        );
    });
}

/// 测试请求 AI 解读
#[test]
#[allow(deprecated)]
fn test_request_ai_interpretation() {
    new_test_ext().execute_with(|| {
        let question_hash = [14u8; 32];

        // 创建占卜
        assert_ok!(Tarot::divine_random(
            RuntimeOrigin::signed(ALICE),
            SpreadType::SingleCard,
            question_hash,
            PrivacyMode::Private
        ));

        let alice_balance_before = Balances::free_balance(ALICE);
        let treasury_balance_before = Balances::free_balance(TREASURY);

        // 请求 AI 解读
        assert_ok!(Tarot::request_ai_interpretation(
            RuntimeOrigin::signed(ALICE),
            0
        ));

        // 检查费用已扣除
        let fee = AiInterpretationFee::get();
        assert_eq!(
            Balances::free_balance(ALICE),
            alice_balance_before - fee
        );
        assert_eq!(
            Balances::free_balance(TREASURY),
            treasury_balance_before + fee
        );

        // 检查请求已记录
        assert!(Tarot::ai_interpretation_requests(0).is_some());
        assert_eq!(Tarot::ai_interpretation_requests(0).unwrap(), ALICE);

        // 检查事件
        System::assert_last_event(
            Event::AiInterpretationRequested {
                reading_id: 0,
                requester: ALICE,
            }
            .into(),
        );
    });
}

/// 测试重复请求 AI 解读
#[test]
#[allow(deprecated)]
fn test_request_ai_interpretation_already_exists() {
    new_test_ext().execute_with(|| {
        let question_hash = [15u8; 32];

        assert_ok!(Tarot::divine_random(
            RuntimeOrigin::signed(ALICE),
            SpreadType::SingleCard,
            question_hash,
            PrivacyMode::Private
        ));

        // 第一次请求成功
        assert_ok!(Tarot::request_ai_interpretation(
            RuntimeOrigin::signed(ALICE),
            0
        ));

        // 第二次请求失败
        assert_noop!(
            Tarot::request_ai_interpretation(RuntimeOrigin::signed(ALICE), 0),
            Error::<Test>::AiRequestAlreadyExists
        );
    });
}

/// 测试提交 AI 解读结果
#[test]
#[allow(deprecated)]
fn test_submit_ai_interpretation() {
    new_test_ext().execute_with(|| {
        let question_hash = [16u8; 32];

        // 创建占卜并请求解读
        assert_ok!(Tarot::divine_random(
            RuntimeOrigin::signed(ALICE),
            SpreadType::SingleCard,
            question_hash,
            PrivacyMode::Private
        ));
        assert_ok!(Tarot::request_ai_interpretation(
            RuntimeOrigin::signed(ALICE),
            0
        ));

        // 提交解读（需要 root 权限）
        let cid: BoundedVec<u8, ConstU32<64>> =
            b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec().try_into().unwrap();

        assert_ok!(Tarot::submit_ai_interpretation(
            RuntimeOrigin::root(),
            0,
            cid.clone()
        ));

        // 检查解读已保存
        let reading = Tarot::readings(0).unwrap();
        assert!(reading.interpretation_cid.is_some());
        assert_eq!(reading.interpretation_cid.unwrap(), cid);

        // 检查请求已移除
        assert!(Tarot::ai_interpretation_requests(0).is_none());

        // 检查事件
        System::assert_last_event(
            Event::AiInterpretationSubmitted {
                reading_id: 0,
                cid,
            }
            .into(),
        );
    });
}

/// 测试非授权提交 AI 解读
#[test]
#[allow(deprecated)]
fn test_submit_ai_interpretation_unauthorized() {
    new_test_ext().execute_with(|| {
        let question_hash = [17u8; 32];

        assert_ok!(Tarot::divine_random(
            RuntimeOrigin::signed(ALICE),
            SpreadType::SingleCard,
            question_hash,
            PrivacyMode::Private
        ));
        assert_ok!(Tarot::request_ai_interpretation(
            RuntimeOrigin::signed(ALICE),
            0
        ));

        let cid: BoundedVec<u8, ConstU32<64>> =
            b"QmTest".to_vec().try_into().unwrap();

        // 普通用户不能提交
        assert_noop!(
            Tarot::submit_ai_interpretation(RuntimeOrigin::signed(ALICE), 0, cid),
            BadOrigin
        );
    });
}

/// 测试用户统计更新
#[test]
fn test_user_stats_update() {
    new_test_ext().execute_with(|| {
        let question_hash = [18u8; 32];

        // 初始统计
        let stats_before = Tarot::user_stats(ALICE);
        assert_eq!(stats_before.total_readings, 0);

        // 进行几次占卜
        for _ in 0..3 {
            assert_ok!(Tarot::divine_random(
                RuntimeOrigin::signed(ALICE),
                SpreadType::SingleCard,
                question_hash,
                PrivacyMode::Private
            ));
        }

        // 检查统计更新
        let stats_after = Tarot::user_stats(ALICE);
        assert_eq!(stats_after.total_readings, 3);
    });
}

/// 测试占卜记录不存在
#[test]
#[allow(deprecated)]
fn test_reading_not_found() {
    new_test_ext().execute_with(|| {
        // 尝试更改不存在的占卜
        assert_noop!(
            Tarot::set_reading_privacy_mode(RuntimeOrigin::signed(ALICE), 999, PrivacyMode::Public),
            Error::<Test>::ReadingNotFound
        );

        // 尝试请求不存在的占卜解读
        assert_noop!(
            Tarot::request_ai_interpretation(RuntimeOrigin::signed(ALICE), 999),
            Error::<Test>::ReadingNotFound
        );
    });
}

/// 测试多种牌阵类型
#[test]
fn test_various_spread_types() {
    new_test_ext().execute_with(|| {
        let question_hash = [19u8; 32];

        // 测试所有牌阵类型
        let spreads = vec![
            (SpreadType::SingleCard, 1),
            (SpreadType::ThreeCardTime, 3),
            (SpreadType::ThreeCardSituation, 3),
            (SpreadType::LoveRelationship, 5),
            (SpreadType::CareerGuidance, 6),
            (SpreadType::DecisionMaking, 7),
        ];

        for (i, (spread_type, expected_count)) in spreads.iter().enumerate() {
            assert_ok!(Tarot::divine_random(
                RuntimeOrigin::signed(ALICE),
                *spread_type,
                question_hash,
                PrivacyMode::Private
            ));

            let reading = Tarot::readings(i as u64).unwrap();
            assert_eq!(reading.cards.len(), *expected_count);
        }
    });
}

/// 测试塔罗牌结构正确性
#[test]
fn test_tarot_card_structure() {
    new_test_ext().execute_with(|| {
        let question_hash = [20u8; 32];

        // 手动指定一些特定的牌来测试结构
        // 愚者(0), 世界(21), 权杖Ace(22), 星币国王(77)
        let cards: BoundedVec<(u8, bool), ConstU32<12>> =
            vec![(0, false), (21, true), (22, false)].try_into().unwrap();

        assert_ok!(Tarot::divine_manual(
            RuntimeOrigin::signed(ALICE),
            cards,
            SpreadType::ThreeCardTime,
            question_hash,
            PrivacyMode::Private
        ));

        let reading = Tarot::readings(0).unwrap();

        // 检查愚者
        let fool = &reading.cards[0];
        assert!(fool.card.is_major());
        assert_eq!(fool.card.number, 0);
        assert_eq!(fool.card.suit, Suit::None);
        assert_eq!(fool.position, CardPosition::Upright);

        // 检查世界
        let world = &reading.cards[1];
        assert!(world.card.is_major());
        assert_eq!(world.card.number, 21);
        assert_eq!(world.position, CardPosition::Reversed);

        // 检查权杖Ace
        let wands_ace = &reading.cards[2];
        assert!(!wands_ace.card.is_major());
        assert_eq!(wands_ace.card.suit, Suit::Wands);
        assert_eq!(wands_ace.card.number, 1);
    });
}
