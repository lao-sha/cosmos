//! # 八字排盘常量定义
//!
//! 本模块包含八字排盘所需的所有常量表，包括：
//! - 藏干表：12地支的藏干数据（✅ 确认辰藏干为"癸水"）
//! - 纳音表：60甲子对应的30种纳音五行
//! - 十神查表：10×10十神关系矩阵
//! - 权重表：月令旺衰权重矩阵（12×36）

use crate::types::*;

// ================================
// 藏干常量表
// ================================

/// 12地支藏干表 (主流派标准)
///
/// ⚠️ 关键确认：辰藏干为"戊乙癸"（使用癸水，不是壬水！）
///
/// 参考依据：
/// - BaziGo: cangganlist[4] = {4, 1, 9} (戊乙癸)
/// - lunar-java: 辰藏"戊乙癸"
/// - tyme4ts: 亥藏"壬甲"
/// - 主流派支持率: 87.5% (7/8项目)
///
/// 格式：[主气天干索引, 中气天干索引, 余气天干索引]
/// - 255(0xFF)表示该位置无藏干
/// - 索引对应：甲(0) 乙(1) 丙(2) 丁(3) 戊(4) 己(5) 庚(6) 辛(7) 壬(8) 癸(9)
///
/// 藏干数量：子(1) 丑(3) 寅(3) 卯(1) 辰(3) 巳(3) 午(2) 未(3) 申(3) 酉(1) 戌(3) 亥(2)
pub const EARTHLY_HIDDEN_STEMS: [[u8; 3]; 12] = [
	[9, 255, 255],     // 子: 癸(主气)
	[5, 9, 7],         // 丑: 己(主) 癸(中) 辛(余)
	[0, 2, 4],         // 寅: 甲(主) 丙(中) 戊(余)
	[1, 255, 255],     // 卯: 乙(主气)
	[4, 1, 9],         // 辰: 戊(主) 乙(中) 癸(余) ✅
	[2, 6, 4],         // 巳: 丙(主) 庚(中) 戊(余)
	[3, 5, 255],       // 午: 丁(主) 己(中)
	[5, 3, 1],         // 未: 己(主) 丁(中) 乙(余)
	[6, 8, 4],         // 申: 庚(主) 壬(中) 戊(余)
	[7, 255, 255],     // 酉: 辛(主气)
	[4, 7, 3],         // 戌: 戊(主) 辛(中) 丁(余)
	[8, 0, 255],       // 亥: 壬(主) 甲(中) ✅ 修复：甲是0不是无效值
];

/// 藏干类型表（主气/中气/余气）
///
/// 与 EARTHLY_HIDDEN_STEMS 对应，标识每个藏干的类型
/// 注意：当对应位置藏干为255时，此表的类型值无意义
pub const CANGGAN_TYPE_TABLE: [[CangGanType; 3]; 12] = [
	[CangGanType::ZhuQi, CangGanType::ZhuQi, CangGanType::ZhuQi],     // 子: 癸(主气)
	[CangGanType::ZhuQi, CangGanType::ZhongQi, CangGanType::YuQi],    // 丑: 己(主) 癸(中) 辛(余)
	[CangGanType::ZhuQi, CangGanType::ZhongQi, CangGanType::YuQi],    // 寅: 甲(主) 丙(中) 戊(余)
	[CangGanType::ZhuQi, CangGanType::ZhuQi, CangGanType::ZhuQi],     // 卯: 乙(主气)
	[CangGanType::ZhuQi, CangGanType::ZhongQi, CangGanType::YuQi],    // 辰: 戊(主) 乙(中) 癸(余)
	[CangGanType::ZhuQi, CangGanType::ZhongQi, CangGanType::YuQi],    // 巳: 丙(主) 庚(中) 戊(余)
	[CangGanType::ZhuQi, CangGanType::ZhongQi, CangGanType::ZhuQi],   // 午: 丁(主) 己(中)
	[CangGanType::ZhuQi, CangGanType::ZhongQi, CangGanType::YuQi],    // 未: 己(主) 丁(中) 乙(余)
	[CangGanType::ZhuQi, CangGanType::ZhongQi, CangGanType::YuQi],    // 申: 庚(主) 壬(中) 戊(余)
	[CangGanType::ZhuQi, CangGanType::ZhuQi, CangGanType::ZhuQi],     // 酉: 辛(主气)
	[CangGanType::ZhuQi, CangGanType::ZhongQi, CangGanType::YuQi],    // 戌: 戊(主) 辛(中) 丁(余)
	[CangGanType::ZhuQi, CangGanType::ZhongQi, CangGanType::ZhuQi],   // 亥: 壬(主) 甲(中) ✅ 修复
];

