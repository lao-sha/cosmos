//! # 六爻 Pallet 测试用例

use crate::{mock::*, types::*, algorithm::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

// ============================================================================
// 基础功能测试
// ============================================================================

#[test]
fn test_divine_by_coins_works() {
    new_test_ext().execute_with(|| {
        // 铜钱起卦：少阴少阳少阴少阳少阴少阳
        // 0个阳面=老阴, 1个阳面=少阳, 2个阳面=少阴, 3个阳面=老阳
        let coins = [2, 1, 2, 1, 2, 1]; // 少阴少阳交替

        assert_ok!(Liuyao::divine_by_coins(
            RuntimeOrigin::signed(ALICE),
            coins,
            (0, 0), // 甲子年
            (2, 2), // 丙寅月
            (4, 4), // 戊辰日
            (6, 6), // 庚午时
        ));

        // 检查卦象创建
        assert!(Liuyao::guas(0).is_some());

        // 检查用户卦象列表
        let user_guas = Liuyao::user_guas(ALICE);
        assert_eq!(user_guas.len(), 1);
        assert_eq!(user_guas[0], 0);

        // 检查用户统计
        let stats = Liuyao::user_stats(ALICE);
        assert_eq!(stats.total_guas, 1);
    });
}

#[test]
fn test_divine_by_numbers_works() {
    new_test_ext().execute_with(|| {
        // 数字起卦：上卦5，下卦3，动爻2
        assert_ok!(Liuyao::divine_by_numbers(
            RuntimeOrigin::signed(ALICE),
            5,  // 上卦数
            3,  // 下卦数
            2,  // 动爻位置
            (0, 0), (2, 2), (4, 4), (6, 6),
        ));

        let gua = Liuyao::guas(0).unwrap();
        assert_eq!(gua.method, DivinationMethod::NumberMethod);
    });
}

#[test]
fn test_divine_random_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Liuyao::divine_random(RuntimeOrigin::signed(ALICE)));

        // 检查卦象创建
        assert!(Liuyao::guas(0).is_some());
    });
}

#[test]
fn test_divine_manual_works() {
    new_test_ext().execute_with(|| {
        // 手动指定：全阳爻，二爻动
        let yaos = [1, 3, 1, 1, 1, 1]; // 少阳、老阳、少阳...

        assert_ok!(Liuyao::divine_manual(
            RuntimeOrigin::signed(ALICE),
            yaos,
            (0, 0), (2, 2), (4, 4), (6, 6),
        ));

        let gua = Liuyao::guas(0).unwrap();
        assert_eq!(gua.method, DivinationMethod::ManualMethod);
        assert!(gua.has_bian_gua); // 有动爻，应该有变卦
    });
}

// ============================================================================
// 参数校验测试
// ============================================================================

#[test]
fn test_invalid_coin_count_fails() {
    new_test_ext().execute_with(|| {
        let invalid_coins = [4, 1, 2, 1, 2, 1]; // 4 超出范围

        assert_noop!(
            Liuyao::divine_by_coins(
                RuntimeOrigin::signed(ALICE),
                invalid_coins,
                (0, 0), (2, 2), (4, 4), (6, 6),
            ),
            Error::<Test>::InvalidCoinCount
        );
    });
}

#[test]
fn test_invalid_number_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Liuyao::divine_by_numbers(
                RuntimeOrigin::signed(ALICE),
                0, 3, 2, // num1 = 0 无效
                (0, 0), (2, 2), (4, 4), (6, 6),
            ),
            Error::<Test>::InvalidNumber
        );
    });
}

#[test]
fn test_invalid_dong_yao_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Liuyao::divine_by_numbers(
                RuntimeOrigin::signed(ALICE),
                5, 3, 7, // dong = 7 超出范围
                (0, 0), (2, 2), (4, 4), (6, 6),
            ),
            Error::<Test>::InvalidDongYao
        );

        assert_noop!(
            Liuyao::divine_by_numbers(
                RuntimeOrigin::signed(ALICE),
                5, 3, 0, // dong = 0 无效
                (0, 0), (2, 2), (4, 4), (6, 6),
            ),
            Error::<Test>::InvalidDongYao
        );
    });
}

// ============================================================================
// 每日限制测试
// ============================================================================

#[test]
fn test_daily_limit_works() {
    new_test_ext().execute_with(|| {
        // 起卦 10 次（MaxDailyGuas = 10）
        for _ in 0..10 {
            assert_ok!(Liuyao::divine_random(RuntimeOrigin::signed(ALICE)));
        }

        // 第 11 次应该失败
        assert_noop!(
            Liuyao::divine_random(RuntimeOrigin::signed(ALICE)),
            Error::<Test>::DailyLimitExceeded
        );
    });
}

// ============================================================================
// 可见性测试
// ============================================================================

#[test]
fn test_set_visibility_works() {
    new_test_ext().execute_with(|| {
        // 创建卦象
        assert_ok!(Liuyao::divine_random(RuntimeOrigin::signed(ALICE)));

        // 设为公开
        assert_ok!(Liuyao::set_gua_visibility(
            RuntimeOrigin::signed(ALICE),
            0,
            true,
        ));

        let gua = Liuyao::guas(0).unwrap();
        assert!(gua.is_public());

        // 检查公开列表
        let public_guas = Liuyao::public_guas();
        assert!(public_guas.contains(&0));

        // 设为私有
        assert_ok!(Liuyao::set_gua_visibility(
            RuntimeOrigin::signed(ALICE),
            0,
            false,
        ));

        let gua = Liuyao::guas(0).unwrap();
        assert!(!gua.is_public());

        let public_guas = Liuyao::public_guas();
        assert!(!public_guas.contains(&0));
    });
}

// ============================================================================
// 加密模式测试
// ============================================================================

#[test]
fn test_divine_by_coins_encrypted_public() {
    new_test_ext().execute_with(|| {
        use pallet_divination_privacy::types::PrivacyMode;

        // Public 模式起卦
        let coins = [2, 1, 2, 1, 2, 1];
        assert_ok!(Liuyao::divine_by_coins_encrypted(
            RuntimeOrigin::signed(ALICE),
            0, // Public 模式
            Some(coins),
            Some((0, 0)),
            Some((2, 2)),
            Some((4, 4)),
            Some((6, 6)),
            None, // 无加密数据
            None, // 无数据哈希
            None, // 无密钥备份
        ));

        // 验证卦象
        let gua = Liuyao::guas(0).unwrap();
        assert_eq!(gua.privacy_mode, PrivacyMode::Public);
        assert!(gua.original_yaos.is_some());
        assert!(gua.is_public());
        assert!(gua.can_interpret());
    });
}

