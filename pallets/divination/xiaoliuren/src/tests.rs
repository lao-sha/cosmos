//! 小六壬 Pallet 单元测试
//!
//! 测试小六壬排盘系统的所有核心功能

use crate::{mock::*, types::*, Error, Event, Pans, UserPans, PublicPans};
use frame_support::{assert_noop, assert_ok, BoundedVec};

// ============================================================================
// 时间起课测试
// ============================================================================

#[test]
fn test_divine_by_time_works() {
    new_test_ext().execute_with(|| {
        // 六月初五辰时（7点）
        assert_ok!(XiaoLiuRen::divine_by_time(
            RuntimeOrigin::signed(1),
            6,  // 农历六月
            5,  // 初五
            7,  // 7点 = 辰时
            None,
            false,
        ));

        // 验证课盘创建
        let pan = Pans::<Test>::get(0).expect("Pan should exist");
        assert_eq!(pan.creator, 1);
        assert_eq!(pan.method, DivinationMethod::TimeMethod);
        assert_eq!(pan.lunar_month, Some(6));
        assert_eq!(pan.lunar_day, Some(5));
        assert_eq!(pan.shi_chen, Some(ShiChen::Chen));

        // 验证三宫计算结果
        // 月宫：从大安起正月，6月 = (6-1) % 6 = 5 → 空亡
        let san_gong = pan.san_gong.expect("Public mode should have san_gong");
        assert_eq!(san_gong.yue_gong, LiuGong::KongWang);
        // 日宫：从空亡起初一，初五 = (5 + 5 - 1) % 6 = 3 → 赤口
        assert_eq!(san_gong.ri_gong, LiuGong::ChiKou);
        // 时宫：从赤口起子时，辰时(5) = (3 + 5 - 1) % 6 = 1 → 留连
        assert_eq!(san_gong.shi_gong, LiuGong::LiuLian);

        // 验证用户课盘索引
        let user_pans = UserPans::<Test>::get(1);
        assert_eq!(user_pans.len(), 1);
        assert_eq!(user_pans[0], 0);

        // 验证事件
        System::assert_has_event(RuntimeEvent::XiaoLiuRen(Event::PanCreated {
            pan_id: 0,
            creator: 1,
            method: DivinationMethod::TimeMethod,
        }));
    });
}

#[test]
fn test_divine_by_time_invalid_params() {
    new_test_ext().execute_with(|| {
        // 无效月份
        assert_noop!(
            XiaoLiuRen::divine_by_time(
                RuntimeOrigin::signed(1),
                0,  // 无效月份
                5,
                7,
                None,
                false,
            ),
            Error::<Test>::InvalidLunarMonth
        );

        assert_noop!(
            XiaoLiuRen::divine_by_time(
                RuntimeOrigin::signed(1),
                13,  // 无效月份
                5,
                7,
                None,
                false,
            ),
            Error::<Test>::InvalidLunarMonth
        );

        // 无效日期
        assert_noop!(
            XiaoLiuRen::divine_by_time(
                RuntimeOrigin::signed(1),
                6,
                0,  // 无效日期
                7,
                None,
                false,
            ),
            Error::<Test>::InvalidLunarDay
        );

        assert_noop!(
            XiaoLiuRen::divine_by_time(
                RuntimeOrigin::signed(1),
                6,
                31,  // 无效日期
                7,
                None,
                false,
            ),
            Error::<Test>::InvalidLunarDay
        );

        // 无效小时
        assert_noop!(
            XiaoLiuRen::divine_by_time(
                RuntimeOrigin::signed(1),
                6,
                5,
                24,  // 无效小时
                None,
                false,
            ),
            Error::<Test>::InvalidHour
        );
    });
}

// ============================================================================
// 数字起课测试
// ============================================================================

#[test]
fn test_divine_by_number_works() {
    new_test_ext().execute_with(|| {
        // 使用数字 1, 2, 3 起课
        assert_ok!(XiaoLiuRen::divine_by_number(
            RuntimeOrigin::signed(1),
            1,
            2,
            3,
            None,
            false,
        ));

        let pan = Pans::<Test>::get(0).expect("Pan should exist");
        assert_eq!(pan.method, DivinationMethod::NumberMethod);
        assert_eq!(pan.param1, Some(1));
        assert_eq!(pan.param2, Some(2));
        assert_eq!(pan.param3, Some(3));

        // 验证三宫计算结果
        let san_gong = pan.san_gong.expect("Public mode should have san_gong");
        // 月宫 = (1-1) % 6 = 0 → 大安
        assert_eq!(san_gong.yue_gong, LiuGong::DaAn);
        // 日宫 = (1+2-2) % 6 = 1 → 留连
        assert_eq!(san_gong.ri_gong, LiuGong::LiuLian);
        // 时宫 = (1+2+3-3) % 6 = 3 → 赤口
        assert_eq!(san_gong.shi_gong, LiuGong::ChiKou);
    });
}

