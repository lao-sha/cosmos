//! # 六爻解卦模块
//!
//! 本模块实现六爻占卜的解卦功能，包括：
//! - 吉凶判断
//! - 用神分析
//! - 应期推算
//! - 综合评分
//!
//! ## 设计特点
//!
//! - **分层存储**: 核心指标链上存储，详细解释链下生成
//! - **存储优化**: 使用枚举索引而非字符串
//! - **实时计算**: 通过 Runtime API 免费获取解卦
//! - **算法可升级**: 无需数据迁移

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

use crate::types::LiuQin;

// ============================================================================
// 吉凶等级枚举
// ============================================================================

/// 吉凶等级（1 byte）
///
/// 用于表示六爻占卜的总体吉凶判断
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
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
// 用神状态枚举
// ============================================================================

/// 用神状态（1 byte）
///
/// 表示用神（关键爻）的旺衰状态和特殊情况
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub enum YongShenState {
    /// 旺相 - 得时得地，事情有利
    #[default]
    WangXiang = 0,
    /// 休囚 - 失时失地，事情不利
    XiuQiu = 1,
    /// 动而化进 - 动爻化进神，事情向好发展
    DongHuaJin = 2,
    /// 动而化退 - 动爻化退神，事情有退步之象
    DongHuaTui = 3,
    /// 动而化空 - 动爻化空亡，事情虚而不实
    DongHuaKong = 4,
    /// 伏藏 - 伏神状态，所求之事隐而未显
    FuCang = 5,
    /// 空亡 - 日空或月空，所求之事虚而不实
    KongWang = 6,
    /// 入墓 - 入墓库，事情受阻，需待时机
    RuMu = 7,
    /// 受克 - 被克制，所求之事受阻
    ShouKe = 8,
    /// 得生 - 被生扶，所求之事有贵人相助
    DeSheng = 9,
}

impl YongShenState {
    /// 获取用神状态名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::WangXiang => "旺相",
            Self::XiuQiu => "休囚",
            Self::DongHuaJin => "动化进",
            Self::DongHuaTui => "动化退",
            Self::DongHuaKong => "动化空",
            Self::FuCang => "伏藏",
            Self::KongWang => "空亡",
            Self::RuMu => "入墓",
            Self::ShouKe => "受克",
            Self::DeSheng => "得生",
        }
    }

    /// 判断是否为有利状态
    pub fn is_favorable(&self) -> bool {
        matches!(self, Self::WangXiang | Self::DongHuaJin | Self::DeSheng)
    }

    /// 判断是否为不利状态
    pub fn is_unfavorable(&self) -> bool {
        matches!(self, Self::XiuQiu | Self::DongHuaTui | Self::DongHuaKong | Self::KongWang | Self::RuMu | Self::ShouKe)
    }
}

// ============================================================================
// 事项类型枚举
// ============================================================================

/// 占问事项类型（1 byte）
///
/// 用于确定用神和解卦方向
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub enum ShiXiangType {
    /// 财运 - 用神为妻财
    #[default]
    CaiYun = 0,
    /// 事业 - 用神为官鬼
    ShiYe = 1,
    /// 婚姻感情 - 男占用妻财，女占用官鬼
    HunYin = 2,
    /// 健康 - 用神为世爻
    JianKang = 3,
    /// 考试学业 - 用神为父母
    KaoShi = 4,
    /// 官司诉讼 - 用神为官鬼
    GuanSi = 5,
    /// 出行 - 用神为世爻
    ChuXing = 6,
    /// 寻人寻物 - 用神为用事之爻
    XunRen = 7,
    /// 天气 - 用神为相关爻
    TianQi = 8,
    /// 其他 - 需要自定义用神
    QiTa = 9,
}

impl ShiXiangType {
    /// 获取事项类型名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::CaiYun => "财运",
            Self::ShiYe => "事业",
            Self::HunYin => "婚姻感情",
            Self::JianKang => "健康",
            Self::KaoShi => "考试学业",
            Self::GuanSi => "官司诉讼",
            Self::ChuXing => "出行",
            Self::XunRen => "寻人寻物",
            Self::TianQi => "天气",
            Self::QiTa => "其他",
        }
    }

    /// 获取默认用神六亲
    pub fn default_yong_shen_qin(&self) -> LiuQin {
        match self {
            Self::CaiYun => LiuQin::QiCai,
            Self::ShiYe => LiuQin::GuanGui,
            Self::HunYin => LiuQin::QiCai, // 男占，女占为 GuanGui
            Self::JianKang => LiuQin::XiongDi, // 世爻
            Self::KaoShi => LiuQin::FuMu,
            Self::GuanSi => LiuQin::GuanGui,
            Self::ChuXing => LiuQin::XiongDi, // 世爻
            Self::XunRen => LiuQin::XiongDi, // 需要自定义
            Self::TianQi => LiuQin::XiongDi, // 需要自定义
            Self::QiTa => LiuQin::XiongDi, // 需要自定义
        }
    }
}

// ============================================================================
// 应期类型枚举
// ============================================================================

/// 应期类型（1 byte）
///
/// 表示事情应验的时间范围
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub enum YingQiType {
    /// 近期（日内）- 应期在日
    #[default]
    JinQi = 0,
    /// 短期（月内）- 应期在月
    DuanQi = 1,
    /// 中期（季度内）- 应期在季
    ZhongQi = 2,
    /// 长期（年内）- 应期在年
    ChangQi = 3,
    /// 远期（年后）- 应期在年后
    YuanQi = 4,
    /// 不确定 - 需要进一步分析
    BuQueDing = 5,
}

impl YingQiType {
    /// 获取应期类型名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::JinQi => "近期（日内）",
            Self::DuanQi => "短期（月内）",
            Self::ZhongQi => "中期（季度内）",
            Self::ChangQi => "长期（年内）",
            Self::YuanQi => "远期（年后）",
            Self::BuQueDing => "不确定",
        }
    }
}

// ============================================================================
// 解卦文本类型枚举
// ============================================================================

