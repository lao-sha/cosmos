//! 梅花易数 - 基础数据类型定义
//!
//! 本模块定义了梅花易数排盘系统所需的所有核心数据结构，包括：
//! - 八卦枚举 (Bagua)
//! - 五行枚举 (WuXing)
//! - 体用关系 (TiYongRelation)
//! - 起卦方式 (DivinationMethod)
//! - 单卦结构 (SingleGua)
//! - 六十四卦结构 (Hexagram)
//! - 完整卦象 (FullDivination)
//! - 加密隐私数据 (EncryptedPrivacyData) - 用于原子性隐私数据存储

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_std::prelude::*;

// 重新导出 privacy pallet 的类型，供外部使用
pub use pallet_divination_privacy::types::PrivacyMode;

/// 八卦枚举 - 先天八卦数序
///
/// 先天八卦数：乾一、兑二、离三、震四、巽五、坎六、艮七、坤八
/// 二进制表示：乾(111)、兑(011)、离(101)、震(001)、巽(110)、坎(010)、艮(100)、坤(000)
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum Bagua {
    /// 乾卦 ☰ - 先天数1，二进制111，五行金，象天
    Qian = 1,
    /// 兑卦 ☱ - 先天数2，二进制011，五行金，象泽
    Dui = 2,
    /// 离卦 ☲ - 先天数3，二进制101，五行火，象火
    Li = 3,
    /// 震卦 ☳ - 先天数4，二进制001，五行木，象雷
    Zhen = 4,
    /// 巽卦 ☴ - 先天数5，二进制110，五行木，象风
    Xun = 5,
    /// 坎卦 ☵ - 先天数6，二进制010，五行水，象水
    Kan = 6,
    /// 艮卦 ☶ - 先天数7，二进制100，五行土，象山
    Gen = 7,
    /// 坤卦 ☷ - 先天数8(或0)，二进制000，五行土，象地
    #[default]
    Kun = 8,
}

impl Bagua {
    /// 从先天八卦数创建八卦（1-8，0和8都对应坤卦）
    ///
    /// # 参数
    /// - `num`: 先天八卦数 (1-8)
    ///
    /// # 返回
    /// - 对应的八卦枚举值
    pub fn from_num(num: u8) -> Self {
        match num % 8 {
            1 => Bagua::Qian,
            2 => Bagua::Dui,
            3 => Bagua::Li,
            4 => Bagua::Zhen,
            5 => Bagua::Xun,
            6 => Bagua::Kan,
            7 => Bagua::Gen,
            _ => Bagua::Kun, // 0 或 8
        }
    }

    /// 获取二进制表示 (3 bits)
    ///
    /// 从下到上：初爻、二爻、三爻
    pub fn binary(&self) -> u8 {
        match self {
            Bagua::Qian => 0b111, // 乾 ☰ 三阳爻
            Bagua::Dui => 0b011,  // 兑 ☱ 上阴下阳
            Bagua::Li => 0b101,   // 离 ☲ 中阴上下阳
            Bagua::Zhen => 0b001, // 震 ☳ 下阳上阴
            Bagua::Xun => 0b110,  // 巽 ☴ 下阴上阳
            Bagua::Kan => 0b010,  // 坎 ☵ 中阳上下阴
            Bagua::Gen => 0b100,  // 艮 ☶ 上阳下阴
            Bagua::Kun => 0b000,  // 坤 ☷ 三阴爻
        }
    }

    /// 从二进制创建八卦
    ///
    /// # 参数
    /// - `binary`: 3位二进制数
    pub fn from_binary(binary: u8) -> Self {
        match binary & 0b111 {
            0b111 => Bagua::Qian,
            0b011 => Bagua::Dui,
            0b101 => Bagua::Li,
            0b001 => Bagua::Zhen,
            0b110 => Bagua::Xun,
            0b010 => Bagua::Kan,
            0b100 => Bagua::Gen,
            _ => Bagua::Kun,
        }
    }

    /// 获取五行属性
    pub fn wuxing(&self) -> WuXing {
        match self {
            Bagua::Qian | Bagua::Dui => WuXing::Jin,  // 乾兑属金
            Bagua::Zhen | Bagua::Xun => WuXing::Mu,   // 震巽属木
            Bagua::Kan => WuXing::Shui,               // 坎属水
            Bagua::Li => WuXing::Huo,                 // 离属火
            Bagua::Gen | Bagua::Kun => WuXing::Tu,    // 艮坤属土
        }
    }

    /// 获取先天八卦数 (1-8)
    pub fn number(&self) -> u8 {
        match self {
            Bagua::Qian => 1,
            Bagua::Dui => 2,
            Bagua::Li => 3,
            Bagua::Zhen => 4,
            Bagua::Xun => 5,
            Bagua::Kan => 6,
            Bagua::Gen => 7,
            Bagua::Kun => 8,
        }
    }
}

