//! # 紫微斗数解卦 Runtime API
//!
//! 本模块定义了紫微斗数解卦的 Runtime API，供前端通过 RPC 调用。
//!
//! ## 设计原则
//!
//! 1. **实时计算**：解卦数据在查询时实时计算，不占用链上存储
//! 2. **免费查询**：所有 API 调用都是免费的链下查询
//! 3. **分层设计**：提供核心数据和详细数据两个层级
//!
//! ## API 列表
//!
//! | API | 功能 | 返回类型 |
//! |-----|------|----------|
//! | `get_interpretation` | 获取完整解卦 | `ZiweiInterpretation` |
//! | `get_overall_score` | 获取整体评分 | `ChartOverallScore` |
//! | `get_palace_interpretation` | 获取单宫解读 | `PalaceInterpretation` |
//! | `get_patterns` | 获取格局列表 | `Vec<PatternInfo>` |
//! | `get_si_hua_analysis` | 获取四化分析 | `SiHuaAnalysis` |
//! | `get_da_xian_interpretation` | 获取大限解读 | `DaXianInterpretation` |
//! | `get_liu_nian_fortune` | 获取流年运势 | `LiuNianFortune` |
//!
//! ## 使用示例
//!
//! ```javascript
//! // 前端调用示例
//! const api = await ApiPromise.create({ provider: wsProvider });
//!
//! // 获取完整解卦
//! const interpretation = await api.call.ziweiInterpretationApi.getInterpretation(chartId);
//!
//! // 获取命宫详细解读
//! const mingGong = await api.call.ziweiInterpretationApi.getPalaceInterpretation(chartId, 0);
//!
//! // 获取格局
//! const patterns = await api.call.ziweiInterpretationApi.getPatterns(chartId);
//! ```
//!
//! ## 数据流程
//!
//! ```text
//! 前端请求 → Runtime API → 读取链上命盘数据 → 实时计算解卦 → 返回结果
//!                              ↓
//!                      score.rs (评分算法)
//!                      pattern.rs (格局识别)
//!                      sihua.rs (四化分析)
//! ```

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

use crate::interpretation::*;
use crate::types::*;

// ============================================================================
// 隐私相关数据结构
// ============================================================================

/// 紫微斗数命盘计算结果（用于 Private 模式临时计算）
///
/// 包含完整的命盘计算数据，但不存储到链上
#[derive(Clone, Encode, Decode, TypeInfo, RuntimeDebug)]
pub struct ZiweiChartResult {
    /// 五行局
    pub wu_xing_ju: WuXing,

    /// 局数
    pub ju_shu: u8,

    /// 命宫位置
    pub ming_gong_pos: u8,

    /// 身宫位置
    pub shen_gong_pos: u8,

    /// 紫微星位置
    pub ziwei_pos: u8,

    /// 天府星位置
    pub tianfu_pos: u8,

    /// 十二宫数据
    pub palaces: [Palace; 12],

    /// 四化星
    pub si_hua_stars: [SiHuaStar; 4],

    /// 起运年龄
    pub qi_yun_age: u8,

    /// 大运顺逆（true=顺行）
    pub da_yun_shun: bool,

    /// 年干
    pub year_gan: TianGan,

    /// 年支
    pub year_zhi: DiZhi,
}

/// 紫微斗数公开元数据
///
/// 返回命盘的公开元数据，不包含敏感信息。
/// 适用于所有隐私模式。
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct ZiweiPublicMetadata {
    /// 命盘记录 ID
    pub id: u64,

    /// 隐私模式
    pub privacy_mode: pallet_divination_privacy::types::PrivacyMode,

    /// 创建时间戳（区块号）
    pub created_at: u64,

    /// 是否有加密数据
    pub has_encrypted_data: bool,

    /// 是否可解读（Public/Partial 模式且有计算数据）
    pub can_interpret: bool,

    /// 五行局（如果公开）
    pub wu_xing_ju: Option<WuXing>,

    /// 局数（如果公开）
    pub ju_shu: Option<u8>,

    /// 命宫位置（如果公开）
    pub ming_gong_pos: Option<u8>,

    /// 是否有 AI 解读
    pub has_ai_interpretation: bool,
}

// ============================================================================
// 流年运势数据结构
// ============================================================================

/// 流年运势
///
/// 根据流年天干地支计算的年度运势
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct LiuNianFortune {
    /// 流年年份
    pub year: u16,

    /// 流年天干
    pub tian_gan: TianGan,

    /// 流年地支
    pub di_zhi: DiZhi,

    /// 流年四化星
    pub si_hua_stars: [SiHuaStar; 4],

    /// 流年运势评分（0-100）
    pub fortune_score: u8,

    /// 运势等级
    pub fortune_level: FortuneLevel,

    /// 流年财运（0-100）
    pub wealth_fortune: u8,

    /// 流年事业（0-100）
    pub career_fortune: u8,

    /// 流年感情（0-100）
    pub relationship_fortune: u8,

    /// 流年健康（0-100）
    pub health_fortune: u8,

    /// 流年关键词（3个索引）
    pub keywords: [u8; 3],

    /// 流年宫位（太岁入宫，0-11）
    pub tai_sui_palace: u8,
}

