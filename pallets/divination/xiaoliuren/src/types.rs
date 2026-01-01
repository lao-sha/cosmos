//! # 小六壬类型定义
//!
//! 本模块定义小六壬排盘系统的所有核心类型。
//!
//! ## 小六壬概述
//!
//! 小六壬是中国古代术数之一，又称"诸葛亮马前课"或"掐指速算"。
//! 通过六宫（大安、留连、速喜、赤口、小吉、空亡）来预测吉凶。
//!
//! ## 流派说明
//!
//! 本模块支持**道家小六壬**和**传统流派**两种体系，两者在五行配属上有所不同：
//!
//! | 六神 | 道家流派 | 传统流派 |
//! |------|---------|---------|
//! | 大安 | 木/阳   | 木      |
//! | 留连 | 土/阴   | 水      |
//! | 速喜 | 火/阳   | 火      |
//! | 赤口 | 金/阴   | 金      |
//! | 小吉 | 水/阳   | 木      |
//! | 空亡 | 土/阴   | 土      |
//!
//! ## 六宫含义（道家流派）
//!
//! - **大安**：属木/阳，临青龙，吉祥安康，方位东方
//! - **留连**：属土/阴，临玄武，延迟纠缠，方位东南
//! - **速喜**：属火/阳，临朱雀，快速喜庆，方位南方
//! - **赤口**：属金/阴，临白虎，口舌是非，方位西方
//! - **小吉**：属水/阳，临六合，和合吉利，方位北方
//! - **空亡**：属土/阴，临勾陈，无果忧虑，方位中央
//!
//! ## 十二宫对应
//!
//! 六神对应的十二宫位：
//! - **大安**：事业宫（外）+ 命宫（内）
//! - **留连**：田宅宫（外）+ 奴仆宫（内）
//! - **速喜**：感情宫（外）+ 夫妻宫（内）
//! - **赤口**：疾厄宫（外）+ 兄弟宫（内）
//! - **小吉**：驿马宫（外）+ 子女宫（内）
//! - **空亡**：福德宫（外）+ 父母宫（内）

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::*, BoundedVec};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

// ============================================================================
// 流派定义
// ============================================================================

/// 小六壬流派枚举
///
/// 不同流派在六神的五行配属上有所不同，主要差异在留连和小吉：
/// - 道家流派：留连属土，小吉属水
/// - 传统流派：留连属水，小吉属木
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum XiaoLiuRenSchool {
    /// 道家流派 - 留连属土，小吉属水（默认）
    #[default]
    DaoJia = 0,
    /// 传统流派 - 留连属水，小吉属木
    ChuanTong = 1,
}

impl XiaoLiuRenSchool {
    /// 获取流派名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::DaoJia => "道家流派",
            Self::ChuanTong => "传统流派",
        }
    }

    /// 获取流派说明
    pub fn description(&self) -> &'static str {
        match self {
            Self::DaoJia => "道家小六壬体系，留连属土临玄武，小吉属水临六合，注重体用关系分析",
            Self::ChuanTong => "传统小六壬体系，留连属水临玄武，小吉属木临六合，注重时辰吉凶判断",
        }
    }
}

// ============================================================================
// 十二宫定义
// ============================================================================

/// 十二宫枚举
///
/// 六神对应的命理十二宫，每个六神对应一对宫位（外宫/内宫）
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum TwelvePalace {
    /// 命宫 - 代表自身命运、性格特点
    #[default]
    MingGong = 0,
    /// 事业宫 - 代表事业发展、工作状态
    ShiYeGong = 1,
    /// 田宅宫 - 代表房产、家庭环境
    TianZhaiGong = 2,
    /// 奴仆宫 - 代表下属、仆从关系
    NuPuGong = 3,
    /// 感情宫 - 代表感情状态、情感发展
    GanQingGong = 4,
    /// 夫妻宫 - 代表婚姻、配偶
    FuQiGong = 5,
    /// 疾厄宫 - 代表健康、疾病
    JiEGong = 6,
    /// 兄弟宫 - 代表兄弟姐妹、朋友同事
    XiongDiGong = 7,
    /// 驿马宫 - 代表出行、变动
    YiMaGong = 8,
    /// 子女宫 - 代表子女、晚辈
    ZiNvGong = 9,
    /// 福德宫 - 代表福气、精神状态
    FuDeGong = 10,
    /// 父母宫 - 代表父母、长辈
    FuMuGong = 11,
}

impl TwelvePalace {
    /// 获取宫位名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::MingGong => "命宫",
            Self::ShiYeGong => "事业宫",
            Self::TianZhaiGong => "田宅宫",
            Self::NuPuGong => "奴仆宫",
            Self::GanQingGong => "感情宫",
            Self::FuQiGong => "夫妻宫",
            Self::JiEGong => "疾厄宫",
            Self::XiongDiGong => "兄弟宫",
            Self::YiMaGong => "驿马宫",
            Self::ZiNvGong => "子女宫",
            Self::FuDeGong => "福德宫",
            Self::FuMuGong => "父母宫",
        }
    }

    /// 获取宫位说明
    pub fn description(&self) -> &'static str {
        match self {
            Self::MingGong => "代表自身命运、性格特点、整体运势",
            Self::ShiYeGong => "代表事业发展、工作状态、官运仕途",
            Self::TianZhaiGong => "代表房产置业、家庭环境、安居状况",
            Self::NuPuGong => "代表下属仆从、支配欲望、阴暗私事",
            Self::GanQingGong => "代表感情状态、情感发展、桃花运势",
            Self::FuQiGong => "代表婚姻状况、配偶信息、夫妻关系",
            Self::JiEGong => "代表健康状况、疾病灾祸、外部伤害",
            Self::XiongDiGong => "代表兄弟姐妹、朋友同事、人际关系",
            Self::YiMaGong => "代表出行远行、变动迁移、交通运势",
            Self::ZiNvGong => "代表子女晚辈、生育状况、子孙运势",
            Self::FuDeGong => "代表福气福报、精神状态、内心修养",
            Self::FuMuGong => "代表父母长辈、祖业遗产、根基来源",
        }
    }
}

/// 宫位对（外宫/内宫）
///
/// 每个六神对应一对宫位，外宫表现在外，内宫表现在内
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct PalacePair {
    /// 外宫（动态宫，表现在外）
    pub outer: TwelvePalace,
    /// 内宫（静态宫，表现在内）
    pub inner: TwelvePalace,
}

