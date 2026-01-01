//! 塔罗牌解卦数据结构模块
//!
//! 本模块定义了塔罗牌占卜的解卦数据结构，包括：
//! - 核心解卦指标（TarotCoreInterpretation）- 约30 bytes
//! - 能量分析（SpreadEnergyAnalysis）
//! - 单牌分析（CardInterpretation）
//! - 完整解卦（TarotFullInterpretation）
//!
//! 设计原则：
//! 1. 分层存储：核心指标链上存储，详细解释通过 Runtime API 实时计算
//! 2. 存储优化：使用枚举索引和位图，避免存储冗余文本
//! 3. 实时计算：通过 Runtime API 免费获取完整解读
//! 4. AI友好：结构化数据便于AI深度解读

use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_std::prelude::*;

// ============================================================================
// 枚举类型定义
// ============================================================================

/// 吉凶倾向等级
///
/// 用于表示占卜结果的总体吉凶倾向
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum FortuneTendency {
    /// 大吉 - 诸事顺遂，心想事成
    Excellent = 0,
    /// 吉 - 事可成，宜进取
    Good = 1,
    /// 中平 - 平稳发展，守成为上
    Neutral = 2,
    /// 小凶 - 小有阻碍，谨慎行事
    MinorBad = 3,
    /// 凶 - 困难重重，需要调整
    Bad = 4,
}

impl Default for FortuneTendency {
    fn default() -> Self {
        FortuneTendency::Neutral
    }
}

/// 主导元素类型
///
/// 表示牌阵中占主导地位的元素能量
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum DominantElement {
    /// 无明显主导元素
    None = 0,
    /// 火元素主导（权杖）- 行动力、激情、创造力
    Fire = 1,
    /// 水元素主导（圣杯）- 情感、直觉、人际关系
    Water = 2,
    /// 风元素主导（宝剑）- 思维、沟通、智力活动
    Air = 3,
    /// 土元素主导（星币）- 物质、工作、实际事务
    Earth = 4,
    /// 灵性主导（大阿卡纳）- 重大转折、命运指引
    Spirit = 5,
}

impl Default for DominantElement {
    fn default() -> Self {
        DominantElement::None
    }
}

/// 能量流动方向
///
/// 表示牌阵中能量的变化趋势
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum EnergyFlow {
    /// 上升 - 能量逐渐增强
    Rising = 0,
    /// 下降 - 能量逐渐减弱
    Declining = 1,
    /// 平稳 - 能量保持稳定
    Stable = 2,
    /// 波动 - 能量起伏不定
    Volatile = 3,
}

impl Default for EnergyFlow {
    fn default() -> Self {
        EnergyFlow::Stable
    }
}

/// 牌间关系类型
///
/// 表示两张牌之间的能量互动关系
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum RelationshipType {
    /// 无明显关系
    None = 0,
    /// 相生 - 能量互相增强
    Generating = 1,
    /// 相克 - 能量互相制约
    Controlling = 2,
    /// 同元素强化 - 同类能量叠加
    SameElementReinforce = 3,
    /// 对立冲突 - 能量相互对抗
    Opposing = 4,
    /// 互补 - 能量相互补充
    Complementary = 5,
}

impl Default for RelationshipType {
    fn default() -> Self {
        RelationshipType::None
    }
}

/// 时间线趋势
///
/// 表示过去/现在/未来的发展趋势
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum TimelineTrend {
    /// 下降趋势
    Declining = 0,
    /// 平稳趋势
    Stable = 1,
    /// 上升趋势
    Rising = 2,
}

impl Default for TimelineTrend {
    fn default() -> Self {
        TimelineTrend::Stable
    }
}

/// 时间线状态
///
/// 表示当前所处的状态位置
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum TimelineState {
    /// 低谷期
    LowPoint = 0,
    /// 平稳期
    Stable = 1,
    /// 高峰期
    HighPoint = 2,
}

impl Default for TimelineState {
    fn default() -> Self {
        TimelineState::Stable
    }
}

/// 整体发展方向
///
/// 表示事态的总体发展方向
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum OverallDirection {
    /// 负面发展
    Negative = 0,
    /// 中性发展
    Neutral = 1,
    /// 正面发展
    Positive = 2,
}

impl Default for OverallDirection {
    fn default() -> Self {
        OverallDirection::Neutral
    }
}

// ============================================================================
// 核心数据结构
// ============================================================================

