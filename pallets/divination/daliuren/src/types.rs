//! # 大六壬排盘类型定义
//!
//! 本模块定义大六壬排盘系统的所有核心类型。
//!
//! ## 大六壬核心概念
//!
//! - **天盘**: 以月将加占时起式，十二地支顺时针旋转
//! - **四课**: 日干阳神、干阴神、日支阳神、支阴神
//! - **三传**: 初传、中传、末传（根据九种课式推导）
//! - **天将**: 十二天将（贵人为首，顺逆排布）
//! - **神煞**: 吉神凶煞判断

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

// ============================================================================
// 天干地支基础类型（复用六爻模块的设计）
// ============================================================================

/// 十天干
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
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

    /// 从索引获取天干
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

    /// 是否为阳干
    pub fn is_yang(&self) -> bool {
        self.index() % 2 == 0
    }

    /// 获取天干五行
    pub fn wu_xing(&self) -> WuXing {
        match self {
            Self::Jia | Self::Yi => WuXing::Wood,
            Self::Bing | Self::Ding => WuXing::Fire,
            Self::Wu | Self::Ji => WuXing::Earth,
            Self::Geng | Self::Xin => WuXing::Metal,
            Self::Ren | Self::Gui => WuXing::Water,
        }
    }

    /// 天干相加
    pub fn add(&self, n: i8) -> Self {
        let new_idx = ((self.index() as i8 + n).rem_euclid(10)) as u8;
        Self::from_index(new_idx)
    }
}

/// 十二地支
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
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

    /// 从索引获取地支
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

    /// 获取地支五行
    pub fn wu_xing(&self) -> WuXing {
        match self {
            Self::Zi | Self::Hai => WuXing::Water,
            Self::Chou | Self::Chen | Self::Wei | Self::Xu => WuXing::Earth,
            Self::Yin | Self::Mao => WuXing::Wood,
            Self::Si | Self::Wu => WuXing::Fire,
            Self::Shen | Self::You => WuXing::Metal,
        }
    }

    /// 地支相加
    pub fn add(&self, n: i8) -> Self {
        let new_idx = ((self.index() as i8 + n).rem_euclid(12)) as u8;
        Self::from_index(new_idx)
    }

    /// 地支相减
    pub fn sub(&self, other: Self) -> i8 {
        (self.index() as i8 - other.index() as i8 + 12) % 12
    }

    /// 六冲
    pub fn liu_chong(&self) -> Self {
        self.add(6)
    }

    /// 是否为六冲关系
    pub fn is_chong(&self, other: Self) -> bool {
        self.liu_chong() == other
    }

    /// 六合
    pub fn liu_he(&self) -> Self {
        // 子丑合、寅亥合、卯戌合、辰酉合、巳申合、午未合
        match self {
            Self::Zi => Self::Chou,
            Self::Chou => Self::Zi,
            Self::Yin => Self::Hai,
            Self::Mao => Self::Xu,
            Self::Chen => Self::You,
            Self::Si => Self::Shen,
            Self::Wu => Self::Wei,
            Self::Wei => Self::Wu,
            Self::Shen => Self::Si,
            Self::You => Self::Chen,
            Self::Xu => Self::Mao,
            Self::Hai => Self::Yin,
        }
    }

    /// 刑
    pub fn xing(&self) -> Self {
        match self {
            Self::Zi => Self::Mao,
            Self::Mao => Self::Zi,
            Self::Chou => Self::Xu,
            Self::Xu => Self::Wei,
            Self::Wei => Self::Chou,
            Self::Yin => Self::Si,
            Self::Si => Self::Shen,
            Self::Shen => Self::Yin,
            // 自刑
            Self::Chen => Self::Chen,
            Self::Wu => Self::Wu,
            Self::You => Self::You,
            Self::Hai => Self::Hai,
        }
    }

    /// 是否为孟（寅申巳亥）
    pub fn is_meng(&self) -> bool {
        matches!(self, Self::Yin | Self::Shen | Self::Si | Self::Hai)
    }

    /// 是否为仲（子午卯酉）
    pub fn is_zhong(&self) -> bool {
        matches!(self, Self::Zi | Self::Wu | Self::Mao | Self::You)
    }

    /// 是否为季（辰戌丑未）
    pub fn is_ji(&self) -> bool {
        matches!(self, Self::Chen | Self::Xu | Self::Chou | Self::Wei)
    }

    /// 获取驿马
    pub fn yi_ma(&self) -> Self {
        match self {
            Self::Shen | Self::Zi | Self::Chen => Self::Yin,
            Self::Yin | Self::Wu | Self::Xu => Self::Shen,
            Self::Si | Self::You | Self::Chou => Self::Hai,
            Self::Hai | Self::Mao | Self::Wei => Self::Si,
        }
    }
}

