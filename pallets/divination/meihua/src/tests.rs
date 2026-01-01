//! 梅花易数 Pallet 单元测试
//!
//! 测试核心功能：
//! - 时间起卦
//! - 双数起卦
//! - 随机起卦
//! - 手动起卦
//! - 单数起卦
//! - AI 解卦请求
//! - 卦象公开状态管理

#![allow(deprecated)]

use crate::{mock::*, Error, pallet::Event};
use crate::types::{Bagua, DivinationMethod};
use frame_support::{assert_noop, assert_ok, BoundedVec};

/// 测试时间起卦基本功能
#[test]
fn divine_by_time_works() {
    new_test_ext().execute_with(|| {
        let question_hash = [0u8; 32];

        // 执行时间起卦
        assert_ok!(Meihua::divine_by_time(
            RuntimeOrigin::signed(1),
            question_hash,
            false,
            1, // 男
            1  // 事业
        ));

        // 验证卦象创建
        assert_eq!(Meihua::next_hexagram_id(), 1);

        // 验证卦象存储
        let hexagram = Meihua::hexagrams(0).expect("Hexagram should exist");
        assert_eq!(hexagram.ben_gua.id, 0);
        assert_eq!(hexagram.ben_gua.diviner, 1);
        assert!(!hexagram.ben_gua.is_public);

        // 验证用户索引
        let user_hexagrams = Meihua::user_hexagrams(1);
        assert_eq!(user_hexagrams.len(), 1);
        assert_eq!(user_hexagrams[0], 0);

        // 验证解卦数据已创建
        let interpretation = Meihua::get_interpretation_data(0).expect("Interpretation should exist");
        assert_eq!(interpretation.basic_info.gender, 1);
        assert_eq!(interpretation.basic_info.category, 1);
    });
}

/// 测试双数起卦
#[test]
fn divine_by_numbers_works() {
    new_test_ext().execute_with(|| {
        let question_hash = [1u8; 32];

        assert_ok!(Meihua::divine_by_numbers(
            RuntimeOrigin::signed(2),
            88,
            66,
            question_hash,
            true,
            2, // 女
            2  // 财运
        ));

        // 验证公开卦象
        let hexagram = Meihua::hexagrams(0).unwrap();
        assert!(hexagram.ben_gua.is_public);

        // 验证公开列表
        let public_list = Meihua::public_hexagrams();
        assert_eq!(public_list.len(), 1);

        // 验证解卦数据
        let interpretation = Meihua::get_interpretation_data(0).expect("Interpretation should exist");
        assert_eq!(interpretation.basic_info.gender, 2);
        assert_eq!(interpretation.basic_info.category, 2);
    });
}

/// 测试随机起卦
#[test]
fn divine_random_works() {
    new_test_ext().execute_with(|| {
        let question_hash = [2u8; 32];

        assert_ok!(Meihua::divine_random(
            RuntimeOrigin::signed(3),
            question_hash,
            false,
            0, // 未指定
            0  // 未指定
        ));

        let hexagram = Meihua::hexagrams(0).unwrap();
        assert_eq!(hexagram.ben_gua.diviner, 3);

        // 验证卦数有效（1-8）
        assert!(hexagram.ben_gua.shang_gua.number() >= 1 && hexagram.ben_gua.shang_gua.number() <= 8);
        assert!(hexagram.ben_gua.xia_gua.number() >= 1 && hexagram.ben_gua.xia_gua.number() <= 8);
        // 验证动爻有效（1-6）
        assert!(hexagram.ben_gua.dong_yao >= 1 && hexagram.ben_gua.dong_yao <= 6);

        // 验证解卦数据
        let interpretation = Meihua::get_interpretation_data(0).expect("Interpretation should exist");
        assert_eq!(interpretation.basic_info.gender, 0);
        assert_eq!(interpretation.basic_info.category, 0);
    });
}