/// 塔罗牌核心解卦结果
///
/// 包含塔罗占卜的核心判断指标
/// 总大小：约 30 bytes
///
/// # 字段说明
/// - 基础判断（4 bytes）：总体能量、主导元素、吉凶倾向、逆位比例
/// - 牌组特征（8 bytes）：各类型牌数量、元素分布、特殊组合
/// - 能量分析（8 bytes）：六大能量指数、综合评分
/// - 元数据（10 bytes）：区块号、算法版本、可信度、保留字段
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub struct TarotCoreInterpretation {
    // ===== 基础判断 (4 bytes) =====

    /// 总体能量等级 (1 byte, 0-100)
    ///
    /// 计算公式：(正位牌数 × 10 + 大阿卡纳数 × 15) / 牌数
    pub overall_energy: u8,

    /// 主导元素 (1 byte)
    pub dominant_element: DominantElement,

    /// 吉凶倾向 (1 byte)
    pub fortune_tendency: FortuneTendency,

    /// 逆位比例 (1 byte, 0-100)
    pub reversed_ratio: u8,

    // ===== 牌组特征 (8 bytes) =====

    /// 大阿卡纳数量 (1 byte, 0-12)
    pub major_arcana_count: u8,

    /// 宫廷牌数量 (1 byte, 0-12)
    pub court_cards_count: u8,

    /// 数字牌数量 (1 byte, 0-12)
    pub number_cards_count: u8,

    /// 元素分布位图 (1 byte)
    ///
    /// bit 0-1: 火元素数量(0-3)
    /// bit 2-3: 水元素数量(0-3)
    /// bit 4-5: 风元素数量(0-3)
    /// bit 6-7: 土元素数量(0-3)
    pub element_bitmap: u8,

    /// 特殊组合标志 (1 byte, 位图)
    ///
    /// bit 0: 愚者+世界组合
    /// bit 1: 三张以上大阿卡纳
    /// bit 2: 同花色三连号
    /// bit 3: 全逆位
    /// bit 4: 全正位
    /// bit 5-7: 保留
    pub special_combination: u8,

    /// 关键牌ID (1 byte, 0-77)
    ///
    /// 牌阵中最重要的牌（通常是第一张或中心牌）
    pub key_card_id: u8,

    /// 关键牌正逆位 (1 byte)
    ///
    /// 0=正位, 1=逆位
    pub key_card_reversed: u8,

    /// 牌阵类型 (1 byte)
    pub spread_type: u8,

    // ===== 能量分析 (8 bytes) =====

    /// 行动力指数 (1 byte, 0-100)
    ///
    /// 基于权杖牌和正位牌比例
    /// 计算公式：权杖牌数 × 25 + 正位比例 × 0.5
    pub action_index: u8,

    /// 情感指数 (1 byte, 0-100)
    ///
    /// 基于圣杯牌数量
    /// 计算公式：圣杯牌数 × 25
    pub emotion_index: u8,

    /// 思维指数 (1 byte, 0-100)
    ///
    /// 基于宝剑牌数量
    /// 计算公式：宝剑牌数 × 25
    pub intellect_index: u8,

    /// 物质指数 (1 byte, 0-100)
    ///
    /// 基于星币牌数量
    /// 计算公式：星币牌数 × 25
    pub material_index: u8,

    /// 灵性指数 (1 byte, 0-100)
    ///
    /// 基于大阿卡纳比例
    /// 计算公式：大阿卡纳比例 × 100
    pub spiritual_index: u8,

    /// 稳定性指数 (1 byte, 0-100)
    ///
    /// 基于正位比例和数字牌分布
    pub stability_index: u8,

    /// 变化性指数 (1 byte, 0-100)
    ///
    /// 基于逆位比例和宫廷牌数量
    pub change_index: u8,

    /// 综合评分 (1 byte, 0-100)
    pub overall_score: u8,

    // ===== 元数据 (10 bytes) =====

    /// 解卦时间戳 - 区块号 (4 bytes)
    pub block_number: u32,

    /// 解卦算法版本 (1 byte)
    pub algorithm_version: u8,

    /// 可信度 (1 byte, 0-100)
    pub confidence: u8,

    /// 保留字段 (4 bytes)
    pub reserved: [u8; 4],
}

impl TarotCoreInterpretation {
    /// 获取火元素数量
    pub fn fire_count(&self) -> u8 {
        self.element_bitmap & 0b00000011
    }

    /// 获取水元素数量
    pub fn water_count(&self) -> u8 {
        (self.element_bitmap >> 2) & 0b00000011
    }

    /// 获取风元素数量
    pub fn air_count(&self) -> u8 {
        (self.element_bitmap >> 4) & 0b00000011
    }

    /// 获取土元素数量
    pub fn earth_count(&self) -> u8 {
        (self.element_bitmap >> 6) & 0b00000011
    }

