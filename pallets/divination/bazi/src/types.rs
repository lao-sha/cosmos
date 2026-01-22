//! # 八字排盘数据类型定义
//!
//! 本模块定义了八字排盘所需的所有数据类型，包括：
//! - 基础类型: 天干、地支、干支、五行
//! - 命理类型: 十神、纳音、藏干、性别
//! - 模式类型: 子时归属模式、节气枚举
//! - 复合类型: 四柱、大运、八字完整信息

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

// ================================
// 基础类型定义
// ================================

/// 天干类型 (0-9)
///
/// 甲(0) 乙(1) 丙(2) 丁(3) 戊(4) 己(5) 庚(6) 辛(7) 壬(8) 癸(9)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct TianGan(pub u8);

impl TianGan {
	/// 创建天干，验证范围 (0-9)
	pub fn new(value: u8) -> Option<Self> {
		if value < 10 {
			Some(Self(value))
		} else {
			None
		}
	}

	/// 转换为五行
	pub fn to_wuxing(&self) -> WuXing {
		match self.0 {
			0 | 1 => WuXing::Mu,    // 甲乙木
			2 | 3 => WuXing::Huo,   // 丙丁火
			4 | 5 => WuXing::Tu,    // 戊己土
			6 | 7 => WuXing::Jin,   // 庚辛金
			8 | 9 => WuXing::Shui,  // 壬癸水
			_ => unreachable!(),
		}
	}

	/// 判断是否为阳干
	pub fn is_yang(&self) -> bool {
		self.0 % 2 == 0
	}

	/// 获取天干名称
	pub fn name(&self) -> &'static str {
		match self.0 {
			0 => "甲",
			1 => "乙",
			2 => "丙",
			3 => "丁",
			4 => "戊",
			5 => "己",
			6 => "庚",
			7 => "辛",
			8 => "壬",
			9 => "癸",
			_ => "未知",
		}
	}
}

/// 地支类型 (0-11)
///
/// 子(0) 丑(1) 寅(2) 卯(3) 辰(4) 巳(5) 午(6) 未(7) 申(8) 酉(9) 戌(10) 亥(11)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct DiZhi(pub u8);

impl DiZhi {
	/// 创建地支，验证范围 (0-11)
	pub fn new(value: u8) -> Option<Self> {
		if value < 12 {
			Some(Self(value))
		} else {
			None
		}
	}

	/// 转换为五行
	pub fn to_wuxing(&self) -> WuXing {
		match self.0 {
			2 | 3 => WuXing::Mu,       // 寅卯木
			5 | 6 => WuXing::Huo,      // 巳午火
			8 | 9 => WuXing::Jin,      // 申酉金
			11 | 0 => WuXing::Shui,    // 亥子水
			1 | 4 | 7 | 10 => WuXing::Tu,  // 辰戌丑未土
			_ => unreachable!(),
		}
	}

	/// 获取地支名称
	pub fn name(&self) -> &'static str {
		match self.0 {
			0 => "子",
			1 => "丑",
			2 => "寅",
			3 => "卯",
			4 => "辰",
			5 => "巳",
			6 => "午",
			7 => "未",
			8 => "申",
			9 => "酉",
			10 => "戌",
			11 => "亥",
			_ => "未知",
		}
	}

	/// 获取对应的时辰范围
	///
	/// 返回: (开始小时, 结束小时)
	pub fn to_shichen(&self) -> (u8, u8) {
		match self.0 {
			0 => (23, 1),   // 子时 23:00-01:00
			n => ((n * 2 - 1), (n * 2 + 1)),
		}
	}
}

/// 干支组合 (0-59)
///
/// 六十甲子从"甲子"(0)到"癸亥"(59)循环
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct GanZhi {
	/// 天干
	pub gan: TianGan,
	/// 地支
	pub zhi: DiZhi,
}

impl GanZhi {
	/// 从索引创建干支 (0-59)
	pub fn from_index(index: u8) -> Option<Self> {
		if index < 60 {
			Some(Self {
				gan: TianGan(index % 10),
				zhi: DiZhi(index % 12),
			})
		} else {
			None
		}
	}

	/// 转换为索引 (0-59)
	pub fn to_index(&self) -> u8 {
		// 使用中国剩余定理求解
		// 找到满足 x ≡ gan (mod 10) 且 x ≡ zhi (mod 12) 的最小非负整数
		for i in 0..6 {
			let candidate = i * 10 + self.gan.0;
			if candidate % 12 == self.zhi.0 {
				return candidate;
			}
		}
		unreachable!()
	}

	/// 下一个干支
	pub fn next(&self) -> Self {
		let next_index = (self.to_index() + 1) % 60;
		Self::from_index(next_index).unwrap()
	}

	/// 上一个干支
	pub fn prev(&self) -> Self {
		let prev_index = (self.to_index() + 59) % 60;
		Self::from_index(prev_index).unwrap()
	}

	/// 获取干支名称
	pub fn name(&self) -> (&'static str, &'static str) {
		(self.gan.name(), self.zhi.name())
	}
}

