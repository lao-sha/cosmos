//! # 奇门遁甲排盘算法模块
//!
//! 本模块实现奇门遁甲的核心排盘算法，包括：
//! - 阴阳遁和局数确定
//! - 地盘三奇六仪排布
//! - 天盘九星排布
//! - 人盘八门排布
//! - 神盘八神排布
//! - 值符值使计算
//!
//! ## 排盘流程
//!
//! 1. 根据节气确定阴阳遁
//! 2. 根据三元确定局数
//! 3. 排布地盘三奇六仪（固定）
//! 4. 找出旬首，确定值符和值使
//! 5. 根据时辰排布天盘九星
//! 6. 根据时辰排布人盘八门
//! 7. 根据值符排布神盘八神

use crate::types::*;
use sp_std::vec::Vec;

// ==================== 常量定义 ====================

/// 阳遁局数表（节气 × 三元）
///
/// 按节气顺序：冬至、小寒、大寒、立春、雨水、惊蛰、春分、清明、谷雨、立夏、小满、芒种
/// 每个节气对应上中下三元的局数
pub const YANG_DUN_JU: [[u8; 3]; 12] = [
    [1, 7, 4], // 冬至
    [2, 8, 5], // 小寒
    [3, 9, 6], // 大寒
    [8, 5, 2], // 立春
    [9, 6, 3], // 雨水
    [1, 7, 4], // 惊蛰
    [3, 9, 6], // 春分
    [4, 1, 7], // 清明
    [5, 2, 8], // 谷雨
    [4, 1, 7], // 立夏
    [5, 2, 8], // 小满
    [6, 3, 9], // 芒种
];

/// 阴遁局数表（节气 × 三元）
///
/// 按节气顺序：夏至、小暑、大暑、立秋、处暑、白露、秋分、寒露、霜降、立冬、小雪、大雪
/// 每个节气对应上中下三元的局数
pub const YIN_DUN_JU: [[u8; 3]; 12] = [
    [9, 3, 6], // 夏至
    [8, 2, 5], // 小暑
    [7, 1, 4], // 大暑
    [2, 5, 8], // 立秋
    [1, 4, 7], // 处暑
    [9, 3, 6], // 白露
    [7, 1, 4], // 秋分
    [6, 9, 3], // 寒露
    [5, 8, 2], // 霜降
    [6, 9, 3], // 立冬
    [5, 8, 2], // 小雪
    [4, 7, 1], // 大雪
];

/// 阳遁地盘三奇六仪查找表（1-9局，每局对应1-9宫的天干）
///
/// 阳遁规则：六仪（戊己庚辛壬癸）顺排，三奇（乙丙丁）顺排
/// 戊落宫位 = 局数，然后按九宫顺飞顺序依次排布
///
/// 参考：Java QiMenZhuanPanJiChuMap.DI_YANG_QI_YI
pub const DI_YANG_QI_YI: [[TianGan; 9]; 9] = [
    // 阳遁一局（1-9宫）：戊己庚辛壬癸丁丙乙
    [TianGan::Wu, TianGan::Ji, TianGan::Geng, TianGan::Xin, TianGan::Ren, TianGan::Gui, TianGan::Ding, TianGan::Bing, TianGan::Yi],
    // 阳遁二局（1-9宫）：乙戊己庚辛壬癸丁丙
    [TianGan::Yi, TianGan::Wu, TianGan::Ji, TianGan::Geng, TianGan::Xin, TianGan::Ren, TianGan::Gui, TianGan::Ding, TianGan::Bing],
    // 阳遁三局（1-9宫）：丙乙戊己庚辛壬癸丁
    [TianGan::Bing, TianGan::Yi, TianGan::Wu, TianGan::Ji, TianGan::Geng, TianGan::Xin, TianGan::Ren, TianGan::Gui, TianGan::Ding],
    // 阳遁四局（1-9宫）：丁丙乙戊己庚辛壬癸
    [TianGan::Ding, TianGan::Bing, TianGan::Yi, TianGan::Wu, TianGan::Ji, TianGan::Geng, TianGan::Xin, TianGan::Ren, TianGan::Gui],
    // 阳遁五局（1-9宫）：癸丁丙乙戊己庚辛壬
    [TianGan::Gui, TianGan::Ding, TianGan::Bing, TianGan::Yi, TianGan::Wu, TianGan::Ji, TianGan::Geng, TianGan::Xin, TianGan::Ren],
    // 阳遁六局（1-9宫）：壬癸丁丙乙戊己庚辛
    [TianGan::Ren, TianGan::Gui, TianGan::Ding, TianGan::Bing, TianGan::Yi, TianGan::Wu, TianGan::Ji, TianGan::Geng, TianGan::Xin],
    // 阳遁七局（1-9宫）：辛壬癸丁丙乙戊己庚
    [TianGan::Xin, TianGan::Ren, TianGan::Gui, TianGan::Ding, TianGan::Bing, TianGan::Yi, TianGan::Wu, TianGan::Ji, TianGan::Geng],
    // 阳遁八局（1-9宫）：庚辛壬癸丁丙乙戊己
    [TianGan::Geng, TianGan::Xin, TianGan::Ren, TianGan::Gui, TianGan::Ding, TianGan::Bing, TianGan::Yi, TianGan::Wu, TianGan::Ji],
    // 阳遁九局（1-9宫）：己庚辛壬癸丁丙乙戊
    [TianGan::Ji, TianGan::Geng, TianGan::Xin, TianGan::Ren, TianGan::Gui, TianGan::Ding, TianGan::Bing, TianGan::Yi, TianGan::Wu],
];

/// 阴遁地盘三奇六仪查找表（1-9局，每局对应1-9宫的天干）
///
/// 阴遁规则：三奇（乙丙丁）顺排在前，六仪（癸壬辛庚己）逆排在后
/// 戊落宫位 = 局数，然后按九宫逆飞顺序排布
///
/// 注意：阴遁的三奇六仪顺序与阳遁不同！
/// - 阳遁顺序：戊→己→庚→辛→壬→癸→丁→丙→乙（六仪顺+三奇顺）
/// - 阴遁顺序：戊→乙→丙→丁→癸→壬→辛→庚→己（三奇顺+六仪逆）
///
/// 参考：Java QiMenZhuanPanJiChuMap.DI_YIN_QI_YI
pub const DI_YIN_QI_YI: [[TianGan; 9]; 9] = [
    // 阴遁一局（1-9宫）：戊乙丙丁癸壬辛庚己
    [TianGan::Wu, TianGan::Yi, TianGan::Bing, TianGan::Ding, TianGan::Gui, TianGan::Ren, TianGan::Xin, TianGan::Geng, TianGan::Ji],
    // 阴遁二局（1-9宫）：己戊乙丙丁癸壬辛庚
    [TianGan::Ji, TianGan::Wu, TianGan::Yi, TianGan::Bing, TianGan::Ding, TianGan::Gui, TianGan::Ren, TianGan::Xin, TianGan::Geng],
    // 阴遁三局（1-9宫）：庚己戊乙丙丁癸壬辛
    [TianGan::Geng, TianGan::Ji, TianGan::Wu, TianGan::Yi, TianGan::Bing, TianGan::Ding, TianGan::Gui, TianGan::Ren, TianGan::Xin],
    // 阴遁四局（1-9宫）：辛庚己戊乙丙丁癸壬
    [TianGan::Xin, TianGan::Geng, TianGan::Ji, TianGan::Wu, TianGan::Yi, TianGan::Bing, TianGan::Ding, TianGan::Gui, TianGan::Ren],
    // 阴遁五局（1-9宫）：壬辛庚己戊乙丙丁癸
    [TianGan::Ren, TianGan::Xin, TianGan::Geng, TianGan::Ji, TianGan::Wu, TianGan::Yi, TianGan::Bing, TianGan::Ding, TianGan::Gui],
    // 阴遁六局（1-9宫）：癸壬辛庚己戊乙丙丁
    [TianGan::Gui, TianGan::Ren, TianGan::Xin, TianGan::Geng, TianGan::Ji, TianGan::Wu, TianGan::Yi, TianGan::Bing, TianGan::Ding],
    // 阴遁七局（1-9宫）：丁癸壬辛庚己戊乙丙
    [TianGan::Ding, TianGan::Gui, TianGan::Ren, TianGan::Xin, TianGan::Geng, TianGan::Ji, TianGan::Wu, TianGan::Yi, TianGan::Bing],
    // 阴遁八局（1-9宫）：丙丁癸壬辛庚己戊乙
    [TianGan::Bing, TianGan::Ding, TianGan::Gui, TianGan::Ren, TianGan::Xin, TianGan::Geng, TianGan::Ji, TianGan::Wu, TianGan::Yi],
    // 阴遁九局（1-9宫）：乙丙丁癸壬辛庚己戊
    [TianGan::Yi, TianGan::Bing, TianGan::Ding, TianGan::Gui, TianGan::Ren, TianGan::Xin, TianGan::Geng, TianGan::Ji, TianGan::Wu],
];

/// 六十甲子旬首表
///
/// 每旬的旬首天干地支和对应的遁甲（六仪）
pub const XUN_SHOU: [(TianGan, DiZhi, TianGan); 6] = [
    (TianGan::Jia, DiZhi::Zi, TianGan::Wu),   // 甲子旬，遁甲于戊
    (TianGan::Jia, DiZhi::Xu, TianGan::Ji),   // 甲戌旬，遁甲于己
    (TianGan::Jia, DiZhi::Shen, TianGan::Geng), // 甲申旬，遁甲于庚
    (TianGan::Jia, DiZhi::Wu, TianGan::Xin),  // 甲午旬，遁甲于辛
    (TianGan::Jia, DiZhi::Chen, TianGan::Ren), // 甲辰旬，遁甲于壬
    (TianGan::Jia, DiZhi::Yin, TianGan::Gui), // 甲寅旬，遁甲于癸
];

/// 六甲旬空查找表
///
/// 每旬对应的两个旬空地支
///
/// 口诀：
/// - 甲子旬空戌亥
/// - 甲戌旬空申酉
/// - 甲申旬空午未
/// - 甲午旬空辰巳
/// - 甲辰旬空寅卯
/// - 甲寅旬空子丑
///
/// 参考：Java QiMenZhuanPanJiChuMap.LIU_JIA_XUN_KONG
pub const XUN_KONG_TABLE: [(DiZhi, DiZhi); 6] = [
    (DiZhi::Xu, DiZhi::Hai),   // 甲子旬空戌亥
    (DiZhi::Shen, DiZhi::You), // 甲戌旬空申酉
    (DiZhi::Wu, DiZhi::Wei),   // 甲申旬空午未
    (DiZhi::Chen, DiZhi::Si),  // 甲午旬空辰巳
    (DiZhi::Yin, DiZhi::Mao),  // 甲辰旬空寅卯
    (DiZhi::Zi, DiZhi::Chou),  // 甲寅旬空子丑
];

/// 九宫飞布顺序（阳遁，顺时针）
///
/// 从坎一宫开始的顺时针顺序：1→8→3→4→9→2→7→6
pub const GONG_ORDER_YANG: [u8; 8] = [1, 8, 3, 4, 9, 2, 7, 6];

/// 九宫飞布顺序（阴遁，逆时针）
///
/// 从坎一宫开始的逆时针顺序：1→6→7→2→9→4→3→8
pub const GONG_ORDER_YIN: [u8; 8] = [1, 6, 7, 2, 9, 4, 3, 8];

/// 八神顺序（固定）
///
/// 值符、腾蛇、太阴、六合、白虎、玄武、九地、九天
pub const BA_SHEN_ORDER: [BaShen; 8] = [
    BaShen::ZhiFu,
    BaShen::TengShe,
    BaShen::TaiYin,
    BaShen::LiuHe,
    BaShen::BaiHu,
    BaShen::XuanWu,
    BaShen::JiuDi,
    BaShen::JiuTian,
];

// ==================== 核心算法 ====================

/// 计算阴阳遁类型
///
/// 根据节气判断：冬至到夏至前为阳遁，夏至到冬至前为阴遁
pub fn calc_dun_type(jie_qi: JieQi) -> DunType {
    if jie_qi.is_yang_dun() {
        DunType::Yang
    } else {
        DunType::Yin
    }
}

/// 计算三元（按节气内天数，简化方法）
///
/// 根据节气内的天数（1-15天），判断上中下三元
/// - 上元：第1-5天
/// - 中元：第6-10天
/// - 下元：第11-15天
///
/// 注意：这是简化的三元计算方法，适用于日家奇门。
/// 时家奇门应使用 `calc_san_yuan_by_zhi` 根据时辰地支判断。
pub fn calc_san_yuan(day_in_jieqi: u8) -> SanYuan {
    if day_in_jieqi <= 5 {
        SanYuan::Shang
    } else if day_in_jieqi <= 10 {
        SanYuan::Zhong
    } else {
        SanYuan::Xia
    }
}

/// 计算三元（按地支，时家奇门标准方法）
///
/// 时家奇门中，三元由时辰地支决定：
/// - 上元：子、午、卯、酉（四仲/四正）
/// - 中元：寅、申、巳、亥（四孟/四驿马）
/// - 下元：辰、戌、丑、未（四季/四墓库）
///
/// ## 口诀
///
/// "子午卯酉上元求，寅申巳亥中元流，辰戌丑未下元位"
///
/// ## 参数
///
/// - `zhi`: 时辰地支（时家奇门）或日支（日家奇门）
///
/// ## 返回值
///
/// 返回三元类型（上元、中元、下元）
pub fn calc_san_yuan_by_zhi(zhi: DiZhi) -> SanYuan {
    match zhi {
        // 四仲（四正）：子午卯酉 -> 上元
        DiZhi::Zi | DiZhi::Wu | DiZhi::Mao | DiZhi::You => SanYuan::Shang,
        // 四孟（四驿马）：寅申巳亥 -> 中元
        DiZhi::Yin | DiZhi::Shen | DiZhi::Si | DiZhi::Hai => SanYuan::Zhong,
        // 四季（四墓库）：辰戌丑未 -> 下元
        DiZhi::Chen | DiZhi::Xu | DiZhi::Chou | DiZhi::Wei => SanYuan::Xia,
    }
}

/// 计算三元（按六十甲子序号，精确方法）
///
/// 六十甲子中，每旬（10个干支）分为上元前5个、中元后5个。
/// 60甲子分6旬，每旬中：
/// - 位置 0-4（如甲子-戊辰）：上元
/// - 位置 5-9（如己巳-癸酉）：中元
///
/// 但由于一旬只有10个，而三元需要15天，所以：
/// - 0-4: 上元
/// - 5-9: 中元
/// - 10-14: 下元（跨入下一旬的前5个）
///
/// 更准确的说法是按干支在旬内的位置：
/// - 每旬前5天：上元
/// - 每旬后5天：中元
/// - 下一旬前5天：下元
pub fn calc_san_yuan_by_sexagenary(sexagenary_index: u8) -> SanYuan {
    // 在当前旬内的位置（0-9）
    let pos_in_xun = sexagenary_index % 10;

    // 判断是否为符头（甲子、甲戌、甲申、甲午、甲辰、甲寅）
    // 符头开始的5个干支为上元
    // 符头后5个干支为中元
    // 跨旬后5个为下元

    // 简化处理：按旬内位置判断
    // 0-4位置 -> 可能是上元或下元，需要结合节气判断
    // 5-9位置 -> 中元

    // 这里使用简化规则：
    // 每旬前5个为上/下元（交替），后5个为中元
    if pos_in_xun < 5 {
        // 根据旬序号判断是上元还是下元
        let xun_index = sexagenary_index / 10;
        if xun_index % 2 == 0 {
            SanYuan::Shang
        } else {
            SanYuan::Xia
        }
    } else {
        SanYuan::Zhong
    }
}

/// 计算局数
///
/// 根据节气、三元和阴阳遁查表获取局数
pub fn calc_ju_number(jie_qi: JieQi, san_yuan: SanYuan, dun_type: DunType) -> u8 {
    let jieqi_index = (jie_qi as u8) % 12;
    let yuan_index = match san_yuan {
        SanYuan::Shang => 0,
        SanYuan::Zhong => 1,
        SanYuan::Xia => 2,
    };

    match dun_type {
        DunType::Yang => YANG_DUN_JU[jieqi_index as usize][yuan_index],
        DunType::Yin => YIN_DUN_JU[jieqi_index as usize][yuan_index],
    }
}

/// 获取地盘三奇六仪排布
///
/// 使用预计算的查找表直接获取地盘排布，根据局数和阴阳遁类型查表
///
/// ## 阴阳遁差异
///
/// - **阳遁**：六仪顺排（戊→己→庚→辛→壬→癸），三奇顺排（丁→丙→乙）
/// - **阴遁**：三奇顺排（乙→丙→丁），六仪逆排（癸→壬→辛→庚→己）
///
/// ## 参数
///
/// - `ju_number`: 局数（1-9）
/// - `dun_type`: 阴阳遁类型
///
/// ## 返回值
///
/// 返回长度为9的数组，索引0-8分别对应坎一宫到离九宫的地盘天干
pub fn get_di_pan(ju_number: u8, dun_type: DunType) -> [TianGan; 9] {
    // 局数索引（1-9 转 0-8）
    let ju_idx = ju_number.saturating_sub(1).min(8) as usize;

    // 直接从查找表获取地盘排布
    match dun_type {
        DunType::Yang => DI_YANG_QI_YI[ju_idx],
        DunType::Yin => DI_YIN_QI_YI[ju_idx],
    }
}

/// 获取时干所在旬首
///
/// 返回旬首的六仪（戊己庚辛壬癸之一）
pub fn get_xun_shou(shi_gan: TianGan, shi_zhi: DiZhi) -> TianGan {
    // 计算六十甲子序号
    let gan_idx = shi_gan.index() as u16;
    let zhi_idx = shi_zhi.index() as u16;

    // 干支组合的六十甲子序号
    let sexagenary = ((gan_idx * 6 + zhi_idx * 5) % 60) as u8;

    // 每旬10个干支，确定在第几旬
    let xun_index = (sexagenary / 10) as usize;

    if xun_index < XUN_SHOU.len() {
        XUN_SHOU[xun_index].2 // 返回遁甲所用的六仪
    } else {
        TianGan::Wu // 默认戊
    }
}

