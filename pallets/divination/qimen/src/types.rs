//! # 奇门遁甲数据类型定义
//!
//! 本模块定义了奇门遁甲排盘系统所需的所有数据类型，包括：
//! - 天干地支
//! - 九宫方位
//! - 九星、八门、八神
//! - 三奇六仪
//! - 排盘结果结构
//!
//! ## 隐私模式支持
//!
//! QimenChart 结构支持三种隐私模式（PrivacyMode）：
//! - Public: 所有数据明文存储，公开可见
//! - Partial: 计算数据明文 + 敏感数据加密（推荐用于专业解读场景）
//! - Private: 全部数据加密，需前端解密后调用 compute_chart API

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use frame_support::traits::ConstU32;
use pallet_divination_privacy::types::PrivacyMode;
use scale_info::TypeInfo;
use sp_std::prelude::*;

// ==================== 常量定义 ====================

/// 命主姓名最大长度（字节）
/// UTF-8 中文字符通常占 3 字节，32 字节约可存储 10 个汉字
pub type MaxNameLen = ConstU32<32>;

/// 占问事宜最大长度（字节）
/// 128 字节约可存储 42 个汉字，足够描述简短的问题
pub type MaxQuestionLen = ConstU32<128>;

// ==================== 性别 ====================

/// 性别
///
/// 用于命主信息，某些占卜分析可能需要考虑性别因素
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum Gender {
    /// 男性（默认）
    #[default]
    Male = 0,
    /// 女性
    Female = 1,
}

impl Gender {
    /// 获取性别名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Male => "男",
            Self::Female => "女",
        }
    }

    /// 从数字获取性别
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Male),
            1 => Some(Self::Female),
            _ => None,
        }
    }
}

// ==================== 天干地支 ====================

/// 十天干
///
/// 甲乙丙丁戊己庚辛壬癸，用于纪年、纪月、纪日、纪时
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum TianGan {
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
    /// 从数字获取天干（0-9）
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::Jia),
            1 => Some(Self::Yi),
            2 => Some(Self::Bing),
            3 => Some(Self::Ding),
            4 => Some(Self::Wu),
            5 => Some(Self::Ji),
            6 => Some(Self::Geng),
            7 => Some(Self::Xin),
            8 => Some(Self::Ren),
            9 => Some(Self::Gui),
            _ => None,
        }
    }

    /// 获取天干索引
    pub fn index(&self) -> u8 {
        *self as u8
    }

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

    /// 判断是否为三奇（乙丙丁）
    pub fn is_san_qi(&self) -> bool {
        matches!(self, Self::Yi | Self::Bing | Self::Ding)
    }

    /// 判断是否为六仪（戊己庚辛壬癸）
    pub fn is_liu_yi(&self) -> bool {
        matches!(self, Self::Wu | Self::Ji | Self::Geng | Self::Xin | Self::Ren | Self::Gui)
    }

    /// 获取天干五行属性
    pub fn wu_xing(&self) -> WuXing {
        match self {
            Self::Jia | Self::Yi => WuXing::Mu,
            Self::Bing | Self::Ding => WuXing::Huo,
            Self::Wu | Self::Ji => WuXing::Tu,
            Self::Geng | Self::Xin => WuXing::Jin,
            Self::Ren | Self::Gui => WuXing::Shui,
        }
    }
}

/// 十二地支
///
/// 子丑寅卯辰巳午未申酉戌亥
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum DiZhi {
    Zi = 0,    // 子（23:00-01:00）
    Chou = 1,  // 丑（01:00-03:00）
    Yin = 2,   // 寅（03:00-05:00）
    Mao = 3,   // 卯（05:00-07:00）
    Chen = 4,  // 辰（07:00-09:00）
    Si = 5,    // 巳（09:00-11:00）
    Wu = 6,    // 午（11:00-13:00）
    Wei = 7,   // 未（13:00-15:00）
    Shen = 8,  // 申（15:00-17:00）
    You = 9,   // 酉（17:00-19:00）
    Xu = 10,   // 戌（19:00-21:00）
    Hai = 11,  // 亥（21:00-23:00）
}

impl DiZhi {
    /// 从数字获取地支（0-11）
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::Zi),
            1 => Some(Self::Chou),
            2 => Some(Self::Yin),
            3 => Some(Self::Mao),
            4 => Some(Self::Chen),
            5 => Some(Self::Si),
            6 => Some(Self::Wu),
            7 => Some(Self::Wei),
            8 => Some(Self::Shen),
            9 => Some(Self::You),
            10 => Some(Self::Xu),
            11 => Some(Self::Hai),
            _ => None,
        }
    }

    /// 从小时获取地支（0-23小时）
    pub fn from_hour(hour: u8) -> Option<Self> {
        match hour {
            23 | 0 => Some(Self::Zi),
            1 | 2 => Some(Self::Chou),
            3 | 4 => Some(Self::Yin),
            5 | 6 => Some(Self::Mao),
            7 | 8 => Some(Self::Chen),
            9 | 10 => Some(Self::Si),
            11 | 12 => Some(Self::Wu),
            13 | 14 => Some(Self::Wei),
            15 | 16 => Some(Self::Shen),
            17 | 18 => Some(Self::You),
            19 | 20 => Some(Self::Xu),
            21 | 22 => Some(Self::Hai),
            _ => None,
        }
    }

    /// 获取地支索引
    pub fn index(&self) -> u8 {
        *self as u8
    }

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

    /// 获取地支五行属性
    pub fn wu_xing(&self) -> WuXing {
        match self {
            Self::Yin | Self::Mao => WuXing::Mu,
            Self::Si | Self::Wu => WuXing::Huo,
            Self::Shen | Self::You => WuXing::Jin,
            Self::Hai | Self::Zi => WuXing::Shui,
            Self::Chen | Self::Xu | Self::Chou | Self::Wei => WuXing::Tu,
        }
    }
}

/// 干支组合
///
/// 用于表示年柱、月柱、日柱、时柱
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct GanZhi {
    pub gan: TianGan,
    pub zhi: DiZhi,
}

impl GanZhi {
    /// 创建新的干支组合
    pub fn new(gan: TianGan, zhi: DiZhi) -> Self {
        Self { gan, zhi }
    }

    /// 从六十甲子序号（0-59）创建干支
    pub fn from_sexagenary(index: u8) -> Option<Self> {
        if index >= 60 {
            return None;
        }
        let gan = TianGan::from_index(index % 10)?;
        let zhi = DiZhi::from_index(index % 12)?;
        Some(Self::new(gan, zhi))
    }

    /// 获取六十甲子序号（0-59）
    pub fn sexagenary_index(&self) -> u8 {
        // 使用通用公式计算六十甲子序号
        let g = self.gan.index() as u16;
        let z = self.zhi.index() as u16;
        // (天干 - 地支) mod 10 配合地支位置
        ((g * 6 + z * 5) % 60) as u8
    }
}

