//! # 紫微斗数评分算法
//!
//! 本模块实现命盘的评分计算，包括：
//!
//! - **单宫评分**：根据主星、辅星、四化计算宫位分数
//! - **整体评分**：加权计算命盘综合评分
//! - **命格判断**：根据评分和格局判断命格等级
//! - **五行分布**：计算命盘五行分布

use crate::types::*;
use super::enums::*;
use super::structs::*;

// ============================================================================
// 评分权重常量
// ============================================================================

/// 主星庙旺加成（庙=25分）
const STAR_MIAO_WEIGHT: i32 = 25;
/// 主星旺加成（旺=20分）
const STAR_WANG_WEIGHT: i32 = 20;
/// 主星得地加成（得=15分）
const STAR_DE_WEIGHT: i32 = 15;
/// 主星平加成（平=0分）
const STAR_PING_WEIGHT: i32 = 0;
/// 主星不得减分（不得=-5分）
const STAR_BUDE_WEIGHT: i32 = -5;
/// 主星落陷减分（陷=-15分）
const STAR_XIAN_WEIGHT: i32 = -15;

/// 六吉星每颗加分
const LIU_JI_WEIGHT: i32 = 5;
/// 六煞星每颗减分
const LIU_SHA_WEIGHT: i32 = -5;

/// 化禄加分
const SI_HUA_LU_WEIGHT: i32 = 15;
/// 化权加分
const SI_HUA_QUAN_WEIGHT: i32 = 10;
/// 化科加分
const SI_HUA_KE_WEIGHT: i32 = 8;
/// 化忌减分
const SI_HUA_JI_WEIGHT: i32 = -20;

/// 禄存加分
const LU_CUN_WEIGHT: i32 = 10;
/// 天马加分
const TIAN_MA_WEIGHT: i32 = 5;

/// 命宫权重（40%）
const MING_GONG_WEIGHT: u32 = 40;
/// 财帛宫权重（15%）
const CAI_BO_WEIGHT: u32 = 15;
/// 官禄宫权重（15%）
const GUAN_LU_WEIGHT: u32 = 15;
/// 夫妻宫权重（15%）
const FU_QI_WEIGHT: u32 = 15;
/// 其他宫位权重（15%）
const OTHER_WEIGHT: u32 = 15;

/// 基础分数
const BASE_SCORE: i32 = 50;

// ============================================================================
// 主星强度计算
// ============================================================================

/// 计算主星强度
///
/// 根据主星的庙旺落陷状态计算星曜强度
///
/// # 参数
/// - `palace`: 宫位数据
///
/// # 返回
/// 主星强度（0-100）
pub fn calculate_star_strength(palace: &Palace) -> u8 {
    let mut total_strength: i32 = 0;
    let mut star_count = 0;

    for i in 0..3 {
        if palace.zhu_xing[i].is_some() {
            star_count += 1;
            let brightness_score = match palace.zhu_xing_brightness[i] {
                StarBrightness::Miao => 100,
                StarBrightness::Wang => 80,
                StarBrightness::De => 60,
                StarBrightness::Ping => 40,
                StarBrightness::BuDe => 25,
                StarBrightness::Xian => 10,
            };
            total_strength += brightness_score;
        }
    }

    if star_count == 0 {
        // 空宫，返回基础强度
        30
    } else {
        (total_strength / star_count).clamp(0, 100) as u8
    }
}

/// 获取主星亮度对评分的影响
///
/// # 参数
/// - `brightness`: 星曜亮度
///
/// # 返回
/// 评分加成/减分
fn get_brightness_score(brightness: StarBrightness) -> i32 {
    match brightness {
        StarBrightness::Miao => STAR_MIAO_WEIGHT,
        StarBrightness::Wang => STAR_WANG_WEIGHT,
        StarBrightness::De => STAR_DE_WEIGHT,
        StarBrightness::Ping => STAR_PING_WEIGHT,
        StarBrightness::BuDe => STAR_BUDE_WEIGHT,
        StarBrightness::Xian => STAR_XIAN_WEIGHT,
    }
}

// ============================================================================
// 四化影响计算
// ============================================================================

/// 计算四化对宫位的影响
///
/// # 参数
/// - `palace`: 宫位数据
///
/// # 返回
/// 四化影响分数（-50 ~ +50）
pub fn calculate_si_hua_impact(palace: &Palace) -> i8 {
    let mut impact: i32 = 0;

    for si_hua in palace.si_hua.iter().flatten() {
        impact += match si_hua {
            SiHua::HuaLu => SI_HUA_LU_WEIGHT,
            SiHua::HuaQuan => SI_HUA_QUAN_WEIGHT,
            SiHua::HuaKe => SI_HUA_KE_WEIGHT,
            SiHua::HuaJi => SI_HUA_JI_WEIGHT,
        };
    }

    impact.clamp(-50, 50) as i8
}

// ============================================================================
// 单宫评分计算
// ============================================================================