/// 五行枚举
///
/// 五行相生：金生水、水生木、木生火、火生土、土生金
/// 五行相克：金克木、木克土、土克水、水克火、火克金
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum WuXing {
    /// 金 - 西方，秋季，白色
    Jin = 0,
    /// 木 - 东方，春季，青色
    Mu = 1,
    /// 水 - 北方，冬季，黑色
    Shui = 2,
    /// 火 - 南方，夏季，红色
    Huo = 3,
    /// 土 - 中央，四季末，黄色
    #[default]
    Tu = 4,
}

impl WuXing {
    /// 判断此五行是否生另一五行
    ///
    /// 相生关系：金→水→木→火→土→金
    pub fn generates(&self, other: &WuXing) -> bool {
        matches!(
            (self, other),
            (WuXing::Jin, WuXing::Shui) |
            (WuXing::Shui, WuXing::Mu) |
            (WuXing::Mu, WuXing::Huo) |
            (WuXing::Huo, WuXing::Tu) |
            (WuXing::Tu, WuXing::Jin)
        )
    }

    /// 判断此五行是否克另一五行
    ///
    /// 相克关系：金→木→土→水→火→金
    pub fn conquers(&self, other: &WuXing) -> bool {
        matches!(
            (self, other),
            (WuXing::Jin, WuXing::Mu) |
            (WuXing::Mu, WuXing::Tu) |
            (WuXing::Tu, WuXing::Shui) |
            (WuXing::Shui, WuXing::Huo) |
            (WuXing::Huo, WuXing::Jin)
        )
    }

    /// 获取生我的五行
    ///
    /// 用于应期推算：体卦旺时应期在生体之五行
    pub fn generated_by(&self) -> WuXing {
        match self {
            WuXing::Jin => WuXing::Tu,   // 土生金
            WuXing::Mu => WuXing::Shui,  // 水生木
            WuXing::Shui => WuXing::Jin, // 金生水
            WuXing::Huo => WuXing::Mu,   // 木生火
            WuXing::Tu => WuXing::Huo,   // 火生土
        }
    }

    /// 获取我生的五行
    ///
    /// 用于应期推算：体卦休囚时应期在体所生之五行
    pub fn generates_to(&self) -> WuXing {
        match self {
            WuXing::Jin => WuXing::Shui, // 金生水
            WuXing::Mu => WuXing::Huo,   // 木生火
            WuXing::Shui => WuXing::Mu,  // 水生木
            WuXing::Huo => WuXing::Tu,   // 火生土
            WuXing::Tu => WuXing::Jin,   // 土生金
        }
    }

    /// 获取克我的五行
    ///
    /// 用于判断忌神
    pub fn conquered_by(&self) -> WuXing {
        match self {
            WuXing::Jin => WuXing::Huo,  // 火克金
            WuXing::Mu => WuXing::Jin,   // 金克木
            WuXing::Shui => WuXing::Tu,  // 土克水
            WuXing::Huo => WuXing::Shui, // 水克火
            WuXing::Tu => WuXing::Mu,    // 木克土
        }
    }

    /// 获取我克的五行
    pub fn conquers_to(&self) -> WuXing {
        match self {
            WuXing::Jin => WuXing::Mu,   // 金克木
            WuXing::Mu => WuXing::Tu,    // 木克土
            WuXing::Shui => WuXing::Huo, // 水克火
            WuXing::Huo => WuXing::Jin,  // 火克金
            WuXing::Tu => WuXing::Shui,  // 土克水
        }
    }

    /// 获取五行对应的先天卦数
    ///
    /// 用于应期推算
    /// 金：乾1、兑2
    /// 木：震4、巽5
    /// 水：坎6
    /// 火：离3
    /// 土：艮7、坤8
    pub fn gua_numbers(&self) -> (u8, Option<u8>) {
        match self {
            WuXing::Jin => (1, Some(2)),   // 乾1、兑2
            WuXing::Mu => (4, Some(5)),    // 震4、巽5
            WuXing::Shui => (6, None),     // 坎6
            WuXing::Huo => (3, None),      // 离3
            WuXing::Tu => (7, Some(8)),    // 艮7、坤8
        }
    }

    /// 从卦数获取五行
    /// 
    /// 乾1、兑2属金，离3属火，震4、巽5属木，坎6属水，艮7、坤8属土
    pub fn from_gua_num(num: u8) -> Self {
        match num {
            1 | 2 => WuXing::Jin,  // 乾、兑属金
            3 => WuXing::Huo,      // 离属火
            4 | 5 => WuXing::Mu,   // 震、巽属木
            6 => WuXing::Shui,     // 坎属水
            7 | 8 => WuXing::Tu,   // 艮、坤属土
            _ => WuXing::Tu,       // 默认土
        }
    }
}