/// 测试手动起卦
#[test]
fn divine_manual_works() {
    new_test_ext().execute_with(|| {
        let question_hash = [3u8; 32];

        // 手动指定：乾上坤下，动爻 3
        assert_ok!(Meihua::divine_manual(
            RuntimeOrigin::signed(1),
            1, // 乾
            8, // 坤
            3, // 第三爻
            question_hash,
            false,
            1, // 男
            3  // 感情
        ));

        let hexagram = Meihua::hexagrams(0).unwrap();

        // 验证卦象
        assert_eq!(hexagram.ben_gua.shang_gua.bagua, Bagua::Qian);
        assert_eq!(hexagram.ben_gua.xia_gua.bagua, Bagua::Kun);
        assert_eq!(hexagram.ben_gua.dong_yao, 3);

        // 动爻 3 在下卦，下卦为用，上卦为体
        assert!(hexagram.ben_gua.ti_is_shang);

        // 验证解卦数据
        let interpretation = Meihua::get_interpretation_data(0).expect("Interpretation should exist");
        assert_eq!(interpretation.basic_info.category, 3);
    });
}

/// 测试无效参数
#[test]
fn divine_manual_invalid_params() {
    new_test_ext().execute_with(|| {
        let question_hash = [0u8; 32];

        // 无效卦数
        assert_noop!(
            Meihua::divine_manual(
                RuntimeOrigin::signed(1),
                0, // 无效
                1,
                1,
                question_hash,
                false,
                1,
                1
            ),
            Error::<Test>::InvalidGuaNum
        );

        // 无效动爻
        assert_noop!(
            Meihua::divine_manual(
                RuntimeOrigin::signed(1),
                1,
                1,
                7, // 无效，应为 1-6
                question_hash,
                false,
                1,
                1
            ),
            Error::<Test>::InvalidDongYao
        );
    });
}

/// 测试每日限制
#[test]
fn daily_limit_works() {
    new_test_ext().execute_with(|| {
        let question_hash = [0u8; 32];

        // 连续起卦直到达到限制
        for _ in 0..50 {
            assert_ok!(Meihua::divine_random(
                RuntimeOrigin::signed(1),
                question_hash,
                false,
                0,
                0
            ));
        }

        // 第 51 次应该失败
        assert_noop!(
            Meihua::divine_random(RuntimeOrigin::signed(1), question_hash, false, 0, 0),
            Error::<Test>::DailyLimitExceeded
        );

        // 不同用户不受影响
        assert_ok!(Meihua::divine_random(
            RuntimeOrigin::signed(2),
            question_hash,
            false,
            0,
            0
        ));
    });
}

/// 测试卦象公开状态切换
#[test]
fn set_visibility_works() {
    new_test_ext().execute_with(|| {
        let question_hash = [0u8; 32];

        // 创建私密卦象
        assert_ok!(Meihua::divine_random(
            RuntimeOrigin::signed(1),
            question_hash,
            false,
            0,
            0
        ));

        // 设置为公开
        assert_ok!(Meihua::set_hexagram_visibility(
            RuntimeOrigin::signed(1),
            0,
            true
        ));

        // 验证
        let hexagram = Meihua::hexagrams(0).unwrap();
        assert!(hexagram.ben_gua.is_public);
        assert_eq!(Meihua::public_hexagrams().len(), 1);

        // 设置为私密
        assert_ok!(Meihua::set_hexagram_visibility(
            RuntimeOrigin::signed(1),
            0,
            false
        ));

        let hexagram = Meihua::hexagrams(0).unwrap();
        assert!(!hexagram.ben_gua.is_public);
        assert_eq!(Meihua::public_hexagrams().len(), 0);
    });
}

/// 测试非所有者无法修改
#[test]
fn not_owner_cannot_modify() {
    new_test_ext().execute_with(|| {
        let question_hash = [0u8; 32];

        // 用户 1 创建卦象
        assert_ok!(Meihua::divine_random(
            RuntimeOrigin::signed(1),
            question_hash,
            false,
            0,
            0
        ));

        // 用户 2 尝试修改
        assert_noop!(
            Meihua::set_hexagram_visibility(RuntimeOrigin::signed(2), 0, true),
            Error::<Test>::NotOwner
        );
    });
}