/// 藏干基础权重表
///
/// 简化版权重，用于基础五行强度计算
/// 格式：[主气权重, 中气权重, 余气权重]
pub const CANGGAN_BASE_WEIGHT: [[u16; 3]; 12] = [
	[1000, 0, 0],       // 子: 癸(1000)
	[500, 300, 200],    // 丑: 己(500) 癸(300) 辛(200)
	[800, 360, 0],      // 寅: 甲(800) 丙(360) 戊(0)
	[1000, 0, 0],       // 卯: 乙(1000)
	[500, 300, 200],    // 辰: 戊(500) 乙(300) 癸(200)
	[800, 300, 200],    // 巳: 丙(800) 庚(300) 戊(200)
	[1000, 600, 0],     // 午: 丁(1000) 己(600)
	[800, 300, 200],    // 未: 己(800) 丁(300) 乙(200)
	[800, 400, 200],    // 申: 庚(800) 壬(400) 戊(200)
	[1000, 0, 0],       // 酉: 辛(1000)
	[800, 300, 200],    // 戌: 戊(800) 辛(300) 丁(200)
	[800, 400, 0],      // 亥: 壬(800) 甲(400)
];

// ================================
// 纳音常量表
// ================================

/// 纳音五行表 (30种)
///
/// 索引对应：干支值 / 2
/// 例如：甲子(0) → 0/2=0 → 海中金
///       乙丑(1) → 1/2=0 → 海中金
///       丙寅(2) → 2/2=1 → 炉中火
pub const NAYIN_TABLE: [NaYin; 30] = [
	NaYin::HaiZhongJin,    // 0: 甲子、乙丑 - 海中金
	NaYin::LuZhongHuo,     // 1: 丙寅、丁卯 - 炉中火
	NaYin::DaLinMu,        // 2: 戊辰、己巳 - 大林木
	NaYin::LuPangTu,       // 3: 庚午、辛未 - 路旁土
	NaYin::JianFengJin,    // 4: 壬申、癸酉 - 剑锋金
	NaYin::ShanTouHuo,     // 5: 甲戌、乙亥 - 山头火
	NaYin::JianXiaShui,    // 6: 丙子、丁丑 - 涧下水
	NaYin::ChengTouTu,     // 7: 戊寅、己卯 - 城头土
	NaYin::BaiLaJin,       // 8: 庚辰、辛巳 - 白蜡金
	NaYin::YangLiuMu,      // 9: 壬午、癸未 - 杨柳木
	NaYin::QuanZhongShui,  // 10: 甲申、乙酉 - 泉中水
	NaYin::WuShangTu,      // 11: 丙戌、丁亥 - 屋上土
	NaYin::PiLiHuo,        // 12: 戊子、己丑 - 霹雳火
	NaYin::SongBaiMu,      // 13: 庚寅、辛卯 - 松柏木
	NaYin::ChangLiuShui,   // 14: 壬辰、癸巳 - 长流水
	NaYin::ShaZhongJin,    // 15: 甲午、乙未 - 沙中金
	NaYin::ShanXiaHuo,     // 16: 丙申、丁酉 - 山下火
	NaYin::PingDiMu,       // 17: 戊戌、己亥 - 平地木
	NaYin::BiShangTu,      // 18: 庚子、辛丑 - 壁上土
	NaYin::JinBoJin,       // 19: 壬寅、癸卯 - 金箔金
	NaYin::FuDengHuo,      // 20: 甲辰、乙巳 - 覆灯火
	NaYin::TianHeShui,     // 21: 丙午、丁未 - 天河水
	NaYin::DaYiTu,         // 22: 戊申、己酉 - 大驿土
	NaYin::ChaiChuanJin,   // 23: 庚戌、辛亥 - 钗钏金
	NaYin::SangTuoMu,      // 24: 壬子、癸丑 - 桑柘木
	NaYin::DaXiShui,       // 25: 甲寅、乙卯 - 大溪水
	NaYin::ShaZhongTu,     // 26: 丙辰、丁巳 - 沙中土
	NaYin::TianShangHuo,   // 27: 戊午、己未 - 天上火
	NaYin::ShiLiuMu,       // 28: 庚申、辛酉 - 石榴木
	NaYin::DaHaiShui,      // 29: 壬戌、癸亥 - 大海水
];

