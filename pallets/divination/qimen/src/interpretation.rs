//! # 奇门遁甲解卦模块
//!
//! 本模块实现奇门遁甲的解卦功能，包括：
//! - 核心解卦数据结构（Layer 1）
//! - 扩展解卦数据结构（Layer 2）
//! - 应期推算数据结构（Layer 3）
//! - 解卦算法实现
//!
//! ## 设计原则
//!
//! 1. **轻量化存储**：核心指标仅 16 bytes，链上存储
//! 2. **实时计算**：扩展数据通过 Runtime API 实时计算
//! 3. **算法升级**：无需数据迁移，立即生效
//! 4. **隐私保护**：敏感信息仅存储哈希值
//!
//! ## 存储层次
//!
//! - Layer 1: 核心指标（链上存储，~16 bytes）
//! - Layer 2: 宫位详解、用神分析（Runtime API 计算）
//! - Layer 3: 应期推算（Runtime API 计算）
//! - Layer 4: AI 解读（IPFS 存储）

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::*, BoundedVec};
use scale_info::TypeInfo;
use sp_std::prelude::*;

extern crate alloc;
use alloc::format;

use crate::types::*;

// ==================== Layer 1: 核心解卦指标 ====================

/// 奇门遁甲核心解卦结果
///
/// 存储空间优化，总大小约 16 bytes
///
/// # 字段说明
///
/// - `ge_ju`: 格局类型（1 byte）
/// - `yong_shen_gong`: 用神宫位 1-9（1 byte）
/// - `zhi_fu_xing`: 值符星（1 byte）
/// - `zhi_shi_men`: 值使门（1 byte）
/// - `ri_gan_gong`: 日干落宫 1-9（1 byte）
/// - `shi_gan_gong`: 时干落宫 1-9（1 byte）
/// - `fortune`: 综合吉凶（1 byte）
/// - `fortune_score`: 吉凶评分 0-100（1 byte）
/// - `wang_shuai`: 旺衰状态（1 byte）
/// - `special_patterns`: 特殊格局标记（1 byte，位标志）
/// - `confidence`: 可信度 0-100（1 byte）
/// - `timestamp`: 解盘时间戳 - 区块号（4 bytes）
/// - `algorithm_version`: 算法版本（1 byte）
///
/// **总大小**: 16 bytes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct QimenCoreInterpretation {
    /// 格局类型 (1 byte)
    pub ge_ju: GeJuType,

    /// 用神宫位 (1 byte, 1-9)
    pub yong_shen_gong: u8,

    /// 值符星 (1 byte)
    pub zhi_fu_xing: JiuXing,

    /// 值使门 (1 byte)
    pub zhi_shi_men: BaMen,

    /// 日干落宫 (1 byte, 1-9)
    pub ri_gan_gong: u8,

    /// 时干落宫 (1 byte, 1-9)
    pub shi_gan_gong: u8,

    /// 综合吉凶 (1 byte)
    pub fortune: Fortune,

    /// 吉凶等级 0-100 (1 byte)
    pub fortune_score: u8,

    /// 旺衰状态 (1 byte)
    pub wang_shuai: WangShuai,

    /// 特殊格局标记 (1 byte, 位标志)
    /// bit 0: 伏吟
    /// bit 1: 反吟
    /// bit 2: 天遁
    /// bit 3: 地遁
    /// bit 4: 人遁
    /// bit 5: 鬼遁
    /// bit 6: 神遁
    /// bit 7: 龙遁
    pub special_patterns: u8,

    /// 可信度 0-100 (1 byte)
    pub confidence: u8,

    /// 解盘时间戳 - 区块号 (4 bytes)
    pub timestamp: u32,

    /// 算法版本 (1 byte)
    pub algorithm_version: u8,
}

impl QimenCoreInterpretation {
    /// 检查是否有特定特殊格局
    pub fn has_special_pattern(&self, bit: u8) -> bool {
        (self.special_patterns & (1 << bit)) != 0
    }

    /// 设置特殊格局标记
    pub fn set_special_pattern(&mut self, bit: u8) {
        self.special_patterns |= 1 << bit;
    }

    /// 检查是否伏吟
    pub fn is_fu_yin(&self) -> bool {
        self.has_special_pattern(0)
    }

    /// 检查是否反吟
    pub fn is_fan_yin(&self) -> bool {
        self.has_special_pattern(1)
    }

    /// 检查是否天遁
    pub fn is_tian_dun(&self) -> bool {
        self.has_special_pattern(2)
    }

    /// 检查是否地遁
    pub fn is_di_dun(&self) -> bool {
        self.has_special_pattern(3)
    }

    /// 检查是否人遁
    pub fn is_ren_dun(&self) -> bool {
        self.has_special_pattern(4)
    }
}

/// 为 QimenCoreInterpretation 实现 Default
///
/// 用于 Private 模式下返回默认的空解卦结果
impl Default for QimenCoreInterpretation {
    fn default() -> Self {
        Self {
            ge_ju: GeJuType::default(),
            yong_shen_gong: 0,
            zhi_fu_xing: JiuXing::default(),
            zhi_shi_men: BaMen::default(),
            ri_gan_gong: 0,
            shi_gan_gong: 0,
            fortune: Fortune::default(),
            fortune_score: 0,
            wang_shuai: WangShuai::default(),
            special_patterns: 0,
            confidence: 0,
            timestamp: 0,
            algorithm_version: 1,
        }
    }
}

// ==================== Layer 2: 扩展解卦数据 ====================

/// 单宫详细解读
///
/// 包含单个宫位的完整解读信息
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct PalaceInterpretation {
    /// 宫位
    pub gong: JiuGong,

    /// 天盘干
    pub tian_pan_gan: TianGan,

    /// 地盘干
    pub di_pan_gan: TianGan,

    /// 九星
    pub xing: JiuXing,

    /// 八门
    pub men: Option<BaMen>,

    /// 八神
    pub shen: Option<BaShen>,

    /// 宫位五行
    pub gong_wuxing: WuXing,

    /// 天盘五行
    pub tian_wuxing: WuXing,

    /// 地盘五行
    pub di_wuxing: WuXing,

    /// 星门关系
    pub xing_men_relation: XingMenRelation,

    /// 宫位旺衰
    pub wang_shuai: WangShuai,

    /// 是否伏吟
    pub is_fu_yin: bool,

    /// 是否反吟
    pub is_fan_yin: bool,

    /// 是否旬空
    pub is_xun_kong: bool,

    /// 是否马星
    pub is_ma_xing: bool,

    /// 宫位吉凶
    pub fortune: Fortune,

    /// 吉凶评分 0-100
    pub fortune_score: u8,
}

