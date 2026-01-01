//! 塔罗牌 - 基础数据类型定义
//!
//! 本模块定义了塔罗牌排盘系统所需的所有核心数据结构，包括：
//! - 牌组类型 (CardType) - 大阿卡纳/小阿卡纳
//! - 花色 (Suit) - 权杖/圣杯/宝剑/星币
//! - 塔罗牌 (TarotCard) - 单张牌的完整信息
//! - 牌阵类型 (SpreadType) - 支持的牌阵布局
//! - 占卜记录 (TarotReading) - 完整的占卜结果

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_std::prelude::*;

// 重新导出 privacy pallet 的类型，供外部使用
pub use pallet_divination_privacy::types::PrivacyMode;

/// 塔罗牌类型 - 大阿卡纳 vs 小阿卡纳
///
/// 大阿卡纳（Major Arcana）: 22张，代表人生重大主题和精神旅程
/// 小阿卡纳（Minor Arcana）: 56张，分四种花色，代表日常生活事务
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum CardType {
    /// 大阿卡纳 - 22张主牌（0-21号）
    #[default]
    MajorArcana = 0,
    /// 小阿卡纳 - 56张副牌（分四种花色）
    MinorArcana = 1,
}

/// 小阿卡纳花色枚举
///
/// 四种花色对应四大元素：
/// - 权杖(Wands) - 火元素，代表激情、创造力、行动
/// - 圣杯(Cups) - 水元素，代表情感、关系、直觉
/// - 宝剑(Swords) - 风元素，代表思想、沟通、冲突
/// - 星币(Pentacles) - 土元素，代表物质、金钱、工作
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum Suit {
    /// 无花色（大阿卡纳专用）
    #[default]
    None = 0,
    /// 权杖 - 火元素
    Wands = 1,
    /// 圣杯 - 水元素
    Cups = 2,
    /// 宝剑 - 风元素
    Swords = 3,
    /// 星币 - 土元素
    Pentacles = 4,
}

impl Suit {
    /// 获取花色对应的元素名称
    pub fn element(&self) -> &'static str {
        match self {
            Suit::None => "无",
            Suit::Wands => "火",
            Suit::Cups => "水",
            Suit::Swords => "风",
            Suit::Pentacles => "土",
        }
    }
}

/// 宫廷牌等级
///
/// 小阿卡纳中的宫廷牌（11-14号）
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum CourtRank {
    /// 非宫廷牌
    #[default]
    None = 0,
    /// 侍从 - 学习者、信使
    Page = 11,
    /// 骑士 - 行动者、追求者
    Knight = 12,
    /// 王后 - 滋养者、直觉
    Queen = 13,
    /// 国王 - 掌控者、权威
    King = 14,
}

/// 牌的正逆位状态
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum CardPosition {
    /// 正位 - 牌意的正面表达
    #[default]
    Upright = 0,
    /// 逆位 - 牌意的负面/内化表达
    Reversed = 1,
}

impl CardPosition {
    /// 从布尔值创建（true=逆位）
    pub fn from_bool(reversed: bool) -> Self {
        if reversed {
            CardPosition::Reversed
        } else {
            CardPosition::Upright
        }
    }

    /// 是否为逆位
    pub fn is_reversed(&self) -> bool {
        matches!(self, CardPosition::Reversed)
    }
}

/// 塔罗牌结构 - 单张牌的完整信息
///
/// 编号规则：
/// - 0-21: 大阿卡纳（愚者=0, 魔术师=1, ... 世界=21）
/// - 22-35: 权杖（Ace=22, 2=23, ... 10=31, Page=32, Knight=33, Queen=34, King=35）
/// - 36-49: 圣杯
/// - 50-63: 宝剑
/// - 64-77: 星币
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct TarotCard {
    /// 牌的唯一编号 (0-77)
    pub id: u8,
    /// 牌的类型（大/小阿卡纳）
    pub card_type: CardType,
    /// 花色（仅小阿卡纳有效）
    pub suit: Suit,
    /// 牌面数值（大阿卡纳0-21，小阿卡纳1-14）
    pub number: u8,
}