/// 解卦文本类型枚举（1 byte）
///
/// 用于链上存储解卦文本索引，前端根据索引显示对应文本
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum JieGuaTextType {
    // ===== 吉凶总断 (0-6) =====
    /// 大吉：诸事顺遂，心想事成
    DaJiZongDuan = 0,
    /// 吉：事可成，宜进取
    JiZongDuan = 1,
    /// 小吉：小有所得，不宜大动
    XiaoJiZongDuan = 2,
    /// 平：平稳无波，守成为上
    PingZongDuan = 3,
    /// 小凶：小有阻碍，谨慎行事
    XiaoXiongZongDuan = 4,
    /// 凶：事难成，宜退守
    XiongZongDuan = 5,
    /// 大凶：诸事不利，静待时机
    DaXiongZongDuan = 6,

    // ===== 用神状态 (7-16) =====
    /// 用神旺相：所求之事有望
    YongShenWangXiang = 7,
    /// 用神休囚：所求之事难成
    YongShenXiuQiu = 8,
    /// 用神动而化进：事情向好发展
    YongShenHuaJin = 9,
    /// 用神动而化退：事情有退步之象
    YongShenHuaTui = 10,
    /// 用神逢空：所求之事虚而不实
    YongShenKong = 11,
    /// 用神入墓：事情受阻，需待时机
    YongShenRuMu = 12,
    /// 用神伏藏：所求之事隐而未显
    YongShenFuCang = 13,
    /// 用神受克：所求之事受阻
    YongShenShouKe = 14,
    /// 用神得生：所求之事有贵人相助
    YongShenDeSheng = 15,
    /// 用神发动：事情有变化
    YongShenFaDong = 16,

    // ===== 世应关系 (17-22) =====
    /// 世应相生：双方和谐，事易成
    ShiYingXiangSheng = 17,
    /// 世应相克：双方有冲突
    ShiYingXiangKe = 18,
    /// 世应比和：双方势均力敌
    ShiYingBiHe = 19,
    /// 世爻旺应爻衰：我强彼弱
    ShiWangYingShuai = 20,
    /// 世爻衰应爻旺：我弱彼强
    ShiShuaiYingWang = 21,
    /// 世应俱空：双方皆虚
    ShiYingJuKong = 22,

    // ===== 动爻断语 (23-28) =====
    /// 无动爻：事情平稳，无大变化
    WuDongYao = 23,
    /// 一爻独发：事情明确，吉凶易断
    YiYaoDuFa = 24,
    /// 多爻齐动：事情复杂，变数较多
    DuoYaoQiDong = 25,
    /// 六爻皆动：大变之象，需谨慎
    LiuYaoJieDong = 26,
    /// 动爻化进：事情向好发展
    DongYaoHuaJin = 27,
    /// 动爻化退：事情有退步之象
    DongYaoHuaTui = 28,

    // ===== 特殊状态 (29-34) =====
    /// 用神逢日冲：近期有变
    YongShenRiChong = 29,
    /// 用神逢月破：本月不利
    YongShenYuePo = 30,
    /// 卦逢六冲：事情难成或有变
    GuaFengLiuChong = 31,
    /// 卦逢六合：事情顺利
    GuaFengLiuHe = 32,
    /// 反吟卦：事情反复
    FanYinGua = 33,
    /// 伏吟卦：事情停滞
    FuYinGua = 34,

    // ===== 应期断语 (35-40) =====
    /// 应期在日：近日可见分晓
    YingQiZaiRi = 35,
    /// 应期在月：本月可见分晓
    YingQiZaiYue = 36,
    /// 应期在季：本季可见分晓
    YingQiZaiJi = 37,
    /// 应期在年：年内可见分晓
    YingQiZaiNian = 38,
    /// 应期待冲：待冲空之日
    YingQiDaiChong = 39,
    /// 应期待合：待合之日
    YingQiDaiHe = 40,
}

impl JieGuaTextType {
    /// 获取解卦文本
    pub fn text(&self) -> &'static str {
        match self {
            Self::DaJiZongDuan => "大吉：诸事顺遂，心想事成",
            Self::JiZongDuan => "吉：事可成，宜进取",
            Self::XiaoJiZongDuan => "小吉：小有所得，不宜大动",
            Self::PingZongDuan => "平：平稳无波，守成为上",
            Self::XiaoXiongZongDuan => "小凶：小有阻碍，谨慎行事",
            Self::XiongZongDuan => "凶：事难成，宜退守",
            Self::DaXiongZongDuan => "大凶：诸事不利，静待时机",
            Self::YongShenWangXiang => "用神旺相：所求之事有望",
            Self::YongShenXiuQiu => "用神休囚：所求之事难成",
            Self::YongShenHuaJin => "用神动而化进：事情向好发展",
            Self::YongShenHuaTui => "用神动而化退：事情有退步之象",
            Self::YongShenKong => "用神逢空：所求之事虚而不实",
            Self::YongShenRuMu => "用神入墓：事情受阻，需待时机",
            Self::YongShenFuCang => "用神伏藏：所求之事隐而未显",
            Self::YongShenShouKe => "用神受克：所求之事受阻",
            Self::YongShenDeSheng => "用神得生：所求之事有贵人相助",
            Self::YongShenFaDong => "用神发动：事情有变化",
            Self::ShiYingXiangSheng => "世应相生：双方和谐，事易成",
            Self::ShiYingXiangKe => "世应相克：双方有冲突",
            Self::ShiYingBiHe => "世应比和：双方势均力敌",
            Self::ShiWangYingShuai => "世爻旺应爻衰：我强彼弱",
            Self::ShiShuaiYingWang => "世爻衰应爻旺：我弱彼强",
            Self::ShiYingJuKong => "世应俱空：双方皆虚",
            Self::WuDongYao => "无动爻：事情平稳，无大变化",
            Self::YiYaoDuFa => "一爻独发：事情明确，吉凶易断",
            Self::DuoYaoQiDong => "多爻齐动：事情复杂，变数较多",
            Self::LiuYaoJieDong => "六爻皆动：大变之象，需谨慎",
            Self::DongYaoHuaJin => "动爻化进：事情向好发展",
            Self::DongYaoHuaTui => "动爻化退：事情有退步之象",
            Self::YongShenRiChong => "用神逢日冲：近期有变",
            Self::YongShenYuePo => "用神逢月破：本月不利",
            Self::GuaFengLiuChong => "卦逢六冲：事情难成或有变",
            Self::GuaFengLiuHe => "卦逢六合：事情顺利",
            Self::FanYinGua => "反吟卦：事情反复",
            Self::FuYinGua => "伏吟卦：事情停滞",
            Self::YingQiZaiRi => "应期在日：近日可见分晓",
            Self::YingQiZaiYue => "应期在月：本月可见分晓",
            Self::YingQiZaiJi => "应期在季：本季可见分晓",
            Self::YingQiZaiNian => "应期在年：年内可见分晓",
            Self::YingQiDaiChong => "应期待冲：待冲空之日",
            Self::YingQiDaiHe => "应期待合：待合之日",
        }
    }
}

// ============================================================================
// 核心解卦结构
// ============================================================================

