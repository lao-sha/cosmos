// This is free and unencumbered software released into the public domain.
//
// Anyone is free to copy, modify, publish, use, compile, sell, or
// distribute this software, either in source code form or as a compiled
// binary, for any purpose, commercial or non-commercial, and by any
// means.
//
// In jurisdictions that recognize copyright laws, the author or authors
// of this software dedicate any and all copyright interest in the
// software to the public domain. We make this dedication for the benefit
// of the public at large and to the detriment of our heirs and
// successors. We intend this dedication to be an overt act of
// relinquishment in perpetuity of all present and future rights to this
// software under copyright law.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
// OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
// ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
// OTHER DEALINGS IN THE SOFTWARE.
//
// For more information, please refer to <http://unlicense.org>

// Substrate and Polkadot dependencies
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstBool, ConstU128, ConstU16, ConstU32, ConstU64, ConstU8, VariantCountOf},
	weights::{
		constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
		IdentityFee, Weight,
	},
};
use frame_system::limits::{BlockLength, BlockWeights};
use pallet_transaction_payment::{ConstFeeMultiplier, FungibleAdapter, Multiplier};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_runtime::{traits::One, Perbill};
use sp_version::RuntimeVersion;

// Local module imports
use super::{
	AccountId, Aura, Balance, Balances, Block, BlockNumber, Hash, Nonce, PalletInfo, Runtime,
	RuntimeCall, RuntimeEvent, RuntimeFreezeReason, RuntimeHoldReason, RuntimeOrigin, RuntimeTask,
	System, EXISTENTIAL_DEPOSIT, SLOT_DURATION, VERSION, UNIT, MINUTES, HOURS, DAYS,
};

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;

	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::with_sensible_defaults(
		Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
		NORMAL_DISPATCH_RATIO,
	);
	pub RuntimeBlockLength: BlockLength = BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

/// The default types are being injected by [`derive_impl`](`frame_support::derive_impl`) from
/// [`SoloChainDefaultConfig`](`struct@frame_system::config_preludes::SolochainDefaultConfig`),
/// but overridden as needed.
#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
	/// The block type for the runtime.
	type Block = Block;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<32>;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
	type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type WeightInfo = ();
	type MaxAuthorities = ConstU32<32>;
	type MaxNominators = ConstU32<0>;
	type MaxSetIdSessionEntries = ConstU64<0>;

	type KeyOwnerProof = sp_core::Void;
	type EquivocationReportSystem = ();
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type DoneSlashHandler = ();
}

parameter_types! {
	pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = FungibleAdapter<Balances, ()>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
	type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

/// Configure the pallet-template in pallets/template.
impl pallet_template::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_template::weights::SubstrateWeight<Runtime>;
}

// ============================================================================
// Divination Pallets Configuration
// ============================================================================

// -------------------- Almanac (黄历) --------------------

parameter_types! {
	pub const MaxBatchSize: u32 = 31;
	pub const MaxHistoryYears: u32 = 3;
}

impl pallet_almanac::Config for Runtime {
	type WeightInfo = ();
	type MaxBatchSize = MaxBatchSize;
	type MaxHistoryYears = MaxHistoryYears;
}

// -------------------- Privacy (隐私授权) --------------------

parameter_types! {
	pub const MaxEncryptedDataLen: u32 = 4096;
	pub const MaxEncryptedKeyLen: u32 = 256;
	pub const MaxGranteesPerRecord: u32 = 100;
	pub const MaxRecordsPerUser: u32 = 10000;
	pub const MaxProvidersPerType: u32 = 10000;
	pub const MaxGrantsPerProvider: u32 = 1000;
	pub const MaxAuthorizationsPerBounty: u32 = 100;
}

impl pallet_divination_privacy::Config for Runtime {
	type MaxEncryptedDataLen = MaxEncryptedDataLen;
	type MaxEncryptedKeyLen = MaxEncryptedKeyLen;
	type MaxGranteesPerRecord = MaxGranteesPerRecord;
	type MaxRecordsPerUser = MaxRecordsPerUser;
	type MaxProvidersPerType = MaxProvidersPerType;
	type MaxGrantsPerProvider = MaxGrantsPerProvider;
	type MaxAuthorizationsPerBounty = MaxAuthorizationsPerBounty;
	type EventHandler = ();
	type WeightInfo = ();
}

// -------------------- AI 解读模块 --------------------

parameter_types! {
	pub TreasuryAccountId: AccountId = sp_runtime::AccountId32::new([0u8; 32]);
}

