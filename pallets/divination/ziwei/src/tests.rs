//! # 紫微斗数 Pallet 测试用例

#![allow(deprecated)]

use crate::{mock::*, types::*, algorithm::*, Error, Event};
use frame_support::{assert_noop, assert_ok, BoundedVec};

// ============================================================================
// 基础功能测试
// ============================================================================

#[test]
fn test_divine_by_time_works() {
    new_test_ext().execute_with(|| {
        // 农历 1990年正月初一 子时 男
        assert_ok!(Ziwei::divine_by_time(
            RuntimeOrigin::signed(ALICE),
            1990,
            1,
            1,
            DiZhi::Zi,
            Gender::Male,
            false,
        ));

        // 检查命盘创建
        assert!(Ziwei::charts(0).is_some());

        // 检查用户命盘列表
        let user_charts = Ziwei::user_charts(ALICE);
        assert_eq!(user_charts.len(), 1);
        assert_eq!(user_charts[0], 0);

        // 检查用户统计
        let stats = Ziwei::user_stats(ALICE);
        assert_eq!(stats.total_charts, 1);
    });
}

#[test]
fn test_divine_manual_works() {
    new_test_ext().execute_with(|| {
        // 手动指定四柱
        assert_ok!(Ziwei::divine_manual(
            RuntimeOrigin::signed(ALICE),
            1985,
            8,
            15,
            DiZhi::Wu,
            Gender::Female,
            TianGan::Yi,
            DiZhi::Chou,
        ));

        let chart = Ziwei::charts(0).unwrap();
        assert_eq!(chart.year_gan, Some(TianGan::Yi));
        assert_eq!(chart.year_zhi, Some(DiZhi::Chou));
        assert_eq!(chart.gender, Some(Gender::Female));
    });
}

#[test]
fn test_divine_random_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ziwei::divine_random(RuntimeOrigin::signed(ALICE)));

        // 检查命盘创建
        assert!(Ziwei::charts(0).is_some());
    });
}

// ============================================================================
// 参数校验测试
// ============================================================================

#[test]
fn test_invalid_lunar_month_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Ziwei::divine_by_time(
                RuntimeOrigin::signed(ALICE),
                1990,
                0, // 无效月份
                1,
                DiZhi::Zi,
                Gender::Male,
                false,
            ),
            Error::<Test>::InvalidLunarMonth
        );

        assert_noop!(
            Ziwei::divine_by_time(
                RuntimeOrigin::signed(ALICE),
                1990,
                13, // 无效月份
                1,
                DiZhi::Zi,
                Gender::Male,
                false,
            ),
            Error::<Test>::InvalidLunarMonth
        );
    });
}

#[test]
fn test_invalid_lunar_day_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Ziwei::divine_by_time(
                RuntimeOrigin::signed(ALICE),
                1990,
                1,
                0, // 无效日期
                DiZhi::Zi,
                Gender::Male,
                false,
            ),
            Error::<Test>::InvalidLunarDay
        );

        assert_noop!(
            Ziwei::divine_by_time(
                RuntimeOrigin::signed(ALICE),
                1990,
                1,
                31, // 无效日期
                DiZhi::Zi,
                Gender::Male,
                false,
            ),
            Error::<Test>::InvalidLunarDay
        );
    });
}

#[test]
fn test_invalid_year_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Ziwei::divine_by_time(
                RuntimeOrigin::signed(ALICE),
                1800, // 年份太小
                1,
                1,
                DiZhi::Zi,
                Gender::Male,
                false,
            ),
            Error::<Test>::InvalidYear
        );
    });
}

// ============================================================================
// 每日限制测试
// ============================================================================

#[test]
fn test_daily_limit_works() {
    new_test_ext().execute_with(|| {
        // 排盘 10 次（MaxDailyCharts = 10）
        for i in 0..10 {
            assert_ok!(Ziwei::divine_by_time(
                RuntimeOrigin::signed(ALICE),
                1990,
                1,
                (i + 1) as u8,
                DiZhi::Zi,
                Gender::Male,
                false,
            ));
        }

        // 第 11 次应该失败
        assert_noop!(
            Ziwei::divine_by_time(
                RuntimeOrigin::signed(ALICE),
                1990,
                1,
                11,
                DiZhi::Zi,
                Gender::Male,
                false,
            ),
            Error::<Test>::DailyLimitExceeded
        );
    });
}

// ============================================================================
// AI 解读测试
// ============================================================================

#[test]
fn test_request_ai_interpretation_works() {
    new_test_ext().execute_with(|| {
        // 先创建命盘
        assert_ok!(Ziwei::divine_by_time(
            RuntimeOrigin::signed(ALICE),
            1990,
            1,
            1,
            DiZhi::Zi,
            Gender::Male,
            false,
        ));

        // 请求 AI 解读
        assert_ok!(Ziwei::request_ai_interpretation(
            RuntimeOrigin::signed(ALICE),
            0,
        ));

        // 检查请求状态
        assert!(Ziwei::ai_interpretation_requests(0));

        // 检查用户统计
        let stats = Ziwei::user_stats(ALICE);
        assert_eq!(stats.ai_interpretations, 1);
    });
}