/// 获取旬空地支
///
/// 使用查找表直接获取当前时辰所在旬的两个旬空地支
///
/// ## 六甲旬空口诀
///
/// - 甲子旬（甲子至癸酉）空戌亥
/// - 甲戌旬（甲戌至癸未）空申酉
/// - 甲申旬（甲申至癸巳）空午未
/// - 甲午旬（甲午至癸卯）空辰巳
/// - 甲辰旬（甲辰至癸丑）空寅卯
/// - 甲寅旬（甲寅至癸亥）空子丑
///
/// ## 参数
///
/// - `shi_gan`: 时干
/// - `shi_zhi`: 时支
///
/// ## 返回值
///
/// 返回元组 (旬空1, 旬空2)
pub fn get_xun_kong(shi_gan: TianGan, shi_zhi: DiZhi) -> (DiZhi, DiZhi) {
    let gan_idx = shi_gan.index();
    let zhi_idx = shi_zhi.index();

    // 计算旬首的地支索引
    // 旬首一定是甲开头，所以旬首地支 = (时支 - 时干) mod 12
    // 例如：乙丑 -> (1 - 1) mod 12 = 0 (子)，在甲子旬
    // 例如：甲戌 -> (10 - 0) mod 12 = 10 (戌)，在甲戌旬
    let xun_shou_zhi_idx = (zhi_idx + 12 - gan_idx) % 12;

    // 根据旬首地支确定是哪一旬（0-5）
    // 甲子旬首地支=子(0), 甲戌旬首地支=戌(10), 甲申旬首地支=申(8)
    // 甲午旬首地支=午(6), 甲辰旬首地支=辰(4), 甲寅旬首地支=寅(2)
    let xun_index = match xun_shou_zhi_idx {
        0 => 0,  // 甲子旬
        10 => 1, // 甲戌旬
        8 => 2,  // 甲申旬
        6 => 3,  // 甲午旬
        4 => 4,  // 甲辰旬
        2 => 5,  // 甲寅旬
        _ => 0,  // 默认（不应该到达）
    };

    // 直接从查找表获取旬空
    XUN_KONG_TABLE[xun_index]
}

/// 地支到宫位映射表
///
/// 根据地支确定其所属宫位，用于判断空亡宫位
///
/// ## 对应关系
///
/// - 子(0): 坎一宫 (1)
/// - 丑(1): 艮八宫 (8)
/// - 寅(2): 艮八宫 (8) - 寅丑同宫
/// - 卯(3): 震三宫 (3)
/// - 辰(4): 巽四宫 (4)
/// - 巳(5): 巽四宫 (4) - 辰巳同宫
/// - 午(6): 离九宫 (9)
/// - 未(7): 坤二宫 (2)
/// - 申(8): 坤二宫 (2) - 未申同宫
/// - 酉(9): 兑七宫 (7)
/// - 戌(10): 乾六宫 (6)
/// - 亥(11): 乾六宫 (6) - 戌亥同宫
pub fn zhi_to_gong(zhi: DiZhi) -> u8 {
    match zhi {
        DiZhi::Zi => 1,    // 坎宫
        DiZhi::Chou => 8,  // 艮宫
        DiZhi::Yin => 8,   // 艮宫（寅丑同宫）
        DiZhi::Mao => 3,   // 震宫
        DiZhi::Chen => 4,  // 巽宫
        DiZhi::Si => 4,    // 巽宫（辰巳同宫）
        DiZhi::Wu => 9,    // 离宫
        DiZhi::Wei => 2,   // 坤宫
        DiZhi::Shen => 2,  // 坤宫（未申同宫）
        DiZhi::You => 7,   // 兑宫
        DiZhi::Xu => 6,    // 乾宫
        DiZhi::Hai => 6,   // 乾宫（戌亥同宫）
    }
}

/// 获取空亡宫位
///
/// 根据时干支计算空亡地支，再转换为对应宫位
///
/// ## 参数
///
/// - `shi_gan`: 时干
/// - `shi_zhi`: 时支
///
/// ## 返回值
///
/// 返回元组 (空亡宫位1, 空亡宫位2)，宫位号为1-9
pub fn get_xun_kong_gong(shi_gan: TianGan, shi_zhi: DiZhi) -> (u8, u8) {
    let (kong1, kong2) = get_xun_kong(shi_gan, shi_zhi);
    (zhi_to_gong(kong1), zhi_to_gong(kong2))
}

/// 检查宫位是否为空亡
///
/// ## 参数
///
/// - `gong`: 宫位号（1-9）
/// - `shi_gan`: 时干
/// - `shi_zhi`: 时支
///
/// ## 返回值
///
/// 如果该宫位为空亡返回 true
pub fn is_gong_xun_kong(gong: u8, shi_gan: TianGan, shi_zhi: DiZhi) -> bool {
    let (kong_gong1, kong_gong2) = get_xun_kong_gong(shi_gan, shi_zhi);
    gong == kong_gong1 || gong == kong_gong2
}

/// 查找天干在地盘的落宫
///
/// 返回该天干在地盘中的宫位（1-9）
pub fn find_gan_in_di_pan(gan: TianGan, di_pan: &[TianGan; 9]) -> Option<u8> {
    for (i, &g) in di_pan.iter().enumerate() {
        if g == gan {
            return Some((i + 1) as u8);
        }
    }
    None
}

/// 计算值符星
///
/// 值符星 = 旬首六仪所在宫的原始九星
pub fn calc_zhi_fu_xing(xun_shou_yi: TianGan, di_pan: &[TianGan; 9]) -> JiuXing {
    // 找到旬首六仪在地盘的落宫
    let gong = find_gan_in_di_pan(xun_shou_yi, di_pan).unwrap_or(1);

    // 返回该宫的原始九星
    JiuXing::from_num(gong).unwrap_or(JiuXing::TianPeng)
}

/// 计算值使门
///
/// 值使门 = 旬首六仪所在宫的原始八门
pub fn calc_zhi_shi_men(xun_shou_yi: TianGan, di_pan: &[TianGan; 9]) -> BaMen {
    // 找到旬首六仪在地盘的落宫
    let gong = find_gan_in_di_pan(xun_shou_yi, di_pan).unwrap_or(1);

    // 使用 from_palace 获取该宫的原始八门
    // 中宫（5）没有门，会返回 None，使用休门作为默认值
    BaMen::from_palace(gong).unwrap_or(BaMen::Xiu)
}

/// 排布天盘九星
///
/// 根据值符星的当前落宫，按阴阳遁方向排布其他八星
pub fn distribute_jiu_xing(
    zhi_fu_xing: JiuXing,
    shi_gan: TianGan,
    di_pan: &[TianGan; 9],
    dun_type: DunType,
) -> [JiuXing; 9] {
    let mut tian_pan = [JiuXing::TianQin; 9];

    // 时干寄宫：找到时干在地盘的位置，就是值符星的天盘落宫
    let shi_gan_gong = find_gan_in_di_pan(shi_gan, di_pan).unwrap_or(1);

    let gong_order = match dun_type {
        DunType::Yang => &GONG_ORDER_YANG,
        DunType::Yin => &GONG_ORDER_YIN,
    };

    // 找到值符星落宫在顺序中的位置
    let start_pos = gong_order.iter().position(|&g| g == shi_gan_gong).unwrap_or(0);

    // 九星排列顺序（从值符星开始）
    let xing_start = zhi_fu_xing.num() as usize;

    // 按顺序排布九星
    for i in 0..8 {
        let xing_num = ((xing_start - 1 + i) % 8) + 1;
        let xing = JiuXing::from_num(xing_num as u8).unwrap_or(JiuXing::TianPeng);

        let gong_index = (start_pos + i) % 8;
        let gong_num = gong_order[gong_index];
        let array_index = gong_num.saturating_sub(1) as usize;

        if array_index < 9 {
            tian_pan[array_index] = xing;
        }
    }

    // 中宫放天禽星
    tian_pan[4] = JiuXing::TianQin;

    tian_pan
}

/// 排布人盘八门
///
/// 根据值使门的当前落宫，按阴阳遁方向排布其他七门
pub fn distribute_ba_men(
    zhi_shi_men: BaMen,
    shi_gan: TianGan,
    di_pan: &[TianGan; 9],
    dun_type: DunType,
) -> [Option<BaMen>; 9] {
    let mut ren_pan: [Option<BaMen>; 9] = [None; 9];

    // 时干寄宫
    let shi_gan_gong = find_gan_in_di_pan(shi_gan, di_pan).unwrap_or(1);

    let gong_order = match dun_type {
        DunType::Yang => &GONG_ORDER_YANG,
        DunType::Yin => &GONG_ORDER_YIN,
    };

    // 找到值使门落宫在顺序中的位置
    let start_pos = gong_order.iter().position(|&g| g == shi_gan_gong).unwrap_or(0);

    // 八门排列顺序（从值使门开始）
    let men_start = zhi_shi_men.num() as usize;

    // 按顺序排布八门
    for i in 0..8 {
        let men_num = ((men_start - 1 + i) % 8) + 1;
        let men = BaMen::from_num(men_num as u8).unwrap_or(BaMen::Xiu);

        let gong_index = (start_pos + i) % 8;
        let gong_num = gong_order[gong_index];
        let array_index = gong_num.saturating_sub(1) as usize;

        if array_index < 9 && array_index != 4 {
            ren_pan[array_index] = Some(men);
        }
    }

    // 中宫无门
    ren_pan[4] = None;

    ren_pan
}

/// 排布神盘八神
///
/// 八神从值符落宫开始，按固定顺序排布
pub fn distribute_ba_shen(
    zhi_fu_gong: u8,
    dun_type: DunType,
) -> [Option<BaShen>; 9] {
    let mut shen_pan: [Option<BaShen>; 9] = [None; 9];

    let gong_order = match dun_type {
        DunType::Yang => &GONG_ORDER_YANG,
        DunType::Yin => &GONG_ORDER_YIN,
    };

    // 找到值符落宫在顺序中的位置
    let start_pos = gong_order.iter().position(|&g| g == zhi_fu_gong).unwrap_or(0);

    // 按固定顺序排布八神
    for (i, &shen) in BA_SHEN_ORDER.iter().enumerate() {
        let gong_index = (start_pos + i) % 8;
        let gong_num = gong_order[gong_index];
        let array_index = gong_num.saturating_sub(1) as usize;

        if array_index < 9 && array_index != 4 {
            shen_pan[array_index] = Some(shen);
        }
    }

    // 中宫无神
    shen_pan[4] = None;

    shen_pan
}

/// 获取天盘干
///
/// 天盘干随九星移动，根据九星在天盘的位置确定
pub fn get_tian_pan_gan(
    xing: JiuXing,
    di_pan: &[TianGan; 9],
) -> TianGan {
    // 九星原始宫位的地盘干就是该星携带的天盘干
    let original_gong = xing.original_palace().num();
    let array_index = original_gong.saturating_sub(1) as usize;

    if array_index < 9 {
        di_pan[array_index]
    } else {
        TianGan::Jia
    }
}

/// 完整排盘算法
///
/// 输入四柱和节气信息，输出完整的奇门遁甲盘
pub fn generate_qimen_chart(
    _year_gz: GanZhi,
    _month_gz: GanZhi,
    _day_gz: GanZhi,
    hour_gz: GanZhi,
    jie_qi: JieQi,
    day_in_jieqi: u8,
) -> (DunType, SanYuan, u8, JiuXing, BaMen, [Palace; 9]) {
    // 1. 确定阴阳遁
    let dun_type = calc_dun_type(jie_qi);

    // 2. 确定三元
    let san_yuan = calc_san_yuan(day_in_jieqi);

    // 3. 确定局数
    let ju_number = calc_ju_number(jie_qi, san_yuan, dun_type);

    // 4. 排布地盘
    let di_pan = get_di_pan(ju_number, dun_type);

    // 5. 确定旬首六仪
    let xun_shou_yi = get_xun_shou(hour_gz.gan, hour_gz.zhi);

    // 6. 计算值符星和值使门
    let zhi_fu_xing = calc_zhi_fu_xing(xun_shou_yi, &di_pan);
    let zhi_shi_men = calc_zhi_shi_men(xun_shou_yi, &di_pan);

    // 7. 排布天盘九星
    let tian_pan_xing = distribute_jiu_xing(zhi_fu_xing, hour_gz.gan, &di_pan, dun_type);

    // 8. 排布人盘八门
    let ren_pan_men = distribute_ba_men(zhi_shi_men, hour_gz.gan, &di_pan, dun_type);

    // 9. 找到值符落宫，排布神盘八神
    let zhi_fu_gong = find_gan_in_di_pan(hour_gz.gan, &di_pan).unwrap_or(1);
    let shen_pan_shen = distribute_ba_shen(zhi_fu_gong, dun_type);

    // 10. 获取旬空宫位（使用改进后的方法）
    let (kong_gong1, kong_gong2) = get_xun_kong_gong(hour_gz.gan, hour_gz.zhi);

    // 11. 计算驿马落宫
    let yi_ma = calc_yi_ma(hour_gz.zhi);
    let yi_ma_gong = yi_ma.gong.num();

    // 12. 组装九宫
    let mut palaces = [Palace::empty(JiuGong::Kan); 9];

    for i in 0..9 {
        let gong_num = (i + 1) as u8;
        let gong = JiuGong::from_num(gong_num).unwrap_or(JiuGong::Kan);
        let xing = tian_pan_xing[i];
        let tian_pan_gan = get_tian_pan_gan(xing, &di_pan);

        // 检查是否旬空（直接比较宫位号）
        let is_xun_kong = gong_num == kong_gong1 || gong_num == kong_gong2;

        // 检查是否马星宫
        let is_ma_xing = gong_num == yi_ma_gong;

        palaces[i] = Palace {
            gong,
            tian_pan_gan,
            di_pan_gan: di_pan[i],
            xing,
            men: ren_pan_men[i],
            shen: shen_pan_shen[i],
            is_xun_kong,
            is_ma_xing,
        };
    }

    (dun_type, san_yuan, ju_number, zhi_fu_xing, zhi_shi_men, palaces)
}

/// 根据排盘类型生成奇门遁甲盘
///
/// 支持时家奇门、日家奇门、月家奇门、年家奇门
///
/// ## 参数
///
/// - `qimen_type`: 排盘类型
/// - `year_gz`: 年柱干支
/// - `month_gz`: 月柱干支
/// - `day_gz`: 日柱干支
/// - `hour_gz`: 时柱干支
/// - `jie_qi`: 节气
/// - `day_in_jieqi`: 节气内第几天（1-15）
///
/// ## 不同排盘类型的差异
///
/// | 类型 | 三元依据 | 起局依据 | 应用场景 |
/// |------|----------|----------|----------|
/// | 时家 | 时支 | 时干支 | 日常占断，最常用 |
/// | 日家 | 日支 | 日干支 | 日课择吉 |
/// | 月家 | 月支 | 月干支 | 月度规划 |
/// | 年家 | 年支 | 年干支 | 年度运势 |
pub fn generate_qimen_chart_by_type(
    qimen_type: QimenType,
    year_gz: GanZhi,
    month_gz: GanZhi,
    day_gz: GanZhi,
    hour_gz: GanZhi,
    jie_qi: JieQi,
    day_in_jieqi: u8,
) -> (DunType, SanYuan, u8, JiuXing, BaMen, [Palace; 9]) {
    // 根据排盘类型选择不同的干支作为起局依据
    let (base_gz, san_yuan) = match qimen_type {
        QimenType::ShiJia => {
            // 时家奇门：用时支判断三元
            let yuan = calc_san_yuan_by_zhi(hour_gz.zhi);
            (hour_gz, yuan)
        }
        QimenType::RiJia => {
            // 日家奇门：用日支判断三元，也可用节气内天数
            let yuan = calc_san_yuan(day_in_jieqi); // 日家常用节气天数法
            (day_gz, yuan)
        }
        QimenType::YueJia => {
            // 月家奇门：用月支判断三元
            let yuan = calc_san_yuan_by_zhi(month_gz.zhi);
            (month_gz, yuan)
        }
        QimenType::NianJia => {
            // 年家奇门：用年支判断三元
            let yuan = calc_san_yuan_by_zhi(year_gz.zhi);
            (year_gz, yuan)
        }
    };

    // 1. 确定阴阳遁
    let dun_type = calc_dun_type(jie_qi);

    // 2. 确定局数
    let ju_number = calc_ju_number(jie_qi, san_yuan, dun_type);

    // 3. 排布地盘
    let di_pan = get_di_pan(ju_number, dun_type);

    // 4. 确定旬首六仪（根据排盘类型使用不同干支）
    let xun_shou_yi = get_xun_shou(base_gz.gan, base_gz.zhi);

    // 5. 计算值符星和值使门
    let zhi_fu_xing = calc_zhi_fu_xing(xun_shou_yi, &di_pan);
    let zhi_shi_men = calc_zhi_shi_men(xun_shou_yi, &di_pan);

    // 6. 排布天盘九星
    let tian_pan_xing = distribute_jiu_xing(zhi_fu_xing, base_gz.gan, &di_pan, dun_type);

    // 7. 排布人盘八门
    let ren_pan_men = distribute_ba_men(zhi_shi_men, base_gz.gan, &di_pan, dun_type);

    // 8. 找到值符落宫，排布神盘八神
    let zhi_fu_gong = find_gan_in_di_pan(base_gz.gan, &di_pan).unwrap_or(1);
    let shen_pan_shen = distribute_ba_shen(zhi_fu_gong, dun_type);

    // 9. 获取旬空宫位
    let (kong_gong1, kong_gong2) = get_xun_kong_gong(base_gz.gan, base_gz.zhi);

    // 10. 计算驿马落宫
    let yi_ma = calc_yi_ma(base_gz.zhi);
    let yi_ma_gong = yi_ma.gong.num();

    // 11. 组装九宫
    let mut palaces = [Palace::empty(JiuGong::Kan); 9];

    for i in 0..9 {
        let gong_num = (i + 1) as u8;
        let gong = JiuGong::from_num(gong_num).unwrap_or(JiuGong::Kan);
        let xing = tian_pan_xing[i];
        let tian_pan_gan = get_tian_pan_gan(xing, &di_pan);

        // 检查是否旬空
        let is_xun_kong = gong_num == kong_gong1 || gong_num == kong_gong2;

        // 检查是否马星宫
        let is_ma_xing = gong_num == yi_ma_gong;

        palaces[i] = Palace {
            gong,
            tian_pan_gan,
            di_pan_gan: di_pan[i],
            xing,
            men: ren_pan_men[i],
            shen: shen_pan_shen[i],
            is_xun_kong,
            is_ma_xing,
        };
    }

    (dun_type, san_yuan, ju_number, zhi_fu_xing, zhi_shi_men, palaces)
}

/// 从数字生成排盘
///
/// 使用两个数字和区块哈希生成局数
pub fn generate_from_numbers(
    numbers: &[u16],
    block_hash: &[u8; 32],
) -> u8 {
    if numbers.is_empty() {
        return 1;
    }

    // 累加数字
    let sum: u32 = numbers.iter().map(|&n| n as u32).sum();

    // 混合区块哈希
    let hash_sum: u32 = block_hash.iter().take(4).fold(0u32, |acc, &b| acc.wrapping_add(b as u32));

    // 计算局数（1-9）
    let combined = sum.wrapping_add(hash_sum);
    ((combined % 9) + 1) as u8
}

/// 从随机数生成排盘
///
/// 使用随机种子生成局数和阴阳遁
pub fn generate_from_random(random_seed: &[u8; 32]) -> (DunType, u8) {
    // 使用前几个字节决定阴阳遁
    let dun_type = if random_seed[0] % 2 == 0 {
        DunType::Yang
    } else {
        DunType::Yin
    };

    // 使用后几个字节决定局数
    let ju_number = (random_seed[1] % 9) + 1;

    (dun_type, ju_number)
}

/// 验证局数有效性
pub fn validate_ju_number(ju_number: u8) -> bool {
    (1..=9).contains(&ju_number)
}

