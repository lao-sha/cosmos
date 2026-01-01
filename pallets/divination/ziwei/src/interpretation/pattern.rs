//! # 紫微斗数格局识别算法
//!
//! 本模块实现命盘格局的识别功能，包括：
//!
//! - **富贵格局**：紫府同宫、紫府朝垣、君臣庆会等（14种）
//! - **权贵格局**：三奇嘉会、双禄夹命、左右夹命等（8种）
//! - **凶格**：羊陀夹命、火铃夹命、四煞冲命等（10种）
//!
//! ## 格局识别原则
//!
//! 1. 优先识别富贵格局，因其对命格影响最大
//! 2. 格局强度根据星曜亮度和会照程度计算
//! 3. 同一命盘可能存在多个格局（吉凶并存）

use frame_support::BoundedVec;
use frame_support::pallet_prelude::ConstU32;

use crate::types::*;
use super::enums::*;
use super::structs::*;

// ============================================================================
// 辅助函数 - 宫位星曜检查
// ============================================================================

/// 检查宫位是否有指定主星
///
/// # 参数
/// - `palace`: 宫位数据
/// - `star`: 要检查的主星
///
/// # 返回
/// 是否包含该主星
pub fn gong_has_star(palace: &Palace, star: ZhuXing) -> bool {
    palace.zhu_xing.iter().any(|s| *s == Some(star))
}

/// 检查宫位是否包含所有指定主星
///
/// # 参数
/// - `palace`: 宫位数据
/// - `stars`: 要检查的主星列表
///
/// # 返回
/// 是否包含所有指定主星
pub fn gong_has_all_stars(palace: &Palace, stars: &[ZhuXing]) -> bool {
    stars.iter().all(|star| gong_has_star(palace, *star))
}

/// 检查宫位是否包含任一指定主星
///
/// # 参数
/// - `palace`: 宫位数据
/// - `stars`: 要检查的主星列表
///
/// # 返回
/// 是否包含任一指定主星
pub fn gong_has_any_star(palace: &Palace, stars: &[ZhuXing]) -> bool {
    stars.iter().any(|star| gong_has_star(palace, *star))
}

/// 检查宫位是否有指定六吉星
///
/// # 参数
/// - `palace`: 宫位数据
/// - `liu_ji_index`: 六吉星索引 (0=文昌, 1=文曲, 2=左辅, 3=右弼, 4=天魁, 5=天钺)
pub fn gong_has_liu_ji(palace: &Palace, liu_ji_index: usize) -> bool {
    liu_ji_index < 6 && palace.liu_ji[liu_ji_index]
}

/// 检查宫位是否有指定六煞星
///
/// # 参数
/// - `palace`: 宫位数据
/// - `liu_sha_index`: 六煞星索引 (0=擎羊, 1=陀罗, 2=火星, 3=铃星, 4=地空, 5=地劫)
pub fn gong_has_liu_sha(palace: &Palace, liu_sha_index: usize) -> bool {
    liu_sha_index < 6 && palace.liu_sha[liu_sha_index]
}

/// 检查宫位是否有指定四化
///
/// # 参数
/// - `palace`: 宫位数据
/// - `si_hua`: 四化类型
pub fn gong_has_si_hua(palace: &Palace, si_hua: SiHua) -> bool {
    palace.si_hua.iter().any(|s| *s == Some(si_hua))
}

// ============================================================================
// 辅助函数 - 宫位关系计算
// ============================================================================

/// 获取三方四正宫位索引
///
/// 三方四正包括：本宫、对宫（冲）、左夹（顺数第5宫）、右夹（逆数第5宫）
///
/// # 参数
/// - `gong_index`: 本宫索引 (0-11)
///
/// # 返回
/// [本宫, 对宫, 三合左, 三合右] 的索引数组
pub fn get_san_fang_indices(gong_index: u8) -> [u8; 4] {
    let ben_gong = gong_index % 12;
    let dui_gong = (gong_index + 6) % 12;      // 对宫（冲位）
    let san_he_left = (gong_index + 4) % 12;   // 三合左（顺数第5宫）
    let san_he_right = (gong_index + 8) % 12;  // 三合右（逆数第5宫）

    [ben_gong, dui_gong, san_he_left, san_he_right]
}

/// 获取夹宫索引
///
/// # 参数
/// - `gong_index`: 本宫索引 (0-11)
///
/// # 返回
/// [前一宫, 后一宫] 的索引数组
pub fn get_jia_gong_indices(gong_index: u8) -> [u8; 2] {
    let prev = if gong_index == 0 { 11 } else { gong_index - 1 };
    let next = (gong_index + 1) % 12;
    [prev, next]
}

/// 检查三方四正是否有指定主星
///
/// # 参数
/// - `palaces`: 十二宫数据
/// - `gong_index`: 本宫索引
/// - `star`: 要检查的主星
pub fn san_fang_has_star(palaces: &[Palace; 12], gong_index: u8, star: ZhuXing) -> bool {
    let indices = get_san_fang_indices(gong_index);
    indices.iter().any(|&idx| gong_has_star(&palaces[idx as usize], star))
}

/// 检查三方四正是否有指定六吉星
pub fn san_fang_has_liu_ji(palaces: &[Palace; 12], gong_index: u8, liu_ji_index: usize) -> bool {
    let indices = get_san_fang_indices(gong_index);
    indices.iter().any(|&idx| gong_has_liu_ji(&palaces[idx as usize], liu_ji_index))
}

/// 检查三方四正是否有指定六煞星
pub fn san_fang_has_liu_sha(palaces: &[Palace; 12], gong_index: u8, liu_sha_index: usize) -> bool {
    let indices = get_san_fang_indices(gong_index);
    indices.iter().any(|&idx| gong_has_liu_sha(&palaces[idx as usize], liu_sha_index))
}

/// 计算格局强度
///
/// 根据星曜亮度计算格局强度
///
/// # 参数
/// - `palaces`: 十二宫数据
/// - `key_palace_indices`: 关键宫位索引
fn calculate_pattern_strength(palaces: &[Palace; 12], key_palace_indices: &[u8]) -> u8 {
    let mut total_brightness: u32 = 0;
    let mut count: u32 = 0;

    for &idx in key_palace_indices {
        let palace = &palaces[idx as usize];
        for i in 0..3 {
            if palace.zhu_xing[i].is_some() {
                total_brightness += palace.zhu_xing_brightness[i].weight() as u32;
                count += 1;
            }
        }
    }

    if count == 0 {
        50 // 默认中等强度
    } else {
        (total_brightness / count).min(100) as u8
    }
}

// ============================================================================
// 富贵格局检测（0-13）
// ============================================================================

/// 检查紫府同宫格
///
/// 条件：紫微、天府同坐命宫
pub fn check_zi_fu_tong_gong(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let ming_gong = &palaces[ming_gong_pos as usize];

    if gong_has_star(ming_gong, ZhuXing::ZiWei) && gong_has_star(ming_gong, ZhuXing::TianFu) {
        let strength = calculate_pattern_strength(palaces, &[ming_gong_pos]);
        Some(PatternInfo::new(PatternType::ZiFuTongGong, strength, [ming_gong_pos, 0, 0]))
    } else {
        None
    }
}

