//! # 空亡计算模块
//!
//! 本模块实现八字命理中的空亡（旬空）计算，包括：
//! - 六十甲子旬空查询
//! - 四柱是否落空亡判断
//! - 空亡信息结构构建
//!
//! ## 旬空规则
//!
//! 六十甲子每十个为一旬，每旬有两个地支空缺：
//! - 甲子旬（甲子到癸酉）: 戌亥空
//! - 甲戌旬（甲戌到癸未）: 申酉空
//! - 甲申旬（甲申到癸巳）: 午未空
//! - 甲午旬（甲午到癸卯）: 辰巳空
//! - 甲辰旬（甲辰到癸丑）: 寅卯空
//! - 甲寅旬（甲寅到癸亥）: 子丑空

use crate::types::{GanZhi, DiZhi, KongWangInfo, SiZhu};

/// 计算干支的旬空地支
///
/// # 参数
///
/// - `ganzhi`: 干支组合
///
/// # 返回
///
/// 该干支所属旬的两个空亡地支
///
/// # 原理
///
/// 六十甲子分为六旬，每旬10个干支，由于地支12个，天干10个，
/// 每旬必然有2个地支空缺，这就是旬空（空亡）。
///
/// # 示例
///
/// ```ignore
/// let ganzhi = GanZhi::from_index(0).unwrap(); // 甲子
/// let (kong1, kong2) = calculate_kongwang(&ganzhi);
/// assert_eq!(kong1.0, 10); // 戌
/// assert_eq!(kong2.0, 11); // 亥
/// ```
pub fn calculate_kongwang(ganzhi: &GanZhi) -> (DiZhi, DiZhi) {
	let index = ganzhi.to_index(); // 0-59
	let xun = index / 10; // 确定哪一旬（0-5）

	match xun {
		0 => (DiZhi(10), DiZhi(11)), // 甲子旬: 戌、亥空
		1 => (DiZhi(8), DiZhi(9)),   // 甲戌旬: 申、酉空
		2 => (DiZhi(6), DiZhi(7)),   // 甲申旬: 午、未空
		3 => (DiZhi(4), DiZhi(5)),   // 甲午旬: 辰、巳空
		4 => (DiZhi(2), DiZhi(3)),   // 甲辰旬: 寅、卯空
		5 => (DiZhi(0), DiZhi(1)),   // 甲寅旬: 子、丑空
		_ => unreachable!("旬空索引超出范围"),
	}
}

/// 检查地支是否落空亡
///
/// # 参数
///
/// - `dizhi`: 要检查的地支
/// - `kongwang`: 旬空的两个地支
///
/// # 返回
///
/// - `true`: 该地支落空亡
/// - `false`: 该地支不落空亡
///
/// # 示例
///
/// ```ignore
/// let dizhi = DiZhi(10); // 戌
/// let kongwang = (DiZhi(10), DiZhi(11)); // 戌亥空
/// assert!(is_kong(dizhi, kongwang)); // 戌落空亡
/// ```
pub fn is_kong(dizhi: DiZhi, kongwang: (DiZhi, DiZhi)) -> bool {
	dizhi == kongwang.0 || dizhi == kongwang.1
}

/// 计算四柱的完整空亡信息
///
/// # 参数
///
/// - `sizhu`: 四柱信息
///
/// # 返回
///
/// `KongWangInfo` 结构，包含：
/// - 四柱各自的旬空地支
/// - 四柱地支是否落空亡的判断
///
/// # 说明
///
/// - 日柱空亡最重要，影响命主本身
/// - 年柱空亡影响祖辈和早年
/// - 月柱空亡影响父母和青年
/// - 时柱空亡影响子女和晚年
///
/// # 示例
///
/// ```ignore
/// let sizhu = SiZhu { ... };
/// let kongwang_info = calculate_all_kongwang(&sizhu);
///
/// if kongwang_info.day_is_kong {
///     println!("日柱落空亡，命主一生多波折");
/// }
/// ```
pub fn calculate_all_kongwang<T: crate::pallet::Config>(sizhu: &SiZhu<T>) -> KongWangInfo {
	// 计算各柱所在旬的空亡（用于显示）
	let year_kongwang = calculate_kongwang(&sizhu.year_zhu.ganzhi);
	let month_kongwang = calculate_kongwang(&sizhu.month_zhu.ganzhi);
	let day_kongwang = calculate_kongwang(&sizhu.day_zhu.ganzhi);
	let hour_kongwang = calculate_kongwang(&sizhu.hour_zhu.ganzhi);

	// 判断是否落空亡：以日柱空亡为准，检查各柱地支是否在日柱空亡中
	// 注意：日支永远不可能在自己的空亡中（地支总在自己所属的旬内）
	let year_is_kong = is_kong(sizhu.year_zhu.ganzhi.zhi, day_kongwang);
	let month_is_kong = is_kong(sizhu.month_zhu.ganzhi.zhi, day_kongwang);
	let day_is_kong = false; // 日支不可能落入日柱空亡
	let hour_is_kong = is_kong(sizhu.hour_zhu.ganzhi.zhi, day_kongwang);

	KongWangInfo {
		year_kongwang,
		month_kongwang,
		day_kongwang,
		hour_kongwang,
		year_is_kong,
		month_is_kong,
		day_is_kong,
		hour_is_kong,
	}
}