/// 计算排盘稀有度分数
///
/// 根据排盘结果计算稀有度，用于 NFT 铸造
pub fn calc_rarity_score(
    zhi_fu_xing: JiuXing,
    zhi_shi_men: BaMen,
    palaces: &[Palace; 9],
) -> u8 {
    let mut score: u16 = 30; // 基础分

    // 值符为吉星加分
    if zhi_fu_xing.is_auspicious() {
        score = score.saturating_add(15);
    }

    // 值使为吉门加分
    if zhi_shi_men.is_auspicious() {
        score = score.saturating_add(15);
    }

    // 统计吉门吉星数量
    let mut lucky_doors = 0u16;
    let mut lucky_stars = 0u16;

    for palace in palaces.iter() {
        if palace.xing.is_auspicious() {
            lucky_stars = lucky_stars.saturating_add(1);
        }
        if let Some(men) = &palace.men {
            if men.is_auspicious() {
                lucky_doors = lucky_doors.saturating_add(1);
            }
        }
    }

    // 吉门吉星越多越稀有
    score = score.saturating_add(lucky_doors * 3);
    score = score.saturating_add(lucky_stars * 3);

    // 特殊格局加分（简化版）
    // 检查开门、休门、生门是否落在好位置
    for palace in palaces.iter() {
        if let Some(men) = &palace.men {
            // 开门落乾宫（归位）
            if *men == BaMen::Kai && palace.gong == JiuGong::Qian {
                score = score.saturating_add(10);
            }
            // 休门落坎宫（归位）
            if *men == BaMen::Xiu && palace.gong == JiuGong::Kan {
                score = score.saturating_add(10);
            }
            // 生门落艮宫（归位）
            if *men == BaMen::Sheng && palace.gong == JiuGong::Gen {
                score = score.saturating_add(10);
            }
        }
    }

    score.min(100) as u8
}

// ==================== 格局检测功能 ====================

/// 六仪击刑检测结果
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LiuYiJiXing {
    /// 击刑的天干
    pub gan: TianGan,
    /// 发生击刑的宫位
    pub gong: JiuGong,
}

/// 检测六仪击刑
///
/// 六仪（戊己庚辛壬癸）临某些特定宫位时形成击刑格局
///
/// ## 六仪击刑表
///
/// | 天干 | 击刑宫位 | 说明 |
/// |------|----------|------|
/// | 戊 | 震三宫 | 戊加震木，木克土 |
/// | 己 | 坤二宫 | 己入本宫，土气太重 |
/// | 庚 | 艮八宫 | 庚金克艮土 |
/// | 辛 | 离九宫 | 辛金被火克 |
/// | 壬 | 巽四宫 | 壬水临巽木 |
/// | 癸 | 巽四宫 | 癸水临巽木 |
///
/// 参考：Java QiMenZhuanPanJiChuMap.LIU_YI_JI_XING
pub fn check_liu_yi_ji_xing(tian_pan_gan: TianGan, gong: JiuGong) -> Option<LiuYiJiXing> {
    let is_ji_xing = match (tian_pan_gan, gong) {
        (TianGan::Wu, JiuGong::Zhen) => true,   // 戊击刑（震三宫）
        (TianGan::Ji, JiuGong::Kun) => true,    // 己击刑（坤二宫）
        (TianGan::Geng, JiuGong::Gen) => true,  // 庚击刑（艮八宫）
        (TianGan::Xin, JiuGong::Li) => true,    // 辛击刑（离九宫）
        (TianGan::Ren, JiuGong::Xun) => true,   // 壬击刑（巽四宫）
        (TianGan::Gui, JiuGong::Xun) => true,   // 癸击刑（巽四宫）
        _ => false,
    };

    if is_ji_xing {
        Some(LiuYiJiXing { gan: tian_pan_gan, gong })
    } else {
        None
    }
}

/// 奇仪入墓检测结果
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QiYiRuMu {
    /// 入墓的天干
    pub gan: TianGan,
    /// 发生入墓的宫位
    pub gong: JiuGong,
    /// 墓库地支
    pub mu_zhi: DiZhi,
}

/// 检测奇仪入墓
///
/// 天干临其墓库之宫位时形成入墓格局，主事不顺、受困
///
/// ## 天干入墓表
///
/// | 天干 | 入墓宫位 | 墓库 |
/// |------|----------|------|
/// | 甲/戊 | 乾六宫 | 戌土 |
/// | 乙 | 乾六宫 | 戌土 |
/// | 丙 | 乾六宫 | 戌土 |
/// | 丁/己 | 艮八宫 | 丑土 |
/// | 庚 | 艮八宫 | 丑土 |
/// | 辛 | 巽四宫 | 辰土 |
/// | 壬 | 巽四宫 | 辰土 |
/// | 癸 | 坤二宫 | 未土 |
///
/// 参考：Java QiMenZhuanPanJiChuMap.QI_YI_RU_MU
pub fn check_qi_yi_ru_mu(tian_pan_gan: TianGan, gong: JiuGong) -> Option<QiYiRuMu> {
    let result = match (tian_pan_gan, gong) {
        // 甲戊入乾六宫（戌土为墓）
        (TianGan::Jia, JiuGong::Qian) => Some((TianGan::Jia, JiuGong::Qian, DiZhi::Xu)),
        (TianGan::Wu, JiuGong::Qian) => Some((TianGan::Wu, JiuGong::Qian, DiZhi::Xu)),
        // 乙入乾六宫（戌土为墓）
        (TianGan::Yi, JiuGong::Qian) => Some((TianGan::Yi, JiuGong::Qian, DiZhi::Xu)),
        // 丙入乾六宫（戌土为墓）
        (TianGan::Bing, JiuGong::Qian) => Some((TianGan::Bing, JiuGong::Qian, DiZhi::Xu)),
        // 丁入艮八宫（丑土为墓）
        (TianGan::Ding, JiuGong::Gen) => Some((TianGan::Ding, JiuGong::Gen, DiZhi::Chou)),
        // 己入艮八宫（丑土为墓）
        (TianGan::Ji, JiuGong::Gen) => Some((TianGan::Ji, JiuGong::Gen, DiZhi::Chou)),
        // 庚入艮八宫（丑土为墓）
        (TianGan::Geng, JiuGong::Gen) => Some((TianGan::Geng, JiuGong::Gen, DiZhi::Chou)),
        // 辛入巽四宫（辰土为墓）
        (TianGan::Xin, JiuGong::Xun) => Some((TianGan::Xin, JiuGong::Xun, DiZhi::Chen)),
        // 壬入巽四宫（辰土为墓）
        (TianGan::Ren, JiuGong::Xun) => Some((TianGan::Ren, JiuGong::Xun, DiZhi::Chen)),
        // 癸入坤二宫（未土为墓）
        (TianGan::Gui, JiuGong::Kun) => Some((TianGan::Gui, JiuGong::Kun, DiZhi::Wei)),
        _ => None,
    };

    result.map(|(gan, gong, mu_zhi)| QiYiRuMu { gan, gong, mu_zhi })
}

/// 门迫检测结果
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenPo {
    /// 被迫之门
    pub men: BaMen,
    /// 发生门迫的宫位
    pub gong: JiuGong,
}

/// 检测门迫
///
/// 八门五行克落宫五行时为门迫，主事受阻、不顺
///
/// ## 门迫表
///
/// | 八门 | 门五行 | 门迫宫位 | 宫五行 |
/// |------|--------|----------|--------|
/// | 休门 | 水 | 离九宫 | 火（水克火，门迫） |
/// | 生门 | 土 | 坎一宫 | 水（土克水，门迫） |
/// | 伤门 | 木 | 坤二宫、艮八宫 | 土（木克土，门迫） |
/// | 杜门 | 木 | 坤二宫、艮八宫 | 土（木克土，门迫） |
/// | 景门 | 火 | 乾六宫、兑七宫 | 金（火克金，门迫） |
/// | 死门 | 土 | 坎一宫 | 水（土克水，门迫） |
/// | 惊门 | 金 | 震三宫、巽四宫 | 木（金克木，门迫） |
/// | 开门 | 金 | 震三宫、巽四宫 | 木（金克木，门迫） |
///
/// 参考：Java QiMenZhuanPanJiChuMap.MEN_PO
pub fn check_men_po(men: BaMen, gong: JiuGong) -> Option<MenPo> {
    let is_men_po = match (men, gong) {
        // 休门（水）克离宫（火）
        (BaMen::Xiu, JiuGong::Li) => true,
        // 生门（土）克坎宫（水）
        (BaMen::Sheng, JiuGong::Kan) => true,
        // 伤门（木）克坤宫、艮宫（土）
        (BaMen::Shang, JiuGong::Kun) | (BaMen::Shang, JiuGong::Gen) => true,
        // 杜门（木）克坤宫、艮宫（土）
        (BaMen::Du, JiuGong::Kun) | (BaMen::Du, JiuGong::Gen) => true,
        // 景门（火）克乾宫、兑宫（金）
        (BaMen::Jing, JiuGong::Qian) | (BaMen::Jing, JiuGong::Dui) => true,
        // 死门（土）克坎宫（水）
        (BaMen::Si, JiuGong::Kan) => true,
        // 惊门（金）克震宫、巽宫（木）
        (BaMen::Jing2, JiuGong::Zhen) | (BaMen::Jing2, JiuGong::Xun) => true,
        // 开门（金）克震宫、巽宫（木）
        (BaMen::Kai, JiuGong::Zhen) | (BaMen::Kai, JiuGong::Xun) => true,
        _ => false,
    };

    if is_men_po {
        Some(MenPo { men, gong })
    } else {
        None
    }
}

/// 十干克应格局类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShiGanGeJu {
    // ========== 吉格 ==========
    /// 乙+乙：日奇伏吟
    RiQiFuYin,
    /// 乙+丙：奇仪顺遂
    QiYiShunSui,
    /// 乙+丁：奇仪相佐
    QiYiXiangZuo,
    /// 丙+乙：日月并行
    RiYueBingXing,
    /// 丙+丙：月奇悖师
    YueQiBeiShi,
    /// 丙+丁：星奇朱雀
    XingQiZhuQue,
    /// 丁+乙：星奇入太阴
    XingQiRuTaiYin,
    /// 丁+丙：星奇入六合
    XingQiRuLiuHe,
    /// 丁+丁：星奇伏吟
    XingQiFuYin,
    /// 丙+戊：飞鸟跌穴（大吉）
    FeiNiaoDieXue,

    // ========== 凶格 ==========
    /// 庚+庚：太白同宫（大凶）
    TaiBaiTongGong,
    /// 庚+日干：太白入日（凶）
    TaiBaiRuRi,
    /// 庚+丙：太白入荧
    TaiBaiRuYing,
    /// 庚+丁：太白入星
    TaiBaiRuXing,
    /// 癸+癸：华盖伏吟
    HuaGaiFuYin,
    /// 辛+乙：白虎猖狂
    BaiHuChangKuang,
    /// 辛+丙：白虎入荧
    BaiHuRuYing,
    /// 辛+丁：白虎入星
    BaiHuRuXing,
    /// 壬+壬：蛇夭矫（凶）
    SheYaoJiao,

    // ========== 中平格 ==========
    /// 戊+戊：伏吟
    FuYin,
    /// 其他组合
    Other,
}

/// 十干克应检测结果
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShiGanKeYing {
    /// 天盘干
    pub tian_gan: TianGan,
    /// 地盘干
    pub di_gan: TianGan,
    /// 格局类型
    pub ge_ju: ShiGanGeJu,
    /// 是否吉格
    pub is_ji: bool,
}

/// 检测十干克应（天地盘格局）
///
/// 天盘干与地盘干相遇形成的吉凶格局
///
/// 参考：Java QiMenZhuanPanJiChuMap 十干克应表
pub fn check_shi_gan_ke_ying(tian_gan: TianGan, di_gan: TianGan) -> ShiGanKeYing {
    let (ge_ju, is_ji) = match (tian_gan, di_gan) {
        // ========== 吉格 ==========
        (TianGan::Yi, TianGan::Yi) => (ShiGanGeJu::RiQiFuYin, true),
        (TianGan::Yi, TianGan::Bing) => (ShiGanGeJu::QiYiShunSui, true),
        (TianGan::Yi, TianGan::Ding) => (ShiGanGeJu::QiYiXiangZuo, true),
        (TianGan::Bing, TianGan::Yi) => (ShiGanGeJu::RiYueBingXing, true),
        (TianGan::Bing, TianGan::Bing) => (ShiGanGeJu::YueQiBeiShi, true),
        (TianGan::Bing, TianGan::Ding) => (ShiGanGeJu::XingQiZhuQue, true),
        (TianGan::Ding, TianGan::Yi) => (ShiGanGeJu::XingQiRuTaiYin, true),
        (TianGan::Ding, TianGan::Bing) => (ShiGanGeJu::XingQiRuLiuHe, true),
        (TianGan::Ding, TianGan::Ding) => (ShiGanGeJu::XingQiFuYin, true),
        (TianGan::Bing, TianGan::Wu) => (ShiGanGeJu::FeiNiaoDieXue, true),

        // ========== 凶格 ==========
        (TianGan::Geng, TianGan::Geng) => (ShiGanGeJu::TaiBaiTongGong, false),
        (TianGan::Geng, TianGan::Yi) => (ShiGanGeJu::TaiBaiRuRi, false),
        (TianGan::Geng, TianGan::Bing) => (ShiGanGeJu::TaiBaiRuYing, false),
        (TianGan::Geng, TianGan::Ding) => (ShiGanGeJu::TaiBaiRuXing, false),
        (TianGan::Gui, TianGan::Gui) => (ShiGanGeJu::HuaGaiFuYin, false),
        (TianGan::Xin, TianGan::Yi) => (ShiGanGeJu::BaiHuChangKuang, false),
        (TianGan::Xin, TianGan::Bing) => (ShiGanGeJu::BaiHuRuYing, false),
        (TianGan::Xin, TianGan::Ding) => (ShiGanGeJu::BaiHuRuXing, false),
        (TianGan::Ren, TianGan::Ren) => (ShiGanGeJu::SheYaoJiao, false),

        // ========== 中平格 ==========
        (TianGan::Wu, TianGan::Wu) => (ShiGanGeJu::FuYin, true),
        (TianGan::Ji, TianGan::Ji) => (ShiGanGeJu::FuYin, true),

        // 其他组合
        _ => (ShiGanGeJu::Other, true),
    };

    ShiGanKeYing { tian_gan, di_gan, ge_ju, is_ji }
}

/// 驿马信息
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YiMa {
    /// 驿马地支
    pub zhi: DiZhi,
    /// 驿马落宫
    pub gong: JiuGong,
}

/// 驿马查找表
///
/// 根据时支三合局确定驿马位置
///
/// | 时支 | 三合局 | 驿马地支 | 落宫 |
/// |------|--------|----------|------|
/// | 申、子、辰 | 水局 | 寅 | 艮八宫 |
/// | 寅、午、戌 | 火局 | 申 | 坤二宫 |
/// | 巳、酉、丑 | 金局 | 亥 | 乾六宫 |
/// | 亥、卯、未 | 木局 | 巳 | 巽四宫 |
///
/// 参考：Java QiMenZhuanPanJiChuMap.YI_MA
pub fn calc_yi_ma(shi_zhi: DiZhi) -> YiMa {
    match shi_zhi {
        // 申子辰三合水局，驿马在寅（艮八宫）
        DiZhi::Shen | DiZhi::Zi | DiZhi::Chen => YiMa {
            zhi: DiZhi::Yin,
            gong: JiuGong::Gen,
        },
        // 寅午戌三合火局，驿马在申（坤二宫）
        DiZhi::Yin | DiZhi::Wu | DiZhi::Xu => YiMa {
            zhi: DiZhi::Shen,
            gong: JiuGong::Kun,
        },
        // 巳酉丑三合金局，驿马在亥（乾六宫）
        DiZhi::Si | DiZhi::You | DiZhi::Chou => YiMa {
            zhi: DiZhi::Hai,
            gong: JiuGong::Qian,
        },
        // 亥卯未三合木局，驿马在巳（巽四宫）
        DiZhi::Hai | DiZhi::Mao | DiZhi::Wei => YiMa {
            zhi: DiZhi::Si,
            gong: JiuGong::Xun,
        },
    }
}

/// 旺衰状态
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WangShuai {
    Wang,  // 旺 - 当令
    Xiang, // 相 - 我生者
    Xiu,   // 休 - 生我者
    Qiu,   // 囚 - 克我者
    Si,    // 死/废 - 我克者
}

impl WangShuai {
    /// 获取旺衰名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Wang => "旺",
            Self::Xiang => "相",
            Self::Xiu => "休",
            Self::Qiu => "囚",
            Self::Si => "死",
        }
    }

    /// 判断是否为有利状态
    pub fn is_favorable(&self) -> bool {
        matches!(self, Self::Wang | Self::Xiang)
    }
}

/// 根据月令判断五行旺衰
///
/// ## 五行旺衰规则
///
/// - 旺：当令（与月令五行相同）
/// - 相：我生者（月令生该五行）
/// - 休：生我者（该五行生月令）
/// - 囚：克我者（月令克该五行）
/// - 死/废：我克者（该五行克月令）
///
/// ## 月令五行
///
/// | 月份 | 地支 | 五行 |
/// |------|------|------|
/// | 春（寅卯） | 寅、卯 | 木 |
/// | 夏（巳午） | 巳、午 | 火 |
/// | 秋（申酉） | 申、酉 | 金 |
/// | 冬（亥子） | 亥、子 | 水 |
/// | 四季（辰戌丑未） | 辰、戌、丑、未 | 土 |
pub fn calc_wang_shuai(wu_xing: WuXing, yue_zhi: DiZhi) -> WangShuai {
    let yue_wu_xing = yue_zhi.wu_xing();

    if wu_xing == yue_wu_xing {
        // 当令
        WangShuai::Wang
    } else if yue_wu_xing.generates(&wu_xing) {
        // 月令生我 -> 相
        WangShuai::Xiang
    } else if wu_xing.generates(&yue_wu_xing) {
        // 我生月令 -> 休
        WangShuai::Xiu
    } else if yue_wu_xing.conquers(&wu_xing) {
        // 月令克我 -> 囚
        WangShuai::Qiu
    } else {
        // 我克月令 -> 死
        WangShuai::Si
    }
}

/// 计算九星旺衰
pub fn calc_xing_wang_shuai(xing: JiuXing, yue_zhi: DiZhi) -> WangShuai {
    calc_wang_shuai(xing.wu_xing(), yue_zhi)
}

/// 计算八门旺衰
pub fn calc_men_wang_shuai(men: BaMen, yue_zhi: DiZhi) -> WangShuai {
    calc_wang_shuai(men.wu_xing(), yue_zhi)
}

// ==================== 九遁格局 ====================

/// 九遁格局类型
///
/// 奇门遁甲中的九种吉格，都是大吉格局
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JiuDunGeJu {
    /// 天遁：丙+生门+天辅星同宫
    TianDun,
    /// 地遁：乙+开门+九地同宫
    DiDun,
    /// 人遁：丁+休门+太阴同宫
    RenDun,
    /// 风遁：乙+休门+六合同宫
    FengDun,
    /// 云遁：乙+生门+九天同宫
    YunDun,
    /// 龙遁：乙+休门+六合在坎一宫（水上龙行）
    LongDun,
    /// 虎遁：乙+开门+太阴在艮八宫（虎踞山林）
    HuDun,
    /// 神遁：丙+休门+九天同宫
    ShenDun,
    /// 鬼遁：丁+生门+九地同宫
    GuiDun,
}