// ============================================================================
// 宫位详细文本
// ============================================================================

/// 宫位详细解读文本
///
/// 用于前端显示的完整解读
#[derive(Clone, Encode, Decode, TypeInfo, RuntimeDebug, Default)]
pub struct PalaceDetailText {
    /// 宫位名称
    pub palace_name: Vec<u8>,

    /// 宫位评分
    pub score: u8,

    /// 吉凶等级
    pub fortune_level: FortuneLevel,

    /// 主星列表名称
    pub main_stars: Vec<Vec<u8>>,

    /// 六吉星名称
    pub liu_ji_names: Vec<Vec<u8>>,

    /// 六煞星名称
    pub liu_sha_names: Vec<Vec<u8>>,

    /// 四化描述
    pub si_hua_desc: Vec<Vec<u8>>,

    /// 关键词列表
    pub keywords: Vec<Vec<u8>>,

    /// 综合解读
    pub summary: Vec<u8>,
}

/// 格局详细文本
#[derive(Clone, Encode, Decode, TypeInfo, RuntimeDebug, Default)]
pub struct PatternDetailText {
    /// 格局名称
    pub name: Vec<u8>,

    /// 是否吉格
    pub is_auspicious: bool,

    /// 格局分数
    pub score: i8,

    /// 格局描述
    pub description: Vec<u8>,

    /// 影响说明
    pub impact: Vec<u8>,

    /// 关键宫位名称
    pub key_palaces: Vec<Vec<u8>>,
}

/// 大限详细文本
#[derive(Clone, Encode, Decode, TypeInfo, RuntimeDebug, Default)]
pub struct DaXianDetailText {
    /// 大限序号
    pub index: u8,

    /// 起始年龄
    pub start_age: u8,

    /// 结束年龄
    pub end_age: u8,

    /// 大限宫位名称
    pub palace_name: Vec<u8>,

    /// 大限评分
    pub score: u8,

    /// 运势等级
    pub fortune_level: FortuneLevel,

    /// 关键词列表
    pub keywords: Vec<Vec<u8>>,

    /// 综合解读
    pub summary: Vec<u8>,

    /// 四化描述
    pub si_hua_desc: Vec<Vec<u8>>,
}

// ============================================================================
// Runtime API 声明
// ============================================================================

