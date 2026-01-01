//! # 八字神煞系统
//!
//! 本模块实现八字命理中的神煞计算，包括：
//! - 天乙贵人、太极贵人、文昌贵人等吉神
//! - 桃花、红鸾、天喜等婚姻神煞
//! - 羊刃、七杀、亡神、劫煞等凶神
//! - 华盖、将星、驿马等特殊神煞
//!
//! 参考 bazi-mcp 项目实现

use crate::types::*;
use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

// ================================
// 神煞类型定义
// ================================

/// 神煞类型枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ShenSha {
    // ========== 贵人类 ==========
    /// 天乙贵人 - 最大吉神，遇难呈祥
    TianYiGuiRen,
    /// 太极贵人 - 聪明好学，有宗教缘
    TaiJiGuiRen,
    /// 天德贵人 - 逢凶化吉，有贵人助
    TianDeGuiRen,
    /// 月德贵人 - 逢凶化吉，有贵人助
    YueDeGuiRen,
    /// 天德合 - 天德贵人之合
    TianDeHe,
    /// 月德合 - 月德贵人之合
    YueDeHe,
    /// 文昌贵人 - 聪明好学，利考试
    WenChangGuiRen,
    /// 福星贵人 - 福禄双全
    FuXingGuiRen,
    /// 国印贵人 - 官运亨通
    GuoYinGuiRen,

    // ========== 桃花婚姻类 ==========
    /// 桃花（咸池）- 异性缘佳
    TaoHua,
    /// 红鸾 - 婚姻吉星
    HongLuan,
    /// 天喜 - 婚姻喜庆
    TianXi,
    /// 孤辰 - 孤独之星
    GuChen,
    /// 寡宿 - 孤独之星
    GuaSu,

    // ========== 财官类 ==========
    /// 金舆 - 豪华座驾，富贵之象
    JinYu,
    /// 将星 - 领导才能
    JiangXing,
    /// 驿马 - 奔波走动
    YiMa,
    /// 华盖 - 聪明孤高
    HuaGai,
    /// 天厨 - 食禄之神
    TianChu,

    // ========== 凶神类 ==========
    /// 羊刃 - 刚强暴躁
    YangRen,
    /// 亡神 - 灾厄之星
    WangShen,
    /// 劫煞 - 劫难之星
    JieSha,
    /// 血刃 - 血光之灾
    XueRen,
    /// 元辰 - 小人之星
    YuanChen,

    // ========== 特殊类 ==========
    /// 天罗 - 困厄之象
    TianLuo,
    /// 地网 - 困厄之象
    DiWang,
    /// 童子煞 - 童子命
    TongZiSha,
    /// 九丑 - 相貌不佳
    JiuChou,
    /// 空亡 - 虚空之象
    KongWang,
}

