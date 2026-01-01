//! # 六爻排盘类型定义
//!
//! 本模块定义六爻排盘系统的所有核心类型。
//!
//! ## 核心概念
//!
//! - **六爻**: 从下往上依次为初爻、二爻、三爻、四爻、五爻、上爻
//! - **本卦/变卦**: 本卦是原始卦象，动爻变化后形成变卦
//! - **纳甲**: 八卦配天干地支的方法
//! - **六亲**: 父母、兄弟、子孙、妻财、官鬼
//! - **六神**: 青龙、朱雀、勾陈、螣蛇、白虎、玄武
//! - **世应**: 世爻代表自己，应爻代表对方

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
}

// ============================================================================
// 五行
// ============================================================================

/// 五行
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
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
}

// ============================================================================
// 八卦（经卦）
// ============================================================================

/// 八卦（三爻卦/经卦）
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum Trigram {
    #[default]
    Qian = 0,  // 乾 ☰ 111
    Dui = 1,   // 兑 ☱ 110
    Li = 2,    // 离 ☲ 101
    Zhen = 3,  // 震 ☳ 100
    Xun = 4,   // 巽 ☴ 011
    Kan = 5,   // 坎 ☵ 010
    Gen = 6,   // 艮 ☶ 001
    Kun = 7,   // 坤 ☷ 000
}

impl Trigram {
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

    /// 获取八卦五行
    pub fn wu_xing(&self) -> WuXing {
        match self {
            Self::Qian | Self::Dui => WuXing::Metal,
            Self::Li => WuXing::Fire,
            Self::Zhen | Self::Xun => WuXing::Wood,
            Self::Kan => WuXing::Water,
            Self::Gen | Self::Kun => WuXing::Earth,
        }
    }

    /// 获取八卦二进制码（从下往上）
    pub fn binary(&self) -> u8 {
        match self {
            Self::Qian => 0b111,
            Self::Dui => 0b110,
            Self::Li => 0b101,
            Self::Zhen => 0b100,
            Self::Xun => 0b011,
            Self::Kan => 0b010,
            Self::Gen => 0b001,
            Self::Kun => 0b000,
        }
    }

    /// 从二进制码获取八卦
    pub fn from_binary(code: u8) -> Self {
        match code & 0b111 {
            0b111 => Self::Qian,
            0b110 => Self::Dui,
            0b101 => Self::Li,
            0b100 => Self::Zhen,
            0b011 => Self::Xun,
            0b010 => Self::Kan,
            0b001 => Self::Gen,
            _ => Self::Kun,
        }
    }

    /// 从索引获取八卦
    pub fn from_index(index: u8) -> Self {
        match index % 8 {
            0 => Self::Qian,
            1 => Self::Dui,
            2 => Self::Li,
            3 => Self::Zhen,
            4 => Self::Xun,
            5 => Self::Kan,
            6 => Self::Gen,
            _ => Self::Kun,
        }
    }

    /// 获取八卦索引
    pub fn index(&self) -> u8 {
        *self as u8
    }
}

// ============================================================================
// 爻
// ============================================================================

/// 爻类型
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum Yao {
    /// 少阴（静爻，阴）
    #[default]
    ShaoYin = 0,
    /// 少阳（静爻，阳）
    ShaoYang = 1,
    /// 老阴（动爻，阴变阳）
    LaoYin = 2,
    /// 老阳（动爻，阳变阴）
    LaoYang = 3,
}