// ============================================================================
// 六宫（六神）定义
// ============================================================================

/// 六宫枚举
///
/// 小六壬的核心六神，按顺序排列为：大安、留连、速喜、赤口、小吉、空亡
/// 采用道家小六壬体系的五行和阴阳配属
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum LiuGong {
    /// 大安 - 身不动时，属木/阳，临青龙，吉祥安康
    #[default]
    DaAn = 0,
    /// 留连 - 人未归时，属土/阴，临玄武，延迟纠缠
    LiuLian = 1,
    /// 速喜 - 人即至时，属火/阳，临朱雀，快速喜庆
    SuXi = 2,
    /// 赤口 - 官事凶时，属金/阴，临白虎，口舌是非
    ChiKou = 3,
    /// 小吉 - 人来喜时，属水/阳，临六合，和合吉利
    XiaoJi = 4,
    /// 空亡 - 音信稀时，属土/阴，临勾陈，无果忧虑
    KongWang = 5,
}

impl LiuGong {
    /// 从索引创建六宫（0-5循环）
    pub fn from_index(index: u8) -> Self {
        match index % 6 {
            0 => Self::DaAn,
            1 => Self::LiuLian,
            2 => Self::SuXi,
            3 => Self::ChiKou,
            4 => Self::XiaoJi,
            _ => Self::KongWang,
        }
    }

    /// 获取六宫索引
    pub fn index(&self) -> u8 {
        *self as u8
    }