impl ShenSha {
    /// 获取神煞名称
    pub fn name(&self) -> &'static str {
        match self {
            ShenSha::TianYiGuiRen => "天乙贵人",
            ShenSha::TaiJiGuiRen => "太极贵人",
            ShenSha::TianDeGuiRen => "天德贵人",
            ShenSha::YueDeGuiRen => "月德贵人",
            ShenSha::TianDeHe => "天德合",
            ShenSha::YueDeHe => "月德合",
            ShenSha::WenChangGuiRen => "文昌贵人",
            ShenSha::FuXingGuiRen => "福星贵人",
            ShenSha::GuoYinGuiRen => "国印贵人",
            ShenSha::TaoHua => "桃花",
            ShenSha::HongLuan => "红鸾",
            ShenSha::TianXi => "天喜",
            ShenSha::GuChen => "孤辰",
            ShenSha::GuaSu => "寡宿",
            ShenSha::JinYu => "金舆",
            ShenSha::JiangXing => "将星",
            ShenSha::YiMa => "驿马",
            ShenSha::HuaGai => "华盖",
            ShenSha::TianChu => "天厨",
            ShenSha::YangRen => "羊刃",
            ShenSha::WangShen => "亡神",
            ShenSha::JieSha => "劫煞",
            ShenSha::XueRen => "血刃",
            ShenSha::YuanChen => "元辰",
            ShenSha::TianLuo => "天罗",
            ShenSha::DiWang => "地网",
            ShenSha::TongZiSha => "童子煞",
            ShenSha::JiuChou => "九丑",
            ShenSha::KongWang => "空亡",
        }
    }

    /// 判断是否为吉神
    pub fn is_auspicious(&self) -> bool {
        matches!(
            self,
            ShenSha::TianYiGuiRen
                | ShenSha::TaiJiGuiRen
                | ShenSha::TianDeGuiRen
                | ShenSha::YueDeGuiRen
                | ShenSha::TianDeHe
                | ShenSha::YueDeHe
                | ShenSha::WenChangGuiRen
                | ShenSha::FuXingGuiRen
                | ShenSha::GuoYinGuiRen
                | ShenSha::HongLuan
                | ShenSha::TianXi
                | ShenSha::JinYu
                | ShenSha::JiangXing
                | ShenSha::TianChu
        )
    }

    /// 判断是否为凶神
    pub fn is_inauspicious(&self) -> bool {
        matches!(
            self,
            ShenSha::YangRen
                | ShenSha::WangShen
                | ShenSha::JieSha
                | ShenSha::XueRen
                | ShenSha::YuanChen
                | ShenSha::TianLuo
                | ShenSha::DiWang
                | ShenSha::TongZiSha
                | ShenSha::JiuChou
        )
    }
}

// ================================
// 神煞常量表
// ================================

/// 天乙贵人表
/// 格式：TIANYI_GUIREN[日干] = [贵人地支1, 贵人地支2]
/// 口诀：甲戊庚牛羊，乙己鼠猴乡，丙丁猪鸡位，壬癸兔蛇藏，六辛逢马虎
pub const TIANYI_GUIREN: [[u8; 2]; 10] = [
    [1, 7],   // 甲: 丑未
    [0, 8],   // 乙: 子申
    [11, 9],  // 丙: 亥酉
    [11, 9],  // 丁: 亥酉
    [1, 7],   // 戊: 丑未
    [0, 8],   // 己: 子申
    [1, 7],   // 庚: 丑未
    [2, 6],   // 辛: 寅午
    [3, 5],   // 壬: 卯巳
    [3, 5],   // 癸: 卯巳
];

/// 太极贵人表
/// 格式：TAIJI_GUIREN[日干] = [贵人地支1, 贵人地支2]
pub const TAIJI_GUIREN: [[u8; 2]; 10] = [
    [0, 6],   // 甲: 子午
    [0, 6],   // 乙: 子午
    [3, 9],   // 丙: 卯酉
    [3, 9],   // 丁: 卯酉
    [4, 10],  // 戊: 辰戌
    [4, 10],  // 己: 辰戌
    [1, 7],   // 庚: 丑未
    [1, 7],   // 辛: 丑未
    [2, 11],  // 壬: 寅亥
    [2, 11],  // 癸: 寅亥
];

/// 文昌贵人表
/// 格式：WENCHANG_GUIREN[日干] = 文昌地支
/// 口诀：甲乙巳午报，申宫丙戊庚；丁己鸡同位，壬辛寅虎生；癸兔见人喜
pub const WENCHANG_GUIREN: [u8; 10] = [
    5,  // 甲: 巳
    6,  // 乙: 午
    8,  // 丙: 申
    9,  // 丁: 酉
    8,  // 戊: 申
    9,  // 己: 酉
    11, // 庚: 亥
    0,  // 辛: 子
    2,  // 壬: 寅
    3,  // 癸: 卯
];

