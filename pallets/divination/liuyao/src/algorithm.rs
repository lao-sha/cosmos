//! # 六爻排盘算法
//!
//! 本模块实现六爻排盘的核心算法，包括：
//! - 纳甲装卦
//! - 世应安装
//! - 六亲配置
//! - 六神排布
//! - 旬空计算
//! - 伏神查找
//!
//! ## 纳甲口诀
//!
//! 乾纳甲壬，坤纳乙癸，震纳庚，巽纳辛，
//! 坎纳戊，离纳己，艮纳丙，兑纳丁。
//!
//! ## 纳支口诀
//!
//! 乾金甲子外壬午，坎水戊寅外戊申，
//! 艮土丙辰外丙戌，震木庚子外庚午，
//! 巽木辛丑外辛未，离火己卯外己酉，
//! 坤土乙未外癸丑，兑金丁巳外丁亥。

use crate::types::*;

// ============================================================================
// 纳甲表
// ============================================================================

/// 内卦天干（根据八卦）
/// 乾纳甲，坤纳乙，震纳庚，巽纳辛，坎纳戊，离纳己，艮纳丙，兑纳丁
pub const INNER_GAN: [TianGan; 8] = [
    TianGan::Jia,  // 乾
    TianGan::Ding, // 兑
    TianGan::Ji,   // 离
    TianGan::Geng, // 震
    TianGan::Xin,  // 巽
    TianGan::Wu,   // 坎
    TianGan::Bing, // 艮
    TianGan::Yi,   // 坤
];

/// 外卦天干
/// 乾纳壬，坤纳癸，其他同内卦
pub const OUTER_GAN: [TianGan; 8] = [
    TianGan::Ren,  // 乾
    TianGan::Ding, // 兑
    TianGan::Ji,   // 离
    TianGan::Geng, // 震
    TianGan::Xin,  // 巽
    TianGan::Wu,   // 坎
    TianGan::Bing, // 艮
    TianGan::Gui,  // 坤
];

/// 八卦纳支起点和方向
/// 格式: (起始地支索引, 是否顺行)
/// 乾：子寅辰午申戌（顺行偶数）
/// 兑：巳卯丑亥酉未（逆行奇数）
/// 离：卯丑亥酉未巳（逆行奇数）
/// 震：子寅辰午申戌（顺行偶数）
/// 巽：丑亥酉未巳卯（逆行奇数）
/// 坎：寅辰午申戌子（顺行偶数）
/// 艮：辰午申戌子寅（顺行偶数）
/// 坤：未巳卯丑亥酉（逆行奇数）
pub const ZHI_START: [(u8, bool); 8] = [
    (0, true),   // 乾：子起，顺行+2
    (5, false),  // 兑：巳起，逆行-2
    (3, false),  // 离：卯起，逆行-2
    (0, true),   // 震：子起，顺行+2
    (1, false),  // 巽：丑起，逆行-2
    (2, true),   // 坎：寅起，顺行+2
    (4, true),   // 艮：辰起，顺行+2
    (7, false),  // 坤：未起，逆行-2
];

// ============================================================================
// 纳甲计算
// ============================================================================

/// 获取内卦某一爻的纳甲
/// trigram: 经卦
/// pos: 爻位（0=初爻, 1=二爻, 2=三爻）
pub fn get_inner_najia(trigram: Trigram, pos: u8) -> (TianGan, DiZhi) {
    let idx = trigram.index() as usize;
    let gan = INNER_GAN[idx];
    let (start, forward) = ZHI_START[idx];

    let zhi_idx = if forward {
        (start + pos * 2) % 12
    } else {
        (start + 12 - pos * 2) % 12
    };

    (gan, DiZhi::from_index(zhi_idx))
}

/// 获取外卦某一爻的纳甲
/// trigram: 经卦
/// pos: 爻位（0=四爻, 1=五爻, 2=上爻，对应内部位置0,1,2）
pub fn get_outer_najia(trigram: Trigram, pos: u8) -> (TianGan, DiZhi) {
    let idx = trigram.index() as usize;
    let gan = OUTER_GAN[idx];
    let (start, forward) = ZHI_START[idx];

    // 外卦地支从第四位开始
    let zhi_idx = if forward {
        (start + (pos + 3) * 2) % 12
    } else {
        (start + 12 - (pos + 3) * 2) % 12
    };

    (gan, DiZhi::from_index(zhi_idx))
}