    /// 获取六宫名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::DaAn => "大安",
            Self::LiuLian => "留连",
            Self::SuXi => "速喜",
            Self::ChiKou => "赤口",
            Self::XiaoJi => "小吉",
            Self::KongWang => "空亡",
        }
    }

    /// 获取五行属性（道家流派）
    ///
    /// 道家小六壬五行配属：
    /// - 大安：木
    /// - 留连：土（传统流派为水）
    /// - 速喜：火
    /// - 赤口：金
    /// - 小吉：水（传统流派为木）
    /// - 空亡：土
    pub fn wu_xing(&self) -> WuXing {
        match self {
            Self::DaAn => WuXing::Wood,     // 木
            Self::LiuLian => WuXing::Earth, // 土（道家流派）
            Self::SuXi => WuXing::Fire,     // 火
            Self::ChiKou => WuXing::Metal,  // 金
            Self::XiaoJi => WuXing::Water,  // 水（道家流派）
            Self::KongWang => WuXing::Earth,// 土
        }
    }

    /// 获取阴阳属性（道家流派）
    ///
    /// 六宫阴阳配属：
    /// - 阳：大安、速喜、小吉
    /// - 阴：留连、赤口、空亡
    pub fn yin_yang(&self) -> YinYang {
        match self {
            Self::DaAn => YinYang::Yang,
            Self::LiuLian => YinYang::Yin,
            Self::SuXi => YinYang::Yang,
            Self::ChiKou => YinYang::Yin,
            Self::XiaoJi => YinYang::Yang,
            Self::KongWang => YinYang::Yin,
        }
    }

    /// 获取对应天将
    pub fn tian_jiang(&self) -> &'static str {
        match self {
            Self::DaAn => "青龙",
            Self::LiuLian => "玄武",
            Self::SuXi => "朱雀",
            Self::ChiKou => "白虎",
            Self::XiaoJi => "六合",
            Self::KongWang => "勾陈",
        }
    }

    /// 获取方位（道家流派）
    pub fn direction(&self) -> &'static str {
        match self {
            Self::DaAn => "东方",
            Self::LiuLian => "东南",  // 道家流派：东南方
            Self::SuXi => "南方",
            Self::ChiKou => "西方",
            Self::XiaoJi => "北方",   // 道家流派：北方
            Self::KongWang => "中央",
        }
    }

    /// 获取颜色（对应五行）
    pub fn color(&self) -> &'static str {
        match self {
            Self::DaAn => "青色",     // 木 - 青色
            Self::LiuLian => "黄色",  // 土 - 黄色（道家流派）
            Self::SuXi => "红色",     // 火 - 红色
            Self::ChiKou => "白色",   // 金 - 白色
            Self::XiaoJi => "黑色",   // 水 - 黑色（道家流派）
            Self::KongWang => "黄色", // 土 - 黄色
        }
    }

    /// 获取吉凶等级（1-5，5最吉）
    pub fn fortune_level(&self) -> u8 {
        match self {
            Self::DaAn => 5,    // 大吉
            Self::SuXi => 4,    // 吉
            Self::XiaoJi => 4,  // 吉
            Self::LiuLian => 2, // 平
            Self::ChiKou => 1,  // 凶
            Self::KongWang => 1, // 凶
        }
    }

    /// 是否吉利
    pub fn is_auspicious(&self) -> bool {
        matches!(self, Self::DaAn | Self::SuXi | Self::XiaoJi)
    }

    /// 获取谋事数
    pub fn mou_shi_numbers(&self) -> [u8; 3] {
        match self {
            Self::DaAn => [1, 5, 7],
            Self::LiuLian => [2, 8, 10],
            Self::SuXi => [3, 6, 9],
            Self::ChiKou => [4, 7, 10],
            Self::XiaoJi => [1, 5, 7],
            Self::KongWang => [3, 6, 9],
        }
    }

    /// 获取卦辞
    pub fn gua_ci(&self) -> &'static str {
        match self {
            Self::DaAn => "大安事事昌，求谋在东方，失物去不远。宅舍保平安，行人身未动，病者主无妨，将军回田野，仔细更推详。",
            Self::LiuLian => "留连事难成，求谋日未明，官事只宜缓。去者来回程，失物南方见，急讨方遂心。更需防口舌，人事且平平。",
            Self::SuXi => "速喜喜来临，求财向南行，失物申未午。逢人路上寻，官事有福德，病者无祸侵，田宅六畜吉，行人有音信。",
            Self::ChiKou => "赤口主口舌，官非切要防，失物急去寻，行人有惊慌。鸡犬多作怪，病者出西方，更须防咀咒，恐怕染瘟殃。",
            Self::XiaoJi => "小吉最吉昌，路上好商量，阴人来报喜。失物在坤方，行人立便至，交易甚是强，凡事皆和合，病者祈上苍。",
            Self::KongWang => "空亡事不祥，阴人多乖张，求财无利益。行人有灾殃，失物寻不见，官事有刑伤。病人逢暗鬼，祈解可安康。",
        }
    }

    /// 获取简要描述（道家流派）
    pub fn brief(&self) -> &'static str {
        match self {
            Self::DaAn => "身不动时，五行属木，阳性，颜色青色，方位东方。临青龙。有静止、心安、吉祥之含义。",
            Self::LiuLian => "人未归时，五行属土，阴性，颜色黄色，方位东南。临玄武。有暗味不明、延迟、纠缠、拖延之含义。",
            Self::SuXi => "人即至时，五行属火，阳性，颜色红色，方位南方。临朱雀。有快速、喜庆、吉利之含义。指时机已到。",
            Self::ChiKou => "官事凶时，五行属金，阴性，颜色白色，方位西方。临白虎。有不吉、惊恐、凶险、口舌是非之含义。",
            Self::XiaoJi => "人来喜时，五行属水，阳性，颜色黑色，方位北方。临六合。有和合、吉利之含义。",
            Self::KongWang => "音信稀时，五行属土，阴性，颜色黄色，方位中央。临勾陈。有不吉、无结果、忧虑之含义。",
        }
    }

    // ========================================================================
    // 流派支持方法
    // ========================================================================

    /// 根据流派获取五行属性
    ///
    /// 道家流派与传统流派在留连和小吉的五行配属上有差异
    pub fn wu_xing_by_school(&self, school: XiaoLiuRenSchool) -> WuXing {
        match school {
            XiaoLiuRenSchool::DaoJia => self.wu_xing(),
            XiaoLiuRenSchool::ChuanTong => self.wu_xing_traditional(),
        }
    }

    /// 传统流派五行配属
    ///
    /// 与道家流派的主要差异：
    /// - 留连：水（道家为土）
    /// - 小吉：木（道家为水）
    pub fn wu_xing_traditional(&self) -> WuXing {
        match self {
            Self::DaAn => WuXing::Wood,     // 木
            Self::LiuLian => WuXing::Water, // 水（传统流派）
            Self::SuXi => WuXing::Fire,     // 火
            Self::ChiKou => WuXing::Metal,  // 金
            Self::XiaoJi => WuXing::Wood,   // 木（传统流派）
            Self::KongWang => WuXing::Earth,// 土
        }
    }

    /// 根据流派获取方位
    pub fn direction_by_school(&self, school: XiaoLiuRenSchool) -> &'static str {
        match school {
            XiaoLiuRenSchool::DaoJia => self.direction(),
            XiaoLiuRenSchool::ChuanTong => self.direction_traditional(),
        }
    }

    /// 传统流派方位
    pub fn direction_traditional(&self) -> &'static str {
        match self {
            Self::DaAn => "东方",
            Self::LiuLian => "北方",   // 传统流派：北方（水）
            Self::SuXi => "南方",
            Self::ChiKou => "西方",
            Self::XiaoJi => "东方",    // 传统流派：东方（木）
            Self::KongWang => "中央",
        }
    }

    /// 根据流派获取颜色
    pub fn color_by_school(&self, school: XiaoLiuRenSchool) -> &'static str {
        match school {
            XiaoLiuRenSchool::DaoJia => self.color(),
            XiaoLiuRenSchool::ChuanTong => self.color_traditional(),
        }
    }

    /// 传统流派颜色
    pub fn color_traditional(&self) -> &'static str {
        match self {
            Self::DaAn => "青色",
            Self::LiuLian => "黑色",   // 传统流派：黑色（水）
            Self::SuXi => "红色",
            Self::ChiKou => "白色",
            Self::XiaoJi => "青色",    // 传统流派：青色（木）
            Self::KongWang => "黄色",
        }
    }

    // ========================================================================
    // 十二宫对应方法
    // ========================================================================

    /// 获取对应的十二宫位对
    ///
    /// 六神对应的十二宫：
    /// - 大安：事业宫（外）+ 命宫（内）
    /// - 留连：田宅宫（外）+ 奴仆宫（内）
    /// - 速喜：感情宫（外）+ 夫妻宫（内）
    /// - 赤口：疾厄宫（外）+ 兄弟宫（内）
    /// - 小吉：驿马宫（外）+ 子女宫（内）
    /// - 空亡：福德宫（外）+ 父母宫（内）
    pub fn twelve_palace(&self) -> PalacePair {
        match self {
            Self::DaAn => PalacePair {
                outer: TwelvePalace::ShiYeGong,
                inner: TwelvePalace::MingGong,
            },
            Self::LiuLian => PalacePair {
                outer: TwelvePalace::TianZhaiGong,
                inner: TwelvePalace::NuPuGong,
            },
            Self::SuXi => PalacePair {
                outer: TwelvePalace::GanQingGong,
                inner: TwelvePalace::FuQiGong,
            },
            Self::ChiKou => PalacePair {
                outer: TwelvePalace::JiEGong,
                inner: TwelvePalace::XiongDiGong,
            },
            Self::XiaoJi => PalacePair {
                outer: TwelvePalace::YiMaGong,
                inner: TwelvePalace::ZiNvGong,
            },
            Self::KongWang => PalacePair {
                outer: TwelvePalace::FuDeGong,
                inner: TwelvePalace::FuMuGong,
            },
        }
    }

    /// 获取藏干
    ///
    /// 六神对应的藏干（天干隐藏于地支中）
    pub fn hidden_stems(&self) -> (&'static str, &'static str) {
        match self {
            Self::DaAn => ("甲", "丁"),
            Self::LiuLian => ("丁", "己"),
            Self::SuXi => ("丙", "辛"),
            Self::ChiKou => ("庚", "癸"),
            Self::XiaoJi => ("壬", "甲"),
            Self::KongWang => ("戊", "乙"),
        }
    }

    /// 获取对应天干
    pub fn tian_gan(&self) -> &'static str {
        match self {
            Self::DaAn => "甲乙",
            Self::LiuLian => "戊己",   // 道家流派（土）
            Self::SuXi => "丙丁",
            Self::ChiKou => "庚辛",
            Self::XiaoJi => "壬癸",    // 道家流派（水）
            Self::KongWang => "戊己",
        }
    }

    /// 获取对应地支月份
    pub fn di_zhi_months(&self) -> &'static str {
        match self {
            Self::DaAn => "寅卯辰月",
            Self::LiuLian => "辰巳月",
            Self::SuXi => "巳午未月",
            Self::ChiKou => "申酉戌月",
            Self::XiaoJi => "亥子丑月",
            Self::KongWang => "丑寅月",
        }
    }

    /// 获取对应季节
    pub fn season(&self) -> &'static str {
        match self {
            Self::DaAn => "春季",
            Self::LiuLian => "春夏之交",
            Self::SuXi => "夏季",
            Self::ChiKou => "秋季",
            Self::XiaoJi => "冬季",
            Self::KongWang => "冬春之交",
        }
    }

    /// 获取扩展数字范围
    ///
    /// 返回四个关联数字
    pub fn number_range(&self) -> [u8; 4] {
        match self {
            Self::DaAn => [1, 7, 4, 5],
            Self::LiuLian => [2, 8, 7, 8],
            Self::SuXi => [3, 9, 6, 9],
            Self::ChiKou => [4, 10, 1, 2],
            Self::XiaoJi => [5, 11, 3, 8],
            Self::KongWang => [6, 12, 5, 10],
        }
    }

    /// 获取详细解释（扩展版）
    pub fn detailed_explanation(&self) -> &'static str {
        match self {
            Self::DaAn => "大安事事昌，求财在坤方，失物去不远，宅舍保安康，行人身未动，病者主无妨。将军回田野，仔细与推详，丢失在附近，可能西南向，安居得吉日，不可动身祥。办事别出屋，求借邀自房，得病凶化吉，久疾得安康，寻人知音信，可能归村庄。口舌能消散，远行要提防，交易别出村，离屯细推详，求财有八分，得全不出房。",
            Self::LiuLian => "留连事未当，求事日莫光，凡事只宜缓，去者未回向，失物南方去，急急行便访。紧记防口舌，人口且平祥，丢失难寻找，窃者又转场，出行定不归，久去拖延长。办事不果断，牵连又返往，求借不易成，被求而彷徨，此日患疾病，几天不复康。找人迷雾中，迷迷又恍惚，口舌继续有，拖拉又伸长，女方嫁吉日，求财六分量。",
            Self::SuXi => "速喜喜临乡，求财往南方，失物申午未，逢人路寻详，官事有福德，病者无大伤。六畜田稼庆，行人有音向，丢失得音信，微乐在面上，出行遇吉利，小喜而顺当。办事如逢春，吉利又荣光，小量可求借，大事难全强，久病见小愈，得病速回康，寻人得知见，口舌见消亡，交易可得成，但不太久长，求财有十分，吉时得顺当。",
            Self::ChiKou => "赤口主口伤，官事且紧防，失物急去找，行人有惊慌，鸡犬多作怪，病者上西方。更须防咒咀，恐怕染瘟殃，找物犯谎口，寻问无音向，出门千口怨，言谈万骂伤。办事犯口舌，难成有阻挡，求借不全顺，闭口无事张，得病千口猜，求医还无妨。寻人得凶音，人心不安详，口舌犯最重，交易口舌防，求财只四分，逢吉才成当。",
            Self::XiaoJi => "小吉最吉昌，路上好商量，阴人来报喜，失物在坤方，行人立刻至，交易甚是强。凡事皆合好，病者保安康，大吉又大顺，万事如意详，出行可得喜，千里吉安祥。诸事可心顺，有忧皆消光，求借自来助，众友愿相帮，重病莫要愁，久病得安康。不见得相见，不打自归庄，千人称赞君，无限上荣光，交易成兴隆，十二分财量。",
            Self::KongWang => "空亡事不长，阴人无主张，求财心白费，行人有灾殃，失物永不见，官事有刑伤。病人遇邪鬼，久病添祸殃，失物难找见，找寻空荡荡，出行不吉利，凶多不吉祥。办事凶为多，处处有阻挡，求借不能成，成事化败伤，得病凶多噩，久患雪加霜。寻人无音信，知音变空想，万口都诽骂，小舟遭狂浪，求财有二分，不吉不利亡。",
        }
    }
}