#[test]
fn test_divine_by_number_wrap_around() {
    new_test_ext().execute_with(|| {
        // 测试取模边界
        assert_ok!(XiaoLiuRen::divine_by_number(
            RuntimeOrigin::signed(1),
            6,
            6,
            6,
            None,
            false,
        ));

        let pan = Pans::<Test>::get(0).expect("Pan should exist");
        let san_gong = pan.san_gong.expect("Public mode should have san_gong");

        // 月宫 = (6-1) % 6 = 5 → 空亡
        assert_eq!(san_gong.yue_gong, LiuGong::KongWang);
        // 日宫 = (6+6-2) % 6 = 10 % 6 = 4 → 小吉
        assert_eq!(san_gong.ri_gong, LiuGong::XiaoJi);
        // 时宫 = (6+6+6-3) % 6 = 15 % 6 = 3 → 赤口
        assert_eq!(san_gong.shi_gong, LiuGong::ChiKou);
    });
}

#[test]
fn test_divine_by_number_invalid_params() {
    new_test_ext().execute_with(|| {
        // 数字必须大于 0
        assert_noop!(
            XiaoLiuRen::divine_by_number(
                RuntimeOrigin::signed(1),
                0,  // 无效
                2,
                3,
                None,
                false,
            ),
            Error::<Test>::NumberMustBePositive
        );
    });
}

// ============================================================================
// 随机起课测试
// ============================================================================

#[test]
fn test_divine_random_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(XiaoLiuRen::divine_random(
            RuntimeOrigin::signed(1),
            None,
            false,
        ));

        let pan = Pans::<Test>::get(0).expect("Pan should exist");
        assert_eq!(pan.method, DivinationMethod::RandomMethod);

        // 验证三宫都有效（索引在 0-5 范围内）
        let san_gong = pan.san_gong.expect("Public mode should have san_gong");
        assert!(san_gong.yue_gong.index() < 6);
        assert!(san_gong.ri_gong.index() < 6);
        assert!(san_gong.shi_gong.index() < 6);
    });
}

// ============================================================================
// 手动指定测试
// ============================================================================

#[test]
fn test_divine_manual_works() {
    new_test_ext().execute_with(|| {
        // 手动指定三宫：大安、速喜、小吉
        assert_ok!(XiaoLiuRen::divine_manual(
            RuntimeOrigin::signed(1),
            0,  // 大安
            2,  // 速喜
            4,  // 小吉
            None,
            false,
        ));

        let pan = Pans::<Test>::get(0).expect("Pan should exist");
        assert_eq!(pan.method, DivinationMethod::ManualMethod);
        let san_gong = pan.san_gong.expect("Public mode should have san_gong");
        assert_eq!(san_gong.yue_gong, LiuGong::DaAn);
        assert_eq!(san_gong.ri_gong, LiuGong::SuXi);
        assert_eq!(san_gong.shi_gong, LiuGong::XiaoJi);
    });
}

#[test]
fn test_divine_manual_invalid_params() {
    new_test_ext().execute_with(|| {
        // 索引超出范围
        assert_noop!(
            XiaoLiuRen::divine_manual(
                RuntimeOrigin::signed(1),
                6,  // 无效索引
                0,
                0,
                None,
                false,
            ),
            Error::<Test>::InvalidParams
        );
    });
}

// ============================================================================
// 公开状态测试
// ============================================================================

#[test]
fn test_public_pan_works() {
    new_test_ext().execute_with(|| {
        // 创建公开课盘
        assert_ok!(XiaoLiuRen::divine_by_number(
            RuntimeOrigin::signed(1),
            1, 2, 3,
            None,
            true,  // 公开
        ));

        // 验证公开列表
        let public_pans = PublicPans::<Test>::get();
        assert_eq!(public_pans.len(), 1);
        assert_eq!(public_pans[0], 0);

        // 验证课盘状态
        let pan = Pans::<Test>::get(0).unwrap();
        assert!(pan.is_public());
    });
}

#[test]
fn test_set_pan_visibility_works() {
    new_test_ext().execute_with(|| {
        // 创建私有课盘
        assert_ok!(XiaoLiuRen::divine_by_number(
            RuntimeOrigin::signed(1),
            1, 2, 3,
            None,
            false,
        ));

        // 设为公开
        assert_ok!(XiaoLiuRen::set_pan_visibility(
            RuntimeOrigin::signed(1),
            0,
            true,
        ));

        let pan = Pans::<Test>::get(0).unwrap();
        assert!(pan.is_public());
        assert_eq!(PublicPans::<Test>::get().len(), 1);

        // 设为私有
        assert_ok!(XiaoLiuRen::set_pan_visibility(
            RuntimeOrigin::signed(1),
            0,
            false,
        ));

        let pan = Pans::<Test>::get(0).unwrap();
        assert!(!pan.is_public());
        assert_eq!(PublicPans::<Test>::get().len(), 0);
    });
}

#[test]
fn test_set_visibility_not_owner() {
    new_test_ext().execute_with(|| {
        // 账户 1 创建课盘
        assert_ok!(XiaoLiuRen::divine_by_number(
            RuntimeOrigin::signed(1),
            1, 2, 3,
            None,
            false,
        ));

        // 账户 2 尝试修改
        assert_noop!(
            XiaoLiuRen::set_pan_visibility(
                RuntimeOrigin::signed(2),
                0,
                true,
            ),
            Error::<Test>::NotOwner
        );
    });
}

// ============================================================================
// AI 解读测试
// ============================================================================

