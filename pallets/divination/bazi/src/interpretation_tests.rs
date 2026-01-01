//! # 八字解盘测试模块
//!
//! 测试八字解盘的各种功能，包括：
//! - 格局分析测试
//! - 用神分析测试
//! - 性格分析测试
//! - 完整解盘流程测试

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpretation::*;
    use crate::types::*;
    use crate::mock::*;
    use frame_support::{assert_ok, assert_err};

    /// 创建测试用的八字数据
    fn create_test_bazi_chart() -> (SiZhu<Test>, WuXingStrength) {
        // 创建测试四柱：甲子年 丙寅月 戊午日 癸亥时
        let year_ganzhi = GanZhi::from_index(0).unwrap(); // 甲子
        let month_ganzhi = GanZhi::from_index(2).unwrap(); // 丙寅
        let day_ganzhi = GanZhi::from_index(54).unwrap(); // 戊午
        let hour_ganzhi = GanZhi::from_index(59).unwrap(); // 癸亥

        let rizhu = TianGan(4); // 戊土日主

        // 构建四柱（简化版，不包含完整的藏干信息）
        let sizhu = SiZhu {
            year_zhu: create_test_zhu(year_ganzhi),
            month_zhu: create_test_zhu(month_ganzhi),
            day_zhu: create_test_zhu(day_ganzhi),
            hour_zhu: create_test_zhu(hour_ganzhi),
            rizhu,
        };

        // 创建测试五行强度
        let wuxing_strength = WuXingStrength {
            jin: 100,
            mu: 200,
            shui: 150,
            huo: 300,
            tu: 250, // 戊土日主，土相对较强
        };

        (sizhu, wuxing_strength)
    }

    /// 创建测试用的柱
    fn create_test_zhu(ganzhi: GanZhi) -> Zhu<Test> {
        use frame_support::BoundedVec;
        
        Zhu {
            ganzhi,
            canggan: BoundedVec::default(),
            nayin: NaYin::HaiZhongJin, // 简化测试
        }
    }

    #[test]
    fn test_analyze_ge_ju() {
        let (sizhu, wuxing_strength) = create_test_bazi_chart();
        
        let ge_ju = analyze_ge_ju(&sizhu, &wuxing_strength);
        
        // 戊土日主，土强度250，总强度1000，占比25%，应该是正格
        assert_eq!(ge_ju, GeJuType::ZhengGe);
    }

    #[test]
    fn test_analyze_qiang_ruo() {
        let (sizhu, wuxing_strength) = create_test_bazi_chart();
        
        let qiang_ruo = analyze_qiang_ruo(&wuxing_strength, sizhu.rizhu);
        
        // 戊土日主，土强度250，总强度1000，占比25%，应该是身弱
        assert_eq!(qiang_ruo, MingJuQiangRuo::ShenRuo);
    }

    #[test]
    fn test_analyze_yong_shen() {
        let (sizhu, wuxing_strength) = create_test_bazi_chart();
        
        let ge_ju = GeJuType::ZhengGe;
        let qiang_ruo = MingJuQiangRuo::ShenRuo;
        
        let (yong_shen, yong_shen_type) = analyze_yong_shen(
            ge_ju,
            qiang_ruo,
            &sizhu,
            &wuxing_strength,
        );
        
        // 身弱的戊土，用神应该是火（生我者为印）
        assert_eq!(yong_shen, WuXing::Huo);
        assert_eq!(yong_shen_type, YongShenType::FuYi);
    }

    #[test]
    fn test_analyze_xing_ge() {
        let (sizhu, _) = create_test_bazi_chart();
        
        let xing_ge = analyze_xing_ge(&sizhu);
        
        // 戊土日主的性格特征
        assert!(xing_ge.zhu_yao_te_dian.contains(&"稳重"));
        assert!(xing_ge.you_dian.contains(&"可靠"));
        assert!(xing_ge.shi_he_zhi_ye.contains(&"房地产"));
    }

    #[test]
    fn test_full_interpretation() {
        let (sizhu, wuxing_strength) = create_test_bazi_chart();
        
        let result = full_interpretation(&sizhu, &wuxing_strength);
        
        // 验证解盘结果的基本结构
        assert_eq!(result.ge_ju, GeJuType::ZhengGe);
        assert_eq!(result.qiang_ruo, MingJuQiangRuo::ShenRuo);
        assert_eq!(result.yong_shen, WuXing::Huo);
        assert_eq!(result.yong_shen_type, YongShenType::FuYi);
        assert!(!result.ji_shen.is_empty());
        assert!(result.zong_he_ping_fen > 0 && result.zong_he_ping_fen <= 100);
        assert!(!result.jie_pan_text.is_empty());
    }

    #[test]
    fn test_cong_qiang_ge() {
        // 测试从强格的情况
        let mut wuxing_strength = WuXingStrength {
            jin: 50,
            mu: 100,
            shui: 50,
            huo: 100,
            tu: 700, // 土极强，占比70%
        };

        let sizhu = create_test_sizhu_for_tu();
        
        let ge_ju = analyze_ge_ju(&sizhu, &wuxing_strength);
        assert_eq!(ge_ju, GeJuType::CongQiangGe);
        
        let qiang_ruo = analyze_qiang_ruo(&wuxing_strength, TianGan(4)); // 戊土
        assert_eq!(qiang_ruo, MingJuQiangRuo::TaiWang);
    }

    #[test]
    fn test_cong_ruo_ge() {
        // 测试从弱格的情况
        let wuxing_strength = WuXingStrength {
            jin: 200,
            mu: 200,
            shui: 200,
            huo: 200,
            tu: 100, // 土极弱，占比10%
        };

        let sizhu = create_test_sizhu_for_tu();
        
        let ge_ju = analyze_ge_ju(&sizhu, &wuxing_strength);
        assert_eq!(ge_ju, GeJuType::CongRuoGe);
        
        let qiang_ruo = analyze_qiang_ruo(&wuxing_strength, TianGan(4)); // 戊土
        assert_eq!(qiang_ruo, MingJuQiangRuo::TaiRuo);
    }

    /// 创建土日主的测试四柱
    fn create_test_sizhu_for_tu() -> SiZhu<Test> {
        let ganzhi = GanZhi::from_index(54).unwrap(); // 戊午
        let zhu = create_test_zhu(ganzhi);
        
        SiZhu {
            year_zhu: zhu.clone(),
            month_zhu: zhu.clone(),
            day_zhu: zhu.clone(),
            hour_zhu: zhu,
            rizhu: TianGan(4), // 戊土
        }
    }

    #[test]
    fn test_different_rizhu_personalities() {
        // 测试不同日主的性格分析
        let test_cases = [
            (TianGan(0), "甲木", "正直"),      // 甲木
            (TianGan(1), "乙木", "温和"),      // 乙木
            (TianGan(2), "丙火", "热情"),      // 丙火
            (TianGan(3), "丁火", "细心"),      // 丁火
            (TianGan(6), "庚金", "果断"),      // 庚金
            (TianGan(8), "壬水", "智慧"),      // 壬水
        ];

        for (rizhu, _name, trait_word) in test_cases.iter() {
            let sizhu = create_sizhu_with_rizhu(*rizhu);
            let xing_ge = analyze_xing_ge(&sizhu);
            
            // 验证每个日主都有对应的性格特征
            assert!(!xing_ge.zhu_yao_te_dian.is_empty());
            assert!(!xing_ge.you_dian.is_empty());
            assert!(!xing_ge.que_dian.is_empty());
            assert!(!xing_ge.shi_he_zhi_ye.is_empty());
        }
    }

    /// 创建指定日主的四柱
    fn create_sizhu_with_rizhu(rizhu: TianGan) -> SiZhu<Test> {
        let ganzhi = GanZhi::from_index(0).unwrap(); // 甲子
        let zhu = create_test_zhu(ganzhi);
        
        SiZhu {
            year_zhu: zhu.clone(),
            month_zhu: zhu.clone(),
            day_zhu: zhu.clone(),
            hour_zhu: zhu,
            rizhu,
        }
    }

    #[test]
    fn test_comprehensive_score_calculation() {
        let test_cases = [
            // (格局, 强弱, 预期分数范围)
            (GeJuType::ZhengGe, MingJuQiangRuo::ZhongHe, 80..=100),
            (GeJuType::ZhengGe, MingJuQiangRuo::ShenWang, 70..=90),
            (GeJuType::CongQiangGe, MingJuQiangRuo::TaiWang, 65..=85),
            (GeJuType::CongRuoGe, MingJuQiangRuo::TaiRuo, 65..=85),
        ];

        for (ge_ju, qiang_ruo, expected_range) in test_cases.iter() {
            let wuxing_strength = WuXingStrength {
                jin: 200,
                mu: 200,
                shui: 200,
                huo: 200,
                tu: 200, // 平衡的五行分布
            };

            let score = calculate_comprehensive_score(ge_ju, qiang_ruo, &wuxing_strength);
            
            assert!(
                expected_range.contains(&score),
                "格局 {:?} + 强弱 {:?} 的分数 {} 不在预期范围 {:?} 内",
                ge_ju, qiang_ruo, score, expected_range
            );
        }
    }

    #[test]
    fn test_interpretation_text_generation() {
        let test_cases = [
            (GeJuType::ZhengGe, MingJuQiangRuo::ShenWang, WuXing::Jin),
            (GeJuType::CongQiangGe, MingJuQiangRuo::TaiWang, WuXing::Mu),
            (GeJuType::CongRuoGe, MingJuQiangRuo::TaiRuo, WuXing::Shui),
        ];

        for (ge_ju, qiang_ruo, yong_shen) in test_cases.iter() {
            let texts = generate_interpretation_text(ge_ju, qiang_ruo, yong_shen);
            
            // 验证生成的文本不为空且包含有意义的内容
            assert!(!texts.is_empty(), "解盘文本不应为空");
            assert!(texts.len() >= 3, "解盘文本应包含格局、强弱、用神三部分描述");
            
            // 验证文本内容的合理性
            let combined_text = texts.join(" ");
            assert!(combined_text.len() > 20, "解盘文本应有足够的长度");
        }
    }

    #[test]
    fn test_wuxing_relationships() {
        // 测试五行生克关系的正确性
        assert_eq!(get_sheng_wo(WuXing::Jin), WuXing::Tu);   // 土生金
        assert_eq!(get_sheng_wo(WuXing::Mu), WuXing::Shui);  // 水生木
        assert_eq!(get_sheng_wo(WuXing::Shui), WuXing::Jin); // 金生水
        assert_eq!(get_sheng_wo(WuXing::Huo), WuXing::Mu);   // 木生火
        assert_eq!(get_sheng_wo(WuXing::Tu), WuXing::Huo);   // 火生土

        assert_eq!(get_ke_wo(WuXing::Jin), WuXing::Huo);   // 火克金
        assert_eq!(get_ke_wo(WuXing::Mu), WuXing::Jin);    // 金克木
        assert_eq!(get_ke_wo(WuXing::Shui), WuXing::Tu);   // 土克水
        assert_eq!(get_ke_wo(WuXing::Huo), WuXing::Shui);  // 水克火
        assert_eq!(get_ke_wo(WuXing::Tu), WuXing::Mu);     // 木克土
    }

    #[test]
    fn test_edge_cases() {
        // 测试边界情况
        
        // 1. 五行强度全为0的情况
        let zero_strength = WuXingStrength {
            jin: 0, mu: 0, shui: 0, huo: 0, tu: 1, // 避免除零错误
        };
        
        let qiang_ruo = analyze_qiang_ruo(&zero_strength, TianGan(4)); // 戊土
        assert_eq!(qiang_ruo, MingJuQiangRuo::TaiWang); // 100%占比
        
        // 2. 五行强度相等的情况
        let equal_strength = WuXingStrength {
            jin: 200, mu: 200, shui: 200, huo: 200, tu: 200,
        };
        
        let qiang_ruo_equal = analyze_qiang_ruo(&equal_strength, TianGan(4)); // 戊土
        assert_eq!(qiang_ruo_equal, MingJuQiangRuo::ShenRuo); // 20%占比
    }
}
