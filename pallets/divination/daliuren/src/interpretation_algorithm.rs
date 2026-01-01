//! # 大六壬解盘算法实现模块
//!
//! 本模块实现大六壬解盘的核心算法，包括：
//! - 三传分析：旺衰、空亡、递生递克
//! - 四课分析：克关系、日干日支状态
//! - 天将分析：吉凶将统计
//! - 神煞分析：吉凶神煞提取
//! - 应期推算：多种应期计算方法
//! - 综合吉凶判断
//!
//! ## 解盘流程
//!
//! ```text
//! 式盘数据 → 三传分析 → 四课分析 → 天将分析 → 神煞分析
//!              ↓
//!         应期推算 → 综合吉凶 → CoreInterpretation
//! ```

use crate::interpretation::*;
use crate::types::*;
use frame_support::BoundedVec;

// ============================================================================
// 核心解盘计算
// ============================================================================

/// 计算核心解盘结果
///
/// 根据式盘数据计算核心解盘指标
///
/// # 参数
/// - `pan`: 大六壬式盘数据
/// - `current_block`: 当前区块号
///
/// # 返回
/// - `CoreInterpretation`: 核心解盘结果
pub fn calculate_core_interpretation<AccountId, BlockNumber, MaxCidLen: frame_support::traits::Get<u32>>(
    pan: &DaLiuRenPan<AccountId, BlockNumber, MaxCidLen>,
    current_block: u32,
) -> CoreInterpretation {
    // 检查是否有计算数据
    // 如果没有（Private 模式），返回默认解盘结果
    if !pan.has_calculation_data() {
        return CoreInterpretation::default_with_timestamp(current_block);
    }

    // 解包所有必需的 Option 字段
    let san_chuan = pan.san_chuan.as_ref().unwrap();
    let tian_jiang_pan = pan.tian_jiang_pan.as_ref().unwrap();
    let si_ke = pan.si_ke.as_ref().unwrap();
    let day_gz = pan.day_gz.unwrap();
    let month_gz = pan.month_gz.unwrap();
    let xun_kong = pan.xun_kong.unwrap();
    let ke_shi = pan.ke_shi.unwrap_or_default();
    let ge_ju = pan.ge_ju.unwrap_or_default();

    // 获取月令五行（用于旺衰判断）
    let month_wuxing = month_gz.1.wu_xing();

    // 1. 分析三传
    let san_chuan_analysis = analyze_san_chuan(
        san_chuan,
        tian_jiang_pan,
        day_gz.0,
        month_wuxing,
        xun_kong,
    );

    // 2. 分析四课
    let si_ke_analysis = analyze_si_ke(
        si_ke,
        day_gz.0,
        day_gz.1,
        month_wuxing,
    );

    // 3. 分析天将
    let tian_jiang_analysis = analyze_tian_jiang(
        tian_jiang_pan,
        san_chuan,
        xun_kong,
    );

    // 4. 计算应期
    let (ying_qi_num, ying_qi_unit, secondary_ying_qi) = calculate_ying_qi(
        san_chuan,
        xun_kong,
    );

    // 5. 综合吉凶判断
    let fortune = calculate_fortune(
        &san_chuan_analysis,
        &si_ke_analysis,
        &tian_jiang_analysis,
        ke_shi,
        ge_ju,
    );

    // 6. 判断趋势
    let trend = calculate_trend(&san_chuan_analysis);

    // 7. 判断成败
    let outcome = calculate_outcome(
        &san_chuan_analysis,
        &tian_jiang_analysis,
        fortune,
    );

    // 8. 计算综合评分
    let score = fortune.to_score();

    // 9. 计算可信度
    let confidence = calculate_confidence(
        ke_shi,
        &san_chuan_analysis,
        &tian_jiang_analysis,
    );

    // 10. 应期可信度
    let ying_qi_confidence = calculate_ying_qi_confidence(
        &san_chuan_analysis,
        xun_kong,
    );

    CoreInterpretation {
        ke_shi,
        ge_ju,
        fortune,
        trend,
        outcome,
        primary_lei_shen: san_chuan.chu,
        primary_wang_shuai: san_chuan_analysis.chu_wang_shuai,
        primary_liu_qin: san_chuan.chu_qin,
        primary_jiang_ji: san_chuan_analysis.chu_jiang_ji,
        ying_qi_num,
        ying_qi_unit,
        secondary_ying_qi,
        ying_qi_confidence,
        score,
        confidence,
        timestamp: current_block,
    }
}