/// 测试 AI 解卦请求
#[test]
fn ai_interpretation_request_works() {
    new_test_ext().execute_with(|| {
        let question_hash = [0u8; 32];

        // 创建卦象
        assert_ok!(Meihua::divine_random(
            RuntimeOrigin::signed(1),
            question_hash,
            false,
            1,
            1
        ));

        // 请求 AI 解卦
        assert_ok!(Meihua::request_ai_interpretation(
            RuntimeOrigin::signed(1),
            0
        ));

        // 验证请求已记录
        assert!(Meihua::ai_interpretation_requests(0).is_some());

        // 不能重复请求
        assert_noop!(
            Meihua::request_ai_interpretation(RuntimeOrigin::signed(1), 0),
            Error::<Test>::AiRequestAlreadyExists
        );
    });
}

/// 测试 AI 解卦结果提交
#[test]
fn ai_interpretation_submit_works() {
    new_test_ext().execute_with(|| {
        let question_hash = [0u8; 32];

        // 创建卦象并请求 AI 解卦
        assert_ok!(Meihua::divine_random(
            RuntimeOrigin::signed(1),
            question_hash,
            false,
            1,
            1
        ));
        assert_ok!(Meihua::request_ai_interpretation(
            RuntimeOrigin::signed(1),
            0
        ));

        // AI 预言机提交结果（使用账户 1 模拟预言机）
        let cid: BoundedVec<u8, frame_support::traits::ConstU32<64>> =
            BoundedVec::try_from(b"QmTestCid123456789".to_vec()).unwrap();

        assert_ok!(Meihua::submit_ai_interpretation(
            RuntimeOrigin::signed(1), // 模拟预言机账户
            0,
            cid.clone()
        ));

        // 验证结果已存储
        let hexagram = Meihua::hexagrams(0).unwrap();
        assert_eq!(hexagram.ben_gua.interpretation_cid, Some(cid));

        // 请求已被移除
        assert!(Meihua::ai_interpretation_requests(0).is_none());
    });
}

/// 测试完整卦象计算
#[test]
fn full_divination_calculation() {
    new_test_ext().execute_with(|| {
        let question_hash = [0u8; 32];

        // 手动指定一个已知卦象进行验证
        // 火雷噬嗑：离上(3)震下(4)，动爻 6
        assert_ok!(Meihua::divine_manual(
            RuntimeOrigin::signed(1),
            3, // 离
            4, // 震
            6, // 第六爻
            question_hash,
            false,
            1,
            1
        ));

        let hexagram = Meihua::hexagrams(0).unwrap();

        // 验证本卦
        assert_eq!(hexagram.ben_gua.shang_gua.bagua, Bagua::Li);
        assert_eq!(hexagram.ben_gua.xia_gua.bagua, Bagua::Zhen);

        // 动爻 6 在上卦，上卦为用，下卦为体
        assert!(!hexagram.ben_gua.ti_is_shang);

        // 验证变卦（离 101 翻转第6位 -> 001 震）
        assert_eq!(hexagram.bian_gua.0.bagua, Bagua::Zhen); // 变卦上卦
        assert_eq!(hexagram.bian_gua.1.bagua, Bagua::Zhen); // 变卦下卦不变

        // 验证互卦
        // 本卦二进制：上卦 101，下卦 001 -> 101001
        // 互卦上卦取 5,4,3 爻 -> 010 -> 坎
        // 互卦下卦取 4,3,2 爻 -> 100 -> 艮
        assert_eq!(hexagram.hu_gua.0.bagua, Bagua::Kan);
        assert_eq!(hexagram.hu_gua.1.bagua, Bagua::Gen);
    });
}

/// 测试事件发送
#[test]
fn events_are_emitted() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let question_hash = [0u8; 32];

        assert_ok!(Meihua::divine_random(
            RuntimeOrigin::signed(1),
            question_hash,
            false,
            0,
            0
        ));

        // 检查事件
        System::assert_has_event(
            Event::<Test>::HexagramCreated {
                hexagram_id: 0,
                diviner: 1,
                method: DivinationMethod::Random,
            }
            .into(),
        );
    });
}