// ================================
// 十神查表
// ================================

/// 十神查表 (10×10矩阵)
///
/// 格式：SHISHEN_TABLE[日主天干][其他天干] = 十神索引
///
/// 十神索引对应：
/// - 0: 比肩, 1: 劫财, 2: 食神, 3: 伤官, 4: 偏财
/// - 5: 正财, 6: 七杀, 7: 正官, 8: 偏印, 9: 正印
pub const SHISHEN_TABLE: [[u8; 10]; 10] = [
	[0, 1, 2, 3, 4, 5, 6, 7, 8, 9], // 甲为日主
	[1, 0, 3, 2, 5, 4, 7, 6, 9, 8], // 乙为日主
	[8, 9, 0, 1, 2, 3, 4, 5, 6, 7], // 丙为日主
	[9, 8, 1, 0, 3, 2, 5, 4, 7, 6], // 丁为日主
	[6, 7, 8, 9, 0, 1, 2, 3, 4, 5], // 戊为日主
	[7, 6, 9, 8, 1, 0, 3, 2, 5, 4], // 己为日主
	[4, 5, 6, 7, 8, 9, 0, 1, 2, 3], // 庚为日主
	[5, 4, 7, 6, 9, 8, 1, 0, 3, 2], // 辛为日主
	[2, 3, 4, 5, 6, 7, 8, 9, 0, 1], // 壬为日主
	[3, 2, 5, 4, 7, 6, 9, 8, 1, 0], // 癸为日主
];

/// 十神枚举数组（用于索引转换）
pub const SHISHEN_ARRAY: [ShiShen; 10] = [
	ShiShen::BiJian,      // 0
	ShiShen::JieCai,      // 1
	ShiShen::ShiShen,     // 2
	ShiShen::ShangGuan,   // 3
	ShiShen::PianCai,     // 4
	ShiShen::ZhengCai,    // 5
	ShiShen::QiSha,       // 6
	ShiShen::ZhengGuan,   // 7
	ShiShen::PianYin,     // 8
	ShiShen::ZhengYin,    // 9
];

// ================================
// 节气近似日期表
// ================================

/// 节气近似日期表（简化版）
///
/// 格式：[节气对应的公历月份, 近似日期]
///
/// ⚠️ 注意：这是简化版实现，精度为±1天
/// 生产环境建议使用寿星天文算法进行精确计算
pub const JIEQI_APPROX_DATES: [(u8, u8); 12] = [
	(1, 6),   // 小寒: 1月6日左右
	(2, 4),   // 立春: 2月4日左右
	(3, 6),   // 惊蛰: 3月6日左右
	(4, 5),   // 清明: 4月5日左右
	(5, 6),   // 立夏: 5月6日左右
	(6, 6),   // 芒种: 6月6日左右
	(7, 7),   // 小暑: 7月7日左右
	(8, 8),   // 立秋: 8月8日左右
	(9, 8),   // 白露: 9月8日左右
	(10, 8),  // 寒露: 10月8日左右
	(11, 7),  // 立冬: 11月7日左右
	(12, 7),  // 大雪: 12月7日左右
];

