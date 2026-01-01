//! # 大六壬排盘算法
//!
//! 本模块实现大六壬排盘的核心算法，包括：
//! - 天盘布局
//! - 四课起法
//! - 三传取法（九种课式）
//! - 天将排布
//! - 空亡计算
//!
//! ## 大六壬起课口诀
//!
//! 月将加时辰，天盘顺时转。
//! 干支起四课，克贼定三传。
//!
//! ## 九种取三传方法
//!
//! 1. 贼克法：下贼上为贼，上克下为克
//! 2. 比用法：俱多先取与日干阴阳相同者
//! 3. 涉害法：涉害深者为用
//! 4. 遥克法：四课无克，二三四课克日干
//! 5. 昂星法：四课俱全无克，阳干取酉上神，阴干取酉所临
//! 6. 别责法：三课不备，阳干取干合地，阴干取支前四位
//! 7. 八专法：八专日（甲寅、庚申、丁未、己未）
//! 8. 伏吟法：天盘地盘相同
//! 9. 返吟法：天盘地盘六冲

extern crate alloc;
use alloc::vec::Vec;

use crate::types::*;

// ============================================================================
// 天盘计算
// ============================================================================

/// 根据月将和占时计算天盘
///
/// 月将加占时，天盘顺时针旋转
/// 例如：月将为午，占时为子，则午加子位，天盘顺转
pub fn calculate_tian_pan(yue_jiang: DiZhi, zhan_shi: DiZhi) -> TianPan {
    let mut positions = [DiZhi::Zi; 12];

    // 月将加占时：月将临占时的地盘位置
    // 天盘 = 月将 - (占时 - 子)
    let offset = zhan_shi.sub(DiZhi::Zi);
    let tian_yin = yue_jiang.add(-offset);

    // 填充天盘十二宫
    for i in 0..12 {
        positions[i] = tian_yin.add(i as i8);
    }

    TianPan { positions }
}

// ============================================================================
// 天将盘计算
// ============================================================================

/// 根据日干和昼夜计算天将盘
///
/// 贵人起法：
/// - 昼占用昼贵人
/// - 夜占用夜贵人
/// - 贵人临巳至戌逆布，否则顺布
///
/// 天将盘说明：
/// - 贵人所临的是天盘地支（即贵人表中的地支）
/// - 计算贵人所在的地盘位置，判断顺逆
/// - 天将随贵人顺/逆布于天盘各地支上
pub fn calculate_tian_jiang_pan(
    tian_pan: &TianPan,
    day_gan: TianGan,
    is_day: bool,
) -> TianJiangPan {
    let mut positions = [TianJiang::GuiRen; 12];

    // 获取贵人所临的天盘地支（从贵人表获取）
    let gui_ren_tian_zhi = get_gui_ren(day_gan, is_day);

    // 计算贵人所临的地盘地支（天盘地支所在的地盘位置）
    let gui_ren_di_zhi = tian_pan.lin(gui_ren_tian_zhi);

    // 判断顺逆：贵人地盘临巳至戌为逆布
    let is_reverse = matches!(
        gui_ren_di_zhi,
        DiZhi::Si | DiZhi::Wu | DiZhi::Wei | DiZhi::Shen | DiZhi::You | DiZhi::Xu
    );

    // 布天将：天将临天盘地支
    // 遍历天盘的每个位置（地盘地支），获取其上的天盘地支，然后计算天将
    for i in 0..12 {
        let di_zhi = DiZhi::from_index(i as u8);      // 地盘地支
        let tian_zhi = tian_pan.get(di_zhi);          // 该位置上的天盘地支

        // 计算天盘地支与贵人所临天盘地支的距离
        let delta = if is_reverse {
            // 逆布：贵人地支 - 天盘地支
            gui_ren_tian_zhi.sub(tian_zhi)
        } else {
            // 顺布：天盘地支 - 贵人地支
            tian_zhi.sub(gui_ren_tian_zhi)
        };

        positions[i] = TianJiang::from_index(((delta + 12) % 12) as u8);
    }

    TianJiangPan { positions, is_reverse }
}

// ============================================================================
// 四课计算
// ============================================================================

/// 计算四课
///
/// 四课起法：
/// - 第一课：日干寄宫上神为干阳神
/// - 第二课：干阳神上神为干阴神
/// - 第三课：日支上神为支阳神
/// - 第四课：支阳神上神为支阴神
pub fn calculate_si_ke(
    tian_pan: &TianPan,
    tian_jiang_pan: &TianJiangPan,
    day_gan: TianGan,
    day_zhi: DiZhi,
) -> SiKe {
    let day_wx = day_gan.wu_xing();

    // 日干寄宫
    let gan_ji_gong = get_ji_gong(day_gan);

    // 第一课：干阳神
    let gan_yang = tian_pan.get(gan_ji_gong);
    let ke1 = KeInfo {
        shang: gan_yang,
        xia: gan_ji_gong,
        tian_jiang: tian_jiang_pan.get(gan_yang),
        liu_qin: LiuQin::from_wu_xing(day_wx, gan_yang.wu_xing()),
    };

    // 第二课：干阴神
    let gan_yin = tian_pan.get(gan_yang);
    let ke2 = KeInfo {
        shang: gan_yin,
        xia: gan_yang,
        tian_jiang: tian_jiang_pan.get(gan_yin),
        liu_qin: LiuQin::from_wu_xing(day_wx, gan_yin.wu_xing()),
    };

    // 第三课：支阳神
    let zhi_yang = tian_pan.get(day_zhi);
    let ke3 = KeInfo {
        shang: zhi_yang,
        xia: day_zhi,
        tian_jiang: tian_jiang_pan.get(zhi_yang),
        liu_qin: LiuQin::from_wu_xing(day_wx, zhi_yang.wu_xing()),
    };

    // 第四课：支阴神
    let zhi_yin = tian_pan.get(zhi_yang);
    let ke4 = KeInfo {
        shang: zhi_yin,
        xia: zhi_yang,
        tian_jiang: tian_jiang_pan.get(zhi_yin),
        liu_qin: LiuQin::from_wu_xing(day_wx, zhi_yin.wu_xing()),
    };

    SiKe { ke1, ke2, ke3, ke4 }
}

// ============================================================================
// 三传计算（核心算法）
// ============================================================================