/// 计算单个宫位的评分
///
/// 综合考虑主星、辅星、四化、禄存、天马等因素
///
/// # 参数
/// - `palace`: 宫位数据
///
/// # 返回
/// 宫位评分（0-100）
pub fn calculate_palace_score(palace: &Palace) -> u8 {
    let mut score: i32 = BASE_SCORE;

    // 1. 主星亮度加成（最高 +25 分，最低 -15 分）
    let mut has_main_star = false;
    for i in 0..3 {
        if palace.zhu_xing[i].is_some() {
            has_main_star = true;
            score += get_brightness_score(palace.zhu_xing_brightness[i]);
        }
    }

    // 空宫惩罚
    if !has_main_star {
        score -= 10;
    }

    // 2. 六吉星加成（每颗 +5 分，最多 +30 分）
    let ji_count = palace.liu_ji.iter().filter(|&&x| x).count() as i32;
    score += ji_count * LIU_JI_WEIGHT;

    // 3. 六煞星减分（每颗 -5 分，最多 -30 分）
    let sha_count = palace.liu_sha.iter().filter(|&&x| x).count() as i32;
    score += sha_count * LIU_SHA_WEIGHT;

    // 4. 四化影响（-20 ~ +30 分）
    for si_hua in palace.si_hua.iter().flatten() {
        score += match si_hua {
            SiHua::HuaLu => SI_HUA_LU_WEIGHT,
            SiHua::HuaQuan => SI_HUA_QUAN_WEIGHT,
            SiHua::HuaKe => SI_HUA_KE_WEIGHT,
            SiHua::HuaJi => SI_HUA_JI_WEIGHT,
        };
    }

    // 5. 禄存天马加成
    if palace.lu_cun {
        score += LU_CUN_WEIGHT;
    }
    if palace.tian_ma {
        score += TIAN_MA_WEIGHT;
    }

    // 限制在 0-100 范围
    score.clamp(0, 100) as u8
}

/// 计算宫位影响因素标志
///
/// # 参数
/// - `palace`: 宫位数据
///
/// # 返回
/// 影响因素位标志（8 bits）
pub fn calculate_palace_factors(palace: &Palace) -> u8 {
    let mut factors: u8 = 0;

    // bit 0: 主星庙旺
    for i in 0..3 {
        if palace.zhu_xing[i].is_some() {
            if matches!(palace.zhu_xing_brightness[i], StarBrightness::Miao | StarBrightness::Wang) {
                factors |= 0b0000_0001;
                break;
            }
        }
    }

    // bit 1: 四化加持（有化禄/化权/化科）
    for si_hua in palace.si_hua.iter().flatten() {
        if matches!(si_hua, SiHua::HuaLu | SiHua::HuaQuan | SiHua::HuaKe) {
            factors |= 0b0000_0010;
            break;
        }
    }

    // bit 2: 六吉会照（有任一六吉星）
    if palace.liu_ji.iter().any(|&x| x) {
        factors |= 0b0000_0100;
    }

    // bit 3: 六煞冲破（有任一六煞星）
    if palace.liu_sha.iter().any(|&x| x) {
        factors |= 0b0000_1000;
    }

    // bit 4: 空宫借星
    let has_main_star = palace.zhu_xing.iter().any(|s| s.is_some());
    if !has_main_star {
        factors |= 0b0001_0000;
    }

    // bit 5: 禄存同宫
    if palace.lu_cun {
        factors |= 0b0010_0000;
    }

    // bit 6: 天马同宫
    if palace.tian_ma {
        factors |= 0b0100_0000;
    }

    factors
}

// ============================================================================
// 宫位解读生成
// ============================================================================

/// 生成单个宫位的解读数据
///
/// # 参数
/// - `palace`: 宫位数据
///
/// # 返回
/// 宫位解读结构
pub fn generate_palace_interpretation(palace: &Palace) -> PalaceInterpretation {
    let score = calculate_palace_score(palace);
    let star_strength = calculate_star_strength(palace);
    let si_hua_impact = calculate_si_hua_impact(palace);
    let factors = calculate_palace_factors(palace);

    let liu_ji_count = palace.liu_ji.iter().filter(|&&x| x).count() as u8;
    let liu_sha_count = palace.liu_sha.iter().filter(|&&x| x).count() as u8;

    // 根据宫位类型选择关键词
    let keywords = select_palace_keywords(palace.gong_wei, score, star_strength);

    PalaceInterpretation {
        gong_wei: palace.gong_wei,
        score,
        fortune_level: FortuneLevel::from_score(score),
        star_strength,
        si_hua_impact,
        liu_ji_count,
        liu_sha_count,
        keywords,
        factors,
    }
}

/// 为宫位选择关键词索引
///
/// # 参数
/// - `gong_wei`: 宫位类型
/// - `score`: 宫位评分
/// - `star_strength`: 主星强度
///
/// # 返回
/// 3个关键词索引
fn select_palace_keywords(gong_wei: GongWei, score: u8, star_strength: u8) -> [u8; 3] {
    // 根据评分区间选择关键词
    // 命宫使用 0-99，其他宫位使用 0-49
    match gong_wei {
        GongWei::MingGong => select_ming_gong_keywords(score, star_strength),
        GongWei::CaiBo => select_cai_bo_keywords(score),
        GongWei::GuanLu => select_guan_lu_keywords(score),
        GongWei::FuQi => select_fu_qi_keywords(score),
        GongWei::JiE => select_ji_e_keywords(score),
        GongWei::FuDe => select_fu_de_keywords(score),
        _ => [0, 1, 2], // 其他宫位使用默认关键词
    }
}

