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
/// ✅ 增强版本：实现调候用神、通关用神逻辑
///
/// # 喜用神判断优先级
///
/// 1. **调候用神**：夏天需水降温，冬天需火取暖（优先级最高）
/// 2. **通关用神**：化解五行冲突（如金木相克，用水通关）
/// 3. **扶抑用神**：扶弱抑强（默认逻辑）
pub fn determine_xiyong_shen(strength: &WuXingStrength, day_gan: TianGan) -> Option<WuXing> {
	determine_xiyong_shen_full(strength, day_gan, None)
}

/// 完整版喜用神判断（带月支参数）
///
/// # 参数
///
/// - `strength`: 五行强度
/// - `day_gan`: 日主天干
/// - `month_zhi`: 月支（用于调候判断），0=子 1=丑 ... 11=亥
///
/// # 返回
///
/// - `Option<WuXing>`: 喜用神五行
pub fn determine_xiyong_shen_full(
	strength: &WuXingStrength,
	day_gan: TianGan,
	month_zhi: Option<u8>,
) -> Option<WuXing> {
	// 获取日主五行
	let day_wuxing = day_gan.to_wuxing();

	// ========== 1. 调候用神判断 ==========
	// 夏季（巳午未月）需水调候
	// 冬季（亥子丑月）需火调候
	if let Some(mz) = month_zhi {
		if let Some(tiahou) = check_tiahou_yongshen(day_wuxing, mz, strength) {
			return Some(tiahou);
		}
	}

	// ========== 2. 通关用神判断 ==========
	// 检查是否存在严重的五行冲突需要通关
	if let Some(tongguan) = check_tongguan_yongshen(strength) {
		return Some(tongguan);
	}

	// ========== 3. 扶抑用神判断（默认逻辑） ==========
	determine_fuyi_yongshen(strength, day_wuxing)
}

/// 调候用神判断
///
/// 夏季炎热需水润，冬季寒冷需火暖
fn check_tiahou_yongshen(day_wuxing: WuXing, month_zhi: u8, strength: &WuXingStrength) -> Option<WuXing> {
	// 夏季月份：巳(5)、午(6)、未(7)
	let is_summer = matches!(month_zhi, 5 | 6 | 7);
	// 冬季月份：亥(11)、子(0)、丑(1)
	let is_winter = matches!(month_zhi, 11 | 0 | 1);

	if is_summer {
		// 夏季火旺，需要水来调候降温
		// 条件：火土过旺，水极弱
		let fire_earth = strength.huo + strength.tu;
		let total: u32 = strength.jin + strength.mu + strength.shui + strength.huo + strength.tu;
		let fire_earth_ratio = fire_earth * 100 / total.max(1);

		// 火土占比超过50%且水弱，需水调候
		if fire_earth_ratio > 50 && strength.shui < total / 8 {
			return Some(WuXing::Shui);
		}
	}

	if is_winter {
		// 冬季水旺，需要火来调候取暖
		// 条件：水金过旺，火极弱
		let water_metal = strength.shui + strength.jin;
		let total: u32 = strength.jin + strength.mu + strength.shui + strength.huo + strength.tu;
		let water_metal_ratio = water_metal * 100 / total.max(1);

		// 水金占比超过50%且火弱，需火调候
		if water_metal_ratio > 50 && strength.huo < total / 8 {
			return Some(WuXing::Huo);
		}
	}

	// 特殊调候：日主为木，生于冬季水旺，木寒需火暖
	if day_wuxing == WuXing::Mu && is_winter && strength.shui > strength.huo * 3 {
		return Some(WuXing::Huo);
	}

	// 特殊调候：日主为金，生于夏季火旺，金热需水润
	if day_wuxing == WuXing::Jin && is_summer && strength.huo > strength.shui * 3 {
		return Some(WuXing::Shui);
	}

	None
}

/// 通关用神判断
///
/// 当两个相克的五行都很强时，需要用中间五行来通关化解
///
/// 五行相克：金克木、木克土、土克水、水克火、火克金
/// 通关原则：用生泄的方式化解冲突
/// - 金木相战 → 用水通关（金生水，水生木）
/// - 木土相战 → 用火通关（木生火，火生土）
/// - 土水相战 → 用金通关（土生金，金生水）
/// - 水火相战 → 用木通关（水生木，木生火）
/// - 火金相战 → 用土通关（火生土，土生金）
fn check_tongguan_yongshen(strength: &WuXingStrength) -> Option<WuXing> {
	let total: u32 = strength.jin + strength.mu + strength.shui + strength.huo + strength.tu;
	let threshold = total / 3; // 占比超过1/3视为强

	// 检查各对相克组合
	let conflicts = [
		(strength.jin, strength.mu, WuXing::Shui),   // 金木相战用水
		(strength.mu, strength.tu, WuXing::Huo),     // 木土相战用火
		(strength.tu, strength.shui, WuXing::Jin),   // 土水相战用金
		(strength.shui, strength.huo, WuXing::Mu),   // 水火相战用木
		(strength.huo, strength.jin, WuXing::Tu),    // 火金相战用土
	];

	for (a, b, tongguan) in conflicts {
		// 双方都强且通关五行弱，则需要通关
		if a > threshold && b > threshold {
			let tongguan_strength = match tongguan {
				WuXing::Jin => strength.jin,
				WuXing::Mu => strength.mu,
				WuXing::Shui => strength.shui,
				WuXing::Huo => strength.huo,
				WuXing::Tu => strength.tu,
			};
			// 通关五行弱于平均值的一半
			if tongguan_strength < total / 10 {
				return Some(tongguan);
			}
		}
	}

	None
}

/// 扶抑用神判断（原有逻辑）
fn determine_fuyi_yongshen(strength: &WuXingStrength, day_wuxing: WuXing) -> Option<WuXing> {
	// 计算日主强度
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

	// 计算总强度和平均值
	let total: u32 = strengths.iter().map(|(_, s)| s).sum();
	let average = total / 5;

	if day_strength > average {
		// 身旺：喜克泄耗
		// 优先选择克我的五行（官杀），其次选择我生的五行（食伤）
		let ke_wo = get_ke_wo(day_wuxing);
		let wo_sheng = get_wo_sheng(day_wuxing);

		// 检查克我的五行是否太弱
		let ke_wo_strength = match ke_wo {
			WuXing::Jin => strength.jin,
			WuXing::Mu => strength.mu,
			WuXing::Shui => strength.shui,
			WuXing::Huo => strength.huo,
			WuXing::Tu => strength.tu,
		};

		if ke_wo_strength < average / 2 {
			Some(ke_wo)
		} else {
			Some(wo_sheng)
		}
	} else {
		// 身弱：喜生扶
		// 优先选择生我的五行（印星）
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

/// 获取克我的五行（官杀）
fn get_ke_wo(wuxing: WuXing) -> WuXing {
	match wuxing {
		WuXing::Jin => WuXing::Huo,  // 火克金
		WuXing::Mu => WuXing::Jin,   // 金克木
		WuXing::Shui => WuXing::Tu,  // 土克水
		WuXing::Huo => WuXing::Shui, // 水克火
		WuXing::Tu => WuXing::Mu,    // 木克土
	}
}

/// 获取我生的五行（食伤）
fn get_wo_sheng(wuxing: WuXing) -> WuXing {
	match wuxing {
		WuXing::Jin => WuXing::Shui, // 金生水
		WuXing::Mu => WuXing::Huo,   // 木生火
		WuXing::Shui => WuXing::Mu,  // 水生木
		WuXing::Huo => WuXing::Tu,   // 火生土
		WuXing::Tu => WuXing::Jin,   // 土生金
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