/// 计算三传和格局
///
/// 九种取三传方法：
/// 1. 先判断伏吟、返吟
/// 2. 贼克法
/// 3. 比用法
/// 4. 涉害法
/// 5. 遥克法
/// 6. 昂星法
/// 7. 别责法
/// 8. 八专法
pub fn calculate_san_chuan(
    tian_pan: &TianPan,
    tian_jiang_pan: &TianJiangPan,
    si_ke: &SiKe,
    day_gan: TianGan,
    day_zhi: DiZhi,
) -> (SanChuan, KeShiType, GeJuType) {
    let day_wx = day_gan.wu_xing();

    // 判断伏吟（支阳神等于日支）
    if si_ke.ke3.shang == day_zhi {
        return calculate_fu_yin(tian_pan, tian_jiang_pan, si_ke, day_gan, day_zhi, day_wx);
    }

    // 判断返吟（支阳神冲日支）
    if si_ke.ke3.shang.is_chong(day_zhi) {
        return calculate_fan_yin(tian_pan, tian_jiang_pan, si_ke, day_gan, day_zhi, day_wx);
    }

    // 尝试贼克法
    if let Some(result) = try_zei_ke(tian_pan, tian_jiang_pan, si_ke, day_gan, day_zhi, day_wx) {
        return result;
    }

    // 尝试遥克法
    if let Some(result) = try_yao_ke(tian_pan, tian_jiang_pan, si_ke, day_gan, day_zhi, day_wx) {
        return result;
    }

    // 尝试昂星法
    if let Some(result) = try_ang_xing(tian_pan, tian_jiang_pan, si_ke, day_gan, day_zhi, day_wx) {
        return result;
    }

    // 尝试别责法
    if let Some(result) = try_bie_ze(tian_pan, tian_jiang_pan, si_ke, day_gan, day_zhi, day_wx) {
        return result;
    }

    // 尝试八专法
    if let Some(result) = try_ba_zhuan(tian_pan, tian_jiang_pan, si_ke, day_gan, day_zhi, day_wx) {
        return result;
    }

    // 默认返回（不应该到达这里）
    let chu = si_ke.ke1.shang;
    let zhong = tian_pan.get(chu);
    let mo = tian_pan.get(zhong);

    (
        build_san_chuan(tian_pan, tian_jiang_pan, chu, zhong, mo, day_gan, day_zhi, day_wx),
        KeShiType::ZeiKe,
        GeJuType::YuanShou,
    )
}

/// 伏吟课
///
/// 伏吟取三传规则：
/// 1. 六乙日、六癸日或阳干：取干阳神为初传
/// 2. 阴日（非六乙、六癸）：取支阳神为初传
/// 3. 中传取初传所刑
/// 4. 初传自刑时：阳干/六乙/六癸取支阳神，阴干取干阳神
/// 5. 末传取中传所刑
/// 6. 中传自刑或初中互刑时：取中传六冲为末传
fn calculate_fu_yin(
    tian_pan: &TianPan,
    tian_jiang_pan: &TianJiangPan,
    si_ke: &SiKe,
    day_gan: TianGan,
    day_zhi: DiZhi,
    day_wx: WuXing,
) -> (SanChuan, KeShiType, GeJuType) {
    // 伏吟：六乙、六癸日或阳干，取干阳神
    // 阴日非六乙、六癸，取支阳神
    let chu = if day_gan == TianGan::Yi || day_gan == TianGan::Gui || day_gan.is_yang() {
        si_ke.ke1.shang
    } else {
        si_ke.ke3.shang
    };

    // 中传取初传所刑
    let mut zhong = chu.xing();

    // 初传自刑处理：
    // 自刑地支：辰辰、午午、酉酉、亥亥
    if chu == zhong {
        zhong = if day_gan == TianGan::Yi || day_gan == TianGan::Gui || day_gan.is_yang() {
            si_ke.ke3.shang
        } else {
            si_ke.ke1.shang
        };
    }

    // 末传取中传所刑
    let mut mo = zhong.xing();

    // 处理特殊情况：
    // 1. 中传自刑：取中传六冲为末传
    // 2. 初中互刑（如子刑卯、卯刑子）：取中传六冲为末传
    if zhong == mo || chu.xing() == zhong {
        mo = zhong.liu_chong();
    }

    let ge_ju = if day_gan.is_yang() {
        GeJuType::ZiRen
    } else {
        GeJuType::ZiXin
    };

    (
        build_san_chuan(tian_pan, tian_jiang_pan, chu, zhong, mo, day_gan, day_zhi, day_wx),
        KeShiType::FuYin,
        ge_ju,
    )
}

/// 返吟课
fn calculate_fan_yin(
    tian_pan: &TianPan,
    tian_jiang_pan: &TianJiangPan,
    si_ke: &SiKe,
    day_gan: TianGan,
    day_zhi: DiZhi,
    day_wx: WuXing,
) -> (SanChuan, KeShiType, GeJuType) {
    // 先尝试贼克
    if let Some(result) = try_zei_ke(tian_pan, tian_jiang_pan, si_ke, day_gan, day_zhi, day_wx) {
        let (san_chuan, _, ge_ju) = result;
        return (san_chuan, KeShiType::FanYin, ge_ju);
    }

    // 无贼克，取驿马
    let yi_ma = day_zhi.yi_ma();
    let chu = yi_ma;
    let zhong = si_ke.ke3.shang;
    let mo = si_ke.ke1.shang;

    (
        build_san_chuan(tian_pan, tian_jiang_pan, chu, zhong, mo, day_gan, day_zhi, day_wx),
        KeShiType::FanYin,
        GeJuType::WuYi,
    )
}