/// 计算完整解盘结果
///
/// 包含所有扩展分析数据
pub fn calculate_full_interpretation<AccountId, BlockNumber, MaxCidLen: frame_support::traits::Get<u32>>(
    pan: &DaLiuRenPan<AccountId, BlockNumber, MaxCidLen>,
    current_block: u32,
    shi_xiang_type: Option<ShiXiangType>,
) -> FullInterpretation {
    // 检查是否有计算数据
    if !pan.has_calculation_data() {
        return FullInterpretation::default_with_timestamp(current_block);
    }

    // 解包必需字段
    let san_chuan = pan.san_chuan.as_ref().unwrap();
    let tian_jiang_pan = pan.tian_jiang_pan.as_ref().unwrap();
    let si_ke = pan.si_ke.as_ref().unwrap();
    let day_gz = pan.day_gz.unwrap();
    let month_gz = pan.month_gz.unwrap();
    let xun_kong = pan.xun_kong.unwrap();

    let month_wuxing = month_gz.1.wu_xing();

    // 计算各项分析
    let san_chuan_analysis = analyze_san_chuan(
        san_chuan,
        tian_jiang_pan,
        day_gz.0,
        month_wuxing,
        xun_kong,
    );

    let si_ke_analysis = analyze_si_ke(
        si_ke,
        day_gz.0,
        day_gz.1,
        month_wuxing,
    );

    let tian_jiang_analysis = analyze_tian_jiang(
        tian_jiang_pan,
        san_chuan,
        xun_kong,
    );

    let shen_sha_analysis = analyze_shen_sha(
        san_chuan,
        day_gz,
        &san_chuan_analysis,
    );

    let ying_qi_analysis = calculate_ying_qi_analysis(
        san_chuan,
        xun_kong,
        shi_xiang_type,
    );

    // 计算核心解盘
    let core = calculate_core_interpretation(pan, current_block);

    // 生成事象断语
    let shi_xiang_hints = shi_xiang_type.map(|t| generate_shi_xiang_hints(
        t,
        &core,
        &san_chuan_analysis,
        &tian_jiang_analysis,
    ));

    FullInterpretation {
        core,
        san_chuan_analysis,
        si_ke_analysis,
        tian_jiang_analysis,
        shen_sha_analysis,
        ying_qi_analysis,
        shi_xiang_hints,
    }
}

// ============================================================================
// 三传分析
// ============================================================================

/// 分析三传
pub fn analyze_san_chuan(
    san_chuan: &SanChuan,
    _tian_jiang_pan: &TianJiangPan,
    _day_gan: TianGan,
    month_wuxing: WuXing,
    xun_kong: (DiZhi, DiZhi),
) -> SanChuanAnalysis {
    // 计算各传旺衰
    let chu_wang_shuai = WangShuai::from_wu_xing(month_wuxing, san_chuan.chu.wu_xing());
    let zhong_wang_shuai = WangShuai::from_wu_xing(month_wuxing, san_chuan.zhong.wu_xing());
    let mo_wang_shuai = WangShuai::from_wu_xing(month_wuxing, san_chuan.mo.wu_xing());

    // 判断天将吉凶
    let chu_jiang_ji = san_chuan.chu_jiang.is_auspicious();
    let zhong_jiang_ji = san_chuan.zhong_jiang.is_auspicious();
    let mo_jiang_ji = san_chuan.mo_jiang.is_auspicious();

    // 判断空亡
    let chu_kong = san_chuan.chu == xun_kong.0 || san_chuan.chu == xun_kong.1;
    let zhong_kong = san_chuan.zhong == xun_kong.0 || san_chuan.zhong == xun_kong.1;
    let mo_kong = san_chuan.mo == xun_kong.0 || san_chuan.mo == xun_kong.1;

    // 判断三传关系
    let (di_sheng, di_ke, lian_ru) = analyze_san_chuan_relation(san_chuan);

    SanChuanAnalysis {
        chu_wang_shuai,
        chu_jiang_ji,
        chu_kong,
        zhong_wang_shuai,
        zhong_jiang_ji,
        zhong_kong,
        mo_wang_shuai,
        mo_jiang_ji,
        mo_kong,
        di_sheng,
        di_ke,
        lian_ru,
    }
}

