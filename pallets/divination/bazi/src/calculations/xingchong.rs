//! # 八字刑冲合会系统
//!
//! 本模块实现八字命理中的地支关系计算，包括：
//! - 六合：子丑合土、寅亥合木等
//! - 三合：申子辰水、亥卯未木等
//! - 半合：两支相合
//! - 六冲：子午冲、丑未冲等
//! - 三刑：寅巳申无恩之刑等
//! - 六害：子未害等
//! - 六破：子酉破等
//! - 天干五合：甲己合土、乙庚合金等
//!
//! 参考 bazi-mcp 项目实现

use crate::types::*;
use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

// ================================
// 关系类型定义
// ================================

/// 地支关系类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum DiZhiGuanXi {
    /// 六合
    LiuHe,
    /// 三合
    SanHe,
    /// 半合
    BanHe,
    /// 六冲
    LiuChong,
    /// 三刑
    SanXing,
    /// 自刑
    ZiXing,
    /// 六害
    LiuHai,
    /// 六破
    LiuPo,
}

impl DiZhiGuanXi {
    /// 获取关系名称
    pub fn name(&self) -> &'static str {
        match self {
            DiZhiGuanXi::LiuHe => "六合",
            DiZhiGuanXi::SanHe => "三合",
            DiZhiGuanXi::BanHe => "半合",
            DiZhiGuanXi::LiuChong => "六冲",
            DiZhiGuanXi::SanXing => "三刑",
            DiZhiGuanXi::ZiXing => "自刑",
            DiZhiGuanXi::LiuHai => "六害",
            DiZhiGuanXi::LiuPo => "六破",
        }
    }

    /// 判断是否为吉利关系
    pub fn is_favorable(&self) -> bool {
        matches!(self, DiZhiGuanXi::LiuHe | DiZhiGuanXi::SanHe | DiZhiGuanXi::BanHe)
    }

    /// 判断是否为不利关系
    pub fn is_unfavorable(&self) -> bool {
        matches!(
            self,
            DiZhiGuanXi::LiuChong
                | DiZhiGuanXi::SanXing
                | DiZhiGuanXi::ZiXing
                | DiZhiGuanXi::LiuHai
                | DiZhiGuanXi::LiuPo
        )
    }
}

/// 天干关系类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum TianGanGuanXi {
    /// 五合（甲己合土、乙庚合金等）
    WuHe,
    /// 相冲（甲庚冲、乙辛冲等）
    XiangChong,
}

impl TianGanGuanXi {
    /// 获取关系名称
    pub fn name(&self) -> &'static str {
        match self {
            TianGanGuanXi::WuHe => "五合",
            TianGanGuanXi::XiangChong => "相冲",
        }
    }
}

/// 合化后的五行
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct HeHuaResult {
    /// 合化后的五行
    pub wuxing: WuXing,
    /// 合化描述索引（对应 LIUHE_DESC 等常量数组）
    pub desc_index: u8,
}

// ================================
// 六合常量表
// ================================

/// 六合表
/// 格式：LIUHE[地支] = (合的地支, 合化五行)
/// 子丑合土，寅亥合木，卯戌合火，辰酉合金，巳申合水，午未合土
pub const LIUHE: [(u8, WuXing); 12] = [
    (1, WuXing::Tu),    // 子(0)合丑(1)化土
    (0, WuXing::Tu),    // 丑(1)合子(0)化土
    (11, WuXing::Mu),   // 寅(2)合亥(11)化木
    (10, WuXing::Huo),  // 卯(3)合戌(10)化火
    (9, WuXing::Jin),   // 辰(4)合酉(9)化金
    (8, WuXing::Shui),  // 巳(5)合申(8)化水
    (7, WuXing::Tu),    // 午(6)合未(7)化土
    (6, WuXing::Tu),    // 未(7)合午(6)化土
    (5, WuXing::Shui),  // 申(8)合巳(5)化水
    (4, WuXing::Jin),   // 酉(9)合辰(4)化金
    (3, WuXing::Huo),   // 戌(10)合卯(3)化火
    (2, WuXing::Mu),    // 亥(11)合寅(2)化木
];

