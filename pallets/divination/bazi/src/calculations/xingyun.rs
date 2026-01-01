//! # 星运计算模块（十二长生）
//!
//! 本模块实现八字命理中的十二长生计算，包括：
//! - 日主在各地支的长生状态查询
//! - 阳干和阴干的分别处理
//! - 四柱星运信息构建
//!
//! ## 十二长生规则
//!
//! 十二长生表示天干在地支中的生旺死绝状态，分为：
//! - 旺相: 长生、冠带、临官、帝旺
//! - 衰败: 衰、病、死、墓、绝
//! - 中性: 沐浴、胎、养
//!
//! 阳干和阴干的十二长生顺序不同：
//! - 阳干顺行：长生 → 沐浴 → 冠带 → ... → 养
//! - 阴干逆行：养 → 绝 → 墓 → ... → 长生

use crate::types::{TianGan, DiZhi, ShiErChangSheng, XingYunInfo, SiZhu};

/// 十二长生查询表（阳干）
///
/// 行：5个阳干（甲、丙、戊、庚、壬）
/// 列：12个地支（子、丑、寅、...、亥）
///
/// 阳干顺行：从长生开始，依次经过十二长生状态
///
/// 参考资料：
/// - 《渊海子平》
/// - 《三命通会》
/// - 《滴天髓》
const CHANGSHENG_TABLE_YANG: [[ShiErChangSheng; 12]; 5] = [
	// 甲木: 长生在亥，沐浴在子，冠带在丑，临官在寅，帝旺在卯
	[
		ShiErChangSheng::MuYu,        // 子(0)  - 沐浴
		ShiErChangSheng::GuanDai,     // 丑(1)  - 冠带
		ShiErChangSheng::LinGuan,     // 寅(2)  - 临官（建禄）
		ShiErChangSheng::DiWang,      // 卯(3)  - 帝旺
		ShiErChangSheng::Shuai,       // 辰(4)  - 衰
		ShiErChangSheng::Bing,        // 巳(5)  - 病
		ShiErChangSheng::Si,          // 午(6)  - 死
		ShiErChangSheng::Mu,          // 未(7)  - 墓（库）
		ShiErChangSheng::Jue,         // 申(8)  - 绝
		ShiErChangSheng::Tai,         // 酉(9)  - 胎
		ShiErChangSheng::Yang,        // 戌(10) - 养
		ShiErChangSheng::ChangSheng,  // 亥(11) - 长生
	],
	// 丙火: 长生在寅，沐浴在卯，冠带在辰，临官在巳，帝旺在午
	[
		ShiErChangSheng::Tai,         // 子(0)  - 胎
		ShiErChangSheng::Yang,        // 丑(1)  - 养
		ShiErChangSheng::ChangSheng,  // 寅(2)  - 长生
		ShiErChangSheng::MuYu,        // 卯(3)  - 沐浴
		ShiErChangSheng::GuanDai,     // 辰(4)  - 冠带
		ShiErChangSheng::LinGuan,     // 巳(5)  - 临官（建禄）
		ShiErChangSheng::DiWang,      // 午(6)  - 帝旺
		ShiErChangSheng::Shuai,       // 未(7)  - 衰
		ShiErChangSheng::Bing,        // 申(8)  - 病
		ShiErChangSheng::Si,          // 酉(9)  - 死
		ShiErChangSheng::Mu,          // 戌(10) - 墓（库）
		ShiErChangSheng::Jue,         // 亥(11) - 绝
	],
	// 戊土: 同丙火（长生在寅，火生土）
	[
		ShiErChangSheng::Tai,         // 子(0)  - 胎
		ShiErChangSheng::Yang,        // 丑(1)  - 养
		ShiErChangSheng::ChangSheng,  // 寅(2)  - 长生
		ShiErChangSheng::MuYu,        // 卯(3)  - 沐浴
		ShiErChangSheng::GuanDai,     // 辰(4)  - 冠带
		ShiErChangSheng::LinGuan,     // 巳(5)  - 临官（建禄）
		ShiErChangSheng::DiWang,      // 午(6)  - 帝旺
		ShiErChangSheng::Shuai,       // 未(7)  - 衰
		ShiErChangSheng::Bing,        // 申(8)  - 病
		ShiErChangSheng::Si,          // 酉(9)  - 死
		ShiErChangSheng::Mu,          // 戌(10) - 墓（库）
		ShiErChangSheng::Jue,         // 亥(11) - 绝
	],
	// 庚金: 长生在巳，沐浴在午，冠带在未，临官在申，帝旺在酉
	[
		ShiErChangSheng::Si,          // 子(0)  - 死
		ShiErChangSheng::Mu,          // 丑(1)  - 墓（库）
		ShiErChangSheng::Jue,         // 寅(2)  - 绝
		ShiErChangSheng::Tai,         // 卯(3)  - 胎
		ShiErChangSheng::Yang,        // 辰(4)  - 养
		ShiErChangSheng::ChangSheng,  // 巳(5)  - 长生
		ShiErChangSheng::MuYu,        // 午(6)  - 沐浴
		ShiErChangSheng::GuanDai,     // 未(7)  - 冠带
		ShiErChangSheng::LinGuan,     // 申(8)  - 临官（建禄）
		ShiErChangSheng::DiWang,      // 酉(9)  - 帝旺
		ShiErChangSheng::Shuai,       // 戌(10) - 衰
		ShiErChangSheng::Bing,        // 亥(11) - 病
	],
	// 壬水: 长生在申，沐浴在酉，冠带在戌，临官在亥，帝旺在子
	[
		ShiErChangSheng::DiWang,      // 子(0)  - 帝旺
		ShiErChangSheng::Shuai,       // 丑(1)  - 衰
		ShiErChangSheng::Bing,        // 寅(2)  - 病
		ShiErChangSheng::Si,          // 卯(3)  - 死
		ShiErChangSheng::Mu,          // 辰(4)  - 墓（库）
		ShiErChangSheng::Jue,         // 巳(5)  - 绝
		ShiErChangSheng::Tai,         // 午(6)  - 胎
		ShiErChangSheng::Yang,        // 未(7)  - 养
		ShiErChangSheng::ChangSheng,  // 申(8)  - 长生
		ShiErChangSheng::MuYu,        // 酉(9)  - 沐浴
		ShiErChangSheng::GuanDai,     // 戌(10) - 冠带
		ShiErChangSheng::LinGuan,     // 亥(11) - 临官（建禄）
	],
];

