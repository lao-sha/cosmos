//! # 紫微斗数排盘算法
//!
//! 本模块实现紫微斗数的核心排盘算法，包括：
//! - 命宫定位
//! - 五行局计算
//! - 紫微星系安星
//! - 天府星系安星
//! - 六吉六煞安星
//! - 四化飞星
//! - 大运起运

use crate::types::*;

// ============================================================================
// 六十甲子纳音五行表
// ============================================================================

/// 六十甲子纳音五行查表
/// 返回 (五行, 局数)
pub fn get_na_yin_wu_xing(tian_gan: TianGan, di_zhi: DiZhi) -> WuXing {
    let gan_idx = tian_gan.index();
    let zhi_idx = di_zhi.index();

    // 六十甲子索引 = (gan_idx * 12 + zhi_idx) 但需要考虑天干地支配对规则
    // 简化：使用 (gan_idx + zhi_idx) % 2 == 0 验证有效配对

    // 纳音五行表（按甲子序排列，每组2个）
    // 甲子乙丑海中金，丙寅丁卯炉中火，戊辰己巳大林木，庚午辛未路旁土，壬申癸酉剑锋金
    // 甲戌乙亥山头火，丙子丁丑涧下水，戊寅己卯城头土，庚辰辛巳白蜡金，壬午癸未杨柳木
    // 甲申乙酉泉中水，丙戌丁亥屋上土，戊子己丑霹雳火，庚寅辛卯松柏木，壬辰癸巳长流水
    // 甲午乙未沙中金，丙申丁酉山下火，戊戌己亥平地木，庚子辛丑壁上土，壬寅癸卯金箔金
    // 甲辰乙巳覆灯火，丙午丁未天河水，戊申己酉大驿土，庚戌辛亥钗钏金，壬子癸丑桑柘木
    // 甲寅乙卯大溪水，丙辰丁巳沙中土，戊午己未天上火，庚申辛酉石榴木，壬戌癸亥大海水

    // 计算六十甲子序数
    let jia_zi_idx = ((gan_idx as u16 * 12 + zhi_idx as u16) % 60) as u8;
    let group = jia_zi_idx / 2;

    match group {
        0 | 4 | 13 | 17 | 23 | 29 => WuXing::Metal,   // 金
        1 | 5 | 8 | 12 | 16 | 21 => WuXing::Fire,     // 火
        2 | 9 | 14 | 18 | 24 | 28 => WuXing::Wood,    // 木
        3 | 7 | 11 | 15 | 19 | 22 => WuXing::Earth,   // 土
        6 | 10 | 20 | 25 | 26 | 27 => WuXing::Water,  // 水
        _ => WuXing::Water,
    }
}

// ============================================================================
// 命宫计算
// ============================================================================

/// 计算命宫位置
/// 口诀：顺数生月，逆数生时
/// 从寅宫起正月，顺数至生月，再从该宫逆数至生时
pub fn calculate_ming_gong(lunar_month: u8, birth_hour: DiZhi) -> u8 {
    // 从寅宫(2)起正月，顺数到生月
    let month_pos = (2 + lunar_month - 1) % 12;
    // 从该宫逆数到生时
    let hour_idx = birth_hour.index();
    let ming_gong = (month_pos + 12 - hour_idx) % 12;
    ming_gong
}

/// 计算身宫位置
/// 口诀：顺数生月，顺数生时
pub fn calculate_shen_gong(lunar_month: u8, birth_hour: DiZhi) -> u8 {
    let month_pos = (2 + lunar_month - 1) % 12;
    let hour_idx = birth_hour.index();
    let shen_gong = (month_pos + hour_idx) % 12;
    shen_gong
}

// ============================================================================
// 五行局计算
// ============================================================================

/// 计算五行局
/// 根据命宫所在宫干支的纳音五行确定
pub fn calculate_wu_xing_ju(year_gan: TianGan, ming_gong_pos: u8) -> (WuXing, u8) {
    // 根据年干确定命宫天干（五虎遁）
    let ming_gan = get_gong_gan(year_gan, ming_gong_pos);
    let ming_zhi = DiZhi::from_index(ming_gong_pos);

    let wu_xing = get_na_yin_wu_xing(ming_gan, ming_zhi);
    let ju_shu = wu_xing.ju_shu();

    (wu_xing, ju_shu)
}

/// 五虎遁 - 根据年干推算各宫天干
/// 甲己之年丙作首（寅宫起丙）
/// 乙庚之年戊为头（寅宫起戊）
/// 丙辛之年庚为首（寅宫起庚）
/// 丁壬之年壬为首（寅宫起壬）
/// 戊癸之年甲为首（寅宫起甲）
pub fn get_gong_gan(year_gan: TianGan, gong_pos: u8) -> TianGan {
    // 确定寅宫的天干
    let yin_gan = match year_gan {
        TianGan::Jia | TianGan::Ji => TianGan::Bing,
        TianGan::Yi | TianGan::Geng => TianGan::Wu,
        TianGan::Bing | TianGan::Xin => TianGan::Geng,
        TianGan::Ding | TianGan::Ren => TianGan::Ren,
        TianGan::Wu | TianGan::Gui => TianGan::Jia,
    };

    // 从寅宫顺推到目标宫
    let offset = (gong_pos + 12 - 2) % 12; // 寅宫是2
    TianGan::from_index((yin_gan.index() + offset as u8) % 10)
}

// ============================================================================
// 紫微星定位
// ============================================================================

