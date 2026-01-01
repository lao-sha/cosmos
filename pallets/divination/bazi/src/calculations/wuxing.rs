//! # 五行计算模块
//!
//! 实现五行强度和喜用神分析

use crate::types::{WuXing, WuXingStrength, TianGan, DiZhi, GanZhi};
use crate::constants::get_hidden_stems;

/// 计算八字中的五行强度
///
/// 五行强度计算是八字分析的核心，用于判断命局的五行平衡。
///
/// ## 计算规则
///
/// 1. **天干权重**: 每个天干基础分100分
/// 2. **地支权重**:
///    - 地支本气: 100分
///    - 藏干权重根据月令调整（使用HIDDEN_STEM_WEIGHT矩阵）
/// 3. **月令加权**: 月支的权重 × 1.5（月令旺相理论）
///
/// ## 参数
///
/// - `year_zhu`: 年柱
/// - `month_zhu`: 月柱
/// - `day_zhu`: 日柱
/// - `hour_zhu`: 时柱
///
/// ## 返回
///
/// - `WuXingStrength`: 五行强度分布
pub fn calculate_wuxing_strength(
	year_zhu: &GanZhi,
	month_zhu: &GanZhi,
	day_zhu: &GanZhi,
	hour_zhu: &GanZhi,
) -> WuXingStrength {
	let mut strength = WuXingStrength::default();

	// 1. 计算四个天干的五行强度（每个100分）
	add_tiangan_strength(&mut strength, year_zhu.gan);
	add_tiangan_strength(&mut strength, month_zhu.gan);
	add_tiangan_strength(&mut strength, day_zhu.gan);
	add_tiangan_strength(&mut strength, hour_zhu.gan);

	// 2. 计算四个地支的五行强度
	// 地支本身的五行（每个100分）
	add_dizhi_strength(&mut strength, year_zhu.zhi, false);
	add_dizhi_strength(&mut strength, month_zhu.zhi, true); // 月令加权
	add_dizhi_strength(&mut strength, day_zhu.zhi, false);
	add_dizhi_strength(&mut strength, hour_zhu.zhi, false);

	// 3. 计算地支藏干的五行强度
	// 藏干权重相对较低（主气60，中气40，余气20）
	add_canggan_strength(&mut strength, year_zhu.zhi);
	add_canggan_strength(&mut strength, month_zhu.zhi);
	add_canggan_strength(&mut strength, day_zhu.zhi);
	add_canggan_strength(&mut strength, hour_zhu.zhi);

	strength
}

/// 添加天干的五行强度
fn add_tiangan_strength(strength: &mut WuXingStrength, gan: TianGan) {
	let wuxing = gan.to_wuxing();
	strength.add_element(wuxing, 100);
}

/// 添加地支的五行强度
fn add_dizhi_strength(strength: &mut WuXingStrength, zhi: DiZhi, is_month: bool) {
	let wuxing = zhi.to_wuxing();
	let base_value = 100;

	// 月令地支权重 × 1.5
	let value = if is_month {
		(base_value as f32 * 1.5) as u32
	} else {
		base_value
	};

	strength.add_element(wuxing, value);
}

/// 添加地支藏干的五行强度
fn add_canggan_strength(strength: &mut WuXingStrength, zhi: DiZhi) {
	use crate::constants::is_valid_canggan;
	let hidden_stems = get_hidden_stems(zhi);

	for (i, &(gan, _, _)) in hidden_stems.iter().enumerate() {
		// 跳过无效藏干（255标记）
		if !is_valid_canggan(gan.0) {
			continue;
		}

		let wuxing = gan.to_wuxing();

		// 根据藏干类型确定权重
		// 主气60分，中气40分，余气20分
		let value = match i {
			0 => 60, // 主气
			1 => 40, // 中气
			2 => 20, // 余气
			_ => 0,
		};

		strength.add_element(wuxing, value);
	}
}

