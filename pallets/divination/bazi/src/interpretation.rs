//! # 八字解盘 V3 - 轻量化实现
//!
//! 合并 V1 和 V2 的优点：
//! - 保留 V2 的轻量化核心指标（13 bytes）
//! - 保留 V1 的性格分析和职业建议
//! - 新增分层存储设计
//! - 支持 Runtime API 实时计算
//!
//! ## 特点
//!
//! - **独立模块**：不依赖 interpretation.rs，可独立使用
//! - **存储优化**：核心数据仅 13 bytes（81% 优化）
//! - **实时计算**：通过 Runtime API 免费获取
//! - **算法升级**：无需数据迁移，立即生效

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use frame_support::pallet_prelude::*;
use crate::types::*;

// ================================
// 类型定义（V3 独立版本）
// ================================

/// 格局类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub enum GeJuType {
    #[default]
    /// 正格 - 身旺财官
    ZhengGe,
    /// 从强格 - 身旺无制
    CongQiangGe,
    /// 从弱格 - 身弱无助
    CongRuoGe,
    /// 从财格 - 财星当令
    CongCaiGe,
    /// 从官格 - 官星当令
    CongGuanGe,
    /// 从儿格 - 食伤当令
    CongErGe,
    /// 化气格 - 干支化合
    HuaQiGe,
    /// 特殊格局
    TeShuge,
}

/// 命局强弱
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub enum MingJuQiangRuo {
    /// 身旺
    ShenWang,
    /// 身弱
    ShenRuo,
    #[default]
    /// 中和
    ZhongHe,
    /// 太旺
    TaiWang,
    /// 太弱
    TaiRuo,
}

/// 用神类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub enum YongShenType {
    #[default]
    /// 扶抑用神 - 扶弱抑强
    FuYi,
    /// 调候用神 - 调节寒暖
    DiaoHou,
    /// 通关用神 - 化解冲突
    TongGuan,
    /// 专旺用神 - 顺势而为
    ZhuanWang,
}

/// 性格特征枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum XingGeTrait {
    /// 正直
    ZhengZhi,
    /// 有主见
    YouZhuJian,
    /// 积极向上
    JiJiXiangShang,
    /// 固执
    GuZhi,
    /// 缺乏变通
    QueFaBianTong,
    /// 温和
    WenHe,
    /// 适应性强
    ShiYingXingQiang,
    /// 有艺术天赋
    YouYiShuTianFu,
    /// 优柔寡断
    YouRouGuaDuan,
    /// 依赖性强
    YiLaiXingQiang,
    /// 热情
    ReQing,
    /// 开朗
    KaiLang,
    /// 有领导力
    YouLingDaoLi,
    /// 急躁
    JiZao,
    /// 缺乏耐心
    QueFaNaiXin,
    /// 细心
    XiXin,
    /// 有创造力
    YouChuangZaoLi,
    /// 善于沟通
    ShanYuGouTong,
    /// 情绪化
    QingXuHua,
    /// 敏感
    MinGan,
    /// 稳重
    WenZhong,
    /// 可靠
    KeLao,
    /// 有责任心
    YouZeRenXin,
    /// 保守
    BaoShou,
    /// 变化慢
    BianHuaMan,
    /// 包容
    BaoRong,
    /// 细致
    XiZhi,
    /// 善于协调
    ShanYuXieTiao,
    /// 犹豫不决
    YouYuBuJue,
    /// 缺乏魄力
    QueFaPoLi,
    /// 果断
    GuoDuan,
    /// 有正义感
    YouZhengYiGan,
    /// 执行力强
    ZhiXingLiQiang,
    /// 刚硬
    GangYing,
    /// 不够圆滑
    BuGouYuanHua,
    /// 精致
    JingZhi,
    /// 有品味
    YouPinWei,
    /// 善于表达
    ShanYuBiaoDa,
    /// 挑剔
    TiaoTi,
    /// 情绪波动大
    QingXuBoDongDa,
    /// 智慧
    ZhiHui,
    /// 灵活
    LingHuo,
    /// 适应力强
    ShiYingLiQiang,
    /// 多变
    DuoBian,
    /// 缺乏恒心
    QueFaHengXin,
    /// 内敛
    NeiLian,
    /// 善于思考
    ShanYuSiKao,
    /// 消极
    XiaoJi,
    /// 缺乏自信
    QueFaZiXin,
}

/// 职业类型枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ZhiYeType {
    /// 教育
    JiaoYu,
    /// 文化
    WenHua,
    /// 环保
    HuanBao,
    /// 农林
    NongLin,
    /// 能源
    NengYuan,
    /// 娱乐
    YuLe,
    /// 餐饮
    CanYin,
    /// 化工
    HuaGong,
    /// 房地产
    FangDiChan,
    /// 建筑
    JianZhu,
    /// 农业
    NongYe,
    /// 服务
    FuWu,
    /// 金融
    JinRong,
    /// 机械
    JiXie,
    /// 军警
    JunJing,
    /// 五金
    WuJin,
    /// 贸易
    MaoYi,
    /// 运输
    YunShu,
    /// 水利
    ShuiLi,
    /// 信息
    XinXi,
}

// ================================
// Layer 1: 核心指标（13 bytes）
// ================================

/// V3 核心解盘结果
///
/// 包含八字命理的核心指标，存储空间优化
/// 总大小：13 bytes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct CoreInterpretation {
    /// 格局类型 (1 byte)
    pub ge_ju: GeJuType,
    /// 命局强弱 (1 byte)
    pub qiang_ruo: MingJuQiangRuo,
    /// 用神 (1 byte)
    pub yong_shen: WuXing,
    /// 用神类型 (1 byte)
    pub yong_shen_type: YongShenType,
    /// 喜神 (1 byte)
    pub xi_shen: WuXing,
    /// 主忌神 (1 byte)
    pub ji_shen: WuXing,
    /// 综合评分 0-100 (1 byte)
    pub score: u8,
    /// 可信度 0-100 (1 byte)
    pub confidence: u8,
    /// 解盘时间戳 - 区块号 (4 bytes)
    pub timestamp: u32,
    /// 算法版本 (1 byte)
    pub algorithm_version: u8,
}

// ================================
// Layer 2: 扩展数据
// ================================

/// 性格分析（压缩版）
///
/// 使用索引存储，减少空间占用
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct CompactXingGe {
    /// 主要性格特点（最多 3 个）
    pub zhu_yao_te_dian: BoundedVec<XingGeTrait, ConstU32<3>>,
    /// 优点（最多 3 个）
    pub you_dian: BoundedVec<XingGeTrait, ConstU32<3>>,
    /// 缺点（最多 2 个）
    pub que_dian: BoundedVec<XingGeTrait, ConstU32<2>>,
    /// 适合职业（最多 4 个）
    pub shi_he_zhi_ye: BoundedVec<ZhiYeType, ConstU32<4>>,
}

/// 扩展忌神
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ExtendedJiShen {
    /// 次忌神列表（最多 2 个）
    pub secondary: BoundedVec<WuXing, ConstU32<2>>,
}

/// V3 完整解盘结果
///
/// 包含核心指标和扩展数据
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct FullInterpretation {
    /// 核心指标（必有）
    pub core: CoreInterpretation,
    /// 性格分析（可选）
    pub xing_ge: Option<CompactXingGe>,
    /// 扩展忌神（可选）
    pub extended_ji_shen: Option<ExtendedJiShen>,
}

// ================================
// V3 解盘算法
// ================================

/// 计算核心解盘（Layer 1）
///
/// 免费实时计算，不存储
/// 要求命盘有计算数据（sizhu 和 wuxing_strength 必须存在）
pub fn calculate_core_interpretation<T: crate::pallet::Config>(
    chart: &BaziChart<T>,
    current_block: u32,
) -> CoreInterpretation {
    // 获取必要数据（如果不存在则返回默认值）
    let sizhu = match &chart.sizhu {
        Some(s) => s,
        None => {
            return CoreInterpretation {
                ge_ju: GeJuType::ZhengGe,
                qiang_ruo: MingJuQiangRuo::ZhongHe,
                yong_shen: WuXing::Tu,
                yong_shen_type: YongShenType::FuYi,
                xi_shen: WuXing::Huo,
                ji_shen: WuXing::Shui,
                score: 50,
                confidence: 0,
                timestamp: current_block,
                algorithm_version: 3,
            };
        }
    };

    let wuxing_strength = chart.wuxing_strength.unwrap_or_default();

    // 1. 分析格局
    let ge_ju = analyze_ge_ju(sizhu, &wuxing_strength);

    // 2. 分析强弱
    let qiang_ruo = analyze_qiang_ruo(&wuxing_strength, sizhu.rizhu);

    // 3. 分析用神
    let (yong_shen, yong_shen_type) = analyze_yong_shen(
        ge_ju,
        qiang_ruo,
        sizhu,
        &wuxing_strength,
    );

    // 4. 推导喜神
    let xi_shen = derive_xi_shen(yong_shen);

    // 5. 推导忌神
    let ji_shen = derive_ji_shen(yong_shen, qiang_ruo, sizhu.rizhu);

    // 6. 计算综合评分
    let score = calculate_comprehensive_score(ge_ju, qiang_ruo, &wuxing_strength);

    // 7. 计算可信度
    let confidence = calculate_confidence_score(chart, ge_ju, &wuxing_strength);

    CoreInterpretation {
        ge_ju,
        qiang_ruo,
        yong_shen,
        yong_shen_type,
        xi_shen,
        ji_shen,
        score,
        confidence,
        timestamp: current_block,
        algorithm_version: 3,
    }
}