#[test]
#[allow(deprecated)]
fn test_request_ai_interpretation_works() {
    new_test_ext().execute_with(|| {
        // 创建课盘
        assert_ok!(XiaoLiuRen::divine_by_number(
            RuntimeOrigin::signed(1),
            1, 2, 3,
            None,
            false,
        ));

        // 请求 AI 解读
        assert_ok!(XiaoLiuRen::request_ai_interpretation(
            RuntimeOrigin::signed(1),
            0,
        ));

        // 验证请求已记录
        assert!(crate::AiInterpretationRequests::<Test>::contains_key(0));

        // 验证事件
        System::assert_has_event(RuntimeEvent::XiaoLiuRen(Event::AiInterpretationRequested {
            pan_id: 0,
            requester: 1,
        }));
    });
}

#[test]
#[allow(deprecated)]
fn test_submit_ai_interpretation_works() {
    new_test_ext().execute_with(|| {
        // 创建课盘
        assert_ok!(XiaoLiuRen::divine_by_number(
            RuntimeOrigin::signed(1),
            1, 2, 3,
            None,
            false,
        ));

        // 请求 AI 解读
        assert_ok!(XiaoLiuRen::request_ai_interpretation(
            RuntimeOrigin::signed(1),
            0,
        ));

        // 模拟 AI 预言机提交结果（账户 1 有预言机权限）
        let cid: BoundedVec<u8, MaxCidLen> =
            BoundedVec::try_from(b"QmTest123".to_vec()).unwrap();

        assert_ok!(XiaoLiuRen::submit_ai_interpretation(
            RuntimeOrigin::signed(1),
            0,
            cid.clone(),
        ));

        // 验证结果已存储
        let pan = Pans::<Test>::get(0).unwrap();
        assert_eq!(pan.ai_interpretation_cid, Some(cid.clone()));

        // 验证请求已移除
        assert!(!crate::AiInterpretationRequests::<Test>::contains_key(0));

        // 验证事件
        System::assert_has_event(RuntimeEvent::XiaoLiuRen(Event::AiInterpretationSubmitted {
            pan_id: 0,
            cid,
        }));
    });
}

#[test]
#[allow(deprecated)]
fn test_ai_interpretation_not_owner() {
    new_test_ext().execute_with(|| {
        // 账户 1 创建课盘
        assert_ok!(XiaoLiuRen::divine_by_number(
            RuntimeOrigin::signed(1),
            1, 2, 3,
            None,
            false,
        ));

        // 账户 2 尝试请求解读
        assert_noop!(
            XiaoLiuRen::request_ai_interpretation(
                RuntimeOrigin::signed(2),
                0,
            ),
            Error::<Test>::NotOwner
        );
    });
}

#[test]
#[allow(deprecated)]
fn test_ai_interpretation_duplicate_request() {
    new_test_ext().execute_with(|| {
        // 创建课盘并请求解读
        assert_ok!(XiaoLiuRen::divine_by_number(
            RuntimeOrigin::signed(1),
            1, 2, 3,
            None,
            false,
        ));

        assert_ok!(XiaoLiuRen::request_ai_interpretation(
            RuntimeOrigin::signed(1),
            0,
        ));

        // 重复请求
        assert_noop!(
            XiaoLiuRen::request_ai_interpretation(
                RuntimeOrigin::signed(1),
                0,
            ),
            Error::<Test>::AiRequestAlreadyExists
        );
    });
}

// ============================================================================
// 每日限制测试
// ============================================================================

#[test]
fn test_daily_limit_enforced() {
    new_test_ext().execute_with(|| {
        // 连续起课直到达到限制
        for _ in 0..50 {
            assert_ok!(XiaoLiuRen::divine_by_number(
                RuntimeOrigin::signed(1),
                1, 2, 3,
                None,
                false,
            ));
        }

        // 超过限制
        assert_noop!(
            XiaoLiuRen::divine_by_number(
                RuntimeOrigin::signed(1),
                1, 2, 3,
                None,
                false,
            ),
            Error::<Test>::DailyLimitExceeded
        );
    });
}

// ============================================================================
// 用户统计测试
// ============================================================================

#[test]
fn test_user_stats_updated() {
    new_test_ext().execute_with(|| {
        // 创建多个课盘
        assert_ok!(XiaoLiuRen::divine_by_number(
            RuntimeOrigin::signed(1),
            1, 2, 3,
            None,
            false,
        ));

        assert_ok!(XiaoLiuRen::divine_by_time(
            RuntimeOrigin::signed(1),
            6, 5, 7,
            None,
            false,
        ));

        // 验证统计
        let stats = crate::UserStatsStorage::<Test>::get(1);
        assert_eq!(stats.total_pans, 2);
        assert_eq!(stats.first_pan_block, 1);
    });
}

// ============================================================================
// 六宫属性测试
// ============================================================================