// ============================================================================
// 五行定义
// ============================================================================

/// 五行枚举
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum WuXing {
    /// 木
    #[default]
    Wood = 0,
    /// 火
    Fire = 1,
    /// 土
    Earth = 2,
    /// 金
    Metal = 3,
    /// 水
    Water = 4,
}

impl WuXing {
    /// 获取五行名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Wood => "木",
            Self::Fire => "火",
            Self::Earth => "土",
            Self::Metal => "金",
            Self::Water => "水",
        }
    }

    /// 我生（子）
    pub fn generates(&self) -> Self {
        match self {
            Self::Wood => Self::Fire,
            Self::Fire => Self::Earth,
            Self::Earth => Self::Metal,
            Self::Metal => Self::Water,
            Self::Water => Self::Wood,
        }
    }

    /// 我克
    pub fn restrains(&self) -> Self {
        match self {
            Self::Wood => Self::Earth,
            Self::Fire => Self::Metal,
            Self::Earth => Self::Water,
            Self::Metal => Self::Wood,
            Self::Water => Self::Fire,
        }
    }

    /// 生我（母）
    pub fn generated_by(&self) -> Self {
        match self {
            Self::Wood => Self::Water,
            Self::Fire => Self::Wood,
            Self::Earth => Self::Fire,
            Self::Metal => Self::Earth,
            Self::Water => Self::Metal,
        }
    }

    /// 克我
    pub fn restrained_by(&self) -> Self {
        match self {
            Self::Wood => Self::Metal,
            Self::Fire => Self::Water,
            Self::Earth => Self::Wood,
            Self::Metal => Self::Fire,
            Self::Water => Self::Earth,
        }
    }
}

// ============================================================================
// 阴阳定义
// ============================================================================

/// 阴阳枚举
///
/// 用于六宫和时辰的阴阳属性
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum YinYang {
    /// 阳
    #[default]
    Yang = 0,
    /// 阴
    Yin = 1,
}

impl YinYang {
    /// 获取阴阳名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Yang => "阳",
            Self::Yin => "阴",
        }
    }

    /// 是否为阳
    pub fn is_yang(&self) -> bool {
        matches!(self, Self::Yang)
    }

    /// 是否为阴
    pub fn is_yin(&self) -> bool {
        matches!(self, Self::Yin)
    }
}

// ============================================================================
// 子时类型区分
// ============================================================================

