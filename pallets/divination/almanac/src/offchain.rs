//! # Off-chain Worker 模块
//!
//! 该模块实现了黄历数据的自动获取和提交逻辑:
//! 1. 从环境变量读取 AppCode
//! 2. 调用阿里云黄历 API
//! 3. 解析 JSON 响应
//! 4. 将数据提交到链上
//!
//! ## AppCode 配置
//! AppCode 通过环境变量 `ALMANAC_APPCODE` 配置，
//! 在节点启动时由 command.rs 读取并传递给 service.rs。
//!
//! ## API 调用
//! 使用阿里云市场的黄历 API:
//! - 域名: https://jmhlysjjr.market.alicloudapi.com
//! - 路径: /holiday/list 或 /almanac/day
//!
//! ## 安全注意
//! - AppCode 不应该存储在链上
//! - 日志中不应打印完整的 AppCode

use crate::{types::*, Config, OcwConfigStorage};
use sp_runtime::offchain::{http, Duration};
use sp_std::prelude::*;

/// OCW 存储键前缀
#[allow(dead_code)]
const OCW_STORAGE_PREFIX: &[u8] = b"almanac::";

/// AppCode 存储键
const APPCODE_KEY: &[u8] = b"almanac::appcode";

/// API 基础 URL
const API_BASE_URL: &str = "https://jmhlysjjr.market.alicloudapi.com";

/// API 请求超时时间 (毫秒)
const API_TIMEOUT_MS: u64 = 10000;

/// 从 API 获取数据并提交到链上
///
/// # 流程
/// 1. 获取 AppCode
/// 2. 确定要获取的日期范围
/// 3. 批量调用 API
/// 4. 解析响应并构造 AlmanacInfo
/// 5. 提交签名交易
pub fn fetch_and_submit<T: Config>() -> Result<(), &'static str> {
    // 1. 获取 AppCode
    let appcode = get_appcode()?;

    log::info!(
        target: "almanac-ocw",
        "🔑 AppCode loaded (length: {} bytes)",
        appcode.len()
    );

    // 2. 获取 OCW 配置
    let config = OcwConfigStorage::<T>::get();

    // 3. 确定要获取的日期范围
    let today = get_today_date();
    let batch_days = config.batch_days;

    log::info!(
        target: "almanac-ocw",
        "📅 Fetching {} days starting from {:?}",
        batch_days,
        today
    );

    // 4. 批量获取数据
    let mut results = Vec::new();
    let (mut year, mut month, mut day) = today;

    for i in 0..batch_days {
        match fetch_almanac_from_api(&appcode, year, month, day) {
            Ok(info) => {
                log::info!(
                    target: "almanac-ocw",
                    "✅ [{}/{}] Fetched almanac for {}-{}-{}",
                    i + 1,
                    batch_days,
                    year,
                    month,
                    day
                );
                results.push(((year, month, day), info));
            }
            Err(e) => {
                log::error!(
                    target: "almanac-ocw",
                    "❌ [{}/{}] Failed to fetch {}-{}-{}: {}",
                    i + 1,
                    batch_days,
                    year,
                    month,
                    day,
                    e
                );
            }
        }

        // 计算下一天
        (year, month, day) = next_day(year, month, day);

        // 延迟以避免 API 限流 (500ms)
        sp_io::offchain::sleep_until(
            sp_io::offchain::timestamp().add(Duration::from_millis(500))
        );
    }

    if results.is_empty() {
        return Err("No data fetched");
    }

    log::info!(
        target: "almanac-ocw",
        "📊 Successfully fetched {} days of data",
        results.len()
    );

    // 5. 提交数据到链上
    // 注意: 这里需要使用签名交易，暂时只打印日志
    // 实际实现需要配置 OCW 签名密钥
    log::info!(
        target: "almanac-ocw",
        "📤 TODO: Submit {} entries to chain",
        results.len()
    );

    Ok(())
}

/// 从环境变量或 OCW 本地存储获取 AppCode
fn get_appcode() -> Result<Vec<u8>, &'static str> {
    // 方式 1: 从 OCW 本地存储读取 (由 service.rs 在启动时写入)
    if let Some(appcode) = sp_io::offchain::local_storage_get(
        sp_core::offchain::StorageKind::PERSISTENT,
        APPCODE_KEY,
    ) {
        if !appcode.is_empty() {
            return Ok(appcode);
        }
    }

    // 方式 2: 从环境变量读取 (std 环境下)
    #[cfg(feature = "std")]
    {
        if let Ok(appcode) = std::env::var("ALMANAC_APPCODE") {
            if !appcode.is_empty() {
                return Ok(appcode.into_bytes());
            }
        }
    }

    Err("AppCode not configured. Set ALMANAC_APPCODE environment variable.")
}