impl Yao {
    /// 获取爻名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::ShaoYin => "少阴",
            Self::ShaoYang => "少阳",
            Self::LaoYin => "老阴",
            Self::LaoYang => "老阳",
        }
    }

    /// 是否为阳爻
    pub fn is_yang(&self) -> bool {
        matches!(self, Self::ShaoYang | Self::LaoYang)
    }

    /// 是否为动爻
    pub fn is_moving(&self) -> bool {
        matches!(self, Self::LaoYin | Self::LaoYang)
    }

    /// 获取本卦爻值（0或1）
    pub fn original_value(&self) -> u8 {
        if self.is_yang() { 1 } else { 0 }
    }

    /// 获取变卦爻值（0或1）
    pub fn changed_value(&self) -> u8 {
        match self {
            Self::ShaoYin => 0,
            Self::ShaoYang => 1,
            Self::LaoYin => 1,  // 阴变阳
            Self::LaoYang => 0, // 阳变阴
        }
    }

    /// 从铜钱数（阳面个数）创建爻
    /// 0个阳面 = 老阴（⚏）
    /// 1个阳面 = 少阳（⚊）
    /// 2个阳面 = 少阴（⚋）
    /// 3个阳面 = 老阳（⚌）
    pub fn from_coin_count(count: u8) -> Self {
        match count % 4 {
            0 => Self::LaoYin,
            1 => Self::ShaoYang,
            2 => Self::ShaoYin,
            _ => Self::LaoYang,
        }
    }
}

// ============================================================================
// 六亲
// ============================================================================

/// 六亲
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
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
            Self::XiongDi => "兄弟",
            Self::FuMu => "父母",
            Self::GuanGui => "官鬼",
            Self::QiCai => "妻财",
            Self::ZiSun => "子孙",
        }
    }

    /// 根据卦宫五行和爻五行计算六亲
    pub fn from_wu_xing(gong_wx: WuXing, yao_wx: WuXing) -> Self {
        let gong_idx = gong_wx.index();
        let yao_idx = yao_wx.index();
        let diff = (yao_idx + 5 - gong_idx) % 5;
        match diff {
            0 => Self::XiongDi, // 同我者为兄弟
            1 => Self::ZiSun,   // 我生者为子孙
            2 => Self::QiCai,   // 我克者为妻财
            3 => Self::GuanGui, // 克我者为官鬼
            _ => Self::FuMu,    // 生我者为父母
        }
    }
}

// ============================================================================
// 六神
// ============================================================================

/// 六神
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum LiuShen {
    /// 青龙 - 吉神
    #[default]
    QingLong = 0,
    /// 朱雀 - 口舌是非
    ZhuQue = 1,
    /// 勾陈 - 田土牵连
    GouChen = 2,
    /// 螣蛇 - 虚惊怪异
    TengShe = 3,
    /// 白虎 - 凶神
    BaiHu = 4,
    /// 玄武 - 暗昧盗贼
    XuanWu = 5,
}

impl LiuShen {
    /// 获取六神名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::QingLong => "青龙",
            Self::ZhuQue => "朱雀",
            Self::GouChen => "勾陈",
            Self::TengShe => "螣蛇",
            Self::BaiHu => "白虎",
            Self::XuanWu => "玄武",
        }
    }

    /// 获取六神索引
    pub fn index(&self) -> u8 {
        *self as u8
    }

    /// 从索引获取六神
    pub fn from_index(index: u8) -> Self {
        match index % 6 {
            0 => Self::QingLong,
            1 => Self::ZhuQue,
            2 => Self::GouChen,
            3 => Self::TengShe,
            4 => Self::BaiHu,
            _ => Self::XuanWu,
        }
    }
}

// ============================================================================
// 卦宫归属
// ============================================================================

/// 卦序（在卦宫中的位置）
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum GuaXu {
    /// 本宫卦（六世卦）
    #[default]
    BenGong = 0,
    /// 一世卦
    YiShi = 1,
    /// 二世卦
    ErShi = 2,
    /// 三世卦
    SanShi = 3,
    /// 四世卦
    SiShi = 4,
    /// 五世卦
    WuShi = 5,
    /// 游魂卦
    YouHun = 6,
    /// 归魂卦
    GuiHun = 7,
}

impl GuaXu {
    /// 获取卦序名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::BenGong => "本宫",
            Self::YiShi => "一世",
            Self::ErShi => "二世",
            Self::SanShi => "三世",
            Self::SiShi => "四世",
            Self::WuShi => "五世",
            Self::YouHun => "游魂",
            Self::GuiHun => "归魂",
        }
    }

    /// 获取世爻位置（1-6）
    pub fn shi_yao_pos(&self) -> u8 {
        match self {
            Self::BenGong => 6,
            Self::YiShi => 1,
            Self::ErShi => 2,
            Self::SanShi => 3,
            Self::SiShi => 4,
            Self::WuShi => 5,
            Self::YouHun => 4,
            Self::GuiHun => 3,
        }
    }

    /// 获取应爻位置（1-6）
    pub fn ying_yao_pos(&self) -> u8 {
        let shi = self.shi_yao_pos();
        if shi > 3 { shi - 3 } else { shi + 3 }
    }
}

