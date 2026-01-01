//! # 八字排盘 Pallet 单元测试
//!
//! 测试所有核心功能的正确性

use crate::{mock::*, constants::*};
use crate::types::{TianGan, DiZhi, WuXing, GanZhi, NaYin, ShiShen, Gender, ZiShiMode, BaziInputType};
use frame_support::assert_ok;

// ================================
// 基础类型测试
// ================================

#[test]
fn test_tiangan_creation() {
	// 测试天干创建
	assert!(TianGan::new(0).is_some()); // 甲
	assert!(TianGan::new(9).is_some()); // 癸
	assert!(TianGan::new(10).is_none()); // 超出范围
}

#[test]
fn test_tiangan_wuxing() {
	// 测试天干转五行
	assert_eq!(TianGan(0).to_wuxing(), WuXing::Mu); // 甲木
	assert_eq!(TianGan(2).to_wuxing(), WuXing::Huo); // 丙火
	assert_eq!(TianGan(4).to_wuxing(), WuXing::Tu); // 戊土
	assert_eq!(TianGan(6).to_wuxing(), WuXing::Jin); // 庚金
	assert_eq!(TianGan(8).to_wuxing(), WuXing::Shui); // 壬水
}

#[test]
fn test_tiangan_yinyang() {
	// 测试天干阴阳
	assert!(TianGan(0).is_yang()); // 甲为阳
	assert!(!TianGan(1).is_yang()); // 乙为阴
}

#[test]
fn test_dizhi_creation() {
	// 测试地支创建
	assert!(DiZhi::new(0).is_some()); // 子
	assert!(DiZhi::new(11).is_some()); // 亥
	assert!(DiZhi::new(12).is_none()); // 超出范围
}

#[test]
fn test_ganzhi_index_conversion() {
	// 测试干支索引转换
	let jiazi = GanZhi::from_index(0).unwrap(); // 甲子
	assert_eq!(jiazi.gan.0, 0); // 甲
	assert_eq!(jiazi.zhi.0, 0); // 子
	assert_eq!(jiazi.to_index(), 0);

	let guihai = GanZhi::from_index(59).unwrap(); // 癸亥
	assert_eq!(guihai.gan.0, 9); // 癸
	assert_eq!(guihai.zhi.0, 11); // 亥
	assert_eq!(guihai.to_index(), 59);
}

#[test]
fn test_ganzhi_next_prev() {
	// 测试干支的前后关系
	let jiazi = GanZhi::from_index(0).unwrap(); // 甲子
	let yichou = jiazi.next(); // 乙丑
	assert_eq!(yichou.gan.0, 1); // 乙
	assert_eq!(yichou.zhi.0, 1); // 丑

	let guihai = GanZhi::from_index(59).unwrap(); // 癸亥
	let next = guihai.next(); // 应该循环回甲子
	assert_eq!(next.gan.0, 0); // 甲
	assert_eq!(next.zhi.0, 0); // 子
}

// ================================
// 常量表测试
// ================================

#[test]
fn test_chen_hidden_stems() {
	// ⚠️ 关键测试：确保辰藏干为"戊乙癸"
	let chen = DiZhi(4); // 辰
	let stems = get_hidden_stems(chen);

	assert_eq!(stems[0].0 .0, 4); // 戊
	assert_eq!(stems[1].0 .0, 1); // 乙
	assert_eq!(stems[2].0 .0, 9); // 癸 (不是壬!)

	// 验证权重
	assert_eq!(stems[0].2, 500); // 主气权重
	assert_eq!(stems[1].2, 300); // 中气权重
	assert_eq!(stems[2].2, 200); // 余气权重
}

#[test]
fn test_hai_hidden_stems() {
	// ⚠️ 关键测试：确保亥藏干为"壬甲"（两个藏干）
	let hai = DiZhi(11); // 亥
	let stems = get_hidden_stems(hai);

	// 主气：壬(8)
	assert_eq!(stems[0].0 .0, 8, "亥主气应该是壬(8)");
	// 中气：甲(0)
	assert_eq!(stems[1].0 .0, 0, "亥中气应该是甲(0)");
	// 余气：无效(255)
	assert_eq!(stems[2].0 .0, 255, "亥余气应该是无效(255)");

	// 验证有效藏干数量
	assert_eq!(get_hidden_stems_count(hai), 2, "亥应该有2个藏干");
}

#[test]
fn test_zi_hidden_stems() {
	// 测试子藏干（只有一个：癸）
	let zi = DiZhi(0); // 子
	let stems = get_hidden_stems(zi);

	assert_eq!(stems[0].0 .0, 9, "子主气应该是癸(9)");
	assert_eq!(stems[1].0 .0, 255, "子中气应该是无效(255)");
	assert_eq!(stems[2].0 .0, 255, "子余气应该是无效(255)");

	assert_eq!(get_hidden_stems_count(zi), 1, "子应该有1个藏干");
}

#[test]
fn test_all_hidden_stems() {
	// 测试所有地支的藏干
	for i in 0..12 {
		let dizhi = DiZhi(i);
		let stems = get_hidden_stems(dizhi);

		// 确保至少有一个藏干
		assert!(stems[0].0 .0 < 10);
	}
}

#[test]
fn test_nayin_calculation() {
	// 测试纳音计算
	let jiazi = GanZhi::from_index(0).unwrap(); // 甲子
	let nayin = calculate_nayin(&jiazi);
	assert_eq!(nayin, NaYin::HaiZhongJin); // 海中金

	let yichou = GanZhi::from_index(1).unwrap(); // 乙丑
	let nayin = calculate_nayin(&yichou);
	assert_eq!(nayin, NaYin::HaiZhongJin); // 同样是海中金

	let bingyin = GanZhi::from_index(2).unwrap(); // 丙寅
	let nayin = calculate_nayin(&bingyin);
	assert_eq!(nayin, NaYin::LuZhongHuo); // 炉中火
}

#[test]
fn test_shishen_calculation() {
	// 测试十神计算
	let rizhu = TianGan(0); // 甲木日主

	// 甲见甲 → 比肩
	assert_eq!(calculate_shishen(rizhu, TianGan(0)), ShiShen::BiJian);
	// 甲见乙 → 劫财
	assert_eq!(calculate_shishen(rizhu, TianGan(1)), ShiShen::JieCai);
	// 甲见丙 → 食神
	assert_eq!(calculate_shishen(rizhu, TianGan(2)), ShiShen::ShiShen);
	// 甲见丁 → 伤官
	assert_eq!(calculate_shishen(rizhu, TianGan(3)), ShiShen::ShangGuan);
}

// ================================
// Pallet 功能测试（占位）
// ================================

#[test]
fn test_create_bazi_chart_placeholder() {
	new_test_ext().execute_with(|| {
		// TODO: 实现完整的创建八字测试
		// 当前只是占位，确保编译通过
	});
}

// ================================
// 边界情况测试
// ================================

#[test]
fn test_leap_year() {
	// 测试闰年判断
	use crate::calculations::ganzhi::is_leap_year;

	assert!(is_leap_year(2000)); // 2000年是闰年
	assert!(is_leap_year(2024)); // 2024年是闰年
	assert!(!is_leap_year(1900)); // 1900年不是闰年
	assert!(!is_leap_year(2023)); // 2023年不是闰年
}

#[test]
fn test_days_in_month() {
	// 测试月份天数
	use crate::calculations::ganzhi::get_days_in_month;

	assert_eq!(get_days_in_month(2024, 1), 31); // 1月31天
	assert_eq!(get_days_in_month(2024, 2), 29); // 2024年2月29天（闰年）
	assert_eq!(get_days_in_month(2023, 2), 28); // 2023年2月28天（平年）
	assert_eq!(get_days_in_month(2024, 4), 30); // 4月30天
}

// ================================
// 月柱计算测试
// ================================

#[test]
fn test_month_ganzhi_1990_11_29() {
	// 关键测试：1990年11月29日12时 月柱应该是 丁亥
	// 1990年是庚午年，年干是庚(6)
	use crate::calculations::calculate_month_ganzhi;

	let gz = calculate_month_ganzhi(1990, 11, 29, 6).expect("月柱计算失败");

	// 丁亥: 天干丁(3), 地支亥(11)
	assert_eq!(gz.gan.0, 3, "月干应该是丁(3)，实际得到: {}", gz.gan.0);
	assert_eq!(gz.zhi.0, 11, "月支应该是亥(11)，实际得到: {}", gz.zhi.0);
}

#[test]
fn test_month_ganzhi_1990_12_23() {
	// 关键测试：1990年12月23日12时 月柱应该是 戊子
	// 1990年是庚午年，年干是庚(6)
	// 1990年大雪：12月7日左右
	// 12月23日在大雪之后，属于子月(0)
	use crate::calculations::calculate_month_ganzhi;

	let gz = calculate_month_ganzhi(1990, 12, 23, 6).expect("月柱计算失败");

	// 戊子: 天干戊(4), 地支子(0)
	assert_eq!(gz.gan.0, 4, "月干应该是戊(4)，实际得到: {}", gz.gan.0);
	assert_eq!(gz.zhi.0, 0, "月支应该是子(0)，实际得到: {}", gz.zhi.0);
}

#[test]
fn test_jieqi_1990_lidong_daxue() {
	// 验证1990年立冬和大雪的计算
	use crate::calculations::jieqi::get_jieqi_time;

	// 1990年立冬(索引15)应该在11月7日左右
	let lidong = get_jieqi_time(1990, 15);
	assert_eq!(lidong.year, 1990, "立冬年份");
	assert_eq!(lidong.month, 11, "立冬月份");
	assert!(lidong.day >= 7 && lidong.day <= 8, "1990年立冬应该在11月7-8日，实际: {}日", lidong.day);

	// 1990年大雪(索引17)应该在12月7日左右
	let daxue = get_jieqi_time(1990, 17);
	assert_eq!(daxue.year, 1990, "大雪年份");
	assert_eq!(daxue.month, 12, "大雪月份");
	assert!(daxue.day >= 6 && daxue.day <= 8, "1990年大雪应该在12月6-8日，实际: {}日", daxue.day);
}