/// 五行类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub enum WuXing {
	/// 金
	Jin,
	/// 木
	Mu,
	/// 水
	Shui,
	/// 火
	Huo,
	#[default]
	/// 土
	Tu,
}

impl WuXing {
	/// 获取五行名称
	pub fn name(&self) -> &'static str {
		match self {
			WuXing::Jin => "金",
			WuXing::Mu => "木",
			WuXing::Shui => "水",
			WuXing::Huo => "火",
			WuXing::Tu => "土",
		}
	}
}

/// 十神类型
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ShiShen {
	/// 比肩 - 同我(同性)
	#[default]
	BiJian,
	/// 劫财 - 同我(异性)
	JieCai,
	/// 食神 - 我生(同性)
	ShiShen,
	/// 伤官 - 我生(异性)
	ShangGuan,
	/// 偏财 - 我克(同性)
	PianCai,
	/// 正财 - 我克(异性)
	ZhengCai,
	/// 七杀 - 克我(同性)
	QiSha,
	/// 正官 - 克我(异性)
	ZhengGuan,
	/// 偏印 - 生我(同性)
	PianYin,
	/// 正印 - 生我(异性)
	ZhengYin,
}

impl ShiShen {
	/// 获取十神名称
	pub fn name(&self) -> &'static str {
		match self {
			ShiShen::BiJian => "比肩",
			ShiShen::JieCai => "劫财",
			ShiShen::ShiShen => "食神",
			ShiShen::ShangGuan => "伤官",
			ShiShen::PianCai => "偏财",
			ShiShen::ZhengCai => "正财",
			ShiShen::QiSha => "七杀",
			ShiShen::ZhengGuan => "正官",
			ShiShen::PianYin => "偏印",
			ShiShen::ZhengYin => "正印",
		}
	}
}

// ================================
// 高级类型定义
// ================================

/// 十二长生（星运）
///
/// 表示天干在地支中的生旺死绝状态
/// 用于判断日主在四柱各支的旺衰程度
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ShiErChangSheng {
	/// 长生 - 如人之初生，生命开始
	#[default]
	ChangSheng = 0,
	/// 沐浴 - 如婴儿沐浴，脆弱之时
	MuYu = 1,
	/// 冠带 - 如人戴冠束带，渐成气象
	GuanDai = 2,
	/// 临官 - 如人临官任职，得意之时
	LinGuan = 3,
	/// 帝旺 - 如帝王当朝，最旺盛时
	DiWang = 4,
	/// 衰 - 如人年老体衰
	Shuai = 5,
	/// 病 - 如人疾病缠身
	Bing = 6,
	/// 死 - 如人气绝身亡
	Si = 7,
	/// 墓 - 如人入墓归土
	Mu = 8,
	/// 绝 - 如人形骸俱灭
	Jue = 9,
	/// 胎 - 如人受胎于母腹
	Tai = 10,
	/// 养 - 如人在母腹中成形
	Yang = 11,
}

impl ShiErChangSheng {
	/// 获取十二长生名称
	pub fn name(&self) -> &'static str {
		match self {
			ShiErChangSheng::ChangSheng => "长生",
			ShiErChangSheng::MuYu => "沐浴",
			ShiErChangSheng::GuanDai => "冠带",
			ShiErChangSheng::LinGuan => "临官",
			ShiErChangSheng::DiWang => "帝旺",
			ShiErChangSheng::Shuai => "衰",
			ShiErChangSheng::Bing => "病",
			ShiErChangSheng::Si => "死",
			ShiErChangSheng::Mu => "墓",
			ShiErChangSheng::Jue => "绝",
			ShiErChangSheng::Tai => "胎",
			ShiErChangSheng::Yang => "养",
		}
	}

	/// 判断是否为旺相状态（长生、冠带、临官、帝旺）
	pub fn is_prosperous(&self) -> bool {
		matches!(self,
			ShiErChangSheng::ChangSheng |
			ShiErChangSheng::GuanDai |
			ShiErChangSheng::LinGuan |
			ShiErChangSheng::DiWang
		)
	}

	/// 判断是否为衰败状态（衰、病、死、墓、绝）
	pub fn is_declining(&self) -> bool {
		matches!(self,
			ShiErChangSheng::Shuai |
			ShiErChangSheng::Bing |
			ShiErChangSheng::Si |
			ShiErChangSheng::Mu |
			ShiErChangSheng::Jue
		)
	}
}

/// 藏干类型
///
/// 每个地支藏有1-3个天干，分为主气、中气、余气
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum CangGanType {
	/// 主气（权重最高）
	#[default]
	ZhuQi,
	/// 中气（权重中等）
	ZhongQi,
	/// 余气（权重最低）
	YuQi,
}

/// 藏干信息
///
/// 包含藏干天干、十神关系、藏干类型和权重
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct CangGanInfo {
	/// 藏干天干
	pub gan: TianGan,
	/// 与日主的十神关系
	pub shishen: ShiShen,
	/// 藏干类型（主气/中气/余气）
	pub canggan_type: CangGanType,
	/// 权重（用于五行强度计算）
	pub weight: u16,
}