/// 子时类型枚举
///
/// 子时横跨两天，在某些流派中需要区分早子时和晚子时：
/// - 早子时（夜子时）：23:00-24:00，属于当天
/// - 晚子时（正子时）：00:00-01:00，属于次日
///
/// 在日干支计算中，早子时算当天，晚子时算次日
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum ZiShiType {
    /// 早子时（夜子时）- 23:00-24:00，属于当天
    #[default]
    EarlyZi = 0,
    /// 晚子时（正子时）- 00:00-01:00，属于次日
    LateZi = 1,
}

impl ZiShiType {
    /// 获取子时类型名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::EarlyZi => "早子时",
            Self::LateZi => "晚子时",
        }
    }

    /// 获取别名
    pub fn alias(&self) -> &'static str {
        match self {
            Self::EarlyZi => "夜子时",
            Self::LateZi => "正子时",
        }
    }

    /// 获取时间范围
    pub fn time_range(&self) -> &'static str {
        match self {
            Self::EarlyZi => "23:00-24:00",
            Self::LateZi => "00:00-01:00",
        }
    }

    /// 是否算作当天
    ///
    /// 早子时算当天，晚子时算次日
    pub fn is_current_day(&self) -> bool {
        matches!(self, Self::EarlyZi)
    }
}

// ============================================================================
// 起课方式
// ============================================================================

/// 起课方式枚举
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum DivinationMethod {
    /// 时间起课（月日时）- 以农历月日时起课（传统方法）
    #[default]
    TimeMethod = 0,
    /// 时间起课（时刻分）- 以时辰、刻、分起课（道家流派）
    TimeKeMethod = 1,
    /// 数字起课 - 以三个数字起课（活数起课法）
    NumberMethod = 2,
    /// 随机起课 - 使用链上随机数起课
    RandomMethod = 3,
    /// 手动指定 - 直接指定三宫结果
    ManualMethod = 4,
}

impl DivinationMethod {
    /// 获取起课方式名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::TimeMethod => "月日时起课",
            Self::TimeKeMethod => "时刻分起课",
            Self::NumberMethod => "数字起课",
            Self::RandomMethod => "随机起课",
            Self::ManualMethod => "手动指定",
        }
    }
}

// ============================================================================
// 十二时辰
// ============================================================================

/// 十二时辰枚举
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum ShiChen {
    /// 子时 (23:00-01:00)
    #[default]
    Zi = 0,
    /// 丑时 (01:00-03:00)
    Chou = 1,
    /// 寅时 (03:00-05:00)
    Yin = 2,
    /// 卯时 (05:00-07:00)
    Mao = 3,
    /// 辰时 (07:00-09:00)
    Chen = 4,
    /// 巳时 (09:00-11:00)
    Si = 5,
    /// 午时 (11:00-13:00)
    Wu = 6,
    /// 未时 (13:00-15:00)
    Wei = 7,
    /// 申时 (15:00-17:00)
    Shen = 8,
    /// 酉时 (17:00-19:00)
    You = 9,
    /// 戌时 (19:00-21:00)
    Xu = 10,
    /// 亥时 (21:00-23:00)
    Hai = 11,
}

impl ShiChen {
    /// 从小时数计算时辰（0-23）
    ///
    /// 时辰对应关系：
    /// - 子时：23:00-01:00（包括23点、0点）
    /// - 丑时：01:00-03:00（包括1点、2点）
    /// - 寅时：03:00-05:00（包括3点、4点）
    /// - 卯时：05:00-07:00（包括5点、6点）
    /// - 辰时：07:00-09:00（包括7点、8点）
    /// - 巳时：09:00-11:00（包括9点、10点）
    /// - 午时：11:00-13:00（包括11点、12点）
    /// - 未时：13:00-15:00（包括13点、14点）
    /// - 申时：15:00-17:00（包括15点、16点）
    /// - 酉时：17:00-19:00（包括17点、18点）
    /// - 戌时：19:00-21:00（包括19点、20点）
    /// - 亥时：21:00-23:00（包括21点、22点）
    pub fn from_hour(hour: u8) -> Self {
        // 子时特殊处理：23点和0点都属于子时
        match hour {
            23 | 0 => Self::Zi,
            1 | 2 => Self::Chou,
            3 | 4 => Self::Yin,
            5 | 6 => Self::Mao,
            7 | 8 => Self::Chen,
            9 | 10 => Self::Si,
            11 | 12 => Self::Wu,
            13 | 14 => Self::Wei,
            15 | 16 => Self::Shen,
            17 | 18 => Self::You,
            19 | 20 => Self::Xu,
            _ => Self::Hai, // 21, 22
        }
    }

    /// 从小时数计算时辰，并返回子时类型（如果是子时）
    ///
    /// 此方法区分早子时（23:00-24:00）和晚子时（00:00-01:00）
    pub fn from_hour_detailed(hour: u8) -> (Self, Option<ZiShiType>) {
        match hour {
            23 => (Self::Zi, Some(ZiShiType::EarlyZi)),  // 早子时
            0 => (Self::Zi, Some(ZiShiType::LateZi)),    // 晚子时
            1 | 2 => (Self::Chou, None),
            3 | 4 => (Self::Yin, None),
            5 | 6 => (Self::Mao, None),
            7 | 8 => (Self::Chen, None),
            9 | 10 => (Self::Si, None),
            11 | 12 => (Self::Wu, None),
            13 | 14 => (Self::Wei, None),
            15 | 16 => (Self::Shen, None),
            17 | 18 => (Self::You, None),
            19 | 20 => (Self::Xu, None),
            _ => (Self::Hai, None), // 21, 22
        }
    }

    /// 获取时辰索引（1-12，用于计算）
    pub fn index(&self) -> u8 {
        (*self as u8) + 1
    }

    /// 获取时辰索引（0-11，用于数组索引）
    pub fn zero_index(&self) -> u8 {
        *self as u8
    }

