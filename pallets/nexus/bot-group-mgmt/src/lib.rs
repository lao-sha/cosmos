#![cfg_attr(not(feature = "std"), no_std)]

//! # Pallet Bot Group Management — 链上群管理动作日志
//!
//! ## 概述
//!
//! 本模块仅存储群管理动作的链上存证（ActionLog）。
//! 群规则配置已完全移至链下（Agent → Node Gossip 同步）。
//!
//! ## 设计原则
//!
//! - 群规则**不上链**（隐私 + 零 Gas + 即时生效）
//! - 动作日志上链存证，实现完全可审计
//! - 详见 `docs/NEXUS_LAYERED_STORAGE_DESIGN.md`

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
	/// 解封用户
	Unban,
	/// 禁言
	Mute,
	/// 解除禁言
	Unmute,
	/// 删除消息
	DeleteMessage,
	/// 置顶消息
	PinMessage,
	/// 取消置顶
	UnpinMessage,
	/// 审批入群
	ApproveJoin,
	/// 拒绝入群
	DeclineJoin,
	/// 发送消息
	SendMessage,
	/// 设置群权限
	SetPermissions,
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
		/// 动作日志已记录
		ActionLogged {
			community_id_hash: [u8; 32],
			action_type: ActionType,
			log_index: u64,
		},
	}

	// ═══════════════════════════════════════════════════════════════
	// 错误
	// ═══════════════════════════════════════════════════════════════

	#[pallet::error]
	pub enum Error<T> {}

	// ═══════════════════════════════════════════════════════════════
	// 调用
	// ═══════════════════════════════════════════════════════════════

	#[pallet::call]
	impl<T: Config> Pallet<T> {
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
		/// 获取社区日志数量
		pub fn log_count(community_id_hash: &[u8; 32]) -> u64 {
			LogCount::<T>::get(community_id_hash)
		}
	}
}