impl TarotCard {
    /// 从编号创建塔罗牌
    ///
    /// # 参数
    /// - `id`: 牌的编号 (0-77)
    ///
    /// # 编号映射
    /// - 0-21: 大阿卡纳
    /// - 22-35: 权杖 (Ace-King)
    /// - 36-49: 圣杯
    /// - 50-63: 宝剑
    /// - 64-77: 星币
    pub fn from_id(id: u8) -> Self {
        let id = id % 78; // 确保在有效范围内

        if id < 22 {
            // 大阿卡纳
            TarotCard {
                id,
                card_type: CardType::MajorArcana,
                suit: Suit::None,
                number: id,
            }
        } else {
            // 小阿卡纳
            let minor_id = id - 22; // 0-55
            let suit_index = minor_id / 14; // 0-3
            let card_number = (minor_id % 14) + 1; // 1-14

            let suit = match suit_index {
                0 => Suit::Wands,
                1 => Suit::Cups,
                2 => Suit::Swords,
                _ => Suit::Pentacles,
            };

            TarotCard {
                id,
                card_type: CardType::MinorArcana,
                suit,
                number: card_number,
            }
        }
    }

    /// 判断是否为大阿卡纳
    pub fn is_major(&self) -> bool {
        matches!(self.card_type, CardType::MajorArcana)
    }

    /// 判断是否为宫廷牌
    pub fn is_court_card(&self) -> bool {
        matches!(self.card_type, CardType::MinorArcana) && self.number >= 11
    }

    /// 获取宫廷牌等级
    pub fn court_rank(&self) -> CourtRank {
        if !self.is_court_card() {
            return CourtRank::None;
        }
        match self.number {
            11 => CourtRank::Page,
            12 => CourtRank::Knight,
            13 => CourtRank::Queen,
            14 => CourtRank::King,
            _ => CourtRank::None,
        }
    }

    // ========================================================================
    // 牌义获取方法
    // ========================================================================