/// 季节枚举
///
/// 用于判断五行旺衰
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum Season {
    /// 春季（正月、二月、三月）- 木旺
    #[default]
    Spring = 0,
    /// 夏季（四月、五月、六月）- 火旺
    Summer = 1,
    /// 秋季（七月、八月、九月）- 金旺
    Autumn = 2,
    /// 冬季（十月、十一月、十二月）- 水旺
    Winter = 3,
}

impl Season {
    /// 从农历月份获取季节
    ///
    /// # 参数
    /// - `lunar_month`: 农历月份（1-12）
    pub fn from_lunar_month(lunar_month: u8) -> Self {
        match lunar_month {
            1 | 2 | 3 => Season::Spring,   // 正、二、三月
            4 | 5 | 6 => Season::Summer,   // 四、五、六月
            7 | 8 | 9 => Season::Autumn,   // 七、八、九月
            10 | 11 | 12 => Season::Winter, // 十、十一、十二月
            _ => Season::Spring,
        }
    }

    /// 获取当令五行（旺）
    pub fn wang_wuxing(&self) -> WuXing {
        match self {
            Season::Spring => WuXing::Mu,   // 春木旺
            Season::Summer => WuXing::Huo,  // 夏火旺
            Season::Autumn => WuXing::Jin,  // 秋金旺
            Season::Winter => WuXing::Shui, // 冬水旺
        }
    }
}

/// 五行旺衰状态枚举
///
/// 梅花易数中五行在不同季节的状态：
/// - 旺：当令，最强
/// - 相：被当令五行所生，次强
/// - 休：生当令五行，力弱
/// - 囚：克当令五行，受制
/// - 死：被当令五行所克，最弱
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum WangShuai {
    /// 旺 - 当令，最强（如春天的木）
    Wang = 4,
    /// 相 - 被当令所生，次强（如春天的火，木生火）
    Xiang = 3,
    /// 休 - 生当令五行，休息（如春天的水，水生木）
    #[default]
    Xiu = 2,
    /// 囚 - 克当令五行，受制（如春天的金，金克木但反被木势所制）
    Qiu = 1,
    /// 死 - 被当令所克，最弱（如春天的土，木克土）
    Si = 0,
}

impl WangShuai {
    /// 计算五行在指定季节的旺衰状态
    ///
    /// # 算法
    /// - 旺：当令五行
    /// - 相：被当令所生（当令生我）
    /// - 休：生当令五行（我生当令）
    /// - 囚：克当令五行（我克当令）
    /// - 死：被当令所克（当令克我）
    ///
    /// # 参数
    /// - `wuxing`: 要判断的五行
    /// - `season`: 当前季节
    pub fn calculate(wuxing: &WuXing, season: &Season) -> Self {
        let wang = season.wang_wuxing(); // 当令五行

        if *wuxing == wang {
            // 当令五行为旺
            WangShuai::Wang
        } else if wang.generates(wuxing) {
            // 当令生我为相
            WangShuai::Xiang
        } else if wuxing.generates(&wang) {
            // 我生当令为休
            WangShuai::Xiu
        } else if wuxing.conquers(&wang) {
            // 我克当令为囚
            WangShuai::Qiu
        } else if wang.conquers(wuxing) {
            // 当令克我为死
            WangShuai::Si
        } else {
            WangShuai::Xiu // fallback
        }
    }

    /// 获取旺衰等级（0-4，4最旺）
    pub fn level(&self) -> u8 {
        *self as u8
    }

    /// 判断是否为旺相状态（有力）
    pub fn is_strong(&self) -> bool {
        matches!(self, WangShuai::Wang | WangShuai::Xiang)
    }

    /// 判断是否为休囚死状态（无力）
    pub fn is_weak(&self) -> bool {
        matches!(self, WangShuai::Xiu | WangShuai::Qiu | WangShuai::Si)
    }
}

/// 应期类型枚举
///
/// 梅花易数应期推算的时间单位
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum YingQiType {
    /// 年应期
    Year = 0,
    /// 月应期
    Month = 1,
    /// 日应期
    #[default]
    Day = 2,
    /// 时应期
    Hour = 3,
}

/// 应期推算结果
///
/// 包含多种可能的应期时间点
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct YingQiResult {
    /// 体卦五行
    pub ti_wuxing: WuXing,
    /// 用卦五行
    pub yong_wuxing: WuXing,
    /// 体卦旺衰状态
    pub ti_wangshuai: WangShuai,
    /// 生体五行（喜神）
    pub sheng_ti_wuxing: WuXing,
    /// 克体五行（忌神）
    pub ke_ti_wuxing: WuXing,
    /// 体卦卦数（用于应期）
    pub ti_gua_num: u8,
    /// 用卦卦数
    pub yong_gua_num: u8,
    /// 主要应期数（基于体用卦数）
    pub primary_num: u8,
    /// 次要应期数（基于五行卦数）
    pub secondary_nums: [u8; 2],
    /// 应期分析文本
    pub analysis: BoundedVec<u8, ConstU32<512>>,
}

