use crate::types::{LiuGong, SanGong, ShiChen, TiYongRelation, WuXingRelation, BaGua, XiaoLiuRenSchool};
use super::enums::*;
use super::core_struct::XiaoLiuRenInterpretation;

// ============================================================================
// 吉凶等级计算
// ============================================================================

/// 计算吉凶等级
///
/// 综合考虑：
/// 1. 时宫（结果）的吉凶等级（权重60%）
/// 2. 三宫整体平均等级（权重40%）
/// 3. 特殊格局加成/减分
/// 4. 体用关系影响
pub fn calculate_ji_xiong_level(
    san_gong: &SanGong,
    shi_chen: Option<ShiChen>,
) -> JiXiongLevel {
    // 1. 基础分数（1-5）
    let base_score = san_gong.fortune_level() as i8;

    // 2. 特殊格局调整（-2 到 +2）
    let pattern_modifier = if san_gong.is_pure() {
        // 纯宫：吉更吉，凶更凶
        if san_gong.shi_gong.is_auspicious() { 2 } else { -2 }
    } else if san_gong.is_all_auspicious() {
        // 全吉
        1
    } else if san_gong.is_all_inauspicious() {
        // 全凶
        -1
    } else {
        0
    };

    // 3. 体用关系调整（-2 到 +1）
    let ti_yong_modifier = if let Some(sc) = shi_chen {
        let ti_yong = TiYongRelation::calculate(san_gong.shi_gong, sc);
        match ti_yong {
            TiYongRelation::YongShengTi => 1,  // 大吉
            TiYongRelation::TiKeYong => 0,     // 小吉
            TiYongRelation::BiJian | TiYongRelation::BiZhu => 0, // 中平
            TiYongRelation::TiShengYong => -1, // 小凶
            TiYongRelation::YongKeTi => -2,    // 大凶
        }
    } else {
        0
    };

    // 4. 计算最终分数（限制在1-7范围）
    let final_score = (base_score + pattern_modifier + ti_yong_modifier).clamp(1, 7);

    // 5. 转换为吉凶等级
    match final_score {
        7 => JiXiongLevel::DaJi,
        6 => JiXiongLevel::Ji,
        5 => JiXiongLevel::XiaoJi,
        4 => JiXiongLevel::Ping,
        3 => JiXiongLevel::XiaoXiong,
        2 => JiXiongLevel::Xiong,
        _ => JiXiongLevel::DaXiong,
    }
}

// ============================================================================
// 综合评分计算
// ============================================================================

/// 计算综合评分（0-100分）
///
/// 评分维度：
/// 1. 时宫吉凶（40分）
/// 2. 三宫整体（20分）
/// 3. 五行关系（20分）
/// 4. 体用关系（10分）
/// 5. 特殊格局（10分）
pub fn calculate_overall_score(
    san_gong: &SanGong,
    shi_chen: Option<ShiChen>,
) -> u8 {
    // 1. 时宫得分（0-40）
    let shi_score = (san_gong.shi_gong.fortune_level() as u16 * 8) as u8;

    // 2. 三宫整体得分（0-20）
    let san_gong_score = (san_gong.fortune_level() as u16 * 4) as u8;

    // 3. 五行关系得分（0-20）
    let wu_xing_score = match san_gong.wu_xing_analysis() {
        WuXingRelation::Sheng => 20,    // 相生
        WuXingRelation::BiHe => 15,     // 比和
        WuXingRelation::XieSheng => 10, // 泄气
        WuXingRelation::Ke => 5,        // 相克
        WuXingRelation::BeiKe => 0,     // 被克
    };

    // 4. 体用关系得分（0-10）
    let ti_yong_score = if let Some(sc) = shi_chen {
        let ti_yong = TiYongRelation::calculate(san_gong.shi_gong, sc);
        match ti_yong {
            TiYongRelation::YongShengTi => 10, // 大吉
            TiYongRelation::TiKeYong => 8,     // 小吉
            TiYongRelation::BiJian => 6,       // 比肩
            TiYongRelation::BiZhu => 5,        // 比助
            TiYongRelation::TiShengYong => 3,  // 小凶
            TiYongRelation::YongKeTi => 0,     // 大凶
        }
    } else {
        5 // 无时辰信息，给予中性分数
    };

    // 5. 特殊格局得分（0-10）
    let pattern_score = if san_gong.is_pure() {
        if san_gong.shi_gong.is_auspicious() { 10 } else { 0 }
    } else if san_gong.is_all_auspicious() {
        10
    } else if san_gong.is_all_inauspicious() {
        0
    } else {
        5
    };

    // 汇总得分（0-100）
    let total = shi_score + san_gong_score + wu_xing_score + ti_yong_score + pattern_score;
    total.min(100)
}