/// 测试单数起卦功能
#[test]
fn divine_by_single_number_works() {
    new_test_ext().execute_with(|| {
        let question_hash = [0u8; 32];

        // 使用数字 38271 起卦
        // 算法：前半 3+8=11，后半 2+7+1=10
        // 上卦 = 11 % 8 = 3（离）
        // 下卦 = 10 % 8 = 2（兑）
        assert_ok!(Meihua::divine_by_single_number(
            RuntimeOrigin::signed(1),
            38271,
            question_hash,
            false,
            1,
            1
        ));

        // 验证卦象创建
        assert_eq!(Meihua::next_hexagram_id(), 1);

        let hexagram = Meihua::hexagrams(0).unwrap();

        // 验证上下卦
        assert_eq!(hexagram.ben_gua.shang_gua.bagua, Bagua::Li);  // 3 = 离
        assert_eq!(hexagram.ben_gua.xia_gua.bagua, Bagua::Dui);   // 2 = 兑

        // 验证起卦方式
        assert_eq!(hexagram.ben_gua.method, DivinationMethod::SingleNumber);
    });
}

/// 测试单数起卦 - 两位数
#[test]
fn divine_by_single_number_two_digits() {
    new_test_ext().execute_with(|| {
        let question_hash = [0u8; 32];

        // 使用数字 36 起卦
        // 前半 3，后半 6
        // 上卦 = 3（离），下卦 = 6（坎）
        assert_ok!(Meihua::divine_by_single_number(
            RuntimeOrigin::signed(1),
            36,
            question_hash,
            false,
            1,
            1
        ));

        let hexagram = Meihua::hexagrams(0).unwrap();
        assert_eq!(hexagram.ben_gua.shang_gua.bagua, Bagua::Li);  // 3 = 离
        assert_eq!(hexagram.ben_gua.xia_gua.bagua, Bagua::Kan);   // 6 = 坎
    });
}

/// 测试单数起卦 - 事件发送
#[test]
fn divine_by_single_number_emits_event() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let question_hash = [0u8; 32];

        assert_ok!(Meihua::divine_by_single_number(
            RuntimeOrigin::signed(1),
            12345,
            question_hash,
            true,
            1,
            1
        ));

        // 检查事件
        System::assert_has_event(
            Event::<Test>::HexagramCreated {
                hexagram_id: 0,
                diviner: 1,
                method: DivinationMethod::SingleNumber,
            }
            .into(),
        );
    });
}

/// 测试卦象详细信息 API
#[test]
fn get_hexagram_detail_works() {
    new_test_ext().execute_with(|| {
        let question_hash = [0u8; 32];

        // 创建一个卦象
        assert_ok!(Meihua::divine_manual(
            RuntimeOrigin::signed(1),
            1, // 乾
            1, // 乾
            1, // 初爻
            question_hash,
            false,
            1,
            1
        ));

        // 获取详细信息
        let detail = Meihua::get_hexagram_detail(0);
        assert!(detail.is_some());

        let detail = detail.unwrap();

        // 验证本卦名称包含"乾"
        let name_str = core::str::from_utf8(&detail.ben_gua.name).unwrap();
        assert!(name_str.contains("乾"));

        // 验证上下卦名称
        let shang_name = core::str::from_utf8(&detail.ben_gua.shang_gua_name).unwrap();
        assert_eq!(shang_name, "乾");
    });
}

/// 测试 calculate_hexagram_detail API
#[test]
fn calculate_hexagram_detail_works() {
    new_test_ext().execute_with(|| {
        // 直接计算卦象详细信息（不需要存储）
        let detail = Meihua::calculate_hexagram_detail(3, 4, 6);

        // 验证本卦（火雷噬嗑）
        let name_str = core::str::from_utf8(&detail.ben_gua.name).unwrap();
        assert!(name_str.contains("火雷"));

        // 验证上卦（离）
        let shang_name = core::str::from_utf8(&detail.ben_gua.shang_gua_name).unwrap();
        assert_eq!(shang_name, "离");

        // 验证下卦（震）
        let xia_name = core::str::from_utf8(&detail.ben_gua.xia_gua_name).unwrap();
        assert_eq!(xia_name, "震");

        // 验证动爻名称
        let yao_name = core::str::from_utf8(&detail.ben_gua.dong_yao_name).unwrap();
        assert_eq!(yao_name, "上爻");  // 第6爻
    });
}