    /// 获取时辰名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Zi => "子时",
            Self::Chou => "丑时",
            Self::Yin => "寅时",
            Self::Mao => "卯时",
            Self::Chen => "辰时",
            Self::Si => "巳时",
            Self::Wu => "午时",
            Self::Wei => "未时",
            Self::Shen => "申时",
            Self::You => "酉时",
            Self::Xu => "戌时",
            Self::Hai => "亥时",
        }
    }

    /// 获取时辰地支名称
    pub fn branch_name(&self) -> &'static str {
        match self {
            Self::Zi => "子",
            Self::Chou => "丑",
            Self::Yin => "寅",
            Self::Mao => "卯",
            Self::Chen => "辰",
            Self::Si => "巳",
            Self::Wu => "午",
            Self::Wei => "未",
            Self::Shen => "申",
            Self::You => "酉",
            Self::Xu => "戌",
            Self::Hai => "亥",
        }
    }

    /// 获取时辰五行属性
    ///
    /// 时辰五行对应：
    /// - 子(水)、丑(土)、寅(木)、卯(木)
    /// - 辰(土)、巳(火)、午(火)、未(土)
    /// - 申(金)、酉(金)、戌(土)、亥(水)
    pub fn wu_xing(&self) -> WuXing {
        match self {
            Self::Zi => WuXing::Water,   // 子 - 水
            Self::Chou => WuXing::Earth, // 丑 - 土
            Self::Yin => WuXing::Wood,   // 寅 - 木
            Self::Mao => WuXing::Wood,   // 卯 - 木
            Self::Chen => WuXing::Earth, // 辰 - 土
            Self::Si => WuXing::Fire,    // 巳 - 火
            Self::Wu => WuXing::Fire,    // 午 - 火
            Self::Wei => WuXing::Earth,  // 未 - 土
            Self::Shen => WuXing::Metal, // 申 - 金
            Self::You => WuXing::Metal,  // 酉 - 金
            Self::Xu => WuXing::Earth,   // 戌 - 土
            Self::Hai => WuXing::Water,  // 亥 - 水
        }
    }

    /// 获取时辰阴阳属性
    ///
    /// 阳：子、寅、辰、午、申、戌（奇数位）
    /// 阴：丑、卯、巳、未、酉、亥（偶数位）
    pub fn yin_yang(&self) -> YinYang {
        match self {
            Self::Zi => YinYang::Yang,   // 阳
            Self::Chou => YinYang::Yin,  // 阴
            Self::Yin => YinYang::Yang,  // 阳
            Self::Mao => YinYang::Yin,   // 阴
            Self::Chen => YinYang::Yang, // 阳
            Self::Si => YinYang::Yin,    // 阴
            Self::Wu => YinYang::Yang,   // 阳
            Self::Wei => YinYang::Yin,   // 阴
            Self::Shen => YinYang::Yang, // 阳
            Self::You => YinYang::Yin,   // 阴
            Self::Xu => YinYang::Yang,   // 阳
            Self::Hai => YinYang::Yin,   // 阴
        }
    }
}

// ============================================================================
// 三宫结果
// ============================================================================

/// 三宫结果
///
/// 小六壬的核心输出：月宫、日宫、时宫
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct SanGong {
    /// 月宫（第一宫）- 代表事情的起因或背景
    pub yue_gong: LiuGong,
    /// 日宫（第二宫）- 代表事情的经过或现状
    pub ri_gong: LiuGong,
    /// 时宫（第三宫）- 代表事情的结果或未来
    pub shi_gong: LiuGong,
}

impl SanGong {
    /// 创建三宫
    pub fn new(yue: LiuGong, ri: LiuGong, shi: LiuGong) -> Self {
        Self {
            yue_gong: yue,
            ri_gong: ri,
            shi_gong: shi,
        }
    }

    /// 获取综合吉凶等级（1-5）
    pub fn fortune_level(&self) -> u8 {
        // 以时宫（结果）为主，综合三宫
        let base = self.shi_gong.fortune_level();
        let avg = (self.yue_gong.fortune_level() + self.ri_gong.fortune_level() + self.shi_gong.fortune_level()) / 3;

        // 结果占60%，过程占40%
        (base * 6 + avg * 4) / 10
    }

    /// 检查是否全吉（三宫皆吉）
    pub fn is_all_auspicious(&self) -> bool {
        self.yue_gong.is_auspicious() && self.ri_gong.is_auspicious() && self.shi_gong.is_auspicious()
    }

    /// 检查是否全凶（三宫皆凶）
    pub fn is_all_inauspicious(&self) -> bool {
        !self.yue_gong.is_auspicious() && !self.ri_gong.is_auspicious() && !self.shi_gong.is_auspicious()
    }

    /// 检查是否为纯宫（三宫相同）
    pub fn is_pure(&self) -> bool {
        self.yue_gong == self.ri_gong && self.ri_gong == self.shi_gong
    }

    /// 获取五行关系分析
    pub fn wu_xing_analysis(&self) -> WuXingRelation {
        let _wx1 = self.yue_gong.wu_xing();
        let wx2 = self.ri_gong.wu_xing();
        let wx3 = self.shi_gong.wu_xing();

        // 分析日宫到时宫的关系（主要看结果）
        if wx2.generates() == wx3 {
            WuXingRelation::Sheng // 生
        } else if wx2.restrains() == wx3 {
            WuXingRelation::Ke // 克
        } else if wx2 == wx3 {
            WuXingRelation::BiHe // 比和
        } else if wx2.generated_by() == wx3 {
            WuXingRelation::XieSheng // 泄
        } else {
            WuXingRelation::BeiKe // 被克
        }
    }
}

/// 五行关系
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum WuXingRelation {
    /// 相生
    #[default]
    Sheng = 0,
    /// 相克
    Ke = 1,
    /// 比和
    BiHe = 2,
    /// 泄气
    XieSheng = 3,
    /// 被克
    BeiKe = 4,
}

impl WuXingRelation {
    /// 获取关系名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Sheng => "相生",
            Self::Ke => "相克",
            Self::BiHe => "比和",
            Self::XieSheng => "泄气",
            Self::BeiKe => "被克",
        }
    }

    /// 获取关系对吉凶的影响（正负值）
    pub fn fortune_modifier(&self) -> i8 {
        match self {
            Self::Sheng => 1,   // 生助为吉
            Self::BiHe => 1,   // 比和为吉
            Self::Ke => -1,    // 克制为凶
            Self::BeiKe => -1, // 被克为凶
            Self::XieSheng => 0, // 泄气为平
        }
    }
}

// ============================================================================
// 体用关系（道家小六壬核心分析）
// ============================================================================

