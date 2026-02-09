#![cfg_attr(not(feature = "std"), no_std)]

//! # Pallet Bot Consensus — 去中心化多节点验证共识 + 节点奖励
//!
//! ## 概述
//!
//! 本模块实现方案 D 的核心链上逻辑：
//! - **节点注册与质押**：项目节点注册、质押保证金、状态管理
//! - **消息确认存证**：批量提交消息确认，链上不可篡改
//! - **Equivocation 检测**：群主双发攻击检测与 Slash
//! - **信誉系统**：节点信誉评分与自动管理
//! - **Leader 统计**：Leader 执行成功率追踪
//! - **订阅管理**：群主订阅 Bot 服务，预存 NXS 按 Era 扣费
//! - **节点奖励**：订阅费分成 + 通胀保底混合模型，按权重分配
//!
//! ## 架构
//!
//! ```text
//! Agent 签名多播 → K 个节点验签 → Gossip 共识 → 批量上链存证
//!                                                    ↓
//!                                           pallet-bot-consensus
//!                                       (节点注册 + 消息确认 + Slash)
//! ```

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
use sp_runtime::{traits::Saturating, Perbill};

/// 节点状态
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum NodeStatus {
	/// 活跃（参与共识）
	Active,
	/// 试用期（新节点，信誉未建立）
	Probation,
	/// 因违规暂停
	Suspended,
	/// 正在退出（冷却期中）
	Exiting,
}

impl Default for NodeStatus {
	fn default() -> Self {
		Self::Probation
	}
}

/// 项目节点信息
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct ProjectNode<T: Config> {
	/// 节点运营者账户
	pub operator: T::AccountId,
	/// 节点的 Ed25519 公钥（用于 gossip 签名验证）
	pub node_public_key: [u8; 32],
	/// 节点 API 端点哈希 (URL 明文在链下节点发现服务中)
	pub endpoint_hash: [u8; 32],
	/// 质押金额（Slash 保证金）
	pub stake: BalanceOf<T>,
	/// 节点状态
	pub status: NodeStatus,
	/// 信誉分 (0-10000)
	pub reputation: u16,
	/// 历史统计：已确认消息数
	pub messages_confirmed: u64,
	/// 历史统计：错过的消息数
	pub messages_missed: u64,
	/// 被举报 Equivocation 次数
	pub equivocations_reported: u32,
	/// 注册区块
	pub registered_at: BlockNumberFor<T>,
	/// 最后活跃区块
	pub last_active: BlockNumberFor<T>,
}

/// 消息确认记录
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct MessageConfirmation<T: Config> {
	/// Bot 所有者
	pub owner: T::AccountId,
	/// 消息序列号
	pub sequence: u64,
	/// 消息哈希
	pub msg_hash: [u8; 32],
	/// 确认节点列表（node_id 列表）
	pub confirmed_by: BoundedVec<NodeId, ConstU32<20>>,
	/// 确认区块
	pub confirmed_at: BlockNumberFor<T>,
}

/// Equivocation 证据记录
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct EquivocationRecord<T: Config> {
	/// 作弊的 owner
	pub owner: T::AccountId,
	/// 消息序列号
	pub sequence: u64,
	/// 第一个消息哈希
	pub msg_hash_a: [u8; 32],
	/// 第一个签名
	pub signature_a: [u8; 64],
	/// 第二个消息哈希
	pub msg_hash_b: [u8; 32],
	/// 第二个签名
	pub signature_b: [u8; 64],
	/// 举报者
	pub reporter: T::AccountId,
	/// 举报区块
	pub reported_at: BlockNumberFor<T>,
	/// 是否已处理
	pub resolved: bool,
}

/// Leader 执行统计
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen, Default)]
pub struct LeaderStats {
	/// 总 Leader 次数
	pub total_leads: u64,
	/// 成功执行次数
	pub successful: u64,
	/// 超时次数
	pub timeout: u64,
	/// 失败次数
	pub failed: u64,
	/// 连续超时次数（用于惩罚判定）
	pub consecutive_timeouts: u32,
}

/// 订阅层级
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum SubscriptionTier {
	/// 基础版: 单群、基础规则
	Basic,
	/// 专业版: 多群、高级规则、优先响应
	Pro,
	/// 企业版: 无限群、自定义规则、SLA 保障
	Enterprise,
}

/// 订阅状态
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum SubscriptionStatus {
	/// 正常
	Active,
	/// 欠费宽限 (1 Era)
	PastDue,
	/// 欠费暂停
	Suspended,
	/// 主动取消
	Cancelled,
}

/// 订阅信息
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Subscription<T: Config> {
	/// 订阅所有者
	pub owner: T::AccountId,
	/// Bot ID 哈希
	pub bot_id_hash: [u8; 32],
	/// 订阅层级
	pub tier: SubscriptionTier,
	/// 每 Era 费用
	pub fee_per_era: BalanceOf<T>,
	/// 开始区块
	pub started_at: BlockNumberFor<T>,
	/// 已付费到的 Era
	pub paid_until_era: u32,
	/// 订阅状态
	pub status: SubscriptionStatus,
}

/// Era 奖励快照
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen, Default)]
pub struct EraRewardInfo<Balance: Default> {
	/// 订阅费收入
	pub subscription_income: Balance,
	/// 通胀铸币
	pub inflation_mint: Balance,
	/// 实际分配给节点
	pub total_distributed: Balance,
	/// 国库份额
	pub treasury_share: Balance,
	/// 参与分配的节点数
	pub node_count: u32,
}

/// 节点ID类型 (32字节)
pub type NodeId = BoundedVec<u8, ConstU32<32>>;