#[test]
fn test_divine_by_coins_encrypted_partial() {
    new_test_ext().execute_with(|| {
        use frame_support::{BoundedVec, traits::ConstU32};
        use pallet_divination_privacy::types::PrivacyMode;

        // Partial 模式起卦
        let coins = [1, 2, 3, 1, 2, 0];
        let encrypted_data: BoundedVec<u8, ConstU32<512>> =
            BoundedVec::try_from(vec![1, 2, 3, 4, 5]).unwrap();
        let data_hash = [0u8; 32];
        let owner_key_backup = [0u8; 80];

        assert_ok!(Liuyao::divine_by_coins_encrypted(
            RuntimeOrigin::signed(ALICE),
            1, // Partial 模式
            Some(coins),
            Some((0, 0)),
            Some((2, 2)),
            Some((4, 4)),
            Some((6, 6)),
            Some(encrypted_data.clone()),
            Some(data_hash),
            Some(owner_key_backup),
        ));

        // 验证卦象
        let gua = Liuyao::guas(0).unwrap();
        assert_eq!(gua.privacy_mode, PrivacyMode::Partial);
        assert!(gua.original_yaos.is_some());
        assert!(gua.can_interpret());
        assert!(!gua.is_public());

        // 验证加密数据存储
        assert!(Liuyao::encrypted_data(0).is_some());
        assert!(Liuyao::owner_key_backup(0).is_some());
    });
}

#[test]
fn test_divine_by_coins_encrypted_private() {
    new_test_ext().execute_with(|| {
        use frame_support::{BoundedVec, traits::ConstU32};
        use pallet_divination_privacy::types::PrivacyMode;

        // Private 模式起卦
        let encrypted_data: BoundedVec<u8, ConstU32<512>> =
            BoundedVec::try_from(vec![5, 6, 7, 8]).unwrap();
        let data_hash = [1u8; 32];
        let owner_key_backup = [2u8; 80];

        assert_ok!(Liuyao::divine_by_coins_encrypted(
            RuntimeOrigin::signed(ALICE),
            2, // Private 模式
            None, // 无明文数据
            None,
            None,
            None,
            None,
            Some(encrypted_data.clone()),
            Some(data_hash),
            Some(owner_key_backup),
        ));

        // 验证卦象
        let gua = Liuyao::guas(0).unwrap();
        assert_eq!(gua.privacy_mode, PrivacyMode::Private);
        assert!(gua.original_yaos.is_none()); // Private 模式无计算数据
        assert!(!gua.can_interpret()); // 无法解读
        assert!(gua.is_private());

        // 验证加密数据存储
        assert!(Liuyao::encrypted_data(0).is_some());
        assert!(Liuyao::owner_key_backup(0).is_some());
    });
}

#[test]
fn test_update_encrypted_data_works() {
    new_test_ext().execute_with(|| {
        use frame_support::{BoundedVec, traits::ConstU32};

        // 先创建一个 Partial 模式的卦象
        let coins = [1, 2, 1, 2, 1, 2];
        let encrypted_data: BoundedVec<u8, ConstU32<512>> =
            BoundedVec::try_from(vec![1, 2, 3]).unwrap();
        let data_hash = [0u8; 32];
        let owner_key_backup = [0u8; 80];

        assert_ok!(Liuyao::divine_by_coins_encrypted(
            RuntimeOrigin::signed(ALICE),
            1,
            Some(coins),
            Some((0, 0)),
            Some((2, 2)),
            Some((4, 4)),
            Some((6, 6)),
            Some(encrypted_data),
            Some(data_hash),
            Some(owner_key_backup),
        ));

        // 更新加密数据
        let new_encrypted_data: BoundedVec<u8, ConstU32<512>> =
            BoundedVec::try_from(vec![9, 8, 7, 6, 5]).unwrap();
        let new_data_hash = [3u8; 32];
        let new_owner_key_backup = [4u8; 80];

        assert_ok!(Liuyao::update_encrypted_data(
            RuntimeOrigin::signed(ALICE),
            0,
            new_encrypted_data.clone(),
            new_data_hash,
            new_owner_key_backup,
        ));

        // 验证更新
        let gua = Liuyao::guas(0).unwrap();
        assert_eq!(gua.sensitive_data_hash, Some(new_data_hash));

        let stored_data = Liuyao::encrypted_data(0).unwrap();
        assert_eq!(stored_data.to_vec(), vec![9, 8, 7, 6, 5]);
    });
}

#[test]
fn test_update_encrypted_data_not_owner() {
    new_test_ext().execute_with(|| {
        use frame_support::{BoundedVec, traits::ConstU32};

        // ALICE 创建卦象
        let coins = [1, 2, 1, 2, 1, 2];
        let encrypted_data: BoundedVec<u8, ConstU32<512>> =
            BoundedVec::try_from(vec![1, 2, 3]).unwrap();

        assert_ok!(Liuyao::divine_by_coins_encrypted(
            RuntimeOrigin::signed(ALICE),
            1,
            Some(coins),
            Some((0, 0)),
            Some((2, 2)),
            Some((4, 4)),
            Some((6, 6)),
            Some(encrypted_data),
            Some([0u8; 32]),
            Some([0u8; 80]),
        ));

        // BOB 尝试更新
        let new_encrypted_data: BoundedVec<u8, ConstU32<512>> =
            BoundedVec::try_from(vec![9, 9, 9]).unwrap();

        assert_noop!(
            Liuyao::update_encrypted_data(
                RuntimeOrigin::signed(BOB),
                0,
                new_encrypted_data,
                [1u8; 32],
                [1u8; 80],
            ),
            Error::<Test>::NotGuaOwner
        );
    });
}

#[test]
fn test_privacy_mode_affects_visibility() {
    new_test_ext().execute_with(|| {
        use frame_support::{BoundedVec, traits::ConstU32};
        use pallet_divination_privacy::types::PrivacyMode;

        // 创建 Partial 模式卦象
        let encrypted_data: BoundedVec<u8, ConstU32<512>> =
            BoundedVec::try_from(vec![1, 2, 3]).unwrap();

        assert_ok!(Liuyao::divine_by_coins_encrypted(
            RuntimeOrigin::signed(ALICE),
            1,
            Some([1, 1, 1, 1, 1, 1]),
            Some((0, 0)),
            Some((2, 2)),
            Some((4, 4)),
            Some((6, 6)),
            Some(encrypted_data),
            Some([0u8; 32]),
            Some([0u8; 80]),
        ));

        let gua = Liuyao::guas(0).unwrap();
        assert_eq!(gua.privacy_mode, PrivacyMode::Partial);
        assert!(!gua.is_public());
        assert!(gua.can_interpret());

        // 尝试设为公开
        assert_ok!(Liuyao::set_gua_visibility(
            RuntimeOrigin::signed(ALICE),
            0,
            true,
        ));

        let gua = Liuyao::guas(0).unwrap();
        assert_eq!(gua.privacy_mode, PrivacyMode::Public);
        assert!(gua.is_public());
    });
}