// ============================================================================
// 卦宫与世应计算
// ============================================================================

/// 计算卦的世应和卦宫
///
/// 寻世诀：
/// 天同二世天变五，地同四世地变初。
/// 本宫六世三世异，人同游魂人变归。
///
/// 认宫诀：
/// 一二三六外卦宫，四五游魂内变更。
/// 若问归魂何所取，归魂内卦是本宫。
pub fn calculate_shi_ying_gong(inner: Trigram, outer: Trigram) -> (GuaXu, Trigram) {
    let inner_bin = inner.binary();
    let outer_bin = outer.binary();

    // 比较内外卦各爻
    let di_tong = (inner_bin & 0b001) == (outer_bin & 0b001);  // 初爻（地）
    let ren_tong = (inner_bin & 0b010) == (outer_bin & 0b010); // 二爻（人）
    let tian_tong = (inner_bin & 0b100) == (outer_bin & 0b100); // 三爻（天）

    let gua_xu;
    let gong;

    if inner_bin == outer_bin {
        // 本宫六世（纯卦）
        gua_xu = GuaXu::BenGong;
        gong = inner;
    } else if tian_tong && !ren_tong && !di_tong {
        // 天同二世
        gua_xu = GuaXu::ErShi;
        gong = outer;
    } else if !tian_tong && ren_tong && di_tong {
        // 天变五
        gua_xu = GuaXu::WuShi;
        gong = outer;
    } else if di_tong && !ren_tong && !tian_tong {
        // 地同四世
        gua_xu = GuaXu::SiShi;
        gong = Trigram::from_binary(inner_bin ^ 0b111);
    } else if !di_tong && ren_tong && tian_tong {
        // 地变初
        gua_xu = GuaXu::YiShi;
        gong = outer;
    } else if ren_tong && !tian_tong && !di_tong {
        // 人同游魂
        gua_xu = GuaXu::YouHun;
        gong = Trigram::from_binary(inner_bin ^ 0b111);
    } else if !ren_tong && tian_tong && di_tong {
        // 人变归魂
        gua_xu = GuaXu::GuiHun;
        gong = inner;
    } else {
        // 三世异
        gua_xu = GuaXu::SanShi;
        gong = outer;
    }

    (gua_xu, gong)
}

// ============================================================================
// 六神排布
// ============================================================================

/// 根据日干排六神
///
/// 六神配日干口诀：
/// 甲乙日起青龙，丙丁日起朱雀，
/// 戊日起勾陈，己日起螣蛇，
/// 庚辛日起白虎，壬癸日起玄武。
pub fn calculate_liu_shen(day_gan: TianGan) -> [LiuShen; 6] {
    let start = match day_gan {
        TianGan::Jia | TianGan::Yi => 0,       // 青龙
        TianGan::Bing | TianGan::Ding => 1,    // 朱雀
        TianGan::Wu => 2,                       // 勾陈
        TianGan::Ji => 3,                       // 螣蛇
        TianGan::Geng | TianGan::Xin => 4,     // 白虎
        TianGan::Ren | TianGan::Gui => 5,      // 玄武
    };

    let mut result = [LiuShen::QingLong; 6];
    for i in 0..6 {
        result[i] = LiuShen::from_index((start + i as u8) % 6);
    }
    result
}

// ============================================================================
// 旬空计算
// ============================================================================

