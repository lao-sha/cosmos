//! # 类型定义
//!
//! 定义 Enclave 内使用的核心数据类型

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

// ============================================================================
// 计算类型
// ============================================================================

/// 计算类型 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ComputeTypeId {
    /// 八字命理
    BaZi = 0,
    /// 梅花易数
    MeiHua = 1,
    /// 奇门遁甲
    QiMen = 2,
    /// 六爻占卜
    LiuYao = 3,
    /// 紫微斗数
    ZiWei = 4,
    /// 塔罗占卜
    Tarot = 5,
    /// 大六壬
    DaLiuRen = 6,
    /// 小六壬
    XiaoLiuRen = 7,
}

impl TryFrom<u8> for ComputeTypeId {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::BaZi),
            1 => Ok(Self::MeiHua),
            2 => Ok(Self::QiMen),
            3 => Ok(Self::LiuYao),
            4 => Ok(Self::ZiWei),
            5 => Ok(Self::Tarot),
            6 => Ok(Self::DaLiuRen),
            7 => Ok(Self::XiaoLiuRen),
            _ => Err(()),
        }
    }
}

// ============================================================================
// 性别
// ============================================================================

/// 性别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum Gender {
    #[default]
    Male = 0,
    Female = 1,
}

// ============================================================================
// 输入参数类型
// ============================================================================

/// 八字输入参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaZiInput {
    /// 出生年
    pub year: u16,
    /// 出生月
    pub month: u8,
    /// 出生日
    pub day: u8,
    /// 出生时辰 (0-23)
    pub hour: u8,
    /// 性别
    pub gender: Gender,
    /// 经度 (用于真太阳时计算，可选)
    pub longitude: Option<f64>,
}

/// 梅花易数输入参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeiHuaInput {
    /// 起卦方式 (0=时间, 1=数字, 2=文字)
    pub method: u8,
    /// 上卦数
    pub upper_num: Option<u16>,
    /// 下卦数
    pub lower_num: Option<u16>,
    /// 动爻数
    pub moving_line: Option<u8>,
    /// 起卦时间戳
    pub timestamp: u64,
}

/// 奇门遁甲输入参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QiMenInput {
    /// 排盘时间戳
    pub datetime: u64,
    /// 盘类型 (0=转盘, 1=飞盘)
    pub pan_type: u8,
    /// 排盘方法 (0=拆补, 1=置闰, 2=茅山)
    pub method: u8,
}

/// 六爻输入参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiuYaoInput {
    /// 六爻数据 (6个值，每个 0-3)
    pub yao_data: [u8; 6],
    /// 起卦时间戳
    pub timestamp: u64,
}

/// 紫微斗数输入参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZiWeiInput {
    /// 农历年
    pub lunar_year: u16,
    /// 农历月
    pub lunar_month: u8,
    /// 农历日
    pub lunar_day: u8,
    /// 时辰 (0-11)
    pub shi_chen: u8,
    /// 性别
    pub gender: Gender,
    /// 是否闰月
    pub is_leap_month: bool,
}

/// 塔罗输入参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarotInput {
    /// 牌阵类型 (0=单牌, 1=三牌, 2=凯尔特十字, 等)
    pub spread: u8,
    /// 随机种子 (32字节)
    pub seed: [u8; 32],
    /// 时间戳
    pub timestamp: u64,
}

/// 大六壬输入参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaLiuRenInput {
    /// 起课时间戳
    pub datetime: u64,
}

/// 小六壬输入参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XiaoLiuRenInput {
    /// 月
    pub month: u8,
    /// 日
    pub day: u8,
    /// 时
    pub hour: u8,
}

/// 统一的计算输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeInput {
    BaZi(BaZiInput),
    MeiHua(MeiHuaInput),
    QiMen(QiMenInput),
    LiuYao(LiuYaoInput),
    ZiWei(ZiWeiInput),
    Tarot(TarotInput),
    DaLiuRen(DaLiuRenInput),
    XiaoLiuRen(XiaoLiuRenInput),
}

// ============================================================================
// 输出结果类型
// ============================================================================

/// 八字结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaZiResult {
    /// 年柱
    pub year_pillar: String,
    /// 月柱
    pub month_pillar: String,
    /// 日柱
    pub day_pillar: String,
    /// 时柱
    pub hour_pillar: String,
    /// 日主
    pub day_master: String,
    /// 五行统计
    pub five_elements: FiveElementsCount,
    /// 十神
    pub ten_gods: Vec<String>,
    /// 大运列表
    pub major_cycles: Vec<MajorCycle>,
    /// 分析摘要
    pub summary: String,
}

/// 五行统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FiveElementsCount {
    pub metal: u8,   // 金
    pub wood: u8,    // 木
    pub water: u8,   // 水
    pub fire: u8,    // 火
    pub earth: u8,   // 土
}

/// 大运
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MajorCycle {
    /// 起运年龄
    pub start_age: u8,
    /// 天干地支
    pub gan_zhi: String,
}

/// 梅花易数结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeiHuaResult {
    /// 本卦
    pub original_hexagram: HexagramInfo,
    /// 变卦
    pub changed_hexagram: HexagramInfo,
    /// 互卦
    pub mutual_hexagram: HexagramInfo,
    /// 体卦
    pub ti_gua: String,
    /// 用卦
    pub yong_gua: String,
    /// 动爻位置
    pub moving_line: u8,
    /// 分析
    pub analysis: String,
}

/// 卦象信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexagramInfo {
    /// 卦名
    pub name: String,
    /// 卦序号 (1-64)
    pub number: u8,
    /// 上卦
    pub upper: String,
    /// 下卦
    pub lower: String,
    /// 卦辞
    pub text: String,
}