/// 用神分析结果
///
/// 根据问事类型分析用神的状态
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct YongShenAnalysis {
    /// 问事类型
    pub question_type: QuestionType,

    /// 主用神宫位
    pub primary_gong: JiuGong,

    /// 主用神类型
    pub primary_type: YongShenType,

    /// 次用神宫位（可选）
    pub secondary_gong: Option<JiuGong>,

    /// 次用神类型（可选）
    pub secondary_type: Option<YongShenType>,

    /// 用神旺衰
    pub wang_shuai: WangShuai,

    /// 用神得力情况
    pub de_li: DeLiStatus,

    /// 用神吉凶
    pub fortune: Fortune,

    /// 用神评分 0-100
    pub score: u8,
}

// ==================== Layer 3: 应期推算 ====================

/// 应期推算结果
///
/// 预测事情应验的时间
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct YingQiAnalysis {
    /// 主应期数（基于用神宫位）
    pub primary_num: u8,

    /// 次应期数（基于值符值使）
    pub secondary_nums: [u8; 2],

    /// 应期单位
    pub unit: YingQiUnit,

    /// 应期范围描述
    pub range_desc: BoundedVec<u8, ConstU32<128>>,

    /// 吉利时间
    pub auspicious_times: BoundedVec<u8, ConstU32<64>>,

    /// 不利时间
    pub inauspicious_times: BoundedVec<u8, ConstU32<64>>,
}

/// 格局详解
///
/// 格局的详细说明和建议
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct GeJuDetail {
    /// 格局类型
    pub ge_ju: GeJuType,

    /// 格局名称
    pub name: BoundedVec<u8, ConstU32<32>>,

    /// 格局描述
    pub description: BoundedVec<u8, ConstU32<256>>,

    /// 格局吉凶
    pub fortune: Fortune,

    /// 适用场景
    pub applicable_scenarios: BoundedVec<QuestionType, ConstU32<8>>,

    /// 注意事项
    pub notes: BoundedVec<u8, ConstU32<256>>,
}

// ==================== 完整解卦数据结构 ====================

/// 奇门遁甲完整解读结果
///
/// 包含所有层次的解读数据
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct QimenFullInterpretation {
    /// 核心指标（必有）
    pub core: QimenCoreInterpretation,

    /// 九宫详细解读（可选）
    pub palaces: Option<Vec<PalaceInterpretation>>,

    /// 用神分析（可选）
    pub yong_shen: Option<YongShenAnalysis>,

    /// 应期推算（可选）
    pub ying_qi: Option<YingQiAnalysis>,

    /// 格局详解（可选）
    pub ge_ju_detail: Option<GeJuDetail>,
}

// ==================== 核心解卦算法实现 ====================

/// 计算核心解卦（Layer 1）
///
/// 免费实时计算，不存储
///
/// # 参数
///
/// - `chart`: 奇门遁甲排盘结果
/// - `current_block`: 当前区块号
///
/// # 返回
///
/// 核心解卦结果
///
/// 注意：此函数仅适用于 Public 和 Partial 模式（计算数据可用）。
/// 对于 Private 模式，需使用 `compute_chart` Runtime API。
pub fn calculate_core_interpretation<AccountId, BlockNumber, MaxCidLen: Get<u32>>(
    chart: &QimenChart<AccountId, BlockNumber, MaxCidLen>,
    current_block: u32,
) -> QimenCoreInterpretation
where
    AccountId: Clone,
    BlockNumber: Clone,
{
    // 检查计算数据是否可用
    let palaces = match chart.get_palaces() {
        Some(p) => p,
        None => {
            // Private 模式或数据不可用，返回默认值
            return QimenCoreInterpretation::default();
        }
    };

    let day_ganzhi = chart.get_day_ganzhi().unwrap_or_default();
    let hour_ganzhi = chart.get_hour_ganzhi().unwrap_or_default();
    let zhi_fu_xing = chart.get_zhi_fu_xing().unwrap_or_default();
    let zhi_shi_men = chart.get_zhi_shi_men().unwrap_or_default();

    // 1. 分析格局
    let ge_ju = analyze_ge_ju(palaces);

    // 2. 确定用神宫位（默认使用日干）
    let yong_shen_gong = determine_yong_shen_gong(chart, QuestionType::General);

    // 3. 计算日干时干落宫
    let ri_gan_gong = find_gan_palace(palaces, day_ganzhi.gan);
    let shi_gan_gong = find_gan_palace(palaces, hour_ganzhi.gan);

    // 4. 分析旺衰
    let wang_shuai = analyze_wang_shuai(chart, yong_shen_gong);

    // 5. 检测特殊格局
    let special_patterns = detect_special_patterns(palaces);

    // 6. 计算综合吉凶
    let (fortune, fortune_score) = calculate_fortune(
        ge_ju,
        wang_shuai,
        zhi_fu_xing,
        zhi_shi_men,
        special_patterns,
    );

    // 7. 计算可信度
    let confidence = calculate_confidence(chart, ge_ju);

    QimenCoreInterpretation {
        ge_ju,
        yong_shen_gong,
        zhi_fu_xing,
        zhi_shi_men,
        ri_gan_gong,
        shi_gan_gong,
        fortune,
        fortune_score,
        wang_shuai,
        special_patterns,
        confidence,
        timestamp: current_block,
        algorithm_version: 1,
    }
}

/// 分析格局
///
/// 检测各种奇门遁甲格局
///
/// # 参数
///
/// - `palaces`: 九宫排盘结果
///
/// # 返回
///
/// 格局类型
fn analyze_ge_ju(palaces: &[Palace; 9]) -> GeJuType {
    // 1. 检查伏吟（天盘地盘相同）
    if is_fu_yin(palaces) {
        return GeJuType::FuYinGe;
    }

    // 2. 检查反吟（天盘地盘对冲）
    if is_fan_yin(palaces) {
        return GeJuType::FanYinGe;
    }

    // 3. 检查三遁（天遁、地遁、人遁）
    if let Some(dun_ge) = check_san_dun(palaces) {
        return dun_ge;
    }

    // 4. 检查鬼遁、神遁、龙遁
    if let Some(special_ge) = check_special_dun(palaces) {
        return special_ge;
    }

    // 5. 检查特殊吉凶格局
    if let Some(special_ge) = check_special_patterns_ge(palaces) {
        return special_ge;
    }

    // 6. 默认为正格
    GeJuType::ZhengGe
}