/// 天德贵人表
/// 格式：TIANDE_GUIREN[月支] = 天德天干或地支（用0-9表示天干，10-21表示地支）
/// 注：正月丁，二月申，三月壬，四月辛...
pub const TIANDE_GUIREN: [u8; 12] = [
    3,   // 寅月(正月): 丁(天干3)
    18,  // 卯月(二月): 申(地支8+10=18)
    8,   // 辰月(三月): 壬(天干8)
    7,   // 巳月(四月): 辛(天干7)
    21,  // 午月(五月): 亥(地支11+10=21)
    0,   // 未月(六月): 甲(天干0)
    9,   // 申月(七月): 癸(天干9)
    12,  // 酉月(八月): 寅(地支2+10=12)
    2,   // 戌月(九月): 丙(天干2)
    1,   // 亥月(十月): 乙(天干1)
    15,  // 子月(十一月): 巳(地支5+10=15)
    6,   // 丑月(十二月): 庚(天干6)
];

/// 月德贵人表
/// 格式：YUEDE_GUIREN[月支] = 月德天干
pub const YUEDE_GUIREN: [u8; 12] = [
    2,  // 寅月: 丙
    0,  // 卯月: 甲
    8,  // 辰月: 壬
    6,  // 巳月: 庚
    2,  // 午月: 丙
    0,  // 未月: 甲
    8,  // 申月: 壬
    6,  // 酉月: 庚
    2,  // 戌月: 丙
    0,  // 亥月: 甲
    8,  // 子月: 壬
    6,  // 丑月: 庚
];

/// 桃花（咸池）表
/// 格式：TAOHUA[年支或日支所属三合局首支] = 桃花地支
/// 申子辰见酉，亥卯未见子，寅午戌见卯，巳酉丑见午
pub const TAOHUA: [u8; 4] = [
    9,  // 申子辰水局: 酉
    0,  // 亥卯未木局: 子
    3,  // 寅午戌火局: 卯
    6,  // 巳酉丑金局: 午
];

/// 华盖表
/// 格式：HUAGAI[年支或日支所属三合局] = 华盖地支
/// 申子辰见辰，亥卯未见未，寅午戌见戌，巳酉丑见丑
pub const HUAGAI: [u8; 4] = [
    4,   // 申子辰: 辰
    7,   // 亥卯未: 未
    10,  // 寅午戌: 戌
    1,   // 巳酉丑: 丑
];

/// 将星表
/// 格式：JIANGXING[年支或日支所属三合局] = 将星地支
/// 申子辰见子，亥卯未见卯，寅午戌见午，巳酉丑见酉
pub const JIANGXING: [u8; 4] = [
    0,  // 申子辰: 子
    3,  // 亥卯未: 卯
    6,  // 寅午戌: 午
    9,  // 巳酉丑: 酉
];

/// 驿马表
/// 格式：YIMA[年支或日支所属三合局] = 驿马地支
/// 申子辰见寅，亥卯未见巳，寅午戌见申，巳酉丑见亥
pub const YIMA: [u8; 4] = [
    2,   // 申子辰: 寅
    5,   // 亥卯未: 巳
    8,   // 寅午戌: 申
    11,  // 巳酉丑: 亥
];

/// 劫煞表
/// 格式：JIESHA[年支或日支所属三合局] = 劫煞地支
/// 申子辰见巳，亥卯未见申，寅午戌见亥，巳酉丑见寅
pub const JIESHA: [u8; 4] = [
    5,   // 申子辰: 巳
    8,   // 亥卯未: 申
    11,  // 寅午戌: 亥
    2,   // 巳酉丑: 寅
];

/// 亡神表
/// 格式：WANGSHEN[年支或日支所属三合局] = 亡神地支
/// 申子辰见亥，亥卯未见寅，寅午戌见巳，巳酉丑见申
pub const WANGSHEN: [u8; 4] = [
    11,  // 申子辰: 亥
    2,   // 亥卯未: 寅
    5,   // 寅午戌: 巳
    8,   // 巳酉丑: 申
];

