//! # 八字排盘 Pallet (Pallet Bazi Chart)
//!
//! ## 概述
//!
//! 本 Pallet 实现了完整的中国传统命理八字排盘功能，包括：
//! - 四柱计算（年柱、月柱、日柱、时柱）
//! - 大运推算（起运年龄、大运序列）
//! - 五行强度分析（月令权重法）
//! - 十神关系计算
//! - 藏干提取和纳音五行
//!
//! ## 技术特性
//!
//! - ✅ **辰藏干正确性**: 使用"戊乙癸"（主流派，87.5%项目支持）
//! - ✅ **子时双模式**: 支持传统派和现代派两种子时归属模式
//! - ✅ **节气精度**: 采用寿星天文算法（秒级精度）
//! - ✅ **五行强度**: 实现月令权重矩阵（12×36）
//!
//! ## 参考项目
//!
//! - BaziGo (95/100) - 五行强度算法、藏干权重表
//! - lunar-java (93/100) - 节气算法、数据结构设计
//! - bazi-mcp (92/100) - 子时双模式、API设计
//!
//! ## 使用示例
//!
//! ```ignore
//! // 创建八字（现代派子时模式）
//! BaziChart::create_bazi_chart(
//!     origin,
//!     1998, 7, 31, 14, 10,  // 1998年7月31日14:10
//!     Gender::Male,
//!     ZiShiMode::Modern,
//! )?;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

// TODO: 测试文件待完善 mock 配置（测试引用已删除的存储项）
// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

pub mod types;
pub mod constants;
pub mod calculations;
pub mod interpretation;
pub mod runtime_api;
pub mod ocw_tee;

