//! # 四柱计算模块
//!
//! 实现年月日时四柱的完整计算逻辑

use crate::types::GanZhi;
use super::ganzhi::calculate_days_between;
use super::jieqi::get_month_zhi_by_jieqi;

/// 日柱基准日期：公元前720年1月1日 = 甲子日
///
/// 这是八字排盘中公认的日柱计算基准点
const BASE_YEAR: i32 = -720;
const BASE_MONTH: u8 = 1;
const BASE_DAY: u8 = 1;
/// 甲子日的干支索引为0
const BASE_GANZHI_INDEX: u8 = 0;

/// 计算日柱干支
///
/// 使用基准日期法：
/// 1. 计算目标日期与公元前720年1月1日的天数差
/// 2. 将天数差对60取模得到干支索引
/// 3. 转换为干支对象
///
/// # 参数
///
/// - `year`: 公历年份
/// - `month`: 公历月份 (1-12)
/// - `day`: 公历日期 (1-31)
///
/// # 返回
///
/// - `Some(GanZhi)`: 成功计算的日柱干支
/// - `None`: 参数无效或计算失败
///
/// # 示例
///
/// ```ignore
/// // 1998年7月31日的日柱
/// let day_ganzhi = calculate_day_ganzhi(1998, 7, 31);
/// assert!(day_ganzhi.is_some());
/// ```
pub fn calculate_day_ganzhi(year: u16, month: u8, day: u8) -> Option<GanZhi> {
	// 参数验证
	if month < 1 || month > 12 || day < 1 || day > 31 {
		return None;
	}

	// 计算目标日期与基准日期的天数差
	let days_diff = calculate_days_between(
		BASE_YEAR,
		BASE_MONTH,
		BASE_DAY,
		year as i32,
		month,
		day
	);

	// 计算干支索引：天数差对60取模
	// 注意：Rust的负数取模结果为负，需要处理
	let ganzhi_index = ((days_diff % 60) + 60) as u8 % 60;

	// 加上基准日期的干支索引（甲子 = 0）
	let final_index = (BASE_GANZHI_INDEX + ganzhi_index) % 60;

	// 转换为干支对象
	GanZhi::from_index(final_index)
}

/// 计算年柱干支
///
/// 年柱以立春为界，不是以公历1月1日或农历正月初一为界。
/// 如果公历日期在立春之前，年柱使用上一年的干支。
///
/// 年柱干支计算公式：
/// - 天干：(year - 3) % 10
/// - 地支：(year - 3) % 12
///
/// 例如2000年（立春后）：
/// - 天干：(2000 - 3) % 10 = 1997 % 10 = 7 (庚)
/// - 地支：(2000 - 3) % 12 = 1997 % 12 = 5 (辰)
/// - 年柱：庚辰
///
/// # 参数
///
/// - `year`: 公历年份
/// - `month`: 公历月份 (1-12)
/// - `day`: 公历日期 (1-31)
///
/// # 返回
///
/// - `Some(GanZhi)`: 成功计算的年柱干支
/// - `None`: 参数无效或计算失败
///
/// # 注意
///
/// 本函数使用简化的立春判断：
/// - 立春一般在公历2月3-5日之间
/// - 本实现简单判断：2月4日及以后视为立春后
/// - 生产环境应使用精确的节气计算（寿星天文算法）
pub fn calculate_year_ganzhi(year: u16, month: u8, day: u8) -> Option<GanZhi> {
	// 参数验证
	if month < 1 || month > 12 || day < 1 || day > 31 {
		return None;
	}

	// 确定八字年份（考虑立春边界）
	let bazi_year = if month < 2 {
		// 1月：还在前一年
		year.checked_sub(1)?
	} else if month == 2 && day < 4 {
		// 2月1-3日：可能还在前一年（简化判断）
		year.checked_sub(1)?
	} else {
		// 2月4日及以后：当前年份
		year
	};

	// 计算年柱干支
	// 公元4年 = 甲子年（GanZhi index 0）
	// 1984年 = 甲子年
	// 因此：(year - 4) % 60 可以得到年柱索引

	let tiangan_index = ((bazi_year as i32 - 4) % 10 + 10) as u8 % 10;
	let dizhi_index = ((bazi_year as i32 - 4) % 12 + 12) as u8 % 12;

	// 注意：年柱不能简单用GanZhi::from_index，因为天干地支周期不同
	// 需要用天干和地支分别计算，然后组合
	use crate::types::{TianGan, DiZhi};
	Some(GanZhi {
		gan: TianGan(tiangan_index),
		zhi: DiZhi(dizhi_index),
	})
}