/// 根据农历日和局数定紫微星位置
/// 这是紫微斗数最核心的安星步骤
pub fn calculate_ziwei_position(lunar_day: u8, ju_shu: u8) -> u8 {
    // 紫微星位置查表（简化版）
    // 实际应根据《紫微斗数全书》的安星诀完整实现

    // 使用公式：紫微位置 = f(日数, 局数)
    // 完整的查表数据
    // 紫微星定位表（已校正）
    // 数据来源：《紫微斗数全书》安星诀 / openzw 参考实现
    // 列顺序：水二局、木三局、金四局、土五局、火六局
    // 数值为地支索引：子=0, 丑=1, 寅=2, 卯=3, 辰=4, 巳=5, 午=6, 未=7, 申=8, 酉=9, 戌=10, 亥=11
    let ziwei_table: [[u8; 5]; 30] = [
        [ 1,  4, 11,  6,  9],   // 日1:  丑、辰、亥、午、酉
        [ 2,  1,  4, 11,  6],   // 日2:  寅、丑、辰、亥、午
        [ 2,  2,  1,  4, 11],   // 日3:  寅、寅、丑、辰、亥
        [ 3,  5,  2,  1,  4],   // 日4:  卯、巳、寅、丑、辰
        [ 3,  2,  0,  2,  1],   // 日5:  卯、寅、子、寅、丑
        [ 4,  3,  5,  7,  2],   // 日6:  辰、卯、巳、未、寅
        [ 4,  6,  2,  0, 10],   // 日7:  辰、午、寅、子、戌
        [ 5,  3,  3,  5,  7],   // 日8:  巳、卯、卯、巳、未
        [ 5,  4,  1,  2,  0],   // 日9:  巳、辰、丑、寅、子
        [ 6,  7,  6,  3,  5],   // 日10: 午、未、午、卯、巳
        [ 6,  4,  3,  8,  2],   // 日11: 午、辰、卯、申、寅
        [ 7,  5,  4,  1,  3],   // 日12: 未、巳、辰、丑、卯
        [ 7,  8,  2,  6, 11],   // 日13: 未、申、寅、午、亥
        [ 8,  5,  7,  3,  8],   // 日14: 申、巳、未、卯、申
        [ 8,  6,  4,  4,  1],   // 日15: 申、午、辰、辰、丑
        [ 9,  9,  5,  9,  6],   // 日16: 酉、酉、巳、酉、午
        [ 9,  6,  3,  2,  3],   // 日17: 酉、午、卯、寅、卯
        [10,  7,  8,  7,  4],   // 日18: 戌、未、申、未、辰 [修正: 土5→7, 火4不变]
        [10, 10,  5, 10,  0],   // 日19: 戌、戌、巳、戌、子 [修正: 火9→0]
        [11,  7,  6,  5,  9],   // 日20: 亥、未、午、巳、酉 [修正: 土3→5, 火6→9]
        [11,  8,  4, 10,  2],   // 日21: 亥、申、辰、戌、寅 [修正: 土8→10, 火3→2]
        [ 0, 11,  9,  3,  7],   // 日22: 子、亥、酉、卯、未 [修正: 土5→3, 火4→7]
        [ 0,  8,  6,  8,  4],   // 日23: 子、申、午、申、辰 [修正: 土2→8, 火1→4]
        [ 1,  9,  7,  5,  5],   // 日24: 丑、酉、未、巳、巳
        [ 1,  0,  5,  6,  1],   // 日25: 丑、子、巳、午、丑
        [ 2,  9, 10, 11, 10],   // 日26: 寅、酉、戌、亥、戌
        [ 2, 10,  7,  4,  3],   // 日27: 寅、戌、未、辰、卯
        [ 3,  1,  8,  9,  8],   // 日28: 卯、丑、申、酉、申
        [ 3, 10,  6,  6,  5],   // 日29: 卯、戌、午、午、巳
        [ 4, 11, 11,  7,  6],   // 日30: 辰、亥、亥、未、午
    ];

    let day_idx = (lunar_day.saturating_sub(1) % 30) as usize;
    let ju_idx = match ju_shu {
        2 => 0,
        3 => 1,
        4 => 2,
        5 => 3,
        6 => 4,
        _ => 0,
    };

    ziwei_table[day_idx][ju_idx]
}

// ============================================================================
// 紫微星系安星
// ============================================================================

/// 安紫微星系（6颗主星）
/// 紫微、天机、太阳、武曲、天同、廉贞
/// 口诀：紫微天机逆行旁，隔一阳武天同当，隔二必是廉贞地，空三复见紫微郎
pub fn place_ziwei_series(ziwei_pos: u8) -> [(ZhuXing, u8); 6] {
    let mut positions = [(ZhuXing::ZiWei, 0u8); 6];

    // 紫微
    positions[0] = (ZhuXing::ZiWei, ziwei_pos);

    // 天机：紫微逆行1宫
    positions[1] = (ZhuXing::TianJi, (ziwei_pos + 11) % 12);

    // 太阳：天机逆行2宫（隔1宫）
    positions[2] = (ZhuXing::TaiYang, (ziwei_pos + 9) % 12);

    // 武曲：太阳逆行1宫
    positions[3] = (ZhuXing::WuQu, (ziwei_pos + 8) % 12);

    // 天同：武曲逆行1宫
    positions[4] = (ZhuXing::TianTong, (ziwei_pos + 7) % 12);

    // 廉贞：天同逆行3宫（隔2宫）
    positions[5] = (ZhuXing::LianZhen, (ziwei_pos + 4) % 12);

    positions
}

// ============================================================================
// 天府星系安星
// ============================================================================