/// 命宫关键词选择（使用 MING_GONG_KEYWORDS 100个）
fn select_ming_gong_keywords(score: u8, star_strength: u8) -> [u8; 3] {
    // 根据评分选择性格/能力/运势关键词
    let personality_idx = match score {
        90..=100 => 0,  // 贵气
        80..=89 => 1,   // 聪慧
        70..=79 => 2,   // 稳重
        60..=69 => 4,   // 温和
        50..=59 => 7,   // 谨慎
        40..=49 => 9,   // 内敛
        30..=39 => 12,  // 固执
        20..=29 => 11,  // 悲观
        _ => 14,        // 保守
    };

    // 能力关键词（20-39）
    let ability_idx = match star_strength {
        80..=100 => 20, // 领导力强
        60..=79 => 21,  // 执行力佳
        40..=59 => 23,  // 分析力强
        20..=39 => 25,  // 学习力快
        _ => 27,        // 抗压力好
    };

    // 运势关键词（40-59）
    let fortune_idx = match score {
        85..=100 => 40, // 一生顺遂
        75..=84 => 44,  // 贵人相助
        65..=74 => 42,  // 中年发达
        55..=64 => 47,  // 平稳发展
        45..=54 => 48,  // 大器晚成
        35..=44 => 46,  // 波折较多
        25..=34 => 50,  // 起伏不定
        _ => 41,        // 早年辛劳
    };

    [personality_idx, ability_idx, fortune_idx]
}

/// 财帛宫关键词选择（使用 CAI_BO_KEYWORDS 50个）
fn select_cai_bo_keywords(score: u8) -> [u8; 3] {
    let wealth_idx = match score {
        85..=100 => 0,  // 财源广进
        75..=84 => 1,   // 财运亨通
        65..=74 => 2,   // 正财旺盛
        55..=64 => 5,   // 理财有道
        45..=54 => 7,   // 积累为主
        35..=44 => 8,   // 财来财去
        _ => 9,         // 破耗较多
    };

    let method_idx = match score {
        70..=100 => 10, // 适合经商
        50..=69 => 15,  // 工资收入
        _ => 19,        // 节流为主
    };

    let advice_idx = match score {
        60..=100 => 40, // 宜投资
        40..=59 => 41,  // 宜储蓄
        _ => 45,        // 谨慎理财
    };

    [wealth_idx, method_idx, advice_idx]
}

/// 官禄宫关键词选择（使用 GUAN_LU_KEYWORDS 50个）
fn select_guan_lu_keywords(score: u8) -> [u8; 3] {
    let career_idx = match score {
        85..=100 => 0,  // 事业有成
        75..=84 => 2,   // 步步高升
        65..=74 => 5,   // 事业平稳
        55..=64 => 7,   // 需要努力
        45..=54 => 8,   // 大器晚成
        _ => 6,         // 波折较多
    };

    let ability_idx = match score {
        70..=100 => 20, // 领导能力强
        50..=69 => 21,  // 执行力佳
        _ => 26,        // 独立工作强
    };

    let status_idx = match score {
        75..=100 => 30, // 升职快
        60..=74 => 37,  // 工作稳定
        _ => 36,        // 压力较大
    };

    [career_idx, ability_idx, status_idx]
}

/// 夫妻宫关键词选择（使用 FU_QI_KEYWORDS 50个）
fn select_fu_qi_keywords(score: u8) -> [u8; 3] {
    let relationship_idx = match score {
        85..=100 => 0,  // 感情顺遂
        75..=84 => 1,   // 婚姻美满
        65..=74 => 2,   // 夫妻和睦
        55..=64 => 8,   // 晚婚为宜
        45..=54 => 5,   // 感情波折
        _ => 6,         // 婚姻不顺
    };

    let partner_idx = match score {
        70..=100 => 10, // 配偶贤良
        50..=69 => 11,  // 配偶能干
        _ => 18,        // 配偶顾家
    };

    let advice_idx = match score {
        65..=100 => 48, // 宜沟通
        45..=64 => 47,  // 宜包容
        _ => 49,        // 宜珍惜
    };

    [relationship_idx, partner_idx, advice_idx]
}

/// 疾厄宫关键词选择（使用 JI_E_KEYWORDS 50个）
fn select_ji_e_keywords(score: u8) -> [u8; 3] {
    let health_idx = match score {
        85..=100 => 0,  // 身体健康
        75..=84 => 1,   // 精力充沛
        65..=74 => 2,   // 体质强健
        55..=64 => 8,   // 需要保养
        45..=54 => 5,   // 体质较弱
        _ => 7,         // 慢性病
    };

    let habit_idx = match score {
        70..=100 => 20, // 作息规律
        50..=69 => 23,  // 注意养生
        _ => 28,        // 压力过大
    };

    let advice_idx = match score {
        60..=100 => 40, // 宜运动
        40..=59 => 43,  // 宜体检
        _ => 49,        // 定期检查
    };

    [health_idx, habit_idx, advice_idx]
}

/// 福德宫关键词选择（使用 FU_DE_KEYWORDS 50个）
fn select_fu_de_keywords(score: u8) -> [u8; 3] {
    let spirit_idx = match score {
        85..=100 => 0,  // 心态平和
        75..=84 => 1,   // 乐观开朗
        65..=74 => 3,   // 知足常乐
        55..=64 => 4,   // 淡泊名利
        45..=54 => 5,   // 焦虑较多
        _ => 6,         // 压力较大
    };

    let fortune_idx = match score {
        75..=100 => 30, // 福气深厚
        60..=74 => 34,  // 一生有福
        45..=59 => 37,  // 忙碌充实
        _ => 35,        // 劳碌命
    };

    let advice_idx = match score {
        60..=100 => 43, // 宜旅游
        40..=59 => 48,  // 宜培养爱好
        _ => 49,        // 宜放松心情
    };

    [spirit_idx, fortune_idx, advice_idx]
}

// ============================================================================
// 整体评分计算
// ============================================================================