/// 尝试贼克法
fn try_zei_ke(
    tian_pan: &TianPan,
    tian_jiang_pan: &TianJiangPan,
    si_ke: &SiKe,
    day_gan: TianGan,
    day_zhi: DiZhi,
    day_wx: WuXing,
) -> Option<(SanChuan, KeShiType, GeJuType)> {
    let ke_list = [&si_ke.ke1, &si_ke.ke2, &si_ke.ke3, &si_ke.ke4];

    // 收集所有贼（下克上）
    let mut zei_list: Vec<(usize, &KeInfo)> = Vec::new();
    for (i, ke) in ke_list.iter().enumerate() {
        if ke.xia.wu_xing().ke(ke.shang.wu_xing()) {
            zei_list.push((i, ke));
        }
    }

    // 收集所有克（上克下）
    let mut ke_list_clone: Vec<(usize, &KeInfo)> = Vec::new();
    for (i, ke) in ke_list.iter().enumerate() {
        if ke.shang.wu_xing().ke(ke.xia.wu_xing()) {
            ke_list_clone.push((i, ke));
        }
    }

    // 先看贼
    if !zei_list.is_empty() {
        // 去重
        let mut unique_zei: Vec<(usize, &KeInfo)> = Vec::new();
        for item in zei_list.iter() {
            if !unique_zei.iter().any(|(_, k)| k.shang == item.1.shang) {
                unique_zei.push(*item);
            }
        }

        if unique_zei.len() == 1 {
            let chu = unique_zei[0].1.shang;
            let zhong = tian_pan.get(chu);
            let mo = tian_pan.get(zhong);
            return Some((
                build_san_chuan(tian_pan, tian_jiang_pan, chu, zhong, mo, day_gan, day_zhi, day_wx),
                KeShiType::ZeiKe,
                GeJuType::ChongShen,
            ));
        } else if unique_zei.len() > 1 {
            return try_bi_yong(tian_pan, tian_jiang_pan, &unique_zei, day_gan, day_zhi, day_wx, true);
        }
    }

    // 再看克
    if !ke_list_clone.is_empty() {
        // 去重
        let mut unique_ke: Vec<(usize, &KeInfo)> = Vec::new();
        for item in ke_list_clone.iter() {
            if !unique_ke.iter().any(|(_, k)| k.shang == item.1.shang) {
                unique_ke.push(*item);
            }
        }

        if unique_ke.len() == 1 {
            let chu = unique_ke[0].1.shang;
            let zhong = tian_pan.get(chu);
            let mo = tian_pan.get(zhong);
            return Some((
                build_san_chuan(tian_pan, tian_jiang_pan, chu, zhong, mo, day_gan, day_zhi, day_wx),
                KeShiType::ZeiKe,
                GeJuType::YuanShou,
            ));
        } else if unique_ke.len() > 1 {
            return try_bi_yong(tian_pan, tian_jiang_pan, &unique_ke, day_gan, day_zhi, day_wx, false);
        }
    }

    None
}

/// 比用法
///
/// 当有多个贼克时，取与日干阴阳相同者。
/// 阴阳判断：阳干（甲丙戊庚壬）为阳，阴干（乙丁己辛癸）为阴
/// 地支阴阳：子寅辰午申戌为阳，丑卯巳未酉亥为阴
fn try_bi_yong(
    tian_pan: &TianPan,
    tian_jiang_pan: &TianJiangPan,
    ke_candidates: &[(usize, &KeInfo)],
    day_gan: TianGan,
    day_zhi: DiZhi,
    day_wx: WuXing,
    is_zei: bool,
) -> Option<(SanChuan, KeShiType, GeJuType)> {
    // 使用日干本身的阴阳，而非日干寄宫的阴阳
    let gan_is_yang = day_gan.is_yang();

    // 筛选阴阳相同者
    let mut bi_list: Vec<(usize, &KeInfo)> = Vec::new();
    for item in ke_candidates.iter() {
        // 地支阴阳：偶数（子寅辰午申戌）为阳，奇数（丑卯巳未酉亥）为阴
        let shang_is_yang = (item.1.shang.index() % 2) == 0;
        if shang_is_yang == gan_is_yang {
            bi_list.push(*item);
        }
    }

    if bi_list.len() == 1 {
        let chu = bi_list[0].1.shang;
        let zhong = tian_pan.get(chu);
        let mo = tian_pan.get(zhong);
        return Some((
            build_san_chuan(tian_pan, tian_jiang_pan, chu, zhong, mo, day_gan, day_zhi, day_wx),
            KeShiType::BiYong,
            GeJuType::ZhiYi,
        ));
    } else if bi_list.is_empty() {
        // 俱不比，用涉害
        return try_she_hai(tian_pan, tian_jiang_pan, ke_candidates, day_gan, day_zhi, day_wx, is_zei);
    } else {
        // 多个俱比，用涉害
        return try_she_hai(tian_pan, tian_jiang_pan, &bi_list, day_gan, day_zhi, day_wx, is_zei);
    }
}