/// 从阿里云 API 获取单日黄历数据
fn fetch_almanac_from_api(
    appcode: &[u8],
    year: u16,
    month: u8,
    day: u8,
) -> Result<AlmanacInfo, &'static str> {
    // 1. 构造请求 URL 和 body
    let url = format!("{}/holiday/list", API_BASE_URL);
    let body = format!("year={}&month={}&day={}", year, month, day);

    // 2. 构造 Authorization header
    let appcode_str = sp_std::str::from_utf8(appcode)
        .map_err(|_| "Invalid AppCode UTF-8")?;
    let auth_header = format!("APPCODE {}", appcode_str);

    // 3. 发送 HTTP 请求
    let deadline = sp_io::offchain::timestamp()
        .add(Duration::from_millis(API_TIMEOUT_MS));

    let request = http::Request::post(&url, vec![body.as_bytes().to_vec()])
        .add_header("Authorization", &auth_header)
        .add_header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
        .deadline(deadline);

    let pending = request.send().map_err(|e| {
        log::error!(target: "almanac-ocw", "Request send error: {:?}", e);
        "Failed to send request"
    })?;

    // 4. 等待响应
    let response = pending
        .try_wait(deadline)
        .map_err(|_| "Request timeout")?
        .map_err(|e| {
            log::error!(target: "almanac-ocw", "Request error: {:?}", e);
            "Request failed"
        })?;

    // 5. 检查响应状态
    if response.code != 200 {
        log::error!(
            target: "almanac-ocw",
            "API returned status: {}",
            response.code
        );
        return Err("API request failed");
    }

    // 6. 读取响应体
    let body = response.body().collect::<Vec<u8>>();
    let json_str = sp_std::str::from_utf8(&body)
        .map_err(|_| "Invalid response UTF-8")?;

    log::debug!(
        target: "almanac-ocw",
        "API response (first 200 chars): {}",
        &json_str[..json_str.len().min(200)]
    );

    // 7. 解析 JSON 响应
    parse_api_response(json_str, year, month, day)
}

/// 解析 API 响应 JSON
///
/// 阿里云黄历 API 响应格式示例:
/// ```json
/// {
///   "code": 200,
///   "data": {
///     "lunar": "甲辰年 腊月 十六",
///     "lunarYear": "甲辰",
///     "lunarMonth": "腊月",
///     "lunarDay": "十六",
///     "suit": "嫁娶 祭祀 祈福 ...",
///     "avoid": "动土 破土 ...",
///     "animalsYear": "龙",
///     "term": "大寒",
///     ...
///   }
/// }
/// ```
fn parse_api_response(
    json_str: &str,
    year: u16,
    month: u8,
    day: u8,
) -> Result<AlmanacInfo, &'static str> {
    use lite_json::{parse_json, JsonValue};

    // 解析 JSON
    let json = parse_json(json_str).map_err(|_| "JSON parse error")?;

    // 提取 data 对象
    let data = match &json {
        JsonValue::Object(obj) => {
            obj.iter()
                .find(|(k, _)| k.iter().collect::<String>() == "data")
                .map(|(_, v)| v)
                .ok_or("Missing 'data' field")?
        }
        _ => return Err("Invalid JSON structure"),
    };

    // 从 data 中提取字段并构造 AlmanacInfo
    let info = AlmanacInfo {
        // 农历信息
        lunar_year: year,
        lunar_month: extract_lunar_month(data).unwrap_or(month),
        lunar_day: extract_lunar_day(data).unwrap_or(day),

        // 干支信息 (需要从 API 响应解析)
        year_tiangan: extract_tiangan(data, "lunarYear").unwrap_or(0),
        year_dizhi: extract_dizhi(data, "lunarYear").unwrap_or(0),
        month_tiangan: 0, // API 可能不返回
        month_dizhi: 0,
        day_tiangan: 0,
        day_dizhi: 0,
        hour_tiangan: 0,
        hour_dizhi: 0,

        // 其他属性
        zodiac: extract_zodiac(data).unwrap_or(0),
        conflict_zodiac: 0,
        sha_direction: 0,
        wuxing: 0,
        jianchu: 0,
        constellation: 0,

        // 宜忌
        suitable: extract_suitable_bits(data),
        avoid: extract_avoid_bits(data),

        // 节气和节日
        solar_term: extract_solar_term(data).unwrap_or(0),
        festivals: 0,
        fortune_level: 2, // 默认平

        // 元数据
        updated_at: sp_io::offchain::timestamp().unix_millis() / 1000,
        source: 0, // OCW API
    };

    Ok(info)
}

// ============================================================================
// JSON 解析辅助函数
// ============================================================================