/// 分析旺衰
///
/// 根据节气和五行关系判断用神的旺衰程度
///
/// # 参数
///
/// - `chart`: 奇门遁甲排盘结果
/// - `yong_shen_gong`: 用神宫位（1-9）
///
/// # 返回
///
/// 旺衰状态（如果数据不可用返回默认值）
fn analyze_wang_shuai<AccountId, BlockNumber, MaxCidLen: Get<u32>>(
    chart: &QimenChart<AccountId, BlockNumber, MaxCidLen>,
    yong_shen_gong: u8,
) -> WangShuai
where
    AccountId: Clone,
    BlockNumber: Clone,
{
    if yong_shen_gong == 0 || yong_shen_gong > 9 {
        return WangShuai::Xiu;
    }

    // 获取宫位数据
    let palaces = match chart.get_palaces() {
        Some(p) => p,
        None => return WangShuai::default(),
    };
    let jie_qi = match chart.get_jie_qi() {
        Some(j) => j,
        None => return WangShuai::default(),
    };

    let palace = &palaces[(yong_shen_gong - 1) as usize];
    let yong_shen_wuxing = palace.tian_pan_gan.wu_xing();

    // 根据节气判断旺衰
    let jie_qi_wuxing = get_jie_qi_wuxing(jie_qi);

    if yong_shen_wuxing == jie_qi_wuxing {
        WangShuai::WangXiang // 当令为旺
    } else if jie_qi_wuxing.generates(&yong_shen_wuxing) {
        WangShuai::Xiang // 生我为相
    } else if yong_shen_wuxing.generates(&jie_qi_wuxing) {
        WangShuai::Xiu // 我生为休
    } else if jie_qi_wuxing.conquers(&yong_shen_wuxing) {
        WangShuai::Qiu // 克我为囚
    } else {
        WangShuai::Si // 我克为死
    }
}

/// 确定用神宫位
///
/// 根据问事类型确定用神，并查找其落宫
///
/// # 参数
///
/// - `chart`: 奇门遁甲排盘结果
/// - `question_type`: 问事类型
///
/// # 返回
///
/// 用神宫位（1-9）（如果数据不可用返回 0）
fn determine_yong_shen_gong<AccountId, BlockNumber, MaxCidLen: Get<u32>>(
    chart: &QimenChart<AccountId, BlockNumber, MaxCidLen>,
    question_type: QuestionType,
) -> u8
where
    AccountId: Clone,
    BlockNumber: Clone,
{
    // 获取必要数据
    let palaces = match chart.get_palaces() {
        Some(p) => p,
        None => return 0,
    };
    let day_ganzhi = match chart.get_day_ganzhi() {
        Some(d) => d,
        None => return 0,
    };
    let hour_ganzhi = match chart.get_hour_ganzhi() {
        Some(h) => h,
        None => return 0,
    };

    // 根据问事类型确定用神
    // 默认使用日干作为用神（代表自己）
    let yong_shen_gan = match question_type {
        QuestionType::General => day_ganzhi.gan,      // 综合运势 - 日干
        QuestionType::Career => day_ganzhi.gan,       // 事业 - 日干
        QuestionType::Wealth => day_ganzhi.gan,       // 财运 - 日干
        QuestionType::Marriage => day_ganzhi.gan,     // 婚姻 - 日干
        QuestionType::Health => day_ganzhi.gan,       // 健康 - 日干
        QuestionType::Study => day_ganzhi.gan,        // 学业 - 日干
        QuestionType::Travel => hour_ganzhi.gan,      // 出行 - 时干
        QuestionType::Lawsuit => day_ganzhi.gan,      // 官司 - 日干
        QuestionType::Finding => hour_ganzhi.gan,     // 寻人寻物 - 时干
        QuestionType::Investment => day_ganzhi.gan,   // 投资 - 日干
        QuestionType::Business => day_ganzhi.gan,     // 合作 - 日干
        QuestionType::Prayer => day_ganzhi.gan,       // 祈福 - 日干
    };

    find_gan_palace(palaces, yong_shen_gan)
}

/// 检测特殊格局
///
/// 使用位标志存储多个特殊格局
///
/// # 参数
///
/// - `palaces`: 九宫排盘结果
///
/// # 返回
///
/// 特殊格局标记（位标志）
fn detect_special_patterns(palaces: &[Palace; 9]) -> u8 {
    let mut patterns = 0u8;

    // bit 0: 伏吟
    if is_fu_yin(palaces) {
        patterns |= 1 << 0;
    }

    // bit 1: 反吟
    if is_fan_yin(palaces) {
        patterns |= 1 << 1;
    }

    // bit 2: 天遁
    if check_tian_dun(palaces) {
        patterns |= 1 << 2;
    }

    // bit 3: 地遁
    if check_di_dun(palaces) {
        patterns |= 1 << 3;
    }

    // bit 4: 人遁
    if check_ren_dun(palaces) {
        patterns |= 1 << 4;
    }

    // bit 5: 鬼遁
    if check_gui_dun(palaces) {
        patterns |= 1 << 5;
    }

    // bit 6: 神遁
    if check_shen_dun(palaces) {
        patterns |= 1 << 6;
    }

    // bit 7: 龙遁
    if check_long_dun(palaces) {
        patterns |= 1 << 7;
    }

    patterns
}

/// 计算综合吉凶
///
/// 根据格局、旺衰、值符值使等因素综合评分
///
/// # 参数
///
/// - `ge_ju`: 格局类型
/// - `wang_shuai`: 旺衰状态
/// - `zhi_fu_xing`: 值符星
/// - `zhi_shi_men`: 值使门
/// - `special_patterns`: 特殊格局标记
///
/// # 返回
///
/// (吉凶等级, 吉凶评分)
fn calculate_fortune(
    ge_ju: GeJuType,
    wang_shuai: WangShuai,
    zhi_fu_xing: JiuXing,
    zhi_shi_men: BaMen,
    special_patterns: u8,
) -> (Fortune, u8) {
    let mut score = 50u8;

    // 1. 格局分 (0-20)
    score = score.saturating_add(match ge_ju {
        GeJuType::TianDunGe | GeJuType::DiDunGe | GeJuType::RenDunGe => 20,
        GeJuType::ShenDunGe | GeJuType::LongDunGe => 15,
        GeJuType::GuiDunGe => 12,
        GeJuType::ZhengGe => 10,
        GeJuType::QingLongFanShou => 18,
        GeJuType::FuYinGe => 0,
        GeJuType::FanYinGe => 0,
        GeJuType::FeiNiaoDieXue => 0,
    });

    // 2. 旺衰分 (0-15)
    score = score.saturating_add(match wang_shuai {
        WangShuai::WangXiang => 15,
        WangShuai::Xiang => 12,
        WangShuai::Xiu => 8,
        WangShuai::Qiu => 4,
        WangShuai::Si => 0,
    });

    // 3. 值符分 (0-10)
    if zhi_fu_xing.is_auspicious() {
        score = score.saturating_add(10);
    }

    // 4. 值使分 (0-10)
    if zhi_shi_men.is_auspicious() {
        score = score.saturating_add(10);
    }

    // 5. 特殊格局加分 (0-15)
    let special_count = special_patterns.count_ones();
    score = score.saturating_add((special_count as u8 * 3).min(15));

    score = score.min(100);

    // 根据分数确定吉凶等级
    let fortune = match score {
        90..=100 => Fortune::DaJi,
        75..=89 => Fortune::ZhongJi,
        60..=74 => Fortune::XiaoJi,
        45..=59 => Fortune::Ping,
        30..=44 => Fortune::XiaoXiong,
        15..=29 => Fortune::ZhongXiong,
        _ => Fortune::DaXiong,
    };

    (fortune, score)
}

