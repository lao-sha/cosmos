//! # 神煞计算模块
//!
//! 本模块实现六爻排盘中常用的神煞计算，包括：
//! - 天乙贵人
//! - 驿马
//! - 桃花
//! - 禄神
//! - 文昌
//! - 劫煞
//! - 华盖
//! - 将星
//!
//! ## 神煞说明
//!
//! 神煞是中国传统命理学中的重要概念，用于辅助判断吉凶。
//! 在六爻占卜中，神煞可以帮助判断爻的特殊含义。

use crate::types::{DiZhi, TianGan};
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

// ============================================================================
// 神煞类型定义
// ============================================================================

/// 神煞类型枚举
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub enum ShenSha {
    /// 天乙贵人 - 最大吉神，主贵人相助
    TianYiGuiRen = 0,
    /// 驿马 - 主奔波、变动、出行
    YiMa = 1,
    /// 桃花 - 主感情、人缘、桃色
    TaoHua = 2,
    /// 禄神 - 主财禄、俸禄
    LuShen = 3,
    /// 文昌 - 主文才、考试、学业
    WenChang = 4,
    /// 劫煞 - 凶煞，主灾祸、劫难
    JieSha = 5,
    /// 华盖 - 主孤独、艺术、宗教
    HuaGai = 6,
    /// 将星 - 主权威、领导
    JiangXing = 7,
    /// 亡神 - 主破败、损失
    WangShen = 8,
    /// 天喜 - 主喜庆、婚姻
    TianXi = 9,
    /// 天医 - 主医药、治疗
    TianYi = 10,
    /// 阳刃 - 主刚烈、凶险
    YangRen = 11,
    /// 灾煞 - 主灾难、血光
    ZaiSha = 12,
    /// 谋星 - 主谋划、策略
    MouXing = 13,
}

impl ShenSha {
    /// 获取神煞名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::TianYiGuiRen => "天乙贵人",
            Self::YiMa => "驿马",
            Self::TaoHua => "桃花",
            Self::LuShen => "禄神",
            Self::WenChang => "文昌",
            Self::JieSha => "劫煞",
            Self::HuaGai => "华盖",
            Self::JiangXing => "将星",
            Self::WangShen => "亡神",
            Self::TianXi => "天喜",
            Self::TianYi => "天医",
            Self::YangRen => "阳刃",
            Self::ZaiSha => "灾煞",
            Self::MouXing => "谋星",
        }
    }

    /// 判断是否为吉神
    pub fn is_auspicious(&self) -> bool {
        matches!(
            self,
            Self::TianYiGuiRen | Self::LuShen | Self::WenChang | Self::JiangXing | Self::TianXi | Self::TianYi
        )
    }

    /// 判断是否为凶煞
    pub fn is_inauspicious(&self) -> bool {
        matches!(self, Self::JieSha | Self::WangShen | Self::YangRen | Self::ZaiSha)
    }
}

// ============================================================================
// 天乙贵人
// ============================================================================

/// 天乙贵人查询表（按日干）
///
/// 口诀：
/// 甲戊庚牛羊，乙己鼠猴乡，
/// 丙丁猪鸡位，壬癸兔蛇藏，
/// 六辛逢马虎，此是贵人方。
const TIAN_YI_GUI_REN: [[DiZhi; 2]; 10] = [
    [DiZhi::Chou, DiZhi::Wei],  // 甲 - 丑未
    [DiZhi::Zi, DiZhi::Shen],   // 乙 - 子申
    [DiZhi::Hai, DiZhi::You],   // 丙 - 亥酉
    [DiZhi::Hai, DiZhi::You],   // 丁 - 亥酉
    [DiZhi::Chou, DiZhi::Wei],  // 戊 - 丑未
    [DiZhi::Zi, DiZhi::Shen],   // 己 - 子申
    [DiZhi::Chou, DiZhi::Wei],  // 庚 - 丑未
    [DiZhi::Wu, DiZhi::Yin],    // 辛 - 午寅
    [DiZhi::Mao, DiZhi::Si],    // 壬 - 卯巳
    [DiZhi::Mao, DiZhi::Si],    // 癸 - 卯巳
];