impl Default for YingQiResult {
    fn default() -> Self {
        Self {
            ti_wuxing: WuXing::default(),
            yong_wuxing: WuXing::default(),
            ti_wangshuai: WangShuai::default(),
            sheng_ti_wuxing: WuXing::default(),
            ke_ti_wuxing: WuXing::default(),
            ti_gua_num: 1,
            yong_gua_num: 1,
            primary_num: 1,
            secondary_nums: [1, 1],
            analysis: BoundedVec::default(),
        }
    }
}

/// 体用关系枚举
///
/// 梅花易数核心概念：体卦代表自身，用卦代表所占之事
/// 吉凶判断：用生体大吉、比和次吉、体克用中平、体生用小凶、用克体大凶
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum TiYongRelation {
    /// 比和 - 体用五行相同，次吉
    #[default]
    BiHe = 0,
    /// 用生体 - 用卦五行生体卦五行，大吉
    YongShengTi = 1,
    /// 体生用 - 体卦五行生用卦五行，小凶（泄气）
    TiShengYong = 2,
    /// 用克体 - 用卦五行克体卦五行，大凶
    YongKeTi = 3,
    /// 体克用 - 体卦五行克用卦五行，中平（需耗力）
    TiKeYong = 4,
}

impl TiYongRelation {
    /// 计算体用关系
    ///
    /// # 参数
    /// - `ti`: 体卦五行
    /// - `yong`: 用卦五行
    pub fn calculate(ti: &WuXing, yong: &WuXing) -> Self {
        if ti == yong {
            TiYongRelation::BiHe
        } else if yong.generates(ti) {
            TiYongRelation::YongShengTi
        } else if ti.generates(yong) {
            TiYongRelation::TiShengYong
        } else if yong.conquers(ti) {
            TiYongRelation::YongKeTi
        } else if ti.conquers(yong) {
            TiYongRelation::TiKeYong
        } else {
            TiYongRelation::BiHe // fallback
        }
    }

    /// 获取吉凶等级 (0-4, 4最吉)
    pub fn fortune_level(&self) -> u8 {
        match self {
            TiYongRelation::YongShengTi => 4, // 大吉
            TiYongRelation::BiHe => 3,        // 次吉
            TiYongRelation::TiKeYong => 2,    // 中平
            TiYongRelation::TiShengYong => 1, // 小凶
            TiYongRelation::YongKeTi => 0,    // 大凶
        }
    }
}

/// 起卦方式枚举
///
/// 梅花易数支持多种起卦方式，每种方式都有其独特的计算规则
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum DivinationMethod {
    /// 农历时间起卦 - 使用农历年月日时起卦（传统方式）
    /// 上卦：(年支数+月数+日数) % 8
    /// 下卦：(年支数+月数+日数+时支数) % 8
    /// 动爻：(年支数+月数+日数+时支数) % 6
    #[default]
    LunarDateTime = 0,
    /// 公历时间起卦 - 使用公历年月日时起卦（现代简化）
    /// 上卦：(年份后两位+月+日) % 8
    /// 下卦：(年份后两位+月+日+小时) % 8
    /// 动爻：(年份后两位+月+日+小时) % 6
    GregorianDateTime = 1,
    /// 双数起卦 - 使用两个数字起卦
    /// 上卦：第一个数 % 8
    /// 下卦：第二个数 % 8
    /// 动爻：(两数之和+时支数) % 6
    TwoNumbers = 2,
    /// 随机起卦 - 使用链上随机数
    Random = 3,
    /// 手动指定 - 直接指定上卦、下卦、动爻
    Manual = 4,
    /// 单数起卦 - 使用一个多位数字起卦
    /// 将数字拆分为前后两半
    /// 上卦：前半段数字之和 % 8
    /// 下卦：后半段数字之和 % 8
    /// 动爻：(前半+后半+时支数) % 6
    SingleNumber = 5,
    /// 链摇起卦 - 用户交互式摇卦
    /// 前端生成6个爻（阴/阳），链上验证并存储
    /// 动爻：最后一次摇卦时间戳 % 6 + 1
    ChainShake = 6,
}

/// 吉凶判断结果
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum Fortune {
    /// 大吉 - 用生体
    DaJi = 4,
    /// 小吉 - 比和
    XiaoJi = 3,
    /// 平 - 体克用
    #[default]
    Ping = 2,
    /// 小凶 - 体生用
    XiaoXiong = 1,
    /// 大凶 - 用克体
    DaXiong = 0,
}