/// 检查紫府朝垣格
///
/// 条件：紫微、天府在三方四正会照命宫
pub fn check_zi_fu_chao_yuan(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let has_ziwei = san_fang_has_star(palaces, ming_gong_pos, ZhuXing::ZiWei);
    let has_tianfu = san_fang_has_star(palaces, ming_gong_pos, ZhuXing::TianFu);

    // 排除紫府同宫的情况（已由其他格局处理）
    let ming_gong = &palaces[ming_gong_pos as usize];
    let is_tong_gong = gong_has_star(ming_gong, ZhuXing::ZiWei) && gong_has_star(ming_gong, ZhuXing::TianFu);

    if has_ziwei && has_tianfu && !is_tong_gong {
        let indices = get_san_fang_indices(ming_gong_pos);
        let strength = calculate_pattern_strength(palaces, &indices);
        Some(PatternInfo::new(PatternType::ZiFuChaoYuan, strength, [ming_gong_pos, indices[1], indices[2]]))
    } else {
        None
    }
}

/// 检查天府朝垣格
///
/// 条件：天府守命，逢禄存或化禄
pub fn check_tian_fu_chao_yuan(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let ming_gong = &palaces[ming_gong_pos as usize];

    if gong_has_star(ming_gong, ZhuXing::TianFu) {
        let has_lu = ming_gong.lu_cun || gong_has_si_hua(ming_gong, SiHua::HuaLu);
        if has_lu {
            let strength = calculate_pattern_strength(palaces, &[ming_gong_pos]);
            return Some(PatternInfo::new(PatternType::TianFuChaoYuan, strength, [ming_gong_pos, 0, 0]));
        }
    }
    None
}

/// 检查君臣庆会格
///
/// 条件：紫微为君，天相、天府为臣，三方会合
pub fn check_jun_chen_qing_hui(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let has_ziwei = san_fang_has_star(palaces, ming_gong_pos, ZhuXing::ZiWei);
    let has_tianfu = san_fang_has_star(palaces, ming_gong_pos, ZhuXing::TianFu);
    let has_tianxiang = san_fang_has_star(palaces, ming_gong_pos, ZhuXing::TianXiang);

    if has_ziwei && has_tianfu && has_tianxiang {
        let indices = get_san_fang_indices(ming_gong_pos);
        let strength = calculate_pattern_strength(palaces, &indices);
        Some(PatternInfo::new(PatternType::JunChenQingHui, strength, [ming_gong_pos, indices[1], indices[2]]))
    } else {
        None
    }
}

/// 检查府相朝垣格
///
/// 条件：天府、天相在命宫或三方会照
pub fn check_fu_xiang_chao_yuan(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let has_tianfu = san_fang_has_star(palaces, ming_gong_pos, ZhuXing::TianFu);
    let has_tianxiang = san_fang_has_star(palaces, ming_gong_pos, ZhuXing::TianXiang);

    if has_tianfu && has_tianxiang {
        let indices = get_san_fang_indices(ming_gong_pos);
        let strength = calculate_pattern_strength(palaces, &indices);
        Some(PatternInfo::new(PatternType::FuXiangChaoYuan, strength, [ming_gong_pos, indices[1], indices[2]]))
    } else {
        None
    }
}

/// 检查机月同梁格
///
/// 条件：天机、太阴、天同、天梁四星会合于命宫三方
pub fn check_ji_yue_tong_liang(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let has_tianji = san_fang_has_star(palaces, ming_gong_pos, ZhuXing::TianJi);
    let has_taiyin = san_fang_has_star(palaces, ming_gong_pos, ZhuXing::TaiYin);
    let has_tiantong = san_fang_has_star(palaces, ming_gong_pos, ZhuXing::TianTong);
    let has_tianliang = san_fang_has_star(palaces, ming_gong_pos, ZhuXing::TianLiang);

    // 至少三颗星会合
    let count = [has_tianji, has_taiyin, has_tiantong, has_tianliang]
        .iter()
        .filter(|&&x| x)
        .count();

    if count >= 3 {
        let indices = get_san_fang_indices(ming_gong_pos);
        let strength = calculate_pattern_strength(palaces, &indices);
        Some(PatternInfo::new(PatternType::JiYueTongLiang, strength, [ming_gong_pos, indices[1], indices[2]]))
    } else {
        None
    }
}

/// 检查日月并明格
///
/// 条件：太阳、太阴在卯酉宫，且都在旺地
pub fn check_ri_yue_bing_ming(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    // 检查太阳和太阴是否在三方会照
    let has_taiyang = san_fang_has_star(palaces, ming_gong_pos, ZhuXing::TaiYang);
    let has_taiyin = san_fang_has_star(palaces, ming_gong_pos, ZhuXing::TaiYin);

    if has_taiyang && has_taiyin {
        // 检查是否在庙旺位置
        let mut taiyang_bright = false;
        let mut taiyin_bright = false;

        let indices = get_san_fang_indices(ming_gong_pos);
        for &idx in &indices {
            let palace = &palaces[idx as usize];
            for i in 0..3 {
                if palace.zhu_xing[i] == Some(ZhuXing::TaiYang) {
                    taiyang_bright = matches!(palace.zhu_xing_brightness[i], StarBrightness::Miao | StarBrightness::Wang);
                }
                if palace.zhu_xing[i] == Some(ZhuXing::TaiYin) {
                    taiyin_bright = matches!(palace.zhu_xing_brightness[i], StarBrightness::Miao | StarBrightness::Wang);
                }
            }
        }

        if taiyang_bright && taiyin_bright {
            let strength = calculate_pattern_strength(palaces, &indices);
            return Some(PatternInfo::new(PatternType::RiYueBingMing, strength, [ming_gong_pos, indices[1], indices[2]]));
        }
    }
    None
}

/// 检查日照雷门格
///
/// 条件：太阳在卯宫守命，且庙旺
pub fn check_ri_zhao_lei_men(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let ming_gong = &palaces[ming_gong_pos as usize];

    // 卯宫 = 地支索引 3
    if ming_gong.di_zhi == DiZhi::Mao && gong_has_star(ming_gong, ZhuXing::TaiYang) {
        for i in 0..3 {
            if ming_gong.zhu_xing[i] == Some(ZhuXing::TaiYang) {
                if matches!(ming_gong.zhu_xing_brightness[i], StarBrightness::Miao | StarBrightness::Wang) {
                    let strength = calculate_pattern_strength(palaces, &[ming_gong_pos]);
                    return Some(PatternInfo::new(PatternType::RiZhaoLeiMen, strength, [ming_gong_pos, 0, 0]));
                }
            }
        }
    }
    None
}