/// 十二长生查询表（阴干）
///
/// 行：5个阴干（乙、丁、己、辛、癸）
/// 列：12个地支（子、丑、寅、...、亥）
///
/// 阴干逆行：从长生开始，逆向经过十二长生状态
///
/// 参考资料：
/// - 《渊海子平》- 阴阳逆顺之说
/// - 《三命通会》- 论阴干长生
/// - 《滴天髓》- 阴干长生诀
///
/// 口诀：
/// - 乙木长生午上行，从午逆数至未养
/// - 丁火酉中寻长生，己土同推不用更
/// - 辛金子位是长生，癸水卯中福禄增
const CHANGSHENG_TABLE_YIN: [[ShiErChangSheng; 12]; 5] = [
	// 乙木: 长生在午，从午逆行（午→巳→辰→卯→寅→丑→子→亥→戌→酉→申→未）
	// 长生→沐浴→冠带→临官→帝旺→衰→病→死→墓→绝→胎→养
	[
		ShiErChangSheng::Bing,        // 子(0)  - 病
		ShiErChangSheng::Shuai,       // 丑(1)  - 衰
		ShiErChangSheng::DiWang,      // 寅(2)  - 帝旺
		ShiErChangSheng::LinGuan,     // 卯(3)  - 临官（建禄）
		ShiErChangSheng::GuanDai,     // 辰(4)  - 冠带
		ShiErChangSheng::MuYu,        // 巳(5)  - 沐浴
		ShiErChangSheng::ChangSheng,  // 午(6)  - 长生
		ShiErChangSheng::Yang,        // 未(7)  - 养
		ShiErChangSheng::Tai,         // 申(8)  - 胎
		ShiErChangSheng::Jue,         // 酉(9)  - 绝
		ShiErChangSheng::Mu,          // 戌(10) - 墓（库）
		ShiErChangSheng::Si,          // 亥(11) - 死
	],
	// 丁火: 长生在酉，从酉逆行（酉→申→未→午→巳→辰→卯→寅→丑→子→亥→戌）
	[
		ShiErChangSheng::Jue,         // 子(0)  - 绝
		ShiErChangSheng::Mu,          // 丑(1)  - 墓（库）
		ShiErChangSheng::Si,          // 寅(2)  - 死
		ShiErChangSheng::Bing,        // 卯(3)  - 病
		ShiErChangSheng::Shuai,       // 辰(4)  - 衰
		ShiErChangSheng::DiWang,      // 巳(5)  - 帝旺
		ShiErChangSheng::LinGuan,     // 午(6)  - 临官（建禄）
		ShiErChangSheng::GuanDai,     // 未(7)  - 冠带
		ShiErChangSheng::MuYu,        // 申(8)  - 沐浴
		ShiErChangSheng::ChangSheng,  // 酉(9)  - 长生
		ShiErChangSheng::Yang,        // 戌(10) - 养
		ShiErChangSheng::Tai,         // 亥(11) - 胎
	],
	// 己土: 同丁火（长生在酉，火生土）
	[
		ShiErChangSheng::Jue,         // 子(0)  - 绝
		ShiErChangSheng::Mu,          // 丑(1)  - 墓（库）
		ShiErChangSheng::Si,          // 寅(2)  - 死
		ShiErChangSheng::Bing,        // 卯(3)  - 病
		ShiErChangSheng::Shuai,       // 辰(4)  - 衰
		ShiErChangSheng::DiWang,      // 巳(5)  - 帝旺
		ShiErChangSheng::LinGuan,     // 午(6)  - 临官（建禄）
		ShiErChangSheng::GuanDai,     // 未(7)  - 冠带
		ShiErChangSheng::MuYu,        // 申(8)  - 沐浴
		ShiErChangSheng::ChangSheng,  // 酉(9)  - 长生
		ShiErChangSheng::Yang,        // 戌(10) - 养
		ShiErChangSheng::Tai,         // 亥(11) - 胎
	],
	// 辛金: 长生在子，从子逆行（子→亥→戌→酉→申→未→午→巳→辰→卯→寅→丑）
	[
		ShiErChangSheng::ChangSheng,  // 子(0)  - 长生
		ShiErChangSheng::Yang,        // 丑(1)  - 养
		ShiErChangSheng::Tai,         // 寅(2)  - 胎
		ShiErChangSheng::Jue,         // 卯(3)  - 绝
		ShiErChangSheng::Mu,          // 辰(4)  - 墓（库）
		ShiErChangSheng::Si,          // 巳(5)  - 死
		ShiErChangSheng::Bing,        // 午(6)  - 病
		ShiErChangSheng::Shuai,       // 未(7)  - 衰
		ShiErChangSheng::DiWang,      // 申(8)  - 帝旺
		ShiErChangSheng::LinGuan,     // 酉(9)  - 临官（建禄）
		ShiErChangSheng::GuanDai,     // 戌(10) - 冠带
		ShiErChangSheng::MuYu,        // 亥(11) - 沐浴
	],
	// 癸水: 长生在卯，从卯逆行（卯→寅→丑→子→亥→戌→酉→申→未→午→巳→辰）
	[
		ShiErChangSheng::DiWang,      // 子(0)  - 帝旺
		ShiErChangSheng::GuanDai,     // 丑(1)  - 冠带
		ShiErChangSheng::MuYu,        // 寅(2)  - 沐浴
		ShiErChangSheng::ChangSheng,  // 卯(3)  - 长生
		ShiErChangSheng::Yang,        // 辰(4)  - 养
		ShiErChangSheng::Tai,         // 巳(5)  - 胎
		ShiErChangSheng::Jue,         // 午(6)  - 绝
		ShiErChangSheng::Mu,          // 未(7)  - 墓（库）
		ShiErChangSheng::Si,          // 申(8)  - 死
		ShiErChangSheng::Bing,        // 酉(9)  - 病
		ShiErChangSheng::Shuai,       // 戌(10) - 衰
		ShiErChangSheng::LinGuan,     // 亥(11) - 临官（建禄）
	],
];

