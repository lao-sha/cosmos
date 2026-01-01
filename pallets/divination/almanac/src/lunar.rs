//! # 农历计算模块
//!
//! 本模块实现了公历与农历的相互转换，以及干支计算。
//! 使用预存储的农历数据表（1901-2100年，200年数据）。
//!
//! ## 数据来源
//! 农历数据基于香港天文台的历书数据。
//!
//! ## 功能
//! - 公历转农历
//! - 干支计算（年/月/日/时）
//! - 生肖计算
//! - 节气计算

use crate::lunar_data::{
    get_leap_month, get_leap_month_days, get_lunar_info, get_month_days,
    get_spring_festival, get_year_days,
};
use sp_std::prelude::*;

// ============================================================================
// 常量定义
// ============================================================================

/// 农历数据起始年份
pub const LUNAR_START_YEAR: u16 = 1901;

/// 农历数据结束年份
pub const LUNAR_END_YEAR: u16 = 2100;

/// 天干名称
pub const TIANGAN: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];

/// 地支名称
pub const DIZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];

/// 生肖名称
pub const SHENGXIAO: [&str; 12] = ["鼠", "牛", "虎", "兔", "龙", "蛇", "马", "羊", "猴", "鸡", "狗", "猪"];

/// 农历月份名称
pub const LUNAR_MONTHS: [&str; 12] = [
    "正月", "二月", "三月", "四月", "五月", "六月",
    "七月", "八月", "九月", "十月", "冬月", "腊月"
];

/// 农历日期名称
pub const LUNAR_DAYS: [&str; 30] = [
    "初一", "初二", "初三", "初四", "初五", "初六", "初七", "初八", "初九", "初十",
    "十一", "十二", "十三", "十四", "十五", "十六", "十七", "十八", "十九", "二十",
    "廿一", "廿二", "廿三", "廿四", "廿五", "廿六", "廿七", "廿八", "廿九", "三十"
];

/// 二十四节气名称
pub const SOLAR_TERMS: [&str; 24] = [
    "小寒", "大寒", "立春", "雨水", "惊蛰", "春分",
    "清明", "谷雨", "立夏", "小满", "芒种", "夏至",
    "小暑", "大暑", "立秋", "处暑", "白露", "秋分",
    "寒露", "霜降", "立冬", "小雪", "大雪", "冬至"
];

// ============================================================================
// 农历日期结构
// ============================================================================

/// 农历日期
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LunarDate {
    /// 农历年
    pub year: u16,
    /// 农历月 (1-12)
    pub month: u8,
    /// 农历日 (1-30)
    pub day: u8,
    /// 是否闰月
    pub is_leap: bool,
}

impl LunarDate {
    /// 获取农历月份名称
    pub fn month_name(&self) -> &'static str {
        if self.month >= 1 && self.month <= 12 {
            LUNAR_MONTHS[(self.month - 1) as usize]
        } else {
            "未知"
        }
    }

    /// 获取农历日期名称
    pub fn day_name(&self) -> &'static str {
        if self.day >= 1 && self.day <= 30 {
            LUNAR_DAYS[(self.day - 1) as usize]
        } else {
            "未知"
        }
    }

    /// 获取完整农历日期字符串（如"正月初一"）
    pub fn full_name(&self) -> (&'static str, &'static str) {
        (self.month_name(), self.day_name())
    }
}

/// 干支
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct GanZhi {
    /// 天干 (0-9)
    pub gan: u8,
    /// 地支 (0-11)
    pub zhi: u8,
}

impl GanZhi {
    /// 创建新的干支
    pub const fn new(gan: u8, zhi: u8) -> Self {
        Self {
            gan: gan % 10,
            zhi: zhi % 12,
        }
    }