/// 计算可信度
///
/// 根据排盘数据的完整性和格局的稀有度计算可信度
///
/// # 参数
///
/// - `chart`: 奇门遁甲排盘结果
/// - `ge_ju`: 格局类型
///
/// # 返回
///
/// 可信度评分（0-100）
fn calculate_confidence<AccountId, BlockNumber, MaxCidLen: Get<u32>>(
    chart: &QimenChart<AccountId, BlockNumber, MaxCidLen>,
    ge_ju: GeJuType,
) -> u8
where
    AccountId: Clone,
    BlockNumber: Clone,
{
    let mut confidence = 100u8;

    // 1. 起局方式影响可信度
    confidence = confidence.saturating_sub(match chart.method {
        DivinationMethod::ByTime => 0,      // 时间起局最准确
        DivinationMethod::Manual => 5,      // 手动指定略低
        DivinationMethod::ByNumbers => 10,  // 数字起局再低
        DivinationMethod::Random => 15,     // 随机起局最低
    });

    // 2. 格局稀有度影响可信度
    if matches!(
        ge_ju,
        GeJuType::QingLongFanShou | GeJuType::FeiNiaoDieXue
    ) {
        confidence = confidence.saturating_sub(10);
    }

    // 3. 阴阳遁影响（阳遁略准）
    if matches!(chart.get_dun_type(), Some(DunType::Yin)) {
        confidence = confidence.saturating_sub(5);
    }

    confidence
}

// ==================== 辅助函数 ====================

/// 查找天干落宫
///
/// 在九宫中查找指定天干所在的宫位
///
/// # 参数
///
/// - `palaces`: 九宫排盘结果
/// - `gan`: 要查找的天干
///
/// # 返回
///
/// 宫位数字（1-9），未找到返回 1
fn find_gan_palace(palaces: &[Palace; 9], gan: TianGan) -> u8 {
    for (i, palace) in palaces.iter().enumerate() {
        if palace.tian_pan_gan == gan {
            return (i + 1) as u8;
        }
    }
    // 如果未找到，返回 1（坎宫）
    1
}

/// 判断是否伏吟
///
/// 伏吟：天盘地盘相同
///
/// # 参数
///
/// - `palaces`: 九宫排盘结果
///
/// # 返回
///
/// 是否伏吟
fn is_fu_yin(palaces: &[Palace; 9]) -> bool {
    let mut fu_yin_count = 0;
    for palace in palaces.iter() {
        if palace.tian_pan_gan == palace.di_pan_gan {
            fu_yin_count += 1;
        }
    }
    // 如果超过一半的宫位伏吟，则判定为伏吟格
    fu_yin_count >= 5
}

/// 判断是否反吟
///
/// 反吟：天盘地盘对冲（五行相克）
///
/// # 参数
///
/// - `palaces`: 九宫排盘结果
///
/// # 返回
///
/// 是否反吟
fn is_fan_yin(palaces: &[Palace; 9]) -> bool {
    let mut fan_yin_count = 0;
    for palace in palaces.iter() {
        let tian_wuxing = palace.tian_pan_gan.wu_xing();
        let di_wuxing = palace.di_pan_gan.wu_xing();
        if tian_wuxing.conquers(&di_wuxing) || di_wuxing.conquers(&tian_wuxing) {
            fan_yin_count += 1;
        }
    }
    // 如果超过一半的宫位反吟，则判定为反吟格
    fan_yin_count >= 5
}

/// 检查三遁（天遁、地遁、人遁）
///
/// # 参数
///
/// - `palaces`: 九宫排盘结果
///
/// # 返回
///
/// 如果是三遁格局，返回对应的格局类型
fn check_san_dun(palaces: &[Palace; 9]) -> Option<GeJuType> {
    // 天遁：丙奇 + 天心星 + 开门
    if check_tian_dun(palaces) {
        return Some(GeJuType::TianDunGe);
    }

    // 地遁：乙奇 + 六合 + 开门
    if check_di_dun(palaces) {
        return Some(GeJuType::DiDunGe);
    }

    // 人遁：丁奇 + 太阴 + 开门
    if check_ren_dun(palaces) {
        return Some(GeJuType::RenDunGe);
    }

    None
}

/// 检查天遁：丙奇 + 天心星 + 开门
fn check_tian_dun(palaces: &[Palace; 9]) -> bool {
    for palace in palaces.iter() {
        if palace.tian_pan_gan == TianGan::Bing
            && palace.xing == JiuXing::TianXin
            && palace.men == Some(BaMen::Kai)
        {
            return true;
        }
    }
    false
}

/// 检查地遁：乙奇 + 六合 + 开门
fn check_di_dun(palaces: &[Palace; 9]) -> bool {
    for palace in palaces.iter() {
        if palace.tian_pan_gan == TianGan::Yi
            && palace.shen == Some(BaShen::LiuHe)
            && palace.men == Some(BaMen::Kai)
        {
            return true;
        }
    }
    false
}

/// 检查人遁：丁奇 + 太阴 + 开门
fn check_ren_dun(palaces: &[Palace; 9]) -> bool {
    for palace in palaces.iter() {
        if palace.tian_pan_gan == TianGan::Ding
            && palace.shen == Some(BaShen::TaiYin)
            && palace.men == Some(BaMen::Kai)
        {
            return true;
        }
    }
    false
}

/// 检查特殊遁（鬼遁、神遁、龙遁）
///
/// # 参数
///
/// - `palaces`: 九宫排盘结果
///
/// # 返回
///
/// 如果是特殊遁格局，返回对应的格局类型
fn check_special_dun(palaces: &[Palace; 9]) -> Option<GeJuType> {
    // 鬼遁：丁奇 + 天心星 + 开门
    if check_gui_dun(palaces) {
        return Some(GeJuType::GuiDunGe);
    }

    // 神遁：九天 + 值符 + 开门
    if check_shen_dun(palaces) {
        return Some(GeJuType::ShenDunGe);
    }

    // 龙遁：九地 + 值符 + 开门
    if check_long_dun(palaces) {
        return Some(GeJuType::LongDunGe);
    }

    None
}