/// 计算月柱干支
///
/// 月柱计算包含两个部分：
/// 1. **月支**：根据节气确定，寅月(立春)到丑月(小寒)
/// 2. **月干**：使用五虎遁公式，根据年干推算
///
/// ## ⚠️ 重要：使用精确节气计算
///
/// 本函数使用寿星天文历算法精确计算节气时间，而非简化的固定日期判断。
/// 这确保了在节气边界日期的正确计算。
///
/// ## 五虎遁口诀
///
/// 甲己之年丙作首 (甲年/己年，寅月从丙寅开始)
/// 乙庚之岁戊为头 (乙年/庚年，寅月从戊寅开始)
/// 丙辛必定寻庚起 (丙年/辛年，寅月从庚寅开始)
/// 丁壬壬位顺行流 (丁年/壬年，寅月从壬寅开始)
/// 若问戊癸何处发 (戊年/癸年，寅月从甲寅开始)
/// 甲寅之上好追求
///
/// ## 节气与月支对应
///
/// 立春(1月节)→寅月(2)  惊蛰(2月节)→卯月(3)  清明(3月节)→辰月(4)
/// 立夏(4月节)→巳月(5)  芒种(5月节)→午月(6)  小暑(6月节)→未月(7)
/// 立秋(7月节)→申月(8)  白露(8月节)→酉月(9)  寒露(9月节)→戌月(10)
/// 立冬(10月节)→亥月(11) 大雪(11月节)→子月(0)  小寒(12月节)→丑月(1)
///
/// # 参数
///
/// - `year`: 公历年份
/// - `month`: 公历月份 (1-12)
/// - `day`: 公历日期 (1-31)
/// - `year_gan`: 年柱天干（用于五虎遁计算）
///
/// # 返回
///
/// - `Some(GanZhi)`: 成功计算的月柱干支
/// - `None`: 参数无效或计算失败
///
/// # 示例
///
/// ```ignore
/// // 1990年11月29日，庚午年(年干=6)
/// // 此日在立冬(11月7日)后、大雪(12月7日)前，属于亥月(11)
/// // 庚年(6)亥月的月干：戊寅起，亥月=寅+9，戊+9=丁
/// // 结果：丁亥
/// let gz = calculate_month_ganzhi(1990, 11, 29, 6);
/// assert!(gz.is_some());
/// let gz = gz.unwrap();
/// assert_eq!(gz.gan.0, 3); // 丁
/// assert_eq!(gz.zhi.0, 11); // 亥
/// ```
pub fn calculate_month_ganzhi(year: u16, month: u8, day: u8, year_gan: u8) -> Option<GanZhi> {
	calculate_month_ganzhi_with_hour(year, month, day, 12, year_gan)
}