/// 测试卦象详细信息 - 错卦和综卦
#[test]
fn hexagram_detail_includes_cuo_zong() {
    new_test_ext().execute_with(|| {
        // 计算乾为天的详细信息
        let detail = Meihua::calculate_hexagram_detail(1, 1, 1);

        // 乾为天的错卦是坤为地
        let cuo_name = core::str::from_utf8(&detail.cuo_gua.name).unwrap();
        assert!(cuo_name.contains("坤"));

        // 乾为天的综卦还是乾为天（因为对称）
        let zong_name = core::str::from_utf8(&detail.zong_gua.name).unwrap();
        assert!(zong_name.contains("乾"));
    });
}

/// 测试卦象详细信息 - 伏卦
#[test]
fn hexagram_detail_includes_fu_gua() {
    new_test_ext().execute_with(|| {
        // 计算乾为天的详细信息
        let detail = Meihua::calculate_hexagram_detail(1, 1, 1);

        // 乾卦的伏卦是巽卦
        let fu_name = core::str::from_utf8(&detail.fu_gua.name).unwrap();
        assert!(fu_name.contains("巽"), "乾卦的伏卦应该包含巽，实际是: {}", fu_name);
    });
}

/// 测试伏卦计算 - 坤卦的伏卦
#[test]
fn fu_gua_for_kun() {
    new_test_ext().execute_with(|| {
        // 坤为地：上坤下坤
        let detail = Meihua::calculate_hexagram_detail(8, 8, 1);

        // 坤卦的伏卦是乾卦
        let fu_name = core::str::from_utf8(&detail.fu_gua.name).unwrap();
        assert!(fu_name.contains("乾"), "坤卦的伏卦应该包含乾，实际是: {}", fu_name);
    });
}

/// 测试体用关系详细解读
#[test]
fn tiyong_interpretation_works() {
    new_test_ext().execute_with(|| {
        // 计算一个卦象
        let detail = Meihua::calculate_hexagram_detail(1, 8, 1);

        // 验证体用解读文本非空
        let tiyong_interp = core::str::from_utf8(&detail.tiyong_interpretation).unwrap();
        assert!(!tiyong_interp.is_empty(), "体用解读不应为空");

        // 应该包含体用相关关键词
        assert!(
            tiyong_interp.contains("体") || tiyong_interp.contains("用"),
            "体用解读应包含关键词，实际是: {}",
            tiyong_interp
        );
    });
}

// ============================================================================
// divine_with_privacy 测试
// ============================================================================

use crate::types::{EncryptedPrivacyData, PrivacyMode};
use pallet_divination_common::DivinationType;

/// 测试带隐私数据的起卦 - 不带加密数据
#[test]
fn divine_with_privacy_without_encrypted_data() {
    new_test_ext().execute_with(|| {
        let question_hash = [0u8; 32];

        // 使用农历时间起卦，不带加密数据
        assert_ok!(Meihua::divine_with_privacy(
            RuntimeOrigin::signed(1),
            question_hash,
            false,
            1,                                  // 男
            Some(1990),                         // 出生年份
            1,                                  // 事业
            DivinationMethod::LunarDateTime,
            None,                               // 无加密数据
        ));

        // 验证卦象创建
        assert_eq!(Meihua::next_hexagram_id(), 1);

        // 验证卦象存储
        let hexagram = Meihua::hexagrams(0).expect("Hexagram should exist");
        assert_eq!(hexagram.ben_gua.id, 0);
        assert_eq!(hexagram.ben_gua.diviner, 1);
        assert_eq!(hexagram.ben_gua.gender, 1);
        assert_eq!(hexagram.ben_gua.birth_year, Some(1990));
        assert!(!hexagram.ben_gua.is_public);

        // 验证事件触发
        System::assert_has_event(
            Event::<Test>::HexagramCreatedWithPrivacy {
                hexagram_id: 0,
                diviner: 1,
                has_encrypted_data: false,
            }
            .into(),
        );

        // 验证隐私模块中没有加密记录
        assert!(!Privacy::encrypted_records(DivinationType::Meihua, 0).is_some());
    });
}