/// 检查鬼遁：丁奇 + 天心星 + 开门
fn check_gui_dun(palaces: &[Palace; 9]) -> bool {
    for palace in palaces.iter() {
        if palace.tian_pan_gan == TianGan::Ding
            && palace.xing == JiuXing::TianXin
            && palace.men == Some(BaMen::Kai)
        {
            return true;
        }
    }
    false
}

/// 检查神遁：九天 + 值符 + 开门
fn check_shen_dun(palaces: &[Palace; 9]) -> bool {
    for palace in palaces.iter() {
        if palace.shen == Some(BaShen::JiuTian)
            && palace.shen == Some(BaShen::ZhiFu)
            && palace.men == Some(BaMen::Kai)
        {
            return true;
        }
    }
    false
}

/// 检查龙遁：九地 + 值符 + 开门
fn check_long_dun(palaces: &[Palace; 9]) -> bool {
    for palace in palaces.iter() {
        if palace.shen == Some(BaShen::JiuDi)
            && palace.shen == Some(BaShen::ZhiFu)
            && palace.men == Some(BaMen::Kai)
        {
            return true;
        }
    }
    false
}

/// 检查特殊吉凶格局
///
/// # 参数
///
/// - `palaces`: 九宫排盘结果
///
/// # 返回
///
/// 如果是特殊格局，返回对应的格局类型
fn check_special_patterns_ge(palaces: &[Palace; 9]) -> Option<GeJuType> {
    // 青龙返首：特殊吉格（简化判断）
    // 实际规则更复杂，这里仅作示例
    for palace in palaces.iter() {
        if palace.xing == JiuXing::TianFu
            && palace.men == Some(BaMen::Sheng)
            && palace.tian_pan_gan.is_san_qi()
        {
            return Some(GeJuType::QingLongFanShou);
        }
    }

    // 飞鸟跌穴：特殊凶格（简化判断）
    for palace in palaces.iter() {
        if palace.xing == JiuXing::TianYing
            && palace.men == Some(BaMen::Si)
        {
            return Some(GeJuType::FeiNiaoDieXue);
        }
    }

    None
}

/// 获取节气五行
///
/// 根据节气确定当令五行
///
/// # 参数
///
/// - `jie_qi`: 节气
///
/// # 返回
///
/// 节气对应的五行
fn get_jie_qi_wuxing(jie_qi: JieQi) -> WuXing {
    use JieQi::*;
    match jie_qi {
        // 冬季：水
        DongZhi | XiaoHan | DaHan => WuXing::Shui,
        // 春季：木
        LiChun | YuShui | JingZhe | ChunFen | QingMing | GuYu => WuXing::Mu,
        // 夏季：火
        LiXia | XiaoMan | MangZhong | XiaZhi | XiaoShu | DaShu => WuXing::Huo,
        // 秋季：金
        LiQiu | ChuShu | BaiLu | QiuFen | HanLu | ShuangJiang => WuXing::Jin,
        // 冬季：水
        LiDong | XiaoXue | DaXue => WuXing::Shui,
    }
}

// ==================== 扩展解卦算法实现 ====================

/// 计算完整解卦（Layer 1 + Layer 2 + Layer 3）
///
/// 包含核心指标、宫位详解、用神分析、应期推算
///
/// # 参数
///
/// - `chart`: 奇门遁甲排盘结果
/// - `current_block`: 当前区块号
/// - `question_type`: 问事类型
///
/// # 返回
///
/// 完整解卦结果
pub fn calculate_full_interpretation<AccountId, BlockNumber, MaxCidLen: Get<u32>>(
    chart: &QimenChart<AccountId, BlockNumber, MaxCidLen>,
    current_block: u32,
    question_type: QuestionType,
) -> QimenFullInterpretation
where
    AccountId: Clone,
    BlockNumber: Clone,
{
    // 1. 计算核心指标
    let core = calculate_core_interpretation(chart, current_block);

    // 2. 计算九宫详细解读
    let palaces = Some(analyze_all_palaces(chart));

    // 3. 计算用神分析
    let yong_shen = Some(analyze_yong_shen(chart, question_type));

    // 4. 计算应期推算
    let ying_qi = Some(calculate_ying_qi(chart, core.yong_shen_gong));

    // 5. 获取格局详解
    let ge_ju_detail = Some(get_ge_ju_detail(core.ge_ju, question_type));

    QimenFullInterpretation {
        core,
        palaces,
        yong_shen,
        ying_qi,
        ge_ju_detail,
    }
}

/// 分析所有宫位
///
/// 对九宫进行详细分析
///
/// # 参数
///
/// - `chart`: 奇门遁甲排盘结果
///
/// # 返回
///
/// 九宫详细解读列表
fn analyze_all_palaces<AccountId, BlockNumber, MaxCidLen: Get<u32>>(
    chart: &QimenChart<AccountId, BlockNumber, MaxCidLen>,
) -> Vec<PalaceInterpretation>
where
    AccountId: Clone,
    BlockNumber: Clone,
{
    let mut palace_interps = Vec::new();

    // 获取九宫数据和节气，如果不可用则返回空列表
    let palaces = match chart.get_palaces() {
        Some(p) => p,
        None => return palace_interps,
    };
    let jie_qi = match chart.get_jie_qi() {
        Some(j) => j,
        None => return palace_interps,
    };

    for palace in palaces.iter() {
        let interp = analyze_palace_detail(palace, jie_qi);
        palace_interps.push(interp);
    }

    palace_interps
}

