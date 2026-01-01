//! # 大六壬解盘数据结构模块
//!
//! 本模块定义大六壬解盘所需的所有数据结构，采用分层设计：
//! - **Layer 1**: 核心指标（约20字节）- 链上存储
//! - **Layer 2**: 扩展分析 - 链上可选存储或Runtime计算
//! - **Layer 3**: 完整解盘 - Runtime API返回
//!
//! ## 设计原则
//!
//! 1. **链上存储最小化**：核心数据仅存储必要指标
//! 2. **运行时计算**：可推导数据通过 Runtime API 实时计算
//! 3. **枚举索引化**：使用枚举类型代替字符串，存储高效
//! 4. **AI友好**：结构化数据便于AI解读和分析
//!
//! ## 大六壬解盘核心理论
//!
//! ### 三传论事
//! - **初传**：事情起因、开始阶段
//! - **中传**：事情发展、经过阶段
//! - **末传**：事情结局、结果阶段
//!
//! ### 类神取用
//! - 根据所占事类取相应类神
//! - 类神旺衰、空亡、天将吉凶决定成败
//!
//! ### 应期推算
//! - 三传相加法
//! - 类神法
//! - 空亡填实法
//! - 六冲六合法

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::*, BoundedVec};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

use crate::types::*;

// ============================================================================
// Layer 1: 核心枚举类型
// ============================================================================

/// 吉凶等级（7级）
///
/// 用于综合吉凶判断
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum FortuneLevel {
    /// 大吉 - 三传递生、吉将临吉神
    DaJi = 0,
    /// 中吉 - 体用相生、有吉无凶
    ZhongJi = 1,
    /// 小吉 - 略有吉象
    XiaoJi = 2,
    /// 平 - 吉凶参半
    #[default]
    Ping = 3,
    /// 小凶 - 略有凶象
    XiaoXiong = 4,
    /// 中凶 - 凶多吉少
    ZhongXiong = 5,
    /// 大凶 - 三传递克、凶将临凶神
    DaXiong = 6,
}

impl FortuneLevel {
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

    /// 是否为吉
    pub fn is_auspicious(&self) -> bool {
        matches!(self, Self::DaJi | Self::ZhongJi | Self::XiaoJi)
    }

    /// 获取评分（0-100）
    pub fn to_score(&self) -> u8 {
        match self {
            Self::DaJi => 95,
            Self::ZhongJi => 80,
            Self::XiaoJi => 65,
            Self::Ping => 50,
            Self::XiaoXiong => 35,
            Self::ZhongXiong => 20,
            Self::DaXiong => 5,
        }
    }
}

/// 事态发展趋势
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum TrendType {
    /// 下降趋势 - 三传递克、末传凶
    Descending = 0,
    /// 平稳趋势 - 三传平和
    #[default]
    Stable = 1,
    /// 上升趋势 - 三传递生、末传吉
    Ascending = 2,
}

impl TrendType {
    /// 获取趋势名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Descending => "下降",
            Self::Stable => "平稳",
            Self::Ascending => "上升",
        }
    }
}

/// 事情成败
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum OutcomeType {
    /// 不成 - 类神空亡、凶将临身
    BuCheng = 0,
    /// 难成 - 类神休囚、有阻碍
    NanCheng = 1,
    /// 可成 - 类神有气、吉多凶少
    #[default]
    KeCheng = 2,
    /// 必成 - 类神旺相、吉将护持
    BiCheng = 3,
}

impl OutcomeType {
    /// 获取成败名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::BuCheng => "不成",
            Self::NanCheng => "难成",
            Self::KeCheng => "可成",
            Self::BiCheng => "必成",
        }
    }
}

/// 事象类型（占断方向）
///
/// 根据所占事类选择相应的类神和判断方法
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum ShiXiangType {
    /// 事业/官运 - 取官鬼爻
    ShiYeGuan = 0,
    /// 财运 - 取妻财爻
    CaiYun = 1,
    /// 婚姻/感情 - 取天后、六合
    HunYinGanQing = 2,
    /// 求名/考试 - 取朱雀、父母爻
    QiuMingKaoShi = 3,
    /// 疾病/健康 - 取病符、天医
    JiBingJianKang = 4,
    /// 出行 - 取驿马
    ChuXing = 5,
    /// 求子 - 取子孙爻
    QiuZi = 6,
    /// 行人/讯息 - 取驿马、朱雀
    XingRenXunXi = 7,
    /// 诉讼 - 取官鬼、勾陈
    SuSong = 8,
    /// 失物/寻人 - 取玄武
    ShiWuXunRen = 9,
    /// 天气 - 取太阳、月建
    TianQi = 10,
    /// 其他/通用
    #[default]
    QiTa = 11,
}