/// 提取农历月份
fn extract_lunar_month(data: &lite_json::JsonValue) -> Option<u8> {
    let month_str = extract_string_field(data, "lunarMonth")?;

    // 将中文月份转换为数字
    match month_str.as_str() {
        "正月" => Some(1),
        "二月" => Some(2),
        "三月" => Some(3),
        "四月" => Some(4),
        "五月" => Some(5),
        "六月" => Some(6),
        "七月" => Some(7),
        "八月" => Some(8),
        "九月" => Some(9),
        "十月" => Some(10),
        "冬月" | "十一月" => Some(11),
        "腊月" | "十二月" => Some(12),
        _ => None,
    }
}

/// 提取农历日期
fn extract_lunar_day(data: &lite_json::JsonValue) -> Option<u8> {
    let day_str = extract_string_field(data, "lunarDay")?;

    // 将中文日期转换为数字
    let day_map = [
        ("初一", 1), ("初二", 2), ("初三", 3), ("初四", 4), ("初五", 5),
        ("初六", 6), ("初七", 7), ("初八", 8), ("初九", 9), ("初十", 10),
        ("十一", 11), ("十二", 12), ("十三", 13), ("十四", 14), ("十五", 15),
        ("十六", 16), ("十七", 17), ("十八", 18), ("十九", 19), ("二十", 20),
        ("廿一", 21), ("廿二", 22), ("廿三", 23), ("廿四", 24), ("廿五", 25),
        ("廿六", 26), ("廿七", 27), ("廿八", 28), ("廿九", 29), ("三十", 30),
    ];

    for (name, num) in day_map.iter() {
        if day_str.contains(name) {
            return Some(*num);
        }
    }

    None
}

/// 从干支字符串提取天干
/// 天干：甲(0)乙(1)丙(2)丁(3)戊(4)己(5)庚(6)辛(7)壬(8)癸(9)
fn extract_tiangan(data: &lite_json::JsonValue, field: &str) -> Option<u8> {
    let ganzhi_str = extract_string_field(data, field)?;
    
    // 天干映射表
    let tiangan_map = [
        ("甲", 0), ("乙", 1), ("丙", 2), ("丁", 3), ("戊", 4),
        ("己", 5), ("庚", 6), ("辛", 7), ("壬", 8), ("癸", 9),
    ];
    
    // 干支字符串的第一个字符是天干
    for (name, num) in tiangan_map.iter() {
        if ganzhi_str.starts_with(name) {
            return Some(*num);
        }
    }
    
    None
}

/// 从干支字符串提取地支
/// 地支：子(0)丑(1)寅(2)卯(3)辰(4)巳(5)午(6)未(7)申(8)酉(9)戌(10)亥(11)
fn extract_dizhi(data: &lite_json::JsonValue, field: &str) -> Option<u8> {
    let ganzhi_str = extract_string_field(data, field)?;
    
    // 地支映射表
    let dizhi_map = [
        ("子", 0), ("丑", 1), ("寅", 2), ("卯", 3), ("辰", 4), ("巳", 5),
        ("午", 6), ("未", 7), ("申", 8), ("酉", 9), ("戌", 10), ("亥", 11),
    ];
    
    // 干支字符串的第二个字符是地支
    for (name, num) in dizhi_map.iter() {
        if ganzhi_str.contains(name) {
            return Some(*num);
        }
    }
    
    None
}

/// 提取生肖
fn extract_zodiac(data: &lite_json::JsonValue) -> Option<u8> {
    let zodiac_str = extract_string_field(data, "animalsYear")?;

    let zodiac_map = [
        ("鼠", 0), ("牛", 1), ("虎", 2), ("兔", 3), ("龙", 4), ("蛇", 5),
        ("马", 6), ("羊", 7), ("猴", 8), ("鸡", 9), ("狗", 10), ("猪", 11),
    ];

    for (name, num) in zodiac_map.iter() {
        if zodiac_str.contains(name) {
            return Some(*num);
        }
    }

    None
}

/// 提取节气
fn extract_solar_term(data: &lite_json::JsonValue) -> Option<u8> {
    let term_str = extract_string_field(data, "term")?;

    if term_str.is_empty() {
        return Some(0); // 无节气
    }

    let term_map = [
        ("立春", 1), ("雨水", 2), ("惊蛰", 3), ("春分", 4), ("清明", 5), ("谷雨", 6),
        ("立夏", 7), ("小满", 8), ("芒种", 9), ("夏至", 10), ("小暑", 11), ("大暑", 12),
        ("立秋", 13), ("处暑", 14), ("白露", 15), ("秋分", 16), ("寒露", 17), ("霜降", 18),
        ("立冬", 19), ("小雪", 20), ("大雪", 21), ("冬至", 22), ("小寒", 23), ("大寒", 24),
    ];

    for (name, num) in term_map.iter() {
        if term_str.contains(name) {
            return Some(*num);
        }
    }

    Some(0)
}

