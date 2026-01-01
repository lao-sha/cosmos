//! # 紫微斗数四化飞星分析
//!
//! 本模块实现四化飞星的分析功能，包括：
//!
//! - **天干四化表**：十天干对应的四化星（禄、权、科、忌）
//! - **宫干四化**：根据宫位天干计算该宫的四化飞星
//! - **飞化落宫**：计算四化飞入哪个宫位
//! - **自化检测**：检测宫位是否有自化现象
//! - **化忌冲破**：检测化忌是否冲破对宫
//!
//! ## 四化基础概念
//!
//! 四化是紫微斗数中最重要的动态分析要素：
//! - **化禄**：主财禄、收入、机会
//! - **化权**：主权力、掌控、地位
//! - **化科**：主名声、学业、贵人
//! - **化忌**：主阻碍、是非、损失
//!
//! ## 四化来源
//!
//! 1. **生年四化**：由年干决定，影响一生
//! 2. **大限四化**：由大限宫干决定，影响十年
//! 3. **流年四化**：由流年天干决定，影响一年
//! 4. **宫干四化**：由各宫天干决定，用于飞星分析

use crate::types::*;
use super::structs::*;
use sp_std::prelude::*;

// ============================================================================
// 天干四化表
// ============================================================================

/// 天干四化表（禄、权、科、忌）
///
/// 根据《紫微斗数全书》，各天干四化如下：
/// - 甲：廉贞化禄、破军化权、武曲化科、太阳化忌
/// - 乙：天机化禄、天梁化权、紫微化科、太阴化忌
/// - 丙：天同化禄、天机化权、文昌化科、廉贞化忌
/// - 丁：太阴化禄、天同化权、天机化科、巨门化忌
/// - 戊：贪狼化禄、太阴化权、右弼化科、天机化忌
/// - 己：武曲化禄、贪狼化权、天梁化科、文曲化忌
/// - 庚：太阳化禄、武曲化权、太阴化科、天同化忌
/// - 辛：巨门化禄、太阳化权、文曲化科、文昌化忌
/// - 壬：天梁化禄、紫微化权、左辅化科、武曲化忌
/// - 癸：破军化禄、巨门化权、太阴化科、贪狼化忌
pub const SI_HUA_TABLE: [[SiHuaStar; 4]; 10] = [
    // 甲：廉贞化禄、破军化权、武曲化科、太阳化忌
    [SiHuaStar::LianZhen, SiHuaStar::PoJun, SiHuaStar::WuQu, SiHuaStar::TaiYang],
    // 乙：天机化禄、天梁化权、紫微化科、太阴化忌
    [SiHuaStar::TianJi, SiHuaStar::TianLiang, SiHuaStar::ZiWei, SiHuaStar::TaiYin],
    // 丙：天同化禄、天机化权、文昌化科、廉贞化忌
    [SiHuaStar::TianTong, SiHuaStar::TianJi, SiHuaStar::WenChang, SiHuaStar::LianZhen],
    // 丁：太阴化禄、天同化权、天机化科、巨门化忌
    [SiHuaStar::TaiYin, SiHuaStar::TianTong, SiHuaStar::TianJi, SiHuaStar::JuMen],
    // 戊：贪狼化禄、太阴化权、右弼化科、天机化忌
    [SiHuaStar::TanLang, SiHuaStar::TaiYin, SiHuaStar::YouBi, SiHuaStar::TianJi],
    // 己：武曲化禄、贪狼化权、天梁化科、文曲化忌
    [SiHuaStar::WuQu, SiHuaStar::TanLang, SiHuaStar::TianLiang, SiHuaStar::WenQu],
    // 庚：太阳化禄、武曲化权、太阴化科、天同化忌
    [SiHuaStar::TaiYang, SiHuaStar::WuQu, SiHuaStar::TaiYin, SiHuaStar::TianTong],
    // 辛：巨门化禄、太阳化权、文曲化科、文昌化忌
    [SiHuaStar::JuMen, SiHuaStar::TaiYang, SiHuaStar::WenQu, SiHuaStar::WenChang],
    // 壬：天梁化禄、紫微化权、左辅化科、武曲化忌
    [SiHuaStar::TianLiang, SiHuaStar::ZiWei, SiHuaStar::ZuoFu, SiHuaStar::WuQu],
    // 癸：破军化禄、巨门化权、太阴化科、贪狼化忌
    [SiHuaStar::PoJun, SiHuaStar::JuMen, SiHuaStar::TaiYin, SiHuaStar::TanLang],
];

/// 四化类型索引
pub const HUA_LU: usize = 0;    // 化禄
pub const HUA_QUAN: usize = 1;  // 化权
pub const HUA_KE: usize = 2;    // 化科
pub const HUA_JI: usize = 3;    // 化忌

// ============================================================================
// 获取四化星
// ============================================================================

/// 根据天干获取四化星
///
/// # 参数
/// - `tian_gan`: 天干
///
/// # 返回
/// 四化星数组 [化禄星, 化权星, 化科星, 化忌星]
pub fn get_si_hua_stars(tian_gan: TianGan) -> [SiHuaStar; 4] {
    SI_HUA_TABLE[tian_gan.index() as usize]
}