/// 分析三传关系
fn analyze_san_chuan_relation(san_chuan: &SanChuan) -> (bool, bool, bool) {
    let chu_wx = san_chuan.chu.wu_xing();
    let zhong_wx = san_chuan.zhong.wu_xing();
    let mo_wx = san_chuan.mo.wu_xing();

    // 递生：初生中，中生末
    let di_sheng = chu_wx.sheng(zhong_wx) && zhong_wx.sheng(mo_wx);

    // 递克：初克中，中克末
    let di_ke = chu_wx.ke(zhong_wx) && zhong_wx.ke(mo_wx);

    // 连茹：三支相连（如寅卯辰、巳午未等）
    let chu_idx = san_chuan.chu.index();
    let zhong_idx = san_chuan.zhong.index();
    let mo_idx = san_chuan.mo.index();

    let lian_ru = (zhong_idx == (chu_idx + 1) % 12 && mo_idx == (zhong_idx + 1) % 12)
        || (zhong_idx == (chu_idx + 11) % 12 && mo_idx == (zhong_idx + 11) % 12);

    (di_sheng, di_ke, lian_ru)
}

// ============================================================================
// 四课分析
// ============================================================================

/// 分析四课
fn analyze_si_ke(
    si_ke: &SiKe,
    day_gan: TianGan,
    day_zhi: DiZhi,
    month_wuxing: WuXing,
) -> SiKeAnalysis {
    let day_gan_wx = day_gan.wu_xing();
    let day_zhi_wx = day_zhi.wu_xing();

    // 干阳神旺衰
    let gan_yang_wang_shuai = WangShuai::from_wu_xing(month_wuxing, si_ke.ke1.shang.wu_xing());

    // 支阳神旺衰
    let zhi_yang_wang_shuai = WangShuai::from_wu_xing(month_wuxing, si_ke.ke3.shang.wu_xing());

    // 日干得助判断（是否有比肩或印星）
    let ri_gan_you_zhu = [&si_ke.ke1, &si_ke.ke2, &si_ke.ke3, &si_ke.ke4]
        .iter()
        .any(|ke| {
            let shang_wx = ke.shang.wu_xing();
            shang_wx == day_gan_wx || shang_wx.sheng(day_gan_wx)
        });

    // 日支得生判断
    let ri_zhi_you_sheng = [&si_ke.ke1, &si_ke.ke2, &si_ke.ke3, &si_ke.ke4]
        .iter()
        .any(|ke| {
            let shang_wx = ke.shang.wu_xing();
            shang_wx.sheng(day_zhi_wx)
        });

    // 统计上克下和下克上数量
    let mut shang_ke_xia_count = 0u8;
    let mut xia_ke_shang_count = 0u8;

    for ke in [&si_ke.ke1, &si_ke.ke2, &si_ke.ke3, &si_ke.ke4] {
        let shang_wx = ke.shang.wu_xing();
        let xia_wx = ke.xia.wu_xing();

        if shang_wx.ke(xia_wx) {
            shang_ke_xia_count += 1;
        }
        if xia_wx.ke(shang_wx) {
            xia_ke_shang_count += 1;
        }
    }

    // 干支关系
    let gan_zhi_he = day_zhi == day_gan_ji_gong(day_gan).liu_he();
    let gan_zhi_chong = day_zhi == day_gan_ji_gong(day_gan).liu_chong();

    SiKeAnalysis {
        ri_gan_you_zhu,
        gan_yang_wang_shuai,
        ri_zhi_you_sheng,
        zhi_yang_wang_shuai,
        shang_ke_xia_count,
        xia_ke_shang_count,
        gan_zhi_he,
        gan_zhi_chong,
    }
}

/// 获取天干寄宫
fn day_gan_ji_gong(gan: TianGan) -> DiZhi {
    get_ji_gong(gan)
}

// ============================================================================
// 天将分析
// ============================================================================

