//! # 奇门遁甲 Pallet 单元测试
//!
//! 本模块包含 pallet-qimen 的所有单元测试

#![allow(deprecated)]

use crate::{mock::*, types::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use pallet_divination_privacy::types::PrivacyMode;

// ==================== 时间起局测试 ====================

#[test]
fn divine_by_time_works() {
    new_test_ext().execute_with(|| {
        // 使用冬至上元阳遁一局的参数
        // 年柱：甲子(0,0)，月柱：丙寅(2,2)，日柱：甲子(0,0)，时柱：甲子(0,0)
        // 节气：冬至(0)，节气内天数：1

        assert_ok!(Qimen::divine_by_time(
            RuntimeOrigin::signed(ALICE),
            (0, 0), // 年柱：甲子
            (2, 2), // 月柱：丙寅
            (0, 0), // 日柱：甲子
            (0, 0), // 时柱：甲子
            0,      // 节气：冬至
            1,      // 节气内天数
            [0u8; 32], // 问题哈希
            false,  // 不公开
            None,   // name
            None,   // gender
            None,   // birth_year
            None,   // question
            None,   // question_type
            0,      // pan_method (转盘)
        ));

        // 验证排盘记录已创建
        assert!(Qimen::charts(0).is_some());

        // 验证事件已发出
        System::assert_has_event(RuntimeEvent::Qimen(Event::ChartCreated {
            chart_id: 0,
            diviner: ALICE,
            dun_type: DunType::Yang,
            ju_number: 1,
        }));

        // 验证用户排盘列表已更新
        let user_charts = Qimen::user_charts(ALICE);
        assert_eq!(user_charts.len(), 1);
        assert_eq!(user_charts[0], 0);

        // 验证用户统计已更新
        let stats = Qimen::user_stats(ALICE);
        assert_eq!(stats.total_charts, 1);
        assert_eq!(stats.yang_dun_count, 1);
    });
}

#[test]
fn divine_by_time_invalid_jieqi_fails() {
    new_test_ext().execute_with(|| {
        // 使用无效的节气（24及以上）
        assert_noop!(
            Qimen::divine_by_time(
                RuntimeOrigin::signed(ALICE),
                (0, 0),
                (2, 2),
                (0, 0),
                (0, 0),
                24, // 无效节气
                1,
                [0u8; 32],
                false,
                None, None, None, None, None, 0,
            ),
            Error::<Test>::InvalidJieQi
        );
    });
}

#[test]
fn divine_by_time_invalid_day_in_jieqi_fails() {
    new_test_ext().execute_with(|| {
        // 节气内天数必须为1-15
        assert_noop!(
            Qimen::divine_by_time(
                RuntimeOrigin::signed(ALICE),
                (0, 0),
                (2, 2),
                (0, 0),
                (0, 0),
                0,
                0, // 无效天数
                [0u8; 32],
                false,
                None, None, None, None, None, 0,
            ),
            Error::<Test>::InvalidDayInJieQi
        );

        assert_noop!(
            Qimen::divine_by_time(
                RuntimeOrigin::signed(ALICE),
                (0, 0),
                (2, 2),
                (0, 0),
                (0, 0),
                0,
                16, // 超出范围
                [0u8; 32],
                false,
                None, None, None, None, None, 0,
            ),
            Error::<Test>::InvalidDayInJieQi
        );
    });
}

// ==================== 数字起局测试 ====================

#[test]
fn divine_by_numbers_works() {
    new_test_ext().execute_with(|| {
        use frame_support::BoundedVec;

        let numbers: BoundedVec<u16, frame_support::traits::ConstU32<16>> =
            vec![3, 7, 9].try_into().unwrap();

        assert_ok!(Qimen::divine_by_numbers(
            RuntimeOrigin::signed(BOB),
            numbers,
            true, // 阳遁
            [1u8; 32],
            true, // 公开
            None, None, None, None, None, 0,
        ));

        // 验证排盘记录已创建
        let chart = Qimen::charts(0).unwrap();
        assert_eq!(chart.diviner, BOB);
        assert_eq!(chart.method, DivinationMethod::ByNumbers);
        assert_eq!(chart.dun_type, Some(DunType::Yang));
        assert_eq!(chart.privacy_mode, PrivacyMode::Public);

        // 验证公开列表已更新
        let public_charts = Qimen::public_charts();
        assert_eq!(public_charts.len(), 1);
    });
}

#[test]
fn divine_by_numbers_empty_fails() {
    new_test_ext().execute_with(|| {
        use frame_support::BoundedVec;

        let numbers: BoundedVec<u16, frame_support::traits::ConstU32<16>> =
            vec![].try_into().unwrap();

        assert_noop!(
            Qimen::divine_by_numbers(
                RuntimeOrigin::signed(ALICE),
                numbers,
                true,
                [0u8; 32],
                false,
                None, None, None, None, None, 0,
            ),
            Error::<Test>::MissingNumberParams
        );
    });
}

// ==================== 随机起局测试 ====================

#[test]
fn divine_random_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Qimen::divine_random(
            RuntimeOrigin::signed(CHARLIE),
            [2u8; 32],
            false,
            None, None, None, None, None, 0,
        ));

        let chart = Qimen::charts(0).unwrap();
        assert_eq!(chart.diviner, CHARLIE);
        assert_eq!(chart.method, DivinationMethod::Random);

        // 局数应该在1-9范围内
        let ju_number = chart.ju_number.unwrap();
        assert!(ju_number >= 1 && ju_number <= 9);
    });
}