impl Fortune {
    /// 从体用关系综合评分计算吉凶
    ///
    /// # 参数
    /// - `ben_relation`: 本卦体用关系
    /// - `bian_relation`: 变卦体用关系（可选）
    pub fn from_relations(
        ben_relation: &TiYongRelation,
        bian_relation: Option<&TiYongRelation>,
    ) -> Self {
        let ben_score = ben_relation.fortune_level();
        let bian_score = bian_relation.map(|r| r.fortune_level()).unwrap_or(2);

        // 综合评分 = 本卦评分 * 0.6 + 变卦评分 * 0.4
        // 简化为整数运算：(本卦 * 3 + 变卦 * 2) / 5
        let total = (ben_score as u16 * 3 + bian_score as u16 * 2) / 5;

        match total {
            4 => Fortune::DaJi,
            3 => Fortune::XiaoJi,
            2 => Fortune::Ping,
            1 => Fortune::XiaoXiong,
            _ => Fortune::DaXiong,
        }
    }
}

/// 单卦结构（优化版）
///
/// 存储优化说明：
/// - 仅存储 Bagua 枚举值（1 byte）
/// - 所有其他属性（五行、二进制、名称等）均可从 Bagua 推导
/// - 一次完整排盘可节省约 200+ bytes
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct SingleGua {
    /// 八卦类型
    pub bagua: Bagua,
}

impl SingleGua {
    /// 从八卦枚举创建单卦
    pub fn new(bagua: Bagua) -> Self {
        Self { bagua }
    }

    /// 从先天八卦数创建单卦（1-8，0和8都对应坤卦）
    pub fn from_num(num: u8) -> Self {
        Self {
            bagua: Bagua::from_num(num),
        }
    }

    /// 从二进制创建单卦
    pub fn from_binary(binary: u8) -> Self {
        Self {
            bagua: Bagua::from_binary(binary),
        }
    }

    /// 获取二进制表示
    pub fn binary(&self) -> u8 {
        self.bagua.binary()
    }

    /// 获取五行属性
    pub fn wuxing(&self) -> WuXing {
        self.bagua.wuxing()
    }

    /// 获取先天八卦数
    pub fn number(&self) -> u8 {
        self.bagua.number()
    }
}

/// 六十四卦结构
///
/// 表示一个完整的六爻卦象，包含上卦、下卦、动爻等核心信息
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(T))]
pub struct Hexagram<AccountId, BlockNumber> {
    /// 卦象唯一ID
    pub id: u64,
    /// 占卜者账户
    pub diviner: AccountId,
    /// 上卦（外卦）
    pub shang_gua: SingleGua,
    /// 下卦（内卦）
    pub xia_gua: SingleGua,
    /// 动爻位置 (1-6)，1为初爻，6为上爻
    pub dong_yao: u8,
    /// 体卦位置：true=上卦为体，false=下卦为体
    /// 规则：动爻在哪卦，哪卦为用，另一卦为体
    pub ti_is_shang: bool,
    /// 占卜问题的哈希值（隐私保护，不存储原文）
    pub question_hash: [u8; 32],
    /// 起卦方式
    pub method: DivinationMethod,
    /// 起卦时的区块号
    pub block_number: BlockNumber,
    /// 起卦时间戳（Unix时间戳，秒）
    pub timestamp: u64,
    /// AI 解读结果的 IPFS CID（可选）
    pub interpretation_cid: Option<BoundedVec<u8, ConstU32<64>>>,
    /// 是否公开（公开的卦象可被其他人查看）
    pub is_public: bool,

    // ========== 占卜者基础信息（公开层） ==========
    /// 性别（0: 未指定, 1: 男, 2: 女）
    /// 用于解卦分析，某些流派需要根据性别判断体用
    pub gender: u8,
    /// 出生年份（可选）
    /// 用于计算本命卦、生肖、应期推算等
    pub birth_year: Option<u16>,
}

impl<AccountId, BlockNumber> Hexagram<AccountId, BlockNumber> {
    /// 获取六爻的完整二进制表示 (6 bits)
    ///
    /// 从下到上依次为：初爻、二爻、三爻、四爻、五爻、上爻
    /// 上卦占高3位，下卦占低3位
    pub fn full_binary(&self) -> u8 {
        (self.shang_gua.binary() << 3) | self.xia_gua.binary()
    }

    /// 获取六十四卦索引 (0-63)
    ///
    /// 用于查表获取卦名、卦辞等
    /// 索引计算规则：
    /// - 内部索引使用 0-7 对应：坤(0)、乾(1)、兑(2)、离(3)、震(4)、巽(5)、坎(6)、艮(7)
    /// - 先天八卦数 8(坤) 转为索引 0，其他保持不变
    /// - 最终索引 = 上卦索引 * 8 + 下卦索引
    pub fn hexagram_index(&self) -> u8 {
        let shang_num = self.shang_gua.number();
        let xia_num = self.xia_gua.number();
        // 先天数转内部索引：8(坤)→0，其他1-7保持不变
        let shang_idx = if shang_num == 8 { 0 } else { shang_num };
        let xia_idx = if xia_num == 8 { 0 } else { xia_num };
        // 直接计算索引，范围 0-63
        shang_idx * 8 + xia_idx
    }