// ============================================================================
// 起卦方式
// ============================================================================

/// 起卦方式
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum DivinationMethod {
    /// 铜钱起卦（三枚铜钱法）
    #[default]
    CoinMethod = 0,
    /// 数字起卦（报数法）
    NumberMethod = 1,
    /// 时间起卦（按时辰起卦）
    TimeMethod = 2,
    /// 随机起卦（区块哈希）
    RandomMethod = 3,
    /// 手动指定（直接输入六爻）
    ManualMethod = 4,
}

impl DivinationMethod {
    /// 获取起卦方式名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::CoinMethod => "铜钱起卦",
            Self::NumberMethod => "数字起卦",
            Self::TimeMethod => "时间起卦",
            Self::RandomMethod => "随机起卦",
            Self::ManualMethod => "手动指定",
        }
    }
}

// ============================================================================
// 卦爻信息
// ============================================================================

/// 单爻信息
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct YaoInfo {
    /// 爻类型（少阴/少阳/老阴/老阳）
    pub yao: Yao,
    /// 纳甲天干
    pub tian_gan: TianGan,
    /// 纳甲地支
    pub di_zhi: DiZhi,
    /// 五行
    pub wu_xing: WuXing,
    /// 六亲
    pub liu_qin: LiuQin,
    /// 六神
    pub liu_shen: LiuShen,
    /// 是否为世爻
    pub is_shi: bool,
    /// 是否为应爻
    pub is_ying: bool,
}

/// 伏神信息
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct FuShenInfo {
    /// 伏神所在爻位（0-5）
    pub position: u8,
    /// 伏神六亲
    pub liu_qin: LiuQin,
    /// 纳甲天干
    pub tian_gan: TianGan,
    /// 纳甲地支
    pub di_zhi: DiZhi,
    /// 五行
    pub wu_xing: WuXing,
}

// ============================================================================
// 六爻卦结构
// ============================================================================

/// 六爻卦象
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
#[scale_info(skip_type_params(MaxCidLen))]
pub struct LiuYaoGua<AccountId, BlockNumber, MaxCidLen: frame_support::traits::Get<u32>> {
    /// 卦象 ID
    pub id: u64,
    /// 创建者账户
    pub creator: AccountId,
    /// 创建区块
    pub created_at: BlockNumber,

    // ===== 隐私控制字段 (v3.4 新增) =====
    /// 隐私模式
    pub privacy_mode: pallet_divination_privacy::types::PrivacyMode,
    /// 加密字段掩码
    /// bit0: 问事内容, bit1: 时间信息, bit2: 本卦信息, bit3: 变卦信息
    pub encrypted_fields: Option<u8>,
    /// 敏感数据哈希（用于完整性验证）
    pub sensitive_data_hash: Option<[u8; 32]>,

    // ===== 起卦信息 =====
    /// 起卦方式
    pub method: DivinationMethod,
    /// 占问事项（可选，链下存储）
    pub question_cid: Option<BoundedVec<u8, MaxCidLen>>,

    // ===== 时间信息 =====
    /// 年干支
    pub year_gz: Option<(TianGan, DiZhi)>,
    /// 月干支
    pub month_gz: Option<(TianGan, DiZhi)>,
    /// 日干支
    pub day_gz: Option<(TianGan, DiZhi)>,
    /// 时干支
    pub hour_gz: Option<(TianGan, DiZhi)>,

    // ===== 本卦信息 =====
    /// 本卦六爻（从初爻到上爻）
    pub original_yaos: Option<[YaoInfo; 6]>,
    /// 本卦内卦（下卦）
    pub original_inner: Option<Trigram>,
    /// 本卦外卦（上卦）
    pub original_outer: Option<Trigram>,
    /// 本卦卦名
    pub original_name_idx: Option<u8>,
    /// 本卦所属卦宫
    pub gong: Option<Trigram>,
    /// 卦序（在卦宫中的位置）
    pub gua_xu: Option<GuaXu>,