/// 计算日旬空
///
/// 六十甲子分六旬，每旬十天，缺两个地支为空亡
pub fn calculate_xun_kong(day_gan: TianGan, day_zhi: DiZhi) -> (DiZhi, DiZhi) {
    let gan_idx = day_gan.index() as i8;
    let zhi_idx = day_zhi.index() as i8;

    // 计算旬首的地支索引
    // 旬首是甲X，从当前日期反推甲日的地支
    let mut xun_zhi = (zhi_idx - gan_idx + 12) % 12;
    if xun_zhi < 0 {
        xun_zhi += 12;
    }

    // 空亡是该旬缺失的两个地支
    // 每旬10天，地支12个，所以缺2个
    // 空亡地支 = 旬首地支 + 10 和 + 11
    let kong1 = DiZhi::from_index(((xun_zhi + 10) % 12) as u8);
    let kong2 = DiZhi::from_index(((xun_zhi + 11) % 12) as u8);

    (kong1, kong2)
}

// ============================================================================
// 伏神计算
// ============================================================================

/// 查找伏神
///
/// 当本卦六亲不全时，缺失的六亲从本宫纯卦中寻找伏神
pub fn find_fu_shen(
    gong: Trigram,
    original_liu_qin: &[LiuQin; 6],
) -> [Option<FuShenInfo>; 6] {
    let mut result: [Option<FuShenInfo>; 6] = [None, None, None, None, None, None];

    // 统计本卦已有的六亲
    let mut has_liu_qin = [false; 5];
    for qin in original_liu_qin.iter() {
        has_liu_qin[*qin as usize] = true;
    }

    // 如果五个六亲都有，则无伏神
    if has_liu_qin.iter().all(|&x| x) {
        return result;
    }

    // 获取本宫纯卦的纳甲
    let gong_wx = gong.wu_xing();

    for i in 0..6 {
        let (gan, zhi) = if i < 3 {
            get_inner_najia(gong, i as u8)
        } else {
            get_outer_najia(gong, (i - 3) as u8)
        };

        let yao_wx = zhi.wu_xing();
        let qin = LiuQin::from_wu_xing(gong_wx, yao_wx);

        // 如果这个六亲在本卦中缺失，则记录为伏神
        if !has_liu_qin[qin as usize] {
            result[i] = Some(FuShenInfo {
                position: i as u8,
                liu_qin: qin,
                tian_gan: gan,
                di_zhi: zhi,
                wu_xing: yao_wx,
            });
            has_liu_qin[qin as usize] = true; // 标记为已找到
        }
    }

    result
}

// ============================================================================
// 变卦计算
// ============================================================================

/// 计算变卦
/// 将动爻变化后得到变卦
pub fn calculate_bian_gua(original_yaos: &[Yao; 6]) -> (Trigram, Trigram, bool) {
    let mut has_bian = false;
    let mut inner_bin: u8 = 0;
    let mut outer_bin: u8 = 0;

    for i in 0..6 {
        let changed_val = original_yaos[i].changed_value();
        if original_yaos[i].is_moving() {
            has_bian = true;
        }

        if i < 3 {
            inner_bin |= changed_val << i;
        } else {
            outer_bin |= changed_val << (i - 3);
        }
    }

    (Trigram::from_binary(inner_bin), Trigram::from_binary(outer_bin), has_bian)
}

// ============================================================================
// 六十四卦索引计算
// ============================================================================

/// 从六爻计算内外卦
pub fn yaos_to_trigrams(yaos: &[Yao; 6]) -> (Trigram, Trigram) {
    let mut inner_bin: u8 = 0;
    let mut outer_bin: u8 = 0;

    for i in 0..3 {
        inner_bin |= yaos[i].original_value() << i;
    }
    for i in 3..6 {
        outer_bin |= yaos[i].original_value() << (i - 3);
    }

    (Trigram::from_binary(inner_bin), Trigram::from_binary(outer_bin))
}

/// 计算六十四卦索引
pub fn calculate_gua_index(inner: Trigram, outer: Trigram) -> u8 {
    // 使用内外卦的二进制值组合
    // 外卦在高3位，内卦在低3位
    let inner_idx = inner.binary();
    let outer_idx = outer.binary();
    (outer_idx << 3) | inner_idx
}

