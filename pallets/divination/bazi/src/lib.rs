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
// 重新导出多方授权加密类型
pub use types::{
	AccessRole, AccessScope, EncryptedKeyEntry, MultiKeyEncryptedBaziChart,
	ServiceProviderType, ServiceProvider,
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
	// 多方授权加密系统存储
	// ================================

	/// 存储映射: 用户 -> X25519 加密公钥
	///
	/// 用户需要注册 X25519 公钥才能：
	/// 1. 创建多方授权加密命盘（作为所有者）
	/// 2. 被授权访问他人的加密命盘（作为被授权方）
	///
	/// # 注意
	/// - 此公钥与账户签名公钥分离，专用于加密操作
	/// - ED25519 账户可从签名密钥派生 X25519 密钥
	/// - SR25519 账户需要独立生成 X25519 密钥对
	#[pallet::storage]
	#[pallet::getter(fn user_encryption_keys)]
	pub type UserEncryptionKeys<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		[u8; 32],  // X25519 公钥
	>;

	/// 存储映射: 八字ID -> 多方授权加密命盘
	///
	/// 支持多方授权的加密命盘存储：
	/// - 四柱索引明文存储，支持 Runtime API 免费计算
	/// - 敏感数据使用 AES-256-GCM 加密
	/// - 每个授权方有独立加密的 DataKey
	/// - 最多支持 10 个授权方
	#[pallet::storage]
	#[pallet::getter(fn multi_key_encrypted_chart_by_id)]
	pub type MultiKeyEncryptedChartById<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64,
		crate::types::MultiKeyEncryptedBaziChart<T>,
	>;

	/// 存储映射: 用户 -> 多方授权加密八字ID列表
	#[pallet::storage]
	#[pallet::getter(fn user_multi_key_encrypted_charts)]
	pub type UserMultiKeyEncryptedCharts<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<u64, T::MaxChartsPerAccount>,
		ValueQuery,
	>;

	// ================================
	// 服务提供者系统存储
	// ================================

	/// 存储映射: 服务提供者账户 -> 服务提供者信息
	///
	/// 服务提供者（命理师、AI 服务等）需要注册后才能：
	/// 1. 被用户发现和选择
	/// 2. 接收用户授权的命盘访问权限
	/// 3. 使用其公钥解密 DataKey
	///
	/// # 注册流程
	/// 1. 调用 `register_provider` 提交公钥和服务类型
	/// 2. 系统存储提供者信息，初始信誉分为 50
	/// 3. 用户授权时从此处获取提供者的 X25519 公钥
	#[pallet::storage]
	#[pallet::getter(fn service_providers)]
	pub type ServiceProviders<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		crate::types::ServiceProvider<T>,
	>;

	/// 存储映射: 服务类型 -> 提供者账户列表
	///
	/// 用于按类型查询和筛选服务提供者
	/// 每种类型最多存储 100 个提供者
	#[pallet::storage]
	#[pallet::getter(fn providers_by_type)]
	pub type ProvidersByType<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		crate::types::ServiceProviderType,
		BoundedVec<T::AccountId, ConstU32<100>>,
		ValueQuery,
	>;

	/// 存储映射: 被授权账户 -> 被授权访问的命盘ID列表
	///
	/// 反向索引：记录每个账户被授权访问哪些命盘
	/// 便于服务提供者查询自己被授权的所有命盘
	///
	/// # 使用场景
	/// 1. 命理师查看自己被授权解读的所有命盘
	/// 2. AI 服务批量处理被授权的命盘
	/// 3. 用户查看自己被授权访问的家族命盘
	///
	/// # 更新时机
	/// - grant_chart_access: 添加新的命盘ID
	/// - revoke_chart_access: 移除命盘ID
	/// - revoke_all_chart_access: 批量移除
	/// - delete_multi_key_encrypted_chart: 移除所有被授权方的索引
	#[pallet::storage]
	#[pallet::getter(fn provider_grants)]
	pub type ProviderGrants<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<u64, T::MaxChartsPerAccount>,  // 复用 MaxChartsPerAccount 限制
		ValueQuery,
	>;

	// ================================
	// 统一隐私模式存储 (Phase 1.2.4)
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
		// 多方授权加密系统事件
		// ================================

		/// 用户加密公钥注册成功 [账户]
		EncryptionKeyRegistered {
			account: T::AccountId,
		},
		/// 用户加密公钥更新成功 [账户]
		EncryptionKeyUpdated {
			account: T::AccountId,
		},
		/// 多方授权加密命盘创建成功 [所有者, 命盘ID, 初始授权数]
		MultiKeyEncryptedChartCreated {
			owner: T::AccountId,
			chart_id: u64,
			initial_grants: u8,
		},
		/// 命盘访问授权成功 [命盘ID, 所有者, 被授权方, 角色, 范围, 过期区块]
		ChartAccessGranted {
			chart_id: u64,
			owner: T::AccountId,
			grantee: T::AccountId,
			role: crate::types::AccessRole,
			scope: crate::types::AccessScope,
			expires_at: u32,
		},
		/// 命盘访问授权撤销 [命盘ID, 所有者, 被撤销方]
		ChartAccessRevoked {
			chart_id: u64,
			owner: T::AccountId,
			revokee: T::AccountId,
		},
		/// 批量撤销所有授权 [命盘ID, 所有者, 撤销数量]
		AllChartAccessRevoked {
			chart_id: u64,
			owner: T::AccountId,
			count: u8,
		},
		/// 多方授权加密命盘删除成功 [所有者, 命盘ID]
		MultiKeyEncryptedChartDeleted {
			owner: T::AccountId,
			chart_id: u64,
		},

		// ================================
		// 服务提供者系统事件
		// ================================

		/// 服务提供者注册成功 [账户, 服务类型]
		ServiceProviderRegistered {
			account: T::AccountId,
			provider_type: crate::types::ServiceProviderType,
		},
		/// 服务提供者公钥更新 [账户]
		ServiceProviderKeyUpdated {
			account: T::AccountId,
		},
		/// 服务提供者状态变更 [账户, 是否激活]
		ServiceProviderStatusChanged {
			account: T::AccountId,
			is_active: bool,
		},
		/// 服务提供者信誉分更新 [账户, 新信誉分]
		ServiceProviderReputationUpdated {
			account: T::AccountId,
			new_reputation: u8,
		},
		/// 服务提供者注销 [账户]
		ServiceProviderUnregistered {
			account: T::AccountId,
		},

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
		// 多方授权加密系统错误
		// ================================

		/// 加密公钥已注册
		EncryptionKeyAlreadyRegistered,
		/// 加密公钥未注册
		EncryptionKeyNotRegistered,
		/// 无效的加密公钥
		InvalidEncryptionKey,
		/// 缺少所有者密钥条目
		MissingOwnerKey,
		/// 授权数量超限（最多10个）
		TooManyAuthorizations,
		/// 不能授权给自己
		CannotGrantToSelf,
		/// 已经授权过
		AlreadyGranted,
		/// 授权不存在
		GrantNotFound,
		/// 不能撤销所有者权限
		CannotRevokeOwner,
		/// 授权已过期
		AuthorizationExpired,
		/// 加密密钥无效
		InvalidEncryptedKey,
		/// 数据哈希不匹配
		DataHashMismatch,
		/// 多方授权加密命盘未找到
		MultiKeyEncryptedChartNotFound,

		// ================================
		// 服务提供者系统错误
		// ================================

		/// 服务提供者已注册
		ProviderAlreadyRegistered,
		/// 服务提供者未注册
		ProviderNotRegistered,
		/// 该类型提供者数量已满（最多100个）
		TooManyProvidersOfType,
		/// 无效的服务提供者类型
		InvalidProviderType,
		/// 服务提供者已被禁用
		ProviderDisabled,

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
		// 多方授权加密系统交易
		// ================================

		/// 注册用户加密公钥
		///
		/// 用户必须先注册 X25519 加密公钥，才能：
		/// 1. 创建多方授权加密命盘
		/// 2. 被授权访问他人的加密命盘
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者
		/// - `public_key`: X25519 公钥（32 bytes）
		///
		/// # 验证
		///
		/// - 公钥必须是 32 bytes
		/// - 用户不能重复注册（已注册的请使用 update_encryption_key）
		///
		/// # 密钥类型说明
		///
		/// 此公钥与 Polkadot 账户的签名公钥（SR25519/ED25519）不同：
		/// - ED25519 账户：可从签名密钥派生 X25519 密钥
		/// - SR25519 账户：需要独立生成 X25519 密钥对
		///
		/// 前端应在用户首次使用加密功能时自动生成并注册。
		#[pallet::call_index(40)]
		#[pallet::weight(T::WeightInfo::create_bazi_chart())]
		pub fn register_encryption_key(
			origin: OriginFor<T>,
			public_key: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 检查是否已注册
			ensure!(
				!UserEncryptionKeys::<T>::contains_key(&who),
				Error::<T>::EncryptionKeyAlreadyRegistered
			);

			// 存储公钥
			UserEncryptionKeys::<T>::insert(&who, public_key);

			// 触发事件
			Self::deposit_event(Event::EncryptionKeyRegistered {
				account: who,
			});

			Ok(())
		}

		/// 更新用户加密公钥
		///
		/// 用于密钥轮换或密钥丢失后重新注册。
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者
		/// - `new_public_key`: 新的 X25519 公钥（32 bytes）
		///
		/// # 验证
		///
		/// - 用户必须已注册加密公钥
		///
		/// # 注意
		///
		/// 更新公钥后，之前用旧公钥加密的 DataKey 将无法解密！
		/// 用户需要：
		/// 1. 用旧私钥解密所有命盘的 DataKey
		/// 2. 用新公钥重新加密并更新链上数据
		///
		/// 建议：除非密钥泄露，否则不要轻易更换公钥。
		#[pallet::call_index(41)]
		#[pallet::weight(T::WeightInfo::create_bazi_chart())]
		pub fn update_encryption_key(
			origin: OriginFor<T>,
			new_public_key: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 检查是否已注册
			ensure!(
				UserEncryptionKeys::<T>::contains_key(&who),
				Error::<T>::EncryptionKeyNotRegistered
			);

			// 更新公钥
			UserEncryptionKeys::<T>::insert(&who, new_public_key);

			// 触发事件
			Self::deposit_event(Event::EncryptionKeyUpdated {
				account: who,
			});

			Ok(())
		}

		/// 创建支持多方授权的加密八字命盘
		///
		/// # 功能
		///
		/// 创建一个支持多方授权的加密命盘，敏感数据使用 AES-256-GCM 加密，
		/// DataKey 使用各授权方的 X25519 公钥加密后存储。
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者（所有者）
		/// - `sizhu_index`: 四柱干支索引（明文，用于免费计算解盘）
		/// - `gender`: 性别（明文，用于大运计算）
		/// - `encrypted_data`: AES-256-GCM 加密的敏感数据（出生时间等）
		/// - `nonce`: 加密使用的 nonce（12 bytes）
		/// - `auth_tag`: 认证标签（16 bytes）
		/// - `encrypted_keys`: 多个加密的 DataKey（最多 10 个授权）
		/// - `data_hash`: 原始敏感数据的 Blake2-256 哈希（用于验证解密正确性）
		///
		/// # 验证
		///
		/// - `sizhu_index` 必须有效（所有干支索引在有效范围内）
		/// - `encrypted_keys` 必须包含一个 `Owner` 角色且账户等于调用者
		/// - 调用者必须已注册加密公钥
		/// - 账户的命盘数量不能超过 `MaxChartsPerAccount`
		///
		/// # 存储结构
		///
		/// 约 400-1400 bytes，取决于授权数量：
		/// - 基础信息：~100 bytes
		/// - encrypted_data：最大 256 bytes
		/// - 每个 EncryptedKeyEntry：~100 bytes
		/// - 最大 10 个授权：~1000 bytes
		///
		/// # 安全特性
		///
		/// - 出生时间等敏感数据在前端加密后存储，链上只有密文
		/// - 每个授权方有独立加密的 DataKey，撤销时只需删除对应条目
		/// - 四柱索引明文存储，支持 Runtime API 免费计算解盘
		#[pallet::call_index(42)]
		#[pallet::weight(T::WeightInfo::create_bazi_chart())]
		pub fn create_multi_key_encrypted_chart(
			origin: OriginFor<T>,
			sizhu_index: crate::types::SiZhuIndex,
			gender: Gender,
			encrypted_data: BoundedVec<u8, ConstU32<256>>,
			nonce: [u8; 12],
			auth_tag: [u8; 16],
			encrypted_keys: BoundedVec<crate::types::EncryptedKeyEntry<T::AccountId>, ConstU32<10>>,
			data_hash: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 验证四柱索引有效性
			ensure!(sizhu_index.is_valid(), Error::<T>::InvalidSiZhuIndex);

			// 2. 验证调用者已注册加密公钥
			ensure!(
				UserEncryptionKeys::<T>::contains_key(&who),
				Error::<T>::EncryptionKeyNotRegistered
			);

			// 3. 验证必须有 Owner 密钥且账户匹配调用者
			let has_owner = encrypted_keys.iter().any(|k|
				k.account == who && k.role == crate::types::AccessRole::Owner
			);
			ensure!(has_owner, Error::<T>::MissingOwnerKey);

			// 4. 检查账户命盘数量限制
			let existing_charts = UserMultiKeyEncryptedCharts::<T>::get(&who);
			ensure!(
				existing_charts.len() < T::MaxChartsPerAccount::get() as usize,
				Error::<T>::TooManyCharts
			);

			// 5. 获取新的 chart_id
			let chart_id = NextChartId::<T>::get();
			ensure!(chart_id < u64::MAX, Error::<T>::ChartIdOverflow);

			// 6. 获取当前区块号
			let current_block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();

			// 7. 构建多方授权加密命盘结构
			let chart = crate::types::MultiKeyEncryptedBaziChart {
				owner: who.clone(),
				sizhu_index,
				gender,
				encrypted_data,
				nonce,
				auth_tag,
				encrypted_keys: encrypted_keys.clone(),
				data_hash,
				created_at: current_block,
			};

			// 8. 存储到 MultiKeyEncryptedChartById
			MultiKeyEncryptedChartById::<T>::insert(chart_id, chart);

			// 9. 添加到用户的多方授权加密八字列表
			UserMultiKeyEncryptedCharts::<T>::try_mutate(&who, |charts| {
				charts.try_push(chart_id).map_err(|_| Error::<T>::TooManyCharts)
			})?;

			// 10. 递增计数器
			NextChartId::<T>::put(chart_id + 1);

			// 11. 计算初始授权数（不含 Owner）
			let initial_grants = encrypted_keys.iter()
				.filter(|k| k.role != crate::types::AccessRole::Owner)
				.count() as u8;

			// 12. 触发事件
			Self::deposit_event(Event::MultiKeyEncryptedChartCreated {
				owner: who,
				chart_id,
				initial_grants,
			});

			Ok(())
		}

		/// 授权新账户访问多方授权加密命盘
		///
		/// # 功能
		///
		/// 所有者可以授权新的账户访问其加密命盘。授权时需要提供用被授权方
		/// X25519 公钥加密的 DataKey。
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者（必须是命盘所有者）
		/// - `chart_id`: 命盘 ID
		/// - `grantee`: 被授权账户
		/// - `encrypted_key`: 用被授权方 X25519 公钥加密的 DataKey（最大 72 bytes）
		/// - `role`: 授权角色（Master/Family/AiService）
		/// - `scope`: 访问范围（ReadOnly/CanComment/FullAccess）
		/// - `expires_at`: 授权结束区块号（0 = 永久有效）
		///
		/// # 权限
		///
		/// - 只有所有者可以授权
		/// - 不能授权给自己（已经是 Owner）
		/// - 不能重复授权同一账户
		/// - 授权数量不能超过 10 个
		///
		/// # 流程
		///
		/// 1. 所有者从链上读取自己的加密 DataKey
		/// 2. 用自己的 X25519 私钥解密 DataKey
		/// 3. 从链上获取被授权方的 X25519 公钥
		/// 4. 用被授权方公钥重新加密 DataKey
		/// 5. 提交此交易
		#[pallet::call_index(43)]
		#[pallet::weight(T::WeightInfo::create_bazi_chart())]
		pub fn grant_chart_access(
			origin: OriginFor<T>,
			chart_id: u64,
			grantee: T::AccountId,
			encrypted_key: BoundedVec<u8, ConstU32<72>>,
			role: crate::types::AccessRole,
			scope: crate::types::AccessScope,
			expires_at: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 获取命盘并验证所有权
			let mut chart = MultiKeyEncryptedChartById::<T>::get(chart_id)
				.ok_or(Error::<T>::MultiKeyEncryptedChartNotFound)?;
			ensure!(chart.owner == who, Error::<T>::NotChartOwner);

			// 2. 不能授权给自己
			ensure!(grantee != who, Error::<T>::CannotGrantToSelf);

			// 3. 检查被授权方是否已注册加密公钥
			ensure!(
				UserEncryptionKeys::<T>::contains_key(&grantee),
				Error::<T>::EncryptionKeyNotRegistered
			);

			// 4. 检查是否已授权
			let already_granted = chart.encrypted_keys.iter()
				.any(|k| k.account == grantee);
			ensure!(!already_granted, Error::<T>::AlreadyGranted);

			// 5. 检查授权数量限制（最多 10 个）
			ensure!(
				chart.encrypted_keys.len() < 10,
				Error::<T>::TooManyAuthorizations
			);

			// 6. 不能授权 Owner 角色（Owner 只能在创建时指定）
			ensure!(
				role != crate::types::AccessRole::Owner,
				Error::<T>::CannotGrantToSelf  // 复用错误：Owner 角色不能被授权
			);

			// 7. 获取当前区块号
			let current_block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();

			// 8. 创建新的授权条目
			let entry = crate::types::EncryptedKeyEntry {
				account: grantee.clone(),
				encrypted_key,
				role,
				scope,
				granted_at: current_block,
				expires_at,
			};

			// 9. 添加到命盘的授权列表
			chart.encrypted_keys.try_push(entry)
				.map_err(|_| Error::<T>::TooManyAuthorizations)?;

			// 10. 更新存储
			MultiKeyEncryptedChartById::<T>::insert(chart_id, chart);

			// 11. 更新反向索引（ProviderGrants）
			ProviderGrants::<T>::try_mutate(&grantee, |grants| {
				// 如果还没有这个 chart_id，则添加
				if !grants.iter().any(|&id| id == chart_id) {
					grants.try_push(chart_id)
						.map_err(|_| Error::<T>::TooManyCharts)
				} else {
					Ok(())
				}
			})?;

			// 12. 触发事件
			Self::deposit_event(Event::ChartAccessGranted {
				chart_id,
				owner: who,
				grantee,
				role,
				scope,
				expires_at,
			});

			Ok(())
		}

		/// 撤销账户的访问权限
		///
		/// # 功能
		///
		/// 所有者可以撤销任何非 Owner 账户的访问权限。
		/// 撤销后，被撤销方无法再从链上读取自己的加密 DataKey。
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者（必须是命盘所有者）
		/// - `chart_id`: 命盘 ID
		/// - `revokee`: 被撤销账户
		///
		/// # 权限
		///
		/// - 只有所有者可以撤销
		/// - 不能撤销 Owner 自己的权限
		/// - 被撤销的授权必须存在
		///
		/// # 安全说明
		///
		/// 撤销授权后：
		/// 1. 链上立即移除 encrypted_key 条目
		/// 2. 被撤销方无法再从链上读取加密的 DataKey
		/// 3. 如果之前已解密并保存了明文数据，链上无法阻止
		#[pallet::call_index(44)]
		#[pallet::weight(T::WeightInfo::delete_bazi_chart())]
		pub fn revoke_chart_access(
			origin: OriginFor<T>,
			chart_id: u64,
			revokee: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 获取命盘并验证所有权
			let mut chart = MultiKeyEncryptedChartById::<T>::get(chart_id)
				.ok_or(Error::<T>::MultiKeyEncryptedChartNotFound)?;
			ensure!(chart.owner == who, Error::<T>::NotChartOwner);

			// 2. 不能撤销所有者自己
			ensure!(revokee != who, Error::<T>::CannotRevokeOwner);

			// 3. 检查授权是否存在
			let existed = chart.encrypted_keys.iter()
				.any(|k| k.account == revokee);
			ensure!(existed, Error::<T>::GrantNotFound);

			// 4. 移除授权
			chart.encrypted_keys.retain(|k| k.account != revokee);

			// 5. 更新存储
			MultiKeyEncryptedChartById::<T>::insert(chart_id, chart);

			// 6. 更新反向索引（ProviderGrants）- 移除该命盘ID
			ProviderGrants::<T>::mutate(&revokee, |grants| {
				grants.retain(|&id| id != chart_id);
			});

			// 7. 触发事件
			Self::deposit_event(Event::ChartAccessRevoked {
				chart_id,
				owner: who,
				revokee,
			});

			Ok(())
		}

		/// 批量撤销所有授权（紧急情况）
		///
		/// # 功能
		///
		/// 所有者可以一次性撤销所有非 Owner 的授权。
		/// 适用于紧急情况，如发现密钥泄露等。
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者（必须是命盘所有者）
		/// - `chart_id`: 命盘 ID
		///
		/// # 权限
		///
		/// - 只有所有者可以执行
		/// - Owner 自己的授权不会被撤销
		#[pallet::call_index(45)]
		#[pallet::weight(T::WeightInfo::delete_bazi_chart())]
		pub fn revoke_all_chart_access(
			origin: OriginFor<T>,
			chart_id: u64,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 获取命盘并验证所有权
			let mut chart = MultiKeyEncryptedChartById::<T>::get(chart_id)
				.ok_or(Error::<T>::MultiKeyEncryptedChartNotFound)?;
			ensure!(chart.owner == who, Error::<T>::NotChartOwner);

			// 2. 收集要撤销的授权账户（不含 Owner）
			let revoked_accounts: sp_std::vec::Vec<_> = chart.encrypted_keys.iter()
				.filter(|k| k.role != crate::types::AccessRole::Owner)
				.map(|k| k.account.clone())
				.collect();

			let revoked_count = revoked_accounts.len() as u8;

			// 3. 只保留 Owner 的授权
			chart.encrypted_keys.retain(|k| k.role == crate::types::AccessRole::Owner);

			// 4. 更新存储
			MultiKeyEncryptedChartById::<T>::insert(chart_id, chart);

			// 5. 更新反向索引（ProviderGrants）- 移除所有被撤销账户的该命盘ID
			for account in revoked_accounts.iter() {
				ProviderGrants::<T>::mutate(account, |grants| {
					grants.retain(|&id| id != chart_id);
				});
			}

			// 6. 触发事件
			Self::deposit_event(Event::AllChartAccessRevoked {
				chart_id,
				owner: who,
				count: revoked_count,
			});

			Ok(())
		}

		/// 删除多方授权加密命盘
		///
		/// # 功能
		///
		/// 所有者可以删除自己的多方授权加密命盘。
		/// 删除后，命盘数据将从链上移除。
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者（必须是命盘所有者）
		/// - `chart_id`: 命盘 ID
		///
		/// # 权限
		///
		/// - 只有所有者可以删除
		#[pallet::call_index(46)]
		#[pallet::weight(T::WeightInfo::delete_bazi_chart())]
		pub fn delete_multi_key_encrypted_chart(
			origin: OriginFor<T>,
			chart_id: u64,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 获取命盘并验证所有权
			let chart = MultiKeyEncryptedChartById::<T>::get(chart_id)
				.ok_or(Error::<T>::MultiKeyEncryptedChartNotFound)?;
			ensure!(chart.owner == who, Error::<T>::NotChartOwner);

			// 2. 收集所有被授权方（不含 Owner），用于清理 ProviderGrants 索引
			let granted_accounts: sp_std::vec::Vec<_> = chart.encrypted_keys.iter()
				.filter(|k| k.role != crate::types::AccessRole::Owner)
				.map(|k| k.account.clone())
				.collect();

			// 3. 从存储中删除
			MultiKeyEncryptedChartById::<T>::remove(chart_id);

			// 4. 从用户的命盘列表中删除
			UserMultiKeyEncryptedCharts::<T>::try_mutate(&who, |charts| -> DispatchResult {
				if let Some(pos) = charts.iter().position(|&id| id == chart_id) {
					charts.remove(pos);
				}
				Ok(())
			})?;

			// 5. 清理所有被授权方的 ProviderGrants 索引
			for account in granted_accounts.iter() {
				ProviderGrants::<T>::mutate(account, |grants| {
					grants.retain(|&id| id != chart_id);
				});
			}

			// 6. 触发事件
			Self::deposit_event(Event::MultiKeyEncryptedChartDeleted {
				owner: who,
				chart_id,
			});

			Ok(())
		}

		// ================================
		// 服务提供者系统交易
		// ================================

		/// 注册为服务提供者
		///
		/// # 功能
		///
		/// 账户注册为服务提供者（命理师、AI 服务等），注册后可以：
		/// 1. 出现在服务提供者列表中，供用户选择
		/// 2. 接收用户授权的命盘访问权限
		/// 3. 使用注册的 X25519 公钥解密 DataKey
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者
		/// - `provider_type`: 服务类型（命理师/AI服务/家族成员/研究机构）
		/// - `public_key`: X25519 公钥（32 bytes），用于接收加密的 DataKey
		///
		/// # 验证
		///
		/// - 账户不能重复注册为提供者
		/// - 公钥必须是 32 bytes
		/// - 该类型的提供者数量不能超过 100 个
		///
		/// # 注册后
		///
		/// - 初始信誉分为 50
		/// - 默认为激活状态
		/// - 同时注册到 UserEncryptionKeys（如未注册）
		#[pallet::call_index(50)]
		#[pallet::weight(T::WeightInfo::create_bazi_chart())]
		pub fn register_provider(
			origin: OriginFor<T>,
			provider_type: crate::types::ServiceProviderType,
			public_key: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 检查是否已注册为提供者
			ensure!(
				!ServiceProviders::<T>::contains_key(&who),
				Error::<T>::ProviderAlreadyRegistered
			);

			// 2. 获取当前区块号
			let current_block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();

			// 3. 创建服务提供者信息
			let provider = crate::types::ServiceProvider::new(
				who.clone(),
				provider_type,
				public_key,
				current_block,
			);

			// 4. 存储提供者信息
			ServiceProviders::<T>::insert(&who, provider);

			// 5. 添加到类型索引
			ProvidersByType::<T>::try_mutate(provider_type, |providers| {
				providers.try_push(who.clone())
					.map_err(|_| Error::<T>::TooManyProvidersOfType)
			})?;

			// 6. 同步注册到 UserEncryptionKeys（如未注册）
			if !UserEncryptionKeys::<T>::contains_key(&who) {
				UserEncryptionKeys::<T>::insert(&who, public_key);
			}

			// 7. 触发事件
			Self::deposit_event(Event::ServiceProviderRegistered {
				account: who,
				provider_type,
			});

			Ok(())
		}

		/// 更新服务提供者公钥
		///
		/// # 功能
		///
		/// 服务提供者更新其 X25519 公钥。
		/// 用于密钥轮换或密钥泄露后重新注册。
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者（必须是已注册的提供者）
		/// - `new_public_key`: 新的 X25519 公钥（32 bytes）
		///
		/// # 注意
		///
		/// 更新公钥后，之前用旧公钥加密的 DataKey 将无法解密！
		/// 需要等待用户重新授权。
		#[pallet::call_index(51)]
		#[pallet::weight(T::WeightInfo::create_bazi_chart())]
		pub fn update_provider_key(
			origin: OriginFor<T>,
			new_public_key: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 获取并验证提供者存在
			let mut provider = ServiceProviders::<T>::get(&who)
				.ok_or(Error::<T>::ProviderNotRegistered)?;

			// 2. 更新公钥
			provider.update_public_key(new_public_key);

			// 3. 存储更新
			ServiceProviders::<T>::insert(&who, provider);

			// 4. 同步更新 UserEncryptionKeys
			UserEncryptionKeys::<T>::insert(&who, new_public_key);

			// 5. 触发事件
			Self::deposit_event(Event::ServiceProviderKeyUpdated {
				account: who,
			});

			Ok(())
		}

		/// 设置服务提供者激活状态
		///
		/// # 功能
		///
		/// 服务提供者可以临时禁用自己，禁用后：
		/// - 不会出现在服务提供者列表中
		/// - 已有的授权仍然有效
		/// - 可以随时重新激活
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者（必须是已注册的提供者）
		/// - `is_active`: 是否激活
		#[pallet::call_index(52)]
		#[pallet::weight(T::WeightInfo::create_bazi_chart())]
		pub fn set_provider_active(
			origin: OriginFor<T>,
			is_active: bool,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 获取并验证提供者存在
			let mut provider = ServiceProviders::<T>::get(&who)
				.ok_or(Error::<T>::ProviderNotRegistered)?;

			// 2. 更新激活状态
			provider.set_active(is_active);

			// 3. 存储更新
			ServiceProviders::<T>::insert(&who, provider);

			// 4. 触发事件
			Self::deposit_event(Event::ServiceProviderStatusChanged {
				account: who,
				is_active,
			});

			Ok(())
		}

		/// 注销服务提供者
		///
		/// # 功能
		///
		/// 服务提供者注销后：
		/// - 从服务提供者列表中移除
		/// - 不再接收新的授权
		/// - 已有的授权仍然有效（用户需要手动撤销）
		///
		/// # 参数
		///
		/// - `origin`: 交易发起者（必须是已注册的提供者）
		#[pallet::call_index(53)]
		#[pallet::weight(T::WeightInfo::delete_bazi_chart())]
		pub fn unregister_provider(
			origin: OriginFor<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 获取提供者信息
			let provider = ServiceProviders::<T>::get(&who)
				.ok_or(Error::<T>::ProviderNotRegistered)?;

			// 2. 从类型索引中移除
			ProvidersByType::<T>::mutate(provider.provider_type, |providers| {
				providers.retain(|a| a != &who);
			});

			// 3. 删除提供者信息
			ServiceProviders::<T>::remove(&who);

			// 4. 触发事件
			Self::deposit_event(Event::ServiceProviderUnregistered {
				account: who,
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

		/// RPC 接口：临时排盘（公历输入，不存储，免费）
		///
		/// 根据公历出生时间计算八字命盘，但不存储到链上。
		/// 适用于用户"试用"功能，决定是否保存后再调用交易接口。
		///
		/// # 参数
		/// - year: 公历年份 (1900-2100)
		/// - month: 公历月份 (1-12)
		/// - day: 公历日期 (1-31)
		/// - hour: 小时 (0-23)
		/// - minute: 分钟 (0-59)
		/// - gender: 性别
		/// - zishi_mode: 子时模式
		/// - longitude: 出生地经度（可选，用于真太阳时修正）
		///
		/// # 返回
		/// - Some(FullBaziChartForApi): 完整命盘数据
		/// - None: 输入参数无效
		pub fn calculate_bazi_temp(
			year: u16,
			month: u8,
			day: u8,
			hour: u8,
			minute: u8,
			gender: Gender,
			zishi_mode: ZiShiMode,
			longitude: Option<i32>,
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
				crate::types::InputCalendarType::Solar, // 公历输入
			))
		}

		/// RPC 接口：临时排盘（内部函数，支持指定日历类型）
		///
		/// 与 `calculate_bazi_temp` 相同，但允许指定输入日历类型
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
				// 公历输入: [year, month, day, hour, minute]
				0 => {
					if params.len() < 5 { return None; }
					Self::calculate_bazi_temp(
						params[0],
						params[1] as u8,
						params[2] as u8,
						params[3] as u8,
						params[4] as u8,
						gender_enum,
						zishi_mode_enum,
						None,
					)
				}
				// 农历输入: [year, month, day, is_leap_month, hour, minute]
				1 => {
					if params.len() < 6 { return None; }
					let lunar_year = params[0];
					let lunar_month = params[1] as u8;
					let lunar_day = params[2] as u8;
					let is_leap = params[3] != 0;
					let hour = params[4] as u8;
					let minute = params[5] as u8;

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
						None,
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
		// V6 新增：多方授权加密系统 API 辅助函数
		// ================================

		/// RPC 接口：获取用户加密公钥
		///
		/// # 参数
		/// - account: 用户账户
		///
		/// # 返回
		/// - Some([u8; 32]): X25519 公钥
		/// - None: 用户未注册加密公钥
		pub fn get_user_encryption_key(account: &T::AccountId) -> Option<[u8; 32]> {
			UserEncryptionKeys::<T>::get(account)
		}

		/// RPC 接口：获取服务提供者信息（JSON）
		///
		/// # 参数
		/// - account: 服务提供者账户
		///
		/// # 返回
		/// - Some(String): JSON 格式的服务提供者信息
		/// - None: 未注册为服务提供者
		pub fn get_service_provider_json(account: &T::AccountId) -> Option<scale_info::prelude::string::String> {
			let provider = ServiceProviders::<T>::get(account)?;

			// 手动构建 JSON
			let provider_type_str = match provider.provider_type {
				crate::types::ServiceProviderType::MingLiShi => "MingLiShi",
				crate::types::ServiceProviderType::AiService => "AiService",
				crate::types::ServiceProviderType::FamilyMember => "FamilyMember",
				crate::types::ServiceProviderType::Research => "Research",
			};

			// 公钥转 hex
			let pub_key_hex: scale_info::prelude::string::String = provider.public_key.iter()
				.map(|b| {
					let high = b >> 4;
					let low = b & 0x0f;
					let high_char = if high < 10 { b'0' + high } else { b'a' + high - 10 };
					let low_char = if low < 10 { b'0' + low } else { b'a' + low - 10 };
					scale_info::prelude::string::String::from_utf8(sp_std::vec![high_char, low_char]).unwrap_or_default()
				})
				.collect();

			let json = scale_info::prelude::format!(
				r#"{{"provider_type":"{}","public_key":"{}","reputation":{},"registered_at":{},"is_active":{}}}"#,
				provider_type_str,
				pub_key_hex,
				provider.reputation,
				provider.registered_at,
				provider.is_active
			);

			Some(json)
		}

		/// RPC 接口：获取某类型的服务提供者列表
		///
		/// # 参数
		/// - provider_type: 服务类型（0-3）
		///
		/// # 返回
		/// - 服务提供者账户列表（只返回激活的）
		pub fn get_providers_by_type_filtered(provider_type: u8) -> sp_std::vec::Vec<T::AccountId> {
			let service_type = match provider_type {
				0 => crate::types::ServiceProviderType::MingLiShi,
				1 => crate::types::ServiceProviderType::AiService,
				2 => crate::types::ServiceProviderType::FamilyMember,
				3 => crate::types::ServiceProviderType::Research,
				_ => return sp_std::vec::Vec::new(),
			};

			ProvidersByType::<T>::get(service_type)
				.into_iter()
				.filter(|account| {
					ServiceProviders::<T>::get(account)
						.map(|p| p.is_active)
						.unwrap_or(false)
				})
				.collect()
		}

		/// RPC 接口：获取被授权访问的命盘列表
		///
		/// # 参数
		/// - account: 账户
		///
		/// # 返回
		/// - 被授权访问的命盘 ID 列表
		pub fn get_provider_grants_list(account: &T::AccountId) -> sp_std::vec::Vec<u64> {
			ProviderGrants::<T>::get(account).into_inner()
		}

		/// RPC 接口：获取多方授权加密命盘的基础信息（JSON）
		///
		/// # 参数
		/// - chart_id: 命盘 ID
		///
		/// # 返回
		/// - Some(String): JSON 格式的命盘基础信息
		/// - None: 命盘不存在
		pub fn get_multi_key_encrypted_chart_info_json(chart_id: u64) -> Option<scale_info::prelude::string::String> {
			use codec::Encode;

			let chart = MultiKeyEncryptedChartById::<T>::get(chart_id)?;

			let gender_str = match chart.gender {
				crate::types::Gender::Male => "Male",
				crate::types::Gender::Female => "Female",
			};

			// 四柱索引
			let sizhu_str = scale_info::prelude::format!(
				r#"{{"year_gan":{},"year_zhi":{},"month_gan":{},"month_zhi":{},"day_gan":{},"day_zhi":{},"hour_gan":{},"hour_zhi":{}}}"#,
				chart.sizhu_index.year_gan,
				chart.sizhu_index.year_zhi,
				chart.sizhu_index.month_gan,
				chart.sizhu_index.month_zhi,
				chart.sizhu_index.day_gan,
				chart.sizhu_index.day_zhi,
				chart.sizhu_index.hour_gan,
				chart.sizhu_index.hour_zhi
			);

			// 授权账户列表
			let grant_accounts: sp_std::vec::Vec<_> = chart.encrypted_keys.iter()
				.map(|k| {
					// 将 AccountId 编码为 hex
					let encoded = k.account.encode();
					let hex: scale_info::prelude::string::String = encoded.iter()
						.map(|b| {
							let high = b >> 4;
							let low = b & 0x0f;
							let high_char = if high < 10 { b'0' + high } else { b'a' + high - 10 };
							let low_char = if low < 10 { b'0' + low } else { b'a' + low - 10 };
							scale_info::prelude::string::String::from_utf8(sp_std::vec![high_char, low_char]).unwrap_or_default()
						})
						.collect();
					scale_info::prelude::format!(r#""0x{}""#, hex)
				})
				.collect();

			let grant_accounts_str = grant_accounts.join(",");

			// 所有者 hex
			let owner_encoded = chart.owner.encode();
			let owner_hex: scale_info::prelude::string::String = owner_encoded.iter()
				.map(|b| {
					let high = b >> 4;
					let low = b & 0x0f;
					let high_char = if high < 10 { b'0' + high } else { b'a' + high - 10 };
					let low_char = if low < 10 { b'0' + low } else { b'a' + low - 10 };
					scale_info::prelude::string::String::from_utf8(sp_std::vec![high_char, low_char]).unwrap_or_default()
				})
				.collect();

			let json = scale_info::prelude::format!(
				r#"{{"owner":"0x{}","sizhu_index":{},"gender":"{}","created_at":{},"grants_count":{},"grant_accounts":[{}]}}"#,
				owner_hex,
				sizhu_str,
				gender_str,
				chart.created_at,
				chart.encrypted_keys.len(),
				grant_accounts_str
			);

			Some(json)
		}

		/// RPC 接口：获取多方授权加密命盘的解盘
		///
		/// 基于四柱索引计算解盘，无需解密敏感数据
		///
		/// # 参数
		/// - chart_id: 多方授权加密命盘 ID
		///
		/// # 返回
		/// - Some(FullInterpretation): 完整解盘结果
		/// - None: 命盘不存在
		pub fn get_multi_key_encrypted_chart_interpretation(chart_id: u64) -> Option<crate::interpretation::FullInterpretation> {
			let chart = MultiKeyEncryptedChartById::<T>::get(chart_id)?;
			let current_block = <frame_system::Pallet<T>>::block_number().saturated_into();

			Some(crate::interpretation::calculate_interpretation_from_index(
				&chart.sizhu_index,
				chart.gender,
				current_block,
			))
		}
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