// ==================== 手动起局测试 ====================

#[test]
fn divine_manual_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Qimen::divine_manual(
            RuntimeOrigin::signed(ALICE),
            false, // 阴遁
            5,     // 五局
            (4, 6), // 时柱：戊午
            [3u8; 32],
            true,
            None, None, None, None, None, 0,
        ));

        let chart = Qimen::charts(0).unwrap();
        assert_eq!(chart.dun_type, Some(DunType::Yin));
        assert_eq!(chart.ju_number, Some(5));
        assert_eq!(chart.method, DivinationMethod::Manual);
    });
}

#[test]
fn divine_manual_invalid_ju_number_fails() {
    new_test_ext().execute_with(|| {
        // 局数必须为1-9
        assert_noop!(
            Qimen::divine_manual(
                RuntimeOrigin::signed(ALICE),
                true,
                0, // 无效局数
                (0, 0),
                [0u8; 32],
                false,
                None, None, None, None, None, 0,
            ),
            Error::<Test>::InvalidJuNumber
        );

        assert_noop!(
            Qimen::divine_manual(
                RuntimeOrigin::signed(ALICE),
                true,
                10, // 超出范围
                (0, 0),
                [0u8; 32],
                false,
                None, None, None, None, None, 0,
            ),
            Error::<Test>::InvalidJuNumber
        );
    });
}

// ==================== 每日限制测试 ====================

#[test]
fn daily_limit_works() {
    new_test_ext().execute_with(|| {
        // 每日最大排盘次数为10次
        for i in 0..10 {
            assert_ok!(Qimen::divine_random(
                RuntimeOrigin::signed(ALICE),
                [i as u8; 32],
                false,
                None, None, None, None, None, 0,
            ));
        }

        // 第11次应该失败
        assert_noop!(
            Qimen::divine_random(
                RuntimeOrigin::signed(ALICE),
                [11u8; 32],
                false,
                None, None, None, None, None, 0,
            ),
            Error::<Test>::DailyLimitExceeded
        );

        // 其他用户不受影响
        assert_ok!(Qimen::divine_random(
            RuntimeOrigin::signed(BOB),
            [0u8; 32],
            false,
            None, None, None, None, None, 0,
        ));
    });
}

// ==================== AI 解读测试 ====================

#[test]
fn request_ai_interpretation_works() {
    new_test_ext().execute_with(|| {
        // 先创建一个排盘
        assert_ok!(Qimen::divine_random(
            RuntimeOrigin::signed(ALICE),
            [0u8; 32],
            false,
            None, None, None, None, None, 0,
        ));

        let initial_balance = Balances::free_balance(ALICE);

        // 请求 AI 解读
        assert_ok!(Qimen::request_ai_interpretation(
            RuntimeOrigin::signed(ALICE),
            0,
        ));

        // 验证费用已扣除
        assert_eq!(
            Balances::free_balance(ALICE),
            initial_balance - 1000 // AiInterpretationFee
        );

        // 验证请求已记录
        assert!(Qimen::ai_interpretation_requests(0).is_some());

        // 验证事件
        System::assert_has_event(RuntimeEvent::Qimen(Event::AiInterpretationRequested {
            chart_id: 0,
            requester: ALICE,
        }));
    });
}