/// 计算完整解盘（Layer 1 + Layer 2）
///
/// 包含性格分析和扩展忌神
pub fn calculate_full_interpretation<T: crate::pallet::Config>(
    chart: &BaziChart<T>,
    current_block: u32,
) -> FullInterpretation {
    // 1. 计算核心指标
    let core = calculate_core_interpretation(chart, current_block);

    // 如果没有四柱数据，返回仅有核心指标的结果
    let sizhu = match &chart.sizhu {
        Some(s) => s,
        None => {
            return FullInterpretation {
                core,
                xing_ge: None,
                extended_ji_shen: None,
            };
        }
    };

    // 2. 计算性格分析
    let xing_ge = Some(analyze_xing_ge(sizhu));

    // 3. 计算扩展忌神
    let extended_ji_shen = Some(analyze_extended_ji_shen(
        core.yong_shen,
        core.qiang_ruo,
        sizhu.rizhu,
    ));

    FullInterpretation {
        core,
        xing_ge,
        extended_ji_shen,
    }
}

/// 基于四柱索引计算完整解盘（用于加密命盘）
///
/// 此函数基于 SiZhuIndex 进行计算，无需访问敏感数据
///
/// # 参数
/// - sizhu_index: 四柱干支索引（8 bytes）
/// - gender: 性别（用于大运方向判断，但此处简化处理）
/// - current_block: 当前区块号
///
/// # 返回
/// - FullInterpretation: 完整解盘结果
///
/// # 限制
/// - 由于没有完整的 BaziChart，五行强度使用简化计算
/// - 可信度会略低于完整命盘的解盘
pub fn calculate_interpretation_from_index(
    sizhu_index: &SiZhuIndex,
    _gender: Gender,
    current_block: u32,
) -> FullInterpretation {
    // 1. 从索引重建干支信息
    let year_ganzhi = sizhu_index.year_ganzhi();
    let month_ganzhi = sizhu_index.month_ganzhi();
    let day_ganzhi = sizhu_index.day_ganzhi();
    let hour_ganzhi = sizhu_index.hour_ganzhi();
    let rizhu = sizhu_index.rizhu();

    // 2. 计算简化的五行强度（基于干支）
    let wuxing_strength = calculate_simple_wuxing_strength(
        &year_ganzhi,
        &month_ganzhi,
        &day_ganzhi,
        &hour_ganzhi,
    );

    // 3. 分析格局（使用简化版本）
    let ge_ju = analyze_ge_ju_from_index(sizhu_index, &wuxing_strength);

    // 4. 分析强弱
    let qiang_ruo = analyze_qiang_ruo(&wuxing_strength, rizhu);

    // 5. 分析用神
    let (yong_shen, yong_shen_type) = analyze_yong_shen_from_index(
        ge_ju,
        qiang_ruo,
        sizhu_index,
        &wuxing_strength,
    );

    // 6. 推导喜神和忌神
    let xi_shen = derive_xi_shen(yong_shen);
    let ji_shen = derive_ji_shen(yong_shen, qiang_ruo, rizhu);

    // 7. 计算综合评分
    let score = calculate_comprehensive_score(ge_ju, qiang_ruo, &wuxing_strength);

    // 8. 计算可信度（因为是索引计算，可信度略低）
    let confidence = calculate_index_confidence(ge_ju, &wuxing_strength);

    let core = CoreInterpretation {
        ge_ju,
        qiang_ruo,
        yong_shen,
        yong_shen_type,
        xi_shen,
        ji_shen,
        score,
        confidence,
        timestamp: current_block,
        algorithm_version: 3,
    };

    // 9. 计算性格分析
    let xing_ge = Some(analyze_xing_ge_from_index(sizhu_index));

    // 10. 计算扩展忌神
    let extended_ji_shen = Some(analyze_extended_ji_shen(
        yong_shen,
        qiang_ruo,
        rizhu,
    ));

    FullInterpretation {
        core,
        xing_ge,
        extended_ji_shen,
    }
}

// ================================
// 核心算法实现
// ================================

/// 分析格局
fn analyze_ge_ju<T: crate::pallet::Config>(
    sizhu: &SiZhu<T>,
    wuxing_strength: &WuXingStrength,
) -> GeJuType {
    let rizhu_wuxing = sizhu.rizhu.to_wuxing();
    let rizhu_strength = get_wuxing_strength(wuxing_strength, rizhu_wuxing);

    let total_strength: u32 =
        wuxing_strength.jin +
        wuxing_strength.mu +
        wuxing_strength.shui +
        wuxing_strength.huo +
        wuxing_strength.tu;

    if total_strength == 0 {
        return GeJuType::ZhengGe;
    }

    let strength_ratio = (rizhu_strength * 100) / total_strength;

    match strength_ratio {
        0..=15 => {
            if has_sheng_fu(sizhu, rizhu_wuxing) {
                GeJuType::ZhengGe
            } else {
                GeJuType::CongRuoGe
            }
        },
        16..=50 => GeJuType::ZhengGe,
        51..=70 => {
            if has_ke_zhi(sizhu, rizhu_wuxing) {
                GeJuType::ZhengGe
            } else {
                GeJuType::CongQiangGe
            }
        },
        _ => GeJuType::CongQiangGe,
    }
}

/// 分析强弱
fn analyze_qiang_ruo(
    wuxing_strength: &WuXingStrength,
    rizhu: TianGan,
) -> MingJuQiangRuo {
    let rizhu_wuxing = rizhu.to_wuxing();
    let rizhu_strength = get_wuxing_strength(wuxing_strength, rizhu_wuxing);

    let total_strength: u32 =
        wuxing_strength.jin +
        wuxing_strength.mu +
        wuxing_strength.shui +
        wuxing_strength.huo +
        wuxing_strength.tu;

    if total_strength == 0 {
        return MingJuQiangRuo::ZhongHe;
    }

    let strength_ratio = (rizhu_strength * 100) / total_strength;

    match strength_ratio {
        0..=15 => MingJuQiangRuo::TaiRuo,
        16..=23 => MingJuQiangRuo::ShenRuo,
        24..=36 => MingJuQiangRuo::ZhongHe,
        37..=50 => MingJuQiangRuo::ShenWang,
        _ => MingJuQiangRuo::TaiWang,
    }
}

/// 分析用神
fn analyze_yong_shen<T: crate::pallet::Config>(
    ge_ju: GeJuType,
    qiang_ruo: MingJuQiangRuo,
    sizhu: &SiZhu<T>,
    _wuxing_strength: &WuXingStrength,
) -> (WuXing, YongShenType) {
    let rizhu_wuxing = sizhu.rizhu.to_wuxing();

    match (ge_ju, qiang_ruo) {
        (GeJuType::ZhengGe, MingJuQiangRuo::ShenWang | MingJuQiangRuo::TaiWang) => {
            (get_ke_wo(rizhu_wuxing), YongShenType::FuYi)
        },
        (GeJuType::ZhengGe, MingJuQiangRuo::ShenRuo | MingJuQiangRuo::TaiRuo) => {
            (get_sheng_wo(rizhu_wuxing), YongShenType::FuYi)
        },
        (GeJuType::ZhengGe, MingJuQiangRuo::ZhongHe) => {
            let season_wuxing = get_season_wuxing(sizhu.month_zhu.ganzhi.zhi);
            (season_wuxing, YongShenType::DiaoHou)
        },
        (GeJuType::CongQiangGe, _) => {
            (rizhu_wuxing, YongShenType::ZhuanWang)
        },
        (GeJuType::CongRuoGe, _) => {
            (get_ke_wo(rizhu_wuxing), YongShenType::ZhuanWang)
        },
        _ => {
            if matches!(qiang_ruo, MingJuQiangRuo::ShenWang | MingJuQiangRuo::TaiWang) {
                (get_ke_wo(rizhu_wuxing), YongShenType::FuYi)
            } else {
                (get_sheng_wo(rizhu_wuxing), YongShenType::FuYi)
            }
        }
    }
}