// ================================
// 解盘功能测试
// ================================

// TODO: 更新为使用新的存储结构 (ChartById + UserCharts) 和 u64 chart_id
// #[test]
// fn test_interpretation_integration() {
// 	new_test_ext().execute_with(|| {
// 		// 集成测试：创建八字并进行解盘
// 		let account_id = 1u64;
//
// 		// 创建八字
// 		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
// 			RuntimeOrigin::signed(account_id),
// 			1990, 11, 15, 14, 30,
// 			Gender::Male,
// 			ZiShiMode::Modern,
// 			None, None,
// 		));
//
// 		// 获取创建的八字ID
// 		let charts = crate::pallet::BaziCharts::<Test>::get(&account_id);
// 		assert_eq!(charts.len(), 1);
//
// 		let chart = &charts[0];
// 		let chart_id = <Test as frame_system::Config>::Hashing::hash_of(chart);
//
// 		// 执行解盘
// 		assert_ok!(crate::pallet::Pallet::<Test>::interpret_bazi_chart(
// 			RuntimeOrigin::signed(account_id),
// 			chart_id,
// 		));
//
// 		// 验证解盘结果已存储
// 		let interpretation = crate::pallet::InterpretationById::<Test>::get(&chart_id);
// 		assert!(interpretation.is_some());
//
// 		let result = interpretation.unwrap();
// 		assert!(result.zong_he_ping_fen > 0 && result.zong_he_ping_fen <= 100);
// 		assert!(!result.jie_pan_text.is_empty());
// 	});
// }

// ================================
// 创建八字命盘集成测试
// ================================

#[test]
fn test_create_bazi_chart_success() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 使用统一接口创建八字命盘（公历输入）
		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 14,
			minute: 30,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,  // name
			input,
			Gender::Male,
			ZiShiMode::Modern,
			None,   // longitude
		));

		// 验证用户的命盘列表
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		assert_eq!(user_charts.len(), 1);

		// 验证命盘存储
		let chart_id = user_charts[0];
		let chart = crate::pallet::ChartById::<Test>::get(chart_id);
		assert!(chart.is_some());

		let chart = chart.unwrap();
		assert_eq!(chart.owner, account_id);
		assert_eq!(chart.birth_time.unwrap().year, 1990);
		assert_eq!(chart.birth_time.unwrap().month, 11);
		assert_eq!(chart.birth_time.unwrap().day, 15);
		assert_eq!(chart.gender, Some(Gender::Male));
	});
}

#[test]
fn test_create_bazi_chart_with_different_zishi_modes() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 使用传统派创建
		let input_traditional = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 23,
			minute: 30, // 23:30 晚子时
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,  // name
			input_traditional,
			Gender::Male,
			ZiShiMode::Traditional, // 传统派：23:00-23:59 属于次日
			None,   // longitude
		));

		// 使用现代派创建
		let input_modern = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 23,
			minute: 30, // 23:30 晚子时
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,  // name
			input_modern,
			Gender::Female,
			ZiShiMode::Modern, // 现代派：23:00-23:59 属于当日
			None,   // longitude
		));

		// 验证创建了两个不同的命盘
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		assert_eq!(user_charts.len(), 2);
	});
}

#[test]
fn test_delete_bazi_chart() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 创建八字
		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 14,
			minute: 30,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,  // name
			input,
			Gender::Male,
			ZiShiMode::Modern,
			None,   // longitude
		));

		// 获取命盘ID
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		let chart_id = user_charts[0];

		// 删除命盘
		assert_ok!(crate::pallet::Pallet::<Test>::delete_bazi_chart(
			RuntimeOrigin::signed(account_id),
			chart_id,
		));

		// 验证命盘已删除
		let chart = crate::pallet::ChartById::<Test>::get(chart_id);
		assert!(chart.is_none());

		// 验证用户列表已更新
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		assert!(user_charts.is_empty());
	});
}

// 注: get_full_bazi_chart 测试已移除，该函数通过 Runtime API JSON 接口实现
// Runtime API 返回 JSON 字符串，在集成测试中验证

#[test]
fn test_get_full_interpretation() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 创建八字（使用统一接口）
		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 14,
			minute: 30,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,  // name
			input,
			Gender::Male,
			ZiShiMode::Modern,
			None,   // longitude
		));

		// 获取命盘ID
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		let chart_id = user_charts[0];

		// 获取完整解盘
		let interpretation = crate::pallet::Pallet::<Test>::get_full_interpretation(chart_id);
		assert!(interpretation.is_some());

		let interpretation = interpretation.unwrap();

		// 验证核心指标
		assert!(interpretation.core.score <= 100);
		assert!(interpretation.core.confidence <= 100);

		// 验证性格分析存在
		assert!(interpretation.xing_ge.is_some());
	});
}

#[test]
fn test_cache_interpretation() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 创建八字（使用统一接口）
		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 14,
			minute: 30,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,  // name
			input,
			Gender::Male,
			ZiShiMode::Modern,
			None,   // longitude
		));

		// 获取命盘ID
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		let chart_id = user_charts[0];

		// 缓存解盘
		assert_ok!(crate::pallet::Pallet::<Test>::cache_interpretation(
			RuntimeOrigin::signed(account_id),
			chart_id,
		));

		// 验证缓存存在
		let cached = crate::pallet::InterpretationCache::<Test>::get(chart_id);
		assert!(cached.is_some());

		let cached = cached.unwrap();
		assert!(cached.score <= 100);
	});
}

// ================================
// 边界情况测试
// ================================

#[test]
fn test_invalid_birth_parameters() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 无效年份
		let invalid_year = BaziInputType::Solar {
			year: 1800,  // 年份太早
			month: 1,
			day: 1,
			hour: 12,
			minute: 0,
		};
		assert!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,
			invalid_year,
			Gender::Male,
			ZiShiMode::Modern,
			None,
		).is_err());

		// 无效月份
		let invalid_month = BaziInputType::Solar {
			year: 1990,
			month: 13,  // 月份无效
			day: 1,
			hour: 12,
			minute: 0,
		};
		assert!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,
			invalid_month,
			Gender::Male,
			ZiShiMode::Modern,
			None,
		).is_err());

		// 无效日期 (day > 31)
		let invalid_day = BaziInputType::Solar {
			year: 1990,
			month: 1,
			day: 32,  // day 超过 31
			hour: 12,
			minute: 0,
		};
		assert!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,
			invalid_day,
			Gender::Male,
			ZiShiMode::Modern,
			None,
		).is_err());

		// 无效小时
		let invalid_hour = BaziInputType::Solar {
			year: 1990,
			month: 1,
			day: 1,
			hour: 25,  // 小时无效
			minute: 0,
		};
		assert!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,
			invalid_hour,
			Gender::Male,
			ZiShiMode::Modern,
			None,
		).is_err());
	});
}

// ================================
// 星运（十二长生）测试
// ================================

// 注: test_xingyun_calculation 测试依赖 get_full_bazi_chart，该函数通过 Runtime API 返回 JSON
// 此处改为单元测试验证 xingyun 模块的计算逻辑

#[test]
fn test_xingyun_calculation_unit() {
	use crate::calculations::xingyun::get_changsheng;
	use crate::types::ShiErChangSheng;

	// 甲木长生在亥
	let rizhu = TianGan(0); // 甲
	let hai = DiZhi(11);    // 亥
	let changsheng = get_changsheng(rizhu, hai);
	assert_eq!(changsheng, ShiErChangSheng::ChangSheng);

	// 甲木帝旺在卯
	let mao = DiZhi(3);
	let changsheng = get_changsheng(rizhu, mao);
	assert_eq!(changsheng, ShiErChangSheng::DiWang);
}

// ================================
// 神煞计算测试
// ================================

// 注: test_shensha_list_in_full_chart 测试依赖 get_full_bazi_chart，该函数通过 Runtime API 返回 JSON
// 此处改为单元测试验证 shensha 模块的计算逻辑

#[test]
fn test_shensha_calculation_unit() {
	use crate::calculations::shensha::*;

	// 甲日见丑或未为天乙贵人
	let jia = TianGan(0);
	assert!(calculate_tianyi_guiren(jia, DiZhi(1)));  // 丑
	assert!(calculate_tianyi_guiren(jia, DiZhi(7)));  // 未
	assert!(!calculate_tianyi_guiren(jia, DiZhi(0))); // 子不是

	// 甲刃在卯
	assert!(calculate_yangren(TianGan(0), DiZhi(3)));
}

// ================================
// 纳音计算测试
// ================================

// 注: test_nayin_in_enhanced_zhu 测试依赖 get_full_bazi_chart，该函数通过 Runtime API 返回 JSON
// 此处保留纳音计算逻辑的单元测试

#[test]
fn test_nayin_calculation_unit() {
	use crate::constants::calculate_nayin;
	use crate::types::NaYin;

	// 甲子、乙丑为海中金
	let jiazi = GanZhi::from_index(0).unwrap();
	let nayin = calculate_nayin(&jiazi);
	assert_eq!(nayin, NaYin::HaiZhongJin);

	let yichou = GanZhi::from_index(1).unwrap();
	let nayin = calculate_nayin(&yichou);
	assert_eq!(nayin, NaYin::HaiZhongJin);

	// 丙寅、丁卯为炉中火
	let bingyin = GanZhi::from_index(2).unwrap();
	let nayin = calculate_nayin(&bingyin);
	assert_eq!(nayin, NaYin::LuZhongHuo);
}

// ================================
// 大运计算测试
// ================================