impl pallet_divination_ai::Config for Runtime {
	type AiCurrency = Balances;
	type DivinationProvider = pallet_divination_common::NullDivinationProvider;
	type BaseInterpretationFee = ConstU128<{ 1 * UNIT }>;
	type MinOracleStake = ConstU128<{ 10 * UNIT }>;
	type DisputeDeposit = ConstU128<{ UNIT / 2 }>;
	type RequestTimeout = ConstU32<{ 10 * MINUTES }>;
	type ProcessingTimeout = ConstU32<{ 5 * MINUTES }>;
	type DisputePeriod = ConstU32<{ 1 * HOURS }>;
	type MaxCidLength = ConstU32<128>;
	type MaxOracles = ConstU32<100>;
	type TreasuryAccount = TreasuryAccountId;
	type ArbitratorOrigin = frame_system::EnsureRoot<AccountId>;
	type GovernanceOrigin = frame_system::EnsureRoot<AccountId>;
}

// -------------------- Market (服务市场) --------------------

parameter_types! {
	pub PlatformAccountId: AccountId = sp_runtime::AccountId32::new([1u8; 32]);
}

impl pallet_divination_market::Config for Runtime {
	type Currency = Balances;
	type DivinationProvider = pallet_divination_common::NullDivinationProvider;
	type MinDeposit = ConstU128<{ 10 * UNIT }>;
	type MinServicePrice = ConstU128<{ UNIT / 10 }>;
	type OrderTimeout = ConstU32<{ 24 * HOURS }>;
	type AcceptTimeout = ConstU32<{ 1 * HOURS }>;
	type ReviewPeriod = ConstU32<{ 7 * DAYS }>;
	type WithdrawalCooldown = ConstU32<{ 1 * HOURS }>;
	type MaxNameLength = ConstU32<64>;
	type MaxBioLength = ConstU32<256>;
	type MaxDescriptionLength = ConstU32<512>;
	type MaxCidLength = ConstU32<64>;
	type MaxPackagesPerProvider = ConstU32<10>;
	type MaxFollowUpsPerOrder = ConstU32<5>;
	type PlatformAccount = PlatformAccountId;
	type GovernanceOrigin = frame_system::EnsureRoot<AccountId>;
	type MinReportDeposit = ConstU128<{ UNIT / 10 }>;
	type ReportTimeout = ConstU32<{ 3 * DAYS }>;
	type ReportCooldownPeriod = ConstU32<{ 1 * DAYS }>;
	type ReportWithdrawWindow = ConstU32<{ 1 * HOURS }>;
	type MaliciousReportPenalty = ConstU16<50>;
	type ReportReviewOrigin = frame_system::EnsureSigned<AccountId>;
	type TreasuryAccount = TreasuryAccountId;
}

// -------------------- NFT 模块 --------------------

impl pallet_divination_nft::Config for Runtime {
	type NftCurrency = Balances;
	type DivinationProvider = pallet_divination_common::NullDivinationProvider;
	type MaxNameLength = ConstU32<64>;
	type MaxCidLength = ConstU32<128>;
	type MaxCollectionsPerUser = ConstU32<50>;
	type MaxNftsPerCollection = ConstU32<1000>;
	type MaxOffersPerNft = ConstU32<100>;
	type BaseMintFee = ConstU128<UNIT>;
	type PlatformFeeRate = ConstU16<250>; // 2.5%
	type MaxRoyaltyRate = ConstU16<2500>; // 25%
	type OfferValidityPeriod = ConstU32<{ 7 * DAYS }>;
	type PlatformAccount = PlatformAccountId;
	type GovernanceOrigin = frame_system::EnsureRoot<AccountId>;
}

// -------------------- Meihua (梅花易数) --------------------

/// 模拟随机数生成器 - 生产环境应使用真正的随机源
pub struct InsecureRandomness;

impl frame_support::traits::Randomness<Hash, BlockNumber> for InsecureRandomness {
	fn random(subject: &[u8]) -> (Hash, BlockNumber) {
		let block = System::block_number();
		let hash = sp_core::hashing::blake2_256(subject);
		(Hash::from_slice(&hash), block)
	}
}

parameter_types! {
	pub const StorageDepositPerKb: u128 = UNIT / 10;
	pub const MinStorageDeposit: u128 = UNIT / 100;
	pub const MaxStorageDeposit: u128 = 100 * UNIT;
}