    /// 获取体卦
    pub fn ti_gua(&self) -> &SingleGua {
        if self.ti_is_shang {
            &self.shang_gua
        } else {
            &self.xia_gua
        }
    }

    /// 获取用卦
    pub fn yong_gua(&self) -> &SingleGua {
        if self.ti_is_shang {
            &self.xia_gua
        } else {
            &self.shang_gua
        }
    }

    /// 计算本卦的体用关系
    pub fn calc_relation(&self) -> TiYongRelation {
        TiYongRelation::calculate(&self.ti_gua().wuxing(), &self.yong_gua().wuxing())
    }
}

/// 完整卦象结构（含本卦、变卦、互卦）
///
/// 包含梅花易数排盘的完整信息
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(T))]
pub struct FullDivination<AccountId, BlockNumber> {
    /// 本卦 - 原始起卦结果
    pub ben_gua: Hexagram<AccountId, BlockNumber>,
    /// 变卦 - 动爻变化后的卦象 (上卦, 下卦)
    pub bian_gua: (SingleGua, SingleGua),
    /// 互卦 - 本卦234爻为下卦、345爻为上卦 (上卦, 下卦)
    pub hu_gua: (SingleGua, SingleGua),
    /// 本卦体用关系
    pub ben_gua_relation: TiYongRelation,
    /// 变卦体用关系
    pub bian_gua_relation: TiYongRelation,
    /// 综合吉凶判断
    pub fortune: Fortune,
}

impl<AccountId: Clone, BlockNumber: Clone> FullDivination<AccountId, BlockNumber> {
    /// 从本卦创建完整卦象
    ///
    /// 自动计算变卦、互卦、体用关系和吉凶
    pub fn from_hexagram(ben_gua: Hexagram<AccountId, BlockNumber>) -> Self {
        // 计算变卦
        let bian_gua = Self::calc_bian_gua(&ben_gua.shang_gua, &ben_gua.xia_gua, ben_gua.dong_yao);

        // 计算互卦
        let hu_gua = Self::calc_hu_gua(&ben_gua.shang_gua, &ben_gua.xia_gua);

        // 计算体用关系
        let ben_gua_relation = ben_gua.calc_relation();

        // 变卦的体用关系（体卦位置不变）
        let bian_ti_wuxing = if ben_gua.ti_is_shang {
            bian_gua.0.wuxing()
        } else {
            bian_gua.1.wuxing()
        };
        let bian_yong_wuxing = if ben_gua.ti_is_shang {
            bian_gua.1.wuxing()
        } else {
            bian_gua.0.wuxing()
        };
        let bian_gua_relation = TiYongRelation::calculate(&bian_ti_wuxing, &bian_yong_wuxing);

        // 综合吉凶判断
        let fortune = Fortune::from_relations(&ben_gua_relation, Some(&bian_gua_relation));

        Self {
            ben_gua,
            bian_gua,
            hu_gua,
            ben_gua_relation,
            bian_gua_relation,
            fortune,
        }
    }

    /// 计算变卦
    ///
    /// 变卦规则：动爻阴阳互变
    /// - 动爻1-3在下卦，下卦对应爻变
    /// - 动爻4-6在上卦，上卦对应爻变
    fn calc_bian_gua(
        shang_gua: &SingleGua,
        xia_gua: &SingleGua,
        dong_yao: u8,
    ) -> (SingleGua, SingleGua) {
        // 组合6爻二进制：上卦占高3位，下卦占低3位
        let full_binary = (shang_gua.binary() << 3) | xia_gua.binary();

        // 翻转动爻位（dong_yao: 1-6 对应 bit 0-5）
        let bit_position = dong_yao - 1;
        let flipped = full_binary ^ (1 << bit_position);

        // 分离上下卦
        let new_shang_binary = (flipped >> 3) & 0b111;
        let new_xia_binary = flipped & 0b111;

        (
            SingleGua::from_binary(new_shang_binary),
            SingleGua::from_binary(new_xia_binary),
        )
    }

    /// 计算互卦
    ///
    /// 互卦规则：
    /// - 互卦上卦：本卦第3、4、5爻
    /// - 互卦下卦：本卦第2、3、4爻
    fn calc_hu_gua(shang_gua: &SingleGua, xia_gua: &SingleGua) -> (SingleGua, SingleGua) {
        // 组合6爻：bits 5-3 为上卦，bits 2-0 为下卦
        let full_binary = (shang_gua.binary() << 3) | xia_gua.binary();

        // 互卦上卦：取本卦第5、4、3爻 (bits 4, 3, 2)
        let hu_shang = (full_binary >> 2) & 0b111;

        // 互卦下卦：取本卦第4、3、2爻 (bits 3, 2, 1)
        let hu_xia = (full_binary >> 1) & 0b111;

        (
            SingleGua::from_binary(hu_shang),
            SingleGua::from_binary(hu_xia),
        )
    }
}