/// 羊刃表
/// 格式：YANGREN[日干] = 羊刃地支
/// 甲刃在卯，乙刃在辰，丙戊刃在午，丁己刃在未，庚刃在酉，辛刃在戌，壬刃在子，癸刃在丑
pub const YANGREN: [u8; 10] = [
    3,   // 甲: 卯
    4,   // 乙: 辰
    6,   // 丙: 午
    7,   // 丁: 未
    6,   // 戊: 午
    7,   // 己: 未
    9,   // 庚: 酉
    10,  // 辛: 戌
    0,   // 壬: 子
    1,   // 癸: 丑
];

/// 红鸾表
/// 格式：HONGLUAN[年支] = 红鸾地支
pub const HONGLUAN: [u8; 12] = [
    3,   // 子年: 卯
    2,   // 丑年: 寅
    1,   // 寅年: 丑
    0,   // 卯年: 子
    11,  // 辰年: 亥
    10,  // 巳年: 戌
    9,   // 午年: 酉
    8,   // 未年: 申
    7,   // 申年: 未
    6,   // 酉年: 午
    5,   // 戌年: 巳
    4,   // 亥年: 辰
];

/// 天喜表
/// 格式：TIANXI[年支] = 天喜地支（红鸾对冲）
pub const TIANXI: [u8; 12] = [
    9,   // 子年: 酉
    8,   // 丑年: 申
    7,   // 寅年: 未
    6,   // 卯年: 午
    5,   // 辰年: 巳
    4,   // 巳年: 辰
    3,   // 午年: 卯
    2,   // 未年: 寅
    1,   // 申年: 丑
    0,   // 酉年: 子
    11,  // 戌年: 亥
    10,  // 亥年: 戌
];

/// 国印贵人表
/// 格式：GUOYIN[年支] = 国印地支
pub const GUOYIN: [u8; 12] = [
    10,  // 子年: 戌
    11,  // 丑年: 亥
    1,   // 寅年: 丑
    0,   // 卯年: 子
    2,   // 辰年: 寅
    3,   // 巳年: 卯
    5,   // 午年: 巳
    4,   // 未年: 辰
    6,   // 申年: 午
    7,   // 酉年: 未
    9,   // 戌年: 酉
    8,   // 亥年: 申
];

/// 金舆表
/// 格式：JINYU[日干] = 金舆地支
pub const JINYU: [u8; 10] = [
    4,   // 甲: 辰
    5,   // 乙: 巳
    7,   // 丙: 未
    8,   // 丁: 申
    7,   // 戊: 未
    8,   // 己: 申
    10,  // 庚: 戌
    11,  // 辛: 亥
    1,   // 壬: 丑
    2,   // 癸: 寅
];

// ================================
// 神煞计算函数
// ================================

/// 获取地支所属三合局索引
/// 返回：0=申子辰(水), 1=亥卯未(木), 2=寅午戌(火), 3=巳酉丑(金)
fn get_sanhe_index(zhi: u8) -> u8 {
    match zhi {
        8 | 0 | 4 => 0,    // 申子辰 - 水局
        11 | 3 | 7 => 1,   // 亥卯未 - 木局
        2 | 6 | 10 => 2,   // 寅午戌 - 火局
        5 | 9 | 1 => 3,    // 巳酉丑 - 金局
        _ => 0,
    }
}

/// 计算天乙贵人
pub fn calculate_tianyi_guiren(day_gan: TianGan, zhi: DiZhi) -> bool {
    let guiren = TIANYI_GUIREN[day_gan.0 as usize];
    zhi.0 == guiren[0] || zhi.0 == guiren[1]
}

/// 计算太极贵人
pub fn calculate_taiji_guiren(day_gan: TianGan, zhi: DiZhi) -> bool {
    let guiren = TAIJI_GUIREN[day_gan.0 as usize];
    zhi.0 == guiren[0] || zhi.0 == guiren[1]
}

/// 计算文昌贵人
pub fn calculate_wenchang_guiren(day_gan: TianGan, zhi: DiZhi) -> bool {
    WENCHANG_GUIREN[day_gan.0 as usize] == zhi.0
}