/// 测试带隐私数据的起卦 - 带加密数据
#[test]
fn divine_with_privacy_with_encrypted_data() {
    new_test_ext().execute_with(|| {
        let question_hash = [1u8; 32];

        // 模拟加密数据
        let encrypted_privacy = EncryptedPrivacyData {
            privacy_mode: PrivacyMode::Private, // 使用 Private 模式，不需要 encrypted_fields
            encrypted_data: vec![1, 2, 3, 4, 5, 6, 7, 8],  // 模拟加密数据
            nonce: [0u8; 24],
            auth_tag: [0u8; 16],
            data_hash: [2u8; 32],
            owner_encrypted_key: vec![9, 10, 11, 12],      // 模拟加密密钥
        };

        // 使用公历时间起卦，带加密数据
        assert_ok!(Meihua::divine_with_privacy(
            RuntimeOrigin::signed(1),
            question_hash,
            true,                                   // 公开
            2,                                      // 女
            Some(1985),                             // 出生年份
            2,                                      // 财运
            DivinationMethod::GregorianDateTime,
            Some(encrypted_privacy),
        ));

        // 验证卦象创建
        let hexagram = Meihua::hexagrams(0).expect("Hexagram should exist");
        assert_eq!(hexagram.ben_gua.gender, 2);
        assert_eq!(hexagram.ben_gua.birth_year, Some(1985));
        assert!(hexagram.ben_gua.is_public);
        assert_eq!(hexagram.ben_gua.method, DivinationMethod::GregorianDateTime);

        // 验证事件触发
        System::assert_has_event(
            Event::<Test>::HexagramCreatedWithPrivacy {
                hexagram_id: 0,
                diviner: 1,
                has_encrypted_data: true,
            }
            .into(),
        );

        // 验证隐私模块中有加密记录
        let encrypted_record = Privacy::encrypted_records(DivinationType::Meihua, 0);
        assert!(encrypted_record.is_some());
        let record = encrypted_record.unwrap();
        assert_eq!(record.privacy_mode, PrivacyMode::Private);
        assert_eq!(record.owner, 1);
    });
}

/// 测试带隐私数据的随机起卦
#[test]
fn divine_with_privacy_random_method() {
    new_test_ext().execute_with(|| {
        let question_hash = [3u8; 32];

        assert_ok!(Meihua::divine_with_privacy(
            RuntimeOrigin::signed(2),
            question_hash,
            false,
            0,                              // 未指定性别
            None,                           // 无出生年份
            0,                              // 未指定类别
            DivinationMethod::Random,
            None,
        ));

        // 验证卦象
        let hexagram = Meihua::hexagrams(0).unwrap();
        assert_eq!(hexagram.ben_gua.diviner, 2);
        assert_eq!(hexagram.ben_gua.gender, 0);
        assert_eq!(hexagram.ben_gua.birth_year, None);
        assert_eq!(hexagram.ben_gua.method, DivinationMethod::Random);
    });
}

/// 测试带隐私数据的起卦 - 无效方法
#[test]
fn divine_with_privacy_invalid_method() {
    new_test_ext().execute_with(|| {
        let question_hash = [4u8; 32];

        // TwoNumbers 方法不支持原子性隐私数据起卦
        assert_noop!(
            Meihua::divine_with_privacy(
                RuntimeOrigin::signed(1),
                question_hash,
                false,
                1,
                Some(1990),
                1,
                DivinationMethod::TwoNumbers,
                None,
            ),
            Error::<Test>::InvalidMethod
        );

        // Manual 方法也不支持
        assert_noop!(
            Meihua::divine_with_privacy(
                RuntimeOrigin::signed(1),
                question_hash,
                false,
                1,
                Some(1990),
                1,
                DivinationMethod::Manual,
                None,
            ),
            Error::<Test>::InvalidMethod
        );

        // SingleNumber 方法也不支持
        assert_noop!(
            Meihua::divine_with_privacy(
                RuntimeOrigin::signed(1),
                question_hash,
                false,
                1,
                Some(1990),
                1,
                DivinationMethod::SingleNumber,
                None,
            ),
            Error::<Test>::InvalidMethod
        );
    });
}