// ============================================================================
// 五行
// ============================================================================

/// 五行
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum WuXing {
    #[default]
    Wood = 0,  // 木
    Fire = 1,  // 火
    Earth = 2, // 土
    Metal = 3, // 金
    Water = 4, // 水
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

    /// 获取五行索引
    pub fn index(&self) -> u8 {
        *self as u8
    }

    /// 从索引获取五行
    pub fn from_index(index: u8) -> Self {
        match index % 5 {
            0 => Self::Wood,
            1 => Self::Fire,
            2 => Self::Earth,
            3 => Self::Metal,
            _ => Self::Water,
        }
    }

    /// 所生五行
    pub fn generates(&self) -> Self {
        match self {
            Self::Wood => Self::Fire,
            Self::Fire => Self::Earth,
            Self::Earth => Self::Metal,
            Self::Metal => Self::Water,
            Self::Water => Self::Wood,
        }
    }

    /// 所克五行
    pub fn restrains(&self) -> Self {
        match self {
            Self::Wood => Self::Earth,
            Self::Fire => Self::Metal,
            Self::Earth => Self::Water,
            Self::Metal => Self::Wood,
            Self::Water => Self::Fire,
        }
    }

    /// 生我者
    pub fn generated_by(&self) -> Self {
        match self {
            Self::Wood => Self::Water,
            Self::Fire => Self::Wood,
            Self::Earth => Self::Fire,
            Self::Metal => Self::Earth,
            Self::Water => Self::Metal,
        }
    }

    /// 克我者
    pub fn restrained_by(&self) -> Self {
        match self {
            Self::Wood => Self::Metal,
            Self::Fire => Self::Water,
            Self::Earth => Self::Wood,
            Self::Metal => Self::Fire,
            Self::Water => Self::Earth,
        }
    }

    /// 是否克另一五行
    pub fn ke(&self, other: Self) -> bool {
        self.restrains() == other
    }

    /// 是否生另一五行
    pub fn sheng(&self, other: Self) -> bool {
        self.generates() == other
    }
}

// ============================================================================
// 十二天将
// ============================================================================

/// 十二天将
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum TianJiang {
    /// 贵人（土）
    #[default]
    GuiRen = 0,
    /// 螣蛇（火）
    TengShe = 1,
    /// 朱雀（火）
    ZhuQue = 2,
    /// 六合（木）
    LiuHe = 3,
    /// 勾陈（土）
    GouChen = 4,
    /// 青龙（木）
    QingLong = 5,
    /// 天空（土）
    TianKong = 6,
    /// 白虎（金）
    BaiHu = 7,
    /// 太常（土）
    TaiChang = 8,
    /// 玄武（水）
    XuanWu = 9,
    /// 太阴（金）
    TaiYin = 10,
    /// 天后（水）
    TianHou = 11,
}

