//! 梅花易数核心算法模块
//!
//! 本模块实现梅花易数排盘的核心计算逻辑，包括：
//! - 卦数计算（上卦、下卦、动爻）
//! - 农历时间起卦、公历时间起卦、双数起卦、单数起卦、随机起卦、链摇起卦
//! - 变卦计算
//! - 互卦计算
//! - 错卦、综卦计算
//! - 体用判断
//! - 吉凶判断

use crate::lunar::LunarDate;
use crate::types::*;

#[cfg(not(feature = "std"))]
use alloc::format;

/// 计算卦数（处理余数为0的情况）
///
/// 梅花易数规则：除8取余，余数为0时按8计
///
/// # 参数
/// - `n`: 待计算的数值
///
/// # 返回
/// - 卦数（1-8）
#[inline]
pub fn calc_gua_num(n: u32) -> u8 {
    let r = (n % 8) as u8;
    if r == 0 {
        8
    } else {
        r
    }
}

/// 计算动爻数（处理余数为0的情况）
///
/// 梅花易数规则：除6取余，余数为0时按6计
///
/// # 参数
/// - `n`: 待计算的数值
///
/// # 返回
/// - 动爻数（1-6）
#[inline]
pub fn calc_dong_yao(n: u32) -> u8 {
    let r = (n % 6) as u8;
    if r == 0 {
        6
    } else {
        r
    }
}

/// 农历时间起卦算法
///
/// 使用农历年月日时计算上卦、下卦、动爻
///
/// # 算法
/// - 上卦数 = (年支数 + 月数 + 日数) % 8
/// - 下卦数 = (年支数 + 月数 + 日数 + 时支数) % 8
/// - 动爻数 = (年支数 + 月数 + 日数 + 时支数) % 6
///
/// # 参数
/// - `lunar`: 农历日期
///
/// # 返回
/// - (上卦数, 下卦数, 动爻数)
pub fn divine_by_datetime(lunar: &LunarDate) -> (u8, u8, u8) {
    let year_num = lunar.year_num() as u32;
    let month_num = lunar.month_num() as u32;
    let day_num = lunar.day_num() as u32;
    let hour_num = lunar.hour_num() as u32;

    // 上卦数：(年数+月数+日数) % 8
    let shang_gua_num = calc_gua_num(year_num + month_num + day_num);

    // 下卦数：(年数+月数+日数+时数) % 8
    let xia_gua_num = calc_gua_num(year_num + month_num + day_num + hour_num);

    // 动爻数：(年数+月数+日数+时数) % 6
    let dong_yao = calc_dong_yao(year_num + month_num + day_num + hour_num);

    (shang_gua_num, xia_gua_num, dong_yao)
}

/// 公历时间起卦算法
///
/// 使用公历年月日时计算上卦、下卦、动爻（现代简化方式）
///
/// # 算法
/// - 上卦数 = (年份后两位 + 月 + 日) % 8
/// - 下卦数 = (年份后两位 + 月 + 日 + 小时) % 8
/// - 动爻数 = (年份后两位 + 月 + 日 + 小时) % 6
///
/// # 参数
/// - `year`: 公历年份（如 2024）
/// - `month`: 公历月份（1-12）
/// - `day`: 公历日期（1-31）
/// - `hour`: 24小时制小时（0-23）
///
/// # 返回
/// - (上卦数, 下卦数, 动爻数)
///
/// # 示例
/// ```ignore
/// // 2024年12月24日 14时
/// let (shang, xia, dong) = divine_by_gregorian_datetime(2024, 12, 24, 14);
/// // 年份后两位=24, 上卦=(24+12+24)%8=60%8=4(震)
/// // 下卦=(24+12+24+14)%8=74%8=2(兑)
/// // 动爻=(24+12+24+14)%6=74%6=2
/// ```
pub fn divine_by_gregorian_datetime(year: u32, month: u8, day: u8, hour: u8) -> (u8, u8, u8) {
    // 年份取后两位
    let year_num = (year % 100) as u32;
    let month_num = month as u32;
    let day_num = day as u32;
    let hour_num = hour as u32;

    // 上卦数：(年份后两位+月+日) % 8
    let shang_gua_num = calc_gua_num(year_num + month_num + day_num);

    // 下卦数：(年份后两位+月+日+小时) % 8
    let xia_gua_num = calc_gua_num(year_num + month_num + day_num + hour_num);

    // 动爻数：(年份后两位+月+日+小时) % 6
    let dong_yao = calc_dong_yao(year_num + month_num + day_num + hour_num);

    (shang_gua_num, xia_gua_num, dong_yao)
}

/// 从 Unix 时间戳提取公历日期时间
///
/// # 参数
/// - `timestamp`: Unix 时间戳（秒）
///
/// # 返回
/// - (年, 月, 日, 小时)
pub fn timestamp_to_gregorian(timestamp: u64) -> (u32, u8, u8, u8) {
    // UTC+8 时区偏移（秒）
    const TIMEZONE_OFFSET: u64 = 8 * 3600;
    let local_timestamp = timestamp + TIMEZONE_OFFSET;

    // 从 Unix 时间戳计算日期
    // Unix 纪元：1970-01-01 00:00:00 UTC
    let days_since_epoch = (local_timestamp / 86400) as i64;
    let seconds_in_day = (local_timestamp % 86400) as u32;
    let hour = (seconds_in_day / 3600) as u8;

    // 使用简化的日期计算（考虑闰年）
    let (year, month, day) = days_to_ymd(days_since_epoch);

    (year as u32, month, day, hour)
}

/// 将天数转换为年月日
///
/// # 参数
/// - `days`: 从 1970-01-01 起的天数
///
/// # 返回
/// - (年, 月, 日)
fn days_to_ymd(days: i64) -> (i32, u8, u8) {
    // 调整到 2000-03-01 为基准（方便处理闰年）
    // 从 1970-01-01 到 2000-03-01 共 11017 天
    let days = days + 719468; // 调整到公元1年3月1日

    let era = if days >= 0 { days / 146097 } else { (days - 146096) / 146097 };
    let doe = (days - era * 146097) as u32; // 400年周期内的天数
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    (y, m as u8, d as u8)
}