// ============================================================================
// 算法测试
// ============================================================================

#[test]
fn test_trigram_properties() {
    assert_eq!(Trigram::Qian.binary(), 0b111);
    assert_eq!(Trigram::Kun.binary(), 0b000);
    assert_eq!(Trigram::Kan.binary(), 0b010);
    assert_eq!(Trigram::Li.binary(), 0b101);

    assert_eq!(Trigram::from_binary(0b111), Trigram::Qian);
    assert_eq!(Trigram::from_binary(0b000), Trigram::Kun);
}

#[test]
fn test_yao_properties() {
    assert!(Yao::ShaoYang.is_yang());
    assert!(Yao::LaoYang.is_yang());
    assert!(!Yao::ShaoYin.is_yang());
    assert!(!Yao::LaoYin.is_yang());

    assert!(Yao::LaoYin.is_moving());
    assert!(Yao::LaoYang.is_moving());
    assert!(!Yao::ShaoYin.is_moving());
    assert!(!Yao::ShaoYang.is_moving());

    // 老阴变阳，老阳变阴
    assert_eq!(Yao::LaoYin.changed_value(), 1);
    assert_eq!(Yao::LaoYang.changed_value(), 0);
}

#[test]
fn test_liu_qin_calculation() {
    // 同我者为兄弟
    assert_eq!(LiuQin::from_wu_xing(WuXing::Wood, WuXing::Wood), LiuQin::XiongDi);
    // 我生者为子孙
    assert_eq!(LiuQin::from_wu_xing(WuXing::Wood, WuXing::Fire), LiuQin::ZiSun);
    // 我克者为妻财
    assert_eq!(LiuQin::from_wu_xing(WuXing::Wood, WuXing::Earth), LiuQin::QiCai);
    // 克我者为官鬼
    assert_eq!(LiuQin::from_wu_xing(WuXing::Wood, WuXing::Metal), LiuQin::GuanGui);
    // 生我者为父母
    assert_eq!(LiuQin::from_wu_xing(WuXing::Wood, WuXing::Water), LiuQin::FuMu);
}

#[test]
fn test_liu_shen_calculation() {
    // 甲乙日起青龙
    let liu_shen = calculate_liu_shen(TianGan::Jia);
    assert_eq!(liu_shen[0], LiuShen::QingLong);
    assert_eq!(liu_shen[1], LiuShen::ZhuQue);
    assert_eq!(liu_shen[2], LiuShen::GouChen);

    // 丙丁日起朱雀
    let liu_shen = calculate_liu_shen(TianGan::Bing);
    assert_eq!(liu_shen[0], LiuShen::ZhuQue);
}

#[test]
fn test_xun_kong_calculation() {
    // 甲子旬空戌亥
    let (kong1, kong2) = calculate_xun_kong(TianGan::Jia, DiZhi::Zi);
    assert_eq!(kong1, DiZhi::Xu);
    assert_eq!(kong2, DiZhi::Hai);

    // 甲戌旬空申酉
    let (kong1, kong2) = calculate_xun_kong(TianGan::Jia, DiZhi::Xu);
    assert_eq!(kong1, DiZhi::Shen);
    assert_eq!(kong2, DiZhi::You);
}

#[test]
fn test_shi_ying_gong_calculation() {
    // 乾为天：本宫六世，卦宫乾
    let (gua_xu, gong) = calculate_shi_ying_gong(Trigram::Qian, Trigram::Qian);
    assert_eq!(gua_xu, GuaXu::BenGong);
    assert_eq!(gong, Trigram::Qian);

    // 坤为地：本宫六世，卦宫坤
    let (gua_xu, gong) = calculate_shi_ying_gong(Trigram::Kun, Trigram::Kun);
    assert_eq!(gua_xu, GuaXu::BenGong);
    assert_eq!(gong, Trigram::Kun);
}

#[test]
fn test_najia_inner() {
    // 乾卦内卦纳甲：甲子、甲寅、甲辰
    let (gan, zhi) = get_inner_najia(Trigram::Qian, 0);
    assert_eq!(gan, TianGan::Jia);
    assert_eq!(zhi, DiZhi::Zi);

    let (gan, zhi) = get_inner_najia(Trigram::Qian, 1);
    assert_eq!(gan, TianGan::Jia);
    assert_eq!(zhi, DiZhi::Yin);

    let (gan, zhi) = get_inner_najia(Trigram::Qian, 2);
    assert_eq!(gan, TianGan::Jia);
    assert_eq!(zhi, DiZhi::Chen);
}

#[test]
fn test_najia_outer() {
    // 乾卦外卦纳甲：壬午、壬申、壬戌
    let (gan, zhi) = get_outer_najia(Trigram::Qian, 0);
    assert_eq!(gan, TianGan::Ren);
    assert_eq!(zhi, DiZhi::Wu);

    let (gan, zhi) = get_outer_najia(Trigram::Qian, 1);
    assert_eq!(gan, TianGan::Ren);
    assert_eq!(zhi, DiZhi::Shen);

    let (gan, zhi) = get_outer_najia(Trigram::Qian, 2);
    assert_eq!(gan, TianGan::Ren);
    assert_eq!(zhi, DiZhi::Xu);
}

#[test]
fn test_coins_to_yaos() {
    let coins = [0, 1, 2, 3, 1, 2];
    let yaos = coins_to_yaos(&coins);

    assert_eq!(yaos[0], Yao::LaoYin);   // 0个阳面
    assert_eq!(yaos[1], Yao::ShaoYang); // 1个阳面
    assert_eq!(yaos[2], Yao::ShaoYin);  // 2个阳面
    assert_eq!(yaos[3], Yao::LaoYang);  // 3个阳面
}

#[test]
fn test_calculate_bian_gua() {
    // 有动爻
    let yaos = [Yao::ShaoYin, Yao::LaoYang, Yao::ShaoYin, Yao::ShaoYin, Yao::ShaoYin, Yao::ShaoYin];
    let (_inner, _outer, has_bian) = calculate_bian_gua(&yaos);
    assert!(has_bian);

    // 无动爻
    let yaos = [Yao::ShaoYin, Yao::ShaoYang, Yao::ShaoYin, Yao::ShaoYin, Yao::ShaoYin, Yao::ShaoYin];
    let (_, _, has_bian) = calculate_bian_gua(&yaos);
    assert!(!has_bian);
}