#[test]
fn test_request_ai_interpretation_not_owner_fails() {
    new_test_ext().execute_with(|| {
        // Alice 创建命盘
        assert_ok!(Ziwei::divine_by_time(
            RuntimeOrigin::signed(ALICE),
            1990,
            1,
            1,
            DiZhi::Zi,
            Gender::Male,
            false,
        ));

        // Bob 不能请求 Alice 的命盘解读
        assert_noop!(
            Ziwei::request_ai_interpretation(RuntimeOrigin::signed(BOB), 0),
            Error::<Test>::NotChartOwner
        );
    });
}

#[test]
fn test_submit_ai_interpretation_works() {
    new_test_ext().execute_with(|| {
        // 创建命盘
        assert_ok!(Ziwei::divine_by_time(
            RuntimeOrigin::signed(ALICE),
            1990,
            1,
            1,
            DiZhi::Zi,
            Gender::Male,
            false,
        ));

        // 请求解读
        assert_ok!(Ziwei::request_ai_interpretation(
            RuntimeOrigin::signed(ALICE),
            0,
        ));

        // 提交解读结果
        let cid: BoundedVec<u8, _> = b"QmTest123".to_vec().try_into().unwrap();
        assert_ok!(Ziwei::submit_ai_interpretation(
            RuntimeOrigin::root(),
            0,
            cid.clone(),
        ));

        // 检查命盘更新
        let chart = Ziwei::charts(0).unwrap();
        assert_eq!(chart.ai_interpretation_cid, Some(cid));

        // 检查请求状态已清除
        assert!(!Ziwei::ai_interpretation_requests(0));
    });
}

// ============================================================================
// 可见性测试
// ============================================================================

#[test]
fn test_set_visibility_works() {
    new_test_ext().execute_with(|| {
        // 创建命盘
        assert_ok!(Ziwei::divine_by_time(
            RuntimeOrigin::signed(ALICE),
            1990,
            1,
            1,
            DiZhi::Zi,
            Gender::Male,
            false,
        ));

        // 设为公开
        assert_ok!(Ziwei::set_chart_visibility(
            RuntimeOrigin::signed(ALICE),
            0,
            true,
        ));

        let chart = Ziwei::charts(0).unwrap();
        assert!(chart.is_public());

        // 检查公开列表
        let public_charts = Ziwei::public_charts();
        assert!(public_charts.contains(&0));

        // 设为私有
        assert_ok!(Ziwei::set_chart_visibility(
            RuntimeOrigin::signed(ALICE),
            0,
            false,
        ));

        let chart = Ziwei::charts(0).unwrap();
        assert!(!chart.is_public());

        let public_charts = Ziwei::public_charts();
        assert!(!public_charts.contains(&0));
    });
}

// ============================================================================
// 算法测试
// ============================================================================

#[test]
fn test_calculate_ming_gong() {
    // 正月子时，命宫在寅
    assert_eq!(calculate_ming_gong(1, DiZhi::Zi), 2);

    // 六月午时，命宫在未
    assert_eq!(calculate_ming_gong(6, DiZhi::Wu), 1);
}

#[test]
fn test_calculate_shen_gong() {
    // 正月子时
    let shen_gong = calculate_shen_gong(1, DiZhi::Zi);
    assert!(shen_gong < 12);
}

#[test]
fn test_place_ziwei_series() {
    let positions = place_ziwei_series(0); // 紫微在子宫

    // 验证紫微星系 6 颗星都有位置
    assert_eq!(positions.len(), 6);
    assert_eq!(positions[0].0, ZhuXing::ZiWei);
    assert_eq!(positions[1].0, ZhuXing::TianJi);
    assert_eq!(positions[2].0, ZhuXing::TaiYang);
    assert_eq!(positions[3].0, ZhuXing::WuQu);
    assert_eq!(positions[4].0, ZhuXing::TianTong);
    assert_eq!(positions[5].0, ZhuXing::LianZhen);
}

#[test]
fn test_place_tianfu_series() {
    let positions = place_tianfu_series(0); // 天府在子宫

    // 验证天府星系 8 颗星都有位置
    assert_eq!(positions.len(), 8);
    assert_eq!(positions[0].0, ZhuXing::TianFu);
    assert_eq!(positions[1].0, ZhuXing::TaiYin);
    assert_eq!(positions[2].0, ZhuXing::TanLang);
    assert_eq!(positions[3].0, ZhuXing::JuMen);
    assert_eq!(positions[4].0, ZhuXing::TianXiang);
    assert_eq!(positions[5].0, ZhuXing::TianLiang);
    assert_eq!(positions[6].0, ZhuXing::QiSha);
    assert_eq!(positions[7].0, ZhuXing::PoJun);
}

#[test]
fn test_get_gong_gan() {
    // 甲年寅宫天干应为丙
    let gan = get_gong_gan(TianGan::Jia, 2);
    assert_eq!(gan, TianGan::Bing);
}