/// 双数起卦算法
///
/// 使用两个数字计算上卦、下卦，配合时辰计算动爻
///
/// # 算法
/// - 上卦数 = num1 % 8
/// - 下卦数 = num2 % 8
/// - 动爻数 = (num1 + num2 + 时支数) % 6
///
/// # 参数
/// - `num1`: 第一个数字（用于上卦）
/// - `num2`: 第二个数字（用于下卦）
/// - `hour_zhi_num`: 时辰地支数（1-12）
///
/// # 返回
/// - (上卦数, 下卦数, 动爻数)
pub fn divine_by_numbers(num1: u16, num2: u16, hour_zhi_num: u8) -> (u8, u8, u8) {
    // 上卦数
    let shang_gua_num = calc_gua_num(num1 as u32);

    // 下卦数
    let xia_gua_num = calc_gua_num(num2 as u32);

    // 动爻数：(两数之和 + 时辰数) % 6
    let dong_yao = calc_dong_yao(num1 as u32 + num2 as u32 + hour_zhi_num as u32);

    (shang_gua_num, xia_gua_num, dong_yao)
}

/// 单数起卦算法
///
/// 将一个多位数字拆分为前后两半，分别计算上卦和下卦
///
/// # 算法
/// - 将数字拆分为前半段和后半段（奇数位时后半多一位）
/// - 上卦数 = 前半段各位数字之和 % 8
/// - 下卦数 = 后半段各位数字之和 % 8
/// - 动爻数 = (前半 + 后半 + 时支数) % 6
///
/// # 示例
/// - 输入 38271（5位）：前半 3+8=11，后半 2+7+1=10
/// - 上卦 = 11 % 8 = 3（离），下卦 = 10 % 8 = 2（兑）
/// - 动爻 = (11 + 10 + 时辰数) % 6
///
/// # 参数
/// - `number`: 多位数字（至少2位）
/// - `hour_zhi_num`: 时辰地支数（1-12）
///
/// # 返回
/// - (上卦数, 下卦数, 动爻数)
pub fn divine_by_single_number(number: u32, hour_zhi_num: u8) -> (u8, u8, u8) {
    // 将数字转换为各位数字（正序存储：高位在前）
    let mut digits: [u8; 10] = [0; 10]; // 最多支持10位数
    let digit_count: u8;
    let mut n = number;

    // 先计算位数
    if n == 0 {
        digits[0] = 0;
        digit_count = 1;
    } else {
        // 临时存储（逆序）
        let mut temp: [u8; 10] = [0; 10];
        let mut temp_count = 0u8;
        while n > 0 && temp_count < 10 {
            temp[temp_count as usize] = (n % 10) as u8;
            n /= 10;
            temp_count += 1;
        }
        digit_count = temp_count;

        // 逆转为正序（高位在前）
        for i in 0..digit_count {
            digits[i as usize] = temp[(digit_count - 1 - i) as usize];
        }
    }

    // 处理单位数情况：直接作为上下卦
    if digit_count == 1 {
        let single_digit = digits[0] as u32;
        let shang_gua_num = calc_gua_num(single_digit);
        let xia_gua_num = calc_gua_num(single_digit);
        let dong_yao = calc_dong_yao(single_digit * 2 + hour_zhi_num as u32);
        return (shang_gua_num, xia_gua_num, dong_yao);
    }

    // 计算前半段和后半段的数字之和
    // 前半段取 0 ~ (digit_count / 2) 位
    // 后半段取 (digit_count / 2) ~ digit_count 位
    let split_point = digit_count / 2;
    let mut first_half_sum: u32 = 0;
    let mut second_half_sum: u32 = 0;

    for i in 0..digit_count {
        if i < split_point {
            // 前半段（高位数字）
            first_half_sum += digits[i as usize] as u32;
        } else {
            // 后半段（低位数字）
            second_half_sum += digits[i as usize] as u32;
        }
    }

    // 上卦数：前半段数字之和 % 8
    let shang_gua_num = calc_gua_num(first_half_sum);

    // 下卦数：后半段数字之和 % 8
    let xia_gua_num = calc_gua_num(second_half_sum);

    // 动爻数：(前半 + 后半 + 时辰数) % 6
    let dong_yao = calc_dong_yao(first_half_sum + second_half_sum + hour_zhi_num as u32);

    (shang_gua_num, xia_gua_num, dong_yao)
}

/// 随机起卦算法
///
/// 从随机种子生成上卦、下卦、动爻
///
/// # 参数
/// - `random_seed`: 32字节的随机种子
///
/// # 返回
/// - (上卦数, 下卦数, 动爻数)
pub fn divine_by_random(random_seed: &[u8; 32]) -> (u8, u8, u8) {
    // 使用种子的前几个字节
    let shang_gua_num = calc_gua_num(random_seed[0] as u32);
    let xia_gua_num = calc_gua_num(random_seed[1] as u32);
    let dong_yao = calc_dong_yao(random_seed[2] as u32);

    (shang_gua_num, xia_gua_num, dong_yao)
}

/// 链摇起卦算法
///
/// 从6个爻的阴阳值构建上下卦，并根据时间戳计算动爻
///
/// # 参数
/// - `yaos`: 6个爻的值（0=阴爻，1=阳爻），按顺序为初爻到上爻
/// - `last_timestamp`: 最后一次摇卦的时间戳（毫秒），用于计算动爻
///
/// # 返回
/// - (上卦数, 下卦数, 动爻数)
///
/// # 爻位说明
/// ```text
/// yaos[5] - 上爻（第6爻）  ─┐
/// yaos[4] - 五爻（第5爻）   │ 上卦
/// yaos[3] - 四爻（第4爻）  ─┘
/// yaos[2] - 三爻（第3爻）  ─┐
/// yaos[1] - 二爻（第2爻）   │ 下卦
/// yaos[0] - 初爻（第1爻）  ─┘
/// ```
///
/// # 示例
/// ```ignore
/// // 6个爻：阳阳阳阴阴阴（下卦乾、上卦坤）
/// let yaos = [1, 1, 1, 0, 0, 0];
/// let (shang, xia, dong) = divine_by_shake(&yaos, 1735000000000);
/// // 上卦二进制 000 = 坤(8)
/// // 下卦二进制 111 = 乾(1)
/// ```
pub fn divine_by_shake(yaos: &[u8; 6], last_timestamp: u64) -> (u8, u8, u8) {
    // 下卦：yaos[0..3]，二进制从低位到高位
    // bit0 = 初爻, bit1 = 二爻, bit2 = 三爻
    let xia_binary = ((yaos[2] & 1) << 2) | ((yaos[1] & 1) << 1) | (yaos[0] & 1);

    // 上卦：yaos[3..6]，二进制从低位到高位
    // bit0 = 四爻, bit1 = 五爻, bit2 = 上爻
    let shang_binary = ((yaos[5] & 1) << 2) | ((yaos[4] & 1) << 1) | (yaos[3] & 1);

    // 从二进制转换为先天八卦数
    let shang_gua_num = Bagua::from_binary(shang_binary).number();
    let xia_gua_num = Bagua::from_binary(xia_binary).number();

    // 动爻：使用最后一次摇卦时间戳计算
    // 取毫秒部分增加随机性
    let dong_yao = calc_dong_yao((last_timestamp % 1000) as u32 + 1);

    (shang_gua_num, xia_gua_num, dong_yao)
}