/// 计算互卦
///
/// 互卦取本卦的2、3、4爻为下卦（内卦），3、4、5爻为上卦（外卦）
/// 爻位从0开始（初爻=0）
///
/// # 参数
/// - `original_yaos`: 本卦六爻数组
///
/// # 返回
/// (互卦内卦, 互卦外卦)
pub fn calculate_hu_gua(original_yaos: &[Yao; 6]) -> (Trigram, Trigram) {
    // 内卦取2,3,4爻（索引1,2,3）
    let inner_bin = (original_yaos[1].original_value()) |
                    (original_yaos[2].original_value() << 1) |
                    (original_yaos[3].original_value() << 2);
    // 外卦取3,4,5爻（索引2,3,4）
    let outer_bin = (original_yaos[2].original_value()) |
                    (original_yaos[3].original_value() << 1) |
                    (original_yaos[4].original_value() << 2);

    (Trigram::from_binary(inner_bin), Trigram::from_binary(outer_bin))
}

/// 计算互卦索引
pub fn calculate_hu_gua_index(original_yaos: &[Yao; 6]) -> u8 {
    let (inner, outer) = calculate_hu_gua(original_yaos);
    calculate_gua_index(inner, outer)
}

// ============================================================================
// 卦身计算
// ============================================================================

/// 计算卦身
///
/// 卦身是根据世爻的阴阳和位置确定的一个地支，用于断事。
///
/// # 口诀
/// - 世爻为阳爻：从子起数到世爻位置
/// - 世爻为阴爻：从午起数到世爻位置
///
/// # 参数
/// - `shi_pos`: 世爻位置（1-6）
/// - `shi_is_yang`: 世爻是否为阳爻
///
/// # 返回
/// 卦身地支
pub fn calculate_gua_shen(shi_pos: u8, shi_is_yang: bool) -> DiZhi {
    // 阳爻从子(0)起，阴爻从午(6)起
    let start = if shi_is_yang { 0 } else { 6 };
    DiZhi::from_index((start + shi_pos - 1) % 12)
}

// ============================================================================
// 起卦方法
// ============================================================================

/// 从铜钱结果创建六爻
/// counts: 六次摇卦结果，每个值为阳面个数(0-3)
pub fn coins_to_yaos(counts: &[u8; 6]) -> [Yao; 6] {
    let mut yaos = [Yao::ShaoYin; 6];
    for i in 0..6 {
        yaos[i] = Yao::from_coin_count(counts[i]);
    }
    yaos
}

/// 从两个数字起卦（报数法）
///
/// # 参数
/// - `upper_num`: 上卦数（对应外卦，用户报的第一个数）
/// - `lower_num`: 下卦数（对应内卦，用户报的第二个数）
/// - `dong`: 动爻位置（1-6，从初爻到上爻）
///
/// # 算法
/// - 上卦数除8取余确定外卦
/// - 下卦数除8取余确定内卦
/// - 动爻位置处的爻为老阴或老阳
pub fn numbers_to_yaos(upper_num: u16, lower_num: u16, dong: u8) -> [Yao; 6] {
    let inner_idx = ((lower_num - 1) % 8) as u8;
    let outer_idx = ((upper_num - 1) % 8) as u8;

    let inner = Trigram::from_index(inner_idx);
    let outer = Trigram::from_index(outer_idx);

    let inner_bin = inner.binary();
    let outer_bin = outer.binary();

    let dong_pos = ((dong - 1) % 6) as usize;

    let mut yaos = [Yao::ShaoYin; 6];
    for i in 0..6 {
        let is_yang = if i < 3 {
            (inner_bin >> i) & 1 == 1
        } else {
            (outer_bin >> (i - 3)) & 1 == 1
        };

        if i == dong_pos {
            // 动爻
            yaos[i] = if is_yang { Yao::LaoYang } else { Yao::LaoYin };
        } else {
            // 静爻
            yaos[i] = if is_yang { Yao::ShaoYang } else { Yao::ShaoYin };
        }
    }

    yaos
}

/// 从随机数创建六爻
pub fn random_to_yaos(random_bytes: &[u8; 32]) -> [Yao; 6] {
    let mut yaos = [Yao::ShaoYin; 6];
    for i in 0..6 {
        // 每4个字节生成一个铜钱结果
        let coin_sum = (random_bytes[i * 4] as u16
            + random_bytes[i * 4 + 1] as u16
            + random_bytes[i * 4 + 2] as u16) % 4;
        yaos[i] = Yao::from_coin_count(coin_sum as u8);
    }
    yaos
}