#[test]
fn test_moving_bitmap() {
    let yaos = [Yao::LaoYin, Yao::ShaoYang, Yao::LaoYang, Yao::ShaoYin, Yao::ShaoYin, Yao::LaoYin];
    let bitmap = calculate_moving_bitmap(&yaos);

    // 初爻、三爻、上爻动
    assert_eq!(bitmap, 0b100101);
}

// ============================================================================
// 类型测试
// ============================================================================

#[test]
fn test_wu_xing_relations() {
    // 木生火
    assert_eq!(WuXing::Wood.generates(), WuXing::Fire);
    // 木克土
    assert_eq!(WuXing::Wood.restrains(), WuXing::Earth);
}

#[test]
fn test_dizhi_wuxing() {
    assert_eq!(DiZhi::Zi.wu_xing(), WuXing::Water);
    assert_eq!(DiZhi::Yin.wu_xing(), WuXing::Wood);
    assert_eq!(DiZhi::Wu.wu_xing(), WuXing::Fire);
    assert_eq!(DiZhi::Shen.wu_xing(), WuXing::Metal);
    assert_eq!(DiZhi::Chou.wu_xing(), WuXing::Earth);
}

#[test]
fn test_gua_xu_shi_ying() {
    // 本宫卦世在六爻
    assert_eq!(GuaXu::BenGong.shi_yao_pos(), 6);
    assert_eq!(GuaXu::BenGong.ying_yao_pos(), 3);

    // 一世卦世在初爻
    assert_eq!(GuaXu::YiShi.shi_yao_pos(), 1);
    assert_eq!(GuaXu::YiShi.ying_yao_pos(), 4);

    // 游魂卦世在四爻
    assert_eq!(GuaXu::YouHun.shi_yao_pos(), 4);
    assert_eq!(GuaXu::YouHun.ying_yao_pos(), 1);

    // 归魂卦世在三爻
    assert_eq!(GuaXu::GuiHun.shi_yao_pos(), 3);
    assert_eq!(GuaXu::GuiHun.ying_yao_pos(), 6);
}

// ============================================================================
// 事件测试
// ============================================================================

#[test]
fn test_events_emitted() {
    new_test_ext().execute_with(|| {
        // 创建卦象
        assert_ok!(Liuyao::divine_random(RuntimeOrigin::signed(ALICE)));

        // 检查事件
        let gua = Liuyao::guas(0).unwrap();
        System::assert_has_event(RuntimeEvent::Liuyao(Event::GuaCreated {
            gua_id: 0,
            creator: ALICE,
            method: DivinationMethod::RandomMethod,
            original_name_idx: gua.original_name_idx,
        }));
    });
}

// ============================================================================
// 六十四卦索引测试
// ============================================================================

#[test]
fn test_gua_index_calculation() {
    use crate::types::gua64;

    // 乾为天: 内乾(7) 外乾(7) => (7<<3)|7 = 63
    assert_eq!(calculate_gua_index(Trigram::Qian, Trigram::Qian), 63);
    assert_eq!(gua64::GUA_NAMES[63], "乾为天");

    // 坤为地: 内坤(0) 外坤(0) => 0
    assert_eq!(calculate_gua_index(Trigram::Kun, Trigram::Kun), 0);
    assert_eq!(gua64::GUA_NAMES[0], "坤为地");

    // 地天泰: 内乾(7) 外坤(0) => (0<<3)|7 = 7
    assert_eq!(calculate_gua_index(Trigram::Qian, Trigram::Kun), 7);
    assert_eq!(gua64::GUA_NAMES[7], "地天泰");

    // 天地否: 内坤(0) 外乾(7) => (7<<3)|0 = 56
    assert_eq!(calculate_gua_index(Trigram::Kun, Trigram::Qian), 56);
    assert_eq!(gua64::GUA_NAMES[56], "天地否");

    // 震为雷: 内震(4) 外震(4) => (4<<3)|4 = 36
    assert_eq!(calculate_gua_index(Trigram::Zhen, Trigram::Zhen), 36);
    assert_eq!(gua64::GUA_NAMES[36], "震为雷");

    // 坎为水: 内坎(2) 外坎(2) => (2<<3)|2 = 18
    assert_eq!(calculate_gua_index(Trigram::Kan, Trigram::Kan), 18);
    assert_eq!(gua64::GUA_NAMES[18], "坎为水");
}

#[test]
fn test_gua_name_function() {
    use crate::types::gua64;

    assert_eq!(gua64::get_gua_name(0), "坤为地");
    assert_eq!(gua64::get_gua_name(63), "乾为天");
    assert_eq!(gua64::get_gua_name(7), "地天泰");
    assert_eq!(gua64::get_gua_name(56), "天地否");
}

// ============================================================================
// 六冲六合测试
// ============================================================================

#[test]
fn test_liu_chong_indices() {
    // 八纯卦都是六冲
    assert!(is_liu_chong_by_index(0));   // 坤为地
    assert!(is_liu_chong_by_index(9));   // 艮为山
    assert!(is_liu_chong_by_index(18));  // 坎为水
    assert!(is_liu_chong_by_index(27));  // 巽为风
    assert!(is_liu_chong_by_index(36));  // 震为雷
    assert!(is_liu_chong_by_index(45));  // 离为火
    assert!(is_liu_chong_by_index(54));  // 兑为泽
    assert!(is_liu_chong_by_index(63));  // 乾为天

    // 天雷无妄和雷天大壮也是六冲
    assert!(is_liu_chong_by_index(60));  // 天雷无妄
    assert!(is_liu_chong_by_index(39));  // 雷天大壮

    // 非六冲卦
    assert!(!is_liu_chong_by_index(7));   // 地天泰
    assert!(!is_liu_chong_by_index(56));  // 天地否
}

#[test]
fn test_liu_chong_by_trigrams() {
    // 纯卦为六冲
    assert!(is_liu_chong(Trigram::Qian, Trigram::Qian));
    assert!(is_liu_chong(Trigram::Kun, Trigram::Kun));

    // 天雷无妄: 外乾(7)内震(4)
    assert!(is_liu_chong(Trigram::Zhen, Trigram::Qian));
    // 雷天大壮: 外震(4)内乾(7)
    assert!(is_liu_chong(Trigram::Qian, Trigram::Zhen));

    // 非六冲
    assert!(!is_liu_chong(Trigram::Qian, Trigram::Kun));
}