impl TianJiang {
    /// 获取天将名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::GuiRen => "贵人",
            Self::TengShe => "螣蛇",
            Self::ZhuQue => "朱雀",
            Self::LiuHe => "六合",
            Self::GouChen => "勾陈",
            Self::QingLong => "青龙",
            Self::TianKong => "天空",
            Self::BaiHu => "白虎",
            Self::TaiChang => "太常",
            Self::XuanWu => "玄武",
            Self::TaiYin => "太阴",
            Self::TianHou => "天后",
        }
    }

    /// 获取简称
    pub fn short_name(&self) -> &'static str {
        match self {
            Self::GuiRen => "贵",
            Self::TengShe => "蛇",
            Self::ZhuQue => "雀",
            Self::LiuHe => "合",
            Self::GouChen => "勾",
            Self::QingLong => "龙",
            Self::TianKong => "空",
            Self::BaiHu => "虎",
            Self::TaiChang => "常",
            Self::XuanWu => "玄",
            Self::TaiYin => "阴",
            Self::TianHou => "后",
        }
    }

    /// 获取天将索引
    pub fn index(&self) -> u8 {
        *self as u8
    }

    /// 从索引获取天将
    pub fn from_index(index: u8) -> Self {
        match index % 12 {
            0 => Self::GuiRen,
            1 => Self::TengShe,
            2 => Self::ZhuQue,
            3 => Self::LiuHe,
            4 => Self::GouChen,
            5 => Self::QingLong,
            6 => Self::TianKong,
            7 => Self::BaiHu,
            8 => Self::TaiChang,
            9 => Self::XuanWu,
            10 => Self::TaiYin,
            _ => Self::TianHou,
        }
    }

    /// 获取天将五行
    pub fn wu_xing(&self) -> WuXing {
        match self {
            Self::GuiRen | Self::GouChen | Self::TianKong | Self::TaiChang => WuXing::Earth,
            Self::TengShe | Self::ZhuQue => WuXing::Fire,
            Self::LiuHe | Self::QingLong => WuXing::Wood,
            Self::BaiHu | Self::TaiYin => WuXing::Metal,
            Self::XuanWu | Self::TianHou => WuXing::Water,
        }
    }

    /// 是否为吉将
    pub fn is_auspicious(&self) -> bool {
        matches!(
            self,
            Self::GuiRen | Self::LiuHe | Self::QingLong | Self::TaiChang | Self::TaiYin | Self::TianHou
        )
    }

    /// 天将相加
    pub fn add(&self, n: i8) -> Self {
        let new_idx = ((self.index() as i8 + n).rem_euclid(12)) as u8;
        Self::from_index(new_idx)
    }
}

// ============================================================================
// 月将
// ============================================================================

/// 月将（十二神）
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum YueJiang {
    /// 神后（子）
    #[default]
    ShenHou = 0,
    /// 大吉（丑）
    DaJi = 1,
    /// 功曹（寅）
    GongCao = 2,
    /// 太冲（卯）
    TaiChong = 3,
    /// 天罡（辰）
    TianGang = 4,
    /// 太乙（巳）
    TaiYi = 5,
    /// 胜光（午）
    ShengGuang = 6,
    /// 小吉（未）
    XiaoJi = 7,
    /// 传送（申）
    ChuanSong = 8,
    /// 从魁（酉）
    CongKui = 9,
    /// 河魁（戌）
    HeKui = 10,
    /// 登明（亥）
    DengMing = 11,
}

impl YueJiang {
    /// 获取月将名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::ShenHou => "神后",
            Self::DaJi => "大吉",
            Self::GongCao => "功曹",
            Self::TaiChong => "太冲",
            Self::TianGang => "天罡",
            Self::TaiYi => "太乙",
            Self::ShengGuang => "胜光",
            Self::XiaoJi => "小吉",
            Self::ChuanSong => "传送",
            Self::CongKui => "从魁",
            Self::HeKui => "河魁",
            Self::DengMing => "登明",
        }
    }

    /// 获取月将对应的地支
    pub fn to_dizhi(&self) -> DiZhi {
        DiZhi::from_index(*self as u8)
    }

    /// 从地支获取月将
    pub fn from_dizhi(zhi: DiZhi) -> Self {
        match zhi.index() {
            0 => Self::ShenHou,
            1 => Self::DaJi,
            2 => Self::GongCao,
            3 => Self::TaiChong,
            4 => Self::TianGang,
            5 => Self::TaiYi,
            6 => Self::ShengGuang,
            7 => Self::XiaoJi,
            8 => Self::ChuanSong,
            9 => Self::CongKui,
            10 => Self::HeKui,
            _ => Self::DengMing,
        }
    }

    /// 获取索引
    pub fn index(&self) -> u8 {
        *self as u8
    }
}