    /// 获取天干名称
    pub fn gan_name(&self) -> &'static str {
        TIANGAN[self.gan as usize % 10]
    }

    /// 获取地支名称
    pub fn zhi_name(&self) -> &'static str {
        DIZHI[self.zhi as usize % 12]
    }

    /// 获取完整干支名称
    pub fn name(&self) -> (&'static str, &'static str) {
        (self.gan_name(), self.zhi_name())
    }

    /// 获取干支索引 (0-59，六十甲子)
    pub fn index(&self) -> u8 {
        // 六十甲子循环
        ((self.gan as i16 * 6 - self.zhi as i16 * 5 + 60) % 60) as u8
    }
}

/// 四柱（八字）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct FourPillars {
    /// 年柱
    pub year: GanZhi,
    /// 月柱
    pub month: GanZhi,
    /// 日柱
    pub day: GanZhi,
    /// 时柱
    pub hour: GanZhi,
}

// ============================================================================
// 公历转农历
// ============================================================================

/// 农历转公历
///
/// # 参数
/// - `lunar_year`: 农历年份 (1901-2100)
/// - `lunar_month`: 农历月份 (1-12)
/// - `lunar_day`: 农历日期 (1-30)
/// - `is_leap_month`: 是否为闰月
///
/// # 返回
/// 公历日期 (year, month, day)，如果输入无效则返回 None
///
/// # 示例
/// ```ignore
/// // 农历 2024年正月初一 转换为公历
/// let (year, month, day) = lunar_to_solar(2024, 1, 1, false)?;
/// // 结果: 2024年2月10日（春节）
/// ```
pub fn lunar_to_solar(
    lunar_year: u16,
    lunar_month: u8,
    lunar_day: u8,
    is_leap_month: bool,
) -> Option<(u16, u8, u8)> {
    // 验证年份范围
    if lunar_year < LUNAR_START_YEAR || lunar_year > LUNAR_END_YEAR {
        return None;
    }

    // 验证月份和日期
    if lunar_month < 1 || lunar_month > 12 || lunar_day < 1 || lunar_day > 30 {
        return None;
    }

    // 获取农历年信息
    let info = get_lunar_info(lunar_year)?;
    let leap_month = get_leap_month(info);

    // 验证闰月有效性
    if is_leap_month && (leap_month == 0 || leap_month != lunar_month) {
        return None; // 该月不是闰月
    }

    // 计算从农历正月初一到目标日期的天数
    let mut days_offset: i32 = 0;

    // 累加之前月份的天数
    for m in 1..lunar_month {
        // 普通月份
        days_offset += get_month_days(info, m) as i32;

        // 检查是否有闰月在当前月之后
        if leap_month > 0 && m == leap_month {
            days_offset += get_leap_month_days(info) as i32;
        }
    }

    // 如果目标是闰月，需要先加上该月的普通月份天数
    if is_leap_month {
        days_offset += get_month_days(info, lunar_month) as i32;
    }

    // 加上当月的天数（减1因为初一是第0天）
    days_offset += (lunar_day - 1) as i32;

    // 获取该年春节日期（农历正月初一对应的公历日期）
    let (sf_month, sf_day) = get_spring_festival(lunar_year);

    // 计算春节的儒略日
    let sf_jd = julian_day(lunar_year, sf_month, sf_day);

    // 目标日期的儒略日
    let target_jd = sf_jd + days_offset;

    // 转换为公历日期
    let (year, month, day) = from_julian_day(target_jd);

    Some((year, month, day))
}

