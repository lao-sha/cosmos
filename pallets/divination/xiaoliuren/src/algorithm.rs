//! # 小六壬排盘算法
//!
//! 本模块实现小六壬排盘的核心算法。
//!
//! ## 起课方法
//!
//! ### 1. 时间起课（传统方法）
//!
//! 按农历月日时起课：
//! - 月宫：从大安起正月，顺数至所求月份
//! - 日宫：从月宫起初一，顺数至所求日期
//! - 时宫：从日宫起子时，顺数至所求时辰
//!
//! ### 2. 数字起课（活数起课法）
//!
//! 取三个数字 x、y、z：
//! - 月宫 = (x - 1) % 6
//! - 日宫 = (x + y - 2) % 6
//! - 时宫 = (x + y + z - 3) % 6
//!
//! ### 3. 随机起课
//!
//! 使用链上随机数生成三个数字，然后按数字起课法计算。

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::ToString;

use crate::types::*;

// ============================================================================
// 时间起课算法
// ============================================================================

/// 时间起课
///
/// 按农历月日时起课，这是最传统的小六壬起课方法。
///
/// # 参数
/// - `lunar_month`: 农历月份（1-12）
/// - `lunar_day`: 农历日期（1-30）
/// - `shi_chen`: 时辰
///
/// # 算法
/// 1. 月宫：从大安起正月，顺数至所求月份
/// 2. 日宫：从月宫起初一，顺数至所求日期
/// 3. 时宫：从日宫起子时，顺数至所求时辰
pub fn divine_by_time(lunar_month: u8, lunar_day: u8, shi_chen: ShiChen) -> SanGong {
    // 月宫：从大安(0)起正月，顺数至所求月份
    // 正月为1，所以 (month - 1) % 6
    let yue_index = (lunar_month.saturating_sub(1)) % 6;
    let yue_gong = LiuGong::from_index(yue_index);

    // 日宫：从月宫起初一，顺数至所求日期
    // 初一对应0，所以 (yue_index + day - 1) % 6
    let ri_index = (yue_index + lunar_day.saturating_sub(1)) % 6;
    let ri_gong = LiuGong::from_index(ri_index);

    // 时宫：从日宫起子时(1)，顺数至所求时辰
    // 子时序号为1，所以 (ri_index + shi_chen_index - 1) % 6
    let shi_index = (ri_index + shi_chen.index().saturating_sub(1)) % 6;
    let shi_gong = LiuGong::from_index(shi_index);

    SanGong::new(yue_gong, ri_gong, shi_gong)
}

/// 从小时数计算时间起课
pub fn divine_by_time_with_hour(lunar_month: u8, lunar_day: u8, hour: u8) -> SanGong {
    let shi_chen = ShiChen::from_hour(hour);
    divine_by_time(lunar_month, lunar_day, shi_chen)
}

// ============================================================================
// 时刻分起课算法（道家流派）
// ============================================================================

/// 时刻分起课
///
/// 按时辰、刻、分起课，这是道家小六壬的起课方法。
/// 适用于需要更精确时间定位的占卜场景。
///
/// # 参数
/// - `shi_chen`: 时辰（1-12，子时为1）
/// - `ke`: 刻数（1-8，每时辰8刻）
/// - `fen`: 分数（1-15，每刻15分）
///
/// # 算法
/// 1. 天宫：从大安起子时，顺数至所求时辰
/// 2. 地宫：从天宫起第一刻，顺数至所求刻数
/// 3. 人宫：从地宫起第一分，顺数至所求分数
///
/// # 刻的计算说明
/// - 一个时辰（2小时）分为8刻
/// - 每刻约15分钟
/// - 刻数 = (分钟 / 15) + 1（如果刚好整除则取上一刻）
/// - 偶数小时（如0点、2点）的刻从5开始计数
/// - 奇数小时（如1点、3点）的刻从1开始计数
pub fn divine_by_hour_ke_fen(shi_chen_index: u8, ke: u8, fen: u8) -> SanGong {
    // 确保输入有效
    let shi_chen_index = shi_chen_index.max(1).min(12);
    let ke = ke.max(1).min(8);
    let fen = fen.max(1).min(15);

    // 天宫：从大安(0)起子时(1)，顺数至所求时辰
    let tian_index = (shi_chen_index.saturating_sub(1)) % 6;
    let tian_gong = LiuGong::from_index(tian_index);

    // 地宫：从天宫起第一刻，顺数至所求刻数
    let di_index = (tian_index + ke.saturating_sub(1)) % 6;
    let di_gong = LiuGong::from_index(di_index);

    // 人宫：从地宫起第一分，顺数至所求分数
    let ren_index = (di_index + fen.saturating_sub(1)) % 6;
    let ren_gong = LiuGong::from_index(ren_index);

    SanGong::new(tian_gong, di_gong, ren_gong)
}