impl pallet_meihua::Config for Runtime {
	type Currency = Balances;
	type Randomness = InsecureRandomness;
	type MaxUserHexagrams = ConstU32<1000>;
	type MaxPublicHexagrams = ConstU32<10000>;
	type DailyFreeDivinations = ConstU32<3>;
	type MaxDailyDivinations = ConstU32<100>;
	type AiInterpretationFee = ConstU128<UNIT>;
	type TreasuryAccount = TreasuryAccountId;
	type AiOracleOrigin = frame_system::EnsureRoot<AccountId>;
	type StorageDepositPerKb = StorageDepositPerKb;
	type MinStorageDeposit = MinStorageDeposit;
	type MaxStorageDeposit = MaxStorageDeposit;
}

// -------------------- Bazi (八字) --------------------

impl pallet_bazi_chart::Config for Runtime {
	type WeightInfo = ();
	type MaxChartsPerAccount = ConstU32<100>;
	type MaxDaYunSteps = ConstU32<12>;
	type MaxCangGan = ConstU32<3>;
	type Currency = Balances;
	type StorageDepositPerKb = StorageDepositPerKb;
	type MinStorageDeposit = MinStorageDeposit;
	type MaxStorageDeposit = MaxStorageDeposit;
}

// -------------------- Liuyao (六爻) --------------------

impl pallet_liuyao::Config for Runtime {
	type Currency = Balances;
	type Randomness = InsecureRandomness;
	type MaxUserGuas = ConstU32<1000>;
	type MaxPublicGuas = ConstU32<10000>;
	type DailyFreeGuas = ConstU32<3>;
	type MaxDailyGuas = ConstU32<100>;
	type MaxCidLen = ConstU32<64>;
	type MaxEncryptedLen = ConstU32<512>;
	type StorageDepositPerKb = StorageDepositPerKb;
	type MinStorageDeposit = MinStorageDeposit;
	type MaxStorageDeposit = MaxStorageDeposit;
}

// -------------------- Qimen (奇门遁甲) --------------------

impl pallet_qimen::Config for Runtime {
	type Currency = Balances;
	type Randomness = InsecureRandomness;
	type MaxUserCharts = ConstU32<1000>;
	type MaxPublicCharts = ConstU32<10000>;
	type DailyFreeCharts = ConstU32<3>;
	type MaxDailyCharts = ConstU32<100>;
	type AiInterpretationFee = ConstU128<UNIT>;
	type TreasuryAccount = TreasuryAccountId;
	type AiOracleOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxCidLen = ConstU32<64>;
	type MaxEncryptedLen = ConstU32<512>;
	type StorageDepositPerKb = StorageDepositPerKb;
	type MinStorageDeposit = MinStorageDeposit;
	type MaxStorageDeposit = MaxStorageDeposit;
}

// -------------------- Ziwei (紫微斗数) --------------------

impl pallet_ziwei::Config for Runtime {
	type Currency = Balances;
	type Randomness = InsecureRandomness;
	type MaxUserCharts = ConstU32<1000>;
	type MaxPublicCharts = ConstU32<10000>;
	type DailyFreeCharts = ConstU32<3>;
	type MaxDailyCharts = ConstU32<100>;
	type AiInterpretationFee = ConstU128<UNIT>;
	type TreasuryAccount = TreasuryAccountId;
	type AiOracleOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxCidLen = ConstU32<64>;
	type MaxEncryptedLen = ConstU32<512>;
	type StorageDepositPerKb = StorageDepositPerKb;
	type MinStorageDeposit = MinStorageDeposit;
	type MaxStorageDeposit = MaxStorageDeposit;
}

// -------------------- Xiaoliuren (小六壬) --------------------

impl pallet_xiaoliuren::Config for Runtime {
	type Currency = Balances;
	type Randomness = InsecureRandomness;
	type MaxUserPans = ConstU32<1000>;
	type MaxPublicPans = ConstU32<10000>;
	type MaxCidLen = ConstU32<64>;
	type DailyFreeDivinations = ConstU32<10>;
	type MaxDailyDivinations = ConstU32<100>;
	type MaxEncryptedLen = ConstU32<512>;
	type AiInterpretationFee = ConstU128<UNIT>;
	type TreasuryAccount = TreasuryAccountId;
	type AiOracleOrigin = frame_system::EnsureRoot<AccountId>;
	type StorageDepositPerKb = StorageDepositPerKb;
	type MinStorageDeposit = MinStorageDeposit;
	type MaxStorageDeposit = MaxStorageDeposit;
}

// -------------------- Daliuren (大六壬) --------------------