/// 分析天将
fn analyze_tian_jiang(
    tian_jiang_pan: &TianJiangPan,
    san_chuan: &SanChuan,
    xun_kong: (DiZhi, DiZhi),
) -> TianJiangAnalysis {
    // 找贵人位置
    let mut gui_ren_lin = DiZhi::Zi;
    for i in 0..12 {
        if tian_jiang_pan.positions[i] == TianJiang::GuiRen {
            gui_ren_lin = DiZhi::from_index(i as u8);
            break;
        }
    }

    // 贵人是否空亡
    let gui_ren_kong = gui_ren_lin == xun_kong.0 || gui_ren_lin == xun_kong.1;

    // 贵人是否入墓（土入辰戌丑未墓）
    let gui_ren_mu = matches!(gui_ren_lin, DiZhi::Chen | DiZhi::Xu | DiZhi::Chou | DiZhi::Wei);

    // 找青龙和白虎位置
    let mut qing_long_lin = DiZhi::Zi;
    let mut bai_hu_lin = DiZhi::Zi;
    for i in 0..12 {
        match tian_jiang_pan.positions[i] {
            TianJiang::QingLong => qing_long_lin = DiZhi::from_index(i as u8),
            TianJiang::BaiHu => bai_hu_lin = DiZhi::from_index(i as u8),
            _ => {}
        }
    }

    // 统计吉凶将数量
    let mut ji_jiang_count = 0u8;
    let mut xiong_jiang_count = 0u8;
    for jiang in &tian_jiang_pan.positions {
        if jiang.is_auspicious() {
            ji_jiang_count += 1;
        } else {
            xiong_jiang_count += 1;
        }
    }

    // 三传天将吉凶统计
    let san_chuan_ji_jiang = [
        san_chuan.chu_jiang,
        san_chuan.zhong_jiang,
        san_chuan.mo_jiang,
    ]
    .iter()
    .filter(|j| j.is_auspicious())
    .count() as u8;

    TianJiangAnalysis {
        gui_ren_lin,
        gui_ren_kong,
        gui_ren_mu,
        qing_long_lin,
        bai_hu_lin,
        ji_jiang_count,
        xiong_jiang_count,
        san_chuan_ji_jiang,
    }
}

// ============================================================================
// 神煞分析
// ============================================================================

/// 分析神煞
fn analyze_shen_sha(
    san_chuan: &SanChuan,
    day_gz: (TianGan, DiZhi),
    san_chuan_analysis: &SanChuanAnalysis,
) -> ShenShaAnalysis {
    let mut ji_shen_sha = BoundedVec::new();
    let mut xiong_shen_sha = BoundedVec::new();

    // 检查驿马入传
    let yi_ma = day_gz.1.yi_ma();
    let yi_ma_ru_chuan = san_chuan.chu == yi_ma
        || san_chuan.zhong == yi_ma
        || san_chuan.mo == yi_ma;

    if yi_ma_ru_chuan {
        let _ = ji_shen_sha.try_push(ShenShaType::YiMa);
    }

    // 检查天罗地网
    // 天罗：辰戌，地网：丑未（简化判断）
    let tian_luo_di_wang = [san_chuan.chu, san_chuan.zhong, san_chuan.mo]
        .iter()
        .any(|zhi| matches!(zhi, DiZhi::Chen | DiZhi::Xu | DiZhi::Chou | DiZhi::Wei));

    if tian_luo_di_wang {
        let _ = xiong_shen_sha.try_push(ShenShaType::TianLuo);
        let _ = xiong_shen_sha.try_push(ShenShaType::DiWang);
    }

    // 检查三刑入传
    let san_xing_ru_chuan = check_san_xing(san_chuan);

    // 检查六害入传
    let liu_hai_ru_chuan = check_liu_hai(san_chuan, day_gz.1);

    // 添加基本吉神
    if !san_chuan_analysis.chu_kong {
        let _ = ji_shen_sha.try_push(ShenShaType::TianYiGuiRen);
    }

    // 添加基本凶神
    if san_chuan_analysis.di_ke {
        let _ = xiong_shen_sha.try_push(ShenShaType::TianGui);
    }

    ShenShaAnalysis {
        ji_shen_sha,
        xiong_shen_sha,
        yi_ma_ru_chuan,
        tian_luo_di_wang,
        liu_hai_ru_chuan,
        san_xing_ru_chuan,
    }
}

