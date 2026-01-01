//! # 紫微斗数类型定义
//!
//! 本模块定义紫微斗数排盘系统的所有核心类型。

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

// ============================================================================
// 天干地支基础类型
// ============================================================================

/// 十天干
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum TianGan {
    #[default]
    Jia = 0,   // 甲
    Yi = 1,    // 乙
    Bing = 2,  // 丙
    Ding = 3,  // 丁
    Wu = 4,    // 戊
    Ji = 5,    // 己
    Geng = 6,  // 庚
    Xin = 7,   // 辛
    Ren = 8,   // 壬
    Gui = 9,   // 癸
}

impl TianGan {
    /// 获取天干名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Jia => "甲",
            Self::Yi => "乙",
            Self::Bing => "丙",
            Self::Ding => "丁",
            Self::Wu => "戊",
            Self::Ji => "己",
            Self::Geng => "庚",
            Self::Xin => "辛",
            Self::Ren => "壬",
            Self::Gui => "癸",
        }
    }

    /// 从数字获取天干
    pub fn from_index(index: u8) -> Self {
        match index % 10 {
            0 => Self::Jia,
            1 => Self::Yi,
            2 => Self::Bing,
            3 => Self::Ding,
            4 => Self::Wu,
            5 => Self::Ji,
            6 => Self::Geng,
            7 => Self::Xin,
            8 => Self::Ren,
            _ => Self::Gui,
        }
    }

    /// 获取天干索引
    pub fn index(&self) -> u8 {
        *self as u8
    }

    /// 获取天干阴阳属性
    pub fn yin_yang(&self) -> YinYang {
        match self {
            Self::Jia | Self::Bing | Self::Wu | Self::Geng | Self::Ren => YinYang::Yang,
            _ => YinYang::Yin,
        }
    }

    /// 获取天干五行属性
    pub fn wu_xing(&self) -> WuXing {
        match self {
            Self::Jia | Self::Yi => WuXing::Wood,
            Self::Bing | Self::Ding => WuXing::Fire,
            Self::Wu | Self::Ji => WuXing::Earth,
            Self::Geng | Self::Xin => WuXing::Metal,
            Self::Ren | Self::Gui => WuXing::Water,
        }
    }
}

/// 十二地支
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum DiZhi {
    #[default]
    Zi = 0,   // 子
    Chou = 1, // 丑
    Yin = 2,  // 寅
    Mao = 3,  // 卯
    Chen = 4, // 辰
    Si = 5,   // 巳
    Wu = 6,   // 午
    Wei = 7,  // 未
    Shen = 8, // 申
    You = 9,  // 酉
    Xu = 10,  // 戌
    Hai = 11, // 亥
}

impl DiZhi {
    /// 获取地支名称
    pub fn name(&self) -> &'static str {
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

    /// 从数字获取地支
    pub fn from_index(index: u8) -> Self {
        match index % 12 {
            0 => Self::Zi,
            1 => Self::Chou,
            2 => Self::Yin,
            3 => Self::Mao,
            4 => Self::Chen,
            5 => Self::Si,
            6 => Self::Wu,
            7 => Self::Wei,
            8 => Self::Shen,
            9 => Self::You,
            10 => Self::Xu,
            _ => Self::Hai,
        }
    }

    /// 获取地支索引
    pub fn index(&self) -> u8 {
        *self as u8
    }

    /// 获取生肖
    pub fn sheng_xiao(&self) -> &'static str {
        match self {
            Self::Zi => "鼠",
            Self::Chou => "牛",
            Self::Yin => "虎",
            Self::Mao => "兔",
            Self::Chen => "龙",
            Self::Si => "蛇",
            Self::Wu => "马",
            Self::Wei => "羊",
            Self::Shen => "猴",
            Self::You => "鸡",
            Self::Xu => "狗",
            Self::Hai => "猪",
        }
    }
}

// ============================================================================
// 五行与阴阳
// ============================================================================

/// 五行
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum WuXing {
    #[default]
    Water = 0, // 水
    Wood = 1,  // 木
    Metal = 2, // 金
    Earth = 3, // 土
    Fire = 4,  // 火
}