impl pallet_daliuren::Config for Runtime {
	type Currency = Balances;
	type Randomness = InsecureRandomness;
	type MaxCidLen = ConstU32<64>;
	type MaxDailyDivinations = ConstU32<50>;
	type MaxEncryptedLen = ConstU32<512>;
	type DivinationFee = ConstU128<UNIT>;
	type AiInterpretationFee = ConstU128<UNIT>;
	type AiSubmitter = frame_system::EnsureSigned<AccountId>;
	type WeightInfo = ();
	type StorageDepositPerKb = StorageDepositPerKb;
	type MinStorageDeposit = MinStorageDeposit;
	type MaxStorageDeposit = MaxStorageDeposit;
}

// -------------------- Tarot (塔罗牌) --------------------

impl pallet_tarot::Config for Runtime {
	type Currency = Balances;
	type Randomness = InsecureRandomness;
	type MaxCardsPerReading = ConstU32<12>;
	type MaxUserReadings = ConstU32<1000>;
	type MaxPublicReadings = ConstU32<10000>;
	type DailyFreeDivinations = ConstU32<3>;
	type MaxDailyDivinations = ConstU32<100>;
	type AiInterpretationFee = ConstU128<UNIT>;
	type TreasuryAccount = TreasuryAccountId;
	type AiOracleOrigin = frame_system::EnsureRoot<AccountId>;
	type StorageDepositPerKb = StorageDepositPerKb;
	type MinStorageDeposit = MinStorageDeposit;
	type MaxStorageDeposit = MaxStorageDeposit;
}

// ============================================================================
// Chat Pallets Configuration
// ============================================================================

// -------------------- Chat Permission (聊天权限) --------------------

impl pallet_chat_permission::Config for Runtime {
	type MaxBlockListSize = ConstU32<1000>;
	type MaxWhitelistSize = ConstU32<1000>;
	type MaxScenesPerPair = ConstU32<50>;
}

// -------------------- Chat Core (私聊核心) --------------------

/// 时间戳提供器 - 使用 pallet_timestamp
pub struct TimestampProvider;

impl frame_support::traits::UnixTime for TimestampProvider {
	fn now() -> core::time::Duration {
		let millis = pallet_timestamp::Pallet::<Runtime>::get();
		core::time::Duration::from_millis(millis)
	}
}

impl pallet_chat_core::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_chat_core::SubstrateWeight<Runtime>;
	type MaxCidLen = ConstU32<128>;
	type MaxSessionsPerUser = ConstU32<1000>;
	type MaxMessagesPerSession = ConstU32<10000>;
	type RateLimitWindow = ConstU32<100>;
	type MaxMessagesPerWindow = ConstU32<50>;
	type MessageExpirationTime = ConstU32<{ 180 * DAYS }>;
	type Randomness = InsecureRandomness;
	type UnixTime = TimestampProvider;
	type MaxNicknameLength = ConstU32<64>;
	type MaxSignatureLength = ConstU32<256>;
}

// -------------------- Chat Group (群聊) --------------------

parameter_types! {
	pub const ChatGroupPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/chatg");
}

impl pallet_chat_group::Config for Runtime {
	type Randomness = InsecureRandomness;
	type TimeProvider = TimestampProvider;
	type MaxGroupNameLen = ConstU32<64>;
	type MaxGroupDescriptionLen = ConstU32<256>;
	type MaxGroupMembers = ConstU32<1000>;
	type MaxGroupsPerUser = ConstU32<100>;
	type MaxMessageLen = ConstU32<4096>;
	type MaxGroupMessageHistory = ConstU32<10000>;
	type MaxCidLen = ConstU32<128>;
	type MaxKeyLen = ConstU32<256>;
	type PalletId = ChatGroupPalletId;
	type MessageRateLimit = ConstU32<60>; // 每分钟最多60条消息
	type GroupCreationCooldown = ConstU32<{ 10 * MINUTES }>; // 创建群组冷却时间
	type WeightInfo = ();
}

// ============================================================================
// Trading Pallets Configuration
// ============================================================================

// -------------------- Pricing (价格预言机) --------------------

impl pallet_trading_pricing::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxPriceDeviation = ConstU16<2000>; // 20% 最大价格偏离
	type ExchangeRateUpdateInterval = ConstU32<{ 24 * HOURS }>; // 24小时更新汇率
}

// -------------------- Credit (信用风控) --------------------

