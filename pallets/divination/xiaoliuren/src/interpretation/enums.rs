use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

// ============================================================================
// 吉凶等级枚举
// ============================================================================

/// 吉凶等级（1 byte）
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum JiXiongLevel {
    /// 大吉 - 诸事顺遂，心想事成
    #[default]
    DaJi = 0,
    /// 吉 - 事可成，宜进取
    Ji = 1,
    /// 小吉 - 小有所得，不宜大动
    XiaoJi = 2,
    /// 平 - 平稳无波，守成为上
    Ping = 3,
    /// 小凶 - 小有阻碍，谨慎行事
    XiaoXiong = 4,
    /// 凶 - 事难成，宜退守
    Xiong = 5,
    /// 大凶 - 诸事不利，静待时机
    DaXiong = 6,
}

impl JiXiongLevel {
    /// 获取吉凶等级名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::DaJi => "大吉",
            Self::Ji => "吉",
            Self::XiaoJi => "小吉",
            Self::Ping => "平",
            Self::XiaoXiong => "小凶",
            Self::Xiong => "凶",
            Self::DaXiong => "大凶",
        }
    }

    /// 获取详细描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::DaJi => "诸事顺遂，心想事成",
            Self::Ji => "事可成，宜进取",
            Self::XiaoJi => "小有所得，不宜大动",
            Self::Ping => "平稳无波，守成为上",
            Self::XiaoXiong => "小有阻碍，谨慎行事",
            Self::Xiong => "事难成，宜退守",
            Self::DaXiong => "诸事不利，静待时机",
        }
    }

    /// 获取数值分数（1-7）
    pub fn score(&self) -> u8 {
        7 - (*self as u8)
    }

    /// 判断是否为吉
    pub fn is_ji(&self) -> bool {
        matches!(self, Self::DaJi | Self::Ji | Self::XiaoJi)
    }

    /// 判断是否为凶
    pub fn is_xiong(&self) -> bool {
        matches!(self, Self::XiaoXiong | Self::Xiong | Self::DaXiong)
    }
}

// ============================================================================
// 建议类型枚举
// ============================================================================

/// 建议类型（1 byte）
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum AdviceType {
    /// 大胆进取 - 大吉时
    #[default]
    JinQu = 0,
    /// 稳步前进 - 吉时
    WenBu = 1,
    /// 守成为主 - 平时
    ShouCheng = 2,
    /// 谨慎观望 - 小凶时
    GuanWang = 3,
    /// 退守待时 - 凶时
    TuiShou = 4,
    /// 静待时机 - 大凶时
    JingDai = 5,
    /// 寻求帮助 - 特殊情况
    XunQiu = 6,
    /// 化解冲克 - 五行不利
    HuaJie = 7,
}

impl AdviceType {
    /// 获取建议类型名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::JinQu => "大胆进取",
            Self::WenBu => "稳步前进",
            Self::ShouCheng => "守成为主",
            Self::GuanWang => "谨慎观望",
            Self::TuiShou => "退守待时",
            Self::JingDai => "静待时机",
            Self::XunQiu => "寻求帮助",
            Self::HuaJie => "化解冲克",
        }
    }

    /// 获取详细建议内容
    pub fn advice(&self) -> &'static str {
        match self {
            Self::JinQu => "时机极佳，诸事皆宜。可大胆行事，积极进取，贵人相助，心想事成。",
            Self::WenBu => "事情顺利，稍加努力即可成功。宜稳步前进，把握机会，不要急于求成。",
            Self::ShouCheng => "平稳无大碍，宜守不宜进。保持现状，巩固基础，等待更好时机。",
            Self::GuanWang => "事多波折，需耐心等待。谨慎观望，不宜冒进，静待时机成熟。",
            Self::TuiShou => "凶险当道，宜退守。避免大事，保持低调，等待时机转变。",
            Self::JingDai => "诸事不利，静待时机。避免冲动，修身养性，积蓄力量待时而动。",
            Self::XunQiu => "独力难支，需要帮助。寻求贵人相助，借力打力，方能化险为夷。",
            Self::HuaJie => "五行冲克，需要化解。调整方位、时间或方式，化解不利因素。",
        }
    }
}