/// 计算临时四柱的空亡信息（不使用泛型）
///
/// # 参数
///
/// - `year_ganzhi`: 年柱干支
/// - `month_ganzhi`: 月柱干支
/// - `day_ganzhi`: 日柱干支
/// - `hour_ganzhi`: 时柱干支
///
/// # 返回
///
/// `KongWangInfo` 结构，包含空亡信息
///
/// # 用途
///
/// 供临时排盘接口使用，避免泛型依赖
pub fn calculate_all_kongwang_temp(
	year_ganzhi: &GanZhi,
	month_ganzhi: &GanZhi,
	day_ganzhi: &GanZhi,
	hour_ganzhi: &GanZhi,
) -> KongWangInfo {
	// 计算各柱所在旬的空亡（用于显示）
	let year_kongwang = calculate_kongwang(year_ganzhi);
	let month_kongwang = calculate_kongwang(month_ganzhi);
	let day_kongwang = calculate_kongwang(day_ganzhi);
	let hour_kongwang = calculate_kongwang(hour_ganzhi);

	// 判断是否落空亡：以日柱空亡为准，检查各柱地支是否在日柱空亡中
	// 注意：日支永远不可能在自己的空亡中（地支总在自己所属的旬内）
	let year_is_kong = is_kong(year_ganzhi.zhi, day_kongwang);
	let month_is_kong = is_kong(month_ganzhi.zhi, day_kongwang);
	let day_is_kong = false; // 日支不可能落入日柱空亡
	let hour_is_kong = is_kong(hour_ganzhi.zhi, day_kongwang);

	KongWangInfo {
		year_kongwang,
		month_kongwang,
		day_kongwang,
		hour_kongwang,
		year_is_kong,
		month_is_kong,
		day_is_kong,
		hour_is_kong,
	}
}

// ================================
// 单元测试
// ================================

#[cfg(test)]
mod tests {
	use super::*;
	use crate::types::{DiZhi, GanZhi};

	#[test]
	fn test_calculate_kongwang_jiazi_xun() {
		// 甲子旬: 戌亥空
		let ganzhi = GanZhi::from_index(0).unwrap(); // 甲子
		let (kong1, kong2) = calculate_kongwang(&ganzhi);
		assert_eq!(kong1, DiZhi(10)); // 戌
		assert_eq!(kong2, DiZhi(11)); // 亥
	}

	#[test]
	fn test_calculate_kongwang_jiaxu_xun() {
		// 甲戌旬: 申酉空
		let ganzhi = GanZhi::from_index(10).unwrap(); // 甲戌
		let (kong1, kong2) = calculate_kongwang(&ganzhi);
		assert_eq!(kong1, DiZhi(8)); // 申
		assert_eq!(kong2, DiZhi(9)); // 酉
	}

	#[test]
	fn test_calculate_kongwang_jiashen_xun() {
		// 甲申旬: 午未空
		let ganzhi = GanZhi::from_index(20).unwrap(); // 甲申
		let (kong1, kong2) = calculate_kongwang(&ganzhi);
		assert_eq!(kong1, DiZhi(6)); // 午
		assert_eq!(kong2, DiZhi(7)); // 未
	}

	#[test]
	fn test_calculate_kongwang_jiawu_xun() {
		// 甲午旬: 辰巳空
		let ganzhi = GanZhi::from_index(30).unwrap(); // 甲午
		let (kong1, kong2) = calculate_kongwang(&ganzhi);
		assert_eq!(kong1, DiZhi(4)); // 辰
		assert_eq!(kong2, DiZhi(5)); // 巳
	}

	#[test]
	fn test_calculate_kongwang_jiachen_xun() {
		// 甲辰旬: 寅卯空
		let ganzhi = GanZhi::from_index(40).unwrap(); // 甲辰
		let (kong1, kong2) = calculate_kongwang(&ganzhi);
		assert_eq!(kong1, DiZhi(2)); // 寅
		assert_eq!(kong2, DiZhi(3)); // 卯
	}