/// 奇门遁甲结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QiMenResult {
    /// 局数
    pub ju_number: i8,
    /// 阴阳遁
    pub yin_yang: String,
    /// 九宫数据
    pub palaces: Vec<PalaceInfo>,
    /// 值符
    pub duty_symbol: String,
    /// 值使
    pub duty_door: String,
    /// 分析
    pub analysis: String,
}

/// 宫位信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalaceInfo {
    /// 宫位 (1-9)
    pub position: u8,
    /// 天盘星
    pub star: String,
    /// 八门
    pub door: String,
    /// 八神
    pub deity: String,
    /// 地盘干
    pub earth_stem: String,
    /// 天盘干
    pub heaven_stem: String,
}

/// 六爻结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiuYaoResult {
    /// 本卦
    pub original_hexagram: HexagramInfo,
    /// 变卦
    pub changed_hexagram: Option<HexagramInfo>,
    /// 六爻详情
    pub lines: Vec<LineInfo>,
    /// 世爻位置
    pub shi_position: u8,
    /// 应爻位置
    pub ying_position: u8,
    /// 分析
    pub analysis: String,
}

/// 爻信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineInfo {
    /// 位置 (1-6，从下到上)
    pub position: u8,
    /// 爻类型 (阴/阳)
    pub yin_yang: String,
    /// 是否动爻
    pub is_moving: bool,
    /// 六亲
    pub six_relative: String,
    /// 六神
    pub six_deity: String,
}

/// 紫微斗数结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZiWeiResult {
    /// 命宫
    pub ming_gong: String,
    /// 身宫
    pub shen_gong: String,
    /// 十二宫
    pub twelve_palaces: Vec<ZiWeiPalace>,
    /// 主星
    pub main_stars: Vec<String>,
    /// 命主
    pub ming_zhu: String,
    /// 身主
    pub shen_zhu: String,
    /// 分析
    pub analysis: String,
}

/// 紫微宫位
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZiWeiPalace {
    /// 宫名
    pub name: String,
    /// 宫支
    pub branch: String,
    /// 主星列表
    pub main_stars: Vec<String>,
    /// 辅星列表
    pub minor_stars: Vec<String>,
}

/// 塔罗结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarotResult {
    /// 抽取的牌
    pub cards: Vec<TarotCard>,
    /// 牌阵名称
    pub spread_name: String,
    /// 解读
    pub interpretation: String,
}

/// 塔罗牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarotCard {
    /// 牌名
    pub name: String,
    /// 牌号 (0-77)
    pub number: u8,
    /// 是否逆位
    pub reversed: bool,
    /// 位置含义
    pub position_meaning: String,
    /// 牌义
    pub meaning: String,
}

/// 大六壬结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaLiuRenResult {
    /// 四课
    pub four_lessons: Vec<String>,
    /// 三传
    pub three_transmissions: Vec<String>,
    /// 天将
    pub heavenly_generals: Vec<String>,
    /// 分析
    pub analysis: String,
}

/// 小六壬结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XiaoLiuRenResult {
    /// 落宫 (大安/留连/速喜/赤口/小吉/空亡)
    pub palace: String,
    /// 宫位序号 (0-5)
    pub palace_index: u8,
    /// 吉凶
    pub fortune: String,
    /// 解释
    pub interpretation: String,
}

/// 统一的计算输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeOutput {
    BaZi(BaZiResult),
    MeiHua(MeiHuaResult),
    QiMen(QiMenResult),
    LiuYao(LiuYaoResult),
    ZiWei(ZiWeiResult),
    Tarot(TarotResult),
    DaLiuRen(DaLiuRenResult),
    XiaoLiuRen(XiaoLiuRenResult),
}

// ============================================================================
// 请求/响应类型
// ============================================================================

/// 计算请求（已解密）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeRequest {
    /// 请求 ID
    pub request_id: u64,
    /// 计算类型
    pub compute_type: ComputeTypeId,
    /// 计算输入
    pub input: ComputeInput,
    /// 请求时间戳
    pub timestamp: u64,
}

/// 计算响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResponse {
    /// 请求 ID
    pub request_id: u64,
    /// 计算类型
    pub compute_type: ComputeTypeId,
    /// 计算输出
    pub output: ComputeOutput,
    /// 完成时间戳
    pub completed_at: u64,
}

/// 加密的请求数据
#[derive(Debug, Clone)]
pub struct EncryptedRequest {
    /// 密文
    pub ciphertext: Vec<u8>,
    /// 临时公钥 (X25519)
    pub ephemeral_pubkey: [u8; 32],
    /// Nonce (12 字节)
    pub nonce: [u8; 12],
    /// 认证标签 (16 字节)
    pub auth_tag: [u8; 16],
}

/// 加密的响应数据
#[derive(Debug, Clone)]
pub struct EncryptedResponse {
    /// 密文
    pub ciphertext: Vec<u8>,
    /// Nonce (12 字节)
    pub nonce: [u8; 12],
    /// 认证标签 (16 字节)
    pub auth_tag: [u8; 16],
    /// 输出哈希 (用于链上验证)
    pub output_hash: [u8; 32],
    /// Enclave 签名
    pub enclave_signature: [u8; 64],
}

/// 计算证明
#[derive(Debug, Clone)]
pub struct ComputationProof {
    /// 输入哈希
    pub input_hash: [u8; 32],
    /// 输出哈希
    pub output_hash: [u8; 32],
    /// Enclave 签名
    pub enclave_signature: [u8; 64],
    /// 时间戳
    pub timestamp: u64,
}