/// 起卦参数结构
///
/// 用于传递起卦所需的参数
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct DivinationParams {
    /// 起卦方式
    pub method: DivinationMethod,
    /// 第一个数字（用于双数起卦）
    pub num1: Option<u16>,
    /// 第二个数字（用于双数起卦）
    pub num2: Option<u16>,
    /// 手动指定的上卦数（1-8）
    pub manual_shang: Option<u8>,
    /// 手动指定的下卦数（1-8）
    pub manual_xia: Option<u8>,
    /// 手动指定的动爻（1-6）
    pub manual_dong_yao: Option<u8>,
    /// 问题哈希
    pub question_hash: [u8; 32],
    /// 是否公开
    pub is_public: bool,
}

/// 卦象详细信息结构（用于 API 查询返回）
///
/// 包含卦象的完整文本信息，适合前端展示
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct HexagramDetail {
    /// 六十四卦名称（如"乾为天"）
    pub name: BoundedVec<u8, ConstU32<32>>,
    /// 上卦名称（如"乾"）
    pub shang_gua_name: BoundedVec<u8, ConstU32<16>>,
    /// 下卦名称（如"乾"）
    pub xia_gua_name: BoundedVec<u8, ConstU32<16>>,
    /// 上卦符号（如"☰"）
    pub shang_gua_symbol: BoundedVec<u8, ConstU32<8>>,
    /// 下卦符号（如"☰"）
    pub xia_gua_symbol: BoundedVec<u8, ConstU32<8>>,
    /// 上卦五行（如"金"）
    pub shang_gua_wuxing: BoundedVec<u8, ConstU32<8>>,
    /// 下卦五行（如"金"）
    pub xia_gua_wuxing: BoundedVec<u8, ConstU32<8>>,
    /// 卦辞
    pub guaci: BoundedVec<u8, ConstU32<256>>,
    /// 动爻名称（如"初爻"）
    pub dong_yao_name: BoundedVec<u8, ConstU32<16>>,
    /// 动爻爻名（如"初九"、"六二"，根据阴阳爻确定）
    pub dong_yao_ming: BoundedVec<u8, ConstU32<16>>,
    /// 动爻爻辞
    pub dong_yao_ci: BoundedVec<u8, ConstU32<256>>,
    /// 体用关系名称（如"用生体"）
    pub tiyong_name: BoundedVec<u8, ConstU32<16>>,
    /// 吉凶名称（如"大吉"）
    pub fortune_name: BoundedVec<u8, ConstU32<16>>,
}

impl HexagramDetail {
    /// 从卦象创建详细信息
    ///
    /// # 参数
    /// - `shang_gua`: 上卦
    /// - `xia_gua`: 下卦
    /// - `dong_yao`: 动爻位置（1-6）
    /// - `relation`: 体用关系
    /// - `fortune`: 吉凶
    pub fn from_hexagram(
        shang_gua: &SingleGua,
        xia_gua: &SingleGua,
        dong_yao: u8,
        relation: &TiYongRelation,
        fortune: &Fortune,
    ) -> Self {
        use crate::constants::*;

        let shang_num = shang_gua.number();
        let xia_num = xia_gua.number();

        // 获取索引（0-7）
        let shang_idx = if shang_num == 8 { 0 } else { shang_num };
        let xia_idx = if xia_num == 8 { 0 } else { xia_num };

        // 获取各项文本
        let name = get_hexagram_name(shang_idx, xia_idx);
        let shang_name = get_bagua_name(shang_num);
        let xia_name = get_bagua_name(xia_num);
        let shang_symbol = get_bagua_symbol(shang_num);
        let xia_symbol = get_bagua_symbol(xia_num);
        let guaci = get_hexagram_guaci(shang_idx, xia_idx);
        let yao_name = get_yao_name(dong_yao);

        // 获取爻名和爻辞
        let yao_ming = get_yao_ming(shang_gua.binary(), xia_gua.binary(), dong_yao);
        let yao_ci = get_yaoci(shang_idx, xia_idx, dong_yao);

        // 五行名称
        let shang_wuxing = WUXING_NAMES[shang_gua.wuxing() as usize];
        let xia_wuxing = WUXING_NAMES[xia_gua.wuxing() as usize];

        // 体用关系名称
        let tiyong = TIYONG_NAMES[*relation as usize];

        // 吉凶名称
        let fortune_str = FORTUNE_NAMES[*fortune as usize];

        // 转换为 BoundedVec
        Self {
            name: BoundedVec::try_from(name.as_bytes().to_vec()).unwrap_or_default(),
            shang_gua_name: BoundedVec::try_from(shang_name.as_bytes().to_vec()).unwrap_or_default(),
            xia_gua_name: BoundedVec::try_from(xia_name.as_bytes().to_vec()).unwrap_or_default(),
            shang_gua_symbol: BoundedVec::try_from(shang_symbol.as_bytes().to_vec()).unwrap_or_default(),
            xia_gua_symbol: BoundedVec::try_from(xia_symbol.as_bytes().to_vec()).unwrap_or_default(),
            shang_gua_wuxing: BoundedVec::try_from(shang_wuxing.as_bytes().to_vec()).unwrap_or_default(),
            xia_gua_wuxing: BoundedVec::try_from(xia_wuxing.as_bytes().to_vec()).unwrap_or_default(),
            guaci: BoundedVec::try_from(guaci.as_bytes().to_vec()).unwrap_or_default(),
            dong_yao_name: BoundedVec::try_from(yao_name.as_bytes().to_vec()).unwrap_or_default(),
            dong_yao_ming: BoundedVec::try_from(yao_ming.as_bytes().to_vec()).unwrap_or_default(),
            dong_yao_ci: BoundedVec::try_from(yao_ci.as_bytes().to_vec()).unwrap_or_default(),
            tiyong_name: BoundedVec::try_from(tiyong.as_bytes().to_vec()).unwrap_or_default(),
            fortune_name: BoundedVec::try_from(fortune_str.as_bytes().to_vec()).unwrap_or_default(),
        }
    }
}

