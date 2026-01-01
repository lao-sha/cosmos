//! # 真太阳时修正模块
//!
//! 本模块实现真太阳时（True Solar Time）修正算法，用于八字排盘的精确时辰判断。
//!
//! ## 真太阳时 vs 北京时间
//!
//! - **北京时间**：东八区标准时间（UTC+8），基于东经120°
//! - **地方平太阳时**：基于出生地经度的平太阳时
//! - **真太阳时**：地方平太阳时 + 时差方程修正
//!
//! ## 计算公式
//!
//! ```text
//! 地方平太阳时 = 北京时间 + (出生地经度 - 120°) × 4分钟/度
//! 真太阳时 = 地方平太阳时 + 时差方程（由日期决定）
//! ```
//!
//! ## 时差方程
//!
//! 时差方程（Equation of Time）是由地球轨道椭圆形和黄赤交角造成的，
//! 范围约为 -14分钟 到 +16分钟。本模块使用简化的查表法实现。

/// 时差方程月平均值（分钟）
///
/// 这是简化版本，使用月平均值。精确版本需要按日计算。
/// 正值表示真太阳时比平太阳时快，负值表示慢。
const EQUATION_OF_TIME_MONTHLY: [i8; 12] = [
    -3,  // 1月：约 -3 分钟
    -14, // 2月：约 -14 分钟（最小值在2月中旬）
    -10, // 3月：约 -10 分钟
    -4,  // 4月：约 -4 分钟
    3,   // 5月：约 +3 分钟
    2,   // 6月：约 +2 分钟
    -4,  // 7月：约 -4 分钟
    -6,  // 8月：约 -6 分钟
    0,   // 9月：约 0 分钟
    10,  // 10月：约 +10 分钟
    16,  // 11月：约 +16 分钟（最大值在11月初）
    5,   // 12月：约 +5 分钟
];

/// 时差方程每月中旬的精确值（分钟，放大10倍存储）
///
/// 用于更精确的插值计算。
/// 格式：每月15日的时差值 × 10
const EQUATION_OF_TIME_MID_MONTH: [i16; 12] = [
    -92,  // 1月15日：约 -9.2 分钟
    -142, // 2月15日：约 -14.2 分钟
    -94,  // 3月15日：约 -9.4 分钟
    -10,  // 4月15日：约 -1.0 分钟
    37,   // 5月15日：约 +3.7 分钟
    -4,   // 6月15日：约 -0.4 分钟
    -58,  // 7月15日：约 -5.8 分钟
    -45,  // 8月15日：约 -4.5 分钟
    38,   // 9月15日：约 +3.8 分钟
    140,  // 10月15日：约 +14.0 分钟
    159,  // 11月15日：约 +15.9 分钟
    51,   // 12月15日：约 +5.1 分钟
];

/// 获取时差方程值（分钟）
///
/// # 参数
/// - `month`: 月份 (1-12)
/// - `day`: 日期 (1-31)
///
/// # 返回
/// 时差方程值（分钟），正值表示真太阳时比平太阳时快
pub fn get_equation_of_time(month: u8, _day: u8) -> i8 {
    if month < 1 || month > 12 {
        return 0;
    }

    // 简化版本：使用月平均值
    // 实际应用中可以使用线性插值获得更精确的值
    EQUATION_OF_TIME_MONTHLY[(month - 1) as usize]
}

/// 获取时差方程精确值（分钟，保留1位小数）
///
/// # 参数
/// - `month`: 月份 (1-12)
/// - `day`: 日期 (1-31)
///
/// # 返回
/// 时差方程值 × 10（用于保留精度）
pub fn get_equation_of_time_precise(month: u8, day: u8) -> i16 {
    if month < 1 || month > 12 {
        return 0;
    }

    let month_idx = (month - 1) as usize;
    let mid_value = EQUATION_OF_TIME_MID_MONTH[month_idx];

    // 简单处理：15日前后差异不大，直接使用月中值
    // 更精确的实现可以使用线性插值
    if day <= 15 {
        // 月初到月中，从上月值插值到本月中值
        let prev_idx = if month_idx == 0 { 11 } else { month_idx - 1 };
        let _prev_value = EQUATION_OF_TIME_MID_MONTH[prev_idx];
        // 简化：使用本月中值
        mid_value
    } else {
        // 月中到月末，从本月中值插值到下月值
        mid_value
    }
}