/// 检查三刑
fn check_san_xing(san_chuan: &SanChuan) -> bool {
    let chuans = [san_chuan.chu, san_chuan.zhong, san_chuan.mo];

    // 寅巳申三刑
    let yin_si_shen = chuans.contains(&DiZhi::Yin)
        && chuans.contains(&DiZhi::Si)
        && chuans.contains(&DiZhi::Shen);

    // 丑戌未三刑
    let chou_xu_wei = chuans.contains(&DiZhi::Chou)
        && chuans.contains(&DiZhi::Xu)
        && chuans.contains(&DiZhi::Wei);

    // 子卯刑
    let zi_mao = chuans.contains(&DiZhi::Zi) && chuans.contains(&DiZhi::Mao);

    yin_si_shen || chou_xu_wei || zi_mao
}

/// 检查六害
fn check_liu_hai(san_chuan: &SanChuan, day_zhi: DiZhi) -> bool {
    // 六害：子未、丑午、寅巳、卯辰、申亥、酉戌
    let liu_hai_pairs: [(DiZhi, DiZhi); 6] = [
        (DiZhi::Zi, DiZhi::Wei),
        (DiZhi::Chou, DiZhi::Wu),
        (DiZhi::Yin, DiZhi::Si),
        (DiZhi::Mao, DiZhi::Chen),
        (DiZhi::Shen, DiZhi::Hai),
        (DiZhi::You, DiZhi::Xu),
    ];

    for chuan in [san_chuan.chu, san_chuan.zhong, san_chuan.mo] {
        for (a, b) in &liu_hai_pairs {
            if (day_zhi == *a && chuan == *b) || (day_zhi == *b && chuan == *a) {
                return true;
            }
        }
    }

    false
}

// ============================================================================
// 应期计算
// ============================================================================

/// 计算应期（简化版）
fn calculate_ying_qi(
    san_chuan: &SanChuan,
    xun_kong: (DiZhi, DiZhi),
) -> (u8, YingQiUnit, DiZhi) {
    // 三传相加法：初传数 + 中传数 + 末传数
    let total = (san_chuan.chu.index() + 1) as u16
        + (san_chuan.zhong.index() + 1) as u16
        + (san_chuan.mo.index() + 1) as u16;

    // 确定应期单位
    let (num, unit) = if total <= 12 {
        (total as u8, YingQiUnit::Ri)
    } else if total <= 30 {
        (total as u8, YingQiUnit::Ri)
    } else if total <= 120 {
        ((total / 10) as u8, YingQiUnit::Xun)
    } else {
        ((total / 30) as u8, YingQiUnit::Yue)
    };

    // 次应期：空亡填实
    let secondary = if xun_kong.0 != DiZhi::Zi {
        xun_kong.0
    } else {
        xun_kong.1
    };

    (num, unit, secondary)
}

/// 计算详细应期分析
pub fn calculate_ying_qi_analysis(
    san_chuan: &SanChuan,
    xun_kong: (DiZhi, DiZhi),
    _shi_xiang_type: Option<ShiXiangType>,
) -> YingQiAnalysis {
    let (num, unit, _secondary_zhi) = calculate_ying_qi(san_chuan, xun_kong);

    let primary = YingQiResult {
        num,
        unit,
        zhi: san_chuan.mo, // 末传为最终应期参考
        method: YingQiMethod::SanChuanXiangJia,
    };

    // 空亡填实应期
    let secondary = Some(YingQiResult {
        num: xun_kong.0.index() + 1,
        unit: YingQiUnit::Ri,
        zhi: xun_kong.0,
        method: YingQiMethod::KongWangTianShi,
    });

    // 六冲应期
    let special = Some(YingQiResult {
        num: san_chuan.chu.liu_chong().index() + 1,
        unit: YingQiUnit::Ri,
        zhi: san_chuan.chu.liu_chong(),
        method: YingQiMethod::LiuChong,
    });

    YingQiAnalysis {
        primary,
        secondary,
        special,
        suggestion_index: 0,
    }
}