#[test]
fn request_ai_interpretation_not_owner_fails() {
    new_test_ext().execute_with(|| {
        // Alice 创建排盘
        assert_ok!(Qimen::divine_random(
            RuntimeOrigin::signed(ALICE),
            [0u8; 32],
            false,
            None, None, None, None, None, 0,
        ));

        // Bob 尝试请求 AI 解读（应该失败）
        assert_noop!(
            Qimen::request_ai_interpretation(RuntimeOrigin::signed(BOB), 0),
            Error::<Test>::NotOwner
        );
    });
}

#[test]
fn request_ai_interpretation_duplicate_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Qimen::divine_random(
            RuntimeOrigin::signed(ALICE),
            [0u8; 32],
            false,
            None, None, None, None, None, 0,
        ));

        assert_ok!(Qimen::request_ai_interpretation(
            RuntimeOrigin::signed(ALICE),
            0,
        ));

        // 重复请求应该失败
        assert_noop!(
            Qimen::request_ai_interpretation(RuntimeOrigin::signed(ALICE), 0),
            Error::<Test>::AiRequestAlreadyExists
        );
    });
}

#[test]
fn submit_ai_interpretation_works() {
    new_test_ext().execute_with(|| {
        use frame_support::BoundedVec;

        // 创建排盘并请求解读
        assert_ok!(Qimen::divine_random(
            RuntimeOrigin::signed(ALICE),
            [0u8; 32],
            false,
            None, None, None, None, None, 0,
        ));

        assert_ok!(Qimen::request_ai_interpretation(
            RuntimeOrigin::signed(ALICE),
            0,
        ));

        // 提交解读结果（需要 Root 权限）
        let cid: BoundedVec<u8, frame_support::traits::ConstU32<64>> =
            b"QmTest123456789".to_vec().try_into().unwrap();

        assert_ok!(Qimen::submit_ai_interpretation(
            RuntimeOrigin::root(),
            0,
            cid.clone(),
        ));

        // 验证解读已保存
        let chart = Qimen::charts(0).unwrap();
        assert_eq!(chart.interpretation_cid, Some(cid.clone()));

        // 验证请求已移除
        assert!(Qimen::ai_interpretation_requests(0).is_none());

        // 验证事件
        System::assert_has_event(RuntimeEvent::Qimen(Event::AiInterpretationSubmitted {
            chart_id: 0,
            cid,
        }));
    });
}

// ==================== 公开状态测试 ====================

#[test]
fn set_chart_visibility_works() {
    new_test_ext().execute_with(|| {
        // 创建私密排盘
        assert_ok!(Qimen::divine_random(
            RuntimeOrigin::signed(ALICE),
            [0u8; 32],
            false, // 私密
            None, None, None, None, None, 0,
        ));

        assert_eq!(Qimen::public_charts().len(), 0);

        // 设置为公开
        assert_ok!(Qimen::set_chart_visibility(
            RuntimeOrigin::signed(ALICE),
            0,
            true,
        ));

        assert_eq!(Qimen::public_charts().len(), 1);

        // 再设置为私密
        assert_ok!(Qimen::set_chart_visibility(
            RuntimeOrigin::signed(ALICE),
            0,
            false,
        ));

        assert_eq!(Qimen::public_charts().len(), 0);
    });
}

#[test]
fn set_chart_visibility_not_owner_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Qimen::divine_random(
            RuntimeOrigin::signed(ALICE),
            [0u8; 32],
            false,
            None, None, None, None, None, 0,
        ));

        // Bob 尝试更改公开状态（应该失败）
        assert_noop!(
            Qimen::set_chart_visibility(RuntimeOrigin::signed(BOB), 0, true),
            Error::<Test>::NotOwner
        );
    });
}

// ==================== 排盘数据验证测试 ====================