#[test]
fn test_calculate_lu_cun() {
    assert_eq!(calculate_lu_cun(TianGan::Jia), 2); // 甲禄在寅
    assert_eq!(calculate_lu_cun(TianGan::Yi), 3);  // 乙禄在卯
    assert_eq!(calculate_lu_cun(TianGan::Bing), 5); // 丙禄在巳
}

#[test]
fn test_get_si_hua_stars() {
    let si_hua = get_si_hua_stars(TianGan::Jia);
    // 甲干四化：廉贞化禄、破军化权、武曲化科、太阳化忌
    assert_eq!(si_hua[0], ZhuXing::LianZhen); // 化禄
    assert_eq!(si_hua[1], ZhuXing::PoJun);    // 化权
    assert_eq!(si_hua[2], ZhuXing::WuQu);     // 化科
    assert_eq!(si_hua[3], ZhuXing::TaiYang);  // 化忌
}

/// 测试完整版四化飞星（支持辅星）
#[test]
fn test_get_si_hua_stars_full() {
    // 甲干四化：廉贞化禄、破军化权、武曲化科、太阳化忌（全主星）
    let jia = get_si_hua_stars_full(TianGan::Jia);
    assert_eq!(jia[0], SiHuaStar::LianZhen);
    assert_eq!(jia[1], SiHuaStar::PoJun);
    assert_eq!(jia[2], SiHuaStar::WuQu);
    assert_eq!(jia[3], SiHuaStar::TaiYang);

    // 丙干四化：天同化禄、天机化权、**文昌**化科、廉贞化忌
    let bing = get_si_hua_stars_full(TianGan::Bing);
    assert_eq!(bing[0], SiHuaStar::TianTong);
    assert_eq!(bing[1], SiHuaStar::TianJi);
    assert_eq!(bing[2], SiHuaStar::WenChang); // 文昌化科（辅星）
    assert_eq!(bing[3], SiHuaStar::LianZhen);

    // 戊干四化：贪狼化禄、太阴化权、**右弼**化科、天机化忌
    let wu = get_si_hua_stars_full(TianGan::Wu);
    assert_eq!(wu[0], SiHuaStar::TanLang);
    assert_eq!(wu[1], SiHuaStar::TaiYin);
    assert_eq!(wu[2], SiHuaStar::YouBi); // 右弼化科（辅星）
    assert_eq!(wu[3], SiHuaStar::TianJi);

    // 己干四化：武曲化禄、贪狼化权、天梁化科、**文曲**化忌
    let ji = get_si_hua_stars_full(TianGan::Ji);
    assert_eq!(ji[0], SiHuaStar::WuQu);
    assert_eq!(ji[1], SiHuaStar::TanLang);
    assert_eq!(ji[2], SiHuaStar::TianLiang);
    assert_eq!(ji[3], SiHuaStar::WenQu); // 文曲化忌（辅星）

    // 辛干四化：巨门化禄、太阳化权、**文曲**化科、**文昌**化忌
    let xin = get_si_hua_stars_full(TianGan::Xin);
    assert_eq!(xin[0], SiHuaStar::JuMen);
    assert_eq!(xin[1], SiHuaStar::TaiYang);
    assert_eq!(xin[2], SiHuaStar::WenQu);   // 文曲化科（辅星）
    assert_eq!(xin[3], SiHuaStar::WenChang); // 文昌化忌（辅星）

    // 壬干四化：天梁化禄、紫微化权、**左辅**化科、武曲化忌
    let ren = get_si_hua_stars_full(TianGan::Ren);
    assert_eq!(ren[0], SiHuaStar::TianLiang);
    assert_eq!(ren[1], SiHuaStar::ZiWei);
    assert_eq!(ren[2], SiHuaStar::ZuoFu); // 左辅化科（辅星）
    assert_eq!(ren[3], SiHuaStar::WuQu);
}

/// 测试 SiHuaStar 类型转换
#[test]
fn test_si_hua_star_conversions() {
    // 主星转换
    let zhu_xing = ZhuXing::ZiWei;
    let si_hua_star = SiHuaStar::from_zhu_xing(zhu_xing);
    assert_eq!(si_hua_star, SiHuaStar::ZiWei);
    assert!(si_hua_star.is_zhu_xing());
    assert!(!si_hua_star.is_fu_xing());
    assert_eq!(si_hua_star.to_zhu_xing(), Some(ZhuXing::ZiWei));

    // 辅星转换
    let liu_ji = LiuJiXing::WenChang;
    let si_hua_star = SiHuaStar::from_liu_ji_xing(liu_ji).unwrap();
    assert_eq!(si_hua_star, SiHuaStar::WenChang);
    assert!(!si_hua_star.is_zhu_xing());
    assert!(si_hua_star.is_fu_xing());
    assert_eq!(si_hua_star.to_liu_ji_xing(), Some(LiuJiXing::WenChang));

    // 不参与四化的辅星
    let tian_kui = LiuJiXing::TianKui;
    assert!(SiHuaStar::from_liu_ji_xing(tian_kui).is_none());
}