impl pallet_trading_credit::Config for Runtime {
	type Currency = Balances;
	// 买家信用配置
	type InitialBuyerCreditScore = ConstU16<500>;
	type OrderCompletedBonus = ConstU16<10>;
	type OrderDefaultPenalty = ConstU16<50>;
	type BlocksPerDay = ConstU32<{ DAYS }>;
	type MinimumBalance = ConstU128<{ 100 * UNIT }>;
	// 做市商信用配置
	type InitialMakerCreditScore = ConstU16<820>;
	type MakerOrderCompletedBonus = ConstU16<2>;
	type MakerOrderTimeoutPenalty = ConstU16<10>;
	type MakerDisputeLossPenalty = ConstU16<20>;
	type MakerSuspensionThreshold = ConstU16<750>;
	type MakerWarningThreshold = ConstU16<800>;
	type CreditWeightInfo = ();
}

// -------------------- Maker (做市商管理) --------------------

/// Pricing Provider 实现
pub struct TradingPricingProvider;

impl pallet_trading_maker::PricingProvider<Balance> for TradingPricingProvider {
	fn get_dust_to_usd_rate() -> Option<Balance> {
		let price = pallet_trading_pricing::Pallet::<Runtime>::get_dust_market_price_weighted();
		if price > 0 {
			Some(price as Balance)
		} else {
			None
		}
	}
}

impl pallet_trading_maker::Config for Runtime {
	type Currency = Balances;
	type MakerCredit = pallet_trading_credit::Pallet<Runtime>;
	type GovernanceOrigin = frame_system::EnsureRoot<AccountId>;
	type Timestamp = TimestampProvider;
	type MakerDepositAmount = ConstU128<{ 1000 * UNIT }>;
	type TargetDepositUsd = ConstU64<1000_000_000>; // 1000 USD
	type DepositReplenishThreshold = ConstU64<950_000_000>; // 950 USD
	type DepositReplenishTarget = ConstU64<1050_000_000>; // 1050 USD
	type PriceCheckInterval = ConstU32<{ HOURS }>; // 每小时检查
	type AppealDeadline = ConstU32<{ 7 * DAYS }>; // 7天申诉期
	type Pricing = TradingPricingProvider;
	type MakerApplicationTimeout = ConstU32<{ 7 * DAYS }>;
	type WithdrawalCooldown = ConstU32<{ 7 * DAYS }>;
	type WeightInfo = ();
}

// -------------------- Bridge (桥接服务) --------------------

/// Bridge Pricing Provider 实现
impl pallet_trading_bridge::pallet::PricingProvider<Balance> for TradingPricingProvider {
	fn get_dust_to_usd_rate() -> Option<Balance> {
		let price = pallet_trading_pricing::Pallet::<Runtime>::get_dust_market_price_weighted();
		if price > 0 {
			Some(price as Balance)
		} else {
			None
		}
	}
}

/// Bridge Maker 接口适配器
pub struct BridgeMakerAdapter;

impl pallet_trading_bridge::pallet::MakerInterface<AccountId, Balance> for BridgeMakerAdapter {
	fn get_maker_application(maker_id: u64) -> Option<pallet_trading_bridge::pallet::MakerApplicationInfo<AccountId, Balance>> {
		pallet_trading_maker::Pallet::<Runtime>::maker_applications(maker_id).map(|app| {
			pallet_trading_bridge::pallet::MakerApplicationInfo {
				account: app.owner,
				tron_address: app.tron_address,
				is_active: app.status == pallet_trading_maker::pallet::ApplicationStatus::Active,
				_phantom: core::marker::PhantomData,
			}
		})
	}

	fn is_maker_active(maker_id: u64) -> bool {
		pallet_trading_maker::Pallet::<Runtime>::is_maker_active(maker_id)
	}

	fn get_maker_id(who: &AccountId) -> Option<u64> {
		pallet_trading_maker::Pallet::<Runtime>::get_maker_id(who)
	}
}

/// Bridge Credit 接口适配器
pub struct BridgeCreditAdapter;

impl pallet_trading_bridge::pallet::CreditInterface for BridgeCreditAdapter {
	fn record_maker_order_completed(maker_id: u64, order_id: u64, response_time_seconds: u32) -> sp_runtime::DispatchResult {
		pallet_trading_credit::Pallet::<Runtime>::record_maker_order_completed(maker_id, order_id, response_time_seconds)
	}

	fn record_maker_order_timeout(maker_id: u64, order_id: u64) -> sp_runtime::DispatchResult {
		pallet_trading_credit::Pallet::<Runtime>::record_maker_order_timeout(maker_id, order_id)
	}

	fn record_maker_dispute_result(maker_id: u64, order_id: u64, maker_win: bool) -> sp_runtime::DispatchResult {
		pallet_trading_credit::Pallet::<Runtime>::record_maker_dispute_result(maker_id, order_id, maker_win)
	}
}