/// 公历转农历
///
/// # 参数
/// - `year`: 公历年份 (1901-2100)
/// - `month`: 公历月份 (1-12)
/// - `day`: 公历日期 (1-31)
///
/// # 返回
/// 农历日期，如果输入无效则返回 None
pub fn solar_to_lunar(year: u16, month: u8, day: u8) -> Option<LunarDate> {
    // 验证年份范围
    if year < LUNAR_START_YEAR || year > LUNAR_END_YEAR {
        return None;
    }

    // 验证日期有效性
    if month < 1 || month > 12 || day < 1 || day > 31 {
        return None;
    }

    // 计算距离该年1月1日的天数
    let days_in_year = days_from_jan1(month, day, is_leap_year(year));

    // 获取该年春节日期
    let (sf_month, sf_day) = get_spring_festival(year);
    let sf_days = days_from_jan1(sf_month, sf_day, is_leap_year(year));

    // 计算相对于春节的天数差
    let mut lunar_year: u16;
    let mut offset: i32;

    if days_in_year >= sf_days {
        // 春节当天或之后
        lunar_year = year;
        offset = (days_in_year - sf_days) as i32;
    } else {
        // 春节之前，属于上一个农历年
        lunar_year = year - 1;
        if lunar_year < LUNAR_START_YEAR {
            return None;
        }
        // 计算上一年春节到年底的天数 + 今年1月1日到该日期的天数
        let prev_year_days = get_year_days(lunar_year) as i32;
        let (prev_sf_month, prev_sf_day) = get_spring_festival(lunar_year);
        let prev_sf_days = days_from_jan1(prev_sf_month, prev_sf_day, is_leap_year(lunar_year)) as i32;
        let prev_year_total = if is_leap_year(lunar_year) { 366 } else { 365 };

        offset = (prev_year_total - prev_sf_days) as i32 + days_in_year as i32;

        // 如果偏移超过上一年农历年的天数，说明计算有误
        if offset >= prev_year_days {
            // 实际上属于当前农历年
            lunar_year = year;
            offset = (days_in_year - sf_days) as i32;
        }
    }

    // 获取农历年信息
    let info = get_lunar_info(lunar_year)?;
    let leap_month = get_leap_month(info);

    // 计算农历月日
    let mut lunar_month = 1u8;
    let mut is_leap = false;
    let mut remaining = offset;

    // 遍历月份
    let mut m = 1u8;
    while m <= 12 {
        // 普通月份
        let month_days = get_month_days(info, m) as i32;

        if remaining < month_days {
            lunar_month = m;
            break;
        }
        remaining -= month_days;

        // 检查闰月
        if leap_month > 0 && m == leap_month {
            let leap_days = get_leap_month_days(info) as i32;
            if remaining < leap_days {
                lunar_month = m;
                is_leap = true;
                break;
            }
            remaining -= leap_days;
        }

        m += 1;
    }

    // 如果遍历完所有月份仍有剩余，说明数据有问题
    if m > 12 && remaining >= 0 {
        lunar_month = 12;
        remaining = remaining.min(29); // 防止溢出
    }

    Some(LunarDate {
        year: lunar_year,
        month: lunar_month,
        day: (remaining + 1).max(1).min(30) as u8,
        is_leap,
    })
}

/// 计算某日期距离该年1月1日的天数（0-based）
fn days_from_jan1(month: u8, day: u8, is_leap: bool) -> i32 {
    let days_in_month = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut total = 0i32;
    for m in 0..(month - 1) as usize {
        total += days_in_month[m] as i32;
    }
    total + (day - 1) as i32
}

/// 判断是否闰年
pub fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

// ============================================================================
// 干支计算
// ============================================================================

/// 计算年干支
///
/// 以立春为界，立春前为上一年
pub fn year_ganzhi(year: u16) -> GanZhi {
    // 1984年为甲子年 (干=0, 支=0)
    let offset = ((year as i32 - 1984) % 60 + 60) % 60;
    GanZhi {
        gan: (offset % 10) as u8,
        zhi: (offset % 12) as u8,
    }
}

/// 计算月干支
///
/// 月干支以节气为界
/// 正月建寅，从寅月开始
pub fn month_ganzhi(year: u16, lunar_month: u8) -> GanZhi {
    // 月地支：正月为寅(2)，依次类推
    let zhi = ((lunar_month as i32 - 1 + 2) % 12) as u8;

    // 月天干：年干决定正月天干
    // 甲己之年丙作首，乙庚之岁戊为头
    // 丙辛之年寻庚上，丁壬壬寅顺行流
    // 戊癸之年何处起，甲寅之上去寻求
    let year_gan = year_ganzhi(year).gan;
    let first_month_gan = match year_gan % 5 {
        0 => 2, // 甲己年，正月丙寅
        1 => 4, // 乙庚年，正月戊寅
        2 => 6, // 丙辛年，正月庚寅
        3 => 8, // 丁壬年，正月壬寅
        4 => 0, // 戊癸年，正月甲寅
        _ => 0,
    };

    let gan = ((first_month_gan + lunar_month as i32 - 1) % 10) as u8;

    GanZhi { gan, zhi }
}