/// 计算天德贵人
pub fn calculate_tiande_guiren(month_zhi: DiZhi, gan: TianGan, zhi: DiZhi) -> bool {
    let tiande = TIANDE_GUIREN[month_zhi.0 as usize];
    if tiande < 10 {
        // 天干
        gan.0 == tiande
    } else {
        // 地支
        zhi.0 == tiande - 10
    }
}

/// 计算月德贵人
pub fn calculate_yuede_guiren(month_zhi: DiZhi, gan: TianGan) -> bool {
    YUEDE_GUIREN[month_zhi.0 as usize] == gan.0
}

/// 计算桃花
pub fn calculate_taohua(year_zhi: DiZhi, target_zhi: DiZhi) -> bool {
    let sanhe_idx = get_sanhe_index(year_zhi.0);
    TAOHUA[sanhe_idx as usize] == target_zhi.0
}

/// 计算华盖
pub fn calculate_huagai(year_zhi: DiZhi, target_zhi: DiZhi) -> bool {
    let sanhe_idx = get_sanhe_index(year_zhi.0);
    HUAGAI[sanhe_idx as usize] == target_zhi.0
}

/// 计算将星
pub fn calculate_jiangxing(year_zhi: DiZhi, target_zhi: DiZhi) -> bool {
    let sanhe_idx = get_sanhe_index(year_zhi.0);
    JIANGXING[sanhe_idx as usize] == target_zhi.0
}

/// 计算驿马
pub fn calculate_yima(year_zhi: DiZhi, target_zhi: DiZhi) -> bool {
    let sanhe_idx = get_sanhe_index(year_zhi.0);
    YIMA[sanhe_idx as usize] == target_zhi.0
}

/// 计算劫煞
pub fn calculate_jiesha(year_zhi: DiZhi, target_zhi: DiZhi) -> bool {
    let sanhe_idx = get_sanhe_index(year_zhi.0);
    JIESHA[sanhe_idx as usize] == target_zhi.0
}

/// 计算亡神
pub fn calculate_wangshen(year_zhi: DiZhi, target_zhi: DiZhi) -> bool {
    let sanhe_idx = get_sanhe_index(year_zhi.0);
    WANGSHEN[sanhe_idx as usize] == target_zhi.0
}

/// 计算羊刃
pub fn calculate_yangren(day_gan: TianGan, zhi: DiZhi) -> bool {
    YANGREN[day_gan.0 as usize] == zhi.0
}

/// 计算红鸾
pub fn calculate_hongluan(year_zhi: DiZhi, target_zhi: DiZhi) -> bool {
    HONGLUAN[year_zhi.0 as usize] == target_zhi.0
}

/// 计算天喜
pub fn calculate_tianxi(year_zhi: DiZhi, target_zhi: DiZhi) -> bool {
    TIANXI[year_zhi.0 as usize] == target_zhi.0
}

/// 计算国印贵人
pub fn calculate_guoyin(year_zhi: DiZhi, target_zhi: DiZhi) -> bool {
    GUOYIN[year_zhi.0 as usize] == target_zhi.0
}

/// 计算金舆
pub fn calculate_jinyu(day_gan: TianGan, zhi: DiZhi) -> bool {
    JINYU[day_gan.0 as usize] == zhi.0
}

/// 单柱神煞信息
#[derive(Clone, Debug, Default, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ZhuShenSha {
    /// 该柱包含的神煞列表（最多10个）
    pub shensha_list: BoundedVec<ShenSha, ConstU32<10>>,
}

/// 四柱神煞信息
#[derive(Clone, Debug, Default, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct SiZhuShenSha {
    /// 年柱神煞
    pub year_shensha: ZhuShenSha,
    /// 月柱神煞
    pub month_shensha: ZhuShenSha,
    /// 日柱神煞
    pub day_shensha: ZhuShenSha,
    /// 时柱神煞
    pub hour_shensha: ZhuShenSha,
}