/// 获取日主在地支的十二长生状态
///
/// # 参数
///
/// - `rizhu`: 日主天干
/// - `dizhi`: 要查询的地支
///
/// # 返回
///
/// 日主在该地支的十二长生状态
///
/// # 原理
///
/// - 阳干（甲丙戊庚壬）顺行查表
/// - 阴干（乙丁己辛癸）逆行查表
///
/// # 示例
///
/// ```ignore
/// let rizhu = TianGan(0); // 甲木
/// let dizhi = DiZhi(2);   // 寅
/// let changsheng = get_changsheng(rizhu, dizhi);
/// assert_eq!(changsheng, ShiErChangSheng::ChangSheng); // 甲木长生于寅
/// ```
pub fn get_changsheng(rizhu: TianGan, dizhi: DiZhi) -> ShiErChangSheng {
	let dizhi_index = dizhi.0 as usize;

	if rizhu.is_yang() {
		// 阳干查表
		let row = (rizhu.0 / 2) as usize; // 0→0(甲), 2→1(丙), 4→2(戊), 6→3(庚), 8→4(壬)
		CHANGSHENG_TABLE_YANG[row][dizhi_index]
	} else {
		// 阴干查表
		let row = (rizhu.0 / 2) as usize; // 1→0(乙), 3→1(丁), 5→2(己), 7→3(辛), 9→4(癸)
		CHANGSHENG_TABLE_YIN[row][dizhi_index]
	}
}