/// 六合描述
pub const LIUHE_DESC: [&str; 6] = [
    "子丑合土",
    "寅亥合木",
    "卯戌合火",
    "辰酉合金",
    "巳申合水",
    "午未合土",
];

// ================================
// 三合常量表
// ================================

/// 三合局
/// 格式：[局首, 局中, 局尾, 化五行索引]
/// 申子辰水局，亥卯未木局，寅午戌火局，巳酉丑金局
pub const SANHE: [[u8; 4]; 4] = [
    [8, 0, 4, 2],   // 申子辰合水 (Shui=2)
    [11, 3, 7, 1],  // 亥卯未合木 (Mu=1)
    [2, 6, 10, 3],  // 寅午戌合火 (Huo=3)
    [5, 9, 1, 0],   // 巳酉丑合金 (Jin=0)
];

/// 三合描述
pub const SANHE_DESC: [&str; 4] = ["申子辰合水", "亥卯未合木", "寅午戌合火", "巳酉丑合金"];

// ================================
// 六冲常量表
// ================================

/// 六冲表
/// 格式：LIUCHONG[地支] = 冲的地支
/// 子午冲，丑未冲，寅申冲，卯酉冲，辰戌冲，巳亥冲
pub const LIUCHONG: [u8; 12] = [
    6,   // 子(0)冲午(6)
    7,   // 丑(1)冲未(7)
    8,   // 寅(2)冲申(8)
    9,   // 卯(3)冲酉(9)
    10,  // 辰(4)冲戌(10)
    11,  // 巳(5)冲亥(11)
    0,   // 午(6)冲子(0)
    1,   // 未(7)冲丑(1)
    2,   // 申(8)冲寅(2)
    3,   // 酉(9)冲卯(3)
    4,   // 戌(10)冲辰(4)
    5,   // 亥(11)冲巳(5)
];

/// 六冲描述
pub const LIUCHONG_DESC: [&str; 6] = ["子午冲", "丑未冲", "寅申冲", "卯酉冲", "辰戌冲", "巳亥冲"];

// ================================
// 三刑常量表
// ================================

/// 无恩之刑：寅刑巳，巳刑申，申刑寅
/// 恃势之刑：丑刑戌，戌刑未，未刑丑
/// 无礼之刑：子刑卯，卯刑子
/// 自刑：辰辰刑，午午刑，酉酉刑，亥亥刑

/// 三刑表（非自刑）
/// 格式：SANXING[地支] = 刑的地支（255表示无刑或自刑）
pub const SANXING: [u8; 12] = [
    3,    // 子(0)刑卯(3) - 无礼之刑
    10,   // 丑(1)刑戌(10) - 恃势之刑
    5,    // 寅(2)刑巳(5) - 无恩之刑
    0,    // 卯(3)刑子(0) - 无礼之刑
    255,  // 辰(4) - 自刑
    8,    // 巳(5)刑申(8) - 无恩之刑
    255,  // 午(6) - 自刑
    1,    // 未(7)刑丑(1) - 恃势之刑
    2,    // 申(8)刑寅(2) - 无恩之刑
    255,  // 酉(9) - 自刑
    7,    // 戌(10)刑未(7) - 恃势之刑
    255,  // 亥(11) - 自刑
];

/// 自刑地支（辰午酉亥）
pub const ZIXING: [u8; 4] = [4, 6, 9, 11];

// ================================
// 六害常量表
// ================================

/// 六害表
/// 格式：LIUHAI[地支] = 害的地支
/// 子未害，丑午害，寅巳害，卯辰害，申亥害，酉戌害
pub const LIUHAI: [u8; 12] = [
    7,   // 子(0)害未(7)
    6,   // 丑(1)害午(6)
    5,   // 寅(2)害巳(5)
    4,   // 卯(3)害辰(4)
    3,   // 辰(4)害卯(3)
    2,   // 巳(5)害寅(2)
    1,   // 午(6)害丑(1)
    0,   // 未(7)害子(0)
    11,  // 申(8)害亥(11)
    10,  // 酉(9)害戌(10)
    9,   // 戌(10)害酉(9)
    8,   // 亥(11)害申(8)
];