/// 推导喜神
fn derive_xi_shen(yong_shen: WuXing) -> WuXing {
    get_sheng_wo(yong_shen)
}

/// 推导主忌神
fn derive_ji_shen(yong_shen: WuXing, qiang_ruo: MingJuQiangRuo, rizhu: TianGan) -> WuXing {
    match qiang_ruo {
        MingJuQiangRuo::ShenWang | MingJuQiangRuo::TaiWang => {
            get_sheng_wo(rizhu.to_wuxing())
        },
        MingJuQiangRuo::ShenRuo | MingJuQiangRuo::TaiRuo => {
            get_ke_wo(rizhu.to_wuxing())
        },
        MingJuQiangRuo::ZhongHe => {
            get_ke_wo(yong_shen)
        },
    }
}

/// 分析扩展忌神
fn analyze_extended_ji_shen(
    yong_shen: WuXing,
    qiang_ruo: MingJuQiangRuo,
    rizhu: TianGan,
) -> ExtendedJiShen {
    let rizhu_wuxing = rizhu.to_wuxing();
    let mut secondary = BoundedVec::new();

    match qiang_ruo {
        MingJuQiangRuo::ShenWang | MingJuQiangRuo::TaiWang => {
            // 身旺忌生扶：印星、比劫
            let _ = secondary.try_push(rizhu_wuxing); // 比劫
        },
        MingJuQiangRuo::ShenRuo | MingJuQiangRuo::TaiRuo => {
            // 身弱忌克泄耗：官杀、食伤、财星
            let _ = secondary.try_push(get_wo_sheng(rizhu_wuxing)); // 食伤
        },
        MingJuQiangRuo::ZhongHe => {
            // 中和忌克用神
            let _ = secondary.try_push(get_sheng_wo(get_ke_wo(yong_shen)));
        },
    }

    ExtendedJiShen { secondary }
}

/// 分析性格特征
fn analyze_xing_ge<T: crate::pallet::Config>(sizhu: &SiZhu<T>) -> CompactXingGe {
    let rizhu = sizhu.rizhu;

    // 基于日主天干的性格特征
    let (traits, weaknesses): ([XingGeTrait; 3], [XingGeTrait; 2]) = match rizhu.0 {
        0 => ([XingGeTrait::ZhengZhi, XingGeTrait::YouZhuJian, XingGeTrait::JiJiXiangShang],
              [XingGeTrait::GuZhi, XingGeTrait::QueFaBianTong]),
        1 => ([XingGeTrait::WenHe, XingGeTrait::ShiYingXingQiang, XingGeTrait::YouYiShuTianFu],
              [XingGeTrait::YouRouGuaDuan, XingGeTrait::YiLaiXingQiang]),
        2 => ([XingGeTrait::ReQing, XingGeTrait::KaiLang, XingGeTrait::YouLingDaoLi],
              [XingGeTrait::JiZao, XingGeTrait::QueFaNaiXin]),
        3 => ([XingGeTrait::XiXin, XingGeTrait::YouChuangZaoLi, XingGeTrait::ShanYuGouTong],
              [XingGeTrait::QingXuHua, XingGeTrait::MinGan]),
        4 => ([XingGeTrait::WenZhong, XingGeTrait::KeLao, XingGeTrait::YouZeRenXin],
              [XingGeTrait::BaoShou, XingGeTrait::BianHuaMan]),
        5 => ([XingGeTrait::BaoRong, XingGeTrait::XiZhi, XingGeTrait::ShanYuXieTiao],
              [XingGeTrait::YouYuBuJue, XingGeTrait::QueFaPoLi]),
        6 => ([XingGeTrait::GuoDuan, XingGeTrait::YouZhengYiGan, XingGeTrait::ZhiXingLiQiang],
              [XingGeTrait::GangYing, XingGeTrait::BuGouYuanHua]),
        7 => ([XingGeTrait::JingZhi, XingGeTrait::YouPinWei, XingGeTrait::ShanYuBiaoDa],
              [XingGeTrait::TiaoTi, XingGeTrait::QingXuBoDongDa]),
        8 => ([XingGeTrait::ZhiHui, XingGeTrait::LingHuo, XingGeTrait::ShiYingLiQiang],
              [XingGeTrait::DuoBian, XingGeTrait::QueFaHengXin]),
        _ => ([XingGeTrait::NeiLian, XingGeTrait::ShanYuSiKao, XingGeTrait::ZhiHui],
              [XingGeTrait::DuoBian, XingGeTrait::QueFaHengXin]),
    };

    // 构建性格分析
    let mut zhu_yao_te_dian = BoundedVec::new();
    let mut you_dian = BoundedVec::new();
    let mut que_dian = BoundedVec::new();

    for t in traits.iter() {
        let _ = zhu_yao_te_dian.try_push(*t);
        let _ = you_dian.try_push(*t);
    }

    for w in weaknesses.iter() {
        let _ = que_dian.try_push(*w);
    }

    // 适合职业
    let shi_he_zhi_ye = get_suitable_careers(rizhu);

    CompactXingGe {
        zhu_yao_te_dian,
        you_dian,
        que_dian,
        shi_he_zhi_ye,
    }
}

/// 获取适合职业
fn get_suitable_careers(rizhu: TianGan) -> BoundedVec<ZhiYeType, ConstU32<4>> {
    let mut careers = BoundedVec::new();

    let career_list: [ZhiYeType; 4] = match rizhu.0 {
        0 | 1 => [ZhiYeType::JiaoYu, ZhiYeType::WenHua, ZhiYeType::HuanBao, ZhiYeType::NongLin],
        2 | 3 => [ZhiYeType::NengYuan, ZhiYeType::YuLe, ZhiYeType::CanYin, ZhiYeType::HuaGong],
        4 | 5 => [ZhiYeType::FangDiChan, ZhiYeType::JianZhu, ZhiYeType::NongYe, ZhiYeType::FuWu],
        6 | 7 => [ZhiYeType::JinRong, ZhiYeType::JiXie, ZhiYeType::JunJing, ZhiYeType::WuJin],
        _ => [ZhiYeType::MaoYi, ZhiYeType::YunShu, ZhiYeType::ShuiLi, ZhiYeType::XinXi],
    };

    for career in career_list.iter() {
        let _ = careers.try_push(*career);
    }

    careers
}

/// 计算综合评分
fn calculate_comprehensive_score(
    ge_ju: GeJuType,
    qiang_ruo: MingJuQiangRuo,
    wuxing_strength: &WuXingStrength,
) -> u8 {
    let mut score = 50u8;

    // 格局分 (0-20)
    score += match ge_ju {
        GeJuType::ZhengGe => 20,
        GeJuType::CongQiangGe | GeJuType::CongRuoGe => 15,
        _ => 10,
    };

    // 强弱分 (0-20)
    score += match qiang_ruo {
        MingJuQiangRuo::ZhongHe => 20,
        MingJuQiangRuo::ShenWang | MingJuQiangRuo::ShenRuo => 15,
        MingJuQiangRuo::TaiWang | MingJuQiangRuo::TaiRuo => 10,
    };

    // 平衡分 (0-10)
    let balance_score = calculate_balance_score(wuxing_strength);
    score = score.saturating_add(balance_score);

    score.min(100)
}

/// 计算五行平衡分
fn calculate_balance_score(wuxing_strength: &WuXingStrength) -> u8 {
    let strengths = [
        wuxing_strength.jin,
        wuxing_strength.mu,
        wuxing_strength.shui,
        wuxing_strength.huo,
        wuxing_strength.tu,
    ];

    let total: u32 = strengths.iter().sum();
    if total == 0 {
        return 0;
    }

    let avg = total / 5;
    let variance: u32 = strengths.iter()
        .map(|&s| {
            let diff = if s > avg { s - avg } else { avg - s };
            diff * diff
        })
        .sum();

    let variance_ratio = (variance * 100) / (avg * avg).max(1);
    match variance_ratio {
        0..=20 => 10,
        21..=50 => 8,
        51..=100 => 5,
        101..=200 => 3,
        _ => 0,
    }
}