/// 计算四柱的星运信息（日主在四柱各支的十二长生）
///
/// # 参数
///
/// - `sizhu`: 四柱信息
///
/// # 返回
///
/// `XingYunInfo` 结构，包含日主在年、月、日、时支的十二长生状态
///
/// # 说明
///
/// - 月支的长生状态最重要（月令）
/// - 日支次之（坐支）
/// - 年支和时支影响相对较小
///
/// # 示例
///
/// ```ignore
/// let sizhu = SiZhu { rizhu: TianGan(0), ... }; // 甲木日主
/// let xingyun = calculate_xingyun(&sizhu);
///
/// if xingyun.month_changsheng.is_prosperous() {
///     println!("月令得力，命主有贵气");
/// }
/// ```
pub fn calculate_xingyun<T: crate::pallet::Config>(sizhu: &SiZhu<T>) -> XingYunInfo {
	let rizhu = sizhu.rizhu;

	XingYunInfo {
		year_changsheng: get_changsheng(rizhu, sizhu.year_zhu.ganzhi.zhi),
		month_changsheng: get_changsheng(rizhu, sizhu.month_zhu.ganzhi.zhi),
		day_changsheng: get_changsheng(rizhu, sizhu.day_zhu.ganzhi.zhi),
		hour_changsheng: get_changsheng(rizhu, sizhu.hour_zhu.ganzhi.zhi),
	}
}

/// 计算临时四柱的星运信息（不使用泛型）
///
/// # 参数
///
/// - `rizhu`: 日主天干
/// - `year_zhi`: 年支
/// - `month_zhi`: 月支
/// - `day_zhi`: 日支
/// - `hour_zhi`: 时支
///
/// # 返回
///
/// `XingYunInfo` 结构，包含日主在各支的十二长生状态
///
/// # 用途
///
/// 供临时排盘接口使用，避免泛型依赖
pub fn calculate_xingyun_temp(
	rizhu: TianGan,
	year_zhi: &DiZhi,
	month_zhi: &DiZhi,
	day_zhi: &DiZhi,
	hour_zhi: &DiZhi,
) -> XingYunInfo {
	XingYunInfo {
		year_changsheng: get_changsheng(rizhu, *year_zhi),
		month_changsheng: get_changsheng(rizhu, *month_zhi),
		day_changsheng: get_changsheng(rizhu, *day_zhi),
		hour_changsheng: get_changsheng(rizhu, *hour_zhi),
	}
}