/// 计算命盘整体评分
///
/// 根据十二宫评分加权计算
///
/// # 参数
/// - `palace_scores`: 十二宫评分数组
/// - `ming_gong_pos`: 命宫位置索引
///
/// # 返回
/// 整体评分（0-100）
pub fn calculate_overall_score(palace_scores: &[u8; 12], ming_gong_pos: u8) -> u8 {
    // 命宫评分（权重 40%）
    let ming_score = palace_scores[ming_gong_pos as usize] as u32;

    // 计算财帛、官禄、夫妻宫的位置（相对于命宫）
    let cai_bo_pos = (ming_gong_pos + 8) % 12;  // 命宫顺数第9宫
    let guan_lu_pos = (ming_gong_pos + 4) % 12; // 命宫顺数第5宫
    let fu_qi_pos = (ming_gong_pos + 10) % 12;  // 命宫顺数第11宫

    let cai_score = palace_scores[cai_bo_pos as usize] as u32;
    let guan_score = palace_scores[guan_lu_pos as usize] as u32;
    let fu_score = palace_scores[fu_qi_pos as usize] as u32;

    // 其他宫位平均分
    let mut other_total: u32 = 0;
    let mut other_count: u32 = 0;
    for (i, &score) in palace_scores.iter().enumerate() {
        let i = i as u8;
        if i != ming_gong_pos && i != cai_bo_pos && i != guan_lu_pos && i != fu_qi_pos {
            other_total += score as u32;
            other_count += 1;
        }
    }
    let other_avg = if other_count > 0 { other_total / other_count } else { 50 };

    // 加权平均
    let overall = (ming_score * MING_GONG_WEIGHT
        + cai_score * CAI_BO_WEIGHT
        + guan_score * GUAN_LU_WEIGHT
        + fu_score * FU_QI_WEIGHT
        + other_avg * OTHER_WEIGHT)
        / 100;

    overall.clamp(0, 100) as u8
}

// ============================================================================
// 吉凶等级判断
// ============================================================================

/// 根据评分判断吉凶等级
///
/// # 参数
/// - `score`: 评分（0-100）
///
/// # 返回
/// 吉凶等级
pub fn determine_fortune_level(score: u8) -> FortuneLevel {
    FortuneLevel::from_score(score)
}

// ============================================================================
// 命格等级判断
// ============================================================================

/// 判断命格等级
///
/// 综合考虑整体评分、格局数量、吉凶格局比例
///
/// # 参数
/// - `overall_score`: 整体评分
/// - `auspicious_patterns`: 吉格数量
/// - `inauspicious_patterns`: 凶格数量
/// - `pattern_total_score`: 格局总分
///
/// # 返回
/// 命格等级
pub fn determine_ming_ge_level(
    overall_score: u8,
    auspicious_patterns: usize,
    inauspicious_patterns: usize,
    pattern_total_score: i32,
) -> MingGeLevel {
    // 综合评分计算
    // 基础分：整体评分
    // 格局加成：吉格+10，凶格-10
    // 格局分数加成
    let pattern_bonus = (auspicious_patterns as i32 * 10) - (inauspicious_patterns as i32 * 10);
    let pattern_score_bonus = (pattern_total_score / 5).clamp(-20, 20);

    let adjusted_score = (overall_score as i32 + pattern_bonus + pattern_score_bonus).clamp(0, 120);

    match adjusted_score {
        100..=120 => MingGeLevel::DiWang,   // 帝王格局（极罕见）
        90..=99 => MingGeLevel::JiGui,      // 极贵格局
        80..=89 => MingGeLevel::DaGui,      // 大贵格局
        70..=79 => MingGeLevel::ZhongGui,   // 中贵格局
        55..=69 => MingGeLevel::XiaoGui,    // 小贵格局
        _ => MingGeLevel::Putong,           // 普通格局
    }
}

// ============================================================================
// 五行分布计算
// ============================================================================

/// 计算命盘五行分布
///
/// 统计各宫位主星的五行属性分布
///
/// # 参数
/// - `palaces`: 十二宫数据
///
/// # 返回
/// 五行分布数组 [金, 木, 水, 火, 土]（每项 0-100）
pub fn calculate_wu_xing_distribution(palaces: &[Palace; 12]) -> [u8; 5] {
    let mut counts = [0u32; 5]; // [金, 木, 水, 火, 土]
    let mut total = 0u32;

    for palace in palaces.iter() {
        for zhu_xing in palace.zhu_xing.iter().flatten() {
            let wu_xing = get_star_wu_xing(*zhu_xing);
            let weight = palace.zhu_xing_brightness[0].weight() as u32;
            counts[wu_xing as usize] += weight;
            total += weight;
        }
    }

    if total == 0 {
        return [20, 20, 20, 20, 20]; // 平均分布
    }

    // 转换为百分比
    [
        ((counts[0] * 100) / total).min(100) as u8, // 金
        ((counts[1] * 100) / total).min(100) as u8, // 木
        ((counts[2] * 100) / total).min(100) as u8, // 水
        ((counts[3] * 100) / total).min(100) as u8, // 火
        ((counts[4] * 100) / total).min(100) as u8, // 土
    ]
}

/// 获取主星的五行属性
///
/// # 参数
/// - `star`: 主星
///
/// # 返回
/// 五行索引（0=金, 1=木, 2=水, 3=火, 4=土）
fn get_star_wu_xing(star: ZhuXing) -> usize {
    match star {
        // 金
        ZhuXing::WuQu | ZhuXing::QiSha => 0,
        // 木
        ZhuXing::TianJi | ZhuXing::TanLang => 1,
        // 水
        ZhuXing::TaiYin | ZhuXing::TianTong | ZhuXing::PoJun => 2,
        // 火
        ZhuXing::TaiYang | ZhuXing::LianZhen => 3,
        // 土
        ZhuXing::ZiWei | ZhuXing::TianFu | ZhuXing::JuMen |
        ZhuXing::TianXiang | ZhuXing::TianLiang => 4,
    }
}