impl WuXing {
    /// 获取五行名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Water => "水",
            Self::Wood => "木",
            Self::Metal => "金",
            Self::Earth => "土",
            Self::Fire => "火",
        }
    }

    /// 获取五行局数
    pub fn ju_shu(&self) -> u8 {
        match self {
            Self::Water => 2,
            Self::Wood => 3,
            Self::Metal => 4,
            Self::Earth => 5,
            Self::Fire => 6,
        }
    }
}

/// 阴阳
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum YinYang {
    #[default]
    Yang = 0, // 阳
    Yin = 1,  // 阴
}

impl YinYang {
    /// 获取阴阳名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Yang => "阳",
            Self::Yin => "阴",
        }
    }
}

// ============================================================================
// 十二宫位
// ============================================================================

/// 十二宫位
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum GongWei {
    #[default]
    MingGong = 0,    // 命宫
    FuMu = 1,        // 父母宫
    FuDe = 2,        // 福德宫
    TianZhai = 3,    // 田宅宫
    GuanLu = 4,      // 官禄宫
    JiaoYou = 5,     // 交友宫（仆役宫）
    QianYi = 6,      // 迁移宫
    JiE = 7,         // 疾厄宫
    CaiBo = 8,       // 财帛宫
    ZiNv = 9,        // 子女宫
    FuQi = 10,       // 夫妻宫
    XiongDi = 11,    // 兄弟宫
}

impl GongWei {
    /// 获取宫位名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::MingGong => "命宫",
            Self::FuMu => "父母宫",
            Self::FuDe => "福德宫",
            Self::TianZhai => "田宅宫",
            Self::GuanLu => "官禄宫",
            Self::JiaoYou => "交友宫",
            Self::QianYi => "迁移宫",
            Self::JiE => "疾厄宫",
            Self::CaiBo => "财帛宫",
            Self::ZiNv => "子女宫",
            Self::FuQi => "夫妻宫",
            Self::XiongDi => "兄弟宫",
        }
    }

    /// 从索引获取宫位
    pub fn from_index(index: u8) -> Self {
        match index % 12 {
            0 => Self::MingGong,
            1 => Self::FuMu,
            2 => Self::FuDe,
            3 => Self::TianZhai,
            4 => Self::GuanLu,
            5 => Self::JiaoYou,
            6 => Self::QianYi,
            7 => Self::JiE,
            8 => Self::CaiBo,
            9 => Self::ZiNv,
            10 => Self::FuQi,
            _ => Self::XiongDi,
        }
    }

    /// 获取宫位索引
    pub fn index(&self) -> u8 {
        *self as u8
    }
}

// ============================================================================
// 十四主星
// ============================================================================

/// 十四主星
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum ZhuXing {
    // 紫微星系（6颗）
    #[default]
    ZiWei = 0,    // 紫微星 - 帝王星
    TianJi = 1,   // 天机星 - 智谋星
    TaiYang = 2,  // 太阳星 - 官禄主
    WuQu = 3,     // 武曲星 - 财星
    TianTong = 4, // 天同星 - 福德主
    LianZhen = 5, // 廉贞星 - 次桃花星

    // 天府星系（8颗）
    TianFu = 6,   // 天府星 - 库星
    TaiYin = 7,   // 太阴星 - 桃花星
    TanLang = 8,  // 贪狼星 - 第一大桃花星
    JuMen = 9,    // 巨门星 - 口舌星
    TianXiang = 10, // 天相星 - 印星
    TianLiang = 11, // 天梁星 - 宗教星
    QiSha = 12,   // 七杀星 - 将星
    PoJun = 13,   // 破军星 - 先锋星
}

impl ZhuXing {
    /// 获取主星名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::ZiWei => "紫微",
            Self::TianJi => "天机",
            Self::TaiYang => "太阳",
            Self::WuQu => "武曲",
            Self::TianTong => "天同",
            Self::LianZhen => "廉贞",
            Self::TianFu => "天府",
            Self::TaiYin => "太阴",
            Self::TanLang => "贪狼",
            Self::JuMen => "巨门",
            Self::TianXiang => "天相",
            Self::TianLiang => "天梁",
            Self::QiSha => "七杀",
            Self::PoJun => "破军",
        }
    }

    /// 判断是否为紫微星系
    pub fn is_ziwei_series(&self) -> bool {
        matches!(
            self,
            Self::ZiWei | Self::TianJi | Self::TaiYang | Self::WuQu | Self::TianTong | Self::LianZhen
        )
    }

    /// 判断是否为天府星系
    pub fn is_tianfu_series(&self) -> bool {
        !self.is_ziwei_series()
    }
}