/// 为 GanZhi 实现 Default
///
/// 默认值为甲子（六十甲子的第一个）
impl Default for GanZhi {
    fn default() -> Self {
        Self {
            gan: TianGan::Jia,
            zhi: DiZhi::Zi,
        }
    }
}

// ==================== 五行 ====================

/// 五行
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum WuXing {
    Jin = 0,   // 金
    Mu = 1,    // 木
    Shui = 2,  // 水
    Huo = 3,   // 火
    Tu = 4,    // 土
}

impl WuXing {
    /// 获取五行名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Jin => "金",
            Self::Mu => "木",
            Self::Shui => "水",
            Self::Huo => "火",
            Self::Tu => "土",
        }
    }

    /// 判断本五行是否生另一五行
    ///
    /// 相生顺序：金生水、水生木、木生火、火生土、土生金
    pub fn generates(&self, other: &WuXing) -> bool {
        matches!(
            (self, other),
            (Self::Jin, Self::Shui)
                | (Self::Shui, Self::Mu)
                | (Self::Mu, Self::Huo)
                | (Self::Huo, Self::Tu)
                | (Self::Tu, Self::Jin)
        )
    }

    /// 判断本五行是否克另一五行
    ///
    /// 相克顺序：金克木、木克土、土克水、水克火、火克金
    pub fn conquers(&self, other: &WuXing) -> bool {
        matches!(
            (self, other),
            (Self::Jin, Self::Mu)
                | (Self::Mu, Self::Tu)
                | (Self::Tu, Self::Shui)
                | (Self::Shui, Self::Huo)
                | (Self::Huo, Self::Jin)
        )
    }
}

// ==================== 九宫 ====================

/// 九宫方位
///
/// 基于洛书九宫，中宫为五，周围八宫按方位分布
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum JiuGong {
    Kan = 1,     // 坎一宫（北方，水）
    Kun = 2,     // 坤二宫（西南，土）
    Zhen = 3,    // 震三宫（东方，木）
    Xun = 4,     // 巽四宫（东南，木）
    Zhong = 5,   // 中五宫（中央，土）
    Qian = 6,    // 乾六宫（西北，金）
    Dui = 7,     // 兑七宫（西方，金）
    Gen = 8,     // 艮八宫（东北，土）
    Li = 9,      // 离九宫（南方，火）
}

impl JiuGong {
    /// 从数字获取九宫（1-9）
    pub fn from_num(num: u8) -> Option<Self> {
        match num {
            1 => Some(Self::Kan),
            2 => Some(Self::Kun),
            3 => Some(Self::Zhen),
            4 => Some(Self::Xun),
            5 => Some(Self::Zhong),
            6 => Some(Self::Qian),
            7 => Some(Self::Dui),
            8 => Some(Self::Gen),
            9 => Some(Self::Li),
            _ => None,
        }
    }

    /// 获取宫位数字
    pub fn num(&self) -> u8 {
        *self as u8
    }

    /// 获取宫位名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Kan => "坎一宫",
            Self::Kun => "坤二宫",
            Self::Zhen => "震三宫",
            Self::Xun => "巽四宫",
            Self::Zhong => "中五宫",
            Self::Qian => "乾六宫",
            Self::Dui => "兑七宫",
            Self::Gen => "艮八宫",
            Self::Li => "离九宫",
        }
    }

    /// 获取宫位五行
    pub fn wu_xing(&self) -> WuXing {
        match self {
            Self::Kan => WuXing::Shui,
            Self::Kun | Self::Zhong | Self::Gen => WuXing::Tu,
            Self::Zhen | Self::Xun => WuXing::Mu,
            Self::Qian | Self::Dui => WuXing::Jin,
            Self::Li => WuXing::Huo,
        }
    }

    /// 获取宫位方向
    pub fn direction(&self) -> &'static str {
        match self {
            Self::Kan => "北",
            Self::Kun => "西南",
            Self::Zhen => "东",
            Self::Xun => "东南",
            Self::Zhong => "中",
            Self::Qian => "西北",
            Self::Dui => "西",
            Self::Gen => "东北",
            Self::Li => "南",
        }
    }

    /// 阳遁顺序下一宫（顺时针）
    ///
    /// 顺序：1→8→3→4→9→2→7→6→1（跳过中宫5）
    pub fn next_yang(&self) -> Self {
        match self {
            Self::Kan => Self::Gen,   // 1→8
            Self::Gen => Self::Zhen,  // 8→3
            Self::Zhen => Self::Xun,  // 3→4
            Self::Xun => Self::Li,    // 4→9
            Self::Li => Self::Kun,    // 9→2
            Self::Kun => Self::Dui,   // 2→7
            Self::Dui => Self::Qian,  // 7→6
            Self::Qian => Self::Kan,  // 6→1
            Self::Zhong => Self::Kun, // 中宫寄坤二宫
        }
    }

    /// 阴遁顺序下一宫（逆时针）
    ///
    /// 顺序：1→6→7→2→9→4→3→8→1（跳过中宫5）
    pub fn next_yin(&self) -> Self {
        match self {
            Self::Kan => Self::Qian,  // 1→6
            Self::Qian => Self::Dui,  // 6→7
            Self::Dui => Self::Kun,   // 7→2
            Self::Kun => Self::Li,    // 2→9
            Self::Li => Self::Xun,    // 9→4
            Self::Xun => Self::Zhen,  // 4→3
            Self::Zhen => Self::Gen,  // 3→8
            Self::Gen => Self::Kan,   // 8→1
            Self::Zhong => Self::Gen, // 中宫寄艮八宫
        }
    }
}

// ==================== 九星 ====================

/// 九星
///
/// 奇门遁甲中的九星，代表天盘
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum JiuXing {
    TianPeng = 1,   // 天蓬星（坎一宫，凶星，水）
    TianRui = 2,    // 天芮星（坤二宫，凶星，土）
    TianChong = 3,  // 天冲星（震三宫，吉星，木）
    TianFu = 4,     // 天辅星（巽四宫，吉星，木）
    TianQin = 5,    // 天禽星（中五宫，吉星，土）
    TianXin = 6,    // 天心星（乾六宫，吉星，金）
    TianZhu = 7,    // 天柱星（兑七宫，凶星，金）
    TianRen = 8,    // 天任星（艮八宫，吉星，土）
    TianYing = 9,   // 天英星（离九宫，凶星，火）
}