sp_api::decl_runtime_apis! {
    /// 紫微斗数解卦 Runtime API
    ///
    /// 提供实时计算的解卦功能，无需链上存储解卦结果
    pub trait ZiweiInterpretationApi {
        // ====================================================================
        // 核心解卦 API
        // ====================================================================

        /// 获取完整解卦数据
        ///
        /// 实时计算命盘的完整解卦，包括：
        /// - 整体评分
        /// - 十二宫解读
        /// - 格局识别
        /// - 四化分析
        /// - 大限解读
        ///
        /// # 参数
        /// - `chart_id`: 命盘ID
        ///
        /// # 返回
        /// 完整解卦数据，如果命盘不存在返回 None
        fn get_interpretation(chart_id: u64) -> Option<ZiweiInterpretation>;

        /// 获取整体评分
        ///
        /// # 参数
        /// - `chart_id`: 命盘ID
        ///
        /// # 返回
        /// 整体评分数据
        fn get_overall_score(chart_id: u64) -> Option<ChartOverallScore>;

        // ====================================================================
        // 宫位解读 API
        // ====================================================================

        /// 获取单个宫位的解读
        ///
        /// # 参数
        /// - `chart_id`: 命盘ID
        /// - `gong_wei`: 宫位类型
        ///
        /// # 返回
        /// 宫位解读数据
        fn get_palace_interpretation(
            chart_id: u64,
            gong_wei: GongWei,
        ) -> Option<PalaceInterpretation>;

        /// 获取宫位详细文本解读
        ///
        /// # 参数
        /// - `chart_id`: 命盘ID
        /// - `gong_wei`: 宫位类型
        ///
        /// # 返回
        /// 包含文本的详细解读
        fn get_palace_detail(
            chart_id: u64,
            gong_wei: GongWei,
        ) -> Option<PalaceDetailText>;

        /// 获取所有宫位的解读
        ///
        /// # 参数
        /// - `chart_id`: 命盘ID
        ///
        /// # 返回
        /// 十二宫解读数组
        fn get_all_palace_interpretations(
            chart_id: u64,
        ) -> Option<[PalaceInterpretation; 12]>;

        // ====================================================================
        // 格局识别 API
        // ====================================================================

        /// 获取命盘格局列表
        ///
        /// # 参数
        /// - `chart_id`: 命盘ID
        ///
        /// # 返回
        /// 识别到的格局列表
        fn get_patterns(chart_id: u64) -> Option<Vec<PatternInfo>>;

        /// 获取格局详细说明
        ///
        /// # 参数
        /// - `pattern_type`: 格局类型
        ///
        /// # 返回
        /// 格局详细文本说明
        fn get_pattern_detail(pattern_type: PatternType) -> PatternDetailText;

        /// 检查是否有特定格局
        ///
        /// # 参数
        /// - `chart_id`: 命盘ID
        /// - `pattern_type`: 要检查的格局类型
        ///
        /// # 返回
        /// 是否存在该格局及其强度
        fn has_pattern(chart_id: u64, pattern_type: PatternType) -> Option<(bool, u8)>;

        // ====================================================================
        // 四化分析 API
        // ====================================================================

        /// 获取四化飞星分析
        ///
        /// # 参数
        /// - `chart_id`: 命盘ID
        ///
        /// # 返回
        /// 四化分析数据
        fn get_si_hua_analysis(chart_id: u64) -> Option<SiHuaAnalysis>;

        /// 获取宫干四化
        ///
        /// # 参数
        /// - `chart_id`: 命盘ID
        /// - `palace_idx`: 宫位索引（0-11）
        ///
        /// # 返回
        /// 该宫干的四化星及其飞入宫位
        fn get_palace_si_hua(
            chart_id: u64,
            palace_idx: u8,
        ) -> Option<([SiHuaStar; 4], [u8; 4])>;

        // ====================================================================
        // 大限运势 API
        // ====================================================================

        /// 获取指定大限的解读
        ///
        /// # 参数
        /// - `chart_id`: 命盘ID
        /// - `da_xian_index`: 大限序号（1-12）
        ///
        /// # 返回
        /// 大限解读数据
        fn get_da_xian_interpretation(
            chart_id: u64,
            da_xian_index: u8,
        ) -> Option<DaXianInterpretation>;

        /// 获取大限详细文本
        ///
        /// # 参数
        /// - `chart_id`: 命盘ID
        /// - `da_xian_index`: 大限序号（1-12）
        ///
        /// # 返回
        /// 大限详细文本解读
        fn get_da_xian_detail(
            chart_id: u64,
            da_xian_index: u8,
        ) -> Option<DaXianDetailText>;

        /// 根据年龄获取当前大限
        ///
        /// # 参数
        /// - `chart_id`: 命盘ID
        /// - `age`: 当前年龄
        ///
        /// # 返回
        /// 当前大限解读
        fn get_current_da_xian(
            chart_id: u64,
            age: u8,
        ) -> Option<DaXianInterpretation>;

        // ====================================================================
        // 流年运势 API
        // ====================================================================

        /// 获取流年运势
        ///
        /// # 参数
        /// - `chart_id`: 命盘ID
        /// - `year`: 公历年份
        ///
        /// # 返回
        /// 流年运势数据
        fn get_liu_nian_fortune(
            chart_id: u64,
            year: u16,
        ) -> Option<LiuNianFortune>;

        /// 获取多年运势趋势
        ///
        /// # 参数
        /// - `chart_id`: 命盘ID
        /// - `start_year`: 起始年份
        /// - `end_year`: 结束年份
        ///
        /// # 返回
        /// 多年运势列表
        fn get_fortune_trend(
            chart_id: u64,
            start_year: u16,
            end_year: u16,
        ) -> Vec<LiuNianFortune>;

        // ====================================================================
        // 批量查询 API
        // ====================================================================

        /// 批量获取解卦结果
        ///
        /// # 参数
        /// - `chart_ids`: 命盘ID列表
        ///
        /// # 返回
        /// 解卦结果列表
        fn get_interpretations_batch(
            chart_ids: Vec<u64>,
        ) -> Vec<Option<ZiweiInterpretation>>;

        // ====================================================================
        // 隐私相关 API
        // ====================================================================

        /// 获取加密数据
        ///
        /// 用于 Partial/Private 模式下获取链上存储的加密数据，
        /// 前端需要使用用户私钥解密。
        ///
        /// # 参数
        /// - `chart_id`: 命盘记录 ID
        ///
        /// # 返回
        /// 加密数据（如果存在）
        fn get_encrypted_data(chart_id: u64) -> Option<Vec<u8>>;

        /// 获取所有者密钥备份
        ///
        /// 用于所有者恢复加密密钥或授权他人查看。
        ///
        /// # 参数
        /// - `chart_id`: 命盘记录 ID
        ///
        /// # 返回
        /// 80 字节的密钥备份（如果存在）
        fn get_owner_key_backup(chart_id: u64) -> Option<[u8; 80]>;

        /// 临时计算命盘（用于 Private 模式）
        ///
        /// 当用户使用 Private 模式保存了命盘，但需要查看解读时：
        /// 1. 前端获取加密数据并解密
        /// 2. 使用解密后的日期时间参数调用此 API
        /// 3. 返回完整的命盘计算结果（不存储）
        ///
        /// # 参数
        /// - `lunar_year`: 农历年份
        /// - `lunar_month`: 农历月份 (1-12)
        /// - `lunar_day`: 农历日期 (1-30)
        /// - `birth_hour`: 出生时辰地支索引 (0-11)
        /// - `gender`: 性别 (0=男, 1=女)
        /// - `is_leap_month`: 是否闰月
        ///
        /// # 返回
        /// 临时命盘计算结果（不存储到链上）
        fn compute_chart(
            lunar_year: u16,
            lunar_month: u8,
            lunar_day: u8,
            birth_hour: u8,
            gender: u8,
            is_leap_month: bool,
        ) -> Option<ZiweiChartResult>;

        /// 获取命盘公开元数据
        ///
        /// 返回命盘的公开元数据，不包含敏感信息。
        /// 适用于所有隐私模式。
        ///
        /// # 参数
        /// - `chart_id`: 命盘记录 ID
        ///
        /// # 返回
        /// 公开元数据
        fn get_public_metadata(chart_id: u64) -> Option<ZiweiPublicMetadata>;
    }
}