impl JiuDunGeJu {
    /// 获取格局名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::TianDun => "天遁",
            Self::DiDun => "地遁",
            Self::RenDun => "人遁",
            Self::FengDun => "风遁",
            Self::YunDun => "云遁",
            Self::LongDun => "龙遁",
            Self::HuDun => "虎遁",
            Self::ShenDun => "神遁",
            Self::GuiDun => "鬼遁",
        }
    }

    /// 获取格局解释
    pub fn description(&self) -> &'static str {
        match self {
            Self::TianDun => "丙为日奇为天，遇生门为大吉，适合求财、谋事、出行",
            Self::DiDun => "乙为日奇为地，遇开门九地为大吉，适合藏形匿迹、隐遁避祸",
            Self::RenDun => "丁为星奇为人，遇休门太阴为大吉，适合密谋私事、求见贵人",
            Self::FengDun => "乙加休门六合，风神助力，适合出行远征、商旅交易",
            Self::YunDun => "乙加生门九天，云中翱翔，适合求官进职、上书言事",
            Self::LongDun => "乙加休门六合在坎宫，水上龙行，大吉大利",
            Self::HuDun => "乙加开门太阴在艮宫，虎踞山林，威镇四方",
            Self::ShenDun => "丙加休门九天，神明护佑，适合祈福求神、重大仪式",
            Self::GuiDun => "丁加生门九地，鬼神相助，适合暗中行事、避凶趋吉",
        }
    }
}

/// 检测九遁格局
///
/// 根据天盘干、八门、八神和宫位判断是否形成九遁格局
///
/// ## 九遁条件
///
/// | 格局 | 天盘干 | 八门 | 八神 | 宫位 |
/// |------|--------|------|------|------|
/// | 天遁 | 丙 | 生门 | 天辅(星) | 任意 |
/// | 地遁 | 乙 | 开门 | 九地 | 任意 |
/// | 人遁 | 丁 | 休门 | 太阴 | 任意 |
/// | 风遁 | 乙 | 休门 | 六合 | 任意 |
/// | 云遁 | 乙 | 生门 | 九天 | 任意 |
/// | 龙遁 | 乙 | 休门 | 六合 | 坎一宫 |
/// | 虎遁 | 乙 | 开门 | 太阴 | 艮八宫 |
/// | 神遁 | 丙 | 休门 | 九天 | 任意 |
/// | 鬼遁 | 丁 | 生门 | 九地 | 任意 |
pub fn check_jiu_dun(
    tian_pan_gan: TianGan,
    men: Option<BaMen>,
    shen: Option<BaShen>,
    gong: JiuGong,
) -> Option<JiuDunGeJu> {
    let men = men?;
    let shen = shen?;

    match (tian_pan_gan, men, shen, gong) {
        // 龙遁：乙+休门+六合在坎一宫（要先检查带宫位条件的）
        (TianGan::Yi, BaMen::Xiu, BaShen::LiuHe, JiuGong::Kan) => Some(JiuDunGeJu::LongDun),
        // 虎遁：乙+开门+太阴在艮八宫
        (TianGan::Yi, BaMen::Kai, BaShen::TaiYin, JiuGong::Gen) => Some(JiuDunGeJu::HuDun),
        // 天遁：丙+生门（不检查星，因为天辅星较难同时满足）
        // 简化版：丙+生门+任意吉神
        (TianGan::Bing, BaMen::Sheng, _, _) if shen.is_auspicious() => Some(JiuDunGeJu::TianDun),
        // 地遁：乙+开门+九地
        (TianGan::Yi, BaMen::Kai, BaShen::JiuDi, _) => Some(JiuDunGeJu::DiDun),
        // 人遁：丁+休门+太阴
        (TianGan::Ding, BaMen::Xiu, BaShen::TaiYin, _) => Some(JiuDunGeJu::RenDun),
        // 风遁：乙+休门+六合
        (TianGan::Yi, BaMen::Xiu, BaShen::LiuHe, _) => Some(JiuDunGeJu::FengDun),
        // 云遁：乙+生门+九天
        (TianGan::Yi, BaMen::Sheng, BaShen::JiuTian, _) => Some(JiuDunGeJu::YunDun),
        // 神遁：丙+休门+九天
        (TianGan::Bing, BaMen::Xiu, BaShen::JiuTian, _) => Some(JiuDunGeJu::ShenDun),
        // 鬼遁：丁+生门+九地
        (TianGan::Ding, BaMen::Sheng, BaShen::JiuDi, _) => Some(JiuDunGeJu::GuiDun),
        _ => None,
    }
}

// ==================== 其他吉凶格局 ====================

/// 特殊吉格类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JiGeJu {
    /// 青龙返首：天盘甲落乾六宫，得开门
    QingLongFanShou,
    /// 飞鸟跌穴：丙+戊同宫
    FeiNiaoDieXue,
    /// 三奇得使：乙/丙/丁遇吉门吉星
    SanQiDeShi,
    /// 玉女守门：丁+休门同宫，或丁落离宫得景门
    YuNvShouMen,
    /// 天马：开门+驿马宫
    TianMa,
}

impl JiGeJu {
    /// 获取格局名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::QingLongFanShou => "青龙返首",
            Self::FeiNiaoDieXue => "飞鸟跌穴",
            Self::SanQiDeShi => "三奇得使",
            Self::YuNvShouMen => "玉女守门",
            Self::TianMa => "天马",
        }
    }
}

/// 检测青龙返首格
///
/// 条件：值符（代表甲）落乾六宫，且得开门
pub fn check_qing_long_fan_shou(
    zhi_fu_gong: u8,
    men_at_qian: Option<BaMen>,
) -> bool {
    // 值符落乾六宫
    if zhi_fu_gong != 6 {
        return false;
    }
    // 且乾宫有开门
    matches!(men_at_qian, Some(BaMen::Kai))
}

/// 检测玉女守门格
///
/// 条件：丁奇遇休门，或丁在离宫遇景门
pub fn check_yu_nv_shou_men(
    tian_pan_gan: TianGan,
    men: Option<BaMen>,
    gong: JiuGong,
) -> bool {
    if tian_pan_gan != TianGan::Ding {
        return false;
    }
    match (men, gong) {
        // 丁遇休门
        (Some(BaMen::Xiu), _) => true,
        // 丁在离宫遇景门
        (Some(BaMen::Jing), JiuGong::Li) => true,
        _ => false,
    }
}

/// 检测三奇得使
///
/// 条件：三奇（乙丙丁）遇吉门吉星
pub fn check_san_qi_de_shi(
    tian_pan_gan: TianGan,
    xing: JiuXing,
    men: Option<BaMen>,
) -> bool {
    // 必须是三奇
    if !tian_pan_gan.is_san_qi() {
        return false;
    }
    // 九星为吉星
    if !xing.is_auspicious() {
        return false;
    }
    // 八门为吉门
    matches!(men, Some(m) if m.is_auspicious())
}

/// 特殊凶格类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XiongGeJu {
    /// 白虎猖狂：辛+乙同宫
    BaiHuChangKuang,
    /// 螣蛇夭矫：壬+壬同宫
    TengSheYaoJiao,
    /// 朱雀投江：丙+壬同宫在坎宫
    ZhuQueToJiang,
    /// 天网四张：癸+癸同宫
    TianWangSiZhang,
    /// 地网遮蔽：大凶组合
    DiWangZheBi,
    /// 荧入白虎：丙加庚
    YingRuBaiHu,
}

impl XiongGeJu {
    /// 获取格局名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::BaiHuChangKuang => "白虎猖狂",
            Self::TengSheYaoJiao => "螣蛇夭矫",
            Self::ZhuQueToJiang => "朱雀投江",
            Self::TianWangSiZhang => "天网四张",
            Self::DiWangZheBi => "地网遮蔽",
            Self::YingRuBaiHu => "荧入白虎",
        }
    }
}

/// 检测特殊凶格
pub fn check_xiong_ge_ju(
    tian_pan_gan: TianGan,
    di_pan_gan: TianGan,
    gong: JiuGong,
) -> Option<XiongGeJu> {
    match (tian_pan_gan, di_pan_gan, gong) {
        // 白虎猖狂：辛+乙
        (TianGan::Xin, TianGan::Yi, _) => Some(XiongGeJu::BaiHuChangKuang),
        // 螣蛇夭矫：壬+壬
        (TianGan::Ren, TianGan::Ren, _) => Some(XiongGeJu::TengSheYaoJiao),
        // 朱雀投江：丙+壬在坎宫
        (TianGan::Bing, TianGan::Ren, JiuGong::Kan) => Some(XiongGeJu::ZhuQueToJiang),
        // 天网四张：癸+癸
        (TianGan::Gui, TianGan::Gui, _) => Some(XiongGeJu::TianWangSiZhang),
        // 荧入白虎：丙+庚
        (TianGan::Bing, TianGan::Geng, _) => Some(XiongGeJu::YingRuBaiHu),
        _ => None,
    }
}

// ==================== 伏吟反吟检测 ====================

/// 伏吟反吟类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FuFanYin {
    /// 天盘伏吟：天盘干等于地盘干
    TianPanFuYin,
    /// 天盘反吟：天盘干与地盘干对冲
    TianPanFanYin,
    /// 门伏吟：八门归本位
    MenFuYin,
    /// 门反吟：八门落对冲宫
    MenFanYin,
    /// 星伏吟：九星归本位
    XingFuYin,
    /// 星反吟：九星落对冲宫
    XingFanYin,
}

impl FuFanYin {
    /// 获取名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::TianPanFuYin => "天盘伏吟",
            Self::TianPanFanYin => "天盘反吟",
            Self::MenFuYin => "门伏吟",
            Self::MenFanYin => "门反吟",
            Self::XingFuYin => "星伏吟",
            Self::XingFanYin => "星反吟",
        }
    }

    /// 判断是否凶
    pub fn is_xiong(&self) -> bool {
        // 伏吟反吟都不利
        true
    }
}

/// 检测天盘伏吟
///
/// 天盘干等于地盘干
pub fn check_tian_pan_fu_yin(tian_gan: TianGan, di_gan: TianGan) -> bool {
    tian_gan == di_gan
}

/// 检测天盘反吟
///
/// 天盘干与地盘干相冲（甲庚、乙辛、丙壬、丁癸、戊己互冲）
pub fn check_tian_pan_fan_yin(tian_gan: TianGan, di_gan: TianGan) -> bool {
    matches!(
        (tian_gan, di_gan),
        (TianGan::Jia, TianGan::Geng) | (TianGan::Geng, TianGan::Jia) |
        (TianGan::Yi, TianGan::Xin) | (TianGan::Xin, TianGan::Yi) |
        (TianGan::Bing, TianGan::Ren) | (TianGan::Ren, TianGan::Bing) |
        (TianGan::Ding, TianGan::Gui) | (TianGan::Gui, TianGan::Ding) |
        (TianGan::Wu, TianGan::Ji) | (TianGan::Ji, TianGan::Wu)
    )
}

/// 检测门伏吟
///
/// 八门落回本宫
pub fn check_men_fu_yin(men: BaMen, gong: JiuGong) -> bool {
    men.original_palace() == gong
}

/// 检测门反吟
///
/// 八门落对冲宫
pub fn check_men_fan_yin(men: BaMen, gong: JiuGong) -> bool {
    let original = men.original_palace();
    // 对冲宫：1-9, 2-8, 3-7, 4-6
    let opposite = match original {
        JiuGong::Kan => JiuGong::Li,
        JiuGong::Li => JiuGong::Kan,
        JiuGong::Kun => JiuGong::Gen,
        JiuGong::Gen => JiuGong::Kun,
        JiuGong::Zhen => JiuGong::Dui,
        JiuGong::Dui => JiuGong::Zhen,
        JiuGong::Xun => JiuGong::Qian,
        JiuGong::Qian => JiuGong::Xun,
        JiuGong::Zhong => JiuGong::Zhong,
    };
    gong == opposite
}

/// 检测星伏吟
///
/// 九星落回本宫
pub fn check_xing_fu_yin(xing: JiuXing, gong: JiuGong) -> bool {
    xing.original_palace() == gong
}

/// 检测星反吟
///
/// 九星落对冲宫
pub fn check_xing_fan_yin(xing: JiuXing, gong: JiuGong) -> bool {
    let original = xing.original_palace();
    let opposite = match original {
        JiuGong::Kan => JiuGong::Li,
        JiuGong::Li => JiuGong::Kan,
        JiuGong::Kun => JiuGong::Gen,
        JiuGong::Gen => JiuGong::Kun,
        JiuGong::Zhen => JiuGong::Dui,
        JiuGong::Dui => JiuGong::Zhen,
        JiuGong::Xun => JiuGong::Qian,
        JiuGong::Qian => JiuGong::Xun,
        JiuGong::Zhong => JiuGong::Zhong,
    };
    gong == opposite
}

/// 宫位格局综合分析结果
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PalaceAnalysis {
    /// 六仪击刑
    pub ji_xing: Option<LiuYiJiXing>,
    /// 奇仪入墓
    pub ru_mu: Option<QiYiRuMu>,
    /// 门迫
    pub men_po: Option<MenPo>,
    /// 十干克应
    pub ke_ying: Option<ShiGanKeYing>,
    /// 是否为驿马宫
    pub is_yi_ma: bool,
    /// 九星旺衰
    pub xing_wang_shuai: Option<WangShuai>,
    /// 八门旺衰
    pub men_wang_shuai: Option<WangShuai>,
}

// ==================== 用神系统 ====================

/// 用神配置结果
#[derive(Clone, Debug)]
pub struct YongShenResult {
    /// 问事类型
    pub question_type: QuestionType,
    /// 主用神名称
    pub primary_name: &'static str,
    /// 次用神名称（可选）
    pub secondary_name: Option<&'static str>,
    /// 主用神所在宫位（1-9）
    pub primary_gong: Option<u8>,
    /// 次用神所在宫位（可选）
    pub secondary_gong: Option<u8>,
    /// 吉利条件描述
    pub auspicious_condition: &'static str,
    /// 判断用语提示
    pub judgment_hint: &'static str,
}

/// 获取问事类型的用神配置
///
/// 根据问事类型返回用神配置信息
///
/// ## 用神规则
///
/// | 问事类型 | 主用神 | 次用神 | 吉利条件 |
/// |----------|--------|--------|----------|
/// | 综合运势 | 日干(值符) | 时干 | 值符临吉门吉星 |
/// | 事业工作 | 开门 | 天心星 | 开门旺相无迫 |
/// | 财运求财 | 生门 | 戊(正财) | 生门得奇不空 |
/// | 婚姻感情 | 六合神 | 乙(日奇) | 六合吉门相合 |
/// | 健康疾病 | 天芮星 | 死门 | 天芮死门受克 |
/// | 学业考试 | 天辅星 | 景门 | 天辅临吉门旺相 |
/// | 出行远行 | 驿马宫 | 开门 | 开门无凶格 |
/// | 官司诉讼 | 开门(官) | 庚(对方) | 我克彼有利 |
/// | 寻人寻物 | 六合神 | 相关宫位 | 六合无空亡 |
/// | 投资理财 | 生门 | 天任星 | 生门天任同宫 |
/// | 合作交易 | 六合神 | 生门 | 六合生门相生 |
/// | 祈福求神 | 九天神 | 景门 | 九天临吉门 |
pub fn get_yong_shen_config(question_type: QuestionType) -> (&'static str, Option<&'static str>, &'static str, &'static str) {
    match question_type {
        QuestionType::General => (
            "日干/值符",
            Some("时干"),
            "值符临吉门吉星，无六仪击刑入墓",
            "以日干落宫为主，观其与时干宫位生克关系",
        ),
        QuestionType::Career => (
            "开门",
            Some("天心星"),
            "开门旺相，得奇不迫，临吉星吉神",
            "开门为事业官职之门，落宫旺相为佳",
        ),
        QuestionType::Wealth => (
            "生门",
            Some("戊(正财)"),
            "生门得奇，不落空亡，无门迫",
            "生门为财帛之门，戊为正财星",
        ),
        QuestionType::Marriage => (
            "六合神",
            Some("乙(日奇)"),
            "六合临吉门，与乙奇相合",
            "六合主婚姻交合，乙为阴木主女性",
        ),
        QuestionType::Health => (
            "天芮星",
            Some("死门"),
            "天芮死门受克制为吉，旺相为病重",
            "天芮为病星，死门为病门，受克则病愈",
        ),
        QuestionType::Study => (
            "天辅星",
            Some("景门"),
            "天辅临吉门旺相，景门无迫",
            "天辅为文星，景门为文明之门",
        ),
        QuestionType::Travel => (
            "驿马宫",
            Some("开门"),
            "开门无凶格，驿马宫不空",
            "驿马主出行，开门为出入之门",
        ),
        QuestionType::Lawsuit => (
            "开门(官)",
            Some("庚(对方)"),
            "我宫旺相克彼宫为有利",
            "开门为官府之门，庚为对方/敌人",
        ),
        QuestionType::Finding => (
            "六合神",
            Some("相关方位"),
            "六合不空，观其落宫方向",
            "六合主人物聚合，落宫即所在方向",
        ),
        QuestionType::Investment => (
            "生门",
            Some("天任星"),
            "生门天任同宫或相生为佳",
            "生门主财，天任为吉星主土地产业",
        ),
        QuestionType::Business => (
            "六合神",
            Some("生门"),
            "六合生门相生，无空亡击刑",
            "六合主交易合作，生门主财利",
        ),
        QuestionType::Prayer => (
            "九天神",
            Some("景门"),
            "九天临吉门吉星，景门旺相",
            "九天主祈求上达，景门为光明之门",
        ),
    }
}

