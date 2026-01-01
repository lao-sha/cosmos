//! # 大运计算模块
//!
//! 实现起运年龄和大运序列的计算

use sp_std::vec::Vec;

use crate::types::{Gender, GanZhi};

/// 起运信息（精确版）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QiYunInfo {
	/// 起运年龄（整数部分）
	pub age_years: u8,
	/// 起运月数（0-11）
	pub age_months: u8,
	/// 起运天数（0-29）
	pub age_days: u8,
	/// 是否顺排
	pub is_shun: bool,
}

/// 计算起运年龄（精确版）
///
/// 起运年龄决定了一个人从几岁开始进入第一步大运。
///
/// ## 计算规则
///
/// 1. **阳男阴女顺排**：出生后顺数到下一个节气的天数
/// 2. **阴男阳女逆排**：出生后逆数到上一个节气的天数
/// 3. **精确换算**：
///    - 3天 = 1年
///    - 1天 = 4个月
///    - 1时辰(2小时) = 10天
///
/// ## 阴阳判断
///
/// - 阳年：甲(0)、丙(2)、戊(4)、庚(6)、壬(8)
/// - 阴年：乙(1)、丁(3)、己(5)、辛(7)、癸(9)
///
/// ## 参数
///
/// - `year_gan`: 年柱天干
/// - `gender`: 性别
/// - `days_to_jieqi`: 距离节气的天数
/// - `hours_to_jieqi`: 距离节气的小时数（0-23）
///
/// ## 返回
///
/// - `QiYunInfo`: 精确的起运信息
pub fn calculate_qiyun_precise(
	year_gan: u8,
	gender: Gender,
	days_to_jieqi: u16,
	hours_to_jieqi: u8,
) -> QiYunInfo {
	// 判断年干阴阳
	let is_yang_year = year_gan % 2 == 0;

	// 判断是否顺排
	// 阳男阴女顺排，阴男阳女逆排
	let is_shun = match (is_yang_year, gender) {
		(true, Gender::Male) => true,   // 阳男顺排
		(false, Gender::Female) => true, // 阴女顺排
		_ => false,                      // 阴男阳女逆排
	};

	// 精确计算起运时间
	// 总小时数
	let total_hours = days_to_jieqi as u32 * 24 + hours_to_jieqi as u32;

	// 换算规则：3天 = 1年，即 72小时 = 1年
	// 1小时 = 1/72年 = 12/72月 = 1/6月 ≈ 5天
	let age_years = (total_hours / 72) as u8;
	let remaining_hours = total_hours % 72;

	// 剩余小时换算成月：1小时 = 1/6月
	// 6小时 = 1月
	let age_months = (remaining_hours / 6) as u8;
	let remaining_hours_2 = remaining_hours % 6;

	// 剩余小时换算成天：1小时 ≈ 5天
	let age_days = (remaining_hours_2 * 5) as u8;

	QiYunInfo {
		age_years,
		age_months,
		age_days,
		is_shun,
	}
}

/// 计算起运年龄（简化版，向后兼容）
///
/// ## 参数
///
/// - `year_gan`: 年柱天干
/// - `gender`: 性别
/// - `days_to_next_jieqi`: 距离下一个节气的天数
///
/// ## 返回
///
/// - `(qiyun_age, is_shun)`: 起运年龄和是否顺排
pub fn calculate_qiyun_age(
	year_gan: u8,
	gender: Gender,
	days_to_next_jieqi: u8,
) -> (u8, bool) {
	let info = calculate_qiyun_precise(year_gan, gender, days_to_next_jieqi as u16, 0);
	(info.age_years, info.is_shun)
}

/// 计算大运序列
///
/// 根据月柱和起运信息生成10-12步大运。
///
/// ## 大运规则
///
/// - 顺排：从月柱往后推（甲子→乙丑→丙寅...）
/// - 逆排：从月柱往前推（甲子→癸亥→壬戌...）
/// - 每步大运10年
///
/// ## 参数
///
/// - `month_ganzhi`: 月柱干支
/// - `birth_year`: 出生年份
/// - `qiyun_age`: 起运年龄
/// - `is_shun`: 是否顺排
/// - `max_steps`: 最多生成多少步大运（默认12步，120年）
///
/// ## 返回
///
/// - `Vec<DaYunStep>`: 大运序列
pub fn calculate_dayun_list(
	month_ganzhi: GanZhi,
	birth_year: u16,
	qiyun_age: u8,
	is_shun: bool,
	max_steps: u8,
) -> Vec<(GanZhi, u8, u16)> {
	let mut result = Vec::new();
	let mut current_ganzhi = month_ganzhi;

	// 第一步大运的起始年龄和年份
	let mut current_age = qiyun_age;
	let mut current_year = birth_year + qiyun_age as u16;

	for step in 0..max_steps {
		// 如果是第一步，使用月柱的下一个/上一个干支
		if step == 0 {
			current_ganzhi = if is_shun {
				current_ganzhi.next()
			} else {
				current_ganzhi.prev()
			};
		} else {
			// 后续步骤继续往后/往前推
			current_ganzhi = if is_shun {
				current_ganzhi.next()
			} else {
				current_ganzhi.prev()
			};
		}

		// 计算本步大运的结束年龄和年份
		let end_age = current_age + 10;
		let end_year = current_year + 10;

		// 存储: (干支, 起始年龄, 起始年份)
		result.push((current_ganzhi, current_age, current_year));

		// 更新下一步的起始值
		current_age = end_age;
		current_year = end_year;
	}

	result
}