/// 涉害法
///
/// 涉害深度计算：从所临地盘支遍历到天盘支，
/// 计算途中受克（贼）或能克（克）的地支和天干寄宫数量。
fn try_she_hai(
    tian_pan: &TianPan,
    tian_jiang_pan: &TianJiangPan,
    ke_candidates: &[(usize, &KeInfo)],
    day_gan: TianGan,
    day_zhi: DiZhi,
    day_wx: WuXing,
    is_zei: bool,
) -> Option<(SanChuan, KeShiType, GeJuType)> {
    // 计算各课涉害深度
    let mut depth_list: Vec<(usize, &KeInfo, u8)> = Vec::new();

    for item in ke_candidates.iter() {
        let shang = item.1.shang;
        let lin_di = tian_pan.lin(shang);
        let mut count: u8 = 0;

        // 从所临地盘支遍历到天盘支
        for i in 0..12 {
            let d = lin_di.add(i as i8);
            if d == shang {
                break;
            }

            if is_zei {
                // 贼：数下克上（数途中能克上神的地支和天干）
                // 1. 地支五行克上神五行
                if d.wu_xing().ke(shang.wu_xing()) {
                    count += 1;
                }
                // 2. 该地支上寄宫的天干五行克上神五行
                let gan_list = get_gan_of_ji_gong(d);
                for g in gan_list {
                    if g.wu_xing().ke(shang.wu_xing()) {
                        count += 1;
                    }
                }
            } else {
                // 克：数上克下（数途中被上神克的地支和天干）
                // 1. 上神五行克地支五行
                if shang.wu_xing().ke(d.wu_xing()) {
                    count += 1;
                }
                // 2. 上神五行克该地支上寄宫的天干五行
                let gan_list = get_gan_of_ji_gong(d);
                for g in gan_list {
                    if shang.wu_xing().ke(g.wu_xing()) {
                        count += 1;
                    }
                }
            }
        }

        depth_list.push((item.0, item.1, count));
    }

    // 找最大涉害深度
    let max_depth = depth_list.iter().map(|(_, _, d)| *d).max().unwrap_or(0);
    let max_list: Vec<_> = depth_list.iter().filter(|(_, _, d)| *d == max_depth).collect();

    if max_list.len() == 1 {
        let chu = max_list[0].1.shang;
        let zhong = tian_pan.get(chu);
        let mo = tian_pan.get(zhong);
        return Some((
            build_san_chuan(tian_pan, tian_jiang_pan, chu, zhong, mo, day_gan, day_zhi, day_wx),
            KeShiType::SheHai,
            GeJuType::SheHaiGe,
        ));
    }

    // 涉害同，从孟发用
    for item in max_list.iter() {
        let lin_di = tian_pan.lin(item.1.shang);
        if lin_di.is_meng() {
            let chu = item.1.shang;
            let zhong = tian_pan.get(chu);
            let mo = tian_pan.get(zhong);
            return Some((
                build_san_chuan(tian_pan, tian_jiang_pan, chu, zhong, mo, day_gan, day_zhi, day_wx),
                KeShiType::SheHai,
                GeJuType::JianJi,
            ));
        }
    }

    // 从仲发用
    for item in max_list.iter() {
        let lin_di = tian_pan.lin(item.1.shang);
        if lin_di.is_zhong() {
            let chu = item.1.shang;
            let zhong = tian_pan.get(chu);
            let mo = tian_pan.get(zhong);
            return Some((
                build_san_chuan(tian_pan, tian_jiang_pan, chu, zhong, mo, day_gan, day_zhi, day_wx),
                KeShiType::SheHai,
                GeJuType::ChaWei,
            ));
        }
    }

    // 复等：阳干取干阳神，阴干取支阳神
    let gan_ji_gong = get_ji_gong(day_gan);
    let chu = if day_gan.is_yang() {
        tian_pan.get(gan_ji_gong)
    } else {
        tian_pan.get(day_zhi)
    };
    let zhong = tian_pan.get(chu);
    let mo = tian_pan.get(zhong);

    Some((
        build_san_chuan(tian_pan, tian_jiang_pan, chu, zhong, mo, day_gan, day_zhi, day_wx),
        KeShiType::SheHai,
        GeJuType::FuDeng,
    ))
}

/// 遥克法
fn try_yao_ke(
    tian_pan: &TianPan,
    tian_jiang_pan: &TianJiangPan,
    si_ke: &SiKe,
    day_gan: TianGan,
    day_zhi: DiZhi,
    day_wx: WuXing,
) -> Option<(SanChuan, KeShiType, GeJuType)> {
    // 八专日不用遥克
    if is_ba_zhuan_day(day_gan, day_zhi) {
        return None;
    }

    let ke_list = [&si_ke.ke2, &si_ke.ke3, &si_ke.ke4];

    // 二三四课克日干
    let mut ko_list: Vec<&KeInfo> = Vec::new();
    for ke in ke_list.iter() {
        if ke.shang.wu_xing().ke(day_wx) {
            ko_list.push(ke);
        }
    }

    // 如果没有克日干的，看日干克的
    if ko_list.is_empty() {
        for ke in ke_list.iter() {
            if day_wx.ke(ke.shang.wu_xing()) {
                ko_list.push(ke);
            }
        }
    }

    if ko_list.is_empty() {
        return None;
    }

    // 去重
    let mut unique_ko: Vec<&KeInfo> = Vec::new();
    for ke in ko_list.iter() {
        if !unique_ko.iter().any(|k| k.shang == ke.shang) {
            unique_ko.push(ke);
        }
    }

    if unique_ko.len() == 1 {
        let chu = unique_ko[0].shang;
        let zhong = tian_pan.get(chu);
        let mo = tian_pan.get(zhong);
        return Some((
            build_san_chuan(tian_pan, tian_jiang_pan, chu, zhong, mo, day_gan, day_zhi, day_wx),
            KeShiType::YaoKe,
            GeJuType::YaoKeGe,
        ));
    }

    // 多个遥克，用比用
    let candidates: Vec<(usize, &KeInfo)> = unique_ko.iter().enumerate().map(|(i, k)| (i, *k)).collect();
    try_bi_yong(tian_pan, tian_jiang_pan, &candidates, day_gan, day_zhi, day_wx, false)
}

/// 昂星法
fn try_ang_xing(
    tian_pan: &TianPan,
    tian_jiang_pan: &TianJiangPan,
    si_ke: &SiKe,
    day_gan: TianGan,
    day_zhi: DiZhi,
    day_wx: WuXing,
) -> Option<(SanChuan, KeShiType, GeJuType)> {
    // 检查四课是否俱全（无重复）
    let ke_shangs = [
        si_ke.ke1.shang,
        si_ke.ke2.shang,
        si_ke.ke3.shang,
        si_ke.ke4.shang,
    ];

    let mut unique_count = 0;
    let mut seen = [false; 12];
    for shang in ke_shangs.iter() {
        if !seen[shang.index() as usize] {
            seen[shang.index() as usize] = true;
            unique_count += 1;
        }
    }

    if unique_count != 4 {
        return None;
    }

    // 阳干（虎视）：酉上神为初传，中传取支阳神，末传取干阳神
    // 阴干（冬蛇掩目）：酉所临地盘支为初传，中传取干阳神，末传取支阳神
    let chu;
    let zhong;
    let mo;
    let ge_ju;

    if day_gan.is_yang() {
        chu = tian_pan.get(DiZhi::You);
        zhong = si_ke.ke3.shang; // 支阳神
        mo = si_ke.ke1.shang;    // 干阳神
        ge_ju = GeJuType::HuShi;
    } else {
        chu = tian_pan.lin(DiZhi::You);
        zhong = si_ke.ke1.shang; // 干阳神（阴干时顺序相反）
        mo = si_ke.ke3.shang;    // 支阳神
        ge_ju = GeJuType::DongSheYanMu;
    }

    Some((
        build_san_chuan(tian_pan, tian_jiang_pan, chu, zhong, mo, day_gan, day_zhi, day_wx),
        KeShiType::AngXing,
        ge_ju,
    ))
}