    /// 检查是否有愚者+世界组合
    pub fn has_fool_world_combo(&self) -> bool {
        self.special_combination & 0b00000001 != 0
    }

    /// 检查是否有三张以上大阿卡纳
    pub fn has_many_major_arcana(&self) -> bool {
        self.special_combination & 0b00000010 != 0
    }

    /// 检查是否有同花色三连号
    pub fn has_same_suit_sequence(&self) -> bool {
        self.special_combination & 0b00000100 != 0
    }

    /// 检查是否全逆位
    pub fn is_all_reversed(&self) -> bool {
        self.special_combination & 0b00001000 != 0
    }

    /// 检查是否全正位
    pub fn is_all_upright(&self) -> bool {
        self.special_combination & 0b00010000 != 0
    }
}

/// 牌阵能量分析
///
/// 分析牌阵的整体能量分布和流动趋势
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub struct SpreadEnergyAnalysis {
    /// 过去能量 (0-100)
    ///
    /// 基于牌阵前1/3的牌
    pub past_energy: u8,

    /// 现在能量 (0-100)
    ///
    /// 基于牌阵中间1/3的牌
    pub present_energy: u8,

    /// 未来能量 (0-100)
    ///
    /// 基于牌阵后1/3的牌
    pub future_energy: u8,

    /// 内在能量 (0-100)
    ///
    /// 基于逆位牌和内心相关位置
    pub inner_energy: u8,

    /// 外在能量 (0-100)
    ///
    /// 基于正位牌和外部环境相关位置
    pub outer_energy: u8,

    /// 能量流动方向
    pub energy_flow: EnergyFlow,

    /// 能量平衡度 (0-100, 100最平衡)
    pub energy_balance: u8,
}

/// 单张牌的解读分析
///
/// 包含单张牌在特定牌阵位置的详细分析
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub struct CardInterpretation {
    /// 牌ID (0-77)
    pub card_id: u8,

    /// 是否逆位
    pub is_reversed: bool,

    /// 在牌阵中的位置索引 (0-based)
    pub spread_position: u8,

    /// 位置权重 (1-10, 10最重要)
    pub position_weight: u8,

    /// 牌的能量强度 (0-100)
    pub energy_strength: u8,

    /// 与前一张牌的关系类型
    pub relation_to_prev: RelationshipType,

    /// 与后一张牌的关系类型
    pub relation_to_next: RelationshipType,
}

/// 牌间关系
///
/// 描述两张牌之间的能量互动关系
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub struct CardRelationship {
    /// 第一张牌索引
    pub card1_index: u8,

    /// 第二张牌索引
    pub card2_index: u8,

    /// 关系类型
    pub relationship_type: RelationshipType,

    /// 关系强度 (0-100)
    pub strength: u8,
}

/// 时间线分析
///
/// 分析牌阵中的时间发展趋势（仅适用于时间相关牌阵）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub struct TimelineAnalysis {
    /// 过去趋势
    pub past_trend: TimelineTrend,

    /// 现在状态
    pub present_state: TimelineState,

    /// 未来趋势
    pub future_trend: TimelineTrend,

    /// 转折点位置 (牌阵索引, 255=无转折点)
    pub turning_point: u8,

    /// 整体发展方向
    pub overall_direction: OverallDirection,
}

/// 塔罗牌完整解卦结果
///
/// 包含所有解卦信息的完整数据结构
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxCards))]
pub struct TarotFullInterpretation<MaxCards: Get<u32>> {
    /// 核心指标（必有）
    pub core: TarotCoreInterpretation,

    /// 牌阵能量分析（必有）
    pub spread_energy: SpreadEnergyAnalysis,

    /// 各牌分析（可选，最多12张）
    pub card_analyses: Option<BoundedVec<CardInterpretation, MaxCards>>,

    /// 牌间关系分析（可选）
    pub card_relationships: Option<BoundedVec<CardRelationship, MaxCards>>,

    /// 时间线分析（可选，仅适用于时间相关牌阵）
    pub timeline_analysis: Option<TimelineAnalysis>,
}

// ============================================================================
// 解读文本类型枚举
// ============================================================================