#[test]
fn test_liu_gong_properties() {
    // 大安
    let da_an = LiuGong::DaAn;
    assert_eq!(da_an.name(), "大安");
    assert_eq!(da_an.wu_xing(), WuXing::Wood);
    assert_eq!(da_an.tian_jiang(), "青龙");
    assert!(da_an.is_auspicious());
    assert_eq!(da_an.fortune_level(), 5);

    // 留连（道家流派：土）
    let liu_lian = LiuGong::LiuLian;
    assert_eq!(liu_lian.name(), "留连");
    assert_eq!(liu_lian.wu_xing(), WuXing::Earth); // 道家流派为土
    assert!(!liu_lian.is_auspicious());

    // 速喜
    let su_xi = LiuGong::SuXi;
    assert_eq!(su_xi.name(), "速喜");
    assert_eq!(su_xi.wu_xing(), WuXing::Fire);
    assert!(su_xi.is_auspicious());

    // 赤口
    let chi_kou = LiuGong::ChiKou;
    assert_eq!(chi_kou.name(), "赤口");
    assert_eq!(chi_kou.wu_xing(), WuXing::Metal);
    assert!(!chi_kou.is_auspicious());

    // 小吉（道家流派：水）
    let xiao_ji = LiuGong::XiaoJi;
    assert_eq!(xiao_ji.name(), "小吉");
    assert_eq!(xiao_ji.wu_xing(), WuXing::Water); // 道家流派为水
    assert!(xiao_ji.is_auspicious());

    // 空亡
    let kong_wang = LiuGong::KongWang;
    assert_eq!(kong_wang.name(), "空亡");
    assert_eq!(kong_wang.wu_xing(), WuXing::Earth);
    assert!(!kong_wang.is_auspicious());
}

// ============================================================================
// 三宫分析测试
// ============================================================================

#[test]
fn test_san_gong_all_auspicious() {
    let all_good = SanGong::new(LiuGong::DaAn, LiuGong::SuXi, LiuGong::XiaoJi);
    assert!(all_good.is_all_auspicious());
    assert!(!all_good.is_all_inauspicious());
    assert!(!all_good.is_pure());
}

#[test]
fn test_san_gong_all_inauspicious() {
    let all_bad = SanGong::new(LiuGong::LiuLian, LiuGong::ChiKou, LiuGong::KongWang);
    assert!(!all_bad.is_all_auspicious());
    assert!(all_bad.is_all_inauspicious());
    assert!(!all_bad.is_pure());
}

#[test]
fn test_san_gong_pure() {
    let pure = SanGong::new(LiuGong::DaAn, LiuGong::DaAn, LiuGong::DaAn);
    assert!(pure.is_pure());
    assert!(pure.is_all_auspicious());
}

// ============================================================================
// 时辰计算测试
// ============================================================================

#[test]
fn test_shi_chen_from_hour() {
    assert_eq!(ShiChen::from_hour(0), ShiChen::Zi);
    assert_eq!(ShiChen::from_hour(1), ShiChen::Chou);
    assert_eq!(ShiChen::from_hour(3), ShiChen::Yin);
    assert_eq!(ShiChen::from_hour(7), ShiChen::Chen);
    assert_eq!(ShiChen::from_hour(11), ShiChen::Wu);
    assert_eq!(ShiChen::from_hour(23), ShiChen::Zi);
}

#[test]
fn test_shi_chen_index() {
    // 时辰索引从 1 开始（用于计算）
    assert_eq!(ShiChen::Zi.index(), 1);
    assert_eq!(ShiChen::Chou.index(), 2);
    assert_eq!(ShiChen::Chen.index(), 5);
    assert_eq!(ShiChen::Hai.index(), 12);
}

// ============================================================================
// 五行关系测试
// ============================================================================

#[test]
fn test_wu_xing_relations() {
    // 木生火
    assert_eq!(WuXing::Wood.generates(), WuXing::Fire);
    // 火生土
    assert_eq!(WuXing::Fire.generates(), WuXing::Earth);
    // 土生金
    assert_eq!(WuXing::Earth.generates(), WuXing::Metal);
    // 金生水
    assert_eq!(WuXing::Metal.generates(), WuXing::Water);
    // 水生木
    assert_eq!(WuXing::Water.generates(), WuXing::Wood);

    // 木克土
    assert_eq!(WuXing::Wood.restrains(), WuXing::Earth);
    // 火克金
    assert_eq!(WuXing::Fire.restrains(), WuXing::Metal);
    // 土克水
    assert_eq!(WuXing::Earth.restrains(), WuXing::Water);
    // 金克木
    assert_eq!(WuXing::Metal.restrains(), WuXing::Wood);
    // 水克火
    assert_eq!(WuXing::Water.restrains(), WuXing::Fire);
}

// ============================================================================
// 新增功能测试
// ============================================================================

#[test]
fn test_divine_by_hour_ke_works() {
    new_test_ext().execute_with(|| {
        // 使用 14:36 (下午2:36) 进行时刻分起课
        assert_ok!(XiaoLiuRen::divine_by_hour_ke(
            RuntimeOrigin::signed(1),
            14,  // 14点 = 未时
            36,  // 36分
            None,
            false,
        ));

        let pan = Pans::<Test>::get(0).expect("Pan should exist");
        assert_eq!(pan.method, DivinationMethod::TimeKeMethod);
        assert_eq!(pan.shi_chen, Some(ShiChen::Wei));

        // 验证三宫已计算
        let san_gong = pan.san_gong.expect("Public mode should have san_gong");
        assert!(san_gong.yue_gong.index() < 6);
        assert!(san_gong.ri_gong.index() < 6);
        assert!(san_gong.shi_gong.index() < 6);
    });
}

