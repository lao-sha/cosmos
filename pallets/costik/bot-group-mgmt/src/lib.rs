#![cfg_attr(not(feature = "std"), no_std)]

//! # Pallet Bot Group Management — 链上群组管理规则
//!
//! ## 概述
//!
//! 本模块存储去中心化群组管理的规则和动作日志：
//! - **群规则配置**：反垃圾、入群审批策略、禁言规则等
//! - **动作日志**：所有群管理动作的链上存证
//! - **规则版本管理**：规则变更历史
//!
//! ## 设计原则
//!
//! 规则存储在链上，节点根据链上规则做出决策，
//! Leader 执行后将动作日志上链存证，实现完全可审计。

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

/// 动作类型
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum ActionType {
	/// 踢出用户
	Ban,
	/// 禁言
	Mute,
	/// 解除禁言
	Unmute,
	/// 删除消息
	DeleteMessage,
	/// 置顶消息
	PinMessage,
	/// 审批入群
	ApproveJoin,
	/// 拒绝入群
	DeclineJoin,
	/// 发送消息
	SendMessage,
}

/// 入群审批策略
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum JoinApprovalPolicy {
	/// 自动通过所有人
	AutoApprove,
	/// 需要管理员手动审批
	ManualApproval,
	/// 账户链上余额 >= 阈值时自动通过
	BalanceThreshold,
	/// 需要在链上有身份绑定
	RequirePlatformBinding,
}

impl Default for JoinApprovalPolicy {
	fn default() -> Self {
		Self::AutoApprove
	}
}

/// 群组规则配置
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct GroupRules<T: Config> {
	/// 群管理员（链上账户）
	pub admin: T::AccountId,
	/// 入群审批策略
	pub join_policy: JoinApprovalPolicy,
	/// 反垃圾: 时间窗口内最大消息数（0 = 不限制）
	pub rate_limit_per_minute: u16,
	/// 反垃圾: 禁言时长（区块数，0 = 不自动禁言）
	pub auto_mute_duration: BlockNumberFor<T>,
	/// 是否启用链接过滤
	pub filter_links: bool,
	/// 是否启用 @everyone 限制
	pub restrict_mentions: bool,
	/// 规则版本号
	pub version: u32,
	/// 最后更新区块
	pub updated_at: BlockNumberFor<T>,
}