/// 计算日干支
///
/// 使用儒略日数计算
pub fn day_ganzhi(year: u16, month: u8, day: u8) -> GanZhi {
    let jd = julian_day(year, month, day);
    // 儒略日 2451911 是 2001年1月1日，该日为辛巳日 (干=7, 支=5)
    let offset = ((jd - 2451911) % 60 + 60) % 60;
    let base_gan = 7i32; // 辛
    let base_zhi = 5i32; // 巳

    GanZhi {
        gan: ((base_gan + offset) % 10) as u8,
        zhi: ((base_zhi + offset) % 12) as u8,
    }
}

/// 计算时辰干支
///
/// 时辰地支固定，天干由日干决定
/// hour: 0-23 小时
pub fn hour_ganzhi(day_gan: u8, hour: u8) -> GanZhi {
    // 时辰：子时(23-1)=0, 丑时(1-3)=1, ...
    // 每两小时一个时辰，23点开始算子时
    let zhi = match hour {
        23 | 0 => 0,  // 子时
        1 | 2 => 1,   // 丑时
        3 | 4 => 2,   // 寅时
        5 | 6 => 3,   // 卯时
        7 | 8 => 4,   // 辰时
        9 | 10 => 5,  // 巳时
        11 | 12 => 6, // 午时
        13 | 14 => 7, // 未时
        15 | 16 => 8, // 申时
        17 | 18 => 9, // 酉时
        19 | 20 => 10, // 戌时
        21 | 22 => 11, // 亥时
        _ => 0,
    };

    // 时干：日干决定子时天干
    // 甲己还加甲，乙庚丙作初
    // 丙辛从戊起，丁壬庚子居
    // 戊癸何方发，壬子是真途
    let zi_gan = match day_gan % 5 {
        0 => 0, // 甲己日，子时甲子
        1 => 2, // 乙庚日，子时丙子
        2 => 4, // 丙辛日，子时戊子
        3 => 6, // 丁壬日，子时庚子
        4 => 8, // 戊癸日，子时壬子
        _ => 0,
    };

    let gan = ((zi_gan + zhi as i32) % 10) as u8;

    GanZhi {
        gan,
        zhi: zhi as u8,
    }
}

/// 计算四柱（八字）
pub fn four_pillars(year: u16, month: u8, day: u8, hour: u8) -> FourPillars {
    let year_gz = year_ganzhi(year);
    let day_gz = day_ganzhi(year, month, day);

    // 获取农历月份（用于月柱计算）
    let lunar = solar_to_lunar(year, month, day);
    let lunar_month = lunar.map(|l| l.month).unwrap_or(month);

    FourPillars {
        year: year_gz,
        month: month_ganzhi(year, lunar_month),
        day: day_gz,
        hour: hour_ganzhi(day_gz.gan, hour),
    }
}

/// 获取生肖
pub fn get_zodiac(year: u16) -> u8 {
    year_ganzhi(year).zhi
}

/// 获取生肖名称
pub fn zodiac_name(year: u16) -> &'static str {
    SHENGXIAO[get_zodiac(year) as usize]
}

// ============================================================================
// 节气计算
// ============================================================================