// ============================================================================
// 辅助函数实现（供 runtime 实现 API 时使用）
// ============================================================================

/// 生成完整解卦数据
///
/// # 参数
/// - `chart`: 命盘数据
///
/// # 返回
/// 完整解卦数据，如果必要字段缺失则返回 None
pub fn generate_interpretation<AccountId, BlockNumber, Moment, MaxCidLen: Get<u32>>(
    chart: &ZiweiChart<AccountId, BlockNumber, Moment, MaxCidLen>,
) -> Option<ZiweiInterpretation> {
    // 检查必要字段是否存在
    let palaces = chart.palaces.as_ref()?;
    let ming_gong_pos = chart.ming_gong_pos?;
    let shen_gong_pos = chart.shen_gong_pos?;
    let si_hua_stars = chart.si_hua_stars?;
    let qi_yun_age = chart.qi_yun_age?;
    let da_yun_shun = chart.da_yun_shun?;

    // 1. 生成十二宫解读
    let mut palace_interpretations: [PalaceInterpretation; 12] = Default::default();
    let mut palace_scores: [u8; 12] = [50u8; 12];

    for (i, palace) in palaces.iter().enumerate() {
        palace_interpretations[i] = generate_palace_interpretation(palace);
        palace_scores[i] = palace_interpretations[i].score;
    }

    // 2. 识别格局
    let patterns = identify_all_patterns(palaces, ming_gong_pos);
    let pattern_total_score: i32 = patterns.iter().map(|p| p.score as i32).sum();
    let auspicious_count = patterns.iter().filter(|p| p.is_auspicious).count();
    let inauspicious_count = patterns.len() - auspicious_count;

    // 3. 计算整体评分
    let overall_score = generate_overall_score(
        &palace_scores,
        ming_gong_pos,
        auspicious_count,
        inauspicious_count,
        pattern_total_score,
    );

    // 4. 四化分析
    let si_hua_analysis = analyze_si_hua(
        palaces,
        ming_gong_pos,
        si_hua_stars,
    );

    // 5. 大限解读
    let da_xian_interpretations = generate_da_xian_interpretations_inner(
        palaces,
        ming_gong_pos,
        qi_yun_age,
        da_yun_shun,
        &palace_scores,
    );

    // 6. 五行分布
    let wu_xing_distribution = calculate_wu_xing_distribution(palaces);

    // 7. 命主身主星
    let ming_zhu_star = calculate_ming_zhu_star(ming_gong_pos);
    let shen_zhu_star = calculate_shen_zhu_star(shen_gong_pos);

    Some(ZiweiInterpretation {
        chart_id: chart.id,
        overall_score,
        palace_interpretations,
        patterns,
        si_hua_analysis,
        da_xian_interpretations,
        wu_xing_distribution,
        ming_zhu_star,
        shen_zhu_star,
        created_at: 0, // 由调用者设置
        ai_interpretation_cid: None,
    })
}

/// 生成大限解读（向后兼容包装器）
#[allow(dead_code)]
fn generate_da_xian_interpretations<AccountId, BlockNumber, Moment, MaxCidLen: Get<u32>>(
    chart: &ZiweiChart<AccountId, BlockNumber, Moment, MaxCidLen>,
    palace_scores: &[u8; 12],
) -> [DaXianInterpretation; 12] {
    // 尝试获取必要字段，如果缺失则返回默认值
    let palaces = match chart.palaces.as_ref() {
        Some(p) => p,
        None => return Default::default(),
    };
    let ming_gong_pos = chart.ming_gong_pos.unwrap_or(0);
    let qi_yun_age = chart.qi_yun_age.unwrap_or(2);
    let da_yun_shun = chart.da_yun_shun.unwrap_or(true);

    generate_da_xian_interpretations_inner(palaces, ming_gong_pos, qi_yun_age, da_yun_shun, palace_scores)
}

/// 生成大限解读（内部实现）
fn generate_da_xian_interpretations_inner(
    palaces: &[Palace; 12],
    ming_gong_pos: u8,
    qi_yun_age: u8,
    da_yun_shun: bool,
    palace_scores: &[u8; 12],
) -> [DaXianInterpretation; 12] {
    let mut da_xian: [DaXianInterpretation; 12] = Default::default();

    for i in 0..12 {
        // 计算大限宫位索引
        let gong_index = if da_yun_shun {
            (ming_gong_pos as usize + i) % 12
        } else {
            (ming_gong_pos as usize + 12 - i) % 12
        };

        // 计算年龄范围
        let start_age = qi_yun_age + (i as u8 * 10);
        let end_age = start_age + 9;

        // 获取该宫干四化
        let si_hua_fei_ru = calculate_fei_hua(palaces, gong_index as u8);

        // 评分
        let base_score = palace_scores[gong_index];
        let score = base_score; // 可以加入大限四化影响

        // 关键词
        let keywords = select_da_xian_keywords(score, i as u8);

        da_xian[i] = DaXianInterpretation {
            index: (i + 1) as u8,
            start_age,
            end_age,
            gong_index: gong_index as u8,
            score,
            fortune_level: FortuneLevel::from_score(score),
            si_hua_fei_ru,
            keywords,
        };
    }

    da_xian
}