impl pallet_trading_bridge::Config for Runtime {
	type Currency = Balances;
	type Escrow = pallet_escrow::Pallet<Runtime>;
	type Pricing = TradingPricingProvider;
	type MakerPallet = BridgeMakerAdapter;
	type Credit = BridgeCreditAdapter;
	type GovernanceOrigin = frame_system::EnsureRoot<AccountId>;
	type SwapTimeout = ConstU32<{ 24 * HOURS }>; // 24小时超时
	type OcwSwapTimeoutBlocks = ConstU32<{ 1 * HOURS }>; // OCW 1小时超时
	type MinSwapAmount = ConstU128<{ 10 * UNIT }>; // 最小兑换10 DUST
	type WeightInfo = ();
}

// -------------------- OTC (场外交易) --------------------

/// OTC Pricing Provider 实现
impl pallet_trading_otc::pallet::PricingProvider<Balance> for TradingPricingProvider {
	fn get_dust_to_usd_rate() -> Option<Balance> {
		let price = pallet_trading_pricing::Pallet::<Runtime>::get_dust_market_price_weighted();
		if price > 0 {
			Some(price as Balance)
		} else {
			None
		}
	}
}

/// OTC Maker 接口适配器
pub struct OtcMakerAdapter;

impl pallet_trading_otc::pallet::MakerInterface<AccountId, Balance> for OtcMakerAdapter {
	fn get_maker_application(maker_id: u64) -> Option<pallet_trading_otc::pallet::MakerApplicationInfo<AccountId, Balance>> {
		pallet_trading_maker::Pallet::<Runtime>::maker_applications(maker_id).map(|app| {
			pallet_trading_otc::pallet::MakerApplicationInfo {
				account: app.owner,
				tron_address: app.tron_address,
				is_active: app.status == pallet_trading_maker::pallet::ApplicationStatus::Active,
				_phantom: core::marker::PhantomData,
			}
		})
	}

	fn is_maker_active(maker_id: u64) -> bool {
		pallet_trading_maker::Pallet::<Runtime>::is_maker_active(maker_id)
	}
}

/// OTC Maker Credit 接口适配器
pub struct OtcMakerCreditAdapter;

impl pallet_trading_otc::pallet::MakerCreditInterface for OtcMakerCreditAdapter {
	fn record_maker_order_completed(maker_id: u64, order_id: u64, response_time_seconds: u32) -> sp_runtime::DispatchResult {
		pallet_trading_credit::Pallet::<Runtime>::record_maker_order_completed(maker_id, order_id, response_time_seconds)
	}

	fn record_maker_order_timeout(maker_id: u64, order_id: u64) -> sp_runtime::DispatchResult {
		pallet_trading_credit::Pallet::<Runtime>::record_maker_order_timeout(maker_id, order_id)
	}

	fn record_maker_dispute_result(maker_id: u64, order_id: u64, maker_win: bool) -> sp_runtime::DispatchResult {
		pallet_trading_credit::Pallet::<Runtime>::record_maker_dispute_result(maker_id, order_id, maker_win)
	}
}

/// OTC Identity Provider - 暂时跳过 KYC 验证
pub struct NullIdentityProvider;

impl pallet_trading_otc::pallet::IdentityVerificationProvider<AccountId> for NullIdentityProvider {
	fn get_highest_judgement_priority(_who: &AccountId) -> Option<u8> {
		// 暂时返回 KnownGood 等级，跳过 KYC 验证
		Some(3)
	}

	fn has_problematic_judgement(_who: &AccountId) -> bool {
		false
	}
}

impl pallet_trading_otc::Config for Runtime {
	type Currency = Balances;
	type Timestamp = TimestampProvider;
	type Escrow = pallet_escrow::Pallet<Runtime>;
	type Credit = pallet_trading_credit::Pallet<Runtime>;
	type MakerCredit = OtcMakerCreditAdapter;
	type Pricing = TradingPricingProvider;
	type MakerPallet = OtcMakerAdapter;
	type CommitteeOrigin = frame_system::EnsureRoot<AccountId>;
	type IdentityProvider = NullIdentityProvider;
	type ChatPermission = pallet_chat_permission::Pallet<Runtime>;
	type OrderTimeout = ConstU64<3600000>; // 1小时（毫秒）
	type EvidenceWindow = ConstU64<86400000>; // 24小时（毫秒）
	type FirstPurchaseUsdValue = ConstU128<10_000_000>; // 10 USD
	type MinFirstPurchaseDustAmount = ConstU128<{ 1 * UNIT }>; // 最小1 DUST
	type MaxFirstPurchaseDustAmount = ConstU128<{ 1000 * UNIT }>; // 最大1000 DUST
	type MaxOrderUsdAmount = ConstU64<200_000_000>; // 200 USD
	type MinOrderUsdAmount = ConstU64<20_000_000>; // 20 USD
	type FirstPurchaseUsdAmount = ConstU64<10_000_000>; // 10 USD
	type AmountValidationTolerance = ConstU16<100>; // 1% 容差
	type MaxFirstPurchaseOrdersPerMaker = ConstU32<5>;
	type WeightInfo = ();
}

