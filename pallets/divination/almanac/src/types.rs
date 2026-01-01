//! 黄历模块 - 数据类型定义
//!
//! 该模块定义了黄历系统中使用的所有数据结构，包括：
//! - AlmanacInfo: 黄历数据结构
//! - OcwConfig: Off-chain Worker 配置
//! - 宜忌事项枚举
//! - 常量定义

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_std::prelude::*;

// ============================================================================
// 常量定义
// ============================================================================

/// 天干名称
pub const TIANGAN_NAMES: [&str; 10] = [
    "甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸",
];

/// 地支名称
pub const DIZHI_NAMES: [&str; 12] = [
    "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
];

/// 生肖名称
pub const ZODIAC_NAMES: [&str; 12] = [
    "鼠", "牛", "虎", "兔", "龙", "蛇", "马", "羊", "猴", "鸡", "狗", "猪",
];

/// 五行名称
pub const WUXING_NAMES: [&str; 5] = ["金", "木", "水", "火", "土"];

/// 建除十二神名称
pub const JIANCHU_NAMES: [&str; 12] = [
    "建", "除", "满", "平", "定", "执", "破", "危", "成", "收", "开", "闭",
];

/// 二十四节气名称
pub const SOLAR_TERM_NAMES: [&str; 25] = [
    "",     // 0: 无节气
    "立春", "雨水", "惊蛰", "春分", "清明", "谷雨",
    "立夏", "小满", "芒种", "夏至", "小暑", "大暑",
    "立秋", "处暑", "白露", "秋分", "寒露", "霜降",
    "立冬", "小雪", "大雪", "冬至", "小寒", "大寒",
];

// ============================================================================
// 宜忌事项枚举
// ============================================================================

/// 宜忌事项
/// 使用 bit 标记，最多支持 64 种事项
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SuitableItem {
    /// 嫁娶
    Marriage = 0,
    /// 纳采（订婚）
    Betrothal = 1,
    /// 祭祀
    Sacrifice = 2,
    /// 祈福
    Prayer = 3,
    /// 出行
    Travel = 4,
    /// 动土
    Groundbreaking = 5,
    /// 破土（下葬挖坑）
    Excavation = 6,
    /// 安葬
    Burial = 7,
    /// 开市/开业
    OpenBusiness = 8,
    /// 交易
    Trading = 9,
    /// 立券（签合同）
    Contract = 10,
    /// 移徙（搬家）
    Moving = 11,
    /// 修造（装修）
    Renovation = 12,
    /// 栽种
    Planting = 13,
    /// 纳财
    ReceiveMoney = 14,
    /// 开光
    Consecration = 15,
    /// 安床
    PlaceBed = 16,
    /// 入宅
    EnterHouse = 17,
    /// 安门
    InstallDoor = 18,
    /// 求嗣（求子）
    PrayForChildren = 19,
    /// 解除
    Remove = 20,
    /// 求医
    SeekMedical = 21,
    /// 词讼（打官司）
    Lawsuit = 22,
    /// 沐浴
    Bathing = 23,
    /// 理发
    Haircut = 24,
    /// 扫舍
    Cleaning = 25,
    /// 会友
    MeetFriends = 26,
    /// 上梁
    RaiseBeam = 27,
    /// 竖柱
    ErectPillar = 28,
    /// 纳畜
    RaiseLivestock = 29,
    /// 伐木
    Logging = 30,
    /// 作灶
    BuildStove = 31,
}