impl JiuXing {
    /// 从数字获取九星（1-9）
    pub fn from_num(num: u8) -> Option<Self> {
        match num {
            1 => Some(Self::TianPeng),
            2 => Some(Self::TianRui),
            3 => Some(Self::TianChong),
            4 => Some(Self::TianFu),
            5 => Some(Self::TianQin),
            6 => Some(Self::TianXin),
            7 => Some(Self::TianZhu),
            8 => Some(Self::TianRen),
            9 => Some(Self::TianYing),
            _ => None,
        }
    }

    /// 获取星序号
    pub fn num(&self) -> u8 {
        *self as u8
    }

    /// 获取星名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::TianPeng => "天蓬",
            Self::TianRui => "天芮",
            Self::TianChong => "天冲",
            Self::TianFu => "天辅",
            Self::TianQin => "天禽",
            Self::TianXin => "天心",
            Self::TianZhu => "天柱",
            Self::TianRen => "天任",
            Self::TianYing => "天英",
        }
    }

    /// 判断是否为吉星
    pub fn is_auspicious(&self) -> bool {
        matches!(
            self,
            Self::TianChong | Self::TianFu | Self::TianQin | Self::TianXin | Self::TianRen
        )
    }

    /// 获取星的五行
    pub fn wu_xing(&self) -> WuXing {
        match self {
            Self::TianPeng => WuXing::Shui,
            Self::TianRui | Self::TianQin | Self::TianRen => WuXing::Tu,
            Self::TianChong | Self::TianFu => WuXing::Mu,
            Self::TianXin | Self::TianZhu => WuXing::Jin,
            Self::TianYing => WuXing::Huo,
        }
    }

    /// 获取星的原始宫位
    pub fn original_palace(&self) -> JiuGong {
        match self {
            Self::TianPeng => JiuGong::Kan,
            Self::TianRui => JiuGong::Kun,
            Self::TianChong => JiuGong::Zhen,
            Self::TianFu => JiuGong::Xun,
            Self::TianQin => JiuGong::Zhong,
            Self::TianXin => JiuGong::Qian,
            Self::TianZhu => JiuGong::Dui,
            Self::TianRen => JiuGong::Gen,
            Self::TianYing => JiuGong::Li,
        }
    }
}

/// 为 JiuXing 实现 Default
///
/// 默认值为天禽星（中五宫）
impl Default for JiuXing {
    fn default() -> Self {
        Self::TianQin
    }
}

// ==================== 八门 ====================

/// 八门
///
/// 奇门遁甲中的八门，代表人盘（人事活动方面）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum BaMen {
    Xiu = 1,    // 休门（坎一宫，吉门，水）
    Si = 2,     // 死门（坤二宫，凶门，土）
    Shang = 3,  // 伤门（震三宫，凶门，木）
    Du = 4,     // 杜门（巽四宫，凶门，木）
    Jing = 5,   // 景门（离九宫，中平，火）
    Kai = 6,    // 开门（乾六宫，大吉，金）
    Jing2 = 7,  // 惊门（兑七宫，凶门，金）
    Sheng = 8,  // 生门（艮八宫，大吉，土）
}

impl BaMen {
    /// 从数字获取八门（1-8）
    pub fn from_num(num: u8) -> Option<Self> {
        match num {
            1 => Some(Self::Xiu),
            2 => Some(Self::Si),
            3 => Some(Self::Shang),
            4 => Some(Self::Du),
            5 => Some(Self::Jing),
            6 => Some(Self::Kai),
            7 => Some(Self::Jing2),
            8 => Some(Self::Sheng),
            _ => None,
        }
    }

    /// 获取门序号
    pub fn num(&self) -> u8 {
        *self as u8
    }

    /// 获取门名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Xiu => "休门",
            Self::Si => "死门",
            Self::Shang => "伤门",
            Self::Du => "杜门",
            Self::Jing => "景门",
            Self::Kai => "开门",
            Self::Jing2 => "惊门",
            Self::Sheng => "生门",
        }
    }

    /// 判断是否为吉门
    pub fn is_auspicious(&self) -> bool {
        matches!(self, Self::Xiu | Self::Kai | Self::Sheng | Self::Jing)
    }

    /// 获取门的五行
    pub fn wu_xing(&self) -> WuXing {
        match self {
            Self::Xiu => WuXing::Shui,
            Self::Si | Self::Sheng => WuXing::Tu,
            Self::Shang | Self::Du => WuXing::Mu,
            Self::Kai | Self::Jing2 => WuXing::Jin,
            Self::Jing => WuXing::Huo,
        }
    }

    /// 获取门的原始宫位
    pub fn original_palace(&self) -> JiuGong {
        match self {
            Self::Xiu => JiuGong::Kan,
            Self::Si => JiuGong::Kun,
            Self::Shang => JiuGong::Zhen,
            Self::Du => JiuGong::Xun,
            Self::Jing => JiuGong::Li,
            Self::Kai => JiuGong::Qian,
            Self::Jing2 => JiuGong::Dui,
            Self::Sheng => JiuGong::Gen,
        }
    }

    /// 根据原始宫位获取八门
    ///
    /// 每个宫位（除中宫外）都有一个原始八门：
    /// - 一宫（坎）：休门
    /// - 二宫（坤）：死门
    /// - 三宫（震）：伤门
    /// - 四宫（巽）：杜门
    /// - 五宫（中）：无门（返回None）
    /// - 六宫（乾）：开门
    /// - 七宫（兑）：惊门
    /// - 八宫（艮）：生门
    /// - 九宫（离）：景门
    pub fn from_palace(gong: u8) -> Option<Self> {
        match gong {
            1 => Some(Self::Xiu),   // 坎宫 - 休门
            2 => Some(Self::Si),    // 坤宫 - 死门
            3 => Some(Self::Shang), // 震宫 - 伤门
            4 => Some(Self::Du),    // 巽宫 - 杜门
            5 => None,              // 中宫 - 无门
            6 => Some(Self::Kai),   // 乾宫 - 开门
            7 => Some(Self::Jing2), // 兑宫 - 惊门
            8 => Some(Self::Sheng), // 艮宫 - 生门
            9 => Some(Self::Jing),  // 离宫 - 景门
            _ => None,
        }
    }
}

/// 为 BaMen 实现 Default
///
/// 默认值为开门（大吉门）
impl Default for BaMen {
    fn default() -> Self {
        Self::Kai
    }
}

// ==================== 八神 ====================

/// 八神
///
/// 奇门遁甲中的八神，代表神盘（天时神煞）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum BaShen {
    ZhiFu = 1,    // 值符（吉神，统领诸神）
    TengShe = 2,  // 腾蛇（凶神，惊恐怪异）
    TaiYin = 3,   // 太阴（吉神，阴私暗昧）
    LiuHe = 4,    // 六合（吉神，婚姻交易）
    BaiHu = 5,    // 白虎（凶神，凶丧血光）
    XuanWu = 6,   // 玄武（凶神，盗贼小人）
    JiuDi = 7,    // 九地（吉神，安静隐伏）
    JiuTian = 8,  // 九天（吉神，远行高举）
}