#[test]
fn test_calculate_da_yun_direction() {
    // 阳男顺行
    assert!(calculate_da_yun_direction(TianGan::Jia, Gender::Male));

    // 阳女逆行
    assert!(!calculate_da_yun_direction(TianGan::Jia, Gender::Female));

    // 阴男逆行
    assert!(!calculate_da_yun_direction(TianGan::Yi, Gender::Male));

    // 阴女顺行
    assert!(calculate_da_yun_direction(TianGan::Yi, Gender::Female));
}

// ============================================================================
// 类型测试
// ============================================================================

#[test]
fn test_tian_gan_properties() {
    assert_eq!(TianGan::Jia.name(), "甲");
    assert_eq!(TianGan::Jia.index(), 0);
    assert_eq!(TianGan::Jia.yin_yang(), YinYang::Yang);
    assert_eq!(TianGan::Jia.wu_xing(), WuXing::Wood);

    assert_eq!(TianGan::Yi.yin_yang(), YinYang::Yin);
    assert_eq!(TianGan::Yi.wu_xing(), WuXing::Wood);
}

#[test]
fn test_di_zhi_properties() {
    assert_eq!(DiZhi::Zi.name(), "子");
    assert_eq!(DiZhi::Zi.index(), 0);
    assert_eq!(DiZhi::Zi.sheng_xiao(), "鼠");

    assert_eq!(DiZhi::Chou.sheng_xiao(), "牛");
    assert_eq!(DiZhi::Yin.sheng_xiao(), "虎");
}

#[test]
fn test_wu_xing_ju_shu() {
    assert_eq!(WuXing::Water.ju_shu(), 2);
    assert_eq!(WuXing::Wood.ju_shu(), 3);
    assert_eq!(WuXing::Metal.ju_shu(), 4);
    assert_eq!(WuXing::Earth.ju_shu(), 5);
    assert_eq!(WuXing::Fire.ju_shu(), 6);
}

#[test]
fn test_zhu_xing_series() {
    assert!(ZhuXing::ZiWei.is_ziwei_series());
    assert!(ZhuXing::TianJi.is_ziwei_series());
    assert!(!ZhuXing::TianFu.is_ziwei_series());

    assert!(ZhuXing::TianFu.is_tianfu_series());
    assert!(ZhuXing::TaiYin.is_tianfu_series());
    assert!(!ZhuXing::ZiWei.is_tianfu_series());
}

#[test]
fn test_star_brightness_weight() {
    assert_eq!(StarBrightness::Miao.weight(), 100);
    assert_eq!(StarBrightness::Wang.weight(), 80);
    assert_eq!(StarBrightness::De.weight(), 60);
    assert_eq!(StarBrightness::Ping.weight(), 40);
    assert_eq!(StarBrightness::BuDe.weight(), 20);
    assert_eq!(StarBrightness::Xian.weight(), 10);
}

// ============================================================================
// 事件测试
// ============================================================================

#[test]
fn test_events_emitted() {
    new_test_ext().execute_with(|| {
        // 创建命盘
        assert_ok!(Ziwei::divine_by_time(
            RuntimeOrigin::signed(ALICE),
            1990,
            1,
            1,
            DiZhi::Zi,
            Gender::Male,
            false,
        ));

        // 检查事件
        System::assert_has_event(RuntimeEvent::Ziwei(Event::ChartCreated {
            chart_id: 0,
            creator: ALICE,
            wu_xing_ju: Ziwei::charts(0).unwrap().wu_xing_ju,
            ju_shu: Ziwei::charts(0).unwrap().ju_shu,
        }));
    });
}

// ============================================================================
// 新增算法测试 - Phase 1: 算法修复
// ============================================================================

/// 测试天府星位置计算（修正后公式）
#[test]
fn test_calculate_tianfu_position() {
    // 紫府对照表验证
    // 紫微在子(0) → 天府在辰(4)
    assert_eq!(calculate_tianfu_position(0), 4);
    // 紫微在丑(1) → 天府在卯(3)
    assert_eq!(calculate_tianfu_position(1), 3);
    // 紫微在寅(2) → 天府在寅(2)（自对）
    assert_eq!(calculate_tianfu_position(2), 2);
    // 紫微在卯(3) → 天府在丑(1)
    assert_eq!(calculate_tianfu_position(3), 1);
    // 紫微在辰(4) → 天府在子(0)
    assert_eq!(calculate_tianfu_position(4), 0);
    // 紫微在巳(5) → 天府在亥(11)
    assert_eq!(calculate_tianfu_position(5), 11);
    // 紫微在午(6) → 天府在戌(10)
    assert_eq!(calculate_tianfu_position(6), 10);
    // 紫微在未(7) → 天府在酉(9)
    assert_eq!(calculate_tianfu_position(7), 9);
    // 紫微在申(8) → 天府在申(8)（自对）
    assert_eq!(calculate_tianfu_position(8), 8);
    // 紫微在酉(9) → 天府在未(7)
    assert_eq!(calculate_tianfu_position(9), 7);
    // 紫微在戌(10) → 天府在午(6)
    assert_eq!(calculate_tianfu_position(10), 6);
    // 紫微在亥(11) → 天府在巳(5)
    assert_eq!(calculate_tianfu_position(11), 5);
}