#[test]
fn test_liu_he_indices() {
    // 六合卦
    assert!(is_liu_he(56));  // 天地否
    assert!(is_liu_he(7));   // 地天泰
    assert!(is_liu_he(50));  // 泽水困
    assert!(is_liu_he(22));  // 水泽节
    assert!(is_liu_he(13));  // 山火贲
    assert!(is_liu_he(4));   // 地雷复
    assert!(is_liu_he(41));  // 火山旅
    assert!(is_liu_he(32));  // 雷地豫

    // 非六合卦
    assert!(!is_liu_he(0));   // 坤为地
    assert!(!is_liu_he(63));  // 乾为天
}

// ============================================================================
// 互卦测试
// ============================================================================

#[test]
fn test_hu_gua_calculation() {
    // 乾为天：全阳爻，互卦也是乾为天
    let yaos = [Yao::ShaoYang; 6];
    let (inner, outer) = calculate_hu_gua(&yaos);
    assert_eq!(inner, Trigram::Qian);
    assert_eq!(outer, Trigram::Qian);

    // 坤为地：全阴爻，互卦也是坤为地
    let yaos = [Yao::ShaoYin; 6];
    let (inner, outer) = calculate_hu_gua(&yaos);
    assert_eq!(inner, Trigram::Kun);
    assert_eq!(outer, Trigram::Kun);
}

#[test]
fn test_hu_gua_index() {
    // 乾为天的互卦
    let yaos = [Yao::ShaoYang; 6];
    let hu_idx = calculate_hu_gua_index(&yaos);
    assert_eq!(hu_idx, 63); // 乾为天
}

// ============================================================================
// 卦身测试
// ============================================================================

#[test]
fn test_gua_shen_calculation() {
    // 世爻在初爻(1)，阳爻：从子数1位 => 子
    assert_eq!(calculate_gua_shen(1, true), DiZhi::Zi);

    // 世爻在二爻(2)，阳爻：从子数2位 => 丑
    assert_eq!(calculate_gua_shen(2, true), DiZhi::Chou);

    // 世爻在六爻(6)，阳爻：从子数6位 => 巳
    assert_eq!(calculate_gua_shen(6, true), DiZhi::Si);

    // 世爻在初爻(1)，阴爻：从午数1位 => 午
    assert_eq!(calculate_gua_shen(1, false), DiZhi::Wu);

    // 世爻在二爻(2)，阴爻：从午数2位 => 未
    assert_eq!(calculate_gua_shen(2, false), DiZhi::Wei);

    // 世爻在六爻(6)，阴爻：从午数6位 => 亥
    assert_eq!(calculate_gua_shen(6, false), DiZhi::Hai);
}

// ============================================================================
// 离为火纳甲验证测试
// ============================================================================

#[test]
fn test_najia_li_gua() {
    // 离为火：内外卦纳甲应为 己卯己丑己亥 己酉己未己巳
    // 参考 najia 项目测试: get_najia('101101') == ['己卯', '己丑', '己亥', '己酉', '己未', '己巳']

    // 内卦纳甲
    let (gan, zhi) = get_inner_najia(Trigram::Li, 0);
    assert_eq!(gan, TianGan::Ji);
    assert_eq!(zhi, DiZhi::Mao);

    let (gan, zhi) = get_inner_najia(Trigram::Li, 1);
    assert_eq!(gan, TianGan::Ji);
    assert_eq!(zhi, DiZhi::Chou);

    let (gan, zhi) = get_inner_najia(Trigram::Li, 2);
    assert_eq!(gan, TianGan::Ji);
    assert_eq!(zhi, DiZhi::Hai);

    // 外卦纳甲
    let (gan, zhi) = get_outer_najia(Trigram::Li, 0);
    assert_eq!(gan, TianGan::Ji);
    assert_eq!(zhi, DiZhi::You);

    let (gan, zhi) = get_outer_najia(Trigram::Li, 1);
    assert_eq!(gan, TianGan::Ji);
    assert_eq!(zhi, DiZhi::Wei);

    let (gan, zhi) = get_outer_najia(Trigram::Li, 2);
    assert_eq!(gan, TianGan::Ji);
    assert_eq!(zhi, DiZhi::Si);
}

// ============================================================================
// 神煞测试
// ============================================================================

#[test]
fn test_tian_yi_gui_ren() {
    use crate::shensha::*;

    // 甲戊庚牛羊 - 甲日贵人在丑未
    let gui_ren = calculate_tian_yi_gui_ren(TianGan::Jia);
    assert_eq!(gui_ren[0], DiZhi::Chou);
    assert_eq!(gui_ren[1], DiZhi::Wei);

    // 乙己鼠猴乡 - 乙日贵人在子申
    let gui_ren = calculate_tian_yi_gui_ren(TianGan::Yi);
    assert_eq!(gui_ren[0], DiZhi::Zi);
    assert_eq!(gui_ren[1], DiZhi::Shen);

    // 丙丁猪鸡位 - 丙日贵人在亥酉
    let gui_ren = calculate_tian_yi_gui_ren(TianGan::Bing);
    assert_eq!(gui_ren[0], DiZhi::Hai);
    assert_eq!(gui_ren[1], DiZhi::You);

    // 壬癸兔蛇藏 - 壬日贵人在卯巳
    let gui_ren = calculate_tian_yi_gui_ren(TianGan::Ren);
    assert_eq!(gui_ren[0], DiZhi::Mao);
    assert_eq!(gui_ren[1], DiZhi::Si);

    // 六辛逢马虎 - 辛日贵人在午寅
    let gui_ren = calculate_tian_yi_gui_ren(TianGan::Xin);
    assert_eq!(gui_ren[0], DiZhi::Wu);
    assert_eq!(gui_ren[1], DiZhi::Yin);
}

#[test]
fn test_yi_ma() {
    use crate::shensha::*;

    // 申子辰马在寅
    assert_eq!(calculate_yi_ma(DiZhi::Shen), DiZhi::Yin);
    assert_eq!(calculate_yi_ma(DiZhi::Zi), DiZhi::Yin);
    assert_eq!(calculate_yi_ma(DiZhi::Chen), DiZhi::Yin);

    // 寅午戌马在申
    assert_eq!(calculate_yi_ma(DiZhi::Yin), DiZhi::Shen);
    assert_eq!(calculate_yi_ma(DiZhi::Wu), DiZhi::Shen);
    assert_eq!(calculate_yi_ma(DiZhi::Xu), DiZhi::Shen);

    // 巳酉丑马在亥
    assert_eq!(calculate_yi_ma(DiZhi::Si), DiZhi::Hai);
    assert_eq!(calculate_yi_ma(DiZhi::You), DiZhi::Hai);
    assert_eq!(calculate_yi_ma(DiZhi::Chou), DiZhi::Hai);

    // 亥卯未马在巳
    assert_eq!(calculate_yi_ma(DiZhi::Hai), DiZhi::Si);
    assert_eq!(calculate_yi_ma(DiZhi::Mao), DiZhi::Si);
    assert_eq!(calculate_yi_ma(DiZhi::Wei), DiZhi::Si);
}