// ================================
// 天干强度表（12月×10天干）
// ================================

/// 天干强度表 (12月×10天干矩阵)
///
/// 参考 BaziGo 的 tianganqiangdulist 实现
///
/// 格式：TIANGAN_STRENGTH[月支索引][天干索引]
/// 月支索引：子(0) 丑(1) 寅(2) 卯(3) 辰(4) 巳(5) 午(6) 未(7) 申(8) 酉(9) 戌(10) 亥(11)
/// 天干索引：甲(0) 乙(1) 丙(2) 丁(3) 戊(4) 己(5) 庚(6) 辛(7) 壬(8) 癸(9)
///
/// 权重说明：
/// - 1200: 当令最旺
/// - 1140-1160: 次旺
/// - 1100: 相旺
/// - 1000: 基础值（休囚）
/// - 1040-1060: 略旺
pub const TIANGAN_STRENGTH: [[u16; 10]; 12] = [
	// 甲    乙     丙     丁     戊     己     庚     辛     壬     癸
	[1200, 1200, 1000, 1000, 1000, 1000, 1000, 1000, 1200, 1200], // 子月 - 水旺木相
	[1060, 1060, 1000, 1000, 1100, 1100, 1140, 1140, 1100, 1100], // 丑月 - 土旺金相
	[1140, 1140, 1200, 1200, 1060, 1060, 1000, 1000, 1000, 1000], // 寅月 - 木旺火相
	[1200, 1200, 1200, 1200, 1000, 1000, 1000, 1000, 1000, 1000], // 卯月 - 木旺火相
	[1100, 1100, 1060, 1060, 1100, 1100, 1100, 1100, 1040, 1040], // 辰月 - 土旺金相
	[1000, 1000, 1140, 1140, 1140, 1140, 1060, 1060, 1060, 1060], // 巳月 - 火旺土相
	[1000, 1000, 1200, 1200, 1200, 1200, 1000, 1000, 1000, 1000], // 午月 - 火旺土相
	[1040, 1040, 1100, 1100, 1160, 1160, 1100, 1100, 1000, 1000], // 未月 - 土旺金相
	[1060, 1060, 1000, 1000, 1000, 1000, 1140, 1140, 1200, 1200], // 申月 - 金旺水相
	[1000, 1000, 1000, 1000, 1000, 1000, 1200, 1200, 1200, 1200], // 酉月 - 金旺水相
	[1000, 1000, 1040, 1040, 1140, 1140, 1160, 1160, 1060, 1060], // 戌月 - 土旺金相
	[1200, 1200, 1000, 1000, 1000, 1000, 1000, 1000, 1140, 1140], // 亥月 - 水旺木相
];

// ================================
// 月令权重矩阵（高级版）
// ================================