// ============================================================================
// Escrow, Referral, IPFS Pallets Configuration
// ============================================================================

// -------------------- Escrow (托管) --------------------

parameter_types! {
	pub const EscrowPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/escro");
}

/// 托管过期策略实现
pub struct DefaultExpiryPolicy;

impl pallet_escrow::ExpiryPolicy<AccountId, BlockNumber> for DefaultExpiryPolicy {
	fn on_expire(_id: u64) -> Result<pallet_escrow::ExpiryAction<AccountId>, sp_runtime::DispatchError> {
		// 默认策略：过期后不执行任何操作
		Ok(pallet_escrow::ExpiryAction::Noop)
	}

	fn now() -> BlockNumber {
		System::block_number()
	}
}

impl pallet_escrow::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EscrowPalletId = EscrowPalletId;
	type AuthorizedOrigin = frame_system::EnsureSigned<AccountId>;
	type AdminOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxExpiringPerBlock = ConstU32<100>;
	type ExpiryPolicy = DefaultExpiryPolicy;
}

// -------------------- Referral (推荐关系) --------------------

/// 基于余额的会员验证 - 账户余额 >= ED 即为有效会员
pub struct BalanceBasedMembership;

impl pallet_referral::MembershipProvider<AccountId> for BalanceBasedMembership {
	fn is_valid_member(who: &AccountId) -> bool {
		// 获取账户可用余额
		let balance = pallet_balances::Pallet::<Runtime>::free_balance(who);
		// 余额 >= ED 即为有效会员
		balance >= EXISTENTIAL_DEPOSIT
	}
}

impl pallet_referral::Config for Runtime {
	type MembershipProvider = BalanceBasedMembership;
	type MaxCodeLen = ConstU32<32>;
	type MaxSearchHops = ConstU32<20>;
}

// -------------------- Stardust IPFS (IPFS存储) --------------------

parameter_types! {
	pub const IpfsSubjectPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/ipfss");
	pub IpfsFeeCollector: AccountId = sp_runtime::AccountId32::new([2u8; 32]);
	pub IpfsPoolAccountId: AccountId = sp_runtime::AccountId32::new([3u8; 32]);
	pub OperatorEscrowAccountId: AccountId = sp_runtime::AccountId32::new([4u8; 32]);
}

/// Creator Provider - 提供内容创建者信息
pub struct NullCreatorProvider;

impl pallet_stardust_ipfs::CreatorProvider<AccountId> for NullCreatorProvider {
	fn creator_of(_deceased_id: u64) -> Option<AccountId> {
		None
	}
}

/// Owner Provider - 提供内容所有者信息
pub struct NullOwnerProvider;

impl pallet_stardust_ipfs::OwnerProvider<AccountId> for NullOwnerProvider {
	fn owner_of(_deceased_id: u64) -> Option<AccountId> {
		None
	}
}

impl pallet_stardust_ipfs::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type Balance = Balance;
	type FeeCollector = IpfsFeeCollector;
	type GovernanceOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxCidHashLen = ConstU32<64>;
	type MaxPeerIdLen = ConstU32<128>;
	type MinOperatorBond = ConstU128<{ 100 * UNIT }>;
	type MinCapacityGiB = ConstU32<10>;
	type WeightInfo = ();
	type SubjectPalletId = IpfsSubjectPalletId;
	type DeceasedDomain = ConstU8<0>;
	type CreatorProvider = NullCreatorProvider;
	type OwnerProvider = NullOwnerProvider;
	type IpfsPoolAccount = IpfsPoolAccountId;
	type OperatorEscrowAccount = OperatorEscrowAccountId;
	type MonthlyPublicFeeQuota = ConstU128<{ 10 * UNIT }>;
	type QuotaResetPeriod = ConstU32<{ 30 * DAYS }>;
	type DefaultBillingPeriod = ConstU32<{ 30 * DAYS }>;
}

// -------------------- Evidence (证据存证) --------------------

parameter_types! {
	pub const EvidenceNsBytes: [u8; 8] = *b"evidence";
}