impl BaShen {
    /// 从数字获取八神（1-8）
    pub fn from_num(num: u8) -> Option<Self> {
        match num {
            1 => Some(Self::ZhiFu),
            2 => Some(Self::TengShe),
            3 => Some(Self::TaiYin),
            4 => Some(Self::LiuHe),
            5 => Some(Self::BaiHu),
            6 => Some(Self::XuanWu),
            7 => Some(Self::JiuDi),
            8 => Some(Self::JiuTian),
            _ => None,
        }
    }

    /// 获取神序号
    pub fn num(&self) -> u8 {
        *self as u8
    }

    /// 获取神名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::ZhiFu => "值符",
            Self::TengShe => "腾蛇",
            Self::TaiYin => "太阴",
            Self::LiuHe => "六合",
            Self::BaiHu => "白虎",
            Self::XuanWu => "玄武",
            Self::JiuDi => "九地",
            Self::JiuTian => "九天",
        }
    }

    /// 判断是否为吉神
    pub fn is_auspicious(&self) -> bool {
        matches!(
            self,
            Self::ZhiFu | Self::TaiYin | Self::LiuHe | Self::JiuDi | Self::JiuTian
        )
    }
}

// ==================== 节气与阴阳遁 ====================

/// 二十四节气
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum JieQi {
    DongZhi = 0,    // 冬至（阳遁一局起）
    XiaoHan = 1,    // 小寒
    DaHan = 2,      // 大寒
    LiChun = 3,     // 立春
    YuShui = 4,     // 雨水
    JingZhe = 5,    // 惊蛰
    ChunFen = 6,    // 春分
    QingMing = 7,   // 清明
    GuYu = 8,       // 谷雨
    LiXia = 9,      // 立夏
    XiaoMan = 10,   // 小满
    MangZhong = 11, // 芒种
    XiaZhi = 12,    // 夏至（阴遁九局起）
    XiaoShu = 13,   // 小暑
    DaShu = 14,     // 大暑
    LiQiu = 15,     // 立秋
    ChuShu = 16,    // 处暑
    BaiLu = 17,     // 白露
    QiuFen = 18,    // 秋分
    HanLu = 19,     // 寒露
    ShuangJiang = 20, // 霜降
    LiDong = 21,    // 立冬
    XiaoXue = 22,   // 小雪
    DaXue = 23,     // 大雪
}

impl JieQi {
    /// 从数字获取节气（0-23）
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::DongZhi),
            1 => Some(Self::XiaoHan),
            2 => Some(Self::DaHan),
            3 => Some(Self::LiChun),
            4 => Some(Self::YuShui),
            5 => Some(Self::JingZhe),
            6 => Some(Self::ChunFen),
            7 => Some(Self::QingMing),
            8 => Some(Self::GuYu),
            9 => Some(Self::LiXia),
            10 => Some(Self::XiaoMan),
            11 => Some(Self::MangZhong),
            12 => Some(Self::XiaZhi),
            13 => Some(Self::XiaoShu),
            14 => Some(Self::DaShu),
            15 => Some(Self::LiQiu),
            16 => Some(Self::ChuShu),
            17 => Some(Self::BaiLu),
            18 => Some(Self::QiuFen),
            19 => Some(Self::HanLu),
            20 => Some(Self::ShuangJiang),
            21 => Some(Self::LiDong),
            22 => Some(Self::XiaoXue),
            23 => Some(Self::DaXue),
            _ => None,
        }
    }

    /// 获取节气名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::DongZhi => "冬至",
            Self::XiaoHan => "小寒",
            Self::DaHan => "大寒",
            Self::LiChun => "立春",
            Self::YuShui => "雨水",
            Self::JingZhe => "惊蛰",
            Self::ChunFen => "春分",
            Self::QingMing => "清明",
            Self::GuYu => "谷雨",
            Self::LiXia => "立夏",
            Self::XiaoMan => "小满",
            Self::MangZhong => "芒种",
            Self::XiaZhi => "夏至",
            Self::XiaoShu => "小暑",
            Self::DaShu => "大暑",
            Self::LiQiu => "立秋",
            Self::ChuShu => "处暑",
            Self::BaiLu => "白露",
            Self::QiuFen => "秋分",
            Self::HanLu => "寒露",
            Self::ShuangJiang => "霜降",
            Self::LiDong => "立冬",
            Self::XiaoXue => "小雪",
            Self::DaXue => "大雪",
        }
    }

    /// 判断是否为阳遁（冬至到夏至前）
    pub fn is_yang_dun(&self) -> bool {
        let index = *self as u8;
        index < 12 // 0-11是阳遁
    }
}

/// 阴阳遁类型
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
pub enum DunType {
    #[default]
    Yang, // 阳遁（冬至后，顺行）
    Yin,  // 阴遁（夏至后，逆行）
}

impl DunType {
    /// 获取遁名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Yang => "阳遁",
            Self::Yin => "阴遁",
        }
    }
}

/// 三元（上元、中元、下元）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum SanYuan {
    Shang, // 上元（每节气第1-5天）
    Zhong, // 中元（每节气第6-10天）
    Xia,   // 下元（每节气第11-15天）
}

impl SanYuan {
    /// 获取元名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Shang => "上元",
            Self::Zhong => "中元",
            Self::Xia => "下元",
        }
    }
}

// ==================== 起卦方式 ====================

/// 排盘类型（时家/日家/月家/年家）
///
/// 奇门遁甲有多种排盘类型，根据不同的时间单位起局
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum QimenType {
    /// 时家奇门：以时辰为单位，最常用
    #[default]
    ShiJia,
    /// 日家奇门：以日为单位
    RiJia,
    /// 月家奇门：以月为单位
    YueJia,
    /// 年家奇门：以年为单位
    NianJia,
}

impl QimenType {
    /// 获取类型名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::ShiJia => "时家奇门",
            Self::RiJia => "日家奇门",
            Self::YueJia => "月家奇门",
            Self::NianJia => "年家奇门",
        }
    }

    /// 获取简短描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::ShiJia => "以时辰为单位起局，每两小时一局，最常用的排盘方式",
            Self::RiJia => "以日为单位起局，每日一局，适合日课择吉",
            Self::YueJia => "以月为单位起局，每月一局，适合月度规划",
            Self::NianJia => "以年为单位起局，每年一局，适合年度大运分析",
        }
    }
}

// ==================== 问事类型与用神 ====================

