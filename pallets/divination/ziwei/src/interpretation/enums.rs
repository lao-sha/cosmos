//! # 紫微斗数解卦枚举类型
//!
//! 本模块定义解卦系统所需的所有枚举类型

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

// ============================================================================
// 吉凶等级枚举
// ============================================================================

/// 吉凶等级（1 byte）
///
/// 用于评估宫位、格局、整体命盘的吉凶程度
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum FortuneLevel {
    /// 大吉 - 诸事顺遂，心想事成
    DaJi = 0,
    /// 吉 - 事可成，宜进取
    Ji = 1,
    /// 小吉 - 小有所得，不宜大动
    XiaoJi = 2,
    /// 平 - 平稳无波，守成为上
    #[default]
    Ping = 3,
    /// 小凶 - 小有阻碍，谨慎行事
    XiaoXiong = 4,
    /// 凶 - 事难成，宜退守
    Xiong = 5,
    /// 大凶 - 诸事不利，静待时机
    DaXiong = 6,
}

impl FortuneLevel {
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
            Self::DaJi => "诸事顺遂，心想事成，贵人相助，大展宏图",
            Self::Ji => "事可成，宜进取，把握机会，积极行动",
            Self::XiaoJi => "小有所得，不宜大动，稳步前进，守成为主",
            Self::Ping => "平稳无波，守成为上，保持现状，等待时机",
            Self::XiaoXiong => "小有阻碍，谨慎行事，避免冒进，化解不利",
            Self::Xiong => "事难成，宜退守，避免大事，低调行事",
            Self::DaXiong => "诸事不利，静待时机，修身养性，积蓄力量",
        }
    }

    /// 获取数值分数（1-7）
    pub fn score(&self) -> u8 {
        7 - (*self as u8)
    }

    /// 判断是否为吉
    pub fn is_auspicious(&self) -> bool {
        matches!(self, Self::DaJi | Self::Ji | Self::XiaoJi)
    }

    /// 判断是否为凶
    pub fn is_inauspicious(&self) -> bool {
        matches!(self, Self::XiaoXiong | Self::Xiong | Self::DaXiong)
    }

    /// 从评分转换为吉凶等级
    ///
    /// # 参数
    /// - score: 评分（0-100）
    ///
    /// # 返回
    /// 对应的吉凶等级
    pub fn from_score(score: u8) -> Self {
        match score {
            90..=100 => Self::DaJi,
            75..=89 => Self::Ji,
            60..=74 => Self::XiaoJi,
            40..=59 => Self::Ping,
            25..=39 => Self::XiaoXiong,
            10..=24 => Self::Xiong,
            _ => Self::DaXiong,
        }
    }
}

// ============================================================================
// 命格等级枚举
// ============================================================================

/// 命格等级（1 byte）
///
/// 根据命盘整体配置判断命格高低
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum MingGeLevel {
    /// 普通格局
    #[default]
    Putong = 0,
    /// 小贵格局
    XiaoGui = 1,
    /// 中贵格局
    ZhongGui = 2,
    /// 大贵格局
    DaGui = 3,
    /// 极贵格局
    JiGui = 4,
    /// 帝王格局
    DiWang = 5,
}

impl MingGeLevel {
    /// 获取命格等级名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Putong => "普通格局",
            Self::XiaoGui => "小贵格局",
            Self::ZhongGui => "中贵格局",
            Self::DaGui => "大贵格局",
            Self::JiGui => "极贵格局",
            Self::DiWang => "帝王格局",
        }
    }

    /// 获取详细描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Putong => "平凡之命，需自力更生，勤奋努力方能有成",
            Self::XiaoGui => "小有贵气，衣食无忧，中产阶层，生活安稳",
            Self::ZhongGui => "中等富贵，事业有成，名利双收，社会地位较高",
            Self::DaGui => "大富大贵，权势显赫，功成名就，位高权重",
            Self::JiGui => "极品贵格，富可敌国，权倾朝野，名垂青史",
            Self::DiWang => "帝王之格，天命所归，统御天下，万世流芳",
        }
    }
}

// ============================================================================
// 格局类型枚举
// ============================================================================