/// 月令旺衰权重矩阵 (12月×36位置)
///
/// 参考 BaziGo 的 dizhiqiangdulist 实现
///
/// 格式：DIZHI_STRENGTH[月支][地支*3 + 藏干序号]
///
/// 地支藏干排列 (36位):
/// 子(癸)        丑(己癸辛)      寅(甲丙戊)    卯(乙)
/// 辰(戊乙癸)    巳(丙戊庚)      午(丁己)      未(己乙丁)
/// 申(庚壬戊)    酉(辛)          戌(戊辛丁)    亥(壬甲)
///
/// 每个地支3个位置，共12地支×3 = 36位置
/// 位置0,1,2对应子藏干；位置3,4,5对应丑藏干...
pub const DIZHI_STRENGTH: [[u16; 36]; 12] = [
	// 子月(水旺) - 索引0
	// 子      丑            寅            卯       辰            巳            午         未            申            酉       戌            亥
	[1200, 0, 0, 500, 360, 200, 840, 300, 0, 1200, 0, 0, 500, 360, 240, 700, 0, 300, 1000, 0, 0, 500, 240, 300, 700, 0, 360, 1000, 0, 0, 500, 300, 200, 840, 360, 0],
	// 丑月(土旺金相) - 索引1
	[1100, 0, 0, 550, 330, 228, 742, 300, 0, 1060, 0, 0, 550, 318, 220, 700, 0, 342, 1000, 0, 0, 550, 212, 300, 798, 0, 330, 1140, 0, 0, 550, 342, 200, 770, 318, 0],
	// 寅月(木旺火相) - 索引2
	[1000, 0, 0, 530, 300, 200, 798, 360, 0, 1140, 0, 0, 530, 342, 200, 840, 0, 300, 1200, 0, 0, 530, 228, 360, 700, 0, 300, 1000, 0, 0, 530, 300, 240, 700, 342, 0],
	// 卯月(木旺火相) - 索引3
	[1000, 0, 0, 500, 300, 200, 840, 360, 0, 1200, 0, 0, 500, 360, 200, 840, 0, 300, 1200, 0, 0, 500, 240, 360, 700, 0, 300, 1000, 0, 0, 500, 300, 240, 700, 360, 0],
	// 辰月(土旺金相) - 索引4
	[1040, 0, 0, 550, 312, 230, 770, 318, 0, 1100, 0, 0, 550, 330, 208, 742, 0, 330, 1060, 0, 0, 550, 220, 318, 770, 0, 312, 1100, 0, 0, 550, 330, 212, 728, 330, 0],
	// 巳月(火旺土相) - 索引5
	[1060, 0, 0, 570, 318, 212, 700, 342, 0, 1000, 0, 0, 600, 300, 200, 840, 0, 300, 1140, 0, 0, 570, 200, 342, 742, 0, 318, 1060, 0, 0, 570, 318, 228, 742, 300, 0],
	// 午月(火旺土相) - 索引6
	[1000, 0, 0, 600, 300, 200, 700, 360, 0, 1000, 0, 0, 600, 300, 200, 840, 0, 300, 1200, 0, 0, 600, 200, 360, 700, 0, 300, 1000, 0, 0, 600, 300, 240, 700, 300, 0],
	// 未月(土旺金相) - 索引7
	[1000, 0, 0, 580, 300, 220, 728, 330, 0, 1040, 0, 0, 580, 312, 200, 798, 0, 330, 1100, 0, 0, 580, 208, 330, 770, 0, 300, 1100, 0, 0, 580, 330, 220, 700, 312, 0],
	// 申月(金旺水相) - 索引8
	[1200, 0, 0, 500, 360, 228, 742, 300, 0, 1060, 0, 0, 500, 318, 240, 700, 0, 342, 1000, 0, 0, 500, 212, 300, 798, 0, 360, 1140, 0, 0, 500, 342, 200, 840, 318, 0],
	// 酉月(金旺水相) - 索引9
	[1200, 0, 0, 500, 360, 248, 700, 300, 0, 1000, 0, 0, 500, 300, 240, 700, 0, 360, 1000, 0, 0, 500, 200, 300, 840, 0, 360, 1200, 0, 0, 500, 360, 200, 840, 300, 0],
	// 戌月(土旺金相) - 索引10
	[1060, 0, 0, 570, 318, 232, 700, 342, 0, 1000, 0, 0, 570, 300, 212, 728, 0, 348, 1040, 0, 0, 570, 200, 312, 812, 0, 318, 1160, 0, 0, 570, 348, 208, 724, 300, 0],
	// 亥月(水旺木相) - 索引11
	[1140, 0, 0, 500, 342, 200, 840, 318, 0, 1200, 0, 0, 500, 360, 228, 742, 0, 300, 1060, 0, 0, 500, 240, 318, 700, 0, 342, 1000, 0, 0, 500, 300, 212, 798, 360, 0],
];