/// 选择大限关键词
fn select_da_xian_keywords(score: u8, da_xian_index: u8) -> [u8; 3] {
    // 根据评分和大限阶段选择关键词
    let phase_keyword = match da_xian_index {
        0..=1 => 60,   // 青年期
        2..=4 => 61,   // 壮年期
        5..=7 => 62,   // 中年期
        _ => 63,       // 晚年期
    };

    let fortune_keyword = match score {
        80..=100 => 64, // 大吉
        60..=79 => 65,  // 顺遂
        40..=59 => 66,  // 平稳
        _ => 67,        // 需谨慎
    };

    let advice_keyword = match score {
        70..=100 => 68, // 宜进取
        50..=69 => 69,  // 守成
        _ => 70,        // 韬光养晦
    };

    [phase_keyword, fortune_keyword, advice_keyword]
}

/// 计算命主星
///
/// 根据命宫位置确定命主星
fn calculate_ming_zhu_star(ming_gong_pos: u8) -> u8 {
    // 命主星根据命宫地支确定
    // 子=贪狼, 丑=巨门, 寅=禄存, 卯=文曲, 辰=廉贞, 巳=武曲
    // 午=破军, 未=武曲, 申=廉贞, 酉=文曲, 戌=禄存, 亥=巨门
    match ming_gong_pos % 12 {
        0 => 8,   // 贪狼
        1 => 9,   // 巨门
        2 => 255, // 禄存（非主星）
        3 => 255, // 文曲（非主星）
        4 => 5,   // 廉贞
        5 => 3,   // 武曲
        6 => 13,  // 破军
        7 => 3,   // 武曲
        8 => 5,   // 廉贞
        9 => 255, // 文曲（非主星）
        10 => 255, // 禄存（非主星）
        _ => 9,   // 巨门
    }
}

/// 计算身主星
///
/// 根据身宫位置确定身主星
fn calculate_shen_zhu_star(shen_gong_pos: u8) -> u8 {
    // 身主星根据年支确定（简化处理，使用身宫位置）
    // 子午=火星, 丑未=天相, 寅申=天梁, 卯酉=天同, 辰戌=文昌, 巳亥=天机
    match shen_gong_pos % 6 {
        0 => 255,  // 火星（非主星）
        1 => 10,   // 天相
        2 => 11,   // 天梁
        3 => 4,    // 天同
        4 => 255,  // 文昌（非主星）
        _ => 1,    // 天机
    }
}

/// 生成格局详细说明
///
/// # 参数
/// - `pattern_type`: 格局类型
///
/// # 返回
/// 格局详细文本
pub fn generate_pattern_detail_text(pattern_type: PatternType) -> PatternDetailText {
    let name = pattern_type.name().as_bytes().to_vec();
    let is_auspicious = pattern_type.is_auspicious();
    let score = pattern_type.base_score();

    let description = get_pattern_description(pattern_type).as_bytes().to_vec();
    let impact = get_pattern_impact(pattern_type).as_bytes().to_vec();

    PatternDetailText {
        name,
        is_auspicious,
        score,
        description,
        impact,
        key_palaces: Vec::new(),
    }
}