// ============================================================================
// 辅星与杂曜
// ============================================================================

/// 六吉星
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub enum LiuJiXing {
    WenChang = 0,  // 文昌
    WenQu = 1,     // 文曲
    ZuoFu = 2,     // 左辅
    YouBi = 3,     // 右弼
    TianKui = 4,   // 天魁
    TianYue = 5,   // 天钺
}

impl LiuJiXing {
    /// 获取六吉星名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::WenChang => "文昌",
            Self::WenQu => "文曲",
            Self::ZuoFu => "左辅",
            Self::YouBi => "右弼",
            Self::TianKui => "天魁",
            Self::TianYue => "天钺",
        }
    }
}

/// 六煞星
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub enum LiuShaXing {
    QingYang = 0,  // 擎羊
    TuoLuo = 1,    // 陀罗
    HuoXing = 2,   // 火星
    LingXing = 3,  // 铃星
    DiKong = 4,    // 地空
    DiJie = 5,     // 地劫
}

impl LiuShaXing {
    /// 获取六煞星名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::QingYang => "擎羊",
            Self::TuoLuo => "陀罗",
            Self::HuoXing => "火星",
            Self::LingXing => "铃星",
            Self::DiKong => "地空",
            Self::DiJie => "地劫",
        }
    }
}

/// 四化星
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub enum SiHua {
    HuaLu = 0,    // 化禄
    HuaQuan = 1,  // 化权
    HuaKe = 2,    // 化科
    HuaJi = 3,    // 化忌
}

impl SiHua {
    /// 获取四化名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::HuaLu => "化禄",
            Self::HuaQuan => "化权",
            Self::HuaKe => "化科",
            Self::HuaJi => "化忌",
        }
    }
}

// ============================================================================
// 四化星联合类型（支持主星和辅星）
// ============================================================================

/// 四化星联合类型
///
/// 四化飞星可以作用于主星（十四主星）或辅星（六吉星），
/// 此枚举统一表示可以参与四化的所有星曜。
///
/// 根据《紫微斗数全书》，各天干四化如下：
/// - 甲：廉贞化禄、破军化权、武曲化科、太阳化忌
/// - 乙：天机化禄、天梁化权、紫微化科、太阴化忌
/// - 丙：天同化禄、天机化权、**文昌**化科、廉贞化忌
/// - 丁：太阴化禄、天同化权、天机化科、巨门化忌
/// - 戊：贪狼化禄、太阴化权、**右弼**化科、天机化忌
/// - 己：武曲化禄、贪狼化权、天梁化科、**文曲**化忌
/// - 庚：太阳化禄、武曲化权、太阴化科、天同化忌
/// - 辛：巨门化禄、太阳化权、**文曲**化科、**文昌**化忌
/// - 壬：天梁化禄、紫微化权、**左辅**化科、武曲化忌
/// - 癸：破军化禄、巨门化权、太阴化科、贪狼化忌
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum SiHuaStar {
    // ===== 主星（十四主星） =====
    /// 紫微星
    #[default]
    ZiWei,
    /// 天机星
    TianJi,
    /// 太阳星
    TaiYang,
    /// 武曲星
    WuQu,
    /// 天同星
    TianTong,
    /// 廉贞星
    LianZhen,
    /// 天府星
    TianFu,
    /// 太阴星
    TaiYin,
    /// 贪狼星
    TanLang,
    /// 巨门星
    JuMen,
    /// 天相星
    TianXiang,
    /// 天梁星
    TianLiang,
    /// 七杀星
    QiSha,
    /// 破军星
    PoJun,

    // ===== 辅星（六吉星中参与四化的） =====
    /// 文昌星（丙化科、辛化忌）
    WenChang,
    /// 文曲星（己化忌、辛化科）
    WenQu,
    /// 左辅星（壬化科）
    ZuoFu,
    /// 右弼星（戊化科）
    YouBi,
}