/// 节气大约日期表
/// 每月两个节气，[第一个节气日期, 第二个节气日期]
const TERM_BASE_DAYS: [[u8; 2]; 12] = [
    [6, 20],   // 1月: 小寒(~6), 大寒(~20)
    [4, 19],   // 2月: 立春(~4), 雨水(~19)
    [6, 21],   // 3月: 惊蛰(~6), 春分(~21)
    [5, 20],   // 4月: 清明(~5), 谷雨(~20)
    [6, 21],   // 5月: 立夏(~6), 小满(~21)
    [6, 21],   // 6月: 芒种(~6), 夏至(~21)
    [7, 23],   // 7月: 小暑(~7), 大暑(~23)
    [8, 23],   // 8月: 立秋(~8), 处暑(~23)
    [8, 23],   // 9月: 白露(~8), 秋分(~23)
    [8, 23],   // 10月: 寒露(~8), 霜降(~23)
    [7, 22],   // 11月: 立冬(~7), 小雪(~22)
    [7, 22],   // 12月: 大雪(~7), 冬至(~22)
];

/// 获取指定日期的节气
///
/// 返回节气索引 (0-23)，如果不是节气日则返回 None
pub fn get_solar_term(year: u16, month: u8, day: u8) -> Option<u8> {
    if month < 1 || month > 12 {
        return None;
    }

    let m_idx = (month - 1) as usize;
    let base = TERM_BASE_DAYS[m_idx];

    // 根据年份微调（简化算法）
    let adjust = ((year as i32 - 2000) / 100) as i32;

    let term1_day = (base[0] as i32 + adjust / 4).max(1).min(28) as u8;
    let term2_day = (base[1] as i32 + adjust / 4).max(1).min(28) as u8;

    if day == term1_day {
        // 第一个节气
        Some((month - 1) * 2)
    } else if day == term2_day {
        // 第二个节气
        Some((month - 1) * 2 + 1)
    } else {
        None
    }
}

/// 获取节气名称
pub fn solar_term_name(index: u8) -> &'static str {
    if index < 24 {
        SOLAR_TERMS[index as usize]
    } else {
        ""
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 计算儒略日数
pub fn julian_day(year: u16, month: u8, day: u8) -> i32 {
    let y = year as i32;
    let m = month as i32;
    let d = day as i32;

    let a = (14 - m) / 12;
    let y2 = y + 4800 - a;
    let m2 = m + 12 * a - 3;

    d + (153 * m2 + 2) / 5 + 365 * y2 + y2 / 4 - y2 / 100 + y2 / 400 - 32045
}

/// 从儒略日计算公历日期
pub fn from_julian_day(jd: i32) -> (u16, u8, u8) {
    let a = jd + 32044;
    let b = (4 * a + 3) / 146097;
    let c = a - (146097 * b) / 4;
    let d = (4 * c + 3) / 1461;
    let e = c - (1461 * d) / 4;
    let m = (5 * e + 2) / 153;

    let day = (e - (153 * m + 2) / 5 + 1) as u8;
    let month = (m + 3 - 12 * (m / 10)) as u8;
    let year = (100 * b + d - 4800 + m / 10) as u16;

    (year, month, day)
}

// ============================================================================
// 时间戳转换（供梅花易数等模块使用）
// ============================================================================

/// 农历转换错误类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LunarConvertError {
    /// 年份超出支持范围（1901-2100）
    InvalidYear,
    /// 日期早于支持的最早日期
    DateTooEarly,
    /// 无效的月份
    InvalidMonth,
    /// 无效的日期
    InvalidDay,
}

/// 梅花易数专用农历日期结构
///
/// 包含时辰地支数等梅花易数起卦所需的字段
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct MeihuaLunarDate {
    /// 农历年份（如 2024）
    pub year: u16,
    /// 年地支数（子=1, 丑=2, ..., 亥=12）
    pub year_zhi_num: u8,
    /// 农历月份（1-12）
    pub month: u8,
    /// 农历日（1-30）
    pub day: u8,
    /// 时辰地支数（子=1, 丑=2, ..., 亥=12）
    pub hour_zhi_num: u8,
    /// 是否闰月
    pub is_leap_month: bool,
}