#[test]
fn test_tao_hua() {
    use crate::shensha::*;

    // 申子辰桃花在酉
    assert_eq!(calculate_tao_hua(DiZhi::Zi), DiZhi::You);

    // 寅午戌桃花在卯
    assert_eq!(calculate_tao_hua(DiZhi::Wu), DiZhi::Mao);

    // 巳酉丑桃花在午
    assert_eq!(calculate_tao_hua(DiZhi::You), DiZhi::Wu);

    // 亥卯未桃花在子
    assert_eq!(calculate_tao_hua(DiZhi::Mao), DiZhi::Zi);
}

#[test]
fn test_lu_shen() {
    use crate::shensha::*;

    // 甲禄在寅
    assert_eq!(calculate_lu_shen(TianGan::Jia), DiZhi::Yin);
    // 乙禄在卯
    assert_eq!(calculate_lu_shen(TianGan::Yi), DiZhi::Mao);
    // 丙戊禄在巳
    assert_eq!(calculate_lu_shen(TianGan::Bing), DiZhi::Si);
    assert_eq!(calculate_lu_shen(TianGan::Wu), DiZhi::Si);
    // 丁己禄在午
    assert_eq!(calculate_lu_shen(TianGan::Ding), DiZhi::Wu);
    assert_eq!(calculate_lu_shen(TianGan::Ji), DiZhi::Wu);
    // 庚禄在申
    assert_eq!(calculate_lu_shen(TianGan::Geng), DiZhi::Shen);
    // 辛禄在酉
    assert_eq!(calculate_lu_shen(TianGan::Xin), DiZhi::You);
    // 壬禄在亥
    assert_eq!(calculate_lu_shen(TianGan::Ren), DiZhi::Hai);
    // 癸禄在子
    assert_eq!(calculate_lu_shen(TianGan::Gui), DiZhi::Zi);
}

#[test]
fn test_wen_chang() {
    use crate::shensha::*;

    // 甲乙巳午报君知
    assert_eq!(calculate_wen_chang(TianGan::Jia), DiZhi::Si);
    assert_eq!(calculate_wen_chang(TianGan::Yi), DiZhi::Wu);
    // 丙戊申宫丁己鸡
    assert_eq!(calculate_wen_chang(TianGan::Bing), DiZhi::Shen);
    assert_eq!(calculate_wen_chang(TianGan::Wu), DiZhi::Shen);
    assert_eq!(calculate_wen_chang(TianGan::Ding), DiZhi::You);
    assert_eq!(calculate_wen_chang(TianGan::Ji), DiZhi::You);
    // 庚猪辛鼠壬逢虎
    assert_eq!(calculate_wen_chang(TianGan::Geng), DiZhi::Hai);
    assert_eq!(calculate_wen_chang(TianGan::Xin), DiZhi::Zi);
    assert_eq!(calculate_wen_chang(TianGan::Ren), DiZhi::Yin);
    // 癸人见卯入云梯
    assert_eq!(calculate_wen_chang(TianGan::Gui), DiZhi::Mao);
}

#[test]
fn test_hua_gai() {
    use crate::shensha::*;

    // 申子辰见辰
    assert_eq!(calculate_hua_gai(DiZhi::Zi), DiZhi::Chen);
    // 寅午戌见戌
    assert_eq!(calculate_hua_gai(DiZhi::Wu), DiZhi::Xu);
    // 巳酉丑见丑
    assert_eq!(calculate_hua_gai(DiZhi::You), DiZhi::Chou);
    // 亥卯未见未
    assert_eq!(calculate_hua_gai(DiZhi::Mao), DiZhi::Wei);
}

#[test]
fn test_jiang_xing() {
    use crate::shensha::*;

    // 申子辰见子
    assert_eq!(calculate_jiang_xing(DiZhi::Zi), DiZhi::Zi);
    // 寅午戌见午
    assert_eq!(calculate_jiang_xing(DiZhi::Wu), DiZhi::Wu);
    // 巳酉丑见酉
    assert_eq!(calculate_jiang_xing(DiZhi::You), DiZhi::You);
    // 亥卯未见卯
    assert_eq!(calculate_jiang_xing(DiZhi::Mao), DiZhi::Mao);
}

#[test]
fn test_jie_sha() {
    use crate::shensha::*;

    // 申子辰见巳为劫
    assert_eq!(calculate_jie_sha(DiZhi::Zi), DiZhi::Si);
    // 亥卯未见申为劫
    assert_eq!(calculate_jie_sha(DiZhi::Mao), DiZhi::Shen);
    // 寅午戌见亥为劫
    assert_eq!(calculate_jie_sha(DiZhi::Wu), DiZhi::Hai);
    // 巳酉丑见寅为劫
    assert_eq!(calculate_jie_sha(DiZhi::You), DiZhi::Yin);
}

#[test]
fn test_all_shen_sha() {
    use crate::shensha::*;

    // 测试甲子日寅月所有神煞
    let info = calculate_all_shen_sha(TianGan::Jia, DiZhi::Zi, DiZhi::Yin);

    // 天乙贵人在丑未
    assert_eq!(info.tian_yi_gui_ren, [DiZhi::Chou, DiZhi::Wei]);
    // 驿马在寅
    assert_eq!(info.yi_ma, DiZhi::Yin);
    // 桃花在酉
    assert_eq!(info.tao_hua, DiZhi::You);
    // 禄神在寅
    assert_eq!(info.lu_shen, DiZhi::Yin);
    // 文昌在巳
    assert_eq!(info.wen_chang, DiZhi::Si);
    // 劫煞在巳
    assert_eq!(info.jie_sha, DiZhi::Si);
    // 华盖在辰
    assert_eq!(info.hua_gai, DiZhi::Chen);
    // 将星在子
    assert_eq!(info.jiang_xing, DiZhi::Zi);
    // 天喜（寅月）在戌
    assert_eq!(info.tian_xi, DiZhi::Xu);
    // 天医（寅月前一位）在丑
    assert_eq!(info.tian_yi, DiZhi::Chou);
    // 阳刃（甲日）在卯
    assert_eq!(info.yang_ren, DiZhi::Mao);
    // 灾煞（子日）在午
    assert_eq!(info.zai_sha, DiZhi::Wu);
    // 谋星（子日）在戌
    assert_eq!(info.mou_xing, DiZhi::Xu);
}