#[test]
fn test_dayun_info() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 创建八字（男命，使用统一接口）
		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 29,
			hour: 12,
			minute: 0,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,  // name
			input,
			Gender::Male,
			ZiShiMode::Modern,
			None,   // longitude
		));

		// 获取命盘ID
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		let chart_id = user_charts[0];

		// 获取命盘
		let chart = crate::pallet::ChartById::<Test>::get(chart_id).unwrap();

		// 验证大运信息
		let dayun = chart.dayun.as_ref().unwrap();
		assert!(dayun.qiyun_age < 11); // 起运年龄一般在 1-10 岁
		assert!(!dayun.dayun_list.is_empty()); // 大运列表不为空
		assert!(dayun.dayun_list.len() >= 8); // 至少有 8 步大运

		// 验证第一步大运
		let first_dayun = &dayun.dayun_list[0];
		assert!(first_dayun.start_age <= first_dayun.end_age);
		assert!((first_dayun.tiangan_shishen as u8) < 10); // 十神有效
	});
}

// ================================
// 藏干（副星）测试
// ================================

#[test]
fn test_canggan_list_in_chart() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 创建八字（使用统一接口）
		let input = BaziInputType::Solar {
			year: 1998,
			month: 7,
			day: 31,
			hour: 14,
			minute: 10,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,  // name
			input,
			Gender::Male,
			ZiShiMode::Modern,
			None,   // longitude
		));

		// 获取命盘ID
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		let chart_id = user_charts[0];

		// 获取命盘
		let chart = crate::pallet::ChartById::<Test>::get(chart_id).unwrap();

		// 验证藏干列表（每柱 1-3 个藏干）
		let sizhu = chart.sizhu.as_ref().unwrap();
		assert!(!sizhu.year_zhu.canggan.is_empty());
		assert!(sizhu.year_zhu.canggan.len() <= 3);

		// 验证藏干详细信息
		for canggan in sizhu.year_zhu.canggan.iter() {
			assert!(canggan.gan.0 < 10); // 天干有效
			assert!((canggan.shishen as u8) < 10); // 十神有效
			assert!((canggan.canggan_type as u8) < 3); // 藏干类型有效（主气/中气/余气）
			assert!(canggan.weight <= 1000); // 权重有效（最大 1000）
		}
	});
}

// ================================
// 用户查询功能测试
// ================================

#[test]
fn test_user_charts_query() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 创建多个八字（使用统一接口）
		for i in 0..3 {
			let input = BaziInputType::Solar {
				year: 1990 + i as u16,
				month: 11,
				day: 29,
				hour: 12,
				minute: 0,
			};

			assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
				RuntimeOrigin::signed(account_id),
				None,  // name
				input,
				Gender::Male,
				ZiShiMode::Modern,
				None,   // longitude
			));
		}

		// 查询用户八字列表
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		assert_eq!(user_charts.len(), 3);

		// 验证每个八字都可以查询到
		for chart_id in user_charts.iter() {
			let chart = crate::pallet::ChartById::<Test>::get(chart_id);
			assert!(chart.is_some());
			assert_eq!(chart.unwrap().owner, account_id);
		}
	});
}

// ================================
// 解盘核心指标测试
// ================================

#[test]
fn test_interpretation_core_fields() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 创建八字
		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 29,
			hour: 12,
			minute: 0,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,  // name
			input,
			Gender::Male,
			ZiShiMode::Modern,
			None,   // longitude
		));

		// 获取命盘ID
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		let chart_id = user_charts[0];

		// 获取完整解盘
		let interpretation = crate::pallet::Pallet::<Test>::get_full_interpretation(chart_id);
		assert!(interpretation.is_some());

		let interpretation = interpretation.unwrap();

		// 验证核心指标
		assert!(interpretation.core.score <= 100);
		assert!(interpretation.core.confidence <= 100);
		assert!((interpretation.core.ge_ju as u8) < 8); // 格局类型有效
		assert!((interpretation.core.qiang_ruo as u8) < 5); // 强弱类型有效
		assert!((interpretation.core.yong_shen as u8) < 5); // 用神五行有效
		assert!((interpretation.core.xi_shen as u8) < 5); // 喜神五行有效
		assert!((interpretation.core.ji_shen as u8) < 5); // 忌神五行有效
	});
}

// ================================
// 农历输入测试
// ================================

#[test]
fn test_create_bazi_chart_from_lunar() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 农历 2024年正月初一 = 公历 2024年2月10日（春节）
		let input = BaziInputType::Lunar {
			year: 2024,
			month: 1,
			day: 1,
			is_leap_month: false,
			hour: 12,
			minute: 30,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,  // name
			input,
			Gender::Male,
			ZiShiMode::Modern,
			None,   // longitude
		));

		// 验证用户的命盘列表
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		assert_eq!(user_charts.len(), 1);

		// 验证命盘存储
		let chart_id = user_charts[0];
		let chart = crate::pallet::ChartById::<Test>::get(chart_id);
		assert!(chart.is_some());

		let chart = chart.unwrap();
		assert_eq!(chart.owner, account_id);
		// 公历应该是 2024年2月10日
		assert_eq!(chart.birth_time.unwrap().year, 2024);
		assert_eq!(chart.birth_time.unwrap().month, 2);
		assert_eq!(chart.birth_time.unwrap().day, 10);
		assert_eq!(chart.gender, Some(Gender::Male));
	});
}

#[test]
fn test_create_bazi_chart_from_lunar_invalid() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 测试无效的农历年份
		let invalid_year = BaziInputType::Lunar {
			year: 1800, // 年份超出范围
			month: 1,
			day: 1,
			is_leap_month: false,
			hour: 12,
			minute: 0,
		};
		assert!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,
			invalid_year,
			Gender::Male,
			ZiShiMode::Modern,
			None,
		).is_err());

		// 测试无效的农历月份
		let invalid_month = BaziInputType::Lunar {
			year: 2024,
			month: 13, // 月份无效
			day: 1,
			is_leap_month: false,
			hour: 12,
			minute: 0,
		};
		assert!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,
			invalid_month,
			Gender::Male,
			ZiShiMode::Modern,
			None,
		).is_err());

		// 测试无效的农历日期
		let invalid_day = BaziInputType::Lunar {
			year: 2024,
			month: 1,
			day: 31, // 农历日最大30
			is_leap_month: false,
			hour: 12,
			minute: 0,
		};
		assert!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,
			invalid_day,
			Gender::Male,
			ZiShiMode::Modern,
			None,
		).is_err());
	});
}

// ================================
// 四柱直接输入测试
// ================================

#[test]
fn test_create_bazi_chart_from_sizhu() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 创建四柱输入：甲子年(0) 丙寅月(2) 戊辰日(4) 甲子时(0)
		let input = BaziInputType::SiZhu {
			year_gz: 0,   // 甲子
			month_gz: 2,  // 丙寅
			day_gz: 4,    // 戊辰
			hour_gz: 0,   // 甲子
			birth_year: 1984,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,  // name
			input,
			Gender::Male,
			ZiShiMode::Modern,
			None,   // longitude
		));

		// 验证用户的命盘列表
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		assert_eq!(user_charts.len(), 1);

		// 验证命盘存储
		let chart_id = user_charts[0];
		let chart = crate::pallet::ChartById::<Test>::get(chart_id);
		assert!(chart.is_some());

		let chart = chart.unwrap();
		assert_eq!(chart.owner, account_id);
		assert_eq!(chart.gender, Some(Gender::Male));

		// 验证四柱
		let sizhu = chart.sizhu.as_ref().unwrap();
		assert_eq!(sizhu.year_zhu.ganzhi.gan.0, 0); // 甲
		assert_eq!(sizhu.year_zhu.ganzhi.zhi.0, 0); // 子
	});
}

#[test]
fn test_create_bazi_chart_from_sizhu_invalid() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 测试无效的干支索引（超过59）
		let invalid_input = BaziInputType::SiZhu {
			year_gz: 60, // 超过59
			month_gz: 0,
			day_gz: 0,
			hour_gz: 0,
			birth_year: 1984,
		};

		// 注意：由于 is_valid() 返回 false，pallet 应该拒绝
		assert!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,
			invalid_input,
			Gender::Male,
			ZiShiMode::Modern,
			None,
		).is_err());
	});
}

// ================================
// 真太阳时修正测试
// ================================

#[test]
fn test_true_solar_time_correction() {
	use crate::calculations::true_solar_time::{apply_true_solar_time, should_apply_correction};

	// 测试是否需要修正判断
	// 北京（116.4°）不需要修正（差距小于7.5°）
	assert!(!should_apply_correction(11_640_000));
	// 乌鲁木齐（87.6°）需要修正（差距 32.4°）
	assert!(should_apply_correction(8_760_000));

	// 测试乌鲁木齐的修正
	// 北京时间 12:00，出生地经度 87.6°
	let result = apply_true_solar_time(2024, 6, 15, 12, 0, 8_760_000);
	// 经度差 = 87.6° - 120° = -32.4°
	// 经度修正 ≈ -130 分钟 ≈ -2小时10分
	// 时差方程（6月）≈ +2 分钟
	// 修正后 ≈ 9:52
	assert_eq!(result.hour, 9);
	assert!(result.minute >= 50 && result.minute <= 54);
	assert_eq!(result.day_offset, 0);
}

#[test]
fn test_create_bazi_chart_with_true_solar_time() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 使用真太阳时修正创建八字（乌鲁木齐）
		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 14,
			minute: 30,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			Some(b"test".to_vec().try_into().unwrap()),  // 带名称
			input,
			Gender::Male,
			ZiShiMode::Modern,
			Some(8_760_000),   // 乌鲁木齐经度 87.6°（传入经度即启用真太阳时）
		));

		// 验证创建成功
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		assert_eq!(user_charts.len(), 1);

		let chart_id = user_charts[0];
		let chart = crate::pallet::ChartById::<Test>::get(chart_id).unwrap();

		// 验证字段
		assert_eq!(chart.longitude, Some(8_760_000));
		assert_eq!(&chart.name[..], b"test");
	});
}

// ================================
// 输入日历类型测试
// ================================

#[test]
fn test_input_calendar_type_solar() {
	use crate::types::InputCalendarType;

	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 公历输入
		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 14,
			minute: 30,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,
			input,
			Gender::Male,
			ZiShiMode::Modern,
			None,
		));

		// 验证输入类型为公历
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		let chart = crate::pallet::ChartById::<Test>::get(user_charts[0]).unwrap();
		assert_eq!(chart.input_calendar_type, Some(InputCalendarType::Solar));
		assert_eq!(chart.input_calendar_type.unwrap().name(), "公历");
	});
}