/// 验证摇卦时间戳的合理性
///
/// 防止机器人批量摇卦，确保有真实的用户交互
///
/// # 参数
/// - `timestamps`: 6次摇卦的时间戳（毫秒）
/// - `min_interval`: 最小间隔（毫秒），默认 300ms
/// - `max_interval`: 最大间隔（毫秒），默认 30000ms（30秒）
///
/// # 返回
/// - `Ok(())`: 时间戳合理
/// - `Err(index)`: 第 index 次摇卦时间间隔不合理
pub fn validate_shake_timestamps(
    timestamps: &[u64; 6],
    min_interval: u64,
    max_interval: u64,
) -> Result<(), usize> {
    for i in 1..6 {
        // 检查时间戳递增
        if timestamps[i] <= timestamps[i - 1] {
            return Err(i);
        }

        let interval = timestamps[i] - timestamps[i - 1];

        // 检查间隔在合理范围内
        if interval < min_interval || interval > max_interval {
            return Err(i);
        }
    }
    Ok(())
}

/// 判断体用卦
///
/// 梅花易数规则：动爻在哪卦，哪卦为用，另一卦为体
/// - 动爻1-3在下卦，下卦为用，上卦为体
/// - 动爻4-6在上卦，上卦为用，下卦为体
///
/// # 参数
/// - `dong_yao`: 动爻位置（1-6）
///
/// # 返回
/// - true: 上卦为体
/// - false: 下卦为体
#[inline]
pub fn determine_ti_is_shang(dong_yao: u8) -> bool {
    // 动爻4-6在上卦，上卦为用，下卦为体
    dong_yao <= 3
}

/// 计算变卦
///
/// 变卦规则：动爻阴阳互变
///
/// # 参数
/// - `shang_gua`: 上卦
/// - `xia_gua`: 下卦
/// - `dong_yao`: 动爻位置（1-6）
///
/// # 返回
/// - (变卦上卦, 变卦下卦)
pub fn calc_bian_gua(shang_gua: &SingleGua, xia_gua: &SingleGua, dong_yao: u8) -> (SingleGua, SingleGua) {
    // 组合6爻二进制：上卦占高3位（爻4-6），下卦占低3位（爻1-3）
    let full_binary = (shang_gua.binary() << 3) | xia_gua.binary();

    // 翻转动爻位
    // dong_yao: 1-6 对应 bit 0-5
    let bit_position = dong_yao - 1;
    let flipped = full_binary ^ (1 << bit_position);

    // 分离上下卦
    let new_shang_binary = (flipped >> 3) & 0b111;
    let new_xia_binary = flipped & 0b111;

    (
        SingleGua::from_binary(new_shang_binary),
        SingleGua::from_binary(new_xia_binary),
    )
}

/// 计算互卦
///
/// 互卦规则：
/// - 互卦上卦：取本卦第3、4、5爻
/// - 互卦下卦：取本卦第2、3、4爻
///
/// # 参数
/// - `shang_gua`: 上卦
/// - `xia_gua`: 下卦
///
/// # 返回
/// - (互卦上卦, 互卦下卦)
pub fn calc_hu_gua(shang_gua: &SingleGua, xia_gua: &SingleGua) -> (SingleGua, SingleGua) {
    // 组合6爻：bits 5-3 为上卦（爻6、5、4），bits 2-0 为下卦（爻3、2、1）
    let full_binary = (shang_gua.binary() << 3) | xia_gua.binary();

    // 互卦上卦：取本卦第5、4、3爻 (bits 4, 3, 2)
    let hu_shang = (full_binary >> 2) & 0b111;

    // 互卦下卦：取本卦第4、3、2爻 (bits 3, 2, 1)
    let hu_xia = (full_binary >> 1) & 0b111;

    (
        SingleGua::from_binary(hu_shang),
        SingleGua::from_binary(hu_xia),
    )
}

/// 计算错卦
///
/// 错卦规则：本卦所有爻阴阳互变（取反）
///
/// # 参数
/// - `shang_gua`: 上卦
/// - `xia_gua`: 下卦
///
/// # 返回
/// - (错卦上卦, 错卦下卦)
pub fn calc_cuo_gua(shang_gua: &SingleGua, xia_gua: &SingleGua) -> (SingleGua, SingleGua) {
    // 上下卦各自取反
    let cuo_shang = (!shang_gua.binary()) & 0b111;
    let cuo_xia = (!xia_gua.binary()) & 0b111;

    (
        SingleGua::from_binary(cuo_shang),
        SingleGua::from_binary(cuo_xia),
    )
}

/// 计算综卦
///
/// 综卦规则：本卦上下颠倒（180°旋转）
///
/// # 参数
/// - `shang_gua`: 上卦
/// - `xia_gua`: 下卦
///
/// # 返回
/// - (综卦上卦, 综卦下卦)
pub fn calc_zong_gua(shang_gua: &SingleGua, xia_gua: &SingleGua) -> (SingleGua, SingleGua) {
    // 组合6爻
    let full_binary = (shang_gua.binary() << 3) | xia_gua.binary();

    // 逆序6个bit
    let mut reversed = 0u8;
    for i in 0..6 {
        if (full_binary >> i) & 1 == 1 {
            reversed |= 1 << (5 - i);
        }
    }

    // 分离上下卦
    let zong_shang = (reversed >> 3) & 0b111;
    let zong_xia = reversed & 0b111;

    (
        SingleGua::from_binary(zong_shang),
        SingleGua::from_binary(zong_xia),
    )
}

/// 计算伏卦（飞伏神）
///
/// 伏卦规则：
/// - 上卦、下卦各自对应一个伏卦
/// - 八卦各有其对应的伏卦关系
///
/// 梅花易数中伏卦用于：
/// - 判断隐藏的五行因素
/// - 推算飞伏神煞
/// - 断卦时参考隐伏之象
///
/// # 参数
/// - `shang_gua`: 上卦
/// - `xia_gua`: 下卦
///
/// # 返回
/// - (伏卦上卦, 伏卦下卦)
pub fn calc_fu_gua(shang_gua: &SingleGua, xia_gua: &SingleGua) -> (SingleGua, SingleGua) {
    use crate::constants::get_fu_gua_num;

    // 分别获取上下卦的伏卦
    let fu_shang_num = get_fu_gua_num(shang_gua.number());
    let fu_xia_num = get_fu_gua_num(xia_gua.number());

    (
        SingleGua::from_num(fu_shang_num),
        SingleGua::from_num(fu_xia_num),
    )
}