	#[test]
	fn test_calculate_kongwang_jiayin_xun() {
		// 甲寅旬: 子丑空
		let ganzhi = GanZhi::from_index(50).unwrap(); // 甲寅
		let (kong1, kong2) = calculate_kongwang(&ganzhi);
		assert_eq!(kong1, DiZhi(0)); // 子
		assert_eq!(kong2, DiZhi(1)); // 丑
	}

	#[test]
	fn test_is_kong_true() {
		let dizhi = DiZhi(10); // 戌
		let kongwang = (DiZhi(10), DiZhi(11)); // 戌亥空
		assert!(is_kong(dizhi, kongwang));
	}

	#[test]
	fn test_is_kong_false() {
		let dizhi = DiZhi(0); // 子
		let kongwang = (DiZhi(10), DiZhi(11)); // 戌亥空
		assert!(!is_kong(dizhi, kongwang));
	}

	#[test]
	fn test_all_xun_coverage() {
		// 测试所有六旬的空亡计算
		let expected = [
			(0, (10, 11)),  // 甲子旬: 戌亥空
			(10, (8, 9)),   // 甲戌旬: 申酉空
			(20, (6, 7)),   // 甲申旬: 午未空
			(30, (4, 5)),   // 甲午旬: 辰巳空
			(40, (2, 3)),   // 甲辰旬: 寅卯空
			(50, (0, 1)),   // 甲寅旬: 子丑空
		];

		for (index, (kong1_val, kong2_val)) in expected.iter() {
			let ganzhi = GanZhi::from_index(*index).unwrap();
			let (kong1, kong2) = calculate_kongwang(&ganzhi);
			assert_eq!(kong1, DiZhi(*kong1_val));
			assert_eq!(kong2, DiZhi(*kong2_val));
		}
	}

	#[test]
	fn test_kongwang_uses_day_pillar() {
		// 验证空亡判断以日柱为准
		// 例：日柱甲子（甲子旬，空亡=戌亥），年柱丙戌（甲申旬）
		// 年支戌应该落入日柱空亡，而非年柱自己的空亡（午未）

		let day_ganzhi = GanZhi::from_index(0).unwrap();    // 甲子，甲子旬，空亡=戌亥
		let year_ganzhi = GanZhi::from_index(22).unwrap();  // 丙戌，甲申旬，空亡=午未

		let day_kongwang = calculate_kongwang(&day_ganzhi);
		let year_kongwang = calculate_kongwang(&year_ganzhi);

		// 日柱空亡应该是戌(10)、亥(11)
		assert_eq!(day_kongwang, (DiZhi(10), DiZhi(11)));
		// 年柱所在旬空亡应该是午(6)、未(7)
		assert_eq!(year_kongwang, (DiZhi(6), DiZhi(7)));

		// 年支是戌(10)
		assert_eq!(year_ganzhi.zhi, DiZhi(10));

		// 正确：年支戌落入日柱空亡（戌亥空）
		assert!(is_kong(year_ganzhi.zhi, day_kongwang));

		// 错误逻辑的结果：年支戌不在年柱自己的空亡（午未空）中
		assert!(!is_kong(year_ganzhi.zhi, year_kongwang));
	}

	#[test]
	fn test_calculate_all_kongwang_temp_correct_logic() {
		// 测试 calculate_all_kongwang_temp 使用正确的日柱空亡逻辑
		// 日柱：甲子（甲子旬，空亡=戌亥）
		// 年柱：丙戌（年支=戌，应落空亡）
		// 月柱：丁亥（月支=亥，应落空亡）
		// 时柱：庚寅（时支=寅，不落空亡）

		let year = GanZhi::from_index(22).unwrap();  // 丙戌
		let month = GanZhi::from_index(23).unwrap(); // 丁亥
		let day = GanZhi::from_index(0).unwrap();    // 甲子
		let hour = GanZhi::from_index(26).unwrap();  // 庚寅

		let info = calculate_all_kongwang_temp(&year, &month, &day, &hour);

		// 日柱空亡是戌亥
		assert_eq!(info.day_kongwang, (DiZhi(10), DiZhi(11)));

		// 年支戌落空亡
		assert!(info.year_is_kong);
		// 月支亥落空亡
		assert!(info.month_is_kong);
		// 日支永不落空亡
		assert!(!info.day_is_kong);
		// 时支寅不落空亡
		assert!(!info.hour_is_kong);
	}
}