/// 问事类型（占断事项分类）
///
/// 奇门遁甲中根据不同的问事类型，有不同的用神和取象规则
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum QuestionType {
    /// 综合运势（默认）
    #[default]
    General,
    /// 事业工作
    Career,
    /// 财运求财
    Wealth,
    /// 婚姻感情
    Marriage,
    /// 健康疾病
    Health,
    /// 学业考试
    Study,
    /// 出行远行
    Travel,
    /// 官司诉讼
    Lawsuit,
    /// 寻人寻物
    Finding,
    /// 投资理财
    Investment,
    /// 合作交易
    Business,
    /// 祈福求神
    Prayer,
}

impl QuestionType {
    /// 获取问事类型名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::General => "综合运势",
            Self::Career => "事业工作",
            Self::Wealth => "财运求财",
            Self::Marriage => "婚姻感情",
            Self::Health => "健康疾病",
            Self::Study => "学业考试",
            Self::Travel => "出行远行",
            Self::Lawsuit => "官司诉讼",
            Self::Finding => "寻人寻物",
            Self::Investment => "投资理财",
            Self::Business => "合作交易",
            Self::Prayer => "祈福求神",
        }
    }

    /// 获取问事类型描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::General => "整体运势分析，综合各方面情况",
            Self::Career => "工作事业、升迁、求职、创业等",
            Self::Wealth => "财运、求财、偏财、正财等",
            Self::Marriage => "婚姻、恋爱、感情、桃花等",
            Self::Health => "疾病、健康、医疗、康复等",
            Self::Study => "考试、学业、资格证、进修等",
            Self::Travel => "出行、旅游、搬迁、远行等",
            Self::Lawsuit => "官司、诉讼、纠纷、仲裁等",
            Self::Finding => "寻人、寻物、失物、走失等",
            Self::Investment => "投资、理财、股票、基金等",
            Self::Business => "合作、交易、谈判、签约等",
            Self::Prayer => "祈福、求神、祭祀、许愿等",
        }
    }
}

/// 用神类型
///
/// 奇门遁甲中的用神，代表问事的主体或对象
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YongShen {
    /// 以九星为用神
    Xing(JiuXing),
    /// 以八门为用神
    Men(BaMen),
    /// 以八神为用神
    Shen(BaShen),
    /// 以天干为用神
    Gan(TianGan),
    /// 以宫位为用神
    Gong(JiuGong),
}

impl YongShen {
    /// 获取用神名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Xing(x) => x.name(),
            Self::Men(m) => m.name(),
            Self::Shen(s) => s.name(),
            Self::Gan(g) => g.name(),
            Self::Gong(g) => g.name(),
        }
    }
}

/// 用神配置
///
/// 每种问事类型对应的用神列表
#[derive(Clone, Debug)]
pub struct YongShenConfig {
    /// 问事类型
    pub question_type: QuestionType,
    /// 主用神（最重要）
    pub primary: YongShen,
    /// 次用神（辅助参考）
    pub secondary: Option<YongShen>,
    /// 相关宫位
    pub related_gongs: &'static [JiuGong],
    /// 吉利条件描述
    pub auspicious_condition: &'static str,
}

/// 起卦方式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum DivinationMethod {
    /// 时间起局：根据排盘时间自动计算
    ByTime,
    /// 数字起局：根据用户输入数字
    ByNumbers,
    /// 随机起局：使用链上随机数
    Random,
    /// 手动指定：直接指定局数
    Manual,
}

impl DivinationMethod {
    /// 获取方式名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::ByTime => "时间起局",
            Self::ByNumbers => "数字起局",
            Self::Random => "随机起局",
            Self::Manual => "手动指定",
        }
    }
}

/// 排盘方法（转盘/飞盘）
///
/// 奇门遁甲有两种主要的排盘方法：
/// - 转盘奇门：九星、八门、八神作为整体旋转，是目前最常用的方法
/// - 飞盘奇门：九星、八门、八神按洛书九宫飞布顺序分别飞入各宫
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum PanMethod {
    /// 转盘奇门（默认）
    ///
    /// 九星、八门、八神作为整体随值符旋转
    /// 特点：
    /// - 整体旋转，关系固定
    /// - 便于理解和计算
    /// - 是当前主流的排盘方法
    #[default]
    ZhuanPan,

    /// 飞盘奇门
    ///
    /// 九星、八门、八神按洛书九宫数序分别飞入各宫
    /// 特点：
    /// - 独立飞布，灵活多变
    /// - 古法排盘方式
    /// - 适合特定的占断场景
    FeiPan,
}

impl PanMethod {
    /// 获取方法名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::ZhuanPan => "转盘奇门",
            Self::FeiPan => "飞盘奇门",
        }
    }

    /// 获取简短描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::ZhuanPan => "九星、八门、八神作为整体旋转，是目前最常用的排盘方法",
            Self::FeiPan => "九星、八门、八神按洛书九宫数序分别飞入各宫，古法排盘方式",
        }
    }

    /// 判断是否为转盘
    pub fn is_zhuan_pan(&self) -> bool {
        matches!(self, Self::ZhuanPan)
    }

    /// 判断是否为飞盘
    pub fn is_fei_pan(&self) -> bool {
        matches!(self, Self::FeiPan)
    }
}

// ==================== 单宫信息 ====================

/// 单宫完整信息
///
/// 包含天盘、地盘、人盘、神盘的所有信息
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Palace {
    /// 宫位
    pub gong: JiuGong,
    /// 天盘干（奇仪）
    pub tian_pan_gan: TianGan,
    /// 地盘干（奇仪）
    pub di_pan_gan: TianGan,
    /// 九星
    pub xing: JiuXing,
    /// 八门
    pub men: Option<BaMen>,  // 中宫无门
    /// 八神
    pub shen: Option<BaShen>, // 中宫无神
    /// 是否为旬空
    pub is_xun_kong: bool,
    /// 是否为马星
    pub is_ma_xing: bool,
}

impl Palace {
    /// 创建空宫位（用于初始化）
    pub fn empty(gong: JiuGong) -> Self {
        Self {
            gong,
            tian_pan_gan: TianGan::Jia,
            di_pan_gan: TianGan::Jia,
            xing: JiuXing::TianQin,
            men: None,
            shen: None,
            is_xun_kong: false,
            is_ma_xing: false,
        }
    }

    /// 判断是否伏吟（天盘干等于地盘干）
    pub fn is_fu_yin(&self) -> bool {
        self.tian_pan_gan == self.di_pan_gan
    }

    /// 判断是否反吟（天盘干与地盘干对冲）
    pub fn is_fan_yin(&self) -> bool {
        // 天干没有直接对冲，此处简化为相克
        self.tian_pan_gan.wu_xing().conquers(&self.di_pan_gan.wu_xing())
    }
}

// ==================== 完整排盘结果 ====================