/// 纳音五行 (30种)
///
/// 六十甲子对应30种纳音五行，每两个相邻干支共享一个纳音
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum NaYin {
	#[default]
	HaiZhongJin,    // 海中金 (甲子乙丑)
	LuZhongHuo,     // 炉中火 (丙寅丁卯)
	DaLinMu,        // 大林木 (戊辰己巳)
	LuPangTu,       // 路旁土 (庚午辛未)
	JianFengJin,    // 剑锋金 (壬申癸酉)
	ShanTouHuo,     // 山头火 (甲戌乙亥)
	JianXiaShui,    // 涧下水 (丙子丁丑)
	ChengTouTu,     // 城头土 (戊寅己卯)
	BaiLaJin,       // 白蜡金 (庚辰辛巳)
	YangLiuMu,      // 杨柳木 (壬午癸未)
	QuanZhongShui,  // 泉中水 (甲申乙酉)
	WuShangTu,      // 屋上土 (丙戌丁亥)
	PiLiHuo,        // 霹雳火 (戊子己丑)
	SongBaiMu,      // 松柏木 (庚寅辛卯)
	ChangLiuShui,   // 长流水 (壬辰癸巳)
	ShaZhongJin,    // 沙中金 (甲午乙未)
	ShanXiaHuo,     // 山下火 (丙申丁酉)
	PingDiMu,       // 平地木 (戊戌己亥)
	BiShangTu,      // 壁上土 (庚子辛丑)
	JinBoJin,       // 金箔金 (壬寅癸卯)
	FuDengHuo,      // 覆灯火 (甲辰乙巳)
	TianHeShui,     // 天河水 (丙午丁未)
	DaYiTu,         // 大驿土 (戊申己酉)
	ChaiChuanJin,   // 钗钏金 (庚戌辛亥)
	SangTuoMu,      // 桑柘木 (壬子癸丑)
	DaXiShui,       // 大溪水 (甲寅乙卯)
	ShaZhongTu,     // 沙中土 (丙辰丁巳)
	TianShangHuo,   // 天上火 (戊午己未)
	ShiLiuMu,       // 石榴木 (庚申辛酉)
	DaHaiShui,      // 大海水 (壬戌癸亥)
}

impl NaYin {
	/// 获取纳音名称
	pub fn name(&self) -> &'static str {
		match self {
			NaYin::HaiZhongJin => "海中金",
			NaYin::LuZhongHuo => "炉中火",
			NaYin::DaLinMu => "大林木",
			NaYin::LuPangTu => "路旁土",
			NaYin::JianFengJin => "剑锋金",
			NaYin::ShanTouHuo => "山头火",
			NaYin::JianXiaShui => "涧下水",
			NaYin::ChengTouTu => "城头土",
			NaYin::BaiLaJin => "白蜡金",
			NaYin::YangLiuMu => "杨柳木",
			NaYin::QuanZhongShui => "泉中水",
			NaYin::WuShangTu => "屋上土",
			NaYin::PiLiHuo => "霹雳火",
			NaYin::SongBaiMu => "松柏木",
			NaYin::ChangLiuShui => "长流水",
			NaYin::ShaZhongJin => "沙中金",
			NaYin::ShanXiaHuo => "山下火",
			NaYin::PingDiMu => "平地木",
			NaYin::BiShangTu => "壁上土",
			NaYin::JinBoJin => "金箔金",
			NaYin::FuDengHuo => "覆灯火",
			NaYin::TianHeShui => "天河水",
			NaYin::DaYiTu => "大驿土",
			NaYin::ChaiChuanJin => "钗钏金",
			NaYin::SangTuoMu => "桑柘木",
			NaYin::DaXiShui => "大溪水",
			NaYin::ShaZhongTu => "沙中土",
			NaYin::TianShangHuo => "天上火",
			NaYin::ShiLiuMu => "石榴木",
			NaYin::DaHaiShui => "大海水",
		}
	}
}

/// 子时归属模式
///
/// ⚠️ 关键功能：支持传统派和现代派两种子时归属模式
///
/// - 传统派：23:00-23:59 属于次日（早子时）
/// - 现代派：23:00-23:59 属于当日（晚子时）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
pub enum ZiShiMode {
	/// 传统派：23:00-23:59 属于次日
	Traditional = 1,
	/// 现代派：23:00-23:59 属于当日
	Modern = 2,
}

/// 输入日历类型（记录原始输入是公历还是农历）
///
/// 用于前端显示和数据分析，不影响八字计算。
/// 所有输入最终都会转换为公历进行计算，但需要记录原始输入类型以便前端正确显示。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
pub enum InputCalendarType {
	/// 公历输入
	#[default]
	Solar = 0,
	/// 农历输入
	Lunar = 1,
	/// 四柱直接输入（无具体日期）
	SiZhu = 2,
}

impl InputCalendarType {
	/// 获取显示名称
	pub fn name(&self) -> &'static str {
		match self {
			InputCalendarType::Solar => "公历",
			InputCalendarType::Lunar => "农历",
			InputCalendarType::SiZhu => "四柱",
		}
	}
}