/// 六害描述
pub const LIUHAI_DESC: [&str; 6] = ["子未害", "丑午害", "寅巳害", "卯辰害", "申亥害", "酉戌害"];

// ================================
// 六破常量表
// ================================

/// 六破表
/// 格式：LIUPO[地支] = 破的地支
/// 子酉破，丑辰破，寅亥破，卯午破，巳申破，未戌破
pub const LIUPO: [u8; 12] = [
    9,   // 子(0)破酉(9)
    4,   // 丑(1)破辰(4)
    11,  // 寅(2)破亥(11)
    6,   // 卯(3)破午(6)
    1,   // 辰(4)破丑(1)
    8,   // 巳(5)破申(8)
    3,   // 午(6)破卯(3)
    10,  // 未(7)破戌(10)
    5,   // 申(8)破巳(5)
    0,   // 酉(9)破子(0)
    7,   // 戌(10)破未(7)
    2,   // 亥(11)破寅(2)
];

// ================================
// 天干五合常量表
// ================================

/// 天干五合表
/// 格式：WUHE[天干] = (合的天干, 合化五行)
/// 甲己合土，乙庚合金，丙辛合水，丁壬合木，戊癸合火
pub const WUHE: [(u8, WuXing); 10] = [
    (5, WuXing::Tu),    // 甲(0)合己(5)化土
    (6, WuXing::Jin),   // 乙(1)合庚(6)化金
    (7, WuXing::Shui),  // 丙(2)合辛(7)化水
    (8, WuXing::Mu),    // 丁(3)合壬(8)化木
    (9, WuXing::Huo),   // 戊(4)合癸(9)化火
    (0, WuXing::Tu),    // 己(5)合甲(0)化土
    (1, WuXing::Jin),   // 庚(6)合乙(1)化金
    (2, WuXing::Shui),  // 辛(7)合丙(2)化水
    (3, WuXing::Mu),    // 壬(8)合丁(3)化木
    (4, WuXing::Huo),   // 癸(9)合戊(4)化火
];

/// 天干五合描述
pub const WUHE_DESC: [&str; 5] = ["甲己合土", "乙庚合金", "丙辛合水", "丁壬合木", "戊癸合火"];

// ================================
// 判断函数
// ================================

/// 判断两地支是否六合
pub fn is_liuhe(zhi1: DiZhi, zhi2: DiZhi) -> Option<HeHuaResult> {
    if LIUHE[zhi1.0 as usize].0 == zhi2.0 {
        let desc_index = match (zhi1.0.min(zhi2.0), zhi1.0.max(zhi2.0)) {
            (0, 1) => 0,   // 子丑合土
            (2, 11) => 1,  // 寅亥合木
            (3, 10) => 2,  // 卯戌合火
            (4, 9) => 3,   // 辰酉合金
            (5, 8) => 4,   // 巳申合水
            (6, 7) => 5,   // 午未合土
            _ => 0,
        };
        Some(HeHuaResult {
            wuxing: LIUHE[zhi1.0 as usize].1,
            desc_index,
        })
    } else {
        None
    }
}

/// 判断三个地支是否三合
pub fn is_sanhe(zhi1: DiZhi, zhi2: DiZhi, zhi3: DiZhi) -> Option<HeHuaResult> {
    let mut zhis = [zhi1.0, zhi2.0, zhi3.0];
    zhis.sort();

    for (i, sanhe) in SANHE.iter().enumerate() {
        let mut sanhe_sorted = [sanhe[0], sanhe[1], sanhe[2]];
        sanhe_sorted.sort();
        if zhis == sanhe_sorted {
            let wuxing = match sanhe[3] {
                0 => WuXing::Jin,
                1 => WuXing::Mu,
                2 => WuXing::Shui,
                3 => WuXing::Huo,
                4 => WuXing::Tu,
                _ => WuXing::Tu,
            };
            return Some(HeHuaResult {
                wuxing,
                desc_index: i as u8,
            });
        }
    }
    None
}