// ============================================================================
// 特殊格局识别
// ============================================================================

/// 识别特殊格局
pub fn identify_special_pattern(
    san_gong: &SanGong,
    shi_chen: Option<ShiChen>,
) -> SpecialPattern {
    let mut pattern = SpecialPattern::new();

    // 1. 检查纯宫
    if san_gong.is_pure() {
        pattern.set_pure();
    }

    // 2. 检查全吉/全凶
    if san_gong.is_all_auspicious() {
        pattern.set_all_auspicious();
    } else if san_gong.is_all_inauspicious() {
        pattern.set_all_inauspicious();
    }

    // 3. 检查五行成环
    let wx1 = san_gong.yue_gong.wu_xing();
    let wx2 = san_gong.ri_gong.wu_xing();
    let wx3 = san_gong.shi_gong.wu_xing();

    // 相生成环：木→火→土 或 火→土→金 等
    if wx1.generates() == wx2 && wx2.generates() == wx3 && wx3.generates() == wx1 {
        pattern.set_sheng_cycle();
    }

    // 相克成环：木→土→水 或 土→水→火 等
    if wx1.restrains() == wx2 && wx2.restrains() == wx3 && wx3.restrains() == wx1 {
        pattern.set_ke_cycle();
    }

    // 4. 检查阴阳和合（体用阴阳互补）
    if let Some(sc) = shi_chen {
        let ti_yy = san_gong.shi_gong.yin_yang();
        let yong_yy = sc.yin_yang();
        if ti_yy != yong_yy {
            pattern.set_yin_yang_harmony();
        }
    }

    // 5. 检查特殊时辰（子午卯酉四正时）
    if let Some(sc) = shi_chen {
        if matches!(sc, ShiChen::Zi | ShiChen::Wu | ShiChen::Mao | ShiChen::You) {
            pattern.set_special_time();
        }
    }

    pattern
}

// ============================================================================
// 应期计算
// ============================================================================

/// 计算应期类型
///
/// 主要根据时宫（结果）判断：
/// - 速喜 → 即刻
/// - 大安、小吉 → 当日
/// - 留连 → 延迟
/// - 空亡 → 难以应验
/// - 赤口 → 需要化解
pub fn calculate_ying_qi(san_gong: &SanGong) -> Option<YingQiType> {
    let ying_qi = match san_gong.shi_gong {
        LiuGong::SuXi => YingQiType::JiKe,        // 速喜 - 即刻
        LiuGong::DaAn => YingQiType::DangRi,      // 大安 - 当日
        LiuGong::XiaoJi => YingQiType::DangRi,    // 小吉 - 当日
        LiuGong::LiuLian => YingQiType::YanChi,   // 留连 - 延迟
        LiuGong::KongWang => YingQiType::NanYi,   // 空亡 - 难以应验
        LiuGong::ChiKou => YingQiType::XuHuaJie,  // 赤口 - 需要化解
    };

    Some(ying_qi)
}

// ============================================================================
// 建议类型确定
// ============================================================================

/// 确定建议类型
///
/// 综合考虑吉凶等级和五行关系
pub fn determine_advice_type(
    ji_xiong_level: &JiXiongLevel,
    wu_xing_relation: &WuXingRelation,
) -> AdviceType {
    // 主要根据吉凶等级
    let base_advice = match ji_xiong_level {
        JiXiongLevel::DaJi => AdviceType::JinQu,
        JiXiongLevel::Ji => AdviceType::WenBu,
        JiXiongLevel::XiaoJi => AdviceType::WenBu,
        JiXiongLevel::Ping => AdviceType::ShouCheng,
        JiXiongLevel::XiaoXiong => AdviceType::GuanWang,
        JiXiongLevel::Xiong => AdviceType::TuiShou,
        JiXiongLevel::DaXiong => AdviceType::JingDai,
    };

    // 五行关系特别不利时，建议化解
    if matches!(wu_xing_relation, WuXingRelation::BeiKe | WuXingRelation::Ke) {
        if ji_xiong_level.is_xiong() {
            return AdviceType::HuaJie;
        }
    }

    base_advice
}

// ============================================================================
// 核心解卦函数
// ============================================================================