/// 获取天干在指定月令下的强度
///
/// ## 参数
/// - `month_zhi`: 月支（0-11对应子-亥）
/// - `tiangan`: 天干（0-9对应甲-癸）
///
/// ## 返回
/// - 天干强度值（1000为基准）
pub fn get_tiangan_strength(month_zhi: u8, tiangan: u8) -> u16 {
	if month_zhi < 12 && tiangan < 10 {
		TIANGAN_STRENGTH[month_zhi as usize][tiangan as usize]
	} else {
		1000 // 默认基准值
	}
}

/// 获取地支藏干在指定月令下的强度
///
/// ## 参数
/// - `month_zhi`: 月支（0-11对应子-亥）
/// - `dizhi`: 地支（0-11对应子-亥）
/// - `canggan_idx`: 藏干序号（0=主气，1=中气，2=余气）
///
/// ## 返回
/// - 藏干强度值
pub fn get_dizhi_canggan_strength(month_zhi: u8, dizhi: u8, canggan_idx: u8) -> u16 {
	if month_zhi < 12 && dizhi < 12 && canggan_idx < 3 {
		let pos = dizhi as usize * 3 + canggan_idx as usize;
		DIZHI_STRENGTH[month_zhi as usize][pos]
	} else {
		0
	}
}

// ================================
// 辅助函数
// ================================

/// 无效藏干标记值
pub const INVALID_CANGGAN: u8 = 255;

/// 获取指定地支的藏干列表
///
/// 返回：[(天干, 藏干类型, 权重), ...]
/// 注意：当藏干索引为255时，表示该位置无藏干，返回的TianGan(255)应视为无效
pub fn get_hidden_stems(dizhi: DiZhi) -> [(TianGan, CangGanType, u16); 3] {
	let stems = EARTHLY_HIDDEN_STEMS[dizhi.0 as usize];
	let types = CANGGAN_TYPE_TABLE[dizhi.0 as usize];
	let weights = CANGGAN_BASE_WEIGHT[dizhi.0 as usize];

	[
		(TianGan(stems[0]), types[0], weights[0]),
		(TianGan(stems[1]), types[1], weights[1]),
		(TianGan(stems[2]), types[2], weights[2]),
	]
}

/// 获取指定地支的有效藏干数量
pub fn get_hidden_stems_count(dizhi: DiZhi) -> usize {
	let stems = EARTHLY_HIDDEN_STEMS[dizhi.0 as usize];
	stems.iter().filter(|&&s| s != INVALID_CANGGAN).count()
}

/// 判断藏干索引是否有效
pub fn is_valid_canggan(gan_index: u8) -> bool {
	gan_index != INVALID_CANGGAN && gan_index < 10
}

/// 计算纳音
///
/// 参考 lunisolar 的算法实现
pub fn calculate_nayin(ganzhi: &GanZhi) -> NaYin {
	let index = (ganzhi.to_index() / 2) as usize;
	NAYIN_TABLE[index]
}

/// 计算十神
///
/// 查表法，快速准确
pub fn calculate_shishen(rizhu: TianGan, other_gan: TianGan) -> ShiShen {
	let index = SHISHEN_TABLE[rizhu.0 as usize][other_gan.0 as usize];
	SHISHEN_ARRAY[index as usize]
}

// ================================
// 十二长生常量表
// ================================

