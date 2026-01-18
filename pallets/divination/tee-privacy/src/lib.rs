//! # TEE 隐私计算模块 (pallet-tee-privacy)
//!
//! 本模块为 Stardust 链提供基于可信执行环境 (TEE) 的隐私计算能力。
//!
//! ## 功能概述
//!
//! 1. **TEE 节点管理**：TEE 节点注册、认证、状态管理
//! 2. **远程认证**：支持 Intel SGX EPID/DCAP、ARM TrustZone 认证
//! 3. **计算请求**：加密数据提交、TEE 内计算、结果返回
//! 4. **计算证明**：证明计算在真实 TEE 环境中执行
//! 5. **经济激励**：质押、奖励、惩罚机制
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                           用户/客户端                                    │
//! │  1. 获取 Enclave 公钥                                                    │
//! │  2. 加密敏感数据 (ECDH + AES-256-GCM)                                   │
//! │  3. 提交计算请求                                                         │
//! └───────────────────────────────────────┬─────────────────────────────────┘
//!                                         │
//!                                         ▼
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                       pallet-tee-privacy (链上)                          │
//! │  ┌─────────────────┐ ┌─────────────────┐ ┌────────────────────────────┐ │
//! │  │ TEE 节点注册    │ │ 请求队列管理    │ │ 结果验证 & 存储             │ │
//! │  │ - 认证验证      │ │ - 超时处理      │ │ - 签名验证                  │ │
//! │  │ - 状态管理      │ │ - 故障转移      │ │ - 证明验证                  │ │
//! │  └─────────────────┘ └─────────────────┘ └────────────────────────────┘ │
//! └───────────────────────────────────────┬─────────────────────────────────┘
//!                                         │
//!                                         ▼
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                        TEE Enclave (链下)                                │
//! │  ┌─────────────────┐ ┌─────────────────┐ ┌────────────────────────────┐ │
//! │  │ 密钥管理        │ │ 解密 & 计算     │ │ 加密结果 & 签名             │ │
//! │  │ - ECDH 密钥    │ │ - 占卜算法      │ │ - 计算证明                  │ │
//! │  │ - 密封存储      │ │ - 业务逻辑      │ │ - Enclave 签名              │ │
//! │  └─────────────────┘ └─────────────────┘ └────────────────────────────┘ │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ### 1. 注册 TEE 节点
//! ```ignore
//! TeePrivacy::register_tee_node(
//!     origin,
//!     enclave_pubkey,
//!     attestation,
//! )?;
//! ```
//!
//! ### 2. 提交计算请求
//! ```ignore
//! TeePrivacy::submit_compute_request(
//!     origin,
//!     compute_type,
//!     encrypted_input,
//!     None, // 自动分配节点
//! )?;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod attestation;
pub mod runtime_api;
pub mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::pallet_prelude::*;
use frame_support::traits::{Currency, ReservableCurrency, ConstU32};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{Saturating, Zero};
use sp_std::vec::Vec;

use crate::types::*;
use crate::weights::WeightInfo;

/// 认证有效期 (约 24 小时，假设 6 秒一个区块)
pub const ATTESTATION_VALIDITY_BLOCKS: u32 = 14400;

/// 最大活跃节点数
pub const MAX_ACTIVE_NODES: u32 = 100;

/// 默认请求超时区块数 (约 10 分钟)
pub const DEFAULT_REQUEST_TIMEOUT_BLOCKS: u32 = 100;

/// 最大故障转移次数
pub const MAX_FAILOVER_COUNT: u8 = 3;