/// 格局类型（1 byte）
///
/// 紫微斗数中的各种格局，包括吉格和凶格
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum PatternType {
    // ===== 富贵格局 (0-13) =====
    /// 紫府同宫 - 紫微天府同坐命宫
    #[default]
    ZiFuTongGong = 0,
    /// 紫府朝垣 - 紫微天府相朝
    ZiFuChaoYuan = 1,
    /// 天府朝垣 - 天府守命逢禄
    TianFuChaoYuan = 2,
    /// 君臣庆会 - 紫微为君，天相天府为臣
    JunChenQingHui = 3,
    /// 府相朝垣 - 天府天相守命
    FuXiangChaoYuan = 4,
    /// 机月同梁 - 天机太阴天同天梁会合
    JiYueTongLiang = 5,
    /// 日月并明 - 太阳太阴在卯酉宫
    RiYueBingMing = 6,
    /// 日照雷门 - 太阳在卯宫
    RiZhaoLeiMen = 7,
    /// 月朗天门 - 太阴在亥宫
    YueLangTianMen = 8,
    /// 明珠出海 - 太阴在酉宫
    MingZhuChuHai = 9,
    /// 阳梁昌禄 - 太阳天梁会昌禄
    YangLiangChangLu = 10,
    /// 贪武同行 - 贪狼武曲在丑未宫
    TanWuTongXing = 11,
    /// 火贪格 - 火星贪狼同宫
    HuoTanGeJu = 12,
    /// 铃贪格 - 铃星贪狼同宫
    LingTanGeJu = 13,

    // ===== 权贵格局 (14-21) =====
    /// 三奇嘉会 - 禄权科三化会合
    SanQiJiaHui = 14,
    /// 双禄夹命 - 禄存化禄夹命宫
    ShuangLuJiaMing = 15,
    /// 双禄夹财 - 禄存化禄夹财帛宫
    ShuangLuJiaCai = 16,
    /// 科权禄夹 - 化科化权化禄夹宫
    KeQuanLuJia = 17,
    /// 左右夹命 - 左辅右弼夹命宫
    ZuoYouJiaMing = 18,
    /// 昌曲夹命 - 文昌文曲夹命宫
    ChangQuJiaMing = 19,
    /// 魁钺夹命 - 天魁天钺夹命宫
    KuiYueJiaMing = 20,
    /// 禄马交驰 - 禄存天马同宫或会照
    LuMaJiaoChiGeJu = 21,

    // ===== 凶格 (22-31) =====
    /// 铃昌陀武 - 铃星文昌陀罗武曲同宫
    LingChangTuoWu = 22,
    /// 巨机同宫 - 巨门天机在辰戌宫
    JiJiTongGong = 23,
    /// 巨日同宫 - 巨门太阳同宫（最忌）
    JuRiTongGong = 24,
    /// 命无正曜 - 命宫无主星（空宫）
    MingWuZhengYao = 25,
    /// 马头带箭 - 午宫擎羊守命
    MaTouDaiJian = 26,
    /// 羊陀夹命 - 擎羊陀罗夹命宫
    YangTuoJiaMing = 27,
    /// 火铃夹命 - 火星铃星夹命宫
    HuoLingJiaMing = 28,
    /// 空劫夹命 - 地空地劫夹命宫
    KongJieJiaMing = 29,
    /// 羊陀夹忌 - 擎羊陀罗夹化忌
    YangTuoJiaJi = 30,
    /// 四煞冲命 - 擎羊陀罗火星铃星冲命
    SiShaChongMing = 31,
}