// ============================================================================
// 六亲
// ============================================================================

/// 六亲
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum LiuQin {
    /// 兄弟 - 同我者
    #[default]
    XiongDi = 0,
    /// 父母 - 生我者
    FuMu = 1,
    /// 官鬼 - 克我者
    GuanGui = 2,
    /// 妻财 - 我克者
    QiCai = 3,
    /// 子孙 - 我生者
    ZiSun = 4,
}

impl LiuQin {
    /// 获取六亲名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::XiongDi => "兄",
            Self::FuMu => "父",
            Self::GuanGui => "官",
            Self::QiCai => "财",
            Self::ZiSun => "子",
        }
    }

    /// 根据日干五行和爻五行计算六亲
    pub fn from_wu_xing(day_wx: WuXing, target_wx: WuXing) -> Self {
        if day_wx == target_wx {
            Self::XiongDi
        } else if day_wx.sheng(target_wx) {
            Self::ZiSun
        } else if day_wx.ke(target_wx) {
            Self::QiCai
        } else if target_wx.ke(day_wx) {
            Self::GuanGui
        } else {
            Self::FuMu
        }
    }
}

// ============================================================================
// 旺衰
// ============================================================================

/// 旺衰状态
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum WangShuai {
    /// 旺
    #[default]
    Wang = 0,
    /// 相
    Xiang = 1,
    /// 休
    Xiu = 2,
    /// 囚
    Qiu = 3,
    /// 死
    Si = 4,
}

impl WangShuai {
    /// 获取旺衰名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Wang => "旺",
            Self::Xiang => "相",
            Self::Xiu => "休",
            Self::Qiu => "囚",
            Self::Si => "死",
        }
    }

    /// 根据月令五行和目标五行计算旺衰
    pub fn from_wu_xing(month_wx: WuXing, target_wx: WuXing) -> Self {
        if month_wx == target_wx {
            Self::Wang
        } else if month_wx.sheng(target_wx) {
            Self::Xiang
        } else if target_wx.sheng(month_wx) {
            Self::Xiu
        } else if target_wx.ke(month_wx) {
            Self::Qiu
        } else {
            Self::Si
        }
    }
}

// ============================================================================
// 课式类型
// ============================================================================

/// 课式类型（九种取三传方法）
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum KeShiType {
    /// 贼克课（下克上为贼）
    #[default]
    ZeiKe = 0,
    /// 比用课
    BiYong = 1,
    /// 涉害课
    SheHai = 2,
    /// 遥克课
    YaoKe = 3,
    /// 昂星课（虎视/冬蛇掩目）
    AngXing = 4,
    /// 别责课
    BieZe = 5,
    /// 八专课
    BaZhuan = 6,
    /// 伏吟课
    FuYin = 7,
    /// 返吟课
    FanYin = 8,
}

impl KeShiType {
    /// 获取课式名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::ZeiKe => "贼克",
            Self::BiYong => "比用",
            Self::SheHai => "涉害",
            Self::YaoKe => "遥克",
            Self::AngXing => "昂星",
            Self::BieZe => "别责",
            Self::BaZhuan => "八专",
            Self::FuYin => "伏吟",
            Self::FanYin => "返吟",
        }
    }
}

// ============================================================================
// 格局类型
// ============================================================================

/// 格局类型
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum GeJuType {
    /// 元首课（一上克下）
    #[default]
    YuanShou = 0,
    /// 重审课（一下贼上）
    ChongShen = 1,
    /// 知一课（比用取一）
    ZhiYi = 2,
    /// 涉害课（涉害深者）
    SheHaiGe = 3,
    /// 见机课（涉害同取孟）
    JianJi = 4,
    /// 察微课（涉害同取仲）
    ChaWei = 5,
    /// 复等课（涉害同取干/支阳神）
    FuDeng = 6,
    /// 遥克课（二三四课克日干）
    YaoKeGe = 7,
    /// 虎视课（昂星阳干）
    HuShi = 8,
    /// 冬蛇掩目（昂星阴干）
    DongSheYanMu = 9,
    /// 别责课（三课备）
    BieZeGe = 10,
    /// 八专课（八专日）
    BaZhuanGe = 11,
    /// 自任课（伏吟阳干）
    ZiRen = 12,
    /// 自信课（伏吟阴干）
    ZiXin = 13,
    /// 无依课（返吟无克）
    WuYi = 14,
}