    /// 获取牌的正位含义
    ///
    /// # 返回
    /// - 正位含义字符串
    pub fn upright_meaning(&self) -> &'static str {
        crate::constants::get_upright_meaning(self.id)
    }

    /// 获取牌的逆位含义
    ///
    /// # 返回
    /// - 逆位含义字符串
    pub fn reversed_meaning(&self) -> &'static str {
        crate::constants::get_reversed_meaning(self.id)
    }

    /// 获取牌的关键词
    ///
    /// # 返回
    /// - 关键词字符串
    pub fn keywords(&self) -> &'static str {
        crate::constants::get_keywords(self.id)
    }

    /// 获取大阿卡纳牌的详细描述
    ///
    /// # 返回
    /// - 牌面描述，小阿卡纳返回 None
    pub fn description(&self) -> Option<&'static str> {
        crate::constants::get_major_description(self.id)
    }

    /// 获取大阿卡纳牌的星座/行星对应
    ///
    /// # 返回
    /// - (天体/星座名称, 元素属性)，小阿卡纳返回 None
    pub fn astrology(&self) -> Option<(&'static str, &'static str)> {
        crate::constants::get_major_astrology(self.id)
    }

    /// 获取大阿卡纳牌的数字象征
    ///
    /// # 返回
    /// - 数字象征意义，小阿卡纳返回 None
    pub fn numerology(&self) -> Option<&'static str> {
        crate::constants::get_major_numerology(self.id)
    }

    /// 获取牌的中文名称
    ///
    /// # 返回
    /// - (主名称, 副名称Option)
    ///   - 大阿卡纳: ("愚者", None)
    ///   - 小阿卡纳: ("权杖", Some("Ace"))
    pub fn display_name(&self) -> (&'static str, Option<&'static str>) {
        crate::constants::get_card_display_name(self.id)
    }

    /// 获取牌的完整中文名称字符串
    ///
    /// # 返回
    /// - 完整名称，如 "愚者" 或 "权杖Ace"
    pub fn full_name(&self) -> &'static str {
        if self.id < 22 {
            crate::constants::MAJOR_ARCANA_NAMES_CN[self.id as usize]
        } else {
            // 小阿卡纳需要组合花色和牌面
            // 这里返回花色名，完整名称需要在前端组合
            let (suit_name, _) = self.display_name();
            suit_name
        }
    }

    /// 获取牌的英文名称
    ///
    /// # 返回
    /// - 英文名称
    pub fn english_name(&self) -> &'static str {
        if self.id < 22 {
            crate::constants::MAJOR_ARCANA_NAMES_EN[self.id as usize]
        } else {
            let suit_index = crate::constants::get_suit_index(self.id) as usize;
            if suit_index < crate::constants::SUIT_NAMES_EN.len() {
                crate::constants::SUIT_NAMES_EN[suit_index]
            } else {
                ""
            }
        }
    }

    /// 获取牌的元素属性
    ///
    /// # 返回
    /// - 元素名称（火/水/风/土），大阿卡纳返回空字符串
    pub fn element(&self) -> &'static str {
        crate::constants::get_card_element(self.id)
    }

    /// 获取花色索引
    ///
    /// # 返回
    /// - 花色索引: 0=无(大阿卡纳), 1=权杖, 2=圣杯, 3=宝剑, 4=星币
    pub fn suit_index(&self) -> u8 {
        crate::constants::get_suit_index(self.id)
    }

    /// 获取花色的详细描述
    ///
    /// # 返回
    /// - 花色描述，大阿卡纳返回 None
    pub fn suit_description(&self) -> Option<&'static str> {
        let suit_index = self.suit_index();
        crate::constants::get_suit_description(suit_index)
    }

    /// 获取数字牌的数字象征意义
    ///
    /// # 返回
    /// - 数字象征意义，仅小阿卡纳数字牌有效（1-10）
    pub fn number_symbolism(&self) -> Option<&'static str> {
        if self.is_major() || self.number > 10 {
            None
        } else {
            crate::constants::get_number_symbolism(self.number)
        }
    }

    /// 获取牌的完整牌义信息
    ///
    /// # 返回
    /// - CardMeaning 结构，包含所有牌义信息
    pub fn get_meaning(&self) -> Option<crate::constants::CardMeaning> {
        crate::constants::get_card_meaning(self.id)
    }
}

/// 牌阵类型枚举
///
/// 定义了多种常用的塔罗牌牌阵
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum SpreadType {
    /// 单张牌 - 快速指引
    #[default]
    SingleCard = 1,
    /// 三张牌（时间线）- 过去/现在/未来
    ThreeCardTime = 3,
    /// 三张牌（情况）- 情况/行动/结果
    ThreeCardSituation = 4,
    /// 五张牌 - 爱情关系牌阵
    LoveRelationship = 5,
    /// 六张牌 - 事业指导牌阵
    CareerGuidance = 6,
    /// 七张牌 - 决策分析牌阵
    DecisionMaking = 7,
    /// 十张牌 - 凯尔特十字（最经典的牌阵）
    CelticCross = 10,
    /// 十二张牌 - 年度运势
    YearForecast = 12,
}

impl SpreadType {
    /// 获取牌阵所需的牌数
    pub fn card_count(&self) -> u8 {
        match self {
            SpreadType::SingleCard => 1,
            SpreadType::ThreeCardTime | SpreadType::ThreeCardSituation => 3,
            SpreadType::LoveRelationship => 5,
            SpreadType::CareerGuidance => 6,
            SpreadType::DecisionMaking => 7,
            SpreadType::CelticCross => 10,
            SpreadType::YearForecast => 12,
        }
    }