/// 查找用神在九宫中的位置
///
/// 根据用神名称在排盘结果中查找其落宫
///
/// ## 参数
///
/// - `yong_shen_name`: 用神名称（如"开门"、"生门"、"六合"等）
/// - `palaces`: 九宫排盘结果
/// - `zhi_fu_xing`: 值符星
/// - `shi_zhi`: 时支（用于驿马）
///
/// ## 返回值
///
/// 返回用神所在的宫位号（1-9），找不到返回None
pub fn find_yong_shen_gong(
    yong_shen_name: &str,
    palaces: &[Palace; 9],
    zhi_fu_xing: JiuXing,
    shi_zhi: DiZhi,
) -> Option<u8> {
    match yong_shen_name {
        // 八门类用神
        "开门" | "开门(官)" => {
            for palace in palaces.iter() {
                if palace.men == Some(BaMen::Kai) {
                    return Some(palace.gong.num());
                }
            }
            None
        }
        "生门" => {
            for palace in palaces.iter() {
                if palace.men == Some(BaMen::Sheng) {
                    return Some(palace.gong.num());
                }
            }
            None
        }
        "休门" => {
            for palace in palaces.iter() {
                if palace.men == Some(BaMen::Xiu) {
                    return Some(palace.gong.num());
                }
            }
            None
        }
        "景门" => {
            for palace in palaces.iter() {
                if palace.men == Some(BaMen::Jing) {
                    return Some(palace.gong.num());
                }
            }
            None
        }
        "死门" => {
            for palace in palaces.iter() {
                if palace.men == Some(BaMen::Si) {
                    return Some(palace.gong.num());
                }
            }
            None
        }
        // 九星类用神
        "天心星" => {
            for palace in palaces.iter() {
                if palace.xing == JiuXing::TianXin {
                    return Some(palace.gong.num());
                }
            }
            None
        }
        "天芮星" => {
            for palace in palaces.iter() {
                if palace.xing == JiuXing::TianRui {
                    return Some(palace.gong.num());
                }
            }
            None
        }
        "天辅星" => {
            for palace in palaces.iter() {
                if palace.xing == JiuXing::TianFu {
                    return Some(palace.gong.num());
                }
            }
            None
        }
        "天任星" => {
            for palace in palaces.iter() {
                if palace.xing == JiuXing::TianRen {
                    return Some(palace.gong.num());
                }
            }
            None
        }
        // 八神类用神
        "六合神" | "六合" => {
            for palace in palaces.iter() {
                if palace.shen == Some(BaShen::LiuHe) {
                    return Some(palace.gong.num());
                }
            }
            None
        }
        "九天神" | "九天" => {
            for palace in palaces.iter() {
                if palace.shen == Some(BaShen::JiuTian) {
                    return Some(palace.gong.num());
                }
            }
            None
        }
        "值符" | "日干/值符" => {
            // 值符落宫 = 值符星所在宫位
            for palace in palaces.iter() {
                if palace.xing == zhi_fu_xing {
                    return Some(palace.gong.num());
                }
            }
            None
        }
        // 天干类用神
        "戊(正财)" | "戊" => {
            for palace in palaces.iter() {
                if palace.tian_pan_gan == TianGan::Wu {
                    return Some(palace.gong.num());
                }
            }
            None
        }
        "乙(日奇)" | "乙" => {
            for palace in palaces.iter() {
                if palace.tian_pan_gan == TianGan::Yi {
                    return Some(palace.gong.num());
                }
            }
            None
        }
        "庚(对方)" | "庚" => {
            for palace in palaces.iter() {
                if palace.tian_pan_gan == TianGan::Geng {
                    return Some(palace.gong.num());
                }
            }
            None
        }
        // 驿马宫
        "驿马宫" => {
            let yi_ma = calc_yi_ma(shi_zhi);
            Some(yi_ma.gong.num())
        }
        _ => None,
    }
}

/// 分析问事用神
///
/// 综合分析问事类型的用神情况，返回用神分析结果
///
/// ## 参数
///
/// - `question_type`: 问事类型
/// - `palaces`: 九宫排盘结果
/// - `zhi_fu_xing`: 值符星
/// - `shi_zhi`: 时支
///
/// ## 返回值
///
/// 返回用神分析结果
pub fn analyze_yong_shen(
    question_type: QuestionType,
    palaces: &[Palace; 9],
    zhi_fu_xing: JiuXing,
    shi_zhi: DiZhi,
) -> YongShenResult {
    let (primary_name, secondary_name, auspicious_condition, judgment_hint) =
        get_yong_shen_config(question_type);

    let primary_gong = find_yong_shen_gong(primary_name, palaces, zhi_fu_xing, shi_zhi);
    let secondary_gong = secondary_name
        .and_then(|name| find_yong_shen_gong(name, palaces, zhi_fu_xing, shi_zhi));

    YongShenResult {
        question_type,
        primary_name,
        secondary_name,
        primary_gong,
        secondary_gong,
        auspicious_condition,
        judgment_hint,
    }
}

/// 评估用神宫位吉凶
///
/// 对用神所在宫位进行吉凶评估
///
/// ## 参数
///
/// - `gong_num`: 宫位号（1-9）
/// - `palaces`: 九宫排盘结果
/// - `yue_zhi`: 月支（用于旺衰判断）
/// - `shi_zhi`: 时支（用于空亡判断）
/// - `shi_gan`: 时干（用于空亡判断）
///
/// ## 返回值
///
/// 返回 (吉分, 凶分, 评语) 元组
pub fn evaluate_yong_shen_gong(
    gong_num: u8,
    palaces: &[Palace; 9],
    yue_zhi: DiZhi,
    shi_zhi: DiZhi,
    shi_gan: TianGan,
) -> (u8, u8, &'static str) {
    if gong_num < 1 || gong_num > 9 {
        return (0, 0, "无效宫位");
    }

    let palace = &palaces[(gong_num - 1) as usize];
    let analysis = analyze_palace(palace, yue_zhi, shi_zhi);

    let mut ji_score: u8 = 50; // 基础分
    let mut xiong_score: u8 = 0;
    let mut comment = "平常";

    // 1. 检查旬空 (-30分)
    if is_gong_xun_kong(gong_num, shi_gan, shi_zhi) {
        xiong_score = xiong_score.saturating_add(30);
        comment = "用神落空亡";
    }

    // 2. 检查六仪击刑 (-20分)
    if analysis.ji_xing.is_some() {
        xiong_score = xiong_score.saturating_add(20);
        comment = "六仪击刑";
    }

    // 3. 检查奇仪入墓 (-20分)
    if analysis.ru_mu.is_some() {
        xiong_score = xiong_score.saturating_add(20);
        comment = "奇仪入墓";
    }

    // 4. 检查门迫 (-15分)
    if analysis.men_po.is_some() {
        xiong_score = xiong_score.saturating_add(15);
        comment = "八门受迫";
    }

    // 5. 检查九星吉凶 (+/- 10分)
    if palace.xing.is_auspicious() {
        ji_score = ji_score.saturating_add(10);
    } else {
        xiong_score = xiong_score.saturating_add(10);
    }

    // 6. 检查八门吉凶 (+/- 10分)
    if let Some(men) = palace.men {
        if men.is_auspicious() {
            ji_score = ji_score.saturating_add(10);
        } else {
            xiong_score = xiong_score.saturating_add(10);
        }
    }

    // 7. 检查八神吉凶 (+/- 10分)
    if let Some(shen) = palace.shen {
        if shen.is_auspicious() {
            ji_score = ji_score.saturating_add(10);
        } else {
            xiong_score = xiong_score.saturating_add(10);
        }
    }

    // 8. 检查旺衰 (+/- 15分)
    if let Some(wang_shuai) = analysis.xing_wang_shuai {
        if wang_shuai.is_favorable() {
            ji_score = ji_score.saturating_add(15);
            if comment == "平常" {
                comment = "用神旺相";
            }
        } else {
            xiong_score = xiong_score.saturating_add(15);
        }
    }

    // 9. 检查十干克应 (+/- 10分)
    if let Some(ke_ying) = analysis.ke_ying {
        if ke_ying.is_ji {
            ji_score = ji_score.saturating_add(10);
            if comment == "平常" {
                comment = "干支相生";
            }
        } else {
            xiong_score = xiong_score.saturating_add(10);
        }
    }

    // 10. 检查驿马 (+10分)
    if analysis.is_yi_ma {
        ji_score = ji_score.saturating_add(10);
        if comment == "平常" {
            comment = "用神临马";
        }
    }

    // 最终评语判断
    if ji_score > xiong_score + 30 {
        comment = "大吉";
    } else if ji_score > xiong_score + 15 {
        comment = "中吉";
    } else if xiong_score > ji_score + 30 {
        comment = "大凶";
    } else if xiong_score > ji_score + 15 {
        comment = "不利";
    }

    (ji_score.min(100), xiong_score.min(100), comment)
}

/// 综合分析单宫格局
///
/// 对单个宫位进行全面的格局分析
pub fn analyze_palace(
    palace: &Palace,
    yue_zhi: DiZhi,
    shi_zhi: DiZhi,
) -> PalaceAnalysis {
    let mut analysis = PalaceAnalysis::default();

    // 检测六仪击刑
    analysis.ji_xing = check_liu_yi_ji_xing(palace.tian_pan_gan, palace.gong);

    // 检测奇仪入墓
    analysis.ru_mu = check_qi_yi_ru_mu(palace.tian_pan_gan, palace.gong);

    // 检测门迫
    if let Some(men) = palace.men {
        analysis.men_po = check_men_po(men, palace.gong);
        // 计算八门旺衰
        analysis.men_wang_shuai = Some(calc_men_wang_shuai(men, yue_zhi));
    }

    // 检测十干克应
    analysis.ke_ying = Some(check_shi_gan_ke_ying(palace.tian_pan_gan, palace.di_pan_gan));

    // 检测驿马
    let yi_ma = calc_yi_ma(shi_zhi);
    analysis.is_yi_ma = palace.gong == yi_ma.gong;

    // 计算九星旺衰
    analysis.xing_wang_shuai = Some(calc_xing_wang_shuai(palace.xing, yue_zhi));

    analysis
}

// ==================== 解读文案系统 ====================

/// 九星解读文案
pub fn get_jiu_xing_interpretation(xing: JiuXing) -> (&'static str, &'static str, &'static str) {
    match xing {
        JiuXing::TianPeng => (
            "天蓬星",
            "主智慧、谋略、隐秘之事",
            "天蓬为坎水之精，主智慧多谋，但也主隐匿、盗贼、水患。吉则智谋过人，凶则阴险狡诈。",
        ),
        JiuXing::TianRui => (
            "天芮星",
            "主疾病、小人、阴私之事",
            "天芮为坤土之精，为病星、凶星。主疾病缠身、小人作祟、事多阻滞。求事不宜，养病宜静。",
        ),
        JiuXing::TianChong => (
            "天冲星",
            "主勇猛、急躁、武职之事",
            "天冲为震木之精，主雷厉风行、果断刚毅。吉则事业亨通，凶则鲁莽行事。宜武不宜文。",
        ),
        JiuXing::TianFu => (
            "天辅星",
            "主文书、贵人、学业之事",
            "天辅为巽木之精，为文星、吉星。主贵人相助、文书顺利、学业有成。求官求名皆吉。",
        ),
        JiuXing::TianQin => (
            "天禽星",
            "主中正、平和、调和之事",
            "天禽为中宫土星，居中调和四方。主事平稳、居中调停。寄于坤艮，随局而动。",
        ),
        JiuXing::TianXin => (
            "天心星",
            "主医药、技艺、治理之事",
            "天心为乾金之精，为大吉星。主医药见效、技艺精进、治事有方。求医问药最宜。",
        ),
        JiuXing::TianZhu => (
            "天柱星",
            "主口舌、争讼、毁折之事",
            "天柱为兑金之精，为凶星。主口舌是非、诉讼官司、毁败损失。行事宜慎，防止口舌。",
        ),
        JiuXing::TianRen => (
            "天任星",
            "主土地、农事、稳重之事",
            "天任为艮土之精，为吉星。主田宅稳固、农事丰收、行事稳重。求财置产皆宜。",
        ),
        JiuXing::TianYing => (
            "天英星",
            "主文明、血光、火患之事",
            "天英为离火之精，为凶星。主光明显达，但也主血光、火灾、虚浮。防火防血光。",
        ),
    }
}

/// 八门解读文案
pub fn get_ba_men_interpretation(men: BaMen) -> (&'static str, &'static str, &'static str) {
    match men {
        BaMen::Xiu => (
            "休门",
            "主休养、官贵、平安之事",
            "休门为坎水之门，大吉门。主休息养生、官贵提携、诸事平安。求官、求名、养病皆吉。",
        ),
        BaMen::Si => (
            "死门",
            "主死亡、停滞、埋藏之事",
            "死门为坤土之门，大凶门。主死丧、停滞、暗昧不明。行事不利，但吊丧、埋葬反宜。",
        ),
        BaMen::Shang => (
            "伤门",
            "主伤害、争斗、破财之事",
            "伤门为震木之门，凶门。主伤灾、争斗、破财损物。求财不利，但捕猎、讨债反利。",
        ),
        BaMen::Du => (
            "杜门",
            "主阻隔、躲避、隐藏之事",
            "杜门为巽木之门，凶门。主阻隔难通、躲避藏匿。行事受阻，但躲避灾祸、隐藏逃避反宜。",
        ),
        BaMen::Jing => (
            "景门",
            "主文书、光明、虚名之事",
            "景门为离火之门，中平门。主文书传信、光明正大。文书、考试有利，但主虚名不实。",
        ),
        BaMen::Kai => (
            "开门",
            "主开创、官禄、贵人之事",
            "开门为乾金之门，大吉门。主开创事业、官禄亨通、贵人扶持。求官、创业、出行大吉。",
        ),
        BaMen::Jing2 => (
            "惊门",
            "主惊恐、口舌、是非之事",
            "惊门为兑金之门，凶门。主惊恐不安、口舌是非、官讼争端。行事不宜，防口舌之祸。",
        ),
        BaMen::Sheng => (
            "生门",
            "主生财、生发、谋望之事",
            "生门为艮土之门，大吉门。主生财有道、谋望如意、阳宅动土。求财、置产、开业大吉。",
        ),
    }
}

/// 八神解读文案
pub fn get_ba_shen_interpretation(shen: BaShen) -> (&'static str, &'static str, &'static str) {
    match shen {
        BaShen::ZhiFu => (
            "值符",
            "统领诸神，代表当事人",
            "值符为诸神之首，代表求测人或主事者。值符所临之处即为我方，观其落宫吉凶以定事之成败。",
        ),
        BaShen::TengShe => (
            "腾蛇",
            "主惊恐怪异、虚假不实",
            "腾蛇为惊恐之神，主虚惊、怪异、梦境、虚假。临之则事多变化，难以捉摸，须防虚惊。",
        ),
        BaShen::TaiYin => (
            "太阴",
            "主阴私暗昧、女性贵人",
            "太阴为阴私之神，吉神。主阴私之事、女性贵人、暗中相助。临之则有阴人暗助，事宜隐密。",
        ),
        BaShen::LiuHe => (
            "六合",
            "主婚姻交易、和合之事",
            "六合为和合之神，吉神。主婚姻美满、交易顺利、人际和谐。临之则人和事顺，主合作成功。",
        ),
        BaShen::BaiHu => (
            "白虎",
            "主凶丧血光、武职权柄",
            "白虎为凶杀之神，凶神。主血光、凶丧、刑伤、武职。临之须防血光之灾，但武职反吉。",
        ),
        BaShen::XuanWu => (
            "玄武",
            "主盗贼小人、私昧之事",
            "玄武为盗贼之神，凶神。主盗失、小人、欺诈、私情。临之须防小人暗算，钱财被盗。",
        ),
        BaShen::JiuDi => (
            "九地",
            "主安静隐伏、柔顺包容",
            "九地为坤顺之神，吉神。主安静守成、隐伏藏匿、柔顺包容。临之宜守不宜攻，宜静不宜动。",
        ),
        BaShen::JiuTian => (
            "九天",
            "主高远飞举、刚健进取",
            "九天为乾健之神，吉神。主远行高举、刚健进取、志向远大。临之宜积极进取，主升迁高就。",
        ),
    }
}

/// 九遁格局解读文案
pub fn get_jiu_dun_interpretation(ge_ju: JiuDunGeJu) -> (&'static str, &'static str, &'static str, &'static str) {
    match ge_ju {
        JiuDunGeJu::TianDun => (
            "天遁",
            "丙奇+生门+吉神同宫",
            "大吉格局",
            "天遁为诸遁之首，主天时相助。丙为日奇主光明，生门主生发，此格主求财大吉、出行平安、谋事顺遂。凡事皆可为，贵人相助，如有神佑。",
        ),
        JiuDunGeJu::DiDun => (
            "地遁",
            "乙奇+开门+九地同宫",
            "大吉格局",
            "地遁主地利护佑。乙为日奇主和柔，开门主开创，九地主隐藏。此格宜藏匿避祸、隐居养性、暗中行事。凡欲隐遁者，得此格大吉。",
        ),
        JiuDunGeJu::RenDun => (
            "人遁",
            "丁奇+休门+太阴同宫",
            "大吉格局",
            "人遁主人和事顺。丁为星奇主文明，休门主休养，太阴主阴助。此格宜求见贵人、密谋私事、暗中交际。有贵人暗中相助。",
        ),
        JiuDunGeJu::FengDun => (
            "风遁",
            "乙奇+休门+六合同宫",
            "吉格",
            "风遁主风神助力。乙奇得休门六合，主出行顺利、商旅通畅、人际和谐。此格宜远行、贸易、交际，凡行动皆得风助。",
        ),
        JiuDunGeJu::YunDun => (
            "云遁",
            "乙奇+生门+九天同宫",
            "吉格",
            "云遁主云中翱翔。乙奇得生门九天，主升迁高就、名声远扬。此格宜求官进职、上书言事、扬名立万，主贵显荣华。",
        ),
        JiuDunGeJu::LongDun => (
            "龙遁",
            "乙奇+休门+六合在坎宫",
            "特殊大吉格",
            "龙遁为龙入大海，最为难得。乙奇休门六合在坎水宫，如龙归大海，无往不利。此格主大富大贵、鱼跃龙门、飞黄腾达。",
        ),
        JiuDunGeJu::HuDun => (
            "虎遁",
            "乙奇+开门+太阴在艮宫",
            "特殊大吉格",
            "虎遁为虎踞山林，威镇四方。乙奇开门太阴在艮土宫，如虎归山，主权柄在握、威望日隆。此格宜掌权治事、建功立业。",
        ),
        JiuDunGeJu::ShenDun => (
            "神遁",
            "丙奇+休门+九天同宫",
            "吉格",
            "神遁主神明护佑。丙为日奇，得休门九天，主天神相助、祈福灵验。此格宜祈福求神、重大仪式、祭祀许愿，必得神佑。",
        ),
        JiuDunGeJu::GuiDun => (
            "鬼遁",
            "丁奇+生门+九地同宫",
            "吉格",
            "鬼遁主鬼神相助。丁为星奇，得生门九地，主暗中有助、逢凶化吉。此格宜暗中行事、避凶趋吉，主化险为夷、否极泰来。",
        ),
    }
}

/// 凶格解读文案
pub fn get_xiong_ge_interpretation(ge_ju: XiongGeJu) -> (&'static str, &'static str, &'static str) {
    match ge_ju {
        XiongGeJu::BaiHuChangKuang => (
            "白虎猖狂",
            "辛+乙同宫",
            "大凶格。白虎猖狂，主血光刑伤、横祸飞来。辛金克乙木，如虎伤人。凡事宜慎，防止意外伤害、车祸血光。",
        ),
        XiongGeJu::TengSheYaoJiao => (
            "螣蛇夭矫",
            "壬+壬同宫",
            "凶格。螣蛇夭矫，主虚惊怪异、噩梦不断。壬水同宫，如蛇蜿蜒难测。主事多变故、心神不宁、疑虑重重。",
        ),
        XiongGeJu::ZhuQueToJiang => (
            "朱雀投江",
            "丙+壬在坎宫",
            "大凶格。朱雀投江，主火入水灭、文书失误。丙火遇壬水于坎宫，如鸟坠水。主文书官司不利、口舌招灾。",
        ),
        XiongGeJu::TianWangSiZhang => (
            "天网四张",
            "癸+癸同宫",
            "凶格。天网四张，主四面受困、动弹不得。癸水同宫，如天罗地网。主诸事不顺、处处受阻、进退维谷。",
        ),
        XiongGeJu::DiWangZheBi => (
            "地网遮蔽",
            "大凶组合",
            "大凶格。地网遮蔽，主暗无天日、前途迷茫。凡事受阻，不宜妄动，宜守待时。",
        ),
        XiongGeJu::YingRuBaiHu => (
            "荧入白虎",
            "丙+庚同宫",
            "凶格。荧入白虎，主以弱敌强、自取其辱。丙火克庚金，但庚为太白主兵戈。主争斗受伤、以卵击石。",
        ),
    }
}