/// 从时间起卦（梅花易数时间起卦法）
///
/// # 参数
/// - `year_zhi`: 年地支索引 (0-11，子=0)
/// - `month_num`: 农历月数 (1-12)
/// - `day_num`: 农历日数 (1-30)
/// - `hour_zhi`: 时辰地支索引 (0-11，子时=0)
///
/// # 算法
/// - 上卦 = (年支 + 月 + 日) % 8
/// - 下卦 = (年支 + 月 + 日 + 时支) % 8
/// - 动爻 = (年支 + 月 + 日 + 时支) % 6 + 1
///
/// 注：年支和时支使用地支序数（子=1，丑=2...亥=12）
pub fn time_to_yaos(year_zhi: u8, month_num: u8, day_num: u8, hour_zhi: u8) -> [Yao; 6] {
    // 地支序数从1开始（子=1，丑=2...）
    let year_num = (year_zhi + 1) as u16;
    let hour_num = (hour_zhi + 1) as u16;
    let month = month_num as u16;
    let day = day_num as u16;

    // 年月日之和定上卦（外卦）
    let upper_sum = year_num + month + day;
    let upper_idx = if upper_sum % 8 == 0 { 8 } else { upper_sum % 8 };

    // 年月日时之和定下卦（内卦）
    let lower_sum = year_num + month + day + hour_num;
    let lower_idx = if lower_sum % 8 == 0 { 8 } else { lower_sum % 8 };

    // 总和定动爻
    let dong_sum = year_num + month + day + hour_num;
    let dong = if dong_sum % 6 == 0 { 6 } else { (dong_sum % 6) as u8 };

    numbers_to_yaos(upper_idx, lower_idx, dong)
}

// ============================================================================
// 动爻位图
// ============================================================================

/// 计算动爻位图
pub fn calculate_moving_bitmap(yaos: &[Yao; 6]) -> u8 {
    let mut bitmap: u8 = 0;
    for i in 0..6 {
        if yaos[i].is_moving() {
            bitmap |= 1 << i;
        }
    }
    bitmap
}

// ============================================================================
// 卦类型判断
// ============================================================================

/// 六冲卦索引列表
///
/// 六冲卦包括八纯卦（内外卦相同）和天雷无妄、雷天大壮
/// 六冲主散，事多阻隔
pub const LIU_CHONG_INDICES: [u8; 10] = [
    0,   // 坤为地
    9,   // 艮为山
    18,  // 坎为水
    27,  // 巽为风
    36,  // 震为雷
    45,  // 离为火
    54,  // 兑为泽
    63,  // 乾为天
    60,  // 天雷无妄（乾震）
    39,  // 雷天大壮（震乾）
];

/// 六合卦索引列表
///
/// 六合卦主合，事多和顺
/// 包括：否、泰、困、节、贲、复、旅、豫
pub const LIU_HE_INDICES: [u8; 8] = [
    56,  // 天地否
    7,   // 地天泰
    50,  // 泽水困
    22,  // 水泽节
    13,  // 山火贲
    4,   // 地雷复
    41,  // 火山旅
    32,  // 雷地豫
];

/// 判断是否为六冲卦（按卦象索引）
///
/// # 参数
/// - `gua_index`: 六十四卦索引 (0-63)
pub fn is_liu_chong_by_index(gua_index: u8) -> bool {
    LIU_CHONG_INDICES.contains(&gua_index)
}

/// 判断是否为六冲卦（按内外卦）
///
/// # 参数
/// - `inner`: 内卦
/// - `outer`: 外卦
pub fn is_liu_chong(inner: Trigram, outer: Trigram) -> bool {
    let index = calculate_gua_index(inner, outer);
    is_liu_chong_by_index(index)
}

/// 判断是否为六合卦
///
/// # 参数
/// - `gua_index`: 六十四卦索引 (0-63)
pub fn is_liu_he(gua_index: u8) -> bool {
    LIU_HE_INDICES.contains(&gua_index)
}