/// 测试火星铃星安星（修正后规则）
#[test]
fn test_calculate_huo_ling() {
    // 寅午戌年（2,6,10）丑宫起火星，卯宫起铃星
    let (huo, ling) = calculate_huo_ling(DiZhi::Yin, DiZhi::Zi); // 寅年子时
    assert_eq!(huo, 1); // 丑(1) + 子(0) = 1
    assert_eq!(ling, 3); // 卯(3) + 子(0) = 3

    // 申子辰年（8,0,4）寅宫起火星，戌宫起铃星
    let (huo, ling) = calculate_huo_ling(DiZhi::Zi, DiZhi::Zi); // 子年子时
    assert_eq!(huo, 2); // 寅(2) + 子(0) = 2
    assert_eq!(ling, 10); // 戌(10) + 子(0) = 10

    // 巳酉丑年（5,9,1）卯宫起火星，戌宫起铃星
    let (huo, ling) = calculate_huo_ling(DiZhi::You, DiZhi::Wu); // 酉年午时
    assert_eq!(huo, (3 + 6) % 12); // 卯(3) + 午(6) = 9
    assert_eq!(ling, (10 + 6) % 12); // 戌(10) + 午(6) = 4

    // 亥卯未年（11,3,7）酉宫起火星，戌宫起铃星
    let (huo, ling) = calculate_huo_ling(DiZhi::Mao, DiZhi::Chen); // 卯年辰时
    assert_eq!(huo, (9 + 4) % 12); // 酉(9) + 辰(4) = 1
    assert_eq!(ling, (10 + 4) % 12); // 戌(10) + 辰(4) = 2
}

/// 测试天马安星
#[test]
fn test_calculate_tian_ma() {
    // 申子辰年马在寅
    assert_eq!(calculate_tian_ma(DiZhi::Shen), 2);
    assert_eq!(calculate_tian_ma(DiZhi::Zi), 2);
    assert_eq!(calculate_tian_ma(DiZhi::Chen), 2);

    // 寅午戌年马在申
    assert_eq!(calculate_tian_ma(DiZhi::Yin), 8);
    assert_eq!(calculate_tian_ma(DiZhi::Wu), 8);
    assert_eq!(calculate_tian_ma(DiZhi::Xu), 8);

    // 亥卯未年马在巳
    assert_eq!(calculate_tian_ma(DiZhi::Hai), 5);
    assert_eq!(calculate_tian_ma(DiZhi::Mao), 5);
    assert_eq!(calculate_tian_ma(DiZhi::Wei), 5);

    // 巳酉丑年马在亥
    assert_eq!(calculate_tian_ma(DiZhi::Si), 11);
    assert_eq!(calculate_tian_ma(DiZhi::You), 11);
    assert_eq!(calculate_tian_ma(DiZhi::Chou), 11);
}

// ============================================================================
// 新增算法测试 - Phase 2: 庙旺表
// ============================================================================

/// 测试星曜亮度转换
#[test]
fn test_star_brightness_from_value() {
    assert_eq!(StarBrightness::from_value(0), StarBrightness::Xian);
    assert_eq!(StarBrightness::from_value(1), StarBrightness::Ping);
    assert_eq!(StarBrightness::from_value(2), StarBrightness::De);
    assert_eq!(StarBrightness::from_value(3), StarBrightness::BuDe);
    assert_eq!(StarBrightness::from_value(4), StarBrightness::Wang);
    assert_eq!(StarBrightness::from_value(5), StarBrightness::Miao);
    assert_eq!(StarBrightness::from_value(99), StarBrightness::Ping); // 默认值
}

/// 测试主星亮度查表
#[test]
fn test_get_star_brightness() {
    // 紫微在丑宫庙
    assert_eq!(get_star_brightness(ZhuXing::ZiWei, DiZhi::Chou), StarBrightness::Miao);
    // 天机在子宫庙
    assert_eq!(get_star_brightness(ZhuXing::TianJi, DiZhi::Zi), StarBrightness::Miao);
    // 太阳在午宫庙
    assert_eq!(get_star_brightness(ZhuXing::TaiYang, DiZhi::Wu), StarBrightness::Miao);
    // 太阴在子宫庙
    assert_eq!(get_star_brightness(ZhuXing::TaiYin, DiZhi::Zi), StarBrightness::Miao);
}

// ============================================================================
// 新增算法测试 - Phase 3: 杂曜星系
// ============================================================================

/// 测试博士十二星
#[test]
fn test_calculate_bo_shi_stars() {
    // 禄存在寅(2)，顺行
    let positions = calculate_bo_shi_stars(2, true);
    assert_eq!(positions[0], 2);  // 博士在寅
    assert_eq!(positions[1], 3);  // 力士在卯
    assert_eq!(positions[11], 1); // 官府在丑

    // 禄存在寅(2)，逆行
    let positions = calculate_bo_shi_stars(2, false);
    assert_eq!(positions[0], 2);  // 博士在寅
    assert_eq!(positions[1], 1);  // 力士在丑
    assert_eq!(positions[11], 3); // 官府在卯
}