/// 解卦核心算法
///
/// 根据三宫结果、时辰、流派计算解卦数据
pub fn interpret(
    san_gong: &SanGong,
    shi_chen: Option<ShiChen>,
    school: XiaoLiuRenSchool,
) -> XiaoLiuRenInterpretation {
    // 1. 计算吉凶等级
    let ji_xiong_level = calculate_ji_xiong_level(san_gong, shi_chen);

    // 2. 计算综合评分
    let overall_score = calculate_overall_score(san_gong, shi_chen);

    // 3. 五行关系分析
    let wu_xing_relation = san_gong.wu_xing_analysis();

    // 4. 体用关系分析（如果有时辰）
    let ti_yong_relation = shi_chen.map(|sc| {
        TiYongRelation::calculate(san_gong.shi_gong, sc)
    });

    // 5. 八卦具象分析
    let ba_gua = Some(BaGua::from_san_gong(san_gong));

    // 6. 特殊格局识别
    let special_pattern = identify_special_pattern(san_gong, shi_chen);

    // 7. 建议类型
    let advice_type = determine_advice_type(&ji_xiong_level, &wu_xing_relation);

    // 8. 应期推算
    let ying_qi = calculate_ying_qi(san_gong);

    // 9. 构建解卦结果
    XiaoLiuRenInterpretation::new(
        ji_xiong_level,
        overall_score,
        wu_xing_relation,
        ti_yong_relation,
        ba_gua,
        special_pattern,
        advice_type,
        school,
        ying_qi,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_ji_xiong_level_all_auspicious() {
        // 全吉：大安、速喜、小吉
        let san_gong = SanGong::new(LiuGong::DaAn, LiuGong::SuXi, LiuGong::XiaoJi);
        let result = calculate_ji_xiong_level(&san_gong, None);
        assert!(result.is_ji());
    }

    #[test]
    fn test_calculate_ji_xiong_level_all_inauspicious() {
        // 全凶：留连、赤口、空亡
        let san_gong = SanGong::new(LiuGong::LiuLian, LiuGong::ChiKou, LiuGong::KongWang);
        let result = calculate_ji_xiong_level(&san_gong, None);
        assert!(result.is_xiong());
    }

    #[test]
    fn test_calculate_ji_xiong_level_pure_auspicious() {
        // 纯宫吉：大安、大安、大安
        let san_gong = SanGong::new(LiuGong::DaAn, LiuGong::DaAn, LiuGong::DaAn);
        let result = calculate_ji_xiong_level(&san_gong, None);
        assert_eq!(result, JiXiongLevel::DaJi);
    }

    #[test]
    fn test_calculate_overall_score() {
        let san_gong = SanGong::new(LiuGong::DaAn, LiuGong::SuXi, LiuGong::XiaoJi);
        let score = calculate_overall_score(&san_gong, None);
        assert!(score > 50); // 全吉应该有较高分
        assert!(score <= 100);
    }

    #[test]
    fn test_identify_special_pattern() {
        let san_gong = SanGong::new(LiuGong::DaAn, LiuGong::DaAn, LiuGong::DaAn);
        let pattern = identify_special_pattern(&san_gong, None);
        assert!(pattern.is_pure());
    }

    #[test]
    fn test_calculate_ying_qi() {
        let san_gong = SanGong::new(LiuGong::SuXi, LiuGong::DaAn, LiuGong::SuXi);
        let ying_qi = calculate_ying_qi(&san_gong);
        assert_eq!(ying_qi, Some(YingQiType::JiKe));
    }

    #[test]
    fn test_interpret_full() {
        let san_gong = SanGong::new(LiuGong::DaAn, LiuGong::SuXi, LiuGong::XiaoJi);
        let shi_chen = Some(ShiChen::Zi);
        let school = XiaoLiuRenSchool::DaoJia;

        let result = interpret(&san_gong, shi_chen, school);

        assert!(result.is_ji());
        assert!(result.overall_score > 50);
        assert_eq!(result.school, school);
        assert!(result.ba_gua.is_some());
        assert!(result.ying_qi.is_some());
    }

    #[test]
    fn test_interpret_no_shichen() {
        let san_gong = SanGong::new(LiuGong::DaAn, LiuGong::LiuLian, LiuGong::ChiKou);
        let result = interpret(&san_gong, None, XiaoLiuRenSchool::DaoJia);

        assert!(result.ti_yong_relation.is_none());
        assert!(result.overall_score > 0);
    }
}