impl SuitableItem {
    /// 获取事项名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Marriage => "嫁娶",
            Self::Betrothal => "纳采",
            Self::Sacrifice => "祭祀",
            Self::Prayer => "祈福",
            Self::Travel => "出行",
            Self::Groundbreaking => "动土",
            Self::Excavation => "破土",
            Self::Burial => "安葬",
            Self::OpenBusiness => "开市",
            Self::Trading => "交易",
            Self::Contract => "立券",
            Self::Moving => "移徙",
            Self::Renovation => "修造",
            Self::Planting => "栽种",
            Self::ReceiveMoney => "纳财",
            Self::Consecration => "开光",
            Self::PlaceBed => "安床",
            Self::EnterHouse => "入宅",
            Self::InstallDoor => "安门",
            Self::PrayForChildren => "求嗣",
            Self::Remove => "解除",
            Self::SeekMedical => "求医",
            Self::Lawsuit => "词讼",
            Self::Bathing => "沐浴",
            Self::Haircut => "理发",
            Self::Cleaning => "扫舍",
            Self::MeetFriends => "会友",
            Self::RaiseBeam => "上梁",
            Self::ErectPillar => "竖柱",
            Self::RaiseLivestock => "纳畜",
            Self::Logging => "伐木",
            Self::BuildStove => "作灶",
        }
    }

    /// 从 bit 位置获取所有启用的事项名称
    pub fn get_items_from_bits(bits: u64) -> Vec<&'static str> {
        let all_items = [
            Self::Marriage,
            Self::Betrothal,
            Self::Sacrifice,
            Self::Prayer,
            Self::Travel,
            Self::Groundbreaking,
            Self::Excavation,
            Self::Burial,
            Self::OpenBusiness,
            Self::Trading,
            Self::Contract,
            Self::Moving,
            Self::Renovation,
            Self::Planting,
            Self::ReceiveMoney,
            Self::Consecration,
            Self::PlaceBed,
            Self::EnterHouse,
            Self::InstallDoor,
            Self::PrayForChildren,
            Self::Remove,
            Self::SeekMedical,
            Self::Lawsuit,
            Self::Bathing,
            Self::Haircut,
            Self::Cleaning,
            Self::MeetFriends,
            Self::RaiseBeam,
            Self::ErectPillar,
            Self::RaiseLivestock,
            Self::Logging,
            Self::BuildStove,
        ];

        let mut result = Vec::new();
        for item in all_items.iter() {
            if bits & (1u64 << (*item as u8)) != 0 {
                result.push(item.name());
            }
        }
        result
    }
}

// ============================================================================
// 黄历数据结构
// ============================================================================

/// 黄历数据结构
///
/// 存储每日黄历的完整信息，采用紧凑的数据格式以节省链上存储空间。
///
/// # 存储优化
/// - 使用 u8 存储枚举值 (天干、地支、生肖等)
/// - 使用 u64 bit 标记存储宜忌事项 (最多 64 种)
/// - 使用 u32 bit 标记存储节日 (最多 32 种)
///
/// # 预估大小
/// - 固定字段: 约 50 bytes
/// - 总计: ~50 bytes/天
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AlmanacInfo {
    // ======== 农历信息 (4 bytes) ========

    /// 农历年份 (如: 2024)
    pub lunar_year: u16,

    /// 农历月份 (1-12, 闰月用 13-24 表示)
    pub lunar_month: u8,

    /// 农历日期 (1-30)
    pub lunar_day: u8,

    // ======== 干支信息 (4 bytes) ========

    /// 年天干 (0-9: 甲乙丙丁戊己庚辛壬癸)
    pub year_tiangan: u8,

    /// 年地支 (0-11: 子丑寅卯辰巳午未申酉戌亥)
    pub year_dizhi: u8,

    /// 月天干
    pub month_tiangan: u8,

    /// 月地支
    pub month_dizhi: u8,

    // ======== 日干支和时辰 (4 bytes) ========

    /// 日天干
    pub day_tiangan: u8,

    /// 日地支
    pub day_dizhi: u8,

    /// 时天干 (子时)
    pub hour_tiangan: u8,

    /// 时地支 (子时)
    pub hour_dizhi: u8,

    // ======== 其他属性 (6 bytes) ========

    /// 生肖 (0-11: 鼠牛虎兔龙蛇马羊猴鸡狗猪)
    pub zodiac: u8,

    /// 冲煞生肖 (0-11)
    pub conflict_zodiac: u8,

    /// 煞方 (0: 东, 1: 南, 2: 西, 3: 北)
    pub sha_direction: u8,

    /// 五行 (0-4: 金木水火土)
    pub wuxing: u8,

    /// 建除十二神 (0-11: 建除满平定执破危成收开闭)
    pub jianchu: u8,

    /// 二十八宿 (0-27)
    pub constellation: u8,

    // ======== 宜忌信息 (16 bytes) ========

    /// 宜 (bit 标记，见 SuitableItem 枚举)
    pub suitable: u64,

    /// 忌 (bit 标记，同上)
    pub avoid: u64,

    // ======== 节气和节日 (6 bytes) ========

    /// 节气 (0: 无, 1-24: 立春至大寒)
    pub solar_term: u8,

    /// 节日标记 (bit 标记)
    /// Bit 0: 元旦, Bit 1: 春节, Bit 2: 清明, Bit 3: 端午
    /// Bit 4: 中秋, Bit 5: 国庆, Bit 6: 元宵, Bit 7: 重阳
    /// ...
    pub festivals: u32,

    /// 吉凶等级 (0: 大吉, 1: 吉, 2: 平, 3: 凶, 4: 大凶)
    pub fortune_level: u8,

    // ======== 元数据 (9 bytes) ========

    /// 数据更新时间戳 (Unix timestamp in seconds)
    pub updated_at: u64,

    /// 数据来源 (0: OCW API, 1: 手动设置, 2: 算法计算)
    pub source: u8,
}