/// 获取格局描述
fn get_pattern_description(pattern: PatternType) -> &'static str {
    match pattern {
        PatternType::ZiFuTongGong => "紫微、天府二星同坐命宫，为帝王与库星同宫，主富贵双全。",
        PatternType::ZiFuChaoYuan => "紫微、天府在三方四正会照命宫，主贵气逼人，一生有成。",
        PatternType::TianFuChaoYuan => "天府守命宫，逢禄存或化禄同宫，主财帛丰盈，衣食无忧。",
        PatternType::JunChenQingHui => "紫微为君，天相、天府为臣，三方会合，主大贵之命。",
        PatternType::FuXiangChaoYuan => "天府、天相在命宫或三方会照，主富贵绑身，地位崇高。",
        PatternType::JiYueTongLiang => "天机、太阴、天同、天梁四星会合，主清贵文秀，宜从政。",
        PatternType::RiYueBingMing => "太阳、太阴在旺地会照命宫，日月并明，主光明磊落，富贵双全。",
        PatternType::RiZhaoLeiMen => "太阳在卯宫守命且庙旺，主早年发达，一生光彩。",
        PatternType::YueLangTianMen => "太阴在亥宫守命且庙旺，主聪明秀气，财帛丰足。",
        PatternType::MingZhuChuHai => "太阴在酉宫守命，主才华出众，文采斐然。",
        PatternType::YangLiangChangLu => "太阳、天梁在三方会文昌、禄存，主功名显达，贵人多助。",
        PatternType::TanWuTongXing => "贪狼、武曲同坐丑未宫，主武贵或偏财运佳。",
        PatternType::HuoTanGeJu => "火星、贪狼同宫于命宫，主暴发横财，但易来得快去得也快。",
        PatternType::LingTanGeJu => "铃星、贪狼同宫于命宫，类似火贪格，主横财暴发。",
        PatternType::SanQiJiaHui => "化禄、化权、化科三化在命宫三方会合，主大富大贵。",
        PatternType::ShuangLuJiaMing => "禄存、化禄夹命宫，主财帛滚滚，一生富裕。",
        PatternType::ShuangLuJiaCai => "禄存、化禄夹财帛宫，主财源广进，理财有道。",
        PatternType::KeQuanLuJia => "化科、化权、化禄夹命宫，主贵气加身，名利双收。",
        PatternType::ZuoYouJiaMing => "左辅、右弼夹命宫，主贵人多助，做事有成。",
        PatternType::ChangQuJiaMing => "文昌、文曲夹命宫，主文采斐然，学业有成。",
        PatternType::KuiYueJiaMing => "天魁、天钺夹命宫，主贵人相助，逢凶化吉。",
        PatternType::LuMaJiaoChiGeJu => "禄存、天马同宫或会照命宫，主财运亨通，发财快速。",
        PatternType::LingChangTuoWu => "铃星、文昌、陀罗、武曲同宫，主波折困顿，需防破财。",
        PatternType::JiJiTongGong => "巨门、天机在辰戌宫同宫，主口舌是非，事业多变。",
        PatternType::JuRiTongGong => "巨门、太阳同宫且太阳落陷，主是非缠身，劳碌奔波。",
        PatternType::MingWuZhengYao => "命宫无主星（空宫），需借对宫星曜，主早年辛劳。",
        PatternType::MaTouDaiJian => "午宫擎羊守命，主性格刚烈，易有血光之灾。",
        PatternType::YangTuoJiaMing => "擎羊、陀罗夹命宫，主一生多灾多难，需特别小心。",
        PatternType::HuoLingJiaMing => "火星、铃星夹命宫，主脾气暴躁，人际关系差。",
        PatternType::KongJieJiaMing => "地空、地劫夹命宫，主钱财难聚，空想多于实际。",
        PatternType::YangTuoJiaJi => "擎羊、陀罗夹化忌，主凶险异常，诸事不利。",
        PatternType::SiShaChongMing => "擎羊、陀罗、火星、铃星冲命宫，主一生坎坷，需多修行。",
    }
}

/// 获取格局影响说明
fn get_pattern_impact(pattern: PatternType) -> &'static str {
    if pattern.is_auspicious() {
        match pattern {
            PatternType::ZiFuTongGong => "有利于事业发展，适合从政或经商，一生富贵。",
            PatternType::SanQiJiaHui => "名利双收，事业上有重大突破的机会。",
            PatternType::ShuangLuJiaMing => "财运极佳，适合投资理财，但需防骄奢。",
            PatternType::ZuoYouJiaMing => "贵人运强，事业发展顺遂，有助力可借。",
            _ => "整体运势向好，把握机遇可有大成。",
        }
    } else {
        match pattern {
            PatternType::YangTuoJiaMing => "需防意外伤害，做事谨慎，避免冲动。",
            PatternType::SiShaChongMing => "人生多波折，宜修心养性，积德行善。",
            PatternType::MingWuZhengYao => "早年辛劳，中晚年转好，需靠自身努力。",
            PatternType::YangTuoJiaJi => "凶险格局，需特别注意健康和安全。",
            _ => "运势有阻，需谨慎行事，低调为宜。",
        }
    }
}

/// 计算流年运势
///
/// # 参数
/// - `chart`: 命盘数据
/// - `year`: 公历年份
///
/// # 返回
/// 流年运势数据，如果必要字段缺失则返回默认值
pub fn calculate_liu_nian_fortune<AccountId, BlockNumber, Moment, MaxCidLen: Get<u32>>(
    chart: &ZiweiChart<AccountId, BlockNumber, Moment, MaxCidLen>,
    year: u16,
) -> LiuNianFortune {
    // 计算流年天干地支
    let tian_gan = TianGan::from_index(((year - 4) % 10) as u8);
    let di_zhi = DiZhi::from_index(((year - 4) % 12) as u8);

    // 流年四化
    let si_hua_stars = crate::algorithm::get_si_hua_stars_full(tian_gan);

    // 太岁入宫（流年地支落入的宫位）
    let tai_sui_palace = di_zhi.index();

    // 尝试获取命盘数据，如果缺失则返回默认评分
    let palaces = match chart.palaces.as_ref() {
        Some(p) => p,
        None => {
            return LiuNianFortune {
                year,
                tian_gan,
                di_zhi,
                si_hua_stars,
                fortune_score: 50,
                fortune_level: FortuneLevel::Ping,
                wealth_fortune: 50,
                career_fortune: 50,
                relationship_fortune: 50,
                health_fortune: 50,
                keywords: select_liu_nian_keywords(50),
                tai_sui_palace,
            };
        }
    };
    let ming_gong_pos = chart.ming_gong_pos.unwrap_or(0);

    // 基础评分（根据太岁宫位的评分）
    let base_score = calculate_palace_score(&palaces[tai_sui_palace as usize]);

    // 流年四化对命宫的影响
    let mut si_hua_bonus: i32 = 0;
    for (i, star) in si_hua_stars.iter().enumerate() {
        // 检查四化星是否落入命宫或三方
        if let Some(star_palace) = find_star_palace(palaces, *star) {
            if star_palace == ming_gong_pos {
                si_hua_bonus += match i {
                    0 => 10,   // 化禄
                    1 => 8,    // 化权
                    2 => 5,    // 化科
                    _ => -15,  // 化忌
                };
            }
        }
    }

    let fortune_score = (base_score as i32 + si_hua_bonus).clamp(0, 100) as u8;

    // 各项运势
    let wealth_fortune = calculate_aspect_fortune_inner(palaces, ming_gong_pos, GongWei::CaiBo);
    let career_fortune = calculate_aspect_fortune_inner(palaces, ming_gong_pos, GongWei::GuanLu);
    let relationship_fortune = calculate_aspect_fortune_inner(palaces, ming_gong_pos, GongWei::FuQi);
    let health_fortune = calculate_aspect_fortune_inner(palaces, ming_gong_pos, GongWei::JiE);

    // 关键词
    let keywords = select_liu_nian_keywords(fortune_score);

    LiuNianFortune {
        year,
        tian_gan,
        di_zhi,
        si_hua_stars,
        fortune_score,
        fortune_level: FortuneLevel::from_score(fortune_score),
        wealth_fortune,
        career_fortune,
        relationship_fortune,
        health_fortune,
        keywords,
        tai_sui_palace,
    }
}