    /// 从牌数推断牌阵类型（默认）
    pub fn from_count(count: u8) -> Self {
        match count {
            1 => SpreadType::SingleCard,
            3 => SpreadType::ThreeCardTime,
            5 => SpreadType::LoveRelationship,
            6 => SpreadType::CareerGuidance,
            7 => SpreadType::DecisionMaking,
            10 => SpreadType::CelticCross,
            12 => SpreadType::YearForecast,
            _ => SpreadType::SingleCard,
        }
    }

    /// 获取牌阵各位置的含义名称
    ///
    /// 返回每个位置对应的解读主题，供前端展示和 AI 解读使用
    pub fn position_names(&self) -> &'static [&'static str] {
        match self {
            SpreadType::SingleCard => &["当前指引"],
            SpreadType::ThreeCardTime => &["过去", "现在", "未来"],
            SpreadType::ThreeCardSituation => &["情况", "行动", "结果"],
            SpreadType::LoveRelationship => &[
                "你的感受",
                "对方的感受",
                "关系现状",
                "挑战",
                "未来发展",
            ],
            SpreadType::CareerGuidance => &[
                "当前状况",
                "优势",
                "挑战",
                "机会",
                "建议行动",
                "未来前景",
            ],
            SpreadType::DecisionMaking => &[
                "当前情况",
                "选择A",
                "选择A结果",
                "选择B",
                "选择B结果",
                "外在影响",
                "最佳建议",
            ],
            SpreadType::CelticCross => &[
                "当前状况",
                "挑战",
                "远因",
                "近因",
                "可能结果",
                "近期发展",
                "你的态度",
                "外在影响",
                "内心期望",
                "最终结果",
            ],
            SpreadType::YearForecast => &[
                "一月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月",
                "十一月", "十二月",
            ],
        }
    }

    /// 获取牌阵的中文名称
    pub fn name(&self) -> &'static str {
        match self {
            SpreadType::SingleCard => "单张牌指引",
            SpreadType::ThreeCardTime => "时间三张牌",
            SpreadType::ThreeCardSituation => "情况三张牌",
            SpreadType::LoveRelationship => "爱情关系牌阵",
            SpreadType::CareerGuidance => "事业指导牌阵",
            SpreadType::DecisionMaking => "决策分析牌阵",
            SpreadType::CelticCross => "凯尔特十字",
            SpreadType::YearForecast => "年度运势",
        }
    }

    /// 获取牌阵的描述说明
    pub fn description(&self) -> &'static str {
        match self {
            SpreadType::SingleCard => "快速获得当下指导，适合日常决策和简单问题",
            SpreadType::ThreeCardTime => "了解过去、现在、未来的发展趋势",
            SpreadType::ThreeCardSituation => "分析问题的情况、行动和结果",
            SpreadType::LoveRelationship => "深入了解感情状况和发展方向",
            SpreadType::CareerGuidance => "全面分析职业发展和工作状况",
            SpreadType::DecisionMaking => "帮助做出重要决定，分析多个选择",
            SpreadType::CelticCross => "最全面的牌阵，深度分析复杂问题",
            SpreadType::YearForecast => "预测一年中每个月的运势发展",
        }
    }

    /// 获取指定位置的含义名称
    ///
    /// # 参数
    /// - `index`: 位置索引（0-based）
    ///
    /// # 返回
    /// - 位置名称，如果索引超出范围返回 None
    pub fn get_position_name(&self, index: usize) -> Option<&'static str> {
        self.position_names().get(index).copied()
    }

    /// 获取牌阵类型的数字标识（用于 constants 模块查询）
    pub fn type_id(&self) -> u8 {
        match self {
            SpreadType::SingleCard => 1,
            SpreadType::ThreeCardTime => 3,
            SpreadType::ThreeCardSituation => 4,
            SpreadType::LoveRelationship => 5,
            SpreadType::CareerGuidance => 6,
            SpreadType::DecisionMaking => 7,
            SpreadType::CelticCross => 10,
            SpreadType::YearForecast => 12,
        }
    }

    /// 获取指定位置的详细信息
    ///
    /// # 参数
    /// - `index`: 位置索引（0-based）
    ///
    /// # 返回
    /// - 位置详情，包含名称、描述、解读指导
    pub fn get_position_info(&self, index: usize) -> Option<&'static crate::constants::SpreadPositionInfo> {
        crate::constants::get_spread_position_info(self.type_id(), index)
    }

    /// 获取所有位置的详细信息
    ///
    /// # 返回
    /// - 所有位置的详情数组
    pub fn get_all_position_info(&self) -> Option<&'static [crate::constants::SpreadPositionInfo]> {
        crate::constants::get_spread_all_positions(self.type_id())
    }
}