/// 计算可信度
fn calculate_confidence_score<T: crate::pallet::Config>(
    chart: &BaziChart<T>,
    ge_ju: GeJuType,
    wuxing_strength: &WuXingStrength,
) -> u8 {
    let mut confidence = 100u8;

    // 时辰精确度（仅当 birth_time 存在时检查）
    if let Some(birth_time) = &chart.birth_time {
        if birth_time.minute == 0 {
            confidence = confidence.saturating_sub(15);
        }
    }

    // 格局稀有度
    if matches!(ge_ju, GeJuType::TeShuge | GeJuType::HuaQiGe) {
        confidence = confidence.saturating_sub(15);
    }

    // 五行失衡
    let max_strength = *[
        wuxing_strength.jin,
        wuxing_strength.mu,
        wuxing_strength.shui,
        wuxing_strength.huo,
        wuxing_strength.tu,
    ].iter().max().unwrap_or(&0);

    let total_strength: u32 =
        wuxing_strength.jin +
        wuxing_strength.mu +
        wuxing_strength.shui +
        wuxing_strength.huo +
        wuxing_strength.tu;

    if total_strength > 0 {
        let max_ratio = (max_strength * 100) / total_strength;
        if max_ratio > 70 {
            confidence = confidence.saturating_sub(20);
        } else if max_ratio > 60 {
            confidence = confidence.saturating_sub(10);
        }
    }

    // 子时模式（仅当存在时检查）
    if matches!(chart.zishi_mode, Some(ZiShiMode::Traditional)) {
        confidence = confidence.saturating_sub(5);
    }

    confidence
}

// ================================
// 五行辅助函数
// ================================

fn get_wuxing_strength(strength: &WuXingStrength, wuxing: WuXing) -> u32 {
    match wuxing {
        WuXing::Jin => strength.jin,
        WuXing::Mu => strength.mu,
        WuXing::Shui => strength.shui,
        WuXing::Huo => strength.huo,
        WuXing::Tu => strength.tu,
    }
}

/// 生我者
fn get_sheng_wo(wuxing: WuXing) -> WuXing {
    match wuxing {
        WuXing::Jin => WuXing::Tu,
        WuXing::Mu => WuXing::Shui,
        WuXing::Shui => WuXing::Jin,
        WuXing::Huo => WuXing::Mu,
        WuXing::Tu => WuXing::Huo,
    }
}

/// 我生者
fn get_wo_sheng(wuxing: WuXing) -> WuXing {
    match wuxing {
        WuXing::Jin => WuXing::Shui,
        WuXing::Mu => WuXing::Huo,
        WuXing::Shui => WuXing::Mu,
        WuXing::Huo => WuXing::Tu,
        WuXing::Tu => WuXing::Jin,
    }
}

/// 克我者
fn get_ke_wo(wuxing: WuXing) -> WuXing {
    match wuxing {
        WuXing::Jin => WuXing::Huo,
        WuXing::Mu => WuXing::Jin,
        WuXing::Shui => WuXing::Tu,
        WuXing::Huo => WuXing::Shui,
        WuXing::Tu => WuXing::Mu,
    }
}

/// 季节五行
fn get_season_wuxing(dizhi: DiZhi) -> WuXing {
    match dizhi.0 {
        0 | 1 | 11 => WuXing::Shui,
        2 | 3 | 4 => WuXing::Mu,
        5 | 6 | 7 => WuXing::Huo,
        8 | 9 | 10 => WuXing::Jin,
        _ => WuXing::Tu,
    }
}

/// 检查生扶
fn has_sheng_fu<T: crate::pallet::Config>(sizhu: &SiZhu<T>, rizhu_wuxing: WuXing) -> bool {
    let sheng_wo = get_sheng_wo(rizhu_wuxing);
    [
        sizhu.year_zhu.ganzhi.gan.to_wuxing(),
        sizhu.month_zhu.ganzhi.gan.to_wuxing(),
        sizhu.hour_zhu.ganzhi.gan.to_wuxing(),
    ]
    .iter()
    .any(|&wx| wx == sheng_wo || wx == rizhu_wuxing)
}

/// 检查克制
fn has_ke_zhi<T: crate::pallet::Config>(sizhu: &SiZhu<T>, rizhu_wuxing: WuXing) -> bool {
    let ke_wo = get_ke_wo(rizhu_wuxing);
    [
        sizhu.year_zhu.ganzhi.gan.to_wuxing(),
        sizhu.month_zhu.ganzhi.gan.to_wuxing(),
        sizhu.hour_zhu.ganzhi.gan.to_wuxing(),
    ]
    .iter()
    .any(|&wx| wx == ke_wo)
}

// ================================
// 基于索引的辅助函数（用于加密命盘）
// ================================

/// 基于四柱干支计算简化的五行强度
fn calculate_simple_wuxing_strength(
    year_ganzhi: &GanZhi,
    month_ganzhi: &GanZhi,
    day_ganzhi: &GanZhi,
    hour_ganzhi: &GanZhi,
) -> WuXingStrength {
    let mut strength = WuXingStrength::default();

    // 天干五行（每个 100 分）
    strength.add_element(year_ganzhi.gan.to_wuxing(), 100);
    strength.add_element(month_ganzhi.gan.to_wuxing(), 100);
    strength.add_element(day_ganzhi.gan.to_wuxing(), 100);
    strength.add_element(hour_ganzhi.gan.to_wuxing(), 100);

    // 地支五行（每个 120 分，地支力量稍大）
    strength.add_element(year_ganzhi.zhi.to_wuxing(), 120);
    strength.add_element(month_ganzhi.zhi.to_wuxing(), 120);
    strength.add_element(day_ganzhi.zhi.to_wuxing(), 120);
    strength.add_element(hour_ganzhi.zhi.to_wuxing(), 120);

    // 月令加成（月支五行额外加 80 分）
    strength.add_element(month_ganzhi.zhi.to_wuxing(), 80);

    strength
}

/// 基于索引分析格局
fn analyze_ge_ju_from_index(
    sizhu_index: &SiZhuIndex,
    wuxing_strength: &WuXingStrength,
) -> GeJuType {
    let rizhu_wuxing = sizhu_index.rizhu().to_wuxing();
    let rizhu_strength = get_wuxing_strength(wuxing_strength, rizhu_wuxing);

    let total_strength: u32 =
        wuxing_strength.jin +
        wuxing_strength.mu +
        wuxing_strength.shui +
        wuxing_strength.huo +
        wuxing_strength.tu;

    if total_strength == 0 {
        return GeJuType::ZhengGe;
    }

    let strength_ratio = (rizhu_strength * 100) / total_strength;

    match strength_ratio {
        0..=15 => {
            if has_sheng_fu_from_index(sizhu_index, rizhu_wuxing) {
                GeJuType::ZhengGe
            } else {
                GeJuType::CongRuoGe
            }
        },
        16..=50 => GeJuType::ZhengGe,
        51..=70 => {
            if has_ke_zhi_from_index(sizhu_index, rizhu_wuxing) {
                GeJuType::ZhengGe
            } else {
                GeJuType::CongQiangGe
            }
        },
        _ => GeJuType::CongQiangGe,
    }
}

/// 基于索引分析用神
fn analyze_yong_shen_from_index(
    ge_ju: GeJuType,
    qiang_ruo: MingJuQiangRuo,
    sizhu_index: &SiZhuIndex,
    _wuxing_strength: &WuXingStrength,
) -> (WuXing, YongShenType) {
    let rizhu_wuxing = sizhu_index.rizhu().to_wuxing();

    match (ge_ju, qiang_ruo) {
        (GeJuType::ZhengGe, MingJuQiangRuo::ShenWang | MingJuQiangRuo::TaiWang) => {
            (get_ke_wo(rizhu_wuxing), YongShenType::FuYi)
        },
        (GeJuType::ZhengGe, MingJuQiangRuo::ShenRuo | MingJuQiangRuo::TaiRuo) => {
            (get_sheng_wo(rizhu_wuxing), YongShenType::FuYi)
        },
        (GeJuType::ZhengGe, MingJuQiangRuo::ZhongHe) => {
            let season_wuxing = get_season_wuxing(DiZhi(sizhu_index.month_zhi));
            (season_wuxing, YongShenType::DiaoHou)
        },
        (GeJuType::CongQiangGe, _) => {
            (rizhu_wuxing, YongShenType::ZhuanWang)
        },
        (GeJuType::CongRuoGe, _) => {
            (get_ke_wo(rizhu_wuxing), YongShenType::ZhuanWang)
        },
        _ => {
            if matches!(qiang_ruo, MingJuQiangRuo::ShenWang | MingJuQiangRuo::TaiWang) {
                (get_ke_wo(rizhu_wuxing), YongShenType::FuYi)
            } else {
                (get_sheng_wo(rizhu_wuxing), YongShenType::FuYi)
            }
        }
    }
}

/// 基于索引检查生扶
fn has_sheng_fu_from_index(sizhu_index: &SiZhuIndex, rizhu_wuxing: WuXing) -> bool {
    let sheng_wo = get_sheng_wo(rizhu_wuxing);
    [
        TianGan(sizhu_index.year_gan).to_wuxing(),
        TianGan(sizhu_index.month_gan).to_wuxing(),
        TianGan(sizhu_index.hour_gan).to_wuxing(),
    ]
    .iter()
    .any(|&wx| wx == sheng_wo || wx == rizhu_wuxing)
}

