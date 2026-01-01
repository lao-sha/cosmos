//! # 干支计算模块
//!
//! 提供干支相关的基础计算功能

/// 计算两个日期之间的天数差
///
/// 使用蔡勒公式(Zeller's Formula)的变种来计算
///
/// # 参数
///
/// - `from_year`: 起始年份
/// - `from_month`: 起始月份 (1-12)
/// - `from_day`: 起始日期 (1-31)
/// - `to_year`: 目标年份
/// - `to_month`: 目标月份 (1-12)
/// - `to_day`: 目标日期 (1-31)
///
/// # 返回
///
/// 两个日期之间的天数差（可能为负）
pub fn calculate_days_between(
	from_year: i32,
	from_month: u8,
	from_day: u8,
	to_year: i32,
	to_month: u8,
	to_day: u8
) -> i32 {
	// 将日期转换为儒略日数(Julian Day Number)，然后相减
	let from_jdn = date_to_julian_day(from_year, from_month, from_day);
	let to_jdn = date_to_julian_day(to_year, to_month, to_day);
	to_jdn - from_jdn
}

/// 将公历日期转换为儒略日数
///
/// 使用儒略日数算法，适用于所有日期包括负数年份
///
/// # 参数
///
/// - `year`: 年份（可以为负，如 -720 表示公元前720年）
/// - `month`: 月份 (1-12)
/// - `day`: 日期 (1-31)
///
/// # 返回
///
/// 儒略日数
fn date_to_julian_day(year: i32, month: u8, day: u8) -> i32 {
	let mut y = year;
	let mut m = month as i32;

	// 将1月和2月视为前一年的13月和14月
	if m <= 2 {
		y -= 1;
		m += 12;
	}

	// 格里高利历修正（1582年10月15日之后）
	let a = y / 100;
	let b = if year > 1582 || (year == 1582 && month > 10) || (year == 1582 && month == 10 && day >= 15) {
		2 - a + a / 4
	} else {
		0
	};

	// 儒略日数计算公式
	let jdn = (365.25 * (y + 4716) as f64) as i32 +
	          (30.6001 * (m + 1) as f64) as i32 +
	          day as i32 +
	          b - 1524;

	jdn
}

/// 判断是否为闰年
pub fn is_leap_year(year: u16) -> bool {
	(year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// 获取指定月份的天数
pub fn get_days_in_month(year: u16, month: u8) -> u8 {
	match month {
		2 => if is_leap_year(year) { 29 } else { 28 },
		4 | 6 | 9 | 11 => 30,
		_ => 31,
	}
}