impl ShiXiangType {
    /// 获取事象名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::ShiYeGuan => "事业官运",
            Self::CaiYun => "财运",
            Self::HunYinGanQing => "婚姻感情",
            Self::QiuMingKaoShi => "求名考试",
            Self::JiBingJianKang => "疾病健康",
            Self::ChuXing => "出行",
            Self::QiuZi => "求子",
            Self::XingRenXunXi => "行人讯息",
            Self::SuSong => "诉讼",
            Self::ShiWuXunRen => "失物寻人",
            Self::TianQi => "天气",
            Self::QiTa => "其他",
        }
    }

    /// 获取主要类神天将
    pub fn primary_tian_jiang(&self) -> TianJiang {
        match self {
            Self::ShiYeGuan => TianJiang::GuiRen,
            Self::CaiYun => TianJiang::QingLong,
            Self::HunYinGanQing => TianJiang::TianHou,
            Self::QiuMingKaoShi => TianJiang::ZhuQue,
            Self::JiBingJianKang => TianJiang::TengShe,
            Self::ChuXing => TianJiang::TaiChang,
            Self::QiuZi => TianJiang::TianHou,
            Self::XingRenXunXi => TianJiang::ZhuQue,
            Self::SuSong => TianJiang::GouChen,
            Self::ShiWuXunRen => TianJiang::XuanWu,
            Self::TianQi => TianJiang::QingLong,
            Self::QiTa => TianJiang::GuiRen,
        }
    }
}

/// 应期单位
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum YingQiUnit {
    /// 日
    #[default]
    Ri = 0,
    /// 旬（10日）
    Xun = 1,
    /// 月
    Yue = 2,
    /// 年
    Nian = 3,
}

impl YingQiUnit {
    /// 获取单位名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Ri => "日",
            Self::Xun => "旬",
            Self::Yue => "月",
            Self::Nian => "年",
        }
    }
}

/// 应期计算方法
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum YingQiMethod {
    /// 三传相加法 - 初+中+末传数
    #[default]
    SanChuanXiangJia = 0,
    /// 类神法 - 类神所临支数
    LeiShen = 1,
    /// 空亡填实 - 空亡支填实之日
    KongWangTianShi = 2,
    /// 六冲应期 - 冲动之日
    LiuChong = 3,
    /// 六合应期 - 合住之日
    LiuHe = 4,
    /// 天将应期 - 天将所临支
    TianJiang = 5,
    /// 生旺墓绝 - 按生旺墓绝计算
    ShengWangMuJue = 6,
}

impl YingQiMethod {
    /// 获取方法名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::SanChuanXiangJia => "三传相加",
            Self::LeiShen => "类神法",
            Self::KongWangTianShi => "空亡填实",
            Self::LiuChong => "六冲应期",
            Self::LiuHe => "六合应期",
            Self::TianJiang => "天将应期",
            Self::ShengWangMuJue => "生旺墓绝",
        }
    }
}

/// 神煞类型
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum ShenShaType {
    // ===== 吉神 (0-19) =====
    /// 天乙贵人
    #[default]
    TianYiGuiRen = 0,
    /// 天德
    TianDe = 1,
    /// 月德
    YueDe = 2,
    /// 天喜
    TianXi = 3,
    /// 生气
    ShengQi = 4,
    /// 驿马（可吉可凶）
    YiMa = 5,
    /// 皇书
    HuangShu = 6,
    /// 皇恩
    HuangEn = 7,
    /// 天医
    TianYi = 8,
    /// 天诏
    TianZhao = 9,
    /// 旬奇
    XunQi = 10,
    /// 旬仪
    XunYi = 11,

    // ===== 凶神 (20-39) =====
    /// 天罗
    TianLuo = 20,
    /// 地网
    DiWang = 21,
    /// 天鬼
    TianGui = 22,
    /// 丧门
    SangMen = 23,
    /// 丧车
    SangChe = 24,
    /// 血支
    XueZhi = 25,
    /// 血忌
    XueJi = 26,
    /// 大耗
    DaHao = 27,
    /// 小耗
    XiaoHao = 28,
    /// 病符
    BingFu = 29,
    /// 孤辰
    GuChen = 30,
    /// 寡宿
    GuaSu = 31,
    /// 死气
    SiQi = 32,
    /// 五墓
    WuMu = 33,
    /// 三丘
    SanQiu = 34,
}