/// 群管理动作日志
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct ActionLog<T: Config> {
	/// 社区 ID 哈希
	pub community_id_hash: [u8; 32],
	/// 动作类型
	pub action_type: ActionType,
	/// 目标用户 ID 哈希（如果适用）
	pub target_user_hash: [u8; 32],
	/// 执行节点 ID 哈希
	pub executor_node_hash: [u8; 32],
	/// 共识节点数
	pub consensus_count: u8,
	/// 消息序列号
	pub sequence: u64,
	/// 消息哈希
	pub msg_hash: [u8; 32],
	/// 记录区块
	pub logged_at: BlockNumberFor<T>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use sp_std::vec::Vec;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// 每个社区最大日志数（环形缓冲）
		#[pallet::constant]
		type MaxLogsPerCommunity: Get<u32>;
	}

	// ═══════════════════════════════════════════════════════════════
	// 存储
	// ═══════════════════════════════════════════════════════════════

	/// 群规则: community_id_hash → GroupRules
	#[pallet::storage]
	#[pallet::getter(fn group_rules)]
	pub type GroupRulesStore<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		[u8; 32],
		GroupRules<T>,
	>;

	/// 动作日志: (community_id_hash, log_index) → ActionLog
	#[pallet::storage]
	#[pallet::getter(fn action_logs)]
	pub type ActionLogs<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		[u8; 32],
		Blake2_128Concat,
		u64,
		ActionLog<T>,
	>;

	/// 日志计数: community_id_hash → next_log_index
	#[pallet::storage]
	pub type LogCount<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		[u8; 32],
		u64,
		ValueQuery,
	>;

	// ═══════════════════════════════════════════════════════════════
	// 事件
	// ═══════════════════════════════════════════════════════════════

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// 群规则已设置/更新
		GroupRulesUpdated {
			community_id_hash: [u8; 32],
			admin: T::AccountId,
			version: u32,
		},
		/// 动作日志已记录
		ActionLogged {
			community_id_hash: [u8; 32],
			action_type: ActionType,
			log_index: u64,
		},
		/// 群规则已删除
		GroupRulesRemoved {
			community_id_hash: [u8; 32],
		},
	}

	// ═══════════════════════════════════════════════════════════════
	// 错误
	// ═══════════════════════════════════════════════════════════════

	#[pallet::error]
	pub enum Error<T> {
		/// 群规则不存在
		RulesNotFound,
		/// 不是群管理员
		NotAdmin,
		/// 群规则已存在
		RulesAlreadyExist,
	}

	// ═══════════════════════════════════════════════════════════════
	// 调用
	// ═══════════════════════════════════════════════════════════════

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// 设置群组规则（首次设置）
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(30_000_000, 0))]
		pub fn set_group_rules(
			origin: OriginFor<T>,
			community_id_hash: [u8; 32],
			join_policy: JoinApprovalPolicy,
			rate_limit_per_minute: u16,
			auto_mute_duration: BlockNumberFor<T>,
			filter_links: bool,
			restrict_mentions: bool,
		) -> DispatchResult {
			let admin = ensure_signed(origin)?;

			let current_block = <frame_system::Pallet<T>>::block_number();

			let version = if let Some(existing) = GroupRulesStore::<T>::get(&community_id_hash) {
				ensure!(existing.admin == admin, Error::<T>::NotAdmin);
				existing.version + 1
			} else {
				1
			};

			let rules = GroupRules {
				admin: admin.clone(),
				join_policy,
				rate_limit_per_minute,
				auto_mute_duration,
				filter_links,
				restrict_mentions,
				version,
				updated_at: current_block,
			};

			GroupRulesStore::<T>::insert(&community_id_hash, &rules);

			Self::deposit_event(Event::GroupRulesUpdated {
				community_id_hash,
				admin,
				version,
			});

			Ok(())
		}

		/// 删除群组规则
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(20_000_000, 0))]
		pub fn remove_group_rules(
			origin: OriginFor<T>,
			community_id_hash: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let rules = GroupRulesStore::<T>::get(&community_id_hash)
				.ok_or(Error::<T>::RulesNotFound)?;
			ensure!(rules.admin == who, Error::<T>::NotAdmin);

			GroupRulesStore::<T>::remove(&community_id_hash);

			Self::deposit_event(Event::GroupRulesRemoved {
				community_id_hash,
			});

			Ok(())
		}

		/// 记录群管理动作日志（节点调用）
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(40_000_000, 0))]
		pub fn log_action(
			origin: OriginFor<T>,
			community_id_hash: [u8; 32],
			action_type: ActionType,
			target_user_hash: [u8; 32],
			executor_node_hash: [u8; 32],
			consensus_count: u8,
			sequence: u64,
			msg_hash: [u8; 32],
		) -> DispatchResult {
			let _submitter = ensure_signed(origin)?;

			let current_block = <frame_system::Pallet<T>>::block_number();
			let log_index = LogCount::<T>::get(&community_id_hash);

			let log = ActionLog {
				community_id_hash,
				action_type: action_type.clone(),
				target_user_hash,
				executor_node_hash,
				consensus_count,
				sequence,
				msg_hash,
				logged_at: current_block,
			};

			ActionLogs::<T>::insert(&community_id_hash, log_index, &log);
			LogCount::<T>::insert(&community_id_hash, log_index + 1);

			Self::deposit_event(Event::ActionLogged {
				community_id_hash,
				action_type,
				log_index,
			});

			Ok(())
		}
	}

	// ═══════════════════════════════════════════════════════════════
	// 辅助函数
	// ═══════════════════════════════════════════════════════════════

	impl<T: Config> Pallet<T> {
		/// 获取群规则（供其他 pallet/off-chain 使用）
		pub fn get_join_policy(community_id_hash: &[u8; 32]) -> JoinApprovalPolicy {
			GroupRulesStore::<T>::get(community_id_hash)
				.map(|r| r.join_policy)
				.unwrap_or_default()
		}

		/// 获取社区日志数量
		pub fn log_count(community_id_hash: &[u8; 32]) -> u64 {
			LogCount::<T>::get(community_id_hash)
		}
	}
}