#[test]
fn test_input_calendar_type_lunar() {
	use crate::types::InputCalendarType;

	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 农历输入
		let input = BaziInputType::Lunar {
			year: 2024,
			month: 1,
			day: 1,
			is_leap_month: false,
			hour: 12,
			minute: 30,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,
			input,
			Gender::Male,
			ZiShiMode::Modern,
			None,
		));

		// 验证输入类型为农历
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		let chart = crate::pallet::ChartById::<Test>::get(user_charts[0]).unwrap();
		assert_eq!(chart.input_calendar_type, Some(InputCalendarType::Lunar));
		assert_eq!(chart.input_calendar_type.unwrap().name(), "农历");
	});
}

#[test]
fn test_input_calendar_type_sizhu() {
	use crate::types::InputCalendarType;

	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 四柱直接输入
		let input = BaziInputType::SiZhu {
			year_gz: 0,   // 甲子
			month_gz: 2,  // 丙寅
			day_gz: 4,    // 戊辰
			hour_gz: 0,   // 甲子
			birth_year: 1984,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart(
			RuntimeOrigin::signed(account_id),
			None,
			input,
			Gender::Male,
			ZiShiMode::Modern,
			None,
		));

		// 验证输入类型为四柱
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		let chart = crate::pallet::ChartById::<Test>::get(user_charts[0]).unwrap();
		assert_eq!(chart.input_calendar_type, Some(InputCalendarType::SiZhu));
		assert_eq!(chart.input_calendar_type.unwrap().name(), "四柱");
	});
}

// ================================
// V6 多方授权加密系统测试
// ================================

use crate::types::{
	SiZhuIndex, AccessRole, AccessScope, EncryptedKeyEntry, ServiceProviderType,
};
use frame_support::{assert_noop, BoundedVec};
use frame_support::traits::ConstU32;

/// 创建测试用的四柱索引
fn test_sizhu_index() -> SiZhuIndex {
	SiZhuIndex {
		year_gan: 0,   // 甲
		year_zhi: 0,   // 子
		month_gan: 2,  // 丙
		month_zhi: 2,  // 寅
		day_gan: 4,    // 戊
		day_zhi: 4,    // 辰
		hour_gan: 0,   // 甲
		hour_zhi: 0,   // 子
	}
}

/// 创建测试用的加密数据（模拟 AES-GCM 加密后的数据）
fn test_encrypted_data() -> BoundedVec<u8, ConstU32<256>> {
	let data: Vec<u8> = (0..64).collect(); // 64 字节模拟加密数据
	BoundedVec::try_from(data).unwrap()
}

/// 创建测试用的 nonce（AES-GCM 标准 12 字节）
fn test_nonce() -> [u8; 12] {
	[1u8; 12]
}

/// 创建测试用的 auth_tag（AES-GCM 标准 16 字节）
fn test_auth_tag() -> [u8; 16] {
	[2u8; 16]
}

/// 创建测试用的数据哈希（Blake2-256，32 字节）
fn test_data_hash() -> [u8; 32] {
	[3u8; 32]
}

/// 创建测试用的 X25519 公钥（32 字节）
fn test_public_key(seed: u8) -> [u8; 32] {
	[seed; 32]
}

/// 创建测试用的加密密钥条目
fn test_encrypted_key_entry(
	account: u64,
	role: AccessRole,
	scope: AccessScope,
) -> EncryptedKeyEntry<u64> {
	let encrypted_key_data: Vec<u8> = (0..48).collect(); // 模拟 sealed box 数据
	EncryptedKeyEntry {
		account,
		encrypted_key: BoundedVec::try_from(encrypted_key_data).unwrap(),
		role,
		scope,
		granted_at: 1,
		expires_at: 0, // 永久有效
	}
}

// ================================
// 用户加密密钥注册测试
// ================================

#[test]
fn test_register_encryption_key_success() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;
		let public_key = test_public_key(0x11);

		// 注册加密公钥
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(account_id),
			public_key,
		));

		// 验证存储
		assert!(crate::pallet::UserEncryptionKeys::<Test>::contains_key(account_id));
		let stored_key = crate::pallet::UserEncryptionKeys::<Test>::get(account_id).unwrap();
		assert_eq!(stored_key, public_key);
	});
}

#[test]
fn test_register_encryption_key_already_registered() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;
		let public_key = test_public_key(0x11);

		// 首次注册成功
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(account_id),
			public_key,
		));

		// 重复注册应失败
		assert_noop!(
			crate::pallet::Pallet::<Test>::register_encryption_key(
				RuntimeOrigin::signed(account_id),
				public_key,
			),
			crate::Error::<Test>::EncryptionKeyAlreadyRegistered
		);
	});
}

#[test]
fn test_update_encryption_key_success() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;
		let old_key = test_public_key(0x11);
		let new_key = test_public_key(0x22);

		// 先注册
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(account_id),
			old_key,
		));

		// 更新密钥
		assert_ok!(crate::pallet::Pallet::<Test>::update_encryption_key(
			RuntimeOrigin::signed(account_id),
			new_key,
		));

		// 验证更新
		let stored_key = crate::pallet::UserEncryptionKeys::<Test>::get(account_id).unwrap();
		assert_eq!(stored_key, new_key);
	});
}

#[test]
fn test_update_encryption_key_not_registered() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;
		let new_key = test_public_key(0x22);

		// 未注册时更新应失败
		assert_noop!(
			crate::pallet::Pallet::<Test>::update_encryption_key(
				RuntimeOrigin::signed(account_id),
				new_key,
			),
			crate::Error::<Test>::EncryptionKeyNotRegistered
		);
	});
}

// ================================
// 服务提供者注册测试
// ================================

#[test]
fn test_register_provider_success() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;
		let public_key = test_public_key(0x33);

		// 注册为命理师
		assert_ok!(crate::pallet::Pallet::<Test>::register_provider(
			RuntimeOrigin::signed(account_id),
			ServiceProviderType::MingLiShi,
			public_key,
		));

		// 验证 ServiceProviders 存储
		assert!(crate::pallet::ServiceProviders::<Test>::contains_key(account_id));
		let provider = crate::pallet::ServiceProviders::<Test>::get(account_id).unwrap();
		assert_eq!(provider.provider_type, ServiceProviderType::MingLiShi);
		assert_eq!(provider.public_key, public_key);
		assert_eq!(provider.reputation, 50); // 初始信誉分
		assert!(provider.is_active);

		// 验证 ProvidersByType 索引
		let providers = crate::pallet::ProvidersByType::<Test>::get(ServiceProviderType::MingLiShi);
		assert!(providers.contains(&account_id));

		// 验证同时注册了 UserEncryptionKeys
		assert!(crate::pallet::UserEncryptionKeys::<Test>::contains_key(account_id));
	});
}

#[test]
fn test_register_provider_already_registered() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;
		let public_key = test_public_key(0x33);

		// 首次注册成功
		assert_ok!(crate::pallet::Pallet::<Test>::register_provider(
			RuntimeOrigin::signed(account_id),
			ServiceProviderType::MingLiShi,
			public_key,
		));

		// 重复注册应失败
		assert_noop!(
			crate::pallet::Pallet::<Test>::register_provider(
				RuntimeOrigin::signed(account_id),
				ServiceProviderType::AiService,
				public_key,
			),
			crate::Error::<Test>::ProviderAlreadyRegistered
		);
	});
}

#[test]
fn test_update_provider_key_success() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;
		let old_key = test_public_key(0x33);
		let new_key = test_public_key(0x44);

		// 先注册
		assert_ok!(crate::pallet::Pallet::<Test>::register_provider(
			RuntimeOrigin::signed(account_id),
			ServiceProviderType::MingLiShi,
			old_key,
		));

		// 更新密钥
		assert_ok!(crate::pallet::Pallet::<Test>::update_provider_key(
			RuntimeOrigin::signed(account_id),
			new_key,
		));

		// 验证更新
		let provider = crate::pallet::ServiceProviders::<Test>::get(account_id).unwrap();
		assert_eq!(provider.public_key, new_key);

		// 同时验证 UserEncryptionKeys 也更新了
		let user_key = crate::pallet::UserEncryptionKeys::<Test>::get(account_id).unwrap();
		assert_eq!(user_key, new_key);
	});
}

#[test]
fn test_set_provider_active_success() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;
		let public_key = test_public_key(0x33);

		// 先注册
		assert_ok!(crate::pallet::Pallet::<Test>::register_provider(
			RuntimeOrigin::signed(account_id),
			ServiceProviderType::MingLiShi,
			public_key,
		));

		// 禁用
		assert_ok!(crate::pallet::Pallet::<Test>::set_provider_active(
			RuntimeOrigin::signed(account_id),
			false,
		));
		let provider = crate::pallet::ServiceProviders::<Test>::get(account_id).unwrap();
		assert!(!provider.is_active);

		// 重新启用
		assert_ok!(crate::pallet::Pallet::<Test>::set_provider_active(
			RuntimeOrigin::signed(account_id),
			true,
		));
		let provider = crate::pallet::ServiceProviders::<Test>::get(account_id).unwrap();
		assert!(provider.is_active);
	});
}

#[test]
fn test_unregister_provider_success() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;
		let public_key = test_public_key(0x33);

		// 先注册
		assert_ok!(crate::pallet::Pallet::<Test>::register_provider(
			RuntimeOrigin::signed(account_id),
			ServiceProviderType::MingLiShi,
			public_key,
		));

		// 注销
		assert_ok!(crate::pallet::Pallet::<Test>::unregister_provider(
			RuntimeOrigin::signed(account_id),
		));

		// 验证删除
		assert!(!crate::pallet::ServiceProviders::<Test>::contains_key(account_id));
		let providers = crate::pallet::ProvidersByType::<Test>::get(ServiceProviderType::MingLiShi);
		assert!(!providers.contains(&account_id));
	});
}