impl SiHuaStar {
    /// 获取星曜名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::ZiWei => "紫微",
            Self::TianJi => "天机",
            Self::TaiYang => "太阳",
            Self::WuQu => "武曲",
            Self::TianTong => "天同",
            Self::LianZhen => "廉贞",
            Self::TianFu => "天府",
            Self::TaiYin => "太阴",
            Self::TanLang => "贪狼",
            Self::JuMen => "巨门",
            Self::TianXiang => "天相",
            Self::TianLiang => "天梁",
            Self::QiSha => "七杀",
            Self::PoJun => "破军",
            Self::WenChang => "文昌",
            Self::WenQu => "文曲",
            Self::ZuoFu => "左辅",
            Self::YouBi => "右弼",
        }
    }

    /// 判断是否为主星
    pub fn is_zhu_xing(&self) -> bool {
        matches!(
            self,
            Self::ZiWei | Self::TianJi | Self::TaiYang | Self::WuQu |
            Self::TianTong | Self::LianZhen | Self::TianFu | Self::TaiYin |
            Self::TanLang | Self::JuMen | Self::TianXiang | Self::TianLiang |
            Self::QiSha | Self::PoJun
        )
    }

    /// 判断是否为辅星
    pub fn is_fu_xing(&self) -> bool {
        matches!(self, Self::WenChang | Self::WenQu | Self::ZuoFu | Self::YouBi)
    }

    /// 从主星枚举转换
    pub fn from_zhu_xing(zhu_xing: ZhuXing) -> Self {
        match zhu_xing {
            ZhuXing::ZiWei => Self::ZiWei,
            ZhuXing::TianJi => Self::TianJi,
            ZhuXing::TaiYang => Self::TaiYang,
            ZhuXing::WuQu => Self::WuQu,
            ZhuXing::TianTong => Self::TianTong,
            ZhuXing::LianZhen => Self::LianZhen,
            ZhuXing::TianFu => Self::TianFu,
            ZhuXing::TaiYin => Self::TaiYin,
            ZhuXing::TanLang => Self::TanLang,
            ZhuXing::JuMen => Self::JuMen,
            ZhuXing::TianXiang => Self::TianXiang,
            ZhuXing::TianLiang => Self::TianLiang,
            ZhuXing::QiSha => Self::QiSha,
            ZhuXing::PoJun => Self::PoJun,
        }
    }

    /// 从六吉星枚举转换（仅支持参与四化的辅星）
    pub fn from_liu_ji_xing(liu_ji: LiuJiXing) -> Option<Self> {
        match liu_ji {
            LiuJiXing::WenChang => Some(Self::WenChang),
            LiuJiXing::WenQu => Some(Self::WenQu),
            LiuJiXing::ZuoFu => Some(Self::ZuoFu),
            LiuJiXing::YouBi => Some(Self::YouBi),
            _ => None, // 天魁、天钺不参与四化
        }
    }

    /// 尝试转换为主星枚举
    pub fn to_zhu_xing(&self) -> Option<ZhuXing> {
        match self {
            Self::ZiWei => Some(ZhuXing::ZiWei),
            Self::TianJi => Some(ZhuXing::TianJi),
            Self::TaiYang => Some(ZhuXing::TaiYang),
            Self::WuQu => Some(ZhuXing::WuQu),
            Self::TianTong => Some(ZhuXing::TianTong),
            Self::LianZhen => Some(ZhuXing::LianZhen),
            Self::TianFu => Some(ZhuXing::TianFu),
            Self::TaiYin => Some(ZhuXing::TaiYin),
            Self::TanLang => Some(ZhuXing::TanLang),
            Self::JuMen => Some(ZhuXing::JuMen),
            Self::TianXiang => Some(ZhuXing::TianXiang),
            Self::TianLiang => Some(ZhuXing::TianLiang),
            Self::QiSha => Some(ZhuXing::QiSha),
            Self::PoJun => Some(ZhuXing::PoJun),
            _ => None,
        }
    }

    /// 尝试转换为六吉星枚举
    pub fn to_liu_ji_xing(&self) -> Option<LiuJiXing> {
        match self {
            Self::WenChang => Some(LiuJiXing::WenChang),
            Self::WenQu => Some(LiuJiXing::WenQu),
            Self::ZuoFu => Some(LiuJiXing::ZuoFu),
            Self::YouBi => Some(LiuJiXing::YouBi),
            _ => None,
        }
    }
}

// ============================================================================
// 星曜亮度
// ============================================================================

/// 星曜亮度（庙旺利陷）
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum StarBrightness {
    Miao = 0,     // 庙 - 最旺
    Wang = 1,     // 旺 - 次旺
    De = 2,       // 得 - 得地
    #[default]
    Ping = 3,     // 平 - 平常
    BuDe = 4,     // 不得 - 不得地
    Xian = 5,     // 陷 - 落陷
}