#[test]
fn chart_palaces_are_valid() {
    new_test_ext().execute_with(|| {
        assert_ok!(Qimen::divine_by_time(
            RuntimeOrigin::signed(ALICE),
            (0, 0), // 甲子年
            (2, 2), // 丙寅月
            (0, 0), // 甲子日
            (0, 0), // 甲子时
            0,      // 冬至
            1,      // 上元
            [0u8; 32],
            false,
            None, None, None, None, None, 0,
        ));

        let chart = Qimen::charts(0).unwrap();

        // 获取九宫数据
        let palaces = chart.palaces.expect("palaces should exist");

        // 验证九宫都有有效数据
        for (i, palace) in palaces.iter().enumerate() {
            let expected_gong = JiuGong::from_num((i + 1) as u8).unwrap();
            assert_eq!(palace.gong, expected_gong);

            // 中宫无门无神
            if palace.gong == JiuGong::Zhong {
                assert!(palace.men.is_none());
                assert!(palace.shen.is_none());
            } else {
                // 其他宫应该有门和神
                assert!(palace.men.is_some());
                assert!(palace.shen.is_some());
            }
        }
    });
}

// ==================== 隐私模式测试 ====================

#[test]
fn privacy_mode_public_works() {
    new_test_ext().execute_with(|| {
        // 创建公开排盘
        assert_ok!(Qimen::divine_random(
            RuntimeOrigin::signed(ALICE),
            [0u8; 32],
            true, // 公开
            None, None, None, None, None, 0,
        ));

        let chart = Qimen::charts(0).unwrap();
        assert_eq!(chart.privacy_mode, PrivacyMode::Public);
        assert!(chart.has_calculation_data());
        assert!(chart.can_interpret());
        assert!(chart.is_public());
    });
}

#[test]
fn privacy_mode_partial_works() {
    new_test_ext().execute_with(|| {
        // 创建私密排盘（默认使用 Partial 模式）
        assert_ok!(Qimen::divine_random(
            RuntimeOrigin::signed(ALICE),
            [0u8; 32],
            false, // 私密
            None, None, None, None, None, 0,
        ));

        let chart = Qimen::charts(0).unwrap();
        assert_eq!(chart.privacy_mode, PrivacyMode::Partial);
        assert!(chart.has_calculation_data());
        assert!(chart.can_interpret());
        assert!(!chart.is_public());
    });
}

#[test]
fn chart_helper_methods_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Qimen::divine_by_time(
            RuntimeOrigin::signed(ALICE),
            (0, 0),
            (2, 2),
            (0, 0),
            (0, 0),
            0,
            1,
            [0u8; 32],
            false,
            None, None, None, None, None, 0,
        ));

        let chart = Qimen::charts(0).unwrap();

        // 测试 helper 方法
        assert!(chart.get_palaces().is_some());
        assert!(chart.get_day_ganzhi().is_some());
        assert!(chart.get_hour_ganzhi().is_some());
        assert!(chart.get_jie_qi().is_some());
        assert!(chart.get_zhi_fu_xing().is_some());
        assert!(chart.get_zhi_shi_men().is_some());
        assert!(chart.get_dun_type().is_some());
    });
}

// ==================== 类型测试 ====================

#[test]
fn test_tian_gan() {
    assert_eq!(TianGan::Jia.name(), "甲");
    assert_eq!(TianGan::Yi.wu_xing(), WuXing::Mu);
    assert!(TianGan::Yi.is_san_qi());
    assert!(TianGan::Wu.is_liu_yi());
    assert!(!TianGan::Jia.is_san_qi());
    assert!(!TianGan::Jia.is_liu_yi());
}

#[test]
fn test_jiu_gong() {
    assert_eq!(JiuGong::Kan.num(), 1);
    assert_eq!(JiuGong::Kan.direction(), "北");
    assert_eq!(JiuGong::Kan.wu_xing(), WuXing::Shui);

    // 测试阳遁顺序
    assert_eq!(JiuGong::Kan.next_yang(), JiuGong::Gen);
    assert_eq!(JiuGong::Gen.next_yang(), JiuGong::Zhen);

    // 测试阴遁顺序
    assert_eq!(JiuGong::Kan.next_yin(), JiuGong::Qian);
    assert_eq!(JiuGong::Qian.next_yin(), JiuGong::Dui);
}