/// 计算天乙贵人
///
/// 天乙贵人是最大的吉神，主贵人相助、逢凶化吉。
///
/// # 参数
/// - `day_gan`: 日干
///
/// # 返回
/// 两个贵人地支（昼贵人和夜贵人）
pub fn calculate_tian_yi_gui_ren(day_gan: TianGan) -> [DiZhi; 2] {
    TIAN_YI_GUI_REN[day_gan.index() as usize]
}

/// 判断地支是否为天乙贵人
pub fn is_tian_yi_gui_ren(day_gan: TianGan, zhi: DiZhi) -> bool {
    let gui_ren = calculate_tian_yi_gui_ren(day_gan);
    gui_ren[0] == zhi || gui_ren[1] == zhi
}

// ============================================================================
// 驿马
// ============================================================================

/// 驿马查询表（按日支/年支三合局）
///
/// 口诀：
/// 申子辰马在寅，寅午戌马在申，
/// 巳酉丑马在亥，亥卯未马在巳。
///
/// 驿马取三合局的冲位
const YI_MA: [DiZhi; 12] = [
    DiZhi::Yin,  // 子 -> 寅（申子辰三合水局，冲寅）
    DiZhi::Hai,  // 丑 -> 亥（巳酉丑三合金局，冲亥）
    DiZhi::Shen, // 寅 -> 申（寅午戌三合火局，冲申）
    DiZhi::Si,   // 卯 -> 巳（亥卯未三合木局，冲巳）
    DiZhi::Yin,  // 辰 -> 寅（申子辰三合水局，冲寅）
    DiZhi::Hai,  // 巳 -> 亥（巳酉丑三合金局，冲亥）
    DiZhi::Shen, // 午 -> 申（寅午戌三合火局，冲申）
    DiZhi::Si,   // 未 -> 巳（亥卯未三合木局，冲巳）
    DiZhi::Yin,  // 申 -> 寅（申子辰三合水局，冲寅）
    DiZhi::Hai,  // 酉 -> 亥（巳酉丑三合金局，冲亥）
    DiZhi::Shen, // 戌 -> 申（寅午戌三合火局，冲申）
    DiZhi::Si,   // 亥 -> 巳（亥卯未三合木局，冲巳）
];

/// 计算驿马
///
/// 驿马主奔波、变动、出行、迁移。
///
/// # 参数
/// - `day_zhi`: 日支（或年支）
///
/// # 返回
/// 驿马地支
pub fn calculate_yi_ma(day_zhi: DiZhi) -> DiZhi {
    YI_MA[day_zhi.index() as usize]
}

/// 判断地支是否为驿马
pub fn is_yi_ma(day_zhi: DiZhi, zhi: DiZhi) -> bool {
    calculate_yi_ma(day_zhi) == zhi
}

// ============================================================================
// 桃花（咸池）
// ============================================================================

/// 桃花查询表（按日支/年支三合局）
///
/// 口诀：
/// 申子辰桃花在酉，寅午戌桃花在卯，
/// 巳酉丑桃花在午，亥卯未桃花在子。
///
/// 桃花取三合局的沐浴位
const TAO_HUA: [DiZhi; 12] = [
    DiZhi::You,  // 子 -> 酉（申子辰三合水局）
    DiZhi::Wu,   // 丑 -> 午（巳酉丑三合金局）
    DiZhi::Mao,  // 寅 -> 卯（寅午戌三合火局）
    DiZhi::Zi,   // 卯 -> 子（亥卯未三合木局）
    DiZhi::You,  // 辰 -> 酉（申子辰三合水局）
    DiZhi::Wu,   // 巳 -> 午（巳酉丑三合金局）
    DiZhi::Mao,  // 午 -> 卯（寅午戌三合火局）
    DiZhi::Zi,   // 未 -> 子（亥卯未三合木局）
    DiZhi::You,  // 申 -> 酉（申子辰三合水局）
    DiZhi::Wu,   // 酉 -> 午（巳酉丑三合金局）
    DiZhi::Mao,  // 戌 -> 卯（寅午戌三合火局）
    DiZhi::Zi,   // 亥 -> 子（亥卯未三合木局）
];