/// 抽取的牌（含位置信息）
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct DrawnCard {
    /// 塔罗牌
    pub card: TarotCard,
    /// 正逆位
    pub position: CardPosition,
    /// 在牌阵中的位置索引 (0-based)
    pub spread_position: u8,
}

impl DrawnCard {
    /// 创建新的抽取牌
    pub fn new(card_id: u8, reversed: bool, spread_position: u8) -> Self {
        Self {
            card: TarotCard::from_id(card_id),
            position: CardPosition::from_bool(reversed),
            spread_position,
        }
    }

    /// 根据正逆位获取牌义
    ///
    /// # 返回
    /// - 正位返回正位含义，逆位返回逆位含义
    pub fn meaning(&self) -> &'static str {
        if self.position.is_reversed() {
            self.card.reversed_meaning()
        } else {
            self.card.upright_meaning()
        }
    }

    /// 获取牌的关键词
    pub fn keywords(&self) -> &'static str {
        self.card.keywords()
    }

    /// 获取牌的完整名称（含正逆位标识）
    ///
    /// # 返回
    /// - 如 "愚者（正位）" 或 "魔术师（逆位）"
    pub fn full_name_with_position(&self) -> (&'static str, &'static str) {
        let name = self.card.full_name();
        let position_str = if self.position.is_reversed() {
            "逆位"
        } else {
            "正位"
        };
        (name, position_str)
    }

    /// 获取牌的显示名称
    pub fn display_name(&self) -> (&'static str, Option<&'static str>) {
        self.card.display_name()
    }

    /// 获取牌的元素属性
    pub fn element(&self) -> &'static str {
        self.card.element()
    }

    /// 判断是否为大阿卡纳
    pub fn is_major(&self) -> bool {
        self.card.is_major()
    }

    /// 判断是否为宫廷牌
    pub fn is_court_card(&self) -> bool {
        self.card.is_court_card()
    }

    /// 获取大阿卡纳的星座/行星对应
    pub fn astrology(&self) -> Option<(&'static str, &'static str)> {
        self.card.astrology()
    }
}

/// 占卜方式枚举
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum DivinationMethod {
    /// 随机抽牌 - 使用链上随机数
    #[default]
    Random = 0,
    /// 时间起卦 - 基于时间戳生成
    ByTime = 1,
    /// 数字起卦 - 基于用户提供的数字
    ByNumbers = 2,
    /// 手动指定 - 直接指定牌面
    Manual = 3,
    /// 带切牌的随机抽牌 - 模拟真实塔罗占卜仪式
    RandomWithCut = 4,
}

/// 完整的塔罗牌占卜记录
///
/// 存储一次完整占卜的所有信息
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxCards))]
pub struct TarotReading<AccountId, BlockNumber, MaxCards: Get<u32>> {
    /// 占卜记录唯一ID
    pub id: u64,
    /// 占卜者账户
    pub diviner: AccountId,
    /// 牌阵类型
    pub spread_type: SpreadType,
    /// 占卜方式
    pub method: DivinationMethod,
    /// 抽取的牌列表
    pub cards: BoundedVec<DrawnCard, MaxCards>,
    /// 占卜问题的哈希值（隐私保护）
    pub question_hash: [u8; 32],
    /// 占卜时的区块号
    pub block_number: BlockNumber,
    /// 占卜时间戳（Unix秒）
    pub timestamp: u64,
    /// AI 解读的 IPFS CID（可选）
    pub interpretation_cid: Option<BoundedVec<u8, ConstU32<64>>>,
    /// 隐私模式
    /// - Public: 公开，所有人可见
    /// - Private: 私密，仅所有者可见
    /// - Authorized: 授权访问，被授权者可见
    pub privacy_mode: PrivacyMode,
}