impl GeJuType {
    /// 获取格局名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::YuanShou => "元首",
            Self::ChongShen => "重审",
            Self::ZhiYi => "知一",
            Self::SheHaiGe => "涉害",
            Self::JianJi => "见机",
            Self::ChaWei => "察微",
            Self::FuDeng => "复等",
            Self::YaoKeGe => "遥克",
            Self::HuShi => "虎视",
            Self::DongSheYanMu => "冬蛇掩目",
            Self::BieZeGe => "别责",
            Self::BaZhuanGe => "八专",
            Self::ZiRen => "自任",
            Self::ZiXin => "自信",
            Self::WuYi => "无依",
        }
    }
}

// ============================================================================
// 起课方式
// ============================================================================

/// 起课方式
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum DivinationMethod {
    /// 时间起课
    #[default]
    TimeMethod = 0,
    /// 随机起课
    RandomMethod = 1,
    /// 手动指定
    ManualMethod = 2,
}

impl DivinationMethod {
    /// 获取起课方式名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::TimeMethod => "时间起课",
            Self::RandomMethod => "随机起课",
            Self::ManualMethod => "手动指定",
        }
    }
}

// ============================================================================
// 四课信息
// ============================================================================

/// 单课信息
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct KeInfo {
    /// 上神（天盘）
    pub shang: DiZhi,
    /// 下神（地盘）
    pub xia: DiZhi,
    /// 天将
    pub tian_jiang: TianJiang,
    /// 六亲
    pub liu_qin: LiuQin,
}

/// 四课
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct SiKe {
    /// 第一课（日干阳神）
    pub ke1: KeInfo,
    /// 第二课（干阴神）
    pub ke2: KeInfo,
    /// 第三课（日支阳神）
    pub ke3: KeInfo,
    /// 第四课（支阴神）
    pub ke4: KeInfo,
}

// ============================================================================
// 三传信息
// ============================================================================

/// 三传
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct SanChuan {
    /// 初传
    pub chu: DiZhi,
    /// 中传
    pub zhong: DiZhi,
    /// 末传
    pub mo: DiZhi,
    /// 初传天将
    pub chu_jiang: TianJiang,
    /// 中传天将
    pub zhong_jiang: TianJiang,
    /// 末传天将
    pub mo_jiang: TianJiang,
    /// 初传六亲
    pub chu_qin: LiuQin,
    /// 中传六亲
    pub zhong_qin: LiuQin,
    /// 末传六亲
    pub mo_qin: LiuQin,
    /// 初传遁干
    pub chu_dun: Option<TianGan>,
    /// 中传遁干
    pub zhong_dun: Option<TianGan>,
    /// 末传遁干
    pub mo_dun: Option<TianGan>,
}

// ============================================================================
// 天盘
// ============================================================================

/// 天盘（十二地支在天盘上的位置）
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct TianPan {
    /// 天盘十二宫（索引为地盘地支，值为天盘地支）
    pub positions: [DiZhi; 12],
}

impl TianPan {
    /// 获取地盘某支上的天盘支
    pub fn get(&self, di_zhi: DiZhi) -> DiZhi {
        self.positions[di_zhi.index() as usize]
    }

    /// 获取天盘支所临的地盘支
    pub fn lin(&self, tian_zhi: DiZhi) -> DiZhi {
        for i in 0..12 {
            if self.positions[i] == tian_zhi {
                return DiZhi::from_index(i as u8);
            }
        }
        DiZhi::Zi
    }
}

// ============================================================================
// 天将盘
// ============================================================================