/// 计算天府星位置（根据紫微位置）
/// 天府与紫微关于寅申轴对称
///
/// # 算法说明
/// 紫府对照表规律：天府位置 = (16 - 紫微位置) % 12
/// 等价于 (4 + 12 - ziwei_pos) % 12
///
/// | 紫微 | 天府 |
/// |------|------|
/// | 子0  | 辰4  |
/// | 丑1  | 卯3  |
/// | 寅2  | 寅2  |
/// | 卯3  | 丑1  |
/// | 辰4  | 子0  |
/// | 巳5  | 亥11 |
/// | 午6  | 戌10 |
/// | 未7  | 酉9  |
/// | 申8  | 申8  |
/// | 酉9  | 未7  |
/// | 戌10 | 午6  |
/// | 亥11 | 巳5  |
pub fn calculate_tianfu_position(ziwei_pos: u8) -> u8 {
    // 修正后的公式，与参考实现一致
    (16 - ziwei_pos) % 12
}

/// 安天府星系（8颗主星）
/// 天府、太阴、贪狼、巨门、天相、天梁、七杀、破军
/// 口诀：天府太阴与贪狼，巨门天相与天梁，七杀空三破军位，八星顺数细推详
pub fn place_tianfu_series(tianfu_pos: u8) -> [(ZhuXing, u8); 8] {
    let mut positions = [(ZhuXing::TianFu, 0u8); 8];

    // 天府
    positions[0] = (ZhuXing::TianFu, tianfu_pos);

    // 太阴：天府顺行1宫
    positions[1] = (ZhuXing::TaiYin, (tianfu_pos + 1) % 12);

    // 贪狼：太阴顺行1宫
    positions[2] = (ZhuXing::TanLang, (tianfu_pos + 2) % 12);

    // 巨门：贪狼顺行1宫
    positions[3] = (ZhuXing::JuMen, (tianfu_pos + 3) % 12);

    // 天相：巨门顺行1宫
    positions[4] = (ZhuXing::TianXiang, (tianfu_pos + 4) % 12);

    // 天梁：天相顺行1宫
    positions[5] = (ZhuXing::TianLiang, (tianfu_pos + 5) % 12);

    // 七杀：天梁顺行4宫（隔3宫）
    positions[6] = (ZhuXing::QiSha, (tianfu_pos + 6) % 12);

    // 破军：七杀顺行4宫（隔3宫）
    positions[7] = (ZhuXing::PoJun, (tianfu_pos + 10) % 12);

    positions
}

// ============================================================================
// 六吉星安星
// ============================================================================

/// 安文昌文曲
/// 文昌：由生时定，从戌宫起子时逆行
/// 文曲：由生时定，从辰宫起子时顺行
pub fn calculate_wen_chang_qu(birth_hour: DiZhi) -> (u8, u8) {
    let hour_idx = birth_hour.index();

    // 文昌：戌宫(10)起子时，逆行
    let wen_chang = (10 + 12 - hour_idx) % 12;

    // 文曲：辰宫(4)起子时，顺行
    let wen_qu = (4 + hour_idx) % 12;

    (wen_chang, wen_qu)
}

/// 安左辅右弼
/// 左辅：由生月定，从辰宫起正月顺行
/// 右弼：由生月定，从戌宫起正月逆行
pub fn calculate_zuo_fu_you_bi(lunar_month: u8) -> (u8, u8) {
    let month_offset = lunar_month - 1;

    // 左辅：辰宫(4)起正月，顺行
    let zuo_fu = (4 + month_offset) % 12;

    // 右弼：戌宫(10)起正月，逆行
    let you_bi = (10 + 12 - month_offset) % 12;

    (zuo_fu, you_bi)
}

/// 安天魁天钺（根据年干）
pub fn calculate_tian_kui_yue(year_gan: TianGan) -> (u8, u8) {
    match year_gan {
        TianGan::Jia | TianGan::Wu => (1, 7),   // 丑、未
        TianGan::Yi | TianGan::Ji => (0, 8),    // 子、申
        TianGan::Bing | TianGan::Ding => (11, 9), // 亥、酉
        TianGan::Geng | TianGan::Xin => (2, 6),  // 寅、午
        TianGan::Ren | TianGan::Gui => (3, 5),   // 卯、巳
    }
}

// ============================================================================
// 六煞星安星
// ============================================================================

/// 安擎羊陀罗（根据年干）
/// 擎羊在禄存之前一位，陀罗在禄存之后一位
pub fn calculate_qing_yang_tuo_luo(year_gan: TianGan) -> (u8, u8) {
    // 先找禄存位置
    let lu_cun = calculate_lu_cun(year_gan);

    // 擎羊：禄存顺行1宫
    let qing_yang = (lu_cun + 1) % 12;

    // 陀罗：禄存逆行1宫
    let tuo_luo = (lu_cun + 11) % 12;

    (qing_yang, tuo_luo)
}

/// 安禄存（根据年干）
pub fn calculate_lu_cun(year_gan: TianGan) -> u8 {
    match year_gan {
        TianGan::Jia => 2,  // 寅
        TianGan::Yi => 3,   // 卯
        TianGan::Bing | TianGan::Wu => 5,  // 巳
        TianGan::Ding | TianGan::Ji => 6,  // 午
        TianGan::Geng => 8,  // 申
        TianGan::Xin => 9,   // 酉
        TianGan::Ren => 11,  // 亥
        TianGan::Gui => 0,   // 子
    }
}