/// 节气枚举 (24节气中的12个节)
///
/// 注：八字月份以节气为界，这里只列出12个"节"（不含"气"）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum JieQi {
	XiaoHan = 0,   // 小寒 (十二月节)
	LiChun = 1,    // 立春 (正月节)
	JingZhe = 2,   // 惊蛰 (二月节)
	QingMing = 3,  // 清明 (三月节)
	LiXia = 4,     // 立夏 (四月节)
	MangZhong = 5, // 芒种 (五月节)
	XiaoShu = 6,   // 小暑 (六月节)
	LiQiu = 7,     // 立秋 (七月节)
	BaiLu = 8,     // 白露 (八月节)
	HanLu = 9,     // 寒露 (九月节)
	LiDong = 10,   // 立冬 (十月节)
	DaXue = 11,    // 大雪 (十一月节)
}

impl JieQi {
	/// 节气对应的八字月份
	pub fn to_bazi_month(&self) -> u8 {
		(*self as u8 % 12) + 1
	}

	/// 获取节气名称
	pub fn name(&self) -> &'static str {
		match self {
			JieQi::XiaoHan => "小寒",
			JieQi::LiChun => "立春",
			JieQi::JingZhe => "惊蛰",
			JieQi::QingMing => "清明",
			JieQi::LiXia => "立夏",
			JieQi::MangZhong => "芒种",
			JieQi::XiaoShu => "小暑",
			JieQi::LiQiu => "立秋",
			JieQi::BaiLu => "白露",
			JieQi::HanLu => "寒露",
			JieQi::LiDong => "立冬",
			JieQi::DaXue => "大雪",
		}
	}
}

/// 性别
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
pub enum Gender {
	/// 男性
	#[default]
	Male = 1,
	/// 女性
	Female = 0,
}

// ================================
// 复合类型定义
// ================================

/// 出生时间
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
pub struct BirthTime {
	/// 公历年份 (1900-2100)
	pub year: u16,
	/// 公历月份 (1-12)
	pub month: u8,
	/// 公历日期 (1-31)
	pub day: u8,
	/// 小时 (0-23)
	pub hour: u8,
	/// 分钟 (0-59)
	pub minute: u8,
}

/// 单个柱 (年/月/日/时)
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Zhu<T: crate::pallet::Config> {
	/// 干支组合
	pub ganzhi: GanZhi,
	/// 藏干信息（含权重和类型）
	pub canggan: BoundedVec<CangGanInfo, T::MaxCangGan>,
	/// 纳音
	pub nayin: NaYin,
}

impl<T: crate::pallet::Config> PartialEq for Zhu<T> {
	fn eq(&self, other: &Self) -> bool {
		self.ganzhi == other.ganzhi &&
		self.canggan.iter().zip(other.canggan.iter()).all(|(a, b)| a == b) &&
		self.nayin == other.nayin
	}
}

impl<T: crate::pallet::Config> Eq for Zhu<T> {}

/// 四柱
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct SiZhu<T: crate::pallet::Config> {
	/// 年柱
	pub year_zhu: Zhu<T>,
	/// 月柱
	pub month_zhu: Zhu<T>,
	/// 日柱
	pub day_zhu: Zhu<T>,
	/// 时柱
	pub hour_zhu: Zhu<T>,
	/// 日主（日柱天干）
	pub rizhu: TianGan,
}

impl<T: crate::pallet::Config> PartialEq for SiZhu<T> {
	fn eq(&self, other: &Self) -> bool {
		self.year_zhu == other.year_zhu &&
		self.month_zhu == other.month_zhu &&
		self.day_zhu == other.day_zhu &&
		self.hour_zhu == other.hour_zhu &&
		self.rizhu == other.rizhu
	}
}

impl<T: crate::pallet::Config> Eq for SiZhu<T> {}

/// 单步大运
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct DaYunStep<T: crate::pallet::Config> {
	/// 大运干支
	pub ganzhi: GanZhi,
	/// 起始年龄
	pub start_age: u8,
	/// 结束年龄
	pub end_age: u8,
	/// 起始年份
	pub start_year: u16,
	/// 结束年份
	pub end_year: u16,
	/// 天干十神
	pub tiangan_shishen: ShiShen,
	/// 藏干十神列表
	pub canggan_shishen: BoundedVec<ShiShen, T::MaxCangGan>,
}

impl<T: crate::pallet::Config> PartialEq for DaYunStep<T> {
	fn eq(&self, other: &Self) -> bool {
		self.ganzhi == other.ganzhi &&
		self.start_age == other.start_age &&
		self.end_age == other.end_age &&
		self.start_year == other.start_year &&
		self.end_year == other.end_year &&
		self.tiangan_shishen == other.tiangan_shishen &&
		self.canggan_shishen.iter().zip(other.canggan_shishen.iter()).all(|(a, b)| a == b)
	}
}

impl<T: crate::pallet::Config> Eq for DaYunStep<T> {}