/// 十二长生起始地支表
///
/// 阳干从长生顺行，阴干从长生逆行
/// 格式：CHANGSHENG_START[天干索引] = 长生所在地支索引
///
/// 阳干长生位：
/// - 甲(0): 亥(11)  乙(1): 午(6)
/// - 丙(2): 寅(2)   丁(3): 酉(9)
/// - 戊(4): 寅(2)   己(5): 酉(9)
/// - 庚(6): 巳(5)   辛(7): 子(0)
/// - 壬(8): 申(8)   癸(9): 卯(3)
pub const CHANGSHENG_START: [u8; 10] = [
	11, // 甲长生在亥
	6,  // 乙长生在午
	2,  // 丙长生在寅
	9,  // 丁长生在酉
	2,  // 戊长生在寅（戊同丙）
	9,  // 己长生在酉（己同丁）
	5,  // 庚长生在巳
	0,  // 辛长生在子
	8,  // 壬长生在申
	3,  // 癸长生在卯
];

/// 计算十二长生
///
/// 根据天干和地支计算十二长生状态
///
/// ## 计算规则
///
/// 1. 找到该天干的长生位（CHANGSHENG_START）
/// 2. 阳干顺行计算：(地支 - 长生位 + 12) % 12
/// 3. 阴干逆行计算：(长生位 - 地支 + 12) % 12
///
/// ## 参数
///
/// - `gan`: 天干
/// - `zhi`: 地支
///
/// ## 返回
///
/// - `ShiErChangSheng`: 十二长生状态
pub fn calculate_changsheng(gan: TianGan, zhi: DiZhi) -> ShiErChangSheng {
	let start = CHANGSHENG_START[gan.0 as usize];
	let is_yang = gan.is_yang();

	let offset = if is_yang {
		// 阳干顺行
		(zhi.0 as i8 - start as i8 + 12) % 12
	} else {
		// 阴干逆行
		(start as i8 - zhi.0 as i8 + 12) % 12
	};

	match offset as u8 {
		0 => ShiErChangSheng::ChangSheng,
		1 => ShiErChangSheng::MuYu,
		2 => ShiErChangSheng::GuanDai,
		3 => ShiErChangSheng::LinGuan,
		4 => ShiErChangSheng::DiWang,
		5 => ShiErChangSheng::Shuai,
		6 => ShiErChangSheng::Bing,
		7 => ShiErChangSheng::Si,
		8 => ShiErChangSheng::Mu,
		9 => ShiErChangSheng::Jue,
		10 => ShiErChangSheng::Tai,
		11 => ShiErChangSheng::Yang,
		_ => ShiErChangSheng::ChangSheng, // 不会到达
	}
}

// ================================
// 空亡计算
// ================================

/// 计算空亡
///
/// 空亡是指六十甲子中每旬（10个干支）中缺少的两个地支。
///
/// ## 空亡表
///
/// - 甲子旬：戌亥空
/// - 甲戌旬：申酉空
/// - 甲申旬：午未空
/// - 甲午旬：辰巳空
/// - 甲辰旬：寅卯空
/// - 甲寅旬：子丑空
///
/// ## 参数
///
/// - `ganzhi`: 干支（通常用日柱）
///
/// ## 返回
///
/// - `(DiZhi, DiZhi)`: 两个空亡地支
pub fn calculate_kongwang(ganzhi: &GanZhi) -> (DiZhi, DiZhi) {
	// 找到该干支所在的旬首（甲X）
	// 六十甲子分为6旬：甲子旬(0-9)、甲戌旬(10-19)、甲申旬(20-29)...
	let index = ganzhi.to_index();
	let xun_start = (index / 10) * 10; // 旬首索引

	// 空亡地支 = 旬首地支 + 10 和 + 11（对12取模）
	// 因为每旬只有10个干支，地支从旬首开始数10个后的两个就是空亡
	let xun_start_zhi = GanZhi::from_index(xun_start).unwrap().zhi.0;
	let kong1 = (xun_start_zhi + 10) % 12;
	let kong2 = (xun_start_zhi + 11) % 12;

	(DiZhi(kong1), DiZhi(kong2))
}

/// 判断地支是否空亡
pub fn is_kongwang(ganzhi: &GanZhi, zhi: DiZhi) -> bool {
	let (kong1, kong2) = calculate_kongwang(ganzhi);
	zhi == kong1 || zhi == kong2
}