/// 判断两地支是否半合
pub fn is_banhe(zhi1: DiZhi, zhi2: DiZhi) -> Option<HeHuaResult> {
    // 半合是三合局中的两个地支
    // 申子、子辰、申辰半合水
    // 亥卯、卯未、亥未半合木
    // 寅午、午戌、寅戌半合火
    // 巳酉、酉丑、巳丑半合金

    let (min, max) = (zhi1.0.min(zhi2.0), zhi1.0.max(zhi2.0));

    let result = match (min, max) {
        // 水局半合
        (0, 8) => Some((0, WuXing::Shui)),   // 申子半合水
        (0, 4) => Some((1, WuXing::Shui)),   // 子辰半合水
        (4, 8) => Some((2, WuXing::Shui)),   // 申辰半合水
        // 木局半合
        (3, 11) => Some((3, WuXing::Mu)),    // 亥卯半合木
        (3, 7) => Some((4, WuXing::Mu)),     // 卯未半合木
        (7, 11) => Some((5, WuXing::Mu)),    // 亥未半合木
        // 火局半合
        (2, 6) => Some((6, WuXing::Huo)),    // 寅午半合火
        (6, 10) => Some((7, WuXing::Huo)),   // 午戌半合火
        (2, 10) => Some((8, WuXing::Huo)),   // 寅戌半合火
        // 金局半合
        (5, 9) => Some((9, WuXing::Jin)),    // 巳酉半合金
        (1, 9) => Some((10, WuXing::Jin)),   // 酉丑半合金
        (1, 5) => Some((11, WuXing::Jin)),   // 巳丑半合金
        _ => None,
    };

    result.map(|(desc_index, wuxing)| HeHuaResult {
        wuxing,
        desc_index,
    })
}

/// 判断两地支是否六冲
pub fn is_liuchong(zhi1: DiZhi, zhi2: DiZhi) -> bool {
    LIUCHONG[zhi1.0 as usize] == zhi2.0
}

/// 判断两地支是否相刑（包括三刑和自刑）
/// 返回刑的类型索引：0-3为自刑（辰午酉亥），4为子卯无礼之刑，5为寅巳申无恩之刑，6为丑戌未恃势之刑
pub fn is_xing(zhi1: DiZhi, zhi2: DiZhi) -> Option<u8> {
    // 检查自刑
    if zhi1.0 == zhi2.0 && ZIXING.contains(&zhi1.0) {
        return Some(match zhi1.0 {
            4 => 0,   // 辰辰自刑
            6 => 1,   // 午午自刑
            9 => 2,   // 酉酉自刑
            11 => 3,  // 亥亥自刑
            _ => 0,
        });
    }

    // 检查三刑
    if SANXING[zhi1.0 as usize] == zhi2.0 && SANXING[zhi1.0 as usize] != 255 {
        return Some(match (zhi1.0, zhi2.0) {
            (0, 3) | (3, 0) => 4,        // 子卯无礼之刑
            (2, 5) | (5, 8) | (8, 2) => 5, // 寅巳申无恩之刑
            (1, 10) | (10, 7) | (7, 1) => 6, // 丑戌未恃势之刑
            _ => 4,
        });
    }

    None
}

/// 判断两地支是否六害
pub fn is_liuhai(zhi1: DiZhi, zhi2: DiZhi) -> bool {
    LIUHAI[zhi1.0 as usize] == zhi2.0
}

/// 判断两地支是否六破
pub fn is_liupo(zhi1: DiZhi, zhi2: DiZhi) -> bool {
    LIUPO[zhi1.0 as usize] == zhi2.0
}

/// 判断两天干是否五合
pub fn is_wuhe(gan1: TianGan, gan2: TianGan) -> Option<HeHuaResult> {
    if WUHE[gan1.0 as usize].0 == gan2.0 {
        let desc_index = match (gan1.0.min(gan2.0), gan1.0.max(gan2.0)) {
            (0, 5) => 0,  // 甲己合土
            (1, 6) => 1,  // 乙庚合金
            (2, 7) => 2,  // 丙辛合水
            (3, 8) => 3,  // 丁壬合木
            (4, 9) => 4,  // 戊癸合火
            _ => 0,
        };
        Some(HeHuaResult {
            wuxing: WUHE[gan1.0 as usize].1,
            desc_index,
        })
    } else {
        None
    }
}