/// 计算体用关系
///
/// # 参数
/// - `ti_gua`: 体卦
/// - `yong_gua`: 用卦
///
/// # 返回
/// - 体用关系枚举
pub fn calc_tiyong_relation(ti_gua: &SingleGua, yong_gua: &SingleGua) -> TiYongRelation {
    TiYongRelation::calculate(&ti_gua.wuxing(), &yong_gua.wuxing())
}

/// 综合吉凶判断
///
/// 根据本卦和变卦的体用关系综合判断吉凶
///
/// # 参数
/// - `ben_relation`: 本卦体用关系
/// - `bian_relation`: 变卦体用关系（可选）
///
/// # 返回
/// - 吉凶判断结果
pub fn calc_fortune(
    ben_relation: &TiYongRelation,
    bian_relation: Option<&TiYongRelation>,
) -> Fortune {
    Fortune::from_relations(ben_relation, bian_relation)
}

/// 计算卦气旺衰
///
/// 根据体卦五行和当前季节（农历月份）判断体卦的旺衰状态
///
/// # 参数
/// - `ti_gua`: 体卦
/// - `lunar_month`: 农历月份（1-12）
///
/// # 返回
/// - 体卦的旺衰状态
pub fn calc_wangshuai(ti_gua: &SingleGua, lunar_month: u8) -> WangShuai {
    let ti_wuxing = ti_gua.wuxing();
    let season = Season::from_lunar_month(lunar_month);
    WangShuai::calculate(&ti_wuxing, &season)
}

/// 计算应期推算结果
///
/// 梅花易数应期推算规则：
/// 1. 体卦旺相时：应期在生体之五行的卦数，或体用卦数之和
/// 2. 体卦休囚时：应期在体所生之五行的卦数，或体卦卦数
/// 3. 用卦克体时：应期在克用之五行的卦数
/// 4. 用卦生体时：应期较快，在用卦卦数
///
/// 应期数可对应：年、月、日、时
///
/// # 参数
/// - `ti_gua`: 体卦
/// - `yong_gua`: 用卦
/// - `lunar_month`: 农历月份（用于判断旺衰）
///
/// # 返回
/// - 应期推算结果
pub fn calc_yingqi(
    ti_gua: &SingleGua,
    yong_gua: &SingleGua,
    lunar_month: u8,
) -> YingQiResult {
    let ti_wuxing = ti_gua.wuxing();
    let yong_wuxing = yong_gua.wuxing();
    let season = Season::from_lunar_month(lunar_month);
    let ti_wangshuai = WangShuai::calculate(&ti_wuxing, &season);

    // 生体五行（喜神）
    let sheng_ti_wuxing = ti_wuxing.generated_by();
    // 克体五行（忌神）
    let ke_ti_wuxing = ti_wuxing.conquered_by();

    // 体用卦数
    let ti_gua_num = ti_gua.number();
    let yong_gua_num = yong_gua.number();

    // 计算应期数
    let (primary_num, secondary_nums) = calc_yingqi_nums(
        &ti_wuxing,
        &yong_wuxing,
        &ti_wangshuai,
        ti_gua_num,
        yong_gua_num,
    );

    // 生成分析文本
    let analysis = generate_yingqi_analysis(
        &ti_wuxing,
        &yong_wuxing,
        &ti_wangshuai,
        &sheng_ti_wuxing,
        ti_gua_num,
        yong_gua_num,
        primary_num,
    );

    YingQiResult {
        ti_wuxing,
        yong_wuxing,
        ti_wangshuai,
        sheng_ti_wuxing,
        ke_ti_wuxing,
        ti_gua_num,
        yong_gua_num,
        primary_num,
        secondary_nums,
        analysis,
    }
}

/// 计算应期数
///
/// 根据体卦旺衰和体用关系计算主要和次要应期数
fn calc_yingqi_nums(
    ti_wuxing: &WuXing,
    yong_wuxing: &WuXing,
    ti_wangshuai: &WangShuai,
    ti_gua_num: u8,
    yong_gua_num: u8,
) -> (u8, [u8; 2]) {
    let relation = TiYongRelation::calculate(ti_wuxing, yong_wuxing);

    // 主要应期数
    let primary_num = if ti_wangshuai.is_strong() {
        // 体卦旺相：应期在用卦数或体用之和
        if relation == TiYongRelation::YongShengTi {
            // 用生体：应期较快，取用卦数
            yong_gua_num
        } else {
            // 其他：取体用卦数之和
            let sum = ti_gua_num as u16 + yong_gua_num as u16;
            if sum > 12 { (sum % 12) as u8 } else { sum as u8 }
        }
    } else {
        // 体卦休囚死：应期较慢
        if relation == TiYongRelation::YongKeTi {
            // 用克体：需等克用五行出现
            let ke_yong = yong_wuxing.conquered_by();
            let (num1, _) = ke_yong.gua_numbers();
            num1
        } else {
            // 其他：取体卦数
            ti_gua_num
        }
    };

    // 次要应期数（基于生体五行的卦数）
    let sheng_ti = ti_wuxing.generated_by();
    let (sec1, sec2_opt) = sheng_ti.gua_numbers();
    let secondary_nums = [sec1, sec2_opt.unwrap_or(sec1)];

    (primary_num, secondary_nums)
}

/// 生成应期分析文本
fn generate_yingqi_analysis(
    ti_wuxing: &WuXing,
    yong_wuxing: &WuXing,
    ti_wangshuai: &WangShuai,
    sheng_ti_wuxing: &WuXing,
    ti_gua_num: u8,
    yong_gua_num: u8,
    primary_num: u8,
) -> frame_support::BoundedVec<u8, frame_support::pallet_prelude::ConstU32<512>> {
    use crate::constants::WUXING_NAMES;

    let ti_name = WUXING_NAMES[*ti_wuxing as usize];
    let _yong_name = WUXING_NAMES[*yong_wuxing as usize];
    let sheng_ti_name = WUXING_NAMES[*sheng_ti_wuxing as usize];

    let wangshuai_str = match ti_wangshuai {
        WangShuai::Wang => "旺",
        WangShuai::Xiang => "相",
        WangShuai::Xiu => "休",
        WangShuai::Qiu => "囚",
        WangShuai::Si => "死",
    };

    let relation = TiYongRelation::calculate(ti_wuxing, yong_wuxing);
    let relation_str = match relation {
        TiYongRelation::BiHe => "比和",
        TiYongRelation::YongShengTi => "用生体",
        TiYongRelation::TiShengYong => "体生用",
        TiYongRelation::YongKeTi => "用克体",
        TiYongRelation::TiKeYong => "体克用",
    };

    // 构建分析文本
    let analysis_text = if ti_wangshuai.is_strong() {
        format!(
            "体卦{}{}，{}。喜神为{}。应期数：{}（可应年、月、日、时）。体卦数{}，用卦数{}。",
            ti_name, wangshuai_str, relation_str, sheng_ti_name,
            primary_num, ti_gua_num, yong_gua_num
        )
    } else {
        format!(
            "体卦{}{}，力弱，{}。喜神为{}生体。应期数：{}（须待时机成熟）。体卦数{}，用卦数{}。",
            ti_name, wangshuai_str, relation_str, sheng_ti_name,
            primary_num, ti_gua_num, yong_gua_num
        )
    };

    frame_support::BoundedVec::try_from(analysis_text.into_bytes())
        .unwrap_or_default()
}