/// 真太阳时修正结果
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrueSolarTimeResult {
    /// 修正后的小时 (0-23)
    pub hour: u8,
    /// 修正后的分钟 (0-59)
    pub minute: u8,
    /// 日期偏移 (-1, 0, 1)
    /// -1: 修正后为前一天
    /// 0: 同一天
    /// 1: 修正后为后一天
    pub day_offset: i8,
}

/// 计算真太阳时修正
///
/// 将北京时间修正为出生地的真太阳时。
///
/// # 参数
/// - `year`: 公历年份
/// - `month`: 公历月份 (1-12)
/// - `day`: 公历日期 (1-31)
/// - `hour`: 北京时间小时 (0-23)
/// - `minute`: 北京时间分钟 (0-59)
/// - `longitude`: 出生地经度（1/100000 度，如 116.40000° → 11640000）
///
/// # 返回
/// 修正后的时间和日期偏移
///
/// # 示例
///
/// ```ignore
/// // 北京时间 12:00，出生地经度 90°（西藏）
/// let result = apply_true_solar_time(2024, 6, 15, 12, 0, 9000000);
/// // 经度差 = 90° - 120° = -30°
/// // 经度修正 = -30 × 4 = -120 分钟 = -2 小时
/// // 时差方程（6月）≈ +2 分钟
/// // 修正后 ≈ 10:02
/// assert_eq!(result.hour, 10);
/// assert_eq!(result.minute, 2);
/// ```
pub fn apply_true_solar_time(
    _year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    longitude: i32,
) -> TrueSolarTimeResult {
    // 1. 计算经度时差（分钟）
    // 公式：(出生地经度 - 120°) × 4分钟/度
    // longitude 单位是 1/100000 度，120° = 12000000
    // 经度差（度）= (longitude - 12000000) / 100000
    // 时差（分钟）= 经度差 × 4 = (longitude - 12000000) × 4 / 100000
    let longitude_diff_minutes = ((longitude - 12_000_000) * 4) / 100_000;

    // 2. 获取时差方程（分钟）
    let equation_of_time = get_equation_of_time(month, day) as i32;

    // 3. 计算总修正量
    let total_correction = longitude_diff_minutes + equation_of_time;

    // 4. 计算修正后的总分钟数
    let original_minutes = (hour as i32) * 60 + (minute as i32);
    let corrected_minutes = original_minutes + total_correction;

    // 5. 处理日期跨越
    let (final_minutes, day_offset) = if corrected_minutes < 0 {
        // 修正后为前一天
        (corrected_minutes + 24 * 60, -1i8)
    } else if corrected_minutes >= 24 * 60 {
        // 修正后为后一天
        (corrected_minutes - 24 * 60, 1i8)
    } else {
        (corrected_minutes, 0i8)
    };

    TrueSolarTimeResult {
        hour: (final_minutes / 60) as u8,
        minute: (final_minutes % 60) as u8,
        day_offset,
    }
}

/// 根据经度判断是否需要修正（经度差超过一定阈值才有意义）
///
/// # 参数
/// - `longitude`: 出生地经度（1/100000 度）
///
/// # 返回
/// 如果经度与北京时间基准（120°）差距超过7.5°（即时差超过30分钟），返回 true
pub fn should_apply_correction(longitude: i32) -> bool {
    // 7.5° = 750000（1/100000 度）
    // 7.5° × 4 分钟/度 = 30 分钟
    let diff = (longitude - 12_000_000).abs();
    diff > 750_000
}