/// 解读文本类型枚举
///
/// 用于链上存储解读文本索引，前端根据索引显示对应文本。
/// 这种设计避免在链上存储大量文本，同时保持解读的一致性。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum InterpretationTextType {
    // ===== 总体能量描述 (0-9) =====

    /// 能量充沛，积极向上
    EnergyHigh = 0,
    /// 能量平稳，稳中求进
    EnergyMedium = 1,
    /// 能量低迷，需要休息
    EnergyLow = 2,
    /// 能量波动，变化较大
    EnergyVolatile = 3,

    // ===== 元素主导描述 (10-19) =====

    /// 火元素主导：行动力强，充满激情
    FireDominant = 10,
    /// 水元素主导：情感丰富，直觉敏锐
    WaterDominant = 11,
    /// 风元素主导：思维活跃，沟通顺畅
    AirDominant = 12,
    /// 土元素主导：务实稳重，注重物质
    EarthDominant = 13,
    /// 灵性主导：重大转折，命运指引
    SpiritDominant = 14,
    /// 元素平衡：各方面均衡发展
    ElementBalanced = 15,

    // ===== 吉凶判断 (20-29) =====

    /// 大吉：诸事顺遂，心想事成
    FortuneExcellent = 20,
    /// 吉：事可成，宜进取
    FortuneGood = 21,
    /// 中平：平稳发展，守成为上
    FortuneNeutral = 22,
    /// 小凶：小有阻碍，谨慎行事
    FortuneMinorBad = 23,
    /// 凶：困难重重，需要调整
    FortuneBad = 24,

    // ===== 特殊组合 (30-39) =====

    /// 愚者+世界：完整的旅程，新的循环
    FoolWorldCombo = 30,
    /// 多张大阿卡纳：重大人生课题
    ManyMajorArcana = 31,
    /// 同花色连号：该领域有重要发展
    SameSuitSequence = 32,
    /// 全逆位：内省时期，需要调整
    AllReversed = 33,
    /// 全正位：外向发展，积极行动
    AllUpright = 34,

    // ===== 行动建议 (40-59) =====

    /// 积极行动，把握机会
    ActionTakeAction = 40,
    /// 谨慎观察，等待时机
    ActionWaitAndSee = 41,
    /// 内省调整，修正方向
    ActionReflect = 42,
    /// 寻求帮助，借助外力
    ActionSeekHelp = 43,
    /// 坚持信念，持续努力
    ActionPersist = 44,
    /// 放下执念，顺其自然
    ActionLetGo = 45,
    /// 沟通交流，化解误会
    ActionCommunicate = 46,
    /// 学习成长，提升自我
    ActionLearn = 47,

    // ===== 时间线描述 (60-69) =====

    /// 过去：基础稳固
    PastSolid = 60,
    /// 过去：经历挑战
    PastChallenging = 61,
    /// 现在：转折点
    PresentTurning = 62,
    /// 现在：稳定期
    PresentStable = 63,
    /// 未来：向好发展
    FutureImproving = 64,
    /// 未来：需要警惕
    FutureWarning = 65,
    /// 整体上升趋势
    TrendRising = 66,
    /// 整体下降趋势
    TrendDeclining = 67,
    /// 整体平稳发展
    TrendStable = 68,

    // ===== 能量指数描述 (70-79) =====

    /// 行动力充沛
    ActionIndexHigh = 70,
    /// 情感丰富
    EmotionIndexHigh = 71,
    /// 思维清晰
    IntellectIndexHigh = 72,
    /// 物质运势好
    MaterialIndexHigh = 73,
    /// 灵性成长期
    SpiritualIndexHigh = 74,
    /// 稳定性强
    StabilityIndexHigh = 75,
    /// 变化性强
    ChangeIndexHigh = 76,
}

impl Default for InterpretationTextType {
    fn default() -> Self {
        InterpretationTextType::FortuneNeutral
    }
}