/// 根据天干获取指定四化类型的星
///
/// # 参数
/// - `tian_gan`: 天干
/// - `si_hua`: 四化类型
///
/// # 返回
/// 对应的四化星
pub fn get_si_hua_star(tian_gan: TianGan, si_hua: SiHua) -> SiHuaStar {
    SI_HUA_TABLE[tian_gan.index() as usize][si_hua as usize]
}

/// 获取宫干四化星
///
/// # 参数
/// - `palace`: 宫位数据
///
/// # 返回
/// 该宫干对应的四化星数组
pub fn get_gong_gan_si_hua(palace: &Palace) -> [SiHuaStar; 4] {
    get_si_hua_stars(palace.tian_gan)
}

// ============================================================================
// 飞化落宫计算
// ============================================================================

/// 查找星曜所在宫位
///
/// # 参数
/// - `palaces`: 十二宫数据
/// - `star`: 要查找的星曜
///
/// # 返回
/// 星曜所在宫位索引（0-11），None 表示未找到
pub fn find_star_palace(palaces: &[Palace; 12], star: SiHuaStar) -> Option<u8> {
    // 根据星曜类型决定查找方式
    if star.is_zhu_xing() {
        // 查找主星
        if let Some(zhu_xing) = star.to_zhu_xing() {
            for (i, palace) in palaces.iter().enumerate() {
                for j in 0..3 {
                    if palace.zhu_xing[j] == Some(zhu_xing) {
                        return Some(i as u8);
                    }
                }
            }
        }
    } else {
        // 查找辅星（六吉星）
        if let Some(liu_ji) = star.to_liu_ji_xing() {
            let liu_ji_idx = liu_ji as usize;
            for (i, palace) in palaces.iter().enumerate() {
                if palace.liu_ji[liu_ji_idx] {
                    return Some(i as u8);
                }
            }
        }
    }
    None
}

/// 计算宫干四化飞入宫位
///
/// # 参数
/// - `palaces`: 十二宫数据
/// - `source_palace_idx`: 发出四化的宫位索引
///
/// # 返回
/// 四化飞入的宫位索引 [化禄落宫, 化权落宫, 化科落宫, 化忌落宫]
/// 255 表示未找到（星曜不在盘上）
pub fn calculate_fei_hua(palaces: &[Palace; 12], source_palace_idx: u8) -> [u8; 4] {
    if source_palace_idx >= 12 {
        return [255; 4];
    }

    let palace = &palaces[source_palace_idx as usize];
    let si_hua_stars = get_gong_gan_si_hua(palace);

    let mut result = [255u8; 4];

    for (i, star) in si_hua_stars.iter().enumerate() {
        if let Some(palace_idx) = find_star_palace(palaces, *star) {
            result[i] = palace_idx;
        }
    }

    result
}

/// 计算多宫四化飞入
///
/// 计算命宫、财帛、官禄、夫妻四宫的四化飞入情况
///
/// # 参数
/// - `palaces`: 十二宫数据
/// - `ming_gong_pos`: 命宫位置
///
/// # 返回
/// 四个宫位的四化飞入情况
pub fn calculate_key_palaces_fei_hua(
    palaces: &[Palace; 12],
    ming_gong_pos: u8,
) -> [[u8; 4]; 4] {
    let cai_bo_pos = (ming_gong_pos + 8) % 12;
    let guan_lu_pos = (ming_gong_pos + 4) % 12;
    let fu_qi_pos = (ming_gong_pos + 10) % 12;

    [
        calculate_fei_hua(palaces, ming_gong_pos),
        calculate_fei_hua(palaces, cai_bo_pos),
        calculate_fei_hua(palaces, guan_lu_pos),
        calculate_fei_hua(palaces, fu_qi_pos),
    ]
}

// ============================================================================
// 自化检测
// ============================================================================

/// 检查宫位是否有自化
///
/// 自化：宫干四化的星刚好在本宫
///
/// # 参数
/// - `palaces`: 十二宫数据
/// - `palace_idx`: 宫位索引
///
/// # 返回
/// 自化类型列表（可能有多种自化）
pub fn check_zi_hua(palaces: &[Palace; 12], palace_idx: u8) -> Vec<SiHua> {
    if palace_idx >= 12 {
        return Vec::new();
    }

    let fei_hua = calculate_fei_hua(palaces, palace_idx);
    let mut result = Vec::new();

    // 检查每种四化是否飞回本宫
    for (i, &target_palace) in fei_hua.iter().enumerate() {
        if target_palace == palace_idx {
            result.push(match i {
                0 => SiHua::HuaLu,
                1 => SiHua::HuaQuan,
                2 => SiHua::HuaKe,
                _ => SiHua::HuaJi,
            });
        }
    }

    result
}

/// 检查所有宫位的自化情况
///
/// # 参数
/// - `palaces`: 十二宫数据
///
/// # 返回
/// 自化宫位位标志（12 bits，bit n = 1 表示第 n 宫有自化）
pub fn check_all_zi_hua(palaces: &[Palace; 12]) -> u16 {
    let mut flags: u16 = 0;

    for i in 0..12 {
        if !check_zi_hua(palaces, i).is_empty() {
            flags |= 1 << i;
        }
    }

    flags
}