/// 大运信息
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct DaYunInfo<T: crate::pallet::Config> {
	/// 起运年龄
	pub qiyun_age: u8,
	/// 起运年份
	pub qiyun_year: u16,
	/// 是否顺排（true=顺，false=逆）
	pub is_shun: bool,
	/// 大运列表（10-12步）
	pub dayun_list: BoundedVec<DaYunStep<T>, T::MaxDaYunSteps>,
}

impl<T: crate::pallet::Config> PartialEq for DaYunInfo<T> {
	fn eq(&self, other: &Self) -> bool {
		self.qiyun_age == other.qiyun_age &&
		self.qiyun_year == other.qiyun_year &&
		self.is_shun == other.is_shun &&
		self.dayun_list.iter().zip(other.dayun_list.iter()).all(|(a, b)| a == b)
	}
}

impl<T: crate::pallet::Config> Eq for DaYunInfo<T> {}

/// 五行强度
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub struct WuXingStrength {
	/// 金的强度
	pub jin: u32,
	/// 木的强度
	pub mu: u32,
	/// 水的强度
	pub shui: u32,
	/// 火的强度
	pub huo: u32,
	/// 土的强度
	pub tu: u32,
}

impl WuXingStrength {
	/// 添加五行强度
	pub fn add_element(&mut self, wuxing: WuXing, value: u32) {
		match wuxing {
			WuXing::Jin => self.jin += value,
			WuXing::Mu => self.mu += value,
			WuXing::Shui => self.shui += value,
			WuXing::Huo => self.huo += value,
			WuXing::Tu => self.tu += value,
		}
	}
}

/// 完整八字信息
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct BaziChart<T: crate::pallet::Config> {
	/// 所有者账户
	pub owner: T::AccountId,
	/// 命盘名称（可选，最大32字节UTF-8，如"张三"、"父亲命盘"等）
	pub name: BoundedVec<u8, ConstU32<32>>,

	// ===== 隐私控制字段 (Phase 1.2.4 新增) =====
	/// 隐私模式 (0=Public, 1=Partial, 2=Private)
	/// - Public: 所有数据明文存储
	/// - Partial: 计算数据明文 + 敏感数据加密（推荐）
	/// - Private: 所有数据加密
	pub privacy_mode: pallet_divination_privacy::types::PrivacyMode,
	/// 加密字段标记（位标志：bit 0=姓名, bit 1=出生日期, bit 2=性别, bit 3=经度）
	pub encrypted_fields: Option<u8>,
	/// 敏感数据哈希（用于完整性验证）
	pub sensitive_data_hash: Option<[u8; 32]>,

	// ===== 出生信息（Private 模式时为 None）=====
	/// 出生时间
	pub birth_time: Option<BirthTime>,
	/// 输入日历类型（记录原始输入是公历还是农历）
	/// 用于前端显示，不影响八字计算
	pub input_calendar_type: Option<InputCalendarType>,
	/// 性别（Private 模式时为 None）
	pub gender: Option<Gender>,
	/// 子时模式
	pub zishi_mode: Option<ZiShiMode>,
	/// 出生地经度（可选，1/100000 度，如 116.40000° → 11640000）
	/// 当有值时，使用真太阳时修正时辰；为 None 时，不使用真太阳时
	pub longitude: Option<i32>,

	// ===== 计算数据（Private 模式时为 None）=====
	/// 四柱
	pub sizhu: Option<SiZhu<T>>,
	/// 大运
	pub dayun: Option<DaYunInfo<T>>,
	/// 五行强度
	pub wuxing_strength: Option<WuXingStrength>,
	/// 喜用神
	pub xiyong_shen: Option<WuXing>,

	/// 创建时间戳（区块号）
	pub timestamp: u64,
}

impl<T: crate::pallet::Config> BaziChart<T> {
	/// 检查是否有计算数据（用于判断是否可以解读）
	///
	/// 当四柱和大运数据都存在时返回 true
	pub fn has_calculation_data(&self) -> bool {
		self.sizhu.is_some() && self.dayun.is_some()
	}

	/// 检查是否可以进行解读（非 Private 模式且有计算数据）
	///
	/// Private 模式下所有计算数据加密，无法直接解读
	pub fn can_interpret(&self) -> bool {
		self.privacy_mode != pallet_divination_privacy::types::PrivacyMode::Private
			&& self.has_calculation_data()
	}

	/// 检查是否公开（向后兼容）
	///
	/// 仅 Public 模式返回 true
	pub fn is_public(&self) -> bool {
		self.privacy_mode == pallet_divination_privacy::types::PrivacyMode::Public
	}

	/// 获取四柱数据（如果可用）
	pub fn get_sizhu(&self) -> Option<&SiZhu<T>> {
		self.sizhu.as_ref()
	}

	/// 获取大运数据（如果可用）
	pub fn get_dayun(&self) -> Option<&DaYunInfo<T>> {
		self.dayun.as_ref()
	}

	/// 获取性别（如果可用）
	pub fn get_gender(&self) -> Option<Gender> {
		self.gender
	}
}