/// 奇门遁甲完整排盘结果
///
/// 包含完整的命主信息和排盘数据，支持链上存储和查询。
///
/// # 隐私模式支持
///
/// QimenChart 支持三种隐私模式：
///
/// ## Public 模式（公开）
/// - 所有字段明文存储
/// - 任何人可查看
/// - `privacy_mode = PrivacyMode::Public`
///
/// ## Partial 模式（部分加密）⭐ 推荐
/// - 计算字段（四柱、九宫等）明文存储，支持 Runtime API 解盘
/// - 敏感字段（姓名、问题等）设为 None，加密存储在 Privacy pallet
/// - `privacy_mode = PrivacyMode::Partial`
/// - `encrypted_fields = Some(NAME | QUESTION)` 标识加密字段
/// - `sensitive_data_hash` 存储敏感数据哈希用于验证
///
/// ## Private 模式（完全加密）
/// - 所有计算字段设为 None
/// - 全部数据加密存储在 Privacy pallet
/// - 需前端解密后调用 `compute_chart` API
/// - `privacy_mode = PrivacyMode::Private`
///
/// # 存储大小估算
/// - 基础字段：约 200 bytes
/// - 命主信息：约 170 bytes（name 32 + gender 1 + birth_year 2 + question 128 + 其他）
/// - 九宫数据：约 180 bytes（9 宫 × 20 bytes）
/// - 隐私字段：约 37 bytes（privacy_mode 1 + encrypted_fields 3 + hash 33）
/// - 总计：约 590 bytes（Public 模式）/ 约 200 bytes（Private 模式）
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxCidLen))]
pub struct QimenChart<AccountId, BlockNumber, MaxCidLen: frame_support::traits::Get<u32>> {
    // ==================== 基础信息 ====================
    /// 排盘 ID
    pub id: u64,
    /// 排盘者账户
    pub diviner: AccountId,
    /// 起局方式（时间/数字/随机/手动）
    pub method: DivinationMethod,

    // ==================== 隐私控制字段（v3.4 新增） ====================
    /// 隐私模式（Public/Partial/Private）
    ///
    /// - Public: 所有数据明文，任何人可见
    /// - Partial: 计算数据明文 + 敏感数据加密（推荐）
    /// - Private: 全部数据加密
    pub privacy_mode: PrivacyMode,

    /// 加密字段标志位（仅 Partial 模式使用）
    ///
    /// 使用 `EncryptedFields` 模块中的常量组合：
    /// - `None`: Public 或 Private 模式
    /// - `Some(flags)`: Partial 模式，flags 标识哪些敏感字段被加密
    ///
    /// 例如：`Some(EncryptedFields::NAME | EncryptedFields::QUESTION)`
    pub encrypted_fields: Option<u16>,

    /// 敏感数据哈希（用于完整性验证）
    ///
    /// 对原始敏感数据进行 SHA-256 哈希，用于：
    /// - 验证解密后数据完整性
    /// - 防止授权方篡改数据
    pub sensitive_data_hash: Option<[u8; 32]>,

    // ==================== 命主敏感信息 ====================
    // Partial/Private 模式下为 None，加密存储在 Privacy pallet
    /// 命主姓名（明文，可选）
    /// UTF-8 编码，最大 32 字节（约 10 个汉字）
    /// Partial/Private 模式下为 None
    pub name: Option<BoundedVec<u8, MaxNameLen>>,
    /// 命主性别（可选）
    /// Partial/Private 模式下为 None
    pub gender: Option<Gender>,
    /// 命主出生年份（可选，用于年命分析）
    /// 例如：1990 表示 1990 年出生
    /// Partial/Private 模式下为 None
    pub birth_year: Option<u16>,
    /// 占问事宜（明文，可选）
    /// UTF-8 编码，最大 128 字节（约 42 个汉字）
    /// Partial/Private 模式下为 None
    pub question: Option<BoundedVec<u8, MaxQuestionLen>>,
    /// 问事类型（用于确定用神）
    pub question_type: Option<QuestionType>,
    /// 排盘方法（转盘/飞盘）
    pub pan_method: PanMethod,

    // ==================== 起局时间（计算数据） ====================
    // Partial 模式下明文存储，Private 模式下为 None
    /// 年柱（Private 模式下为 None）
    pub year_ganzhi: Option<GanZhi>,
    /// 月柱（Private 模式下为 None）
    pub month_ganzhi: Option<GanZhi>,
    /// 日柱（Private 模式下为 None）
    pub day_ganzhi: Option<GanZhi>,
    /// 时柱（Private 模式下为 None）
    pub hour_ganzhi: Option<GanZhi>,
    /// 节气（Private 模式下为 None）
    pub jie_qi: Option<JieQi>,

    // ==================== 局数信息（计算数据） ====================
    // Partial 模式下明文存储，Private 模式下为 None
    /// 阴阳遁（Private 模式下为 None）
    pub dun_type: Option<DunType>,
    /// 三元（Private 模式下为 None）
    pub san_yuan: Option<SanYuan>,
    /// 局数（1-9）（Private 模式下为 None）
    pub ju_number: Option<u8>,

    // ==================== 盘面数据（计算数据） ====================
    // Partial 模式下明文存储，Private 模式下为 None
    /// 值符星（Private 模式下为 None）
    pub zhi_fu_xing: Option<JiuXing>,
    /// 值使门（Private 模式下为 None）
    pub zhi_shi_men: Option<BaMen>,
    /// 九宫排盘结果（Private 模式下为 None）
    pub palaces: Option<[Palace; 9]>,

    // ==================== 元数据 ====================
    /// 排盘时间戳（秒）
    pub timestamp: u64,
    /// 创建区块号
    pub block_number: BlockNumber,
    /// AI 解读 IPFS CID（可选）
    pub interpretation_cid: Option<BoundedVec<u8, MaxCidLen>>,
    /// 问题哈希（隐私保护，用于验证）
    /// 用于在不暴露原始问题的情况下进行验证
    pub question_hash: [u8; 32],
}