/// 检查特定宫位是否有特定类型的自化
///
/// # 参数
/// - `palaces`: 十二宫数据
/// - `palace_idx`: 宫位索引
/// - `si_hua`: 四化类型
///
/// # 返回
/// 是否有该类型的自化
pub fn has_zi_hua(palaces: &[Palace; 12], palace_idx: u8, si_hua: SiHua) -> bool {
    if palace_idx >= 12 {
        return false;
    }

    let fei_hua = calculate_fei_hua(palaces, palace_idx);
    fei_hua[si_hua as usize] == palace_idx
}

// ============================================================================
// 化忌冲破检测
// ============================================================================

/// 检查化忌是否冲破对宫
///
/// 化忌飞入对宫（冲宫）会对对宫造成破坏
///
/// # 参数
/// - `palaces`: 十二宫数据
/// - `source_palace_idx`: 发出四化的宫位索引
///
/// # 返回
/// 被化忌冲破的宫位索引，None 表示没有冲破
pub fn check_hua_ji_chong_po(palaces: &[Palace; 12], source_palace_idx: u8) -> Option<u8> {
    if source_palace_idx >= 12 {
        return None;
    }

    let fei_hua = calculate_fei_hua(palaces, source_palace_idx);
    let hua_ji_palace = fei_hua[HUA_JI];

    if hua_ji_palace >= 12 {
        return None;
    }

    // 计算源宫的对宫
    let dui_gong = (source_palace_idx + 6) % 12;

    // 检查化忌是否落入对宫（冲破）
    if hua_ji_palace == dui_gong {
        Some(dui_gong)
    } else {
        None
    }
}

/// 检查所有宫位的化忌冲破情况
///
/// # 参数
/// - `palaces`: 十二宫数据
///
/// # 返回
/// 被化忌冲破的宫位位标志（12 bits）
pub fn check_all_hua_ji_chong_po(palaces: &[Palace; 12]) -> u16 {
    let mut flags: u16 = 0;

    for i in 0..12 {
        if let Some(chong_po_palace) = check_hua_ji_chong_po(palaces, i) {
            flags |= 1 << chong_po_palace;
        }
    }

    flags
}

/// 检查命宫是否被化忌冲破
///
/// # 参数
/// - `palaces`: 十二宫数据
/// - `ming_gong_pos`: 命宫位置
///
/// # 返回
/// 冲破命宫的宫位索引列表
pub fn check_ming_gong_hua_ji_chong(
    palaces: &[Palace; 12],
    ming_gong_pos: u8,
) -> Vec<u8> {
    let mut result = Vec::new();

    for i in 0..12 {
        let fei_hua = calculate_fei_hua(palaces, i);
        if fei_hua[HUA_JI] == ming_gong_pos {
            result.push(i);
        }
    }

    result
}

// ============================================================================
// 综合四化分析
// ============================================================================

/// 执行综合四化分析
///
/// # 参数
/// - `palaces`: 十二宫数据
/// - `ming_gong_pos`: 命宫位置
/// - `sheng_nian_si_hua`: 生年四化星
///
/// # 返回
/// 完整的四化分析结果
pub fn analyze_si_hua(
    palaces: &[Palace; 12],
    ming_gong_pos: u8,
    sheng_nian_si_hua: [SiHuaStar; 4],
) -> SiHuaAnalysis {
    let cai_bo_pos = (ming_gong_pos + 8) % 12;
    let guan_lu_pos = (ming_gong_pos + 4) % 12;
    let fu_qi_pos = (ming_gong_pos + 10) % 12;

    SiHuaAnalysis {
        sheng_nian_si_hua,
        ming_gong_fei_ru: calculate_fei_hua(palaces, ming_gong_pos),
        cai_bo_fei_ru: calculate_fei_hua(palaces, cai_bo_pos),
        guan_lu_fei_ru: calculate_fei_hua(palaces, guan_lu_pos),
        fu_qi_fei_ru: calculate_fei_hua(palaces, fu_qi_pos),
        zi_hua_palaces: check_all_zi_hua(palaces),
        hua_ji_chong_po: check_all_hua_ji_chong_po(palaces),
    }
}

// ============================================================================
// 四化影响评估
// ============================================================================

/// 四化影响类型
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SiHuaImpactType {
    /// 非常吉利（化禄+化科同宫等）
    VeryAuspicious,
    /// 吉利（有化禄或化科）
    Auspicious,
    /// 中性
    Neutral,
    /// 不利（有化忌）
    Inauspicious,
    /// 非常不利（化忌冲破或自化忌）
    VeryInauspicious,
}