/// 计算单柱神煞
///
/// ## 参数
/// - `year_zhi`: 年支
/// - `month_zhi`: 月支
/// - `day_gan`: 日干
/// - `target_gan`: 目标柱天干
/// - `target_zhi`: 目标柱地支
///
/// ## 返回
/// - 该柱包含的神煞列表
pub fn calculate_zhu_shensha(
    year_zhi: DiZhi,
    month_zhi: DiZhi,
    day_gan: TianGan,
    target_gan: TianGan,
    target_zhi: DiZhi,
) -> ZhuShenSha {
    let mut shensha_list = BoundedVec::<ShenSha, ConstU32<10>>::default();

    // 检查天乙贵人
    if calculate_tianyi_guiren(day_gan, target_zhi) {
        let _ = shensha_list.try_push(ShenSha::TianYiGuiRen);
    }

    // 检查太极贵人
    if calculate_taiji_guiren(day_gan, target_zhi) {
        let _ = shensha_list.try_push(ShenSha::TaiJiGuiRen);
    }

    // 检查文昌贵人
    if calculate_wenchang_guiren(day_gan, target_zhi) {
        let _ = shensha_list.try_push(ShenSha::WenChangGuiRen);
    }

    // 检查天德贵人
    if calculate_tiande_guiren(month_zhi, target_gan, target_zhi) {
        let _ = shensha_list.try_push(ShenSha::TianDeGuiRen);
    }

    // 检查月德贵人
    if calculate_yuede_guiren(month_zhi, target_gan) {
        let _ = shensha_list.try_push(ShenSha::YueDeGuiRen);
    }

    // 检查桃花
    if calculate_taohua(year_zhi, target_zhi) {
        let _ = shensha_list.try_push(ShenSha::TaoHua);
    }

    // 检查华盖
    if calculate_huagai(year_zhi, target_zhi) {
        let _ = shensha_list.try_push(ShenSha::HuaGai);
    }

    // 检查将星
    if calculate_jiangxing(year_zhi, target_zhi) {
        let _ = shensha_list.try_push(ShenSha::JiangXing);
    }

    // 检查驿马
    if calculate_yima(year_zhi, target_zhi) {
        let _ = shensha_list.try_push(ShenSha::YiMa);
    }

    // 检查羊刃
    if calculate_yangren(day_gan, target_zhi) {
        let _ = shensha_list.try_push(ShenSha::YangRen);
    }

    // 检查红鸾
    if calculate_hongluan(year_zhi, target_zhi) {
        let _ = shensha_list.try_push(ShenSha::HongLuan);
    }

    // 检查天喜
    if calculate_tianxi(year_zhi, target_zhi) {
        let _ = shensha_list.try_push(ShenSha::TianXi);
    }

    // 检查国印
    if calculate_guoyin(year_zhi, target_zhi) {
        let _ = shensha_list.try_push(ShenSha::GuoYinGuiRen);
    }

    // 检查金舆
    if calculate_jinyu(day_gan, target_zhi) {
        let _ = shensha_list.try_push(ShenSha::JinYu);
    }

    // 检查劫煞
    if calculate_jiesha(year_zhi, target_zhi) {
        let _ = shensha_list.try_push(ShenSha::JieSha);
    }

    // 检查亡神
    if calculate_wangshen(year_zhi, target_zhi) {
        let _ = shensha_list.try_push(ShenSha::WangShen);
    }

    ZhuShenSha { shensha_list }
}

/// 计算四柱完整神煞
pub fn calculate_sizhu_shensha(
    year_ganzhi: &GanZhi,
    month_ganzhi: &GanZhi,
    day_ganzhi: &GanZhi,
    hour_ganzhi: &GanZhi,
) -> SiZhuShenSha {
    let year_zhi = year_ganzhi.zhi;
    let month_zhi = month_ganzhi.zhi;
    let day_gan = day_ganzhi.gan;

    SiZhuShenSha {
        year_shensha: calculate_zhu_shensha(
            year_zhi,
            month_zhi,
            day_gan,
            year_ganzhi.gan,
            year_ganzhi.zhi,
        ),
        month_shensha: calculate_zhu_shensha(
            year_zhi,
            month_zhi,
            day_gan,
            month_ganzhi.gan,
            month_ganzhi.zhi,
        ),
        day_shensha: calculate_zhu_shensha(
            year_zhi,
            month_zhi,
            day_gan,
            day_ganzhi.gan,
            day_ganzhi.zhi,
        ),
        hour_shensha: calculate_zhu_shensha(
            year_zhi,
            month_zhi,
            day_gan,
            hour_ganzhi.gan,
            hour_ganzhi.zhi,
        ),
    }
}