#[test]
fn test_jiu_xing() {
    assert_eq!(JiuXing::TianPeng.name(), "天蓬");
    assert!(!JiuXing::TianPeng.is_auspicious()); // 凶星
    assert!(JiuXing::TianChong.is_auspicious()); // 吉星
    assert_eq!(JiuXing::TianPeng.original_palace(), JiuGong::Kan);
}

#[test]
fn test_ba_men() {
    assert_eq!(BaMen::Xiu.name(), "休门");
    assert!(BaMen::Xiu.is_auspicious()); // 吉门
    assert!(!BaMen::Si.is_auspicious()); // 凶门
    assert_eq!(BaMen::Kai.original_palace(), JiuGong::Qian);
}

#[test]
fn test_ba_shen() {
    assert_eq!(BaShen::ZhiFu.name(), "值符");
    assert!(BaShen::ZhiFu.is_auspicious()); // 吉神
    assert!(!BaShen::BaiHu.is_auspicious()); // 凶神
}

#[test]
fn test_wu_xing_relations() {
    // 测试相生
    assert!(WuXing::Jin.generates(&WuXing::Shui)); // 金生水
    assert!(WuXing::Shui.generates(&WuXing::Mu));  // 水生木
    assert!(!WuXing::Jin.generates(&WuXing::Mu));  // 金不生木

    // 测试相克
    assert!(WuXing::Jin.conquers(&WuXing::Mu));    // 金克木
    assert!(WuXing::Mu.conquers(&WuXing::Tu));     // 木克土
    assert!(!WuXing::Jin.conquers(&WuXing::Shui)); // 金不克水
}

// ==================== 解卦测试 ====================

#[test]
fn test_core_interpretation_size() {
    use crate::interpretation::QimenCoreInterpretation;
    use codec::Encode;

    let core = QimenCoreInterpretation {
        ge_ju: GeJuType::ZhengGe,
        yong_shen_gong: 1,
        zhi_fu_xing: JiuXing::TianQin,
        zhi_shi_men: BaMen::Kai,
        ri_gan_gong: 1,
        shi_gan_gong: 1,
        fortune: Fortune::Ping,
        fortune_score: 50,
        wang_shuai: WangShuai::Xiu,
        special_patterns: 0,
        confidence: 80,
        timestamp: 1000000,
        algorithm_version: 1,
    };

    let encoded = core.encode();
    println!("✅ QimenCoreInterpretation 编码大小: {} bytes", encoded.len());
    assert!(
        encoded.len() <= 20,
        "QimenCoreInterpretation 编码大小应 <= 20 bytes，实际: {} bytes",
        encoded.len()
    );
}

// ==================== 加密接口测试 ====================

#[test]
fn divine_by_solar_time_encrypted_public_works() {
    new_test_ext().execute_with(|| {
        // Public 模式（encryption_level = 0）
        assert_ok!(Qimen::divine_by_solar_time_encrypted(
            RuntimeOrigin::signed(ALICE),
            0,      // encryption_level: Public
            2024,   // solar_year
            1,      // solar_month
            15,     // solar_day
            10,     // hour
            [0u8; 32], // question_hash
            None,   // encrypted_data (不需要)
            None,   // data_hash (不需要)
            None,   // owner_key_backup (不需要)
            Some(0), // question_type: General
            0,      // pan_method: ZhuanPan
        ));

        // 验证排盘记录已创建
        let chart = Qimen::charts(0).unwrap();
        assert_eq!(chart.diviner, ALICE);
        assert_eq!(chart.privacy_mode, PrivacyMode::Public);
        assert!(chart.has_calculation_data());
        assert!(chart.can_interpret());

        // 验证公开列表已更新
        let public_charts = Qimen::public_charts();
        assert_eq!(public_charts.len(), 1);
    });
}