impl<T: crate::pallet::Config> PartialEq for BaziChart<T> {
	fn eq(&self, other: &Self) -> bool {
		self.owner == other.owner &&
		self.name == other.name &&
		self.privacy_mode == other.privacy_mode &&
		self.encrypted_fields == other.encrypted_fields &&
		self.sensitive_data_hash == other.sensitive_data_hash &&
		self.birth_time == other.birth_time &&
		self.input_calendar_type == other.input_calendar_type &&
		self.gender == other.gender &&
		self.zishi_mode == other.zishi_mode &&
		self.longitude == other.longitude &&
		self.sizhu == other.sizhu &&
		self.dayun == other.dayun &&
		self.wuxing_strength == other.wuxing_strength &&
		self.xiyong_shen == other.xiyong_shen &&
		self.timestamp == other.timestamp
	}
}

impl<T: crate::pallet::Config> Eq for BaziChart<T> {}

// ================================
// 加密存储类型定义
// ================================

/// 四柱干支索引（8 bytes）
///
/// 仅保存四柱的干支索引，不包含任何敏感信息（如出生时间）
/// 这个索引足以进行命理计算，但无法反推出具体出生时间
///
/// # 安全特性
/// - 无法反推出生时间（只有干支索引）
/// - 支持 Runtime API 实时计算解盘
/// - 配合加密数据实现隐私保护
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
pub struct SiZhuIndex {
	/// 年柱天干索引 (0-9)
	pub year_gan: u8,
	/// 年柱地支索引 (0-11)
	pub year_zhi: u8,
	/// 月柱天干索引 (0-9)
	pub month_gan: u8,
	/// 月柱地支索引 (0-11)
	pub month_zhi: u8,
	/// 日柱天干索引 (0-9)
	pub day_gan: u8,
	/// 日柱地支索引 (0-11)
	pub day_zhi: u8,
	/// 时柱天干索引 (0-9)
	pub hour_gan: u8,
	/// 时柱地支索引 (0-11)
	pub hour_zhi: u8,
}

impl SiZhuIndex {
	/// 从四柱创建索引
	pub fn from_sizhu<T: crate::pallet::Config>(sizhu: &SiZhu<T>) -> Self {
		Self {
			year_gan: sizhu.year_zhu.ganzhi.gan.0,
			year_zhi: sizhu.year_zhu.ganzhi.zhi.0,
			month_gan: sizhu.month_zhu.ganzhi.gan.0,
			month_zhi: sizhu.month_zhu.ganzhi.zhi.0,
			day_gan: sizhu.day_zhu.ganzhi.gan.0,
			day_zhi: sizhu.day_zhu.ganzhi.zhi.0,
			hour_gan: sizhu.hour_zhu.ganzhi.gan.0,
			hour_zhi: sizhu.hour_zhu.ganzhi.zhi.0,
		}
	}

	/// 获取年柱干支
	pub fn year_ganzhi(&self) -> GanZhi {
		GanZhi {
			gan: TianGan(self.year_gan),
			zhi: DiZhi(self.year_zhi),
		}
	}

	/// 获取月柱干支
	pub fn month_ganzhi(&self) -> GanZhi {
		GanZhi {
			gan: TianGan(self.month_gan),
			zhi: DiZhi(self.month_zhi),
		}
	}

	/// 获取日柱干支
	pub fn day_ganzhi(&self) -> GanZhi {
		GanZhi {
			gan: TianGan(self.day_gan),
			zhi: DiZhi(self.day_zhi),
		}
	}

	/// 获取时柱干支
	pub fn hour_ganzhi(&self) -> GanZhi {
		GanZhi {
			gan: TianGan(self.hour_gan),
			zhi: DiZhi(self.hour_zhi),
		}
	}

	/// 获取日主天干
	pub fn rizhu(&self) -> TianGan {
		TianGan(self.day_gan)
	}

	/// 验证索引有效性
	pub fn is_valid(&self) -> bool {
		self.year_gan < 10 && self.year_zhi < 12 &&
		self.month_gan < 10 && self.month_zhi < 12 &&
		self.day_gan < 10 && self.day_zhi < 12 &&
		self.hour_gan < 10 && self.hour_zhi < 12
	}
}

/// 加密的八字命盘
///
/// # 存储结构
/// - `sizhu_index`: 四柱索引（8 bytes）- 明文存储，用于计算
/// - `gender`: 性别（1 byte）- 明文存储，用于大运计算
/// - `encrypted_data`: 加密数据（最大 256 bytes）- 包含敏感信息
/// - `data_hash`: 数据哈希（32 bytes）- 用于验证解密正确性
///
/// # 加密数据内容（前端加密）
/// - 出生时间（年月日时分）
/// - 子时模式
/// - 大运信息
/// - 其他敏感数据
///
/// # 安全特性
/// - 出生时间永不明文存储
/// - 用户自己管理密钥（钱包签名派生）
/// - Runtime API 仍可基于 sizhu_index 免费计算解盘
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct EncryptedBaziChart<T: crate::pallet::Config> {
	/// 所有者账户
	pub owner: T::AccountId,
	/// 四柱干支索引（明文，用于计算）
	pub sizhu_index: SiZhuIndex,
	/// 性别（明文，用于大运计算）
	pub gender: Gender,
	/// 加密的敏感数据（AES-256-GCM 加密）
	pub encrypted_data: BoundedVec<u8, ConstU32<256>>,
	/// 原始数据的 Blake2-256 哈希（用于验证解密正确性）
	pub data_hash: [u8; 32],
	/// 创建时间戳（区块号）
	pub created_at: u32,
}