/// 安火星铃星（根据年支和时辰）
///
/// # 火星起点口诀
/// 寅午戌年丑宫起，申子辰年寅宫起，巳酉丑年卯宫起，亥卯未年酉宫起
///
/// # 铃星起点口诀
/// 寅午戌年卯宫起，其他年戌宫起
///
/// # 参数
/// - year_zhi: 年支（决定起点宫位）
/// - birth_hour: 出生时辰（从起点宫位顺数）
///
/// # 返回
/// (火星位置, 铃星位置)
pub fn calculate_huo_ling(year_zhi: DiZhi, birth_hour: DiZhi) -> (u8, u8) {
    let year_idx = year_zhi.index();
    let hour_idx = birth_hour.index();

    // 火星起点（根据年支三合局分组）
    // 寅午戌 (2,6,10) → 丑宫(1)起
    // 申子辰 (8,0,4)  → 寅宫(2)起
    // 巳酉丑 (5,9,1)  → 卯宫(3)起
    // 亥卯未 (11,3,7) → 酉宫(9)起
    let huo_start = match year_idx {
        2 | 6 | 10 => 1,   // 寅午戌年，丑宫起
        8 | 0 | 4 => 2,    // 申子辰年，寅宫起
        5 | 9 | 1 => 3,    // 巳酉丑年，卯宫起
        11 | 3 | 7 => 9,   // 亥卯未年，酉宫起
        _ => 2,            // 默认寅宫
    };

    // 铃星起点
    // 寅午戌年卯宫起，其他年戌宫起
    let ling_start = match year_idx {
        2 | 6 | 10 => 3,   // 寅午戌年，卯宫起
        _ => 10,           // 其他年，戌宫起
    };

    // 从起点宫位顺数到出生时辰
    let huo_xing = (huo_start + hour_idx) % 12;
    let ling_xing = (ling_start + hour_idx) % 12;

    (huo_xing, ling_xing)
}

/// 安地空地劫（根据时辰）
pub fn calculate_di_kong_jie(birth_hour: DiZhi) -> (u8, u8) {
    let hour_idx = birth_hour.index();

    // 地空：亥宫起子时逆行
    let di_kong = (11 + 12 - hour_idx) % 12;

    // 地劫：亥宫起子时顺行
    let di_jie = (11 + hour_idx) % 12;

    (di_kong, di_jie)
}

// ============================================================================
// 四化飞星
// ============================================================================

/// 获取生年四化星（旧版兼容接口，仅返回主星）
///
/// 返回 (化禄星, 化权星, 化科星, 化忌星)
///
/// # 注意
/// 此函数为兼容旧代码保留，对于涉及辅星的四化使用了替代主星：
/// - 丙干化科：正确为文昌，此处返回天同
/// - 戊干化科：正确为右弼，此处返回太阳
/// - 己干化忌：正确为文曲，此处返回天机
/// - 辛干化科：正确为文曲，此处返回天机
/// - 辛干化忌：正确为文昌，此处返回天同
/// - 壬干化科：正确为左辅，此处返回天府
///
/// 建议使用 `get_si_hua_stars_full()` 获取完整准确的四化星。
#[deprecated(since = "1.1.0", note = "请使用 get_si_hua_stars_full() 获取准确的四化星")]
pub fn get_si_hua_stars(year_gan: TianGan) -> [ZhuXing; 4] {
    match year_gan {
        TianGan::Jia => [ZhuXing::LianZhen, ZhuXing::PoJun, ZhuXing::WuQu, ZhuXing::TaiYang],
        TianGan::Yi => [ZhuXing::TianJi, ZhuXing::TianLiang, ZhuXing::ZiWei, ZhuXing::TaiYin],
        TianGan::Bing => [ZhuXing::TianTong, ZhuXing::TianJi, ZhuXing::TianTong, ZhuXing::LianZhen], // 文昌化科→天同(占位)
        TianGan::Ding => [ZhuXing::TaiYin, ZhuXing::TianTong, ZhuXing::TianJi, ZhuXing::JuMen],
        TianGan::Wu => [ZhuXing::TanLang, ZhuXing::TaiYin, ZhuXing::TaiYang, ZhuXing::TianJi], // 右弼化科→太阳(占位)
        TianGan::Ji => [ZhuXing::WuQu, ZhuXing::TanLang, ZhuXing::TianLiang, ZhuXing::TianJi], // 文曲化忌→天机(占位)
        TianGan::Geng => [ZhuXing::TaiYang, ZhuXing::WuQu, ZhuXing::TaiYin, ZhuXing::TianTong],
        TianGan::Xin => [ZhuXing::JuMen, ZhuXing::TaiYang, ZhuXing::TianJi, ZhuXing::TianTong], // 文曲化科→天机、文昌化忌→天同(占位)
        TianGan::Ren => [ZhuXing::TianLiang, ZhuXing::ZiWei, ZhuXing::TianFu, ZhuXing::WuQu], // 左辅化科→天府(占位)
        TianGan::Gui => [ZhuXing::PoJun, ZhuXing::JuMen, ZhuXing::TaiYin, ZhuXing::TanLang],
    }
}