/// 测试博士十二星类型
#[test]
fn test_bo_shi_xing() {
    assert_eq!(BoShiXing::BoShi.name(), "博士");
    assert!(BoShiXing::BoShi.is_ji());
    assert!(BoShiXing::QingLong.is_ji());
    assert!(BoShiXing::XiShen.is_ji());

    assert!(BoShiXing::XiaoHao.is_xiong());
    assert!(BoShiXing::DaHao.is_xiong());
    assert!(BoShiXing::GuanFu.is_xiong());
}

/// 测试长生起点
#[test]
fn test_calculate_chang_sheng_start() {
    assert_eq!(calculate_chang_sheng_start(WuXing::Water), 8); // 申
    assert_eq!(calculate_chang_sheng_start(WuXing::Earth), 8); // 申
    assert_eq!(calculate_chang_sheng_start(WuXing::Wood), 11); // 亥
    assert_eq!(calculate_chang_sheng_start(WuXing::Metal), 5); // 巳
    assert_eq!(calculate_chang_sheng_start(WuXing::Fire), 2);  // 寅
}

/// 测试长生十二宫
#[test]
fn test_calculate_chang_sheng_positions() {
    // 水局，顺行
    let positions = calculate_chang_sheng_positions(WuXing::Water, true);
    assert_eq!(positions[0], 8);  // 长生在申
    assert_eq!(positions[1], 9);  // 沐浴在酉
    assert_eq!(positions[4], 0);  // 帝旺在子

    // 木局，逆行
    let positions = calculate_chang_sheng_positions(WuXing::Wood, false);
    assert_eq!(positions[0], 11); // 长生在亥
    assert_eq!(positions[1], 10); // 沐浴在戌
}

/// 测试长生十二宫类型
#[test]
fn test_chang_sheng() {
    assert_eq!(ChangSheng::ChangSheng.name(), "长生");
    assert!(ChangSheng::ChangSheng.is_ji());
    assert!(ChangSheng::DiWang.is_ji());
    assert!(ChangSheng::LinGuan.is_ji());

    assert!(ChangSheng::MuYu.is_xiong());
    assert!(ChangSheng::Si.is_xiong());
    assert!(ChangSheng::Mu.is_xiong());
}

/// 测试命主计算
#[test]
fn test_calculate_ming_zhu() {
    // 子宫命主贪狼
    assert_eq!(get_ming_zhu_name(DiZhi::Zi), "贪狼");
    assert_eq!(calculate_ming_zhu(DiZhi::Zi), Some(ZhuXing::TanLang));

    // 丑宫命主巨门
    assert_eq!(get_ming_zhu_name(DiZhi::Chou), "巨门");
    assert_eq!(calculate_ming_zhu(DiZhi::Chou), Some(ZhuXing::JuMen));

    // 寅宫命主禄存（辅星返回None）
    assert_eq!(get_ming_zhu_name(DiZhi::Yin), "禄存");
    assert_eq!(calculate_ming_zhu(DiZhi::Yin), None);

    // 午宫命主破军
    assert_eq!(get_ming_zhu_name(DiZhi::Wu), "破军");
    assert_eq!(calculate_ming_zhu(DiZhi::Wu), Some(ZhuXing::PoJun));
}

/// 测试身主计算
#[test]
fn test_calculate_shen_zhu() {
    // 子年身主火星
    assert_eq!(get_shen_zhu_name(DiZhi::Zi), "火星");
    assert_eq!(calculate_shen_zhu(DiZhi::Zi), None);

    // 丑年身主天相
    assert_eq!(get_shen_zhu_name(DiZhi::Chou), "天相");
    assert_eq!(calculate_shen_zhu(DiZhi::Chou), Some(ZhuXing::TianXiang));

    // 寅年身主天梁
    assert_eq!(get_shen_zhu_name(DiZhi::Yin), "天梁");
    assert_eq!(calculate_shen_zhu(DiZhi::Yin), Some(ZhuXing::TianLiang));
}

// ============================================================================
// 新增算法测试 - Phase 4: 大限
// ============================================================================

/// 测试大限详情生成
#[test]
fn test_generate_da_xian_details() {
    // 命宫在寅(2)，金四局，顺行，甲年
    let da_xians = generate_da_xian_details(2, 4, true, TianGan::Jia);

    // 第一大限：4-13岁，在寅宫
    assert_eq!(da_xians[0].0, 1);  // 序号
    assert_eq!(da_xians[0].1, 4);  // 起始年龄
    assert_eq!(da_xians[0].2, 13); // 结束年龄
    assert_eq!(da_xians[0].3, DiZhi::Yin); // 宫位

    // 第二大限：14-23岁，在卯宫
    assert_eq!(da_xians[1].0, 2);
    assert_eq!(da_xians[1].1, 14);
    assert_eq!(da_xians[1].2, 23);
    assert_eq!(da_xians[1].3, DiZhi::Mao);

    // 逆行测试
    let da_xians_rev = generate_da_xian_details(2, 4, false, TianGan::Jia);
    // 第二大限应在丑宫
    assert_eq!(da_xians_rev[1].3, DiZhi::Chou);
}