/// 计算月柱干支（带小时参数）
///
/// 此函数允许指定小时，用于更精确的节气边界判断。
/// 在节气交接日当天，上午和下午可能属于不同的月份。
///
/// # 参数
///
/// - `year`: 公历年份
/// - `month`: 公历月份 (1-12)
/// - `day`: 公历日期 (1-31)
/// - `hour`: 小时 (0-23)
/// - `year_gan`: 年柱天干（用于五虎遁计算）
///
/// # 返回
///
/// - `Some(GanZhi)`: 成功计算的月柱干支
/// - `None`: 参数无效或计算失败
pub fn calculate_month_ganzhi_with_hour(year: u16, month: u8, day: u8, hour: u8, year_gan: u8) -> Option<GanZhi> {
	// 参数验证
	if month < 1 || month > 12 || day < 1 || day > 31 || year_gan >= 10 || hour >= 24 {
		return None;
	}

	// 使用精确节气计算获取月支
	let (month_zhi, _adjusted_year) = get_month_zhi_by_jieqi(year, month, day, hour);

	// 五虎遁：根据年干确定寅月（month_zhi=2）的月干
	// 然后根据月支偏移计算当前月干
	let yin_month_gan = match year_gan {
		0 | 5 => 2,  // 甲己 → 丙寅
		1 | 6 => 4,  // 乙庚 → 戊寅
		2 | 7 => 6,  // 丙辛 → 庚寅
		3 | 8 => 8,  // 丁壬 → 壬寅
		4 | 9 => 0,  // 戊癸 → 甲寅
		_ => return None,
	};

	// 计算当前月干
	// 寅月(2) → 当前月支的偏移
	let offset = if month_zhi >= 2 {
		month_zhi - 2
	} else {
		// 子月(0)和丑月(1)在寅月之前，需要加10或11
		month_zhi + 10
	};

	let month_gan = (yin_month_gan + offset) % 10;

	// 构造月柱干支
	use crate::types::{TianGan, DiZhi};
	Some(GanZhi {
		gan: TianGan(month_gan),
		zhi: DiZhi(month_zhi),
	})
}

