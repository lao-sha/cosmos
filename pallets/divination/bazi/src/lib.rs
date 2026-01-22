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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod types;
pub mod constants;
pub mod calculations;
pub mod interpretation;
pub mod runtime_api;

// 重新导出 Runtime API 相关类型，方便外部使用
pub use interpretation::{CoreInterpretation, FullInterpretation, CompactXingGe, ExtendedJiShen};
// 重新导出加密存储类型
pub use types::{SiZhuIndex, EncryptedBaziChart, BaziInputType, InputCalendarType};
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
	use sp_runtime::traits::Saturating;

	pub use crate::types::*;

	// 导入押金相关类型
	pub use pallet_divination_common::deposit::{
		PrivacyMode as DepositPrivacyMode,
		DepositRecord,
	};

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

		// ================================
		// 存储押金相关配置 (Phase 2.1)
		// ================================

		/// 货币类型（用于押金锁定/释放）
		#[pallet::no_default]
		type Currency: frame_support::traits::ReservableCurrency<Self::AccountId>;

		/// 每 KB 存储押金基础费率
		///
		/// 建议值：
		/// - 测试网: 100（0.0000001 DUST）
		/// - 主网: 10_000_000_000（0.01 DUST）
		#[pallet::constant]
		#[pallet::no_default]
		type StorageDepositPerKb: Get<u128>;

		/// 最小存储押金
		///
		/// 建议值：
		/// - 测试网: 10（0.00000001 DUST）
		/// - 主网: 1_000_000_000（0.001 DUST）
		#[pallet::constant]
		#[pallet::no_default]
		type MinStorageDeposit: Get<u128>;

		/// 最大存储押金（防止溢出）
		///
		/// 建议值：
		/// - 测试网: 100_000_000（0.1 DUST）
		/// - 主网: 100_000_000_000_000（100 DUST）
		#[pallet::constant]
		#[pallet::no_default]
		type MaxStorageDeposit: Get<u128>;
	}

	/// 余额类型别名
	pub type BalanceOf<T> = <<T as Config>::Currency as frame_support::traits::Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

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

	/// 存储映射: 八字ID -> 核心解盘结果（13 bytes）
	///
	/// 可选缓存：用户可以选择将解盘结果缓存到链上
	/// - 优点：后续查询更快，无需重新计算
	/// - 缺点：需要支付少量 gas 费用
	///
	/// 如果未缓存，前端可以通过 Runtime API `get_interpretation()` 实时计算（免费）
	#[pallet::storage]
	#[pallet::getter(fn interpretation_cache)]
	pub type InterpretationCache<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64,
		crate::interpretation::CoreInterpretation,
	>;

	/// 存储映射: 八字ID -> 加密的八字命盘
	///
	/// 隐私保护版本的八字存储：
	/// - 敏感数据（出生时间等）在前端加密后存储
	/// - 四柱索引明文存储，支持 Runtime API 免费计算
	/// - 用户通过钱包签名派生密钥进行加解密
	#[pallet::storage]
	#[pallet::getter(fn encrypted_chart_by_id)]
	pub type EncryptedChartById<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64,
		crate::types::EncryptedBaziChart<T>,
	>;

	/// 存储映射: 用户 -> 加密八字ID列表
	#[pallet::storage]
	#[pallet::getter(fn user_encrypted_charts)]
	pub type UserEncryptedCharts<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<u64, T::MaxChartsPerAccount>,
		ValueQuery,
	>;

	// ================================
	// 注：多方授权存储已迁移至 pallet-divination-privacy
	// 包括: UserEncryptionKeys, ServiceProviders, ProvidersByType,
	//       ProviderGrants, MultiKeyEncryptedChartById, UserMultiKeyEncryptedCharts
	// 现在通过 T::PrivacyProvider trait 调用
	// ================================

	// ================================
	// BaziChart 统一隐私模式存储 (保留)
	// 用于 BaziChart 结构的 Partial/Private 模式
	// ================================

	/// 存储映射: 命盘ID -> 加密的敏感数据
	///
	/// 用于 Partial/Private 模式存储加密的敏感数据
	/// - Partial 模式：仅加密出生时间、姓名等敏感信息
	/// - Private 模式：加密所有计算数据
	#[pallet::storage]
	#[pallet::getter(fn encrypted_data)]
	pub type EncryptedData<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64,  // chart_id
		BoundedVec<u8, ConstU32<512>>,  // 加密数据（最大 512 bytes）
	>;

	/// 存储映射: 命盘ID -> 所有者加密密钥包
	///
	/// 存储用所有者 X25519 公钥加密的 DataKey
	/// 格式：临时公钥(32) + nonce(12) + 加密DataKey(48) = 92 bytes
	#[pallet::storage]
	#[pallet::getter(fn owner_key_backup)]
	pub type OwnerKeyBackup<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64,  // chart_id
		[u8; 92],  // 加密密钥包
	>;

	// ================================
	// 存储押金记录 (Phase 2.1)
	// ================================

	/// 存储映射: 命盘ID -> 押金记录
	///
	/// 记录用户创建命盘时锁定的押金信息：
	/// - 押金金额（根据数据大小和隐私模式计算）
	/// - 创建区块号（用于计算返还比例）
	/// - 数据大小（字节）
	/// - 隐私模式
	///
	/// # 押金返还规则
	///
	/// | 删除时机 | 返还比例 |
	/// |---------|---------|
	/// | 30天内删除 | 100% |
	/// | 30天后删除 | 90% |
	///
	/// 扣除的 10% 进入国库。
	#[pallet::storage]
	#[pallet::getter(fn deposit_records)]
	pub type DepositRecords<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64,  // chart_id
		DepositRecord<BalanceOf<T>, BlockNumberFor<T>>,
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
		/// 八字解盘结果已缓存（13 bytes 核心指标）[八字ID, 所有者]
		BaziInterpretationCached {
			chart_id: u64,
			owner: T::AccountId,
		},
		/// 加密八字命盘创建成功 [所有者, 八字ID]
		EncryptedBaziChartCreated {
			owner: T::AccountId,
			chart_id: u64,
		},
		/// 加密八字命盘删除成功 [所有者, 八字ID]
		EncryptedBaziChartDeleted {
			owner: T::AccountId,
			chart_id: u64,
		},

		// ================================
		// 注：多方授权事件已迁移至 pallet-divination-privacy
		// 包括: EncryptionKeyRegistered, MultiKeyEncryptedChartCreated,
		//       ChartAccessGranted, ChartAccessRevoked, ServiceProvider* 等
		// ================================

		// ================================
		// 统一隐私模式事件 (Phase 1.2.4)
		// ================================

		/// 带隐私模式的八字命盘创建成功
		///
		/// # 参数
		/// - owner: 所有者账户
		/// - chart_id: 命盘ID
		/// - privacy_mode: 隐私模式
		BaziChartCreatedWithPrivacy {
			owner: T::AccountId,
			chart_id: u64,
			privacy_mode: pallet_divination_privacy::types::PrivacyMode,
		},
		/// 加密数据更新成功
		///
		/// # 参数
		/// - chart_id: 命盘ID
		/// - owner: 所有者账户
		EncryptedDataUpdated {
			chart_id: u64,
			owner: T::AccountId,
		},

		// ================================
		// 存储押金事件 (Phase 2.1)
		// ================================

		/// 存储押金已锁定
		///
		/// 用户创建命盘时锁定存储押金
		///
		/// # 参数
		/// - chart_id: 命盘ID
		/// - owner: 所有者账户
		/// - deposit: 押金金额
		/// - privacy_mode: 隐私模式 (0=Public, 1=Partial, 2=Private)
		StorageDepositLocked {
			chart_id: u64,
			owner: T::AccountId,
			deposit: BalanceOf<T>,
			privacy_mode: u8,  // 使用 u8 避免 DecodeWithMemTracking trait 问题
		},
		/// 存储押金已返还
		///
		/// 用户删除命盘时返还存储押金
		///
		/// # 参数
		/// - chart_id: 命盘ID
		/// - owner: 所有者账户
		/// - refund: 返还金额
		/// - treasury: 进入国库金额（超过30天时）
		StorageDepositRefunded {
			chart_id: u64,
			owner: T::AccountId,
			refund: BalanceOf<T>,
			treasury: BalanceOf<T>,
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
		/// 加密数据过长
		EncryptedDataTooLong,
		/// 加密八字未找到
		EncryptedChartNotFound,
		/// 农历日期无效或转换失败
		InvalidLunarDate,
		/// 输入参数无效
		InvalidInput,

		// ================================
		// 注：多方授权错误已迁移至 pallet-divination-privacy
		// ================================

		// ================================
		// 统一隐私模式错误 (Phase 1.2.4)
		// ================================

		/// 无效的隐私模式（应为 0=Public, 1=Partial, 2=Private）
		InvalidPrivacyMode,
		/// Public 模式不应包含加密数据
		PublicModeNoEncryptedData,
		/// Partial/Private 模式缺少加密数据
		EncryptedDataRequired,
		/// Partial 模式缺少计算参数
		PartialModeRequiresCalculationParams,

		// ================================
		// 存储押金相关错误 (Phase 2.1)
		// ================================

		/// 押金余额不足
		InsufficientDepositBalance,
		/// 押金记录未找到
		DepositRecordNotFound,
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
			let (sizhu, birth_time, birth_year) = Self::calculate_sizhu_from_input_with_solar_time(
				&input,
				zishi_mode,
				longitude,
			)?;

			// 4. 获取日主天干
			let day_ganzhi = sizhu.day_zhu.ganzhi;
			let year_ganzhi = sizhu.year_zhu.ganzhi;
			let month_ganzhi = sizhu.month_zhu.ganzhi;
			let hour_ganzhi = sizhu.hour_zhu.ganzhi;

			// 5. 计算大运
			// 简化版：假设距离下一个节气6天（生产环境需要精确计算）
			let days_to_jieqi = 6u8;
			let (qiyun_age, is_shun) = crate::calculations::calculate_qiyun_age(year_ganzhi.gan.0, gender, days_to_jieqi);
			let qiyun_year = birth_year + qiyun_age as u16;

			// 生成大运列表（12步，120年）
			let dayun_list_simple = crate::calculations::calculate_dayun_list(month_ganzhi, birth_year, qiyun_age, is_shun, 12);

			// 转换为DaYunStep类型
			let mut dayun_steps = BoundedVec::<DaYunStep<T>, T::MaxDaYunSteps>::default();
			for (gz, start_age, start_year) in dayun_list_simple {
				let end_age = start_age + 10;
				let end_year = start_year + 10;

				// 计算十神
				let tiangan_shishen = crate::constants::calculate_shishen(day_ganzhi.gan, gz.gan);

				// 计算藏干十神
				let hidden_stems = crate::constants::get_hidden_stems(gz.zhi);
				let mut canggan_shishen = BoundedVec::<ShiShen, T::MaxCangGan>::default();
				for (cg_gan, _, _) in hidden_stems.iter() {
					// 跳过无效藏干
					if !crate::constants::is_valid_canggan(cg_gan.0) {
						continue;
					}
					let cg_shishen = crate::constants::calculate_shishen(day_ganzhi.gan, *cg_gan);
					canggan_shishen.try_push(cg_shishen).map_err(|_| Error::<T>::TooManyCangGan)?;
				}

				let step = DaYunStep {
					ganzhi: gz,
					start_age,
					end_age,
					start_year,
					end_year,
					tiangan_shishen,
					canggan_shishen,
				};

				dayun_steps.try_push(step).map_err(|_| Error::<T>::TooManyDaYunSteps)?;
			}

			let dayun_info = DaYunInfo {
				qiyun_age,
				qiyun_year,
				is_shun,
				dayun_list: dayun_steps,
			};

			// 6. 计算五行强度
			let wuxing_strength = crate::calculations::calculate_wuxing_strength(
				&year_ganzhi,
				&month_ganzhi,
				&day_ganzhi,
				&hour_ganzhi,
			);

			// 7. 判断喜用神
			let xiyong_shen = crate::calculations::determine_xiyong_shen(&wuxing_strength, day_ganzhi.gan);

			// 8. 确定输入日历类型（记录原始输入是公历还是农历）
			let input_calendar_type = match input {
				crate::types::BaziInputType::Solar { .. } => crate::types::InputCalendarType::Solar,
				crate::types::BaziInputType::Lunar { .. } => crate::types::InputCalendarType::Lunar,
				crate::types::BaziInputType::SiZhu { .. } => crate::types::InputCalendarType::SiZhu,
			};

			// 9. 构建八字信息（默认使用 Public 模式）
			let bazi_chart = BaziChart {
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
				// 计算数据
				sizhu: Some(sizhu),
				dayun: Some(dayun_info),
				wuxing_strength: Some(wuxing_strength),
				xiyong_shen,
				timestamp: frame_system::Pallet::<T>::block_number().saturated_into(),
			};

			// 10. 存储八字
			let chart_id = NextChartId::<T>::get();
			ensure!(chart_id < u64::MAX, Error::<T>::ChartIdOverflow);

			ChartById::<T>::insert(chart_id, bazi_chart);

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

			// 获取八字信息
			let chart = ChartById::<T>::get(chart_id)
				.ok_or(Error::<T>::ChartNotFound)?;

			// 验证所有权
			ensure!(chart.owner == who, Error::<T>::NotChartOwner);

			// 从 ChartById 中删除
			ChartById::<T>::remove(chart_id);

			// 从用户的八字列表中删除
			UserCharts::<T>::try_mutate(&who, |charts| -> DispatchResult {
				if let Some(pos) = charts.iter().position(|&id| id == chart_id) {
					charts.remove(pos);
				}
				Ok(())
			})?;

			// 触发事件
			Self::deposit_event(Event::BaziChartDeleted {
				owner: who,
				chart_id,
			});

			Ok(())
		}

		/// 缓存八字解盘结果（核心指标，13 bytes）
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者
		/// - `chart_id`: 八字ID
		///
		/// # 功能
		///
		/// 1. 验证八字存在和所有权
		/// 2. 实时计算核心解盘结果
		/// 3. 将结果缓存到链上 `InterpretationCache`
		///
		/// # 优点
		///
		/// - 后续查询无需重新计算，速度更快
		/// - 可以在前端优先使用缓存结果
		///
		/// # 缺点
		///
		/// - 需要支付少量 gas 费用（约 13 bytes 存储成本）
		///
		/// # 注意
		///
		/// - 如果不缓存，前端可以直接调用 Runtime API `get_interpretation()` 免费实时计算
		/// - 缓存后算法升级不会自动更新缓存，需要重新缓存
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::create_bazi_chart())]
		pub fn cache_interpretation(
			origin: OriginFor<T>,
			chart_id: u64,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 获取八字信息
			let chart = ChartById::<T>::get(chart_id)
				.ok_or(Error::<T>::ChartNotFound)?;

			// 验证所有权
			ensure!(chart.owner == who, Error::<T>::NotChartOwner);

			// 实时计算核心解盘结果
			let current_block = <frame_system::Pallet<T>>::block_number().saturated_into();
			let interpretation = crate::interpretation::calculate_core_interpretation(&chart, current_block);

			// 缓存到链上
			InterpretationCache::<T>::insert(chart_id, interpretation);

			// 触发事件
			Self::deposit_event(Event::BaziInterpretationCached {
				chart_id,
				owner: who,
			});

			Ok(())
		}

		/// 创建加密的八字命盘
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者
		/// - `sizhu_index`: 四柱干支索引（明文，用于计算）
		/// - `gender`: 性别（明文，用于大运计算）
		/// - `encrypted_data`: AES-256-GCM 加密的敏感数据
		/// - `data_hash`: 原始数据的 Blake2-256 哈希（用于验证解密正确性）
		///
		/// # 功能
		///
		/// 1. 验证四柱索引有效性
		/// 2. 存储加密的八字信息
		/// 3. 触发创建事件
		///
		/// # 安全特性
		///
		/// - 出生时间等敏感数据在前端加密后存储
		/// - 四柱索引明文存储，支持 Runtime API 免费计算解盘
		/// - 用户通过钱包签名派生密钥进行加解密，无需输入密码
		///
		/// # 存储结构（约 50 bytes + 加密数据长度）
		///
		/// - `sizhu_index`: 8 bytes（四柱索引）
		/// - `gender`: 1 byte
		/// - `encrypted_data`: 可变（最大 256 bytes）
		/// - `data_hash`: 32 bytes
		/// - `created_at`: 4 bytes
		/// - `owner`: 32 bytes（AccountId）
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::create_bazi_chart())]
		pub fn create_encrypted_chart(
			origin: OriginFor<T>,
			sizhu_index: crate::types::SiZhuIndex,
			gender: Gender,
			encrypted_data: BoundedVec<u8, ConstU32<256>>,
			data_hash: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 验证四柱索引有效性
			ensure!(sizhu_index.is_valid(), Error::<T>::InvalidSiZhuIndex);

			// 2. 检查账户八字数量限制
			let existing_charts = UserEncryptedCharts::<T>::get(&who);
			ensure!(
				existing_charts.len() < T::MaxChartsPerAccount::get() as usize,
				Error::<T>::TooManyCharts
			);

			// 3. 获取新的 chart_id
			let chart_id = NextChartId::<T>::get();
			ensure!(chart_id < u64::MAX, Error::<T>::ChartIdOverflow);

			// 4. 获取当前区块号
			let current_block = <frame_system::Pallet<T>>::block_number().saturated_into();

			// 5. 构建加密八字结构
			let encrypted_chart = crate::types::EncryptedBaziChart {
				owner: who.clone(),
				sizhu_index,
				gender,
				encrypted_data,
				data_hash,
				created_at: current_block,
			};

			// 6. 存储到 EncryptedChartById
			EncryptedChartById::<T>::insert(chart_id, encrypted_chart);

			// 7. 添加到用户的加密八字列表
			UserEncryptedCharts::<T>::try_mutate(&who, |charts| {
				charts.try_push(chart_id).map_err(|_| Error::<T>::TooManyCharts)
			})?;

			// 8. 递增计数器
			NextChartId::<T>::put(chart_id + 1);

			// 9. 触发事件
			Self::deposit_event(Event::EncryptedBaziChartCreated {
				owner: who,
				chart_id,
			});

			Ok(())
		}

		/// 删除加密的八字命盘
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者
		/// - `chart_id`: 八字ID
		///
		/// # 权限
		///
		/// 只有八字所有者可以删除自己的加密八字
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::delete_bazi_chart())]
		pub fn delete_encrypted_chart(
			origin: OriginFor<T>,
			chart_id: u64,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 获取加密八字信息
			let chart = EncryptedChartById::<T>::get(chart_id)
				.ok_or(Error::<T>::EncryptedChartNotFound)?;

			// 验证所有权
			ensure!(chart.owner == who, Error::<T>::NotChartOwner);

			// 从 EncryptedChartById 中删除
			EncryptedChartById::<T>::remove(chart_id);

			// 从用户的加密八字列表中删除
			UserEncryptedCharts::<T>::try_mutate(&who, |charts| -> DispatchResult {
				if let Some(pos) = charts.iter().position(|&id| id == chart_id) {
					charts.remove(pos);
				}
				Ok(())
			})?;

			// 触发事件
			Self::deposit_event(Event::EncryptedBaziChartDeleted {
				owner: who,
				chart_id,
			});

			Ok(())
		}

		// ================================
		// 统一隐私模式交易 (Phase 1.2.4)
		// ================================

		/// 创建带隐私模式的八字命盘
		///
		/// # 隐私模式
		///
		/// - **Public (0)**: 所有数据明文存储，可公开查看
		/// - **Partial (1)**: 计算数据明文 + 敏感数据加密 ⭐推荐
		/// - **Private (2)**: 所有数据加密，无法链上解读
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者
		/// - `privacy_mode`: 隐私模式 (0=Public, 1=Partial, 2=Private)
		/// - `name`: 命盘名称（可选）
		/// - `input`: 输入类型（Partial 模式必填，Private 模式可选）
		/// - `gender`: 性别（Partial 模式必填）
		/// - `zishi_mode`: 子时模式（Partial 模式必填）
		/// - `longitude`: 出生地经度（可选）
		/// - `encrypted_data`: 加密的敏感数据（Partial/Private 模式必填）
		/// - `data_hash`: 原始数据哈希（用于验证解密正确性）
		/// - `owner_key_backup`: 所有者加密密钥包（92 bytes）
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::create_bazi_chart())]
		pub fn create_bazi_chart_encrypted(
			origin: OriginFor<T>,
			privacy_mode: u8,
			name: Option<BoundedVec<u8, ConstU32<32>>>,
			input: Option<BaziInputType>,
			gender: Option<Gender>,
			zishi_mode: Option<ZiShiMode>,
			longitude: Option<i32>,
			encrypted_data: Option<BoundedVec<u8, ConstU32<512>>>,
			data_hash: Option<[u8; 32]>,
			owner_key_backup: Option<[u8; 92]>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 验证并转换隐私模式
			let privacy = match privacy_mode {
				0 => pallet_divination_privacy::types::PrivacyMode::Public,
				1 => pallet_divination_privacy::types::PrivacyMode::Partial,
				2 => pallet_divination_privacy::types::PrivacyMode::Private,
				_ => return Err(Error::<T>::InvalidPrivacyMode.into()),
			};

			// 2. 根据隐私模式验证参数
			match privacy {
				pallet_divination_privacy::types::PrivacyMode::Public => {
					// Public 模式不应有加密数据
					ensure!(encrypted_data.is_none(), Error::<T>::PublicModeNoEncryptedData);
					// 必须有计算参数
					ensure!(input.is_some() && gender.is_some() && zishi_mode.is_some(),
						Error::<T>::PartialModeRequiresCalculationParams);
				},
				pallet_divination_privacy::types::PrivacyMode::Partial => {
					// Partial 模式必须有加密数据
					ensure!(encrypted_data.is_some() && data_hash.is_some() && owner_key_backup.is_some(),
						Error::<T>::EncryptedDataRequired);
					// 必须有计算参数
					ensure!(input.is_some() && gender.is_some() && zishi_mode.is_some(),
						Error::<T>::PartialModeRequiresCalculationParams);
				},
				pallet_divination_privacy::types::PrivacyMode::Private => {
					// Private 模式必须有加密数据
					ensure!(encrypted_data.is_some() && data_hash.is_some() && owner_key_backup.is_some(),
						Error::<T>::EncryptedDataRequired);
					// 计算参数可选（前端已加密）
				},
			}

			// 3. 检查账户八字数量限制
			let existing_charts = UserCharts::<T>::get(&who);
			ensure!(
				existing_charts.len() < T::MaxChartsPerAccount::get() as usize,
				Error::<T>::TooManyCharts
			);

			// 4. 根据隐私模式构建命盘
			let chart_id = NextChartId::<T>::get();
			ensure!(chart_id < u64::MAX, Error::<T>::ChartIdOverflow);

			let bazi_chart = if privacy == pallet_divination_privacy::types::PrivacyMode::Private {
				// Private 模式：不存储计算数据
				BaziChart {
					owner: who.clone(),
					name: name.unwrap_or_default(),
					privacy_mode: privacy,
					encrypted_fields: Some(0xFF), // 所有字段加密
					sensitive_data_hash: data_hash,
					birth_time: None,
					input_calendar_type: None,
					gender: None,
					zishi_mode: None,
					longitude: None,
					sizhu: None,
					dayun: None,
					wuxing_strength: None,
					xiyong_shen: None,
					timestamp: frame_system::Pallet::<T>::block_number().saturated_into(),
				}
			} else {
				// Public/Partial 模式：计算并存储数据
				let input_val = input.ok_or(Error::<T>::PartialModeRequiresCalculationParams)?;
				let gender_val = gender.ok_or(Error::<T>::PartialModeRequiresCalculationParams)?;
				let zishi_mode_val = zishi_mode.ok_or(Error::<T>::PartialModeRequiresCalculationParams)?;

				ensure!(input_val.is_valid(), Error::<T>::InvalidInput);

				// 计算四柱
				let (sizhu, birth_time, birth_year) = Self::calculate_sizhu_from_input_with_solar_time(
					&input_val,
					zishi_mode_val,
					longitude,
				)?;

				let day_ganzhi = sizhu.day_zhu.ganzhi;
				let year_ganzhi = sizhu.year_zhu.ganzhi;
				let month_ganzhi = sizhu.month_zhu.ganzhi;
				let hour_ganzhi = sizhu.hour_zhu.ganzhi;

				// 计算大运
				let days_to_jieqi = 6u8;
				let (qiyun_age, is_shun) = crate::calculations::calculate_qiyun_age(year_ganzhi.gan.0, gender_val, days_to_jieqi);
				let qiyun_year = birth_year + qiyun_age as u16;

				let dayun_list_simple = crate::calculations::calculate_dayun_list(month_ganzhi, birth_year, qiyun_age, is_shun, 12);

				let mut dayun_steps = BoundedVec::<DaYunStep<T>, T::MaxDaYunSteps>::default();
				for (gz, start_age, start_year) in dayun_list_simple {
					let end_age = start_age + 10;
					let end_year = start_year + 10;
					let tiangan_shishen = crate::constants::calculate_shishen(day_ganzhi.gan, gz.gan);

					let hidden_stems = crate::constants::get_hidden_stems(gz.zhi);
					let mut canggan_shishen = BoundedVec::<ShiShen, T::MaxCangGan>::default();
					for (cg_gan, _, _) in hidden_stems.iter() {
						if !crate::constants::is_valid_canggan(cg_gan.0) {
							continue;
						}
						let cg_shishen = crate::constants::calculate_shishen(day_ganzhi.gan, *cg_gan);
						canggan_shishen.try_push(cg_shishen).map_err(|_| Error::<T>::TooManyCangGan)?;
					}

					let step = DaYunStep {
						ganzhi: gz,
						start_age,
						end_age,
						start_year,
						end_year,
						tiangan_shishen,
						canggan_shishen,
					};
					dayun_steps.try_push(step).map_err(|_| Error::<T>::TooManyDaYunSteps)?;
				}

				let dayun_info = DaYunInfo {
					qiyun_age,
					qiyun_year,
					is_shun,
					dayun_list: dayun_steps,
				};

				// 计算五行强度和喜用神
				let wuxing_strength = crate::calculations::calculate_wuxing_strength(
					&year_ganzhi,
					&month_ganzhi,
					&day_ganzhi,
					&hour_ganzhi,
				);
				let xiyong_shen = crate::calculations::determine_xiyong_shen(&wuxing_strength, day_ganzhi.gan);

				let input_calendar_type = match input_val {
					crate::types::BaziInputType::Solar { .. } => crate::types::InputCalendarType::Solar,
					crate::types::BaziInputType::Lunar { .. } => crate::types::InputCalendarType::Lunar,
					crate::types::BaziInputType::SiZhu { .. } => crate::types::InputCalendarType::SiZhu,
				};

				BaziChart {
					owner: who.clone(),
					name: name.unwrap_or_default(),
					privacy_mode: privacy,
					encrypted_fields: if privacy == pallet_divination_privacy::types::PrivacyMode::Partial {
						Some(0x0F) // 敏感字段加密（姓名、出生日期、性别、经度）
					} else {
						None
					},
					sensitive_data_hash: data_hash,
					birth_time: Some(birth_time),
					input_calendar_type: Some(input_calendar_type),
					gender: Some(gender_val),
					zishi_mode: Some(zishi_mode_val),
					longitude,
					sizhu: Some(sizhu),
					dayun: Some(dayun_info),
					wuxing_strength: Some(wuxing_strength),
					xiyong_shen,
					timestamp: frame_system::Pallet::<T>::block_number().saturated_into(),
				}
			};

			// 5. 存储命盘
			ChartById::<T>::insert(chart_id, bazi_chart);

			// 6. 存储加密数据（Partial/Private 模式）
			if let Some(enc_data) = encrypted_data {
				EncryptedData::<T>::insert(chart_id, enc_data);
			}
			if let Some(key_backup) = owner_key_backup {
				OwnerKeyBackup::<T>::insert(chart_id, key_backup);
			}

			// 7. 更新用户命盘列表
			UserCharts::<T>::try_mutate(&who, |charts| {
				charts.try_push(chart_id).map_err(|_| Error::<T>::TooManyCharts)
			})?;

			NextChartId::<T>::put(chart_id + 1);

			// 8. 触发事件
			Self::deposit_event(Event::BaziChartCreatedWithPrivacy {
				owner: who,
				chart_id,
				privacy_mode: privacy,
			});

			Ok(())
		}

		/// 更新加密数据
		///
		/// 允许所有者更新命盘的加密数据（例如：重新加密或添加新信息）
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者（必须是命盘所有者）
		/// - `chart_id`: 命盘ID
		/// - `encrypted_data`: 新的加密数据
		/// - `data_hash`: 新的数据哈希
		/// - `owner_key_backup`: 新的所有者密钥包
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::create_bazi_chart())]
		pub fn update_encrypted_data(
			origin: OriginFor<T>,
			chart_id: u64,
			encrypted_data: BoundedVec<u8, ConstU32<512>>,
			data_hash: [u8; 32],
			owner_key_backup: [u8; 92],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 获取命盘并验证所有权
			let mut chart = ChartById::<T>::get(chart_id)
				.ok_or(Error::<T>::ChartNotFound)?;
			ensure!(chart.owner == who, Error::<T>::NotChartOwner);

			// 2. 验证命盘使用加密模式
			ensure!(
				chart.privacy_mode != pallet_divination_privacy::types::PrivacyMode::Public,
				Error::<T>::PublicModeNoEncryptedData
			);

			// 3. 更新加密数据
			EncryptedData::<T>::insert(chart_id, encrypted_data);
			OwnerKeyBackup::<T>::insert(chart_id, owner_key_backup);

			// 4. 更新命盘的数据哈希
			chart.sensitive_data_hash = Some(data_hash);
			ChartById::<T>::insert(chart_id, chart);

			// 5. 触发事件
			Self::deposit_event(Event::EncryptedDataUpdated {
				chart_id,
				owner: who,
			});

			Ok(())
		}

		// ================================
		// 注：多方授权/服务提供者交易已迁移至 pallet-divination-privacy
		// 用户应直接调用 Privacy 模块的相关函数:
		// - Privacy::register_encryption_key (原 call_index 40)
		// - Privacy::update_encryption_key (原 call_index 41)
		// - Privacy::create_encrypted_record (原 call_index 42)
		// - Privacy::grant_access (原 call_index 43)
		// - Privacy::revoke_access (原 call_index 44)
		// - Privacy::revoke_all_access (原 call_index 45)
		// - Privacy::delete_encrypted_record (原 call_index 46)
		// - Privacy::register_provider (原 call_index 50)
		// - Privacy::update_provider_key (原 call_index 51)
		// - Privacy::set_provider_active (原 call_index 52)
		// - Privacy::unregister_provider (原 call_index 53)
		// ================================
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

		/// 根据输入类型计算四柱和出生时间
		///
		/// # 参数
		/// - `input`: 输入类型（公历/农历/四柱）
		/// - `zishi_mode`: 子时模式
		///
		/// # 返回
		/// - `Ok((SiZhu, BirthTime, birth_year))`: 四柱、出生时间、出生年份
		/// - `Err`: 计算失败
		fn calculate_sizhu_from_input(
			input: &BaziInputType,
			zishi_mode: ZiShiMode,
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

					// 计算日柱
					let day_ganzhi = calculate_day_ganzhi(year, month, day)
						.ok_or(Error::<T>::InvalidDay)?;

					// 计算年柱
					let year_ganzhi = calculate_year_ganzhi(year, month, day)
						.ok_or(Error::<T>::InvalidYear)?;

					// 计算月柱
					let month_ganzhi = calculate_month_ganzhi(year, month, day, year_ganzhi.gan.0)
						.ok_or(Error::<T>::InvalidMonth)?;

					// 计算时柱（处理子时双模式）
					let (hour_ganzhi, is_next_day) = calculate_hour_ganzhi(hour, day_ganzhi.gan.0, zishi_mode)
						.ok_or(Error::<T>::InvalidHour)?;

					// 如果是次日子时（传统派23:00），需要重新计算日柱
					let (final_day_ganzhi, final_hour_ganzhi) = if is_next_day {
						let next_day_ganzhi = day_ganzhi.next();
						let (final_hour, _) = calculate_hour_ganzhi(hour, next_day_ganzhi.gan.0, zishi_mode)
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

					// 使用转换后的公历日期计算四柱
					let day_ganzhi = calculate_day_ganzhi(solar_year, solar_month, solar_day)
						.ok_or(Error::<T>::InvalidDay)?;

					let year_ganzhi = calculate_year_ganzhi(solar_year, solar_month, solar_day)
						.ok_or(Error::<T>::InvalidYear)?;

					let month_ganzhi = calculate_month_ganzhi(solar_year, solar_month, solar_day, year_ganzhi.gan.0)
						.ok_or(Error::<T>::InvalidMonth)?;

					let (hour_ganzhi, is_next_day) = calculate_hour_ganzhi(hour, day_ganzhi.gan.0, zishi_mode)
						.ok_or(Error::<T>::InvalidHour)?;

					let (final_day_ganzhi, final_hour_ganzhi) = if is_next_day {
						let next_day_ganzhi = day_ganzhi.next();
						let (final_hour, _) = calculate_hour_ganzhi(hour, next_day_ganzhi.gan.0, zishi_mode)
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

					// 出生时间记录转换后的公历日期
					let birth_time = BirthTime {
						year: solar_year,
						month: solar_month,
						day: solar_day,
						hour,
						minute,
					};

					Ok((sizhu, birth_time, solar_year))
				}

				// 四柱直接输入
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

		/// 根据输入类型计算四柱和出生时间（支持真太阳时修正）
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
		fn calculate_sizhu_from_input_with_solar_time(
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
			let chart = ChartById::<T>::get(chart_id)?;
			let current_block = <frame_system::Pallet<T>>::block_number().saturated_into();

			Some(crate::interpretation::calculate_full_interpretation(&chart, current_block))
		}

		/// RPC 接口：基于加密命盘的四柱索引计算解盘
		///
		/// 此函数由 Runtime API 调用，不消耗 gas，不上链
		///
		/// # 参数
		/// - chart_id: 加密八字命盘ID
		///
		/// # 返回
		/// - Some(FullInterpretation): 完整解盘结果
		/// - None: 命盘不存在
		///
		/// # 特点
		/// - 基于四柱索引计算，无需解密敏感数据
		/// - 完全免费（无 gas 费用）
		/// - 保护用户隐私
		pub fn get_encrypted_chart_interpretation(chart_id: u64) -> Option<crate::interpretation::FullInterpretation> {
			let encrypted_chart = EncryptedChartById::<T>::get(chart_id)?;
			let current_block = <frame_system::Pallet<T>>::block_number().saturated_into();

			Some(crate::interpretation::calculate_interpretation_from_index(
				&encrypted_chart.sizhu_index,
				encrypted_chart.gender,
				current_block,
			))
		}

		/// 检查加密命盘是否存在
		pub fn encrypted_chart_exists(chart_id: u64) -> bool {
			EncryptedChartById::<T>::contains_key(chart_id)
		}

		/// 获取加密命盘所有者
		pub fn get_encrypted_chart_owner(chart_id: u64) -> Option<T::AccountId> {
			EncryptedChartById::<T>::get(chart_id).map(|chart| chart.owner)
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
			let chart = ChartById::<T>::get(chart_id)?;
			Some(crate::interpretation::build_full_bazi_chart_for_api(&chart))
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

		// ================================
		// 注：多方授权加密系统 API 已迁移至 pallet-divination-privacy
		// 请使用 Privacy 模块的 Runtime API:
		// - get_user_encryption_key -> Privacy::get_user_public_key()
		// - get_service_provider_json -> Privacy::get_provider_info()
		// - get_providers_by_type_filtered -> Privacy::get_providers_by_type()
		// - get_provider_grants_list -> Privacy::get_provider_grants()
		// - get_multi_key_encrypted_chart_info_json -> Privacy::get_encrypted_record_info()
		// ================================
	}

	// ================================
	// 存储押金管理辅助函数 (Phase 2.1)
	// ================================

	impl<T: Config> Pallet<T>
	where
		BalanceOf<T>: From<u32> + sp_runtime::traits::Saturating + core::ops::Div<Output = BalanceOf<T>> + sp_runtime::traits::Zero + PartialOrd + Copy,
		BlockNumberFor<T>: From<u32> + sp_runtime::traits::Saturating + PartialOrd + Copy,
	{
		/// 计算存储押金
		///
		/// 根据数据大小和隐私模式计算押金金额
		///
		/// # 公式
		/// ```text
		/// 押金 = 基础费率 × ceil(数据大小 / 1024) × 隐私模式系数 / 100
		/// ```
		///
		/// # 参数
		/// - `data_size`: 数据大小（字节）
		/// - `privacy_mode`: 隐私模式
		///
		/// # 返回
		/// 押金金额（已限制在最小/最大范围内）
		pub fn calculate_deposit(data_size: u32, privacy_mode: DepositPrivacyMode) -> BalanceOf<T> {
			// 计算 KB 数（向上取整）
			let size_kb = (data_size.saturating_add(1023)) / 1024;
			let size_kb = if size_kb == 0 { 1 } else { size_kb };

			// 获取隐私模式系数
			let multiplier = privacy_mode.multiplier();

			// 获取配置值（u128 类型）
			let base_rate = T::StorageDepositPerKb::get();
			let min_deposit = T::MinStorageDeposit::get();
			let max_deposit = T::MaxStorageDeposit::get();

			// 计算押金（u128）
			// deposit = base_rate × size_kb × multiplier / 100
			let deposit_u128 = base_rate
				.saturating_mul(size_kb as u128)
				.saturating_mul(multiplier as u128)
				/ 100u128;

			// 限制在最小/最大范围内
			let clamped = if deposit_u128 < min_deposit {
				min_deposit
			} else if deposit_u128 > max_deposit {
				max_deposit
			} else {
				deposit_u128
			};

			// 转换为 BalanceOf<T>（使用 SaturatedConversion）
			clamped.saturated_into()
		}

		/// 锁定存储押金
		///
		/// 从用户余额中锁定存储押金
		///
		/// # 参数
		/// - `who`: 用户账户
		/// - `chart_id`: 命盘ID
		/// - `data_size`: 数据大小（字节）
		/// - `privacy_mode`: 隐私模式
		///
		/// # 返回
		/// - `Ok(押金金额)`: 锁定成功
		/// - `Err`: 余额不足或其他错误
		pub fn reserve_storage_deposit(
			who: &T::AccountId,
			chart_id: u64,
			data_size: u32,
			privacy_mode: DepositPrivacyMode,
		) -> Result<BalanceOf<T>, DispatchError> {
			// 1. 计算押金
			let deposit_amount = Self::calculate_deposit(data_size, privacy_mode);

			// 2. 锁定押金
			<T::Currency as frame_support::traits::ReservableCurrency<T::AccountId>>::reserve(
				who,
				deposit_amount,
			).map_err(|_| Error::<T>::InsufficientDepositBalance)?;

			// 3. 记录押金信息
			let current_block = <frame_system::Pallet<T>>::block_number();
			let record = DepositRecord {
				amount: deposit_amount,
				created_at: current_block,
				data_size,
				privacy_mode,
			};
			DepositRecords::<T>::insert(chart_id, record);

			Ok(deposit_amount)
		}

		/// 返还存储押金
		///
		/// 根据存储时长计算返还比例并释放押金
		///
		/// # 参数
		/// - `who`: 用户账户
		/// - `chart_id`: 命盘ID
		///
		/// # 返回
		/// - `Ok((返还金额, 国库金额))`: 返还成功
		/// - `Err`: 押金记录不存在
		pub fn unreserve_storage_deposit(
			who: &T::AccountId,
			chart_id: u64,
		) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
			// 1. 获取押金记录
			let record = DepositRecords::<T>::take(chart_id)
				.ok_or(Error::<T>::DepositRecordNotFound)?;

			// 2. 计算返还金额
			let current_block = <frame_system::Pallet<T>>::block_number();
			let (refund_amount, treasury_amount) = pallet_divination_common::deposit::calculate_refund_amount(
				record.amount,
				record.created_at,
				current_block,
			);

			// 3. 释放押金（返还给用户）
			let actually_unreserved = <T::Currency as frame_support::traits::ReservableCurrency<T::AccountId>>::unreserve(
				who,
				refund_amount,
			);

			// 4. TODO: 将 treasury_amount 转入国库
			// 当前简化实现：treasury_amount 仍然在用户锁定余额中，需要手动处理
			// 正式实现需要：
			// - 调用 T::Treasury::deposit(treasury_amount)
			// - 或者使用 OnUnbalanced trait 处理

			// 如果有 treasury_amount，也从用户余额中释放（实际应转入国库）
			if !treasury_amount.is_zero() {
				let _ = <T::Currency as frame_support::traits::ReservableCurrency<T::AccountId>>::unreserve(
					who,
					treasury_amount,
				);
			}

			// 返回实际返还和国库金额
			Ok((refund_amount.saturating_sub(refund_amount.saturating_sub(actually_unreserved)), treasury_amount))
		}

		/// 估算特定占卜类型和隐私模式的押金
		///
		/// 用于前端预估费用显示
		///
		/// # 参数
		/// - `privacy_mode`: 隐私模式 (0=Public, 1=Partial, 2=Private)
		///
		/// # 返回
		/// 估算的押金金额
		pub fn estimate_deposit(privacy_mode: u8) -> Option<BalanceOf<T>> {
			let mode = DepositPrivacyMode::from_u8(privacy_mode)?;
			// 八字数据估算大小: 1500 bytes (根据 STORAGE_DEPOSIT_AND_DELETION_ANALYSIS.md)
			let data_size = pallet_divination_common::deposit::estimate_data_size(1, mode); // 1 = Bazi
			Some(Self::calculate_deposit(data_size, mode))
		}
	}

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