/// 体用关系枚举
///
/// 道家小六壬的体用关系分析：
/// - 体：人宫（时宫），代表求测者自身
/// - 用：时辰，代表外部环境或时机
///
/// 六种体用关系：
/// - 用生体（大吉）：外部环境生助自身
/// - 体克用（小吉）：自身克制环境，占主动
/// - 用克体（大凶）：外部环境克制自身
/// - 体生用（小凶）：自身精力外泄
/// - 比肩（中平）：五行相同，阴阳相同
/// - 比助（中平）：五行相同，阴阳不同
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum TiYongRelation {
    /// 用生体 - 大吉（外部环境生助自身）
    #[default]
    YongShengTi = 0,
    /// 体克用 - 小吉（自身克制环境）
    TiKeYong = 1,
    /// 用克体 - 大凶（外部环境克制自身）
    YongKeTi = 2,
    /// 体生用 - 小凶（自身精力外泄）
    TiShengYong = 3,
    /// 比肩 - 中平（五行相同，阴阳相同）
    BiJian = 4,
    /// 比助 - 中平（五行相同，阴阳不同）
    BiZhu = 5,
}

impl TiYongRelation {
    /// 获取体用关系名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::YongShengTi => "用生体",
            Self::TiKeYong => "体克用",
            Self::YongKeTi => "用克体",
            Self::TiShengYong => "体生用",
            Self::BiJian => "比肩",
            Self::BiZhu => "比助",
        }
    }

    /// 获取吉凶描述
    pub fn fortune_desc(&self) -> &'static str {
        match self {
            Self::YongShengTi => "大吉",
            Self::TiKeYong => "小吉",
            Self::YongKeTi => "大凶",
            Self::TiShengYong => "小凶",
            Self::BiJian => "中平",
            Self::BiZhu => "中平",
        }
    }

    /// 获取吉凶等级（1-6，6最吉）
    pub fn fortune_level(&self) -> u8 {
        match self {
            Self::YongShengTi => 6, // 大吉
            Self::TiKeYong => 5,    // 小吉
            Self::BiJian => 4,      // 中平
            Self::BiZhu => 3,       // 中平
            Self::TiShengYong => 2, // 小凶
            Self::YongKeTi => 1,    // 大凶
        }
    }

    /// 计算体用关系
    ///
    /// # 参数
    /// - `ti`: 体（人宫/时宫的六宫）
    /// - `yong`: 用（时辰）
    ///
    /// # 返回
    /// 体用关系
    pub fn calculate(ti: LiuGong, yong: ShiChen) -> Self {
        let ti_wx = ti.wu_xing();
        let ti_yy = ti.yin_yang();
        let yong_wx = yong.wu_xing();
        let yong_yy = yong.yin_yang();

        // 五行相同的情况
        if ti_wx == yong_wx {
            if ti_yy == yong_yy {
                return Self::BiJian; // 比肩
            } else {
                return Self::BiZhu; // 比助
            }
        }

        // 用生体：用的五行生体的五行
        if yong_wx.generates() == ti_wx {
            return Self::YongShengTi;
        }

        // 体生用：体的五行生用的五行
        if ti_wx.generates() == yong_wx {
            return Self::TiShengYong;
        }

        // 体克用：体的五行克用的五行
        if ti_wx.restrains() == yong_wx {
            return Self::TiKeYong;
        }

        // 用克体：用的五行克体的五行
        if yong_wx.restrains() == ti_wx {
            return Self::YongKeTi;
        }

        // 默认返回比助（理论上不应到达这里）
        Self::BiZhu
    }
}

// ============================================================================
// 八卦定义（用于八卦具象法）
// ============================================================================

/// 八卦枚举
///
/// 用于三宫转化八卦的具象法分析
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum BaGua {
    /// 乾卦 ☰ - 阳阳阳，五行属金
    #[default]
    Qian = 0,
    /// 兑卦 ☱ - 阴阳阳，五行属金
    Dui = 1,
    /// 离卦 ☲ - 阳阴阳，五行属火
    Li = 2,
    /// 震卦 ☳ - 阴阴阳，五行属木
    Zhen = 3,
    /// 巽卦 ☴ - 阳阳阴，五行属木
    Xun = 4,
    /// 坎卦 ☵ - 阴阳阴，五行属水
    Kan = 5,
    /// 艮卦 ☶ - 阳阴阴，五行属土
    Gen = 6,
    /// 坤卦 ☷ - 阴阴阴，五行属土
    Kun = 7,
}