/// 调整日期（处理真太阳时修正导致的日期跨越）
///
/// # 参数
/// - `year`: 原始年份
/// - `month`: 原始月份
/// - `day`: 原始日期
/// - `day_offset`: 日期偏移（-1=前一天，1=后一天）
///
/// # 返回
/// 调整后的 (年, 月, 日)
pub fn adjust_date(year: u16, month: u8, day: u8, day_offset: i8) -> (u16, u8, u8) {
    if day_offset == 0 {
        return (year, month, day);
    }

    if day_offset > 0 {
        // 后一天
        let days_in_month = get_days_in_month(year, month);
        if day < days_in_month {
            (year, month, day + 1)
        } else if month < 12 {
            (year, month + 1, 1)
        } else {
            (year + 1, 1, 1)
        }
    } else {
        // 前一天
        if day > 1 {
            (year, month, day - 1)
        } else if month > 1 {
            let prev_month = month - 1;
            let prev_days = get_days_in_month(year, prev_month);
            (year, prev_month, prev_days)
        } else {
            // 1月1日的前一天是上一年12月31日
            (year - 1, 12, 31)
        }
    }
}

/// 获取指定月份的天数
fn get_days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            // 判断闰年
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30, // 默认值
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equation_of_time() {
        // 2月时差最小（约 -14 分钟）
        assert_eq!(get_equation_of_time(2, 15), -14);
        // 11月时差最大（约 +16 分钟）
        assert_eq!(get_equation_of_time(11, 15), 16);
        // 9月时差接近 0
        assert_eq!(get_equation_of_time(9, 15), 0);
    }

    #[test]
    fn test_apply_true_solar_time_beijing() {
        // 北京（东经116.4°）的修正应该很小
        let result = apply_true_solar_time(2024, 6, 15, 12, 0, 11_640_000);

        // 经度差 = 116.4° - 120° = -3.6°
        // 经度修正 = -3.6 × 4 = -14.4 分钟 ≈ -14 分钟
        // 时差方程（6月）≈ +2 分钟
        // 总修正 ≈ -12 分钟
        // 12:00 - 12分钟 = 11:48
        assert_eq!(result.hour, 11);
        assert!(result.minute >= 46 && result.minute <= 50);
        assert_eq!(result.day_offset, 0);
    }

    #[test]
    fn test_apply_true_solar_time_xinjiang() {
        // 乌鲁木齐（东经87.6°）的修正较大
        let result = apply_true_solar_time(2024, 6, 15, 12, 0, 8_760_000);

        // 经度差 = 87.6° - 120° = -32.4°
        // 经度修正 = -32.4 × 4 = -129.6 分钟 ≈ -2小时10分
        // 时差方程（6月）≈ +2 分钟
        // 总修正 ≈ -2小时8分
        // 12:00 - 2:08 = 9:52
        assert_eq!(result.hour, 9);
        assert!(result.minute >= 50 && result.minute <= 54);
        assert_eq!(result.day_offset, 0);
    }

    #[test]
    fn test_apply_true_solar_time_cross_day() {
        // 测试跨天情况：新疆凌晨1点
        let result = apply_true_solar_time(2024, 6, 15, 1, 0, 8_760_000);

        // 修正约 -2小时8分
        // 1:00 - 2:08 = -1:08 → 前一天 22:52
        assert_eq!(result.hour, 22);
        assert!(result.minute >= 50 && result.minute <= 54);
        assert_eq!(result.day_offset, -1);
    }

    #[test]
    fn test_should_apply_correction() {
        // 北京（116.4°）不需要修正（差距小于7.5°）
        assert!(!should_apply_correction(11_640_000));

        // 上海（121.5°）不需要修正
        assert!(!should_apply_correction(12_150_000));

        // 成都（104°）需要修正（差距 16°）
        assert!(should_apply_correction(10_400_000));

        // 乌鲁木齐（87.6°）需要修正（差距 32.4°）
        assert!(should_apply_correction(8_760_000));
    }
}