// ================================
// 多方授权加密命盘创建测试
// ================================

#[test]
fn test_create_multi_key_encrypted_chart_success() {
	new_test_ext().execute_with(|| {
		let owner = 1u64;
		let public_key = test_public_key(0x11);

		// 先注册加密公钥
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(owner),
			public_key,
		));

		// 创建 Owner 的密钥条目
		let owner_entry = test_encrypted_key_entry(owner, AccessRole::Owner, AccessScope::FullAccess);
		let encrypted_keys: BoundedVec<EncryptedKeyEntry<u64>, ConstU32<10>> =
			BoundedVec::try_from(vec![owner_entry]).unwrap();

		// 创建多方授权加密命盘
		assert_ok!(crate::pallet::Pallet::<Test>::create_multi_key_encrypted_chart(
			RuntimeOrigin::signed(owner),
			test_sizhu_index(),
			Gender::Male,
			test_encrypted_data(),
			test_nonce(),
			test_auth_tag(),
			encrypted_keys,
			test_data_hash(),
		));

		// 验证存储
		let user_charts = crate::pallet::UserMultiKeyEncryptedCharts::<Test>::get(owner);
		assert_eq!(user_charts.len(), 1);

		let chart_id = user_charts[0];
		let chart = crate::pallet::MultiKeyEncryptedChartById::<Test>::get(chart_id).unwrap();
		assert_eq!(chart.owner, owner);
		assert_eq!(chart.gender, Gender::Male);
		assert_eq!(chart.encrypted_keys.len(), 1);
	});
}

#[test]
fn test_create_multi_key_encrypted_chart_not_registered() {
	new_test_ext().execute_with(|| {
		let owner = 1u64;

		// 不注册加密公钥直接创建应失败
		let owner_entry = test_encrypted_key_entry(owner, AccessRole::Owner, AccessScope::FullAccess);
		let encrypted_keys: BoundedVec<EncryptedKeyEntry<u64>, ConstU32<10>> =
			BoundedVec::try_from(vec![owner_entry]).unwrap();

		assert_noop!(
			crate::pallet::Pallet::<Test>::create_multi_key_encrypted_chart(
				RuntimeOrigin::signed(owner),
				test_sizhu_index(),
				Gender::Male,
				test_encrypted_data(),
				test_nonce(),
				test_auth_tag(),
				encrypted_keys,
				test_data_hash(),
			),
			crate::Error::<Test>::EncryptionKeyNotRegistered
		);
	});
}

// ================================
// 授权访问测试
// ================================

#[test]
fn test_grant_chart_access_success() {
	new_test_ext().execute_with(|| {
		let owner = 1u64;
		let grantee = 2u64;
		let owner_key = test_public_key(0x11);
		let grantee_key = test_public_key(0x22);

		// 注册双方加密公钥
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(owner),
			owner_key,
		));
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(grantee),
			grantee_key,
		));

		// 创建命盘
		let owner_entry = test_encrypted_key_entry(owner, AccessRole::Owner, AccessScope::FullAccess);
		let encrypted_keys: BoundedVec<EncryptedKeyEntry<u64>, ConstU32<10>> =
			BoundedVec::try_from(vec![owner_entry]).unwrap();

		assert_ok!(crate::pallet::Pallet::<Test>::create_multi_key_encrypted_chart(
			RuntimeOrigin::signed(owner),
			test_sizhu_index(),
			Gender::Male,
			test_encrypted_data(),
			test_nonce(),
			test_auth_tag(),
			encrypted_keys,
			test_data_hash(),
		));

		let chart_id = crate::pallet::UserMultiKeyEncryptedCharts::<Test>::get(owner)[0];

		// 授权给 grantee
		let encrypted_key_for_grantee: Vec<u8> = (0..48).collect();
		let encrypted_key_bounded: BoundedVec<u8, ConstU32<72>> =
			BoundedVec::try_from(encrypted_key_for_grantee).unwrap();

		assert_ok!(crate::pallet::Pallet::<Test>::grant_chart_access(
			RuntimeOrigin::signed(owner),
			chart_id,
			grantee,
			encrypted_key_bounded,
			AccessRole::Master,
			AccessScope::CanComment,
			0, // 永久有效
		));

		// 验证授权
		let chart = crate::pallet::MultiKeyEncryptedChartById::<Test>::get(chart_id).unwrap();
		assert_eq!(chart.encrypted_keys.len(), 2);
		assert!(chart.get_key_entry(&grantee).is_some());

		// 验证 ProviderGrants 索引
		let grants = crate::pallet::ProviderGrants::<Test>::get(grantee);
		assert!(grants.contains(&chart_id));
	});
}

#[test]
fn test_grant_chart_access_not_owner() {
	new_test_ext().execute_with(|| {
		let owner = 1u64;
		let attacker = 2u64;
		let grantee = 3u64;
		let owner_key = test_public_key(0x11);
		let attacker_key = test_public_key(0x22);
		let grantee_key = test_public_key(0x33);

		// 注册所有人的加密公钥
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(owner),
			owner_key,
		));
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(attacker),
			attacker_key,
		));
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(grantee),
			grantee_key,
		));

		// owner 创建命盘
		let owner_entry = test_encrypted_key_entry(owner, AccessRole::Owner, AccessScope::FullAccess);
		let encrypted_keys: BoundedVec<EncryptedKeyEntry<u64>, ConstU32<10>> =
			BoundedVec::try_from(vec![owner_entry]).unwrap();

		assert_ok!(crate::pallet::Pallet::<Test>::create_multi_key_encrypted_chart(
			RuntimeOrigin::signed(owner),
			test_sizhu_index(),
			Gender::Male,
			test_encrypted_data(),
			test_nonce(),
			test_auth_tag(),
			encrypted_keys,
			test_data_hash(),
		));

		let chart_id = crate::pallet::UserMultiKeyEncryptedCharts::<Test>::get(owner)[0];

		// attacker 尝试授权应失败
		let encrypted_key_for_grantee: Vec<u8> = (0..48).collect();
		let encrypted_key_bounded: BoundedVec<u8, ConstU32<72>> =
			BoundedVec::try_from(encrypted_key_for_grantee).unwrap();

		assert_noop!(
			crate::pallet::Pallet::<Test>::grant_chart_access(
				RuntimeOrigin::signed(attacker), // 非所有者
				chart_id,
				grantee,
				encrypted_key_bounded,
				AccessRole::Master,
				AccessScope::CanComment,
				0,
			),
			crate::Error::<Test>::NotChartOwner
		);
	});
}

// ================================
// 撤销访问测试
// ================================

#[test]
fn test_revoke_chart_access_success() {
	new_test_ext().execute_with(|| {
		let owner = 1u64;
		let grantee = 2u64;
		let owner_key = test_public_key(0x11);
		let grantee_key = test_public_key(0x22);

		// 注册双方加密公钥
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(owner),
			owner_key,
		));
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(grantee),
			grantee_key,
		));

		// 创建命盘并授权
		let owner_entry = test_encrypted_key_entry(owner, AccessRole::Owner, AccessScope::FullAccess);
		let encrypted_keys: BoundedVec<EncryptedKeyEntry<u64>, ConstU32<10>> =
			BoundedVec::try_from(vec![owner_entry]).unwrap();

		assert_ok!(crate::pallet::Pallet::<Test>::create_multi_key_encrypted_chart(
			RuntimeOrigin::signed(owner),
			test_sizhu_index(),
			Gender::Male,
			test_encrypted_data(),
			test_nonce(),
			test_auth_tag(),
			encrypted_keys,
			test_data_hash(),
		));

		let chart_id = crate::pallet::UserMultiKeyEncryptedCharts::<Test>::get(owner)[0];

		// 授权
		let encrypted_key_for_grantee: Vec<u8> = (0..48).collect();
		let encrypted_key_bounded: BoundedVec<u8, ConstU32<72>> =
			BoundedVec::try_from(encrypted_key_for_grantee).unwrap();

		assert_ok!(crate::pallet::Pallet::<Test>::grant_chart_access(
			RuntimeOrigin::signed(owner),
			chart_id,
			grantee,
			encrypted_key_bounded,
			AccessRole::Master,
			AccessScope::CanComment,
			0,
		));

		// 撤销授权
		assert_ok!(crate::pallet::Pallet::<Test>::revoke_chart_access(
			RuntimeOrigin::signed(owner),
			chart_id,
			grantee,
		));

		// 验证撤销
		let chart = crate::pallet::MultiKeyEncryptedChartById::<Test>::get(chart_id).unwrap();
		assert_eq!(chart.encrypted_keys.len(), 1); // 只剩 Owner
		assert!(chart.get_key_entry(&grantee).is_none());

		// 验证 ProviderGrants 索引已清理
		let grants = crate::pallet::ProviderGrants::<Test>::get(grantee);
		assert!(!grants.contains(&chart_id));
	});
}