/// 牌阵位置含义（用于前端展示）
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxLen))]
pub struct SpreadPosition<MaxLen: Get<u32>> {
    /// 位置索引
    pub index: u8,
    /// 位置名称（如"过去"、"现在"、"未来"）
    pub name: BoundedVec<u8, MaxLen>,
}

/// 占卜统计信息
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct DivinationStats {
    /// 总占卜次数
    pub total_readings: u64,
    /// 大阿卡纳出现次数
    pub major_arcana_count: u64,
    /// 逆位出现次数
    pub reversed_count: u64,
    /// 最常出现的牌ID
    pub most_frequent_card: u8,
    /// 最常出现的牌次数
    pub most_frequent_count: u32,
}

// ============================================================================
// 隐私数据结构
// ============================================================================

/// 加密隐私数据参数
///
/// 用于 `divine_with_privacy` 函数的原子性隐私数据存储。
/// 前端负责加密数据，链上只存储加密后的数据。
///
/// ## 加密方案
///
/// ```text
/// 加密流程：
/// ┌──────────────┐    ┌─────────────────┐    ┌────────────────┐
/// │ DivinerPriv- │───>│ JSON.stringify  │───>│ AES-256-GCM    │───> encrypted_data
/// │ ateData      │    │                 │    │ (DataKey加密)   │
/// └──────────────┘    └─────────────────┘    └────────────────┘
///
/// 密钥分发：
/// ┌──────────┐    ┌─────────────────────┐    ┌─────────────────┐
/// │ DataKey  │───>│ X25519 封装         │───>│ encrypted_key   │
/// │ (随机)   │    │ (用接收者公钥加密)   │    │ (存入授权条目)   │
/// └──────────┘    └─────────────────────┘    └─────────────────┘
/// ```
///
/// ## 隐私数据内容（前端加密前的明文结构）
///
/// ```text
/// {
///   "question": "我的感情运势如何？",  // 占卜问题
///   "notes": "备注信息"                // 备注
/// }
/// ```
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, PartialEq, Eq, Debug)]
pub struct EncryptedPrivacyData {
    /// 隐私模式
    /// - Public: 公开，所有人可见
    /// - Private: 私密，仅所有者可见
    /// - Authorized: 授权访问，被授权者可见
    pub privacy_mode: PrivacyMode,

    /// 加密的敏感数据（AES-256-GCM 加密后的密文）
    ///
    /// 前端使用随机生成的 DataKey 加密原始数据，
    /// DataKey 再用接收者公钥加密后存储在 owner_encrypted_key 中。
    pub encrypted_data: Vec<u8>,

    /// 加密随机数（24 字节）
    ///
    /// AES-256-GCM 加密使用的 nonce，每次加密必须唯一。
    /// 24 字节 = 192 位，足够安全。
    pub nonce: [u8; 24],

    /// 认证标签（16 字节）
    ///
    /// AES-GCM 的认证标签，用于验证密文完整性和真实性。
    /// 解密时会验证此标签，防止篡改。
    pub auth_tag: [u8; 16],

    /// 数据哈希（32 字节）
    ///
    /// 原始明文数据的 Blake2-256 哈希。
    /// 用于解密后验证数据完整性。
    pub data_hash: [u8; 32],

    /// 所有者的加密数据密钥
    ///
    /// DataKey 经过 X25519 密钥封装后的密文。
    /// 格式：[临时公钥(32字节) | 加密的DataKey(32字节)]
    ///
    /// 解密流程：
    /// 1. 提取临时公钥（前32字节）
    /// 2. 使用自己的私钥和临时公钥进行 ECDH
    /// 3. 用共享密钥解密 DataKey
    /// 4. 用 DataKey 解密 encrypted_data
    pub owner_encrypted_key: Vec<u8>,
}