/// 检查月朗天门格
///
/// 条件：太阴在亥宫守命，且庙旺
pub fn check_yue_lang_tian_men(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let ming_gong = &palaces[ming_gong_pos as usize];

    // 亥宫 = 地支索引 11
    if ming_gong.di_zhi == DiZhi::Hai && gong_has_star(ming_gong, ZhuXing::TaiYin) {
        for i in 0..3 {
            if ming_gong.zhu_xing[i] == Some(ZhuXing::TaiYin) {
                if matches!(ming_gong.zhu_xing_brightness[i], StarBrightness::Miao | StarBrightness::Wang) {
                    let strength = calculate_pattern_strength(palaces, &[ming_gong_pos]);
                    return Some(PatternInfo::new(PatternType::YueLangTianMen, strength, [ming_gong_pos, 0, 0]));
                }
            }
        }
    }
    None
}

/// 检查明珠出海格
///
/// 条件：太阴在酉宫守命
pub fn check_ming_zhu_chu_hai(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let ming_gong = &palaces[ming_gong_pos as usize];

    // 酉宫 = 地支索引 9
    if ming_gong.di_zhi == DiZhi::You && gong_has_star(ming_gong, ZhuXing::TaiYin) {
        let strength = calculate_pattern_strength(palaces, &[ming_gong_pos]);
        Some(PatternInfo::new(PatternType::MingZhuChuHai, strength, [ming_gong_pos, 0, 0]))
    } else {
        None
    }
}

/// 检查阳梁昌禄格
///
/// 条件：太阳、天梁在三方，会文昌、禄存
pub fn check_yang_liang_chang_lu(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let has_taiyang = san_fang_has_star(palaces, ming_gong_pos, ZhuXing::TaiYang);
    let has_tianliang = san_fang_has_star(palaces, ming_gong_pos, ZhuXing::TianLiang);
    let has_wenchang = san_fang_has_liu_ji(palaces, ming_gong_pos, 0); // 文昌

    // 检查禄存
    let indices = get_san_fang_indices(ming_gong_pos);
    let has_lucun = indices.iter().any(|&idx| palaces[idx as usize].lu_cun);

    if has_taiyang && has_tianliang && has_wenchang && has_lucun {
        let strength = calculate_pattern_strength(palaces, &indices);
        Some(PatternInfo::new(PatternType::YangLiangChangLu, strength, [ming_gong_pos, indices[1], indices[2]]))
    } else {
        None
    }
}

/// 检查贪武同行格
///
/// 条件：贪狼、武曲同坐丑未宫
pub fn check_tan_wu_tong_xing(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let ming_gong = &palaces[ming_gong_pos as usize];

    // 丑宫=1, 未宫=7
    if matches!(ming_gong.di_zhi, DiZhi::Chou | DiZhi::Wei) {
        if gong_has_star(ming_gong, ZhuXing::TanLang) && gong_has_star(ming_gong, ZhuXing::WuQu) {
            let strength = calculate_pattern_strength(palaces, &[ming_gong_pos]);
            return Some(PatternInfo::new(PatternType::TanWuTongXing, strength, [ming_gong_pos, 0, 0]));
        }
    }
    None
}

/// 检查火贪格
///
/// 条件：火星、贪狼同宫于命宫
pub fn check_huo_tan_ge(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let ming_gong = &palaces[ming_gong_pos as usize];

    if gong_has_star(ming_gong, ZhuXing::TanLang) && gong_has_liu_sha(ming_gong, 2) {
        // 火星索引=2
        let strength = calculate_pattern_strength(palaces, &[ming_gong_pos]);
        Some(PatternInfo::new(PatternType::HuoTanGeJu, strength, [ming_gong_pos, 0, 0]))
    } else {
        None
    }
}

/// 检查铃贪格
///
/// 条件：铃星、贪狼同宫于命宫
pub fn check_ling_tan_ge(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let ming_gong = &palaces[ming_gong_pos as usize];

    if gong_has_star(ming_gong, ZhuXing::TanLang) && gong_has_liu_sha(ming_gong, 3) {
        // 铃星索引=3
        let strength = calculate_pattern_strength(palaces, &[ming_gong_pos]);
        Some(PatternInfo::new(PatternType::LingTanGeJu, strength, [ming_gong_pos, 0, 0]))
    } else {
        None
    }
}

// ============================================================================
// 权贵格局检测（14-21）
// ============================================================================

/// 检查三奇嘉会格
///
/// 条件：化禄、化权、化科三化在命宫三方会合
pub fn check_san_qi_jia_hui(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let indices = get_san_fang_indices(ming_gong_pos);

    let mut has_lu = false;
    let mut has_quan = false;
    let mut has_ke = false;

    for &idx in &indices {
        let palace = &palaces[idx as usize];
        if gong_has_si_hua(palace, SiHua::HuaLu) {
            has_lu = true;
        }
        if gong_has_si_hua(palace, SiHua::HuaQuan) {
            has_quan = true;
        }
        if gong_has_si_hua(palace, SiHua::HuaKe) {
            has_ke = true;
        }
    }

    if has_lu && has_quan && has_ke {
        let strength = calculate_pattern_strength(palaces, &indices);
        Some(PatternInfo::new(PatternType::SanQiJiaHui, strength, [ming_gong_pos, indices[1], indices[2]]))
    } else {
        None
    }
}

/// 检查双禄夹命格
///
/// 条件：禄存、化禄夹命宫
pub fn check_shuang_lu_jia_ming(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let [prev, next] = get_jia_gong_indices(ming_gong_pos);
    let prev_palace = &palaces[prev as usize];
    let next_palace = &palaces[next as usize];

    let prev_has_lu = prev_palace.lu_cun || gong_has_si_hua(prev_palace, SiHua::HuaLu);
    let next_has_lu = next_palace.lu_cun || gong_has_si_hua(next_palace, SiHua::HuaLu);

    if prev_has_lu && next_has_lu {
        let strength = calculate_pattern_strength(palaces, &[prev, next]);
        Some(PatternInfo::new(PatternType::ShuangLuJiaMing, strength, [ming_gong_pos, prev, next]))
    } else {
        None
    }
}

/// 检查双禄夹财格
///
/// 条件：禄存、化禄夹财帛宫
pub fn check_shuang_lu_jia_cai(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let cai_bo_pos = (ming_gong_pos + 8) % 12;
    let [prev, next] = get_jia_gong_indices(cai_bo_pos);
    let prev_palace = &palaces[prev as usize];
    let next_palace = &palaces[next as usize];

    let prev_has_lu = prev_palace.lu_cun || gong_has_si_hua(prev_palace, SiHua::HuaLu);
    let next_has_lu = next_palace.lu_cun || gong_has_si_hua(next_palace, SiHua::HuaLu);

    if prev_has_lu && next_has_lu {
        let strength = calculate_pattern_strength(palaces, &[prev, next]);
        Some(PatternInfo::new(PatternType::ShuangLuJiaCai, strength, [cai_bo_pos, prev, next]))
    } else {
        None
    }
}