/// 分析单宫详细信息
///
/// 包含宫位的完整解读
///
/// # 参数
///
/// - `palace`: 宫位信息
/// - `jie_qi`: 节气
///
/// # 返回
///
/// 单宫详细解读
pub fn analyze_palace_detail(palace: &Palace, jie_qi: JieQi) -> PalaceInterpretation {
    // 1. 获取五行
    let gong_wuxing = palace.gong.wu_xing();
    let tian_wuxing = palace.tian_pan_gan.wu_xing();
    let di_wuxing = palace.di_pan_gan.wu_xing();

    // 2. 分析星门关系
    let xing_men_relation = if let Some(men) = palace.men {
        let xing_wuxing = palace.xing.wu_xing();
        let men_wuxing = men.wu_xing();

        if xing_wuxing == men_wuxing {
            XingMenRelation::BiHe
        } else if xing_wuxing.generates(&men_wuxing) {
            XingMenRelation::XingShengMen
        } else if men_wuxing.generates(&xing_wuxing) {
            XingMenRelation::MenShengXing
        } else if xing_wuxing.conquers(&men_wuxing) {
            XingMenRelation::XingKeMen
        } else {
            XingMenRelation::MenKeXing
        }
    } else {
        XingMenRelation::BiHe
    };

    // 3. 分析宫位旺衰
    let jie_qi_wuxing = get_jie_qi_wuxing(jie_qi);
    let wang_shuai = if tian_wuxing == jie_qi_wuxing {
        WangShuai::WangXiang
    } else if jie_qi_wuxing.generates(&tian_wuxing) {
        WangShuai::Xiang
    } else if tian_wuxing.generates(&jie_qi_wuxing) {
        WangShuai::Xiu
    } else if jie_qi_wuxing.conquers(&tian_wuxing) {
        WangShuai::Qiu
    } else {
        WangShuai::Si
    };

    // 4. 判断特殊状态
    let is_fu_yin = palace.tian_pan_gan == palace.di_pan_gan;
    let is_fan_yin = tian_wuxing.conquers(&di_wuxing) || di_wuxing.conquers(&tian_wuxing);
    let is_xun_kong = palace.is_xun_kong;
    let is_ma_xing = palace.is_ma_xing;

    // 5. 计算宫位吉凶
    let (fortune, fortune_score) = calculate_palace_fortune(
        palace,
        wang_shuai,
        xing_men_relation,
        is_fu_yin,
        is_fan_yin,
    );

    PalaceInterpretation {
        gong: palace.gong,
        tian_pan_gan: palace.tian_pan_gan,
        di_pan_gan: palace.di_pan_gan,
        xing: palace.xing,
        men: palace.men,
        shen: palace.shen,
        gong_wuxing,
        tian_wuxing,
        di_wuxing,
        xing_men_relation,
        wang_shuai,
        is_fu_yin,
        is_fan_yin,
        is_xun_kong,
        is_ma_xing,
        fortune,
        fortune_score,
    }
}

/// 计算宫位吉凶
///
/// 根据宫位的各种因素综合评分
///
/// # 参数
///
/// - `palace`: 宫位信息
/// - `wang_shuai`: 旺衰状态
/// - `xing_men_relation`: 星门关系
/// - `is_fu_yin`: 是否伏吟
/// - `is_fan_yin`: 是否反吟
///
/// # 返回
///
/// (吉凶等级, 吉凶评分)
fn calculate_palace_fortune(
    palace: &Palace,
    wang_shuai: WangShuai,
    xing_men_relation: XingMenRelation,
    is_fu_yin: bool,
    is_fan_yin: bool,
) -> (Fortune, u8) {
    let mut score = 50u8;

    // 1. 旺衰分 (0-20)
    score = score.saturating_add(match wang_shuai {
        WangShuai::WangXiang => 20,
        WangShuai::Xiang => 15,
        WangShuai::Xiu => 10,
        WangShuai::Qiu => 5,
        WangShuai::Si => 0,
    });

    // 2. 九星分 (0-15)
    if palace.xing.is_auspicious() {
        score = score.saturating_add(15);
    }

    // 3. 八门分 (0-15)
    if let Some(men) = palace.men {
        if men.is_auspicious() {
            score = score.saturating_add(15);
        }
    }

    // 4. 八神分 (0-10)
    if let Some(shen) = palace.shen {
        if shen.is_auspicious() {
            score = score.saturating_add(10);
        }
    }

    // 5. 星门关系分 (0-10)
    if xing_men_relation.is_auspicious() {
        score = score.saturating_add(10);
    }

    // 6. 伏吟反吟扣分
    if is_fu_yin {
        score = score.saturating_sub(20);
    }
    if is_fan_yin {
        score = score.saturating_sub(15);
    }

    score = score.min(100);

    // 根据分数确定吉凶等级
    let fortune = match score {
        85..=100 => Fortune::DaJi,
        70..=84 => Fortune::ZhongJi,
        55..=69 => Fortune::XiaoJi,
        40..=54 => Fortune::Ping,
        25..=39 => Fortune::XiaoXiong,
        10..=24 => Fortune::ZhongXiong,
        _ => Fortune::DaXiong,
    };

    (fortune, score)
}

/// 分析用神
///
/// 根据问事类型分析用神的状态
///
/// # 参数
///
/// - `chart`: 奇门遁甲排盘结果
/// - `question_type`: 问事类型
///
/// # 返回
///
/// 用神分析结果
pub fn analyze_yong_shen<AccountId, BlockNumber, MaxCidLen: Get<u32>>(
    chart: &QimenChart<AccountId, BlockNumber, MaxCidLen>,
    question_type: QuestionType,
) -> YongShenAnalysis
where
    AccountId: Clone,
    BlockNumber: Clone,
{
    // 获取必要数据，如果不可用返回默认值
    let palaces = match chart.get_palaces() {
        Some(p) => p,
        None => {
            return YongShenAnalysis {
                question_type,
                primary_gong: JiuGong::Kan,
                primary_type: YongShenType::RiGan,
                secondary_gong: None,
                secondary_type: None,
                wang_shuai: WangShuai::default(),
                de_li: DeLiStatus::default(),
                fortune: Fortune::default(),
                score: 0,
            };
        }
    };
    let day_ganzhi = chart.get_day_ganzhi().unwrap_or_default();
    let hour_ganzhi = chart.get_hour_ganzhi().unwrap_or_default();

    // 1. 确定主用神
    let primary_type = YongShenType::RiGan;
    let primary_gong_num = find_gan_palace(palaces, day_ganzhi.gan);
    let primary_gong = JiuGong::from_num(primary_gong_num).unwrap_or(JiuGong::Kan);

    // 2. 确定次用神（根据问事类型）
    let (secondary_gong, secondary_type) = match question_type {
        QuestionType::Travel | QuestionType::Finding => {
            let shi_gan_gong_num = find_gan_palace(palaces, hour_ganzhi.gan);
            let shi_gan_gong = JiuGong::from_num(shi_gan_gong_num).unwrap_or(JiuGong::Kan);
            (Some(shi_gan_gong), Some(YongShenType::ShiGan))
        }
        _ => (None, None),
    };

    // 3. 分析用神旺衰
    let wang_shuai = analyze_wang_shuai(chart, primary_gong_num);

    // 4. 判断用神得力情况
    let palace = &palaces[(primary_gong_num - 1) as usize];
    let de_li = calculate_de_li_status(palace, wang_shuai);

    // 5. 计算用神吉凶
    let (fortune, score) = calculate_yong_shen_fortune(palace, wang_shuai, de_li);

    YongShenAnalysis {
        question_type,
        primary_gong,
        primary_type,
        secondary_gong,
        secondary_type,
        wang_shuai,
        de_li,
        fortune,
        score,
    }
}