/// 六爻核心解卦结果
///
/// 包含六爻占卜的核心判断指标
/// 总大小：约 20 bytes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct LiuYaoCoreInterpretation {
    // ===== 基础判断 (4 bytes) =====

    /// 总体吉凶 (1 byte)
    pub ji_xiong: JiXiongLevel,

    /// 用神六亲 (1 byte) - 根据占问事项确定
    pub yong_shen_qin: LiuQin,

    /// 用神状态 (1 byte)
    pub yong_shen_state: YongShenState,

    /// 用神所在爻位 (1 byte, 0-5, 255=伏神)
    pub yong_shen_pos: u8,

    // ===== 动态分析 (4 bytes) =====

    /// 世爻状态 (1 byte)
    pub shi_yao_state: YongShenState,

    /// 应爻状态 (1 byte)
    pub ying_yao_state: YongShenState,

    /// 动爻数量 (1 byte, 0-6)
    pub dong_yao_count: u8,

    /// 主要动爻位置 (1 byte, 位图)
    pub dong_yao_bitmap: u8,

    // ===== 特殊状态 (4 bytes) =====

    /// 旬空爻位图 (1 byte) - 哪些爻逢空
    pub xun_kong_bitmap: u8,

    /// 月破爻位图 (1 byte) - 哪些爻月破
    pub yue_po_bitmap: u8,

    /// 日冲爻位图 (1 byte) - 哪些爻日冲
    pub ri_chong_bitmap: u8,

    /// 化空/化退位图 (1 byte) - 动爻变化状态
    pub hua_kong_bitmap: u8,

    // ===== 应期与评分 (4 bytes) =====

    /// 应期类型 (1 byte)
    pub ying_qi: YingQiType,

    /// 应期地支 (1 byte, 0-11)
    pub ying_qi_zhi: u8,

    /// 综合评分 (1 byte, 0-100)
    pub score: u8,

    /// 可信度 (1 byte, 0-100)
    pub confidence: u8,

    // ===== 元数据 (4 bytes) =====

    /// 解卦时间戳 - 区块号 (4 bytes)
    pub timestamp: u32,
}

impl LiuYaoCoreInterpretation {
    /// 创建新的核心解卦结果
    pub fn new(timestamp: u32) -> Self {
        Self {
            ji_xiong: JiXiongLevel::Ping,
            yong_shen_qin: LiuQin::XiongDi,
            yong_shen_state: YongShenState::WangXiang,
            yong_shen_pos: 255,
            shi_yao_state: YongShenState::WangXiang,
            ying_yao_state: YongShenState::WangXiang,
            dong_yao_count: 0,
            dong_yao_bitmap: 0,
            xun_kong_bitmap: 0,
            yue_po_bitmap: 0,
            ri_chong_bitmap: 0,
            hua_kong_bitmap: 0,
            ying_qi: YingQiType::BuQueDing,
            ying_qi_zhi: 0,
            score: 50,
            confidence: 50,
            timestamp,
        }
    }

    /// 检查用神是否有利
    pub fn is_yong_shen_favorable(&self) -> bool {
        self.yong_shen_state.is_favorable()
    }

    /// 检查用神是否不利
    pub fn is_yong_shen_unfavorable(&self) -> bool {
        self.yong_shen_state.is_unfavorable()
    }

    /// 获取动爻数量
    pub fn get_dong_yao_count(&self) -> u8 {
        self.dong_yao_count
    }

    /// 检查指定爻位是否为动爻
    pub fn is_dong_yao(&self, pos: u8) -> bool {
        if pos >= 6 {
            return false;
        }
        (self.dong_yao_bitmap >> pos) & 1 == 1
    }

    /// 检查指定爻位是否逢空
    pub fn is_xun_kong(&self, pos: u8) -> bool {
        if pos >= 6 {
            return false;
        }
        (self.xun_kong_bitmap >> pos) & 1 == 1
    }

    /// 检查指定爻位是否月破
    pub fn is_yue_po(&self, pos: u8) -> bool {
        if pos >= 6 {
            return false;
        }
        (self.yue_po_bitmap >> pos) & 1 == 1
    }

    /// 检查指定爻位是否日冲
    pub fn is_ri_chong(&self, pos: u8) -> bool {
        if pos >= 6 {
            return false;
        }
        (self.ri_chong_bitmap >> pos) & 1 == 1
    }
}

// ============================================================================
// 第二阶段：扩展结构
// ============================================================================

/// 动爻变化类型（1 byte）
///
/// 表示动爻变化后的状态
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub enum HuaType {
    /// 化进神 - 变爻地支在本爻地支之后（顺行），事情向好发展
    #[default]
    HuaJin = 0,
    /// 化退神 - 变爻地支在本爻地支之前（逆行），事情有退步之象
    HuaTui = 1,
    /// 化回头生 - 变爻五行生本爻五行，得助力
    HuaHuiTouSheng = 2,
    /// 化回头克 - 变爻五行克本爻五行，受制约
    HuaHuiTouKe = 3,
    /// 化空亡 - 变爻逢空，事情虚而不实
    HuaKong = 4,
    /// 化墓 - 变爻入墓，事情受阻
    HuaMu = 5,
    /// 化绝 - 变爻逢绝，事情难成
    HuaJue = 6,
    /// 化比和 - 变爻与本爻五行相同
    HuaBiHe = 7,
}

impl HuaType {
    /// 获取变化类型名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::HuaJin => "化进",
            Self::HuaTui => "化退",
            Self::HuaHuiTouSheng => "化回头生",
            Self::HuaHuiTouKe => "化回头克",
            Self::HuaKong => "化空",
            Self::HuaMu => "化墓",
            Self::HuaJue => "化绝",
            Self::HuaBiHe => "化比和",
        }
    }

    /// 判断是否为有利变化
    pub fn is_favorable(&self) -> bool {
        matches!(self, Self::HuaJin | Self::HuaHuiTouSheng | Self::HuaBiHe)
    }

    /// 判断是否为不利变化
    pub fn is_unfavorable(&self) -> bool {
        matches!(self, Self::HuaTui | Self::HuaHuiTouKe | Self::HuaKong | Self::HuaMu | Self::HuaJue)
    }
}

// ============================================================================
// 单爻分析结构
// ============================================================================

/// 单爻分析结果
///
/// 包含单个爻位的详细分析信息
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub struct YaoAnalysis {
    /// 爻位 (0-5，初爻到上爻)
    pub position: u8,

    /// 旺衰状态
    pub wang_shuai: YongShenState,

    /// 是否逢空
    pub is_kong: bool,

    /// 是否月破
    pub is_yue_po: bool,

    /// 是否日冲
    pub is_ri_chong: bool,

    /// 是否动爻
    pub is_dong: bool,

    /// 动爻变化类型（如果是动爻）
    /// 使用 u8 存储，255 表示非动爻
    pub hua_type: u8,

    /// 神煞数量（最多4个）
    pub shen_sha_count: u8,

    /// 神煞索引（最多4个，每个1字节）
    pub shen_sha_1: u8,
    pub shen_sha_2: u8,
    pub shen_sha_3: u8,
    pub shen_sha_4: u8,
}

impl YaoAnalysis {
    /// 创建新的爻分析
    pub fn new(position: u8) -> Self {
        Self {
            position,
            wang_shuai: YongShenState::WangXiang,
            is_kong: false,
            is_yue_po: false,
            is_ri_chong: false,
            is_dong: false,
            hua_type: 255, // 非动爻
            shen_sha_count: 0,
            shen_sha_1: 255,
            shen_sha_2: 255,
            shen_sha_3: 255,
            shen_sha_4: 255,
        }
    }