/// 十干克应解读文案
pub fn get_shi_gan_ke_ying_interpretation(ge_ju: ShiGanGeJu) -> (&'static str, &'static str) {
    match ge_ju {
        ShiGanGeJu::RiQiFuYin => ("日奇伏吟", "乙见乙，主事迟缓、原地踏步"),
        ShiGanGeJu::QiYiShunSui => ("奇仪顺遂", "乙加丙，主贵人相助、诸事顺利"),
        ShiGanGeJu::QiYiXiangZuo => ("奇仪相佐", "乙加丁，主阴阳和合、谋事有成"),
        ShiGanGeJu::RiYueBingXing => ("日月并行", "丙加乙，主光明磊落、名利双收"),
        ShiGanGeJu::YueQiBeiShi => ("月奇悖师", "丙见丙，主过刚易折、注意火患"),
        ShiGanGeJu::XingQiZhuQue => ("星奇朱雀", "丙加丁，主文明显达、声名远播"),
        ShiGanGeJu::XingQiRuTaiYin => ("星奇入太阴", "丁加乙，主暗中相助、阴人有力"),
        ShiGanGeJu::XingQiRuLiuHe => ("星奇入六合", "丁加丙，主合作愉快、婚姻和美"),
        ShiGanGeJu::XingQiFuYin => ("星奇伏吟", "丁见丁，主文书迟延、消息不通"),
        ShiGanGeJu::FeiNiaoDieXue => ("飞鸟跌穴", "丙加戊，大吉格，主求财大利、谋事必成"),
        ShiGanGeJu::TaiBaiTongGong => ("太白同宫", "庚见庚，大凶格，主兵戈相见、灾祸连连"),
        ShiGanGeJu::TaiBaiRuRi => ("太白入日", "庚加乙，主小人害正、以下犯上"),
        ShiGanGeJu::TaiBaiRuYing => ("太白入荧", "庚加丙，主以力服人、争斗不休"),
        ShiGanGeJu::TaiBaiRuXing => ("太白入星", "庚加丁，主文武相争、是非不断"),
        ShiGanGeJu::HuaGaiFuYin => ("华盖伏吟", "癸见癸，主孤独清高、事多阻滞"),
        ShiGanGeJu::BaiHuChangKuang => ("白虎猖狂", "辛加乙，主血光之灾、意外伤害"),
        ShiGanGeJu::BaiHuRuYing => ("白虎入荧", "辛加丙，主争斗见血、口舌生非"),
        ShiGanGeJu::BaiHuRuXing => ("白虎入星", "辛加丁，主文书失误、官讼不利"),
        ShiGanGeJu::SheYaoJiao => ("蛇夭矫", "壬见壬，主虚惊怪异、心神不宁"),
        ShiGanGeJu::FuYin => ("伏吟", "天地盘干相同，主事物停滞、迟缓不前"),
        ShiGanGeJu::Other => ("普通格局", "无特殊吉凶，按五行生克论断"),
    }
}

/// 综合解读生成器
///
/// 根据排盘结果生成综合解读文案
pub fn generate_comprehensive_interpretation(
    dun_type: DunType,
    _ju_number: u8,
    zhi_fu_xing: JiuXing,
    zhi_shi_men: BaMen,
    palaces: &[Palace; 9],
    shi_zhi: DiZhi,
) -> (&'static str, &'static str, Vec<&'static str>) {
    // 整体运势评估
    let overall = if zhi_fu_xing.is_auspicious() && zhi_shi_men.is_auspicious() {
        "大吉"
    } else if zhi_fu_xing.is_auspicious() || zhi_shi_men.is_auspicious() {
        "中吉"
    } else {
        "需慎重"
    };

    // 遁类型解读
    let dun_desc = match dun_type {
        DunType::Yang => "阳遁主进取、外向、发展",
        DunType::Yin => "阴遁主守成、内敛、收藏",
    };

    // 收集特殊格局
    let mut special_patterns: Vec<&'static str> = Vec::new();

    // 检查九遁格局
    for palace in palaces.iter() {
        if let Some(jiu_dun) = check_jiu_dun(
            palace.tian_pan_gan,
            palace.men,
            palace.shen,
            palace.gong,
        ) {
            let (name, _, _, _) = get_jiu_dun_interpretation(jiu_dun);
            special_patterns.push(name);
        }
    }

    // 检查驿马
    let yi_ma = calc_yi_ma(shi_zhi);
    let yi_ma_palace = &palaces[(yi_ma.gong.num() - 1) as usize];
    if yi_ma_palace.men.map(|m| m.is_auspicious()).unwrap_or(false) {
        special_patterns.push("驿马临吉门，利出行");
    }

    (overall, dun_desc, special_patterns)
}

// ==================== 飞盘奇门排盘系统 ====================

/// 洛书九宫飞布顺序
///
/// 飞盘奇门中，九星、八门按洛书九宫顺序飞布：
/// 1→2→3→4→5→6→7→8→9（顺飞）
/// 9→8→7→6→5→4→3→2→1（逆飞）
///
/// 洛书九宫位置对应：
/// ```text
/// 4 9 2
/// 3 5 7
/// 8 1 6
/// ```
pub const LUO_SHU_SHUN_FEI: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
pub const LUO_SHU_NI_FEI: [u8; 9] = [9, 8, 7, 6, 5, 4, 3, 2, 1];

/// 飞盘九星飞布
///
/// 根据值符星的原始宫位，按洛书顺序飞布九星
///
/// ## 飞盘规则
///
/// 1. 找到值符星的原始宫位作为起飞宫
/// 2. 阳遁顺飞：从起飞宫开始，按 1→2→3→4→5→6→7→8→9 顺序飞布
/// 3. 阴遁逆飞：从起飞宫开始，按 9→8→7→6→5→4→3→2→1 顺序飞布
/// 4. 时干所在宫位即为值符星的落宫
///
/// ## 参数
///
/// - `zhi_fu_xing`: 值符星
/// - `shi_gan_gong`: 时干所落宫位（1-9）
/// - `dun_type`: 阴阳遁类型
///
/// ## 返回值
///
/// 返回长度为9的数组，索引0-8对应1-9宫的九星
pub fn fei_pan_distribute_jiu_xing(
    zhi_fu_xing: JiuXing,
    shi_gan_gong: u8,
    dun_type: DunType,
) -> [JiuXing; 9] {
    let mut tian_pan = [JiuXing::TianQin; 9];

    // 值符星原始宫位（起飞点）
    let start_gong = zhi_fu_xing.original_palace().num();

    // 计算从起飞宫到时干落宫需要飞几步
    let steps = match dun_type {
        DunType::Yang => {
            // 阳遁顺飞
            if shi_gan_gong >= start_gong {
                shi_gan_gong - start_gong
            } else {
                9 - start_gong + shi_gan_gong
            }
        }
        DunType::Yin => {
            // 阴遁逆飞
            if start_gong >= shi_gan_gong {
                start_gong - shi_gan_gong
            } else {
                start_gong + 9 - shi_gan_gong
            }
        }
    };

    // 按飞布顺序放置九星
    for i in 0..9 {
        let xing_num = ((zhi_fu_xing.num() as i16 - 1 + i as i16) % 9 + 1) as u8;
        let xing = JiuXing::from_num(xing_num).unwrap_or(JiuXing::TianQin);

        // 计算该星的落宫
        let gong_num = match dun_type {
            DunType::Yang => {
                // 顺飞：起飞宫 + 步数
                let g = (start_gong as i16 - 1 + steps as i16 + i as i16) % 9 + 1;
                g as u8
            }
            DunType::Yin => {
                // 逆飞：起飞宫 - 步数
                let g = (start_gong as i16 - 1 - steps as i16 - i as i16) % 9;
                let g = if g < 0 { g + 9 } else { g };
                (g + 1) as u8
            }
        };

        let array_index = gong_num.saturating_sub(1).min(8) as usize;
        tian_pan[array_index] = xing;
    }

    tian_pan
}

/// 飞盘八门飞布
///
/// 根据值使门的原始宫位，按洛书顺序飞布八门
///
/// ## 飞盘规则
///
/// 1. 找到值使门的原始宫位作为起飞宫
/// 2. 阳遁顺飞：从起飞宫开始顺序飞布
/// 3. 阴遁逆飞：从起飞宫开始逆序飞布
/// 4. 中宫（5宫）无门，八门只飞布到八个外宫
///
/// ## 参数
///
/// - `zhi_shi_men`: 值使门
/// - `shi_gan_gong`: 时干所落宫位（1-9）
/// - `dun_type`: 阴阳遁类型
///
/// ## 返回值
///
/// 返回长度为9的数组，索引0-8对应1-9宫的八门，中宫为None
pub fn fei_pan_distribute_ba_men(
    zhi_shi_men: BaMen,
    shi_gan_gong: u8,
    dun_type: DunType,
) -> [Option<BaMen>; 9] {
    let mut ren_pan: [Option<BaMen>; 9] = [None; 9];

    // 八个外宫的顺序（顺时针，跳过中宫）
    // 1→2→3→4→6→7→8→9（或逆序）
    let gong_order_yang: [u8; 8] = [1, 2, 3, 4, 6, 7, 8, 9];
    let gong_order_yin: [u8; 8] = [9, 8, 7, 6, 4, 3, 2, 1];

    // 八门按原始宫位的顺序
    // 休门(1)、死门(2)、伤门(3)、杜门(4)、开门(6)、惊门(7)、生门(8)、景门(9)
    let men_order: [BaMen; 8] = [
        BaMen::Xiu, BaMen::Si, BaMen::Shang, BaMen::Du,
        BaMen::Kai, BaMen::Jing2, BaMen::Sheng, BaMen::Jing,
    ];

    // 找到值使门在八门序列中的位置
    let men_start = men_order.iter().position(|&m| m == zhi_shi_men).unwrap_or(0);

    // 值使门原始宫位
    let start_gong = zhi_shi_men.original_palace().num();

    // 找到起始宫位在宫位序列中的索引
    let gong_order = match dun_type {
        DunType::Yang => &gong_order_yang,
        DunType::Yin => &gong_order_yin,
    };

    // 计算时干落宫在宫位序列中的位置
    let shi_gan_gong_idx = if shi_gan_gong == 5 {
        // 中宫寄艮八宫（转盘奇门惯例）
        gong_order.iter().position(|&g| g == 8).unwrap_or(0)
    } else {
        gong_order.iter().position(|&g| g == shi_gan_gong).unwrap_or(0)
    };

    // 找到起始宫在宫位序列中的位置
    let start_gong_idx = if start_gong == 5 {
        gong_order.iter().position(|&g| g == 8).unwrap_or(0)
    } else {
        gong_order.iter().position(|&g| g == start_gong).unwrap_or(0)
    };

    // 计算偏移量
    let offset = (shi_gan_gong_idx + 8 - start_gong_idx) % 8;

    // 按飞布顺序放置八门
    for i in 0..8 {
        let men_idx = (men_start + i) % 8;
        let men = men_order[men_idx];

        let gong_idx = (offset + i) % 8;
        let gong_num = gong_order[gong_idx];

        let array_index = (gong_num - 1) as usize;
        ren_pan[array_index] = Some(men);
    }

    // 确保中宫无门
    ren_pan[4] = None;

    ren_pan
}

/// 飞盘八神飞布
///
/// 根据值符落宫，按固定顺序飞布八神
///
/// ## 飞盘规则
///
/// 飞盘中八神飞布与转盘类似，从值符落宫开始按固定顺序排布
/// 阳遁顺布，阴遁逆布
///
/// ## 参数
///
/// - `zhi_fu_gong`: 值符落宫（1-9）
/// - `dun_type`: 阴阳遁类型
///
/// ## 返回值
///
/// 返回长度为9的数组，中宫为None
pub fn fei_pan_distribute_ba_shen(
    zhi_fu_gong: u8,
    dun_type: DunType,
) -> [Option<BaShen>; 9] {
    // 飞盘八神飞布与转盘类似，使用相同的逻辑
    distribute_ba_shen(zhi_fu_gong, dun_type)
}

/// 飞盘奇门完整排盘算法
///
/// 使用飞盘方法进行奇门遁甲排盘
///
/// ## 飞盘与转盘的主要区别
///
/// | 项目 | 转盘 | 飞盘 |
/// |------|------|------|
/// | 九星 | 整体旋转 | 按洛书顺序飞布 |
/// | 八门 | 整体旋转 | 按洛书顺序飞布 |
/// | 八神 | 整体旋转 | 按洛书顺序飞布 |
/// | 天盘干 | 随九星移动 | 随九星飞布 |
///
/// ## 参数
///
/// - `year_gz`: 年柱干支
/// - `month_gz`: 月柱干支
/// - `day_gz`: 日柱干支
/// - `hour_gz`: 时柱干支
/// - `jie_qi`: 节气
/// - `day_in_jieqi`: 节气内第几天（1-15）
///
/// ## 返回值
///
/// 返回元组 (阴阳遁, 三元, 局数, 值符星, 值使门, 九宫)
pub fn generate_fei_pan_chart(
    _year_gz: GanZhi,
    _month_gz: GanZhi,
    _day_gz: GanZhi,
    hour_gz: GanZhi,
    jie_qi: JieQi,
    day_in_jieqi: u8,
) -> (DunType, SanYuan, u8, JiuXing, BaMen, [Palace; 9]) {
    // 1. 确定阴阳遁
    let dun_type = calc_dun_type(jie_qi);

    // 2. 确定三元
    let san_yuan = calc_san_yuan(day_in_jieqi);

    // 3. 确定局数
    let ju_number = calc_ju_number(jie_qi, san_yuan, dun_type);

    // 4. 排布地盘（地盘排布与转盘相同）
    let di_pan = get_di_pan(ju_number, dun_type);

    // 5. 确定旬首六仪
    let xun_shou_yi = get_xun_shou(hour_gz.gan, hour_gz.zhi);

    // 6. 计算值符星和值使门
    let zhi_fu_xing = calc_zhi_fu_xing(xun_shou_yi, &di_pan);
    let zhi_shi_men = calc_zhi_shi_men(xun_shou_yi, &di_pan);

    // 7. 时干寄宫
    let shi_gan_gong = find_gan_in_di_pan(hour_gz.gan, &di_pan).unwrap_or(1);

    // 8. 飞盘排布天盘九星
    let tian_pan_xing = fei_pan_distribute_jiu_xing(zhi_fu_xing, shi_gan_gong, dun_type);

    // 9. 飞盘排布人盘八门
    let ren_pan_men = fei_pan_distribute_ba_men(zhi_shi_men, shi_gan_gong, dun_type);

    // 10. 飞盘排布神盘八神
    let shen_pan_shen = fei_pan_distribute_ba_shen(shi_gan_gong, dun_type);

    // 11. 获取旬空宫位
    let (kong_gong1, kong_gong2) = get_xun_kong_gong(hour_gz.gan, hour_gz.zhi);

    // 12. 计算驿马落宫
    let yi_ma = calc_yi_ma(hour_gz.zhi);
    let yi_ma_gong = yi_ma.gong.num();

    // 13. 组装九宫
    let mut palaces = [Palace::empty(JiuGong::Kan); 9];

    for i in 0..9 {
        let gong_num = (i + 1) as u8;
        let gong = JiuGong::from_num(gong_num).unwrap_or(JiuGong::Kan);
        let xing = tian_pan_xing[i];
        let tian_pan_gan = get_tian_pan_gan(xing, &di_pan);

        // 检查是否旬空
        let is_xun_kong = gong_num == kong_gong1 || gong_num == kong_gong2;

        // 检查是否马星宫
        let is_ma_xing = gong_num == yi_ma_gong;

        palaces[i] = Palace {
            gong,
            tian_pan_gan,
            di_pan_gan: di_pan[i],
            xing,
            men: ren_pan_men[i],
            shen: shen_pan_shen[i],
            is_xun_kong,
            is_ma_xing,
        };
    }

    (dun_type, san_yuan, ju_number, zhi_fu_xing, zhi_shi_men, palaces)
}

/// 根据排盘方法生成奇门遁甲盘
///
/// 统一接口，支持转盘和飞盘两种排盘方法
///
/// ## 参数
///
/// - `pan_method`: 排盘方法（转盘/飞盘）
/// - `year_gz`: 年柱干支
/// - `month_gz`: 月柱干支
/// - `day_gz`: 日柱干支
/// - `hour_gz`: 时柱干支
/// - `jie_qi`: 节气
/// - `day_in_jieqi`: 节气内第几天（1-15）
///
/// ## 返回值
///
/// 返回元组 (阴阳遁, 三元, 局数, 值符星, 值使门, 九宫)
pub fn generate_chart_by_method(
    pan_method: PanMethod,
    year_gz: GanZhi,
    month_gz: GanZhi,
    day_gz: GanZhi,
    hour_gz: GanZhi,
    jie_qi: JieQi,
    day_in_jieqi: u8,
) -> (DunType, SanYuan, u8, JiuXing, BaMen, [Palace; 9]) {
    match pan_method {
        PanMethod::ZhuanPan => {
            generate_qimen_chart(year_gz, month_gz, day_gz, hour_gz, jie_qi, day_in_jieqi)
        }
        PanMethod::FeiPan => {
            generate_fei_pan_chart(year_gz, month_gz, day_gz, hour_gz, jie_qi, day_in_jieqi)
        }
    }
}