impl EncryptedPrivacyData {
    /// 创建新的加密隐私数据
    ///
    /// # 参数
    /// - `privacy_mode`: 隐私模式
    /// - `encrypted_data`: 加密后的数据
    /// - `nonce`: 24字节加密随机数
    /// - `auth_tag`: 16字节认证标签
    /// - `data_hash`: 32字节数据哈希
    /// - `owner_encrypted_key`: 所有者的加密密钥
    pub fn new(
        privacy_mode: PrivacyMode,
        encrypted_data: Vec<u8>,
        nonce: [u8; 24],
        auth_tag: [u8; 16],
        data_hash: [u8; 32],
        owner_encrypted_key: Vec<u8>,
    ) -> Self {
        Self {
            privacy_mode,
            encrypted_data,
            nonce,
            auth_tag,
            data_hash,
            owner_encrypted_key,
        }
    }

    /// 检查加密数据是否为空
    pub fn is_empty(&self) -> bool {
        self.encrypted_data.is_empty()
    }

    /// 获取加密数据长度
    pub fn encrypted_data_len(&self) -> usize {
        self.encrypted_data.len()
    }

    /// 获取加密密钥长度
    pub fn encrypted_key_len(&self) -> usize {
        self.owner_encrypted_key.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tarot_card_from_id() {
        // 测试大阿卡纳
        let fool = TarotCard::from_id(0);
        assert!(fool.is_major());
        assert_eq!(fool.number, 0);
        assert_eq!(fool.suit, Suit::None);

        let world = TarotCard::from_id(21);
        assert!(world.is_major());
        assert_eq!(world.number, 21);

        // 测试小阿卡纳 - 权杖Ace
        let wands_ace = TarotCard::from_id(22);
        assert!(!wands_ace.is_major());
        assert_eq!(wands_ace.suit, Suit::Wands);
        assert_eq!(wands_ace.number, 1);

        // 测试小阿卡纳 - 权杖国王
        let wands_king = TarotCard::from_id(35);
        assert!(wands_king.is_court_card());
        assert_eq!(wands_king.court_rank(), CourtRank::King);

        // 测试小阿卡纳 - 圣杯Ace
        let cups_ace = TarotCard::from_id(36);
        assert_eq!(cups_ace.suit, Suit::Cups);
        assert_eq!(cups_ace.number, 1);

        // 测试小阿卡纳 - 星币国王（最后一张）
        let pentacles_king = TarotCard::from_id(77);
        assert_eq!(pentacles_king.suit, Suit::Pentacles);
        assert_eq!(pentacles_king.number, 14);
    }

    #[test]
    fn test_spread_type_card_count() {
        assert_eq!(SpreadType::SingleCard.card_count(), 1);
        assert_eq!(SpreadType::ThreeCardTime.card_count(), 3);
        assert_eq!(SpreadType::CelticCross.card_count(), 10);
        assert_eq!(SpreadType::YearForecast.card_count(), 12);
    }

    #[test]
    fn test_card_position() {
        assert!(!CardPosition::Upright.is_reversed());
        assert!(CardPosition::Reversed.is_reversed());
        assert_eq!(CardPosition::from_bool(true), CardPosition::Reversed);
        assert_eq!(CardPosition::from_bool(false), CardPosition::Upright);
    }

    // ========================================================================
    // 牌义方法测试
    // ========================================================================

    #[test]
    fn test_tarot_card_meaning_methods() {
        // 测试大阿卡纳 - 愚者
        let fool = TarotCard::from_id(0);
        assert!(fool.upright_meaning().contains("新的开始"));
        assert!(fool.reversed_meaning().contains("鲁莽"));
        assert!(fool.keywords().contains("自由"));
        assert!(fool.description().is_some());
        assert!(fool.description().unwrap().contains("悬崖"));

        // 测试星座对应
        let astrology = fool.astrology();
        assert!(astrology.is_some());
        assert_eq!(astrology.unwrap().0, "天王星");

        // 测试数字象征
        let numerology = fool.numerology();
        assert!(numerology.is_some());
        assert!(numerology.unwrap().contains("0"));
    }

    #[test]
    fn test_tarot_card_names() {
        // 测试大阿卡纳名称
        let magician = TarotCard::from_id(1);
        assert_eq!(magician.full_name(), "魔术师");
        assert_eq!(magician.english_name(), "The Magician");

        let (name, sub_name) = magician.display_name();
        assert_eq!(name, "魔术师");
        assert!(sub_name.is_none());

        // 测试小阿卡纳名称
        let wands_ace = TarotCard::from_id(22);
        let (suit_name, card_name) = wands_ace.display_name();
        assert_eq!(suit_name, "权杖");
        assert_eq!(card_name, Some("Ace"));
    }

    #[test]
    fn test_tarot_card_element() {
        // 大阿卡纳无元素
        let fool = TarotCard::from_id(0);
        assert_eq!(fool.element(), "");

        // 权杖 - 火
        let wands = TarotCard::from_id(22);
        assert_eq!(wands.element(), "火");

        // 圣杯 - 水
        let cups = TarotCard::from_id(36);
        assert_eq!(cups.element(), "水");

        // 宝剑 - 风
        let swords = TarotCard::from_id(50);
        assert_eq!(swords.element(), "风");

        // 星币 - 土
        let pentacles = TarotCard::from_id(64);
        assert_eq!(pentacles.element(), "土");
    }

    #[test]
    fn test_minor_arcana_meanings() {
        // 测试权杖 Ace
        let wands_ace = TarotCard::from_id(22);
        assert!(wands_ace.upright_meaning().contains("创意"));
        assert!(wands_ace.suit_description().is_some());
        assert!(wands_ace.suit_description().unwrap().contains("火元素"));

        // 测试数字象征
        let wands_two = TarotCard::from_id(23);
        let symbolism = wands_two.number_symbolism();
        assert!(symbolism.is_some());
        assert!(symbolism.unwrap().contains("2"));
    }

    #[test]
    fn test_court_card_meanings() {
        // 测试权杖国王
        let wands_king = TarotCard::from_id(35);
        assert!(wands_king.is_court_card());
        assert!(wands_king.upright_meaning().contains("领袖"));
        assert!(wands_king.number_symbolism().is_none()); // 宫廷牌无数字象征
    }

    #[test]
    fn test_drawn_card_meaning() {
        // 正位牌
        let upright_card = DrawnCard::new(0, false, 0);
        assert!(upright_card.meaning().contains("新的开始"));
        assert!(!upright_card.position.is_reversed());

        // 逆位牌
        let reversed_card = DrawnCard::new(0, true, 0);
        assert!(reversed_card.meaning().contains("鲁莽"));
        assert!(reversed_card.position.is_reversed());

        // 名称带正逆位
        let (name, pos) = upright_card.full_name_with_position();
        assert_eq!(name, "愚者");
        assert_eq!(pos, "正位");

        let (name, pos) = reversed_card.full_name_with_position();
        assert_eq!(name, "愚者");
        assert_eq!(pos, "逆位");
    }

    #[test]
    fn test_get_card_meaning() {
        // 测试获取完整牌义
        let fool = TarotCard::from_id(0);
        let meaning = fool.get_meaning();
        assert!(meaning.is_some());

        let m = meaning.unwrap();
        assert_eq!(m.name, "愚者");
        assert_eq!(m.name_en, "The Fool");
        assert!(m.description.is_some());
        assert!(m.astrology.is_some());

        // 测试小阿卡纳
        let wands_ace = TarotCard::from_id(22);
        let meaning = wands_ace.get_meaning();
        assert!(meaning.is_some());

        let m = meaning.unwrap();
        assert_eq!(m.element, "火");
        assert!(m.description.is_none()); // 小阿卡纳无描述
    }
}