// ================================
// 胎元、命宫、身宫计算
// ================================

/// 计算胎元
///
/// 胎元 = 月干进一位 + 月支进三位
///
/// ## 参数
///
/// - `month_ganzhi`: 月柱干支
///
/// ## 返回
///
/// - `GanZhi`: 胎元干支
pub fn calculate_taiyuan(month_ganzhi: &GanZhi) -> GanZhi {
	let gan = TianGan((month_ganzhi.gan.0 + 1) % 10);
	let zhi = DiZhi((month_ganzhi.zhi.0 + 3) % 12);
	GanZhi { gan, zhi }
}

/// 计算命宫
///
/// 命宫计算公式：
/// 月支 + 时支 = 14 时，命宫地支 = 寅
/// 不足14时，命宫地支 = 14 - (月支 + 时支)
/// 超过14时，命宫地支 = 26 - (月支 + 时支)
///
/// 命宫天干：根据年干确定
///
/// ## 参数
///
/// - `year_gan`: 年柱天干
/// - `month_zhi`: 月支
/// - `hour_zhi`: 时支
///
/// ## 返回
///
/// - `GanZhi`: 命宫干支
pub fn calculate_minggong(year_gan: TianGan, month_zhi: DiZhi, hour_zhi: DiZhi) -> GanZhi {
	// 月支和时支之和（用寅月=1，子时=1的编码）
	// 这里转换为：寅(2)=1, 卯(3)=2, ... 丑(1)=12
	// 和：子(0)=1, 丑(1)=2, ... 亥(11)=12
	let month_val = if month_zhi.0 >= 2 { month_zhi.0 - 1 } else { month_zhi.0 + 11 };
	let hour_val = hour_zhi.0 + 1;

	let sum = month_val + hour_val;
	let ming_zhi_val = if sum <= 14 {
		14 - sum
	} else {
		26 - sum
	};

	// 转回标准地支编码
	let ming_zhi = DiZhi(if ming_zhi_val == 0 { 2 } else if ming_zhi_val <= 10 { ming_zhi_val + 2 } else { ming_zhi_val - 10 } as u8 % 12);

	// 命宫天干：使用五虎遁
	let yin_month_gan = match year_gan.0 {
		0 | 5 => 2,  // 甲己 → 丙寅
		1 | 6 => 4,  // 乙庚 → 戊寅
		2 | 7 => 6,  // 丙辛 → 庚寅
		3 | 8 => 8,  // 丁壬 → 壬寅
		4 | 9 => 0,  // 戊癸 → 甲寅
		_ => 0,
	};

	let offset = if ming_zhi.0 >= 2 { ming_zhi.0 - 2 } else { ming_zhi.0 + 10 };
	let ming_gan = TianGan((yin_month_gan + offset) % 10);

	GanZhi { gan: ming_gan, zhi: ming_zhi }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_chen_hidden_stems() {
		// ⚠️ 关键测试：确保辰藏干为"戊乙癸"
		let chen = DiZhi(4); // 辰
		let stems = get_hidden_stems(chen);

		assert_eq!(stems[0].0 .0, 4); // 戊
		assert_eq!(stems[1].0 .0, 1); // 乙
		assert_eq!(stems[2].0 .0, 9); // 癸 (不是壬！)
	}

	#[test]
	fn test_nayin_calculation() {
		// 测试纳音计算
		let jiazi = GanZhi::from_index(0).unwrap(); // 甲子
		let nayin = calculate_nayin(&jiazi);
		assert_eq!(nayin, NaYin::HaiZhongJin); // 海中金
	}

	#[test]
	fn test_shishen_calculation() {
		// 测试十神计算：甲木日主见丙火 → 食神
		let rizhu = TianGan(0); // 甲
		let other = TianGan(2); // 丙
		let shishen = calculate_shishen(rizhu, other);
		assert_eq!(shishen, ShiShen::ShiShen); // 食神
	}
}