/// 计算桃花
///
/// 桃花主感情、人缘、桃色事件。
///
/// # 参数
/// - `day_zhi`: 日支（或年支）
///
/// # 返回
/// 桃花地支
pub fn calculate_tao_hua(day_zhi: DiZhi) -> DiZhi {
    TAO_HUA[day_zhi.index() as usize]
}

/// 判断地支是否为桃花
pub fn is_tao_hua(day_zhi: DiZhi, zhi: DiZhi) -> bool {
    calculate_tao_hua(day_zhi) == zhi
}

// ============================================================================
// 禄神
// ============================================================================

/// 禄神查询表（按日干）
///
/// 口诀：
/// 甲禄在寅，乙禄在卯，丙戊禄在巳，
/// 丁己禄在午，庚禄在申，辛禄在酉，
/// 壬禄在亥，癸禄在子。
///
/// 禄神即日干的临官位
const LU_SHEN: [DiZhi; 10] = [
    DiZhi::Yin,  // 甲 -> 寅
    DiZhi::Mao,  // 乙 -> 卯
    DiZhi::Si,   // 丙 -> 巳
    DiZhi::Wu,   // 丁 -> 午
    DiZhi::Si,   // 戊 -> 巳
    DiZhi::Wu,   // 己 -> 午
    DiZhi::Shen, // 庚 -> 申
    DiZhi::You,  // 辛 -> 酉
    DiZhi::Hai,  // 壬 -> 亥
    DiZhi::Zi,   // 癸 -> 子
];

/// 计算禄神
///
/// 禄神主财禄、俸禄、官禄。
///
/// # 参数
/// - `day_gan`: 日干
///
/// # 返回
/// 禄神地支
pub fn calculate_lu_shen(day_gan: TianGan) -> DiZhi {
    LU_SHEN[day_gan.index() as usize]
}

/// 判断地支是否为禄神
pub fn is_lu_shen(day_gan: TianGan, zhi: DiZhi) -> bool {
    calculate_lu_shen(day_gan) == zhi
}

// ============================================================================
// 文昌
// ============================================================================

/// 文昌查询表（按日干）
///
/// 口诀：
/// 甲乙巳午报君知，丙戊申宫丁己鸡，
/// 庚猪辛鼠壬逢虎，癸人见卯入云梯。
const WEN_CHANG: [DiZhi; 10] = [
    DiZhi::Si,   // 甲 -> 巳
    DiZhi::Wu,   // 乙 -> 午
    DiZhi::Shen, // 丙 -> 申
    DiZhi::You,  // 丁 -> 酉
    DiZhi::Shen, // 戊 -> 申
    DiZhi::You,  // 己 -> 酉
    DiZhi::Hai,  // 庚 -> 亥
    DiZhi::Zi,   // 辛 -> 子
    DiZhi::Yin,  // 壬 -> 寅
    DiZhi::Mao,  // 癸 -> 卯
];

/// 计算文昌
///
/// 文昌主文才、考试、学业、文书。
///
/// # 参数
/// - `day_gan`: 日干
///
/// # 返回
/// 文昌地支
pub fn calculate_wen_chang(day_gan: TianGan) -> DiZhi {
    WEN_CHANG[day_gan.index() as usize]
}

/// 判断地支是否为文昌
pub fn is_wen_chang(day_gan: TianGan, zhi: DiZhi) -> bool {
    calculate_wen_chang(day_gan) == zhi
}

// ============================================================================
// 劫煞
// ============================================================================