/// 测试带隐私数据的起卦 - 每日限制
#[test]
fn divine_with_privacy_daily_limit() {
    new_test_ext().execute_with(|| {
        let question_hash = [5u8; 32];

        // 先消耗每日限制
        for _ in 0..50 {
            assert_ok!(Meihua::divine_random(
                RuntimeOrigin::signed(1),
                question_hash,
                false,
                0,
                0
            ));
        }

        // divine_with_privacy 也应受每日限制
        assert_noop!(
            Meihua::divine_with_privacy(
                RuntimeOrigin::signed(1),
                question_hash,
                false,
                1,
                Some(1990),
                1,
                DivinationMethod::LunarDateTime,
                None,
            ),
            Error::<Test>::DailyLimitExceeded
        );
    });
}

/// 测试带隐私数据的起卦 - 无效性别参数
#[test]
fn divine_with_privacy_invalid_gender() {
    new_test_ext().execute_with(|| {
        let question_hash = [6u8; 32];

        assert_noop!(
            Meihua::divine_with_privacy(
                RuntimeOrigin::signed(1),
                question_hash,
                false,
                5,   // 无效性别（应为 0-2）
                Some(1990),
                1,
                DivinationMethod::LunarDateTime,
                None,
            ),
            Error::<Test>::InvalidGender
        );
    });
}

/// 测试带隐私数据的起卦 - 无效类别参数
#[test]
fn divine_with_privacy_invalid_category() {
    new_test_ext().execute_with(|| {
        let question_hash = [7u8; 32];

        assert_noop!(
            Meihua::divine_with_privacy(
                RuntimeOrigin::signed(1),
                question_hash,
                false,
                1,
                Some(1990),
                10,  // 无效类别（应为 0-6）
                DivinationMethod::LunarDateTime,
                None,
            ),
            Error::<Test>::InvalidCategory
        );
    });
}

/// 测试带隐私数据的起卦 - 解卦数据创建
#[test]
fn divine_with_privacy_creates_interpretation() {
    new_test_ext().execute_with(|| {
        let question_hash = [8u8; 32];

        assert_ok!(Meihua::divine_with_privacy(
            RuntimeOrigin::signed(1),
            question_hash,
            false,
            1,
            Some(1988),
            3,  // 感情
            DivinationMethod::LunarDateTime,
            None,
        ));

        // 验证解卦数据已创建
        let interpretation = Meihua::get_interpretation_data(0).expect("Interpretation should exist");
        assert_eq!(interpretation.basic_info.gender, 1);
        assert_eq!(interpretation.basic_info.category, 3);
    });
}

/// 测试带隐私数据的起卦 - 原子性（事务回滚）
#[test]
fn divine_with_privacy_atomicity() {
    new_test_ext().execute_with(|| {
        let question_hash = [9u8; 32];

        // 先创建一个卦象
        assert_ok!(Meihua::divine_with_privacy(
            RuntimeOrigin::signed(1),
            question_hash,
            false,
            1,
            Some(1990),
            1,
            DivinationMethod::LunarDateTime,
            None,
        ));

        let first_hexagram_id = Meihua::next_hexagram_id();
        assert_eq!(first_hexagram_id, 1);

        // 创建第二个带加密数据的卦象
        let encrypted_privacy = EncryptedPrivacyData {
            privacy_mode: PrivacyMode::Private,
            encrypted_data: vec![1, 2, 3],
            nonce: [0u8; 24],
            auth_tag: [0u8; 16],
            data_hash: [0u8; 32],
            owner_encrypted_key: vec![4, 5, 6],
        };

        assert_ok!(Meihua::divine_with_privacy(
            RuntimeOrigin::signed(1),
            question_hash,
            false,
            2,
            Some(1995),
            2,
            DivinationMethod::Random,
            Some(encrypted_privacy),
        ));

        // 验证两个卦象都创建成功
        assert_eq!(Meihua::next_hexagram_id(), 2);
        assert!(Meihua::hexagrams(0).is_some());
        assert!(Meihua::hexagrams(1).is_some());

        // 验证第二个卦象的隐私记录
        let record = Privacy::encrypted_records(DivinationType::Meihua, 1);
        assert!(record.is_some());
    });
}