/// QimenChart 辅助方法
///
/// 提供便捷的方法来访问可选字段，
/// 主要用于解读模块中安全地处理不同隐私模式下的数据访问。
impl<AccountId, BlockNumber, MaxCidLen: frame_support::traits::Get<u32>>
    QimenChart<AccountId, BlockNumber, MaxCidLen>
{
    /// 检查是否有计算数据可用
    ///
    /// 对于 Public 和 Partial 模式，计算数据应该可用。
    /// 对于 Private 模式，计算数据为 None。
    pub fn has_calculation_data(&self) -> bool {
        self.palaces.is_some()
            && self.year_ganzhi.is_some()
            && self.zhi_fu_xing.is_some()
            && self.zhi_shi_men.is_some()
    }

    /// 检查是否可以进行解读
    ///
    /// 只有当计算数据可用时（Public 或 Partial 模式），才能进行解读。
    /// Private 模式需要前端解密后调用 compute_chart API。
    pub fn can_interpret(&self) -> bool {
        use pallet_divination_privacy::types::PrivacyMode;
        match self.privacy_mode {
            PrivacyMode::Public | PrivacyMode::Partial => self.has_calculation_data(),
            PrivacyMode::Private => false,
        }
    }

    /// 获取九宫排盘数据（用于解读）
    ///
    /// # 返回
    /// - `Some(&[Palace; 9])`: 如果数据可用
    /// - `None`: 如果是 Private 模式或数据不可用
    pub fn get_palaces(&self) -> Option<&[Palace; 9]> {
        self.palaces.as_ref()
    }

    /// 获取日柱干支（用于确定用神）
    pub fn get_day_ganzhi(&self) -> Option<GanZhi> {
        self.day_ganzhi
    }

    /// 获取时柱干支
    pub fn get_hour_ganzhi(&self) -> Option<GanZhi> {
        self.hour_ganzhi
    }

    /// 获取节气
    pub fn get_jie_qi(&self) -> Option<JieQi> {
        self.jie_qi
    }

    /// 获取值符星
    pub fn get_zhi_fu_xing(&self) -> Option<JiuXing> {
        self.zhi_fu_xing
    }

    /// 获取值使门
    pub fn get_zhi_shi_men(&self) -> Option<BaMen> {
        self.zhi_shi_men
    }

    /// 获取阴阳遁类型
    pub fn get_dun_type(&self) -> Option<DunType> {
        self.dun_type
    }

    /// 检查是否是公开模式
    pub fn is_public(&self) -> bool {
        use pallet_divination_privacy::types::PrivacyMode;
        self.privacy_mode == PrivacyMode::Public
    }
}

/// 用户统计信息
#[derive(Clone, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct UserStats {
    /// 总排盘次数
    pub total_charts: u64,
    /// 阳遁次数
    pub yang_dun_count: u64,
    /// 阴遁次数
    pub yin_dun_count: u64,
    /// 获得吉门次数
    pub lucky_door_count: u64,
    /// 获得吉星次数
    pub lucky_star_count: u64,
}

impl UserStats {
    /// 更新统计数据
    ///
    /// # 参数
    /// - `dun_type`: 阴阳遁类型
    /// - `zhi_fu_xing`: 值符星
    /// - `zhi_shi_men`: 值使门
    pub fn update_from_chart(
        &mut self,
        dun_type: DunType,
        zhi_fu_xing: JiuXing,
        zhi_shi_men: BaMen,
    ) {
        self.total_charts = self.total_charts.saturating_add(1);

        match dun_type {
            DunType::Yang => self.yang_dun_count = self.yang_dun_count.saturating_add(1),
            DunType::Yin => self.yin_dun_count = self.yin_dun_count.saturating_add(1),
        }

        if zhi_shi_men.is_auspicious() {
            self.lucky_door_count = self.lucky_door_count.saturating_add(1);
        }

        if zhi_fu_xing.is_auspicious() {
            self.lucky_star_count = self.lucky_star_count.saturating_add(1);
        }
    }
}

// ==================== 解卦相关类型 ====================

/// 格局类型
///
/// 奇门遁甲中的各种格局，影响吉凶判断
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum GeJuType {
    /// 正格 - 常规格局
    ZhengGe = 0,
    /// 伏吟格 - 天盘地盘相同
    FuYinGe = 1,
    /// 反吟格 - 天盘地盘对冲
    FanYinGe = 2,
    /// 天遁格 - 丙奇+天心星+开门
    TianDunGe = 3,
    /// 地遁格 - 乙奇+六合+开门
    DiDunGe = 4,
    /// 人遁格 - 丁奇+太阴+开门
    RenDunGe = 5,
    /// 鬼遁格 - 丁奇+天心星+开门
    GuiDunGe = 6,
    /// 神遁格 - 九天+值符+开门
    ShenDunGe = 7,
    /// 龙遁格 - 九地+值符+开门
    LongDunGe = 8,
    /// 青龙返首 - 特殊吉格
    QingLongFanShou = 9,
    /// 飞鸟跌穴 - 特殊凶格
    FeiNiaoDieXue = 10,
}

impl GeJuType {
    /// 获取格局名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::ZhengGe => "正格",
            Self::FuYinGe => "伏吟格",
            Self::FanYinGe => "反吟格",
            Self::TianDunGe => "天遁格",
            Self::DiDunGe => "地遁格",
            Self::RenDunGe => "人遁格",
            Self::GuiDunGe => "鬼遁格",
            Self::ShenDunGe => "神遁格",
            Self::LongDunGe => "龙遁格",
            Self::QingLongFanShou => "青龙返首",
            Self::FeiNiaoDieXue => "飞鸟跌穴",
        }
    }

    /// 判断是否为吉格
    pub fn is_auspicious(&self) -> bool {
        matches!(
            self,
            Self::TianDunGe
                | Self::DiDunGe
                | Self::RenDunGe
                | Self::ShenDunGe
                | Self::LongDunGe
                | Self::QingLongFanShou
        )
    }

    /// 判断是否为凶格
    pub fn is_inauspicious(&self) -> bool {
        matches!(self, Self::FuYinGe | Self::FanYinGe | Self::FeiNiaoDieXue)
    }
}

impl Default for GeJuType {
    fn default() -> Self {
        Self::ZhengGe
    }
}

/// 旺衰状态
///
/// 根据节气和五行关系判断用神的旺衰程度
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum WangShuai {
    /// 旺相 - 得令得时，最强
    WangXiang = 0,
    /// 相 - 次旺
    Xiang = 1,
    /// 休 - 休息
    Xiu = 2,
    /// 囚 - 受制
    Qiu = 3,
    /// 死 - 最弱
    Si = 4,
}

impl WangShuai {
    /// 获取旺衰名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::WangXiang => "旺相",
            Self::Xiang => "相",
            Self::Xiu => "休",
            Self::Qiu => "囚",
            Self::Si => "死",
        }
    }

    /// 判断是否强旺
    pub fn is_strong(&self) -> bool {
        matches!(self, Self::WangXiang | Self::Xiang)
    }

    /// 判断是否衰弱
    pub fn is_weak(&self) -> bool {
        matches!(self, Self::Qiu | Self::Si)
    }
}

impl Default for WangShuai {
    fn default() -> Self {
        Self::Xiu
    }
}

/// 吉凶等级
///
/// 综合评估后的吉凶判断
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum Fortune {
    /// 大吉
    DaJi = 0,
    /// 中吉
    ZhongJi = 1,
    /// 小吉
    XiaoJi = 2,
    /// 平
    Ping = 3,
    /// 小凶
    XiaoXiong = 4,
    /// 中凶
    ZhongXiong = 5,
    /// 大凶
    DaXiong = 6,
}