    // ===== 变卦信息 =====
    /// 是否有变卦
    pub has_bian_gua: bool,
    /// 变卦六爻（如有）
    pub changed_yaos: Option<[YaoInfo; 6]>,
    /// 变卦内卦
    pub changed_inner: Option<Trigram>,
    /// 变卦外卦
    pub changed_outer: Option<Trigram>,
    /// 变卦卦名
    pub changed_name_idx: Option<u8>,

    // ===== 互卦信息 =====
    /// 互卦内卦（取2,3,4爻组成）
    pub hu_inner: Option<Trigram>,
    /// 互卦外卦（取3,4,5爻组成）
    pub hu_outer: Option<Trigram>,
    /// 互卦卦名索引 (0-63)
    pub hu_name_idx: Option<u8>,

    // ===== 卦身 =====
    /// 卦身地支（阳爻从子起数到世爻位置，阴爻从午起数）
    pub gua_shen: Option<DiZhi>,

    // ===== 动爻位置 =====
    /// 动爻位置（位图，bit0=初爻, bit5=上爻）
    pub moving_yaos: Option<u8>,

    // ===== 旬空 =====
    /// 日旬空（两个地支）
    pub xun_kong: Option<(DiZhi, DiZhi)>,

    // ===== 伏神 =====
    /// 伏神列表（最多5个，缺哪个六亲就伏哪个）
    pub fu_shen: Option<[Option<FuShenInfo>; 6]>,
}

impl<AccountId, BlockNumber, MaxCidLen: frame_support::traits::Get<u32>>
    LiuYaoGua<AccountId, BlockNumber, MaxCidLen>
{
    /// 检查是否有计算数据
    pub fn has_calculation_data(&self) -> bool {
        self.original_yaos.is_some() && self.original_inner.is_some()
    }

    /// 检查是否可解读
    pub fn can_interpret(&self) -> bool {
        self.privacy_mode != pallet_divination_privacy::types::PrivacyMode::Private
            && self.has_calculation_data()
    }

    /// 是否公开
    pub fn is_public(&self) -> bool {
        self.privacy_mode == pallet_divination_privacy::types::PrivacyMode::Public
    }

    /// 是否完全隐私
    pub fn is_private(&self) -> bool {
        self.privacy_mode == pallet_divination_privacy::types::PrivacyMode::Private
    }
}

// ============================================================================
// 用户统计
// ============================================================================

/// 用户统计数据
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct UserStats {
    /// 总排盘次数
    pub total_guas: u32,
    /// 首次排盘区块
    pub first_gua_block: u32,
}

// ============================================================================
// 六十四卦索引
// ============================================================================