impl<T: crate::pallet::Config> PartialEq for EncryptedBaziChart<T> {
	fn eq(&self, other: &Self) -> bool {
		self.owner == other.owner &&
		self.sizhu_index == other.sizhu_index &&
		self.gender == other.gender &&
		self.encrypted_data == other.encrypted_data &&
		self.data_hash == other.data_hash &&
		self.created_at == other.created_at
	}
}

impl<T: crate::pallet::Config> Eq for EncryptedBaziChart<T> {}

// ================================
// 统一输入类型定义
// ================================

/// 八字创建输入类型（统一接口）
///
/// 支持三种输入方式，最终都会转换为四柱进行计算：
/// - Solar: 公历日期输入（最常用）
/// - Lunar: 农历日期输入（适合传统用户）
/// - SiZhu: 四柱直接输入（适合专业用户）
///
/// # 使用示例
///
/// ```ignore
/// // 公历输入
/// let input = BaziInputType::Solar {
///     year: 1990, month: 5, day: 15,
///     hour: 14, minute: 30,
/// };
///
/// // 农历输入
/// let input = BaziInputType::Lunar {
///     year: 2024, month: 1, day: 1,
///     is_leap_month: false,
///     hour: 12, minute: 0,
/// };
///
/// // 四柱直接输入
/// let input = BaziInputType::SiZhu {
///     year_gz: 0,   // 甲子
///     month_gz: 2,  // 丙寅
///     day_gz: 4,    // 戊辰
///     hour_gz: 0,   // 甲子
///     birth_year: 1984,  // 用于大运起运年份
/// };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
pub enum BaziInputType {
	/// 公历日期输入
	///
	/// 最常用的输入方式，系统内部按节气划分月份
	Solar {
		/// 公历年份 (1900-2100)
		year: u16,
		/// 公历月份 (1-12)
		month: u8,
		/// 公历日期 (1-31)
		day: u8,
		/// 小时 (0-23)
		hour: u8,
		/// 分钟 (0-59)
		minute: u8,
	},

	/// 农历日期输入
	///
	/// 适合习惯使用农历的传统用户
	/// 系统会自动转换为公历，然后按节气计算
	Lunar {
		/// 农历年份 (1901-2100)
		year: u16,
		/// 农历月份 (1-12)
		month: u8,
		/// 农历日期 (1-30)
		day: u8,
		/// 是否闰月
		is_leap_month: bool,
		/// 小时 (0-23)
		hour: u8,
		/// 分钟 (0-59)
		minute: u8,
	},

	/// 四柱直接输入
	///
	/// 适合专业用户直接输入四柱干支
	/// 跳过日期验证，直接使用提供的干支组合
	SiZhu {
		/// 年柱干支索引 (0-59)
		year_gz: u8,
		/// 月柱干支索引 (0-59)
		month_gz: u8,
		/// 日柱干支索引 (0-59)
		day_gz: u8,
		/// 时柱干支索引 (0-59)
		hour_gz: u8,
		/// 出生年份（用于计算大运起运年份）
		birth_year: u16,
	},
}

impl BaziInputType {
	/// 验证输入有效性
	pub fn is_valid(&self) -> bool {
		match self {
			BaziInputType::Solar { year, month, day, hour, minute } => {
				*year >= 1900 && *year <= 2100 &&
				*month >= 1 && *month <= 12 &&
				*day >= 1 && *day <= 31 &&
				*hour < 24 &&
				*minute < 60
			}
			BaziInputType::Lunar { year, month, day, is_leap_month: _, hour, minute } => {
				*year >= 1901 && *year <= 2100 &&
				*month >= 1 && *month <= 12 &&
				*day >= 1 && *day <= 30 &&
				*hour < 24 &&
				*minute < 60
			}
			BaziInputType::SiZhu { year_gz, month_gz, day_gz, hour_gz, birth_year } => {
				*year_gz < 60 && *month_gz < 60 && *day_gz < 60 && *hour_gz < 60 &&
				*birth_year >= 1900 && *birth_year <= 2100
			}
		}
	}

	/// 获取出生年份（用于大运计算）
	pub fn get_birth_year(&self) -> u16 {
		match self {
			BaziInputType::Solar { year, .. } => *year,
			BaziInputType::Lunar { year, .. } => *year,
			BaziInputType::SiZhu { birth_year, .. } => *birth_year,
		}
	}
}

/// 四柱直接输入结构（用于测试和内部使用）
///
/// 与 BaziInputType::SiZhu 功能相同，但作为独立结构便于测试
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct SiZhuDirectInput {
	/// 年柱干支索引 (0-59)
	pub year_gz: u8,
	/// 月柱干支索引 (0-59)
	pub month_gz: u8,
	/// 日柱干支索引 (0-59)
	pub day_gz: u8,
	/// 时柱干支索引 (0-59)
	pub hour_gz: u8,
}