/// 获取生年四化星（完整版，支持主星和辅星）
///
/// 返回 [化禄星, 化权星, 化科星, 化忌星]
///
/// 根据《紫微斗数全书》安星诀，各天干四化如下：
/// - 甲：廉贞化禄、破军化权、武曲化科、太阳化忌
/// - 乙：天机化禄、天梁化权、紫微化科、太阴化忌
/// - 丙：天同化禄、天机化权、文昌化科、廉贞化忌
/// - 丁：太阴化禄、天同化权、天机化科、巨门化忌
/// - 戊：贪狼化禄、太阴化权、右弼化科、天机化忌
/// - 己：武曲化禄、贪狼化权、天梁化科、文曲化忌
/// - 庚：太阳化禄、武曲化权、太阴化科、天同化忌
/// - 辛：巨门化禄、太阳化权、文曲化科、文昌化忌
/// - 壬：天梁化禄、紫微化权、左辅化科、武曲化忌
/// - 癸：破军化禄、巨门化权、太阴化科、贪狼化忌
pub fn get_si_hua_stars_full(year_gan: TianGan) -> [SiHuaStar; 4] {
    match year_gan {
        // 甲：廉贞化禄、破军化权、武曲化科、太阳化忌
        TianGan::Jia => [
            SiHuaStar::LianZhen,
            SiHuaStar::PoJun,
            SiHuaStar::WuQu,
            SiHuaStar::TaiYang,
        ],
        // 乙：天机化禄、天梁化权、紫微化科、太阴化忌
        TianGan::Yi => [
            SiHuaStar::TianJi,
            SiHuaStar::TianLiang,
            SiHuaStar::ZiWei,
            SiHuaStar::TaiYin,
        ],
        // 丙：天同化禄、天机化权、文昌化科、廉贞化忌
        TianGan::Bing => [
            SiHuaStar::TianTong,
            SiHuaStar::TianJi,
            SiHuaStar::WenChang, // 正确：文昌
            SiHuaStar::LianZhen,
        ],
        // 丁：太阴化禄、天同化权、天机化科、巨门化忌
        TianGan::Ding => [
            SiHuaStar::TaiYin,
            SiHuaStar::TianTong,
            SiHuaStar::TianJi,
            SiHuaStar::JuMen,
        ],
        // 戊：贪狼化禄、太阴化权、右弼化科、天机化忌
        TianGan::Wu => [
            SiHuaStar::TanLang,
            SiHuaStar::TaiYin,
            SiHuaStar::YouBi, // 正确：右弼
            SiHuaStar::TianJi,
        ],
        // 己：武曲化禄、贪狼化权、天梁化科、文曲化忌
        TianGan::Ji => [
            SiHuaStar::WuQu,
            SiHuaStar::TanLang,
            SiHuaStar::TianLiang,
            SiHuaStar::WenQu, // 正确：文曲
        ],
        // 庚：太阳化禄、武曲化权、太阴化科、天同化忌
        TianGan::Geng => [
            SiHuaStar::TaiYang,
            SiHuaStar::WuQu,
            SiHuaStar::TaiYin,
            SiHuaStar::TianTong,
        ],
        // 辛：巨门化禄、太阳化权、文曲化科、文昌化忌
        TianGan::Xin => [
            SiHuaStar::JuMen,
            SiHuaStar::TaiYang,
            SiHuaStar::WenQu,   // 正确：文曲
            SiHuaStar::WenChang, // 正确：文昌
        ],
        // 壬：天梁化禄、紫微化权、左辅化科、武曲化忌
        TianGan::Ren => [
            SiHuaStar::TianLiang,
            SiHuaStar::ZiWei,
            SiHuaStar::ZuoFu, // 正确：左辅
            SiHuaStar::WuQu,
        ],
        // 癸：破军化禄、巨门化权、太阴化科、贪狼化忌
        TianGan::Gui => [
            SiHuaStar::PoJun,
            SiHuaStar::JuMen,
            SiHuaStar::TaiYin,
            SiHuaStar::TanLang,
        ],
    }
}

/// 获取指定天干的四化信息描述
///
/// 返回格式化的四化星名称字符串
pub fn describe_si_hua(year_gan: TianGan) -> (&'static str, &'static str, &'static str, &'static str) {
    let stars = get_si_hua_stars_full(year_gan);
    (stars[0].name(), stars[1].name(), stars[2].name(), stars[3].name())
}

// ============================================================================
// 大运计算
// ============================================================================

/// 计算起运年龄
/// 根据命宫五行局数确定
pub fn calculate_qi_yun_age(ju_shu: u8) -> u8 {
    ju_shu
}

/// 判断大运顺逆
/// 阳男阴女顺行，阴男阳女逆行
pub fn calculate_da_yun_direction(year_gan: TianGan, gender: Gender) -> bool {
    let is_yang = year_gan.yin_yang() == YinYang::Yang;
    let is_male = gender == Gender::Male;

    // 阳男阴女顺行（返回true）
    // 阴男阳女逆行（返回false）
    (is_yang && is_male) || (!is_yang && !is_male)
}

// ============================================================================
// 星曜亮度
// ============================================================================