/// 六十四卦索引常量
pub mod gua64 {
    /// 六十四卦名数组
    ///
    /// 索引计算规则: index = (outer_trigram.binary() << 3) | inner_trigram.binary()
    ///
    /// 八卦二进制（从下到上，阳=1，阴=0）:
    /// - 坤=000(0), 艮=001(1), 坎=010(2), 巽=011(3)
    /// - 震=100(4), 离=101(5), 兑=110(6), 乾=111(7)
    pub const GUA_NAMES: [&str; 64] = [
        "坤为地",    // 0:  内坤(0) 外坤(0)
        "地山谦",    // 1:  内艮(1) 外坤(0)
        "地水师",    // 2:  内坎(2) 外坤(0)
        "地风升",    // 3:  内巽(3) 外坤(0)
        "地雷复",    // 4:  内震(4) 外坤(0)
        "地火明夷",  // 5:  内离(5) 外坤(0)
        "地泽临",    // 6:  内兑(6) 外坤(0)
        "地天泰",    // 7:  内乾(7) 外坤(0)
        "山地剥",    // 8:  内坤(0) 外艮(1)
        "艮为山",    // 9:  内艮(1) 外艮(1)
        "山水蒙",    // 10: 内坎(2) 外艮(1)
        "山风蛊",    // 11: 内巽(3) 外艮(1)
        "山雷颐",    // 12: 内震(4) 外艮(1)
        "山火贲",    // 13: 内离(5) 外艮(1)
        "山泽损",    // 14: 内兑(6) 外艮(1)
        "山天大畜",  // 15: 内乾(7) 外艮(1)
        "水地比",    // 16: 内坤(0) 外坎(2)
        "水山蹇",    // 17: 内艮(1) 外坎(2)
        "坎为水",    // 18: 内坎(2) 外坎(2)
        "水风井",    // 19: 内巽(3) 外坎(2)
        "水雷屯",    // 20: 内震(4) 外坎(2)
        "水火既济",  // 21: 内离(5) 外坎(2)
        "水泽节",    // 22: 内兑(6) 外坎(2)
        "水天需",    // 23: 内乾(7) 外坎(2)
        "风地观",    // 24: 内坤(0) 外巽(3)
        "风山渐",    // 25: 内艮(1) 外巽(3)
        "风水涣",    // 26: 内坎(2) 外巽(3)
        "巽为风",    // 27: 内巽(3) 外巽(3)
        "风雷益",    // 28: 内震(4) 外巽(3)
        "风火家人",  // 29: 内离(5) 外巽(3)
        "风泽中孚",  // 30: 内兑(6) 外巽(3)
        "风天小畜",  // 31: 内乾(7) 外巽(3)
        "雷地豫",    // 32: 内坤(0) 外震(4)
        "雷山小过",  // 33: 内艮(1) 外震(4)
        "雷水解",    // 34: 内坎(2) 外震(4)
        "雷风恒",    // 35: 内巽(3) 外震(4)
        "震为雷",    // 36: 内震(4) 外震(4)
        "雷火丰",    // 37: 内离(5) 外震(4)
        "雷泽归妹",  // 38: 内兑(6) 外震(4)
        "雷天大壮",  // 39: 内乾(7) 外震(4)
        "火地晋",    // 40: 内坤(0) 外离(5)
        "火山旅",    // 41: 内艮(1) 外离(5)
        "火水未济",  // 42: 内坎(2) 外离(5)
        "火风鼎",    // 43: 内巽(3) 外离(5)
        "火雷噬嗑",  // 44: 内震(4) 外离(5)
        "离为火",    // 45: 内离(5) 外离(5)
        "火泽睽",    // 46: 内兑(6) 外离(5)
        "火天大有",  // 47: 内乾(7) 外离(5)
        "泽地萃",    // 48: 内坤(0) 外兑(6)
        "泽山咸",    // 49: 内艮(1) 外兑(6)
        "泽水困",    // 50: 内坎(2) 外兑(6)
        "泽风大过",  // 51: 内巽(3) 外兑(6)
        "泽雷随",    // 52: 内震(4) 外兑(6)
        "泽火革",    // 53: 内离(5) 外兑(6)
        "兑为泽",    // 54: 内兑(6) 外兑(6)
        "泽天夬",    // 55: 内乾(7) 外兑(6)
        "天地否",    // 56: 内坤(0) 外乾(7)
        "天山遁",    // 57: 内艮(1) 外乾(7)
        "天水讼",    // 58: 内坎(2) 外乾(7)
        "天风姤",    // 59: 内巽(3) 外乾(7)
        "天雷无妄",  // 60: 内震(4) 外乾(7)
        "天火同人",  // 61: 内离(5) 外乾(7)
        "天泽履",    // 62: 内兑(6) 外乾(7)
        "乾为天",    // 63: 内乾(7) 外乾(7)
    ];

    /// 从内外卦计算六十四卦索引
    ///
    /// # 参数
    /// - `inner`: 内卦（下卦）的二进制值 (0-7)
    /// - `outer`: 外卦（上卦）的二进制值 (0-7)
    ///
    /// # 返回
    /// 六十四卦索引 (0-63)
    pub fn get_gua_index(inner: u8, outer: u8) -> u8 {
        (outer << 3) | inner
    }

    /// 获取卦名
    pub fn get_gua_name(index: u8) -> &'static str {
        GUA_NAMES[(index & 0x3F) as usize]
    }
}