/// 天将盘
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct TianJiangPan {
    /// 天将盘十二宫（索引为地支，值为天将）
    pub positions: [TianJiang; 12],
    /// 是否逆布
    pub is_reverse: bool,
}

impl TianJiangPan {
    /// 获取某地支上的天将
    pub fn get(&self, zhi: DiZhi) -> TianJiang {
        self.positions[zhi.index() as usize]
    }
}

// ============================================================================
// 大六壬式盘
// ============================================================================

/// 大六壬式盘
///
/// 存储完整的大六壬排盘结果
///
/// ## 隐私模式说明
///
/// 支持三种隐私模式：
/// - **Public**: 所有数据明文存储，任何人可查看
/// - **Partial**: 计算数据明文，敏感数据（问题内容等）加密
/// - **Private**: 所有数据加密，仅存储元数据
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
#[scale_info(skip_type_params(MaxCidLen))]
pub struct DaLiuRenPan<AccountId, BlockNumber, MaxCidLen: frame_support::traits::Get<u32>> {
    /// 式盘 ID
    pub id: u64,
    /// 创建者账户
    pub creator: AccountId,
    /// 创建区块
    pub created_at: BlockNumber,

    // ============ 隐私控制字段 ============

    /// 隐私模式（必有）
    pub privacy_mode: pallet_divination_privacy::types::PrivacyMode,
    /// 加密字段位图（可选，Partial 模式使用）
    /// bit 0: question_cid 已加密
    /// bit 1: 时间信息已加密（Private 模式）
    pub encrypted_fields: Option<u8>,
    /// 敏感数据哈希（用于验证完整性）
    pub sensitive_data_hash: Option<[u8; 32]>,

    // ===== 起课信息 =====
    /// 起课方式
    pub method: DivinationMethod,
    /// 占问事项（可选，链下存储）
    pub question_cid: Option<BoundedVec<u8, MaxCidLen>>,

    // ===== 时间信息（Private 模式时为 None）=====
    /// 年干支
    pub year_gz: Option<(TianGan, DiZhi)>,
    /// 月干支
    pub month_gz: Option<(TianGan, DiZhi)>,
    /// 日干支
    pub day_gz: Option<(TianGan, DiZhi)>,
    /// 时干支
    pub hour_gz: Option<(TianGan, DiZhi)>,

    // ===== 起课参数（Private 模式时为 None）=====
    /// 月将
    pub yue_jiang: Option<DiZhi>,
    /// 占时
    pub zhan_shi: Option<DiZhi>,
    /// 是否昼占
    pub is_day: Option<bool>,

    // ===== 式盘信息（Private 模式时为 None）=====
    /// 天盘
    pub tian_pan: Option<TianPan>,
    /// 天将盘
    pub tian_jiang_pan: Option<TianJiangPan>,
    /// 四课
    pub si_ke: Option<SiKe>,
    /// 三传
    pub san_chuan: Option<SanChuan>,

    // ===== 课式与格局（Private 模式时为 None）=====
    /// 课式类型
    pub ke_shi: Option<KeShiType>,
    /// 格局类型
    pub ge_ju: Option<GeJuType>,

    // ===== 空亡（Private 模式时为 None）=====
    /// 日旬空（两个地支）
    pub xun_kong: Option<(DiZhi, DiZhi)>,

    // ===== AI 解读 =====
    /// AI 解读 CID
    pub ai_interpretation_cid: Option<BoundedVec<u8, MaxCidLen>>,
}

impl<AccountId, BlockNumber, MaxCidLen: frame_support::traits::Get<u32>>
    DaLiuRenPan<AccountId, BlockNumber, MaxCidLen>
{
    /// 检查是否有计算数据（用于解盘）
    pub fn has_calculation_data(&self) -> bool {
        self.san_chuan.is_some()
    }

    /// 检查是否可解读
    ///
    /// Private 模式无计算数据，无法解读
    pub fn can_interpret(&self) -> bool {
        self.san_chuan.is_some() && self.si_ke.is_some()
    }

    /// 检查是否公开
    pub fn is_public(&self) -> bool {
        matches!(
            self.privacy_mode,
            pallet_divination_privacy::types::PrivacyMode::Public
        )
    }

    /// 检查是否完全私有
    pub fn is_private(&self) -> bool {
        matches!(
            self.privacy_mode,
            pallet_divination_privacy::types::PrivacyMode::Private
        )
    }
}