/// 从时分计算时刻分起课
///
/// # 参数
/// - `hour`: 小时（0-23）
/// - `minute`: 分钟（0-59）
///
/// # 返回
/// (时辰索引, 刻数, 分数, 三宫结果)
pub fn divine_by_hour_minute(hour: u8, minute: u8) -> (u8, u8, u8, SanGong) {
    let shi_chen = ShiChen::from_hour(hour);
    let shi_chen_index = shi_chen.index(); // 1-12

    // 计算刻数
    // 偶数小时：刻从5开始（5,6,7,8,1,2,3,4）
    // 奇数小时：刻从1开始（1,2,3,4,5,6,7,8）
    let is_even_hour = hour % 2 == 0;
    let quarter_in_hour = minute / 15; // 0-3 表示当前小时内的第几个15分钟段

    let ke = if is_even_hour {
        // 偶数小时（如0点、2点）的前半时辰
        (quarter_in_hour + 4) % 8 + 1
    } else {
        // 奇数小时（如1点、3点）的后半时辰
        quarter_in_hour + 1
    };
    let ke = ke.max(1) as u8;

    // 计算分数（1-15，不能为0）
    let fen = (minute % 15).max(0) + 1;
    let fen = fen as u8;

    let san_gong = divine_by_hour_ke_fen(shi_chen_index, ke, fen);
    (shi_chen_index, ke, fen, san_gong)
}

// ============================================================================
// 数字起课算法
// ============================================================================

/// 数字起课
///
/// 活数起课法：取三个数字计算三宫。
///
/// # 参数
/// - `x`: 第一个数字（≥1）
/// - `y`: 第二个数字（≥1）
/// - `z`: 第三个数字（≥1）
///
/// # 算法
/// - 月宫 = (x - 1) % 6
/// - 日宫 = (x + y - 2) % 6
/// - 时宫 = (x + y + z - 3) % 6
pub fn divine_by_number(x: u8, y: u8, z: u8) -> SanGong {
    // 确保输入至少为1
    let x = x.max(1);
    let y = y.max(1);
    let z = z.max(1);

    // 月宫
    let yue_index = (x - 1) % 6;
    let yue_gong = LiuGong::from_index(yue_index);

    // 日宫
    let ri_index = (x.saturating_add(y).saturating_sub(2)) % 6;
    let ri_gong = LiuGong::from_index(ri_index);

    // 时宫
    let shi_index = (x.saturating_add(y).saturating_add(z).saturating_sub(3)) % 6;
    let shi_gong = LiuGong::from_index(shi_index);

    SanGong::new(yue_gong, ri_gong, shi_gong)
}

/// 从多位数字起课（活数起课法变体）
///
/// 输入一个数字（如1436），按以下规则计算：
/// 1. 各位数字相加
/// 2. 减去（位数 - 1）
/// 3. 除以6取余数
///
/// # 参数
/// - `number`: 输入数字
///
/// # 返回
/// 返回三宫结果（月宫=结果，日宫=结果，时宫=结果）
///
/// 注意：此为简化版，三宫结果相同。如需更复杂的计算，
/// 请使用 divine_by_three_numbers 传入三个数字。
pub fn divine_by_multi_digit(number: u32) -> SanGong {
    let digits: Vec<u32> = number
        .to_string()
        .chars()
        .filter_map(|c| c.to_digit(10))
        .collect();

    let digit_count = digits.len() as u32;
    let digit_sum: u32 = digits.iter().sum();

    // 位数为1时不减，否则减去（位数-1）
    let adjustment = if digit_count <= 1 { 0 } else { digit_count - 1 };
    let result = digit_sum.saturating_sub(adjustment);

    // 取模得到六宫索引（1-6对应0-5）
    let index = if result % 6 == 0 { 5 } else { (result % 6) as u8 - 1 };
    let gong = LiuGong::from_index(index);

    // 返回三宫（简化版，三宫相同）
    SanGong::new(gong, gong, gong)
}