/// 余额类型别名
pub type BalanceOf<T> = <<T as Config>::Currency as frame_support::traits::Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::traits::{Currency, ReservableCurrency};
	use frame_system::pallet_prelude::BlockNumberFor;
	use sp_std::vec::Vec;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {

		/// 货币类型（用于质押和 Slash）
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// 最低质押金额
		#[pallet::constant]
		type MinStake: Get<BalanceOf<Self>>;

		/// 节点退出冷却期（区块数）
		#[pallet::constant]
		type ExitCooldownPeriod: Get<BlockNumberFor<Self>>;

		/// 最大活跃节点数
		#[pallet::constant]
		type MaxNodes: Get<u32>;

		/// Slash 比例分子（如 10 = 10%）
		#[pallet::constant]
		type SlashPercentage: Get<u32>;

		/// 举报者奖励比例分子（占 Slash 金额，如 50 = 50%）
		#[pallet::constant]
		type ReporterRewardPercentage: Get<u32>;

		/// 信誉低于此值自动 Suspend
		#[pallet::constant]
		type SuspendThreshold: Get<u16>;

		/// 信誉低于此值强制退出
		#[pallet::constant]
		type ForceExitThreshold: Get<u16>;

		/// Era 长度（区块数）
		#[pallet::constant]
		type EraLength: Get<BlockNumberFor<Self>>;

		/// 每 Era 基础通胀铸币量
		#[pallet::constant]
		type InflationPerEra: Get<BalanceOf<Self>>;

		/// 最低在线率（获得奖励的门槛）
		#[pallet::constant]
		type MinUptimeForReward: Get<Perbill>;

		/// 节点单 Era 奖励上限（占总池比例）
		#[pallet::constant]
		type MaxRewardShare: Get<Perbill>;

		/// Basic 层级每 Era 费用
		#[pallet::constant]
		type BasicFeePerEra: Get<BalanceOf<Self>>;

		/// Pro 层级每 Era 费用
		#[pallet::constant]
		type ProFeePerEra: Get<BalanceOf<Self>>;

		/// Enterprise 层级每 Era 费用
		#[pallet::constant]
		type EnterpriseFeePerEra: Get<BalanceOf<Self>>;

		/// Bot 注册表（查 Bot owner 验证订阅权限）
		type BotRegistry: BotRegistryProvider<Self::AccountId>;
	}

	// ═══════════════════════════════════════════════════════════════
	// 存储
	// ═══════════════════════════════════════════════════════════════

	/// 节点信息: node_id → ProjectNode
	#[pallet::storage]
	#[pallet::getter(fn nodes)]
	pub type Nodes<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NodeId,
		ProjectNode<T>,
	>;

	/// 活跃节点列表（确定性选择算法用）
	#[pallet::storage]
	#[pallet::getter(fn active_node_list)]
	pub type ActiveNodeList<T: Config> = StorageValue<
		_,
		BoundedVec<NodeId, ConstU32<100>>,
		ValueQuery,
	>;

	/// 运营者 → 节点ID 映射（一个运营者可以注册多个节点）
	#[pallet::storage]
	#[pallet::getter(fn operator_nodes)]
	pub type OperatorNodes<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<NodeId, ConstU32<10>>,
		ValueQuery,
	>;

	/// 消息确认记录: msg_id → MessageConfirmation
	#[pallet::storage]
	#[pallet::getter(fn message_confirmations)]
	pub type MessageConfirmations<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		[u8; 32],
		MessageConfirmation<T>,
	>;

	/// Equivocation 证据记录: (owner, sequence) → EquivocationRecord
	#[pallet::storage]
	#[pallet::getter(fn equivocation_records)]
	pub type EquivocationRecords<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		u64,
		EquivocationRecord<T>,
	>;

	/// Leader 执行统计: node_id → LeaderStats
	#[pallet::storage]
	#[pallet::getter(fn leader_stats)]
	pub type LeaderStatsStore<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NodeId,
		LeaderStats,
		ValueQuery,
	>;

	/// 节点退出请求时间: node_id → 请求退出的区块号
	#[pallet::storage]
	pub type ExitRequests<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NodeId,
		BlockNumberFor<T>,
	>;

	/// 订阅信息: bot_id_hash → Subscription
	#[pallet::storage]
	#[pallet::getter(fn subscriptions)]
	pub type Subscriptions<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		[u8; 32],
		Subscription<T>,
	>;

	/// 订阅 Escrow 余额: bot_id_hash → Balance
	#[pallet::storage]
	#[pallet::getter(fn subscription_escrow)]
	pub type SubscriptionEscrow<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		[u8; 32],
		BalanceOf<T>,
		ValueQuery,
	>;

	/// 当前 Era
	#[pallet::storage]
	#[pallet::getter(fn current_era)]
	pub type CurrentEra<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// 上次 Era 切换的区块号
	#[pallet::storage]
	pub type EraStartBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	/// Era 奖励信息: era → EraRewardInfo
	#[pallet::storage]
	#[pallet::getter(fn era_rewards)]
	pub type EraRewards<T: Config> = StorageMap<
		_,
		Twox64Concat,
		u32,
		EraRewardInfo<BalanceOf<T>>,
	>;

	/// 节点待领取奖励: node_id → Balance
	#[pallet::storage]
	#[pallet::getter(fn node_pending_rewards)]
	pub type NodePendingRewards<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NodeId,
		BalanceOf<T>,
		ValueQuery,
	>;

	/// 节点历史总收益: node_id → Balance
	#[pallet::storage]
	#[pallet::getter(fn node_total_earned)]
	pub type NodeTotalEarned<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NodeId,
		BalanceOf<T>,
		ValueQuery,
	>;

	// ═══════════════════════════════════════════════════════════════
	// 事件
	// ═══════════════════════════════════════════════════════════════

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// 新节点注册
		NodeRegistered {
			node_id: NodeId,
			operator: T::AccountId,
			stake: BalanceOf<T>,
		},
		/// 节点开始退出（进入冷却期）
		NodeExitRequested {
			node_id: NodeId,
			operator: T::AccountId,
			cooldown_until: BlockNumberFor<T>,
		},
		/// 节点退出完成，质押释放
		NodeExited {
			node_id: NodeId,
			operator: T::AccountId,
			stake_returned: BalanceOf<T>,
		},
		/// 节点被暂停
		NodeSuspended {
			node_id: NodeId,
			reason: SuspendReason,
		},
		/// 节点从试用转为活跃
		NodeActivated {
			node_id: NodeId,
		},
		/// 消息确认批量提交
		ConfirmationsSubmitted {
			submitter: T::AccountId,
			count: u32,
		},
		/// Equivocation 举报
		EquivocationReported {
			owner: T::AccountId,
			sequence: u64,
			reporter: T::AccountId,
		},
		/// Equivocation Slash 执行
		EquivocationSlashed {
			owner: T::AccountId,
			slash_amount: BalanceOf<T>,
			reporter_reward: BalanceOf<T>,
		},
		/// 信誉分变化
		ReputationUpdated {
			node_id: NodeId,
			old_reputation: u16,
			new_reputation: u16,
		},
		/// Leader 超时举报
		LeaderTimeoutReported {
			node_id: NodeId,
			consecutive_timeouts: u32,
		},
		/// 节点离线举报
		NodeOfflineReported {
			node_id: NodeId,
			reporter: T::AccountId,
			evidence_count: u32,
		},
		/// 新订阅创建
		Subscribed {
			bot_id_hash: [u8; 32],
			owner: T::AccountId,
			tier: SubscriptionTier,
			deposit: BalanceOf<T>,
		},
		/// 订阅充值
		SubscriptionDeposited {
			bot_id_hash: [u8; 32],
			amount: BalanceOf<T>,
			new_escrow: BalanceOf<T>,
		},
		/// 订阅取消
		SubscriptionCancelled {
			bot_id_hash: [u8; 32],
			refunded: BalanceOf<T>,
		},
		/// 层级变更
		TierChanged {
			bot_id_hash: [u8; 32],
			old_tier: SubscriptionTier,
			new_tier: SubscriptionTier,
		},
		/// Era 完成
		EraCompleted {
			era: u32,
			subscription_income: BalanceOf<T>,
			inflation_mint: BalanceOf<T>,
			node_count: u32,
		},
		/// 奖励领取
		RewardsClaimed {
			node_id: NodeId,
			operator: T::AccountId,
			amount: BalanceOf<T>,
		},
		/// 订阅暂停 (欠费)
		SubscriptionSuspended {
			bot_id_hash: [u8; 32],
		},
	}

	/// 暂停原因
	#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
	pub enum SuspendReason {
		/// 信誉过低
		LowReputation,
		/// Equivocation
		Equivocation,
		/// 长期离线
		Offline,
	}

	// ═══════════════════════════════════════════════════════════════
	// 错误
	// ═══════════════════════════════════════════════════════════════

	#[pallet::error]
	pub enum Error<T> {
		/// 质押金额不足
		InsufficientStake,
		/// 节点 ID 已存在
		NodeAlreadyExists,
		/// 节点不存在
		NodeNotFound,
		/// 调用者不是节点运营者
		NotNodeOperator,
		/// 节点不在活跃状态
		NodeNotActive,
		/// 节点已在退出中
		NodeAlreadyExiting,
		/// 冷却期未结束
		CooldownNotExpired,
		/// 活跃节点数已达上限
		TooManyNodes,
		/// 运营者节点数已达上限
		TooManyOperatorNodes,
		/// 无效签名
		InvalidSignature,
		/// Equivocation 证据无效（两个哈希相同）
		InvalidEquivocationEvidence,
		/// 该 Equivocation 已被举报
		EquivocationAlreadyReported,
		/// 确认列表为空
		EmptyConfirmations,
		/// 消息确认已存在
		ConfirmationAlreadyExists,
		/// 节点不在退出状态
		NodeNotExiting,
		/// 节点运营者列表溢出
		OperatorNodeOverflow,
		/// 活跃列表溢出
		ActiveListOverflow,
		/// Bot 未注册
		BotNotRegistered,
		/// 调用者不是 Bot 所有者
		NotBotOwner,
		/// 充值金额不足（至少 1 Era 费用）
		InsufficientDeposit,
		/// 订阅已存在
		SubscriptionAlreadyExists,
		/// 订阅不存在
		SubscriptionNotFound,
		/// 无待领取奖励
		NoPendingRewards,
		/// 层级未变化
		SameTier,
		/// 订阅已取消
		SubscriptionAlreadyCancelled,
	}

	// ═══════════════════════════════════════════════════════════════
	// 调用
	// ═══════════════════════════════════════════════════════════════

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// 注册项目节点（需质押）
		///
		/// - `node_id`: 唯一节点标识
		/// - `node_public_key`: Ed25519 公钥（gossip 签名验证）
		/// - `endpoint_hash`: 节点 API 端点哈希
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(50_000_000, 0))]
		pub fn register_node(
			origin: OriginFor<T>,
			node_id: NodeId,
			node_public_key: [u8; 32],
			endpoint_hash: [u8; 32],
		) -> DispatchResult {
			let operator = ensure_signed(origin)?;

			// 节点 ID 不能重复
			ensure!(!Nodes::<T>::contains_key(&node_id), Error::<T>::NodeAlreadyExists);

			// 活跃节点数不超限
			let active_list = ActiveNodeList::<T>::get();
			ensure!(
				(active_list.len() as u32) < T::MaxNodes::get(),
				Error::<T>::TooManyNodes
			);

			// 运营者节点数不超限
			let mut op_nodes = OperatorNodes::<T>::get(&operator);
			ensure!(op_nodes.len() < 10, Error::<T>::TooManyOperatorNodes);

			// 质押
			let stake = T::MinStake::get();
			T::Currency::reserve(&operator, stake)
				.map_err(|_| Error::<T>::InsufficientStake)?;

			let current_block = <frame_system::Pallet<T>>::block_number();

			// 创建节点
			let node = ProjectNode {
				operator: operator.clone(),
				node_public_key,
				endpoint_hash,
				stake,
				status: NodeStatus::Probation,
				reputation: 5000,
				messages_confirmed: 0,
				messages_missed: 0,
				equivocations_reported: 0,
				registered_at: current_block,
				last_active: current_block,
			};

			// 存储
			Nodes::<T>::insert(&node_id, &node);

			// 更新运营者节点列表
			op_nodes.try_push(node_id.clone())
				.map_err(|_| Error::<T>::OperatorNodeOverflow)?;
			OperatorNodes::<T>::insert(&operator, op_nodes);

			// 加入活跃列表（试用期节点也参与选择，但权重低）
			ActiveNodeList::<T>::try_mutate(|list| {
				list.try_push(node_id.clone())
					.map_err(|_| Error::<T>::ActiveListOverflow)
			})?;

			Self::deposit_event(Event::NodeRegistered {
				node_id,
				operator,
				stake,
			});

			Ok(())
		}

		/// 节点请求退出（进入冷却期）
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(30_000_000, 0))]
		pub fn request_exit(
			origin: OriginFor<T>,
			node_id: NodeId,
		) -> DispatchResult {
			let operator = ensure_signed(origin)?;

			let node = Nodes::<T>::get(&node_id).ok_or(Error::<T>::NodeNotFound)?;
			ensure!(node.operator == operator, Error::<T>::NotNodeOperator);
			ensure!(node.status != NodeStatus::Exiting, Error::<T>::NodeAlreadyExiting);

			let current_block = <frame_system::Pallet<T>>::block_number();
			let cooldown_until = current_block.saturating_add(T::ExitCooldownPeriod::get());

			// 更新状态为 Exiting
			Nodes::<T>::mutate(&node_id, |maybe_node| {
				if let Some(n) = maybe_node {
					n.status = NodeStatus::Exiting;
				}
			});

			// 从活跃列表移除
			ActiveNodeList::<T>::mutate(|list| {
				list.retain(|id| id != &node_id);
			});

			// 记录退出请求时间
			ExitRequests::<T>::insert(&node_id, current_block);

			Self::deposit_event(Event::NodeExitRequested {
				node_id,
				operator,
				cooldown_until,
			});

			Ok(())
		}

		/// 完成节点退出（冷却期结束后，释放质押）
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(30_000_000, 0))]
		pub fn finalize_exit(
			origin: OriginFor<T>,
			node_id: NodeId,
		) -> DispatchResult {
			let operator = ensure_signed(origin)?;

			let node = Nodes::<T>::get(&node_id).ok_or(Error::<T>::NodeNotFound)?;
			ensure!(node.operator == operator, Error::<T>::NotNodeOperator);
			ensure!(node.status == NodeStatus::Exiting, Error::<T>::NodeNotExiting);

			// 检查冷却期
			let exit_block = ExitRequests::<T>::get(&node_id)
				.ok_or(Error::<T>::NodeNotExiting)?;
			let current_block = <frame_system::Pallet<T>>::block_number();
			ensure!(
				current_block >= exit_block.saturating_add(T::ExitCooldownPeriod::get()),
				Error::<T>::CooldownNotExpired
			);

			// 释放质押
			let stake = node.stake;
			T::Currency::unreserve(&operator, stake);

			// 清理存储
			Nodes::<T>::remove(&node_id);
			ExitRequests::<T>::remove(&node_id);

			// 从运营者节点列表移除
			OperatorNodes::<T>::mutate(&operator, |list| {
				list.retain(|id| id != &node_id);
			});

			Self::deposit_event(Event::NodeExited {
				node_id,
				operator,
				stake_returned: stake,
			});

			Ok(())
		}

		/// 批量提交消息确认（节点定期调用）
		///
		/// 每个确认包含: (msg_id, msg_hash, 确认节点列表)
		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(100_000_000, 0))]
		pub fn submit_confirmations(
			origin: OriginFor<T>,
			confirmations: Vec<(
				[u8; 32],                        // msg_id
				T::AccountId,                    // owner
				u64,                             // sequence
				[u8; 32],                        // msg_hash
				Vec<NodeId>,                     // confirmed_by nodes
			)>,
		) -> DispatchResult {
			let submitter = ensure_signed(origin)?;

			ensure!(!confirmations.is_empty(), Error::<T>::EmptyConfirmations);

			// 验证提交者是活跃节点
			let active_list = ActiveNodeList::<T>::get();
			let is_active = active_list.iter().any(|id| {
				Nodes::<T>::get(id)
					.map(|n| n.operator == submitter)
					.unwrap_or(false)
			});
			ensure!(is_active, Error::<T>::NodeNotActive);

			let current_block = <frame_system::Pallet<T>>::block_number();
			let mut count = 0u32;

			for (msg_id, owner, sequence, msg_hash, confirmed_by) in confirmations.iter() {
				// 跳过已存在的确认
				if MessageConfirmations::<T>::contains_key(msg_id) {
					continue;
				}

				let bounded_confirmed: BoundedVec<NodeId, ConstU32<20>> =
					confirmed_by.clone()
						.try_into()
						.unwrap_or_default();

				let confirmation = MessageConfirmation {
					owner: owner.clone(),
					sequence: *sequence,
					msg_hash: *msg_hash,
					confirmed_by: bounded_confirmed,
					confirmed_at: current_block,
				};

				MessageConfirmations::<T>::insert(msg_id, confirmation);
				count = count.saturating_add(1);
			}

			if count > 0 {
				Self::deposit_event(Event::ConfirmationsSubmitted {
					submitter,
					count,
				});
			}

			Ok(())
		}

		/// 提交 Equivocation 证据（任何节点可调用）
		///
		/// 证据: 同一 (owner, sequence) 对应两个不同的 msg_hash
		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(60_000_000, 0))]
		pub fn report_equivocation(
			origin: OriginFor<T>,
			owner: T::AccountId,
			sequence: u64,
			msg_hash_a: [u8; 32],
			signature_a: [u8; 64],
			msg_hash_b: [u8; 32],
			signature_b: [u8; 64],
		) -> DispatchResult {
			let reporter = ensure_signed(origin)?;

			// 两个哈希必须不同
			ensure!(msg_hash_a != msg_hash_b, Error::<T>::InvalidEquivocationEvidence);

			// 不能重复举报
			ensure!(
				!EquivocationRecords::<T>::contains_key(&owner, sequence),
				Error::<T>::EquivocationAlreadyReported
			);

			// TODO: Sprint 5 — 验证两个 Ed25519 签名的有效性
			// 当前简化版: 存储证据，后续完善链上验签

			let current_block = <frame_system::Pallet<T>>::block_number();

			let record = EquivocationRecord {
				owner: owner.clone(),
				sequence,
				msg_hash_a,
				signature_a,
				msg_hash_b,
				signature_b,
				reporter: reporter.clone(),
				reported_at: current_block,
				resolved: false,
			};

			EquivocationRecords::<T>::insert(&owner, sequence, record);

			Self::deposit_event(Event::EquivocationReported {
				owner,
				sequence,
				reporter,
			});

			Ok(())
		}

		/// 举报节点离线
		///
		/// 提供证据: 该节点应参与但未响应的消息 ID 列表
		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(40_000_000, 0))]
		pub fn report_node_offline(
			origin: OriginFor<T>,
			node_id: NodeId,
			evidence_msg_ids: Vec<[u8; 32]>,
		) -> DispatchResult {
			let reporter = ensure_signed(origin)?;

			let mut node = Nodes::<T>::get(&node_id).ok_or(Error::<T>::NodeNotFound)?;
			let evidence_count = evidence_msg_ids.len() as u32;

			// 扣信誉: 每条证据 -10
			let penalty = (evidence_count as u16).saturating_mul(10);
			let old_reputation = node.reputation;
			node.reputation = node.reputation.saturating_sub(penalty);
			node.messages_missed = node.messages_missed.saturating_add(evidence_count as u64);

			// 检查是否需要 Suspend
			if node.reputation < T::SuspendThreshold::get() && node.status == NodeStatus::Active {
				node.status = NodeStatus::Suspended;

				// 从活跃列表移除
				ActiveNodeList::<T>::mutate(|list| {
					list.retain(|id| id != &node_id);
				});

				Self::deposit_event(Event::NodeSuspended {
					node_id: node_id.clone(),
					reason: SuspendReason::Offline,
				});
			}

			let new_reputation = node.reputation;
			Nodes::<T>::insert(&node_id, node);

			Self::deposit_event(Event::ReputationUpdated {
				node_id: node_id.clone(),
				old_reputation,
				new_reputation,
			});

			Self::deposit_event(Event::NodeOfflineReported {
				node_id,
				reporter,
				evidence_count,
			});

			Ok(())
		}

		/// 举报 Leader 超时
		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(30_000_000, 0))]
		pub fn report_leader_timeout(
			origin: OriginFor<T>,
			node_id: NodeId,
		) -> DispatchResult {
			let _reporter = ensure_signed(origin)?;

			ensure!(Nodes::<T>::contains_key(&node_id), Error::<T>::NodeNotFound);

			LeaderStatsStore::<T>::mutate(&node_id, |stats| {
				stats.total_leads = stats.total_leads.saturating_add(1);
				stats.timeout = stats.timeout.saturating_add(1);
				stats.consecutive_timeouts = stats.consecutive_timeouts.saturating_add(1);
			});

			let stats = LeaderStatsStore::<T>::get(&node_id);

			// 连续 3 次超时 → 扣信誉 100
			if stats.consecutive_timeouts >= 3 {
				Nodes::<T>::mutate(&node_id, |maybe_node| {
					if let Some(n) = maybe_node {
						let old_rep = n.reputation;
						n.reputation = n.reputation.saturating_sub(100);

						Self::deposit_event(Event::ReputationUpdated {
							node_id: node_id.clone(),
							old_reputation: old_rep,
							new_reputation: n.reputation,
						});
					}
				});
			}

			Self::deposit_event(Event::LeaderTimeoutReported {
				node_id,
				consecutive_timeouts: stats.consecutive_timeouts,
			});

			Ok(())
		}

		/// 记录 Leader 成功执行（重置连续超时计数）
		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(20_000_000, 0))]
		pub fn report_leader_success(
			origin: OriginFor<T>,
			node_id: NodeId,
		) -> DispatchResult {
			let _submitter = ensure_signed(origin)?;

			ensure!(Nodes::<T>::contains_key(&node_id), Error::<T>::NodeNotFound);

			LeaderStatsStore::<T>::mutate(&node_id, |stats| {
				stats.total_leads = stats.total_leads.saturating_add(1);
				stats.successful = stats.successful.saturating_add(1);
				stats.consecutive_timeouts = 0;
			});

			// 成功执行加信誉 +1
			Nodes::<T>::mutate(&node_id, |maybe_node| {
				if let Some(n) = maybe_node {
					n.reputation = n.reputation.saturating_add(1).min(10000);
					n.last_active = <frame_system::Pallet<T>>::block_number();
				}
			});

			Ok(())
		}

		/// 将节点从试用期转为活跃（管理员操作或自动触发）
		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_parts(20_000_000, 0))]
		pub fn activate_node(
			origin: OriginFor<T>,
			node_id: NodeId,
		) -> DispatchResult {
			let operator = ensure_signed(origin)?;

			let mut node = Nodes::<T>::get(&node_id).ok_or(Error::<T>::NodeNotFound)?;
			ensure!(node.operator == operator, Error::<T>::NotNodeOperator);
			ensure!(node.status == NodeStatus::Probation, Error::<T>::NodeNotActive);

			node.status = NodeStatus::Active;
			Nodes::<T>::insert(&node_id, node);

			Self::deposit_event(Event::NodeActivated { node_id });

			Ok(())
		}

		// ═══════════════════════════════════════════════════════════════
		// 订阅管理
		// ═══════════════════════════════════════════════════════════════

		/// 开通订阅（预存 deposit_amount）
		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_parts(50_000_000, 5_000))]
		pub fn subscribe(
			origin: OriginFor<T>,
			bot_id_hash: [u8; 32],
			tier: SubscriptionTier,
			deposit_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 验证 Bot 存在且调用者是 owner
			ensure!(
				T::BotRegistry::bot_exists(&bot_id_hash),
				Error::<T>::BotNotRegistered
			);
			ensure!(
				T::BotRegistry::is_bot_owner(&bot_id_hash, &who),
				Error::<T>::NotBotOwner
			);

			// 不能重复订阅
			ensure!(
				!Subscriptions::<T>::contains_key(&bot_id_hash),
				Error::<T>::SubscriptionAlreadyExists
			);

			// 计算每 Era 费用
			let fee_per_era = Self::tier_fee(&tier);

			// 至少预存 1 Era 费用
			ensure!(deposit_amount >= fee_per_era, Error::<T>::InsufficientDeposit);

			// 从账户转入锁定
			T::Currency::reserve(&who, deposit_amount)
				.map_err(|_| Error::<T>::InsufficientDeposit)?;

			let current_block = <frame_system::Pallet<T>>::block_number();
			let current_era = CurrentEra::<T>::get();

			let sub = Subscription {
				owner: who.clone(),
				bot_id_hash,
				tier,
				fee_per_era,
				started_at: current_block,
				paid_until_era: current_era,
				status: SubscriptionStatus::Active,
			};

			Subscriptions::<T>::insert(&bot_id_hash, sub);
			SubscriptionEscrow::<T>::insert(&bot_id_hash, deposit_amount);

			Self::deposit_event(Event::Subscribed {
				bot_id_hash,
				owner: who,
				tier,
				deposit: deposit_amount,
			});

			Ok(())
		}

		/// 充值订阅余额
		#[pallet::call_index(10)]
		#[pallet::weight(Weight::from_parts(30_000_000, 5_000))]
		pub fn deposit_subscription(
			origin: OriginFor<T>,
			bot_id_hash: [u8; 32],
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let sub = Subscriptions::<T>::get(&bot_id_hash)
				.ok_or(Error::<T>::SubscriptionNotFound)?;
			ensure!(sub.owner == who, Error::<T>::NotBotOwner);
			ensure!(sub.status != SubscriptionStatus::Cancelled, Error::<T>::SubscriptionAlreadyCancelled);

			T::Currency::reserve(&who, amount)
				.map_err(|_| Error::<T>::InsufficientDeposit)?;

			let new_escrow = SubscriptionEscrow::<T>::mutate(&bot_id_hash, |e| {
				*e = e.saturating_add(amount);
				*e
			});

			// 如果之前欠费，充值后恢复
			if sub.status == SubscriptionStatus::PastDue || sub.status == SubscriptionStatus::Suspended {
				Subscriptions::<T>::mutate(&bot_id_hash, |maybe_sub| {
					if let Some(s) = maybe_sub {
						s.status = SubscriptionStatus::Active;
					}
				});
			}

			Self::deposit_event(Event::SubscriptionDeposited {
				bot_id_hash,
				amount,
				new_escrow,
			});

			Ok(())
		}

		/// 取消订阅（剩余 Escrow 退还）
		#[pallet::call_index(11)]
		#[pallet::weight(Weight::from_parts(40_000_000, 5_000))]
		pub fn cancel_subscription(
			origin: OriginFor<T>,
			bot_id_hash: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let sub = Subscriptions::<T>::get(&bot_id_hash)
				.ok_or(Error::<T>::SubscriptionNotFound)?;
			ensure!(sub.owner == who, Error::<T>::NotBotOwner);
			ensure!(sub.status != SubscriptionStatus::Cancelled, Error::<T>::SubscriptionAlreadyCancelled);

			// 退还 Escrow
			let escrow = SubscriptionEscrow::<T>::get(&bot_id_hash);
			if escrow > BalanceOf::<T>::from(0u32) {
				T::Currency::unreserve(&who, escrow);
			}

			SubscriptionEscrow::<T>::remove(&bot_id_hash);
			Subscriptions::<T>::mutate(&bot_id_hash, |maybe_sub| {
				if let Some(s) = maybe_sub {
					s.status = SubscriptionStatus::Cancelled;
				}
			});

			Self::deposit_event(Event::SubscriptionCancelled {
				bot_id_hash,
				refunded: escrow,
			});

			Ok(())
		}

		/// 升级/降级层级
		#[pallet::call_index(12)]
		#[pallet::weight(Weight::from_parts(30_000_000, 5_000))]
		pub fn change_tier(
			origin: OriginFor<T>,
			bot_id_hash: [u8; 32],
			new_tier: SubscriptionTier,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let sub = Subscriptions::<T>::get(&bot_id_hash)
				.ok_or(Error::<T>::SubscriptionNotFound)?;
			ensure!(sub.owner == who, Error::<T>::NotBotOwner);
			ensure!(sub.status != SubscriptionStatus::Cancelled, Error::<T>::SubscriptionAlreadyCancelled);
			ensure!(sub.tier != new_tier, Error::<T>::SameTier);

			let old_tier = sub.tier;
			let new_fee = Self::tier_fee(&new_tier);

			Subscriptions::<T>::mutate(&bot_id_hash, |maybe_sub| {
				if let Some(s) = maybe_sub {
					s.tier = new_tier;
					s.fee_per_era = new_fee;
				}
			});

			Self::deposit_event(Event::TierChanged {
				bot_id_hash,
				old_tier,
				new_tier,
			});

			Ok(())
		}

		/// 领取节点奖励
		#[pallet::call_index(13)]
		#[pallet::weight(Weight::from_parts(40_000_000, 5_000))]
		pub fn claim_rewards(
			origin: OriginFor<T>,
			node_id: NodeId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let node = Nodes::<T>::get(&node_id).ok_or(Error::<T>::NodeNotFound)?;
			ensure!(node.operator == who, Error::<T>::NotNodeOperator);

			let pending = NodePendingRewards::<T>::get(&node_id);
			ensure!(pending > BalanceOf::<T>::from(0u32), Error::<T>::NoPendingRewards);

			// 从 reserved 解锁并转入 free (通胀部分通过 deposit_creating)
			// 简化实现: 直接 deposit_creating 给运营者
			let _ = T::Currency::deposit_creating(&who, pending);

			NodePendingRewards::<T>::remove(&node_id);

			Self::deposit_event(Event::RewardsClaimed {
				node_id,
				operator: who,
				amount: pending,
			});

			Ok(())
		}
	}

	// ═══════════════════════════════════════════════════════════════
	// Hooks
	// ═══════════════════════════════════════════════════════════════

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			let era_start = EraStartBlock::<T>::get();
			let era_length = T::EraLength::get();

			// 检查是否到达 Era 边界
			if era_length > BlockNumberFor::<T>::from(0u32)
				&& now >= era_start.saturating_add(era_length)
			{
				let era = CurrentEra::<T>::get();
				Self::on_era_end(era);
				EraStartBlock::<T>::put(now);
				return Weight::from_parts(200_000_000, 10_000);
			}

			Weight::from_parts(5_000, 0)
		}
	}

	// ═══════════════════════════════════════════════════════════════
	// 辅助函数
	// ═══════════════════════════════════════════════════════════════

	impl<T: Config> Pallet<T> {
		/// 获取活跃节点数量
		pub fn active_node_count() -> u32 {
			ActiveNodeList::<T>::get().len() as u32
		}

		/// 检查节点是否活跃
		pub fn is_node_active(node_id: &NodeId) -> bool {
			Nodes::<T>::get(node_id)
				.map(|n| n.status == NodeStatus::Active || n.status == NodeStatus::Probation)
				.unwrap_or(false)
		}

		/// 更新节点信誉（内部函数）
		pub fn update_reputation(node_id: &NodeId, delta: i32) {
			Nodes::<T>::mutate(node_id, |maybe_node| {
				if let Some(n) = maybe_node {
					let old = n.reputation;
					if delta >= 0 {
						n.reputation = n.reputation.saturating_add(delta as u16).min(10000);
					} else {
						n.reputation = n.reputation.saturating_sub((-delta) as u16);
					}

					// 检查是否需要自动 Suspend
					if n.reputation < T::SuspendThreshold::get()
						&& (n.status == NodeStatus::Active || n.status == NodeStatus::Probation)
					{
						n.status = NodeStatus::Suspended;

						ActiveNodeList::<T>::mutate(|list| {
							list.retain(|id| id != node_id);
						});

						Self::deposit_event(Event::NodeSuspended {
							node_id: node_id.clone(),
							reason: SuspendReason::LowReputation,
						});
					}

					Self::deposit_event(Event::ReputationUpdated {
						node_id: node_id.clone(),
						old_reputation: old,
						new_reputation: n.reputation,
					});
				}
			});
		}

		/// 获取层级对应的每 Era 费用
		pub fn tier_fee(tier: &SubscriptionTier) -> BalanceOf<T> {
			match tier {
				SubscriptionTier::Basic => T::BasicFeePerEra::get(),
				SubscriptionTier::Pro => T::ProFeePerEra::get(),
				SubscriptionTier::Enterprise => T::EnterpriseFeePerEra::get(),
			}
		}

		/// 计算节点奖励权重
		///
		/// weight = reputation × uptime × leader_bonus / 10^8
		/// - reputation: 0-10000
		/// - uptime: 0-10000 (新节点默认 5000)
		/// - leader_bonus: 10000-15000 (1.0x - 1.5x)
		pub fn compute_node_weight(node: &ProjectNode<T>, stats: &LeaderStats) -> u128 {
			let rep = node.reputation as u128;

			let total = node.messages_confirmed.saturating_add(node.messages_missed);
			let uptime = if total == 0 {
				5000u128
			} else {
				(node.messages_confirmed as u128)
					.saturating_mul(10000)
					.checked_div(total as u128)
					.unwrap_or(0)
			};

			let leader_bonus = if stats.total_leads == 0 {
				10000u128
			} else {
				10000u128.saturating_add(
					(stats.successful as u128)
						.saturating_mul(5000)
						.checked_div(stats.total_leads as u128)
						.unwrap_or(0)
				)
			};

			rep.saturating_mul(uptime)
				.saturating_mul(leader_bonus)
				.checked_div(100_000_000)
				.unwrap_or(0)
		}

		/// Era 结束处理: 扣费 + 铸币 + 分配
		fn on_era_end(era: u32) {
			use sp_runtime::traits::Zero;

			let mut subscription_income: BalanceOf<T> = Zero::zero();

			// ═══ 1. 收取订阅费 ═══
			for (bot_hash, sub) in Subscriptions::<T>::iter() {
				if sub.status == SubscriptionStatus::Cancelled {
					continue;
				}

				let escrow = SubscriptionEscrow::<T>::get(&bot_hash);
				if escrow >= sub.fee_per_era {
					// 扣费: unreserve fee 金额 (销毁/转入池)
					T::Currency::unreserve(&sub.owner, sub.fee_per_era);
					SubscriptionEscrow::<T>::mutate(&bot_hash, |e| {
						*e = e.saturating_sub(sub.fee_per_era);
					});
					subscription_income = subscription_income.saturating_add(sub.fee_per_era);

					Subscriptions::<T>::mutate(&bot_hash, |maybe_sub| {
						if let Some(s) = maybe_sub {
							s.paid_until_era = era;
							s.status = SubscriptionStatus::Active;
						}
					});
				} else {
					// 欠费处理
					let old_status = sub.status;
					let new_status = match old_status {
						SubscriptionStatus::Active => SubscriptionStatus::PastDue,
						SubscriptionStatus::PastDue => SubscriptionStatus::Suspended,
						other => other,
					};

					Subscriptions::<T>::mutate(&bot_hash, |maybe_sub| {
						if let Some(s) = maybe_sub {
							s.status = new_status;
						}
					});

					if new_status == SubscriptionStatus::Suspended {
						Self::deposit_event(Event::SubscriptionSuspended {
							bot_id_hash: bot_hash,
						});
					}
				}
			}

			// ═══ 2. 订阅收入分配 (80% 节点 / 10% 国库 / 10% Agent) ═══
			let node_share = subscription_income
				.saturating_mul(80u32.into()) / 100u32.into();
			let treasury_share = subscription_income
				.saturating_mul(10u32.into()) / 100u32.into();
			// 剩余 10% Agent 补贴 — 暂留在池中

			// ═══ 3. 通胀铸币 ═══
			let inflation = T::InflationPerEra::get();

			// ═══ 4. 合并奖励池 ═══
			let total_pool = node_share.saturating_add(inflation);

			// ═══ 5. 按权重分配给节点 ═══
			let active_nodes = ActiveNodeList::<T>::get();
			let mut eligible: sp_std::vec::Vec<(NodeId, u128)> = sp_std::vec::Vec::new();
			let mut total_weight: u128 = 0;

			for node_id in active_nodes.iter() {
				if let Some(node) = Nodes::<T>::get(node_id) {
					let stats = LeaderStatsStore::<T>::get(node_id);

					// 在线率检查
					let total_msgs = node.messages_confirmed.saturating_add(node.messages_missed);
					let uptime_ok = if total_msgs == 0 {
						true // 新节点默认通过
					} else {
						Perbill::from_rational(node.messages_confirmed, total_msgs)
							>= T::MinUptimeForReward::get()
					};

					if node.status == NodeStatus::Active && uptime_ok {
						let w = Self::compute_node_weight(&node, &stats);
						total_weight = total_weight.saturating_add(w);
						eligible.push((node_id.clone(), w));
					}
				}
			}

			let mut actually_distributed: BalanceOf<T> = Zero::zero();

			if total_weight > 0 && !eligible.is_empty() {
				let max_per_node = T::MaxRewardShare::get() * total_pool;

				for (node_id, w) in &eligible {
					// raw_reward = total_pool * w / total_weight
					let raw_reward = Perbill::from_rational(*w, total_weight) * total_pool;
					let capped = raw_reward.min(max_per_node);

					NodePendingRewards::<T>::mutate(node_id, |p| {
						*p = p.saturating_add(capped);
					});
					NodeTotalEarned::<T>::mutate(node_id, |t| {
						*t = t.saturating_add(capped);
					});
					actually_distributed = actually_distributed.saturating_add(capped);
				}
			}

			// ═══ 6. 记录 Era 信息 ═══
			EraRewards::<T>::insert(era, EraRewardInfo {
				subscription_income,
				inflation_mint: inflation,
				total_distributed: actually_distributed,
				treasury_share,
				node_count: eligible.len() as u32,
			});

			Self::deposit_event(Event::EraCompleted {
				era,
				subscription_income,
				inflation_mint: inflation,
				node_count: eligible.len() as u32,
			});

			// ═══ 7. 推进 Era ═══
			CurrentEra::<T>::put(era.saturating_add(1));
		}
	}
}

/// Bot 注册表 Provider trait（解耦 pallet-bot-registry 依赖）
pub trait BotRegistryProvider<AccountId> {
	/// Bot 是否存在
	fn bot_exists(bot_id_hash: &[u8; 32]) -> bool;
	/// 是否为 Bot 的所有者
	fn is_bot_owner(bot_id_hash: &[u8; 32], who: &AccountId) -> bool;
}