impl ShenShaType {
    /// 获取神煞名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::TianYiGuiRen => "天乙贵人",
            Self::TianDe => "天德",
            Self::YueDe => "月德",
            Self::TianXi => "天喜",
            Self::ShengQi => "生气",
            Self::YiMa => "驿马",
            Self::HuangShu => "皇书",
            Self::HuangEn => "皇恩",
            Self::TianYi => "天医",
            Self::TianZhao => "天诏",
            Self::XunQi => "旬奇",
            Self::XunYi => "旬仪",
            Self::TianLuo => "天罗",
            Self::DiWang => "地网",
            Self::TianGui => "天鬼",
            Self::SangMen => "丧门",
            Self::SangChe => "丧车",
            Self::XueZhi => "血支",
            Self::XueJi => "血忌",
            Self::DaHao => "大耗",
            Self::XiaoHao => "小耗",
            Self::BingFu => "病符",
            Self::GuChen => "孤辰",
            Self::GuaSu => "寡宿",
            Self::SiQi => "死气",
            Self::WuMu => "五墓",
            Self::SanQiu => "三丘",
        }
    }

    /// 是否为吉神
    pub fn is_auspicious(&self) -> bool {
        (*self as u8) < 20
    }
}

// ============================================================================
// Layer 1: 核心解盘结构（约20字节）
// ============================================================================

/// 大六壬核心解盘结果
///
/// 存储优化版本，约20字节，包含最关键的解盘指标
///
/// ## 字段说明
/// - 课式格局：决定事物发展模式
/// - 吉凶判断：综合吉凶、趋势、成败
/// - 类神分析：主类神的状态
/// - 应期推算：事情应验时间
/// - 评分元数据：综合评分和可信度
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct CoreInterpretation {
    // ===== 课式判断 (2 bytes) =====
    /// 课式类型 (1 byte)
    pub ke_shi: KeShiType,
    /// 格局类型 (1 byte)
    pub ge_ju: GeJuType,

    // ===== 吉凶判断 (3 bytes) =====
    /// 综合吉凶等级 (1 byte)
    pub fortune: FortuneLevel,
    /// 事态发展趋势 (1 byte)
    pub trend: TrendType,
    /// 事情成败 (1 byte)
    pub outcome: OutcomeType,

    // ===== 类神分析 (4 bytes) =====
    /// 主类神（初传地支）(1 byte)
    pub primary_lei_shen: DiZhi,
    /// 主类神旺衰 (1 byte)
    pub primary_wang_shuai: WangShuai,
    /// 主类神六亲 (1 byte)
    pub primary_liu_qin: LiuQin,
    /// 主类神天将吉凶：true=吉 (1 byte)
    pub primary_jiang_ji: bool,

    // ===== 应期推算 (4 bytes) =====
    /// 应期数（主应期）(1 byte)
    pub ying_qi_num: u8,
    /// 应期单位 (1 byte)
    pub ying_qi_unit: YingQiUnit,
    /// 次应期地支 (1 byte)
    pub secondary_ying_qi: DiZhi,
    /// 应期可信度 0-100 (1 byte)
    pub ying_qi_confidence: u8,

    // ===== 评分与元数据 (6 bytes) =====
    /// 综合评分 0-100 (1 byte)
    pub score: u8,
    /// 解盘可信度 0-100 (1 byte)
    pub confidence: u8,
    /// 解盘区块号 (4 bytes)
    pub timestamp: u32,
}

impl CoreInterpretation {
    /// 创建带时间戳的默认解盘结果
    ///
    /// 用于 Private 模式下无计算数据时返回
    pub fn default_with_timestamp(timestamp: u32) -> Self {
        Self {
            timestamp,
            ..Default::default()
        }
    }
}

// ============================================================================
// Layer 2: 扩展分析结构
// ============================================================================