impl MeihuaLunarDate {
    /// 获取梅花易数起卦用的年数
    ///
    /// 使用年地支数（子=1, 丑=2, ..., 亥=12）
    pub fn year_num(&self) -> u16 {
        self.year_zhi_num as u16
    }

    /// 获取月数（1-12）
    pub fn month_num(&self) -> u16 {
        self.month as u16
    }

    /// 获取日数（1-30）
    pub fn day_num(&self) -> u16 {
        self.day as u16
    }

    /// 获取时辰数（子=1, 丑=2, ..., 亥=12）
    pub fn hour_num(&self) -> u16 {
        self.hour_zhi_num as u16
    }
}

/// 从小时数获取时辰地支数
///
/// # 参数
/// - `hour`: 24小时制的小时数（0-23）
///
/// # 返回
/// 时辰地支数（子=1, 丑=2, ..., 亥=12）
///
/// # 时辰对照
/// - 子时：23:00-01:00 → 1
/// - 丑时：01:00-03:00 → 2
/// - ...
/// - 亥时：21:00-23:00 → 12
pub fn hour_to_dizhi_num(hour: u8) -> u8 {
    match hour {
        23 | 0 => 1,   // 子
        1 | 2 => 2,    // 丑
        3 | 4 => 3,    // 寅
        5 | 6 => 4,    // 卯
        7 | 8 => 5,    // 辰
        9 | 10 => 6,   // 巳
        11 | 12 => 7,  // 午
        13 | 14 => 8,  // 未
        15 | 16 => 9,  // 申
        17 | 18 => 10, // 酉
        19 | 20 => 11, // 戌
        21 | 22 => 12, // 亥
        _ => 1,
    }
}

/// 从年份获取年地支数
///
/// # 算法
/// 以1984年为甲子年（地支数1）为基准
///
/// # 参数
/// - `year`: 年份
///
/// # 返回
/// 年地支数（子=1, 丑=2, ..., 亥=12）
pub fn year_to_dizhi_num(year: u16) -> u8 {
    // 1984年为甲子年，地支为子=0（0-based）
    // 转换为1-based：子=1, 丑=2, ..., 亥=12
    let offset = ((year as i32 - 1984) % 12 + 12) % 12;
    (offset as u8) + 1
}