    /// 获取动爻变化类型
    pub fn get_hua_type(&self) -> Option<HuaType> {
        if self.hua_type < 8 {
            Some(match self.hua_type {
                0 => HuaType::HuaJin,
                1 => HuaType::HuaTui,
                2 => HuaType::HuaHuiTouSheng,
                3 => HuaType::HuaHuiTouKe,
                4 => HuaType::HuaKong,
                5 => HuaType::HuaMu,
                6 => HuaType::HuaJue,
                _ => HuaType::HuaBiHe,
            })
        } else {
            None
        }
    }

    /// 设置动爻变化类型
    pub fn set_hua_type(&mut self, hua: HuaType) {
        self.hua_type = hua as u8;
    }

    /// 判断爻位是否有利
    pub fn is_favorable(&self) -> bool {
        self.wang_shuai.is_favorable() && !self.is_kong && !self.is_yue_po
    }

    /// 判断爻位是否不利
    pub fn is_unfavorable(&self) -> bool {
        self.wang_shuai.is_unfavorable() || self.is_kong || self.is_yue_po
    }
}

// ============================================================================
// 六亲分析结构
// ============================================================================

/// 单个六亲状态
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub struct QinState {
    /// 出现次数 (0-6)
    pub count: u8,

    /// 爻位列表（位图，bit0=初爻，bit5=上爻）
    pub positions: u8,

    /// 是否有伏神
    pub has_fu_shen: bool,

    /// 伏神位置（如果有，0-5，255=无）
    pub fu_shen_pos: u8,

    /// 整体旺衰
    pub wang_shuai: YongShenState,
}

impl QinState {
    /// 创建新的六亲状态
    pub fn new() -> Self {
        Self {
            count: 0,
            positions: 0,
            has_fu_shen: false,
            fu_shen_pos: 255,
            wang_shuai: YongShenState::WangXiang,
        }
    }

    /// 添加爻位
    pub fn add_position(&mut self, pos: u8) {
        if pos < 6 {
            self.positions |= 1 << pos;
            self.count = self.positions.count_ones() as u8;
        }
    }

    /// 检查是否在指定爻位
    pub fn has_position(&self, pos: u8) -> bool {
        if pos >= 6 {
            return false;
        }
        (self.positions >> pos) & 1 == 1
    }

    /// 获取所有爻位
    pub fn get_positions(&self) -> [bool; 6] {
        let mut result = [false; 6];
        for i in 0..6 {
            result[i] = self.has_position(i as u8);
        }
        result
    }
}

/// 六亲状态分析
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub struct LiuQinAnalysis {
    /// 父母爻状态
    pub fu_mu: QinState,
    /// 兄弟爻状态
    pub xiong_di: QinState,
    /// 子孙爻状态
    pub zi_sun: QinState,
    /// 妻财爻状态
    pub qi_cai: QinState,
    /// 官鬼爻状态
    pub guan_gui: QinState,
}

impl LiuQinAnalysis {
    /// 创建新的六亲分析
    pub fn new() -> Self {
        Self {
            fu_mu: QinState::new(),
            xiong_di: QinState::new(),
            zi_sun: QinState::new(),
            qi_cai: QinState::new(),
            guan_gui: QinState::new(),
        }
    }

    /// 根据六亲类型获取状态
    pub fn get_qin_state(&self, qin: LiuQin) -> &QinState {
        match qin {
            LiuQin::FuMu => &self.fu_mu,
            LiuQin::XiongDi => &self.xiong_di,
            LiuQin::ZiSun => &self.zi_sun,
            LiuQin::QiCai => &self.qi_cai,
            LiuQin::GuanGui => &self.guan_gui,
        }
    }

    /// 根据六亲类型获取可变状态
    pub fn get_qin_state_mut(&mut self, qin: LiuQin) -> &mut QinState {
        match qin {
            LiuQin::FuMu => &mut self.fu_mu,
            LiuQin::XiongDi => &mut self.xiong_di,
            LiuQin::ZiSun => &mut self.zi_sun,
            LiuQin::QiCai => &mut self.qi_cai,
            LiuQin::GuanGui => &mut self.guan_gui,
        }
    }

    /// 检查是否缺少某个六亲
    pub fn is_missing(&self, qin: LiuQin) -> bool {
        let state = self.get_qin_state(qin);
        state.count == 0 && !state.has_fu_shen
    }
}

// ============================================================================
// 卦象分析结构
// ============================================================================

/// 卦象综合分析
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub struct GuaXiangAnalysis {
    /// 本卦卦名索引 (0-63)
    pub ben_gua_idx: u8,

    /// 变卦卦名索引 (0-63, 255=无变卦)
    pub bian_gua_idx: u8,

    /// 互卦卦名索引 (0-63)
    pub hu_gua_idx: u8,

    /// 卦宫 (0-7，对应八卦)
    pub gong: u8,

    /// 卦序 (0-7，本宫/一世/二世/.../游魂/归魂)
    pub gua_xu: u8,

    /// 世爻位置 (0-5)
    pub shi_pos: u8,

    /// 应爻位置 (0-5)
    pub ying_pos: u8,

    /// 卦身地支 (0-11)
    pub gua_shen: u8,

    /// 本卦五行 (0-4)
    pub ben_gua_wuxing: u8,

    /// 变卦五行 (0-4, 255=无变卦)
    pub bian_gua_wuxing: u8,

    /// 是否六冲卦
    pub is_liu_chong: bool,

    /// 是否六合卦
    pub is_liu_he: bool,

    /// 是否反吟卦
    pub is_fan_yin: bool,

    /// 是否伏吟卦
    pub is_fu_yin: bool,
}

impl GuaXiangAnalysis {
    /// 创建新的卦象分析
    pub fn new() -> Self {
        Self {
            ben_gua_idx: 0,
            bian_gua_idx: 255,
            hu_gua_idx: 0,
            gong: 0,
            gua_xu: 0,
            shi_pos: 0,
            ying_pos: 3,
            gua_shen: 0,
            ben_gua_wuxing: 0,
            bian_gua_wuxing: 255,
            is_liu_chong: false,
            is_liu_he: false,
            is_fan_yin: false,
            is_fu_yin: false,
        }
    }

    /// 是否有变卦
    pub fn has_bian_gua(&self) -> bool {
        self.bian_gua_idx < 64
    }

    /// 获取卦宫名称
    pub fn gong_name(&self) -> &'static str {
        match self.gong {
            0 => "乾宫",
            1 => "兑宫",
            2 => "离宫",
            3 => "震宫",
            4 => "巽宫",
            5 => "坎宫",
            6 => "艮宫",
            7 => "坤宫",
            _ => "未知",
        }
    }

    /// 获取卦序名称
    pub fn gua_xu_name(&self) -> &'static str {
        match self.gua_xu {
            0 => "本宫",
            1 => "一世",
            2 => "二世",
            3 => "三世",
            4 => "四世",
            5 => "五世",
            6 => "游魂",
            7 => "归魂",
            _ => "未知",
        }
    }
}