/// 完整排盘详细信息结构
///
/// 包含本卦、变卦、互卦、错卦、综卦、伏卦的详细信息
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct FullDivinationDetail {
    /// 本卦详细信息
    pub ben_gua: HexagramDetail,
    /// 变卦详细信息
    pub bian_gua: HexagramDetail,
    /// 互卦详细信息
    pub hu_gua: HexagramDetail,
    /// 错卦详细信息
    pub cuo_gua: HexagramDetail,
    /// 综卦详细信息
    pub zong_gua: HexagramDetail,
    /// 伏卦详细信息（新增）
    pub fu_gua: HexagramDetail,
    /// 体用关系详细解读（新增）
    pub tiyong_interpretation: BoundedVec<u8, ConstU32<256>>,
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
///   "name": "张三",           // 姓名
///   "birthDate": "1990-06-15", // 完整出生日期
///   "birthHour": 14,          // 出生时辰（0-23）
///   "notes": "备注信息"        // 备注
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
    fn test_bagua_from_num() {
        assert_eq!(Bagua::from_num(1), Bagua::Qian);
        assert_eq!(Bagua::from_num(2), Bagua::Dui);
        assert_eq!(Bagua::from_num(8), Bagua::Kun);
        assert_eq!(Bagua::from_num(0), Bagua::Kun);
        assert_eq!(Bagua::from_num(9), Bagua::Qian); // 9 % 8 = 1
    }

    #[test]
    fn test_bagua_binary() {
        assert_eq!(Bagua::Qian.binary(), 0b111);
        assert_eq!(Bagua::Kun.binary(), 0b000);
        assert_eq!(Bagua::Kan.binary(), 0b010);
        assert_eq!(Bagua::Li.binary(), 0b101);
    }

    #[test]
    fn test_bagua_wuxing() {
        assert_eq!(Bagua::Qian.wuxing(), WuXing::Jin);
        assert_eq!(Bagua::Dui.wuxing(), WuXing::Jin);
        assert_eq!(Bagua::Zhen.wuxing(), WuXing::Mu);
        assert_eq!(Bagua::Kan.wuxing(), WuXing::Shui);
        assert_eq!(Bagua::Li.wuxing(), WuXing::Huo);
        assert_eq!(Bagua::Kun.wuxing(), WuXing::Tu);
    }

    #[test]
    fn test_wuxing_relations() {
        // 金生水
        assert!(WuXing::Jin.generates(&WuXing::Shui));
        // 金克木
        assert!(WuXing::Jin.conquers(&WuXing::Mu));
        // 水生木
        assert!(WuXing::Shui.generates(&WuXing::Mu));
    }

    #[test]
    fn test_tiyong_relation() {
        // 用生体 - 大吉
        assert_eq!(
            TiYongRelation::calculate(&WuXing::Shui, &WuXing::Jin),
            TiYongRelation::YongShengTi
        );
        // 比和 - 次吉
        assert_eq!(
            TiYongRelation::calculate(&WuXing::Jin, &WuXing::Jin),
            TiYongRelation::BiHe
        );
        // 用克体 - 大凶
        assert_eq!(
            TiYongRelation::calculate(&WuXing::Mu, &WuXing::Jin),
            TiYongRelation::YongKeTi
        );
    }
}