/// 三传分析
///
/// 分析三传的旺衰、空亡、天将吉凶及相互关系
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct SanChuanAnalysis {
    // ===== 初传分析（事情起因/开始）=====
    /// 初传旺衰
    pub chu_wang_shuai: WangShuai,
    /// 初传天将吉凶
    pub chu_jiang_ji: bool,
    /// 初传空亡
    pub chu_kong: bool,

    // ===== 中传分析（事情经过/发展）=====
    /// 中传旺衰
    pub zhong_wang_shuai: WangShuai,
    /// 中传天将吉凶
    pub zhong_jiang_ji: bool,
    /// 中传空亡
    pub zhong_kong: bool,

    // ===== 末传分析（事情结果/结局）=====
    /// 末传旺衰
    pub mo_wang_shuai: WangShuai,
    /// 末传天将吉凶
    pub mo_jiang_ji: bool,
    /// 末传空亡
    pub mo_kong: bool,

    // ===== 三传关系 =====
    /// 三传递生：true=递生（吉）
    pub di_sheng: bool,
    /// 三传递克：true=递克（凶）
    pub di_ke: bool,
    /// 三传连茹：true=连茹（三支相连）
    pub lian_ru: bool,
}

/// 四课分析
///
/// 分析四课的克关系和日干日支状态
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct SiKeAnalysis {
    // ===== 日干相关（第一、二课代表求测人）=====
    /// 日干得助：是否有比肩/印星
    pub ri_gan_you_zhu: bool,
    /// 干阳神旺衰
    pub gan_yang_wang_shuai: WangShuai,

    // ===== 日支相关（第三、四课代表所测事物）=====
    /// 日支得生：是否有生扶
    pub ri_zhi_you_sheng: bool,
    /// 支阳神旺衰
    pub zhi_yang_wang_shuai: WangShuai,

    // ===== 克关系 =====
    /// 上克下数量（克课）
    pub shang_ke_xia_count: u8,
    /// 下克上数量（贼课）
    pub xia_ke_shang_count: u8,

    // ===== 干支关系 =====
    /// 日干日支相合
    pub gan_zhi_he: bool,
    /// 日干日支相冲
    pub gan_zhi_chong: bool,
}

/// 天将分析
///
/// 分析天将盘的吉凶状态
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct TianJiangAnalysis {
    /// 贵人（天乙贵人）所临地盘支
    pub gui_ren_lin: DiZhi,
    /// 贵人是否空亡
    pub gui_ren_kong: bool,
    /// 贵人是否入墓
    pub gui_ren_mu: bool,

    /// 青龙所临地盘支（主财喜）
    pub qing_long_lin: DiZhi,
    /// 白虎所临地盘支（主凶险）
    pub bai_hu_lin: DiZhi,

    /// 吉将数量（贵人、六合、青龙、太常、太阴、天后）
    pub ji_jiang_count: u8,
    /// 凶将数量（螣蛇、朱雀、勾陈、天空、白虎、玄武）
    pub xiong_jiang_count: u8,

    /// 三传天将吉凶统计（吉将数）
    pub san_chuan_ji_jiang: u8,
}

/// 神煞分析
///
/// 分析神煞的吉凶状态
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct ShenShaAnalysis {
    // ===== 吉神煞 (最多8个) =====
    /// 吉神煞列表
    pub ji_shen_sha: BoundedVec<ShenShaType, ConstU32<8>>,

    // ===== 凶神煞 (最多8个) =====
    /// 凶神煞列表
    pub xiong_shen_sha: BoundedVec<ShenShaType, ConstU32<8>>,

    // ===== 特殊神煞状态 =====
    /// 驿马入传
    pub yi_ma_ru_chuan: bool,
    /// 天罗地网
    pub tian_luo_di_wang: bool,
    /// 六害入传
    pub liu_hai_ru_chuan: bool,
    /// 三刑入传
    pub san_xing_ru_chuan: bool,
}

/// 应期分析结果
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct YingQiResult {
    /// 数值
    pub num: u8,
    /// 单位
    pub unit: YingQiUnit,
    /// 地支（应期对应的地支）
    pub zhi: DiZhi,
    /// 计算方法
    pub method: YingQiMethod,
}

/// 应期详细分析
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct YingQiAnalysis {
    /// 主应期（三传相加法）
    pub primary: YingQiResult,

    /// 次应期（类神法）
    pub secondary: Option<YingQiResult>,

    /// 特殊应期（空亡填实、冲合等）
    pub special: Option<YingQiResult>,

    /// 应期综合建议（文本索引）
    pub suggestion_index: u8,
}

// ============================================================================
// Layer 3: 完整解盘结构
// ============================================================================