/// 计算四柱的神煞列表（Runtime API 专用）
///
/// # 参数
///
/// - `sizhu`: 四柱信息
///
/// # 返回
///
/// 神煞条目列表，包含神煞类型、位置、吉凶属性
///
/// # 示例
///
/// ```ignore
/// let shensha_list = calculate_shensha_list(&sizhu);
/// for entry in shensha_list {
///     println!("{:?}出现在{:?}，属性：{:?}",
///         entry.shensha.name(),
///         entry.position.name(),
///         entry.nature.name()
///     );
/// }
/// ```
pub fn calculate_shensha_list<T: crate::pallet::Config>(
    sizhu: &crate::types::SiZhu<T>,
) -> sp_std::vec::Vec<crate::types::ShenShaEntry> {
    use crate::types::{ShenShaEntry, ShenShaNature, SiZhuPosition};

    let year_ganzhi = &sizhu.year_zhu.ganzhi;
    let month_ganzhi = &sizhu.month_zhu.ganzhi;
    let day_ganzhi = &sizhu.day_zhu.ganzhi;
    let hour_ganzhi = &sizhu.hour_zhu.ganzhi;

    // 计算四柱神煞
    let sizhu_shensha = calculate_sizhu_shensha(
        year_ganzhi,
        month_ganzhi,
        day_ganzhi,
        hour_ganzhi,
    );

    let mut result = sp_std::vec::Vec::new();

    // 年柱神煞
    for shensha in sizhu_shensha.year_shensha.shensha_list.iter() {
        let nature = if shensha.is_auspicious() {
            ShenShaNature::JiShen
        } else if shensha.is_inauspicious() {
            ShenShaNature::XiongShen
        } else {
            ShenShaNature::Neutral
        };

        result.push(ShenShaEntry {
            shensha: *shensha,
            position: SiZhuPosition::Year,
            nature,
        });
    }

    // 月柱神煞
    for shensha in sizhu_shensha.month_shensha.shensha_list.iter() {
        let nature = if shensha.is_auspicious() {
            ShenShaNature::JiShen
        } else if shensha.is_inauspicious() {
            ShenShaNature::XiongShen
        } else {
            ShenShaNature::Neutral
        };

        result.push(ShenShaEntry {
            shensha: *shensha,
            position: SiZhuPosition::Month,
            nature,
        });
    }

    // 日柱神煞
    for shensha in sizhu_shensha.day_shensha.shensha_list.iter() {
        let nature = if shensha.is_auspicious() {
            ShenShaNature::JiShen
        } else if shensha.is_inauspicious() {
            ShenShaNature::XiongShen
        } else {
            ShenShaNature::Neutral
        };

        result.push(ShenShaEntry {
            shensha: *shensha,
            position: SiZhuPosition::Day,
            nature,
        });
    }

    // 时柱神煞
    for shensha in sizhu_shensha.hour_shensha.shensha_list.iter() {
        let nature = if shensha.is_auspicious() {
            ShenShaNature::JiShen
        } else if shensha.is_inauspicious() {
            ShenShaNature::XiongShen
        } else {
            ShenShaNature::Neutral
        };

        result.push(ShenShaEntry {
            shensha: *shensha,
            position: SiZhuPosition::Hour,
            nature,
        });
    }

    result
}

