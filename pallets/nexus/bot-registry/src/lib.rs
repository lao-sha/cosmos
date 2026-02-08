#![cfg_attr(not(feature = "std"), no_std)]

//! # Pallet Bot Registry — Bot 注册与多平台身份绑定
//!
//! ## 概述
//!
//! 本模块管理方案 D 架构中的 Bot 注册与身份体系：
//! - **Bot 注册**：群主注册 Bot，存储 Ed25519 公钥（Token 永不上链）
//! - **公钥管理**：更换 Agent 公钥（密钥轮换）
//! - **多平台支持**：Platform enum 覆盖 TG/Discord/Slack/Matrix/Farcaster
//! - **社区绑定**：同一链上社区绑定多个平台群组
//! - **用户身份绑定**：同一链上账户绑定多个平台 ID
//!
//! ## 安全原则
//!
//! Bot Token **永远不上链**，只保存在群主 Local Agent 内存中。
//! 链上只存储 `bot_id_hash = SHA256(bot_token)` 用于标识。

extern crate alloc;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;

/// 社交平台枚举
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum Platform {
	/// Telegram
	Telegram,
	/// Discord
	Discord,
	/// Slack
	Slack,
	/// Matrix (Element)
	Matrix,
	/// Farcaster
	Farcaster,
}

impl Default for Platform {
	fn default() -> Self {
		Self::Telegram
	}
}

/// Bot 状态
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum BotStatus {
	/// 活跃
	Active,
	/// 暂停（余额不足或手动暂停）
	Suspended,
	/// 已停用
	Deactivated,
}

impl Default for BotStatus {
	fn default() -> Self {
		Self::Active
	}
}

/// Bot 注册信息
///
/// 哈希约定（链下 Agent/DApp 计算）:
/// - `bot_id_hash = SHA256(bot_token || hash_salt)`
/// - `community_id_hash = SHA256(platform_name || ":" || community_id || hash_salt)`
/// - hash_salt 为 16 字节随机值，防止彩虹表 + 跨平台碰撞
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct BotRegistration<T: Config> {
	/// Bot 所有者（群主）
	pub owner: T::AccountId,
	/// 平台类型
	pub platform: Platform,
	/// bot_id_hash = SHA256(bot_token || hash_salt)
	pub bot_id_hash: [u8; 32],
	/// community_id_hash = SHA256(platform || ":" || community_id || hash_salt)
	pub community_id_hash: [u8; 32],
	/// 哈希盐值（16 字节随机数，防彩虹表 + 跨平台碰撞）
	pub hash_salt: [u8; 16],
	/// 群主 Agent 的 Ed25519 公钥（用于验签）
	pub owner_public_key: [u8; 32],
	/// Bot 状态
	pub status: BotStatus,
	/// 注册区块
	pub registered_at: BlockNumberFor<T>,
	/// 最后更新区块
	pub updated_at: BlockNumberFor<T>,
}

/// 社区平台绑定信息
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct CommunityBinding<T: Config> {
	/// 绑定者（社区管理员）
	pub binder: T::AccountId,
	/// 平台侧社区 ID 哈希（如 TG chat_id 的哈希）
	pub platform_community_id_hash: [u8; 32],
	/// 绑定的 Bot（如果有）
	pub bot_id_hash: Option<[u8; 32]>,
	/// 绑定区块
	pub bound_at: BlockNumberFor<T>,
}