// ============================================================================
// 整体评分结构生成
// ============================================================================

/// 生成命盘整体评分结构
///
/// # 参数
/// - `palace_scores`: 十二宫评分
/// - `ming_gong_pos`: 命宫位置
/// - `auspicious_patterns`: 吉格数量
/// - `inauspicious_patterns`: 凶格数量
/// - `pattern_total_score`: 格局总分
///
/// # 返回
/// 整体评分结构
pub fn generate_overall_score(
    palace_scores: &[u8; 12],
    ming_gong_pos: u8,
    auspicious_patterns: usize,
    inauspicious_patterns: usize,
    pattern_total_score: i32,
) -> ChartOverallScore {
    let overall_score = calculate_overall_score(palace_scores, ming_gong_pos);
    let ming_ge_level = determine_ming_ge_level(
        overall_score,
        auspicious_patterns,
        inauspicious_patterns,
        pattern_total_score,
    );

    // 计算各项指数（从对应宫位评分获取）
    let cai_bo_pos = (ming_gong_pos + 8) % 12;
    let guan_lu_pos = (ming_gong_pos + 4) % 12;
    let fu_qi_pos = (ming_gong_pos + 10) % 12;
    let ji_e_pos = (ming_gong_pos + 7) % 12;
    let fu_de_pos = (ming_gong_pos + 2) % 12;

    ChartOverallScore {
        overall_score,
        ming_ge_level,
        wealth_index: palace_scores[cai_bo_pos as usize],
        career_index: palace_scores[guan_lu_pos as usize],
        relationship_index: palace_scores[fu_qi_pos as usize],
        health_index: palace_scores[ji_e_pos as usize],
        fortune_index: palace_scores[fu_de_pos as usize],
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_palace() -> Palace {
        Palace {
            gong_wei: GongWei::MingGong,
            di_zhi: DiZhi::Zi,
            tian_gan: TianGan::Jia,
            zhu_xing: [Some(ZhuXing::ZiWei), None, None],
            zhu_xing_brightness: [StarBrightness::Miao, StarBrightness::Ping, StarBrightness::Ping],
            liu_ji: [true, true, false, false, false, false], // 文昌文曲
            liu_sha: [false, false, false, false, false, false],
            si_hua: [Some(SiHua::HuaLu), None, None, None],
            lu_cun: true,
            tian_ma: false,
        }
    }

    #[test]
    fn test_calculate_star_strength() {
        let palace = create_test_palace();
        let strength = calculate_star_strength(&palace);
        assert_eq!(strength, 100); // 庙位紫微
    }

    #[test]
    fn test_calculate_star_strength_empty() {
        let palace = Palace::default();
        let strength = calculate_star_strength(&palace);
        assert_eq!(strength, 30); // 空宫
    }

    #[test]
    fn test_calculate_si_hua_impact() {
        let palace = create_test_palace();
        let impact = calculate_si_hua_impact(&palace);
        assert_eq!(impact, 15); // 化禄 +15
    }

    #[test]
    fn test_calculate_si_hua_impact_ji() {
        let mut palace = create_test_palace();
        palace.si_hua = [Some(SiHua::HuaJi), None, None, None];
        let impact = calculate_si_hua_impact(&palace);
        assert_eq!(impact, -20); // 化忌 -20
    }

    #[test]
    fn test_calculate_palace_score() {
        let palace = create_test_palace();
        let score = calculate_palace_score(&palace);
        // 基础 50 + 庙位 25 + 六吉 10 + 化禄 15 + 禄存 10 = 110 -> clamp to 100
        assert_eq!(score, 100);
    }

    #[test]
    fn test_calculate_palace_score_poor() {
        let mut palace = Palace::default();
        palace.zhu_xing = [Some(ZhuXing::TanLang), None, None];
        palace.zhu_xing_brightness = [StarBrightness::Xian, StarBrightness::Ping, StarBrightness::Ping];
        palace.liu_sha = [true, true, true, false, false, false]; // 3颗煞星
        palace.si_hua = [Some(SiHua::HuaJi), None, None, None];

        let score = calculate_palace_score(&palace);
        // 基础 50 + 陷 -15 + 煞星 -15 + 化忌 -20 = 0
        assert_eq!(score, 0);
    }

    #[test]
    fn test_calculate_palace_factors() {
        let palace = create_test_palace();
        let factors = calculate_palace_factors(&palace);

        assert!(factors & 0b0000_0001 != 0); // 主星庙旺
        assert!(factors & 0b0000_0010 != 0); // 四化加持
        assert!(factors & 0b0000_0100 != 0); // 六吉会照
        assert!(factors & 0b0000_1000 == 0); // 无六煞
        assert!(factors & 0b0001_0000 == 0); // 非空宫
        assert!(factors & 0b0010_0000 != 0); // 有禄存
        assert!(factors & 0b0100_0000 == 0); // 无天马
    }

    #[test]
    fn test_calculate_overall_score() {
        let palace_scores = [80, 70, 75, 65, 70, 60, 55, 50, 85, 60, 90, 70];
        let overall = calculate_overall_score(&palace_scores, 0);
        // 命宫 80*40 + 财帛 85*15 + 官禄 70*15 + 夫妻 90*15 + 其他平均 ~62*15
        // = 3200 + 1275 + 1050 + 1350 + 930 = 7805 / 100 = 78
        assert!(overall >= 70 && overall <= 85);
    }

    #[test]
    fn test_determine_fortune_level() {
        assert_eq!(determine_fortune_level(95), FortuneLevel::DaJi);
        assert_eq!(determine_fortune_level(80), FortuneLevel::Ji);
        assert_eq!(determine_fortune_level(65), FortuneLevel::XiaoJi);
        assert_eq!(determine_fortune_level(50), FortuneLevel::Ping);
        assert_eq!(determine_fortune_level(30), FortuneLevel::XiaoXiong);
        assert_eq!(determine_fortune_level(15), FortuneLevel::Xiong);
        assert_eq!(determine_fortune_level(5), FortuneLevel::DaXiong);
    }

    #[test]
    fn test_determine_ming_ge_level() {
        // 高分 + 多吉格 = 极贵或帝王
        let level = determine_ming_ge_level(90, 5, 0, 100);
        assert!(matches!(level, MingGeLevel::DiWang | MingGeLevel::JiGui));

        // 中等分数 + 一些吉格 = 中贵或大贵
        let level2 = determine_ming_ge_level(70, 2, 1, 20);
        assert!(matches!(level2, MingGeLevel::ZhongGui | MingGeLevel::DaGui));

        // 低分 + 凶格 = 普通
        assert_eq!(
            determine_ming_ge_level(40, 0, 3, -50),
            MingGeLevel::Putong
        );
    }

    #[test]
    fn test_calculate_wu_xing_distribution() {
        let mut palaces: [Palace; 12] = Default::default();

        // 添加一些主星
        palaces[0].zhu_xing = [Some(ZhuXing::ZiWei), None, None]; // 土
        palaces[0].zhu_xing_brightness = [StarBrightness::Miao, StarBrightness::Ping, StarBrightness::Ping];
        palaces[1].zhu_xing = [Some(ZhuXing::TaiYang), None, None]; // 火
        palaces[1].zhu_xing_brightness = [StarBrightness::Wang, StarBrightness::Ping, StarBrightness::Ping];

        let distribution = calculate_wu_xing_distribution(&palaces);

        // 土应该最高（紫微庙位100权重）
        // 火次之（太阳旺位80权重）
        assert!(distribution[4] > distribution[3]); // 土 > 火
    }

    #[test]
    fn test_generate_palace_interpretation() {
        let palace = create_test_palace();
        let interp = generate_palace_interpretation(&palace);

        assert_eq!(interp.gong_wei, GongWei::MingGong);
        assert_eq!(interp.score, 100);
        assert_eq!(interp.fortune_level, FortuneLevel::DaJi);
        assert_eq!(interp.star_strength, 100);
        assert_eq!(interp.si_hua_impact, 15);
        assert_eq!(interp.liu_ji_count, 2);
        assert_eq!(interp.liu_sha_count, 0);
    }

    #[test]
    fn test_generate_overall_score() {
        let palace_scores = [85, 75, 80, 70, 75, 65, 60, 55, 90, 65, 95, 75];
        let overall = generate_overall_score(&palace_scores, 0, 3, 1, 50);

        assert!(overall.overall_score >= 70);
        assert_eq!(overall.wealth_index, palace_scores[8]); // 财帛宫
        assert_eq!(overall.career_index, palace_scores[4]); // 官禄宫
        assert_eq!(overall.relationship_index, palace_scores[10]); // 夫妻宫
        // 命格等级与评分和格局相关
        assert!(!matches!(overall.ming_ge_level, MingGeLevel::Putong));
    }

    // ========================================================================
    // 补充测试：评分边界条件
    // ========================================================================

    #[test]
    fn test_calculate_star_strength_multiple_stars() {
        // 测试多颗主星的强度计算
        let mut palace = Palace::default();
        palace.zhu_xing = [Some(ZhuXing::ZiWei), Some(ZhuXing::TianFu), None];
        palace.zhu_xing_brightness = [StarBrightness::Miao, StarBrightness::Wang, StarBrightness::Ping];

        let strength = calculate_star_strength(&palace);
        // (100 + 80) / 2 = 90
        assert_eq!(strength, 90);
    }

    #[test]
    fn test_calculate_star_strength_three_stars() {
        // 测试三颗主星的强度计算
        let mut palace = Palace::default();
        palace.zhu_xing = [Some(ZhuXing::ZiWei), Some(ZhuXing::TianFu), Some(ZhuXing::TianJi)];
        palace.zhu_xing_brightness = [StarBrightness::Miao, StarBrightness::Wang, StarBrightness::De];

        let strength = calculate_star_strength(&palace);
        // (100 + 80 + 60) / 3 = 80
        assert_eq!(strength, 80);
    }

    #[test]
    fn test_calculate_star_strength_low_brightness() {
        // 测试落陷星曜
        let mut palace = Palace::default();
        palace.zhu_xing = [Some(ZhuXing::TanLang), None, None];
        palace.zhu_xing_brightness = [StarBrightness::Xian, StarBrightness::Ping, StarBrightness::Ping];

        let strength = calculate_star_strength(&palace);
        assert_eq!(strength, 10); // 陷位 = 10
    }

    #[test]
    fn test_calculate_si_hua_impact_multiple() {
        // 测试多个四化同宫
        let mut palace = Palace::default();
        palace.si_hua = [Some(SiHua::HuaLu), Some(SiHua::HuaQuan), Some(SiHua::HuaKe), None];

        let impact = calculate_si_hua_impact(&palace);
        // 化禄 15 + 化权 10 + 化科 8 = 33
        assert_eq!(impact, 33);
    }

    #[test]
    fn test_calculate_si_hua_impact_mixed() {
        // 测试吉凶混合四化
        let mut palace = Palace::default();
        palace.si_hua = [Some(SiHua::HuaLu), Some(SiHua::HuaJi), None, None];

        let impact = calculate_si_hua_impact(&palace);
        // 化禄 15 + 化忌 -20 = -5
        assert_eq!(impact, -5);
    }

    #[test]
    fn test_calculate_si_hua_impact_clamp() {
        // 测试四化影响钳制在 -50 ~ +50
        let mut palace = Palace::default();
        palace.si_hua = [Some(SiHua::HuaLu), Some(SiHua::HuaQuan), Some(SiHua::HuaKe), Some(SiHua::HuaLu)];

        let impact = calculate_si_hua_impact(&palace);
        // 实际上四化最多4个，都是吉化也不会超50，但测试钳制逻辑
        assert!(impact <= 50 && impact >= -50);
    }

    #[test]
    fn test_calculate_palace_score_all_liu_ji() {
        // 测试六吉星全满
        let mut palace = Palace::default();
        palace.zhu_xing = [Some(ZhuXing::ZiWei), None, None];
        palace.zhu_xing_brightness = [StarBrightness::Miao, StarBrightness::Ping, StarBrightness::Ping];
        palace.liu_ji = [true, true, true, true, true, true]; // 6颗六吉星

        let score = calculate_palace_score(&palace);
        // 基础 50 + 庙位 25 + 六吉 30 = 105 -> clamp to 100
        assert_eq!(score, 100);
    }

    #[test]
    fn test_calculate_palace_score_all_liu_sha() {
        // 测试六煞星全满
        let mut palace = Palace::default();
        palace.zhu_xing = [Some(ZhuXing::TanLang), None, None];
        palace.zhu_xing_brightness = [StarBrightness::Ping, StarBrightness::Ping, StarBrightness::Ping];
        palace.liu_sha = [true, true, true, true, true, true]; // 6颗六煞星

        let score = calculate_palace_score(&palace);
        // 基础 50 + 平 0 + 六煞 -30 = 20
        assert_eq!(score, 20);
    }

    #[test]
    fn test_calculate_palace_score_with_tian_ma() {
        // 测试天马加成
        let mut palace = Palace::default();
        palace.zhu_xing = [Some(ZhuXing::TianFu), None, None];
        palace.zhu_xing_brightness = [StarBrightness::Wang, StarBrightness::Ping, StarBrightness::Ping];
        palace.tian_ma = true;

        let score = calculate_palace_score(&palace);
        // 基础 50 + 旺位 20 + 天马 5 = 75
        assert_eq!(score, 75);
    }

    #[test]
    fn test_calculate_palace_score_empty_palace() {
        // 测试空宫评分
        let palace = Palace::default();
        let score = calculate_palace_score(&palace);
        // 基础 50 + 空宫惩罚 -10 = 40
        assert_eq!(score, 40);
    }

    #[test]
    fn test_calculate_palace_factors_all_flags() {
        // 测试所有标志位
        let mut palace = Palace::default();
        palace.zhu_xing = [Some(ZhuXing::ZiWei), None, None];
        palace.zhu_xing_brightness = [StarBrightness::Miao, StarBrightness::Ping, StarBrightness::Ping];
        palace.si_hua = [Some(SiHua::HuaLu), None, None, None];
        palace.liu_ji = [true, false, false, false, false, false];
        palace.liu_sha = [true, false, false, false, false, false];
        palace.lu_cun = true;
        palace.tian_ma = true;

        let factors = calculate_palace_factors(&palace);

        assert!(factors & 0b0000_0001 != 0); // 主星庙旺
        assert!(factors & 0b0000_0010 != 0); // 四化加持
        assert!(factors & 0b0000_0100 != 0); // 六吉会照
        assert!(factors & 0b0000_1000 != 0); // 六煞冲破
        assert!(factors & 0b0001_0000 == 0); // 非空宫
        assert!(factors & 0b0010_0000 != 0); // 有禄存
        assert!(factors & 0b0100_0000 != 0); // 有天马
    }

    #[test]
    fn test_calculate_palace_factors_empty_palace() {
        // 测试空宫标志
        let palace = Palace::default();
        let factors = calculate_palace_factors(&palace);

        assert!(factors & 0b0001_0000 != 0); // 空宫借星
        assert!(factors & 0b0000_0001 == 0); // 无主星庙旺
    }

    #[test]
    fn test_calculate_overall_score_different_ming_gong() {
        // 测试不同命宫位置
        let palace_scores = [60, 70, 80, 90, 50, 55, 65, 75, 85, 45, 95, 40];

        // 命宫在第0宫
        let overall1 = calculate_overall_score(&palace_scores, 0);
        // 命宫在第6宫
        let overall2 = calculate_overall_score(&palace_scores, 6);

        // 不同命宫位置应得到不同评分
        assert_ne!(overall1, overall2);
    }

    #[test]
    fn test_determine_ming_ge_level_all_levels() {
        // 测试所有命格等级边界
        // 计算公式：adjusted = overall + (吉格*10 - 凶格*10) + (格局分/5).clamp(-20,20)

        // DiWang: adjusted >= 100
        // 100 + 0 + 0 = 100
        assert_eq!(determine_ming_ge_level(100, 0, 0, 0), MingGeLevel::DiWang);

        // JiGui: adjusted 90-99
        // 90 + 0 + 0 = 90
        assert_eq!(determine_ming_ge_level(90, 0, 0, 0), MingGeLevel::JiGui);

        // DaGui: adjusted 80-89
        // 80 + 0 + 0 = 80
        assert_eq!(determine_ming_ge_level(80, 0, 0, 0), MingGeLevel::DaGui);

        // ZhongGui: adjusted 70-79
        // 70 + 0 + 0 = 70
        assert_eq!(determine_ming_ge_level(70, 0, 0, 0), MingGeLevel::ZhongGui);

        // XiaoGui: adjusted 55-69
        // 55 + 0 + 0 = 55
        assert_eq!(determine_ming_ge_level(55, 0, 0, 0), MingGeLevel::XiaoGui);

        // Putong: adjusted < 55
        // 40 + 0 + 0 = 40
        assert_eq!(determine_ming_ge_level(40, 0, 0, 0), MingGeLevel::Putong);

        // 测试格局加成影响
        // 50 + (2*10 - 0*10) + (40/5=8) = 50 + 20 + 8 = 78 -> ZhongGui
        assert_eq!(determine_ming_ge_level(50, 2, 0, 40), MingGeLevel::ZhongGui);

        // 测试凶格减分
        // 80 + (0*10 - 3*10) + 0 = 80 - 30 = 50 -> Putong
        assert_eq!(determine_ming_ge_level(80, 0, 3, 0), MingGeLevel::Putong);
    }

    #[test]
    fn test_calculate_wu_xing_distribution_all_elements() {
        // 测试五行分布完整覆盖
        let mut palaces: [Palace; 12] = Default::default();

        // 金：武曲
        palaces[0].zhu_xing = [Some(ZhuXing::WuQu), None, None];
        palaces[0].zhu_xing_brightness = [StarBrightness::Miao, StarBrightness::Ping, StarBrightness::Ping];

        // 木：天机
        palaces[1].zhu_xing = [Some(ZhuXing::TianJi), None, None];
        palaces[1].zhu_xing_brightness = [StarBrightness::Wang, StarBrightness::Ping, StarBrightness::Ping];

        // 水：太阴
        palaces[2].zhu_xing = [Some(ZhuXing::TaiYin), None, None];
        palaces[2].zhu_xing_brightness = [StarBrightness::De, StarBrightness::Ping, StarBrightness::Ping];

        // 火：太阳
        palaces[3].zhu_xing = [Some(ZhuXing::TaiYang), None, None];
        palaces[3].zhu_xing_brightness = [StarBrightness::Ping, StarBrightness::Ping, StarBrightness::Ping];

        // 土：紫微
        palaces[4].zhu_xing = [Some(ZhuXing::ZiWei), None, None];
        palaces[4].zhu_xing_brightness = [StarBrightness::BuDe, StarBrightness::Ping, StarBrightness::Ping];

        let distribution = calculate_wu_xing_distribution(&palaces);

        // 所有五行都应该有值
        assert!(distribution[0] > 0); // 金
        assert!(distribution[1] > 0); // 木
        assert!(distribution[2] > 0); // 水
        assert!(distribution[3] > 0); // 火
        assert!(distribution[4] > 0); // 土
    }

    #[test]
    fn test_calculate_wu_xing_distribution_empty() {
        // 测试空命盘的五行分布
        let palaces: [Palace; 12] = Default::default();
        let distribution = calculate_wu_xing_distribution(&palaces);

        // 空命盘应返回平均分布
        assert_eq!(distribution, [20, 20, 20, 20, 20]);
    }

    #[test]
    fn test_select_ming_gong_keywords_high_score() {
        let keywords = select_ming_gong_keywords(95, 90);
        // 高分应选择贵气相关关键词
        assert_eq!(keywords[0], 0);  // 贵气
        assert_eq!(keywords[1], 20); // 领导力强
        assert_eq!(keywords[2], 40); // 一生顺遂
    }

    #[test]
    fn test_select_ming_gong_keywords_low_score() {
        let keywords = select_ming_gong_keywords(15, 15);
        // 低分应选择保守相关关键词
        assert_eq!(keywords[0], 14); // 保守
        assert_eq!(keywords[1], 27); // 抗压力好
        assert_eq!(keywords[2], 41); // 早年辛劳
    }

    #[test]
    fn test_select_cai_bo_keywords() {
        let keywords = select_cai_bo_keywords(90);
        assert_eq!(keywords[0], 0);  // 财源广进
        assert_eq!(keywords[1], 10); // 适合经商
        assert_eq!(keywords[2], 40); // 宜投资
    }

    #[test]
    fn test_select_guan_lu_keywords() {
        let keywords = select_guan_lu_keywords(85);
        assert_eq!(keywords[0], 0);  // 事业有成
        assert_eq!(keywords[1], 20); // 领导能力强
        assert_eq!(keywords[2], 30); // 升职快
    }

    #[test]
    fn test_select_fu_qi_keywords() {
        let keywords = select_fu_qi_keywords(90);
        assert_eq!(keywords[0], 0);  // 感情顺遂
        assert_eq!(keywords[1], 10); // 配偶贤良
        assert_eq!(keywords[2], 48); // 宜沟通
    }

    #[test]
    fn test_select_ji_e_keywords() {
        let keywords = select_ji_e_keywords(80);
        assert_eq!(keywords[0], 1);  // 精力充沛
        assert_eq!(keywords[1], 20); // 作息规律
        assert_eq!(keywords[2], 40); // 宜运动
    }

    #[test]
    fn test_select_fu_de_keywords() {
        let keywords = select_fu_de_keywords(90);
        assert_eq!(keywords[0], 0);  // 心态平和
        assert_eq!(keywords[1], 30); // 福气深厚
        assert_eq!(keywords[2], 43); // 宜旅游
    }
}