#[test]
fn test_divine_by_hour_ke_invalid_params() {
    new_test_ext().execute_with(|| {
        // 无效小时
        assert_noop!(
            XiaoLiuRen::divine_by_hour_ke(
                RuntimeOrigin::signed(1),
                25,  // 无效小时
                30,
                None,
                false,
            ),
            Error::<Test>::InvalidHour
        );

        // 无效分钟
        assert_noop!(
            XiaoLiuRen::divine_by_hour_ke(
                RuntimeOrigin::signed(1),
                14,
                60,  // 无效分钟
                None,
                false,
            ),
            Error::<Test>::InvalidParams
        );
    });
}

#[test]
fn test_divine_by_digits_works() {
    new_test_ext().execute_with(|| {
        // 使用数字 1436 起课
        assert_ok!(XiaoLiuRen::divine_by_digits(
            RuntimeOrigin::signed(1),
            1436,
            None,
            false,
        ));

        let pan = Pans::<Test>::get(0).expect("Pan should exist");
        assert_eq!(pan.method, DivinationMethod::NumberMethod);

        // 验证三宫已计算（对于纯数字起课，三宫相同）
        let san_gong = pan.san_gong.expect("Public mode should have san_gong");
        assert!(san_gong.yue_gong.index() < 6);
    });
}

#[test]
fn test_divine_by_three_numbers_works() {
    new_test_ext().execute_with(|| {
        // 使用三个数字起课
        assert_ok!(XiaoLiuRen::divine_by_three_numbers(
            RuntimeOrigin::signed(1),
            7,
            14,
            21,
            None,
            false,
        ));

        let pan = Pans::<Test>::get(0).expect("Pan should exist");
        assert_eq!(pan.method, DivinationMethod::NumberMethod);

        // 验证三宫计算
        let san_gong = pan.san_gong.expect("Public mode should have san_gong");
        // 月宫 = (7-1) % 6 = 0 → 大安
        assert_eq!(san_gong.yue_gong, LiuGong::DaAn);
        // 日宫 = (0 + 14 - 1) % 6 = 13 % 6 = 1 → 留连
        assert_eq!(san_gong.ri_gong, LiuGong::LiuLian);
        // 时宫 = (1 + 21 - 1) % 6 = 21 % 6 = 3 → 赤口
        assert_eq!(san_gong.shi_gong, LiuGong::ChiKou);
    });
}

// ============================================================================
// 多流派支持测试
// ============================================================================

#[test]
fn test_liu_gong_school_wuxing() {
    // 测试道家流派五行
    assert_eq!(LiuGong::LiuLian.wu_xing(), WuXing::Earth); // 道家：土
    assert_eq!(LiuGong::XiaoJi.wu_xing(), WuXing::Water);  // 道家：水

    // 测试传统流派五行
    assert_eq!(LiuGong::LiuLian.wu_xing_traditional(), WuXing::Water); // 传统：水
    assert_eq!(LiuGong::XiaoJi.wu_xing_traditional(), WuXing::Wood);   // 传统：木

    // 测试按流派获取
    assert_eq!(
        LiuGong::LiuLian.wu_xing_by_school(XiaoLiuRenSchool::DaoJia),
        WuXing::Earth
    );
    assert_eq!(
        LiuGong::LiuLian.wu_xing_by_school(XiaoLiuRenSchool::ChuanTong),
        WuXing::Water
    );
}

// ============================================================================
// 十二宫对应测试
// ============================================================================

#[test]
fn test_twelve_palace_mapping() {
    // 大安对应事业宫（外）+ 命宫（内）
    let da_an_palace = LiuGong::DaAn.twelve_palace();
    assert_eq!(da_an_palace.outer, TwelvePalace::ShiYeGong);
    assert_eq!(da_an_palace.inner, TwelvePalace::MingGong);

    // 留连对应田宅宫（外）+ 奴仆宫（内）
    let liu_lian_palace = LiuGong::LiuLian.twelve_palace();
    assert_eq!(liu_lian_palace.outer, TwelvePalace::TianZhaiGong);
    assert_eq!(liu_lian_palace.inner, TwelvePalace::NuPuGong);

    // 速喜对应感情宫（外）+ 夫妻宫（内）
    let su_xi_palace = LiuGong::SuXi.twelve_palace();
    assert_eq!(su_xi_palace.outer, TwelvePalace::GanQingGong);
    assert_eq!(su_xi_palace.inner, TwelvePalace::FuQiGong);
}

// ============================================================================
// 藏干测试
// ============================================================================

#[test]
fn test_hidden_stems() {
    assert_eq!(LiuGong::DaAn.hidden_stems(), ("甲", "丁"));
    assert_eq!(LiuGong::LiuLian.hidden_stems(), ("丁", "己"));
    assert_eq!(LiuGong::SuXi.hidden_stems(), ("丙", "辛"));
    assert_eq!(LiuGong::ChiKou.hidden_stems(), ("庚", "癸"));
    assert_eq!(LiuGong::XiaoJi.hidden_stems(), ("壬", "甲"));
    assert_eq!(LiuGong::KongWang.hidden_stems(), ("戊", "乙"));
}

// ============================================================================
// 早子时/晚子时测试
// ============================================================================