/// 计算得力状态
///
/// 根据旺衰和星门神的吉凶判断得力程度
///
/// # 参数
///
/// - `palace`: 宫位信息
/// - `wang_shuai`: 旺衰状态
///
/// # 返回
///
/// 得力状态
fn calculate_de_li_status(palace: &Palace, wang_shuai: WangShuai) -> DeLiStatus {
    let is_wang = wang_shuai.is_strong();
    let is_ji_xing = palace.xing.is_auspicious();
    let is_ji_men = palace.men.map(|m| m.is_auspicious()).unwrap_or(false);

    match (is_wang, is_ji_xing, is_ji_men) {
        (true, true, true) => DeLiStatus::DaDeLi,
        (true, true, false) | (true, false, true) => DeLiStatus::DeLi,
        (true, false, false) | (false, true, true) => DeLiStatus::Ping,
        (false, true, false) | (false, false, true) => DeLiStatus::ShiLi,
        (false, false, false) => DeLiStatus::DaShiLi,
    }
}

/// 计算用神吉凶
///
/// 根据用神的旺衰和得力情况计算吉凶
///
/// # 参数
///
/// - `palace`: 宫位信息
/// - `wang_shuai`: 旺衰状态
/// - `de_li`: 得力状态
///
/// # 返回
///
/// (吉凶等级, 吉凶评分)
fn calculate_yong_shen_fortune(
    palace: &Palace,
    wang_shuai: WangShuai,
    de_li: DeLiStatus,
) -> (Fortune, u8) {
    let mut score = 50u8;

    // 1. 旺衰分 (0-25)
    score = score.saturating_add(match wang_shuai {
        WangShuai::WangXiang => 25,
        WangShuai::Xiang => 20,
        WangShuai::Xiu => 12,
        WangShuai::Qiu => 6,
        WangShuai::Si => 0,
    });

    // 2. 得力分 (0-25)
    score = score.saturating_add(match de_li {
        DeLiStatus::DaDeLi => 25,
        DeLiStatus::DeLi => 20,
        DeLiStatus::Ping => 12,
        DeLiStatus::ShiLi => 6,
        DeLiStatus::DaShiLi => 0,
    });

    // 3. 特殊状态扣分
    if palace.is_xun_kong {
        score = score.saturating_sub(15);
    }

    score = score.min(100);

    // 根据分数确定吉凶等级
    let fortune = match score {
        85..=100 => Fortune::DaJi,
        70..=84 => Fortune::ZhongJi,
        55..=69 => Fortune::XiaoJi,
        40..=54 => Fortune::Ping,
        25..=39 => Fortune::XiaoXiong,
        10..=24 => Fortune::ZhongXiong,
        _ => Fortune::DaXiong,
    };

    (fortune, score)
}

/// 计算应期
///
/// 预测事情应验的时间
///
/// # 参数
///
/// - `chart`: 奇门遁甲排盘结果
/// - `yong_shen_gong`: 用神宫位
///
/// # 返回
///
/// 应期推算结果
pub fn calculate_ying_qi<AccountId, BlockNumber, MaxCidLen: Get<u32>>(
    chart: &QimenChart<AccountId, BlockNumber, MaxCidLen>,
    yong_shen_gong: u8,
) -> YingQiAnalysis
where
    AccountId: Clone,
    BlockNumber: Clone,
{
    // 获取必要数据
    let palaces = match chart.get_palaces() {
        Some(p) => p,
        None => {
            return YingQiAnalysis {
                primary_num: yong_shen_gong,
                secondary_nums: [1, 1],
                unit: YingQiUnit::default(),
                range_desc: BoundedVec::default(),
                auspicious_times: BoundedVec::default(),
                inauspicious_times: BoundedVec::default(),
            };
        }
    };
    let zhi_fu_xing = chart.get_zhi_fu_xing().unwrap_or_default();
    let zhi_shi_men = chart.get_zhi_shi_men().unwrap_or_default();

    // 1. 主应期数（基于用神宫位）
    let primary_num = yong_shen_gong;

    // 2. 次应期数（基于值符值使）
    let zhi_fu_gong = find_xing_palace(palaces, zhi_fu_xing);
    let zhi_shi_gong = find_men_palace(palaces, zhi_shi_men);
    let secondary_nums = [zhi_fu_gong, zhi_shi_gong];

    // 3. 确定应期单位（根据旺衰）
    let wang_shuai = analyze_wang_shuai(chart, yong_shen_gong);
    let unit = match wang_shuai {
        WangShuai::WangXiang => YingQiUnit::Day,
        WangShuai::Xiang => YingQiUnit::Xun,
        WangShuai::Xiu => YingQiUnit::Month,
        WangShuai::Qiu => YingQiUnit::Season,
        WangShuai::Si => YingQiUnit::Year,
    };

    // 4. 生成应期范围描述
    let range_desc = generate_ying_qi_desc(primary_num, unit);

    // 5. 吉利时间（简化）
    let auspicious_times = BoundedVec::try_from(vec![]).unwrap_or_default();

    // 6. 不利时间（简化）
    let inauspicious_times = BoundedVec::try_from(vec![]).unwrap_or_default();

    YingQiAnalysis {
        primary_num,
        secondary_nums,
        unit,
        range_desc,
        auspicious_times,
        inauspicious_times,
    }
}

/// 查找九星落宫
fn find_xing_palace(palaces: &[Palace; 9], xing: JiuXing) -> u8 {
    for (i, palace) in palaces.iter().enumerate() {
        if palace.xing == xing {
            return (i + 1) as u8;
        }
    }
    1
}

/// 查找八门落宫
fn find_men_palace(palaces: &[Palace; 9], men: BaMen) -> u8 {
    for (i, palace) in palaces.iter().enumerate() {
        if palace.men == Some(men) {
            return (i + 1) as u8;
        }
    }
    1
}

/// 生成应期描述
fn generate_ying_qi_desc(num: u8, unit: YingQiUnit) -> BoundedVec<u8, ConstU32<128>> {
    let desc = match unit {
        YingQiUnit::Hour => format!("应期约在 {} 个时辰内", num),
        YingQiUnit::Day => format!("应期约在 {} 日内", num),
        YingQiUnit::Xun => format!("应期约在 {} 旬内", num),
        YingQiUnit::Month => format!("应期约在 {} 个月内", num),
        YingQiUnit::Season => format!("应期约在 {} 个季度内", num),
        YingQiUnit::Year => format!("应期约在 {} 年内", num),
    };

    BoundedVec::try_from(desc.into_bytes()).unwrap_or_default()
}