#[test]
fn test_revoke_all_chart_access_success() {
	new_test_ext().execute_with(|| {
		let owner = 1u64;
		let grantee1 = 2u64;
		let grantee2 = 3u64;
		let owner_key = test_public_key(0x11);
		let grantee1_key = test_public_key(0x22);
		let grantee2_key = test_public_key(0x33);

		// 注册所有人的加密公钥
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(owner),
			owner_key,
		));
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(grantee1),
			grantee1_key,
		));
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(grantee2),
			grantee2_key,
		));

		// 创建命盘
		let owner_entry = test_encrypted_key_entry(owner, AccessRole::Owner, AccessScope::FullAccess);
		let encrypted_keys: BoundedVec<EncryptedKeyEntry<u64>, ConstU32<10>> =
			BoundedVec::try_from(vec![owner_entry]).unwrap();

		assert_ok!(crate::pallet::Pallet::<Test>::create_multi_key_encrypted_chart(
			RuntimeOrigin::signed(owner),
			test_sizhu_index(),
			Gender::Male,
			test_encrypted_data(),
			test_nonce(),
			test_auth_tag(),
			encrypted_keys,
			test_data_hash(),
		));

		let chart_id = crate::pallet::UserMultiKeyEncryptedCharts::<Test>::get(owner)[0];

		// 授权给两人
		let encrypted_key1: BoundedVec<u8, ConstU32<72>> =
			BoundedVec::try_from((0..48).collect::<Vec<u8>>()).unwrap();
		let encrypted_key2: BoundedVec<u8, ConstU32<72>> =
			BoundedVec::try_from((0..48).collect::<Vec<u8>>()).unwrap();

		assert_ok!(crate::pallet::Pallet::<Test>::grant_chart_access(
			RuntimeOrigin::signed(owner),
			chart_id,
			grantee1,
			encrypted_key1,
			AccessRole::Master,
			AccessScope::CanComment,
			0,
		));
		assert_ok!(crate::pallet::Pallet::<Test>::grant_chart_access(
			RuntimeOrigin::signed(owner),
			chart_id,
			grantee2,
			encrypted_key2,
			AccessRole::Family,
			AccessScope::ReadOnly,
			0,
		));

		// 撤销所有授权
		assert_ok!(crate::pallet::Pallet::<Test>::revoke_all_chart_access(
			RuntimeOrigin::signed(owner),
			chart_id,
		));

		// 验证只剩 Owner
		let chart = crate::pallet::MultiKeyEncryptedChartById::<Test>::get(chart_id).unwrap();
		assert_eq!(chart.encrypted_keys.len(), 1);
		assert!(chart.get_owner_key().is_some());

		// 验证 ProviderGrants 索引已全部清理
		let grants1 = crate::pallet::ProviderGrants::<Test>::get(grantee1);
		let grants2 = crate::pallet::ProviderGrants::<Test>::get(grantee2);
		assert!(!grants1.contains(&chart_id));
		assert!(!grants2.contains(&chart_id));
	});
}

// ================================
// 删除多方授权加密命盘测试
// ================================

#[test]
fn test_delete_multi_key_encrypted_chart_success() {
	new_test_ext().execute_with(|| {
		let owner = 1u64;
		let grantee = 2u64;
		let owner_key = test_public_key(0x11);
		let grantee_key = test_public_key(0x22);

		// 注册双方加密公钥
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(owner),
			owner_key,
		));
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(grantee),
			grantee_key,
		));

		// 创建命盘并授权
		let owner_entry = test_encrypted_key_entry(owner, AccessRole::Owner, AccessScope::FullAccess);
		let encrypted_keys: BoundedVec<EncryptedKeyEntry<u64>, ConstU32<10>> =
			BoundedVec::try_from(vec![owner_entry]).unwrap();

		assert_ok!(crate::pallet::Pallet::<Test>::create_multi_key_encrypted_chart(
			RuntimeOrigin::signed(owner),
			test_sizhu_index(),
			Gender::Male,
			test_encrypted_data(),
			test_nonce(),
			test_auth_tag(),
			encrypted_keys,
			test_data_hash(),
		));

		let chart_id = crate::pallet::UserMultiKeyEncryptedCharts::<Test>::get(owner)[0];

		// 授权给 grantee
		let encrypted_key_for_grantee: BoundedVec<u8, ConstU32<72>> =
			BoundedVec::try_from((0..48).collect::<Vec<u8>>()).unwrap();

		assert_ok!(crate::pallet::Pallet::<Test>::grant_chart_access(
			RuntimeOrigin::signed(owner),
			chart_id,
			grantee,
			encrypted_key_for_grantee,
			AccessRole::Master,
			AccessScope::CanComment,
			0,
		));

		// 删除命盘
		assert_ok!(crate::pallet::Pallet::<Test>::delete_multi_key_encrypted_chart(
			RuntimeOrigin::signed(owner),
			chart_id,
		));

		// 验证删除
		assert!(!crate::pallet::MultiKeyEncryptedChartById::<Test>::contains_key(chart_id));
		let user_charts = crate::pallet::UserMultiKeyEncryptedCharts::<Test>::get(owner);
		assert!(!user_charts.contains(&chart_id));

		// 验证 ProviderGrants 索引已清理
		let grants = crate::pallet::ProviderGrants::<Test>::get(grantee);
		assert!(!grants.contains(&chart_id));
	});
}

#[test]
fn test_delete_multi_key_encrypted_chart_not_owner() {
	new_test_ext().execute_with(|| {
		let owner = 1u64;
		let attacker = 2u64;
		let owner_key = test_public_key(0x11);
		let attacker_key = test_public_key(0x22);

		// 注册双方加密公钥
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(owner),
			owner_key,
		));
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(attacker),
			attacker_key,
		));

		// owner 创建命盘
		let owner_entry = test_encrypted_key_entry(owner, AccessRole::Owner, AccessScope::FullAccess);
		let encrypted_keys: BoundedVec<EncryptedKeyEntry<u64>, ConstU32<10>> =
			BoundedVec::try_from(vec![owner_entry]).unwrap();

		assert_ok!(crate::pallet::Pallet::<Test>::create_multi_key_encrypted_chart(
			RuntimeOrigin::signed(owner),
			test_sizhu_index(),
			Gender::Male,
			test_encrypted_data(),
			test_nonce(),
			test_auth_tag(),
			encrypted_keys,
			test_data_hash(),
		));

		let chart_id = crate::pallet::UserMultiKeyEncryptedCharts::<Test>::get(owner)[0];

		// attacker 尝试删除应失败
		assert_noop!(
			crate::pallet::Pallet::<Test>::delete_multi_key_encrypted_chart(
				RuntimeOrigin::signed(attacker),
				chart_id,
			),
			crate::Error::<Test>::NotChartOwner
		);
	});
}

// ================================
// ProviderGrants 索引完整性测试
// ================================

#[test]
fn test_provider_grants_index_integrity() {
	new_test_ext().execute_with(|| {
		let owner = 1u64;
		let master = 2u64;
		let owner_key = test_public_key(0x11);
		let master_key = test_public_key(0x22);

		// 注册加密公钥
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(owner),
			owner_key,
		));
		assert_ok!(crate::pallet::Pallet::<Test>::register_encryption_key(
			RuntimeOrigin::signed(master),
			master_key,
		));

		// 创建两个命盘
		for _ in 0..2 {
			let owner_entry = test_encrypted_key_entry(owner, AccessRole::Owner, AccessScope::FullAccess);
			let encrypted_keys: BoundedVec<EncryptedKeyEntry<u64>, ConstU32<10>> =
				BoundedVec::try_from(vec![owner_entry]).unwrap();

			assert_ok!(crate::pallet::Pallet::<Test>::create_multi_key_encrypted_chart(
				RuntimeOrigin::signed(owner),
				test_sizhu_index(),
				Gender::Male,
				test_encrypted_data(),
				test_nonce(),
				test_auth_tag(),
				encrypted_keys,
				test_data_hash(),
			));
		}

		let user_charts = crate::pallet::UserMultiKeyEncryptedCharts::<Test>::get(owner);
		let chart_id_1 = user_charts[0];
		let chart_id_2 = user_charts[1];

		// 将 master 授权给两个命盘
		let encrypted_key: BoundedVec<u8, ConstU32<72>> =
			BoundedVec::try_from((0..48).collect::<Vec<u8>>()).unwrap();

		assert_ok!(crate::pallet::Pallet::<Test>::grant_chart_access(
			RuntimeOrigin::signed(owner),
			chart_id_1,
			master,
			encrypted_key.clone(),
			AccessRole::Master,
			AccessScope::CanComment,
			0,
		));
		assert_ok!(crate::pallet::Pallet::<Test>::grant_chart_access(
			RuntimeOrigin::signed(owner),
			chart_id_2,
			master,
			encrypted_key.clone(),
			AccessRole::Master,
			AccessScope::CanComment,
			0,
		));

		// 验证 ProviderGrants 包含两个命盘
		let grants = crate::pallet::ProviderGrants::<Test>::get(master);
		assert_eq!(grants.len(), 2);
		assert!(grants.contains(&chart_id_1));
		assert!(grants.contains(&chart_id_2));

		// 撤销第一个命盘的授权
		assert_ok!(crate::pallet::Pallet::<Test>::revoke_chart_access(
			RuntimeOrigin::signed(owner),
			chart_id_1,
			master,
		));

		// 验证 ProviderGrants 只剩一个
		let grants = crate::pallet::ProviderGrants::<Test>::get(master);
		assert_eq!(grants.len(), 1);
		assert!(!grants.contains(&chart_id_1));
		assert!(grants.contains(&chart_id_2));

		// 删除第二个命盘
		assert_ok!(crate::pallet::Pallet::<Test>::delete_multi_key_encrypted_chart(
			RuntimeOrigin::signed(owner),
			chart_id_2,
		));

		// 验证 ProviderGrants 完全清空
		let grants = crate::pallet::ProviderGrants::<Test>::get(master);
		assert!(grants.is_empty());
	});
}

// ================================
// 统一隐私模式测试 (Phase 1.2.4)
// ================================

#[test]
fn test_create_bazi_chart_encrypted_public_mode() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// Public 模式：所有数据明文存储
		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 14,
			minute: 30,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart_encrypted(
			RuntimeOrigin::signed(account_id),
			0, // Public 模式
			Some(b"test".to_vec().try_into().unwrap()), // name
			Some(input),
			Some(Gender::Male),
			Some(ZiShiMode::Modern),
			None, // longitude
			None, // encrypted_data (Public 模式不需要)
			None, // data_hash
			None, // owner_key_backup
		));

		// 验证用户的命盘列表
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		assert_eq!(user_charts.len(), 1);

		// 验证命盘存储
		let chart_id = user_charts[0];
		let chart = crate::pallet::ChartById::<Test>::get(chart_id).unwrap();
		assert_eq!(chart.owner, account_id);
		assert_eq!(chart.privacy_mode, pallet_divination_privacy::types::PrivacyMode::Public);
		assert!(chart.encrypted_fields.is_none()); // Public 模式无加密字段
		assert!(chart.sensitive_data_hash.is_none()); // 无敏感数据哈希

		// 验证计算数据存在
		assert!(chart.birth_time.is_some());
		assert!(chart.sizhu.is_some());
		assert!(chart.dayun.is_some());

		// 验证没有加密数据存储
		assert!(!crate::pallet::EncryptedData::<Test>::contains_key(chart_id));
		assert!(!crate::pallet::OwnerKeyBackup::<Test>::contains_key(chart_id));
	});
}