impl StarBrightness {
    /// 获取亮度名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Miao => "庙",
            Self::Wang => "旺",
            Self::De => "得",
            Self::Ping => "平",
            Self::BuDe => "不",
            Self::Xian => "陷",
        }
    }

    /// 获取亮度权重（用于评分）
    pub fn weight(&self) -> u8 {
        match self {
            Self::Miao => 100,
            Self::Wang => 80,
            Self::De => 60,
            Self::Ping => 40,
            Self::BuDe => 20,
            Self::Xian => 10,
        }
    }

    /// 从数值创建亮度枚举
    ///
    /// # 映射规则
    /// 庙旺表数值: 0=陷, 1=平, 2=得, 3=利(不得), 4=旺, 5=庙
    ///
    /// # 参数
    /// - value: 庙旺表中的数值
    ///
    /// # 返回
    /// 对应的亮度枚举
    pub fn from_value(value: u8) -> Self {
        match value {
            0 => Self::Xian,   // 陷
            1 => Self::Ping,   // 平
            2 => Self::De,     // 得
            3 => Self::BuDe,   // 利（介于得与平之间）
            4 => Self::Wang,   // 旺
            5 => Self::Miao,   // 庙
            _ => Self::Ping,   // 默认平
        }
    }
}

// ============================================================================
// 博士十二星
// ============================================================================

/// 博士十二星
/// 从禄存起博士，依次顺排（阳男阴女顺行，阴男阳女逆行）
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum BoShiXing {
    #[default]
    BoShi = 0,    // 博士 - 聪明才智
    LiShi = 1,    // 力士 - 权力威势
    QingLong = 2, // 青龙 - 喜庆吉祥
    XiaoHao = 3,  // 小耗 - 小破财
    JiangJun = 4, // 将军 - 威武刚强
    ZouShu = 5,   // 奏书 - 文书事务
    FeiLian = 6,  // 飞廉 - 是非口舌
    XiShen = 7,   // 喜神 - 喜庆之事
    BingFu = 8,   // 病符 - 疾病灾厄
    DaHao = 9,    // 大耗 - 大破财
    FuBing = 10,  // 伏兵 - 暗藏危机
    GuanFu = 11,  // 官府 - 官司诉讼
}

impl BoShiXing {
    /// 博士十二星名称
    pub const NAMES: [&'static str; 12] = [
        "博士", "力士", "青龙", "小耗", "将军", "奏书",
        "飞廉", "喜神", "病符", "大耗", "伏兵", "官府",
    ];

    /// 获取星名
    pub fn name(&self) -> &'static str {
        Self::NAMES[*self as usize]
    }

    /// 从索引创建
    pub fn from_index(idx: u8) -> Self {
        match idx % 12 {
            0 => Self::BoShi,
            1 => Self::LiShi,
            2 => Self::QingLong,
            3 => Self::XiaoHao,
            4 => Self::JiangJun,
            5 => Self::ZouShu,
            6 => Self::FeiLian,
            7 => Self::XiShen,
            8 => Self::BingFu,
            9 => Self::DaHao,
            10 => Self::FuBing,
            _ => Self::GuanFu,
        }
    }

    /// 是否为吉星
    pub fn is_ji(&self) -> bool {
        matches!(self, Self::BoShi | Self::QingLong | Self::XiShen)
    }

    /// 是否为凶星
    pub fn is_xiong(&self) -> bool {
        matches!(self, Self::XiaoHao | Self::FeiLian | Self::BingFu | Self::DaHao | Self::FuBing | Self::GuanFu)
    }
}

// ============================================================================
// 长生十二宫
// ============================================================================

/// 长生十二宫
/// 从五行局起长生，依次顺/逆排
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum ChangSheng {
    #[default]
    ChangSheng = 0, // 长生 - 生命开始
    MuYu = 1,       // 沐浴 - 洗礼净化
    GuanDai = 2,    // 冠带 - 成年礼
    LinGuan = 3,    // 临官 - 任职做官
    DiWang = 4,     // 帝旺 - 最旺盛期
    Shuai = 5,      // 衰 - 开始衰退
    Bing = 6,       // 病 - 生病状态
    Si = 7,         // 死 - 死亡阶段
    Mu = 8,         // 墓 - 入墓安葬
    Jue = 9,        // 绝 - 断绝时期
    Tai = 10,       // 胎 - 受胎阶段
    Yang = 11,      // 养 - 养育阶段
}