/// 劫煞查询表（按日支/年支三合局）
///
/// 口诀：
/// 申子辰见巳为劫，亥卯未见申为劫，
/// 寅午戌见亥为劫，巳酉丑见寅为劫。
///
/// 劫煞取三合局的绝位
const JIE_SHA: [DiZhi; 12] = [
    DiZhi::Si,   // 子 -> 巳（申子辰）
    DiZhi::Yin,  // 丑 -> 寅（巳酉丑）
    DiZhi::Hai,  // 寅 -> 亥（寅午戌）
    DiZhi::Shen, // 卯 -> 申（亥卯未）
    DiZhi::Si,   // 辰 -> 巳（申子辰）
    DiZhi::Yin,  // 巳 -> 寅（巳酉丑）
    DiZhi::Hai,  // 午 -> 亥（寅午戌）
    DiZhi::Shen, // 未 -> 申（亥卯未）
    DiZhi::Si,   // 申 -> 巳（申子辰）
    DiZhi::Yin,  // 酉 -> 寅（巳酉丑）
    DiZhi::Hai,  // 戌 -> 亥（寅午戌）
    DiZhi::Shen, // 亥 -> 申（亥卯未）
];

/// 计算劫煞
///
/// 劫煞主灾祸、劫难、意外。
///
/// # 参数
/// - `day_zhi`: 日支（或年支）
///
/// # 返回
/// 劫煞地支
pub fn calculate_jie_sha(day_zhi: DiZhi) -> DiZhi {
    JIE_SHA[day_zhi.index() as usize]
}

/// 判断地支是否为劫煞
pub fn is_jie_sha(day_zhi: DiZhi, zhi: DiZhi) -> bool {
    calculate_jie_sha(day_zhi) == zhi
}

// ============================================================================
// 华盖
// ============================================================================

/// 华盖查询表（按日支/年支三合局）
///
/// 口诀：
/// 申子辰见辰，寅午戌见戌，
/// 巳酉丑见丑，亥卯未见未。
///
/// 华盖取三合局的墓库
const HUA_GAI: [DiZhi; 12] = [
    DiZhi::Chen, // 子 -> 辰（申子辰）
    DiZhi::Chou, // 丑 -> 丑（巳酉丑）
    DiZhi::Xu,   // 寅 -> 戌（寅午戌）
    DiZhi::Wei,  // 卯 -> 未（亥卯未）
    DiZhi::Chen, // 辰 -> 辰（申子辰）
    DiZhi::Chou, // 巳 -> 丑（巳酉丑）
    DiZhi::Xu,   // 午 -> 戌（寅午戌）
    DiZhi::Wei,  // 未 -> 未（亥卯未）
    DiZhi::Chen, // 申 -> 辰（申子辰）
    DiZhi::Chou, // 酉 -> 丑（巳酉丑）
    DiZhi::Xu,   // 戌 -> 戌（寅午戌）
    DiZhi::Wei,  // 亥 -> 未（亥卯未）
];

/// 计算华盖
///
/// 华盖主孤独、艺术、宗教、玄学。
///
/// # 参数
/// - `day_zhi`: 日支（或年支）
///
/// # 返回
/// 华盖地支
pub fn calculate_hua_gai(day_zhi: DiZhi) -> DiZhi {
    HUA_GAI[day_zhi.index() as usize]
}

/// 判断地支是否为华盖
pub fn is_hua_gai(day_zhi: DiZhi, zhi: DiZhi) -> bool {
    calculate_hua_gai(day_zhi) == zhi
}

// ============================================================================
// 将星
// ============================================================================