#[test]
fn test_zi_shi_type() {
    // 23点为早子时
    let (shi_chen, zi_type) = ShiChen::from_hour_detailed(23);
    assert_eq!(shi_chen, ShiChen::Zi);
    assert_eq!(zi_type, Some(ZiShiType::EarlyZi));

    // 0点为晚子时
    let (shi_chen, zi_type) = ShiChen::from_hour_detailed(0);
    assert_eq!(shi_chen, ShiChen::Zi);
    assert_eq!(zi_type, Some(ZiShiType::LateZi));

    // 其他时辰没有子时类型
    let (shi_chen, zi_type) = ShiChen::from_hour_detailed(7);
    assert_eq!(shi_chen, ShiChen::Chen);
    assert_eq!(zi_type, None);
}

// ============================================================================
// 体用关系测试
// ============================================================================

#[test]
fn test_ti_yong_relation() {
    // 用生体测试：人宫为木，时辰为水
    // 水生木 = 用生体（大吉）
    let relation = TiYongRelation::calculate(LiuGong::DaAn, ShiChen::Zi);
    // 大安属木，子时属水，水生木 = 用生体
    assert_eq!(relation, TiYongRelation::YongShengTi);
    assert_eq!(relation.fortune_desc(), "大吉");

    // 体克用测试：人宫为木，时辰为土
    // 木克土 = 体克用（小吉）
    let relation = TiYongRelation::calculate(LiuGong::DaAn, ShiChen::Chou);
    // 大安属木，丑时属土，木克土 = 体克用
    assert_eq!(relation, TiYongRelation::TiKeYong);
    assert_eq!(relation.fortune_desc(), "小吉");
}

// ============================================================================
// 八卦具象法测试
// ============================================================================

#[test]
fn test_ba_gua_from_san_gong() {
    // 阳阳阳 = 乾
    let san_gong = SanGong::new(LiuGong::DaAn, LiuGong::SuXi, LiuGong::XiaoJi);
    let ba_gua = BaGua::from_san_gong(&san_gong);
    assert_eq!(ba_gua, BaGua::Qian);

    // 阴阴阴 = 坤
    let san_gong = SanGong::new(LiuGong::LiuLian, LiuGong::ChiKou, LiuGong::KongWang);
    let ba_gua = BaGua::from_san_gong(&san_gong);
    assert_eq!(ba_gua, BaGua::Kun);
}

// ============================================================================
// 六神扩展属性测试
// ============================================================================

#[test]
fn test_liu_gong_extended_properties() {
    // 测试对应季节
    assert_eq!(LiuGong::DaAn.season(), "春季");
    assert_eq!(LiuGong::SuXi.season(), "夏季");
    assert_eq!(LiuGong::ChiKou.season(), "秋季");
    assert_eq!(LiuGong::XiaoJi.season(), "冬季");

    // 测试对应天干
    assert_eq!(LiuGong::DaAn.tian_gan(), "甲乙");
    assert_eq!(LiuGong::SuXi.tian_gan(), "丙丁");
    assert_eq!(LiuGong::ChiKou.tian_gan(), "庚辛");
    assert_eq!(LiuGong::XiaoJi.tian_gan(), "壬癸");

    // 测试数字范围
    assert_eq!(LiuGong::DaAn.number_range(), [1, 7, 4, 5]);
    assert_eq!(LiuGong::SuXi.number_range(), [3, 9, 6, 9]);
}

// ============================================================================
// 解卦集成测试
// ============================================================================

#[test]
fn test_interpretation_lazy_loading() {
    new_test_ext().execute_with(|| {
        // 1. 创建课盘
        assert_ok!(XiaoLiuRen::divine_by_time(
            RuntimeOrigin::signed(1),
            6,  // 农历六月
            5,  // 初五
            7,  // 7点 = 辰时
            None,
            false,
        ));

        // 2. 首次获取解卦（应该计算并缓存）
        let interpretation = XiaoLiuRen::get_or_create_interpretation(0);
        assert!(interpretation.is_some());

        let interp = interpretation.unwrap();
        // 验证解卦结果
        assert!(interp.overall_score > 0);
        assert!(interp.overall_score <= 100);
        assert!(interp.ying_qi.is_some());

        // 3. 再次获取（应该从缓存读取）
        let cached = XiaoLiuRen::get_or_create_interpretation(0);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().overall_score, interp.overall_score);

        // 4. 验证缓存存储
        let stored = crate::Interpretations::<Test>::get(0);
        assert!(stored.is_some());
    });
}

#[test]
fn test_interpretation_batch() {
    new_test_ext().execute_with(|| {
        // 创建多个课盘
        for i in 0..3 {
            assert_ok!(XiaoLiuRen::divine_by_number(
                RuntimeOrigin::signed(1),
                (i + 1) as u8,
                (i + 2) as u8,
                (i + 3) as u8,
                None,
                false,
            ));
        }

        // 批量获取解卦
        let results = XiaoLiuRen::get_interpretations_batch(vec![0, 1, 2, 999]);

        assert_eq!(results.len(), 4);
        assert!(results[0].is_some());
        assert!(results[1].is_some());
        assert!(results[2].is_some());
        assert!(results[3].is_none()); // 不存在的课盘
    });
}