#[test]
fn divine_by_solar_time_encrypted_partial_works() {
    new_test_ext().execute_with(|| {
        use frame_support::BoundedVec;

        // 模拟加密数据
        let encrypted_data: BoundedVec<u8, frame_support::traits::ConstU32<512>> =
            vec![1, 2, 3, 4, 5].try_into().unwrap();
        let data_hash = [1u8; 32];
        let owner_key_backup = [2u8; 80];

        // Partial 模式（encryption_level = 1）
        assert_ok!(Qimen::divine_by_solar_time_encrypted(
            RuntimeOrigin::signed(ALICE),
            1,      // encryption_level: Partial
            2024,   // solar_year
            3,      // solar_month
            20,     // solar_day
            14,     // hour
            [0u8; 32], // question_hash
            Some(encrypted_data),
            Some(data_hash),
            Some(owner_key_backup),
            Some(1), // question_type: Career
            0,      // pan_method
        ));

        // 验证排盘记录已创建
        let chart = Qimen::charts(0).unwrap();
        assert_eq!(chart.diviner, ALICE);
        assert_eq!(chart.privacy_mode, PrivacyMode::Partial);
        assert!(chart.has_calculation_data()); // Partial 模式仍有计算数据
        assert!(chart.can_interpret());
        assert!(!chart.is_public());

        // 验证加密数据已存储
        assert!(Qimen::encrypted_data(0).is_some());

        // 验证密钥备份已存储
        assert_eq!(Qimen::owner_key_backup(0), Some([2u8; 80]));

        // 验证不在公开列表中
        assert_eq!(Qimen::public_charts().len(), 0);
    });
}

#[test]
fn divine_by_solar_time_encrypted_private_works() {
    new_test_ext().execute_with(|| {
        use frame_support::BoundedVec;

        // 模拟加密数据
        let encrypted_data: BoundedVec<u8, frame_support::traits::ConstU32<512>> =
            vec![10, 20, 30, 40, 50].try_into().unwrap();
        let data_hash = [3u8; 32];
        let owner_key_backup = [4u8; 80];

        // Private 模式（encryption_level = 2）
        assert_ok!(Qimen::divine_by_solar_time_encrypted(
            RuntimeOrigin::signed(BOB),
            2,      // encryption_level: Private
            2024,   // solar_year
            6,      // solar_month
            15,     // solar_day
            8,      // hour
            [0u8; 32], // question_hash
            Some(encrypted_data),
            Some(data_hash),
            Some(owner_key_backup),
            None,   // question_type
            1,      // pan_method: FeiPan
        ));

        // 验证排盘记录已创建
        let chart = Qimen::charts(0).unwrap();
        assert_eq!(chart.diviner, BOB);
        assert_eq!(chart.privacy_mode, PrivacyMode::Private);
        assert!(!chart.has_calculation_data()); // Private 模式无计算数据
        assert!(!chart.can_interpret()); // 无法直接解读

        // 验证加密数据已存储
        assert!(Qimen::encrypted_data(0).is_some());
        assert!(Qimen::owner_key_backup(0).is_some());
    });
}

#[test]
fn divine_by_solar_time_encrypted_invalid_level_fails() {
    new_test_ext().execute_with(|| {
        // 无效的加密级别（3）
        assert_noop!(
            Qimen::divine_by_solar_time_encrypted(
                RuntimeOrigin::signed(ALICE),
                3,      // 无效加密级别
                2024,
                1,
                15,
                10,
                [0u8; 32],
                None,
                None,
                None,
                None,
                0,
            ),
            Error::<Test>::InvalidEncryptionLevel
        );
    });
}

#[test]
fn divine_by_solar_time_encrypted_missing_data_fails() {
    new_test_ext().execute_with(|| {
        // Partial 模式缺少加密数据
        assert_noop!(
            Qimen::divine_by_solar_time_encrypted(
                RuntimeOrigin::signed(ALICE),
                1,      // Partial 模式
                2024,
                1,
                15,
                10,
                [0u8; 32],
                None,   // 缺少 encrypted_data
                Some([0u8; 32]),
                Some([0u8; 80]),
                None,
                0,
            ),
            Error::<Test>::EncryptedDataMissing
        );

        // Partial 模式缺少数据哈希
        assert_noop!(
            Qimen::divine_by_solar_time_encrypted(
                RuntimeOrigin::signed(ALICE),
                1,
                2024,
                1,
                15,
                10,
                [0u8; 32],
                Some(vec![1, 2, 3].try_into().unwrap()),
                None,   // 缺少 data_hash
                Some([0u8; 80]),
                None,
                0,
            ),
            Error::<Test>::DataHashMissing
        );

        // Partial 模式缺少密钥备份
        assert_noop!(
            Qimen::divine_by_solar_time_encrypted(
                RuntimeOrigin::signed(ALICE),
                1,
                2024,
                1,
                15,
                10,
                [0u8; 32],
                Some(vec![1, 2, 3].try_into().unwrap()),
                Some([0u8; 32]),
                None,   // 缺少 owner_key_backup
                None,
                0,
            ),
            Error::<Test>::OwnerKeyBackupMissing
        );
    });
}