impl ChangSheng {
    /// 长生十二宫名称
    pub const NAMES: [&'static str; 12] = [
        "长生", "沐浴", "冠带", "临官", "帝旺", "衰",
        "病", "死", "墓", "绝", "胎", "养",
    ];

    /// 获取宫名
    pub fn name(&self) -> &'static str {
        Self::NAMES[*self as usize]
    }

    /// 从索引创建
    pub fn from_index(idx: u8) -> Self {
        match idx % 12 {
            0 => Self::ChangSheng,
            1 => Self::MuYu,
            2 => Self::GuanDai,
            3 => Self::LinGuan,
            4 => Self::DiWang,
            5 => Self::Shuai,
            6 => Self::Bing,
            7 => Self::Si,
            8 => Self::Mu,
            9 => Self::Jue,
            10 => Self::Tai,
            _ => Self::Yang,
        }
    }

    /// 是否为吉位
    pub fn is_ji(&self) -> bool {
        matches!(self, Self::ChangSheng | Self::GuanDai | Self::LinGuan | Self::DiWang)
    }

    /// 是否为凶位
    pub fn is_xiong(&self) -> bool {
        matches!(self, Self::MuYu | Self::Bing | Self::Si | Self::Mu | Self::Jue)
    }
}

// ============================================================================
// 性别
// ============================================================================

/// 性别
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum Gender {
    #[default]
    Male = 0,   // 男
    Female = 1, // 女
}

impl Gender {
    /// 获取性别名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Male => "男",
            Self::Female => "女",
        }
    }
}

// ============================================================================
// 宫位数据结构
// ============================================================================

/// 宫位信息
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct Palace {
    /// 宫位类型
    pub gong_wei: GongWei,
    /// 宫位所在地支
    pub di_zhi: DiZhi,
    /// 宫干
    pub tian_gan: TianGan,
    /// 主星列表（最多3颗）
    pub zhu_xing: [Option<ZhuXing>; 3],
    /// 主星亮度
    pub zhu_xing_brightness: [StarBrightness; 3],
    /// 六吉星
    pub liu_ji: [bool; 6],
    /// 六煞星
    pub liu_sha: [bool; 6],
    /// 四化（生年四化在此宫的星）
    pub si_hua: [Option<SiHua>; 4],
    /// 禄存所在
    pub lu_cun: bool,
    /// 天马所在
    pub tian_ma: bool,
}

// ============================================================================
// 命盘主结构
// ============================================================================

/// 紫微斗数命盘
///
/// 支持三种隐私模式：
/// - Public (0): 所有数据明文存储，任何人可查看
/// - Partial (1): 计算数据明文 + 敏感数据（姓名、问题）加密
/// - Private (2): 全部数据加密，需前端解密后调用 compute_chart API
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
#[scale_info(skip_type_params(MaxCidLen))]
pub struct ZiweiChart<AccountId, BlockNumber, Moment, MaxCidLen: frame_support::traits::Get<u32>> {
    /// 命盘ID
    pub id: u64,
    /// 创建者账户
    pub creator: AccountId,
    /// 创建区块
    pub created_at: BlockNumber,
    /// 时间戳（毫秒）
    pub timestamp: Moment,

    // ===== 隐私控制字段 (v3.4 新增) =====
    /// 隐私模式 (0=Public, 1=Partial, 2=Private)
    pub privacy_mode: pallet_divination_privacy::types::PrivacyMode,
    /// 加密字段标记（位标志：bit 0=姓名, bit 1=出生日期, bit 2=性别, bit 3=问题）
    pub encrypted_fields: Option<u8>,
    /// 敏感数据哈希（用于完整性验证）
    pub sensitive_data_hash: Option<[u8; 32]>,

    // ===== 出生信息 =====
    /// 农历年（Private 模式时为 None）
    pub lunar_year: Option<u16>,
    /// 农历月（Private 模式时为 None）
    pub lunar_month: Option<u8>,
    /// 农历日（Private 模式时为 None）
    pub lunar_day: Option<u8>,
    /// 出生时辰（Private 模式时为 None）
    pub birth_hour: Option<DiZhi>,
    /// 性别（Private 模式时为 None）
    pub gender: Option<Gender>,
    /// 是否闰月
    pub is_leap_month: bool,