/// 计算临时四柱的神煞列表（不使用泛型）
///
/// # 参数
///
/// - `year_ganzhi`: 年柱干支
/// - `month_ganzhi`: 月柱干支
/// - `day_ganzhi`: 日柱干支
/// - `hour_ganzhi`: 时柱干支
///
/// # 返回
///
/// 神煞条目列表
///
/// # 用途
///
/// 供临时排盘接口使用，避免泛型依赖
pub fn calculate_shensha_list_temp(
    year_ganzhi: &crate::types::GanZhi,
    month_ganzhi: &crate::types::GanZhi,
    day_ganzhi: &crate::types::GanZhi,
    hour_ganzhi: &crate::types::GanZhi,
) -> sp_std::vec::Vec<crate::types::ShenShaEntry> {
    use crate::types::{ShenShaEntry, ShenShaNature, SiZhuPosition};

    // 计算四柱神煞
    let sizhu_shensha = calculate_sizhu_shensha(
        year_ganzhi,
        month_ganzhi,
        day_ganzhi,
        hour_ganzhi,
    );

    let mut result = sp_std::vec::Vec::new();

    // 年柱神煞
    for shensha in sizhu_shensha.year_shensha.shensha_list.iter() {
        let nature = if shensha.is_auspicious() {
            ShenShaNature::JiShen
        } else if shensha.is_inauspicious() {
            ShenShaNature::XiongShen
        } else {
            ShenShaNature::Neutral
        };

        result.push(ShenShaEntry {
            shensha: *shensha,
            position: SiZhuPosition::Year,
            nature,
        });
    }

    // 月柱神煞
    for shensha in sizhu_shensha.month_shensha.shensha_list.iter() {
        let nature = if shensha.is_auspicious() {
            ShenShaNature::JiShen
        } else if shensha.is_inauspicious() {
            ShenShaNature::XiongShen
        } else {
            ShenShaNature::Neutral
        };

        result.push(ShenShaEntry {
            shensha: *shensha,
            position: SiZhuPosition::Month,
            nature,
        });
    }

    // 日柱神煞
    for shensha in sizhu_shensha.day_shensha.shensha_list.iter() {
        let nature = if shensha.is_auspicious() {
            ShenShaNature::JiShen
        } else if shensha.is_inauspicious() {
            ShenShaNature::XiongShen
        } else {
            ShenShaNature::Neutral
        };

        result.push(ShenShaEntry {
            shensha: *shensha,
            position: SiZhuPosition::Day,
            nature,
        });
    }

    // 时柱神煞
    for shensha in sizhu_shensha.hour_shensha.shensha_list.iter() {
        let nature = if shensha.is_auspicious() {
            ShenShaNature::JiShen
        } else if shensha.is_inauspicious() {
            ShenShaNature::XiongShen
        } else {
            ShenShaNature::Neutral
        };

        result.push(ShenShaEntry {
            shensha: *shensha,
            position: SiZhuPosition::Hour,
            nature,
        });
    }

    result
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tianyi_guiren() {
        // 甲日见丑或未为天乙贵人
        let jia = TianGan(0);
        assert!(calculate_tianyi_guiren(jia, DiZhi(1)));  // 丑
        assert!(calculate_tianyi_guiren(jia, DiZhi(7)));  // 未
        assert!(!calculate_tianyi_guiren(jia, DiZhi(0))); // 子不是
    }

    #[test]
    fn test_taohua() {
        // 申子辰见酉为桃花
        assert!(calculate_taohua(DiZhi(0), DiZhi(9)));  // 子年见酉
        assert!(calculate_taohua(DiZhi(8), DiZhi(9)));  // 申年见酉
        assert!(calculate_taohua(DiZhi(4), DiZhi(9)));  // 辰年见酉
    }

    #[test]
    fn test_yangren() {
        // 甲刃在卯
        assert!(calculate_yangren(TianGan(0), DiZhi(3)));
        // 丙刃在午
        assert!(calculate_yangren(TianGan(2), DiZhi(6)));
    }

    #[test]
    fn test_shensha_is_auspicious() {
        assert!(ShenSha::TianYiGuiRen.is_auspicious());
        assert!(ShenSha::HongLuan.is_auspicious());
        assert!(!ShenSha::YangRen.is_auspicious());
    }
}