// ============================================================================
// 月令旺衰计算
// ============================================================================

/// 五行旺衰状态
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WangShuai {
    /// 旺 - 当令，最强
    Wang = 0,
    /// 相 - 得令生，次强
    Xiang = 1,
    /// 休 - 休息，力弱
    Xiu = 2,
    /// 囚 - 被克，较弱
    Qiu = 3,
    /// 死 - 克令，最弱
    Si = 4,
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

    /// 是否有力
    pub fn is_strong(&self) -> bool {
        matches!(self, Self::Wang | Self::Xiang)
    }

    /// 是否无力
    pub fn is_weak(&self) -> bool {
        matches!(self, Self::Xiu | Self::Qiu | Self::Si)
    }
}

/// 月支对应的五行（月令）
///
/// 寅卯月木旺，巳午月火旺，申酉月金旺，亥子月水旺，
/// 辰戌丑未月土旺
fn month_zhi_to_wu_xing(month_zhi: DiZhi) -> WuXing {
    match month_zhi {
        DiZhi::Yin | DiZhi::Mao => WuXing::Wood,
        DiZhi::Si | DiZhi::Wu => WuXing::Fire,
        DiZhi::Shen | DiZhi::You => WuXing::Metal,
        DiZhi::Hai | DiZhi::Zi => WuXing::Water,
        DiZhi::Chen | DiZhi::Xu | DiZhi::Chou | DiZhi::Wei => WuXing::Earth,
    }
}

/// 计算爻的旺衰状态
///
/// # 五行旺衰规则
/// - 旺：与月令五行相同
/// - 相：被月令五行所生
/// - 休：生月令五行者
/// - 囚：克月令五行者
/// - 死：被月令五行所克
///
/// # 参数
/// - `yao_wx`: 爻的五行
/// - `month_zhi`: 月支
///
/// # 返回
/// 旺衰状态
pub fn calculate_wang_shuai(yao_wx: WuXing, month_zhi: DiZhi) -> WangShuai {
    let month_wx = month_zhi_to_wu_xing(month_zhi);

    if yao_wx == month_wx {
        // 同我者旺
        WangShuai::Wang
    } else if month_wx.generates() == yao_wx {
        // 月令生我者相
        WangShuai::Xiang
    } else if yao_wx.generates() == month_wx {
        // 我生月令者休
        WangShuai::Xiu
    } else if yao_wx.restrains() == month_wx {
        // 我克月令者囚
        WangShuai::Qiu
    } else {
        // 月令克我者死
        WangShuai::Si
    }
}

// ============================================================================
// 日辰冲合分析
// ============================================================================

/// 地支六冲对照表
/// 子午冲、丑未冲、寅申冲、卯酉冲、辰戌冲、巳亥冲
const DI_ZHI_CHONG: [(DiZhi, DiZhi); 6] = [
    (DiZhi::Zi, DiZhi::Wu),
    (DiZhi::Chou, DiZhi::Wei),
    (DiZhi::Yin, DiZhi::Shen),
    (DiZhi::Mao, DiZhi::You),
    (DiZhi::Chen, DiZhi::Xu),
    (DiZhi::Si, DiZhi::Hai),
];

/// 地支六合对照表
/// 子丑合土、寅亥合木、卯戌合火、辰酉合金、巳申合水、午未合火
const DI_ZHI_HE: [(DiZhi, DiZhi, WuXing); 6] = [
    (DiZhi::Zi, DiZhi::Chou, WuXing::Earth),
    (DiZhi::Yin, DiZhi::Hai, WuXing::Wood),
    (DiZhi::Mao, DiZhi::Xu, WuXing::Fire),
    (DiZhi::Chen, DiZhi::You, WuXing::Metal),
    (DiZhi::Si, DiZhi::Shen, WuXing::Water),
    (DiZhi::Wu, DiZhi::Wei, WuXing::Fire),
];

/// 判断两个地支是否相冲
pub fn is_di_zhi_chong(zhi1: DiZhi, zhi2: DiZhi) -> bool {
    for (a, b) in DI_ZHI_CHONG.iter() {
        if (*a == zhi1 && *b == zhi2) || (*a == zhi2 && *b == zhi1) {
            return true;
        }
    }
    false
}