/// 从多位数字起课，返回单个六神
///
/// 这是 divine_by_multi_digit 的简化版本
pub fn divine_by_multi_digit_single(number: u32) -> LiuGong {
    let digits: Vec<u32> = number
        .to_string()
        .chars()
        .filter_map(|c| c.to_digit(10))
        .collect();

    let digit_count = digits.len() as u32;
    let digit_sum: u32 = digits.iter().sum();

    // 位数为1时不减，否则减去（位数-1）
    let adjustment = if digit_count <= 1 { 0 } else { digit_count - 1 };
    let result = digit_sum.saturating_sub(adjustment);

    // 取模得到六宫索引（1-6对应0-5）
    let index = if result % 6 == 0 { 5 } else { (result % 6) as u8 - 1 };

    LiuGong::from_index(index)
}

/// 完整的多位数字起课（返回三宫）
///
/// 适用于连续输入三个数字的场景
pub fn divine_by_three_numbers(num1: u32, num2: u32, num3: u32) -> SanGong {
    let x = process_multi_digit(num1);
    let y = process_multi_digit(num2);
    let z = process_multi_digit(num3);

    divine_by_number(x, y, z)
}

/// 处理多位数字为单个数字（1-60范围）
fn process_multi_digit(number: u32) -> u8 {
    let digits: Vec<u32> = number
        .to_string()
        .chars()
        .filter_map(|c| c.to_digit(10))
        .collect();

    let digit_sum: u32 = digits.iter().sum();

    // 转换为1-60范围内的数字
    let result = if digit_sum == 0 { 1 } else { ((digit_sum - 1) % 60) + 1 };

    result as u8
}

// ============================================================================
// 随机起课算法
// ============================================================================

/// 从随机字节生成起课参数
///
/// # 参数
/// - `random_bytes`: 随机字节数组
///
/// # 返回
/// 返回 (x, y, z) 三个数字（1-60范围）
pub fn random_to_params(random_bytes: &[u8; 32]) -> (u8, u8, u8) {
    // 使用前三个字节生成1-60范围的数字
    let x = (random_bytes[0] % 60) + 1;
    let y = (random_bytes[1] % 60) + 1;
    let z = (random_bytes[2] % 60) + 1;

    (x, y, z)
}

/// 随机起课
pub fn divine_random(random_bytes: &[u8; 32]) -> SanGong {
    let (x, y, z) = random_to_params(random_bytes);
    divine_by_number(x, y, z)
}

// ============================================================================
// 手动指定
// ============================================================================