/// 基于索引检查克制
fn has_ke_zhi_from_index(sizhu_index: &SiZhuIndex, rizhu_wuxing: WuXing) -> bool {
    let ke_wo = get_ke_wo(rizhu_wuxing);
    [
        TianGan(sizhu_index.year_gan).to_wuxing(),
        TianGan(sizhu_index.month_gan).to_wuxing(),
        TianGan(sizhu_index.hour_gan).to_wuxing(),
    ]
    .iter()
    .any(|&wx| wx == ke_wo)
}

/// 基于索引计算可信度
fn calculate_index_confidence(
    ge_ju: GeJuType,
    wuxing_strength: &WuXingStrength,
) -> u8 {
    let mut confidence = 85u8; // 基于索引的基础可信度较低

    // 格局稀有度
    if matches!(ge_ju, GeJuType::TeShuge | GeJuType::HuaQiGe) {
        confidence = confidence.saturating_sub(15);
    }

    // 五行失衡
    let max_strength = *[
        wuxing_strength.jin,
        wuxing_strength.mu,
        wuxing_strength.shui,
        wuxing_strength.huo,
        wuxing_strength.tu,
    ].iter().max().unwrap_or(&0);

    let total_strength: u32 =
        wuxing_strength.jin +
        wuxing_strength.mu +
        wuxing_strength.shui +
        wuxing_strength.huo +
        wuxing_strength.tu;

    if total_strength > 0 {
        let max_ratio = (max_strength * 100) / total_strength;
        if max_ratio > 70 {
            confidence = confidence.saturating_sub(15);
        } else if max_ratio > 60 {
            confidence = confidence.saturating_sub(8);
        }
    }

    confidence
}

/// 基于索引分析性格
fn analyze_xing_ge_from_index(sizhu_index: &SiZhuIndex) -> CompactXingGe {
    let rizhu = sizhu_index.rizhu();

    // 基于日主天干的性格特征
    let (traits, weaknesses): ([XingGeTrait; 3], [XingGeTrait; 2]) = match rizhu.0 {
        0 => ([XingGeTrait::ZhengZhi, XingGeTrait::YouZhuJian, XingGeTrait::JiJiXiangShang],
              [XingGeTrait::GuZhi, XingGeTrait::QueFaBianTong]),
        1 => ([XingGeTrait::WenHe, XingGeTrait::ShiYingXingQiang, XingGeTrait::YouYiShuTianFu],
              [XingGeTrait::YouRouGuaDuan, XingGeTrait::YiLaiXingQiang]),
        2 => ([XingGeTrait::ReQing, XingGeTrait::KaiLang, XingGeTrait::YouLingDaoLi],
              [XingGeTrait::JiZao, XingGeTrait::QueFaNaiXin]),
        3 => ([XingGeTrait::XiXin, XingGeTrait::YouChuangZaoLi, XingGeTrait::ShanYuGouTong],
              [XingGeTrait::QingXuHua, XingGeTrait::MinGan]),
        4 => ([XingGeTrait::WenZhong, XingGeTrait::KeLao, XingGeTrait::YouZeRenXin],
              [XingGeTrait::BaoShou, XingGeTrait::BianHuaMan]),
        5 => ([XingGeTrait::BaoRong, XingGeTrait::XiZhi, XingGeTrait::ShanYuXieTiao],
              [XingGeTrait::YouYuBuJue, XingGeTrait::QueFaPoLi]),
        6 => ([XingGeTrait::GuoDuan, XingGeTrait::YouZhengYiGan, XingGeTrait::ZhiXingLiQiang],
              [XingGeTrait::GangYing, XingGeTrait::BuGouYuanHua]),
        7 => ([XingGeTrait::JingZhi, XingGeTrait::YouPinWei, XingGeTrait::ShanYuBiaoDa],
              [XingGeTrait::TiaoTi, XingGeTrait::QingXuBoDongDa]),
        8 => ([XingGeTrait::ZhiHui, XingGeTrait::LingHuo, XingGeTrait::ShiYingLiQiang],
              [XingGeTrait::DuoBian, XingGeTrait::QueFaHengXin]),
        _ => ([XingGeTrait::NeiLian, XingGeTrait::ShanYuSiKao, XingGeTrait::ZhiHui],
              [XingGeTrait::DuoBian, XingGeTrait::QueFaHengXin]),
    };

    // 构建性格分析
    let mut zhu_yao_te_dian = BoundedVec::new();
    let mut you_dian = BoundedVec::new();
    let mut que_dian = BoundedVec::new();

    for t in traits.iter() {
        let _ = zhu_yao_te_dian.try_push(*t);
        let _ = you_dian.try_push(*t);
    }

    for w in weaknesses.iter() {
        let _ = que_dian.try_push(*w);
    }

    // 适合职业
    let shi_he_zhi_ye = get_suitable_careers(rizhu);

    CompactXingGe {
        zhu_yao_te_dian,
        you_dian,
        que_dian,
        shi_he_zhi_ye,
    }
}

// ================================
// Runtime API 数据结构（V5 新增）
// ================================

/// 完整八字命盘（用于 Runtime API 返回）
///
/// 包含所有计算字段的完整命盘数据，用于 JSON 序列化
#[derive(Clone, Debug, Default)]
pub struct FullBaziChartForApi {
    /// 性别
    pub gender: Gender,
    /// 出生年份
    pub birth_year: u16,
    /// 输入日历类型（记录原始输入是公历还是农历）
    /// 用于前端显示，不影响八字计算
    pub input_calendar_type: crate::types::InputCalendarType,
    /// 四柱干支
    pub sizhu: SiZhuForApi,
    /// 空亡信息
    pub kongwang: KongWangInfo,
    /// 星运信息（十二长生）
    pub xingyun: XingYunInfo,
    /// 神煽列表
    pub shensha_list: sp_std::vec::Vec<ShenShaEntry>,
    /// 五行强度
    pub wuxing_strength: WuXingStrength,
    /// 大运列表（12步）
    pub dayun_list: sp_std::vec::Vec<DaYunForApi>,
    /// 起运信息
    pub qiyun: QiYunForApi,
    /// 命盘分析（用神、喜神、忌神等）
    pub analysis: AnalysisForApi,
}

/// 大运信息（用于 API）
#[derive(Clone, Debug, Default)]
pub struct DaYunForApi {
    /// 大运干支
    pub ganzhi: GanZhi,
    /// 天干十神
    pub tiangan_shishen: ShiShen,
    /// 地支本气十神
    pub dizhi_benqi_shishen: ShiShen,
    /// 起始年龄
    pub start_age: u8,
    /// 起始年份
    pub start_year: u16,
    /// 交运月份
    pub start_month: u8,
    /// 交运日期
    pub start_day: u8,
    /// 结束年龄
    pub end_age: u8,
    /// 结束年份
    pub end_year: u16,
    /// 十二长生
    pub changsheng: ShiErChangSheng,
    /// 流年列表（10年）
    pub liunian_list: sp_std::vec::Vec<LiuNianForApi>,
}

/// 流年信息（用于 API）
#[derive(Clone, Debug, Default)]
pub struct LiuNianForApi {
    /// 流年干支
    pub ganzhi: GanZhi,
    /// 天干十神
    pub tiangan_shishen: ShiShen,
    /// 地支本气十神
    pub dizhi_benqi_shishen: ShiShen,
    /// 年份
    pub year: u16,
    /// 年龄
    pub age: u8,
}

/// 起运信息（用于 API）
#[derive(Clone, Debug, Default)]
pub struct QiYunForApi {
    /// 起运年龄（岁）
    pub age_years: u8,
    /// 起运月数
    pub age_months: u8,
    /// 起运天数
    pub age_days: u8,
    /// 是否顺排
    pub is_shun: bool,
    /// 交运年份
    pub jiaoyun_year: u16,
    /// 交运月份
    pub jiaoyun_month: u8,
    /// 交运日期
    pub jiaoyun_day: u8,
}

/// 命盘分析（用于 API）
#[derive(Clone, Debug, Default)]
pub struct AnalysisForApi {
    /// 格局类型
    pub ge_ju: GeJuType,
    /// 命局强弱
    pub qiang_ruo: MingJuQiangRuo,
    /// 用神五行
    pub yong_shen: WuXing,
    /// 用神类型
    pub yong_shen_type: YongShenType,
    /// 喜神五行
    pub xi_shen: WuXing,
    /// 忌神五行
    pub ji_shen: WuXing,
    /// 综合评分 (0-100)
    pub score: u8,
}

/// 四柱信息（用于 API）
#[derive(Clone, Debug, Default)]
pub struct SiZhuForApi {
    /// 年柱
    pub year_zhu: ZhuForApi,
    /// 月柱
    pub month_zhu: ZhuForApi,
    /// 日柱
    pub day_zhu: ZhuForApi,
    /// 时柱
    pub hour_zhu: ZhuForApi,
    /// 日主
    pub rizhu: TianGan,
}