impl Default for AlmanacInfo {
    fn default() -> Self {
        Self {
            lunar_year: 0,
            lunar_month: 0,
            lunar_day: 0,
            year_tiangan: 0,
            year_dizhi: 0,
            month_tiangan: 0,
            month_dizhi: 0,
            day_tiangan: 0,
            day_dizhi: 0,
            hour_tiangan: 0,
            hour_dizhi: 0,
            zodiac: 0,
            conflict_zodiac: 0,
            sha_direction: 0,
            wuxing: 0,
            jianchu: 0,
            constellation: 0,
            suitable: 0,
            avoid: 0,
            solar_term: 0,
            festivals: 0,
            fortune_level: 2, // 默认为平
            updated_at: 0,
            source: 0,
        }
    }
}

impl AlmanacInfo {
    /// 获取年干支字符串
    pub fn year_ganzhi(&self) -> (&'static str, &'static str) {
        (
            TIANGAN_NAMES[self.year_tiangan as usize % 10],
            DIZHI_NAMES[self.year_dizhi as usize % 12],
        )
    }

    /// 获取月干支字符串
    pub fn month_ganzhi(&self) -> (&'static str, &'static str) {
        (
            TIANGAN_NAMES[self.month_tiangan as usize % 10],
            DIZHI_NAMES[self.month_dizhi as usize % 12],
        )
    }

    /// 获取日干支字符串
    pub fn day_ganzhi(&self) -> (&'static str, &'static str) {
        (
            TIANGAN_NAMES[self.day_tiangan as usize % 10],
            DIZHI_NAMES[self.day_dizhi as usize % 12],
        )
    }

    /// 获取生肖名称
    pub fn zodiac_name(&self) -> &'static str {
        ZODIAC_NAMES[self.zodiac as usize % 12]
    }

    /// 获取五行名称
    pub fn wuxing_name(&self) -> &'static str {
        WUXING_NAMES[self.wuxing as usize % 5]
    }

    /// 获取建除名称
    pub fn jianchu_name(&self) -> &'static str {
        JIANCHU_NAMES[self.jianchu as usize % 12]
    }

    /// 获取节气名称
    pub fn solar_term_name(&self) -> &'static str {
        SOLAR_TERM_NAMES[self.solar_term as usize % 25]
    }

    /// 获取宜事项列表
    pub fn suitable_items(&self) -> Vec<&'static str> {
        SuitableItem::get_items_from_bits(self.suitable)
    }

    /// 获取忌事项列表
    pub fn avoid_items(&self) -> Vec<&'static str> {
        SuitableItem::get_items_from_bits(self.avoid)
    }

    /// 检查是否宜某事
    pub fn is_suitable(&self, item: SuitableItem) -> bool {
        self.suitable & (1u64 << (item as u8)) != 0
    }

    /// 检查是否忌某事
    pub fn is_avoid(&self, item: SuitableItem) -> bool {
        self.avoid & (1u64 << (item as u8)) != 0
    }

    /// 获取吉凶等级描述
    pub fn fortune_description(&self) -> &'static str {
        match self.fortune_level {
            0 => "大吉",
            1 => "吉",
            2 => "平",
            3 => "凶",
            4 => "大凶",
            _ => "未知",
        }
    }
}

// ============================================================================
// OCW 配置
// ============================================================================

/// Off-chain Worker 配置
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct OcwConfig {
    /// 是否启用 OCW 自动更新
    pub enabled: bool,

    /// 每日更新的 UTC 小时 (0-23)
    pub update_hour: u8,

    /// 批量获取天数 (1-90)
    pub batch_days: u8,

    /// 上次成功更新的时间戳
    pub last_update: u64,

    /// 连续失败次数
    pub failure_count: u8,

    /// 最大重试次数
    pub max_retries: u8,
}