#[test]
fn test_interpretation_all_liu_gong() {
    use crate::interpretation::interpret;

    // 测试所有六宫组合的解卦
    let liu_gong_list = [
        LiuGong::DaAn,
        LiuGong::LiuLian,
        LiuGong::SuXi,
        LiuGong::ChiKou,
        LiuGong::XiaoJi,
        LiuGong::KongWang,
    ];

    for &yue in &liu_gong_list {
        for &ri in &liu_gong_list {
            for &shi in &liu_gong_list {
                let san_gong = SanGong::new(yue, ri, shi);
                let interp = interpret(&san_gong, None, XiaoLiuRenSchool::DaoJia);

                // 验证基本属性
                assert!(interp.overall_score <= 100);
                assert!(interp.ji_xiong_score() >= 1 && interp.ji_xiong_score() <= 7);
                assert!(interp.ying_qi.is_some());
            }
        }
    }
}

#[test]
fn test_interpretation_special_patterns() {
    use crate::interpretation::interpret;

    // 纯宫全吉
    let pure_good = SanGong::new(LiuGong::DaAn, LiuGong::DaAn, LiuGong::DaAn);
    let interp = interpret(&pure_good, None, XiaoLiuRenSchool::DaoJia);
    assert!(interp.special_pattern.is_pure());
    assert!(interp.is_ji());

    // 纯宫全凶
    let pure_bad = SanGong::new(LiuGong::KongWang, LiuGong::KongWang, LiuGong::KongWang);
    let interp = interpret(&pure_bad, None, XiaoLiuRenSchool::DaoJia);
    assert!(interp.special_pattern.is_pure());
    assert!(interp.is_xiong());

    // 全吉
    let all_good = SanGong::new(LiuGong::DaAn, LiuGong::SuXi, LiuGong::XiaoJi);
    let interp = interpret(&all_good, None, XiaoLiuRenSchool::DaoJia);
    assert!(interp.special_pattern.is_all_auspicious());
    assert!(interp.is_ji());

    // 全凶
    let all_bad = SanGong::new(LiuGong::LiuLian, LiuGong::ChiKou, LiuGong::KongWang);
    let interp = interpret(&all_bad, None, XiaoLiuRenSchool::DaoJia);
    assert!(interp.special_pattern.is_all_inauspicious());
    assert!(interp.is_xiong());
}

// ============================================================================
// 加密模式测试
// ============================================================================

#[test]
fn test_divine_by_time_encrypted_public() {
    new_test_ext().execute_with(|| {
        use pallet_divination_privacy::types::PrivacyMode;

        // Public 模式起课
        assert_ok!(XiaoLiuRen::divine_by_time_encrypted(
            RuntimeOrigin::signed(1),
            0, // Public 模式
            Some(6),  // 农历六月
            Some(5),  // 初五
            Some(7),  // 7点 = 辰时
            None, // 无问题 CID
            None, // 无加密数据
            None, // 无数据哈希
            None, // 无密钥备份
        ));

        // 验证课盘
        let pan = Pans::<Test>::get(0).unwrap();
        assert_eq!(pan.privacy_mode, PrivacyMode::Public);
        assert!(pan.san_gong.is_some());
        assert!(pan.is_public());
        assert!(pan.can_interpret());
    });
}

#[test]
fn test_divine_by_time_encrypted_partial() {
    new_test_ext().execute_with(|| {
        use pallet_divination_privacy::types::PrivacyMode;

        // Partial 模式起课
        let encrypted_data: BoundedVec<u8, crate::mock::MaxEncryptedLen> =
            BoundedVec::try_from(vec![1, 2, 3, 4, 5]).unwrap();
        let data_hash = [0u8; 32];
        let owner_key_backup = [0u8; 80];

        assert_ok!(XiaoLiuRen::divine_by_time_encrypted(
            RuntimeOrigin::signed(1),
            1, // Partial 模式
            Some(6),
            Some(5),
            Some(7),
            None,
            Some(encrypted_data.clone()),
            Some(data_hash),
            Some(owner_key_backup),
        ));

        // 验证课盘
        let pan = Pans::<Test>::get(0).unwrap();
        assert_eq!(pan.privacy_mode, PrivacyMode::Partial);
        assert!(pan.san_gong.is_some()); // 计算数据仍然存在
        assert!(pan.can_interpret());
        assert!(!pan.is_public());

        // 验证加密数据存储
        assert!(crate::EncryptedDataStorage::<Test>::contains_key(0));
        assert!(crate::OwnerKeyBackupStorage::<Test>::contains_key(0));
    });
}

#[test]
fn test_divine_by_time_encrypted_private() {
    new_test_ext().execute_with(|| {
        use pallet_divination_privacy::types::PrivacyMode;

        // Private 模式起课
        let encrypted_data: BoundedVec<u8, crate::mock::MaxEncryptedLen> =
            BoundedVec::try_from(vec![5, 6, 7, 8]).unwrap();
        let data_hash = [1u8; 32];
        let owner_key_backup = [2u8; 80];

        assert_ok!(XiaoLiuRen::divine_by_time_encrypted(
            RuntimeOrigin::signed(1),
            2, // Private 模式
            None, // 无明文数据
            None,
            None,
            None,
            Some(encrypted_data.clone()),
            Some(data_hash),
            Some(owner_key_backup),
        ));

        // 验证课盘
        let pan = Pans::<Test>::get(0).unwrap();
        assert_eq!(pan.privacy_mode, PrivacyMode::Private);
        assert!(pan.san_gong.is_none()); // Private 模式无计算数据
        assert!(!pan.can_interpret()); // 无法解读
        assert!(pan.is_private());

        // 验证加密数据存储
        assert!(crate::EncryptedDataStorage::<Test>::contains_key(0));
        assert!(crate::OwnerKeyBackupStorage::<Test>::contains_key(0));
    });
}