/// 用户平台身份绑定
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct UserPlatformBinding {
	/// 平台侧用户 ID 哈希
	pub platform_user_id_hash: [u8; 32],
	/// 是否已验证
	pub verified: bool,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use sp_std::vec::Vec;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// 每个 owner 最多注册的 Bot 数量
		#[pallet::constant]
		type MaxBotsPerOwner: Get<u32>;

		/// 每个社区最大绑定平台数
		#[pallet::constant]
		type MaxPlatformsPerCommunity: Get<u32>;

		/// 每个用户最大平台绑定数
		#[pallet::constant]
		type MaxPlatformBindingsPerUser: Get<u32>;
	}

	// ═══════════════════════════════════════════════════════════════
	// 存储
	// ═══════════════════════════════════════════════════════════════

	/// Bot 注册表: bot_id_hash → BotRegistration
	#[pallet::storage]
	#[pallet::getter(fn bots)]
	pub type Bots<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		[u8; 32],
		BotRegistration<T>,
	>;

	/// 群主拥有的 Bot 列表: owner → Vec<bot_id_hash>
	#[pallet::storage]
	#[pallet::getter(fn owner_bots)]
	pub type OwnerBots<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<[u8; 32], ConstU32<20>>,
		ValueQuery,
	>;

	/// 社区平台绑定: (community_id_hash, Platform) → CommunityBinding
	#[pallet::storage]
	#[pallet::getter(fn community_platforms)]
	pub type CommunityPlatforms<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		[u8; 32],
		Blake2_128Concat,
		Platform,
		CommunityBinding<T>,
	>;

	/// 用户多平台身份绑定: (AccountId, Platform) → UserPlatformBinding
	#[pallet::storage]
	#[pallet::getter(fn user_platform_bindings)]
	pub type UserPlatformBindings<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		Platform,
		UserPlatformBinding,
	>;

	// ═══════════════════════════════════════════════════════════════
	// 事件
	// ═══════════════════════════════════════════════════════════════

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Bot 已注册
		BotRegistered {
			owner: T::AccountId,
			bot_id_hash: [u8; 32],
			platform: Platform,
		},
		/// Bot 公钥已更新
		BotPublicKeyUpdated {
			bot_id_hash: [u8; 32],
			old_key: [u8; 32],
			new_key: [u8; 32],
		},
		/// Bot 已停用
		BotDeactivated {
			bot_id_hash: [u8; 32],
			owner: T::AccountId,
		},
		/// Bot 已暂停
		BotSuspended {
			bot_id_hash: [u8; 32],
		},
		/// Bot 已恢复
		BotReactivated {
			bot_id_hash: [u8; 32],
		},
		/// 社区平台绑定
		CommunityPlatformBound {
			community_id_hash: [u8; 32],
			platform: Platform,
			binder: T::AccountId,
		},
		/// 社区平台解绑
		CommunityPlatformUnbound {
			community_id_hash: [u8; 32],
			platform: Platform,
		},
		/// 用户平台身份绑定
		UserPlatformBound {
			who: T::AccountId,
			platform: Platform,
		},
		/// 用户平台身份解绑
		UserPlatformUnbound {
			who: T::AccountId,
			platform: Platform,
		},
	}

	// ═══════════════════════════════════════════════════════════════
	// 错误
	// ═══════════════════════════════════════════════════════════════

	#[pallet::error]
	pub enum Error<T> {
		/// Bot 已存在（bot_id_hash 重复）
		BotAlreadyExists,
		/// Bot 不存在
		BotNotFound,
		/// 调用者不是 Bot 所有者
		NotBotOwner,
		/// Bot 已停用，无法操作
		BotAlreadyDeactivated,
		/// Bot 未暂停
		BotNotSuspended,
		/// 群主的 Bot 数量已达上限
		TooManyBots,
		/// 社区平台绑定已存在
		CommunityBindingAlreadyExists,
		/// 社区平台绑定不存在
		CommunityBindingNotFound,
		/// 用户平台绑定已存在
		UserBindingAlreadyExists,
		/// 用户平台绑定不存在
		UserBindingNotFound,
		/// 新公钥与旧公钥相同
		PublicKeyUnchanged,
		/// Owner Bot 列表溢出
		OwnerBotOverflow,
	}

	// ═══════════════════════════════════════════════════════════════
	// 调用
	// ═══════════════════════════════════════════════════════════════

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// 注册 Bot
		///
		/// - `bot_id_hash`: SHA256(bot_token || hash_salt)
		/// - `community_id_hash`: SHA256(platform || ":" || community_id || hash_salt)
		/// - `hash_salt`: 16 字节随机盐值（Agent 生成）
		/// - `platform`: 平台类型
		/// - `owner_public_key`: Agent 的 Ed25519 公钥
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(40_000_000, 0))]
		pub fn register_bot(
			origin: OriginFor<T>,
			bot_id_hash: [u8; 32],
			community_id_hash: [u8; 32],
			hash_salt: [u8; 16],
			platform: Platform,
			owner_public_key: [u8; 32],
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			ensure!(!Bots::<T>::contains_key(&bot_id_hash), Error::<T>::BotAlreadyExists);

			let current_block = <frame_system::Pallet<T>>::block_number();

			let registration = BotRegistration {
				owner: owner.clone(),
				platform,
				bot_id_hash,
				community_id_hash,
				hash_salt,
				owner_public_key,
				status: BotStatus::Active,
				registered_at: current_block,
				updated_at: current_block,
			};

			Bots::<T>::insert(&bot_id_hash, &registration);

			// 更新 owner 的 bot 列表
			OwnerBots::<T>::try_mutate(&owner, |list| {
				list.try_push(bot_id_hash)
					.map_err(|_| Error::<T>::OwnerBotOverflow)
			})?;

			Self::deposit_event(Event::BotRegistered {
				owner,
				bot_id_hash,
				platform,
			});

			Ok(())
		}

		/// 更换 Bot 的 Agent 公钥（密钥轮换）
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(25_000_000, 0))]
		pub fn update_bot_public_key(
			origin: OriginFor<T>,
			bot_id_hash: [u8; 32],
			new_public_key: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut bot = Bots::<T>::get(&bot_id_hash).ok_or(Error::<T>::BotNotFound)?;
			ensure!(bot.owner == who, Error::<T>::NotBotOwner);
			ensure!(bot.status != BotStatus::Deactivated, Error::<T>::BotAlreadyDeactivated);
			ensure!(bot.owner_public_key != new_public_key, Error::<T>::PublicKeyUnchanged);

			let old_key = bot.owner_public_key;
			bot.owner_public_key = new_public_key;
			bot.updated_at = <frame_system::Pallet<T>>::block_number();

			Bots::<T>::insert(&bot_id_hash, &bot);

			Self::deposit_event(Event::BotPublicKeyUpdated {
				bot_id_hash,
				old_key,
				new_key: new_public_key,
			});

			Ok(())
		}

		/// 停用 Bot（不可逆）
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(25_000_000, 0))]
		pub fn deactivate_bot(
			origin: OriginFor<T>,
			bot_id_hash: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut bot = Bots::<T>::get(&bot_id_hash).ok_or(Error::<T>::BotNotFound)?;
			ensure!(bot.owner == who, Error::<T>::NotBotOwner);
			ensure!(bot.status != BotStatus::Deactivated, Error::<T>::BotAlreadyDeactivated);

			bot.status = BotStatus::Deactivated;
			bot.updated_at = <frame_system::Pallet<T>>::block_number();
			Bots::<T>::insert(&bot_id_hash, &bot);

			// 从 owner 列表移除
			OwnerBots::<T>::mutate(&who, |list| {
				list.retain(|h| h != &bot_id_hash);
			});

			Self::deposit_event(Event::BotDeactivated {
				bot_id_hash,
				owner: who,
			});

			Ok(())
		}

		/// 暂停 Bot（可恢复，管理/计费用）
		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(20_000_000, 0))]
		pub fn suspend_bot(
			origin: OriginFor<T>,
			bot_id_hash: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut bot = Bots::<T>::get(&bot_id_hash).ok_or(Error::<T>::BotNotFound)?;
			ensure!(bot.owner == who, Error::<T>::NotBotOwner);
			ensure!(bot.status == BotStatus::Active, Error::<T>::BotAlreadyDeactivated);

			bot.status = BotStatus::Suspended;
			bot.updated_at = <frame_system::Pallet<T>>::block_number();
			Bots::<T>::insert(&bot_id_hash, &bot);

			Self::deposit_event(Event::BotSuspended { bot_id_hash });

			Ok(())
		}

		/// 恢复已暂停的 Bot
		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(20_000_000, 0))]
		pub fn reactivate_bot(
			origin: OriginFor<T>,
			bot_id_hash: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut bot = Bots::<T>::get(&bot_id_hash).ok_or(Error::<T>::BotNotFound)?;
			ensure!(bot.owner == who, Error::<T>::NotBotOwner);
			ensure!(bot.status == BotStatus::Suspended, Error::<T>::BotNotSuspended);

			bot.status = BotStatus::Active;
			bot.updated_at = <frame_system::Pallet<T>>::block_number();
			Bots::<T>::insert(&bot_id_hash, &bot);

			Self::deposit_event(Event::BotReactivated { bot_id_hash });

			Ok(())
		}

		/// 绑定社区到平台
		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(30_000_000, 0))]
		pub fn bind_community_platform(
			origin: OriginFor<T>,
			community_id_hash: [u8; 32],
			platform: Platform,
			platform_community_id_hash: [u8; 32],
			bot_id_hash: Option<[u8; 32]>,
		) -> DispatchResult {
			let binder = ensure_signed(origin)?;

			ensure!(
				!CommunityPlatforms::<T>::contains_key(&community_id_hash, &platform),
				Error::<T>::CommunityBindingAlreadyExists
			);

			let current_block = <frame_system::Pallet<T>>::block_number();

			let binding = CommunityBinding {
				binder: binder.clone(),
				platform_community_id_hash,
				bot_id_hash,
				bound_at: current_block,
			};

			CommunityPlatforms::<T>::insert(&community_id_hash, &platform, &binding);

			Self::deposit_event(Event::CommunityPlatformBound {
				community_id_hash,
				platform,
				binder,
			});

			Ok(())
		}

		/// 解除社区平台绑定
		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(25_000_000, 0))]
		pub fn unbind_community_platform(
			origin: OriginFor<T>,
			community_id_hash: [u8; 32],
			platform: Platform,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let binding = CommunityPlatforms::<T>::get(&community_id_hash, &platform)
				.ok_or(Error::<T>::CommunityBindingNotFound)?;

			ensure!(binding.binder == who, Error::<T>::NotBotOwner);

			CommunityPlatforms::<T>::remove(&community_id_hash, &platform);

			Self::deposit_event(Event::CommunityPlatformUnbound {
				community_id_hash,
				platform,
			});

			Ok(())
		}

		/// 绑定用户平台身份
		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(25_000_000, 0))]
		pub fn bind_user_platform(
			origin: OriginFor<T>,
			platform: Platform,
			platform_user_id_hash: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				!UserPlatformBindings::<T>::contains_key(&who, &platform),
				Error::<T>::UserBindingAlreadyExists
			);

			let binding = UserPlatformBinding {
				platform_user_id_hash,
				verified: false,
			};

			UserPlatformBindings::<T>::insert(&who, &platform, &binding);

			Self::deposit_event(Event::UserPlatformBound {
				who,
				platform,
			});

			Ok(())
		}

		/// 解除用户平台身份绑定
		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_parts(20_000_000, 0))]
		pub fn unbind_user_platform(
			origin: OriginFor<T>,
			platform: Platform,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				UserPlatformBindings::<T>::contains_key(&who, &platform),
				Error::<T>::UserBindingNotFound
			);

			UserPlatformBindings::<T>::remove(&who, &platform);

			Self::deposit_event(Event::UserPlatformUnbound {
				who,
				platform,
			});

			Ok(())
		}
	}

	// ═══════════════════════════════════════════════════════════════
	// 辅助函数
	// ═══════════════════════════════════════════════════════════════

	impl<T: Config> Pallet<T> {
		/// 获取 Bot 的公钥（供其他 pallet 使用）
		pub fn get_bot_public_key(bot_id_hash: &[u8; 32]) -> Option<[u8; 32]> {
			Bots::<T>::get(bot_id_hash).map(|b| b.owner_public_key)
		}

		/// 检查 Bot 是否处于活跃状态
		pub fn is_bot_active(bot_id_hash: &[u8; 32]) -> bool {
			Bots::<T>::get(bot_id_hash)
				.map(|b| b.status == BotStatus::Active)
				.unwrap_or(false)
		}

		/// 获取 Bot 的 owner
		pub fn get_bot_owner(bot_id_hash: &[u8; 32]) -> Option<T::AccountId> {
			Bots::<T>::get(bot_id_hash).map(|b| b.owner)
		}

		/// 获取 Bot 的哈希盐值
		pub fn get_bot_salt(bot_id_hash: &[u8; 32]) -> Option<[u8; 16]> {
			Bots::<T>::get(bot_id_hash).map(|b| b.hash_salt)
		}
	}
}