/// 14主星庙旺表常量（索引：星曜, 地支）
/// 值: 0=陷, 1=平, 2=得, 3=利, 4=旺, 5=庙
/// 顺序：子丑寅卯辰巳午未申酉戌亥
///
/// 数据来源：《紫微斗数全书》安星诀 / 三合派庙旺表
const ZHU_XING_BRIGHTNESS: [[u8; 12]; 14] = [
    // 紫微: 平庙旺旺陷旺庙庙旺平陷旺
    [1, 5, 4, 4, 0, 4, 5, 5, 4, 1, 0, 4],
    // 天机: 庙陷旺旺庙平庙陷平旺庙平
    [5, 0, 4, 4, 5, 1, 5, 0, 1, 4, 5, 1],
    // 太阳: 陷陷旺庙旺庙庙得得陷陷陷
    [0, 0, 4, 5, 4, 5, 5, 2, 2, 0, 0, 0],
    // 武曲: 旺庙利旺陷庙旺庙利旺陷庙
    [4, 5, 3, 4, 0, 5, 4, 5, 3, 4, 0, 5],
    // 天同: 旺旺陷庙平旺陷平陷庙平旺
    [4, 4, 0, 5, 1, 4, 0, 1, 0, 5, 1, 4],
    // 廉贞: 平庙陷平旺庙平庙陷平旺庙
    [1, 5, 0, 1, 4, 5, 1, 5, 0, 1, 4, 5],
    // 天府: 庙得旺庙庙旺庙得旺庙庙旺
    [5, 2, 4, 5, 5, 4, 5, 2, 4, 5, 5, 4],
    // 太阴: 庙庙得陷陷陷陷陷得旺庙庙
    [5, 5, 2, 0, 0, 0, 0, 0, 2, 4, 5, 5],
    // 贪狼: 庙平庙旺旺旺旺平庙旺旺旺
    [5, 1, 5, 4, 4, 4, 4, 1, 5, 4, 4, 4],
    // 巨门: 旺陷庙旺陷平平陷庙旺陷平
    [4, 0, 5, 4, 0, 1, 1, 0, 5, 4, 0, 1],
    // 天相: 庙得旺庙陷旺庙得旺庙陷旺
    [5, 2, 4, 5, 0, 4, 5, 2, 4, 5, 0, 4],
    // 天梁: 庙庙庙旺陷旺庙庙庙旺陷旺
    [5, 5, 5, 4, 0, 4, 5, 5, 5, 4, 0, 4],
    // 七杀: 庙平旺旺平庙旺平旺旺平庙
    [5, 1, 4, 4, 1, 5, 4, 1, 4, 4, 1, 5],
    // 破军: 旺陷庙旺陷平旺陷庙旺陷平
    [4, 0, 5, 4, 0, 1, 4, 0, 5, 4, 0, 1],
];

/// 六吉星庙旺表常量
/// 顺序：文昌、文曲、左辅、右弼、天魁、天钺
const LIU_JI_BRIGHTNESS: [[u8; 12]; 6] = [
    // 文昌: 平庙平庙陷陷陷庙平庙平平
    [1, 5, 1, 5, 0, 0, 0, 5, 1, 5, 1, 1],
    // 文曲: 平平庙庙陷陷庙平平庙陷平
    [1, 1, 5, 5, 0, 0, 5, 1, 1, 5, 0, 1],
    // 左辅: 庙庙庙庙庙庙庙庙庙庙庙庙 (全庙)
    [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5],
    // 右弼: 庙庙庙庙庙庙庙庙庙庙庙庙 (全庙)
    [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5],
    // 天魁: 平庙庙平平庙平平平旺平庙
    [1, 5, 5, 1, 1, 5, 1, 1, 1, 4, 1, 5],
    // 天钺: 庙平平庙庙平旺庙庙平平平
    [5, 1, 1, 5, 5, 1, 4, 5, 5, 1, 1, 1],
];

/// 六煞星庙旺表常量
/// 顺序：擎羊、陀罗、火星、铃星、地空、地劫
const LIU_SHA_BRIGHTNESS: [[u8; 12]; 6] = [
    // 擎羊: 陷陷庙陷陷旺陷陷庙陷陷旺
    [0, 0, 5, 0, 0, 4, 0, 0, 5, 0, 0, 4],
    // 陀罗: 旺陷陷庙旺陷陷庙旺陷陷庙
    [4, 0, 0, 5, 4, 0, 0, 5, 4, 0, 0, 5],
    // 火星: 利陷庙旺利陷庙旺利陷庙旺
    [3, 0, 5, 4, 3, 0, 5, 4, 3, 0, 5, 4],
    // 铃星: 利陷旺庙利陷旺庙利陷旺庙
    [3, 0, 4, 5, 3, 0, 4, 5, 3, 0, 4, 5],
    // 地空: 平平平平平平平平平平平平 (无庙旺)
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    // 地劫: 平平平平平平平平平平平平 (无庙旺)
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
];

/// 获取主星在某宫位的亮度（完整版）
///
/// # 参数
/// - star: 主星类型
/// - di_zhi: 宫位地支
///
/// # 返回
/// 星曜亮度
pub fn get_star_brightness(star: ZhuXing, di_zhi: DiZhi) -> StarBrightness {
    let zhi_idx = di_zhi.index() as usize;
    let star_idx = star as usize;

    if star_idx < ZHU_XING_BRIGHTNESS.len() {
        let brightness_val = ZHU_XING_BRIGHTNESS[star_idx][zhi_idx];
        StarBrightness::from_value(brightness_val)
    } else {
        StarBrightness::Ping
    }
}

/// 获取六吉星在某宫位的亮度
///
/// # 参数
/// - star_type: 0=文昌, 1=文曲, 2=左辅, 3=右弼, 4=天魁, 5=天钺
/// - di_zhi: 宫位地支
///
/// # 返回
/// 星曜亮度
pub fn get_liu_ji_brightness(star_type: u8, di_zhi: DiZhi) -> StarBrightness {
    let zhi_idx = di_zhi.index() as usize;
    if (star_type as usize) < LIU_JI_BRIGHTNESS.len() {
        let brightness_val = LIU_JI_BRIGHTNESS[star_type as usize][zhi_idx];
        StarBrightness::from_value(brightness_val)
    } else {
        StarBrightness::Ping
    }
}

/// 获取六煞星在某宫位的亮度
///
/// # 参数
/// - star_type: 0=擎羊, 1=陀罗, 2=火星, 3=铃星, 4=地空, 5=地劫
/// - di_zhi: 宫位地支
///
/// # 返回
/// 星曜亮度
pub fn get_liu_sha_brightness(star_type: u8, di_zhi: DiZhi) -> StarBrightness {
    let zhi_idx = di_zhi.index() as usize;
    if (star_type as usize) < LIU_SHA_BRIGHTNESS.len() {
        let brightness_val = LIU_SHA_BRIGHTNESS[star_type as usize][zhi_idx];
        StarBrightness::from_value(brightness_val)
    } else {
        StarBrightness::Ping
    }
}