/// 评估宫位的四化影响
///
/// # 参数
/// - `palaces`: 十二宫数据
/// - `palace_idx`: 宫位索引
/// - `analysis`: 四化分析结果
///
/// # 返回
/// 四化影响类型
pub fn evaluate_si_hua_impact(
    palaces: &[Palace; 12],
    palace_idx: u8,
    analysis: &SiHuaAnalysis,
) -> SiHuaImpactType {
    if palace_idx >= 12 {
        return SiHuaImpactType::Neutral;
    }

    let palace = &palaces[palace_idx as usize];

    // 检查是否有化忌
    let has_hua_ji = palace.si_hua.iter().any(|s| *s == Some(SiHua::HuaJi));

    // 检查是否被化忌冲破
    let is_chong_po = analysis.has_hua_ji_chong_po(palace_idx);

    // 检查是否有自化忌
    let has_zi_hua_ji = has_zi_hua(palaces, palace_idx, SiHua::HuaJi);

    // 检查是否有吉化
    let has_hua_lu = palace.si_hua.iter().any(|s| *s == Some(SiHua::HuaLu));
    let has_hua_quan = palace.si_hua.iter().any(|s| *s == Some(SiHua::HuaQuan));
    let has_hua_ke = palace.si_hua.iter().any(|s| *s == Some(SiHua::HuaKe));

    // 综合判断
    if is_chong_po || has_zi_hua_ji {
        SiHuaImpactType::VeryInauspicious
    } else if has_hua_ji {
        SiHuaImpactType::Inauspicious
    } else if has_hua_lu && (has_hua_quan || has_hua_ke) {
        SiHuaImpactType::VeryAuspicious
    } else if has_hua_lu || has_hua_quan || has_hua_ke {
        SiHuaImpactType::Auspicious
    } else {
        SiHuaImpactType::Neutral
    }
}

/// 获取四化影响分数
///
/// # 参数
/// - `impact_type`: 四化影响类型
///
/// # 返回
/// 影响分数（-30 ~ +20）
pub fn get_si_hua_impact_score(impact_type: SiHuaImpactType) -> i8 {
    match impact_type {
        SiHuaImpactType::VeryAuspicious => 20,
        SiHuaImpactType::Auspicious => 10,
        SiHuaImpactType::Neutral => 0,
        SiHuaImpactType::Inauspicious => -15,
        SiHuaImpactType::VeryInauspicious => -30,
    }
}

// ============================================================================
// 四化关系描述
// ============================================================================

/// 获取四化类型名称
pub fn get_si_hua_name(si_hua: SiHua) -> &'static str {
    si_hua.name()
}

/// 获取四化影响简述
///
/// # 参数
/// - `si_hua`: 四化类型
/// - `target_gong`: 目标宫位
///
/// # 返回
/// 影响简述
pub fn get_si_hua_impact_brief(si_hua: SiHua, target_gong: GongWei) -> &'static str {
    match (si_hua, target_gong) {
        // 化禄的影响
        (SiHua::HuaLu, GongWei::MingGong) => "财运亨通，一生富裕",
        (SiHua::HuaLu, GongWei::CaiBo) => "财源广进，收入丰厚",
        (SiHua::HuaLu, GongWei::GuanLu) => "事业有成，升职加薪",
        (SiHua::HuaLu, GongWei::FuQi) => "婚姻美满，配偶富裕",
        (SiHua::HuaLu, GongWei::FuDe) => "福泽深厚，生活安乐",
        (SiHua::HuaLu, GongWei::TianZhai) => "不动产运佳，家宅兴旺",
        (SiHua::HuaLu, _) => "有利发展",

        // 化权的影响
        (SiHua::HuaQuan, GongWei::MingGong) => "性格刚强，有领导力",
        (SiHua::HuaQuan, GongWei::GuanLu) => "事业权重，掌控力强",
        (SiHua::HuaQuan, GongWei::FuQi) => "配偶强势，需要包容",
        (SiHua::HuaQuan, _) => "有掌控力",

        // 化科的影响
        (SiHua::HuaKe, GongWei::MingGong) => "聪明有文采，贵人相助",
        (SiHua::HuaKe, GongWei::GuanLu) => "名声在外，学术有成",
        (SiHua::HuaKe, GongWei::FuMu) => "长辈有学识，家教良好",
        (SiHua::HuaKe, _) => "有贵气",

        // 化忌的影响
        (SiHua::HuaJi, GongWei::MingGong) => "一生操劳，多波折",
        (SiHua::HuaJi, GongWei::CaiBo) => "破财损耗，理财不利",
        (SiHua::HuaJi, GongWei::GuanLu) => "事业阻碍，工作不顺",
        (SiHua::HuaJi, GongWei::FuQi) => "感情波折，婚姻不顺",
        (SiHua::HuaJi, GongWei::JiE) => "健康有虞，需多注意",
        (SiHua::HuaJi, _) => "有阻碍",
    }
}