#[test]
fn test_create_bazi_chart_encrypted_partial_mode() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 准备加密数据（模拟前端加密的敏感数据）
		let encrypted_data: Vec<u8> = (0..128).collect();
		let data_hash: [u8; 32] = [0xAB; 32];
		let owner_key_backup: [u8; 92] = [0xCD; 92];

		// Partial 模式：计算数据明文 + 敏感数据加密
		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 14,
			minute: 30,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart_encrypted(
			RuntimeOrigin::signed(account_id),
			1, // Partial 模式
			Some(b"partial_test".to_vec().try_into().unwrap()),
			Some(input),
			Some(Gender::Female),
			Some(ZiShiMode::Traditional),
			None,
			Some(BoundedVec::try_from(encrypted_data.clone()).unwrap()),
			Some(data_hash),
			Some(owner_key_backup),
		));

		// 验证命盘列表
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		assert_eq!(user_charts.len(), 1);

		let chart_id = user_charts[0];
		let chart = crate::pallet::ChartById::<Test>::get(chart_id).unwrap();

		// 验证隐私模式
		assert_eq!(chart.privacy_mode, pallet_divination_privacy::types::PrivacyMode::Partial);
		assert_eq!(chart.encrypted_fields, Some(0x0F)); // 敏感字段加密标记
		assert_eq!(chart.sensitive_data_hash, Some(data_hash));

		// 验证计算数据存在（Partial 模式保留计算数据）
		assert!(chart.birth_time.is_some());
		assert!(chart.sizhu.is_some());
		assert!(chart.dayun.is_some());
		assert!(chart.wuxing_strength.is_some());

		// 验证加密数据已存储
		assert!(crate::pallet::EncryptedData::<Test>::contains_key(chart_id));
		let stored_encrypted_data = crate::pallet::EncryptedData::<Test>::get(chart_id).unwrap();
		assert_eq!(stored_encrypted_data.len(), 128);

		// 验证密钥备份已存储
		assert!(crate::pallet::OwnerKeyBackup::<Test>::contains_key(chart_id));
		let stored_key_backup = crate::pallet::OwnerKeyBackup::<Test>::get(chart_id).unwrap();
		assert_eq!(stored_key_backup, owner_key_backup);
	});
}

#[test]
fn test_create_bazi_chart_encrypted_private_mode() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 准备加密数据
		let encrypted_data: Vec<u8> = (0..=255).collect();
		let data_hash: [u8; 32] = [0xEF; 32];
		let owner_key_backup: [u8; 92] = [0x12; 92];

		// Private 模式：所有数据加密
		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart_encrypted(
			RuntimeOrigin::signed(account_id),
			2, // Private 模式
			Some(b"private_test".to_vec().try_into().unwrap()),
			None, // 计算参数可选（Private 模式）
			None,
			None,
			None,
			Some(BoundedVec::try_from(encrypted_data.clone()).unwrap()),
			Some(data_hash),
			Some(owner_key_backup),
		));

		// 验证命盘列表
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		assert_eq!(user_charts.len(), 1);

		let chart_id = user_charts[0];
		let chart = crate::pallet::ChartById::<Test>::get(chart_id).unwrap();

		// 验证隐私模式
		assert_eq!(chart.privacy_mode, pallet_divination_privacy::types::PrivacyMode::Private);
		assert_eq!(chart.encrypted_fields, Some(0xFF)); // 所有字段加密
		assert_eq!(chart.sensitive_data_hash, Some(data_hash));

		// 验证计算数据为 None（Private 模式不存储计算数据）
		assert!(chart.birth_time.is_none());
		assert!(chart.sizhu.is_none());
		assert!(chart.dayun.is_none());
		assert!(chart.wuxing_strength.is_none());
		assert!(chart.gender.is_none());
		assert!(chart.zishi_mode.is_none());

		// 验证加密数据已存储
		assert!(crate::pallet::EncryptedData::<Test>::contains_key(chart_id));
		assert!(crate::pallet::OwnerKeyBackup::<Test>::contains_key(chart_id));
	});
}

#[test]
fn test_create_bazi_chart_encrypted_invalid_privacy_mode() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 无效的隐私模式 (> 2)
		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 14,
			minute: 30,
		};

		assert_noop!(
			crate::pallet::Pallet::<Test>::create_bazi_chart_encrypted(
				RuntimeOrigin::signed(account_id),
				3, // 无效模式
				None,
				Some(input),
				Some(Gender::Male),
				Some(ZiShiMode::Modern),
				None,
				None,
				None,
				None,
			),
			crate::Error::<Test>::InvalidPrivacyMode
		);
	});
}

#[test]
fn test_create_bazi_chart_encrypted_public_mode_with_encrypted_data() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// Public 模式不应包含加密数据
		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 14,
			minute: 30,
		};
		let encrypted_data: Vec<u8> = (0..64).collect();

		assert_noop!(
			crate::pallet::Pallet::<Test>::create_bazi_chart_encrypted(
				RuntimeOrigin::signed(account_id),
				0, // Public 模式
				None,
				Some(input),
				Some(Gender::Male),
				Some(ZiShiMode::Modern),
				None,
				Some(BoundedVec::try_from(encrypted_data).unwrap()), // 不应有加密数据
				None,
				None,
			),
			crate::Error::<Test>::PublicModeNoEncryptedData
		);
	});
}

#[test]
fn test_create_bazi_chart_encrypted_partial_mode_missing_encrypted_data() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// Partial 模式必须有加密数据
		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 14,
			minute: 30,
		};

		assert_noop!(
			crate::pallet::Pallet::<Test>::create_bazi_chart_encrypted(
				RuntimeOrigin::signed(account_id),
				1, // Partial 模式
				None,
				Some(input),
				Some(Gender::Male),
				Some(ZiShiMode::Modern),
				None,
				None, // 缺少加密数据
				None,
				None,
			),
			crate::Error::<Test>::EncryptedDataRequired
		);
	});
}

#[test]
fn test_create_bazi_chart_encrypted_partial_mode_missing_calculation_params() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// Partial 模式必须有计算参数
		let encrypted_data: Vec<u8> = (0..64).collect();
		let data_hash: [u8; 32] = [0xAB; 32];
		let owner_key_backup: [u8; 92] = [0xCD; 92];

		assert_noop!(
			crate::pallet::Pallet::<Test>::create_bazi_chart_encrypted(
				RuntimeOrigin::signed(account_id),
				1, // Partial 模式
				None,
				None, // 缺少 input
				Some(Gender::Male),
				Some(ZiShiMode::Modern),
				None,
				Some(BoundedVec::try_from(encrypted_data).unwrap()),
				Some(data_hash),
				Some(owner_key_backup),
			),
			crate::Error::<Test>::PartialModeRequiresCalculationParams
		);
	});
}

#[test]
fn test_update_encrypted_data_success() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 先创建 Partial 模式命盘
		let encrypted_data: Vec<u8> = (0..64).collect();
		let data_hash: [u8; 32] = [0xAB; 32];
		let owner_key_backup: [u8; 92] = [0xCD; 92];

		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 14,
			minute: 30,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart_encrypted(
			RuntimeOrigin::signed(account_id),
			1, // Partial 模式
			None,
			Some(input),
			Some(Gender::Male),
			Some(ZiShiMode::Modern),
			None,
			Some(BoundedVec::try_from(encrypted_data).unwrap()),
			Some(data_hash),
			Some(owner_key_backup),
		));

		let chart_id = crate::pallet::UserCharts::<Test>::get(account_id)[0];

		// 更新加密数据
		let new_encrypted_data: Vec<u8> = (100..200).collect();
		let new_data_hash: [u8; 32] = [0xEF; 32];
		let new_owner_key_backup: [u8; 92] = [0x12; 92];

		assert_ok!(crate::pallet::Pallet::<Test>::update_encrypted_data(
			RuntimeOrigin::signed(account_id),
			chart_id,
			BoundedVec::try_from(new_encrypted_data.clone()).unwrap(),
			new_data_hash,
			new_owner_key_backup,
		));

		// 验证更新后的加密数据
		let stored_encrypted_data = crate::pallet::EncryptedData::<Test>::get(chart_id).unwrap();
		assert_eq!(stored_encrypted_data.len(), 100);
		assert_eq!(stored_encrypted_data[0], 100);

		// 验证更新后的密钥备份
		let stored_key_backup = crate::pallet::OwnerKeyBackup::<Test>::get(chart_id).unwrap();
		assert_eq!(stored_key_backup, new_owner_key_backup);

		// 验证命盘的数据哈希已更新
		let chart = crate::pallet::ChartById::<Test>::get(chart_id).unwrap();
		assert_eq!(chart.sensitive_data_hash, Some(new_data_hash));
	});
}

#[test]
fn test_update_encrypted_data_not_owner() {
	new_test_ext().execute_with(|| {
		let owner = 1u64;
		let attacker = 2u64;

		// owner 创建 Partial 模式命盘
		let encrypted_data: Vec<u8> = (0..64).collect();
		let data_hash: [u8; 32] = [0xAB; 32];
		let owner_key_backup: [u8; 92] = [0xCD; 92];

		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 14,
			minute: 30,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart_encrypted(
			RuntimeOrigin::signed(owner),
			1,
			None,
			Some(input),
			Some(Gender::Male),
			Some(ZiShiMode::Modern),
			None,
			Some(BoundedVec::try_from(encrypted_data).unwrap()),
			Some(data_hash),
			Some(owner_key_backup),
		));

		let chart_id = crate::pallet::UserCharts::<Test>::get(owner)[0];

		// attacker 尝试更新应失败
		let new_encrypted_data: Vec<u8> = (100..200).collect();
		let new_data_hash: [u8; 32] = [0xEF; 32];
		let new_owner_key_backup: [u8; 92] = [0x12; 92];

		assert_noop!(
			crate::pallet::Pallet::<Test>::update_encrypted_data(
				RuntimeOrigin::signed(attacker), // 非所有者
				chart_id,
				BoundedVec::try_from(new_encrypted_data).unwrap(),
				new_data_hash,
				new_owner_key_backup,
			),
			crate::Error::<Test>::NotChartOwner
		);
	});
}