#[test]
fn update_encrypted_data_works() {
    new_test_ext().execute_with(|| {
        use frame_support::BoundedVec;

        // 先创建一个 Partial 模式的排盘
        let initial_data: BoundedVec<u8, frame_support::traits::ConstU32<512>> =
            vec![1, 2, 3].try_into().unwrap();

        assert_ok!(Qimen::divine_by_solar_time_encrypted(
            RuntimeOrigin::signed(ALICE),
            1,
            2024, 1, 15, 10,
            [0u8; 32],
            Some(initial_data),
            Some([1u8; 32]),
            Some([1u8; 80]),
            None,
            0,
        ));

        // 更新加密数据
        let new_data: BoundedVec<u8, frame_support::traits::ConstU32<512>> =
            vec![4, 5, 6, 7, 8].try_into().unwrap();
        let new_hash = [2u8; 32];
        let new_key_backup = [3u8; 80];

        assert_ok!(Qimen::update_encrypted_data(
            RuntimeOrigin::signed(ALICE),
            0,
            new_data.clone(),
            new_hash,
            new_key_backup,
        ));

        // 验证更新成功
        let chart = Qimen::charts(0).unwrap();
        assert_eq!(chart.sensitive_data_hash, Some(new_hash));
        assert_eq!(Qimen::encrypted_data(0), Some(new_data));
        assert_eq!(Qimen::owner_key_backup(0), Some(new_key_backup));

        // 验证事件
        System::assert_has_event(RuntimeEvent::Qimen(Event::EncryptedDataUpdated {
            chart_id: 0,
            data_hash: new_hash,
        }));
    });
}

#[test]
fn update_encrypted_data_not_owner_fails() {
    new_test_ext().execute_with(|| {
        use frame_support::BoundedVec;

        // Alice 创建排盘
        let initial_data: BoundedVec<u8, frame_support::traits::ConstU32<512>> =
            vec![1, 2, 3].try_into().unwrap();

        assert_ok!(Qimen::divine_by_solar_time_encrypted(
            RuntimeOrigin::signed(ALICE),
            1,
            2024, 1, 15, 10,
            [0u8; 32],
            Some(initial_data),
            Some([1u8; 32]),
            Some([1u8; 80]),
            None,
            0,
        ));

        // Bob 尝试更新（应该失败）
        let new_data: BoundedVec<u8, frame_support::traits::ConstU32<512>> =
            vec![4, 5, 6].try_into().unwrap();

        assert_noop!(
            Qimen::update_encrypted_data(
                RuntimeOrigin::signed(BOB),
                0,
                new_data,
                [0u8; 32],
                [0u8; 80],
            ),
            Error::<Test>::NotOwner
        );
    });
}

#[test]
fn update_encrypted_data_public_chart_fails() {
    new_test_ext().execute_with(|| {
        use frame_support::BoundedVec;

        // 创建 Public 模式的排盘
        assert_ok!(Qimen::divine_by_solar_time_encrypted(
            RuntimeOrigin::signed(ALICE),
            0,  // Public
            2024, 1, 15, 10,
            [0u8; 32],
            None,
            None,
            None,
            None,
            0,
        ));

        // 尝试更新加密数据（Public 模式不允许）
        let new_data: BoundedVec<u8, frame_support::traits::ConstU32<512>> =
            vec![1, 2, 3].try_into().unwrap();

        assert_noop!(
            Qimen::update_encrypted_data(
                RuntimeOrigin::signed(ALICE),
                0,
                new_data,
                [0u8; 32],
                [0u8; 80],
            ),
            Error::<Test>::InvalidEncryptionLevel
        );
    });
}