/// 获取自化类型的含义
///
/// # 参数
/// - `si_hua`: 自化类型
///
/// # 返回
/// 自化含义
pub fn get_zi_hua_meaning(si_hua: SiHua) -> &'static str {
    match si_hua {
        SiHua::HuaLu => "自化禄：财来财去，虽有收入但难留存",
        SiHua::HuaQuan => "自化权：刚愎自用，不听人言",
        SiHua::HuaKe => "自化科：虚名在外，难以落实",
        SiHua::HuaJi => "自化忌：自找麻烦，容易招惹是非",
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_empty_palaces() -> [Palace; 12] {
        let mut palaces: [Palace; 12] = Default::default();
        for (i, palace) in palaces.iter_mut().enumerate() {
            palace.gong_wei = GongWei::from_index(i as u8);
            palace.di_zhi = DiZhi::from_index(i as u8);
            palace.tian_gan = TianGan::from_index(i as u8);
        }
        palaces
    }

    #[test]
    fn test_get_si_hua_stars() {
        // 甲干四化
        let jia_si_hua = get_si_hua_stars(TianGan::Jia);
        assert_eq!(jia_si_hua[0], SiHuaStar::LianZhen);  // 化禄
        assert_eq!(jia_si_hua[1], SiHuaStar::PoJun);     // 化权
        assert_eq!(jia_si_hua[2], SiHuaStar::WuQu);      // 化科
        assert_eq!(jia_si_hua[3], SiHuaStar::TaiYang);   // 化忌

        // 乙干四化
        let yi_si_hua = get_si_hua_stars(TianGan::Yi);
        assert_eq!(yi_si_hua[0], SiHuaStar::TianJi);
        assert_eq!(yi_si_hua[1], SiHuaStar::TianLiang);
        assert_eq!(yi_si_hua[2], SiHuaStar::ZiWei);
        assert_eq!(yi_si_hua[3], SiHuaStar::TaiYin);
    }

    #[test]
    fn test_get_si_hua_star() {
        assert_eq!(get_si_hua_star(TianGan::Jia, SiHua::HuaLu), SiHuaStar::LianZhen);
        assert_eq!(get_si_hua_star(TianGan::Jia, SiHua::HuaJi), SiHuaStar::TaiYang);
        assert_eq!(get_si_hua_star(TianGan::Gui, SiHua::HuaLu), SiHuaStar::PoJun);
    }

    #[test]
    fn test_find_star_palace() {
        let mut palaces = create_empty_palaces();
        palaces[3].zhu_xing = [Some(ZhuXing::ZiWei), None, None];

        let result = find_star_palace(&palaces, SiHuaStar::ZiWei);
        assert_eq!(result, Some(3));

        // 查找不存在的星
        let result2 = find_star_palace(&palaces, SiHuaStar::TianFu);
        assert_eq!(result2, None);
    }

    #[test]
    fn test_find_liu_ji_star_palace() {
        let mut palaces = create_empty_palaces();
        palaces[5].liu_ji = [true, false, false, false, false, false]; // 文昌在第5宫

        let result = find_star_palace(&palaces, SiHuaStar::WenChang);
        assert_eq!(result, Some(5));

        palaces[8].liu_ji = [false, true, false, false, false, false]; // 文曲在第8宫
        let result2 = find_star_palace(&palaces, SiHuaStar::WenQu);
        assert_eq!(result2, Some(8));
    }

    #[test]
    fn test_calculate_fei_hua() {
        let mut palaces = create_empty_palaces();
        // 设置第0宫天干为甲
        palaces[0].tian_gan = TianGan::Jia;
        // 甲干四化：廉贞禄、破军权、武曲科、太阳忌
        palaces[2].zhu_xing = [Some(ZhuXing::LianZhen), None, None];
        palaces[4].zhu_xing = [Some(ZhuXing::PoJun), None, None];
        palaces[6].zhu_xing = [Some(ZhuXing::WuQu), None, None];
        palaces[8].zhu_xing = [Some(ZhuXing::TaiYang), None, None];

        let fei_hua = calculate_fei_hua(&palaces, 0);
        assert_eq!(fei_hua[0], 2);  // 化禄落第2宫
        assert_eq!(fei_hua[1], 4);  // 化权落第4宫
        assert_eq!(fei_hua[2], 6);  // 化科落第6宫
        assert_eq!(fei_hua[3], 8);  // 化忌落第8宫
    }

    #[test]
    fn test_check_zi_hua() {
        let mut palaces = create_empty_palaces();
        // 第0宫天干为甲，甲干化禄星是廉贞
        palaces[0].tian_gan = TianGan::Jia;
        // 廉贞在第0宫（自化禄）
        palaces[0].zhu_xing = [Some(ZhuXing::LianZhen), None, None];

        let zi_hua = check_zi_hua(&palaces, 0);
        assert!(zi_hua.contains(&SiHua::HuaLu));
    }

    #[test]
    fn test_check_hua_ji_chong_po() {
        let mut palaces = create_empty_palaces();
        // 第0宫天干为甲，甲干化忌星是太阳
        palaces[0].tian_gan = TianGan::Jia;
        // 太阳在第6宫（第0宫的对宫，形成化忌冲破）
        palaces[6].zhu_xing = [Some(ZhuXing::TaiYang), None, None];

        let chong_po = check_hua_ji_chong_po(&palaces, 0);
        assert_eq!(chong_po, Some(6));
    }

    #[test]
    fn test_check_hua_ji_chong_po_none() {
        let mut palaces = create_empty_palaces();
        palaces[0].tian_gan = TianGan::Jia;
        // 太阳在第3宫（不是对宫）
        palaces[3].zhu_xing = [Some(ZhuXing::TaiYang), None, None];

        let chong_po = check_hua_ji_chong_po(&palaces, 0);
        assert_eq!(chong_po, None);
    }

    #[test]
    fn test_analyze_si_hua() {
        let mut palaces = create_empty_palaces();
        for (i, palace) in palaces.iter_mut().enumerate() {
            palace.tian_gan = TianGan::from_index(i as u8);
        }

        // 放置一些星曜
        palaces[0].zhu_xing = [Some(ZhuXing::ZiWei), None, None];
        palaces[2].zhu_xing = [Some(ZhuXing::TianJi), None, None];
        palaces[4].zhu_xing = [Some(ZhuXing::TaiYang), None, None];

        let sheng_nian_si_hua = [
            SiHuaStar::TianJi,
            SiHuaStar::TianLiang,
            SiHuaStar::ZiWei,
            SiHuaStar::TaiYin,
        ];

        let analysis = analyze_si_hua(&palaces, 0, sheng_nian_si_hua);

        assert_eq!(analysis.sheng_nian_si_hua[0], SiHuaStar::TianJi);
        // 检查命宫飞化（命宫在第0宫，天干为甲）
        // 甲干四化：廉贞禄、破军权、武曲科、太阳忌
    }

    #[test]
    fn test_check_all_zi_hua() {
        let mut palaces = create_empty_palaces();

        // 第0宫天干为甲，廉贞在第0宫（自化禄）
        palaces[0].tian_gan = TianGan::Jia;
        palaces[0].zhu_xing = [Some(ZhuXing::LianZhen), None, None];

        // 第1宫天干为乙，天机在第1宫（自化禄）
        palaces[1].tian_gan = TianGan::Yi;
        palaces[1].zhu_xing = [Some(ZhuXing::TianJi), None, None];

        let flags = check_all_zi_hua(&palaces);
        assert!(flags & (1 << 0) != 0); // 第0宫有自化
        assert!(flags & (1 << 1) != 0); // 第1宫有自化
        assert!(flags & (1 << 2) == 0); // 第2宫无自化
    }

    #[test]
    fn test_evaluate_si_hua_impact() {
        let mut palaces = create_empty_palaces();
        palaces[0].si_hua = [Some(SiHua::HuaLu), Some(SiHua::HuaKe), None, None];

        let analysis = SiHuaAnalysis::default();
        let impact = evaluate_si_hua_impact(&palaces, 0, &analysis);
        assert_eq!(impact, SiHuaImpactType::VeryAuspicious);
    }

    #[test]
    fn test_evaluate_si_hua_impact_inauspicious() {
        let mut palaces = create_empty_palaces();
        palaces[0].si_hua = [Some(SiHua::HuaJi), None, None, None];

        let analysis = SiHuaAnalysis::default();
        let impact = evaluate_si_hua_impact(&palaces, 0, &analysis);
        assert_eq!(impact, SiHuaImpactType::Inauspicious);
    }

    #[test]
    fn test_get_si_hua_impact_score() {
        assert_eq!(get_si_hua_impact_score(SiHuaImpactType::VeryAuspicious), 20);
        assert_eq!(get_si_hua_impact_score(SiHuaImpactType::Auspicious), 10);
        assert_eq!(get_si_hua_impact_score(SiHuaImpactType::Neutral), 0);
        assert_eq!(get_si_hua_impact_score(SiHuaImpactType::Inauspicious), -15);
        assert_eq!(get_si_hua_impact_score(SiHuaImpactType::VeryInauspicious), -30);
    }

    #[test]
    fn test_si_hua_table_completeness() {
        // 确保所有10个天干都有四化定义
        for i in 0..10 {
            let tian_gan = TianGan::from_index(i);
            let si_hua = get_si_hua_stars(tian_gan);
            // 每个四化都应该是有效的星曜
            for star in si_hua.iter() {
                assert!(star.name().len() > 0);
            }
        }
    }

    #[test]
    fn test_get_zi_hua_meaning() {
        assert!(get_zi_hua_meaning(SiHua::HuaLu).contains("财来财去"));
        assert!(get_zi_hua_meaning(SiHua::HuaJi).contains("自找麻烦"));
    }

    #[test]
    fn test_get_si_hua_impact_brief() {
        let brief = get_si_hua_impact_brief(SiHua::HuaLu, GongWei::MingGong);
        assert!(brief.contains("财运"));

        let brief2 = get_si_hua_impact_brief(SiHua::HuaJi, GongWei::CaiBo);
        assert!(brief2.contains("破财"));
    }

    // ========================================================================
    // 补充测试：更多四化分析测试
    // ========================================================================

    #[test]
    fn test_all_tian_gan_si_hua() {
        // 测试所有天干的四化定义
        // 甲：廉贞化禄、破军化权、武曲化科、太阳化忌
        let jia = get_si_hua_stars(TianGan::Jia);
        assert_eq!(jia[HUA_LU], SiHuaStar::LianZhen);
        assert_eq!(jia[HUA_QUAN], SiHuaStar::PoJun);
        assert_eq!(jia[HUA_KE], SiHuaStar::WuQu);
        assert_eq!(jia[HUA_JI], SiHuaStar::TaiYang);

        // 乙：天机化禄、天梁化权、紫微化科、太阴化忌
        let yi = get_si_hua_stars(TianGan::Yi);
        assert_eq!(yi[HUA_LU], SiHuaStar::TianJi);
        assert_eq!(yi[HUA_QUAN], SiHuaStar::TianLiang);
        assert_eq!(yi[HUA_KE], SiHuaStar::ZiWei);
        assert_eq!(yi[HUA_JI], SiHuaStar::TaiYin);

        // 丙：天同化禄、天机化权、文昌化科、廉贞化忌
        let bing = get_si_hua_stars(TianGan::Bing);
        assert_eq!(bing[HUA_LU], SiHuaStar::TianTong);
        assert_eq!(bing[HUA_QUAN], SiHuaStar::TianJi);
        assert_eq!(bing[HUA_KE], SiHuaStar::WenChang);
        assert_eq!(bing[HUA_JI], SiHuaStar::LianZhen);

        // 丁：太阴化禄、天同化权、天机化科、巨门化忌
        let ding = get_si_hua_stars(TianGan::Ding);
        assert_eq!(ding[HUA_LU], SiHuaStar::TaiYin);
        assert_eq!(ding[HUA_KE], SiHuaStar::TianJi);
        assert_eq!(ding[HUA_JI], SiHuaStar::JuMen);

        // 戊：贪狼化禄、太阴化权、右弼化科、天机化忌
        let wu = get_si_hua_stars(TianGan::Wu);
        assert_eq!(wu[HUA_LU], SiHuaStar::TanLang);
        assert_eq!(wu[HUA_JI], SiHuaStar::TianJi);

        // 己：武曲化禄、贪狼化权、天梁化科、文曲化忌
        let ji = get_si_hua_stars(TianGan::Ji);
        assert_eq!(ji[HUA_LU], SiHuaStar::WuQu);
        assert_eq!(ji[HUA_JI], SiHuaStar::WenQu);

        // 庚：太阳化禄、武曲化权、太阴化科、天同化忌
        let geng = get_si_hua_stars(TianGan::Geng);
        assert_eq!(geng[HUA_LU], SiHuaStar::TaiYang);
        assert_eq!(geng[HUA_JI], SiHuaStar::TianTong);

        // 辛：巨门化禄、太阳化权、文曲化科、文昌化忌
        let xin = get_si_hua_stars(TianGan::Xin);
        assert_eq!(xin[HUA_LU], SiHuaStar::JuMen);
        assert_eq!(xin[HUA_JI], SiHuaStar::WenChang);

        // 壬：天梁化禄、紫微化权、左辅化科、武曲化忌
        let ren = get_si_hua_stars(TianGan::Ren);
        assert_eq!(ren[HUA_LU], SiHuaStar::TianLiang);
        assert_eq!(ren[HUA_JI], SiHuaStar::WuQu);

        // 癸：破军化禄、巨门化权、太阴化科、贪狼化忌
        let gui = get_si_hua_stars(TianGan::Gui);
        assert_eq!(gui[HUA_LU], SiHuaStar::PoJun);
        assert_eq!(gui[HUA_JI], SiHuaStar::TanLang);
    }

    #[test]
    fn test_evaluate_si_hua_impact_neutral() {
        let palaces = create_empty_palaces();
        let analysis = SiHuaAnalysis::default();
        let impact = evaluate_si_hua_impact(&palaces, 0, &analysis);
        assert_eq!(impact, SiHuaImpactType::Neutral);
    }

    #[test]
    fn test_evaluate_si_hua_impact_auspicious() {
        let mut palaces = create_empty_palaces();
        palaces[0].si_hua = [Some(SiHua::HuaLu), None, None, None];

        let analysis = SiHuaAnalysis::default();
        let impact = evaluate_si_hua_impact(&palaces, 0, &analysis);
        assert_eq!(impact, SiHuaImpactType::Auspicious);
    }

    #[test]
    fn test_evaluate_si_hua_impact_out_of_range() {
        let palaces = create_empty_palaces();
        let analysis = SiHuaAnalysis::default();
        // 测试超出范围的索引
        let impact = evaluate_si_hua_impact(&palaces, 12, &analysis);
        assert_eq!(impact, SiHuaImpactType::Neutral);

        let impact2 = evaluate_si_hua_impact(&palaces, 255, &analysis);
        assert_eq!(impact2, SiHuaImpactType::Neutral);
    }

    #[test]
    fn test_find_star_palace_with_liu_sha() {
        let mut palaces = create_empty_palaces();
        // 测试辅星定位
        palaces[3].liu_ji = [false, false, true, false, false, false]; // 左辅
        let result = find_star_palace(&palaces, SiHuaStar::ZuoFu);
        assert_eq!(result, Some(3));

        palaces[7].liu_ji = [false, false, false, true, false, false]; // 右弼
        let result2 = find_star_palace(&palaces, SiHuaStar::YouBi);
        assert_eq!(result2, Some(7));
    }

    #[test]
    fn test_has_zi_hua() {
        let mut palaces = create_empty_palaces();
        // 第0宫天干为甲，甲干化禄星是廉贞
        palaces[0].tian_gan = TianGan::Jia;
        // 廉贞在第0宫（自化禄）
        palaces[0].zhu_xing = [Some(ZhuXing::LianZhen), None, None];

        assert!(has_zi_hua(&palaces, 0, SiHua::HuaLu));
        assert!(!has_zi_hua(&palaces, 0, SiHua::HuaJi));
    }

    #[test]
    fn test_si_hua_analysis_default() {
        let analysis = SiHuaAnalysis::default();
        // 检查默认值
        for star in analysis.sheng_nian_si_hua.iter() {
            assert_eq!(*star, SiHuaStar::ZiWei); // 默认值
        }
        for pos in analysis.ming_gong_fei_ru.iter() {
            assert_eq!(*pos, 0);
        }
    }

    #[test]
    fn test_si_hua_analysis_has_hua_ji_chong_po() {
        let mut analysis = SiHuaAnalysis::default();
        analysis.hua_ji_chong_po = 0b0000_0101; // 第0、2宫有化忌冲破

        assert!(analysis.has_hua_ji_chong_po(0));
        assert!(!analysis.has_hua_ji_chong_po(1));
        assert!(analysis.has_hua_ji_chong_po(2));
        assert!(!analysis.has_hua_ji_chong_po(12)); // 超出范围
    }

    #[test]
    fn test_si_hua_analysis_has_zi_hua() {
        let mut analysis = SiHuaAnalysis::default();
        analysis.zi_hua_palaces = 0b0000_1010; // 第1、3宫有自化

        assert!(!analysis.has_zi_hua(0));
        assert!(analysis.has_zi_hua(1));
        assert!(!analysis.has_zi_hua(2));
        assert!(analysis.has_zi_hua(3));
    }

    #[test]
    fn test_find_multiple_stars() {
        let mut palaces = create_empty_palaces();
        // 紫微在第0宫
        palaces[0].zhu_xing = [Some(ZhuXing::ZiWei), None, None];
        // 天府在第6宫
        palaces[6].zhu_xing = [Some(ZhuXing::TianFu), None, None];
        // 太阳在第3宫
        palaces[3].zhu_xing = [Some(ZhuXing::TaiYang), None, None];

        assert_eq!(find_star_palace(&palaces, SiHuaStar::ZiWei), Some(0));
        assert_eq!(find_star_palace(&palaces, SiHuaStar::TianFu), Some(6));
        assert_eq!(find_star_palace(&palaces, SiHuaStar::TaiYang), Some(3));
        assert_eq!(find_star_palace(&palaces, SiHuaStar::TaiYin), None);
    }

    #[test]
    fn test_calculate_fei_hua_star_not_found() {
        let mut palaces = create_empty_palaces();
        palaces[0].tian_gan = TianGan::Jia;
        // 甲干四化的星曜都不在命盘上
        // 应该返回默认值255表示未找到

        let fei_hua = calculate_fei_hua(&palaces, 0);
        // 当星曜不在命盘时，应返回255
        for pos in fei_hua.iter() {
            assert_eq!(*pos, 255);
        }
    }

    #[test]
    fn test_get_si_hua_name() {
        assert_eq!(get_si_hua_name(SiHua::HuaLu), "化禄");
        assert_eq!(get_si_hua_name(SiHua::HuaQuan), "化权");
        assert_eq!(get_si_hua_name(SiHua::HuaKe), "化科");
        assert_eq!(get_si_hua_name(SiHua::HuaJi), "化忌");
    }

    #[test]
    fn test_si_hua_impact_brief_all_cases() {
        // 测试化权的影响
        let brief = get_si_hua_impact_brief(SiHua::HuaQuan, GongWei::MingGong);
        assert!(brief.contains("领导力"));

        let brief2 = get_si_hua_impact_brief(SiHua::HuaQuan, GongWei::GuanLu);
        assert!(brief2.contains("事业"));

        // 测试化科的影响
        let brief3 = get_si_hua_impact_brief(SiHua::HuaKe, GongWei::MingGong);
        assert!(brief3.contains("聪明"));

        // 测试化忌的影响
        let brief4 = get_si_hua_impact_brief(SiHua::HuaJi, GongWei::JiE);
        assert!(brief4.contains("健康"));

        // 测试默认情况
        let brief5 = get_si_hua_impact_brief(SiHua::HuaLu, GongWei::JiE);
        assert!(brief5.contains("发展"));
    }

    #[test]
    fn test_check_all_hua_ji_chong_po() {
        let mut palaces = create_empty_palaces();
        // 第0宫天干为甲，甲干化忌星是太阳
        palaces[0].tian_gan = TianGan::Jia;
        // 太阳在第6宫（对宫），形成化忌冲破
        palaces[6].zhu_xing = [Some(ZhuXing::TaiYang), None, None];

        let flags = check_all_hua_ji_chong_po(&palaces);
        // 第6宫被第0宫化忌冲破
        assert!(flags & (1 << 6) != 0);
    }
}