#[test]
fn test_update_encrypted_data_on_public_mode() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 创建 Public 模式命盘
		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 14,
			minute: 30,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart_encrypted(
			RuntimeOrigin::signed(account_id),
			0, // Public 模式
			None,
			Some(input),
			Some(Gender::Male),
			Some(ZiShiMode::Modern),
			None,
			None,
			None,
			None,
		));

		let chart_id = crate::pallet::UserCharts::<Test>::get(account_id)[0];

		// 尝试在 Public 模式命盘上更新加密数据应失败
		let new_encrypted_data: Vec<u8> = (100..200).collect();
		let new_data_hash: [u8; 32] = [0xEF; 32];
		let new_owner_key_backup: [u8; 92] = [0x12; 92];

		assert_noop!(
			crate::pallet::Pallet::<Test>::update_encrypted_data(
				RuntimeOrigin::signed(account_id),
				chart_id,
				BoundedVec::try_from(new_encrypted_data).unwrap(),
				new_data_hash,
				new_owner_key_backup,
			),
			crate::Error::<Test>::PublicModeNoEncryptedData
		);
	});
}

#[test]
fn test_update_encrypted_data_chart_not_found() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;
		let non_existent_chart_id = 9999u64;

		let new_encrypted_data: Vec<u8> = (100..200).collect();
		let new_data_hash: [u8; 32] = [0xEF; 32];
		let new_owner_key_backup: [u8; 92] = [0x12; 92];

		assert_noop!(
			crate::pallet::Pallet::<Test>::update_encrypted_data(
				RuntimeOrigin::signed(account_id),
				non_existent_chart_id,
				BoundedVec::try_from(new_encrypted_data).unwrap(),
				new_data_hash,
				new_owner_key_backup,
			),
			crate::Error::<Test>::ChartNotFound
		);
	});
}

#[test]
fn test_delete_bazi_chart_with_encrypted_data() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 创建 Partial 模式命盘
		let encrypted_data: Vec<u8> = (0..64).collect();
		let data_hash: [u8; 32] = [0xAB; 32];
		let owner_key_backup: [u8; 92] = [0xCD; 92];

		let input = BaziInputType::Solar {
			year: 1990,
			month: 11,
			day: 15,
			hour: 14,
			minute: 30,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart_encrypted(
			RuntimeOrigin::signed(account_id),
			1, // Partial 模式
			None,
			Some(input),
			Some(Gender::Male),
			Some(ZiShiMode::Modern),
			None,
			Some(BoundedVec::try_from(encrypted_data).unwrap()),
			Some(data_hash),
			Some(owner_key_backup),
		));

		let chart_id = crate::pallet::UserCharts::<Test>::get(account_id)[0];

		// 验证加密数据存在
		assert!(crate::pallet::EncryptedData::<Test>::contains_key(chart_id));
		assert!(crate::pallet::OwnerKeyBackup::<Test>::contains_key(chart_id));

		// 删除命盘
		assert_ok!(crate::pallet::Pallet::<Test>::delete_bazi_chart(
			RuntimeOrigin::signed(account_id),
			chart_id,
		));

		// 验证命盘已删除
		assert!(!crate::pallet::ChartById::<Test>::contains_key(chart_id));

		// 注意：当前实现中，delete_bazi_chart 不会自动清理 EncryptedData 和 OwnerKeyBackup
		// 这是一个潜在的优化点，可以在后续版本中添加
	});
}

#[test]
fn test_privacy_mode_with_lunar_input() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 使用农历输入创建 Partial 模式命盘
		let encrypted_data: Vec<u8> = (0..64).collect();
		let data_hash: [u8; 32] = [0xAB; 32];
		let owner_key_backup: [u8; 92] = [0xCD; 92];

		let input = BaziInputType::Lunar {
			year: 2024,
			month: 1,
			day: 1,
			is_leap_month: false,
			hour: 12,
			minute: 30,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart_encrypted(
			RuntimeOrigin::signed(account_id),
			1, // Partial 模式
			Some(b"lunar_partial".to_vec().try_into().unwrap()),
			Some(input),
			Some(Gender::Male),
			Some(ZiShiMode::Modern),
			None,
			Some(BoundedVec::try_from(encrypted_data).unwrap()),
			Some(data_hash),
			Some(owner_key_backup),
		));

		let chart_id = crate::pallet::UserCharts::<Test>::get(account_id)[0];
		let chart = crate::pallet::ChartById::<Test>::get(chart_id).unwrap();

		// 验证隐私模式
		assert_eq!(chart.privacy_mode, pallet_divination_privacy::types::PrivacyMode::Partial);

		// 验证农历输入已转换为公历
		let birth_time = chart.birth_time.unwrap();
		assert_eq!(birth_time.year, 2024);
		assert_eq!(birth_time.month, 2); // 农历正月初一 = 公历2024年2月10日
		assert_eq!(birth_time.day, 10);

		// 验证输入日历类型
		assert_eq!(chart.input_calendar_type, Some(crate::types::InputCalendarType::Lunar));
	});
}

#[test]
fn test_privacy_mode_with_sizhu_input() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 使用四柱直接输入创建 Partial 模式命盘
		let encrypted_data: Vec<u8> = (0..64).collect();
		let data_hash: [u8; 32] = [0xAB; 32];
		let owner_key_backup: [u8; 92] = [0xCD; 92];

		let input = BaziInputType::SiZhu {
			year_gz: 0,   // 甲子
			month_gz: 2,  // 丙寅
			day_gz: 4,    // 戊辰
			hour_gz: 0,   // 甲子
			birth_year: 1984,
		};

		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart_encrypted(
			RuntimeOrigin::signed(account_id),
			1, // Partial 模式
			Some(b"sizhu_partial".to_vec().try_into().unwrap()),
			Some(input),
			Some(Gender::Male),
			Some(ZiShiMode::Modern),
			None,
			Some(BoundedVec::try_from(encrypted_data).unwrap()),
			Some(data_hash),
			Some(owner_key_backup),
		));

		let chart_id = crate::pallet::UserCharts::<Test>::get(account_id)[0];
		let chart = crate::pallet::ChartById::<Test>::get(chart_id).unwrap();

		// 验证隐私模式
		assert_eq!(chart.privacy_mode, pallet_divination_privacy::types::PrivacyMode::Partial);

		// 验证四柱
		let sizhu = chart.sizhu.unwrap();
		assert_eq!(sizhu.year_zhu.ganzhi.gan.0, 0); // 甲
		assert_eq!(sizhu.year_zhu.ganzhi.zhi.0, 0); // 子

		// 验证输入日历类型
		assert_eq!(chart.input_calendar_type, Some(crate::types::InputCalendarType::SiZhu));
	});
}

#[test]
fn test_multiple_privacy_mode_charts() {
	new_test_ext().execute_with(|| {
		let account_id = 1u64;

		// 创建 Public 模式命盘
		let input1 = BaziInputType::Solar {
			year: 1990,
			month: 1,
			day: 1,
			hour: 12,
			minute: 0,
		};
		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart_encrypted(
			RuntimeOrigin::signed(account_id),
			0,
			None,
			Some(input1),
			Some(Gender::Male),
			Some(ZiShiMode::Modern),
			None,
			None,
			None,
			None,
		));

		// 创建 Partial 模式命盘
		let input2 = BaziInputType::Solar {
			year: 1991,
			month: 2,
			day: 2,
			hour: 14,
			minute: 0,
		};
		let encrypted_data: Vec<u8> = (0..64).collect();
		let data_hash: [u8; 32] = [0xAB; 32];
		let owner_key_backup: [u8; 92] = [0xCD; 92];
		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart_encrypted(
			RuntimeOrigin::signed(account_id),
			1,
			None,
			Some(input2),
			Some(Gender::Female),
			Some(ZiShiMode::Traditional),
			None,
			Some(BoundedVec::try_from(encrypted_data.clone()).unwrap()),
			Some(data_hash),
			Some(owner_key_backup),
		));

		// 创建 Private 模式命盘
		let data_hash2: [u8; 32] = [0xEF; 32];
		let owner_key_backup2: [u8; 92] = [0x12; 92];
		assert_ok!(crate::pallet::Pallet::<Test>::create_bazi_chart_encrypted(
			RuntimeOrigin::signed(account_id),
			2,
			None,
			None,
			None,
			None,
			None,
			Some(BoundedVec::try_from(encrypted_data).unwrap()),
			Some(data_hash2),
			Some(owner_key_backup2),
		));

		// 验证创建了 3 个命盘
		let user_charts = crate::pallet::UserCharts::<Test>::get(account_id);
		assert_eq!(user_charts.len(), 3);

		// 验证各命盘的隐私模式
		let chart1 = crate::pallet::ChartById::<Test>::get(user_charts[0]).unwrap();
		assert_eq!(chart1.privacy_mode, pallet_divination_privacy::types::PrivacyMode::Public);

		let chart2 = crate::pallet::ChartById::<Test>::get(user_charts[1]).unwrap();
		assert_eq!(chart2.privacy_mode, pallet_divination_privacy::types::PrivacyMode::Partial);

		let chart3 = crate::pallet::ChartById::<Test>::get(user_charts[2]).unwrap();
		assert_eq!(chart3.privacy_mode, pallet_divination_privacy::types::PrivacyMode::Private);
	});
}