/// 获取格局详解
///
/// 根据格局类型返回详细说明
///
/// # 参数
///
/// - `ge_ju`: 格局类型
/// - `question_type`: 问事类型
///
/// # 返回
///
/// 格局详解
fn get_ge_ju_detail(ge_ju: GeJuType, _question_type: QuestionType) -> GeJuDetail {
    let name = BoundedVec::try_from(ge_ju.name().as_bytes().to_vec()).unwrap_or_default();

    let (description, fortune, applicable_scenarios, notes) = match ge_ju {
        GeJuType::ZhengGe => (
            "常规格局，无特殊吉凶，需综合其他因素判断。",
            Fortune::Ping,
            vec![
                QuestionType::General,
                QuestionType::Career,
                QuestionType::Wealth,
            ],
            "正格需要重点看用神旺衰和星门吉凶。",
        ),
        GeJuType::FuYinGe => (
            "伏吟格，天盘地盘相同，主迟滞、反复、难以进展。",
            Fortune::XiaoXiong,
            vec![QuestionType::General],
            "伏吟主事情停滞不前，需耐心等待时机。",
        ),
        GeJuType::FanYinGe => (
            "反吟格，天盘地盘对冲，主变动、反复、不稳定。",
            Fortune::XiaoXiong,
            vec![QuestionType::General],
            "反吟主事情多变，需谨慎应对变化。",
        ),
        GeJuType::TianDunGe => (
            "天遁格，丙奇+天心星+开门，大吉之格，利于求官、考试、谋事。",
            Fortune::DaJi,
            vec![
                QuestionType::Career,
                QuestionType::Study,
                QuestionType::Business,
            ],
            "天遁格遇之大吉，宜积极进取，把握机会。",
        ),
        GeJuType::DiDunGe => (
            "地遁格，乙奇+六合+开门，吉格，利于求财、合作、婚姻。",
            Fortune::ZhongJi,
            vec![
                QuestionType::Wealth,
                QuestionType::Marriage,
                QuestionType::Business,
            ],
            "地遁格利于合作共赢，宜寻求贵人相助。",
        ),
        GeJuType::RenDunGe => (
            "人遁格，丁奇+太阴+开门，吉格，利于隐秘之事、策划、谋略。",
            Fortune::ZhongJi,
            vec![
                QuestionType::Business,
                QuestionType::Investment,
                QuestionType::Prayer,
            ],
            "人遁格宜暗中谋划，不宜张扬。",
        ),
        GeJuType::GuiDunGe => (
            "鬼遁格，丁奇+天心星+开门，特殊格局，利于玄学、医疗。",
            Fortune::XiaoJi,
            vec![QuestionType::Health, QuestionType::Prayer],
            "鬼遁格适合处理特殊事务。",
        ),
        GeJuType::ShenDunGe => (
            "神遁格，九天+值符+开门，吉格，利于高远之事、创新。",
            Fortune::ZhongJi,
            vec![QuestionType::Career, QuestionType::Study],
            "神遁格宜高瞻远瞩，开拓创新。",
        ),
        GeJuType::LongDunGe => (
            "龙遁格，九地+值符+开门，吉格，利于稳固、长久之事。",
            Fortune::ZhongJi,
            vec![QuestionType::Career, QuestionType::Investment],
            "龙遁格宜稳扎稳打，长远规划。",
        ),
        GeJuType::QingLongFanShou => (
            "青龙返首，特殊吉格，主贵人相助、事业有成。",
            Fortune::DaJi,
            vec![QuestionType::Career, QuestionType::Business],
            "青龙返首遇之大吉，宜积极行动。",
        ),
        GeJuType::FeiNiaoDieXue => (
            "飞鸟跌穴，特殊凶格，主失败、挫折、不利。",
            Fortune::DaXiong,
            vec![QuestionType::General],
            "飞鸟跌穴遇之不利，宜谨慎行事，避免冒险。",
        ),
    };

    let description_vec = BoundedVec::try_from(description.as_bytes().to_vec()).unwrap_or_default();
    let notes_vec = BoundedVec::try_from(notes.as_bytes().to_vec()).unwrap_or_default();
    let scenarios_vec = BoundedVec::try_from(applicable_scenarios).unwrap_or_default();

    GeJuDetail {
        ge_ju,
        name,
        description: description_vec,
        fortune,
        applicable_scenarios: scenarios_vec,
        notes: notes_vec,
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;
    use codec::Encode;

    #[test]
    fn test_core_interpretation_size() {
        let core = QimenCoreInterpretation {
            ge_ju: GeJuType::ZhengGe,
            yong_shen_gong: 1,
            zhi_fu_xing: JiuXing::TianQin,
            zhi_shi_men: BaMen::Kai,
            ri_gan_gong: 1,
            shi_gan_gong: 1,
            fortune: Fortune::Ping,
            fortune_score: 50,
            wang_shuai: WangShuai::Xiu,
            special_patterns: 0,
            confidence: 80,
            timestamp: 1000000,
            algorithm_version: 1,
        };

        let encoded = core.encode();
        println!("✅ QimenCoreInterpretation 编码大小: {} bytes", encoded.len());
        assert!(
            encoded.len() <= 20,
            "QimenCoreInterpretation 编码大小应 <= 20 bytes，实际: {} bytes",
            encoded.len()
        );
    }

    #[test]
    fn test_special_patterns() {
        let mut core = QimenCoreInterpretation {
            ge_ju: GeJuType::ZhengGe,
            yong_shen_gong: 1,
            zhi_fu_xing: JiuXing::TianQin,
            zhi_shi_men: BaMen::Kai,
            ri_gan_gong: 1,
            shi_gan_gong: 1,
            fortune: Fortune::Ping,
            fortune_score: 50,
            wang_shuai: WangShuai::Xiu,
            special_patterns: 0,
            confidence: 80,
            timestamp: 1000000,
            algorithm_version: 1,
        };

        // 测试设置特殊格局
        core.set_special_pattern(0); // 伏吟
        assert!(core.is_fu_yin());

        core.set_special_pattern(2); // 天遁
        assert!(core.is_tian_dun());

        // 验证多个标记可以同时存在
        assert!(core.is_fu_yin());
        assert!(core.is_tian_dun());
    }

    #[test]
    fn test_encode_decode() {
        let original = QimenCoreInterpretation {
            ge_ju: GeJuType::TianDunGe,
            yong_shen_gong: 5,
            zhi_fu_xing: JiuXing::TianXin,
            zhi_shi_men: BaMen::Kai,
            ri_gan_gong: 3,
            shi_gan_gong: 7,
            fortune: Fortune::DaJi,
            fortune_score: 95,
            wang_shuai: WangShuai::WangXiang,
            special_patterns: 0b00000100, // 天遁
            confidence: 90,
            timestamp: 2000000,
            algorithm_version: 1,
        };

        let encoded = original.encode();
        let decoded = QimenCoreInterpretation::decode(&mut &encoded[..]).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(decoded.ge_ju, GeJuType::TianDunGe);
        assert_eq!(decoded.fortune, Fortune::DaJi);
        assert_eq!(decoded.fortune_score, 95);
    }
}