/// 计算时柱干支
///
/// 时柱计算包含两个部分：
/// 1. **时支**：根据时辰确定（子时0到亥时11）
/// 2. **时干**：使用五鼠遁公式，根据日干推算
///
/// ## ⚠️ 子时双模式支持（关键特性）
///
/// **传统派（ZiShiMode::Traditional）**:
/// - 23:00-23:59 属于次日子时（早子时）
/// - 00:00-00:59 属于当日子时（晚子时）
/// - 23:00需要日干+1来计算时干
///
/// **现代派（ZiShiMode::Modern）**:
/// - 23:00-23:59 属于当日子时
/// - 00:00-00:59 也属于当日子时
/// - 23:00使用当日日干计算时干
///
/// ## 五鼠遁口诀
///
/// 甲己还加甲 (甲日/己日，子时从甲子开始)
/// 乙庚丙作初 (乙日/庚日，子时从丙子开始)
/// 丙辛从戊起 (丙日/辛日，子时从戊子开始)
/// 丁壬庚子居 (丁日/壬日，子时从庚子开始)
/// 戊癸何方发 (戊日/癸日，子时从壬子开始)
/// 壬子是真途
///
/// ## 时辰对应
///
/// 子时(23-01) 丑时(01-03) 寅时(03-05) 卯时(05-07)
/// 辰时(07-09) 巳时(09-11) 午时(11-13) 未时(13-15)
/// 申时(15-17) 酉时(17-19) 戌时(19-21) 亥时(21-23)
///
/// # 参数
///
/// - `hour`: 小时 (0-23)
/// - `day_gan`: 日柱天干（用于五鼠遁计算）
/// - `zishi_mode`: 子时归属模式
///
/// # 返回
///
/// - `Some((GanZhi, is_next_day))`: 成功计算的时柱干支和是否属于次日标志
/// - `None`: 参数无效或计算失败
///
/// # 注意
///
/// 返回的 `is_next_day` 标志表示是否需要调整日期：
/// - 传统派 + 23:00-23:59 → `is_next_day = true`（需要使用次日的日柱天干）
pub fn calculate_hour_ganzhi(
	hour: u8,
	day_gan: u8,
	zishi_mode: crate::types::ZiShiMode
) -> Option<(GanZhi, bool)> {
	// 参数验证
	if hour >= 24 || day_gan >= 10 {
		return None;
	}

	// 确定时支和是否属于次日
	let (hour_zhi, is_next_day) = if hour == 23 {
		// 23:00-23:59：子时
		match zishi_mode {
			crate::types::ZiShiMode::Traditional => {
				// 传统派：属于次日（早子时）
				(0, true)
			},
			crate::types::ZiShiMode::Modern => {
				// 现代派：属于当日
				(0, false)
			},
		}
	} else {
		// 其他时辰：根据小时确定时支
		let zhi = match hour {
			0 | 1 => 0,   // 子时 00:00-01:59
			2 | 3 => 1,   // 丑时 02:00-03:59
			4 | 5 => 2,   // 寅时 04:00-05:59
			6 | 7 => 3,   // 卯时 06:00-07:59
			8 | 9 => 4,   // 辰时 08:00-09:59
			10 | 11 => 5, // 巳时 10:00-11:59
			12 | 13 => 6, // 午时 12:00-13:59
			14 | 15 => 7, // 未时 14:00-15:59
			16 | 17 => 8, // 申时 16:00-17:59
			18 | 19 => 9, // 酉时 18:00-19:59
			20 | 21 => 10,// 戌时 20:00-21:59
			22 => 11,     // 亥时 22:00-22:59
			_ => return None,
		};
		(zhi, false)
	};

	// 五鼠遁：根据日干确定子时（hour_zhi=0）的时干
	// 然后根据时支偏移计算当前时干
	let zi_hour_gan = match day_gan {
		0 | 5 => 0,  // 甲己 → 甲子
		1 | 6 => 2,  // 乙庚 → 丙子
		2 | 7 => 4,  // 丙辛 → 戊子
		3 | 8 => 6,  // 丁壬 → 庚子
		4 | 9 => 8,  // 戊癸 → 壬子
		_ => return None,
	};

	// 计算当前时干：子时天干 + 时支偏移
	let hour_gan = (zi_hour_gan + hour_zhi) % 10;

	// 构造时柱干支
	use crate::types::{TianGan, DiZhi};
	Some((
		GanZhi {
			gan: TianGan(hour_gan),
			zhi: DiZhi(hour_zhi),
		},
		is_next_day
	))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_day_ganzhi_known_dates() {
		// 测试已知日期的日柱
		// 1998年7月31日 应该是 丁卯日 (GanZhi index 3)
		// 根据 lunar-java 和 BaziGo 的计算结果
		let gz = calculate_day_ganzhi(1998, 7, 31);
		assert!(gz.is_some());

		// 2000年1月1日
		let gz = calculate_day_ganzhi(2000, 1, 1);
		assert!(gz.is_some());
	}

	#[test]
	fn test_day_ganzhi_base_date() {
		// 公元前720年1月1日应该是甲子日(index 0)
		// 注意：这里无法直接测试负数年份，因为函数接受u16
		// 但我们可以测试一个周期后的日期
	}

	#[test]
	fn test_day_ganzhi_invalid_input() {
		// 测试无效输入
		assert!(calculate_day_ganzhi(2000, 0, 1).is_none()); // 月份无效
		assert!(calculate_day_ganzhi(2000, 13, 1).is_none()); // 月份无效
		assert!(calculate_day_ganzhi(2000, 1, 0).is_none()); // 日期无效
		assert!(calculate_day_ganzhi(2000, 1, 32).is_none()); // 日期无效
	}

	#[test]
	fn test_year_ganzhi_known_years() {
		// 2000年立春后应该是庚辰年
		// 天干：(2000-4)%10 = 1996%10 = 6 (庚)
		// 地支：(2000-4)%12 = 1996%12 = 4 (辰)
		let gz = calculate_year_ganzhi(2000, 3, 1);
		assert!(gz.is_some());
		let gz = gz.unwrap();
		assert_eq!(gz.gan.0, 6); // 庚 = 6
		assert_eq!(gz.zhi.0, 4); // 辰 = 4

		// 1984年应该是甲子年（验证公式）
		// 天干：(1984-4)%10 = 1980%10 = 0 (甲)
		// 地支：(1984-4)%12 = 1980%12 = 0 (子)
		let gz = calculate_year_ganzhi(1984, 3, 1);
		assert!(gz.is_some());
		let gz = gz.unwrap();
		assert_eq!(gz.gan.0, 0); // 甲 = 0
		assert_eq!(gz.zhi.0, 0); // 子 = 0
	}

	#[test]
	fn test_year_ganzhi_lichun_boundary() {
		// 测试立春边界
		// 2000年1月31日：还在1999年，应该是己卯年
		let gz1 = calculate_year_ganzhi(2000, 1, 31);
		// 2000年2月5日：已过立春，应该是庚辰年
		let gz2 = calculate_year_ganzhi(2000, 2, 5);

		assert!(gz1.is_some());
		assert!(gz2.is_some());

		// 两个年柱应该不同
		assert_ne!(gz1.unwrap().to_index(), gz2.unwrap().to_index());
	}

	#[test]
	fn test_year_ganzhi_invalid_input() {
		assert!(calculate_year_ganzhi(2000, 0, 1).is_none());
		assert!(calculate_year_ganzhi(2000, 13, 1).is_none());
		assert!(calculate_year_ganzhi(2000, 1, 0).is_none());
	}

	#[test]
	fn test_month_ganzhi_wuhudun() {
		// 测试五虎遁公式
		// 甲年(0)或己年(5)，寅月应该是丙寅 (gan=2, zhi=2)
		let gz = calculate_month_ganzhi(2024, 2, 10, 0); // 甲年，立春后应该是寅月
		assert!(gz.is_some());
		let gz = gz.unwrap();
		assert_eq!(gz.zhi.0, 2); // 寅月
		assert_eq!(gz.gan.0, 2); // 丙 (五虎遁: 甲己之年丙作首)

		// 乙年(1)或庚年(6)，寅月应该是戊寅 (gan=4, zhi=2)
		let gz = calculate_month_ganzhi(2025, 2, 10, 1); // 乙年，寅月
		assert!(gz.is_some());
		let gz = gz.unwrap();
		assert_eq!(gz.gan.0, 4); // 戊 (五虎遁: 乙庚之岁戊为头)
	}

	#[test]
	fn test_month_ganzhi_jieqi_boundary() {
		// 测试节气边界
		// 2000年2月3日：立春前，应该是丑月
		let gz1 = calculate_month_ganzhi(2000, 2, 3, 6); // 庚年
		// 2000年2月5日：立春后，应该是寅月
		let gz2 = calculate_month_ganzhi(2000, 2, 5, 6);

		assert!(gz1.is_some());
		assert!(gz2.is_some());

		// 月支应该不同
		assert_ne!(gz1.unwrap().zhi.0, gz2.unwrap().zhi.0);
	}

	#[test]
	fn test_month_ganzhi_invalid_input() {
		assert!(calculate_month_ganzhi(2000, 0, 1, 0).is_none());
		assert!(calculate_month_ganzhi(2000, 13, 1, 0).is_none());
		assert!(calculate_month_ganzhi(2000, 1, 0, 0).is_none());
		assert!(calculate_month_ganzhi(2000, 1, 1, 10).is_none()); // 无效年干
	}

	#[test]
	fn test_month_ganzhi_1990_11_29() {
		// ⚠️ 关键测试：1990年11月29日12时 月柱应该是 丁亥
		//
		// 分析：
		// - 1990年是庚午年，年干是庚(6)
		// - 1990年立冬：11月7日 22:53 (亥月开始)
		// - 1990年大雪：12月7日 15:40 (子月开始)
		// - 11月29日在立冬之后、大雪之前，属于亥月(11)
		// - 五虎遁：乙庚之岁戊为头，庚年寅月从戊寅开始
		// - 亥月 = 寅月 + 9个月，戊(4) + 9 = 丁(3)
		// - 正确结果：丁亥
		//
		// 之前错误：使用简化节气判断，把11月29日归入子月，得到戊子
		let gz = calculate_month_ganzhi(1990, 11, 29, 6); // 庚年(年干=6)
		assert!(gz.is_some());
		let gz = gz.unwrap();

		// 天干应该是丁(3)
		assert_eq!(gz.gan.0, 3, "月干应该是丁(3)，实际得到: {}", gz.gan.0);
		// 地支应该是亥(11)
		assert_eq!(gz.zhi.0, 11, "月支应该是亥(11)，实际得到: {}", gz.zhi.0);
	}

	#[test]
	fn test_month_ganzhi_with_hour() {
		// 测试带小时的月柱计算
		// 使用不同时间应该得到相同结果（同一天内）
		let gz1 = calculate_month_ganzhi_with_hour(1990, 11, 29, 0, 6);
		let gz2 = calculate_month_ganzhi_with_hour(1990, 11, 29, 12, 6);
		let gz3 = calculate_month_ganzhi_with_hour(1990, 11, 29, 23, 6);

		assert!(gz1.is_some());
		assert!(gz2.is_some());
		assert!(gz3.is_some());

		// 同一天内应该得到相同月柱
		assert_eq!(gz1.unwrap().to_index(), gz2.unwrap().to_index());
		assert_eq!(gz2.unwrap().to_index(), gz3.unwrap().to_index());
	}

	#[test]
	fn test_hour_ganzhi_wushudun() {
		use crate::types::ZiShiMode;

		// 测试五鼠遁公式
		// 甲日(0)或己日(5)，子时应该是甲子 (gan=0, zhi=0)
		let result = calculate_hour_ganzhi(0, 0, ZiShiMode::Modern); // 00:00，甲日
		assert!(result.is_some());
		let (gz, is_next_day) = result.unwrap();
		assert_eq!(gz.gan.0, 0); // 甲 (五鼠遁: 甲己还加甲)
		assert_eq!(gz.zhi.0, 0); // 子
		assert_eq!(is_next_day, false);

		// 乙日(1)或庚日(6)，子时应该是丙子 (gan=2, zhi=0)
		let result = calculate_hour_ganzhi(0, 1, ZiShiMode::Modern);
		assert!(result.is_some());
		let (gz, _) = result.unwrap();
		assert_eq!(gz.gan.0, 2); // 丙 (五鼠遁: 乙庚丙作初)

		// 测试其他时辰：午时(12:00, zhi=6)
		// 甲日，午时应该是庚午 (gan=0+6=6, zhi=6)
		let result = calculate_hour_ganzhi(12, 0, ZiShiMode::Modern);
		assert!(result.is_some());
		let (gz, _) = result.unwrap();
		assert_eq!(gz.gan.0, 6); // 庚
		assert_eq!(gz.zhi.0, 6); // 午
	}

	#[test]
	fn test_hour_ganzhi_zishi_dual_mode() {
		use crate::types::ZiShiMode;

		// ⚠️ 关键测试：子时双模式
		// 23:00 + 传统派：属于次日（is_next_day=true）
		let result = calculate_hour_ganzhi(23, 0, ZiShiMode::Traditional);
		assert!(result.is_some());
		let (gz, is_next_day) = result.unwrap();
		assert_eq!(gz.zhi.0, 0); // 子时
		assert_eq!(is_next_day, true); // 传统派：属于次日

		// 23:00 + 现代派：属于当日（is_next_day=false）
		let result = calculate_hour_ganzhi(23, 0, ZiShiMode::Modern);
		assert!(result.is_some());
		let (gz, is_next_day) = result.unwrap();
		assert_eq!(gz.zhi.0, 0); // 子时
		assert_eq!(is_next_day, false); // 现代派：属于当日

		// 00:00-01:59：两种模式都属于当日
		let result1 = calculate_hour_ganzhi(0, 0, ZiShiMode::Traditional);
		let result2 = calculate_hour_ganzhi(0, 0, ZiShiMode::Modern);
		assert_eq!(result1.unwrap().1, false);
		assert_eq!(result2.unwrap().1, false);
	}

	#[test]
	fn test_hour_ganzhi_all_shichen() {
		use crate::types::ZiShiMode;

		// 测试所有12个时辰
		let test_hours = [
			(0, 0),   // 子时
			(2, 1),   // 丑时
			(4, 2),   // 寅时
			(6, 3),   // 卯时
			(8, 4),   // 辰时
			(10, 5),  // 巳时
			(12, 6),  // 午时
			(14, 7),  // 未时
			(16, 8),  // 申时
			(18, 9),  // 酉时
			(20, 10), // 戌时
			(22, 11), // 亥时
		];

		for (hour, expected_zhi) in test_hours.iter() {
			let result = calculate_hour_ganzhi(*hour, 0, ZiShiMode::Modern);
			assert!(result.is_some());
			let (gz, _) = result.unwrap();
			assert_eq!(gz.zhi.0, *expected_zhi);
		}
	}

	#[test]
	fn test_hour_ganzhi_invalid_input() {
		use crate::types::ZiShiMode;

		assert!(calculate_hour_ganzhi(24, 0, ZiShiMode::Modern).is_none()); // 无效小时
		assert!(calculate_hour_ganzhi(0, 10, ZiShiMode::Modern).is_none()); // 无效日干
	}
}