#[test]
fn test_update_encrypted_data_works() {
    new_test_ext().execute_with(|| {
        // 先创建一个 Partial 模式的课盘
        let encrypted_data: BoundedVec<u8, crate::mock::MaxEncryptedLen> =
            BoundedVec::try_from(vec![1, 2, 3]).unwrap();

        assert_ok!(XiaoLiuRen::divine_by_time_encrypted(
            RuntimeOrigin::signed(1),
            1,
            Some(6),
            Some(5),
            Some(7),
            None,
            Some(encrypted_data),
            Some([0u8; 32]),
            Some([0u8; 80]),
        ));

        // 更新加密数据
        let new_encrypted_data: BoundedVec<u8, crate::mock::MaxEncryptedLen> =
            BoundedVec::try_from(vec![9, 8, 7, 6, 5]).unwrap();
        let new_data_hash = [3u8; 32];
        let new_owner_key_backup = [4u8; 80];

        assert_ok!(XiaoLiuRen::update_encrypted_data(
            RuntimeOrigin::signed(1),
            0,
            new_encrypted_data.clone(),
            new_data_hash,
            new_owner_key_backup,
        ));

        // 验证更新
        let pan = Pans::<Test>::get(0).unwrap();
        assert_eq!(pan.sensitive_data_hash, Some(new_data_hash));

        let stored_data = crate::EncryptedDataStorage::<Test>::get(0).unwrap();
        assert_eq!(stored_data.to_vec(), vec![9, 8, 7, 6, 5]);
    });
}

#[test]
fn test_update_encrypted_data_not_owner() {
    new_test_ext().execute_with(|| {
        // 账户 1 创建课盘
        let encrypted_data: BoundedVec<u8, crate::mock::MaxEncryptedLen> =
            BoundedVec::try_from(vec![1, 2, 3]).unwrap();

        assert_ok!(XiaoLiuRen::divine_by_time_encrypted(
            RuntimeOrigin::signed(1),
            1,
            Some(6),
            Some(5),
            Some(7),
            None,
            Some(encrypted_data),
            Some([0u8; 32]),
            Some([0u8; 80]),
        ));

        // 账户 2 尝试更新
        let new_encrypted_data: BoundedVec<u8, crate::mock::MaxEncryptedLen> =
            BoundedVec::try_from(vec![9, 9, 9]).unwrap();

        assert_noop!(
            XiaoLiuRen::update_encrypted_data(
                RuntimeOrigin::signed(2),
                0,
                new_encrypted_data,
                [1u8; 32],
                [1u8; 80],
            ),
            Error::<Test>::NotOwner
        );
    });
}

#[test]
fn test_privacy_mode_affects_visibility() {
    new_test_ext().execute_with(|| {
        use pallet_divination_privacy::types::PrivacyMode;

        // 创建 Partial 模式课盘
        let encrypted_data: BoundedVec<u8, crate::mock::MaxEncryptedLen> =
            BoundedVec::try_from(vec![1, 2, 3]).unwrap();

        assert_ok!(XiaoLiuRen::divine_by_time_encrypted(
            RuntimeOrigin::signed(1),
            1, // Partial
            Some(6),
            Some(5),
            Some(7),
            None,
            Some(encrypted_data),
            Some([0u8; 32]),
            Some([0u8; 80]),
        ));

        let pan = Pans::<Test>::get(0).unwrap();
        assert_eq!(pan.privacy_mode, PrivacyMode::Partial);
        assert!(!pan.is_public());
        assert!(pan.can_interpret());

        // 尝试设为公开
        assert_ok!(XiaoLiuRen::set_pan_visibility(
            RuntimeOrigin::signed(1),
            0,
            true,
        ));

        let pan = Pans::<Test>::get(0).unwrap();
        assert_eq!(pan.privacy_mode, PrivacyMode::Public);
        assert!(pan.is_public());
    });
}

#[test]
fn test_encrypted_event_emitted() {
    new_test_ext().execute_with(|| {
        use pallet_divination_privacy::types::PrivacyMode;

        let encrypted_data: BoundedVec<u8, crate::mock::MaxEncryptedLen> =
            BoundedVec::try_from(vec![1, 2, 3]).unwrap();

        assert_ok!(XiaoLiuRen::divine_by_time_encrypted(
            RuntimeOrigin::signed(1),
            1, // Partial
            Some(6),
            Some(5),
            Some(7),
            None,
            Some(encrypted_data),
            Some([0u8; 32]),
            Some([0u8; 80]),
        ));

        // 验证事件
        System::assert_has_event(RuntimeEvent::XiaoLiuRen(Event::EncryptedPanCreated {
            pan_id: 0,
            creator: 1,
            privacy_mode: PrivacyMode::Partial,
            method: DivinationMethod::TimeMethod,
        }));
    });
}