// ============================================================================
// 应期类型枚举
// ============================================================================

/// 应期类型（1 byte）
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum YingQiType {
    /// 即刻应验 - 速喜
    #[default]
    JiKe = 0,
    /// 当日应验 - 大安、小吉
    DangRi = 1,
    /// 数日应验 - 3-7天
    ShuRi = 2,
    /// 延迟应验 - 留连，10天以上
    YanChi = 3,
    /// 难以应验 - 空亡
    NanYi = 4,
    /// 需要化解 - 赤口
    XuHuaJie = 5,
}

impl YingQiType {
    /// 获取应期类型名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::JiKe => "即刻应验",
            Self::DangRi => "当日应验",
            Self::ShuRi => "数日应验",
            Self::YanChi => "延迟应验",
            Self::NanYi => "难以应验",
            Self::XuHuaJie => "需要化解",
        }
    }

    /// 获取应期描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::JiKe => "事情发展迅速，立见分晓，当下即可知晓结果",
            Self::DangRi => "当日之内便有消息，无需久等，顺利进展",
            Self::ShuRi => "数日之内（3-7天）会有结果，稍安勿躁",
            Self::YanChi => "事情进展缓慢，需要10天以上才能见分晓，耐心等待",
            Self::NanYi => "所求之事虚而不实，难以应验，建议另作他图",
            Self::XuHuaJie => "有口舌阻碍，需要化解不利因素后方能应验",
        }
    }

    /// 获取时间范围（天数）
    pub fn days_range(&self) -> (u8, u8) {
        match self {
            Self::JiKe => (0, 0),      // 立即
            Self::DangRi => (0, 1),    // 当日
            Self::ShuRi => (3, 7),     // 3-7天
            Self::YanChi => (10, 30),  // 10-30天
            Self::NanYi => (0, 0),     // 不确定
            Self::XuHuaJie => (0, 0),  // 需化解
        }
    }
}

// ============================================================================
// 特殊格局（位标志）
// ============================================================================

/// 特殊格局（1 byte，使用位标志）
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct SpecialPattern(pub u8);

impl SpecialPattern {
    /// 无特殊格局
    pub const NONE: u8 = 0b0000_0000;
    /// 纯宫（三宫相同）
    pub const PURE: u8 = 0b0000_0001;
    /// 全吉（三宫皆吉）
    pub const ALL_AUSPICIOUS: u8 = 0b0000_0010;
    /// 全凶（三宫皆凶）
    pub const ALL_INAUSPICIOUS: u8 = 0b0000_0100;
    /// 五行相生成环
    pub const SHENG_CYCLE: u8 = 0b0000_1000;
    /// 五行相克成环
    pub const KE_CYCLE: u8 = 0b0001_0000;
    /// 阴阳和合（体用阴阳互补）
    pub const YIN_YANG_HARMONY: u8 = 0b0010_0000;
    /// 特殊时辰（子午卯酉）
    pub const SPECIAL_TIME: u8 = 0b0100_0000;
    /// 预留
    pub const RESERVED: u8 = 0b1000_0000;

    /// 创建空格局
    pub fn new() -> Self {
        Self(Self::NONE)
    }

    /// 检查是否为纯宫
    pub fn is_pure(&self) -> bool {
        self.0 & Self::PURE != 0
    }

    /// 检查是否全吉
    pub fn is_all_auspicious(&self) -> bool {
        self.0 & Self::ALL_AUSPICIOUS != 0
    }

    /// 检查是否全凶
    pub fn is_all_inauspicious(&self) -> bool {
        self.0 & Self::ALL_INAUSPICIOUS != 0
    }

    /// 检查是否有相生成环
    pub fn is_sheng_cycle(&self) -> bool {
        self.0 & Self::SHENG_CYCLE != 0
    }