/// 将星查询表（按日支/年支三合局）
///
/// 口诀：
/// 申子辰见子，寅午戌见午，
/// 巳酉丑见酉，亥卯未见卯。
///
/// 将星取三合局的帝旺位
const JIANG_XING: [DiZhi; 12] = [
    DiZhi::Zi,  // 子 -> 子（申子辰）
    DiZhi::You, // 丑 -> 酉（巳酉丑）
    DiZhi::Wu,  // 寅 -> 午（寅午戌）
    DiZhi::Mao, // 卯 -> 卯（亥卯未）
    DiZhi::Zi,  // 辰 -> 子（申子辰）
    DiZhi::You, // 巳 -> 酉（巳酉丑）
    DiZhi::Wu,  // 午 -> 午（寅午戌）
    DiZhi::Mao, // 未 -> 卯（亥卯未）
    DiZhi::Zi,  // 申 -> 子（申子辰）
    DiZhi::You, // 酉 -> 酉（巳酉丑）
    DiZhi::Wu,  // 戌 -> 午（寅午戌）
    DiZhi::Mao, // 亥 -> 卯（亥卯未）
];

/// 计算将星
///
/// 将星主权威、领导、威望。
///
/// # 参数
/// - `day_zhi`: 日支（或年支）
///
/// # 返回
/// 将星地支
pub fn calculate_jiang_xing(day_zhi: DiZhi) -> DiZhi {
    JIANG_XING[day_zhi.index() as usize]
}

/// 判断地支是否为将星
pub fn is_jiang_xing(day_zhi: DiZhi, zhi: DiZhi) -> bool {
    calculate_jiang_xing(day_zhi) == zhi
}

// ============================================================================
// 亡神
// ============================================================================

/// 亡神查询表（按日支/年支三合局）
///
/// 口诀：
/// 申子辰见亥，寅午戌见巳，
/// 巳酉丑见申，亥卯未见寅。
///
/// 亡神取三合局前一位的长生位
const WANG_SHEN: [DiZhi; 12] = [
    DiZhi::Hai,  // 子 -> 亥（申子辰）
    DiZhi::Shen, // 丑 -> 申（巳酉丑）
    DiZhi::Si,   // 寅 -> 巳（寅午戌）
    DiZhi::Yin,  // 卯 -> 寅（亥卯未）
    DiZhi::Hai,  // 辰 -> 亥（申子辰）
    DiZhi::Shen, // 巳 -> 申（巳酉丑）
    DiZhi::Si,   // 午 -> 巳（寅午戌）
    DiZhi::Yin,  // 未 -> 寅（亥卯未）
    DiZhi::Hai,  // 申 -> 亥（申子辰）
    DiZhi::Shen, // 酉 -> 申（巳酉丑）
    DiZhi::Si,   // 戌 -> 巳（寅午戌）
    DiZhi::Yin,  // 亥 -> 寅（亥卯未）
];

/// 计算亡神
///
/// 亡神主破败、损失、虚耗。
///
/// # 参数
/// - `day_zhi`: 日支（或年支）
///
/// # 返回
/// 亡神地支
pub fn calculate_wang_shen(day_zhi: DiZhi) -> DiZhi {
    WANG_SHEN[day_zhi.index() as usize]
}

/// 判断地支是否为亡神
pub fn is_wang_shen(day_zhi: DiZhi, zhi: DiZhi) -> bool {
    calculate_wang_shen(day_zhi) == zhi
}

// ============================================================================
// 天喜
// ============================================================================

/// 天喜查询表（按月支）
///
/// 口诀：
/// 春天占卜天喜在戌，夏天占卜天喜在丑，
/// 秋天占卜天喜在辰，冬天占卜天喜在未。
///
/// 更详细的按月支对应：
/// 寅卯辰月天喜在戌，巳午未月天喜在丑，
/// 申酉戌月天喜在辰，亥子丑月天喜在未。
const TIAN_XI: [DiZhi; 12] = [
    DiZhi::Wei,  // 子月（冬）-> 未
    DiZhi::Wei,  // 丑月（冬）-> 未
    DiZhi::Xu,   // 寅月（春）-> 戌
    DiZhi::Xu,   // 卯月（春）-> 戌
    DiZhi::Xu,   // 辰月（春）-> 戌
    DiZhi::Chou, // 巳月（夏）-> 丑
    DiZhi::Chou, // 午月（夏）-> 丑
    DiZhi::Chou, // 未月（夏）-> 丑
    DiZhi::Chen, // 申月（秋）-> 辰
    DiZhi::Chen, // 酉月（秋）-> 辰
    DiZhi::Chen, // 戌月（秋）-> 辰
    DiZhi::Wei,  // 亥月（冬）-> 未
];