/// 别责法
fn try_bie_ze(
    tian_pan: &TianPan,
    tian_jiang_pan: &TianJiangPan,
    si_ke: &SiKe,
    day_gan: TianGan,
    day_zhi: DiZhi,
    day_wx: WuXing,
) -> Option<(SanChuan, KeShiType, GeJuType)> {
    // 检查是否三课不备
    let ke_shangs = [
        si_ke.ke1.shang,
        si_ke.ke2.shang,
        si_ke.ke3.shang,
        si_ke.ke4.shang,
    ];

    let mut unique_count = 0;
    let mut seen = [false; 12];
    for shang in ke_shangs.iter() {
        if !seen[shang.index() as usize] {
            seen[shang.index() as usize] = true;
            unique_count += 1;
        }
    }

    if unique_count != 3 {
        return None;
    }

    // 阳干：干合神上神
    // 阴干：支前四位
    let chu;

    if day_gan.is_yang() {
        // 干合：甲己合、乙庚合、丙辛合、丁壬合、戊癸合
        let he_gan = day_gan.add(5);
        let he_ji_gong = get_ji_gong(he_gan);
        chu = tian_pan.get(he_ji_gong);
    } else {
        chu = day_zhi.add(4);
    }

    let zhong = si_ke.ke1.shang;
    let mo = zhong;

    Some((
        build_san_chuan(tian_pan, tian_jiang_pan, chu, zhong, mo, day_gan, day_zhi, day_wx),
        KeShiType::BieZe,
        GeJuType::BieZeGe,
    ))
}