/// 时间戳转梅花易数农历日期
///
/// # 参数
/// - `timestamp`: Unix时间戳（秒）
///
/// # 返回
/// - Ok(MeihuaLunarDate): 转换成功
/// - Err(LunarConvertError): 转换失败
pub fn timestamp_to_meihua_lunar(timestamp: u64) -> Result<MeihuaLunarDate, LunarConvertError> {
    // 转换为北京时间（UTC+8）
    let local_timestamp = timestamp + 8 * 3600;

    // 计算公历日期
    // Unix时间戳基准：1970-01-01 00:00:00 UTC
    // 儒略日2440588 = 1970-01-01
    let days_since_epoch = (local_timestamp / 86400) as i32;
    let jd = days_since_epoch + 2440588;
    let (year, month, day) = from_julian_day(jd);

    // 计算时辰
    let hour = ((local_timestamp % 86400) / 3600) as u8;
    let hour_zhi_num = hour_to_dizhi_num(hour);

    // 公历转农历
    let lunar = solar_to_lunar(year, month, day)
        .ok_or(LunarConvertError::InvalidYear)?;

    // 计算年地支数
    let year_zhi_num = year_to_dizhi_num(lunar.year);

    Ok(MeihuaLunarDate {
        year: lunar.year,
        year_zhi_num,
        month: lunar.month,
        day: lunar.day,
        hour_zhi_num,
        is_leap_month: lunar.is_leap,
    })
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_year_ganzhi() {
        // 2024年是甲辰年
        let gz = year_ganzhi(2024);
        assert_eq!(gz.gan, 0); // 甲
        assert_eq!(gz.zhi, 4); // 辰
        assert_eq!(gz.gan_name(), "甲");
        assert_eq!(gz.zhi_name(), "辰");
    }

    #[test]
    fn test_day_ganzhi() {
        // 验证日干支计算
        let gz = day_ganzhi(2024, 1, 1);
        assert!(gz.gan < 10);
        assert!(gz.zhi < 12);
    }

    #[test]
    fn test_julian_day() {
        // 2000年1月1日的儒略日是 2451545
        let jd = julian_day(2000, 1, 1);
        assert_eq!(jd, 2451545);

        // 验证往返转换
        let (y, m, d) = from_julian_day(jd);
        assert_eq!((y, m, d), (2000, 1, 1));
    }

    #[test]
    fn test_zodiac() {
        // 2024年是龙年
        assert_eq!(get_zodiac(2024), 4); // 辰=龙
        assert_eq!(zodiac_name(2024), "龙");

        // 2023年是兔年
        assert_eq!(get_zodiac(2023), 3); // 卯=兔
        assert_eq!(zodiac_name(2023), "兔");
    }

    #[test]
    fn test_solar_to_lunar() {
        // 测试基本功能
        // 超出范围的日期返回 None
        assert!(solar_to_lunar(1800, 1, 1).is_none());
        assert!(solar_to_lunar(2200, 1, 1).is_none());

        // 有效日期应返回 Some
        let lunar = solar_to_lunar(2024, 6, 15);
        assert!(lunar.is_some());

        if let Some(l) = lunar {
            // 基本范围验证
            assert!(l.year >= 2023 && l.year <= 2025);
            assert!(l.month >= 1 && l.month <= 12);
            assert!(l.day >= 1 && l.day <= 30);
        }
    }

    #[test]
    fn test_hour_ganzhi() {
        // 测试时辰干支
        let day_gan = 0; // 甲日
        let gz = hour_ganzhi(day_gan, 0); // 子时
        assert_eq!(gz.gan, 0); // 甲
        assert_eq!(gz.zhi, 0); // 子
    }

    #[test]
    fn test_four_pillars() {
        // 测试四柱计算
        let pillars = four_pillars(2024, 6, 15, 12);
        assert_eq!(pillars.year.gan, 0); // 甲
        assert_eq!(pillars.year.zhi, 4); // 辰
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
    }

    #[test]
    fn test_lunar_date_names() {
        let lunar = LunarDate {
            year: 2024,
            month: 1,
            day: 1,
            is_leap: false,
        };
        assert_eq!(lunar.month_name(), "正月");
        assert_eq!(lunar.day_name(), "初一");
    }

    #[test]
    fn test_lunar_to_solar() {
        // 测试农历转公历

        // 2024年春节：农历正月初一 = 公历 2024年2月10日
        let result = lunar_to_solar(2024, 1, 1, false);
        assert!(result.is_some());
        let (year, month, day) = result.unwrap();
        assert_eq!(year, 2024);
        assert_eq!(month, 2);
        assert_eq!(day, 10);

        // 测试无效输入
        assert!(lunar_to_solar(1800, 1, 1, false).is_none()); // 年份超出范围
        assert!(lunar_to_solar(2024, 13, 1, false).is_none()); // 月份无效
        assert!(lunar_to_solar(2024, 1, 31, false).is_none()); // 日期无效

        // 测试无效闰月
        assert!(lunar_to_solar(2024, 1, 1, true).is_none()); // 正月不是闰月
    }

    #[test]
    fn test_lunar_solar_roundtrip() {
        // 测试公历→农历→公历往返转换
        let original = (2024, 6, 15);
        let lunar = solar_to_lunar(original.0, original.1, original.2);
        assert!(lunar.is_some());

        let l = lunar.unwrap();
        let back = lunar_to_solar(l.year, l.month, l.day, l.is_leap);
        assert!(back.is_some());

        let (year, month, day) = back.unwrap();
        assert_eq!(year, original.0);
        assert_eq!(month, original.1);
        assert_eq!(day, original.2);
    }
}