/// 计算天喜
///
/// 天喜主喜庆、婚姻、添丁。
///
/// # 参数
/// - `month_zhi`: 月支
///
/// # 返回
/// 天喜地支
pub fn calculate_tian_xi(month_zhi: DiZhi) -> DiZhi {
    TIAN_XI[month_zhi.index() as usize]
}

/// 判断地支是否为天喜
pub fn is_tian_xi(month_zhi: DiZhi, zhi: DiZhi) -> bool {
    calculate_tian_xi(month_zhi) == zhi
}

// ============================================================================
// 天医
// ============================================================================

/// 计算天医
///
/// 天医取月支前一位。
/// 主医药、治疗、健康。
///
/// # 参数
/// - `month_zhi`: 月支
///
/// # 返回
/// 天医地支
pub fn calculate_tian_yi(month_zhi: DiZhi) -> DiZhi {
    // 月支前一位（逆行一位）
    let idx = (month_zhi.index() + 11) % 12;
    DiZhi::from_index(idx)
}

/// 判断地支是否为天医
pub fn is_tian_yi(month_zhi: DiZhi, zhi: DiZhi) -> bool {
    calculate_tian_yi(month_zhi) == zhi
}

// ============================================================================
// 阳刃
// ============================================================================

/// 阳刃查询表（按日干）
///
/// 口诀：
/// 甲刃在卯，乙刃在寅，丙戊刃在午，丁己刃在巳，
/// 庚刃在酉，辛刃在申，壬刃在子，癸刃在亥。
///
/// 阳刃即日干临官的下一位（帝旺位）
const YANG_REN: [DiZhi; 10] = [
    DiZhi::Mao,  // 甲 -> 卯
    DiZhi::Yin,  // 乙 -> 寅（乙为阴干，取辰位）实际应为寅
    DiZhi::Wu,   // 丙 -> 午
    DiZhi::Si,   // 丁 -> 巳
    DiZhi::Wu,   // 戊 -> 午
    DiZhi::Si,   // 己 -> 巳
    DiZhi::You,  // 庚 -> 酉
    DiZhi::Shen, // 辛 -> 申
    DiZhi::Zi,   // 壬 -> 子
    DiZhi::Hai,  // 癸 -> 亥
];

/// 计算阳刃
///
/// 阳刃主刚烈、凶险、血光。
/// 为日干帝旺之位。
///
/// # 参数
/// - `day_gan`: 日干
///
/// # 返回
/// 阳刃地支
pub fn calculate_yang_ren(day_gan: TianGan) -> DiZhi {
    YANG_REN[day_gan.index() as usize]
}

/// 判断地支是否为阳刃
pub fn is_yang_ren(day_gan: TianGan, zhi: DiZhi) -> bool {
    calculate_yang_ren(day_gan) == zhi
}

// ============================================================================
// 灾煞
// ============================================================================

/// 灾煞查询表（按日支三合局）
///
/// 口诀：
/// 申子辰日灾煞在午，巳酉丑日灾煞在卯，
/// 寅午戌日灾煞在子，亥卯未日灾煞在酉。
///
/// 灾煞取三合局的对冲位
const ZAI_SHA: [DiZhi; 12] = [
    DiZhi::Wu,  // 子 -> 午（申子辰）
    DiZhi::Mao, // 丑 -> 卯（巳酉丑）
    DiZhi::Zi,  // 寅 -> 子（寅午戌）
    DiZhi::You, // 卯 -> 酉（亥卯未）
    DiZhi::Wu,  // 辰 -> 午（申子辰）
    DiZhi::Mao, // 巳 -> 卯（巳酉丑）
    DiZhi::Zi,  // 午 -> 子（寅午戌）
    DiZhi::You, // 未 -> 酉（亥卯未）
    DiZhi::Wu,  // 申 -> 午（申子辰）
    DiZhi::Mao, // 酉 -> 卯（巳酉丑）
    DiZhi::Zi,  // 戌 -> 子（寅午戌）
    DiZhi::You, // 亥 -> 酉（亥卯未）
];