/// 计算特定宫位的流年运势（向后兼容包装器）
#[allow(dead_code)]
fn calculate_aspect_fortune<AccountId, BlockNumber, Moment, MaxCidLen: Get<u32>>(
    chart: &ZiweiChart<AccountId, BlockNumber, Moment, MaxCidLen>,
    _year: u16,
    gong_wei: GongWei,
) -> u8 {
    let palaces = match chart.palaces.as_ref() {
        Some(p) => p,
        None => return 50,
    };
    let ming_gong_pos = chart.ming_gong_pos.unwrap_or(0);
    calculate_aspect_fortune_inner(palaces, ming_gong_pos, gong_wei)
}

/// 计算特定宫位的流年运势（内部实现）
fn calculate_aspect_fortune_inner(
    palaces: &[Palace; 12],
    ming_gong_pos: u8,
    gong_wei: GongWei,
) -> u8 {
    // 获取对应宫位的索引
    let gong_index = match gong_wei {
        GongWei::CaiBo => (ming_gong_pos + 8) % 12,
        GongWei::GuanLu => (ming_gong_pos + 4) % 12,
        GongWei::FuQi => (ming_gong_pos + 10) % 12,
        GongWei::JiE => (ming_gong_pos + 7) % 12,
        _ => ming_gong_pos,
    };

    calculate_palace_score(&palaces[gong_index as usize])
}

/// 临时计算命盘（供 Runtime API 使用）
///
/// 用于 Private 模式下，前端解密出生时间后调用此函数计算命盘
pub fn compute_chart_result(
    lunar_year: u16,
    lunar_month: u8,
    lunar_day: u8,
    birth_hour: u8,
    gender: u8,
    _is_leap_month: bool,
) -> Option<ZiweiChartResult> {
    use crate::algorithm::*;

    // 参数校验
    if lunar_month < 1 || lunar_month > 12 {
        return None;
    }
    if lunar_day < 1 || lunar_day > 30 {
        return None;
    }
    if birth_hour > 11 {
        return None;
    }

    // 转换参数
    let birth_hour_dizhi = DiZhi::from_index(birth_hour);
    let gender_enum = if gender == 0 { Gender::Male } else { Gender::Female };

    // 计算年干支
    let year_gan = TianGan::from_index(((lunar_year - 4) % 10) as u8);
    let year_zhi = DiZhi::from_index(((lunar_year - 4) % 12) as u8);

    // 计算命宫位置
    let ming_gong_pos = calculate_ming_gong(lunar_month, birth_hour_dizhi);

    // 计算身宫位置
    let shen_gong_pos = calculate_shen_gong(lunar_month, birth_hour_dizhi);

    // 计算五行局
    let (wu_xing_ju, ju_shu) = calculate_wu_xing_ju(year_gan, ming_gong_pos);

    // 计算紫微星位置
    let ziwei_pos = calculate_ziwei_position(lunar_day, ju_shu);

    // 计算天府星位置
    let tianfu_pos = calculate_tianfu_position(ziwei_pos);

    // 初始化十二宫
    let mut palaces = init_palaces(year_gan, ming_gong_pos);

    // 安紫微星系
    let ziwei_series = place_ziwei_series(ziwei_pos);
    for (star, pos) in ziwei_series.iter() {
        let palace = &mut palaces[*pos as usize];
        for slot in palace.zhu_xing.iter_mut() {
            if slot.is_none() {
                *slot = Some(*star);
                break;
            }
        }
    }

    // 安天府星系
    let tianfu_series = place_tianfu_series(tianfu_pos);
    for (star, pos) in tianfu_series.iter() {
        let palace = &mut palaces[*pos as usize];
        for slot in palace.zhu_xing.iter_mut() {
            if slot.is_none() {
                *slot = Some(*star);
                break;
            }
        }
    }

    // 安六吉星
    let (wen_chang, wen_qu) = calculate_wen_chang_qu(birth_hour_dizhi);
    let (zuo_fu, you_bi) = calculate_zuo_fu_you_bi(lunar_month);
    let (tian_kui, tian_yue) = calculate_tian_kui_yue(year_gan);

    palaces[wen_chang as usize].liu_ji[0] = true;
    palaces[wen_qu as usize].liu_ji[1] = true;
    palaces[zuo_fu as usize].liu_ji[2] = true;
    palaces[you_bi as usize].liu_ji[3] = true;
    palaces[tian_kui as usize].liu_ji[4] = true;
    palaces[tian_yue as usize].liu_ji[5] = true;

    // 安六煞星
    let (qing_yang, tuo_luo) = calculate_qing_yang_tuo_luo(year_gan);
    let (huo_xing, ling_xing) = calculate_huo_ling(year_zhi, birth_hour_dizhi);
    let (di_kong, di_jie) = calculate_di_kong_jie(birth_hour_dizhi);

    palaces[qing_yang as usize].liu_sha[0] = true;
    palaces[tuo_luo as usize].liu_sha[1] = true;
    palaces[huo_xing as usize].liu_sha[2] = true;
    palaces[ling_xing as usize].liu_sha[3] = true;
    palaces[di_kong as usize].liu_sha[4] = true;
    palaces[di_jie as usize].liu_sha[5] = true;

    // 安禄存天马
    let lu_cun = calculate_lu_cun(year_gan);
    palaces[lu_cun as usize].lu_cun = true;
    let tian_ma_pos = calculate_tian_ma(year_zhi);
    palaces[tian_ma_pos as usize].tian_ma = true;

    // 获取四化星
    let si_hua_stars = get_si_hua_stars_full(year_gan);

    // 计算起运
    let qi_yun_age = calculate_qi_yun_age(ju_shu);
    let da_yun_shun = calculate_da_yun_direction(year_gan, gender_enum);

    Some(ZiweiChartResult {
        wu_xing_ju,
        ju_shu,
        ming_gong_pos,
        shen_gong_pos,
        ziwei_pos,
        tianfu_pos,
        palaces,
        si_hua_stars,
        qi_yun_age,
        da_yun_shun,
        year_gan,
        year_zhi,
    })
}