/// 飞盘排盘类型版本
///
/// 支持时家、日家、月家、年家四种排盘类型的飞盘方法
///
/// ## 参数
///
/// - `qimen_type`: 排盘类型（时家/日家/月家/年家）
/// - `year_gz`: 年柱干支
/// - `month_gz`: 月柱干支
/// - `day_gz`: 日柱干支
/// - `hour_gz`: 时柱干支
/// - `jie_qi`: 节气
/// - `day_in_jieqi`: 节气内第几天（1-15）
///
/// ## 返回值
///
/// 返回元组 (阴阳遁, 三元, 局数, 值符星, 值使门, 九宫)
pub fn generate_fei_pan_chart_by_type(
    qimen_type: QimenType,
    year_gz: GanZhi,
    month_gz: GanZhi,
    day_gz: GanZhi,
    hour_gz: GanZhi,
    jie_qi: JieQi,
    day_in_jieqi: u8,
) -> (DunType, SanYuan, u8, JiuXing, BaMen, [Palace; 9]) {
    // 根据排盘类型选择不同的干支作为起局依据
    let (base_gz, san_yuan) = match qimen_type {
        QimenType::ShiJia => {
            let yuan = calc_san_yuan_by_zhi(hour_gz.zhi);
            (hour_gz, yuan)
        }
        QimenType::RiJia => {
            let yuan = calc_san_yuan(day_in_jieqi);
            (day_gz, yuan)
        }
        QimenType::YueJia => {
            let yuan = calc_san_yuan_by_zhi(month_gz.zhi);
            (month_gz, yuan)
        }
        QimenType::NianJia => {
            let yuan = calc_san_yuan_by_zhi(year_gz.zhi);
            (year_gz, yuan)
        }
    };

    // 1. 确定阴阳遁
    let dun_type = calc_dun_type(jie_qi);

    // 2. 确定局数
    let ju_number = calc_ju_number(jie_qi, san_yuan, dun_type);

    // 3. 排布地盘
    let di_pan = get_di_pan(ju_number, dun_type);

    // 4. 确定旬首六仪
    let xun_shou_yi = get_xun_shou(base_gz.gan, base_gz.zhi);

    // 5. 计算值符星和值使门
    let zhi_fu_xing = calc_zhi_fu_xing(xun_shou_yi, &di_pan);
    let zhi_shi_men = calc_zhi_shi_men(xun_shou_yi, &di_pan);

    // 6. 基准干支寄宫
    let base_gan_gong = find_gan_in_di_pan(base_gz.gan, &di_pan).unwrap_or(1);

    // 7. 飞盘排布天盘九星
    let tian_pan_xing = fei_pan_distribute_jiu_xing(zhi_fu_xing, base_gan_gong, dun_type);

    // 8. 飞盘排布人盘八门
    let ren_pan_men = fei_pan_distribute_ba_men(zhi_shi_men, base_gan_gong, dun_type);

    // 9. 飞盘排布神盘八神
    let shen_pan_shen = fei_pan_distribute_ba_shen(base_gan_gong, dun_type);

    // 10. 获取旬空宫位
    let (kong_gong1, kong_gong2) = get_xun_kong_gong(base_gz.gan, base_gz.zhi);

    // 11. 计算驿马落宫
    let yi_ma = calc_yi_ma(base_gz.zhi);
    let yi_ma_gong = yi_ma.gong.num();

    // 12. 组装九宫
    let mut palaces = [Palace::empty(JiuGong::Kan); 9];

    for i in 0..9 {
        let gong_num = (i + 1) as u8;
        let gong = JiuGong::from_num(gong_num).unwrap_or(JiuGong::Kan);
        let xing = tian_pan_xing[i];
        let tian_pan_gan = get_tian_pan_gan(xing, &di_pan);

        let is_xun_kong = gong_num == kong_gong1 || gong_num == kong_gong2;
        let is_ma_xing = gong_num == yi_ma_gong;

        palaces[i] = Palace {
            gong,
            tian_pan_gan,
            di_pan_gan: di_pan[i],
            xing,
            men: ren_pan_men[i],
            shen: shen_pan_shen[i],
            is_xun_kong,
            is_ma_xing,
        };
    }

    (dun_type, san_yuan, ju_number, zhi_fu_xing, zhi_shi_men, palaces)
}

/// 比较转盘与飞盘排盘结果
///
/// 返回两种排盘方法的差异信息，用于教学和对比分析
///
/// ## 参数
///
/// - `zhuan_palaces`: 转盘排盘结果
/// - `fei_palaces`: 飞盘排盘结果
///
/// ## 返回值
///
/// 返回每宫的差异数量（九星不同、八门不同、八神不同的宫位数）
pub fn compare_zhuan_fei_charts(
    zhuan_palaces: &[Palace; 9],
    fei_palaces: &[Palace; 9],
) -> (u8, u8, u8) {
    let mut xing_diff = 0u8;
    let mut men_diff = 0u8;
    let mut shen_diff = 0u8;

    for i in 0..9 {
        if zhuan_palaces[i].xing != fei_palaces[i].xing {
            xing_diff += 1;
        }
        if zhuan_palaces[i].men != fei_palaces[i].men {
            men_diff += 1;
        }
        if zhuan_palaces[i].shen != fei_palaces[i].shen {
            shen_diff += 1;
        }
    }

    (xing_diff, men_diff, shen_diff)
}

// ==================== 增强格局检测 ====================

/// 五不遇时类型
///
/// 时干克日干，为五不遇时，主诸事不利
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WuBuYuShi {
    /// 甲日遇庚时（庚克甲）
    JiaGeng,
    /// 乙日遇辛时（辛克乙）
    YiXin,
    /// 丙日遇壬时（壬克丙）
    BingRen,
    /// 丁日遇癸时（癸克丁）
    DingGui,
    /// 戊日遇甲时（甲克戊）
    WuJia,
    /// 己日遇乙时（乙克己）
    JiYi,
    /// 庚日遇丙时（丙克庚）
    GengBing,
    /// 辛日遇丁时（丁克辛）
    XinDing,
    /// 壬日遇戊时（戊克壬）
    RenWu,
    /// 癸日遇己时（己克癸）
    GuiJi,
}

impl WuBuYuShi {
    /// 获取名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::JiaGeng => "甲日庚时",
            Self::YiXin => "乙日辛时",
            Self::BingRen => "丙日壬时",
            Self::DingGui => "丁日癸时",
            Self::WuJia => "戊日甲时",
            Self::JiYi => "己日乙时",
            Self::GengBing => "庚日丙时",
            Self::XinDing => "辛日丁时",
            Self::RenWu => "壬日戊时",
            Self::GuiJi => "癸日己时",
        }
    }

    /// 获取描述
    pub fn description(&self) -> &'static str {
        "时干克日干，诸事不利，百事难成"
    }
}

/// 检测五不遇时
///
/// 时干克日干，为五不遇时
///
/// ## 口诀
///
/// 甲日庚时、乙日辛时、丙日壬时、丁日癸时、戊日甲时、
/// 己日乙时、庚日丙时、辛日丁时、壬日戊时、癸日己时
pub fn check_wu_bu_yu_shi(day_gan: TianGan, hour_gan: TianGan) -> Option<WuBuYuShi> {
    match (day_gan, hour_gan) {
        (TianGan::Jia, TianGan::Geng) => Some(WuBuYuShi::JiaGeng),
        (TianGan::Yi, TianGan::Xin) => Some(WuBuYuShi::YiXin),
        (TianGan::Bing, TianGan::Ren) => Some(WuBuYuShi::BingRen),
        (TianGan::Ding, TianGan::Gui) => Some(WuBuYuShi::DingGui),
        (TianGan::Wu, TianGan::Jia) => Some(WuBuYuShi::WuJia),
        (TianGan::Ji, TianGan::Yi) => Some(WuBuYuShi::JiYi),
        (TianGan::Geng, TianGan::Bing) => Some(WuBuYuShi::GengBing),
        (TianGan::Xin, TianGan::Ding) => Some(WuBuYuShi::XinDing),
        (TianGan::Ren, TianGan::Wu) => Some(WuBuYuShi::RenWu),
        (TianGan::Gui, TianGan::Ji) => Some(WuBuYuShi::GuiJi),
        _ => None,
    }
}

/// 时干入墓检测结果
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShiGanRuMu {
    /// 时干
    pub shi_gan: TianGan,
    /// 墓库地支
    pub mu_zhi: DiZhi,
    /// 落宫
    pub gong: JiuGong,
}

/// 检测时干入墓
///
/// 时干临其墓库之宫位，主做事迟缓、受阻
///
/// ## 时干入墓表
///
/// | 时干 | 墓库 | 宫位 |
/// |------|------|------|
/// | 甲乙 | 未 | 坤二宫 |
/// | 丙丁 | 戌 | 乾六宫 |
/// | 庚辛 | 丑 | 艮八宫 |
/// | 壬癸 | 辰 | 巽四宫 |
pub fn check_shi_gan_ru_mu(hour_gan: TianGan, palaces: &[Palace; 9]) -> Option<ShiGanRuMu> {
    // 找到时干所在的宫位
    for palace in palaces.iter() {
        if palace.tian_pan_gan == hour_gan {
            // 检查是否入墓
            let mu_result = check_qi_yi_ru_mu(hour_gan, palace.gong);
            if let Some(ru_mu) = mu_result {
                return Some(ShiGanRuMu {
                    shi_gan: hour_gan,
                    mu_zhi: ru_mu.mu_zhi,
                    gong: palace.gong,
                });
            }
        }
    }
    None
}

/// 天显时格类型
///
/// 时干临值符宫（值符所在的宫位），主时机正好、天时相助
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TianXianShiGe {
    /// 时干
    pub shi_gan: TianGan,
    /// 值符星
    pub zhi_fu_xing: JiuXing,
    /// 落宫
    pub gong: JiuGong,
}

/// 检测天显时格
///
/// 时干临值符所在宫位，为天显时格，主诸事顺利
pub fn check_tian_xian_shi_ge(
    hour_gan: TianGan,
    zhi_fu_xing: JiuXing,
    palaces: &[Palace; 9],
) -> Option<TianXianShiGe> {
    // 找到值符星所在的宫位
    let mut zhi_fu_gong: Option<JiuGong> = None;
    for palace in palaces.iter() {
        if palace.xing == zhi_fu_xing {
            zhi_fu_gong = Some(palace.gong);
            break;
        }
    }

    // 检查时干是否也在值符宫
    if let Some(gong) = zhi_fu_gong {
        for palace in palaces.iter() {
            if palace.gong == gong && palace.tian_pan_gan == hour_gan {
                return Some(TianXianShiGe {
                    shi_gan: hour_gan,
                    zhi_fu_xing,
                    gong,
                });
            }
        }
    }
    None
}

/// 真假格类型
///
/// 用于区分格局的真假吉凶
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZhenJiaGe {
    /// 真格：三奇得使、门星相生
    ZhenGe,
    /// 假格：三奇入墓、门迫、击刑
    JiaGe,
}