/// 计算灾煞
///
/// 灾煞主灾难、血光、意外。
///
/// # 参数
/// - `day_zhi`: 日支（或年支）
///
/// # 返回
/// 灾煞地支
pub fn calculate_zai_sha(day_zhi: DiZhi) -> DiZhi {
    ZAI_SHA[day_zhi.index() as usize]
}

/// 判断地支是否为灾煞
pub fn is_zai_sha(day_zhi: DiZhi, zhi: DiZhi) -> bool {
    calculate_zai_sha(day_zhi) == zhi
}

// ============================================================================
// 谋星
// ============================================================================

/// 谋星查询表（按日支三合局）
///
/// 口诀：
/// 逢申子辰日谋星在戌，逢巳酉丑日谋星在未，
/// 逢寅午戌日谋星在辰，逢亥卯未日谋星在丑。
///
/// 谋星取三合局的墓库
const MOU_XING: [DiZhi; 12] = [
    DiZhi::Xu,   // 子 -> 戌（申子辰）
    DiZhi::Wei,  // 丑 -> 未（巳酉丑）
    DiZhi::Chen, // 寅 -> 辰（寅午戌）
    DiZhi::Chou, // 卯 -> 丑（亥卯未）
    DiZhi::Xu,   // 辰 -> 戌（申子辰）
    DiZhi::Wei,  // 巳 -> 未（巳酉丑）
    DiZhi::Chen, // 午 -> 辰（寅午戌）
    DiZhi::Chou, // 未 -> 丑（亥卯未）
    DiZhi::Xu,   // 申 -> 戌（申子辰）
    DiZhi::Wei,  // 酉 -> 未（巳酉丑）
    DiZhi::Chen, // 戌 -> 辰（寅午戌）
    DiZhi::Chou, // 亥 -> 丑（亥卯未）
];

/// 计算谋星
///
/// 谋星主谋划、策略、机智。
///
/// # 参数
/// - `day_zhi`: 日支（或年支）
///
/// # 返回
/// 谋星地支
pub fn calculate_mou_xing(day_zhi: DiZhi) -> DiZhi {
    MOU_XING[day_zhi.index() as usize]
}

/// 判断地支是否为谋星
pub fn is_mou_xing(day_zhi: DiZhi, zhi: DiZhi) -> bool {
    calculate_mou_xing(day_zhi) == zhi
}

// ============================================================================
// 综合查询
// ============================================================================

/// 神煞信息结构
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct ShenShaInfo {
    /// 天乙贵人（两个）
    pub tian_yi_gui_ren: [DiZhi; 2],
    /// 驿马
    pub yi_ma: DiZhi,
    /// 桃花
    pub tao_hua: DiZhi,
    /// 禄神
    pub lu_shen: DiZhi,
    /// 文昌
    pub wen_chang: DiZhi,
    /// 劫煞
    pub jie_sha: DiZhi,
    /// 华盖
    pub hua_gai: DiZhi,
    /// 将星
    pub jiang_xing: DiZhi,
    /// 亡神
    pub wang_shen: DiZhi,
    /// 天喜
    pub tian_xi: DiZhi,
    /// 天医
    pub tian_yi: DiZhi,
    /// 阳刃
    pub yang_ren: DiZhi,
    /// 灾煞
    pub zai_sha: DiZhi,
    /// 谋星
    pub mou_xing: DiZhi,
}