    /// 检查是否有相克成环
    pub fn is_ke_cycle(&self) -> bool {
        self.0 & Self::KE_CYCLE != 0
    }

    /// 检查是否阴阳和合
    pub fn is_yin_yang_harmony(&self) -> bool {
        self.0 & Self::YIN_YANG_HARMONY != 0
    }

    /// 检查是否特殊时辰
    pub fn is_special_time(&self) -> bool {
        self.0 & Self::SPECIAL_TIME != 0
    }

    /// 设置纯宫
    pub fn set_pure(&mut self) {
        self.0 |= Self::PURE;
    }

    /// 设置全吉
    pub fn set_all_auspicious(&mut self) {
        self.0 |= Self::ALL_AUSPICIOUS;
    }

    /// 设置全凶
    pub fn set_all_inauspicious(&mut self) {
        self.0 |= Self::ALL_INAUSPICIOUS;
    }

    /// 设置相生成环
    pub fn set_sheng_cycle(&mut self) {
        self.0 |= Self::SHENG_CYCLE;
    }

    /// 设置相克成环
    pub fn set_ke_cycle(&mut self) {
        self.0 |= Self::KE_CYCLE;
    }

    /// 设置阴阳和合
    pub fn set_yin_yang_harmony(&mut self) {
        self.0 |= Self::YIN_YANG_HARMONY;
    }

    /// 设置特殊时辰
    pub fn set_special_time(&mut self) {
        self.0 |= Self::SPECIAL_TIME;
    }

    /// 获取所有激活的格局列表
    pub fn get_patterns(&self) -> Vec<&'static str> {
        let mut patterns = Vec::new();
        if self.is_pure() { patterns.push("纯宫"); }
        if self.is_all_auspicious() { patterns.push("全吉"); }
        if self.is_all_inauspicious() { patterns.push("全凶"); }
        if self.is_sheng_cycle() { patterns.push("五行相生成环"); }
        if self.is_ke_cycle() { patterns.push("五行相克成环"); }
        if self.is_yin_yang_harmony() { patterns.push("阴阳和合"); }
        if self.is_special_time() { patterns.push("特殊时辰"); }
        patterns
    }

    /// 判断是否有任何特殊格局
    pub fn has_any(&self) -> bool {
        self.0 != Self::NONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ji_xiong_level() {
        assert_eq!(JiXiongLevel::DaJi.name(), "大吉");
        assert!(JiXiongLevel::DaJi.is_ji());
        assert!(!JiXiongLevel::DaJi.is_xiong());
        assert_eq!(JiXiongLevel::DaXiong.score(), 1);
    }

    #[test]
    fn test_advice_type() {
        assert_eq!(AdviceType::JinQu.name(), "大胆进取");
        assert!(!AdviceType::JinQu.advice().is_empty());
    }

    #[test]
    fn test_ying_qi_type() {
        assert_eq!(YingQiType::JiKe.name(), "即刻应验");
        assert_eq!(YingQiType::JiKe.days_range(), (0, 0));
        assert_eq!(YingQiType::ShuRi.days_range(), (3, 7));
    }

    #[test]
    fn test_special_pattern() {
        let mut pattern = SpecialPattern::new();
        assert!(!pattern.has_any());

        pattern.set_pure();
        assert!(pattern.is_pure());
        assert!(pattern.has_any());

        pattern.set_all_auspicious();
        assert!(pattern.is_all_auspicious());

        let patterns = pattern.get_patterns();
        assert!(patterns.len() >= 2);
    }

    #[test]
    fn test_special_pattern_bits() {
        let mut pattern = SpecialPattern::new();
        pattern.set_pure();
        pattern.set_sheng_cycle();
        pattern.set_yin_yang_harmony();

        assert!(pattern.is_pure());
        assert!(pattern.is_sheng_cycle());
        assert!(pattern.is_yin_yang_harmony());
        assert!(!pattern.is_all_auspicious());
    }
}