impl SiZhuDirectInput {
	/// 创建四柱直接输入
	pub fn new(year_gz: u8, month_gz: u8, day_gz: u8, hour_gz: u8) -> Self {
		Self { year_gz, month_gz, day_gz, hour_gz }
	}

	/// 验证有效性
	pub fn is_valid(&self) -> bool {
		self.year_gz < 60 && self.month_gz < 60 && self.day_gz < 60 && self.hour_gz < 60
	}

	/// 转换为 BaziInputType
	pub fn to_input_type(self, birth_year: u16) -> BaziInputType {
		BaziInputType::SiZhu {
			year_gz: self.year_gz,
			month_gz: self.month_gz,
			day_gz: self.day_gz,
			hour_gz: self.hour_gz,
			birth_year,
		}
	}
}

// ================================
// 空亡信息类型定义
// ================================

/// 空亡信息
///
/// 包含四柱各自的旬空地支和是否落空亡的判断
///
/// # 空亡规则
///
/// 六十甲子每十个为一旬，每旬有两个地支空缺：
/// - 甲子旬（甲子到癸酉）: 戌亥空
/// - 甲戌旬（甲戌到癸未）: 申酉空
/// - 甲申旬（甲申到癸巳）: 午未空
/// - 甲午旬（甲午到癸卯）: 辰巳空
/// - 甲辰旬（甲辰到癸丑）: 寅卯空
/// - 甲寅旬（甲寅到癸亥）: 子丑空
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct KongWangInfo {
	/// 年柱的旬空地支对
	pub year_kongwang: (DiZhi, DiZhi),
	/// 月柱的旬空地支对
	pub month_kongwang: (DiZhi, DiZhi),
	/// 日柱的旬空地支对（最重要）
	pub day_kongwang: (DiZhi, DiZhi),
	/// 时柱的旬空地支对
	pub hour_kongwang: (DiZhi, DiZhi),
	/// 年柱地支是否落空亡
	pub year_is_kong: bool,
	/// 月柱地支是否落空亡
	pub month_is_kong: bool,
	/// 日柱地支是否落空亡
	pub day_is_kong: bool,
	/// 时柱地支是否落空亡
	pub hour_is_kong: bool,
}

// ================================
// 星运信息类型定义
// ================================

/// 星运信息（十二长生）
///
/// 包含日主在四柱各支的十二长生状态
///
/// # 十二长生
///
/// 表示天干在地支中的生旺死绝状态：
/// - 旺相: 长生、冠带、临官、帝旺
/// - 衰败: 衰、病、死、墓、绝
/// - 中性: 沐浴、胎、养
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct XingYunInfo {
	/// 日主在年支的十二长生
	pub year_changsheng: ShiErChangSheng,
	/// 日主在月支的十二长生（最重要，月令）
	pub month_changsheng: ShiErChangSheng,
	/// 日主在日支的十二长生（坐支）
	pub day_changsheng: ShiErChangSheng,
	/// 日主在时支的十二长生
	pub hour_changsheng: ShiErChangSheng,
}

// ================================
// 神煞相关类型定义
// ================================

/// 四柱位置枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum SiZhuPosition {
	/// 年柱
	Year = 0,
	/// 月柱
	Month = 1,
	/// 日柱
	Day = 2,
	/// 时柱
	Hour = 3,
}

impl SiZhuPosition {
	/// 获取位置名称
	pub fn name(&self) -> &'static str {
		match self {
			SiZhuPosition::Year => "年柱",
			SiZhuPosition::Month => "月柱",
			SiZhuPosition::Day => "日柱",
			SiZhuPosition::Hour => "时柱",
		}
	}
}

/// 神煞吉凶性质
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ShenShaNature {
	/// 吉神
	JiShen = 0,
	/// 凶神
	XiongShen = 1,
	/// 中性（吉凶参半或视情况而定）
	Neutral = 2,
}

impl ShenShaNature {
	/// 获取性质名称
	pub fn name(&self) -> &'static str {
		match self {
			ShenShaNature::JiShen => "吉神",
			ShenShaNature::XiongShen => "凶神",
			ShenShaNature::Neutral => "中性",
		}
	}
}

/// 神煞条目
///
/// 包含神煞类型、出现位置、吉凶属性
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ShenShaEntry {
	/// 神煞类型
	pub shensha: crate::calculations::shensha::ShenSha,
	/// 出现位置（年/月/日/时柱）
	pub position: SiZhuPosition,
	/// 吉凶属性
	pub nature: ShenShaNature,
}

// ================================
// 注：多方授权加密存储类型已迁移至 pallet-divination-privacy
// 包括: AccessRole, AccessScope, EncryptedKeyEntry,
//       MultiKeyEncryptedBaziChart, ServiceProviderType, ServiceProvider
// 请使用: pallet_divination_privacy::types::{...}
// ================================