/// 判断两天干是否相冲
pub fn is_tiangan_chong(gan1: TianGan, gan2: TianGan) -> bool {
    // 天干相冲：甲庚冲，乙辛冲，丙壬冲，丁癸冲
    let diff = (gan1.0 as i8 - gan2.0 as i8).abs();
    diff == 6 && gan1.0 < 4 && gan2.0 >= 4 || gan2.0 < 4 && gan1.0 >= 4
}

// ================================
// 关系结果结构
// ================================

/// 单个关系记录
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct GuanXiRecord {
    /// 关系类型
    pub guanxi_type: DiZhiGuanXi,
    /// 涉及的柱位置（0=年,1=月,2=日,3=时）
    pub zhu_idx1: u8,
    /// 涉及的柱位置
    pub zhu_idx2: u8,
    /// 描述索引（对应常量表）
    pub desc_index: u8,
    /// 合化五行（如果适用）
    pub hehua_wuxing: Option<WuXing>,
}

/// 天干关系记录
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct TianGanGuanXiRecord {
    /// 关系类型
    pub guanxi_type: TianGanGuanXi,
    /// 涉及的柱位置（0=年,1=月,2=日,3=时）
    pub zhu_idx1: u8,
    /// 涉及的柱位置
    pub zhu_idx2: u8,
    /// 合化五行（如果适用）
    pub hehua_wuxing: Option<WuXing>,
    /// 描述索引
    pub desc_index: u8,
}

/// 四柱关系分析结果
#[derive(Clone, Debug, Default, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct SiZhuGuanXi {
    /// 地支六合列表
    pub liuhe_list: BoundedVec<GuanXiRecord, ConstU32<6>>,
    /// 地支半合列表
    pub banhe_list: BoundedVec<GuanXiRecord, ConstU32<6>>,
    /// 地支六冲列表
    pub liuchong_list: BoundedVec<GuanXiRecord, ConstU32<6>>,
    /// 地支三刑列表
    pub xing_list: BoundedVec<GuanXiRecord, ConstU32<6>>,
    /// 地支六害列表
    pub liuhai_list: BoundedVec<GuanXiRecord, ConstU32<6>>,
    /// 天干五合列表
    pub tiangan_wuhe_list: BoundedVec<TianGanGuanXiRecord, ConstU32<6>>,
}