#[test]
fn test_get_shen_sha_for_zhi() {
    use crate::shensha::*;

    // 甲子日寅月查询寅的神煞
    // 寅是驿马、禄神
    let shen_sha = get_shen_sha_for_zhi(TianGan::Jia, DiZhi::Zi, DiZhi::Yin, DiZhi::Yin);

    // 验证驿马和禄神都存在
    let has_yi_ma = shen_sha.iter().any(|s| *s == Some(ShenSha::YiMa));
    let has_lu_shen = shen_sha.iter().any(|s| *s == Some(ShenSha::LuShen));

    assert!(has_yi_ma, "甲子日寅应该是驿马");
    assert!(has_lu_shen, "甲子日寅应该是禄神");
}

#[test]
fn test_shen_sha_properties() {
    use crate::shensha::*;

    // 测试吉神判断
    assert!(ShenSha::TianYiGuiRen.is_auspicious());
    assert!(ShenSha::LuShen.is_auspicious());
    assert!(ShenSha::WenChang.is_auspicious());
    assert!(ShenSha::JiangXing.is_auspicious());
    assert!(ShenSha::TianXi.is_auspicious());
    assert!(ShenSha::TianYi.is_auspicious());

    // 测试凶煞判断
    assert!(ShenSha::JieSha.is_inauspicious());
    assert!(ShenSha::WangShen.is_inauspicious());
    assert!(ShenSha::YangRen.is_inauspicious());
    assert!(ShenSha::ZaiSha.is_inauspicious());

    // 中性神煞
    assert!(!ShenSha::YiMa.is_auspicious());
    assert!(!ShenSha::YiMa.is_inauspicious());
    assert!(!ShenSha::TaoHua.is_auspicious());
    assert!(!ShenSha::HuaGai.is_auspicious());
    assert!(!ShenSha::MouXing.is_auspicious());
}

// ============================================================================
// 新增神煞测试
// ============================================================================

#[test]
fn test_tian_xi() {
    use crate::shensha::*;

    // 春天（寅卯辰月）天喜在戌
    assert_eq!(calculate_tian_xi(DiZhi::Yin), DiZhi::Xu);
    assert_eq!(calculate_tian_xi(DiZhi::Mao), DiZhi::Xu);
    assert_eq!(calculate_tian_xi(DiZhi::Chen), DiZhi::Xu);

    // 夏天（巳午未月）天喜在丑
    assert_eq!(calculate_tian_xi(DiZhi::Si), DiZhi::Chou);
    assert_eq!(calculate_tian_xi(DiZhi::Wu), DiZhi::Chou);
    assert_eq!(calculate_tian_xi(DiZhi::Wei), DiZhi::Chou);

    // 秋天（申酉戌月）天喜在辰
    assert_eq!(calculate_tian_xi(DiZhi::Shen), DiZhi::Chen);
    assert_eq!(calculate_tian_xi(DiZhi::You), DiZhi::Chen);
    assert_eq!(calculate_tian_xi(DiZhi::Xu), DiZhi::Chen);

    // 冬天（亥子丑月）天喜在未
    assert_eq!(calculate_tian_xi(DiZhi::Hai), DiZhi::Wei);
    assert_eq!(calculate_tian_xi(DiZhi::Zi), DiZhi::Wei);
    assert_eq!(calculate_tian_xi(DiZhi::Chou), DiZhi::Wei);
}

#[test]
fn test_tian_yi_shensha() {
    use crate::shensha::*;

    // 天医为月支前一位
    assert_eq!(calculate_tian_yi(DiZhi::Yin), DiZhi::Chou);  // 寅月天医在丑
    assert_eq!(calculate_tian_yi(DiZhi::Zi), DiZhi::Hai);    // 子月天医在亥
    assert_eq!(calculate_tian_yi(DiZhi::Wu), DiZhi::Si);     // 午月天医在巳
}

#[test]
fn test_yang_ren() {
    use crate::shensha::*;

    // 甲刃在卯
    assert_eq!(calculate_yang_ren(TianGan::Jia), DiZhi::Mao);
    // 丙戊刃在午
    assert_eq!(calculate_yang_ren(TianGan::Bing), DiZhi::Wu);
    assert_eq!(calculate_yang_ren(TianGan::Wu), DiZhi::Wu);
    // 庚刃在酉
    assert_eq!(calculate_yang_ren(TianGan::Geng), DiZhi::You);
    // 壬刃在子
    assert_eq!(calculate_yang_ren(TianGan::Ren), DiZhi::Zi);
}

#[test]
fn test_zai_sha() {
    use crate::shensha::*;

    // 申子辰日灾煞在午
    assert_eq!(calculate_zai_sha(DiZhi::Zi), DiZhi::Wu);
    assert_eq!(calculate_zai_sha(DiZhi::Shen), DiZhi::Wu);
    assert_eq!(calculate_zai_sha(DiZhi::Chen), DiZhi::Wu);

    // 寅午戌日灾煞在子
    assert_eq!(calculate_zai_sha(DiZhi::Yin), DiZhi::Zi);
    assert_eq!(calculate_zai_sha(DiZhi::Wu), DiZhi::Zi);
    assert_eq!(calculate_zai_sha(DiZhi::Xu), DiZhi::Zi);
}

#[test]
fn test_mou_xing() {
    use crate::shensha::*;

    // 申子辰日谋星在戌
    assert_eq!(calculate_mou_xing(DiZhi::Zi), DiZhi::Xu);
    // 寅午戌日谋星在辰
    assert_eq!(calculate_mou_xing(DiZhi::Wu), DiZhi::Chen);
    // 巳酉丑日谋星在未
    assert_eq!(calculate_mou_xing(DiZhi::You), DiZhi::Wei);
    // 亥卯未日谋星在丑
    assert_eq!(calculate_mou_xing(DiZhi::Mao), DiZhi::Chou);
}

// ============================================================================
// 旺衰测试
// ============================================================================