// ================================
// 单元测试
// ================================

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_jiamu_changsheng_at_hai() {
		// 甲木长生在亥
		let rizhu = TianGan(0); // 甲
		let dizhi = DiZhi(11);  // 亥
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::ChangSheng);
	}

	#[test]
	fn test_jiamu_diwang_at_mao() {
		// 甲木帝旺在卯
		let rizhu = TianGan(0); // 甲
		let dizhi = DiZhi(3);   // 卯
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::DiWang);
	}

	#[test]
	fn test_jiamu_linguan_at_yin() {
		// 甲木临官（建禄）在寅
		let rizhu = TianGan(0); // 甲
		let dizhi = DiZhi(2);   // 寅
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::LinGuan);
	}

	#[test]
	fn test_binghuo_changsheng_at_yin() {
		// 丙火长生在寅
		let rizhu = TianGan(2); // 丙
		let dizhi = DiZhi(2);   // 寅
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::ChangSheng);
	}

	#[test]
	fn test_binghuo_diwang_at_wu() {
		// 丙火帝旺在午
		let rizhu = TianGan(2); // 丙
		let dizhi = DiZhi(6);   // 午
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::DiWang);
	}

	#[test]
	fn test_gengjin_changsheng_at_si() {
		// 庚金长生在巳
		let rizhu = TianGan(6); // 庚
		let dizhi = DiZhi(5);   // 巳
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::ChangSheng);
	}

	#[test]
	fn test_gengjin_diwang_at_you() {
		// 庚金帝旺在酉
		let rizhu = TianGan(6); // 庚
		let dizhi = DiZhi(9);   // 酉
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::DiWang);
	}

	#[test]
	fn test_renshui_changsheng_at_shen() {
		// 壬水长生在申
		let rizhu = TianGan(8); // 壬
		let dizhi = DiZhi(8);   // 申
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::ChangSheng);
	}

	#[test]
	fn test_renshui_diwang_at_zi() {
		// 壬水帝旺在子
		let rizhu = TianGan(8); // 壬
		let dizhi = DiZhi(0);   // 子
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::DiWang);
	}

	#[test]
	fn test_yimu_changsheng_at_wu() {
		// 乙木长生在午
		let rizhu = TianGan(1); // 乙
		let dizhi = DiZhi(6);   // 午
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::ChangSheng);
	}

	#[test]
	fn test_yimu_diwang_at_yin() {
		// 乙木帝旺在寅（逆行）
		let rizhu = TianGan(1); // 乙
		let dizhi = DiZhi(2);   // 寅
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::DiWang);
	}

	#[test]
	fn test_dinghuo_changsheng_at_you() {
		// 丁火长生在酉
		let rizhu = TianGan(3); // 丁
		let dizhi = DiZhi(9);   // 酉
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::ChangSheng);
	}

	#[test]
	fn test_dinghuo_diwang_at_si() {
		// 丁火帝旺在巳（逆行）
		let rizhu = TianGan(3); // 丁
		let dizhi = DiZhi(5);   // 巳
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::DiWang);
	}

	#[test]
	fn test_xinjin_changsheng_at_zi() {
		// 辛金长生在子
		let rizhu = TianGan(7); // 辛
		let dizhi = DiZhi(0);   // 子
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::ChangSheng);
	}

	#[test]
	fn test_xinjin_diwang_at_shen() {
		// 辛金帝旺在申（逆行）
		let rizhu = TianGan(7); // 辛
		let dizhi = DiZhi(8);   // 申
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::DiWang);
	}

	#[test]
	fn test_guishui_changsheng_at_mao() {
		// 癸水长生在卯
		let rizhu = TianGan(9); // 癸
		let dizhi = DiZhi(3);   // 卯
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::ChangSheng);
	}

	#[test]
	fn test_guishui_diwang_at_zi() {
		// 癸水帝旺在子（逆行）
		let rizhu = TianGan(9); // 癸
		let dizhi = DiZhi(0);   // 子
		let changsheng = get_changsheng(rizhu, dizhi);
		assert_eq!(changsheng, ShiErChangSheng::DiWang);
	}

	#[test]
	fn test_is_prosperous() {
		assert!(ShiErChangSheng::ChangSheng.is_prosperous());
		assert!(ShiErChangSheng::DiWang.is_prosperous());
		assert!(!ShiErChangSheng::Shuai.is_prosperous());
	}

	#[test]
	fn test_is_declining() {
		assert!(ShiErChangSheng::Shuai.is_declining());
		assert!(ShiErChangSheng::Si.is_declining());
		assert!(!ShiErChangSheng::DiWang.is_declining());
	}
}