/// 单柱信息（用于 API）
#[derive(Clone, Debug, Default)]
pub struct ZhuForApi {
    /// 干支组合
    pub ganzhi: GanZhi,
    /// 天干十神
    pub tiangan_shishen: ShiShen,
    /// 地支本气十神
    pub dizhi_benqi_shishen: ShiShen,
    /// 藏干列表
    pub canggan_list: sp_std::vec::Vec<CangGanForApi>,
    /// 纳音
    pub nayin: NaYin,
    /// 十二长生
    pub changsheng: ShiErChangSheng,
    /// 自坐（本柱天干与地支本气的十神关系）
    pub zizuo: ShiShen,
}

/// 藏干信息（用于 API）
#[derive(Clone, Debug, Default)]
pub struct CangGanForApi {
    /// 藏干天干
    pub gan: TianGan,
    /// 与日主的十神关系
    pub shishen: ShiShen,
    /// 藏干类型
    pub canggan_type: CangGanType,
    /// 权重
    pub weight: u16,
}

impl FullBaziChartForApi {
    /// 转换为调试友好的 JSON 字符串
    pub fn to_debug_json(&self) -> scale_info::prelude::string::String {
        use scale_info::prelude::format;

        // 手动构建 JSON 字符串
        let mut json = scale_info::prelude::string::String::from("{");

        // 基本信息
        json.push_str(&format!("\"gender\":\"{:?}\",", self.gender));
        json.push_str(&format!("\"birthYear\":{},", self.birth_year));

        // 四柱
        json.push_str("\"sizhu\":{");
        json.push_str(&format!("\"yearZhu\":{},", self.sizhu.year_zhu.to_json()));
        json.push_str(&format!("\"monthZhu\":{},", self.sizhu.month_zhu.to_json()));
        json.push_str(&format!("\"dayZhu\":{},", self.sizhu.day_zhu.to_json()));
        json.push_str(&format!("\"hourZhu\":{},", self.sizhu.hour_zhu.to_json()));
        json.push_str(&format!("\"rizhu\":\"{}\"", self.sizhu.rizhu.name()));
        json.push_str("},");

        // 空亡
        json.push_str(&format!(
            "\"kongwang\":{{\"yearKong\":{},\"monthKong\":{},\"dayKong\":{},\"hourKong\":{}}},",
            self.kongwang.year_is_kong,
            self.kongwang.month_is_kong,
            self.kongwang.day_is_kong,
            self.kongwang.hour_is_kong
        ));

        // 星运
        json.push_str(&format!(
            "\"xingyun\":{{\"year\":\"{}\",\"month\":\"{}\",\"day\":\"{}\",\"hour\":\"{}\"}},",
            self.xingyun.year_changsheng.name(),
            self.xingyun.month_changsheng.name(),
            self.xingyun.day_changsheng.name(),
            self.xingyun.hour_changsheng.name()
        ));

        // 神煞列表
        json.push_str("\"shenshaList\":[");
        for (i, shensha) in self.shensha_list.iter().enumerate() {
            if i > 0 { json.push_str(","); }
            let nature_str = match shensha.nature {
                crate::types::ShenShaNature::JiShen => "Ji",
                crate::types::ShenShaNature::XiongShen => "Xiong",
                crate::types::ShenShaNature::Neutral => "Neutral",
            };
            json.push_str(&format!(
                "{{\"shensha\":\"{}\",\"position\":\"{}\",\"nature\":\"{}\"}}",
                shensha.shensha.name(),
                shensha.position.name(),
                nature_str
            ));
        }
        json.push_str("],");

        // 五行强度
        json.push_str(&format!(
            "\"wuxingStrength\":{{\"jin\":{},\"mu\":{},\"shui\":{},\"huo\":{},\"tu\":{}}},",
            self.wuxing_strength.jin,
            self.wuxing_strength.mu,
            self.wuxing_strength.shui,
            self.wuxing_strength.huo,
            self.wuxing_strength.tu
        ));

        // 起运信息
        json.push_str(&format!(
            "\"qiyun\":{{\"ageYears\":{},\"ageMonths\":{},\"ageDays\":{},\"isShun\":{},\"jiaoyunYear\":{},\"jiaoyunMonth\":{},\"jiaoyunDay\":{}}},",
            self.qiyun.age_years,
            self.qiyun.age_months,
            self.qiyun.age_days,
            self.qiyun.is_shun,
            self.qiyun.jiaoyun_year,
            self.qiyun.jiaoyun_month,
            self.qiyun.jiaoyun_day
        ));

        // 命盘分析
        json.push_str(&format!(
            "\"analysis\":{{\"geJu\":\"{:?}\",\"qiangRuo\":\"{:?}\",\"yongShen\":\"{:?}\",\"yongShenType\":\"{:?}\",\"xiShen\":\"{:?}\",\"jiShen\":\"{:?}\",\"score\":{}}},",
            self.analysis.ge_ju,
            self.analysis.qiang_ruo,
            self.analysis.yong_shen,
            self.analysis.yong_shen_type,
            self.analysis.xi_shen,
            self.analysis.ji_shen,
            self.analysis.score
        ));

        // 大运列表
        json.push_str("\"dayunList\":[");
        for (i, dayun) in self.dayun_list.iter().enumerate() {
            if i > 0 { json.push_str(","); }
            json.push_str(&format!(
                "{{\"ganzhi\":\"{}{}\",\"tianganShishen\":\"{}\",\"dizhiBenqiShishen\":\"{}\",\"startAge\":{},\"startYear\":{},\"startMonth\":{},\"startDay\":{},\"endAge\":{},\"endYear\":{},\"changsheng\":\"{}\",\"liunianList\":[",
                dayun.ganzhi.gan.name(),
                dayun.ganzhi.zhi.name(),
                dayun.tiangan_shishen.name(),
                dayun.dizhi_benqi_shishen.name(),
                dayun.start_age,
                dayun.start_year,
                dayun.start_month,
                dayun.start_day,
                dayun.end_age,
                dayun.end_year,
                dayun.changsheng.name()
            ));
            // 流年列表
            for (j, liunian) in dayun.liunian_list.iter().enumerate() {
                if j > 0 { json.push_str(","); }
                json.push_str(&format!(
                    "{{\"ganzhi\":\"{}{}\",\"tianganShishen\":\"{}\",\"dizhiBenqiShishen\":\"{}\",\"year\":{},\"age\":{}}}",
                    liunian.ganzhi.gan.name(),
                    liunian.ganzhi.zhi.name(),
                    liunian.tiangan_shishen.name(),
                    liunian.dizhi_benqi_shishen.name(),
                    liunian.year,
                    liunian.age
                ));
            }
            json.push_str("]}");
        }
        json.push_str("]");

        json.push_str("}");
        json
    }
}

impl ZhuForApi {
    /// 转换为 JSON 字符串
    fn to_json(&self) -> scale_info::prelude::string::String {
        use scale_info::prelude::format;

        let mut json = scale_info::prelude::string::String::from("{");
        json.push_str(&format!("\"ganzhi\":\"{}{}\"", self.ganzhi.gan.name(), self.ganzhi.zhi.name()));
        json.push_str(&format!(",\"tianganShishen\":\"{}\"", self.tiangan_shishen.name()));
        json.push_str(&format!(",\"dizhiBenqiShishen\":\"{}\"", self.dizhi_benqi_shishen.name()));
        json.push_str(&format!(",\"nayin\":\"{}\"", self.nayin.name()));
        json.push_str(&format!(",\"changsheng\":\"{}\"", self.changsheng.name()));
        json.push_str(&format!(",\"zizuo\":\"{}\"", self.zizuo.name()));

        // 藏干列表
        json.push_str(",\"cangganList\":[");
        for (i, cg) in self.canggan_list.iter().enumerate() {
            if i > 0 { json.push_str(","); }
            json.push_str(&format!(
                "{{\"gan\":\"{}\",\"shishen\":\"{}\",\"type\":\"{:?}\",\"weight\":{}}}",
                cg.gan.name(),
                cg.shishen.name(),
                cg.canggan_type,
                cg.weight
            ));
        }
        json.push_str("]");

        json.push_str("}");
        json
    }
}