/// 判断两个地支是否相合，如果合则返回合化的五行
pub fn is_di_zhi_he(zhi1: DiZhi, zhi2: DiZhi) -> Option<WuXing> {
    for (a, b, wx) in DI_ZHI_HE.iter() {
        if (*a == zhi1 && *b == zhi2) || (*a == zhi2 && *b == zhi1) {
            return Some(*wx);
        }
    }
    None
}

/// 获取地支的冲支
pub fn get_chong_zhi(zhi: DiZhi) -> DiZhi {
    // 冲支是相差6位
    DiZhi::from_index((zhi.index() + 6) % 12)
}

/// 获取地支的合支
pub fn get_he_zhi(zhi: DiZhi) -> DiZhi {
    match zhi {
        DiZhi::Zi => DiZhi::Chou,
        DiZhi::Chou => DiZhi::Zi,
        DiZhi::Yin => DiZhi::Hai,
        DiZhi::Mao => DiZhi::Xu,
        DiZhi::Chen => DiZhi::You,
        DiZhi::Si => DiZhi::Shen,
        DiZhi::Wu => DiZhi::Wei,
        DiZhi::Wei => DiZhi::Wu,
        DiZhi::Shen => DiZhi::Si,
        DiZhi::You => DiZhi::Chen,
        DiZhi::Xu => DiZhi::Mao,
        DiZhi::Hai => DiZhi::Yin,
    }
}

/// 日辰与爻的关系
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RiChenGuanXi {
    /// 无特殊关系
    None,
    /// 日辰冲爻（暗动或冲散）
    RiChong,
    /// 日辰合爻（合住或合起）
    RiHe,
    /// 日辰生爻
    RiSheng,
    /// 日辰克爻
    RiKe,
    /// 爻生日辰（泄气）
    XieQi,
    /// 爻克日辰（耗气）
    HaoQi,
}

/// 分析日辰与爻的关系
///
/// # 参数
/// - `day_zhi`: 日支
/// - `yao_zhi`: 爻的地支
/// - `yao_wx`: 爻的五行
///
/// # 返回
/// 日辰与爻的关系
pub fn analyze_ri_chen(day_zhi: DiZhi, yao_zhi: DiZhi, yao_wx: WuXing) -> RiChenGuanXi {
    let day_wx = day_zhi.wu_xing();

    // 先看冲合
    if is_di_zhi_chong(day_zhi, yao_zhi) {
        return RiChenGuanXi::RiChong;
    }
    if is_di_zhi_he(day_zhi, yao_zhi).is_some() {
        return RiChenGuanXi::RiHe;
    }

    // 再看生克
    if day_wx.generates() == yao_wx {
        RiChenGuanXi::RiSheng
    } else if day_wx.restrains() == yao_wx {
        RiChenGuanXi::RiKe
    } else if yao_wx.generates() == day_wx {
        RiChenGuanXi::XieQi
    } else if yao_wx.restrains() == day_wx {
        RiChenGuanXi::HaoQi
    } else {
        RiChenGuanXi::None
    }
}

// ============================================================================
// 动爻作用关系
// ============================================================================

/// 动爻对静爻的作用类型
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DongYaoZuoYong {
    /// 动生静
    DongShengJing,
    /// 动克静
    DongKeJing,
    /// 动泄静（静生动）
    DongXieJing,
    /// 动耗静（静克动）
    DongHaoJing,
    /// 比和（同五行）
    BiHe,
    /// 无作用
    None,
}

/// 计算动爻对静爻的作用
///
/// # 参数
/// - `dong_wx`: 动爻五行
/// - `jing_wx`: 静爻五行
pub fn calculate_dong_jing_zuoyong(dong_wx: WuXing, jing_wx: WuXing) -> DongYaoZuoYong {
    if dong_wx == jing_wx {
        DongYaoZuoYong::BiHe
    } else if dong_wx.generates() == jing_wx {
        DongYaoZuoYong::DongShengJing
    } else if dong_wx.restrains() == jing_wx {
        DongYaoZuoYong::DongKeJing
    } else if jing_wx.generates() == dong_wx {
        DongYaoZuoYong::DongXieJing
    } else if jing_wx.restrains() == dong_wx {
        DongYaoZuoYong::DongHaoJing
    } else {
        DongYaoZuoYong::None
    }
}