impl BaGua {
    /// 获取八卦名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Qian => "乾",
            Self::Dui => "兑",
            Self::Li => "离",
            Self::Zhen => "震",
            Self::Xun => "巽",
            Self::Kan => "坎",
            Self::Gen => "艮",
            Self::Kun => "坤",
        }
    }

    /// 获取八卦符号
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Qian => "☰",
            Self::Dui => "☱",
            Self::Li => "☲",
            Self::Zhen => "☳",
            Self::Xun => "☴",
            Self::Kan => "☵",
            Self::Gen => "☶",
            Self::Kun => "☷",
        }
    }

    /// 获取八卦五行
    pub fn wu_xing(&self) -> WuXing {
        match self {
            Self::Qian => WuXing::Metal,
            Self::Dui => WuXing::Metal,
            Self::Li => WuXing::Fire,
            Self::Zhen => WuXing::Wood,
            Self::Xun => WuXing::Wood,
            Self::Kan => WuXing::Water,
            Self::Gen => WuXing::Earth,
            Self::Kun => WuXing::Earth,
        }
    }

    /// 获取八卦阴阳
    pub fn yin_yang(&self) -> YinYang {
        match self {
            Self::Qian => YinYang::Yang, // 纯阳
            Self::Dui => YinYang::Yin,   // 少阴
            Self::Li => YinYang::Yang,   // 中女
            Self::Zhen => YinYang::Yang, // 长男
            Self::Xun => YinYang::Yang,  // 长女
            Self::Kan => YinYang::Yang,  // 中男
            Self::Gen => YinYang::Yin,   // 少男
            Self::Kun => YinYang::Yang,  // 纯阴（但属阳卦）
        }
    }

    /// 获取八卦简要描述
    pub fn brief(&self) -> &'static str {
        match self {
            Self::Qian => "五行属金，方位为西北，人物为老年男性或当官的。为46岁以上男性。天、父、老人、官贵、头、骨、马、金、宝珠、玉。",
            Self::Dui => "五行属金，方位为西方，人物为小女儿或少女。为1-15岁女性。泽、少女、舌、妾、肺、羊、毁抓之物、带口之器。",
            Self::Li => "五行属火，方位南方，人物为二女儿或中年女性。为16-30岁女性。火、雉、日、目、电、中女、甲胄、戈兵、文书。",
            Self::Zhen => "五行属木，方位为东方，人物为大儿子、军警人员。为31-45岁男性。雷、长男、足、发、龙、百虫、蹄、竹。",
            Self::Xun => "五行属木，方位东南，人物为大女儿或大儿媳妇。为31-45岁女性。风、长女、僧尼、鸡、股、百禽、百草、香气。",
            Self::Kan => "五行属水，方位北方，人物为二儿子或中年男性。为16-30岁男性。水、雨雪、工、猪、中男、沟渎、弓轮、耳、血、月。",
            Self::Gen => "五行属土，方位东北，人物为小儿子或少年男性。为1-15岁男性。山、土、少男、童子、狗、手、指、径路、门阙。",
            Self::Kun => "五行属土，方位为西南，人物为老年妇女或女主人。为46岁以上的女性。地、母、老妇、土、牛、釜、布帛、文章。",
        }
    }

    /// 从三爻（阴阳）创建八卦
    ///
    /// # 参数
    /// - `yao1`: 上爻（天宫阴阳）
    /// - `yao2`: 中爻（地宫阴阳）
    /// - `yao3`: 下爻（人宫阴阳）
    ///
    /// # 爻位说明
    /// 阳爻用 YinYang::Yang 表示
    /// 阴爻用 YinYang::Yin 表示
    pub fn from_yao(yao1: YinYang, yao2: YinYang, yao3: YinYang) -> Self {
        match (yao1, yao2, yao3) {
            (YinYang::Yang, YinYang::Yang, YinYang::Yang) => Self::Qian, // 阳阳阳 = 乾
            (YinYang::Yin, YinYang::Yang, YinYang::Yang) => Self::Dui,   // 阴阳阳 = 兑
            (YinYang::Yang, YinYang::Yin, YinYang::Yang) => Self::Li,    // 阳阴阳 = 离
            (YinYang::Yin, YinYang::Yin, YinYang::Yang) => Self::Zhen,   // 阴阴阳 = 震
            (YinYang::Yang, YinYang::Yang, YinYang::Yin) => Self::Xun,   // 阳阳阴 = 巽
            (YinYang::Yin, YinYang::Yang, YinYang::Yin) => Self::Kan,    // 阴阳阴 = 坎
            (YinYang::Yang, YinYang::Yin, YinYang::Yin) => Self::Gen,    // 阳阴阴 = 艮
            (YinYang::Yin, YinYang::Yin, YinYang::Yin) => Self::Kun,     // 阴阴阴 = 坤
        }
    }

    /// 从三宫转化为八卦
    ///
    /// 将三宫的阴阳属性转化为八卦
    pub fn from_san_gong(san_gong: &SanGong) -> Self {
        Self::from_yao(
            san_gong.yue_gong.yin_yang(), // 天宫（月宫）
            san_gong.ri_gong.yin_yang(),  // 地宫（日宫）
            san_gong.shi_gong.yin_yang(), // 人宫（时宫）
        )
    }
}

// ============================================================================
// 小六壬课盘
// ============================================================================

/// 小六壬课盘
///
/// 存储完整的小六壬排盘结果
///
/// ## 隐私模式说明
///
/// 支持三种隐私模式：
/// - **Public**: 所有数据明文存储，任何人可查看
/// - **Partial**: 计算数据明文，敏感数据（问题内容等）加密
/// - **Private**: 所有数据加密，仅存储元数据
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
#[scale_info(skip_type_params(MaxCidLen))]
pub struct XiaoLiuRenPan<AccountId, BlockNumber, MaxCidLen: Get<u32>> {
    /// 课盘ID
    pub id: u64,
    /// 创建者
    pub creator: AccountId,
    /// 创建区块
    pub created_at: BlockNumber,

    // ============ 隐私控制字段 ============

    /// 隐私模式（必有）
    pub privacy_mode: pallet_divination_privacy::types::PrivacyMode,
    /// 加密字段位图（可选，Partial 模式使用）
    /// bit 0: question_cid 已加密
    /// bit 1: san_gong 已加密（Private 模式）
    pub encrypted_fields: Option<u8>,
    /// 敏感数据哈希（用于验证完整性）
    pub sensitive_data_hash: Option<[u8; 32]>,

    // ============ 起课信息 ============

    /// 起课方式
    pub method: DivinationMethod,
    /// 占问事项CID（IPFS）
    pub question_cid: Option<BoundedVec<u8, MaxCidLen>>,

    /// 起课参数（Private 模式时为 None）
    /// 对于时间起课：月、日、时
    /// 对于数字起课：三个数字
    pub param1: Option<u8>,
    pub param2: Option<u8>,
    pub param3: Option<u8>,

    /// 农历信息（可选，时间起课时使用）
    pub lunar_month: Option<u8>,
    pub lunar_day: Option<u8>,
    pub shi_chen: Option<ShiChen>,

    /// 三宫结果（Private 模式时为 None）
    pub san_gong: Option<SanGong>,

    /// AI 解读 CID
    pub ai_interpretation_cid: Option<BoundedVec<u8, MaxCidLen>>,
}

impl<AccountId, BlockNumber, MaxCidLen: Get<u32>> XiaoLiuRenPan<AccountId, BlockNumber, MaxCidLen> {
    /// 检查是否有计算数据（用于解盘）
    pub fn has_calculation_data(&self) -> bool {
        self.san_gong.is_some()
    }

    /// 检查是否可解读
    ///
    /// Private 模式无计算数据，无法解读
    pub fn can_interpret(&self) -> bool {
        self.san_gong.is_some()
    }

    /// 检查是否公开
    pub fn is_public(&self) -> bool {
        matches!(self.privacy_mode, pallet_divination_privacy::types::PrivacyMode::Public)
    }

    /// 检查是否完全私有
    pub fn is_private(&self) -> bool {
        matches!(self.privacy_mode, pallet_divination_privacy::types::PrivacyMode::Private)
    }
}

// ============================================================================
// 用户统计
// ============================================================================

/// 用户统计数据
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct UserStats {
    /// 总起课次数
    pub total_pans: u32,
    /// AI 解读次数
    pub ai_interpretations: u32,
    /// 首次起课区块
    pub first_pan_block: u32,
}