/// 构建完整命盘用于 API 返回（从已存储的 BaziChart）
///
/// # 注意
/// 此函数假设命盘处于 Public 或 Partial 模式，sizhu 等字段存在。
/// Private 模式的命盘应使用其他方法处理。
pub fn build_full_bazi_chart_for_api<T: crate::pallet::Config>(
    chart: &BaziChart<T>,
) -> FullBaziChartForApi {
    use crate::calculations::{xingyun, kongwang, shensha};

    // 获取四柱（假设存在，如果不存在则返回默认值）
    let sizhu = match &chart.sizhu {
        Some(s) => s,
        None => {
            // Private 模式下没有四柱数据，返回空结构
            return FullBaziChartForApi::default();
        }
    };

    let rizhu = sizhu.rizhu;

    // 计算空亡
    let kongwang_info = kongwang::calculate_all_kongwang_temp(
        &sizhu.year_zhu.ganzhi,
        &sizhu.month_zhu.ganzhi,
        &sizhu.day_zhu.ganzhi,
        &sizhu.hour_zhu.ganzhi,
    );

    // 计算星运
    let xingyun_info = xingyun::calculate_xingyun_temp(
        rizhu,
        &sizhu.year_zhu.ganzhi.zhi,
        &sizhu.month_zhu.ganzhi.zhi,
        &sizhu.day_zhu.ganzhi.zhi,
        &sizhu.hour_zhu.ganzhi.zhi,
    );

    // 计算神煞
    let shensha_list = shensha::calculate_shensha_list_temp(
        &sizhu.year_zhu.ganzhi,
        &sizhu.month_zhu.ganzhi,
        &sizhu.day_zhu.ganzhi,
        &sizhu.hour_zhu.ganzhi,
    );

    // 构建四柱信息
    let sizhu_for_api = SiZhuForApi {
        year_zhu: build_zhu_for_api(&sizhu.year_zhu.ganzhi, rizhu, xingyun_info.year_changsheng),
        month_zhu: build_zhu_for_api(&sizhu.month_zhu.ganzhi, rizhu, xingyun_info.month_changsheng),
        day_zhu: build_zhu_for_api(&sizhu.day_zhu.ganzhi, rizhu, xingyun_info.day_changsheng),
        hour_zhu: build_zhu_for_api(&sizhu.hour_zhu.ganzhi, rizhu, xingyun_info.hour_changsheng),
        rizhu,
    };

    // 获取基本信息
    let gender = chart.gender.unwrap_or(Gender::Male);
    let birth_year = chart.birth_time.map(|bt| bt.year).unwrap_or(0);
    let wuxing_strength = chart.wuxing_strength.unwrap_or_default();

    // 计算起运和大运（复用 temp 函数的逻辑）
    use crate::calculations::dayun;
    let qiyun_info = dayun::calculate_qiyun_precise(sizhu.year_zhu.ganzhi.gan.0, gender, 15, 0);
    // 计算首次交运日期
    let jiaoyun_year = birth_year + qiyun_info.age_years as u16;
    let total_months = qiyun_info.age_months;
    let jiaoyun_month = if total_months == 0 { 1 } else { ((total_months - 1) % 12) + 1 };
    let jiaoyun_day = if qiyun_info.age_days == 0 { 1 } else { qiyun_info.age_days.min(28) };
    
    let qiyun = QiYunForApi {
        age_years: qiyun_info.age_years,
        age_months: qiyun_info.age_months,
        age_days: qiyun_info.age_days,
        is_shun: qiyun_info.is_shun,
        jiaoyun_year,
        jiaoyun_month,
        jiaoyun_day,
    };

    let dayun_raw = dayun::calculate_dayun_list(
        sizhu.month_zhu.ganzhi,
        birth_year,
        qiyun_info.age_years,
        qiyun_info.is_shun,
        12,
    );

    // 构建大运列表
    use crate::constants::{calculate_shishen, get_hidden_stems, is_valid_canggan};
    let mut dayun_list = sp_std::vec::Vec::new();
    for (ganzhi, start_age, start_year) in dayun_raw.iter() {
        let tiangan_shishen = calculate_shishen(rizhu, ganzhi.gan);
        let hidden_stems = get_hidden_stems(ganzhi.zhi);
        let dizhi_benqi_shishen = if is_valid_canggan(hidden_stems[0].0.0) {
            calculate_shishen(rizhu, hidden_stems[0].0)
        } else {
            ShiShen::BiJian
        };
        let changsheng = xingyun::get_changsheng(rizhu, ganzhi.zhi);

        // 流年列表
        let mut liunian_list = sp_std::vec::Vec::new();
        for i in 0..10u8 {
            let ln_year = start_year + i as u16;
            let ln_age = start_age + i;
            let ln_ganzhi_index = ((ln_year as i32 - 4) % 60) as u8;
            if let Some(ln_ganzhi) = GanZhi::from_index(ln_ganzhi_index) {
                let ln_tiangan_shishen = calculate_shishen(rizhu, ln_ganzhi.gan);
                let ln_hidden = get_hidden_stems(ln_ganzhi.zhi);
                let ln_dizhi_shishen = if is_valid_canggan(ln_hidden[0].0.0) {
                    calculate_shishen(rizhu, ln_hidden[0].0)
                } else {
                    ShiShen::BiJian
                };
                liunian_list.push(LiuNianForApi {
                    ganzhi: ln_ganzhi,
                    tiangan_shishen: ln_tiangan_shishen,
                    dizhi_benqi_shishen: ln_dizhi_shishen,
                    year: ln_year,
                    age: ln_age,
                });
            }
        }

        dayun_list.push(DaYunForApi {
            ganzhi: *ganzhi,
            tiangan_shishen,
            dizhi_benqi_shishen,
            start_age: *start_age,
            start_year: *start_year,
            start_month: jiaoyun_month,
            start_day: jiaoyun_day,
            end_age: start_age + 9,
            end_year: start_year + 9,
            changsheng,
            liunian_list,
        });
    }

    // 计算命盘分析
    let sizhu_index = SiZhuIndex {
        year_gan: sizhu.year_zhu.ganzhi.gan.0,
        year_zhi: sizhu.year_zhu.ganzhi.zhi.0,
        month_gan: sizhu.month_zhu.ganzhi.gan.0,
        month_zhi: sizhu.month_zhu.ganzhi.zhi.0,
        day_gan: sizhu.day_zhu.ganzhi.gan.0,
        day_zhi: sizhu.day_zhu.ganzhi.zhi.0,
        hour_gan: sizhu.hour_zhu.ganzhi.gan.0,
        hour_zhi: sizhu.hour_zhu.ganzhi.zhi.0,
    };
    let ge_ju = analyze_ge_ju_from_index(&sizhu_index, &wuxing_strength);
    let qiang_ruo = analyze_qiang_ruo(&wuxing_strength, rizhu);
    let (yong_shen, yong_shen_type) = analyze_yong_shen_from_index(ge_ju, qiang_ruo, &sizhu_index, &wuxing_strength);
    let xi_shen = derive_xi_shen(yong_shen);
    let ji_shen = derive_ji_shen(yong_shen, qiang_ruo, rizhu);
    let score = calculate_comprehensive_score(ge_ju, qiang_ruo, &wuxing_strength);

    let analysis = AnalysisForApi {
        ge_ju,
        qiang_ruo,
        yong_shen,
        yong_shen_type,
        xi_shen,
        ji_shen,
        score,
    };

    FullBaziChartForApi {
        gender,
        birth_year,
        input_calendar_type: chart.input_calendar_type.unwrap_or(crate::types::InputCalendarType::Solar),
        sizhu: sizhu_for_api,
        kongwang: kongwang_info,
        xingyun: xingyun_info,
        shensha_list,
        wuxing_strength,
        dayun_list,
        qiyun,
        analysis,
    }
}