/// 事象断语提示
///
/// 用于前端显示预设断语
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct ShiXiangHints {
    /// 占问类型
    pub shi_xiang_type: ShiXiangType,

    /// 主断语索引（预设断语表索引）
    pub primary_hint_index: u8,

    /// 辅助断语索引（最多3个）
    pub secondary_hints: BoundedVec<u8, ConstU32<3>>,

    /// 注意事项索引
    pub caution_index: Option<u8>,
}

/// 大六壬完整解盘结果
///
/// 用于 Runtime API 返回，不直接存储
#[derive(Clone, Encode, Decode, TypeInfo, RuntimeDebug)]
pub struct FullInterpretation {
    /// 核心解盘（必有）
    pub core: CoreInterpretation,

    /// 三传分析
    pub san_chuan_analysis: SanChuanAnalysis,

    /// 四课分析
    pub si_ke_analysis: SiKeAnalysis,

    /// 天将分析
    pub tian_jiang_analysis: TianJiangAnalysis,

    /// 神煞分析
    pub shen_sha_analysis: ShenShaAnalysis,

    /// 应期详细分析
    pub ying_qi_analysis: YingQiAnalysis,

    /// 事象断语提示（可选）
    pub shi_xiang_hints: Option<ShiXiangHints>,
}

impl Default for FullInterpretation {
    fn default() -> Self {
        Self {
            core: CoreInterpretation::default(),
            san_chuan_analysis: SanChuanAnalysis::default(),
            si_ke_analysis: SiKeAnalysis::default(),
            tian_jiang_analysis: TianJiangAnalysis::default(),
            shen_sha_analysis: ShenShaAnalysis::default(),
            ying_qi_analysis: YingQiAnalysis::default(),
            shi_xiang_hints: None,
        }
    }
}

impl FullInterpretation {
    /// 创建带时间戳的默认解盘结果
    ///
    /// 用于 Private 模式下无计算数据时返回
    pub fn default_with_timestamp(timestamp: u32) -> Self {
        Self {
            core: CoreInterpretation::default_with_timestamp(timestamp),
            ..Default::default()
        }
    }
}

// ============================================================================
// AI 解读数据结构
// ============================================================================

/// AI解读请求数据
///
/// 发送给AI Oracle的结构化数据
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, RuntimeDebug)]
pub struct AiInterpretationRequest<AccountId, BlockNumber, MaxCidLen: Get<u32>> {
    /// 式盘ID
    pub pan_id: u64,

    /// 请求者
    pub requester: AccountId,

    /// 式盘创建区块
    pub pan_created_at: BlockNumber,

    /// 链上解盘结果
    pub interpretation: CoreInterpretation,

    /// 占问类型
    pub shi_xiang_type: ShiXiangType,

    /// 问题哈希（隐私保护）
    pub question_hash: [u8; 32],

    /// 问题CID（可选，链下存储）
    pub question_cid: Option<BoundedVec<u8, MaxCidLen>>,

    /// 请求时间戳（区块号）
    pub request_timestamp: BlockNumber,
}

/// AI解读结果
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, RuntimeDebug)]
pub struct AiInterpretationResult<MaxCidLen: Get<u32>> {
    /// 式盘ID
    pub pan_id: u64,

    /// 解读内容IPFS CID
    pub interpretation_cid: BoundedVec<u8, MaxCidLen>,

    /// 解读摘要（链上存储，最多256字节）
    pub summary: BoundedVec<u8, ConstU32<256>>,

    /// AI吉凶评分（与链上解盘对比）
    pub ai_fortune_score: u8,

    /// AI可信度评分
    pub ai_confidence: u8,

    /// AI建议关键词索引（最多5个）
    pub advice_keywords: BoundedVec<u8, ConstU32<5>>,

    /// 提交时间戳（区块号）
    pub submit_timestamp: u32,

    /// AI模型版本
    pub model_version: BoundedVec<u8, ConstU32<16>>,
}

impl<MaxCidLen: Get<u32>> Default for AiInterpretationResult<MaxCidLen> {
    fn default() -> Self {
        Self {
            pan_id: 0,
            interpretation_cid: BoundedVec::default(),
            summary: BoundedVec::default(),
            ai_fortune_score: 50,
            ai_confidence: 50,
            advice_keywords: BoundedVec::default(),
            submit_timestamp: 0,
            model_version: BoundedVec::default(),
        }
    }
}

// ============================================================================
// 断语索引表（常量）
// ============================================================================