// ============================================================================
// 综合吉凶判断
// ============================================================================

/// 计算综合吉凶
fn calculate_fortune(
    san_chuan_analysis: &SanChuanAnalysis,
    si_ke_analysis: &SiKeAnalysis,
    tian_jiang_analysis: &TianJiangAnalysis,
    _ke_shi: KeShiType,
    ge_ju: GeJuType,
) -> FortuneLevel {
    let mut score: i32 = 50; // 基础分

    // 三传分析加分
    // 递生加分
    if san_chuan_analysis.di_sheng {
        score += 20;
    }
    // 递克减分
    if san_chuan_analysis.di_ke {
        score -= 20;
    }
    // 连茹略减分
    if san_chuan_analysis.lian_ru {
        score -= 5;
    }

    // 初传旺衰
    match san_chuan_analysis.chu_wang_shuai {
        WangShuai::Wang => score += 15,
        WangShuai::Xiang => score += 10,
        WangShuai::Xiu => score -= 5,
        WangShuai::Qiu => score -= 10,
        WangShuai::Si => score -= 15,
    }

    // 末传旺衰（结果）
    match san_chuan_analysis.mo_wang_shuai {
        WangShuai::Wang => score += 10,
        WangShuai::Xiang => score += 5,
        WangShuai::Xiu => score -= 3,
        WangShuai::Qiu => score -= 8,
        WangShuai::Si => score -= 12,
    }

    // 三传天将吉凶
    if san_chuan_analysis.chu_jiang_ji {
        score += 8;
    } else {
        score -= 8;
    }
    if san_chuan_analysis.mo_jiang_ji {
        score += 8;
    } else {
        score -= 8;
    }

    // 空亡减分
    if san_chuan_analysis.chu_kong {
        score -= 15;
    }
    if san_chuan_analysis.mo_kong {
        score -= 10;
    }

    // 四课分析
    if si_ke_analysis.ri_gan_you_zhu {
        score += 5;
    }
    // 上克下多为吉
    score += (si_ke_analysis.shang_ke_xia_count as i32) * 3;
    // 下克上多为凶
    score -= (si_ke_analysis.xia_ke_shang_count as i32) * 3;

    // 天将分析
    score += (tian_jiang_analysis.san_chuan_ji_jiang as i32) * 5;
    if tian_jiang_analysis.gui_ren_kong {
        score -= 10;
    }
    if tian_jiang_analysis.gui_ren_mu {
        score -= 5;
    }

    // 课式格局加减分
    match ge_ju {
        GeJuType::YuanShou => score += 10, // 元首格大吉
        GeJuType::ChongShen => score -= 5, // 重审格需谨慎
        GeJuType::ZiRen | GeJuType::ZiXin => score += 5, // 伏吟格中性偏吉
        GeJuType::WuYi => score -= 10, // 无依格凶
        _ => {}
    }

    // 转换为吉凶等级
    match score {
        s if s >= 80 => FortuneLevel::DaJi,
        s if s >= 65 => FortuneLevel::ZhongJi,
        s if s >= 55 => FortuneLevel::XiaoJi,
        s if s >= 45 => FortuneLevel::Ping,
        s if s >= 35 => FortuneLevel::XiaoXiong,
        s if s >= 20 => FortuneLevel::ZhongXiong,
        _ => FortuneLevel::DaXiong,
    }
}

/// 计算趋势
fn calculate_trend(san_chuan_analysis: &SanChuanAnalysis) -> TrendType {
    // 比较初传和末传的状态
    let chu_score = wang_shuai_to_score(san_chuan_analysis.chu_wang_shuai)
        + if san_chuan_analysis.chu_jiang_ji { 10 } else { 0 }
        - if san_chuan_analysis.chu_kong { 15 } else { 0 };

    let mo_score = wang_shuai_to_score(san_chuan_analysis.mo_wang_shuai)
        + if san_chuan_analysis.mo_jiang_ji { 10 } else { 0 }
        - if san_chuan_analysis.mo_kong { 15 } else { 0 };

    if san_chuan_analysis.di_sheng {
        TrendType::Ascending
    } else if san_chuan_analysis.di_ke {
        TrendType::Descending
    } else if mo_score > chu_score + 10 {
        TrendType::Ascending
    } else if mo_score < chu_score - 10 {
        TrendType::Descending
    } else {
        TrendType::Stable
    }
}

