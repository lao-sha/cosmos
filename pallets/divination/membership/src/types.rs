//! Type definitions for the membership pallet.

use codec::{Decode, Encode, MaxEncodedLen, DecodeWithMemTracking};
use frame_support::{pallet_prelude::*, BoundedVec};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// Membership tier levels.
#[derive(
    Clone, Copy, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default,
)]
pub enum MemberTier {
    #[default]
    Free = 0,
    Bronze = 1,
    Silver = 2,
    Gold = 3,
    Platinum = 4,
    Diamond = 5,
}

impl MemberTier {
    /// Check if this tier is at least as high as another tier.
    pub fn is_at_least(&self, other: MemberTier) -> bool {
        (*self as u8) >= (other as u8)
    }

    /// Get tier from u8 value.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(MemberTier::Free),
            1 => Some(MemberTier::Bronze),
            2 => Some(MemberTier::Silver),
            3 => Some(MemberTier::Gold),
            4 => Some(MemberTier::Platinum),
            5 => Some(MemberTier::Diamond),
            _ => None,
        }
    }
}

/// Subscription duration options.
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum SubscriptionDuration {
    /// Monthly subscription.
    Monthly,
    /// Yearly subscription (with discount).
    Yearly,
}

/// Member subscription information.
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct MemberInfo<BlockNumber, Balance> {
    /// Current membership tier.
    pub tier: MemberTier,
    /// Block number when subscription expires.
    pub expires_at: BlockNumber,
    /// Block number when subscription started.
    pub subscribed_at: BlockNumber,
    /// Total amount paid (lifetime value).
    pub total_paid: Balance,
    /// Whether to auto-renew (currently disabled).
    pub auto_renew: bool,
}

/// Gender enumeration.
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Gender {
    /// Male.
    Male,
    /// Female.
    Female,
    /// Other or prefer not to say.
    Other,
}

/// Birth date structure.
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct BirthDate {
    /// Year (1900-2100).
    pub year: u16,
    /// Month (1-12).
    pub month: u8,
    /// Day (1-31).
    pub day: u8,
}

impl BirthDate {
    /// Create a new birth date with validation.
    pub fn new(year: u16, month: u8, day: u8) -> Option<Self> {
        if year < 1900 || year > 2100 {
            return None;
        }
        if month < 1 || month > 12 {
            return None;
        }
        if day < 1 || day > 31 {
            return None;
        }
        Some(Self { year, month, day })
    }
}

/// Encrypted sensitive data (real name, detailed address).
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxLen))]
pub struct EncryptedSensitiveData<MaxLen: Get<u32>> {
    /// AES-256-GCM encrypted ciphertext.
    pub ciphertext: BoundedVec<u8, MaxLen>,
    /// GCM nonce (12 bytes).
    pub nonce: [u8; 12],
    /// Encryption version for future upgrades.
    pub version: u8,
}

/// Member profile with partial encryption.
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxDisplayName, MaxEncrypted))]
pub struct MemberProfile<BlockNumber, MaxDisplayName: Get<u32>, MaxEncrypted: Get<u32>> {
    /// Display name (plaintext, public).
    pub display_name: BoundedVec<u8, MaxDisplayName>,

    // ========== Divination data (plaintext) ==========
    /// Gender.
    pub gender: Option<Gender>,
    /// Birth date (Gregorian calendar).
    pub birth_date: Option<BirthDate>,
    /// Birth hour (0-23, None if unknown).
    pub birth_hour: Option<u8>,
    /// Birth location longitude (precision: 0.0001 degree, e.g., 1164532 = 116.4532°E).
    pub longitude: Option<i32>,
    /// Birth location latitude.
    pub latitude: Option<i32>,

    // ========== Privacy data (encrypted) ==========
    /// Encrypted sensitive data (real name, detailed address).
    pub encrypted_sensitive: Option<EncryptedSensitiveData<MaxEncrypted>>,

    // ========== Metadata ==========
    /// Whether user is a service provider.
    pub is_provider: bool,
    /// Whether provider is verified by admin.
    pub provider_verified: bool,
    /// Last update block number.
    pub updated_at: BlockNumber,
}

/// DUST reward balance and statistics.
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub struct RewardBalance<BlockNumber, Balance: Default> {
    /// Total rewards earned (lifetime).
    pub total_earned: Balance,
    /// Rewards earned today.
    pub today_earned: Balance,
    /// Current day number (for daily reset).
    pub today_date: u32,
    /// Last update block number.
    pub last_updated: BlockNumber,
}

/// Reward transaction types.
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RewardTxType {
    /// Daily check-in reward.
    CheckIn,
    /// Divination creation reward (merged with AI).
    Divination,
    /// AI interpretation cashback.
    AiCashback,
    /// Data deletion reward.
    Delete,
    /// Market order cashback.
    MarketCashback,
    /// Review submission reward.
    Review,
    /// Referral reward.
    Referral,
    /// NFT minting reward.
    NftMint,
    /// NFT trading reward.
    NftTrade,
}

/// Reward transaction record.
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct RewardTransaction<BlockNumber, Balance> {
    /// Transaction type.
    pub tx_type: RewardTxType,
    /// Reward amount.
    pub amount: Balance,
    /// Block number when reward was granted.
    pub timestamp: BlockNumber,
    /// Memo (e.g., "bazi_chart", "ai_interpretation").
    pub memo: BoundedVec<u8, ConstU32<32>>,
}

/// Daily check-in record.
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub struct CheckInRecord {
    /// Consecutive check-in days (streak).
    pub streak: u32,
    /// Last check-in day number (since genesis).
    pub last_check_in_day: u32,
    /// Total check-in days (lifetime).
    pub total_days: u32,
    /// This week's check-in bitmap (bit 0-6 = Mon-Sun).
    pub this_week: u8,
}

/// Global membership statistics.
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub struct MembershipStats<Balance: Default> {
    /// Number of members per tier [Free, Bronze, Silver, Gold, Platinum, Diamond].
    pub tier_counts: [u32; 6],
    /// Total revenue from subscriptions.
    pub total_revenue: Balance,
    /// Total rewards issued.
    pub total_rewards_issued: Balance,
}

/// Member profile summary (for external modules).
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MemberProfileSummary<BlockNumber> {
    /// Display name.
    pub display_name: sp_std::vec::Vec<u8>,
    /// Gender.
    pub gender: Option<Gender>,
    /// Birth date.
    pub birth_date: Option<BirthDate>,
    /// Birth hour.
    pub birth_hour: Option<u8>,
    /// Longitude.
    pub longitude: Option<i32>,
    /// Latitude.
    pub latitude: Option<i32>,
    /// Is service provider.
    pub is_provider: bool,
    /// Is provider verified.
    pub provider_verified: bool,
    /// Last update.
    pub updated_at: BlockNumber,
}

/// Redeem item types for future implementation.
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RedeemItem {
    /// Free AI interpretation credit.
    FreeAi,
    /// Storage deposit coupon.
    DepositCoupon,
    /// Membership upgrade (1 month).
    MembershipUpgrade,
}