/// 课式断语索引
///
/// 每种课式对应的基本断语
pub const KE_SHI_HINTS: [&str; 9] = [
    "贼克课：下克上为贼，事多暗昧、小人作祟",
    "比用课：同类相比，需仔细斟酌",
    "涉害课：涉害深者为用，事多艰难",
    "遥克课：遥相克制，事情迂回",
    "昂星课：高悬明照，宜静不宜动",
    "别责课：三课不全，责任分明",
    "八专课：干支同类，一意孤行",
    "伏吟课：天地重合，静中有动",
    "返吟课：天地相冲，动中求静",
];

/// 格局断语索引
///
/// 每种格局对应的基本断语
pub const GE_JU_HINTS: [&str; 15] = [
    "元首格：上克下，顺势而为，大吉之兆",
    "重审格：下克上，需再三审视",
    "知一格：比用取一，择善而从",
    "涉害格：涉害深者，事多周折",
    "见机格：见机行事，当机立断",
    "察微格：察微知著，细节定成败",
    "复等格：复杂局面，需等待时机",
    "遥克格：遥相呼应，缓则有成",
    "虎视格：虎视眈眈，静观其变",
    "冬蛇掩目：隐藏玄机，不宜妄动",
    "别责格：责任分明，各司其职",
    "八专格：专一则成，杂则败",
    "自任格：自力更生，独立完成",
    "自信格：坚定信心，终有所成",
    "无依格：无所依托，随遇而安",
];

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use codec::Encode;

    #[test]
    fn test_core_interpretation_size() {
        let core = CoreInterpretation {
            ke_shi: KeShiType::ZeiKe,
            ge_ju: GeJuType::YuanShou,
            fortune: FortuneLevel::ZhongJi,
            trend: TrendType::Ascending,
            outcome: OutcomeType::KeCheng,
            primary_lei_shen: DiZhi::Yin,
            primary_wang_shuai: WangShuai::Wang,
            primary_liu_qin: LiuQin::GuanGui,
            primary_jiang_ji: true,
            ying_qi_num: 3,
            ying_qi_unit: YingQiUnit::Yue,
            secondary_ying_qi: DiZhi::Wu,
            ying_qi_confidence: 80,
            score: 75,
            confidence: 85,
            timestamp: 1000000,
        };

        let encoded = core.encode();
        println!("CoreInterpretation 编码大小: {} bytes", encoded.len());
        assert!(encoded.len() <= 20, "CoreInterpretation 应小于等于20字节");
    }

    #[test]
    fn test_fortune_level() {
        assert!(FortuneLevel::DaJi.is_auspicious());
        assert!(FortuneLevel::ZhongJi.is_auspicious());
        assert!(!FortuneLevel::XiaoXiong.is_auspicious());
        assert!(!FortuneLevel::DaXiong.is_auspicious());

        assert_eq!(FortuneLevel::DaJi.to_score(), 95);
        assert_eq!(FortuneLevel::Ping.to_score(), 50);
        assert_eq!(FortuneLevel::DaXiong.to_score(), 5);
    }

    #[test]
    fn test_shen_sha_type() {
        assert!(ShenShaType::TianYiGuiRen.is_auspicious());
        assert!(ShenShaType::YiMa.is_auspicious());
        assert!(!ShenShaType::TianLuo.is_auspicious());
        assert!(!ShenShaType::TianGui.is_auspicious());
    }

    #[test]
    fn test_shi_xiang_type() {
        assert_eq!(ShiXiangType::ShiYeGuan.primary_tian_jiang(), TianJiang::GuiRen);
        assert_eq!(ShiXiangType::CaiYun.primary_tian_jiang(), TianJiang::QingLong);
        assert_eq!(ShiXiangType::HunYinGanQing.primary_tian_jiang(), TianJiang::TianHou);
    }

    #[test]
    fn test_san_chuan_analysis() {
        let analysis = SanChuanAnalysis {
            chu_wang_shuai: WangShuai::Wang,
            chu_jiang_ji: true,
            chu_kong: false,
            zhong_wang_shuai: WangShuai::Xiang,
            zhong_jiang_ji: true,
            zhong_kong: false,
            mo_wang_shuai: WangShuai::Xiu,
            mo_jiang_ji: false,
            mo_kong: false,
            di_sheng: true,
            di_ke: false,
            lian_ru: false,
        };

        let encoded = analysis.encode();
        println!("SanChuanAnalysis 编码大小: {} bytes", encoded.len());
    }
}