/// 旺衰转评分
fn wang_shuai_to_score(ws: WangShuai) -> i32 {
    match ws {
        WangShuai::Wang => 20,
        WangShuai::Xiang => 15,
        WangShuai::Xiu => 10,
        WangShuai::Qiu => 5,
        WangShuai::Si => 0,
    }
}

/// 计算成败
fn calculate_outcome(
    san_chuan_analysis: &SanChuanAnalysis,
    tian_jiang_analysis: &TianJiangAnalysis,
    fortune: FortuneLevel,
) -> OutcomeType {
    // 初传空亡通常不成
    if san_chuan_analysis.chu_kong && san_chuan_analysis.mo_kong {
        return OutcomeType::BuCheng;
    }

    // 根据吉凶等级判断
    match fortune {
        FortuneLevel::DaJi => OutcomeType::BiCheng,
        FortuneLevel::ZhongJi => {
            if san_chuan_analysis.chu_kong {
                OutcomeType::KeCheng
            } else {
                OutcomeType::BiCheng
            }
        }
        FortuneLevel::XiaoJi => OutcomeType::KeCheng,
        FortuneLevel::Ping => {
            if tian_jiang_analysis.san_chuan_ji_jiang >= 2 {
                OutcomeType::KeCheng
            } else {
                OutcomeType::NanCheng
            }
        }
        FortuneLevel::XiaoXiong => OutcomeType::NanCheng,
        FortuneLevel::ZhongXiong | FortuneLevel::DaXiong => OutcomeType::BuCheng,
    }
}

/// 计算可信度
fn calculate_confidence(
    ke_shi: KeShiType,
    san_chuan_analysis: &SanChuanAnalysis,
    tian_jiang_analysis: &TianJiangAnalysis,
) -> u8 {
    let mut confidence = 80u8;

    // 特殊课式可信度较低
    match ke_shi {
        KeShiType::BieZe | KeShiType::BaZhuan => confidence -= 10,
        KeShiType::FuYin | KeShiType::FanYin => confidence -= 5,
        _ => {}
    }

    // 三传全空亡降低可信度
    if san_chuan_analysis.chu_kong && san_chuan_analysis.zhong_kong && san_chuan_analysis.mo_kong {
        confidence -= 20;
    }

    // 贵人空亡或入墓降低可信度
    if tian_jiang_analysis.gui_ren_kong {
        confidence -= 10;
    }
    if tian_jiang_analysis.gui_ren_mu {
        confidence -= 5;
    }

    confidence.max(30)
}

/// 计算应期可信度
fn calculate_ying_qi_confidence(
    san_chuan_analysis: &SanChuanAnalysis,
    _xun_kong: (DiZhi, DiZhi),
) -> u8 {
    let mut confidence = 75u8;

    // 空亡影响应期准确度
    if san_chuan_analysis.chu_kong {
        confidence -= 15;
    }
    if san_chuan_analysis.mo_kong {
        confidence -= 10;
    }

    // 递生递克关系明确时应期更准
    if san_chuan_analysis.di_sheng || san_chuan_analysis.di_ke {
        confidence += 10;
    }

    confidence.min(95).max(30)
}

// ============================================================================
// 事象断语生成
// ============================================================================