/// 货币类型别名
pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    // ========================================================================
    // Pallet 配置
    // ========================================================================

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Pallet 配置 trait
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// 运行时事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 货币类型
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 最大节点数
        #[pallet::constant]
        type MaxNodes: Get<u32>;

        /// 最大待处理请求数
        #[pallet::constant]
        type MaxPendingRequests: Get<u32>;

        /// 认证有效期（区块数）
        #[pallet::constant]
        type AttestationValidity: Get<u32>;

        /// 允许的 MRENCLAVE 列表最大长度
        #[pallet::constant]
        type MaxAllowedMrEnclaves: Get<u32>;

        /// 请求超时区块数
        #[pallet::constant]
        type RequestTimeout: Get<u32>;

        /// 最小质押金额
        #[pallet::constant]
        type MinimumStake: Get<BalanceOf<Self>>;

        /// 计算费用基础价格
        #[pallet::constant]
        type BaseComputeFee: Get<BalanceOf<Self>>;

        /// 惩罚比例 (千分比，例如 100 = 10%)
        #[pallet::constant]
        type SlashRatio: Get<u32>;

        /// 最大批处理请求数量
        #[pallet::constant]
        type MaxBatchSize: Get<u32>;

        /// 权重信息
        type WeightInfo: WeightInfo;
    }

    // ========================================================================
    // 存储定义
    // ========================================================================

    // -------------------- TEE 节点管理 --------------------

    /// TEE 节点信息
    ///
    /// AccountId -> TeeNode
    #[pallet::storage]
    #[pallet::getter(fn tee_nodes)]
    pub type TeeNodes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, TeeNode<T::AccountId>>;

    /// 活跃节点列表
    ///
    /// 存储所有活跃状态的 TEE 节点账户
    #[pallet::storage]
    #[pallet::getter(fn active_nodes)]
    pub type ActiveNodes<T: Config> =
        StorageValue<_, BoundedVec<T::AccountId, T::MaxNodes>, ValueQuery>;

    /// 节点总数
    #[pallet::storage]
    #[pallet::getter(fn node_count)]
    pub type NodeCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    // -------------------- 允许的 Enclave 配置 --------------------

    /// 允许的 MRENCLAVE 列表
    ///
    /// 只有在此列表中的 Enclave 度量值才能注册
    #[pallet::storage]
    #[pallet::getter(fn allowed_mr_enclaves)]
    pub type AllowedMrEnclaves<T: Config> =
        StorageValue<_, BoundedVec<[u8; 32], T::MaxAllowedMrEnclaves>, ValueQuery>;

    /// 允许的 MRSIGNER 列表
    ///
    /// 只有在此列表中的签名者才能注册
    #[pallet::storage]
    #[pallet::getter(fn allowed_mr_signers)]
    pub type AllowedMrSigners<T: Config> =
        StorageValue<_, BoundedVec<[u8; 32], T::MaxAllowedMrEnclaves>, ValueQuery>;

    // -------------------- 计算请求管理 --------------------

    /// 请求 ID 计数器
    #[pallet::storage]
    #[pallet::getter(fn next_request_id)]
    pub type NextRequestId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// 计算请求存储
    ///
    /// request_id -> ComputeRequestInfo
    #[pallet::storage]
    #[pallet::getter(fn compute_requests)]
    pub type ComputeRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        ComputeRequestInfo<T::AccountId, BlockNumberFor<T>>,
    >;

    /// 计算结果存储
    ///
    /// request_id -> ComputeResultInfo
    #[pallet::storage]
    #[pallet::getter(fn compute_results)]
    pub type ComputeResults<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, ComputeResultInfo<T::AccountId>>;

    /// 待处理请求列表（按超时区块排序）
    ///
    /// 用于 on_finalize 检查超时
    #[pallet::storage]
    #[pallet::getter(fn pending_requests)]
    pub type PendingRequests<T: Config> =
        StorageValue<_, BoundedVec<u64, T::MaxPendingRequests>, ValueQuery>;

    /// 节点当前处理的请求
    ///
    /// node_account -> request_id
    #[pallet::storage]
    #[pallet::getter(fn node_current_request)]
    pub type NodeCurrentRequest<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64>;

    // -------------------- 经济激励 --------------------

    /// 节点质押信息
    ///
    /// node_account -> StakeInfo
    #[pallet::storage]
    #[pallet::getter(fn node_stakes)]
    pub type NodeStakes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, StakeInfo<BalanceOf<T>, BlockNumberFor<T>>>;

    /// 节点统计信息
    ///
    /// node_account -> NodeStats
    #[pallet::storage]
    #[pallet::getter(fn node_stats)]
    pub type NodeStats<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, NodeStatistics>;

    /// 累计奖励池
    #[pallet::storage]
    #[pallet::getter(fn reward_pool)]
    pub type RewardPool<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// 累计惩罚金额
    #[pallet::storage]
    #[pallet::getter(fn total_slashed)]
    pub type TotalSlashed<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    // -------------------- 审计日志 --------------------

    /// 审计日志 ID 计数器
    #[pallet::storage]
    #[pallet::getter(fn next_audit_log_id)]
    pub type NextAuditLogId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// 审计日志存储
    ///
    /// log_id -> AuditLogEntry
    #[pallet::storage]
    #[pallet::getter(fn audit_logs)]
    pub type AuditLogs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        crate::types::AuditLogEntry<T::AccountId, BlockNumberFor<T>>,
    >;

    /// 按账户索引的审计日志
    ///
    /// account -> Vec<log_id>
    #[pallet::storage]
    #[pallet::getter(fn account_audit_logs)]
    pub type AccountAuditLogs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<100>>,
        ValueQuery,
    >;

    /// 审计日志是否启用
    #[pallet::storage]
    #[pallet::getter(fn audit_enabled)]
    pub type AuditEnabled<T: Config> = StorageValue<_, bool, ValueQuery>;

    // ========================================================================
    // 创世配置
    // ========================================================================

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// 初始允许的 MRENCLAVE 列表
        pub allowed_mr_enclaves: Vec<[u8; 32]>,
        /// 初始允许的 MRSIGNER 列表
        pub allowed_mr_signers: Vec<[u8; 32]>,
        #[serde(skip)]
        pub _phantom: core::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // 初始化允许的 MRENCLAVE 列表
            let mr_enclaves: BoundedVec<[u8; 32], T::MaxAllowedMrEnclaves> =
                self.allowed_mr_enclaves.clone().try_into().expect("Too many MREnclave entries");
            AllowedMrEnclaves::<T>::put(mr_enclaves);

            // 初始化允许的 MRSIGNER 列表
            let mr_signers: BoundedVec<[u8; 32], T::MaxAllowedMrEnclaves> =
                self.allowed_mr_signers.clone().try_into().expect("Too many MRSigner entries");
            AllowedMrSigners::<T>::put(mr_signers);
        }
    }

    // ========================================================================
    // 事件定义
    // ========================================================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // -------------------- 节点管理事件 --------------------

        /// TEE 节点已注册
        TeeNodeRegistered {
            account: T::AccountId,
            enclave_pubkey: [u8; 32],
            tee_type: TeeType,
        },

        /// TEE 节点认证已更新
        AttestationUpdated {
            account: T::AccountId,
            mr_enclave: [u8; 32],
            timestamp: u64,
        },

        /// TEE 节点状态已更新
        NodeStatusUpdated {
            account: T::AccountId,
            old_status: TeeNodeStatus,
            new_status: TeeNodeStatus,
        },

        /// TEE 节点已注销
        TeeNodeDeregistered { account: T::AccountId },

        // -------------------- 计算请求事件 --------------------

        /// 计算请求已提交
        ComputeRequestSubmitted {
            request_id: u64,
            requester: T::AccountId,
            compute_type_id: u8,
            assigned_node: Option<T::AccountId>,
        },

        /// 计算结果已提交
        ComputeResultSubmitted {
            request_id: u64,
            executor: T::AccountId,
            output_hash: [u8; 32],
        },

        /// 计算请求已取消
        ComputeRequestCancelled {
            request_id: u64,
            requester: T::AccountId,
        },

        /// 请求超时（故障转移）
        RequestTimeout {
            request_id: u64,
            old_node: T::AccountId,
            new_node: Option<T::AccountId>,
            failover_count: u8,
        },

        /// 请求最终失败
        RequestFailed {
            request_id: u64,
            reason: FailureReason,
        },

        // -------------------- 经济激励事件 --------------------

        /// 节点已质押
        Staked {
            account: T::AccountId,
            amount: BalanceOf<T>,
        },

        /// 申请解除质押
        UnstakeRequested {
            account: T::AccountId,
            amount: BalanceOf<T>,
            unlock_at: BlockNumberFor<T>,
        },

        /// 已提取解除质押的金额
        Withdrawn {
            account: T::AccountId,
            amount: BalanceOf<T>,
        },

        /// 奖励已发放
        RewardPaid {
            account: T::AccountId,
            amount: BalanceOf<T>,
        },

        /// 节点已被惩罚
        Slashed {
            account: T::AccountId,
            amount: BalanceOf<T>,
            reason: FailureReason,
        },

        // -------------------- 配置事件 --------------------

        /// 允许的 MRENCLAVE 已添加
        MrEnclaveAllowed { mr_enclave: [u8; 32] },

        /// 允许的 MRENCLAVE 已移除
        MrEnclaveDisallowed { mr_enclave: [u8; 32] },

        /// 允许的 MRSIGNER 已添加
        MrSignerAllowed { mr_signer: [u8; 32] },

        /// 允许的 MRSIGNER 已移除
        MrSignerDisallowed { mr_signer: [u8; 32] },

        // -------------------- 批处理事件 --------------------

        /// 批量计算请求已提交
        BatchComputeRequestsSubmitted {
            requester: T::AccountId,
            request_ids: Vec<u64>,
            count: u32,
        },

        /// 批量计算结果已提交
        BatchComputeResultsSubmitted {
            executor: T::AccountId,
            request_ids: Vec<u64>,
            count: u32,
        },

        // -------------------- 审计日志事件 --------------------

        /// 审计日志状态已更改
        AuditStatusChanged { enabled: bool },
    }

    // ========================================================================
    // 错误定义
    // ========================================================================

    #[pallet::error]
    pub enum Error<T> {
        // -------------------- 节点管理错误 --------------------

        /// 节点已注册
        NodeAlreadyRegistered,

        /// 节点未注册
        NodeNotRegistered,

        /// 节点列表已满
        NodeListFull,

        /// 无效的 Enclave 公钥
        InvalidEnclavePubkey,

        /// 节点不活跃
        NodeNotActive,

        /// 节点状态无效
        InvalidNodeStatus,

        // -------------------- 认证错误 --------------------

        /// 无效的认证报告
        InvalidAttestation,

        /// 认证已过期
        AttestationExpired,

        /// MRENCLAVE 不在允许列表中
        MrEnclaveNotAllowed,

        /// MRSIGNER 不在允许列表中
        MrSignerNotAllowed,

        /// IAS 签名验证失败
        IasSignatureVerificationFailed,

        /// 认证时间戳无效
        InvalidAttestationTimestamp,

        // -------------------- 配置错误 --------------------

        /// 允许列表已满
        AllowedListFull,

        /// 条目已存在
        EntryAlreadyExists,

        /// 条目不存在
        EntryNotFound,

        // -------------------- 计算请求错误 --------------------

        /// 请求不存在
        RequestNotFound,

        /// 请求已完成
        RequestAlreadyCompleted,

        /// 请求已取消
        RequestAlreadyCancelled,

        /// 请求列表已满
        RequestListFull,

        /// 非请求者
        NotRequester,

        /// 非分配节点
        NotAssignedNode,

        /// 无可用节点
        NoAvailableNodes,

        /// 无效的计算证明
        InvalidComputationProof,

        /// 无效的 Enclave 签名
        InvalidEnclaveSignature,

        /// 请求超时
        RequestTimeout,

        // -------------------- 经济激励错误 --------------------

        /// 质押金额不足
        InsufficientStake,

        /// 余额不足
        InsufficientBalance,

        /// 未质押
        NotStaked,

        /// 正在解除质押
        AlreadyUnbonding,

        /// 未在解除质押
        NotUnbonding,

        /// 解锁时间未到
        UnlockTimeNotReached,

        /// 无可领取奖励
        NoRewardsToClaim,

        // -------------------- 通用错误 --------------------

        /// 操作未授权
        Unauthorized,

        /// 数值溢出
        Overflow,

        // -------------------- 批处理错误 --------------------

        /// 批处理为空
        EmptyBatch,

        /// 批处理数量超出限制
        BatchSizeExceeded,

        /// 批处理中包含无效请求
        InvalidBatchItem,
    }

    // ========================================================================
    // 交易接口
    // ========================================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // ====================================================================
        // TEE 节点管理
        // ====================================================================

        /// 注册 TEE 节点
        ///
        /// TEE 运营者提交 Enclave 公钥和远程认证报告进行注册。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（TEE 节点运营账户）
        /// - `enclave_pubkey`: Enclave Ed25519 公钥（32 字节）
        /// - `attestation`: 远程认证报告
        ///
        /// # 错误
        /// - `NodeAlreadyRegistered`: 节点已注册
        /// - `InvalidEnclavePubkey`: 无效的公钥
        /// - `InvalidAttestation`: 无效的认证报告
        /// - `MrEnclaveNotAllowed`: MRENCLAVE 不在允许列表
        /// - `NodeListFull`: 节点列表已满
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::register_tee_node())]
        pub fn register_tee_node(
            origin: OriginFor<T>,
            enclave_pubkey: [u8; 32],
            attestation: TeeAttestation,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查节点是否已注册
            ensure!(
                !TeeNodes::<T>::contains_key(&who),
                Error::<T>::NodeAlreadyRegistered
            );

            // 验证 Enclave 公钥（非全零）
            ensure!(
                enclave_pubkey != [0u8; 32],
                Error::<T>::InvalidEnclavePubkey
            );

            // 验证认证报告
            Self::verify_attestation(&attestation)?;

            // 获取当前时间戳
            let now = pallet_timestamp::Pallet::<T>::get();
            let timestamp = now.try_into().ok().unwrap_or(0u64);

            // 创建 TEE 节点信息
            let node = TeeNode {
                account: who.clone(),
                enclave_pubkey,
                attestation: attestation.clone(),
                registered_at: timestamp,
                status: TeeNodeStatus::Active,
            };

            // 存储节点信息
            TeeNodes::<T>::insert(&who, node);

            // 添加到活跃节点列表
            ActiveNodes::<T>::try_mutate(|nodes| {
                nodes
                    .try_push(who.clone())
                    .map_err(|_| Error::<T>::NodeListFull)
            })?;

            // 更新节点计数
            NodeCount::<T>::mutate(|count| *count = count.saturating_add(1));

            // 触发事件
            Self::deposit_event(Event::TeeNodeRegistered {
                account: who,
                enclave_pubkey,
                tee_type: attestation.tee_type,
            });

            Ok(())
        }

        /// 更新认证报告
        ///
        /// TEE 节点定期更新远程认证报告以保持活跃状态。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是已注册的 TEE 节点）
        /// - `attestation`: 新的远程认证报告
        ///
        /// # 错误
        /// - `NodeNotRegistered`: 节点未注册
        /// - `InvalidAttestation`: 无效的认证报告
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::update_attestation())]
        pub fn update_attestation(
            origin: OriginFor<T>,
            attestation: TeeAttestation,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取节点信息
            TeeNodes::<T>::try_mutate(&who, |maybe_node| {
                let node = maybe_node.as_mut().ok_or(Error::<T>::NodeNotRegistered)?;

                // 验证认证报告
                Self::verify_attestation(&attestation).map_err(|_| Error::<T>::InvalidAttestation)?;

                // 更新认证信息
                node.attestation = attestation.clone();

                Ok::<(), Error<T>>(())
            })?;

            // 触发事件
            Self::deposit_event(Event::AttestationUpdated {
                account: who,
                mr_enclave: attestation.mr_enclave,
                timestamp: attestation.timestamp,
            });

            Ok(())
        }

        /// 更新节点状态
        ///
        /// TEE 节点运营者可以暂停或恢复节点服务。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是节点所有者）
        /// - `new_status`: 新状态（Active 或 Suspended）
        ///
        /// # 错误
        /// - `NodeNotRegistered`: 节点未注册
        /// - `InvalidNodeStatus`: 无效的状态转换
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::update_node_status())]
        pub fn update_node_status(
            origin: OriginFor<T>,
            new_status: TeeNodeStatus,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 只允许设置 Active 或 Suspended 状态
            ensure!(
                matches!(new_status, TeeNodeStatus::Active | TeeNodeStatus::Suspended),
                Error::<T>::InvalidNodeStatus
            );

            let old_status = TeeNodes::<T>::try_mutate(&who, |maybe_node| {
                let node = maybe_node.as_mut().ok_or(Error::<T>::NodeNotRegistered)?;

                // 不能从 Deregistered 状态恢复
                ensure!(
                    node.status != TeeNodeStatus::Deregistered,
                    Error::<T>::InvalidNodeStatus
                );

                let old = node.status;
                node.status = new_status;

                Ok::<TeeNodeStatus, Error<T>>(old)
            })?;

            // 更新活跃节点列表
            if old_status == TeeNodeStatus::Active && new_status == TeeNodeStatus::Suspended {
                // 从活跃列表移除
                ActiveNodes::<T>::mutate(|nodes| {
                    nodes.retain(|account| account != &who);
                });
            } else if old_status == TeeNodeStatus::Suspended && new_status == TeeNodeStatus::Active
            {
                // 添加回活跃列表
                ActiveNodes::<T>::try_mutate(|nodes| {
                    if !nodes.contains(&who) {
                        nodes
                            .try_push(who.clone())
                            .map_err(|_| Error::<T>::NodeListFull)
                    } else {
                        Ok(())
                    }
                })?;
            }

            // 触发事件
            Self::deposit_event(Event::NodeStatusUpdated {
                account: who,
                old_status,
                new_status,
            });

            Ok(())
        }

        /// 注销 TEE 节点
        ///
        /// TEE 节点运营者可以永久注销节点。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是节点所有者）
        ///
        /// # 错误
        /// - `NodeNotRegistered`: 节点未注册
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::deregister_tee_node())]
        pub fn deregister_tee_node(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取并更新节点状态
            TeeNodes::<T>::try_mutate(&who, |maybe_node| {
                let node = maybe_node.as_mut().ok_or(Error::<T>::NodeNotRegistered)?;
                node.status = TeeNodeStatus::Deregistered;
                Ok::<(), Error<T>>(())
            })?;

            // 从活跃节点列表移除
            ActiveNodes::<T>::mutate(|nodes| {
                nodes.retain(|account| account != &who);
            });

            // 更新节点计数
            NodeCount::<T>::mutate(|count| *count = count.saturating_sub(1));

            // 触发事件
            Self::deposit_event(Event::TeeNodeDeregistered { account: who });

            Ok(())
        }

        // ====================================================================
        // 计算请求管理
        // ====================================================================

        /// 提交计算请求
        ///
        /// 用户提交加密数据的哈希，请求 TEE 节点进行计算。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（请求者）
        /// - `compute_type_id`: 计算类型 ID (0=BaZi, 1=MeiHua, 等)
        /// - `input_hash`: 加密输入数据的哈希
        /// - `assigned_node`: 指定节点（可选，None 表示自动分配）
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::submit_compute_request())]
        pub fn submit_compute_request(
            origin: OriginFor<T>,
            compute_type_id: u8,
            input_hash: [u8; 32],
            assigned_node: Option<T::AccountId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 分配节点（如果未指定则自动选择）
            let node = match assigned_node {
                Some(ref n) => {
                    ensure!(Self::is_node_active(n), Error::<T>::NodeNotActive);
                    Some(n.clone())
                }
                None => Self::select_random_node(),
            };

            ensure!(node.is_some(), Error::<T>::NoAvailableNodes);

            // 生成请求 ID
            let request_id = NextRequestId::<T>::get();
            NextRequestId::<T>::put(request_id.saturating_add(1));

            // 获取当前区块和超时区块
            let current_block = frame_system::Pallet::<T>::block_number();
            let timeout_blocks: BlockNumberFor<T> = T::RequestTimeout::get().into();
            let timeout_at = current_block.saturating_add(timeout_blocks);

            // 创建请求信息
            let request = ComputeRequestInfo {
                id: request_id,
                requester: who.clone(),
                compute_type_id,
                input_hash,
                assigned_node: node.clone(),
                created_at: current_block,
                timeout_at,
                status: RequestStatus::Processing,
                failover_count: 0,
                failure_reason: None,
            };

            // 存储请求
            ComputeRequests::<T>::insert(request_id, request);

            // 添加到待处理列表
            PendingRequests::<T>::try_mutate(|pending| {
                pending
                    .try_push(request_id)
                    .map_err(|_| Error::<T>::RequestListFull)
            })?;

            // 记录节点当前处理的请求
            if let Some(ref n) = node {
                NodeCurrentRequest::<T>::insert(n, request_id);
            }

            // 触发事件
            Self::deposit_event(Event::ComputeRequestSubmitted {
                request_id,
                requester: who,
                compute_type_id,
                assigned_node: node,
            });

            Ok(())
        }

        /// 提交计算结果
        ///
        /// TEE 节点提交计算结果和证明。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是分配的 TEE 节点）
        /// - `request_id`: 请求 ID
        /// - `output_hash`: 加密输出数据的哈希
        /// - `enclave_signature`: Enclave 签名
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::submit_compute_result())]
        pub fn submit_compute_result(
            origin: OriginFor<T>,
            request_id: u64,
            output_hash: [u8; 32],
            enclave_signature: [u8; 64],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取并验证请求
            ComputeRequests::<T>::try_mutate(request_id, |maybe_request| {
                let request = maybe_request.as_mut().ok_or(Error::<T>::RequestNotFound)?;

                // 验证状态
                ensure!(
                    request.status == RequestStatus::Processing,
                    Error::<T>::RequestAlreadyCompleted
                );

                // 验证是分配的节点
                ensure!(
                    request.assigned_node.as_ref() == Some(&who),
                    Error::<T>::NotAssignedNode
                );

                // TODO: Phase 4 将验证 Enclave 签名
                // Self::verify_enclave_signature(&who, &output_hash, &enclave_signature)?;

                // 更新请求状态
                request.status = RequestStatus::Completed;

                Ok::<(), Error<T>>(())
            })?;

            // 获取时间戳
            let now = pallet_timestamp::Pallet::<T>::get();
            let timestamp: u64 = now.try_into().ok().unwrap_or(0);

            // 存储结果
            let result = ComputeResultInfo {
                request_id,
                executor: who.clone(),
                output_hash,
                enclave_signature,
                completed_at: timestamp,
            };
            ComputeResults::<T>::insert(request_id, result);

            // 从待处理列表移除
            PendingRequests::<T>::mutate(|pending| {
                pending.retain(|&id| id != request_id);
            });

            // 清除节点当前请求
            NodeCurrentRequest::<T>::remove(&who);

            // 更新节点统计
            NodeStats::<T>::mutate(&who, |maybe_stats| {
                let stats = maybe_stats.get_or_insert_with(NodeStatistics::default);
                stats.completed_requests = stats.completed_requests.saturating_add(1);
                let current_block: u64 = frame_system::Pallet::<T>::block_number()
                    .try_into()
                    .ok()
                    .unwrap_or(0);
                stats.last_active_block = current_block;
            });

            // 触发事件
            Self::deposit_event(Event::ComputeResultSubmitted {
                request_id,
                executor: who,
                output_hash,
            });

            Ok(())
        }

        /// 取消计算请求
        ///
        /// 请求者可以取消待处理的请求。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是请求者）
        /// - `request_id`: 请求 ID
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::cancel_compute_request())]
        pub fn cancel_compute_request(
            origin: OriginFor<T>,
            request_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取并验证请求
            let assigned_node = ComputeRequests::<T>::try_mutate(request_id, |maybe_request| {
                let request = maybe_request.as_mut().ok_or(Error::<T>::RequestNotFound)?;

                // 验证是请求者
                ensure!(request.requester == who, Error::<T>::NotRequester);

                // 验证状态允许取消
                ensure!(
                    matches!(request.status, RequestStatus::Pending | RequestStatus::Processing),
                    Error::<T>::RequestAlreadyCompleted
                );

                // 更新状态
                request.status = RequestStatus::Failed;
                request.failure_reason = Some(FailureReason::Unknown);

                Ok::<Option<T::AccountId>, Error<T>>(request.assigned_node.clone())
            })?;

            // 从待处理列表移除
            PendingRequests::<T>::mutate(|pending| {
                pending.retain(|&id| id != request_id);
            });

            // 清除节点当前请求
            if let Some(node) = assigned_node {
                NodeCurrentRequest::<T>::remove(&node);
            }

            // 触发事件
            Self::deposit_event(Event::ComputeRequestCancelled {
                request_id,
                requester: who,
            });

            Ok(())
        }

        // ====================================================================
        // 经济激励
        // ====================================================================

        /// 质押
        ///
        /// TEE 节点运营者质押代币以参与计算服务。
        ///
        /// # 参数
        /// - `origin`: 交易发起者
        /// - `amount`: 质押金额
        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::stake())]
        pub fn stake(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证节点已注册
            ensure!(TeeNodes::<T>::contains_key(&who), Error::<T>::NodeNotRegistered);

            // 验证金额达到最低要求
            ensure!(amount >= T::MinimumStake::get(), Error::<T>::InsufficientStake);

            // 锁定代币
            T::Currency::reserve(&who, amount).map_err(|_| Error::<T>::InsufficientBalance)?;

            // 更新质押信息
            let current_block = frame_system::Pallet::<T>::block_number();
            NodeStakes::<T>::mutate(&who, |maybe_stake| {
                let stake = maybe_stake.get_or_insert_with(|| StakeInfo {
                    amount: BalanceOf::<T>::default(),
                    staked_at: current_block,
                    unlock_at: None,
                    is_unbonding: false,
                });
                stake.amount = stake.amount.saturating_add(amount);
                stake.is_unbonding = false;
                stake.unlock_at = None;
            });

            // 触发事件
            Self::deposit_event(Event::Staked {
                account: who,
                amount,
            });

            Ok(())
        }

        /// 申请解除质押
        ///
        /// TEE 节点运营者申请解除质押，需要等待解锁期。
        #[pallet::call_index(8)]
        #[pallet::weight(<T as Config>::WeightInfo::request_unstake())]
        pub fn request_unstake(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let (amount, unlock_at) = NodeStakes::<T>::try_mutate(&who, |maybe_stake| {
                let stake = maybe_stake.as_mut().ok_or(Error::<T>::NotStaked)?;

                ensure!(!stake.is_unbonding, Error::<T>::AlreadyUnbonding);

                let current_block = frame_system::Pallet::<T>::block_number();
                // 解锁期：约 7 天
                let unlock_period: BlockNumberFor<T> = (7 * 24 * 600u32).into(); // 假设 6 秒一个区块
                let unlock_at = current_block.saturating_add(unlock_period);

                stake.is_unbonding = true;
                stake.unlock_at = Some(unlock_at);

                Ok::<(BalanceOf<T>, BlockNumberFor<T>), Error<T>>((stake.amount, unlock_at))
            })?;

            // 触发事件
            Self::deposit_event(Event::UnstakeRequested {
                account: who,
                amount,
                unlock_at,
            });

            Ok(())
        }

        /// 提取解除质押的金额
        ///
        /// 解锁期过后，提取解除质押的代币。
        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::withdraw_unstaked())]
        pub fn withdraw_unstaked(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let amount = NodeStakes::<T>::try_mutate_exists(&who, |maybe_stake| {
                let stake = maybe_stake.as_mut().ok_or(Error::<T>::NotStaked)?;

                ensure!(stake.is_unbonding, Error::<T>::NotUnbonding);

                let current_block = frame_system::Pallet::<T>::block_number();
                let unlock_at = stake.unlock_at.ok_or(Error::<T>::NotUnbonding)?;
                ensure!(current_block >= unlock_at, Error::<T>::UnlockTimeNotReached);

                let amount = stake.amount;

                // 解锁代币
                T::Currency::unreserve(&who, amount);

                // 删除质押记录
                *maybe_stake = None;

                Ok::<BalanceOf<T>, Error<T>>(amount)
            })?;

            // 触发事件
            Self::deposit_event(Event::Withdrawn {
                account: who,
                amount,
            });

            Ok(())
        }

        // ====================================================================
        // 批处理优化
        // ====================================================================

        /// 批量提交计算请求
        ///
        /// 允许用户一次提交多个计算请求，提高效率并减少交易费用。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（请求者）
        /// - `requests`: 批量请求项列表
        ///
        /// # 错误
        /// - `EmptyBatch`: 批处理为空
        /// - `BatchSizeExceeded`: 批处理数量超出限制
        /// - `NoAvailableNodes`: 无可用 TEE 节点
        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::submit_batch_compute_requests(requests.len() as u32))]
        pub fn submit_batch_compute_requests(
            origin: OriginFor<T>,
            requests: Vec<BatchRequestItem>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证批处理大小
            ensure!(!requests.is_empty(), Error::<T>::EmptyBatch);
            ensure!(
                requests.len() as u32 <= T::MaxBatchSize::get(),
                Error::<T>::BatchSizeExceeded
            );

            // 获取可用节点（为整个批次分配同一节点以提高效率）
            let node = Self::select_random_node();
            ensure!(node.is_some(), Error::<T>::NoAvailableNodes);

            let current_block = frame_system::Pallet::<T>::block_number();
            let timeout_blocks: BlockNumberFor<T> = T::RequestTimeout::get().into();
            let timeout_at = current_block.saturating_add(timeout_blocks);

            let mut request_ids = Vec::with_capacity(requests.len());

            for item in requests.iter() {
                // 生成请求 ID
                let request_id = NextRequestId::<T>::get();
                NextRequestId::<T>::put(request_id.saturating_add(1));

                // 创建请求信息
                let request = ComputeRequestInfo {
                    id: request_id,
                    requester: who.clone(),
                    compute_type_id: item.compute_type_id,
                    input_hash: item.input_hash,
                    assigned_node: node.clone(),
                    created_at: current_block,
                    timeout_at,
                    status: RequestStatus::Processing,
                    failover_count: 0,
                    failure_reason: None,
                };

                // 存储请求
                ComputeRequests::<T>::insert(request_id, request);

                // 添加到待处理列表（忽略满的情况，批处理优先保证部分成功）
                let _ = PendingRequests::<T>::try_mutate(|pending| {
                    pending.try_push(request_id)
                });

                request_ids.push(request_id);
            }

            // 记录节点当前处理的请求（使用最后一个请求 ID）
            if let (Some(ref n), Some(&last_id)) = (&node, request_ids.last()) {
                NodeCurrentRequest::<T>::insert(n, last_id);
            }

            let count = request_ids.len() as u32;

            // 触发事件
            Self::deposit_event(Event::BatchComputeRequestsSubmitted {
                requester: who,
                request_ids,
                count,
            });

            Ok(())
        }

        /// 批量提交计算结果
        ///
        /// 允许 TEE 节点一次提交多个计算结果，提高效率。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是分配的 TEE 节点）
        /// - `results`: 批量结果项列表
        ///
        /// # 错误
        /// - `EmptyBatch`: 批处理为空
        /// - `BatchSizeExceeded`: 批处理数量超出限制
        /// - `NotAssignedNode`: 非分配节点
        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::submit_batch_compute_results(results.len() as u32))]
        pub fn submit_batch_compute_results(
            origin: OriginFor<T>,
            results: Vec<BatchResultItem>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证批处理大小
            ensure!(!results.is_empty(), Error::<T>::EmptyBatch);
            ensure!(
                results.len() as u32 <= T::MaxBatchSize::get(),
                Error::<T>::BatchSizeExceeded
            );

            let now = pallet_timestamp::Pallet::<T>::get();
            let timestamp: u64 = now.try_into().ok().unwrap_or(0);

            let mut completed_ids = Vec::with_capacity(results.len());

            for item in results.iter() {
                // 验证请求存在且是分配给该节点的
                let request_result = ComputeRequests::<T>::try_mutate(item.request_id, |maybe_request| {
                    let request = maybe_request.as_mut().ok_or(Error::<T>::RequestNotFound)?;

                    // 验证状态
                    if request.status != RequestStatus::Processing {
                        return Err(Error::<T>::RequestAlreadyCompleted);
                    }

                    // 验证是分配的节点
                    if request.assigned_node.as_ref() != Some(&who) {
                        return Err(Error::<T>::NotAssignedNode);
                    }

                    // 更新请求状态
                    request.status = RequestStatus::Completed;

                    Ok::<(), Error<T>>(())
                });

                // 跳过无效的请求，继续处理其他
                if request_result.is_err() {
                    continue;
                }

                // 存储结果
                let result = ComputeResultInfo {
                    request_id: item.request_id,
                    executor: who.clone(),
                    output_hash: item.output_hash,
                    enclave_signature: item.enclave_signature,
                    completed_at: timestamp,
                };
                ComputeResults::<T>::insert(item.request_id, result);

                // 从待处理列表移除
                PendingRequests::<T>::mutate(|pending| {
                    pending.retain(|&id| id != item.request_id);
                });

                completed_ids.push(item.request_id);
            }

            // 清除节点当前请求
            NodeCurrentRequest::<T>::remove(&who);

            // 更新节点统计
            let completed_count = completed_ids.len() as u64;
            if completed_count > 0 {
                NodeStats::<T>::mutate(&who, |maybe_stats| {
                    let stats = maybe_stats.get_or_insert_with(NodeStatistics::default);
                    stats.completed_requests = stats.completed_requests.saturating_add(completed_count);
                    let current_block: u64 = frame_system::Pallet::<T>::block_number()
                        .try_into()
                        .ok()
                        .unwrap_or(0);
                    stats.last_active_block = current_block;
                });
            }

            let count = completed_ids.len() as u32;

            // 触发事件
            Self::deposit_event(Event::BatchComputeResultsSubmitted {
                executor: who,
                request_ids: completed_ids,
                count,
            });

            Ok(())
        }

        // ====================================================================
        // Enclave 升级机制 (治理功能)
        // ====================================================================

        /// 添加允许的 MRENCLAVE
        ///
        /// 治理机制可以通过此函数添加新的可信 Enclave 度量值。
        /// 这用于 Enclave 升级场景。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是 Root 权限）
        /// - `mr_enclave`: 新的 MRENCLAVE 值（32字节）
        ///
        /// # 错误
        /// - `AllowedListFull`: 允许列表已满
        /// - `EntryAlreadyExists`: 条目已存在
        #[pallet::call_index(12)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn add_allowed_mr_enclave(
            origin: OriginFor<T>,
            mr_enclave: [u8; 32],
        ) -> DispatchResult {
            ensure_root(origin)?;

            AllowedMrEnclaves::<T>::try_mutate(|enclaves| {
                // 检查是否已存在
                ensure!(!enclaves.contains(&mr_enclave), Error::<T>::EntryAlreadyExists);

                // 添加新条目
                enclaves
                    .try_push(mr_enclave)
                    .map_err(|_| Error::<T>::AllowedListFull)?;

                Ok::<(), Error<T>>(())
            })?;

            // 触发事件
            Self::deposit_event(Event::MrEnclaveAllowed { mr_enclave });

            Ok(())
        }

        /// 移除允许的 MRENCLAVE
        ///
        /// 治理机制可以通过此函数移除不再信任的 Enclave 度量值。
        /// 这用于废弃旧版本 Enclave。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是 Root 权限）
        /// - `mr_enclave`: 要移除的 MRENCLAVE 值
        ///
        /// # 错误
        /// - `EntryNotFound`: 条目不存在
        #[pallet::call_index(13)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn remove_allowed_mr_enclave(
            origin: OriginFor<T>,
            mr_enclave: [u8; 32],
        ) -> DispatchResult {
            ensure_root(origin)?;

            AllowedMrEnclaves::<T>::try_mutate(|enclaves| {
                // 查找并移除
                let pos = enclaves
                    .iter()
                    .position(|x| x == &mr_enclave)
                    .ok_or(Error::<T>::EntryNotFound)?;

                enclaves.remove(pos);

                Ok::<(), Error<T>>(())
            })?;

            // 触发事件
            Self::deposit_event(Event::MrEnclaveDisallowed { mr_enclave });

            Ok(())
        }

        /// 添加允许的 MRSIGNER
        ///
        /// 治理机制可以通过此函数添加新的可信签名者度量值。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是 Root 权限）
        /// - `mr_signer`: 新的 MRSIGNER 值（32字节）
        ///
        /// # 错误
        /// - `AllowedListFull`: 允许列表已满
        /// - `EntryAlreadyExists`: 条目已存在
        #[pallet::call_index(14)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn add_allowed_mr_signer(
            origin: OriginFor<T>,
            mr_signer: [u8; 32],
        ) -> DispatchResult {
            ensure_root(origin)?;

            AllowedMrSigners::<T>::try_mutate(|signers| {
                // 检查是否已存在
                ensure!(!signers.contains(&mr_signer), Error::<T>::EntryAlreadyExists);

                // 添加新条目
                signers
                    .try_push(mr_signer)
                    .map_err(|_| Error::<T>::AllowedListFull)?;

                Ok::<(), Error<T>>(())
            })?;

            // 触发事件
            Self::deposit_event(Event::MrSignerAllowed { mr_signer });

            Ok(())
        }

        /// 移除允许的 MRSIGNER
        ///
        /// 治理机制可以通过此函数移除不再信任的签名者度量值。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是 Root 权限）
        /// - `mr_signer`: 要移除的 MRSIGNER 值
        ///
        /// # 错误
        /// - `EntryNotFound`: 条目不存在
        #[pallet::call_index(15)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn remove_allowed_mr_signer(
            origin: OriginFor<T>,
            mr_signer: [u8; 32],
        ) -> DispatchResult {
            ensure_root(origin)?;

            AllowedMrSigners::<T>::try_mutate(|signers| {
                // 查找并移除
                let pos = signers
                    .iter()
                    .position(|x| x == &mr_signer)
                    .ok_or(Error::<T>::EntryNotFound)?;

                signers.remove(pos);

                Ok::<(), Error<T>>(())
            })?;

            // 触发事件
            Self::deposit_event(Event::MrSignerDisallowed { mr_signer });

            Ok(())
        }

        // ====================================================================
        // 审计日志管理
        // ====================================================================

        /// 启用或禁用审计日志
        ///
        /// 治理机制可以通过此函数控制审计日志功能。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是 Root 权限）
        /// - `enabled`: 是否启用审计日志
        #[pallet::call_index(16)]
        #[pallet::weight(Weight::from_parts(10_000_000, 0))]
        pub fn set_audit_enabled(
            origin: OriginFor<T>,
            enabled: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;

            AuditEnabled::<T>::put(enabled);

            // 触发事件
            Self::deposit_event(Event::AuditStatusChanged { enabled });

            Ok(())
        }
    }

    // ========================================================================
    // Hooks
    // ========================================================================

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// 区块结束时处理超时请求
        fn on_finalize(block_number: BlockNumberFor<T>) {
            Self::process_timeout_requests(block_number);
        }
    }

    // ========================================================================
    // 内部函数
    // ========================================================================

    impl<T: Config> Pallet<T> {
        /// 验证远程认证报告
        ///
        /// 检查认证报告的有效性，包括：
        /// 1. MRENCLAVE 是否在允许列表中
        /// 2. MRSIGNER 是否在允许列表中（如果列表非空）
        /// 3. 认证时间戳是否有效
        ///
        /// # 参数
        /// - `attestation`: 远程认证报告
        ///
        /// # 返回
        /// - `Ok(())`: 验证通过
        /// - `Err(Error)`: 验证失败
        pub fn verify_attestation(attestation: &TeeAttestation) -> DispatchResult {
            // 1. 检查 MRENCLAVE 是否在允许列表中
            let allowed_enclaves = AllowedMrEnclaves::<T>::get();
            if !allowed_enclaves.is_empty() {
                ensure!(
                    allowed_enclaves.contains(&attestation.mr_enclave),
                    Error::<T>::MrEnclaveNotAllowed
                );
            }

            // 2. 检查 MRSIGNER 是否在允许列表中（如果列表非空）
            let allowed_signers = AllowedMrSigners::<T>::get();
            if !allowed_signers.is_empty() {
                ensure!(
                    allowed_signers.contains(&attestation.mr_signer),
                    Error::<T>::MrSignerNotAllowed
                );
            }

            // 3. 检查认证时间戳（不能是未来时间，且不能太旧）
            let now = pallet_timestamp::Pallet::<T>::get();
            let current_timestamp: u64 = now.try_into().ok().unwrap_or(0);

            // 认证时间戳不能是未来时间（允许 1 分钟误差）
            ensure!(
                attestation.timestamp <= current_timestamp.saturating_add(60),
                Error::<T>::InvalidAttestationTimestamp
            );

            // 认证不能太旧（默认 24 小时 = 86400 秒）
            let max_age: u64 = 86400;
            ensure!(
                attestation.timestamp >= current_timestamp.saturating_sub(max_age),
                Error::<T>::AttestationExpired
            );

            // TODO: Phase 4 将实现完整的 IAS 签名验证
            // Self::verify_ias_signature(attestation)?;

            Ok(())
        }

        /// 检查节点是否活跃
        ///
        /// 节点活跃条件：
        /// 1. 节点已注册
        /// 2. 状态为 Active
        /// 3. 认证未过期
        pub fn is_node_active(account: &T::AccountId) -> bool {
            if let Some(node) = TeeNodes::<T>::get(account) {
                if node.status != TeeNodeStatus::Active {
                    return false;
                }

                // 检查认证是否过期
                let now = pallet_timestamp::Pallet::<T>::get();
                let current_timestamp: u64 = now.try_into().ok().unwrap_or(0);
                let max_age: u64 = 86400; // 24 小时

                node.attestation.timestamp >= current_timestamp.saturating_sub(max_age)
            } else {
                false
            }
        }

        /// 获取活跃节点数量
        pub fn active_node_count() -> u32 {
            ActiveNodes::<T>::get().len() as u32
        }

        /// 获取节点 Enclave 公钥
        pub fn get_enclave_pubkey(account: &T::AccountId) -> Option<[u8; 32]> {
            TeeNodes::<T>::get(account).map(|node| node.enclave_pubkey)
        }

        /// 获取节点状态
        pub fn get_node_status(account: &T::AccountId) -> Option<TeeNodeStatus> {
            TeeNodes::<T>::get(account).map(|node| node.status)
        }

        /// 随机选择一个活跃节点
        ///
        /// 使用区块哈希作为随机源选择节点
        pub fn select_random_node() -> Option<T::AccountId> {
            let nodes = ActiveNodes::<T>::get();
            if nodes.is_empty() {
                return None;
            }

            // 使用当前区块号作为简单随机源
            let block_number = frame_system::Pallet::<T>::block_number();
            let index = block_number
                .try_into()
                .unwrap_or(0usize)
                .checked_rem(nodes.len())
                .unwrap_or(0);

            nodes.get(index).cloned()
        }

        /// 处理超时请求
        ///
        /// 在 on_finalize 中调用，检查并处理超时的请求
        pub fn process_timeout_requests(current_block: BlockNumberFor<T>) {
            let pending = PendingRequests::<T>::get();
            let mut requests_to_remove = Vec::new();

            for &request_id in pending.iter() {
                if let Some(mut request) = ComputeRequests::<T>::get(request_id) {
                    // 检查是否超时
                    if current_block >= request.timeout_at {
                        let old_node = request.assigned_node.clone();

                        // 清除旧节点的当前请求
                        if let Some(ref node) = old_node {
                            NodeCurrentRequest::<T>::remove(node);

                            // 更新节点统计
                            NodeStats::<T>::mutate(node, |maybe_stats| {
                                let stats = maybe_stats.get_or_insert_with(NodeStatistics::default);
                                stats.timeout_requests = stats.timeout_requests.saturating_add(1);
                            });

                            // 惩罚节点
                            Self::slash_node(node, FailureReason::Timeout);
                        }

                        // 检查是否可以故障转移
                        if request.failover_count < MAX_FAILOVER_COUNT {
                            // 尝试分配新节点
                            let new_node = Self::select_random_node_excluding(old_node.as_ref());

                            if let Some(ref new_n) = new_node {
                                // 更新请求
                                request.assigned_node = new_node.clone();
                                request.failover_count = request.failover_count.saturating_add(1);
                                let timeout_blocks: BlockNumberFor<T> = T::RequestTimeout::get().into();
                                request.timeout_at = current_block.saturating_add(timeout_blocks);

                                // 保存failover_count用于事件
                                let failover_count = request.failover_count;

                                ComputeRequests::<T>::insert(request_id, request);

                                // 记录新节点的当前请求
                                NodeCurrentRequest::<T>::insert(new_n, request_id);

                                // 触发事件
                                Self::deposit_event(Event::RequestTimeout {
                                    request_id,
                                    old_node: old_node.unwrap_or_else(|| new_n.clone()),
                                    new_node: Some(new_n.clone()),
                                    failover_count,
                                });
                            } else {
                                // 没有可用节点，标记失败
                                request.status = RequestStatus::Failed;
                                request.failure_reason = Some(FailureReason::NodeFailure);
                                ComputeRequests::<T>::insert(request_id, request);
                                requests_to_remove.push(request_id);

                                Self::deposit_event(Event::RequestFailed {
                                    request_id,
                                    reason: FailureReason::NodeFailure,
                                });
                            }
                        } else {
                            // 达到最大故障转移次数，标记失败
                            request.status = RequestStatus::Failed;
                            request.failure_reason = Some(FailureReason::Timeout);
                            ComputeRequests::<T>::insert(request_id, request);
                            requests_to_remove.push(request_id);

                            Self::deposit_event(Event::RequestFailed {
                                request_id,
                                reason: FailureReason::Timeout,
                            });
                        }
                    }
                }
            }

            // 从待处理列表中移除失败的请求
            if !requests_to_remove.is_empty() {
                PendingRequests::<T>::mutate(|pending| {
                    pending.retain(|&id| !requests_to_remove.contains(&id));
                });
            }
        }

        /// 选择一个活跃节点（排除指定节点）
        fn select_random_node_excluding(exclude: Option<&T::AccountId>) -> Option<T::AccountId> {
            let nodes = ActiveNodes::<T>::get();
            let filtered: Vec<_> = nodes
                .iter()
                .filter(|n| exclude.map_or(true, |e| *n != e))
                .filter(|n| Self::is_node_active(n))
                .cloned()
                .collect();

            if filtered.is_empty() {
                return None;
            }

            let block_number = frame_system::Pallet::<T>::block_number();
            let index = block_number
                .try_into()
                .unwrap_or(0usize)
                .checked_rem(filtered.len())
                .unwrap_or(0);

            filtered.get(index).cloned()
        }

        /// 惩罚节点
        ///
        /// 根据配置的惩罚比例扣除质押
        fn slash_node(account: &T::AccountId, reason: FailureReason) {
            if let Some(mut stake_info) = NodeStakes::<T>::get(account) {
                // 计算惩罚金额 (千分比)
                let slash_ratio = T::SlashRatio::get();
                // slash_amount = amount * slash_ratio / 1000
                let slash_amount = stake_info
                    .amount
                    .saturating_mul(slash_ratio.into())
                    .checked_div(&1000u32.into())
                    .unwrap_or_else(Zero::zero);

                if !slash_amount.is_zero() {
                    // 从质押中扣除
                    stake_info.amount = stake_info.amount.saturating_sub(slash_amount);
                    NodeStakes::<T>::insert(account, stake_info);

                    // 从保留余额中扣除（实际销毁）
                    let _ = T::Currency::slash_reserved(account, slash_amount);

                    // 更新累计惩罚
                    TotalSlashed::<T>::mutate(|total| {
                        *total = total.saturating_add(slash_amount);
                    });

                    // 更新节点统计
                    NodeStats::<T>::mutate(account, |maybe_stats| {
                        let stats = maybe_stats.get_or_insert_with(NodeStatistics::default);
                        stats.slash_count = stats.slash_count.saturating_add(1);
                        let slash_u128: u128 = slash_amount.try_into().ok().unwrap_or(0);
                        stats.total_slashed = stats.total_slashed.saturating_add(slash_u128);
                    });

                    // 触发事件
                    Self::deposit_event(Event::Slashed {
                        account: account.clone(),
                        amount: slash_amount,
                        reason,
                    });
                }
            }
        }

        // ====================================================================
        // 审计日志辅助函数
        // ====================================================================

        /// 记录审计日志
        ///
        /// 如果审计功能已启用，则记录审计日志条目
        pub fn log_audit_event(
            event_type: AuditEventType,
            account: &T::AccountId,
            data_hash: [u8; 32],
            success: bool,
        ) {
            // 检查审计功能是否启用
            if !AuditEnabled::<T>::get() {
                return;
            }

            // 生成日志 ID
            let log_id = NextAuditLogId::<T>::get();
            NextAuditLogId::<T>::put(log_id.saturating_add(1));

            // 获取当前区块和时间戳
            let block_number = frame_system::Pallet::<T>::block_number();
            let now = pallet_timestamp::Pallet::<T>::get();
            let timestamp: u64 = now.try_into().ok().unwrap_or(0);

            // 创建日志条目
            let entry = crate::types::AuditLogEntry {
                id: log_id,
                event_type,
                account: account.clone(),
                block_number,
                timestamp,
                data_hash,
                success,
            };

            // 存储日志
            AuditLogs::<T>::insert(log_id, entry);

            // 更新账户索引（保留最近 100 条）
            AccountAuditLogs::<T>::mutate(account, |logs| {
                // 如果满了，移除最旧的
                if logs.len() >= 100 {
                    logs.remove(0);
                }
                let _ = logs.try_push(log_id);
            });
        }

        /// 获取账户的审计日志
        pub fn get_account_audit_logs(account: &T::AccountId) -> Vec<crate::types::AuditLogEntry<T::AccountId, BlockNumberFor<T>>> {
            let log_ids = AccountAuditLogs::<T>::get(account);
            log_ids
                .iter()
                .filter_map(|&id| AuditLogs::<T>::get(id))
                .collect()
        }

        /// 获取指定范围的审计日志
        pub fn get_audit_logs_range(start_id: u64, count: u32) -> Vec<crate::types::AuditLogEntry<T::AccountId, BlockNumberFor<T>>> {
            let mut logs = Vec::new();
            for id in start_id..start_id.saturating_add(count as u64) {
                if let Some(entry) = AuditLogs::<T>::get(id) {
                    logs.push(entry);
                }
            }
            logs
        }
    }
}