impl PatternType {
    /// 获取格局名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::ZiFuTongGong => "紫府同宫",
            Self::ZiFuChaoYuan => "紫府朝垣",
            Self::TianFuChaoYuan => "天府朝垣",
            Self::JunChenQingHui => "君臣庆会",
            Self::FuXiangChaoYuan => "府相朝垣",
            Self::JiYueTongLiang => "机月同梁",
            Self::RiYueBingMing => "日月并明",
            Self::RiZhaoLeiMen => "日照雷门",
            Self::YueLangTianMen => "月朗天门",
            Self::MingZhuChuHai => "明珠出海",
            Self::YangLiangChangLu => "阳梁昌禄",
            Self::TanWuTongXing => "贪武同行",
            Self::HuoTanGeJu => "火贪格",
            Self::LingTanGeJu => "铃贪格",
            Self::SanQiJiaHui => "三奇嘉会",
            Self::ShuangLuJiaMing => "双禄夹命",
            Self::ShuangLuJiaCai => "双禄夹财",
            Self::KeQuanLuJia => "科权禄夹",
            Self::ZuoYouJiaMing => "左右夹命",
            Self::ChangQuJiaMing => "昌曲夹命",
            Self::KuiYueJiaMing => "魁钺夹命",
            Self::LuMaJiaoChiGeJu => "禄马交驰",
            Self::LingChangTuoWu => "铃昌陀武",
            Self::JiJiTongGong => "巨机同宫",
            Self::JuRiTongGong => "巨日同宫",
            Self::MingWuZhengYao => "命无正曜",
            Self::MaTouDaiJian => "马头带箭",
            Self::YangTuoJiaMing => "羊陀夹命",
            Self::HuoLingJiaMing => "火铃夹命",
            Self::KongJieJiaMing => "空劫夹命",
            Self::YangTuoJiaJi => "羊陀夹忌",
            Self::SiShaChongMing => "四煞冲命",
        }
    }

    /// 判断是否为吉格
    pub fn is_auspicious(&self) -> bool {
        (*self as u8) <= 21
    }

    /// 判断是否为凶格
    pub fn is_inauspicious(&self) -> bool {
        (*self as u8) >= 22
    }

    /// 获取格局基础分数
    pub fn base_score(&self) -> i8 {
        match self {
            // 富贵格局
            Self::ZiFuTongGong => 50,
            Self::ZiFuChaoYuan => 45,
            Self::TianFuChaoYuan => 40,
            Self::JunChenQingHui => 45,
            Self::FuXiangChaoYuan => 40,
            Self::JiYueTongLiang => 35,
            Self::RiYueBingMing => 50,
            Self::RiZhaoLeiMen => 40,
            Self::YueLangTianMen => 40,
            Self::MingZhuChuHai => 35,
            Self::YangLiangChangLu => 40,
            Self::TanWuTongXing => 35,
            Self::HuoTanGeJu => 35,
            Self::LingTanGeJu => 30,

            // 权贵格局
            Self::SanQiJiaHui => 45,
            Self::ShuangLuJiaMing => 40,
            Self::ShuangLuJiaCai => 35,
            Self::KeQuanLuJia => 40,
            Self::ZuoYouJiaMing => 35,
            Self::ChangQuJiaMing => 30,
            Self::KuiYueJiaMing => 30,
            Self::LuMaJiaoChiGeJu => 30,

            // 凶格
            Self::LingChangTuoWu => -40,
            Self::JiJiTongGong => -30,
            Self::JuRiTongGong => -35,
            Self::MingWuZhengYao => -20,
            Self::MaTouDaiJian => -35,
            Self::YangTuoJiaMing => -40,
            Self::HuoLingJiaMing => -35,
            Self::KongJieJiaMing => -30,
            Self::YangTuoJiaJi => -45,
            Self::SiShaChongMing => -50,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fortune_level() {
        assert_eq!(FortuneLevel::DaJi.name(), "大吉");
        assert!(FortuneLevel::DaJi.is_auspicious());
        assert!(!FortuneLevel::DaJi.is_inauspicious());
        assert_eq!(FortuneLevel::DaJi.score(), 7);

        assert_eq!(FortuneLevel::DaXiong.name(), "大凶");
        assert!(!FortuneLevel::DaXiong.is_auspicious());
        assert!(FortuneLevel::DaXiong.is_inauspicious());
        assert_eq!(FortuneLevel::DaXiong.score(), 1);
    }

    #[test]
    fn test_fortune_level_from_score() {
        assert_eq!(FortuneLevel::from_score(95), FortuneLevel::DaJi);
        assert_eq!(FortuneLevel::from_score(80), FortuneLevel::Ji);
        assert_eq!(FortuneLevel::from_score(65), FortuneLevel::XiaoJi);
        assert_eq!(FortuneLevel::from_score(50), FortuneLevel::Ping);
        assert_eq!(FortuneLevel::from_score(30), FortuneLevel::XiaoXiong);
        assert_eq!(FortuneLevel::from_score(15), FortuneLevel::Xiong);
        assert_eq!(FortuneLevel::from_score(5), FortuneLevel::DaXiong);
    }

    #[test]
    fn test_ming_ge_level() {
        assert_eq!(MingGeLevel::Putong.name(), "普通格局");
        assert_eq!(MingGeLevel::DiWang.name(), "帝王格局");
    }

    #[test]
    fn test_pattern_type() {
        assert_eq!(PatternType::ZiFuTongGong.name(), "紫府同宫");
        assert!(PatternType::ZiFuTongGong.is_auspicious());
        assert!(!PatternType::ZiFuTongGong.is_inauspicious());
        assert_eq!(PatternType::ZiFuTongGong.base_score(), 50);

        assert_eq!(PatternType::YangTuoJiaMing.name(), "羊陀夹命");
        assert!(!PatternType::YangTuoJiaMing.is_auspicious());
        assert!(PatternType::YangTuoJiaMing.is_inauspicious());
        assert_eq!(PatternType::YangTuoJiaMing.base_score(), -40);
    }
}