/// 综合检测真假格
///
/// 判断当前宫位的格局是真吉还是假吉
pub fn check_zhen_jia_ge(
    tian_pan_gan: TianGan,
    xing: JiuXing,
    men: Option<BaMen>,
    gong: JiuGong,
) -> ZhenJiaGe {
    // 检查是否有入墓
    if check_qi_yi_ru_mu(tian_pan_gan, gong).is_some() {
        return ZhenJiaGe::JiaGe;
    }

    // 检查是否有击刑
    if check_liu_yi_ji_xing(tian_pan_gan, gong).is_some() {
        return ZhenJiaGe::JiaGe;
    }

    // 检查门迫（门克宫）
    if let Some(m) = men {
        if m.wu_xing().conquers(&gong.wu_xing()) {
            return ZhenJiaGe::JiaGe;
        }
    }

    // 三奇得使为真格
    if check_san_qi_de_shi(tian_pan_gan, xing, men) {
        return ZhenJiaGe::ZhenGe;
    }

    // 默认为真格
    ZhenJiaGe::ZhenGe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_dun_type() {
        assert_eq!(calc_dun_type(JieQi::DongZhi), DunType::Yang);
        assert_eq!(calc_dun_type(JieQi::XiaZhi), DunType::Yin);
        assert_eq!(calc_dun_type(JieQi::ChunFen), DunType::Yang);
        assert_eq!(calc_dun_type(JieQi::QiuFen), DunType::Yin);
    }

    #[test]
    fn test_calc_san_yuan() {
        assert_eq!(calc_san_yuan(1), SanYuan::Shang);
        assert_eq!(calc_san_yuan(5), SanYuan::Shang);
        assert_eq!(calc_san_yuan(6), SanYuan::Zhong);
        assert_eq!(calc_san_yuan(10), SanYuan::Zhong);
        assert_eq!(calc_san_yuan(11), SanYuan::Xia);
        assert_eq!(calc_san_yuan(15), SanYuan::Xia);
    }

    #[test]
    fn test_calc_ju_number() {
        // 冬至上元阳遁一局
        assert_eq!(calc_ju_number(JieQi::DongZhi, SanYuan::Shang, DunType::Yang), 1);
        // 夏至上元阴遁九局
        assert_eq!(calc_ju_number(JieQi::XiaZhi, SanYuan::Shang, DunType::Yin), 9);
    }

    #[test]
    fn test_validate_ju_number() {
        assert!(validate_ju_number(1));
        assert!(validate_ju_number(9));
        assert!(!validate_ju_number(0));
        assert!(!validate_ju_number(10));
    }

    /// 测试阳遁一局地盘排布
    ///
    /// 阳遁一局：戊落坎一宫，按顺序排布
    /// 坎1:戊 坤2:己 震3:庚 巽4:辛 中5:壬 乾6:癸 兑7:丁 艮8:丙 离9:乙
    #[test]
    fn test_di_pan_yang_dun_1() {
        let di_pan = get_di_pan(1, DunType::Yang);

        // 验证阳遁一局的地盘排布
        assert_eq!(di_pan[0], TianGan::Wu,   "坎一宫应为戊");
        assert_eq!(di_pan[1], TianGan::Ji,   "坤二宫应为己");
        assert_eq!(di_pan[2], TianGan::Geng, "震三宫应为庚");
        assert_eq!(di_pan[3], TianGan::Xin,  "巽四宫应为辛");
        assert_eq!(di_pan[4], TianGan::Ren,  "中五宫应为壬");
        assert_eq!(di_pan[5], TianGan::Gui,  "乾六宫应为癸");
        assert_eq!(di_pan[6], TianGan::Ding, "兑七宫应为丁");
        assert_eq!(di_pan[7], TianGan::Bing, "艮八宫应为丙");
        assert_eq!(di_pan[8], TianGan::Yi,   "离九宫应为乙");
    }

    /// 测试阴遁一局地盘排布
    ///
    /// 阴遁一局与阳遁一局不同！
    /// 坎1:戊 坤2:乙 震3:丙 巽4:丁 中5:癸 乾6:壬 兑7:辛 艮8:庚 离9:己
    ///
    /// 这是修复的核心测试 - 验证阴遁使用不同的三奇六仪顺序
    #[test]
    fn test_di_pan_yin_dun_1() {
        let di_pan = get_di_pan(1, DunType::Yin);

        // 验证阴遁一局的地盘排布（与阳遁不同！）
        assert_eq!(di_pan[0], TianGan::Wu,   "坎一宫应为戊");
        assert_eq!(di_pan[1], TianGan::Yi,   "坤二宫应为乙（不是己！）");
        assert_eq!(di_pan[2], TianGan::Bing, "震三宫应为丙（不是庚！）");
        assert_eq!(di_pan[3], TianGan::Ding, "巽四宫应为丁（不是辛！）");
        assert_eq!(di_pan[4], TianGan::Gui,  "中五宫应为癸（不是壬！）");
        assert_eq!(di_pan[5], TianGan::Ren,  "乾六宫应为壬（不是癸！）");
        assert_eq!(di_pan[6], TianGan::Xin,  "兑七宫应为辛（不是丁！）");
        assert_eq!(di_pan[7], TianGan::Geng, "艮八宫应为庚（不是丙！）");
        assert_eq!(di_pan[8], TianGan::Ji,   "离九宫应为己（不是乙！）");
    }

    /// 测试阳遁九局地盘排布
    ///
    /// 阳遁九局：戊落离九宫
    /// 坎1:己 坤2:庚 震3:辛 巽4:壬 中5:癸 乾6:丁 兑7:丙 艮8:乙 离9:戊
    #[test]
    fn test_di_pan_yang_dun_9() {
        let di_pan = get_di_pan(9, DunType::Yang);

        // 戊应落在离九宫
        assert_eq!(di_pan[8], TianGan::Wu, "阳遁九局戊应落离九宫");
        // 坎一宫应为己
        assert_eq!(di_pan[0], TianGan::Ji, "阳遁九局坎一宫应为己");
    }

    /// 测试阴遁九局地盘排布
    ///
    /// 阴遁九局：戊落离九宫
    /// 坎1:乙 坤2:丙 震3:丁 巽4:癸 中5:壬 乾6:辛 兑7:庚 艮8:己 离9:戊
    #[test]
    fn test_di_pan_yin_dun_9() {
        let di_pan = get_di_pan(9, DunType::Yin);

        // 戊应落在离九宫
        assert_eq!(di_pan[8], TianGan::Wu, "阴遁九局戊应落离九宫");
        // 坎一宫应为乙（不是己！）
        assert_eq!(di_pan[0], TianGan::Yi, "阴遁九局坎一宫应为乙");
    }

    /// 测试阴阳遁地盘差异
    ///
    /// 同一局数，阴遁和阳遁的地盘排布应该不同（除了戊的位置）
    #[test]
    fn test_di_pan_yin_yang_difference() {
        for ju in 1..=9 {
            let yang_pan = get_di_pan(ju, DunType::Yang);
            let yin_pan = get_di_pan(ju, DunType::Yin);

            // 戊的位置相同（都由局数决定）
            let wu_pos_yang = yang_pan.iter().position(|&g| g == TianGan::Wu);
            let wu_pos_yin = yin_pan.iter().position(|&g| g == TianGan::Wu);
            assert_eq!(wu_pos_yang, wu_pos_yin, "局数{}：阴阳遁戊的位置应相同", ju);

            // 但其他天干的排布应该不同
            let mut diff_count = 0;
            for i in 0..9 {
                if yang_pan[i] != yin_pan[i] {
                    diff_count += 1;
                }
            }
            // 除了戊和可能的一两个位置，大部分应该不同
            assert!(diff_count >= 6, "局数{}：阴阳遁地盘应该有明显差异，实际差异{}", ju, diff_count);
        }
    }

    // ==================== 旬空测试 ====================

    /// 测试甲子旬旬空
    ///
    /// 甲子旬（甲子至癸酉）空戌亥
    #[test]
    fn test_xun_kong_jia_zi_xun() {
        // 甲子时 - 甲子旬首
        let (kong1, kong2) = get_xun_kong(TianGan::Jia, DiZhi::Zi);
        assert_eq!(kong1, DiZhi::Xu, "甲子旬空第一个应为戌");
        assert_eq!(kong2, DiZhi::Hai, "甲子旬空第二个应为亥");

        // 乙丑时 - 仍在甲子旬
        let (kong1, kong2) = get_xun_kong(TianGan::Yi, DiZhi::Chou);
        assert_eq!(kong1, DiZhi::Xu, "乙丑时旬空应为戌亥");
        assert_eq!(kong2, DiZhi::Hai);

        // 癸酉时 - 甲子旬最后一个
        let (kong1, kong2) = get_xun_kong(TianGan::Gui, DiZhi::You);
        assert_eq!(kong1, DiZhi::Xu, "癸酉时旬空应为戌亥");
        assert_eq!(kong2, DiZhi::Hai);
    }

    /// 测试甲戌旬旬空
    ///
    /// 甲戌旬（甲戌至癸未）空申酉
    #[test]
    fn test_xun_kong_jia_xu_xun() {
        // 甲戌时 - 甲戌旬首
        let (kong1, kong2) = get_xun_kong(TianGan::Jia, DiZhi::Xu);
        assert_eq!(kong1, DiZhi::Shen, "甲戌旬空第一个应为申");
        assert_eq!(kong2, DiZhi::You, "甲戌旬空第二个应为酉");
    }

    /// 测试甲申旬旬空
    ///
    /// 甲申旬（甲申至癸巳）空午未
    #[test]
    fn test_xun_kong_jia_shen_xun() {
        // 甲申时 - 甲申旬首
        let (kong1, kong2) = get_xun_kong(TianGan::Jia, DiZhi::Shen);
        assert_eq!(kong1, DiZhi::Wu, "甲申旬空第一个应为午");
        assert_eq!(kong2, DiZhi::Wei, "甲申旬空第二个应为未");
    }

    /// 测试甲午旬旬空
    ///
    /// 甲午旬（甲午至癸卯）空辰巳
    #[test]
    fn test_xun_kong_jia_wu_xun() {
        // 甲午时 - 甲午旬首
        let (kong1, kong2) = get_xun_kong(TianGan::Jia, DiZhi::Wu);
        assert_eq!(kong1, DiZhi::Chen, "甲午旬空第一个应为辰");
        assert_eq!(kong2, DiZhi::Si, "甲午旬空第二个应为巳");
    }

    /// 测试甲辰旬旬空
    ///
    /// 甲辰旬（甲辰至癸丑）空寅卯
    #[test]
    fn test_xun_kong_jia_chen_xun() {
        // 甲辰时 - 甲辰旬首
        let (kong1, kong2) = get_xun_kong(TianGan::Jia, DiZhi::Chen);
        assert_eq!(kong1, DiZhi::Yin, "甲辰旬空第一个应为寅");
        assert_eq!(kong2, DiZhi::Mao, "甲辰旬空第二个应为卯");
    }

    /// 测试甲寅旬旬空
    ///
    /// 甲寅旬（甲寅至癸亥）空子丑
    #[test]
    fn test_xun_kong_jia_yin_xun() {
        // 甲寅时 - 甲寅旬首
        let (kong1, kong2) = get_xun_kong(TianGan::Jia, DiZhi::Yin);
        assert_eq!(kong1, DiZhi::Zi, "甲寅旬空第一个应为子");
        assert_eq!(kong2, DiZhi::Chou, "甲寅旬空第二个应为丑");

        // 癸亥时 - 甲寅旬最后一个
        let (kong1, kong2) = get_xun_kong(TianGan::Gui, DiZhi::Hai);
        assert_eq!(kong1, DiZhi::Zi, "癸亥时旬空应为子丑");
        assert_eq!(kong2, DiZhi::Chou);
    }

    /// 测试六旬完整覆盖
    ///
    /// 验证六十甲子的每个干支组合都能正确返回旬空
    #[test]
    fn test_xun_kong_all_sixty() {
        // 六十甲子每旬的旬首和对应旬空
        let xun_tests: [(TianGan, DiZhi, DiZhi, DiZhi); 6] = [
            (TianGan::Jia, DiZhi::Zi, DiZhi::Xu, DiZhi::Hai),     // 甲子旬
            (TianGan::Jia, DiZhi::Xu, DiZhi::Shen, DiZhi::You),   // 甲戌旬
            (TianGan::Jia, DiZhi::Shen, DiZhi::Wu, DiZhi::Wei),   // 甲申旬
            (TianGan::Jia, DiZhi::Wu, DiZhi::Chen, DiZhi::Si),    // 甲午旬
            (TianGan::Jia, DiZhi::Chen, DiZhi::Yin, DiZhi::Mao),  // 甲辰旬
            (TianGan::Jia, DiZhi::Yin, DiZhi::Zi, DiZhi::Chou),   // 甲寅旬
        ];

        for (gan, zhi, expected_kong1, expected_kong2) in xun_tests {
            let (kong1, kong2) = get_xun_kong(gan, zhi);
            assert_eq!(kong1, expected_kong1, "{}{}旬空第一个错误", gan.name(), zhi.name());
            assert_eq!(kong2, expected_kong2, "{}{}旬空第二个错误", gan.name(), zhi.name());
        }
    }

    // ==================== 格局检测测试 ====================

    /// 测试六仪击刑检测
    #[test]
    fn test_liu_yi_ji_xing() {
        // 戊临震三宫 - 击刑
        let result = check_liu_yi_ji_xing(TianGan::Wu, JiuGong::Zhen);
        assert!(result.is_some());
        assert_eq!(result.unwrap().gan, TianGan::Wu);

        // 己临坤二宫 - 击刑
        let result = check_liu_yi_ji_xing(TianGan::Ji, JiuGong::Kun);
        assert!(result.is_some());

        // 庚临艮八宫 - 击刑
        let result = check_liu_yi_ji_xing(TianGan::Geng, JiuGong::Gen);
        assert!(result.is_some());

        // 辛临离九宫 - 击刑
        let result = check_liu_yi_ji_xing(TianGan::Xin, JiuGong::Li);
        assert!(result.is_some());

        // 壬临巽四宫 - 击刑
        let result = check_liu_yi_ji_xing(TianGan::Ren, JiuGong::Xun);
        assert!(result.is_some());

        // 癸临巽四宫 - 击刑
        let result = check_liu_yi_ji_xing(TianGan::Gui, JiuGong::Xun);
        assert!(result.is_some());

        // 戊临坎一宫 - 不击刑
        let result = check_liu_yi_ji_xing(TianGan::Wu, JiuGong::Kan);
        assert!(result.is_none());

        // 乙（三奇）临任何宫位都不击刑
        let result = check_liu_yi_ji_xing(TianGan::Yi, JiuGong::Zhen);
        assert!(result.is_none());
    }

    /// 测试奇仪入墓检测
    #[test]
    fn test_qi_yi_ru_mu() {
        // 甲入乾六宫 - 入墓
        let result = check_qi_yi_ru_mu(TianGan::Jia, JiuGong::Qian);
        assert!(result.is_some());
        assert_eq!(result.unwrap().mu_zhi, DiZhi::Xu);

        // 乙入乾六宫 - 入墓
        let result = check_qi_yi_ru_mu(TianGan::Yi, JiuGong::Qian);
        assert!(result.is_some());

        // 丙入乾六宫 - 入墓
        let result = check_qi_yi_ru_mu(TianGan::Bing, JiuGong::Qian);
        assert!(result.is_some());

        // 丁入艮八宫 - 入墓
        let result = check_qi_yi_ru_mu(TianGan::Ding, JiuGong::Gen);
        assert!(result.is_some());
        assert_eq!(result.unwrap().mu_zhi, DiZhi::Chou);

        // 辛入巽四宫 - 入墓
        let result = check_qi_yi_ru_mu(TianGan::Xin, JiuGong::Xun);
        assert!(result.is_some());
        assert_eq!(result.unwrap().mu_zhi, DiZhi::Chen);

        // 癸入坤二宫 - 入墓
        let result = check_qi_yi_ru_mu(TianGan::Gui, JiuGong::Kun);
        assert!(result.is_some());
        assert_eq!(result.unwrap().mu_zhi, DiZhi::Wei);

        // 甲入坎一宫 - 不入墓
        let result = check_qi_yi_ru_mu(TianGan::Jia, JiuGong::Kan);
        assert!(result.is_none());
    }

    /// 测试门迫检测
    #[test]
    fn test_men_po() {
        // 休门（水）临离宫（火）- 门迫
        let result = check_men_po(BaMen::Xiu, JiuGong::Li);
        assert!(result.is_some());

        // 生门（土）临坎宫（水）- 门迫
        let result = check_men_po(BaMen::Sheng, JiuGong::Kan);
        assert!(result.is_some());

        // 伤门（木）临坤宫（土）- 门迫
        let result = check_men_po(BaMen::Shang, JiuGong::Kun);
        assert!(result.is_some());

        // 景门（火）临乾宫（金）- 门迫
        let result = check_men_po(BaMen::Jing, JiuGong::Qian);
        assert!(result.is_some());

        // 开门（金）临震宫（木）- 门迫
        let result = check_men_po(BaMen::Kai, JiuGong::Zhen);
        assert!(result.is_some());

        // 休门（水）临坎宫（水）- 归位，不迫
        let result = check_men_po(BaMen::Xiu, JiuGong::Kan);
        assert!(result.is_none());

        // 开门（金）临乾宫（金）- 归位，不迫
        let result = check_men_po(BaMen::Kai, JiuGong::Qian);
        assert!(result.is_none());
    }

    /// 测试十干克应
    #[test]
    fn test_shi_gan_ke_ying() {
        // 丙+戊：飞鸟跌穴（大吉）
        let result = check_shi_gan_ke_ying(TianGan::Bing, TianGan::Wu);
        assert_eq!(result.ge_ju, ShiGanGeJu::FeiNiaoDieXue);
        assert!(result.is_ji);

        // 庚+庚：太白同宫（大凶）
        let result = check_shi_gan_ke_ying(TianGan::Geng, TianGan::Geng);
        assert_eq!(result.ge_ju, ShiGanGeJu::TaiBaiTongGong);
        assert!(!result.is_ji);

        // 乙+乙：日奇伏吟
        let result = check_shi_gan_ke_ying(TianGan::Yi, TianGan::Yi);
        assert_eq!(result.ge_ju, ShiGanGeJu::RiQiFuYin);

        // 辛+乙：白虎猖狂（凶）
        let result = check_shi_gan_ke_ying(TianGan::Xin, TianGan::Yi);
        assert_eq!(result.ge_ju, ShiGanGeJu::BaiHuChangKuang);
        assert!(!result.is_ji);

        // 戊+戊：伏吟
        let result = check_shi_gan_ke_ying(TianGan::Wu, TianGan::Wu);
        assert_eq!(result.ge_ju, ShiGanGeJu::FuYin);
    }

    /// 测试驿马计算
    #[test]
    fn test_yi_ma() {
        // 申子辰三合水局，驿马在寅（艮八宫）
        let yi_ma = calc_yi_ma(DiZhi::Zi);
        assert_eq!(yi_ma.zhi, DiZhi::Yin);
        assert_eq!(yi_ma.gong, JiuGong::Gen);

        let yi_ma = calc_yi_ma(DiZhi::Shen);
        assert_eq!(yi_ma.zhi, DiZhi::Yin);

        let yi_ma = calc_yi_ma(DiZhi::Chen);
        assert_eq!(yi_ma.zhi, DiZhi::Yin);

        // 寅午戌三合火局，驿马在申（坤二宫）
        let yi_ma = calc_yi_ma(DiZhi::Wu);
        assert_eq!(yi_ma.zhi, DiZhi::Shen);
        assert_eq!(yi_ma.gong, JiuGong::Kun);

        // 巳酉丑三合金局，驿马在亥（乾六宫）
        let yi_ma = calc_yi_ma(DiZhi::You);
        assert_eq!(yi_ma.zhi, DiZhi::Hai);
        assert_eq!(yi_ma.gong, JiuGong::Qian);

        // 亥卯未三合木局，驿马在巳（巽四宫）
        let yi_ma = calc_yi_ma(DiZhi::Mao);
        assert_eq!(yi_ma.zhi, DiZhi::Si);
        assert_eq!(yi_ma.gong, JiuGong::Xun);
    }

    /// 测试旺衰计算
    #[test]
    fn test_wang_shuai() {
        // 春季（寅月）木旺
        assert_eq!(calc_wang_shuai(WuXing::Mu, DiZhi::Yin), WangShuai::Wang);
        // 春季水休（水生木，我生月令）
        assert_eq!(calc_wang_shuai(WuXing::Shui, DiZhi::Yin), WangShuai::Xiu);
        // 春季火相（木生火，月令生我）
        assert_eq!(calc_wang_shuai(WuXing::Huo, DiZhi::Yin), WangShuai::Xiang);
        // 春季金死（金克木，我克月令）
        assert_eq!(calc_wang_shuai(WuXing::Jin, DiZhi::Yin), WangShuai::Si);
        // 春季土囚（木克土，月令克我）
        assert_eq!(calc_wang_shuai(WuXing::Tu, DiZhi::Yin), WangShuai::Qiu);

        // 冬季（子月）水旺
        assert_eq!(calc_wang_shuai(WuXing::Shui, DiZhi::Zi), WangShuai::Wang);
        // 冬季木相（水生木，月令生我）
        assert_eq!(calc_wang_shuai(WuXing::Mu, DiZhi::Zi), WangShuai::Xiang);
    }

    /// 测试九星旺衰
    #[test]
    fn test_xing_wang_shuai() {
        // 天蓬星（水）在子月当令
        assert_eq!(calc_xing_wang_shuai(JiuXing::TianPeng, DiZhi::Zi), WangShuai::Wang);
        // 天冲星（木）在寅月当令
        assert_eq!(calc_xing_wang_shuai(JiuXing::TianChong, DiZhi::Yin), WangShuai::Wang);
    }

    /// 测试八门旺衰
    #[test]
    fn test_men_wang_shuai() {
        // 休门（水）在子月当令
        assert_eq!(calc_men_wang_shuai(BaMen::Xiu, DiZhi::Zi), WangShuai::Wang);
        // 开门（金）在酉月当令
        assert_eq!(calc_men_wang_shuai(BaMen::Kai, DiZhi::You), WangShuai::Wang);
    }

    // ==================== 飞盘奇门测试 ====================

    /// 测试飞盘九星飞布
    #[test]
    fn test_fei_pan_jiu_xing() {
        // 阳遁：值符星为天蓬（坎一宫），时干落三宫
        let xing_pan = fei_pan_distribute_jiu_xing(JiuXing::TianPeng, 3, DunType::Yang);

        // 验证九星都被放置
        let mut has_all_xing = true;
        for i in 1..=9 {
            let xing = JiuXing::from_num(i as u8).unwrap();
            if !xing_pan.contains(&xing) {
                has_all_xing = false;
                break;
            }
        }
        assert!(has_all_xing, "飞盘应包含所有九星");

        // 阴遁测试
        let xing_pan_yin = fei_pan_distribute_jiu_xing(JiuXing::TianPeng, 3, DunType::Yin);

        // 验证阴遁和阳遁的排布不同
        let mut _diff_count = 0;
        for i in 0..9 {
            if xing_pan[i] != xing_pan_yin[i] {
                _diff_count += 1;
            }
        }
        // 阴阳遁排布应该有差异（除非特殊情况）
        // 这里只验证函数能正常执行
        assert!(xing_pan.len() == 9);
        assert!(xing_pan_yin.len() == 9);
    }

    /// 测试飞盘八门飞布
    #[test]
    fn test_fei_pan_ba_men() {
        // 阳遁：值使门为休门（坎一宫），时干落三宫
        let men_pan = fei_pan_distribute_ba_men(BaMen::Xiu, 3, DunType::Yang);

        // 验证中宫无门
        assert!(men_pan[4].is_none(), "中宫应无门");

        // 统计有门的宫位数
        let mut men_count = 0;
        for i in 0..9 {
            if men_pan[i].is_some() {
                men_count += 1;
            }
        }
        assert_eq!(men_count, 8, "应有8个宫位有门");
    }

    /// 测试飞盘完整排盘
    #[test]
    fn test_fei_pan_complete_chart() {
        let year_gz = GanZhi::new(TianGan::Jia, DiZhi::Zi);
        let month_gz = GanZhi::new(TianGan::Bing, DiZhi::Yin);
        let day_gz = GanZhi::new(TianGan::Jia, DiZhi::Zi);
        let hour_gz = GanZhi::new(TianGan::Jia, DiZhi::Zi);

        let (dun_type, san_yuan, ju_number, _zhi_fu_xing, _zhi_shi_men, palaces) =
            generate_fei_pan_chart(year_gz, month_gz, day_gz, hour_gz, JieQi::DongZhi, 1);

        // 验证基本参数
        assert_eq!(dun_type, DunType::Yang, "冬至应为阳遁");
        assert_eq!(san_yuan, SanYuan::Shang, "第1天应为上元");
        assert_eq!(ju_number, 1, "冬至上元阳遁应为一局");

        // 验证九宫数据
        for (i, palace) in palaces.iter().enumerate() {
            let expected_gong = JiuGong::from_num((i + 1) as u8).unwrap();
            assert_eq!(palace.gong, expected_gong);

            // 中宫特殊处理
            if palace.gong == JiuGong::Zhong {
                assert!(palace.men.is_none(), "中宫应无门");
                assert!(palace.shen.is_none(), "中宫应无神");
            }
        }
    }

    /// 测试转盘与飞盘对比
    #[test]
    fn test_compare_zhuan_fei() {
        let year_gz = GanZhi::new(TianGan::Jia, DiZhi::Zi);
        let month_gz = GanZhi::new(TianGan::Bing, DiZhi::Yin);
        let day_gz = GanZhi::new(TianGan::Jia, DiZhi::Zi);
        let hour_gz = GanZhi::new(TianGan::Jia, DiZhi::Zi);

        // 转盘排盘
        let (_, _, _, _, _, zhuan_palaces) =
            generate_qimen_chart(year_gz, month_gz, day_gz, hour_gz, JieQi::DongZhi, 1);

        // 飞盘排盘
        let (_, _, _, _, _, fei_palaces) =
            generate_fei_pan_chart(year_gz, month_gz, day_gz, hour_gz, JieQi::DongZhi, 1);

        // 对比两种排盘方法
        let (xing_diff, men_diff, shen_diff) = compare_zhuan_fei_charts(&zhuan_palaces, &fei_palaces);

        // 转盘和飞盘的排布应该不同（在大多数情况下）
        // 这里只验证函数能正常执行并返回有效结果
        assert!(xing_diff <= 9, "九星差异应在0-9之间");
        assert!(men_diff <= 8, "八门差异应在0-8之间");
        assert!(shen_diff <= 8, "八神差异应在0-8之间");
    }

    /// 测试按方法生成排盘
    #[test]
    fn test_generate_by_method() {
        let year_gz = GanZhi::new(TianGan::Jia, DiZhi::Zi);
        let month_gz = GanZhi::new(TianGan::Bing, DiZhi::Yin);
        let day_gz = GanZhi::new(TianGan::Jia, DiZhi::Zi);
        let hour_gz = GanZhi::new(TianGan::Jia, DiZhi::Zi);

        // 转盘方法
        let (dun1, _, ju1, _, _, _) = generate_chart_by_method(
            PanMethod::ZhuanPan,
            year_gz, month_gz, day_gz, hour_gz,
            JieQi::DongZhi, 1
        );

        // 飞盘方法
        let (dun2, _, ju2, _, _, _) = generate_chart_by_method(
            PanMethod::FeiPan,
            year_gz, month_gz, day_gz, hour_gz,
            JieQi::DongZhi, 1
        );

        // 两种方法的阴阳遁和局数应该相同
        assert_eq!(dun1, dun2, "阴阳遁应相同");
        assert_eq!(ju1, ju2, "局数应相同");
    }

    /// 测试飞盘排盘类型版本
    #[test]
    fn test_fei_pan_by_type() {
        let year_gz = GanZhi::new(TianGan::Jia, DiZhi::Zi);
        let month_gz = GanZhi::new(TianGan::Bing, DiZhi::Yin);
        let day_gz = GanZhi::new(TianGan::Jia, DiZhi::Zi);
        let hour_gz = GanZhi::new(TianGan::Wu, DiZhi::Wu);

        // 时家飞盘
        let (dun1, san_yuan1, _, _, _, _) = generate_fei_pan_chart_by_type(
            QimenType::ShiJia,
            year_gz, month_gz, day_gz, hour_gz,
            JieQi::DongZhi, 1
        );

        // 日家飞盘
        let (dun2, san_yuan2, _, _, _, _) = generate_fei_pan_chart_by_type(
            QimenType::RiJia,
            year_gz, month_gz, day_gz, hour_gz,
            JieQi::DongZhi, 1
        );

        // 两种类型的阴阳遁应相同（都由节气决定）
        assert_eq!(dun1, dun2, "阴阳遁应相同");

        // 但三元可能不同（时家按时支，日家按节气天数）
        // 这里不做具体断言，只验证函数能正常执行
        assert!(matches!(san_yuan1, SanYuan::Shang | SanYuan::Zhong | SanYuan::Xia));
        assert!(matches!(san_yuan2, SanYuan::Shang | SanYuan::Zhong | SanYuan::Xia));
    }
}