/// 分析四柱地支关系
pub fn analyze_sizhu_guanxi(
    year_ganzhi: &GanZhi,
    month_ganzhi: &GanZhi,
    day_ganzhi: &GanZhi,
    hour_ganzhi: &GanZhi,
) -> SiZhuGuanXi {
    let zhis = [
        year_ganzhi.zhi,
        month_ganzhi.zhi,
        day_ganzhi.zhi,
        hour_ganzhi.zhi,
    ];
    let gans = [
        year_ganzhi.gan,
        month_ganzhi.gan,
        day_ganzhi.gan,
        hour_ganzhi.gan,
    ];

    let mut result = SiZhuGuanXi::default();

    // 遍历所有柱两两组合
    for i in 0..4 {
        for j in (i + 1)..4 {
            // 检查地支六合
            if let Some(hehua) = is_liuhe(zhis[i], zhis[j]) {
                let record = GuanXiRecord {
                    guanxi_type: DiZhiGuanXi::LiuHe,
                    zhu_idx1: i as u8,
                    zhu_idx2: j as u8,
                    desc_index: hehua.desc_index,
                    hehua_wuxing: Some(hehua.wuxing),
                };
                let _ = result.liuhe_list.try_push(record);
            }

            // 检查地支半合
            if let Some(hehua) = is_banhe(zhis[i], zhis[j]) {
                let record = GuanXiRecord {
                    guanxi_type: DiZhiGuanXi::BanHe,
                    zhu_idx1: i as u8,
                    zhu_idx2: j as u8,
                    desc_index: hehua.desc_index,
                    hehua_wuxing: Some(hehua.wuxing),
                };
                let _ = result.banhe_list.try_push(record);
            }

            // 检查地支六冲
            if is_liuchong(zhis[i], zhis[j]) {
                // 六冲描述索引：基于两个地支的最小值
                let desc_index = zhis[i].0.min(zhis[j].0);
                let record = GuanXiRecord {
                    guanxi_type: DiZhiGuanXi::LiuChong,
                    zhu_idx1: i as u8,
                    zhu_idx2: j as u8,
                    desc_index,
                    hehua_wuxing: None,
                };
                let _ = result.liuchong_list.try_push(record);
            }

            // 检查地支三刑
            if let Some(xing_idx) = is_xing(zhis[i], zhis[j]) {
                let record = GuanXiRecord {
                    guanxi_type: DiZhiGuanXi::SanXing,
                    zhu_idx1: i as u8,
                    zhu_idx2: j as u8,
                    desc_index: xing_idx,
                    hehua_wuxing: None,
                };
                let _ = result.xing_list.try_push(record);
            }

            // 检查地支六害
            if is_liuhai(zhis[i], zhis[j]) {
                // 六害描述索引：基于两个地支的最小值
                let desc_index = zhis[i].0.min(zhis[j].0);
                let record = GuanXiRecord {
                    guanxi_type: DiZhiGuanXi::LiuHai,
                    zhu_idx1: i as u8,
                    zhu_idx2: j as u8,
                    desc_index,
                    hehua_wuxing: None,
                };
                let _ = result.liuhai_list.try_push(record);
            }

            // 检查天干五合
            if let Some(hehua) = is_wuhe(gans[i], gans[j]) {
                let record = TianGanGuanXiRecord {
                    guanxi_type: TianGanGuanXi::WuHe,
                    zhu_idx1: i as u8,
                    zhu_idx2: j as u8,
                    hehua_wuxing: Some(hehua.wuxing),
                    desc_index: hehua.desc_index,
                };
                let _ = result.tiangan_wuhe_list.try_push(record);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liuhe() {
        // 子丑合土
        let result = is_liuhe(DiZhi(0), DiZhi(1));
        assert!(result.is_some());
        assert_eq!(result.unwrap().wuxing, WuXing::Tu);

        // 寅亥合木
        let result = is_liuhe(DiZhi(2), DiZhi(11));
        assert!(result.is_some());
        assert_eq!(result.unwrap().wuxing, WuXing::Mu);
    }

    #[test]
    fn test_sanhe() {
        // 申子辰合水
        let result = is_sanhe(DiZhi(8), DiZhi(0), DiZhi(4));
        assert!(result.is_some());
        assert_eq!(result.unwrap().wuxing, WuXing::Shui);

        // 寅午戌合火
        let result = is_sanhe(DiZhi(2), DiZhi(6), DiZhi(10));
        assert!(result.is_some());
        assert_eq!(result.unwrap().wuxing, WuXing::Huo);
    }

    #[test]
    fn test_banhe() {
        // 申子半合水
        let result = is_banhe(DiZhi(8), DiZhi(0));
        assert!(result.is_some());
        assert_eq!(result.unwrap().wuxing, WuXing::Shui);

        // 卯未半合木
        let result = is_banhe(DiZhi(3), DiZhi(7));
        assert!(result.is_some());
        assert_eq!(result.unwrap().wuxing, WuXing::Mu);
    }

    #[test]
    fn test_liuchong() {
        // 子午冲
        assert!(is_liuchong(DiZhi(0), DiZhi(6)));
        // 寅申冲
        assert!(is_liuchong(DiZhi(2), DiZhi(8)));
        // 子丑不冲
        assert!(!is_liuchong(DiZhi(0), DiZhi(1)));
    }

    #[test]
    fn test_xing() {
        // 子卯刑
        assert!(is_xing(DiZhi(0), DiZhi(3)).is_some());
        // 午午自刑
        assert!(is_xing(DiZhi(6), DiZhi(6)).is_some());
    }

    #[test]
    fn test_wuhe() {
        // 甲己合土
        let result = is_wuhe(TianGan(0), TianGan(5));
        assert!(result.is_some());
        assert_eq!(result.unwrap().wuxing, WuXing::Tu);

        // 乙庚合金
        let result = is_wuhe(TianGan(1), TianGan(6));
        assert!(result.is_some());
        assert_eq!(result.unwrap().wuxing, WuXing::Jin);
    }
}