/// 检查科权禄夹格
///
/// 条件：化科、化权、化禄夹宫
pub fn check_ke_quan_lu_jia(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let [prev, next] = get_jia_gong_indices(ming_gong_pos);

    let mut hua_count = 0;
    for &idx in &[prev, next] {
        let palace = &palaces[idx as usize];
        if gong_has_si_hua(palace, SiHua::HuaLu) ||
           gong_has_si_hua(palace, SiHua::HuaQuan) ||
           gong_has_si_hua(palace, SiHua::HuaKe) {
            hua_count += 1;
        }
    }

    if hua_count >= 2 {
        let strength = calculate_pattern_strength(palaces, &[prev, next]);
        Some(PatternInfo::new(PatternType::KeQuanLuJia, strength, [ming_gong_pos, prev, next]))
    } else {
        None
    }
}

/// 检查左右夹命格
///
/// 条件：左辅、右弼夹命宫
pub fn check_zuo_you_jia_ming(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let [prev, next] = get_jia_gong_indices(ming_gong_pos);

    // 左辅=2, 右弼=3
    let prev_has_zuo = gong_has_liu_ji(&palaces[prev as usize], 2);
    let prev_has_you = gong_has_liu_ji(&palaces[prev as usize], 3);
    let next_has_zuo = gong_has_liu_ji(&palaces[next as usize], 2);
    let next_has_you = gong_has_liu_ji(&palaces[next as usize], 3);

    if (prev_has_zuo && next_has_you) || (prev_has_you && next_has_zuo) {
        let strength = calculate_pattern_strength(palaces, &[prev, next]);
        Some(PatternInfo::new(PatternType::ZuoYouJiaMing, strength, [ming_gong_pos, prev, next]))
    } else {
        None
    }
}

/// 检查昌曲夹命格
///
/// 条件：文昌、文曲夹命宫
pub fn check_chang_qu_jia_ming(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let [prev, next] = get_jia_gong_indices(ming_gong_pos);

    // 文昌=0, 文曲=1
    let prev_has_chang = gong_has_liu_ji(&palaces[prev as usize], 0);
    let prev_has_qu = gong_has_liu_ji(&palaces[prev as usize], 1);
    let next_has_chang = gong_has_liu_ji(&palaces[next as usize], 0);
    let next_has_qu = gong_has_liu_ji(&palaces[next as usize], 1);

    if (prev_has_chang && next_has_qu) || (prev_has_qu && next_has_chang) {
        let strength = calculate_pattern_strength(palaces, &[prev, next]);
        Some(PatternInfo::new(PatternType::ChangQuJiaMing, strength, [ming_gong_pos, prev, next]))
    } else {
        None
    }
}

/// 检查魁钺夹命格
///
/// 条件：天魁、天钺夹命宫
pub fn check_kui_yue_jia_ming(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let [prev, next] = get_jia_gong_indices(ming_gong_pos);

    // 天魁=4, 天钺=5
    let prev_has_kui = gong_has_liu_ji(&palaces[prev as usize], 4);
    let prev_has_yue = gong_has_liu_ji(&palaces[prev as usize], 5);
    let next_has_kui = gong_has_liu_ji(&palaces[next as usize], 4);
    let next_has_yue = gong_has_liu_ji(&palaces[next as usize], 5);

    if (prev_has_kui && next_has_yue) || (prev_has_yue && next_has_kui) {
        let strength = calculate_pattern_strength(palaces, &[prev, next]);
        Some(PatternInfo::new(PatternType::KuiYueJiaMing, strength, [ming_gong_pos, prev, next]))
    } else {
        None
    }
}

/// 检查禄马交驰格
///
/// 条件：禄存、天马同宫或会照命宫
pub fn check_lu_ma_jiao_chi(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let indices = get_san_fang_indices(ming_gong_pos);

    let mut has_lucun = false;
    let mut has_tianma = false;
    let mut lucun_pos: u8 = 0;
    let mut tianma_pos: u8 = 0;

    for &idx in &indices {
        let palace = &palaces[idx as usize];
        if palace.lu_cun {
            has_lucun = true;
            lucun_pos = idx;
        }
        if palace.tian_ma {
            has_tianma = true;
            tianma_pos = idx;
        }
    }

    if has_lucun && has_tianma {
        let strength = calculate_pattern_strength(palaces, &[lucun_pos, tianma_pos]);
        Some(PatternInfo::new(PatternType::LuMaJiaoChiGeJu, strength, [ming_gong_pos, lucun_pos, tianma_pos]))
    } else {
        None
    }
}

// ============================================================================
// 凶格检测（22-31）
// ============================================================================

/// 检查铃昌陀武格
///
/// 条件：铃星、文昌、陀罗、武曲同宫
pub fn check_ling_chang_tuo_wu(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let ming_gong = &palaces[ming_gong_pos as usize];

    let has_wuqu = gong_has_star(ming_gong, ZhuXing::WuQu);
    let has_wenchang = gong_has_liu_ji(ming_gong, 0);  // 文昌
    let has_lingxing = gong_has_liu_sha(ming_gong, 3); // 铃星
    let has_tuoluo = gong_has_liu_sha(ming_gong, 1);   // 陀罗

    if has_wuqu && has_wenchang && has_lingxing && has_tuoluo {
        let strength = calculate_pattern_strength(palaces, &[ming_gong_pos]);
        Some(PatternInfo::new(PatternType::LingChangTuoWu, strength, [ming_gong_pos, 0, 0]))
    } else {
        None
    }
}

/// 检查巨机同宫格（凶）
///
/// 条件：巨门、天机在辰戌宫同宫
pub fn check_ju_ji_tong_gong(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let ming_gong = &palaces[ming_gong_pos as usize];

    // 辰宫=4, 戌宫=10
    if matches!(ming_gong.di_zhi, DiZhi::Chen | DiZhi::Xu) {
        if gong_has_star(ming_gong, ZhuXing::JuMen) && gong_has_star(ming_gong, ZhuXing::TianJi) {
            let strength = calculate_pattern_strength(palaces, &[ming_gong_pos]);
            return Some(PatternInfo::new(PatternType::JiJiTongGong, strength, [ming_gong_pos, 0, 0]));
        }
    }
    None
}

/// 检查巨日同宫格（凶）
///
/// 条件：巨门、太阳同宫（最忌）
pub fn check_ju_ri_tong_gong(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let ming_gong = &palaces[ming_gong_pos as usize];

    if gong_has_star(ming_gong, ZhuXing::JuMen) && gong_has_star(ming_gong, ZhuXing::TaiYang) {
        // 检查太阳是否落陷
        for i in 0..3 {
            if ming_gong.zhu_xing[i] == Some(ZhuXing::TaiYang) {
                if matches!(ming_gong.zhu_xing_brightness[i], StarBrightness::Xian | StarBrightness::BuDe) {
                    let strength = calculate_pattern_strength(palaces, &[ming_gong_pos]);
                    return Some(PatternInfo::new(PatternType::JuRiTongGong, strength, [ming_gong_pos, 0, 0]));
                }
            }
        }
    }
    None
}