/// 手动指定三宫
pub fn divine_manual(yue: u8, ri: u8, shi: u8) -> SanGong {
    SanGong::new(
        LiuGong::from_index(yue % 6),
        LiuGong::from_index(ri % 6),
        LiuGong::from_index(shi % 6),
    )
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 获取六宫的详细解读信息
pub fn get_gong_detail(gong: LiuGong) -> GongDetail {
    GongDetail {
        name: gong.name(),
        wu_xing: gong.wu_xing().name(),
        tian_jiang: gong.tian_jiang(),
        direction: gong.direction(),
        color: gong.color(),
        fortune_level: gong.fortune_level(),
        is_auspicious: gong.is_auspicious(),
        mou_shi: gong.mou_shi_numbers(),
        brief: gong.brief(),
        gua_ci: gong.gua_ci(),
    }
}

/// 六宫详细信息
#[derive(Clone, Debug)]
pub struct GongDetail {
    pub name: &'static str,
    pub wu_xing: &'static str,
    pub tian_jiang: &'static str,
    pub direction: &'static str,
    pub color: &'static str,
    pub fortune_level: u8,
    pub is_auspicious: bool,
    pub mou_shi: [u8; 3],
    pub brief: &'static str,
    pub gua_ci: &'static str,
}

/// 分析三宫综合信息
pub fn analyze_san_gong(san_gong: &SanGong) -> SanGongAnalysis {
    SanGongAnalysis {
        fortune_level: san_gong.fortune_level(),
        is_all_auspicious: san_gong.is_all_auspicious(),
        is_all_inauspicious: san_gong.is_all_inauspicious(),
        is_pure: san_gong.is_pure(),
        wu_xing_relation: san_gong.wu_xing_analysis(),
        yue_detail: get_gong_detail(san_gong.yue_gong),
        ri_detail: get_gong_detail(san_gong.ri_gong),
        shi_detail: get_gong_detail(san_gong.shi_gong),
    }
}

/// 三宫综合分析结果
#[derive(Clone, Debug)]
pub struct SanGongAnalysis {
    pub fortune_level: u8,
    pub is_all_auspicious: bool,
    pub is_all_inauspicious: bool,
    pub is_pure: bool,
    pub wu_xing_relation: WuXingRelation,
    pub yue_detail: GongDetail,
    pub ri_detail: GongDetail,
    pub shi_detail: GongDetail,
}

// ============================================================================
// 八卦具象法分析（道家高级分析）
// ============================================================================

use crate::types::{BaGua, TiYongRelation};

/// 八卦具象法分析结果
#[derive(Clone, Debug)]
pub struct BaGuaAnalysis {
    /// 八卦
    pub ba_gua: BaGua,
    /// 八卦名称
    pub name: &'static str,
    /// 八卦符号
    pub symbol: &'static str,
    /// 八卦五行
    pub wu_xing: &'static str,
    /// 八卦阴阳
    pub yin_yang: &'static str,
    /// 八卦描述
    pub description: &'static str,
    /// 三爻组合（上中下）
    pub yao_pattern: (&'static str, &'static str, &'static str),
}

/// 分析三宫转化八卦
///
/// 将三宫的阴阳属性转化为八卦，用于高级分析
pub fn analyze_ba_gua(san_gong: &SanGong) -> BaGuaAnalysis {
    let ba_gua = BaGua::from_san_gong(san_gong);

    let yao1 = san_gong.yue_gong.yin_yang();
    let yao2 = san_gong.ri_gong.yin_yang();
    let yao3 = san_gong.shi_gong.yin_yang();

    BaGuaAnalysis {
        ba_gua,
        name: ba_gua.name(),
        symbol: ba_gua.symbol(),
        wu_xing: ba_gua.wu_xing().name(),
        yin_yang: ba_gua.yin_yang().name(),
        description: ba_gua.brief(),
        yao_pattern: (
            if yao1.is_yang() { "阳" } else { "阴" },
            if yao2.is_yang() { "阳" } else { "阴" },
            if yao3.is_yang() { "阳" } else { "阴" },
        ),
    }
}

/// 体用关系分析结果
#[derive(Clone, Debug)]
pub struct TiYongAnalysis {
    /// 体用关系
    pub relation: TiYongRelation,
    /// 关系名称
    pub name: &'static str,
    /// 吉凶描述
    pub fortune_desc: &'static str,
    /// 吉凶等级
    pub fortune_level: u8,
    /// 体（人宫）信息
    pub ti_gong: LiuGong,
    pub ti_wu_xing: &'static str,
    pub ti_yin_yang: &'static str,
    /// 用（时辰）信息
    pub yong_shi_chen: ShiChen,
    pub yong_wu_xing: &'static str,
    pub yong_yin_yang: &'static str,
}

/// 分析体用关系
///
/// 体：人宫（时宫），代表求测者自身
/// 用：时辰，代表外部环境
pub fn analyze_ti_yong(san_gong: &SanGong, shi_chen: ShiChen) -> TiYongAnalysis {
    let ti_gong = san_gong.shi_gong; // 人宫为体
    let relation = TiYongRelation::calculate(ti_gong, shi_chen);

    TiYongAnalysis {
        relation,
        name: relation.name(),
        fortune_desc: relation.fortune_desc(),
        fortune_level: relation.fortune_level(),
        ti_gong,
        ti_wu_xing: ti_gong.wu_xing().name(),
        ti_yin_yang: ti_gong.yin_yang().name(),
        yong_shi_chen: shi_chen,
        yong_wu_xing: shi_chen.wu_xing().name(),
        yong_yin_yang: shi_chen.yin_yang().name(),
    }
}

/// 完整分析结果（包含八卦和体用）
#[derive(Clone, Debug)]
pub struct FullAnalysis {
    /// 基础三宫分析
    pub san_gong: SanGongAnalysis,
    /// 八卦具象分析
    pub ba_gua: BaGuaAnalysis,
    /// 体用关系分析（如果有时辰信息）
    pub ti_yong: Option<TiYongAnalysis>,
}

/// 完整分析三宫
///
/// 包括基础分析、八卦具象法和体用关系
pub fn full_analysis(san_gong: &SanGong, shi_chen: Option<ShiChen>) -> FullAnalysis {
    FullAnalysis {
        san_gong: analyze_san_gong(san_gong),
        ba_gua: analyze_ba_gua(san_gong),
        ti_yong: shi_chen.map(|sc| analyze_ti_yong(san_gong, sc)),
    }
}

// ============================================================================
// 三宫具象法（三盘分析）
// ============================================================================

/// 三盘结果
///
/// 三宫具象法将三宫转换为天盘、地盘、人盘，
/// 通过不同的排列顺序得到不同的八卦具象
#[derive(Clone, Debug)]
pub struct SanPan {
    /// 天盘（人-天-地顺序）
    pub tian_pan: PanResult,
    /// 地盘（天-地-人顺序）
    pub di_pan: PanResult,
    /// 人盘（天-人-地顺序）
    pub ren_pan: PanResult,
}

/// 单盘分析结果
#[derive(Clone, Debug)]
pub struct PanResult {
    /// 三宫排列
    pub gongs: [LiuGong; 3],
    /// 转换得到的八卦
    pub ba_gua: BaGua,
    /// 八卦五行
    pub ba_gua_wu_xing: WuXing,
    /// 与主宫的五行关系
    pub relation: WuXingRelation,
    /// 关系描述
    pub relation_desc: &'static str,
}

/// 计算三宫具象法
///
/// 将三宫通过不同排列得到三个盘：
/// - 天盘：人-天-地顺序，主看官事、贵人
/// - 地盘：天-地-人顺序，主看财运、居所
/// - 人盘：天-人-地顺序，主看人事、感情
pub fn calculate_san_gong_juxiang(san_gong: &SanGong) -> SanPan {
    let tian_pan = calculate_tian_pan(san_gong);
    let di_pan = calculate_di_pan(san_gong);
    let ren_pan = calculate_ren_pan(san_gong);

    SanPan { tian_pan, di_pan, ren_pan }
}

/// 计算天盘（人-天-地顺序）
fn calculate_tian_pan(san_gong: &SanGong) -> PanResult {
    // 重排为：人宫-天宫-地宫
    let gongs = [san_gong.shi_gong, san_gong.yue_gong, san_gong.ri_gong];

    // 计算八卦
    let ba_gua = BaGua::from_yao(
        gongs[0].yin_yang(),
        gongs[1].yin_yang(),
        gongs[2].yin_yang(),
    );

    let ba_gua_wu_xing = ba_gua.wu_xing();

    // 计算八卦与天宫的五行关系
    let tian_gong_wu_xing = san_gong.yue_gong.wu_xing();
    let relation = calculate_wuxing_relation(ba_gua_wu_xing, tian_gong_wu_xing);

    PanResult {
        gongs,
        ba_gua,
        ba_gua_wu_xing,
        relation,
        relation_desc: relation.name(),
    }
}

/// 计算地盘（天-地-人顺序）
fn calculate_di_pan(san_gong: &SanGong) -> PanResult {
    // 保持原顺序：天宫-地宫-人宫
    let gongs = [san_gong.yue_gong, san_gong.ri_gong, san_gong.shi_gong];

    // 计算八卦
    let ba_gua = BaGua::from_yao(
        gongs[0].yin_yang(),
        gongs[1].yin_yang(),
        gongs[2].yin_yang(),
    );

    let ba_gua_wu_xing = ba_gua.wu_xing();

    // 计算八卦与地宫的五行关系
    let di_gong_wu_xing = san_gong.ri_gong.wu_xing();
    let relation = calculate_wuxing_relation(ba_gua_wu_xing, di_gong_wu_xing);

    PanResult {
        gongs,
        ba_gua,
        ba_gua_wu_xing,
        relation,
        relation_desc: relation.name(),
    }
}

/// 计算人盘（天-人-地顺序）
fn calculate_ren_pan(san_gong: &SanGong) -> PanResult {
    // 重排为：天宫-人宫-地宫
    let gongs = [san_gong.yue_gong, san_gong.shi_gong, san_gong.ri_gong];

    // 计算八卦
    let ba_gua = BaGua::from_yao(
        gongs[0].yin_yang(),
        gongs[1].yin_yang(),
        gongs[2].yin_yang(),
    );

    let ba_gua_wu_xing = ba_gua.wu_xing();

    // 计算八卦与人宫的五行关系
    let ren_gong_wu_xing = san_gong.shi_gong.wu_xing();
    let relation = calculate_wuxing_relation(ba_gua_wu_xing, ren_gong_wu_xing);

    PanResult {
        gongs,
        ba_gua,
        ba_gua_wu_xing,
        relation,
        relation_desc: relation.name(),
    }
}

/// 计算两个五行之间的关系
fn calculate_wuxing_relation(wx1: WuXing, wx2: WuXing) -> WuXingRelation {
    if wx1 == wx2 {
        WuXingRelation::BiHe // 比和
    } else if wx1.generates() == wx2 {
        WuXingRelation::Sheng // 生出
    } else if wx1.restrains() == wx2 {
        WuXingRelation::Ke // 克出
    } else if wx1.generated_by() == wx2 {
        WuXingRelation::XieSheng // 被生/泄
    } else {
        WuXingRelation::BeiKe // 被克
    }
}

// ============================================================================
// 详细五行关系分析
// ============================================================================

/// 详细五行关系分析结果
#[derive(Clone, Debug)]
pub struct DetailedWuXingAnalysis {
    /// 天地关系（月宫-日宫）
    pub tian_di: WuXingRelationDetail,
    /// 天人关系（月宫-时宫）
    pub tian_ren: WuXingRelationDetail,
    /// 地人关系（日宫-时宫）
    pub di_ren: WuXingRelationDetail,
    /// 体用关系（如果有时辰）
    pub ti_yong: Option<TiYongRelation>,
    /// 综合吉凶
    pub overall_fortune: FortuneSummary,
}

/// 五行关系详情
#[derive(Clone, Debug)]
pub struct WuXingRelationDetail {
    /// 源五行
    pub from_wu_xing: WuXing,
    /// 目标五行
    pub to_wu_xing: WuXing,
    /// 关系类型
    pub relation: WuXingRelation,
    /// 关系名称
    pub name: &'static str,
    /// 对吉凶的影响
    pub fortune_impact: i8,
}

/// 综合吉凶摘要
#[derive(Clone, Debug)]
pub struct FortuneSummary {
    /// 吉凶等级（1-10，10最吉）
    pub level: u8,
    /// 吉凶描述
    pub description: &'static str,
    /// 建议
    pub advice: &'static str,
}

/// 分析详细五行关系
pub fn analyze_detailed_wuxing(san_gong: &SanGong, shi_chen: Option<ShiChen>) -> DetailedWuXingAnalysis {
    // 获取三宫五行
    let yue_wx = san_gong.yue_gong.wu_xing();
    let ri_wx = san_gong.ri_gong.wu_xing();
    let shi_wx = san_gong.shi_gong.wu_xing();

    // 计算天地关系
    let tian_di_rel = calculate_wuxing_relation(yue_wx, ri_wx);
    let tian_di = WuXingRelationDetail {
        from_wu_xing: yue_wx,
        to_wu_xing: ri_wx,
        relation: tian_di_rel,
        name: tian_di_rel.name(),
        fortune_impact: tian_di_rel.fortune_modifier(),
    };

    // 计算天人关系
    let tian_ren_rel = calculate_wuxing_relation(yue_wx, shi_wx);
    let tian_ren = WuXingRelationDetail {
        from_wu_xing: yue_wx,
        to_wu_xing: shi_wx,
        relation: tian_ren_rel,
        name: tian_ren_rel.name(),
        fortune_impact: tian_ren_rel.fortune_modifier(),
    };

    // 计算地人关系
    let di_ren_rel = calculate_wuxing_relation(ri_wx, shi_wx);
    let di_ren = WuXingRelationDetail {
        from_wu_xing: ri_wx,
        to_wu_xing: shi_wx,
        relation: di_ren_rel,
        name: di_ren_rel.name(),
        fortune_impact: di_ren_rel.fortune_modifier(),
    };

    // 计算体用关系
    let ti_yong = shi_chen.map(|sc| TiYongRelation::calculate(san_gong.shi_gong, sc));

    // 计算综合吉凶
    let overall_fortune = calculate_overall_fortune(san_gong, &tian_di, &tian_ren, &di_ren, ti_yong);

    DetailedWuXingAnalysis {
        tian_di,
        tian_ren,
        di_ren,
        ti_yong,
        overall_fortune,
    }
}

/// 计算综合吉凶
fn calculate_overall_fortune(
    san_gong: &SanGong,
    tian_di: &WuXingRelationDetail,
    tian_ren: &WuXingRelationDetail,
    di_ren: &WuXingRelationDetail,
    ti_yong: Option<TiYongRelation>,
) -> FortuneSummary {
    // 基础分数：三宫吉凶平均值 * 2
    let base_score = san_gong.fortune_level() as i16 * 2;

    // 五行关系加成
    let relation_score = (tian_di.fortune_impact + tian_ren.fortune_impact + di_ren.fortune_impact) as i16;

    // 体用关系加成
    let ti_yong_score = ti_yong.map(|ty| ty.fortune_level() as i16 - 3).unwrap_or(0);

    // 特殊情况加成
    let special_score = if san_gong.is_pure() {
        3 // 纯宫加分
    } else if san_gong.is_all_auspicious() {
        2 // 全吉加分
    } else if san_gong.is_all_inauspicious() {
        -2 // 全凶减分
    } else {
        0
    };

    // 计算最终分数（限制在1-10范围）
    let total_score = base_score + relation_score + ti_yong_score + special_score;
    let level = total_score.clamp(1, 10) as u8;

    // 根据分数确定描述和建议
    let (description, advice) = match level {
        9..=10 => ("大吉大利", "诸事皆宜，可大胆行事，贵人相助，心想事成"),
        7..=8 => ("吉祥顺遂", "事情顺利，稍加努力即可成功，宜积极进取"),
        5..=6 => ("中平之象", "平稳无大碍，宜守不宜进，谨慎行事可保平安"),
        3..=4 => ("小有阻碍", "事多波折，需耐心等待，不宜冒进，宜静观其变"),
        _ => ("诸事不顺", "凶险当道，宜守勿进，多加小心，避免大事"),
    };

    FortuneSummary {
        level,
        description,
        advice,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divine_by_time() {
        // 测试时间起课
        // 六月初五辰时
        let result = divine_by_time(6, 5, ShiChen::Chen);

        // 月宫：从大安起正月，6月 = (6-1) % 6 = 5 → 空亡
        assert_eq!(result.yue_gong, LiuGong::KongWang);

        // 日宫：从空亡起初一，初五 = (5 + 5 - 1) % 6 = 3 → 赤口
        assert_eq!(result.ri_gong, LiuGong::ChiKou);

        // 时宫：从赤口起子时，辰时(5) = (3 + 5 - 1) % 6 = 1 → 留连
        assert_eq!(result.shi_gong, LiuGong::LiuLian);
    }

    #[test]
    fn test_divine_by_number() {
        // 测试数字起课
        let result = divine_by_number(1, 2, 3);

        // 月宫 = (1-1) % 6 = 0 → 大安
        assert_eq!(result.yue_gong, LiuGong::DaAn);

        // 日宫 = (1+2-2) % 6 = 1 → 留连
        assert_eq!(result.ri_gong, LiuGong::LiuLian);

        // 时宫 = (1+2+3-3) % 6 = 3 → 赤口
        assert_eq!(result.shi_gong, LiuGong::ChiKou);
    }

    #[test]
    fn test_divine_by_number_wrap() {
        // 测试取模边界
        let result = divine_by_number(6, 6, 6);

        // 月宫 = (6-1) % 6 = 5 → 空亡
        assert_eq!(result.yue_gong, LiuGong::KongWang);

        // 日宫 = (6+6-2) % 6 = 10 % 6 = 4 → 小吉
        assert_eq!(result.ri_gong, LiuGong::XiaoJi);

        // 时宫 = (6+6+6-3) % 6 = 15 % 6 = 3 → 赤口
        assert_eq!(result.shi_gong, LiuGong::ChiKou);
    }

    #[test]
    fn test_divine_by_multi_digit() {
        // 测试1436
        let result = divine_by_multi_digit_single(1436);
        // 1+4+3+6 = 14, 14-3 = 11, 11 % 6 = 5
        // 5不为0，所以 index = 5 - 1 = 4 → 小吉(索引4)
        assert_eq!(result, LiuGong::XiaoJi);

        // 测试18
        let result = divine_by_multi_digit_single(18);
        // 1+8 = 9, 9-1 = 8, 8 % 6 = 2
        // 2不为0，所以 index = 2 - 1 = 1 → 留连(索引1)
        assert_eq!(result, LiuGong::LiuLian);

        // 测试返回三宫的版本
        let san_gong = divine_by_multi_digit(1436);
        assert_eq!(san_gong.yue_gong, LiuGong::XiaoJi);
        assert_eq!(san_gong.ri_gong, LiuGong::XiaoJi);
        assert_eq!(san_gong.shi_gong, LiuGong::XiaoJi);
    }

    #[test]
    fn test_shi_chen_from_hour() {
        assert_eq!(ShiChen::from_hour(0), ShiChen::Zi);
        assert_eq!(ShiChen::from_hour(1), ShiChen::Chou);
        assert_eq!(ShiChen::from_hour(3), ShiChen::Yin);
        assert_eq!(ShiChen::from_hour(7), ShiChen::Chen);
        assert_eq!(ShiChen::from_hour(11), ShiChen::Wu);
        assert_eq!(ShiChen::from_hour(23), ShiChen::Zi);
    }

    #[test]
    fn test_liu_gong_properties() {
        let da_an = LiuGong::DaAn;
        assert_eq!(da_an.name(), "大安");
        assert_eq!(da_an.wu_xing(), WuXing::Wood);
        assert_eq!(da_an.tian_jiang(), "青龙");
        assert!(da_an.is_auspicious());
        assert_eq!(da_an.fortune_level(), 5);

        let kong_wang = LiuGong::KongWang;
        assert_eq!(kong_wang.name(), "空亡");
        assert_eq!(kong_wang.wu_xing(), WuXing::Earth);
        assert!(!kong_wang.is_auspicious());
    }

    #[test]
    fn test_san_gong_analysis() {
        // 全吉
        let all_good = SanGong::new(LiuGong::DaAn, LiuGong::SuXi, LiuGong::XiaoJi);
        assert!(all_good.is_all_auspicious());
        assert!(!all_good.is_all_inauspicious());

        // 全凶
        let all_bad = SanGong::new(LiuGong::LiuLian, LiuGong::ChiKou, LiuGong::KongWang);
        assert!(!all_bad.is_all_auspicious());
        assert!(all_bad.is_all_inauspicious());

        // 纯宫
        let pure = SanGong::new(LiuGong::DaAn, LiuGong::DaAn, LiuGong::DaAn);
        assert!(pure.is_pure());
    }

    #[test]
    fn test_random_to_params() {
        let bytes = [10u8; 32];
        let (x, y, z) = random_to_params(&bytes);

        // 10 % 60 + 1 = 11
        assert_eq!(x, 11);
        assert_eq!(y, 11);
        assert_eq!(z, 11);
    }

    #[test]
    fn test_divine_manual() {
        let result = divine_manual(0, 1, 2);
        assert_eq!(result.yue_gong, LiuGong::DaAn);
        assert_eq!(result.ri_gong, LiuGong::LiuLian);
        assert_eq!(result.shi_gong, LiuGong::SuXi);
    }
}