// ============================================================================
// 用户统计
// ============================================================================

/// 用户统计数据
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct UserStats {
    /// 总起课次数
    pub total_pans: u32,
    /// AI 解读次数
    pub ai_interpretations: u32,
    /// 首次起课区块
    pub first_pan_block: u32,
}

// ============================================================================
// 天干寄宫表
// ============================================================================

/// 天干寄宫（用于计算四课）
pub const GAN_JI_GONG: [(TianGan, DiZhi); 10] = [
    (TianGan::Jia, DiZhi::Yin),   // 甲寄寅
    (TianGan::Yi, DiZhi::Chen),   // 乙寄辰
    (TianGan::Bing, DiZhi::Si),   // 丙寄巳
    (TianGan::Ding, DiZhi::Wei),  // 丁寄未
    (TianGan::Wu, DiZhi::Si),     // 戊寄巳
    (TianGan::Ji, DiZhi::Wei),    // 己寄未
    (TianGan::Geng, DiZhi::Shen), // 庚寄申
    (TianGan::Xin, DiZhi::Xu),    // 辛寄戌
    (TianGan::Ren, DiZhi::Hai),   // 壬寄亥
    (TianGan::Gui, DiZhi::Chou),  // 癸寄丑
];

/// 获取天干寄宫
pub fn get_ji_gong(gan: TianGan) -> DiZhi {
    GAN_JI_GONG[gan.index() as usize].1
}

/// 获取某地支上寄宫的天干列表
///
/// 用于涉害法计算，需要知道某地支上有哪些天干寄宫
/// 例如：巳上有丙、戊寄宫；未上有丁、己寄宫
pub fn get_gan_of_ji_gong(zhi: DiZhi) -> Vec<TianGan> {
    let mut result = Vec::new();
    for i in 0..10 {
        let gan = TianGan::from_index(i);
        if get_ji_gong(gan) == zhi {
            result.push(gan);
        }
    }
    result
}

// ============================================================================
// 贵人起法
// ============================================================================

/// 昼贵人表
pub const ZHOU_GUI: [(TianGan, DiZhi); 10] = [
    (TianGan::Jia, DiZhi::Wei),  // 甲昼贵未
    (TianGan::Yi, DiZhi::Shen),  // 乙昼贵申
    (TianGan::Bing, DiZhi::You), // 丙昼贵酉
    (TianGan::Ding, DiZhi::Hai), // 丁昼贵亥
    (TianGan::Wu, DiZhi::Chou),  // 戊昼贵丑
    (TianGan::Ji, DiZhi::Zi),    // 己昼贵子
    (TianGan::Geng, DiZhi::Chou),// 庚昼贵丑
    (TianGan::Xin, DiZhi::Yin),  // 辛昼贵寅
    (TianGan::Ren, DiZhi::Mao),  // 壬昼贵卯
    (TianGan::Gui, DiZhi::Si),   // 癸昼贵巳
];

/// 夜贵人表
pub const YE_GUI: [(TianGan, DiZhi); 10] = [
    (TianGan::Jia, DiZhi::Chou), // 甲夜贵丑
    (TianGan::Yi, DiZhi::Zi),    // 乙夜贵子
    (TianGan::Bing, DiZhi::Hai), // 丙夜贵亥
    (TianGan::Ding, DiZhi::You), // 丁夜贵酉
    (TianGan::Wu, DiZhi::Wei),   // 戊夜贵未
    (TianGan::Ji, DiZhi::Shen),  // 己夜贵申
    (TianGan::Geng, DiZhi::Wei), // 庚夜贵未
    (TianGan::Xin, DiZhi::Wu),   // 辛夜贵午
    (TianGan::Ren, DiZhi::Si),   // 壬夜贵巳
    (TianGan::Gui, DiZhi::Mao),  // 癸夜贵卯
];