// ============================================================================
// 神煞汇总结构
// ============================================================================

/// 神煞汇总
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub struct ShenShaSummary {
    /// 吉神数量
    pub ji_shen_count: u8,

    /// 凶煞数量
    pub xiong_sha_count: u8,

    /// 吉神列表（最多8个，存储神煞索引）
    pub ji_shen: [u8; 8],

    /// 吉神对应爻位（最多8个）
    pub ji_shen_pos: [u8; 8],

    /// 凶煞列表（最多8个，存储神煞索引）
    pub xiong_sha: [u8; 8],

    /// 凶煞对应爻位（最多8个）
    pub xiong_sha_pos: [u8; 8],
}

impl ShenShaSummary {
    /// 创建新的神煞汇总
    pub fn new() -> Self {
        Self {
            ji_shen_count: 0,
            xiong_sha_count: 0,
            ji_shen: [255; 8],
            ji_shen_pos: [255; 8],
            xiong_sha: [255; 8],
            xiong_sha_pos: [255; 8],
        }
    }

    /// 添加吉神
    pub fn add_ji_shen(&mut self, shen_sha_idx: u8, pos: u8) {
        if self.ji_shen_count < 8 {
            let idx = self.ji_shen_count as usize;
            self.ji_shen[idx] = shen_sha_idx;
            self.ji_shen_pos[idx] = pos;
            self.ji_shen_count += 1;
        }
    }

    /// 添加凶煞
    pub fn add_xiong_sha(&mut self, shen_sha_idx: u8, pos: u8) {
        if self.xiong_sha_count < 8 {
            let idx = self.xiong_sha_count as usize;
            self.xiong_sha[idx] = shen_sha_idx;
            self.xiong_sha_pos[idx] = pos;
            self.xiong_sha_count += 1;
        }
    }

    /// 判断吉神是否多于凶煞
    pub fn is_ji_dominant(&self) -> bool {
        self.ji_shen_count > self.xiong_sha_count
    }
}

// ============================================================================
// 完整解卦结构
// ============================================================================

/// 六爻完整解卦结果
///
/// 包含核心指标和所有扩展分析
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct LiuYaoFullInterpretation {
    /// 核心指标（必有）
    pub core: LiuYaoCoreInterpretation,

    /// 卦象分析（必有）
    pub gua_xiang: GuaXiangAnalysis,

    /// 六亲分析
    pub liu_qin: LiuQinAnalysis,

    /// 神煞汇总
    pub shen_sha: ShenShaSummary,

    /// 各爻分析（6个爻）
    pub yao_0: YaoAnalysis,
    pub yao_1: YaoAnalysis,
    pub yao_2: YaoAnalysis,
    pub yao_3: YaoAnalysis,
    pub yao_4: YaoAnalysis,
    pub yao_5: YaoAnalysis,
}

impl LiuYaoFullInterpretation {
    /// 创建新的完整解卦
    pub fn new(timestamp: u32) -> Self {
        Self {
            core: LiuYaoCoreInterpretation::new(timestamp),
            gua_xiang: GuaXiangAnalysis::new(),
            liu_qin: LiuQinAnalysis::new(),
            shen_sha: ShenShaSummary::new(),
            yao_0: YaoAnalysis::new(0),
            yao_1: YaoAnalysis::new(1),
            yao_2: YaoAnalysis::new(2),
            yao_3: YaoAnalysis::new(3),
            yao_4: YaoAnalysis::new(4),
            yao_5: YaoAnalysis::new(5),
        }
    }

    /// 获取指定爻位的分析
    pub fn get_yao(&self, pos: u8) -> Option<&YaoAnalysis> {
        match pos {
            0 => Some(&self.yao_0),
            1 => Some(&self.yao_1),
            2 => Some(&self.yao_2),
            3 => Some(&self.yao_3),
            4 => Some(&self.yao_4),
            5 => Some(&self.yao_5),
            _ => None,
        }
    }

    /// 获取指定爻位的可变分析
    pub fn get_yao_mut(&mut self, pos: u8) -> Option<&mut YaoAnalysis> {
        match pos {
            0 => Some(&mut self.yao_0),
            1 => Some(&mut self.yao_1),
            2 => Some(&mut self.yao_2),
            3 => Some(&mut self.yao_3),
            4 => Some(&mut self.yao_4),
            5 => Some(&mut self.yao_5),
            _ => None,
        }
    }

    /// 获取世爻分析
    pub fn get_shi_yao(&self) -> Option<&YaoAnalysis> {
        self.get_yao(self.gua_xiang.shi_pos)
    }

    /// 获取应爻分析
    pub fn get_ying_yao(&self) -> Option<&YaoAnalysis> {
        self.get_yao(self.gua_xiang.ying_pos)
    }

    /// 获取所有动爻
    pub fn get_dong_yaos(&self) -> [Option<&YaoAnalysis>; 6] {
        let mut result: [Option<&YaoAnalysis>; 6] = [None; 6];
        for i in 0..6 {
            if let Some(yao) = self.get_yao(i as u8) {
                if yao.is_dong {
                    result[i] = Some(yao);
                }
            }
        }
        result
    }
}

// ============================================================================
// 解卦算法
// ============================================================================

use crate::algorithm::calculate_xun_kong;
use crate::types::{DiZhi, LiuYaoGua, WuXing};

/// 判断五行是否生某五行
fn wx_generates(from: WuXing, to: WuXing) -> bool {
    from.generates() == to
}

/// 判断五行是否克某五行
fn wx_restrains(from: WuXing, to: WuXing) -> bool {
    from.restrains() == to
}

/// 判断地支是否相冲
///
/// 六冲：子午、丑未、寅申、卯酉、辰戌、巳亥
fn zhi_is_chong(zhi1: DiZhi, zhi2: DiZhi) -> bool {
    let idx1 = zhi1.index();
    let idx2 = zhi2.index();
    // 相差6位为相冲
    (idx1 + 6) % 12 == idx2 || (idx2 + 6) % 12 == idx1
}

