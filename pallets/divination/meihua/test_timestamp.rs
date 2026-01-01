// 临时测试文件：验证时间戳 1765609194 的农历转换结果

use crate::lunar::timestamp_to_lunar;

fn main() {
    // 时间戳: 1765609194
    // 公历: 2025-12-13 06:59:54 UTC
    // 北京时间: 2025-12-13 14:59:54 (UTC+8)

    let timestamp = 1765609194u64;

    match timestamp_to_lunar(timestamp) {
        Ok(lunar) => {
            println!("时间戳: {}", timestamp);
            println!("农历年份: {}", lunar.year);
            println!("农历月份: {}", lunar.month);
            println!("农历日: {}", lunar.day);
            println!("年地支数: {}", lunar.year_zhi_num);
            println!("时辰地支数: {}", lunar.hour_zhi_num);
            println!("是否闰月: {}", lunar.is_leap_month);
        },
        Err(e) => {
            println!("转换失败: {:?}", e);
        }
    }
}