/// 检查命无正曜格（空宫）
///
/// 条件：命宫无主星
pub fn check_ming_wu_zheng_yao(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let ming_gong = &palaces[ming_gong_pos as usize];

    let has_main_star = ming_gong.zhu_xing.iter().any(|s| s.is_some());

    if !has_main_star {
        Some(PatternInfo::new(PatternType::MingWuZhengYao, 50, [ming_gong_pos, 0, 0]))
    } else {
        None
    }
}

/// 检查马头带箭格
///
/// 条件：午宫擎羊守命
pub fn check_ma_tou_dai_jian(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let ming_gong = &palaces[ming_gong_pos as usize];

    // 午宫=6，擎羊索引=0
    if ming_gong.di_zhi == DiZhi::Wu && gong_has_liu_sha(ming_gong, 0) {
        let strength = calculate_pattern_strength(palaces, &[ming_gong_pos]);
        Some(PatternInfo::new(PatternType::MaTouDaiJian, strength, [ming_gong_pos, 0, 0]))
    } else {
        None
    }
}

/// 检查羊陀夹命格
///
/// 条件：擎羊、陀罗夹命宫
pub fn check_yang_tuo_jia_ming(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let [prev, next] = get_jia_gong_indices(ming_gong_pos);

    // 擎羊=0, 陀罗=1
    let prev_has_yang = gong_has_liu_sha(&palaces[prev as usize], 0);
    let prev_has_tuo = gong_has_liu_sha(&palaces[prev as usize], 1);
    let next_has_yang = gong_has_liu_sha(&palaces[next as usize], 0);
    let next_has_tuo = gong_has_liu_sha(&palaces[next as usize], 1);

    if (prev_has_yang && next_has_tuo) || (prev_has_tuo && next_has_yang) {
        let strength = calculate_pattern_strength(palaces, &[prev, next]);
        Some(PatternInfo::new(PatternType::YangTuoJiaMing, strength, [ming_gong_pos, prev, next]))
    } else {
        None
    }
}

/// 检查火铃夹命格
///
/// 条件：火星、铃星夹命宫
pub fn check_huo_ling_jia_ming(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let [prev, next] = get_jia_gong_indices(ming_gong_pos);

    // 火星=2, 铃星=3
    let prev_has_huo = gong_has_liu_sha(&palaces[prev as usize], 2);
    let prev_has_ling = gong_has_liu_sha(&palaces[prev as usize], 3);
    let next_has_huo = gong_has_liu_sha(&palaces[next as usize], 2);
    let next_has_ling = gong_has_liu_sha(&palaces[next as usize], 3);

    if (prev_has_huo && next_has_ling) || (prev_has_ling && next_has_huo) {
        let strength = calculate_pattern_strength(palaces, &[prev, next]);
        Some(PatternInfo::new(PatternType::HuoLingJiaMing, strength, [ming_gong_pos, prev, next]))
    } else {
        None
    }
}

/// 检查空劫夹命格
///
/// 条件：地空、地劫夹命宫
pub fn check_kong_jie_jia_ming(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let [prev, next] = get_jia_gong_indices(ming_gong_pos);

    // 地空=4, 地劫=5
    let prev_has_kong = gong_has_liu_sha(&palaces[prev as usize], 4);
    let prev_has_jie = gong_has_liu_sha(&palaces[prev as usize], 5);
    let next_has_kong = gong_has_liu_sha(&palaces[next as usize], 4);
    let next_has_jie = gong_has_liu_sha(&palaces[next as usize], 5);

    if (prev_has_kong && next_has_jie) || (prev_has_jie && next_has_kong) {
        let strength = calculate_pattern_strength(palaces, &[prev, next]);
        Some(PatternInfo::new(PatternType::KongJieJiaMing, strength, [ming_gong_pos, prev, next]))
    } else {
        None
    }
}

/// 检查羊陀夹忌格
///
/// 条件：擎羊、陀罗夹化忌
pub fn check_yang_tuo_jia_ji(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    // 检查命宫是否有化忌
    let ming_gong = &palaces[ming_gong_pos as usize];
    if !gong_has_si_hua(ming_gong, SiHua::HuaJi) {
        return None;
    }

    let [prev, next] = get_jia_gong_indices(ming_gong_pos);

    // 擎羊=0, 陀罗=1
    let prev_has_yang = gong_has_liu_sha(&palaces[prev as usize], 0);
    let prev_has_tuo = gong_has_liu_sha(&palaces[prev as usize], 1);
    let next_has_yang = gong_has_liu_sha(&palaces[next as usize], 0);
    let next_has_tuo = gong_has_liu_sha(&palaces[next as usize], 1);

    if (prev_has_yang && next_has_tuo) || (prev_has_tuo && next_has_yang) {
        let strength = calculate_pattern_strength(palaces, &[prev, next]);
        Some(PatternInfo::new(PatternType::YangTuoJiaJi, strength, [ming_gong_pos, prev, next]))
    } else {
        None
    }
}

/// 检查四煞冲命格
///
/// 条件：擎羊、陀罗、火星、铃星在三方四正冲命
pub fn check_si_sha_chong_ming(palaces: &[Palace; 12], ming_gong_pos: u8) -> Option<PatternInfo> {
    let indices = get_san_fang_indices(ming_gong_pos);

    let mut sha_count = 0;

    // 统计三方四正中的煞星数量
    for &idx in &indices {
        let palace = &palaces[idx as usize];
        // 擎羊=0, 陀罗=1, 火星=2, 铃星=3
        for i in 0..4 {
            if gong_has_liu_sha(palace, i) {
                sha_count += 1;
            }
        }
    }

    // 至少3颗煞星形成冲命格局
    if sha_count >= 3 {
        let strength = ((sha_count as u32) * 25).min(100) as u8;
        Some(PatternInfo::new(PatternType::SiShaChongMing, strength, [ming_gong_pos, indices[1], indices[2]]))
    } else {
        None
    }
}

// ============================================================================
// 汇总函数
// ============================================================================