/// 提取宜事项 bit 标记
fn extract_suitable_bits(data: &lite_json::JsonValue) -> u64 {
    let suit_str = match extract_string_field(data, "suit") {
        Some(s) => s,
        None => return 0,
    };

    parse_items_to_bits(&suit_str)
}

/// 提取忌事项 bit 标记
fn extract_avoid_bits(data: &lite_json::JsonValue) -> u64 {
    let avoid_str = match extract_string_field(data, "avoid") {
        Some(s) => s,
        None => return 0,
    };

    parse_items_to_bits(&avoid_str)
}

/// 将事项字符串解析为 bit 标记
fn parse_items_to_bits(items_str: &str) -> u64 {
    let mut bits: u64 = 0;

    let item_map = [
        ("嫁娶", SuitableItem::Marriage),
        ("纳采", SuitableItem::Betrothal),
        ("祭祀", SuitableItem::Sacrifice),
        ("祈福", SuitableItem::Prayer),
        ("出行", SuitableItem::Travel),
        ("动土", SuitableItem::Groundbreaking),
        ("破土", SuitableItem::Excavation),
        ("安葬", SuitableItem::Burial),
        ("开市", SuitableItem::OpenBusiness),
        ("开业", SuitableItem::OpenBusiness),
        ("交易", SuitableItem::Trading),
        ("立券", SuitableItem::Contract),
        ("签约", SuitableItem::Contract),
        ("移徙", SuitableItem::Moving),
        ("搬家", SuitableItem::Moving),
        ("修造", SuitableItem::Renovation),
        ("装修", SuitableItem::Renovation),
        ("栽种", SuitableItem::Planting),
        ("纳财", SuitableItem::ReceiveMoney),
        ("开光", SuitableItem::Consecration),
        ("安床", SuitableItem::PlaceBed),
        ("入宅", SuitableItem::EnterHouse),
        ("安门", SuitableItem::InstallDoor),
        ("求嗣", SuitableItem::PrayForChildren),
        ("解除", SuitableItem::Remove),
        ("求医", SuitableItem::SeekMedical),
        ("词讼", SuitableItem::Lawsuit),
        ("沐浴", SuitableItem::Bathing),
        ("理发", SuitableItem::Haircut),
        ("扫舍", SuitableItem::Cleaning),
        ("会友", SuitableItem::MeetFriends),
        ("上梁", SuitableItem::RaiseBeam),
        ("竖柱", SuitableItem::ErectPillar),
        ("纳畜", SuitableItem::RaiseLivestock),
        ("伐木", SuitableItem::Logging),
        ("作灶", SuitableItem::BuildStove),
    ];

    for (name, item) in item_map.iter() {
        if items_str.contains(name) {
            bits |= 1u64 << (*item as u8);
        }
    }

    bits
}

/// 从 JSON 对象中提取字符串字段
fn extract_string_field(data: &lite_json::JsonValue, field: &str) -> Option<String> {
    use lite_json::JsonValue;

    match data {
        JsonValue::Object(obj) => {
            for (key, value) in obj.iter() {
                let key_str: String = key.iter().collect();
                if key_str == field {
                    if let JsonValue::String(chars) = value {
                        return Some(chars.iter().collect());
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// 获取今天的日期 (UTC)
fn get_today_date() -> DateKey {
    let timestamp = sp_io::offchain::timestamp().unix_millis() / 1000;

    // 简单的时间戳转日期 (不考虑时区)
    let days_since_epoch = timestamp / 86400;

    // 从 1970-01-01 开始计算
    let mut year: u16 = 1970;
    let mut remaining_days = days_since_epoch as i64;

    // 计算年份
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    // 计算月份和日期
    let days_in_months = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month: u8 = 1;
    for days in days_in_months.iter() {
        if remaining_days < *days as i64 {
            break;
        }
        remaining_days -= *days as i64;
        month += 1;
    }

    let day = (remaining_days + 1) as u8;

    (year, month, day)
}

/// 判断是否闰年
fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_items_to_bits() {
        let items = "嫁娶 祭祀 出行 开市";
        let bits = parse_items_to_bits(items);

        assert!(bits & (1 << SuitableItem::Marriage as u8) != 0);
        assert!(bits & (1 << SuitableItem::Sacrifice as u8) != 0);
        assert!(bits & (1 << SuitableItem::Travel as u8) != 0);
        assert!(bits & (1 << SuitableItem::OpenBusiness as u8) != 0);
        assert!(bits & (1 << SuitableItem::Burial as u8) == 0);
    }

    #[test]
    fn test_extract_lunar_month() {
        // 这需要构造 JsonValue，暂时跳过
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
    }
}