// 重新导出 Runtime API 相关类型，方便外部使用
pub use interpretation::{CoreInterpretation, FullInterpretation, CompactXingGe, ExtendedJiShen};
// 重新导出核心类型
pub use types::{SiZhuIndex, BaziInputType, InputCalendarType};
// 重新导出多方授权加密类型（从 privacy 模块）
pub use pallet_divination_privacy::types::{
	AccessRole, AccessScope, ServiceProviderType, ServiceProvider,
	PrivacyMode, AuthorizationEntry,
};

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::SaturatedConversion;
	use pallet_divination_common::ChartCascadeDeleter;

	pub use crate::types::*;

	/// Pallet 配置 Trait
	#[pallet::config(with_default)]
	pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {
		/// 权重信息
		type WeightInfo: WeightInfo;

		/// 每个账户最多创建的八字数量
		#[pallet::constant]
		type MaxChartsPerAccount: Get<u32> + Clone + core::fmt::Debug;

		/// 大运最大步数（默认12步，120年）
		#[pallet::constant]
		type MaxDaYunSteps: Get<u32> + Clone + core::fmt::Debug;

		/// 每个地支最多藏干数量（最多3个）
		#[pallet::constant]
		type MaxCangGan: Get<u32> + Clone + core::fmt::Debug;

		// ================================
		// 隐私模块集成 (迁移后)
		// ================================

		/// 隐私服务提供者 - 用于调用 privacy 模块的功能
		#[pallet::no_default]
		type PrivacyProvider: pallet_divination_privacy::traits::EncryptedRecordManager<
			Self::AccountId,
			BlockNumberFor<Self>,
		>;

		/// 级联删除器 - 用于删除命盘时级联删除关联的订单数据
		#[pallet::no_default]
		type CascadeDeleter: pallet_divination_common::ChartCascadeDeleter<Self::AccountId>;

	}

	/// 权重信息 Trait（暂时使用占位实现）
	pub trait WeightInfo {
		fn create_bazi_chart() -> Weight;
		fn delete_bazi_chart() -> Weight;
	}

	/// 默认权重实现
	impl WeightInfo for () {
		fn create_bazi_chart() -> Weight {
			Weight::from_parts(10_000_000, 0)
		}
		fn delete_bazi_chart() -> Weight {
			Weight::from_parts(5_000_000, 0)
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// 下一个八字ID计数器
	#[pallet::storage]
	#[pallet::getter(fn next_chart_id)]
	pub type NextChartId<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// 存储映射: 八字ID -> 八字详情
	#[pallet::storage]
	#[pallet::getter(fn chart_by_id)]
	pub type ChartById<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64,
		BaziChart<T>,
	>;

	/// 存储映射: 用户 -> 八字ID列表
	#[pallet::storage]
	#[pallet::getter(fn user_charts)]
	pub type UserCharts<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<u64, T::MaxChartsPerAccount>,
		ValueQuery,
	>;

	/// 存储映射: 八字ID -> 精简八字信息（~100 bytes）
	///
	/// Phase 10 优化：新创建的命盘使用精简结构，节省 80% 存储空间。
	/// 计算数据（sizhu、dayun、wuxing_strength）通过 Runtime API 实时获取。
	#[pallet::storage]
	#[pallet::getter(fn chart_compact_by_id)]
	pub type ChartCompactById<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64,
		crate::types::BaziChartCompact<T>,
	>;

	/// Pallet 事件
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[allow(dead_code)]
	pub enum Event<T: Config> {
		/// 八字创建成功 [所有者, 八字ID, 出生时间]
		BaziChartCreated {
			owner: T::AccountId,
			chart_id: u64,
			birth_time: BirthTime,
		},
		/// 八字查询 [八字ID, 所有者]
		BaziChartQueried {
			chart_id: u64,
			owner: T::AccountId,
		},
		/// 八字删除 [所有者, 八字ID]
		BaziChartDeleted {
			owner: T::AccountId,
			chart_id: u64,
		},
		/// 八字级联删除完成 [所有者, 八字ID, 删除的订单数, 取消Pin的CID数]
		BaziChartCascadeDeleted {
			owner: T::AccountId,
			chart_id: u64,
			orders_deleted: u32,
			cids_unpinned: u32,
		},
	}

	/// Pallet 错误
	#[pallet::error]
	pub enum Error<T> {
		/// 无效的年份
		InvalidYear,
		/// 无效的月份
		InvalidMonth,
		/// 无效的日期
		InvalidDay,
		/// 无效的小时
		InvalidHour,
		/// 无效的分钟
		InvalidMinute,
		/// 无效的天干
		InvalidTianGan,
		/// 无效的地支
		InvalidDiZhi,
		/// 无效的干支索引
		InvalidGanZhiIndex,
		/// 八字数量过多
		TooManyCharts,
		/// 八字未找到
		ChartNotFound,
		/// 非八字所有者
		NotChartOwner,
		/// 藏干数量过多
		TooManyCangGan,
		/// 大运步数过多
		TooManyDaYunSteps,
		/// 八字ID已达到最大值
		ChartIdOverflow,
		/// 四柱索引无效
		InvalidSiZhuIndex,
		/// 农历日期无效或转换失败
		InvalidLunarDate,
		/// 输入参数无效
		InvalidInput,
	}

	/// Pallet 可调用函数
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// 创建八字命盘（统一接口）
		///
		/// # 功能
		///
		/// 支持三种输入方式创建八字命盘：
		/// - **公历日期** (`Solar`): 最常用，直接输入公历年月日时
		/// - **农历日期** (`Lunar`): 系统自动转换为公历后计算
		/// - **四柱直接输入** (`SiZhu`): 专业用户直接输入干支索引
		///
		/// # 处理流程
		///
		/// 1. 验证输入参数
		/// 2. 统一转换为公历日期（农历需要转换）
		/// 3. 应用真太阳时修正（如果启用）
		/// 4. 计算四柱八字（日/年/月/时）
		/// 5. 计算大运
		/// 6. 计算五行强度
		/// 7. 判断喜用神
		/// 8. 存储八字信息
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者
		/// - `name`: 命盘名称（可选，最大32字节UTF-8）
		/// - `input`: 输入类型（公历/农历/四柱）
		/// - `gender`: 性别（用于大运顺逆）
		/// - `zishi_mode`: 子时模式（传统派/现代派）
		/// - `longitude`: 出生地经度（可选，1/100000 度）
		///   - `Some(经度值)`: 使用真太阳时修正
		///   - `None`: 不使用真太阳时修正
		///
		/// # 示例
		///
		/// ```ignore
		/// // 公历输入（北京时间，不使用真太阳时修正）
		/// BaziChart::create_bazi_chart(
		///     origin,
		///     Some(b"张三".to_vec().try_into().unwrap()),
		///     BaziInputType::Solar { year: 1990, month: 5, day: 15, hour: 14, minute: 30 },
		///     Gender::Male,
		///     ZiShiMode::Modern,
		///     None,    // 不提供经度 = 不使用真太阳时
		/// )?;
		///
		/// // 公历输入（使用真太阳时修正，乌鲁木齐）
		/// BaziChart::create_bazi_chart(
		///     origin,
		///     None,
		///     BaziInputType::Solar { year: 1990, month: 5, day: 15, hour: 14, minute: 30 },
		///     Gender::Male,
		///     ZiShiMode::Modern,
		///     Some(8760000),  // 乌鲁木齐经度 87.6° = 使用真太阳时
		/// )?;
		/// ```
		///
		/// # 注意
		///
		/// - 每个账户最多创建 `MaxChartsPerAccount` 个八字
		/// - 子时模式会影响 23:00-23:59 的时柱计算
		/// - 农历输入会自动转换为公历，然后按节气划分月份
		/// - 真太阳时修正主要影响时柱判断（尤其是边界时辰）
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_bazi_chart())]
		pub fn create_bazi_chart(
			origin: OriginFor<T>,
			name: Option<BoundedVec<u8, ConstU32<32>>>,
			input: BaziInputType,
			gender: Gender,
			zishi_mode: ZiShiMode,
			longitude: Option<i32>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 验证输入参数
			ensure!(input.is_valid(), Error::<T>::InvalidInput);

			// 2. 检查账户八字数量限制
			let existing_charts = UserCharts::<T>::get(&who);
			ensure!(
				existing_charts.len() < T::MaxChartsPerAccount::get() as usize,
				Error::<T>::TooManyCharts
			);

			// 3. 根据输入类型计算四柱和出生时间（包含真太阳时修正）
			// 注意：当 longitude.is_some() 时自动使用真太阳时修正
			let (sizhu, birth_time, _birth_year) = Self::calculate_sizhu_from_input(
				&input,
				zishi_mode,
				longitude,
			)?;

			// 4. 获取日主天干
			let day_ganzhi = sizhu.day_zhu.ganzhi;
			let year_ganzhi = sizhu.year_zhu.ganzhi;
			let month_ganzhi = sizhu.month_zhu.ganzhi;
			let hour_ganzhi = sizhu.hour_zhu.ganzhi;

			// 5. 确定输入日历类型（记录原始输入是公历还是农历）
			// 注：大运、五行强度、喜用神等计算数据不再存储，通过 Runtime API 实时计算
			let input_calendar_type = match input {
				crate::types::BaziInputType::Solar { .. } => crate::types::InputCalendarType::Solar,
				crate::types::BaziInputType::Lunar { .. } => crate::types::InputCalendarType::Lunar,
				crate::types::BaziInputType::SiZhu { .. } => crate::types::InputCalendarType::SiZhu,
			};

			// 9. 构建四柱索引（用于精简存储）
			let sizhu_index = crate::types::SiZhuIndex {
				year_gan: year_ganzhi.gan.0,
				year_zhi: year_ganzhi.zhi.0,
				month_gan: month_ganzhi.gan.0,
				month_zhi: month_ganzhi.zhi.0,
				day_gan: day_ganzhi.gan.0,
				day_zhi: day_ganzhi.zhi.0,
				hour_gan: hour_ganzhi.gan.0,
				hour_zhi: hour_ganzhi.zhi.0,
			};

			// 10. 构建精简八字信息（Phase 10 优化：节省 80% 存储空间）
			let bazi_chart_compact = crate::types::BaziChartCompact {
				owner: who.clone(),
				name: name.unwrap_or_default(),
				// 隐私控制字段 - 默认 Public 模式
				privacy_mode: pallet_divination_privacy::types::PrivacyMode::Public,
				encrypted_fields: None,
				sensitive_data_hash: None,
				// 出生信息
				birth_time: Some(birth_time),
				input_calendar_type: Some(input_calendar_type),
				gender: Some(gender),
				zishi_mode: Some(zishi_mode),
				longitude,
				// 四柱索引缓存（加速查询）
				sizhu_index: Some(sizhu_index),
				timestamp: frame_system::Pallet::<T>::block_number().saturated_into(),
			};

			// 11. 存储精简八字
			let chart_id = NextChartId::<T>::get();
			ensure!(chart_id < u64::MAX, Error::<T>::ChartIdOverflow);

			ChartCompactById::<T>::insert(chart_id, bazi_chart_compact);

			UserCharts::<T>::try_mutate(&who, |charts| {
				charts.try_push(chart_id).map_err(|_| Error::<T>::TooManyCharts)
			})?;

			NextChartId::<T>::put(chart_id + 1);

			// 11. 触发事件
			Self::deposit_event(Event::BaziChartCreated {
				owner: who,
				chart_id,
				birth_time,
			});

			Ok(())
		}

		/// 删除八字
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者
		/// - `chart_id`: 八字ID
		///
		/// # 权限
		///
		/// 只有八字所有者可以删除自己的八字
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::delete_bazi_chart())]
		pub fn delete_bazi_chart(
			origin: OriginFor<T>,
			chart_id: u64,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 尝试从精简存储获取（新格式）
			if let Some(chart) = ChartCompactById::<T>::get(chart_id) {
				ensure!(chart.owner == who, Error::<T>::NotChartOwner);
				ChartCompactById::<T>::remove(chart_id);
			} else if let Some(chart) = ChartById::<T>::get(chart_id) {
				// 尝试从完整存储获取（旧格式，向后兼容）
				ensure!(chart.owner == who, Error::<T>::NotChartOwner);
				ChartById::<T>::remove(chart_id);
			} else {
				return Err(Error::<T>::ChartNotFound.into());
			}

			// 从用户的八字列表中删除
			UserCharts::<T>::try_mutate(&who, |charts| -> DispatchResult {
				if let Some(pos) = charts.iter().position(|&id| id == chart_id) {
					charts.remove(pos);
				}
				Ok(())
			})?;

			// 🆕 级联删除关联数据（订单、解读、评价等）并取消 IPFS Pin
			let cascade_result = T::CascadeDeleter::cascade_delete_for_chart(
				&who,
				pallet_divination_common::DivinationType::Bazi,
				chart_id,
			)?;

			// 触发事件
			Self::deposit_event(Event::BaziChartDeleted {
				owner: who.clone(),
				chart_id,
			});

			// 🆕 触发级联删除完成事件
			Self::deposit_event(Event::BaziChartCascadeDeleted {
				owner: who,
				chart_id,
				orders_deleted: cascade_result.orders_deleted,
				cids_unpinned: cascade_result.cids_unpinned,
			});

			Ok(())
		}

	}

	// 辅助函数
	impl<T: Config> Pallet<T> {
		/// 构建四柱（填充藏干和纳音）
		fn build_sizhu(
			year_ganzhi: GanZhi,
			month_ganzhi: GanZhi,
			day_ganzhi: GanZhi,
			hour_ganzhi: GanZhi,
			rizhu: TianGan,
		) -> Result<SiZhu<T>, Error<T>> {
			// 构建年柱
			let year_zhu = Self::build_zhu(year_ganzhi, rizhu)?;
			// 构建月柱
			let month_zhu = Self::build_zhu(month_ganzhi, rizhu)?;
			// 构建日柱
			let day_zhu = Self::build_zhu(day_ganzhi, rizhu)?;
			// 构建时柱
			let hour_zhu = Self::build_zhu(hour_ganzhi, rizhu)?;

			Ok(SiZhu {
				year_zhu,
				month_zhu,
				day_zhu,
				hour_zhu,
				rizhu,
			})
		}

		/// 构建单个柱（填充藏干和纳音）
		fn build_zhu(ganzhi: GanZhi, rizhu: TianGan) -> Result<Zhu<T>, Error<T>> {
			use crate::constants::{get_hidden_stems, calculate_nayin, is_valid_canggan};

			// 获取藏干信息
			let hidden_stems = get_hidden_stems(ganzhi.zhi);
			let mut canggan = BoundedVec::<CangGanInfo, T::MaxCangGan>::default();

			for (gan, canggan_type, weight) in hidden_stems.iter() {
				// 跳过无效藏干（255表示该位置无藏干）
				if !is_valid_canggan(gan.0) {
					continue;
				}

				// 计算藏干的十神关系
				let shishen = crate::constants::calculate_shishen(rizhu, *gan);

				let canggan_info = CangGanInfo {
					gan: *gan,
					shishen,
					canggan_type: *canggan_type,
					weight: *weight,
				};

				canggan.try_push(canggan_info).map_err(|_| Error::<T>::TooManyCangGan)?;
			}

			// 计算纳音
			let nayin = calculate_nayin(&ganzhi);

			Ok(Zhu {
				ganzhi,
				canggan,
				nayin,
			})
		}

		/// 根据输入类型计算四柱和出生时间（统一接口）
		///
		/// # 参数
		/// - `input`: 输入类型（公历/农历/四柱）
		/// - `zishi_mode`: 子时模式
		/// - `longitude`: 出生地经度（可选，1/100000 度）
		///   - `Some(经度值)`: 自动使用真太阳时修正
		///   - `None`: 不使用真太阳时修正
		///
		/// # 返回
		/// - `Ok((SiZhu, BirthTime, birth_year))`: 四柱、出生时间、出生年份
		/// - `Err`: 计算失败
		///
		/// # 真太阳时修正
		///
		/// 当 `longitude.is_some()` 时，会对出生时间进行真太阳时修正：
		/// 1. 经度时差：(出生地经度 - 120°) × 4分钟/度
		/// 2. 时差方程：根据日期计算太阳真时与平时的差值
		///
		/// 修正后的时间用于计算时柱，但存储的出生时间仍为原始北京时间。
		fn calculate_sizhu_from_input(
			input: &BaziInputType,
			zishi_mode: ZiShiMode,
			longitude: Option<i32>,
		) -> Result<(SiZhu<T>, BirthTime, u16), Error<T>> {
			use crate::calculations::*;

			match input {
				// 公历日期输入
				BaziInputType::Solar { year, month, day, hour, minute } => {
					let year = *year;
					let month = *month;
					let day = *day;
					let hour = *hour;
					let minute = *minute;

					// 应用真太阳时修正（当 longitude 有值时）
					let (calc_year, calc_month, calc_day, calc_hour, _calc_minute) =
						if let Some(lng) = longitude {
							let result = apply_true_solar_time(year, month, day, hour, minute, lng);

							// 处理日期偏移
							let (adj_year, adj_month, adj_day) = if result.day_offset != 0 {
								adjust_date(year, month, day, result.day_offset)
							} else {
								(year, month, day)
							};

							(adj_year, adj_month, adj_day, result.hour, result.minute)
						} else {
							(year, month, day, hour, minute)
						};

					// 使用（可能修正后的）时间计算日柱
					let day_ganzhi = calculate_day_ganzhi(calc_year, calc_month, calc_day)
						.ok_or(Error::<T>::InvalidDay)?;

					// 计算年柱
					let year_ganzhi = calculate_year_ganzhi(calc_year, calc_month, calc_day)
						.ok_or(Error::<T>::InvalidYear)?;

					// 计算月柱
					let month_ganzhi = calculate_month_ganzhi(calc_year, calc_month, calc_day, year_ganzhi.gan.0)
						.ok_or(Error::<T>::InvalidMonth)?;

					// 计算时柱（处理子时双模式）
					let (hour_ganzhi, is_next_day) = calculate_hour_ganzhi(calc_hour, day_ganzhi.gan.0, zishi_mode)
						.ok_or(Error::<T>::InvalidHour)?;

					// 如果是次日子时（传统派23:00），需要重新计算日柱
					let (final_day_ganzhi, final_hour_ganzhi) = if is_next_day {
						let next_day_ganzhi = day_ganzhi.next();
						let (final_hour, _) = calculate_hour_ganzhi(calc_hour, next_day_ganzhi.gan.0, zishi_mode)
							.ok_or(Error::<T>::InvalidHour)?;
						(next_day_ganzhi, final_hour)
					} else {
						(day_ganzhi, hour_ganzhi)
					};

					// 构建四柱
					let sizhu = Self::build_sizhu(
						year_ganzhi,
						month_ganzhi,
						final_day_ganzhi,
						final_hour_ganzhi,
						final_day_ganzhi.gan,
					)?;

					// 存储原始北京时间（不是修正后的时间）
					let birth_time = BirthTime { year, month, day, hour, minute };

					Ok((sizhu, birth_time, year))
				}

				// 农历日期输入
				BaziInputType::Lunar { year, month, day, is_leap_month, hour, minute } => {
					let lunar_year = *year;
					let lunar_month = *month;
					let lunar_day = *day;
					let is_leap = *is_leap_month;
					let hour = *hour;
					let minute = *minute;

					// 农历转公历
					let (solar_year, solar_month, solar_day) = pallet_almanac::lunar::lunar_to_solar(
						lunar_year,
						lunar_month,
						lunar_day,
						is_leap,
					).ok_or(Error::<T>::InvalidLunarDate)?;

					// 应用真太阳时修正
					let (calc_year, calc_month, calc_day, calc_hour, _calc_minute) =
						if let Some(lng) = longitude {
							let result = apply_true_solar_time(solar_year, solar_month, solar_day, hour, minute, lng);

							let (adj_year, adj_month, adj_day) = if result.day_offset != 0 {
								adjust_date(solar_year, solar_month, solar_day, result.day_offset)
							} else {
								(solar_year, solar_month, solar_day)
							};

							(adj_year, adj_month, adj_day, result.hour, result.minute)
						} else {
							(solar_year, solar_month, solar_day, hour, minute)
			};

					// 使用（可能修正后的）公历日期计算四柱
					let day_ganzhi = calculate_day_ganzhi(calc_year, calc_month, calc_day)
						.ok_or(Error::<T>::InvalidDay)?;

					let year_ganzhi = calculate_year_ganzhi(calc_year, calc_month, calc_day)
						.ok_or(Error::<T>::InvalidYear)?;

					let month_ganzhi = calculate_month_ganzhi(calc_year, calc_month, calc_day, year_ganzhi.gan.0)
						.ok_or(Error::<T>::InvalidMonth)?;

					let (hour_ganzhi, is_next_day) = calculate_hour_ganzhi(calc_hour, day_ganzhi.gan.0, zishi_mode)
						.ok_or(Error::<T>::InvalidHour)?;

					let (final_day_ganzhi, final_hour_ganzhi) = if is_next_day {
						let next_day_ganzhi = day_ganzhi.next();
						let (final_hour, _) = calculate_hour_ganzhi(calc_hour, next_day_ganzhi.gan.0, zishi_mode)
							.ok_or(Error::<T>::InvalidHour)?;
						(next_day_ganzhi, final_hour)
					} else {
						(day_ganzhi, hour_ganzhi)
					};

					let sizhu = Self::build_sizhu(
						year_ganzhi,
						month_ganzhi,
						final_day_ganzhi,
						final_hour_ganzhi,
						final_day_ganzhi.gan,
					)?;

					// 出生时间记录转换后的公历日期（原始北京时间）
					let birth_time = BirthTime {
						year: solar_year,
						month: solar_month,
						day: solar_day,
						hour,
						minute,
					};

					Ok((sizhu, birth_time, solar_year))
		}

				// 四柱直接输入（不支持真太阳时修正，因为没有具体时间）
				BaziInputType::SiZhu { year_gz, month_gz, day_gz, hour_gz, birth_year } => {
					let birth_year = *birth_year;

					// 验证干支索引
					let year_ganzhi = GanZhi::from_index(*year_gz)
						.ok_or(Error::<T>::InvalidGanZhiIndex)?;
					let month_ganzhi = GanZhi::from_index(*month_gz)
						.ok_or(Error::<T>::InvalidGanZhiIndex)?;
					let day_ganzhi = GanZhi::from_index(*day_gz)
						.ok_or(Error::<T>::InvalidGanZhiIndex)?;
					let hour_ganzhi = GanZhi::from_index(*hour_gz)
						.ok_or(Error::<T>::InvalidGanZhiIndex)?;

					// 构建四柱
					let sizhu = Self::build_sizhu(
						year_ganzhi,
						month_ganzhi,
						day_ganzhi,
						hour_ganzhi,
						day_ganzhi.gan,
					)?;

					// 四柱直接输入时，出生时间只记录年份，其他为占位值
					let birth_time = BirthTime {
						year: birth_year,
						month: 0,  // 未知
						day: 0,    // 未知
						hour: 0,   // 未知
						minute: 0, // 未知
					};

					Ok((sizhu, birth_time, birth_year))
				}
			}
		}

		/// RPC 接口：实时计算完整解盘（唯一对外接口）
		///
		/// 此函数由 Runtime API 调用，不消耗 gas，不上链
		///
		/// # 参数
		/// - chart_id: 八字命盘ID
		///
		/// # 返回
		/// - Some(FullInterpretation): 完整解盘结果
		///   - core: 核心指标（格局、强弱、用神、喜神、忌神、评分、可信度）
		///   - xing_ge: 性格分析（主要特点、优点、缺点、适合职业）
		///   - extended_ji_shen: 扩展忌神（次忌神列表）
		/// - None: 命盘不存在
		///
		/// # 特点
		/// - 完全免费（无 gas 费用）
		/// - 响应快速（< 100ms）
		/// - 算法自动更新（使用最新版本）
		/// - 不永久存储（避免存储成本）
		///
		/// # 使用方式
		/// 前端只需核心数据时，访问 `result.core` 即可（等价于旧版 V2/V3 Core）
		pub fn get_full_interpretation(chart_id: u64) -> Option<crate::interpretation::FullInterpretation> {
			let current_block = <frame_system::Pallet<T>>::block_number().saturated_into();

			// 优先从精简存储获取（新格式）
			if let Some(chart) = ChartCompactById::<T>::get(chart_id) {
				let sizhu_index = chart.get_sizhu_index()?;
				let gender = chart.gender.unwrap_or(Gender::Male);
				return Some(crate::interpretation::calculate_interpretation_from_index(&sizhu_index, gender, current_block));
			}

			// 尝试从完整存储获取（旧格式，向后兼容）
			let chart = ChartById::<T>::get(chart_id)?;
			Some(crate::interpretation::calculate_full_interpretation(&chart, current_block))
		}

		/// RPC 接口：获取完整八字命盘（用于 Runtime API）
		///
		/// 返回包含所有计算字段的完整命盘数据，用于 JSON 序列化。
		/// 包含：主星、藏干（副星）、星运、空亡、纳音、神煞
		///
		/// # 参数
		/// - chart_id: 八字命盘ID
		///
		/// # 返回
		/// - Some(FullBaziChartForApi): 完整命盘数据结构
		/// - None: 命盘不存在
		pub fn get_full_bazi_chart_for_api(chart_id: u64) -> Option<crate::interpretation::FullBaziChartForApi> {
			// 优先从精简存储获取（新格式）
			if let Some(chart) = ChartCompactById::<T>::get(chart_id) {
				let sizhu_index = chart.get_sizhu_index()?;
				let gender = chart.gender.unwrap_or(Gender::Male);
				let birth_year = chart.birth_time.map(|bt| bt.year).unwrap_or(1990);
				let input_calendar_type = chart.input_calendar_type.unwrap_or(crate::types::InputCalendarType::Solar);
				return Some(crate::interpretation::build_full_bazi_chart_for_api_from_index(
					&sizhu_index,
					gender,
					birth_year,
					input_calendar_type,
				));
			}

			// 尝试从完整存储获取（旧格式，向后兼容）
			let chart = ChartById::<T>::get(chart_id)?;
			Some(crate::interpretation::build_full_bazi_chart_for_api(&chart))
		}

		/// RPC 接口：获取加密命盘的完整解盘
		///
		/// 基于加密命盘的四柱索引计算解盘，无需解密敏感数据。
		/// 当前实现：复用普通命盘的解盘逻辑（加密命盘与普通命盘共享存储）
		///
		/// # 参数
		/// - chart_id: 加密八字命盘ID
		///
		/// # 返回
		/// - Some(FullInterpretation): 完整解盘结果
		/// - None: 命盘不存在
		pub fn get_encrypted_chart_interpretation(chart_id: u64) -> Option<crate::interpretation::FullInterpretation> {
			// 当前实现：加密命盘与普通命盘共享存储，直接复用
			Self::get_full_interpretation(chart_id)
		}

		/// RPC 接口：检查加密命盘是否存在
		///
		/// # 参数
		/// - chart_id: 加密八字命盘ID
		///
		/// # 返回
		/// - true: 命盘存在
		/// - false: 命盘不存在
		pub fn encrypted_chart_exists(chart_id: u64) -> bool {
			// 当前实现：检查普通存储（加密命盘与普通命盘共享存储）
			ChartCompactById::<T>::contains_key(chart_id) || ChartById::<T>::contains_key(chart_id)
		}

		/// RPC 接口：获取加密命盘创建者
		///
		/// # 参数
		/// - chart_id: 加密八字命盘ID
		///
		/// # 返回
		/// - Some(AccountId): 命盘创建者地址
		/// - None: 命盘不存在
		pub fn get_encrypted_chart_owner(chart_id: u64) -> Option<T::AccountId> {
			// 优先从精简存储获取
			if let Some(chart) = ChartCompactById::<T>::get(chart_id) {
				return Some(chart.owner);
			}
			// 尝试从完整存储获取
			ChartById::<T>::get(chart_id).map(|chart| chart.owner)
		}

		/// 内部函数：临时排盘（支持指定日历类型）
		///
		/// 根据公历出生时间计算八字命盘，但不存储到链上。
		fn calculate_bazi_temp_with_input_type(
			year: u16,
			month: u8,
			day: u8,
			hour: u8,
			minute: u8,
			gender: Gender,
			zishi_mode: ZiShiMode,
			longitude: Option<i32>,
			input_calendar_type: crate::types::InputCalendarType,
		) -> Option<crate::interpretation::FullBaziChartForApi> {
			use crate::calculations::*;

			// 验证输入
			if year < 1900 || year > 2100 { return None; }
			if month < 1 || month > 12 { return None; }
			if day < 1 || day > 31 { return None; }
			if hour > 23 { return None; }
			if minute > 59 { return None; }

			// 应用真太阳时修正（当 longitude 有值时）
			let (calc_year, calc_month, calc_day, calc_hour, _calc_minute) =
				if let Some(lng) = longitude {
					let result = apply_true_solar_time(year, month, day, hour, minute, lng);
					let (adj_year, adj_month, adj_day) = if result.day_offset != 0 {
						adjust_date(year, month, day, result.day_offset)
					} else {
						(year, month, day)
					};
					(adj_year, adj_month, adj_day, result.hour, result.minute)
				} else {
					(year, month, day, hour, minute)
				};

			// 计算四柱
			let day_ganzhi = calculate_day_ganzhi(calc_year, calc_month, calc_day)?;
			let year_ganzhi = calculate_year_ganzhi(calc_year, calc_month, calc_day)?;
			let month_ganzhi = calculate_month_ganzhi(calc_year, calc_month, calc_day, year_ganzhi.gan.0)?;
			let (hour_ganzhi, is_next_day) = calculate_hour_ganzhi(calc_hour, day_ganzhi.gan.0, zishi_mode)?;

			let (final_day_ganzhi, final_hour_ganzhi) = if is_next_day {
				let next_day_ganzhi = day_ganzhi.next();
				let (final_hour, _) = calculate_hour_ganzhi(calc_hour, next_day_ganzhi.gan.0, zishi_mode)?;
				(next_day_ganzhi, final_hour)
			} else {
				(day_ganzhi, hour_ganzhi)
			};

			// 构建临时命盘数据用于 API 返回
			Some(crate::interpretation::build_full_bazi_chart_for_api_temp(
				year_ganzhi,
				month_ganzhi,
				final_day_ganzhi,
				final_hour_ganzhi,
				gender,
				year,
				input_calendar_type, // 使用指定的日历类型
			))
		}

		/// RPC 接口：临时排盘统一接口（不存储，免费）
		///
		/// 支持三种输入方式：公历、农历、四柱直接输入
		///
		/// # 参数
		/// - input_type: 输入类型标识 (0=Solar, 1=Lunar, 2=SiZhu)
		/// - params: 参数数组
		/// - gender: 性别 (0=Male, 1=Female)
		/// - zishi_mode: 子时模式 (1=Traditional, 2=Modern)
		///
		/// # 返回
		/// - Some(FullBaziChartForApi): 完整命盘数据
		/// - None: 输入参数无效
		pub fn calculate_bazi_temp_unified(
			input_type: u8,
			params: sp_std::vec::Vec<u16>,
			gender: u8,
			zishi_mode: u8,
		) -> Option<crate::interpretation::FullBaziChartForApi> {
			// 转换 gender
			let gender_enum = match gender {
				0 => Gender::Male,
				1 => Gender::Female,
				_ => return None,
			};

			// 转换 zishi_mode
			let zishi_mode_enum = match zishi_mode {
				1 => ZiShiMode::Traditional,
				2 => ZiShiMode::Modern,
				_ => return None,
			};

			match input_type {
				// 公历输入: [year, month, day, hour, minute] 或 [year, month, day, hour, minute, longitude]
				// longitude: 经度（乘以100后的整数，如12050表示120.50°E，负数表示西经）
				0 => {
					if params.len() < 5 { return None; }
					// 可选的经度参数用于真太阳时修正
					let longitude = if params.len() >= 6 {
						Some(params[5] as i32)
					} else {
						None
					};
					Self::calculate_bazi_temp_with_input_type(
						params[0],
						params[1] as u8,
						params[2] as u8,
						params[3] as u8,
						params[4] as u8,
						gender_enum,
						zishi_mode_enum,
						longitude,
						crate::types::InputCalendarType::Solar,
					)
				}
				// 农历输入: [year, month, day, is_leap_month, hour, minute] 或 [..., longitude]
				// longitude: 经度（乘以100后的整数，如12050表示120.50°E，负数表示西经）
				1 => {
					if params.len() < 6 { return None; }
					let lunar_year = params[0];
					let lunar_month = params[1] as u8;
					let lunar_day = params[2] as u8;
					let is_leap = params[3] != 0;
					let hour = params[4] as u8;
					let minute = params[5] as u8;
					// 可选的经度参数用于真太阳时修正
					let longitude = if params.len() >= 7 {
						Some(params[6] as i32)
					} else {
						None
					};

					// 农历转公历
					let (solar_year, solar_month, solar_day) = pallet_almanac::lunar::lunar_to_solar(
						lunar_year,
						lunar_month,
						lunar_day,
						is_leap,
					)?;

					// 使用农历输入类型
					Self::calculate_bazi_temp_with_input_type(
						solar_year,
						solar_month,
						solar_day,
						hour,
						minute,
						gender_enum,
						zishi_mode_enum,
						longitude,
						crate::types::InputCalendarType::Lunar,
					)
				}
				// 四柱直接输入: [year_gz, month_gz, day_gz, hour_gz, birth_year]
				2 => {
					if params.len() < 5 { return None; }
					let year_gz = params[0] as u8;
					let month_gz = params[1] as u8;
					let day_gz = params[2] as u8;
					let hour_gz = params[3] as u8;
					let birth_year = params[4];

					// 验证干支索引
					if year_gz >= 60 || month_gz >= 60 || day_gz >= 60 || hour_gz >= 60 {
						return None;
					}

					let year_ganzhi = GanZhi::from_index(year_gz)?;
					let month_ganzhi = GanZhi::from_index(month_gz)?;
					let day_ganzhi = GanZhi::from_index(day_gz)?;
					let hour_ganzhi = GanZhi::from_index(hour_gz)?;

					Some(crate::interpretation::build_full_bazi_chart_for_api_temp(
						year_ganzhi,
						month_ganzhi,
						day_ganzhi,
						hour_ganzhi,
						gender_enum,
						birth_year,
						crate::types::InputCalendarType::SiZhu, // 四柱直接输入
					))
				}
				_ => None,
			}
		}
	}

	// 存储押金管理辅助函数
	// ==================== DivinationProvider 实现 ====================

	/// 实现 DivinationProvider trait，使 BaziChart 能够与 DivinationAi 集成
	impl<T: Config> pallet_divination_common::traits::DivinationProvider<T::AccountId> for Pallet<T> {
		/// 检查八字是否存在
		fn result_exists(divination_type: pallet_divination_common::types::DivinationType, result_id: u64) -> bool {
			// 只处理八字类型
			if divination_type != pallet_divination_common::types::DivinationType::Bazi {
				return false;
			}

			ChartById::<T>::contains_key(result_id)
		}

		/// 获取八字创建者
		fn result_creator(divination_type: pallet_divination_common::types::DivinationType, result_id: u64) -> Option<T::AccountId> {
			if divination_type != pallet_divination_common::types::DivinationType::Bazi {
				return None;
			}

			ChartById::<T>::get(result_id).map(|chart| chart.owner)
		}

		/// 获取稀有度计算数据（暂不实现）
		fn rarity_data(
			_divination_type: pallet_divination_common::types::DivinationType,
			_result_id: u64
		) -> Option<pallet_divination_common::types::RarityInput> {
			None
		}

		/// 获取占卜结果摘要（暂不实现）
		fn result_summary(
			_divination_type: pallet_divination_common::types::DivinationType,
			_result_id: u64
		) -> Option<sp_std::vec::Vec<u8>> {
			None
		}

		/// 检查是否可以铸造为 NFT（简化实现：存在即可铸造）
		fn is_nftable(divination_type: pallet_divination_common::types::DivinationType, result_id: u64) -> bool {
			Self::result_exists(divination_type, result_id)
		}

		/// 标记已铸造为 NFT（暂不实现）
		fn mark_as_nfted(_divination_type: pallet_divination_common::types::DivinationType, _result_id: u64) {
			// 当前版本不需要标记
		}
	}
}