impl InterpretationTextType {
    /// 获取文本类型的中文描述
    pub fn description(&self) -> &'static str {
        match self {
            // 能量描述
            InterpretationTextType::EnergyHigh => "能量充沛，积极向上",
            InterpretationTextType::EnergyMedium => "能量平稳，稳中求进",
            InterpretationTextType::EnergyLow => "能量低迷，需要休息",
            InterpretationTextType::EnergyVolatile => "能量波动，变化较大",

            // 元素主导
            InterpretationTextType::FireDominant => "火元素主导：行动力强，充满激情",
            InterpretationTextType::WaterDominant => "水元素主导：情感丰富，直觉敏锐",
            InterpretationTextType::AirDominant => "风元素主导：思维活跃，沟通顺畅",
            InterpretationTextType::EarthDominant => "土元素主导：务实稳重，注重物质",
            InterpretationTextType::SpiritDominant => "灵性主导：重大转折，命运指引",
            InterpretationTextType::ElementBalanced => "元素平衡：各方面均衡发展",

            // 吉凶判断
            InterpretationTextType::FortuneExcellent => "大吉：诸事顺遂，心想事成",
            InterpretationTextType::FortuneGood => "吉：事可成，宜进取",
            InterpretationTextType::FortuneNeutral => "中平：平稳发展，守成为上",
            InterpretationTextType::FortuneMinorBad => "小凶：小有阻碍，谨慎行事",
            InterpretationTextType::FortuneBad => "凶：困难重重，需要调整",

            // 特殊组合
            InterpretationTextType::FoolWorldCombo => "愚者与世界相遇：完整的旅程，新的循环开始",
            InterpretationTextType::ManyMajorArcana => "多张大阿卡纳出现：重大人生课题，命运转折",
            InterpretationTextType::SameSuitSequence => "同花色连号：该领域有重要发展和突破",
            InterpretationTextType::AllReversed => "全逆位：内省时期，需要调整心态和方向",
            InterpretationTextType::AllUpright => "全正位：外向发展期，积极行动会有收获",

            // 行动建议
            InterpretationTextType::ActionTakeAction => "建议：积极行动，把握当前机会",
            InterpretationTextType::ActionWaitAndSee => "建议：谨慎观察，等待更好时机",
            InterpretationTextType::ActionReflect => "建议：内省调整，修正前进方向",
            InterpretationTextType::ActionSeekHelp => "建议：寻求帮助，借助外力突破",
            InterpretationTextType::ActionPersist => "建议：坚持信念，持续努力终有回报",
            InterpretationTextType::ActionLetGo => "建议：放下执念，顺其自然会更好",
            InterpretationTextType::ActionCommunicate => "建议：加强沟通交流，化解可能的误会",
            InterpretationTextType::ActionLearn => "建议：学习成长，提升自我能力",

            // 时间线
            InterpretationTextType::PastSolid => "过去：打下了稳固的基础",
            InterpretationTextType::PastChallenging => "过去：经历了一些挑战和考验",
            InterpretationTextType::PresentTurning => "现在：处于重要的转折点",
            InterpretationTextType::PresentStable => "现在：处于相对稳定的时期",
            InterpretationTextType::FutureImproving => "未来：形势将向好发展",
            InterpretationTextType::FutureWarning => "未来：需要警惕潜在风险",
            InterpretationTextType::TrendRising => "整体趋势：能量上升，形势向好",
            InterpretationTextType::TrendDeclining => "整体趋势：能量下降，需要调整",
            InterpretationTextType::TrendStable => "整体趋势：平稳发展，稳中求进",

            // 能量指数
            InterpretationTextType::ActionIndexHigh => "行动力充沛，适合积极推进计划",
            InterpretationTextType::EmotionIndexHigh => "情感丰富，人际关系是重点",
            InterpretationTextType::IntellectIndexHigh => "思维清晰，适合做重要决策",
            InterpretationTextType::MaterialIndexHigh => "物质运势好，财务方面有利",
            InterpretationTextType::SpiritualIndexHigh => "灵性成长期，适合内在修炼",
            InterpretationTextType::StabilityIndexHigh => "稳定性强，适合长期规划",
            InterpretationTextType::ChangeIndexHigh => "变化性强，需要灵活应对",
        }
    }
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_interpretation_size() {
        use sp_std::mem::size_of;

        // 验证核心解卦结构大小约为30 bytes
        let size = size_of::<TarotCoreInterpretation>();
        assert!(size <= 32, "TarotCoreInterpretation size {} exceeds 32 bytes", size);
    }

    #[test]
    fn test_element_bitmap() {
        let mut core = TarotCoreInterpretation::default();

        // 设置元素分布：火2、水1、风3、土0
        core.element_bitmap = 0b00_11_01_10;

        assert_eq!(core.fire_count(), 2);
        assert_eq!(core.water_count(), 1);
        assert_eq!(core.air_count(), 3);
        assert_eq!(core.earth_count(), 0);
    }

    #[test]
    fn test_special_combination_flags() {
        let mut core = TarotCoreInterpretation::default();

        // 设置特殊组合标志
        core.special_combination = 0b00010111;

        assert!(core.has_fool_world_combo());
        assert!(core.has_many_major_arcana());
        assert!(core.has_same_suit_sequence());
        assert!(!core.is_all_reversed());
        assert!(core.is_all_upright());
    }

    #[test]
    fn test_fortune_tendency() {
        let tendency = FortuneTendency::Good;
        assert_eq!(tendency as u8, 1);
    }

    #[test]
    fn test_dominant_element() {
        let element = DominantElement::Fire;
        assert_eq!(element as u8, 1);
    }

    #[test]
    fn test_relationship_type() {
        let relation = RelationshipType::Generating;
        assert_eq!(relation as u8, 1);
    }
}