#[test]
fn test_wang_shuai_calculation() {
    use crate::algorithm::*;

    // 寅月木旺
    assert_eq!(calculate_wang_shuai(WuXing::Wood, DiZhi::Yin), WangShuai::Wang);  // 木旺
    assert_eq!(calculate_wang_shuai(WuXing::Fire, DiZhi::Yin), WangShuai::Xiang); // 火相（木生火）
    assert_eq!(calculate_wang_shuai(WuXing::Water, DiZhi::Yin), WangShuai::Xiu);  // 水休（水生木）
    assert_eq!(calculate_wang_shuai(WuXing::Metal, DiZhi::Yin), WangShuai::Qiu);  // 金囚（金克木）
    assert_eq!(calculate_wang_shuai(WuXing::Earth, DiZhi::Yin), WangShuai::Si);   // 土死（木克土）

    // 午月火旺
    assert_eq!(calculate_wang_shuai(WuXing::Fire, DiZhi::Wu), WangShuai::Wang);   // 火旺
    assert_eq!(calculate_wang_shuai(WuXing::Earth, DiZhi::Wu), WangShuai::Xiang); // 土相

    // 子月水旺
    assert_eq!(calculate_wang_shuai(WuXing::Water, DiZhi::Zi), WangShuai::Wang);  // 水旺
    assert_eq!(calculate_wang_shuai(WuXing::Wood, DiZhi::Zi), WangShuai::Xiang);  // 木相
}

#[test]
fn test_wang_shuai_properties() {
    use crate::algorithm::*;

    assert!(WangShuai::Wang.is_strong());
    assert!(WangShuai::Xiang.is_strong());
    assert!(WangShuai::Xiu.is_weak());
    assert!(WangShuai::Qiu.is_weak());
    assert!(WangShuai::Si.is_weak());
}

// ============================================================================
// 地支冲合测试
// ============================================================================

#[test]
fn test_di_zhi_chong() {
    use crate::algorithm::*;

    // 子午冲
    assert!(is_di_zhi_chong(DiZhi::Zi, DiZhi::Wu));
    assert!(is_di_zhi_chong(DiZhi::Wu, DiZhi::Zi));
    // 卯酉冲
    assert!(is_di_zhi_chong(DiZhi::Mao, DiZhi::You));
    // 寅申冲
    assert!(is_di_zhi_chong(DiZhi::Yin, DiZhi::Shen));
    // 非冲
    assert!(!is_di_zhi_chong(DiZhi::Zi, DiZhi::Yin));
}

#[test]
fn test_di_zhi_he() {
    use crate::algorithm::*;

    // 子丑合土
    assert_eq!(is_di_zhi_he(DiZhi::Zi, DiZhi::Chou), Some(WuXing::Earth));
    // 寅亥合木
    assert_eq!(is_di_zhi_he(DiZhi::Yin, DiZhi::Hai), Some(WuXing::Wood));
    // 卯戌合火
    assert_eq!(is_di_zhi_he(DiZhi::Mao, DiZhi::Xu), Some(WuXing::Fire));
    // 非合
    assert_eq!(is_di_zhi_he(DiZhi::Zi, DiZhi::Yin), None);
}

#[test]
fn test_get_chong_zhi() {
    use crate::algorithm::*;

    assert_eq!(get_chong_zhi(DiZhi::Zi), DiZhi::Wu);
    assert_eq!(get_chong_zhi(DiZhi::Chou), DiZhi::Wei);
    assert_eq!(get_chong_zhi(DiZhi::Yin), DiZhi::Shen);
}

#[test]
fn test_ri_chen_guanxi() {
    use crate::algorithm::*;

    // 子日冲午
    assert_eq!(analyze_ri_chen(DiZhi::Zi, DiZhi::Wu, WuXing::Fire), RiChenGuanXi::RiChong);
    // 子日合丑
    assert_eq!(analyze_ri_chen(DiZhi::Zi, DiZhi::Chou, WuXing::Earth), RiChenGuanXi::RiHe);
    // 子日生寅（水生木）
    assert_eq!(analyze_ri_chen(DiZhi::Zi, DiZhi::Yin, WuXing::Wood), RiChenGuanXi::RiSheng);
}

// ============================================================================
// 动爻作用测试
// ============================================================================

#[test]
fn test_dong_jing_zuoyong() {
    use crate::algorithm::*;

    // 木动生火静
    assert_eq!(calculate_dong_jing_zuoyong(WuXing::Wood, WuXing::Fire), DongYaoZuoYong::DongShengJing);
    // 金动克木静
    assert_eq!(calculate_dong_jing_zuoyong(WuXing::Metal, WuXing::Wood), DongYaoZuoYong::DongKeJing);
    // 同五行比和
    assert_eq!(calculate_dong_jing_zuoyong(WuXing::Water, WuXing::Water), DongYaoZuoYong::BiHe);
}

#[test]
fn test_hui_tou_zuoyong() {
    use crate::algorithm::*;

    // 变爻生本爻（水变生木本）
    assert_eq!(calculate_hui_tou(WuXing::Wood, WuXing::Water), HuiTouZuoYong::HuiTouSheng);
    // 变爻克本爻（金变克木本）
    assert_eq!(calculate_hui_tou(WuXing::Wood, WuXing::Metal), HuiTouZuoYong::HuiTouKe);
    // 本爻生变爻（木本生火变）
    assert_eq!(calculate_hui_tou(WuXing::Wood, WuXing::Fire), HuiTouZuoYong::HuiTouXie);
}

// ============================================================================
// 反吟伏吟测试
// ============================================================================

#[test]
fn test_fan_yin() {
    use crate::algorithm::*;

    // 乾变坤是反吟（内外卦都相冲）
    assert!(is_fan_yin(Trigram::Qian, Trigram::Qian, Trigram::Kun, Trigram::Kun));
    // 非反吟
    assert!(!is_fan_yin(Trigram::Qian, Trigram::Qian, Trigram::Qian, Trigram::Kun));
}

#[test]
fn test_fu_yin() {
    use crate::algorithm::*;

    // 本变相同是伏吟
    assert!(is_fu_yin(Trigram::Qian, Trigram::Qian, Trigram::Qian, Trigram::Qian));
    // 非伏吟
    assert!(!is_fu_yin(Trigram::Qian, Trigram::Qian, Trigram::Kun, Trigram::Kun));
}

// ============================================================================
// 床帐香闺测试
// ============================================================================

#[test]
fn test_chuang_zhang() {
    use crate::algorithm::*;

    // 卦身在子（水），床帐在木（寅卯）
    let cz = calculate_chuang_zhang(DiZhi::Zi);
    assert_eq!(cz[0], DiZhi::Yin);
    assert_eq!(cz[1], DiZhi::Mao);
}

#[test]
fn test_xiang_gui() {
    use crate::algorithm::*;

    // 卦身在子（水），香闺在火（巳午）
    let xg = calculate_xiang_gui(DiZhi::Zi);
    assert_eq!(xg[0], DiZhi::Si);
    assert_eq!(xg[1], DiZhi::Wu);
}