impl Fortune {
    /// 获取吉凶名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::DaJi => "大吉",
            Self::ZhongJi => "中吉",
            Self::XiaoJi => "小吉",
            Self::Ping => "平",
            Self::XiaoXiong => "小凶",
            Self::ZhongXiong => "中凶",
            Self::DaXiong => "大凶",
        }
    }

    /// 判断是否吉利
    pub fn is_auspicious(&self) -> bool {
        matches!(self, Self::DaJi | Self::ZhongJi | Self::XiaoJi)
    }

    /// 判断是否凶险
    pub fn is_inauspicious(&self) -> bool {
        matches!(self, Self::XiaoXiong | Self::ZhongXiong | Self::DaXiong)
    }
}

impl Default for Fortune {
    fn default() -> Self {
        Self::Ping
    }
}

/// 星门关系
///
/// 九星与八门之间的五行生克关系
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum XingMenRelation {
    /// 星生门 - 吉
    XingShengMen = 0,
    /// 门生星 - 平
    MenShengXing = 1,
    /// 星克门 - 凶
    XingKeMen = 2,
    /// 门克星 - 平
    MenKeXing = 3,
    /// 比和 - 吉
    BiHe = 4,
}

impl XingMenRelation {
    /// 获取关系名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::XingShengMen => "星生门",
            Self::MenShengXing => "门生星",
            Self::XingKeMen => "星克门",
            Self::MenKeXing => "门克星",
            Self::BiHe => "比和",
        }
    }

    /// 判断是否吉利
    pub fn is_auspicious(&self) -> bool {
        matches!(self, Self::XingShengMen | Self::BiHe)
    }
}

impl Default for XingMenRelation {
    fn default() -> Self {
        Self::BiHe
    }
}

/// 用神类型
///
/// 根据问事类型确定的用神
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum YongShenType {
    /// 日干 - 代表自己
    RiGan,
    /// 时干 - 代表事情
    ShiGan,
    /// 值符 - 代表贵人
    ZhiFu,
    /// 值使 - 代表行动
    ZhiShi,
    /// 年命 - 代表本命
    NianMing,
    /// 特定星 - 根据问事类型
    SpecificXing(JiuXing),
    /// 特定门 - 根据问事类型
    SpecificMen(BaMen),
    /// 特定宫 - 根据问事类型
    SpecificGong(JiuGong),
}

impl YongShenType {
    /// 获取用神名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::RiGan => "日干",
            Self::ShiGan => "时干",
            Self::ZhiFu => "值符",
            Self::ZhiShi => "值使",
            Self::NianMing => "年命",
            Self::SpecificXing(_) => "特定星",
            Self::SpecificMen(_) => "特定门",
            Self::SpecificGong(_) => "特定宫",
        }
    }
}

impl Default for YongShenType {
    fn default() -> Self {
        Self::RiGan
    }
}

/// 得力状态
///
/// 用神在盘中的得力程度
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum DeLiStatus {
    /// 大得力 - 旺相+吉星吉门
    DaDeLi = 0,
    /// 得力 - 旺相或吉星吉门
    DeLi = 1,
    /// 平 - 休囚但无克
    Ping = 2,
    /// 失力 - 休囚+凶星凶门
    ShiLi = 3,
    /// 大失力 - 死绝+凶星凶门
    DaShiLi = 4,
}

impl DeLiStatus {
    /// 获取得力名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::DaDeLi => "大得力",
            Self::DeLi => "得力",
            Self::Ping => "平",
            Self::ShiLi => "失力",
            Self::DaShiLi => "大失力",
        }
    }

    /// 判断是否得力
    pub fn is_favorable(&self) -> bool {
        matches!(self, Self::DaDeLi | Self::DeLi)
    }
}

impl Default for DeLiStatus {
    fn default() -> Self {
        Self::Ping
    }
}

/// 应期单位
///
/// 事情应验的时间单位
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum YingQiUnit {
    /// 时辰
    Hour = 0,
    /// 日
    Day = 1,
    /// 旬（10天）
    Xun = 2,
    /// 月
    Month = 3,
    /// 季
    Season = 4,
    /// 年
    Year = 5,
}

impl YingQiUnit {
    /// 获取单位名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Hour => "时辰",
            Self::Day => "日",
            Self::Xun => "旬",
            Self::Month => "月",
            Self::Season => "季",
            Self::Year => "年",
        }
    }
}

impl Default for YingQiUnit {
    fn default() -> Self {
        Self::Day
    }
}

// ==================== Runtime API 返回类型 ====================

/// 临时排盘结果
///
/// `compute_chart` Runtime API 的返回类型。
/// 包含完整的排盘和解读结果，不存储敏感数据（姓名、问题等）。
///
/// # 使用场景
///
/// 1. **Private 模式解盘**：前端解密加密数据后，调用 `compute_chart` API
/// 2. **临时排盘**：用户想预览排盘结果但不想保存
/// 3. **预览功能**：在用户确认保存前展示排盘结果
///
/// # 与 QimenChart 的区别
///
/// - `QimenChart`：链上存储的完整记录，包含所有元数据
/// - `QimenChartResult`：临时计算结果，仅包含排盘数据，不存储
///
/// # 存储大小估算
/// - 四柱：8 bytes（4 × 2 bytes）
/// - 局数信息：5 bytes
/// - 盘面数据：约 180 bytes（9 宫 × 20 bytes）
/// - 解读结果：约 10 bytes
/// - **总计**：约 200 bytes
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct QimenChartResult {
    // ==================== 四柱 ====================
    /// 年柱
    pub year_ganzhi: GanZhi,
    /// 月柱
    pub month_ganzhi: GanZhi,
    /// 日柱
    pub day_ganzhi: GanZhi,
    /// 时柱
    pub hour_ganzhi: GanZhi,

    // ==================== 局数信息 ====================
    /// 节气
    pub jie_qi: JieQi,
    /// 阴阳遁
    pub dun_type: DunType,
    /// 三元
    pub san_yuan: SanYuan,
    /// 局数（1-9）
    pub ju_number: u8,

    // ==================== 盘面数据 ====================
    /// 值符星
    pub zhi_fu_xing: JiuXing,
    /// 值使门
    pub zhi_shi_men: BaMen,
    /// 九宫排盘结果
    pub palaces: [Palace; 9],

    // ==================== 解读结果（可选）====================
    /// 格局分析
    pub ge_ju: Option<GeJuType>,
    /// 综合吉凶
    pub fortune: Option<Fortune>,
    /// 用神得力状态
    pub yong_shen_status: Option<DeLiStatus>,
}