/// 八专法
fn try_ba_zhuan(
    tian_pan: &TianPan,
    tian_jiang_pan: &TianJiangPan,
    si_ke: &SiKe,
    day_gan: TianGan,
    day_zhi: DiZhi,
    day_wx: WuXing,
) -> Option<(SanChuan, KeShiType, GeJuType)> {
    if !is_ba_zhuan_day(day_gan, day_zhi) {
        return None;
    }

    // 阳干：干阳神前二位
    // 阴干：支阴神后二位
    let chu;

    if day_gan.is_yang() {
        chu = si_ke.ke1.shang.add(2);
    } else {
        chu = si_ke.ke4.shang.add(-2);
    }

    let zhong = si_ke.ke1.shang;
    let mo = zhong;

    Some((
        build_san_chuan(tian_pan, tian_jiang_pan, chu, zhong, mo, day_gan, day_zhi, day_wx),
        KeShiType::BaZhuan,
        GeJuType::BaZhuanGe,
    ))
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 构建三传结构
fn build_san_chuan(
    _tian_pan: &TianPan,
    tian_jiang_pan: &TianJiangPan,
    chu: DiZhi,
    zhong: DiZhi,
    mo: DiZhi,
    day_gan: TianGan,
    day_zhi: DiZhi,
    day_wx: WuXing,
) -> SanChuan {
    SanChuan {
        chu,
        zhong,
        mo,
        chu_jiang: tian_jiang_pan.get(chu),
        zhong_jiang: tian_jiang_pan.get(zhong),
        mo_jiang: tian_jiang_pan.get(mo),
        chu_qin: LiuQin::from_wu_xing(day_wx, chu.wu_xing()),
        zhong_qin: LiuQin::from_wu_xing(day_wx, zhong.wu_xing()),
        mo_qin: LiuQin::from_wu_xing(day_wx, mo.wu_xing()),
        chu_dun: calculate_dun_gan(chu, day_gan, day_zhi),
        zhong_dun: calculate_dun_gan(zhong, day_gan, day_zhi),
        mo_dun: calculate_dun_gan(mo, day_gan, day_zhi),
    }
}

/// 计算遁干
pub fn calculate_dun_gan(zhi: DiZhi, day_gan: TianGan, day_zhi: DiZhi) -> Option<TianGan> {
    // 计算旬首
    let delta = day_gan.index() as i8;
    let xun_shou = day_zhi.add(-delta);

    // 计算地支在旬中的位置
    let pos = zhi.sub(xun_shou);

    // 空亡（位置10、11）无遁干
    if pos >= 10 {
        return None;
    }

    Some(TianGan::from_index(pos as u8))
}

/// 计算空亡
pub fn calculate_xun_kong(day_gan: TianGan, day_zhi: DiZhi) -> (DiZhi, DiZhi) {
    let delta = day_gan.index() as i8;
    let xun_shou = day_zhi.add(-delta);

    let kong1 = xun_shou.add(10);
    let kong2 = xun_shou.add(11);

    (kong1, kong2)
}

/// 从随机数生成式盘参数
pub fn random_to_params(random_bytes: &[u8; 32]) -> (DiZhi, DiZhi, bool) {
    let yue_jiang = DiZhi::from_index(random_bytes[0] % 12);
    let zhan_shi = DiZhi::from_index(random_bytes[1] % 12);
    let is_day = random_bytes[2] % 2 == 0;

    (yue_jiang, zhan_shi, is_day)
}

// ============================================================================
// 神煞计算
// ============================================================================

/// 计算完整的神煞信息
///
/// # 参数
/// - `year_zhi`: 年支（太岁）
/// - `month_zhi`: 月支（月建）
/// - `day_gan`: 日干
/// - `day_zhi`: 日支
///
/// # 返回
/// 完整的神煞信息结构体
pub fn calculate_shen_sha(
    year_zhi: DiZhi,
    month_zhi: DiZhi,
    day_gan: TianGan,
    day_zhi: DiZhi,
) -> ShenShaInfo {
    // 计算旬空
    let xun_kong = calculate_xun_kong(day_gan, day_zhi);

    // 旬首（空亡后一位）
    let xun_shou = xun_kong.1.add(1);

    // 旬奇：旬首为戌子取丑，申午取子，寅辰取亥
    let xun_qi = match xun_shou {
        DiZhi::Xu | DiZhi::Zi => Some(DiZhi::Chou),
        DiZhi::Shen | DiZhi::Wu => Some(DiZhi::Zi),
        DiZhi::Yin | DiZhi::Chen => Some(DiZhi::Hai),
        _ => None,
    };

    // 旬仪：即旬首
    let xun_yi = xun_shou;

    // 日奇：甲至己从午逆排，庚至癸从未顺排
    let ri_qi = {
        let d = day_gan.index() as i8;
        if d <= 5 {
            DiZhi::Wu.add(-d)
        } else {
            DiZhi::Wei.add(d - 6)
        }
    };

    // 支仪：子至巳从午逆排，午至亥从未顺排
    let zhi_yi = {
        let d = day_zhi.index() as i8;
        if d <= 5 {
            DiZhi::Wu.add(-d)
        } else {
            DiZhi::Wei.add(d - 6)
        }
    };

    // 天罗：日干寄宫前一位
    let tian_luo = get_ji_gong(day_gan).add(1);
    // 地网：天罗六冲
    let di_wang = tian_luo.add(6);

    // === 日支相关神煞 ===
    // 驿马
    let yi_ma = day_zhi.yi_ma();

    // === 月建相关神煞 ===
    // 月驿马
    let yue_yi_ma = calculate_yi_ma(month_zhi);

    // 天马：寅月午，每月加2
    let tian_ma = DiZhi::Wu.add(((month_zhi.index() as i8 - DiZhi::Yin.index() as i8 + 12) % 12) * 2);

    // 皇书：春寅夏巳秋申冬亥
    let huang_shu = match month_zhi {
        DiZhi::Yin | DiZhi::Mao | DiZhi::Chen => DiZhi::Yin,
        DiZhi::Si | DiZhi::Wu | DiZhi::Wei => DiZhi::Si,
        DiZhi::Shen | DiZhi::You | DiZhi::Xu => DiZhi::Shen,
        DiZhi::Hai | DiZhi::Zi | DiZhi::Chou => DiZhi::Hai,
    };

    // 皇恩：寅月未，每月加2
    let huang_en = DiZhi::Wei.add(((month_zhi.index() as i8 - DiZhi::Yin.index() as i8 + 12) % 12) * 2);

    // 天诏/飞魂：月建前三位
    let tian_zhao = month_zhi.add(-3);

    // 天喜：春戌夏丑秋辰冬未
    let tian_xi = match month_zhi {
        DiZhi::Yin | DiZhi::Mao | DiZhi::Chen => DiZhi::Xu,
        DiZhi::Si | DiZhi::Wu | DiZhi::Wei => DiZhi::Chou,
        DiZhi::Shen | DiZhi::You | DiZhi::Xu => DiZhi::Chen,
        DiZhi::Hai | DiZhi::Zi | DiZhi::Chou => DiZhi::Wei,
    };

    // 生气：月建前两位
    let sheng_qi = month_zhi.add(-2);

    // 死气/谩语：月建后四位
    let si_qi = month_zhi.add(4);

    // 三丘：找孟月，取前一位
    let san_qiu = {
        let mut zhi = month_zhi;
        for _ in 0..3 {
            if zhi.is_meng() {
                break;
            }
            zhi = zhi.add(-1);
        }
        zhi.add(-1)
    };
    // 五墓：三丘六冲
    let wu_mu = san_qiu.add(6);

    // 寡宿：同三丘逻辑
    let gua_su = {
        let mut zhi = month_zhi;
        for _ in 0..3 {
            if zhi.is_meng() {
                break;
            }
            zhi = zhi.add(-1);
        }
        zhi.add(-1)
    };
    // 孤辰：寡宿后四位
    let gu_chen = gua_su.add(4);

    // 天医/天巫：月建后两位
    let tian_yi_shen = month_zhi.add(2);
    // 地医/地巫：天医六冲
    let di_yi = tian_yi_shen.add(6);

    // 破碎
    let po_sui = {
        let delta = (month_zhi.index() as i8 - DiZhi::Yin.index() as i8 + 12) % 12;
        match delta % 3 {
            0 => Some(DiZhi::You),
            1 => Some(DiZhi::Si),
            2 => Some(DiZhi::Chou),
            _ => None,
        }
    };

    // 月厌：子与月建相对
    let yue_yan = DiZhi::Zi.add(DiZhi::Zi.sub(month_zhi));

    // 血支：月建前一位
    let xue_zhi = month_zhi.add(-1);

    // 血忌
    let xue_ji = {
        let delta = (month_zhi.index() as i8 - DiZhi::Yin.index() as i8 + 12) % 12;
        if delta % 2 == 0 {
            DiZhi::Chou.add((delta / 2) as i8)
        } else {
            DiZhi::Wei.add(((delta - 1) / 2) as i8)
        }
    };

    // 丧车：春酉夏子秋卯冬午
    let sang_che = {
        let n = ((month_zhi.index() as i8 - DiZhi::Yin.index() as i8 + 12) % 12) / 3;
        [DiZhi::You, DiZhi::Zi, DiZhi::Mao, DiZhi::Wu][n as usize]
    };

    // 丧魂：找孟月，加5
    let sang_hun = {
        let mut zhi = month_zhi;
        for _ in 0..3 {
            if zhi.is_meng() {
                break;
            }
            zhi = zhi.add(4);
        }
        zhi.add(5)
    };

    // 天鬼：找孟月，加7
    let tian_gui = {
        let mut zhi = month_zhi;
        for _ in 0..3 {
            if zhi.is_meng() {
                break;
            }
            zhi = zhi.add(4);
        }
        zhi.add(7)
    };

    // 信神：寅月酉，顺行
    let xin_shen = DiZhi::You.add((month_zhi.index() as i8 - DiZhi::Yin.index() as i8 + 12) % 12);

    // 天鸡：寅月酉，逆行
    let tian_ji = DiZhi::You.add(-((month_zhi.index() as i8 - DiZhi::Yin.index() as i8 + 12) % 12));

    // 大时：找仲月，减3
    let da_shi = {
        let mut zhi = month_zhi;
        for _ in 0..3 {
            if zhi.is_zhong() {
                break;
            }
            zhi = zhi.add(4);
        }
        zhi.add(-3)
    };

    // 小时：即月建
    let xiao_shi = month_zhi;

    // === 年支相关神煞 ===
    // 年驿马
    let nian_yi_ma = calculate_yi_ma(year_zhi);

    // 大耗：太岁六冲
    let da_hao = year_zhi.add(6);

    // 小耗：太岁后五位
    let xiao_hao = year_zhi.add(5);

    // 病符：太岁前一位
    let bing_fu = year_zhi.add(-1);

    ShenShaInfo {
        yi_ma,
        xun_kong,
        xun_qi,
        xun_yi,
        ri_qi,
        zhi_yi,
        tian_luo,
        di_wang,
        yue_yi_ma,
        tian_ma,
        huang_shu,
        huang_en,
        tian_zhao,
        tian_xi,
        sheng_qi,
        si_qi,
        san_qiu,
        wu_mu,
        gu_chen,
        gua_su,
        tian_yi_shen,
        di_yi,
        po_sui,
        yue_yan,
        xue_zhi,
        xue_ji,
        sang_che,
        sang_hun,
        tian_gui,
        xin_shen,
        tian_ji,
        da_shi,
        xiao_shi,
        nian_yi_ma,
        da_hao,
        xiao_hao,
        bing_fu,
    }
}

/// 计算驿马（通用函数）
///
/// 驿马取法：申子辰马在寅，寅午戌马在申，亥卯未马在巳，巳酉丑马在亥
fn calculate_yi_ma(zhi: DiZhi) -> DiZhi {
    // 找到所属三合局的孟月
    let mut meng = zhi;
    for _ in 0..3 {
        if meng.is_meng() {
            break;
        }
        meng = meng.add(4);
    }
    // 驿马为孟月六冲
    meng.add(6)
}

// ============================================================================
// 年命计算
// ============================================================================

/// 计算行年
///
/// 行年起法：
/// - 男命从丙寅顺行，每年一位
/// - 女命从壬申逆行，每年一位
///
/// # 参数
/// - `birth_year`: 出生年支
/// - `current_year`: 当前年支
/// - `is_male`: 是否男命
///
/// # 返回
/// 行年干支
pub fn calculate_xing_nian(
    birth_year_zhi: DiZhi,
    current_year_zhi: DiZhi,
    is_male: bool,
) -> (TianGan, DiZhi) {
    // 计算年龄（支数差距）
    let age = ((current_year_zhi.index() as i8 - birth_year_zhi.index() as i8 + 12) % 12) as i8;

    if is_male {
        // 男命从丙寅顺行
        let gan = TianGan::Bing.add(age);
        let zhi = DiZhi::Yin.add(age);
        (gan, zhi)
    } else {
        // 女命从壬申逆行
        let gan = TianGan::Ren.add(-age);
        let zhi = DiZhi::Shen.add(-age);
        (gan, zhi)
    }
}

// ============================================================================
// 格局判断
// ============================================================================

/// 格局判断结果
#[derive(Clone, Default)]
pub struct GuaTiResult {
    /// 检测到的格局列表
    pub gua_ti_list: Vec<&'static str>,
}

/// 判断所有格局
///
/// 根据式盘信息判断属于哪些格局（卦体）
pub fn judge_gua_ti(
    tian_pan: &TianPan,
    tian_jiang_pan: &TianJiangPan,
    si_ke: &SiKe,
    san_chuan: &SanChuan,
    ke_shi: KeShiType,
    day_gan: TianGan,
    day_zhi: DiZhi,
    month_zhi: DiZhi,
    year_zhi: DiZhi,
) -> GuaTiResult {
    let mut result = GuaTiResult { gua_ti_list: Vec::new() };

    // 伏吟卦
    if ke_shi == KeShiType::FuYin {
        result.gua_ti_list.push("伏吟卦");
    }

    // 返吟卦
    if ke_shi == KeShiType::FanYin {
        result.gua_ti_list.push("返吟卦");
    }

    // 连茹卦：三传递进或递退
    if is_lian_ru(san_chuan) {
        result.gua_ti_list.push("连茹卦");
    }

    // 连珠卦
    if is_lian_zhu(san_chuan, year_zhi, month_zhi, day_zhi) {
        result.gua_ti_list.push("连珠卦");
    }

    // 三奇卦：旬奇入三传
    if is_san_qi(san_chuan, day_gan, day_zhi) {
        result.gua_ti_list.push("三奇卦");
    }

    // 六仪卦：旬仪入三传
    if is_liu_yi(san_chuan, day_gan, day_zhi) {
        result.gua_ti_list.push("六仪卦");
    }

    // 度厄卦：四课三上克下或三下克上
    if is_du_e(si_ke) {
        result.gua_ti_list.push("度厄卦");
    }

    // 三光卦
    if is_san_guang(tian_jiang_pan, san_chuan, day_gan, day_zhi, month_zhi) {
        result.gua_ti_list.push("三光卦");
    }

    // 三阳卦
    if is_san_yang(tian_jiang_pan, san_chuan, day_gan, day_zhi, month_zhi) {
        result.gua_ti_list.push("三阳卦");
    }

    // 九丑卦
    if is_jiu_chou(si_ke, day_gan, day_zhi) {
        result.gua_ti_list.push("九丑卦");
    }

    // 罗网卦
    if is_luo_wang(tian_pan, si_ke, san_chuan, day_gan) {
        result.gua_ti_list.push("罗网卦");
    }

    result
}

/// 判断连茹卦
pub fn is_lian_ru(san_chuan: &SanChuan) -> bool {
    let c = san_chuan.chu.index() as i8;
    let z = san_chuan.zhong.index() as i8;
    let m = san_chuan.mo.index() as i8;

    // 递进
    if (z - c + 12) % 12 == 1 && (m - z + 12) % 12 == 1 {
        return true;
    }
    // 递退
    if (c - z + 12) % 12 == 1 && (z - m + 12) % 12 == 1 {
        return true;
    }
    false
}

/// 判断连珠卦
pub fn is_lian_zhu(san_chuan: &SanChuan, year_zhi: DiZhi, month_zhi: DiZhi, day_zhi: DiZhi) -> bool {
    let c = san_chuan.chu;
    let z = san_chuan.zhong;
    let m = san_chuan.mo;

    // 三传从孟递进
    if c.is_meng() {
        let ci = c.index() as i8;
        let zi = z.index() as i8;
        let mi = m.index() as i8;
        if (zi - ci + 12) % 12 == 1 && (mi - zi + 12) % 12 == 1 {
            return true;
        }
    }

    // 三传从季递退
    if c.is_ji() {
        let ci = c.index() as i8;
        let zi = z.index() as i8;
        let mi = m.index() as i8;
        if (ci - zi + 12) % 12 == 1 && (zi - mi + 12) % 12 == 1 {
            return true;
        }
    }

    // 年月日入三传
    let sc = [c, z, m];
    if (year_zhi == sc[0] && month_zhi == sc[1] && day_zhi == sc[2]) ||
       (day_zhi == sc[0] && month_zhi == sc[1] && year_zhi == sc[2]) {
        return true;
    }

    false
}

/// 判断三奇卦
pub fn is_san_qi(san_chuan: &SanChuan, day_gan: TianGan, day_zhi: DiZhi) -> bool {
    let xun_kong = calculate_xun_kong(day_gan, day_zhi);
    let xun_shou = xun_kong.1.add(1);

    // 旬奇
    let xun_qi = match xun_shou {
        DiZhi::Xu | DiZhi::Zi => Some(DiZhi::Chou),
        DiZhi::Shen | DiZhi::Wu => Some(DiZhi::Zi),
        DiZhi::Yin | DiZhi::Chen => Some(DiZhi::Hai),
        _ => None,
    };

    if let Some(qi) = xun_qi {
        let sc = [san_chuan.chu, san_chuan.zhong, san_chuan.mo];
        return sc.contains(&qi);
    }
    false
}

/// 判断六仪卦
pub fn is_liu_yi(san_chuan: &SanChuan, day_gan: TianGan, day_zhi: DiZhi) -> bool {
    let xun_kong = calculate_xun_kong(day_gan, day_zhi);
    let xun_shou = xun_kong.1.add(1);

    let sc = [san_chuan.chu, san_chuan.zhong, san_chuan.mo];
    sc.contains(&xun_shou)
}

/// 判断度厄卦
pub fn is_du_e(si_ke: &SiKe) -> bool {
    let kes = [&si_ke.ke1, &si_ke.ke2, &si_ke.ke3, &si_ke.ke4];

    let mut shang_ke_xia = 0; // 上克下
    let mut xia_ke_shang = 0; // 下克上

    for ke in kes.iter() {
        if ke.shang.wu_xing().ke(ke.xia.wu_xing()) {
            shang_ke_xia += 1;
        }
        if ke.xia.wu_xing().ke(ke.shang.wu_xing()) {
            xia_ke_shang += 1;
        }
    }

    shang_ke_xia == 3 || xia_ke_shang == 3
}

/// 判断三光卦
fn is_san_guang(
    tian_jiang_pan: &TianJiangPan,
    san_chuan: &SanChuan,
    day_gan: TianGan,
    day_zhi: DiZhi,
    month_zhi: DiZhi,
) -> bool {
    let month_wx = month_zhi.wu_xing();

    // 初传旺相
    let chu_wx = san_chuan.chu.wu_xing();
    let chu_ws = WangShuai::from_wu_xing(month_wx, chu_wx);
    if !matches!(chu_ws, WangShuai::Wang | WangShuai::Xiang) {
        return false;
    }

    // 日干旺相
    let gan_ws = WangShuai::from_wu_xing(month_wx, day_gan.wu_xing());
    if !matches!(gan_ws, WangShuai::Wang | WangShuai::Xiang) {
        return false;
    }

    // 日支旺相
    let zhi_ws = WangShuai::from_wu_xing(month_wx, day_zhi.wu_xing());
    if !matches!(zhi_ws, WangShuai::Wang | WangShuai::Xiang) {
        return false;
    }

    // 三传乘吉将
    let chu_jiang = tian_jiang_pan.get(san_chuan.chu);
    let zhong_jiang = tian_jiang_pan.get(san_chuan.zhong);
    let mo_jiang = tian_jiang_pan.get(san_chuan.mo);

    chu_jiang.is_auspicious() || zhong_jiang.is_auspicious() || mo_jiang.is_auspicious()
}

/// 判断三阳卦
fn is_san_yang(
    tian_jiang_pan: &TianJiangPan,
    san_chuan: &SanChuan,
    day_gan: TianGan,
    day_zhi: DiZhi,
    month_zhi: DiZhi,
) -> bool {
    // 天将顺布
    if tian_jiang_pan.is_reverse {
        return false;
    }

    // 发用旺相
    let month_wx = month_zhi.wu_xing();
    let chu_wx = san_chuan.chu.wu_xing();
    let chu_ws = WangShuai::from_wu_xing(month_wx, chu_wx);
    if !matches!(chu_ws, WangShuai::Wang | WangShuai::Xiang) {
        return false;
    }

    // 干支乘阳将（螣蛇、朱雀、六合、勾陈、青龙）
    let yang_jiang = [
        TianJiang::TengShe,
        TianJiang::ZhuQue,
        TianJiang::LiuHe,
        TianJiang::GouChen,
        TianJiang::QingLong,
    ];

    let gan_ji_gong = get_ji_gong(day_gan);
    let gan_jiang = tian_jiang_pan.get(gan_ji_gong);
    let zhi_jiang = tian_jiang_pan.get(day_zhi);

    yang_jiang.contains(&gan_jiang) && yang_jiang.contains(&zhi_jiang)
}

/// 判断九丑卦
pub fn is_jiu_chou(si_ke: &SiKe, day_gan: TianGan, day_zhi: DiZhi) -> bool {
    // 日干必须是乙、戊、己、辛、壬
    if !matches!(
        day_gan,
        TianGan::Yi | TianGan::Wu | TianGan::Ji | TianGan::Xin | TianGan::Ren
    ) {
        return false;
    }

    // 日支必须是子、卯、午、酉
    if !day_zhi.is_zhong() {
        return false;
    }

    // 支上神为丑
    si_ke.ke3.shang == DiZhi::Chou
}

/// 判断罗网卦
fn is_luo_wang(
    _tian_pan: &TianPan,
    si_ke: &SiKe,
    san_chuan: &SanChuan,
    day_gan: TianGan,
) -> bool {
    // 天罗：日干寄宫前一位
    let tian_luo = get_ji_gong(day_gan).add(1);
    // 地网：天罗六冲
    let di_wang = tian_luo.add(6);

    let check_list = [
        si_ke.ke1.shang,
        si_ke.ke3.shang,
        san_chuan.chu,
    ];

    for zhi in check_list.iter() {
        if *zhi == tian_luo || *zhi == di_wang {
            return true;
        }
    }

    false
}