/// 生成事象断语
fn generate_shi_xiang_hints(
    shi_xiang_type: ShiXiangType,
    core: &CoreInterpretation,
    san_chuan_analysis: &SanChuanAnalysis,
    tian_jiang_analysis: &TianJiangAnalysis,
) -> ShiXiangHints {
    // 主断语索引（基于吉凶等级）
    let primary_hint_index = core.fortune as u8;

    // 辅助断语
    let mut secondary_hints = BoundedVec::new();

    // 根据三传状态添加辅助断语
    if san_chuan_analysis.di_sheng {
        let _ = secondary_hints.try_push(1); // 递生断语
    }
    if san_chuan_analysis.di_ke {
        let _ = secondary_hints.try_push(2); // 递克断语
    }
    if san_chuan_analysis.chu_kong {
        let _ = secondary_hints.try_push(3); // 空亡断语
    }

    // 注意事项
    let caution_index = if tian_jiang_analysis.gui_ren_kong {
        Some(1u8) // 贵人空亡注意
    } else if san_chuan_analysis.mo_kong {
        Some(2u8) // 末传空亡注意
    } else {
        None
    };

    ShiXiangHints {
        shi_xiang_type,
        primary_hint_index,
        secondary_hints,
        caution_index,
    }
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_san_chuan_relation() {
        // 测试递生关系：水生木，木生火
        let san_chuan = SanChuan {
            chu: DiZhi::Zi,      // 水
            zhong: DiZhi::Yin,   // 木
            mo: DiZhi::Wu,       // 火
            chu_jiang: TianJiang::GuiRen,
            zhong_jiang: TianJiang::QingLong,
            mo_jiang: TianJiang::ZhuQue,
            chu_qin: LiuQin::XiongDi,
            zhong_qin: LiuQin::ZiSun,
            mo_qin: LiuQin::QiCai,
            chu_dun: None,
            zhong_dun: None,
            mo_dun: None,
        };

        let (di_sheng, di_ke, _lian_ru) = analyze_san_chuan_relation(&san_chuan);
        assert!(di_sheng, "水生木，木生火应为递生");
        assert!(!di_ke, "不应为递克");
    }

    #[test]
    fn test_calculate_fortune() {
        let san_chuan_analysis = SanChuanAnalysis {
            chu_wang_shuai: WangShuai::Wang,
            chu_jiang_ji: true,
            chu_kong: false,
            zhong_wang_shuai: WangShuai::Xiang,
            zhong_jiang_ji: true,
            zhong_kong: false,
            mo_wang_shuai: WangShuai::Wang,
            mo_jiang_ji: true,
            mo_kong: false,
            di_sheng: true,
            di_ke: false,
            lian_ru: false,
        };

        let si_ke_analysis = SiKeAnalysis {
            ri_gan_you_zhu: true,
            gan_yang_wang_shuai: WangShuai::Wang,
            ri_zhi_you_sheng: true,
            zhi_yang_wang_shuai: WangShuai::Xiang,
            shang_ke_xia_count: 2,
            xia_ke_shang_count: 0,
            gan_zhi_he: false,
            gan_zhi_chong: false,
        };

        let tian_jiang_analysis = TianJiangAnalysis {
            gui_ren_lin: DiZhi::Wei,
            gui_ren_kong: false,
            gui_ren_mu: false,
            qing_long_lin: DiZhi::Yin,
            bai_hu_lin: DiZhi::Shen,
            ji_jiang_count: 6,
            xiong_jiang_count: 6,
            san_chuan_ji_jiang: 3,
        };

        let fortune = calculate_fortune(
            &san_chuan_analysis,
            &si_ke_analysis,
            &tian_jiang_analysis,
            KeShiType::ZeiKe,
            GeJuType::YuanShou,
        );

        // 递生、旺相、吉将应为大吉或中吉
        assert!(
            matches!(fortune, FortuneLevel::DaJi | FortuneLevel::ZhongJi),
            "良好条件应为吉"
        );
    }

    #[test]
    fn test_calculate_ying_qi() {
        let san_chuan = SanChuan {
            chu: DiZhi::Yin,     // 3
            zhong: DiZhi::Mao,   // 4
            mo: DiZhi::Chen,     // 5
            chu_jiang: TianJiang::GuiRen,
            zhong_jiang: TianJiang::QingLong,
            mo_jiang: TianJiang::LiuHe,
            chu_qin: LiuQin::XiongDi,
            zhong_qin: LiuQin::XiongDi,
            mo_qin: LiuQin::XiongDi,
            chu_dun: None,
            zhong_dun: None,
            mo_dun: None,
        };

        let xun_kong = (DiZhi::Xu, DiZhi::Hai);
        let (num, unit, _secondary) = calculate_ying_qi(&san_chuan, xun_kong);

        // 3 + 4 + 5 = 12
        assert_eq!(num, 12, "应期数应为12");
        assert_eq!(unit, YingQiUnit::Ri, "应期单位应为日");
    }
}