    // ===== 四柱信息 =====
    /// 年干（Private 模式时为 None）
    pub year_gan: Option<TianGan>,
    /// 年支（Private 模式时为 None）
    pub year_zhi: Option<DiZhi>,

    // ===== 命盘核心（Private 模式时为 None）=====
    /// 五行局
    pub wu_xing_ju: Option<WuXing>,
    /// 局数
    pub ju_shu: Option<u8>,
    /// 命宫位置（地支索引 0-11）
    pub ming_gong_pos: Option<u8>,
    /// 身宫位置
    pub shen_gong_pos: Option<u8>,
    /// 紫微星位置
    pub ziwei_pos: Option<u8>,
    /// 天府星位置
    pub tianfu_pos: Option<u8>,

    // ===== 十二宫排布（Private 模式时为 None）=====
    /// 十二宫数据
    pub palaces: Option<[Palace; 12]>,

    // ===== 四化信息（Private 模式时为 None）=====
    /// 生年四化星（使用 SiHuaStar 支持主星和辅星）
    pub si_hua_stars: Option<[SiHuaStar; 4]>,

    // ===== 大运信息（Private 模式时为 None）=====
    /// 起运年龄
    pub qi_yun_age: Option<u8>,
    /// 大运顺逆（true=顺行，false=逆行）
    pub da_yun_shun: Option<bool>,

    // ===== 状态 =====
    /// AI 解读 CID
    pub ai_interpretation_cid: Option<BoundedVec<u8, MaxCidLen>>,
}

impl<AccountId, BlockNumber, Moment, MaxCidLen: frame_support::traits::Get<u32>>
    ZiweiChart<AccountId, BlockNumber, Moment, MaxCidLen>
{
    /// 检查是否有计算数据（用于判断是否可以解读）
    pub fn has_calculation_data(&self) -> bool {
        self.palaces.is_some() && self.wu_xing_ju.is_some()
    }

    /// 检查是否可以进行解读（非 Private 模式且有计算数据）
    pub fn can_interpret(&self) -> bool {
        self.privacy_mode != pallet_divination_privacy::types::PrivacyMode::Private
            && self.has_calculation_data()
    }

    /// 检查是否公开（向后兼容）
    pub fn is_public(&self) -> bool {
        self.privacy_mode == pallet_divination_privacy::types::PrivacyMode::Public
    }

    /// 获取十二宫数据（如果可用）
    pub fn get_palaces(&self) -> Option<&[Palace; 12]> {
        self.palaces.as_ref()
    }

    /// 获取五行局（如果可用）
    pub fn get_wu_xing_ju(&self) -> Option<WuXing> {
        self.wu_xing_ju
    }

    /// 获取年干支（如果可用）
    pub fn get_year_ganzhi(&self) -> Option<(TianGan, DiZhi)> {
        match (self.year_gan, self.year_zhi) {
            (Some(gan), Some(zhi)) => Some((gan, zhi)),
            _ => None,
        }
    }

    /// 获取出生时辰（如果可用）
    pub fn get_birth_hour(&self) -> Option<DiZhi> {
        self.birth_hour
    }

    /// 获取四化星（如果可用）
    pub fn get_si_hua_stars(&self) -> Option<&[SiHuaStar; 4]> {
        self.si_hua_stars.as_ref()
    }
}

// ============================================================================
// 起盘方式
// ============================================================================

/// 起盘方式
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum DivinationMethod {
    /// 时间起盘（根据出生时间自动计算）
    #[default]
    ByTime = 0,
    /// 手动指定（直接输入四柱信息）
    Manual = 1,
    /// 随机起盘（测试/娱乐用途）
    Random = 2,
}

impl DivinationMethod {
    /// 获取起盘方式名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::ByTime => "时间起盘",
            Self::Manual => "手动指定",
            Self::Random => "随机起盘",
        }
    }
}

// ============================================================================
// 用户统计
// ============================================================================

/// 用户统计数据
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct UserStats {
    /// 总排盘次数
    pub total_charts: u32,
    /// AI 解读次数
    pub ai_interpretations: u32,
    /// 首次排盘区块
    pub first_chart_block: u32,
}