/// 计算爻的旺衰状态
///
/// 根据月令和日辰判断爻的旺衰
///
/// # 参数
/// - `yao_zhi`: 爻的地支
/// - `month_zhi`: 月令地支
/// - `day_zhi`: 日辰地支
///
/// # 返回
/// 用神状态
pub fn calculate_wang_shuai(yao_zhi: DiZhi, month_zhi: DiZhi, day_zhi: DiZhi) -> YongShenState {
    let yao_wx = yao_zhi.wu_xing();
    let month_wx = month_zhi.wu_xing();
    let day_wx = day_zhi.wu_xing();

    // 月令生扶或比和为旺
    let month_sheng = wx_generates(month_wx, yao_wx);
    let month_bi = month_wx == yao_wx;

    // 日辰生扶或比和为相
    let day_sheng = wx_generates(day_wx, yao_wx);
    let day_bi = day_wx == yao_wx;

    // 月克为休，日克为囚
    let month_ke = wx_restrains(month_wx, yao_wx);
    let day_ke = wx_restrains(day_wx, yao_wx);

    if month_sheng || month_bi {
        if day_sheng || day_bi {
            YongShenState::WangXiang // 月日都生扶，旺相
        } else if day_ke {
            YongShenState::WangXiang // 月旺日克，仍为旺
        } else {
            YongShenState::WangXiang
        }
    } else if month_ke {
        if day_sheng || day_bi {
            YongShenState::XiuQiu // 月克日生，休囚
        } else {
            YongShenState::XiuQiu // 月克日不生，休囚
        }
    } else {
        // 月不生不克
        if day_sheng || day_bi {
            YongShenState::WangXiang
        } else if day_ke {
            YongShenState::XiuQiu
        } else {
            YongShenState::WangXiang // 中性状态默认旺相
        }
    }
}

/// 判断地支是否逢空
pub fn is_zhi_kong(zhi: DiZhi, kong1: DiZhi, kong2: DiZhi) -> bool {
    zhi == kong1 || zhi == kong2
}

/// 判断地支是否月破
///
/// 月破：爻与月令相冲
pub fn is_zhi_yue_po(yao_zhi: DiZhi, month_zhi: DiZhi) -> bool {
    zhi_is_chong(yao_zhi, month_zhi)
}

/// 判断地支是否日冲
pub fn is_zhi_ri_chong(yao_zhi: DiZhi, day_zhi: DiZhi) -> bool {
    zhi_is_chong(yao_zhi, day_zhi)
}

/// 计算动爻变化类型
///
/// # 参数
/// - `original_zhi`: 本爻地支
/// - `changed_zhi`: 变爻地支
/// - `kong1`, `kong2`: 旬空地支
///
/// # 返回
/// 变化类型
pub fn calculate_hua_type(
    original_zhi: DiZhi,
    changed_zhi: DiZhi,
    kong1: DiZhi,
    kong2: DiZhi,
) -> HuaType {
    let orig_wx = original_zhi.wu_xing();
    let chan_wx = changed_zhi.wu_xing();

    // 检查化空
    if is_zhi_kong(changed_zhi, kong1, kong2) {
        return HuaType::HuaKong;
    }

    // 检查入墓（简化判断）
    // 木墓在未，火墓在戌，金墓在丑，水墓在辰，土墓在辰
    let mu_zhi = match orig_wx {
        WuXing::Wood => DiZhi::Wei,
        WuXing::Fire => DiZhi::Xu,
        WuXing::Metal => DiZhi::Chou,
        WuXing::Water => DiZhi::Chen,
        WuXing::Earth => DiZhi::Chen,
    };
    if changed_zhi == mu_zhi {
        return HuaType::HuaMu;
    }

    // 检查回头生克
    if wx_generates(chan_wx, orig_wx) {
        return HuaType::HuaHuiTouSheng;
    }
    if wx_restrains(chan_wx, orig_wx) {
        return HuaType::HuaHuiTouKe;
    }

    // 检查比和
    if orig_wx == chan_wx {
        return HuaType::HuaBiHe;
    }

    // 检查化进化退（简化：根据地支序号判断）
    let orig_idx = original_zhi.index();
    let chan_idx = changed_zhi.index();

    // 同五行时判断进退
    if orig_wx == chan_wx {
        if chan_idx > orig_idx {
            return HuaType::HuaJin;
        } else {
            return HuaType::HuaTui;
        }
    }

    // 默认化进
    HuaType::HuaJin
}

/// 计算综合吉凶
///
/// 根据用神状态、世应状态、动爻情况等综合判断
pub fn calculate_ji_xiong(
    yong_shen_state: YongShenState,
    shi_state: YongShenState,
    dong_count: u8,
    has_kong: bool,
    has_yue_po: bool,
) -> JiXiongLevel {
    let mut score: i8 = 50; // 基础分

    // 用神状态评分
    match yong_shen_state {
        YongShenState::WangXiang => score += 20,
        YongShenState::DeSheng => score += 15,
        YongShenState::DongHuaJin => score += 10,
        YongShenState::XiuQiu => score -= 15,
        YongShenState::ShouKe => score -= 20,
        YongShenState::KongWang => score -= 25,
        YongShenState::RuMu => score -= 20,
        YongShenState::DongHuaTui => score -= 10,
        YongShenState::DongHuaKong => score -= 15,
        YongShenState::FuCang => score -= 10,
    }

    // 世爻状态评分
    if shi_state.is_favorable() {
        score += 10;
    } else if shi_state.is_unfavorable() {
        score -= 10;
    }

    // 空亡月破扣分
    if has_kong {
        score -= 15;
    }
    if has_yue_po {
        score -= 20;
    }

    // 动爻数量影响
    if dong_count == 0 {
        // 无动爻，事情平稳
    } else if dong_count == 1 {
        // 一爻独发，吉凶分明
        score += 5;
    } else if dong_count >= 4 {
        // 多爻齐动，变数大
        score -= 5;
    }

    // 转换为吉凶等级
    if score >= 80 {
        JiXiongLevel::DaJi
    } else if score >= 65 {
        JiXiongLevel::Ji
    } else if score >= 55 {
        JiXiongLevel::XiaoJi
    } else if score >= 45 {
        JiXiongLevel::Ping
    } else if score >= 35 {
        JiXiongLevel::XiaoXiong
    } else if score >= 20 {
        JiXiongLevel::Xiong
    } else {
        JiXiongLevel::DaXiong
    }
}

/// 计算应期类型
///
/// 根据用神状态和卦象特征推断应期
pub fn calculate_ying_qi(
    yong_shen_state: YongShenState,
    has_kong: bool,
    dong_count: u8,
) -> YingQiType {
    // 用神逢空，待出空
    if has_kong {
        return YingQiType::ZhongQi; // 季度内
    }

    // 用神旺相，应期近
    if yong_shen_state.is_favorable() {
        if dong_count > 0 {
            return YingQiType::JinQi; // 日内
        } else {
            return YingQiType::DuanQi; // 月内
        }
    }

    // 用神休囚，应期远
    if yong_shen_state.is_unfavorable() {
        return YingQiType::ChangQi; // 年内
    }

    YingQiType::BuQueDing
}