/// 完整排盘计算
///
/// 根据上卦数、下卦数、动爻数，计算完整的卦象信息
///
/// # 参数
/// - `shang_num`: 上卦数（1-8）
/// - `xia_num`: 下卦数（1-8）
/// - `dong_yao`: 动爻（1-6）
///
/// # 返回
/// - (本卦上卦, 本卦下卦, 变卦上卦, 变卦下卦, 互卦上卦, 互卦下卦, 体用关系, 吉凶)
pub fn full_divination(
    shang_num: u8,
    xia_num: u8,
    dong_yao: u8,
) -> (
    SingleGua,
    SingleGua,
    SingleGua,
    SingleGua,
    SingleGua,
    SingleGua,
    bool,
    TiYongRelation,
    TiYongRelation,
    Fortune,
) {
    // 创建本卦
    let shang_gua = SingleGua::from_num(shang_num);
    let xia_gua = SingleGua::from_num(xia_num);

    // 计算变卦
    let (bian_shang, bian_xia) = calc_bian_gua(&shang_gua, &xia_gua, dong_yao);

    // 计算互卦
    let (hu_shang, hu_xia) = calc_hu_gua(&shang_gua, &xia_gua);

    // 判断体用
    let ti_is_shang = determine_ti_is_shang(dong_yao);

    // 获取体卦和用卦
    let (ti_gua, yong_gua) = if ti_is_shang {
        (&shang_gua, &xia_gua)
    } else {
        (&xia_gua, &shang_gua)
    };

    // 计算本卦体用关系
    let ben_relation = calc_tiyong_relation(ti_gua, yong_gua);

    // 计算变卦体用关系（体卦位置不变）
    let (bian_ti, bian_yong) = if ti_is_shang {
        (&bian_shang, &bian_xia)
    } else {
        (&bian_xia, &bian_shang)
    };
    let bian_relation = calc_tiyong_relation(bian_ti, bian_yong);

    // 综合吉凶判断
    let fortune = calc_fortune(&ben_relation, Some(&bian_relation));

    (
        shang_gua,
        xia_gua,
        bian_shang,
        bian_xia,
        hu_shang,
        hu_xia,
        ti_is_shang,
        ben_relation,
        bian_relation,
        fortune,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_gua_num() {
        assert_eq!(calc_gua_num(1), 1);
        assert_eq!(calc_gua_num(8), 8);
        assert_eq!(calc_gua_num(9), 1);
        assert_eq!(calc_gua_num(16), 8);
        assert_eq!(calc_gua_num(0), 8);
    }

    #[test]
    fn test_calc_dong_yao() {
        assert_eq!(calc_dong_yao(1), 1);
        assert_eq!(calc_dong_yao(6), 6);
        assert_eq!(calc_dong_yao(7), 1);
        assert_eq!(calc_dong_yao(12), 6);
        assert_eq!(calc_dong_yao(0), 6);
    }

    #[test]
    fn test_determine_ti_is_shang() {
        // 动爻1-3，上卦为体
        assert!(determine_ti_is_shang(1));
        assert!(determine_ti_is_shang(2));
        assert!(determine_ti_is_shang(3));
        // 动爻4-6，下卦为体
        assert!(!determine_ti_is_shang(4));
        assert!(!determine_ti_is_shang(5));
        assert!(!determine_ti_is_shang(6));
    }

    #[test]
    fn test_calc_bian_gua() {
        // 乾卦（111）动爻1，翻转初爻(bit0)：111 -> 110 = 巽
        let shang = SingleGua::from_num(1); // 乾 (111)
        let xia = SingleGua::from_num(1);   // 乾 (111)
        let (bian_shang, bian_xia) = calc_bian_gua(&shang, &xia, 1);

        // 六爻为 111_111，翻转bit0变为 111_110
        // 上卦 111 = 乾，下卦 110 = 巽
        assert_eq!(bian_shang.bagua, Bagua::Qian); // 上卦不变
        assert_eq!(bian_xia.bagua, Bagua::Xun);    // 下卦变为巽
    }

    #[test]
    fn test_calc_hu_gua() {
        // 乾为天（䷀）：111 111
        // 互卦上卦取345爻：111
        // 互卦下卦取234爻：111
        // 所以乾为天的互卦还是乾为天
        let shang = SingleGua::from_num(1); // 乾
        let xia = SingleGua::from_num(1);   // 乾
        let (hu_shang, hu_xia) = calc_hu_gua(&shang, &xia);

        assert_eq!(hu_shang.bagua, Bagua::Qian);
        assert_eq!(hu_xia.bagua, Bagua::Qian);
    }

    #[test]
    fn test_calc_cuo_gua() {
        // 乾卦（111）的错卦是坤卦（000）
        let shang = SingleGua::from_num(1); // 乾
        let xia = SingleGua::from_num(1);   // 乾
        let (cuo_shang, cuo_xia) = calc_cuo_gua(&shang, &xia);

        assert_eq!(cuo_shang.bagua, Bagua::Kun);
        assert_eq!(cuo_xia.bagua, Bagua::Kun);
    }

    #[test]
    fn test_divine_by_datetime() {
        // 模拟农历日期：甲辰年冬月二十子时
        // 年支数=5（辰），月数=11，日数=20，时支数=1（子）
        let lunar = LunarDate {
            year: 2024,
            year_zhi_num: 5, // 辰
            month: 11,
            day: 20,
            hour_zhi_num: 1, // 子
            is_leap_month: false,
        };

        let (shang, xia, dong) = divine_by_datetime(&lunar);

        // 上卦：(5+11+20) % 8 = 36 % 8 = 4 → 震
        assert_eq!(shang, 4);
        // 下卦：(5+11+20+1) % 8 = 37 % 8 = 5 → 巽
        assert_eq!(xia, 5);
        // 动爻：(5+11+20+1) % 6 = 37 % 6 = 1
        assert_eq!(dong, 1);
    }

    #[test]
    fn test_full_divination() {
        // 测试完整排盘：上卦3（离），下卦4（震），动爻6
        let result = full_divination(3, 4, 6);

        // 本卦：火雷噬嗑
        assert_eq!(result.0.bagua, Bagua::Li);   // 上卦离
        assert_eq!(result.1.bagua, Bagua::Zhen); // 下卦震

        // 动爻6在上卦，上卦为用，下卦为体
        assert!(!result.6); // ti_is_shang = false

        // 验证变卦计算
        // 离（101）上卦动第6爻变为震（001）
        assert_eq!(result.2.bagua, Bagua::Zhen); // 变卦上卦
        assert_eq!(result.3.bagua, Bagua::Zhen); // 变卦下卦不变
    }

    #[test]
    fn test_divine_by_single_number() {
        // 测试单数起卦算法
        // 输入 38271：
        // 数字拆分：前半 3,8 = 11，后半 2,7,1 = 10
        // 上卦 = 11 % 8 = 3（离）
        // 下卦 = 10 % 8 = 2（兑）
        // 动爻 = (11 + 10 + 1) % 6 = 22 % 6 = 4
        let (shang, xia, dong) = divine_by_single_number(38271, 1);

        assert_eq!(shang, 3); // 离
        assert_eq!(xia, 2);   // 兑
        assert_eq!(dong, 4);
    }

    #[test]
    fn test_divine_by_single_number_two_digits() {
        // 测试两位数
        // 输入 36：前半 3，后半 6
        // 上卦 = 3 % 8 = 3（离）
        // 下卦 = 6 % 8 = 6（坎）
        // 动爻 = (3 + 6 + 1) % 6 = 10 % 6 = 4
        let (shang, xia, dong) = divine_by_single_number(36, 1);

        assert_eq!(shang, 3); // 离
        assert_eq!(xia, 6);   // 坎
        assert_eq!(dong, 4);
    }

    #[test]
    fn test_divine_by_single_number_four_digits() {
        // 测试四位数
        // 输入 1234：前半 1+2=3，后半 3+4=7
        // 上卦 = 3 % 8 = 3（离）
        // 下卦 = 7 % 8 = 7（艮）
        // 动爻 = (3 + 7 + 1) % 6 = 11 % 6 = 5
        let (shang, xia, dong) = divine_by_single_number(1234, 1);

        assert_eq!(shang, 3); // 离
        assert_eq!(xia, 7);   // 艮
        assert_eq!(dong, 5);
    }

    #[test]
    fn test_divine_by_single_number_single_digit() {
        // 测试单位数
        // 输入 5：上下卦都是5
        // 上卦 = 5 % 8 = 5（巽）
        // 下卦 = 5 % 8 = 5（巽）
        // 动爻 = (5 * 2 + 1) % 6 = 11 % 6 = 5
        let (shang, xia, dong) = divine_by_single_number(5, 1);

        assert_eq!(shang, 5); // 巽
        assert_eq!(xia, 5);   // 巽
        assert_eq!(dong, 5);
    }

    #[test]
    fn test_divine_by_single_number_remainder_zero() {
        // 测试余数为0的情况
        // 输入 88：前半 8，后半 8
        // 上卦 = 8 % 8 = 0 → 8（坤）
        // 下卦 = 8 % 8 = 0 → 8（坤）
        // 动爻 = (8 + 8 + 1) % 6 = 17 % 6 = 5
        let (shang, xia, dong) = divine_by_single_number(88, 1);

        assert_eq!(shang, 8); // 坤
        assert_eq!(xia, 8);   // 坤
        assert_eq!(dong, 5);
    }

    // ==================== P2 功能测试：卦气旺衰 ====================

    #[test]
    fn test_calc_wangshuai_spring() {
        // 春季（正月）：木旺
        // 震卦属木 -> 旺
        let zhen = SingleGua::from_num(4); // 震 = 木
        assert_eq!(calc_wangshuai(&zhen, 1), WangShuai::Wang);

        // 离卦属火 -> 相（木生火）
        let li = SingleGua::from_num(3); // 离 = 火
        assert_eq!(calc_wangshuai(&li, 1), WangShuai::Xiang);

        // 坎卦属水 -> 休（水生木）
        let kan = SingleGua::from_num(6); // 坎 = 水
        assert_eq!(calc_wangshuai(&kan, 1), WangShuai::Xiu);

        // 乾卦属金 -> 囚（金克木）
        let qian = SingleGua::from_num(1); // 乾 = 金
        assert_eq!(calc_wangshuai(&qian, 1), WangShuai::Qiu);

        // 坤卦属土 -> 死（木克土）
        let kun = SingleGua::from_num(8); // 坤 = 土
        assert_eq!(calc_wangshuai(&kun, 1), WangShuai::Si);
    }

    #[test]
    fn test_calc_wangshuai_summer() {
        // 夏季（四月）：火旺
        // 离卦属火 -> 旺
        let li = SingleGua::from_num(3); // 离 = 火
        assert_eq!(calc_wangshuai(&li, 4), WangShuai::Wang);

        // 坤卦属土 -> 相（火生土）
        let kun = SingleGua::from_num(8); // 坤 = 土
        assert_eq!(calc_wangshuai(&kun, 4), WangShuai::Xiang);

        // 震卦属木 -> 休（木生火）
        let zhen = SingleGua::from_num(4); // 震 = 木
        assert_eq!(calc_wangshuai(&zhen, 4), WangShuai::Xiu);

        // 坎卦属水 -> 囚（水克火）
        let kan = SingleGua::from_num(6); // 坎 = 水
        assert_eq!(calc_wangshuai(&kan, 4), WangShuai::Qiu);

        // 乾卦属金 -> 死（火克金）
        let qian = SingleGua::from_num(1); // 乾 = 金
        assert_eq!(calc_wangshuai(&qian, 4), WangShuai::Si);
    }

    #[test]
    fn test_calc_wangshuai_autumn() {
        // 秋季（七月）：金旺
        // 乾卦属金 -> 旺
        let qian = SingleGua::from_num(1); // 乾 = 金
        assert_eq!(calc_wangshuai(&qian, 7), WangShuai::Wang);

        // 坎卦属水 -> 相（金生水）
        let kan = SingleGua::from_num(6); // 坎 = 水
        assert_eq!(calc_wangshuai(&kan, 7), WangShuai::Xiang);
    }

    #[test]
    fn test_calc_wangshuai_winter() {
        // 冬季（十月）：水旺
        // 坎卦属水 -> 旺
        let kan = SingleGua::from_num(6); // 坎 = 水
        assert_eq!(calc_wangshuai(&kan, 10), WangShuai::Wang);

        // 震卦属木 -> 相（水生木）
        let zhen = SingleGua::from_num(4); // 震 = 木
        assert_eq!(calc_wangshuai(&zhen, 10), WangShuai::Xiang);
    }

    #[test]
    fn test_wangshuai_is_strong() {
        // 测试旺相为有力
        assert!(WangShuai::Wang.is_strong());
        assert!(WangShuai::Xiang.is_strong());
        assert!(!WangShuai::Xiu.is_strong());
        assert!(!WangShuai::Qiu.is_strong());
        assert!(!WangShuai::Si.is_strong());
    }

    // ==================== P2 功能测试：应期推算 ====================

    #[test]
    fn test_calc_yingqi_basic() {
        // 测试应期推算基本功能
        // 体卦：乾（金），用卦：离（火），春季
        let ti_gua = SingleGua::from_num(1);  // 乾 = 金
        let yong_gua = SingleGua::from_num(3); // 离 = 火

        let result = calc_yingqi(&ti_gua, &yong_gua, 1);

        // 验证基本信息
        assert_eq!(result.ti_wuxing, WuXing::Jin);
        assert_eq!(result.yong_wuxing, WuXing::Huo);
        assert_eq!(result.ti_gua_num, 1);
        assert_eq!(result.yong_gua_num, 3);

        // 春季金囚
        assert_eq!(result.ti_wangshuai, WangShuai::Qiu);

        // 生金的是土
        assert_eq!(result.sheng_ti_wuxing, WuXing::Tu);

        // 克金的是火
        assert_eq!(result.ke_ti_wuxing, WuXing::Huo);
    }

    #[test]
    fn test_calc_yingqi_strong_ti() {
        // 体卦旺相时的应期推算
        // 体卦：震（木），用卦：坎（水），春季（木旺）
        let ti_gua = SingleGua::from_num(4);  // 震 = 木
        let yong_gua = SingleGua::from_num(6); // 坎 = 水

        let result = calc_yingqi(&ti_gua, &yong_gua, 1);

        // 体卦春季木旺
        assert!(result.ti_wangshuai.is_strong());

        // 用生体（水生木），应期取用卦数
        assert_eq!(result.primary_num, 6); // 坎卦数
    }

    #[test]
    fn test_calc_yingqi_weak_ti_ke() {
        // 体卦休囚死且被克时的应期推算
        // 体卦：震（木），用卦：乾（金），秋季（金旺克木）
        let ti_gua = SingleGua::from_num(4);  // 震 = 木
        let yong_gua = SingleGua::from_num(1); // 乾 = 金

        let result = calc_yingqi(&ti_gua, &yong_gua, 7);

        // 体卦秋季木死（被金克）
        assert!(result.ti_wangshuai.is_weak());

        // 用克体，需等克用五行（火克金）出现
        // 火对应离卦(3)
        assert_eq!(result.primary_num, 3);
    }

    #[test]
    fn test_calc_yingqi_analysis_text() {
        // 验证分析文本生成
        let ti_gua = SingleGua::from_num(1);  // 乾 = 金
        let yong_gua = SingleGua::from_num(3); // 离 = 火

        let result = calc_yingqi(&ti_gua, &yong_gua, 1);

        // 分析文本应该包含关键信息
        let analysis_str = core::str::from_utf8(&result.analysis).unwrap_or("");
        assert!(analysis_str.contains("金"));  // 体卦五行
        assert!(analysis_str.contains("囚"));  // 旺衰状态
    }

    #[test]
    fn test_season_from_lunar_month() {
        // 测试农历月份到季节的转换
        assert_eq!(Season::from_lunar_month(1), Season::Spring);
        assert_eq!(Season::from_lunar_month(2), Season::Spring);
        assert_eq!(Season::from_lunar_month(3), Season::Spring);
        assert_eq!(Season::from_lunar_month(4), Season::Summer);
        assert_eq!(Season::from_lunar_month(5), Season::Summer);
        assert_eq!(Season::from_lunar_month(6), Season::Summer);
        assert_eq!(Season::from_lunar_month(7), Season::Autumn);
        assert_eq!(Season::from_lunar_month(8), Season::Autumn);
        assert_eq!(Season::from_lunar_month(9), Season::Autumn);
        assert_eq!(Season::from_lunar_month(10), Season::Winter);
        assert_eq!(Season::from_lunar_month(11), Season::Winter);
        assert_eq!(Season::from_lunar_month(12), Season::Winter);
    }

    #[test]
    fn test_wuxing_gua_numbers() {
        // 测试五行对应的卦数
        assert_eq!(WuXing::Jin.gua_numbers(), (1, Some(2)));  // 乾1、兑2
        assert_eq!(WuXing::Mu.gua_numbers(), (4, Some(5)));   // 震4、巽5
        assert_eq!(WuXing::Shui.gua_numbers(), (6, None));    // 坎6
        assert_eq!(WuXing::Huo.gua_numbers(), (3, None));     // 离3
        assert_eq!(WuXing::Tu.gua_numbers(), (7, Some(8)));   // 艮7、坤8
    }

    #[test]
    fn test_wuxing_relationships() {
        // 测试五行生克关系
        // 金生水
        assert_eq!(WuXing::Jin.generates_to(), WuXing::Shui);
        assert_eq!(WuXing::Shui.generated_by(), WuXing::Jin);

        // 金克木
        assert_eq!(WuXing::Jin.conquers_to(), WuXing::Mu);
        assert_eq!(WuXing::Mu.conquered_by(), WuXing::Jin);

        // 木生火
        assert_eq!(WuXing::Mu.generates_to(), WuXing::Huo);
        assert_eq!(WuXing::Huo.generated_by(), WuXing::Mu);

        // 水克火
        assert_eq!(WuXing::Shui.conquers_to(), WuXing::Huo);
        assert_eq!(WuXing::Huo.conquered_by(), WuXing::Shui);
    }

    #[test]
    fn test_calc_fu_gua() {
        // 乾卦的伏卦是巽卦
        let qian = SingleGua::from_num(1);
        let (fu_shang, fu_xia) = calc_fu_gua(&qian, &qian);
        assert_eq!(fu_shang.bagua, Bagua::Xun);
        assert_eq!(fu_xia.bagua, Bagua::Xun);

        // 坤卦的伏卦是乾卦
        let kun = SingleGua::from_num(8);
        let (fu_shang, fu_xia) = calc_fu_gua(&kun, &kun);
        assert_eq!(fu_shang.bagua, Bagua::Qian);
        assert_eq!(fu_xia.bagua, Bagua::Qian);

        // 离坎互为伏卦
        let li = SingleGua::from_num(3);
        let kan = SingleGua::from_num(6);
        let (fu_shang, _) = calc_fu_gua(&li, &kan);
        assert_eq!(fu_shang.bagua, Bagua::Kan);  // 离的伏卦是坎
    }

    // ==================== 公历起卦测试 ====================

    #[test]
    fn test_divine_by_gregorian_datetime() {
        // 2024年12月24日 14时
        // 年份后两位=24, 月=12, 日=24, 时=14
        // 上卦 = (24+12+24) % 8 = 60 % 8 = 4 → 震
        // 下卦 = (24+12+24+14) % 8 = 74 % 8 = 2 → 兑
        // 动爻 = (24+12+24+14) % 6 = 74 % 6 = 2
        let (shang, xia, dong) = divine_by_gregorian_datetime(2024, 12, 24, 14);

        assert_eq!(shang, 4); // 震
        assert_eq!(xia, 2);   // 兑
        assert_eq!(dong, 2);
    }

    #[test]
    fn test_divine_by_gregorian_datetime_zero_remainder() {
        // 测试余数为0的情况
        // 设计一个例子使余数为0
        // 年=2000, 月=4, 日=4, 时=0
        // 上卦 = (0+4+4) % 8 = 8 % 8 = 0 → 8(坤)
        // 下卦 = (0+4+4+0) % 8 = 8 % 8 = 0 → 8(坤)
        // 动爻 = (0+4+4+0) % 6 = 8 % 6 = 2
        let (shang, xia, dong) = divine_by_gregorian_datetime(2000, 4, 4, 0);

        assert_eq!(shang, 8); // 坤
        assert_eq!(xia, 8);   // 坤
        assert_eq!(dong, 2);
    }

    #[test]
    fn test_timestamp_to_gregorian() {
        // 2024-12-24 14:00:00 UTC 的时间戳
        // Unix timestamp: 1735048800
        let timestamp = 1735048800u64;
        let (year, month, day, hour) = timestamp_to_gregorian(timestamp);

        assert_eq!(year, 2024);
        assert_eq!(month, 12);
        assert_eq!(day, 24);
        // UTC+8 时区，14:00 UTC = 22:00 北京时间
        assert_eq!(hour, 22);
    }

    #[test]
    fn test_timestamp_to_gregorian_epoch() {
        // Unix 纪元：1970-01-01 00:00:00 UTC
        let timestamp = 0u64;
        let (year, month, day, hour) = timestamp_to_gregorian(timestamp);

        assert_eq!(year, 1970);
        assert_eq!(month, 1);
        assert_eq!(day, 1);
        assert_eq!(hour, 8); // UTC+8
    }

    // ==================== 链摇起卦测试 ====================

    #[test]
    fn test_divine_by_shake_qian_kun() {
        // 6个阳爻 [1,1,1,1,1,1] = 乾为天
        // 下卦 111 = 乾(1), 上卦 111 = 乾(1)
        let yaos = [1, 1, 1, 1, 1, 1];
        let (shang, xia, _dong) = divine_by_shake(&yaos, 1735000000000);

        assert_eq!(shang, 1); // 乾
        assert_eq!(xia, 1);   // 乾
    }

    #[test]
    fn test_divine_by_shake_kun() {
        // 6个阴爻 [0,0,0,0,0,0] = 坤为地
        // 下卦 000 = 坤(8), 上卦 000 = 坤(8)
        let yaos = [0, 0, 0, 0, 0, 0];
        let (shang, xia, _dong) = divine_by_shake(&yaos, 1735000000000);

        assert_eq!(shang, 8); // 坤
        assert_eq!(xia, 8);   // 坤
    }

    #[test]
    fn test_divine_by_shake_mixed() {
        // [1,1,1,0,0,0] = 下卦乾(111)，上卦坤(000) = 天地否
        let yaos = [1, 1, 1, 0, 0, 0];
        let (shang, xia, _dong) = divine_by_shake(&yaos, 1735000000000);

        assert_eq!(xia, 1);   // 乾
        assert_eq!(shang, 8); // 坤
    }

    #[test]
    fn test_divine_by_shake_kan_li() {
        // 坎卦 010，离卦 101
        // [0,1,0,1,0,1] = 下卦坎，上卦离 = 火水未济
        let yaos = [0, 1, 0, 1, 0, 1];
        let (shang, xia, _dong) = divine_by_shake(&yaos, 1735000000000);

        assert_eq!(xia, 6);   // 坎
        assert_eq!(shang, 3); // 离
    }

    #[test]
    fn test_divine_by_shake_dong_yao() {
        // 测试动爻计算
        // 时间戳最后三位决定动爻
        let yaos = [1, 1, 1, 1, 1, 1];

        // 时间戳 xxx001 -> (1+1) % 6 = 2
        let (_, _, dong1) = divine_by_shake(&yaos, 1735000000001);
        assert_eq!(dong1, 2);

        // 时间戳 xxx005 -> (5+1) % 6 = 0 -> 6
        let (_, _, dong2) = divine_by_shake(&yaos, 1735000000005);
        assert_eq!(dong2, 6);
    }

    #[test]
    fn test_validate_shake_timestamps_valid() {
        // 有效的时间戳序列
        let timestamps = [
            1735000000000,
            1735000000500,  // +500ms
            1735000001200,  // +700ms
            1735000002000,  // +800ms
            1735000003500,  // +1500ms
            1735000005000,  // +1500ms
        ];

        assert!(validate_shake_timestamps(&timestamps, 300, 30_000).is_ok());
    }

    #[test]
    fn test_validate_shake_timestamps_too_fast() {
        // 间隔太短（小于300ms）
        let timestamps = [
            1735000000000,
            1735000000100,  // +100ms，太快
            1735000001000,
            1735000002000,
            1735000003000,
            1735000004000,
        ];

        assert!(validate_shake_timestamps(&timestamps, 300, 30_000).is_err());
    }

    #[test]
    fn test_validate_shake_timestamps_too_slow() {
        // 间隔太长（超过30秒）
        let timestamps = [
            1735000000000,
            1735000035000,  // +35秒，太慢
            1735000036000,
            1735000037000,
            1735000038000,
            1735000039000,
        ];

        assert!(validate_shake_timestamps(&timestamps, 300, 30_000).is_err());
    }

    #[test]
    fn test_validate_shake_timestamps_not_increasing() {
        // 时间戳不递增
        let timestamps = [
            1735000000000,
            1735000001000,
            1735000000500,  // 回退了
            1735000002000,
            1735000003000,
            1735000004000,
        ];

        assert!(validate_shake_timestamps(&timestamps, 300, 30_000).is_err());
    }
}