/// 计算所有神煞
///
/// # 参数
/// - `day_gan`: 日干
/// - `day_zhi`: 日支
/// - `month_zhi`: 月支（用于天喜、天医）
///
/// # 返回
/// 所有神煞信息
pub fn calculate_all_shen_sha(day_gan: TianGan, day_zhi: DiZhi, month_zhi: DiZhi) -> ShenShaInfo {
    ShenShaInfo {
        tian_yi_gui_ren: calculate_tian_yi_gui_ren(day_gan),
        yi_ma: calculate_yi_ma(day_zhi),
        tao_hua: calculate_tao_hua(day_zhi),
        lu_shen: calculate_lu_shen(day_gan),
        wen_chang: calculate_wen_chang(day_gan),
        jie_sha: calculate_jie_sha(day_zhi),
        hua_gai: calculate_hua_gai(day_zhi),
        jiang_xing: calculate_jiang_xing(day_zhi),
        wang_shen: calculate_wang_shen(day_zhi),
        tian_xi: calculate_tian_xi(month_zhi),
        tian_yi: calculate_tian_yi(month_zhi),
        yang_ren: calculate_yang_ren(day_gan),
        zai_sha: calculate_zai_sha(day_zhi),
        mou_xing: calculate_mou_xing(day_zhi),
    }
}

/// 查询地支携带的所有神煞
///
/// # 参数
/// - `day_gan`: 日干
/// - `day_zhi`: 日支
/// - `month_zhi`: 月支
/// - `target_zhi`: 要查询的地支
///
/// # 返回
/// 该地支携带的所有神煞列表（最多14个）
pub fn get_shen_sha_for_zhi(
    day_gan: TianGan,
    day_zhi: DiZhi,
    month_zhi: DiZhi,
    target_zhi: DiZhi,
) -> [Option<ShenSha>; 14] {
    let mut result: [Option<ShenSha>; 14] = [None; 14];
    let mut idx = 0;

    if is_tian_yi_gui_ren(day_gan, target_zhi) {
        result[idx] = Some(ShenSha::TianYiGuiRen);
        idx += 1;
    }
    if is_yi_ma(day_zhi, target_zhi) {
        result[idx] = Some(ShenSha::YiMa);
        idx += 1;
    }
    if is_tao_hua(day_zhi, target_zhi) {
        result[idx] = Some(ShenSha::TaoHua);
        idx += 1;
    }
    if is_lu_shen(day_gan, target_zhi) {
        result[idx] = Some(ShenSha::LuShen);
        idx += 1;
    }
    if is_wen_chang(day_gan, target_zhi) {
        result[idx] = Some(ShenSha::WenChang);
        idx += 1;
    }
    if is_jie_sha(day_zhi, target_zhi) {
        result[idx] = Some(ShenSha::JieSha);
        idx += 1;
    }
    if is_hua_gai(day_zhi, target_zhi) {
        result[idx] = Some(ShenSha::HuaGai);
        idx += 1;
    }
    if is_jiang_xing(day_zhi, target_zhi) {
        result[idx] = Some(ShenSha::JiangXing);
        idx += 1;
    }
    if is_wang_shen(day_zhi, target_zhi) {
        result[idx] = Some(ShenSha::WangShen);
        idx += 1;
    }
    if is_tian_xi(month_zhi, target_zhi) {
        result[idx] = Some(ShenSha::TianXi);
        idx += 1;
    }
    if is_tian_yi(month_zhi, target_zhi) {
        result[idx] = Some(ShenSha::TianYi);
        idx += 1;
    }
    if is_yang_ren(day_gan, target_zhi) {
        result[idx] = Some(ShenSha::YangRen);
        idx += 1;
    }
    if is_zai_sha(day_zhi, target_zhi) {
        result[idx] = Some(ShenSha::ZaiSha);
        idx += 1;
    }
    if is_mou_xing(day_zhi, target_zhi) {
        result[idx] = Some(ShenSha::MouXing);
    }

    result
}