/// 变爻回头生克类型
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HuiTouZuoYong {
    /// 回头生（变爻生本爻）
    HuiTouSheng,
    /// 回头克（变爻克本爻）
    HuiTouKe,
    /// 回头泄（本爻生变爻）
    HuiTouXie,
    /// 回头耗（本爻克变爻）
    HuiTouHao,
    /// 比和
    BiHe,
}

/// 计算变爻回头生克
///
/// # 参数
/// - `ben_wx`: 本爻五行
/// - `bian_wx`: 变爻五行
pub fn calculate_hui_tou(ben_wx: WuXing, bian_wx: WuXing) -> HuiTouZuoYong {
    if ben_wx == bian_wx {
        HuiTouZuoYong::BiHe
    } else if bian_wx.generates() == ben_wx {
        HuiTouZuoYong::HuiTouSheng
    } else if bian_wx.restrains() == ben_wx {
        HuiTouZuoYong::HuiTouKe
    } else if ben_wx.generates() == bian_wx {
        HuiTouZuoYong::HuiTouXie
    } else {
        HuiTouZuoYong::HuiTouHao
    }
}

// ============================================================================
// 反吟伏吟
// ============================================================================

/// 判断是否为反吟卦（本卦与变卦六冲）
///
/// 反吟主反复、变动、不顺
pub fn is_fan_yin(ben_inner: Trigram, ben_outer: Trigram, bian_inner: Trigram, bian_outer: Trigram) -> bool {
    // 本卦与变卦内外卦都相冲
    // 简化判断：内外卦都相冲（二进制异或为0b111）
    ben_inner.binary() ^ bian_inner.binary() == 0b111 &&
    ben_outer.binary() ^ bian_outer.binary() == 0b111
}

/// 判断是否为伏吟卦（本卦与变卦相同）
///
/// 伏吟主停滞、拖延、呻吟
pub fn is_fu_yin(ben_inner: Trigram, ben_outer: Trigram, bian_inner: Trigram, bian_outer: Trigram) -> bool {
    ben_inner == bian_inner && ben_outer == bian_outer
}

// ============================================================================
// 床帐香闺（婚姻用神）
// ============================================================================

/// 计算床帐（卦身所生的地支）
///
/// 床帐为卦身所生之支，用于婚姻占卜
///
/// # 参数
/// - `gua_shen`: 卦身地支
///
/// # 返回
/// 床帐地支列表（被卦身五行所生的地支）
pub fn calculate_chuang_zhang(gua_shen: DiZhi) -> [DiZhi; 2] {
    let gua_shen_wx = gua_shen.wu_xing();
    let sheng_wx = gua_shen_wx.generates();

    // 找出该五行对应的地支
    let mut result = [DiZhi::Zi; 2];
    let mut idx = 0;

    for i in 0..12 {
        let zhi = DiZhi::from_index(i);
        if zhi.wu_xing() == sheng_wx && idx < 2 {
            result[idx] = zhi;
            idx += 1;
        }
    }

    result
}

/// 计算香闺（卦身所克的地支）
///
/// 香闺为卦身所克之支，用于婚姻占卜
///
/// # 参数
/// - `gua_shen`: 卦身地支
///
/// # 返回
/// 香闺地支列表（被卦身五行所克的地支）
pub fn calculate_xiang_gui(gua_shen: DiZhi) -> [DiZhi; 2] {
    let gua_shen_wx = gua_shen.wu_xing();
    let ke_wx = gua_shen_wx.restrains();

    // 找出该五行对应的地支
    let mut result = [DiZhi::Zi; 2];
    let mut idx = 0;

    for i in 0..12 {
        let zhi = DiZhi::from_index(i);
        if zhi.wu_xing() == ke_wx && idx < 2 {
            result[idx] = zhi;
            idx += 1;
        }
    }

    result
}