// ============================================================================
// 隐私模式测试 - Phase 1.2.4
// ============================================================================

#[test]
fn test_divine_by_time_encrypted_public_mode() {
    use pallet_divination_privacy::types::PrivacyMode;
    #[allow(unused_imports)]
    use frame_support::BoundedVec;

    new_test_ext().execute_with(|| {
        // Public 模式（加密级别 0）- 无需加密数据
        assert_ok!(Ziwei::divine_by_time_encrypted(
            RuntimeOrigin::signed(ALICE),
            0, // encryption_level = Public
            1990,
            1,
            1,
            DiZhi::Zi,
            Gender::Male,
            false,
            None, // 无加密数据
            None, // 无数据哈希
            None, // 无密钥备份
        ));

        // 验证命盘创建
        let chart = Ziwei::charts(0).unwrap();
        assert_eq!(chart.privacy_mode, PrivacyMode::Public);
        assert!(chart.has_calculation_data());
        assert!(chart.can_interpret());
        assert!(chart.is_public());

        // 验证计算数据存在
        assert!(chart.palaces.is_some());
        assert!(chart.wu_xing_ju.is_some());
        assert!(chart.ming_gong_pos.is_some());
    });
}

#[test]
fn test_divine_by_time_encrypted_partial_mode() {
    use pallet_divination_privacy::types::PrivacyMode;
    use frame_support::BoundedVec;

    new_test_ext().execute_with(|| {
        // 准备加密数据
        let encrypted_data: BoundedVec<u8, _> = vec![1, 2, 3, 4, 5].try_into().unwrap();
        let data_hash = [0u8; 32];
        let owner_key_backup = [0u8; 80];

        // Partial 模式（加密级别 1）- 需要加密数据
        assert_ok!(Ziwei::divine_by_time_encrypted(
            RuntimeOrigin::signed(ALICE),
            1, // encryption_level = Partial
            1990,
            1,
            1,
            DiZhi::Zi,
            Gender::Male,
            false,
            Some(encrypted_data),
            Some(data_hash),
            Some(owner_key_backup),
        ));

        // 验证命盘
        let chart = Ziwei::charts(0).unwrap();
        assert_eq!(chart.privacy_mode, PrivacyMode::Partial);
        assert!(chart.has_calculation_data());
        assert!(chart.can_interpret());
        assert!(!chart.is_public());

        // 验证加密数据存储
        assert!(Ziwei::encrypted_data(0).is_some());
        assert!(Ziwei::owner_key_backup(0).is_some());
    });
}

#[test]
fn test_divine_by_time_encrypted_private_mode() {
    use pallet_divination_privacy::types::PrivacyMode;
    use frame_support::BoundedVec;

    new_test_ext().execute_with(|| {
        // 准备加密数据
        let encrypted_data: BoundedVec<u8, _> = vec![1, 2, 3, 4, 5].try_into().unwrap();
        let data_hash = [0u8; 32];
        let owner_key_backup = [0u8; 80];

        // Private 模式（加密级别 2）- 不存储计算数据
        assert_ok!(Ziwei::divine_by_time_encrypted(
            RuntimeOrigin::signed(ALICE),
            2, // encryption_level = Private
            1990,
            1,
            1,
            DiZhi::Zi,
            Gender::Male,
            false,
            Some(encrypted_data),
            Some(data_hash),
            Some(owner_key_backup),
        ));

        // 验证命盘
        let chart = Ziwei::charts(0).unwrap();
        assert_eq!(chart.privacy_mode, PrivacyMode::Private);
        assert!(!chart.has_calculation_data()); // 无计算数据
        assert!(!chart.can_interpret()); // 无法解读
        assert!(!chart.is_public());

        // 验证计算数据不存在
        assert!(chart.palaces.is_none());
        assert!(chart.wu_xing_ju.is_none());
        assert!(chart.ming_gong_pos.is_none());

        // 但加密数据和元数据存在
        assert!(Ziwei::encrypted_data(0).is_some());
        assert!(Ziwei::owner_key_backup(0).is_some());
    });
}

#[test]
fn test_divine_by_time_encrypted_missing_data_fails() {
    new_test_ext().execute_with(|| {
        // Partial 模式缺少加密数据应该失败
        assert_noop!(
            Ziwei::divine_by_time_encrypted(
                RuntimeOrigin::signed(ALICE),
                1, // encryption_level = Partial
                1990,
                1,
                1,
                DiZhi::Zi,
                Gender::Male,
                false,
                None, // 缺少加密数据
                None,
                None,
            ),
            Error::<Test>::EncryptedDataMissing
        );

        // Private 模式缺少加密数据应该失败
        assert_noop!(
            Ziwei::divine_by_time_encrypted(
                RuntimeOrigin::signed(ALICE),
                2, // encryption_level = Private
                1990,
                1,
                1,
                DiZhi::Zi,
                Gender::Male,
                false,
                None, // 缺少加密数据
                None,
                None,
            ),
            Error::<Test>::EncryptedDataMissing
        );
    });
}