/// 获取贵人所临地支
pub fn get_gui_ren(gan: TianGan, is_day: bool) -> DiZhi {
    if is_day {
        ZHOU_GUI[gan.index() as usize].1
    } else {
        YE_GUI[gan.index() as usize].1
    }
}

// ============================================================================
// 八专日
// ============================================================================

/// 八专日列表（甲寅、庚申、丁未、己未四日）
pub const BA_ZHUAN_DAYS: [(TianGan, DiZhi); 4] = [
    (TianGan::Jia, DiZhi::Yin),
    (TianGan::Geng, DiZhi::Shen),
    (TianGan::Ding, DiZhi::Wei),
    (TianGan::Ji, DiZhi::Wei),
];

/// 判断是否为八专日
pub fn is_ba_zhuan_day(gan: TianGan, zhi: DiZhi) -> bool {
    BA_ZHUAN_DAYS.iter().any(|(g, z)| *g == gan && *z == zhi)
}

// ============================================================================
// 神煞系统
// ============================================================================

/// 神煞信息
///
/// 大六壬神煞分为：
/// - 年神煞：根据太岁起算
/// - 月神煞：根据月建起算
/// - 日神煞：根据日干支起算
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct ShenShaInfo {
    // ===== 日神煞 =====
    /// 驿马（日支起）
    pub yi_ma: DiZhi,
    /// 旬空（两个）
    pub xun_kong: (DiZhi, DiZhi),
    /// 旬奇
    pub xun_qi: Option<DiZhi>,
    /// 旬仪
    pub xun_yi: DiZhi,
    /// 日奇
    pub ri_qi: DiZhi,
    /// 支仪
    pub zhi_yi: DiZhi,
    /// 天罗
    pub tian_luo: DiZhi,
    /// 地网
    pub di_wang: DiZhi,

    // ===== 月神煞 =====
    /// 月驿马
    pub yue_yi_ma: DiZhi,
    /// 天马
    pub tian_ma: DiZhi,
    /// 皇书
    pub huang_shu: DiZhi,
    /// 皇恩
    pub huang_en: DiZhi,
    /// 天诏/飞魂
    pub tian_zhao: DiZhi,
    /// 天喜
    pub tian_xi: DiZhi,
    /// 生气
    pub sheng_qi: DiZhi,
    /// 死气/谩语
    pub si_qi: DiZhi,
    /// 三丘
    pub san_qiu: DiZhi,
    /// 五墓
    pub wu_mu: DiZhi,
    /// 孤辰
    pub gu_chen: DiZhi,
    /// 寡宿
    pub gua_su: DiZhi,
    /// 天医/天巫
    pub tian_yi_shen: DiZhi,
    /// 地医/地巫
    pub di_yi: DiZhi,
    /// 破碎
    pub po_sui: Option<DiZhi>,
    /// 月厌
    pub yue_yan: DiZhi,
    /// 血支
    pub xue_zhi: DiZhi,
    /// 血忌
    pub xue_ji: DiZhi,
    /// 丧车
    pub sang_che: DiZhi,
    /// 丧魂
    pub sang_hun: DiZhi,
    /// 天鬼
    pub tian_gui: DiZhi,
    /// 信神
    pub xin_shen: DiZhi,
    /// 天鸡
    pub tian_ji: DiZhi,
    /// 大时
    pub da_shi: DiZhi,
    /// 小时
    pub xiao_shi: DiZhi,

    // ===== 年神煞 =====
    /// 年驿马
    pub nian_yi_ma: DiZhi,
    /// 大耗
    pub da_hao: DiZhi,
    /// 小耗
    pub xiao_hao: DiZhi,
    /// 病符
    pub bing_fu: DiZhi,
}

// ============================================================================
// 年命信息
// ============================================================================

/// 年命信息（用于命盘占断）
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct NianMingInfo {
    /// 本命（出生年干支）
    pub ben_ming: (TianGan, DiZhi),
    /// 行年（当年的行年干支）
    pub xing_nian: (TianGan, DiZhi),
    /// 性别（true=男，false=女）
    pub is_male: bool,
}