/// 从 LiuYaoGua 计算核心解卦结果
///
/// # 参数
/// - `gua`: 六爻卦象
/// - `shi_xiang`: 占问事项类型
/// - `timestamp`: 当前区块号
///
/// # 返回
/// 核心解卦结果
pub fn calculate_core_interpretation<AccountId, BlockNumber, CidLen>(
    gua: &LiuYaoGua<AccountId, BlockNumber, CidLen>,
    shi_xiang: ShiXiangType,
    timestamp: u32,
) -> LiuYaoCoreInterpretation
where
    CidLen: frame_support::traits::Get<u32>,
{
    let mut result = LiuYaoCoreInterpretation::new(timestamp);

    // 解包 Option 字段（使用默认值处理 Private 模式）
    let original_yaos = gua.original_yaos.unwrap_or_default();
    let changed_yaos = gua.changed_yaos.unwrap_or_default();
    let day_gz = gua.day_gz.unwrap_or_default();
    let month_gz = gua.month_gz.unwrap_or_default();
    let gua_xu = gua.gua_xu.unwrap_or_default();
    let moving_yaos = gua.moving_yaos.unwrap_or(0);

    // 1. 确定用神六亲
    result.yong_shen_qin = shi_xiang.default_yong_shen_qin();

    // 2. 查找用神位置
    let mut yong_shen_pos: u8 = 255;
    for i in 0..6 {
        if original_yaos[i].liu_qin == result.yong_shen_qin {
            yong_shen_pos = i as u8;
            break;
        }
    }
    result.yong_shen_pos = yong_shen_pos;

    // 3. 计算旬空
    let (kong1, kong2) = calculate_xun_kong(day_gz.0, day_gz.1);

    // 4. 计算各爻状态
    let mut xun_kong_bitmap: u8 = 0;
    let mut yue_po_bitmap: u8 = 0;
    let mut ri_chong_bitmap: u8 = 0;

    for i in 0..6 {
        let yao_zhi = original_yaos[i].di_zhi;

        // 检查旬空
        if is_zhi_kong(yao_zhi, kong1, kong2) {
            xun_kong_bitmap |= 1 << i;
        }

        // 检查月破
        if is_zhi_yue_po(yao_zhi, month_gz.1) {
            yue_po_bitmap |= 1 << i;
        }

        // 检查日冲
        if is_zhi_ri_chong(yao_zhi, day_gz.1) {
            ri_chong_bitmap |= 1 << i;
        }
    }

    result.xun_kong_bitmap = xun_kong_bitmap;
    result.yue_po_bitmap = yue_po_bitmap;
    result.ri_chong_bitmap = ri_chong_bitmap;

    // 5. 动爻分析
    result.dong_yao_bitmap = moving_yaos;
    result.dong_yao_count = moving_yaos.count_ones() as u8;

    // 6. 计算用神状态
    if yong_shen_pos < 6 {
        let yong_zhi = original_yaos[yong_shen_pos as usize].di_zhi;
        let yong_kong = is_zhi_kong(yong_zhi, kong1, kong2);
        let yong_yue_po = is_zhi_yue_po(yong_zhi, month_gz.1);

        if yong_kong {
            result.yong_shen_state = YongShenState::KongWang;
        } else if yong_yue_po {
            result.yong_shen_state = YongShenState::ShouKe; // 月破类似受克
        } else {
            result.yong_shen_state =
                calculate_wang_shuai(yong_zhi, month_gz.1, day_gz.1);
        }

        // 检查用神是否动爻
        if (moving_yaos >> yong_shen_pos) & 1 == 1 {
            // 用神发动，检查变化
            if gua.has_bian_gua {
                let changed_zhi = changed_yaos[yong_shen_pos as usize].di_zhi;
                let hua = calculate_hua_type(yong_zhi, changed_zhi, kong1, kong2);
                result.yong_shen_state = match hua {
                    HuaType::HuaJin => YongShenState::DongHuaJin,
                    HuaType::HuaTui => YongShenState::DongHuaTui,
                    HuaType::HuaKong => YongShenState::DongHuaKong,
                    _ => result.yong_shen_state,
                };
            }
        }
    } else {
        // 用神伏藏
        result.yong_shen_state = YongShenState::FuCang;
    }

    // 7. 世应状态
    let shi_pos = gua_xu.shi_yao_pos() as usize - 1;
    let ying_pos = gua_xu.ying_yao_pos() as usize - 1;

    if shi_pos < 6 {
        let shi_zhi = original_yaos[shi_pos].di_zhi;
        let shi_kong = is_zhi_kong(shi_zhi, kong1, kong2);
        if shi_kong {
            result.shi_yao_state = YongShenState::KongWang;
        } else {
            result.shi_yao_state =
                calculate_wang_shuai(shi_zhi, month_gz.1, day_gz.1);
        }
    }

    if ying_pos < 6 {
        let ying_zhi = original_yaos[ying_pos].di_zhi;
        let ying_kong = is_zhi_kong(ying_zhi, kong1, kong2);
        if ying_kong {
            result.ying_yao_state = YongShenState::KongWang;
        } else {
            result.ying_yao_state =
                calculate_wang_shuai(ying_zhi, month_gz.1, day_gz.1);
        }
    }

    // 8. 计算吉凶
    let has_yong_kong =
        yong_shen_pos < 6 && (xun_kong_bitmap >> yong_shen_pos) & 1 == 1;
    let has_yong_po =
        yong_shen_pos < 6 && (yue_po_bitmap >> yong_shen_pos) & 1 == 1;

    result.ji_xiong = calculate_ji_xiong(
        result.yong_shen_state,
        result.shi_yao_state,
        result.dong_yao_count,
        has_yong_kong,
        has_yong_po,
    );

    // 9. 计算应期
    result.ying_qi = calculate_ying_qi(
        result.yong_shen_state,
        has_yong_kong,
        result.dong_yao_count,
    );

    // 10. 计算综合评分
    result.score = match result.ji_xiong {
        JiXiongLevel::DaJi => 90,
        JiXiongLevel::Ji => 75,
        JiXiongLevel::XiaoJi => 60,
        JiXiongLevel::Ping => 50,
        JiXiongLevel::XiaoXiong => 40,
        JiXiongLevel::Xiong => 25,
        JiXiongLevel::DaXiong => 10,
    };

    // 11. 可信度评估
    result.confidence = if result.dong_yao_count == 1 {
        85 // 一爻独发，判断明确
    } else if result.dong_yao_count == 0 {
        75 // 静卦，相对稳定
    } else if result.dong_yao_count <= 3 {
        70 // 多爻动，变数较大
    } else {
        60 // 多爻齐动，判断困难
    };

    result
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use codec::Encode;

    #[test]
    fn test_core_interpretation_size() {
        let core = LiuYaoCoreInterpretation::new(1000000);
        let encoded = core.encode();
        println!("✅ LiuYaoCoreInterpretation 编码大小: {} bytes", encoded.len());
        assert!(encoded.len() <= 25, "核心解卦结构应该 <= 25 bytes");
    }

    #[test]
    fn test_ji_xiong_level() {
        assert_eq!(JiXiongLevel::DaJi.name(), "大吉");
        assert!(JiXiongLevel::DaJi.is_ji());
        assert!(!JiXiongLevel::DaXiong.is_ji());
        assert!(JiXiongLevel::DaXiong.is_xiong());
    }

    #[test]
    fn test_yong_shen_state() {
        assert!(YongShenState::WangXiang.is_favorable());
        assert!(YongShenState::XiuQiu.is_unfavorable());
        assert_eq!(YongShenState::WangXiang.name(), "旺相");
    }

    #[test]
    fn test_shi_xiang_type() {
        assert_eq!(ShiXiangType::CaiYun.name(), "财运");
        assert_eq!(ShiXiangType::CaiYun.default_yong_shen_qin(), LiuQin::QiCai);
    }

    #[test]
    fn test_ying_qi_type() {
        assert_eq!(YingQiType::JinQi.name(), "近期（日内）");
    }

    #[test]
    fn test_jie_gua_text_type() {
        assert_eq!(JieGuaTextType::DaJiZongDuan.text(), "大吉：诸事顺遂，心想事成");
    }

    #[test]
    fn test_core_interpretation_methods() {
        let mut core = LiuYaoCoreInterpretation::new(1000000);
        core.dong_yao_bitmap = 0b000101; // 第0和2爻为动爻
        core.xun_kong_bitmap = 0b001000; // 第3爻逢空

        assert!(core.is_dong_yao(0));
        assert!(!core.is_dong_yao(1));
        assert!(core.is_dong_yao(2));
        assert!(core.is_xun_kong(3));
        assert!(!core.is_xun_kong(0));
    }

    // ========== 第二阶段测试 ==========

    #[test]
    fn test_hua_type() {
        // 测试化进化退
        assert_eq!(HuaType::HuaJin.name(), "化进");
        assert_eq!(HuaType::HuaTui.name(), "化退");
        assert_eq!(HuaType::HuaHuiTouSheng.name(), "化回头生");
        assert_eq!(HuaType::HuaHuiTouKe.name(), "化回头克");

        // 测试有利/不利判断
        assert!(HuaType::HuaJin.is_favorable());
        assert!(HuaType::HuaHuiTouSheng.is_favorable());
        assert!(HuaType::HuaBiHe.is_favorable());
        assert!(HuaType::HuaTui.is_unfavorable());
        assert!(HuaType::HuaKong.is_unfavorable());
        assert!(HuaType::HuaMu.is_unfavorable());
    }

    #[test]
    fn test_yao_analysis() {
        // 测试创建
        let yao = YaoAnalysis::new(2);
        assert_eq!(yao.position, 2);
        assert_eq!(yao.wang_shuai, YongShenState::WangXiang);
        assert!(!yao.is_kong);
        assert!(!yao.is_dong);
        assert_eq!(yao.hua_type, 255);

        // 测试有利判断
        assert!(yao.is_favorable());

        // 测试设置变化类型
        let mut yao2 = YaoAnalysis::new(0);
        yao2.set_hua_type(HuaType::HuaJin);
        assert_eq!(yao2.hua_type, 0);
        assert_eq!(yao2.get_hua_type(), Some(HuaType::HuaJin));

        // 测试不利状态
        let mut yao3 = YaoAnalysis::new(1);
        yao3.is_kong = true;
        assert!(!yao3.is_favorable());
        assert!(yao3.is_unfavorable());
    }

    #[test]
    fn test_qin_state() {
        let mut qin = QinState::new();
        assert_eq!(qin.count, 0);
        assert_eq!(qin.positions, 0);

        // 添加爻位
        qin.add_position(0);
        qin.add_position(3);
        assert_eq!(qin.count, 2);
        assert!(qin.has_position(0));
        assert!(qin.has_position(3));
        assert!(!qin.has_position(1));

        // 获取所有爻位
        let positions = qin.get_positions();
        assert!(positions[0]);
        assert!(!positions[1]);
        assert!(!positions[2]);
        assert!(positions[3]);
    }

    #[test]
    fn test_liu_qin_analysis() {
        let mut analysis = LiuQinAnalysis::new();

        // 添加父母爻位置
        analysis.get_qin_state_mut(LiuQin::FuMu).add_position(2);
        assert_eq!(analysis.fu_mu.count, 1);
        assert!(analysis.fu_mu.has_position(2));

        // 检查是否缺少
        assert!(!analysis.is_missing(LiuQin::FuMu));
        assert!(analysis.is_missing(LiuQin::QiCai)); // 没有妻财
    }

    #[test]
    fn test_gua_xiang_analysis() {
        let gua = GuaXiangAnalysis::new();
        assert_eq!(gua.ben_gua_idx, 0);
        assert_eq!(gua.bian_gua_idx, 255);
        assert!(!gua.has_bian_gua());

        // 测试卦宫名称
        let mut gua2 = GuaXiangAnalysis::new();
        gua2.gong = 0;
        assert_eq!(gua2.gong_name(), "乾宫");
        gua2.gong = 7;
        assert_eq!(gua2.gong_name(), "坤宫");

        // 测试卦序名称
        gua2.gua_xu = 0;
        assert_eq!(gua2.gua_xu_name(), "本宫");
        gua2.gua_xu = 6;
        assert_eq!(gua2.gua_xu_name(), "游魂");
    }

    #[test]
    fn test_shen_sha_summary() {
        let mut summary = ShenShaSummary::new();
        assert_eq!(summary.ji_shen_count, 0);
        assert_eq!(summary.xiong_sha_count, 0);

        // 添加吉神
        summary.add_ji_shen(1, 0);
        summary.add_ji_shen(2, 1);
        assert_eq!(summary.ji_shen_count, 2);
        assert_eq!(summary.ji_shen[0], 1);
        assert_eq!(summary.ji_shen_pos[0], 0);

        // 添加凶煞
        summary.add_xiong_sha(5, 2);
        assert_eq!(summary.xiong_sha_count, 1);

        // 测试吉凶对比
        assert!(summary.is_ji_dominant());
    }

    #[test]
    fn test_full_interpretation() {
        let full = LiuYaoFullInterpretation::new(1000000);

        // 测试获取爻分析
        assert!(full.get_yao(0).is_some());
        assert!(full.get_yao(5).is_some());
        assert!(full.get_yao(6).is_none());

        // 测试编码大小
        let encoded = full.encode();
        println!("✅ LiuYaoFullInterpretation 编码大小: {} bytes", encoded.len());
        // 完整解卦应该在合理范围内（约 100-200 bytes）
        assert!(encoded.len() <= 250, "完整解卦结构应该 <= 250 bytes");
    }

    #[test]
    fn test_get_dong_yaos() {
        let mut full = LiuYaoFullInterpretation::new(1000000);

        // 设置动爻
        full.yao_0.is_dong = true;
        full.yao_2.is_dong = true;

        let dong_yaos = full.get_dong_yaos();
        assert!(dong_yaos[0].is_some());
        assert!(dong_yaos[1].is_none());
        assert!(dong_yaos[2].is_some());
    }
}