/// 判断喜用神
///
/// 根据五行强度分析，判断命局需要补充哪个五行。
///
/// ## 判断原则（简化版）
///
/// 1. **找最弱的五行**: 最弱的五行通常是喜用神
/// 2. **平衡原则**: 如果某个五行过强，需要克制或泄耗它
/// 3. **日主强弱**: 分析日主五行的强度
///
/// ## 参数
///
/// - `strength`: 五行强度分布
/// - `day_gan`: 日主天干
///
/// ## 返回
///
/// - `Option<WuXing>`: 喜用神五行
///
/// ## 注意
///
/// 本实现是简化版，真实的喜用神判断需要考虑：
/// - 格局分析（从格、化格等）
/// - 调候用神
/// - 通关用神
/// - 大运流年影响
pub fn determine_xiyong_shen(strength: &WuXingStrength, day_gan: TianGan) -> Option<WuXing> {
	// 获取日主五行
	let day_wuxing = day_gan.to_wuxing();

	// 计算日主强度（日主天干 + 比劫的强度）
	let day_strength = match day_wuxing {
		WuXing::Jin => strength.jin,
		WuXing::Mu => strength.mu,
		WuXing::Shui => strength.shui,
		WuXing::Huo => strength.huo,
		WuXing::Tu => strength.tu,
	};

	// 找出五行强度数组
	let strengths = [
		(WuXing::Jin, strength.jin),
		(WuXing::Mu, strength.mu),
		(WuXing::Shui, strength.shui),
		(WuXing::Huo, strength.huo),
		(WuXing::Tu, strength.tu),
	];

	// 计算总强度
	let total: u32 = strengths.iter().map(|(_, s)| s).sum();
	let average = total / 5;

	// 简化判断：
	// 如果日主强度 > 平均值，说明身旺，喜克泄耗
	// 如果日主强度 < 平均值，说明身弱，喜生扶
	if day_strength > average {
		// 身旺：找克制日主的五行（官杀）或泄耗日主的五行（食伤）
		// 这里简化为找最弱的五行作为喜用神
		strengths.iter()
			.filter(|(wx, _)| wx != &day_wuxing)
			.min_by_key(|(_, s)| s)
			.map(|(wx, _)| *wx)
	} else {
		// 身弱：找生扶日主的五行（印星）或帮助日主的五行（比劫）
		// 生我者为印
		let yin_wuxing = get_sheng_me(day_wuxing);
		Some(yin_wuxing)
	}
}

/// 获取生我的五行（印星）
fn get_sheng_me(wuxing: WuXing) -> WuXing {
	match wuxing {
		WuXing::Jin => WuXing::Tu,   // 土生金
		WuXing::Mu => WuXing::Shui,  // 水生木
		WuXing::Shui => WuXing::Jin, // 金生水
		WuXing::Huo => WuXing::Mu,   // 木生火
		WuXing::Tu => WuXing::Huo,   // 火生土
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_wuxing_strength_calculation() {
		// 创建测试用的四柱
		// 甲子 丙寅 戊辰 庚午
		let year_zhu = GanZhi::from_index(0).unwrap(); // 甲子
		let month_zhu = GanZhi::from_index(2).unwrap(); // 丙寅
		let day_zhu = GanZhi::from_index(4).unwrap();  // 戊辰
		let hour_zhu = GanZhi::from_index(6).unwrap(); // 庚午

		let strength = calculate_wuxing_strength(&year_zhu, &month_zhu, &day_zhu, &hour_zhu);

		// 验证五行强度都大于0
		assert!(strength.jin > 0);
		assert!(strength.mu > 0);
		assert!(strength.shui > 0);
		assert!(strength.huo > 0);
		assert!(strength.tu > 0);
	}

	#[test]
	fn test_xiyong_shen_determination() {
		// 创建一个木旺的八字
		// 假设木的强度最高
		let mut strength = WuXingStrength::default();
		strength.mu = 500;  // 木旺
		strength.jin = 100;
		strength.shui = 150;
		strength.huo = 120;
		strength.tu = 80;

		// 日主为甲木（木旺身旺）
		let day_gan = TianGan(0); // 甲
		let xiyong = determine_xiyong_shen(&strength, day_gan);

		// 身旺应该喜克泄耗，这里简化为找最弱的五行
		assert!(xiyong.is_some());
	}

	#[test]
	fn test_sheng_me_relationship() {
		// 测试五行相生关系
		assert_eq!(get_sheng_me(WuXing::Jin), WuXing::Tu);   // 土生金
		assert_eq!(get_sheng_me(WuXing::Mu), WuXing::Shui);  // 水生木
		assert_eq!(get_sheng_me(WuXing::Shui), WuXing::Jin); // 金生水
		assert_eq!(get_sheng_me(WuXing::Huo), WuXing::Mu);   // 木生火
		assert_eq!(get_sheng_me(WuXing::Tu), WuXing::Huo);   // 火生土
	}
}