#[test]
fn test_divine_by_time_encrypted_invalid_level_fails() {
    new_test_ext().execute_with(|| {
        // 无效的加密级别应该失败
        assert_noop!(
            Ziwei::divine_by_time_encrypted(
                RuntimeOrigin::signed(ALICE),
                3, // 无效的加密级别
                1990,
                1,
                1,
                DiZhi::Zi,
                Gender::Male,
                false,
                None,
                None,
                None,
            ),
            Error::<Test>::InvalidEncryptionLevel
        );
    });
}

#[test]
fn test_update_encrypted_data_works() {
    use frame_support::BoundedVec;

    new_test_ext().execute_with(|| {
        // 先创建 Partial 模式命盘
        let encrypted_data: BoundedVec<u8, _> = vec![1, 2, 3].try_into().unwrap();
        let data_hash = [1u8; 32];
        let owner_key_backup = [1u8; 80];

        assert_ok!(Ziwei::divine_by_time_encrypted(
            RuntimeOrigin::signed(ALICE),
            1,
            1990,
            1,
            1,
            DiZhi::Zi,
            Gender::Male,
            false,
            Some(encrypted_data),
            Some(data_hash),
            Some(owner_key_backup),
        ));

        // 更新加密数据
        let new_encrypted_data: BoundedVec<u8, _> = vec![4, 5, 6, 7].try_into().unwrap();
        let new_data_hash = [2u8; 32];
        let new_owner_key_backup = [2u8; 80];

        assert_ok!(Ziwei::update_encrypted_data(
            RuntimeOrigin::signed(ALICE),
            0,
            new_encrypted_data.clone(),
            new_data_hash,
            new_owner_key_backup,
        ));

        // 验证更新
        let stored_data = Ziwei::encrypted_data(0).unwrap();
        assert_eq!(stored_data.to_vec(), vec![4, 5, 6, 7]);

        let chart = Ziwei::charts(0).unwrap();
        assert_eq!(chart.sensitive_data_hash, Some(new_data_hash));
    });
}

#[test]
fn test_update_encrypted_data_not_owner_fails() {
    use frame_support::BoundedVec;

    new_test_ext().execute_with(|| {
        // Alice 创建命盘
        let encrypted_data: BoundedVec<u8, _> = vec![1, 2, 3].try_into().unwrap();
        assert_ok!(Ziwei::divine_by_time_encrypted(
            RuntimeOrigin::signed(ALICE),
            1,
            1990,
            1,
            1,
            DiZhi::Zi,
            Gender::Male,
            false,
            Some(encrypted_data.clone()),
            Some([0u8; 32]),
            Some([0u8; 80]),
        ));

        // Bob 尝试更新应该失败
        assert_noop!(
            Ziwei::update_encrypted_data(
                RuntimeOrigin::signed(BOB),
                0,
                encrypted_data,
                [0u8; 32],
                [0u8; 80],
            ),
            Error::<Test>::NotChartOwner
        );
    });
}

#[test]
fn test_compute_chart_result_works() {
    use crate::runtime_api::compute_chart_result;

    // 正常计算
    let result = compute_chart_result(1990, 1, 1, 0, 0, false);
    assert!(result.is_some());

    let chart = result.unwrap();
    assert!(chart.ju_shu >= 2 && chart.ju_shu <= 6);
    assert!(chart.ming_gong_pos < 12);
    assert!(chart.shen_gong_pos < 12);
    assert!(chart.ziwei_pos < 12);
    assert!(chart.tianfu_pos < 12);

    // 无效参数应返回 None
    let invalid_month = compute_chart_result(1990, 0, 1, 0, 0, false);
    assert!(invalid_month.is_none());

    let invalid_day = compute_chart_result(1990, 1, 0, 0, 0, false);
    assert!(invalid_day.is_none());

    let invalid_hour = compute_chart_result(1990, 1, 1, 12, 0, false);
    assert!(invalid_hour.is_none());
}

#[test]
fn test_ziwei_public_metadata_default() {
    use crate::runtime_api::ZiweiPublicMetadata;

    let metadata = ZiweiPublicMetadata::default();
    assert_eq!(metadata.id, 0);
    assert!(!metadata.has_encrypted_data);
    assert!(!metadata.can_interpret);
    assert!(!metadata.has_ai_interpretation);
}

#[test]
fn test_ziwei_chart_result_fields() {
    use crate::runtime_api::compute_chart_result;

    let result = compute_chart_result(1985, 8, 15, 6, 1, false).unwrap();

    // 验证所有字段都有效
    assert!(matches!(
        result.wu_xing_ju,
        WuXing::Water | WuXing::Wood | WuXing::Metal | WuXing::Earth | WuXing::Fire
    ));
    assert!(result.qi_yun_age >= 2 && result.qi_yun_age <= 6);

    // 验证十二宫都已初始化
    for palace in result.palaces.iter() {
        assert!(palace.di_zhi.index() < 12);
    }
}