/// 构建完整命盘用于 API 返回（从临时计算的四柱）
///
/// # 参数
/// - `year_ganzhi`: 年柱干支
/// - `month_ganzhi`: 月柱干支
/// - `day_ganzhi`: 日柱干支
/// - `hour_ganzhi`: 时柱干支
/// - `gender`: 性别
/// - `birth_year`: 出生年份
/// - `input_calendar_type`: 输入日历类型（公历/农历/四柱）
pub fn build_full_bazi_chart_for_api_temp(
    year_ganzhi: GanZhi,
    month_ganzhi: GanZhi,
    day_ganzhi: GanZhi,
    hour_ganzhi: GanZhi,
    gender: Gender,
    birth_year: u16,
    input_calendar_type: crate::types::InputCalendarType,
) -> FullBaziChartForApi {
    use crate::calculations::{xingyun, kongwang, shensha, wuxing, dayun};
    use crate::constants::{calculate_shishen, get_hidden_stems, is_valid_canggan};

    let rizhu = day_ganzhi.gan;

    // 计算空亡
    let kongwang_info = kongwang::calculate_all_kongwang_temp(
        &year_ganzhi,
        &month_ganzhi,
        &day_ganzhi,
        &hour_ganzhi,
    );

    // 计算星运
    let xingyun_info = xingyun::calculate_xingyun_temp(
        rizhu,
        &year_ganzhi.zhi,
        &month_ganzhi.zhi,
        &day_ganzhi.zhi,
        &hour_ganzhi.zhi,
    );

    // 计算神煞
    let shensha_list = shensha::calculate_shensha_list_temp(
        &year_ganzhi,
        &month_ganzhi,
        &day_ganzhi,
        &hour_ganzhi,
    );

    // 计算五行强度
    let wuxing_strength = wuxing::calculate_wuxing_strength(
        &year_ganzhi,
        &month_ganzhi,
        &day_ganzhi,
        &hour_ganzhi,
    );

    // 构建四柱信息
    let sizhu_for_api = SiZhuForApi {
        year_zhu: build_zhu_for_api(&year_ganzhi, rizhu, xingyun_info.year_changsheng),
        month_zhu: build_zhu_for_api(&month_ganzhi, rizhu, xingyun_info.month_changsheng),
        day_zhu: build_zhu_for_api(&day_ganzhi, rizhu, xingyun_info.day_changsheng),
        hour_zhu: build_zhu_for_api(&hour_ganzhi, rizhu, xingyun_info.hour_changsheng),
        rizhu,
    };

    // 计算起运信息（默认距离节气15天）
    let qiyun_info = dayun::calculate_qiyun_precise(year_ganzhi.gan.0, gender, 15, 0);
    // 计算首次交运日期：出生年份 + 起运年龄
    let jiaoyun_year = birth_year + qiyun_info.age_years as u16;
    // 交运月日根据起运月数和天数估算（假设出生在1月1日）
    let total_months = qiyun_info.age_months;
    let jiaoyun_month = if total_months == 0 { 1 } else { ((total_months - 1) % 12) + 1 };
    let jiaoyun_day = if qiyun_info.age_days == 0 { 1 } else { qiyun_info.age_days.min(28) };
    
    let qiyun = QiYunForApi {
        age_years: qiyun_info.age_years,
        age_months: qiyun_info.age_months,
        age_days: qiyun_info.age_days,
        is_shun: qiyun_info.is_shun,
        jiaoyun_year,
        jiaoyun_month,
        jiaoyun_day,
    };

    // 计算大运列表（12步）
    let dayun_raw = dayun::calculate_dayun_list(
        month_ganzhi,
        birth_year,
        qiyun_info.age_years,
        qiyun_info.is_shun,
        12,
    );

    // 构建大运列表（含流年）
    let mut dayun_list = sp_std::vec::Vec::new();
    for (ganzhi, start_age, start_year) in dayun_raw.iter() {
        // 计算大运十神
        let tiangan_shishen = calculate_shishen(rizhu, ganzhi.gan);
        let hidden_stems = get_hidden_stems(ganzhi.zhi);
        let dizhi_benqi_shishen = if is_valid_canggan(hidden_stems[0].0.0) {
            calculate_shishen(rizhu, hidden_stems[0].0)
        } else {
            ShiShen::BiJian
        };

        // 计算大运十二长生
        let changsheng = xingyun::get_changsheng(rizhu, ganzhi.zhi);

        // 构建流年列表（10年）
        let mut liunian_list = sp_std::vec::Vec::new();
        for i in 0..10u8 {
            let ln_year = start_year + i as u16;
            let ln_age = start_age + i;
            // 流年干支 = (年份 - 4) % 60
            let ln_ganzhi_index = ((ln_year as i32 - 4) % 60) as u8;
            if let Some(ln_ganzhi) = GanZhi::from_index(ln_ganzhi_index) {
                let ln_tiangan_shishen = calculate_shishen(rizhu, ln_ganzhi.gan);
                let ln_hidden = get_hidden_stems(ln_ganzhi.zhi);
                let ln_dizhi_shishen = if is_valid_canggan(ln_hidden[0].0.0) {
                    calculate_shishen(rizhu, ln_hidden[0].0)
                } else {
                    ShiShen::BiJian
                };
                liunian_list.push(LiuNianForApi {
                    ganzhi: ln_ganzhi,
                    tiangan_shishen: ln_tiangan_shishen,
                    dizhi_benqi_shishen: ln_dizhi_shishen,
                    year: ln_year,
                    age: ln_age,
                });
            }
        }

        dayun_list.push(DaYunForApi {
            ganzhi: *ganzhi,
            tiangan_shishen,
            dizhi_benqi_shishen,
            start_age: *start_age,
            start_year: *start_year,
            start_month: jiaoyun_month,
            start_day: jiaoyun_day,
            end_age: start_age + 9,
            end_year: start_year + 9,
            changsheng,
            liunian_list,
        });
    }

    // 计算命盘分析（用神、喜神、忌神）
    let sizhu_index = SiZhuIndex {
        year_gan: year_ganzhi.gan.0,
        year_zhi: year_ganzhi.zhi.0,
        month_gan: month_ganzhi.gan.0,
        month_zhi: month_ganzhi.zhi.0,
        day_gan: day_ganzhi.gan.0,
        day_zhi: day_ganzhi.zhi.0,
        hour_gan: hour_ganzhi.gan.0,
        hour_zhi: hour_ganzhi.zhi.0,
    };
    let ge_ju = analyze_ge_ju_from_index(&sizhu_index, &wuxing_strength);
    let qiang_ruo = analyze_qiang_ruo(&wuxing_strength, rizhu);
    let (yong_shen, yong_shen_type) = analyze_yong_shen_from_index(ge_ju, qiang_ruo, &sizhu_index, &wuxing_strength);
    let xi_shen = derive_xi_shen(yong_shen);
    let ji_shen = derive_ji_shen(yong_shen, qiang_ruo, rizhu);
    let score = calculate_comprehensive_score(ge_ju, qiang_ruo, &wuxing_strength);

    let analysis = AnalysisForApi {
        ge_ju,
        qiang_ruo,
        yong_shen,
        yong_shen_type,
        xi_shen,
        ji_shen,
        score,
    };

    FullBaziChartForApi {
        gender,
        birth_year,
        input_calendar_type,
        sizhu: sizhu_for_api,
        kongwang: kongwang_info,
        xingyun: xingyun_info,
        shensha_list,
        wuxing_strength,
        dayun_list,
        qiyun,
        analysis,
    }
}

/// 构建单柱信息
fn build_zhu_for_api(
    ganzhi: &GanZhi,
    rizhu: TianGan,
    changsheng: ShiErChangSheng,
) -> ZhuForApi {
    use crate::constants::{calculate_shishen, calculate_nayin, get_hidden_stems, is_valid_canggan};

    // 计算天干十神
    let tiangan_shishen = calculate_shishen(rizhu, ganzhi.gan);

    // 获取藏干并计算本气十神
    let hidden_stems = get_hidden_stems(ganzhi.zhi);
    let dizhi_benqi_shishen = if is_valid_canggan(hidden_stems[0].0.0) {
        calculate_shishen(rizhu, hidden_stems[0].0)
    } else {
        ShiShen::BiJian // 默认值
    };

    // 构建藏干列表
    let mut canggan_list = sp_std::vec::Vec::new();
    for (gan, canggan_type, weight) in hidden_stems.iter() {
        if is_valid_canggan(gan.0) {
            canggan_list.push(CangGanForApi {
                gan: *gan,
                shishen: calculate_shishen(rizhu, *gan),
                canggan_type: *canggan_type,
                weight: *weight,
            });
        }
    }

    // 计算纳音
    let nayin = calculate_nayin(ganzhi);

    // 计算自坐（本柱天干与地支本气的十神关系）
    let zizuo = if is_valid_canggan(hidden_stems[0].0.0) {
        calculate_shishen(ganzhi.gan, hidden_stems[0].0)
    } else {
        ShiShen::BiJian
    };

    ZhuForApi {
        ganzhi: *ganzhi,
        tiangan_shishen,
        dizhi_benqi_shishen,
        canggan_list,
        nayin,
        changsheng,
        zizuo,
    }
}

// ================================
// 单元测试
// ================================

#[cfg(test)]
mod tests {
    use super::*;
    use codec::Encode;

    #[test]
    fn test_core_interpretation_size() {
        let core = CoreInterpretation {
            ge_ju: GeJuType::ZhengGe,
            qiang_ruo: MingJuQiangRuo::ZhongHe,
            yong_shen: WuXing::Huo,
            yong_shen_type: YongShenType::FuYi,
            xi_shen: WuXing::Mu,
            ji_shen: WuXing::Shui,
            score: 75,
            confidence: 85,
            timestamp: 1000000,
            algorithm_version: 3,
        };

        let encoded = core.encode();
        assert!(encoded.len() <= 13, "CoreInterpretation 编码大小: {} bytes", encoded.len());
        println!("✅ CoreInterpretation 编码大小: {} bytes", encoded.len());
    }
}