/// 识别命盘中的所有格局
///
/// # 参数
/// - `palaces`: 十二宫数据
/// - `ming_gong_pos`: 命宫位置索引
///
/// # 返回
/// 识别到的格局列表（最多10个）
pub fn identify_all_patterns(
    palaces: &[Palace; 12],
    ming_gong_pos: u8,
) -> BoundedVec<PatternInfo, ConstU32<10>> {
    let mut patterns: BoundedVec<PatternInfo, ConstU32<10>> = BoundedVec::new();

    // 富贵格局（优先检测）
    if let Some(p) = check_zi_fu_tong_gong(palaces, ming_gong_pos) {
        let _ = patterns.try_push(p);
    }
    if let Some(p) = check_zi_fu_chao_yuan(palaces, ming_gong_pos) {
        let _ = patterns.try_push(p);
    }
    if let Some(p) = check_tian_fu_chao_yuan(palaces, ming_gong_pos) {
        let _ = patterns.try_push(p);
    }
    if let Some(p) = check_jun_chen_qing_hui(palaces, ming_gong_pos) {
        let _ = patterns.try_push(p);
    }
    if let Some(p) = check_fu_xiang_chao_yuan(palaces, ming_gong_pos) {
        let _ = patterns.try_push(p);
    }
    if let Some(p) = check_ji_yue_tong_liang(palaces, ming_gong_pos) {
        let _ = patterns.try_push(p);
    }
    if let Some(p) = check_ri_yue_bing_ming(palaces, ming_gong_pos) {
        let _ = patterns.try_push(p);
    }
    if let Some(p) = check_ri_zhao_lei_men(palaces, ming_gong_pos) {
        let _ = patterns.try_push(p);
    }
    if let Some(p) = check_yue_lang_tian_men(palaces, ming_gong_pos) {
        let _ = patterns.try_push(p);
    }
    if let Some(p) = check_ming_zhu_chu_hai(palaces, ming_gong_pos) {
        let _ = patterns.try_push(p);
    }

    // 如果格局数量未满，继续检测其他格局
    if patterns.len() < 10 {
        if let Some(p) = check_yang_liang_chang_lu(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_tan_wu_tong_xing(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_huo_tan_ge(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_ling_tan_ge(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }

    // 权贵格局
    if patterns.len() < 10 {
        if let Some(p) = check_san_qi_jia_hui(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_shuang_lu_jia_ming(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_shuang_lu_jia_cai(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_ke_quan_lu_jia(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_zuo_you_jia_ming(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_chang_qu_jia_ming(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_kui_yue_jia_ming(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_lu_ma_jiao_chi(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }

    // 凶格（后检测，但也重要）
    if patterns.len() < 10 {
        if let Some(p) = check_ling_chang_tuo_wu(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_ju_ji_tong_gong(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_ju_ri_tong_gong(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_ming_wu_zheng_yao(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_ma_tou_dai_jian(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_yang_tuo_jia_ming(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_huo_ling_jia_ming(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_kong_jie_jia_ming(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_yang_tuo_jia_ji(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }
    if patterns.len() < 10 {
        if let Some(p) = check_si_sha_chong_ming(palaces, ming_gong_pos) {
            let _ = patterns.try_push(p);
        }
    }

    patterns
}

/// 计算格局加成分
///
/// # 参数
/// - `patterns`: 格局列表
///
/// # 返回
/// 格局总加成分（-50 ~ +50）
pub fn calculate_pattern_bonus(patterns: &BoundedVec<PatternInfo, ConstU32<10>>) -> i32 {
    let total: i32 = patterns.iter().map(|p| p.score as i32).sum();
    total.clamp(-50, 50)
}

/// 统计吉格数量
pub fn count_auspicious_patterns(patterns: &BoundedVec<PatternInfo, ConstU32<10>>) -> usize {
    patterns.iter().filter(|p| p.is_auspicious).count()
}

/// 统计凶格数量
pub fn count_inauspicious_patterns(patterns: &BoundedVec<PatternInfo, ConstU32<10>>) -> usize {
    patterns.iter().filter(|p| !p.is_auspicious).count()
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
    fn test_gong_has_star() {
        let mut palaces = create_empty_palaces();
        palaces[0].zhu_xing = [Some(ZhuXing::ZiWei), Some(ZhuXing::TianFu), None];

        assert!(gong_has_star(&palaces[0], ZhuXing::ZiWei));
        assert!(gong_has_star(&palaces[0], ZhuXing::TianFu));
        assert!(!gong_has_star(&palaces[0], ZhuXing::TaiYang));
    }

    #[test]
    fn test_gong_has_all_stars() {
        let mut palaces = create_empty_palaces();
        palaces[0].zhu_xing = [Some(ZhuXing::ZiWei), Some(ZhuXing::TianFu), None];

        assert!(gong_has_all_stars(&palaces[0], &[ZhuXing::ZiWei, ZhuXing::TianFu]));
        assert!(!gong_has_all_stars(&palaces[0], &[ZhuXing::ZiWei, ZhuXing::TaiYang]));
    }

    #[test]
    fn test_get_san_fang_indices() {
        let indices = get_san_fang_indices(0);
        assert_eq!(indices, [0, 6, 4, 8]);

        let indices2 = get_san_fang_indices(3);
        assert_eq!(indices2, [3, 9, 7, 11]);
    }

    #[test]
    fn test_get_jia_gong_indices() {
        let indices = get_jia_gong_indices(0);
        assert_eq!(indices, [11, 1]);

        let indices2 = get_jia_gong_indices(6);
        assert_eq!(indices2, [5, 7]);
    }

    #[test]
    fn test_check_zi_fu_tong_gong() {
        let mut palaces = create_empty_palaces();
        palaces[0].zhu_xing = [Some(ZhuXing::ZiWei), Some(ZhuXing::TianFu), None];
        palaces[0].zhu_xing_brightness = [StarBrightness::Miao, StarBrightness::Wang, StarBrightness::Ping];

        let result = check_zi_fu_tong_gong(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::ZiFuTongGong);
        assert!(pattern.is_auspicious);
    }

    #[test]
    fn test_check_zi_fu_tong_gong_not_found() {
        let mut palaces = create_empty_palaces();
        palaces[0].zhu_xing = [Some(ZhuXing::ZiWei), None, None];

        let result = check_zi_fu_tong_gong(&palaces, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_check_san_qi_jia_hui() {
        let mut palaces = create_empty_palaces();
        // 在三方四正放置化禄、化权、化科
        palaces[0].si_hua = [Some(SiHua::HuaLu), None, None, None];
        palaces[6].si_hua = [Some(SiHua::HuaQuan), None, None, None];  // 对宫
        palaces[4].si_hua = [Some(SiHua::HuaKe), None, None, None];    // 三合

        let result = check_san_qi_jia_hui(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::SanQiJiaHui);
    }

    #[test]
    fn test_check_yang_tuo_jia_ming() {
        let mut palaces = create_empty_palaces();
        // 擎羊在前一宫，陀罗在后一宫
        palaces[11].liu_sha = [true, false, false, false, false, false]; // 擎羊
        palaces[1].liu_sha = [false, true, false, false, false, false];  // 陀罗

        let result = check_yang_tuo_jia_ming(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::YangTuoJiaMing);
        assert!(!pattern.is_auspicious);
    }

    #[test]
    fn test_check_ming_wu_zheng_yao() {
        let palaces = create_empty_palaces();
        // 命宫无主星

        let result = check_ming_wu_zheng_yao(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::MingWuZhengYao);
    }

    #[test]
    fn test_check_si_sha_chong_ming() {
        let mut palaces = create_empty_palaces();
        // 在三方四正放置多颗煞星
        palaces[0].liu_sha = [true, false, false, false, false, false]; // 擎羊
        palaces[6].liu_sha = [false, true, false, false, false, false]; // 陀罗
        palaces[4].liu_sha = [false, false, true, false, false, false]; // 火星

        let result = check_si_sha_chong_ming(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::SiShaChongMing);
    }

    #[test]
    fn test_identify_all_patterns() {
        let mut palaces = create_empty_palaces();
        palaces[0].zhu_xing = [Some(ZhuXing::ZiWei), Some(ZhuXing::TianFu), None];
        palaces[0].zhu_xing_brightness = [StarBrightness::Miao, StarBrightness::Wang, StarBrightness::Ping];

        let patterns = identify_all_patterns(&palaces, 0);
        assert!(!patterns.is_empty());

        // 应该识别到紫府同宫
        assert!(patterns.iter().any(|p| p.pattern_type == PatternType::ZiFuTongGong));
    }

    #[test]
    fn test_calculate_pattern_bonus() {
        let mut palaces = create_empty_palaces();
        palaces[0].zhu_xing = [Some(ZhuXing::ZiWei), Some(ZhuXing::TianFu), None];
        palaces[0].zhu_xing_brightness = [StarBrightness::Miao, StarBrightness::Wang, StarBrightness::Ping];

        let patterns = identify_all_patterns(&palaces, 0);
        let bonus = calculate_pattern_bonus(&patterns);

        // 吉格应该有正加成
        assert!(bonus > 0);
    }

    #[test]
    fn test_count_patterns() {
        let mut palaces = create_empty_palaces();
        palaces[0].zhu_xing = [Some(ZhuXing::ZiWei), Some(ZhuXing::TianFu), None];
        palaces[0].zhu_xing_brightness = [StarBrightness::Miao, StarBrightness::Wang, StarBrightness::Ping];

        let patterns = identify_all_patterns(&palaces, 0);
        let auspicious = count_auspicious_patterns(&patterns);
        let inauspicious = count_inauspicious_patterns(&patterns);

        assert!(auspicious >= 1);
        assert_eq!(auspicious + inauspicious, patterns.len());
    }

    #[test]
    fn test_check_zuo_you_jia_ming() {
        let mut palaces = create_empty_palaces();
        // 左辅在前一宫，右弼在后一宫
        palaces[11].liu_ji = [false, false, true, false, false, false]; // 左辅
        palaces[1].liu_ji = [false, false, false, true, false, false];  // 右弼

        let result = check_zuo_you_jia_ming(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::ZuoYouJiaMing);
        assert!(pattern.is_auspicious);
    }

    #[test]
    fn test_check_lu_ma_jiao_chi() {
        let mut palaces = create_empty_palaces();
        palaces[0].lu_cun = true;
        palaces[4].tian_ma = true;  // 三合位置

        let result = check_lu_ma_jiao_chi(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::LuMaJiaoChiGeJu);
    }

    // ========================================================================
    // 补充测试：更多格局检测
    // ========================================================================

    #[test]
    fn test_check_zi_fu_chao_yuan() {
        let mut palaces = create_empty_palaces();
        // 紫微在对宫
        palaces[6].zhu_xing = [Some(ZhuXing::ZiWei), None, None];
        // 天府在三合
        palaces[4].zhu_xing = [Some(ZhuXing::TianFu), None, None];
        palaces[0].zhu_xing_brightness = [StarBrightness::Miao, StarBrightness::Ping, StarBrightness::Ping];

        let result = check_zi_fu_chao_yuan(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::ZiFuChaoYuan);
    }

    #[test]
    fn test_check_ri_yue_bing_ming() {
        let mut palaces = create_empty_palaces();
        // 太阳在三合位旺位
        palaces[4].zhu_xing = [Some(ZhuXing::TaiYang), None, None];
        palaces[4].zhu_xing_brightness = [StarBrightness::Miao, StarBrightness::Ping, StarBrightness::Ping];
        // 太阴在三合位旺位
        palaces[8].zhu_xing = [Some(ZhuXing::TaiYin), None, None];
        palaces[8].zhu_xing_brightness = [StarBrightness::Wang, StarBrightness::Ping, StarBrightness::Ping];

        let result = check_ri_yue_bing_ming(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::RiYueBingMing);
    }

    #[test]
    fn test_check_ri_zhao_lei_men() {
        let mut palaces = create_empty_palaces();
        // 命宫在卯宫(3)
        palaces[3].gong_wei = GongWei::MingGong;
        palaces[3].di_zhi = DiZhi::Mao;
        palaces[3].zhu_xing = [Some(ZhuXing::TaiYang), None, None];
        palaces[3].zhu_xing_brightness = [StarBrightness::Miao, StarBrightness::Ping, StarBrightness::Ping];

        let result = check_ri_zhao_lei_men(&palaces, 3);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::RiZhaoLeiMen);
    }

    #[test]
    fn test_check_yue_lang_tian_men() {
        let mut palaces = create_empty_palaces();
        // 命宫在亥宫(11)
        palaces[11].gong_wei = GongWei::MingGong;
        palaces[11].di_zhi = DiZhi::Hai;
        palaces[11].zhu_xing = [Some(ZhuXing::TaiYin), None, None];
        palaces[11].zhu_xing_brightness = [StarBrightness::Miao, StarBrightness::Ping, StarBrightness::Ping];

        let result = check_yue_lang_tian_men(&palaces, 11);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::YueLangTianMen);
    }

    #[test]
    fn test_check_huo_ling_jia_ming() {
        let mut palaces = create_empty_palaces();
        // 火星在前一宫，铃星在后一宫
        palaces[11].liu_sha = [false, false, true, false, false, false]; // 火星
        palaces[1].liu_sha = [false, false, false, true, false, false];  // 铃星

        let result = check_huo_ling_jia_ming(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::HuoLingJiaMing);
        assert!(!pattern.is_auspicious);
    }

    #[test]
    fn test_check_kong_jie_jia_ming() {
        let mut palaces = create_empty_palaces();
        // 地空在前一宫，地劫在后一宫
        palaces[11].liu_sha = [false, false, false, false, true, false]; // 地空
        palaces[1].liu_sha = [false, false, false, false, false, true];  // 地劫

        let result = check_kong_jie_jia_ming(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::KongJieJiaMing);
        assert!(!pattern.is_auspicious);
    }

    #[test]
    fn test_check_yang_tuo_jia_ji() {
        let mut palaces = create_empty_palaces();
        // 命宫有化忌
        palaces[0].si_hua = [Some(SiHua::HuaJi), None, None, None];
        // 擎羊在前一宫，陀罗在后一宫
        palaces[11].liu_sha = [true, false, false, false, false, false]; // 擎羊
        palaces[1].liu_sha = [false, true, false, false, false, false];  // 陀罗

        let result = check_yang_tuo_jia_ji(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::YangTuoJiaJi);
        assert!(!pattern.is_auspicious);
    }

    #[test]
    fn test_check_yang_tuo_jia_ji_no_ji() {
        let mut palaces = create_empty_palaces();
        // 命宫无化忌
        // 擎羊在前一宫，陀罗在后一宫
        palaces[11].liu_sha = [true, false, false, false, false, false];
        palaces[1].liu_sha = [false, true, false, false, false, false];

        let result = check_yang_tuo_jia_ji(&palaces, 0);
        assert!(result.is_none()); // 没有化忌，格局不成立
    }

    #[test]
    fn test_check_chang_qu_jia_ming() {
        let mut palaces = create_empty_palaces();
        // 文昌在前一宫，文曲在后一宫
        palaces[11].liu_ji = [true, false, false, false, false, false]; // 文昌
        palaces[1].liu_ji = [false, true, false, false, false, false];  // 文曲

        let result = check_chang_qu_jia_ming(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::ChangQuJiaMing);
        assert!(pattern.is_auspicious);
    }

    #[test]
    fn test_check_kui_yue_jia_ming() {
        let mut palaces = create_empty_palaces();
        // 天魁在前一宫，天钺在后一宫
        palaces[11].liu_ji = [false, false, false, false, true, false]; // 天魁
        palaces[1].liu_ji = [false, false, false, false, false, true];  // 天钺

        let result = check_kui_yue_jia_ming(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::KuiYueJiaMing);
        assert!(pattern.is_auspicious);
    }

    #[test]
    fn test_check_shuang_lu_jia_ming() {
        let mut palaces = create_empty_palaces();
        // 禄存在前一宫
        palaces[11].lu_cun = true;
        // 化禄在后一宫
        palaces[1].si_hua = [Some(SiHua::HuaLu), None, None, None];

        let result = check_shuang_lu_jia_ming(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::ShuangLuJiaMing);
        assert!(pattern.is_auspicious);
    }

    #[test]
    fn test_check_ma_tou_dai_jian() {
        let mut palaces = create_empty_palaces();
        // 命宫在午宫(6)，有擎羊
        palaces[6].di_zhi = DiZhi::Wu;
        palaces[6].liu_sha = [true, false, false, false, false, false]; // 擎羊

        let result = check_ma_tou_dai_jian(&palaces, 6);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::MaTouDaiJian);
        assert!(!pattern.is_auspicious);
    }

    #[test]
    fn test_check_ma_tou_dai_jian_wrong_palace() {
        let mut palaces = create_empty_palaces();
        // 命宫不在午宫，有擎羊
        palaces[0].di_zhi = DiZhi::Zi;
        palaces[0].liu_sha = [true, false, false, false, false, false];

        let result = check_ma_tou_dai_jian(&palaces, 0);
        assert!(result.is_none()); // 不在午宫，格局不成立
    }

    #[test]
    fn test_check_huo_tan_ge() {
        let mut palaces = create_empty_palaces();
        // 命宫有贪狼和火星
        palaces[0].zhu_xing = [Some(ZhuXing::TanLang), None, None];
        palaces[0].liu_sha = [false, false, true, false, false, false]; // 火星

        let result = check_huo_tan_ge(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::HuoTanGeJu);
        assert!(pattern.is_auspicious);
    }

    #[test]
    fn test_check_ling_tan_ge() {
        let mut palaces = create_empty_palaces();
        // 命宫有贪狼和铃星
        palaces[0].zhu_xing = [Some(ZhuXing::TanLang), None, None];
        palaces[0].liu_sha = [false, false, false, true, false, false]; // 铃星

        let result = check_ling_tan_ge(&palaces, 0);
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::LingTanGeJu);
        assert!(pattern.is_auspicious);
    }

    #[test]
    fn test_calculate_pattern_strength() {
        let mut palaces = create_empty_palaces();
        palaces[0].zhu_xing = [Some(ZhuXing::ZiWei), None, None];
        palaces[0].zhu_xing_brightness = [StarBrightness::Miao, StarBrightness::Ping, StarBrightness::Ping];

        let strength = calculate_pattern_strength(&palaces, &[0]);
        assert!(strength > 0);
        assert!(strength <= 100);
    }

    #[test]
    fn test_gong_has_liu_ji() {
        let mut palaces = create_empty_palaces();
        palaces[0].liu_ji = [true, true, false, false, false, false];

        assert!(gong_has_liu_ji(&palaces[0], 0)); // 文昌
        assert!(gong_has_liu_ji(&palaces[0], 1)); // 文曲
        assert!(!gong_has_liu_ji(&palaces[0], 2)); // 左辅
    }

    #[test]
    fn test_gong_has_liu_sha() {
        let mut palaces = create_empty_palaces();
        palaces[0].liu_sha = [true, false, true, false, false, false];

        assert!(gong_has_liu_sha(&palaces[0], 0)); // 擎羊
        assert!(!gong_has_liu_sha(&palaces[0], 1)); // 陀罗
        assert!(gong_has_liu_sha(&palaces[0], 2)); // 火星
    }

    #[test]
    fn test_gong_has_si_hua() {
        let mut palaces = create_empty_palaces();
        palaces[0].si_hua = [Some(SiHua::HuaLu), Some(SiHua::HuaJi), None, None];

        assert!(gong_has_si_hua(&palaces[0], SiHua::HuaLu));
        assert!(gong_has_si_hua(&palaces[0], SiHua::HuaJi));
        assert!(!gong_has_si_hua(&palaces[0], SiHua::HuaQuan));
        assert!(!gong_has_si_hua(&palaces[0], SiHua::HuaKe));
    }

    #[test]
    fn test_identify_patterns_with_multiple() {
        let mut palaces = create_empty_palaces();
        // 紫府同宫
        palaces[0].zhu_xing = [Some(ZhuXing::ZiWei), Some(ZhuXing::TianFu), None];
        palaces[0].zhu_xing_brightness = [StarBrightness::Miao, StarBrightness::Wang, StarBrightness::Ping];
        // 左辅右弼夹命
        palaces[11].liu_ji = [false, false, true, false, false, false]; // 左辅
        palaces[1].liu_ji = [false, false, false, true, false, false];  // 右弼

        let patterns = identify_all_patterns(&palaces, 0);
        assert!(patterns.len() >= 2);

        // 应该同时有紫府同宫和左右夹命
        assert!(patterns.iter().any(|p| p.pattern_type == PatternType::ZiFuTongGong));
        assert!(patterns.iter().any(|p| p.pattern_type == PatternType::ZuoYouJiaMing));
    }

    #[test]
    fn test_pattern_info_new() {
        let info = PatternInfo::new(PatternType::ZiFuTongGong, 80, [0, 6, 4]);
        assert_eq!(info.pattern_type, PatternType::ZiFuTongGong);
        assert_eq!(info.strength, 80);
        assert!(info.is_auspicious);
        // score = base_score(50) * strength(80) / 100 = 40
        assert_eq!(info.score, 40);
        assert_eq!(info.key_palaces, [0, 6, 4]);
    }

    #[test]
    fn test_pattern_bonus_clamp() {
        // 测试加成分的钳制逻辑
        let mut patterns: BoundedVec<PatternInfo, ConstU32<10>> = BoundedVec::new();
        // 添加多个高分吉格
        for _ in 0..5 {
            let _ = patterns.try_push(PatternInfo::new(PatternType::ZiFuTongGong, 100, [0, 0, 0]));
        }

        let bonus = calculate_pattern_bonus(&patterns);
        assert_eq!(bonus, 50); // 应该钳制在50
    }
}