// ============================================================================
// 综合排盘
// ============================================================================

/// 初始化十二宫结构
pub fn init_palaces(year_gan: TianGan, ming_gong_pos: u8) -> [Palace; 12] {
    let mut palaces: [Palace; 12] = Default::default();

    for i in 0..12 {
        let pos = i as u8;
        palaces[i].di_zhi = DiZhi::from_index(pos);
        palaces[i].tian_gan = get_gong_gan(year_gan, pos);

        // 计算宫位类型（从命宫开始顺排）
        let gong_idx = (pos + 12 - ming_gong_pos) % 12;
        palaces[i].gong_wei = GongWei::from_index(gong_idx);
    }

    palaces
}

// ============================================================================
// 天马安星
// ============================================================================

/// 安天马（根据年支）
///
/// # 口诀
/// 申子辰年马在寅，寅午戌年马在申，亥卯未年马在巳，巳酉丑年马在亥
///
/// # 三合局原理
/// 天马落在年支三合局的冲位：
/// - 申子辰三合水局，马星在寅（申冲寅）
/// - 寅午戌三合火局，马星在申（寅冲申）
/// - 亥卯未三合木局，马星在巳（亥冲巳）
/// - 巳酉丑三合金局，马星在亥（巳冲亥）
///
/// # 参数
/// - year_zhi: 年支
///
/// # 返回
/// 天马所在宫位索引（0-11）
pub fn calculate_tian_ma(year_zhi: DiZhi) -> u8 {
    let year_idx = year_zhi.index();

    match year_idx {
        // 申子辰年（8,0,4）→ 寅(2)
        8 | 0 | 4 => 2,
        // 寅午戌年（2,6,10）→ 申(8)
        2 | 6 | 10 => 8,
        // 亥卯未年（11,3,7）→ 巳(5)
        11 | 3 | 7 => 5,
        // 巳酉丑年（5,9,1）→ 亥(11)
        5 | 9 | 1 => 11,
        _ => 2, // 默认寅宫
    }
}

// ============================================================================
// 命主身主计算
// ============================================================================

/// 计算命主星（根据命宫地支）
///
/// # 口诀
/// 命宫地支定命主：
/// - 子宫 → 贪狼
/// - 丑亥宫 → 巨门
/// - 寅戌宫 → 禄存（此处返回 None，禄存属于辅星）
/// - 卯酉宫 → 文曲（此处返回 None，文曲属于辅星）
/// - 辰申宫 → 廉贞
/// - 巳未宫 → 武曲
/// - 午宫 → 破军
///
/// # 参数
/// - ming_gong_zhi: 命宫地支
///
/// # 返回
/// 命主星（主星类型，禄存/文曲返回 None）
pub fn calculate_ming_zhu(ming_gong_zhi: DiZhi) -> Option<ZhuXing> {
    match ming_gong_zhi {
        DiZhi::Zi => Some(ZhuXing::TanLang),
        DiZhi::Chou | DiZhi::Hai => Some(ZhuXing::JuMen),
        DiZhi::Yin | DiZhi::Xu => None, // 禄存（辅星，需特殊处理）
        DiZhi::Mao | DiZhi::You => None, // 文曲（辅星，需特殊处理）
        DiZhi::Chen | DiZhi::Shen => Some(ZhuXing::LianZhen),
        DiZhi::Si | DiZhi::Wei => Some(ZhuXing::WuQu),
        DiZhi::Wu => Some(ZhuXing::PoJun),
    }
}

/// 获取命主星名称（包含辅星）
///
/// # 参数
/// - ming_gong_zhi: 命宫地支
///
/// # 返回
/// 命主星名称
pub fn get_ming_zhu_name(ming_gong_zhi: DiZhi) -> &'static str {
    match ming_gong_zhi {
        DiZhi::Zi => "贪狼",
        DiZhi::Chou | DiZhi::Hai => "巨门",
        DiZhi::Yin | DiZhi::Xu => "禄存",
        DiZhi::Mao | DiZhi::You => "文曲",
        DiZhi::Chen | DiZhi::Shen => "廉贞",
        DiZhi::Si | DiZhi::Wei => "武曲",
        DiZhi::Wu => "破军",
    }
}

/// 计算身主星（根据年支）
///
/// # 口诀
/// 年支定身主：
/// - 子午年 → 火星
/// - 丑未年 → 天相
/// - 寅申年 → 天梁
/// - 卯酉年 → 天同
/// - 辰戌年 → 文昌
/// - 巳亥年 → 天机
///
/// # 参数
/// - year_zhi: 年支
///
/// # 返回
/// 身主星名称
pub fn get_shen_zhu_name(year_zhi: DiZhi) -> &'static str {
    match year_zhi {
        DiZhi::Zi | DiZhi::Wu => "火星",
        DiZhi::Chou | DiZhi::Wei => "天相",
        DiZhi::Yin | DiZhi::Shen => "天梁",
        DiZhi::Mao | DiZhi::You => "天同",
        DiZhi::Chen | DiZhi::Xu => "文昌",
        DiZhi::Si | DiZhi::Hai => "天机",
    }
}