/// 证据授权适配器 - 暂时允许所有签名用户
pub struct AlwaysAuthorizedEvidence;

impl pallet_evidence::pallet::EvidenceAuthorizer<AccountId> for AlwaysAuthorizedEvidence {
	fn is_authorized(_ns: [u8; 8], _who: &AccountId) -> bool {
		// 暂时允许所有签名用户提交证据
		// 后续可以对接更细粒度的权限系统
		true
	}
}

/// 家庭关系验证适配器 - 暂时总是返回true
pub struct AlwaysFamilyMember;

impl pallet_evidence::pallet::FamilyRelationVerifier<AccountId> for AlwaysFamilyMember {
	fn is_family_member(_user: &AccountId, _deceased_id: u64) -> bool {
		// 暂时总是返回true，后续对接逝者家庭关系模块
		true
	}

	fn is_authorized_for_deceased(_user: &AccountId, _deceased_id: u64) -> bool {
		// 暂时总是返回true，后续对接逝者授权模块
		true
	}
}

impl pallet_evidence::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// Phase 1.5 新参数
	type MaxContentCidLen = ConstU32<64>;
	type MaxSchemeLen = ConstU32<32>;
	// 旧版参数（向后兼容）
	type MaxCidLen = ConstU32<64>;
	type MaxImg = ConstU32<20>;
	type MaxVid = ConstU32<10>;
	type MaxDoc = ConstU32<20>;
	type MaxMemoLen = ConstU32<512>;
	type MaxAuthorizedUsers = ConstU32<50>;
	type MaxKeyLen = ConstU32<512>;
	type EvidenceNsBytes = EvidenceNsBytes;
	type Authorizer = AlwaysAuthorizedEvidence;
	type MaxPerSubjectTarget = ConstU32<1000>;
	type MaxPerSubjectNs = ConstU32<1000>;
	type WindowBlocks = ConstU32<{ 10 * MINUTES }>;
	type MaxPerWindow = ConstU32<100>;
	type EnableGlobalCidDedup = ConstBool<true>;
	type MaxListLen = ConstU32<100>;
	type WeightInfo = pallet_evidence::weights::SubstrateWeight<Runtime>;
	type FamilyVerifier = AlwaysFamilyMember;
	// IPFS 相关
	type IpfsPinner = pallet_stardust_ipfs::Pallet<Runtime>;
	type Balance = Balance;
	type DefaultStoragePrice = ConstU128<{ UNIT / 10 }>;
}

// -------------------- Arbitration (仲裁) --------------------

/// 仲裁域路由器 - 空实现（待对接业务模块）
pub struct NullArbitrationRouter;

impl pallet_arbitration::pallet::ArbitrationRouter<AccountId, Balance> for NullArbitrationRouter {
	fn can_dispute(_domain: [u8; 8], _who: &AccountId, _id: u64) -> bool {
		// 暂时允许所有仲裁请求
		true
	}

	fn apply_decision(_domain: [u8; 8], _id: u64, _decision: pallet_arbitration::pallet::Decision) -> sp_runtime::DispatchResult {
		// 暂时空操作，后续对接业务模块
		Ok(())
	}

	fn get_counterparty(_domain: [u8; 8], _initiator: &AccountId, _id: u64) -> Result<AccountId, sp_runtime::DispatchError> {
		// 暂时返回一个固定的账户作为对方
		Ok(sp_runtime::AccountId32::new([5u8; 32]))
	}

	fn get_order_amount(_domain: [u8; 8], _id: u64) -> Result<Balance, sp_runtime::DispatchError> {
		// 暂时返回固定金额
		Ok(100 * UNIT)
	}
}

impl pallet_arbitration::pallet::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxEvidence = ConstU32<20>;
	type MaxCidLen = ConstU32<64>;
	type Escrow = pallet_escrow::Pallet<Runtime>;
	type WeightInfo = pallet_arbitration::weights::SubstrateWeight<Runtime>;
	type Router = NullArbitrationRouter;
	type DecisionOrigin = frame_system::EnsureRoot<AccountId>;
	type Fungible = Balances;
	type RuntimeHoldReason = RuntimeHoldReason;
	type DepositRatioBps = ConstU16<1500>; // 15% 押金比例
	type ResponseDeadline = ConstU32<{ 7 * DAYS }>; // 7天应诉期限
	type RejectedSlashBps = ConstU16<3000>; // 驳回时罚没30%
	type PartialSlashBps = ConstU16<5000>; // 部分胜诉罚没50%
	type TreasuryAccount = TreasuryAccountId;
}