/// 选择流年关键词
fn select_liu_nian_keywords(score: u8) -> [u8; 3] {
    let fortune = match score {
        85..=100 => 80, // 大吉
        70..=84 => 81,  // 吉
        55..=69 => 82,  // 平
        40..=54 => 83,  // 小凶
        _ => 84,        // 凶
    };

    let advice = match score {
        70..=100 => 85, // 宜积极
        50..=69 => 86,  // 宜稳健
        _ => 87,        // 宜守成
    };

    let focus = match score {
        60..=100 => 88, // 把握机遇
        _ => 89,        // 化解困难
    };

    [fortune, advice, focus]
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liu_nian_fortune_default() {
        let fortune = LiuNianFortune::default();
        assert_eq!(fortune.year, 0);
        assert_eq!(fortune.fortune_score, 0);
    }

    #[test]
    fn test_palace_detail_text_default() {
        let detail = PalaceDetailText::default();
        assert!(detail.palace_name.is_empty());
        assert_eq!(detail.score, 0);
    }

    #[test]
    fn test_pattern_detail_text_default() {
        let detail = PatternDetailText::default();
        assert!(detail.name.is_empty());
        assert!(!detail.is_auspicious);
    }

    #[test]
    fn test_generate_pattern_detail_text() {
        let detail = generate_pattern_detail_text(PatternType::ZiFuTongGong);
        assert!(!detail.name.is_empty());
        assert!(detail.is_auspicious);
        assert_eq!(detail.score, 50);
    }

    #[test]
    fn test_calculate_ming_zhu_star() {
        assert_eq!(calculate_ming_zhu_star(0), 8);  // 子宫-贪狼
        assert_eq!(calculate_ming_zhu_star(6), 13); // 午宫-破军
    }

    #[test]
    fn test_calculate_shen_zhu_star() {
        assert_eq!(calculate_shen_zhu_star(1), 10);  // 天相
        assert_eq!(calculate_shen_zhu_star(2), 11);  // 天梁
    }

    #[test]
    fn test_select_da_xian_keywords() {
        let keywords = select_da_xian_keywords(85, 0);
        assert_eq!(keywords[0], 60); // 青年期
        assert_eq!(keywords[1], 64); // 大吉
    }

    #[test]
    fn test_select_liu_nian_keywords() {
        let keywords = select_liu_nian_keywords(90);
        assert_eq!(keywords[0], 80); // 大吉
        assert_eq!(keywords[1], 85); // 宜积极
    }

    #[test]
    fn test_get_pattern_description() {
        let desc = get_pattern_description(PatternType::ZiFuTongGong);
        assert!(desc.contains("紫微"));
        assert!(desc.contains("天府"));
    }

    #[test]
    fn test_get_pattern_impact() {
        let impact = get_pattern_impact(PatternType::ZiFuTongGong);
        assert!(impact.contains("事业"));

        let impact2 = get_pattern_impact(PatternType::YangTuoJiaMing);
        assert!(impact2.contains("意外"));
    }
}