/// 计算身主星（返回可能的主星类型）
///
/// # 参数
/// - year_zhi: 年支
///
/// # 返回
/// 身主星（主星类型，火星/文昌返回 None）
pub fn calculate_shen_zhu(year_zhi: DiZhi) -> Option<ZhuXing> {
    match year_zhi {
        DiZhi::Zi | DiZhi::Wu => None, // 火星（煞星）
        DiZhi::Chou | DiZhi::Wei => Some(ZhuXing::TianXiang),
        DiZhi::Yin | DiZhi::Shen => Some(ZhuXing::TianLiang),
        DiZhi::Mao | DiZhi::You => Some(ZhuXing::TianTong),
        DiZhi::Chen | DiZhi::Xu => None, // 文昌（辅星）
        DiZhi::Si | DiZhi::Hai => Some(ZhuXing::TianJi),
    }
}

// ============================================================================
// 博士十二星
// ============================================================================

/// 安博士十二星（从禄存起博士）
///
/// # 口诀
/// 禄存起博士，阳男阴女顺行，阴男阳女逆行
///
/// # 参数
/// - lu_cun_pos: 禄存位置（0-11）
/// - is_shun: 是否顺行
///
/// # 返回
/// 12个位置数组，索引对应博士十二星（0=博士, 1=力士, ... 11=官府）
pub fn calculate_bo_shi_stars(lu_cun_pos: u8, is_shun: bool) -> [u8; 12] {
    let mut positions = [0u8; 12];
    for i in 0..12 {
        if is_shun {
            positions[i] = (lu_cun_pos + i as u8) % 12;
        } else {
            positions[i] = (lu_cun_pos + 12 - i as u8) % 12;
        }
    }
    positions
}

/// 获取博士十二星在各宫的分布
///
/// # 参数
/// - lu_cun_pos: 禄存位置
/// - is_shun: 是否顺行
///
/// # 返回
/// 12元素数组，索引为宫位，值为所在星的索引（0-11），None表示无博士星
pub fn get_bo_shi_in_palaces(lu_cun_pos: u8, is_shun: bool) -> [Option<BoShiXing>; 12] {
    let positions = calculate_bo_shi_stars(lu_cun_pos, is_shun);
    let mut result = [None; 12];
    for (star_idx, &palace_pos) in positions.iter().enumerate() {
        result[palace_pos as usize] = Some(BoShiXing::from_index(star_idx as u8));
    }
    result
}

// ============================================================================
// 长生十二宫
// ============================================================================

/// 获取长生起点（根据五行局）
///
/// # 口诀
/// 水土局长生在申，木局长生在亥，金局长生在巳，火局长生在寅
///
/// # 参数
/// - wu_xing: 五行局
///
/// # 返回
/// 长生起点宫位（0-11）
pub fn calculate_chang_sheng_start(wu_xing: WuXing) -> u8 {
    match wu_xing {
        WuXing::Water | WuXing::Earth => 8, // 申
        WuXing::Wood => 11,                  // 亥
        WuXing::Metal => 5,                  // 巳
        WuXing::Fire => 2,                   // 寅
    }
}

/// 安长生十二宫
///
/// # 参数
/// - wu_xing: 五行局
/// - is_shun: 是否顺行（阳男阴女顺行，阴男阳女逆行）
///
/// # 返回
/// 12个位置数组，索引对应长生十二宫（0=长生, 1=沐浴, ... 11=养）
pub fn calculate_chang_sheng_positions(wu_xing: WuXing, is_shun: bool) -> [u8; 12] {
    let start = calculate_chang_sheng_start(wu_xing);
    let mut positions = [0u8; 12];
    for i in 0..12 {
        if is_shun {
            positions[i] = (start + i as u8) % 12;
        } else {
            positions[i] = (start + 12 - i as u8) % 12;
        }
    }
    positions
}

/// 获取长生十二宫在各宫的分布
///
/// # 参数
/// - wu_xing: 五行局
/// - is_shun: 是否顺行
///
/// # 返回
/// 12元素数组，索引为宫位，值为长生十二宫的枚举
pub fn get_chang_sheng_in_palaces(wu_xing: WuXing, is_shun: bool) -> [ChangSheng; 12] {
    let positions = calculate_chang_sheng_positions(wu_xing, is_shun);
    let mut result = [ChangSheng::ChangSheng; 12];
    for (cs_idx, &palace_pos) in positions.iter().enumerate() {
        result[palace_pos as usize] = ChangSheng::from_index(cs_idx as u8);
    }
    result
}

// ============================================================================
// 大限计算
// ============================================================================

/// 生成十二大限详情
///
/// # 参数
/// - ming_gong_pos: 命宫位置
/// - ju_shu: 五行局数（起运年龄）
/// - is_shun: 是否顺行（阳男阴女顺行，阴男阳女逆行）
/// - year_gan: 年干（用于计算各大限宫干）
///
/// # 返回
/// 12个大限信息元组：(序号, 起始年龄, 结束年龄, 宫位地支, 宫位天干)
pub fn generate_da_xian_details(
    ming_gong_pos: u8,
    ju_shu: u8,
    is_shun: bool,
    year_gan: TianGan,
) -> [(u8, u8, u8, DiZhi, TianGan); 12] {
    let mut da_xians = [(0u8, 0u8, 0u8, DiZhi::Zi, TianGan::Jia); 12];
    let mut current_age = ju_shu;

    for i in 0..12 {
        let gong_pos = if is_shun {
            (ming_gong_pos + i as u8) % 12
        } else {
            (ming_gong_pos + 12 - i as u8) % 12
        };

        da_xians[i] = (
            (i + 1) as u8,                        // 序号
            current_age,                           // 起始年龄
            current_age + 9,                       // 结束年龄
            DiZhi::from_index(gong_pos),          // 宫位地支
            get_gong_gan(year_gan, gong_pos),     // 宫位天干
        );

        current_age += 10;
    }

    da_xians
}