impl Default for OcwConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            update_hour: 0,      // UTC 00:00 (北京时间 08:00)
            batch_days: 7,       // 每次获取 7 天
            last_update: 0,
            failure_count: 0,
            max_retries: 3,
        }
    }
}

// ============================================================================
// 日期键类型
// ============================================================================

/// 日期键类型 (year, month, day)
pub type DateKey = (u16, u8, u8);

/// 验证日期是否有效
pub fn validate_date(year: u16, month: u8, day: u8) -> bool {
    if year < 1900 || year > 2100 {
        return false;
    }
    if month < 1 || month > 12 {
        return false;
    }
    if day < 1 || day > 31 {
        return false;
    }

    // 简单的月份天数验证
    let max_day = match month {
        2 => {
            // 闰年判断
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };

    day <= max_day
}

/// 计算下一天
pub fn next_day(year: u16, month: u8, day: u8) -> DateKey {
    let max_day = match month {
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };

    if day < max_day {
        (year, month, day + 1)
    } else if month < 12 {
        (year, month + 1, 1)
    } else {
        (year + 1, 1, 1)
    }
}

// ============================================================================
// 数据来源枚举
// ============================================================================

/// 数据来源
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DataSource {
    /// 通过 OCW 从 API 获取
    OcwApi = 0,
    /// 手动设置
    Manual = 1,
    /// 算法计算
    Calculated = 2,
}

impl From<u8> for DataSource {
    fn from(value: u8) -> Self {
        match value {
            0 => DataSource::OcwApi,
            1 => DataSource::Manual,
            2 => DataSource::Calculated,
            _ => DataSource::Manual,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_date() {
        assert!(validate_date(2024, 1, 1));
        assert!(validate_date(2024, 12, 31));
        assert!(validate_date(2024, 2, 29)); // 2024 是闰年
        assert!(!validate_date(2023, 2, 29)); // 2023 不是闰年
        assert!(!validate_date(2024, 13, 1)); // 无效月份
        assert!(!validate_date(2024, 4, 31)); // 4月没有31日
    }

    #[test]
    fn test_next_day() {
        assert_eq!(next_day(2024, 1, 1), (2024, 1, 2));
        assert_eq!(next_day(2024, 1, 31), (2024, 2, 1));
        assert_eq!(next_day(2024, 2, 29), (2024, 3, 1)); // 闰年
        assert_eq!(next_day(2023, 2, 28), (2023, 3, 1)); // 非闰年
        assert_eq!(next_day(2024, 12, 31), (2025, 1, 1)); // 跨年
    }

    #[test]
    fn test_suitable_items() {
        let bits: u64 = (1 << SuitableItem::Marriage as u8)
            | (1 << SuitableItem::Travel as u8)
            | (1 << SuitableItem::OpenBusiness as u8);

        let items = SuitableItem::get_items_from_bits(bits);
        assert!(items.contains(&"嫁娶"));
        assert!(items.contains(&"出行"));
        assert!(items.contains(&"开市"));
        assert!(!items.contains(&"安葬"));
    }

    #[test]
    fn test_almanac_info_methods() {
        let info = AlmanacInfo {
            year_tiangan: 0, // 甲
            year_dizhi: 4,   // 辰
            zodiac: 4,       // 龙
            wuxing: 1,       // 木
            jianchu: 0,      // 建
            solar_term: 3,   // 惊蛰
            suitable: 1 << SuitableItem::Marriage as u8,
            avoid: 1 << SuitableItem::Burial as u8,
            fortune_level: 1, // 吉
            ..Default::default()
        };

        assert_eq!(info.year_ganzhi(), ("甲", "辰"));
        assert_eq!(info.zodiac_name(), "龙");
        assert_eq!(info.wuxing_name(), "木");
        assert_eq!(info.jianchu_name(), "建");
        assert_eq!(info.solar_term_name(), "惊蛰");
        assert!(info.is_suitable(SuitableItem::Marriage));
        assert!(!info.is_suitable(SuitableItem::Burial));
        assert!(info.is_avoid(SuitableItem::Burial));
        assert_eq!(info.fortune_description(), "吉");
    }
}