/// 计算大运中的十神关系
///
/// 根据日主天干计算大运天干的十神关系
///
/// ## 参数
///
/// - `day_gan`: 日主天干
/// - `dayun_gan`: 大运天干
///
/// ## 返回
///
/// - 十神索引 (0-9)
pub fn calculate_dayun_shishen(day_gan: u8, dayun_gan: u8) -> u8 {
	// 使用十神查表
	// 这里返回索引，实际使用时需要转换为ShiShen枚举
	use crate::constants::SHISHEN_TABLE;
	SHISHEN_TABLE[day_gan as usize][dayun_gan as usize]
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_qiyun_age_calculation() {
		// 阳男顺排：甲年(0) + 男性
		// 假设距离下一个节气6天，起运年龄应该是 6/3 = 2岁
		let (age, is_shun) = calculate_qiyun_age(0, Gender::Male, 6);
		assert_eq!(age, 2);
		assert_eq!(is_shun, true); // 阳男顺排

		// 阴女顺排：乙年(1) + 女性
		let (age, is_shun) = calculate_qiyun_age(1, Gender::Female, 9);
		assert_eq!(age, 3);
		assert_eq!(is_shun, true); // 阴女顺排

		// 阴男逆排：乙年(1) + 男性
		let (age, is_shun) = calculate_qiyun_age(1, Gender::Male, 12);
		assert_eq!(age, 4);
		assert_eq!(is_shun, false); // 阴男逆排

		// 阳女逆排：甲年(0) + 女性
		let (age, is_shun) = calculate_qiyun_age(0, Gender::Female, 15);
		assert_eq!(age, 5);
		assert_eq!(is_shun, false); // 阳女逆排
	}

	#[test]
	fn test_dayun_list_shun() {
		// 测试顺排大运
		// 从甲子月开始，顺排应该是：乙丑、丙寅、丁卯...
		let month_ganzhi = GanZhi::from_index(0).unwrap(); // 甲子
		let dayun_list = calculate_dayun_list(month_ganzhi, 2000, 2, true, 5);

		assert_eq!(dayun_list.len(), 5);

		// 第一步大运：乙丑(1)，2岁起，2002年起
		assert_eq!(dayun_list[0].0.to_index(), 1); // 乙丑
		assert_eq!(dayun_list[0].1, 2); // 2岁
		assert_eq!(dayun_list[0].2, 2002); // 2002年

		// 第二步大运：丙寅(2)，12岁起，2012年起
		assert_eq!(dayun_list[1].0.to_index(), 2); // 丙寅
		assert_eq!(dayun_list[1].1, 12);
		assert_eq!(dayun_list[1].2, 2012);
	}

	#[test]
	fn test_dayun_list_ni() {
		// 测试逆排大运
		// 从甲子月开始，逆排应该是：癸亥、壬戌、辛酉...
		let month_ganzhi = GanZhi::from_index(0).unwrap(); // 甲子
		let dayun_list = calculate_dayun_list(month_ganzhi, 2000, 3, false, 5);

		assert_eq!(dayun_list.len(), 5);

		// 第一步大运：癸亥(59)，3岁起，2003年起
		assert_eq!(dayun_list[0].0.to_index(), 59); // 癸亥
		assert_eq!(dayun_list[0].1, 3);
		assert_eq!(dayun_list[0].2, 2003);

		// 第二步大运：壬戌(58)，13岁起，2013年起
		assert_eq!(dayun_list[1].0.to_index(), 58); // 壬戌
		assert_eq!(dayun_list[1].1, 13);
		assert_eq!(dayun_list[1].2, 2013);
	}

	#[test]
	fn test_dayun_shishen() {
		// 测试大运十神计算
		// 甲日主(0)，甲大运(0) → 比肩(0)
		let shishen = calculate_dayun_shishen(0, 0);
		assert_eq!(shishen, 0); // 比肩

		// 甲日主(0)，乙大运(1) → 劫财(1)
		let shishen = calculate_dayun_shishen(0, 1);
		assert_eq!(shishen, 1); // 劫财
	}
}

