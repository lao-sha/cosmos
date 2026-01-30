//! # OCW + TEE 通用架构模块 (pallet-divination-ocw-tee)
//!
//! 本模块为所有占卜模块提供统一的 OCW + TEE 隐私计算基础设施。
//!
//! ## 功能概述
//!
//! 1. **请求管理**：统一的待处理请求队列
//! 2. **OCW 调度**：自动处理待处理请求
//! 3. **TEE 通信**：与 TEE 节点的安全通信
//! 4. **IPFS 存储**：JSON 清单的分布式存储
//! 5. **重试机制**：失败请求的自动重试
//! 6. **模块注册**：插件化的占卜模块注册
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                        应用层（各占卜模块）                               │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐           │
//! │  │  BaZi   │ │  Qimen  │ │ MeiHua  │ │ LiuYao  │ │  ZiWei  │  ...      │
//! │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘           │
//! │       │           │           │           │           │                 │
//! │       └───────────┴───────────┴───────────┴───────────┘                 │
//! │                               │                                          │
//! │                               ▼                                          │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                      通用层（pallet-divination-ocw-tee）                 │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
//! │  │ 请求管理    │  │ OCW 调度    │  │ TEE 通信    │  │ IPFS 存储   │    │
//! │  │ (Pending)   │  │ (Scheduler) │  │ (Client)    │  │ (Uploader)  │    │
//! │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘    │
//! │                                                                          │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
//! │  │ 隐私模式    │  │ 模块注册    │  │ 重试机制    │  │ 事件通知    │    │
//! │  │ (Privacy)   │  │ (Registry)  │  │ (Retry)     │  │ (Events)    │    │
//! │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘    │
//! │                                                                          │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ### 1. 创建公开模式请求
//! ```ignore
//! OcwTee::create_public_request(
//!     origin,
//!     DivinationType::BaZi,
//!     input_data,
//! )?;
//! ```
//!
//! ### 2. 创建加密模式请求
//! ```ignore
//! OcwTee::create_encrypted_request(
//!     origin,
//!     DivinationType::BaZi,
//!     encrypted_input,
//!     user_pubkey,
//!     PrivacyMode::Encrypted,
//! )?;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod crypto;
pub mod traits;
pub mod types;
pub mod registry;

#[cfg(test)]
mod tests;

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

pub use traits::*;
pub use types::*;
pub use registry::*;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
// use sp_runtime::traits::Saturating; // 暂未使用
use sp_std::prelude::*;

/// 最大重试次数
pub const MAX_RETRY_COUNT: u8 = 3;

/// OCW 处理间隔（区块数）
pub const DEFAULT_OCW_INTERVAL: u32 = 5;

/// 每区块最大处理请求数
pub const MAX_REQUESTS_PER_BLOCK: u32 = 10;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use pallet_storage_service::IpfsPinner;

    // ========================================================================
    // Pallet 配置
    // ========================================================================

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Pallet 配置 trait
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 运行时事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// OCW 处理间隔（区块数）
        #[pallet::constant]
        type OcwInterval: Get<u32>;

        /// 最大重试次数
        #[pallet::constant]
        type MaxRetryCount: Get<u8>;

        /// 每区块最大处理请求数
        #[pallet::constant]
        type MaxRequestsPerBlock: Get<u32>;

        /// 最大输入数据长度
        #[pallet::constant]
        type MaxInputDataLen: Get<u32>;

        /// TEE 客户端
        type TeeClient: TeeClient;

        /// IPFS 客户端
        type IpfsClient: IpfsClient;

        /// TEE 节点管理（兼容旧接口）
        type TeeNodeManager: TeeNodeManager<Self::AccountId>;

        /// TEE Privacy 集成（深度集成接口）
        /// 
        /// 通过此接口与 pallet-tee-privacy 交互：
        /// - 请求队列管理
        /// - TEE 节点获取
        /// - 结果提交与奖励
        type TeePrivacy: TeePrivacyIntegration<Self::AccountId, BlockNumberFor<Self>>;

        /// IPFS Pin 服务提供者（Phase 15 集成）
        /// 
        /// 通过此接口与 pallet-cosmos-ipfs 交互：
        /// - 自动 Pin 加密数据到 IPFS
        /// - 使用三级扣费机制（IpfsPool → SubjectFunding → Grace）
        type IpfsPinner: pallet_storage_service::IpfsPinner<Self::AccountId, u128>;

        /// 模块注册表（Public 模式处理）
        /// 
        /// 通过此接口调用各占卜模块的计算逻辑：
        /// - 解码输入数据
        /// - 执行计算
        /// - 生成 JSON 清单
        type ModuleRegistry: ModuleRegistry;
    }

    // ========================================================================
    // 存储定义
    // ========================================================================
    //
    // 注意：请求队列由 pallet-tee-privacy 统一管理
    // 本模块仅存储占卜结果相关数据

    /// 用户的请求索引（本地缓存，主数据在 tee-privacy）
    #[pallet::storage]
    #[pallet::getter(fn user_requests)]
    pub type UserRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<100>>,
        ValueQuery,
    >;

    /// 已完成的占卜结果存储
    /// 
    /// 存储完整的占卜结果，包括 IPFS CID、隐私模式、索引等
    #[pallet::storage]
    #[pallet::getter(fn completed_results)]
    pub type CompletedResults<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        DivinationOnChain<T::AccountId, BlockNumberFor<T>, BoundedVec<u8, ConstU32<256>>>,
    >;

    /// 请求到占卜类型的映射
    /// 
    /// 记录每个请求对应的占卜类型和隐私模式
    #[pallet::storage]
    #[pallet::getter(fn request_metadata)]
    pub type RequestMetadata<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        (DivinationType, PrivacyMode),
    >;

    /// 请求的加密输入数据（临时存储）
    /// 
    /// OCW 处理完成后删除
    #[pallet::storage]
    pub type RequestInputData<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        InputData<T::MaxInputDataLen>,
    >;

    /// 请求的用户公钥（用于加密返回结果）
    #[pallet::storage]
    pub type RequestUserPubkey<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        [u8; 32],
    >;

    /// 请求的版本信息（临时存储，用于创建过程）
    #[pallet::storage]
    pub type RequestVersionInfo<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        VersionInfo,
    >;

    // ========== 版本控制存储 ==========

    /// 版本链索引：首版 request_id -> 所有版本的 request_id 列表
    #[pallet::storage]
    #[pallet::getter(fn version_chain)]
    pub type VersionChain<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // first_version_id
        BoundedVec<u64, ConstU32<100>>,  // 所有版本的 request_id
        ValueQuery,
    >;

    /// 最新版本索引：首版 request_id -> 最新版本的 request_id
    #[pallet::storage]
    #[pallet::getter(fn latest_version)]
    pub type LatestVersion<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // first_version_id
        u64,  // latest request_id
    >;

    // ========================================================================
    // 事件定义
    // ========================================================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 请求已创建
        RequestCreated {
            request_id: u64,
            requester: T::AccountId,
            divination_type: DivinationType,
            privacy_mode: PrivacyMode,
        },

        /// 请求处理中
        RequestProcessing {
            request_id: u64,
            assigned_node: Option<T::AccountId>,
        },

        /// 请求已完成
        RequestCompleted {
            request_id: u64,
            manifest_cid: Vec<u8>,
        },

        /// 请求失败
        RequestFailed {
            request_id: u64,
            reason: Vec<u8>,
            retry_count: u8,
        },

        /// 请求超时
        RequestTimeout { request_id: u64 },

        /// 请求重试
        RequestRetry {
            request_id: u64,
            retry_count: u8,
        },

        /// 结果已存储
        ResultStored {
            request_id: u64,
            owner: T::AccountId,
            divination_type: DivinationType,
            manifest_cid: Vec<u8>,
        },

        /// 请求已取消
        RequestCancelled {
            request_id: u64,
            owner: T::AccountId,
        },

        /// 占卜已更新（新版本创建）
        DivinationUpdated {
            /// 首版请求 ID
            first_version_id: u64,
            /// 新版本请求 ID
            new_request_id: u64,
            /// 新版本号
            version: u32,
            /// 所有者
            owner: T::AccountId,
        },
    }

    // ========================================================================
    // 错误定义
    // ========================================================================

    #[pallet::error]
    pub enum Error<T> {
        /// 请求不存在
        RequestNotFound,

        /// 请求已处理
        RequestAlreadyProcessed,

        /// 待处理请求队列已满
        PendingQueueFull,

        /// 输入数据过长
        InputDataTooLong,

        /// 无效的隐私模式
        InvalidPrivacyMode,

        /// 公开模式需要使用专用接口
        UsePublicExtrinsic,

        /// 加密模式需要用户公钥
        UserPubkeyRequired,

        /// TEE 节点不可用
        TeeNodeUnavailable,

        /// 无权限
        Unauthorized,

        /// 用户请求列表已满
        UserRequestListFull,

        /// 结果不存在
        ResultNotFound,

        /// 结果已存在
        ResultAlreadyExists,

        /// CID 过长
        CidTooLong,

        /// 索引过长
        IndexTooLong,

        /// 不是所有者
        NotOwner,

        /// 无法取消（请求不在 Pending 状态）
        CannotCancel,

        /// 版本过多
        TooManyVersions,

        /// 无效的版本 ID
        InvalidVersionId,

        /// PublicEncrypted 模式需要使用专用接口
        UsePublicEncryptedExtrinsic,
    }

    // ========================================================================
    // Hooks
    // ========================================================================

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// 区块结束时清理超时请求
        fn on_finalize(block_number: BlockNumberFor<T>) {
            Self::cleanup_timeout_requests(block_number);
        }

        /// OCW 入口点
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            // 检查处理间隔
            let interval = T::OcwInterval::get();
            let block_num: u32 = block_number
                .try_into()
                .unwrap_or(0u32);

            if block_num % interval != 0 {
                return;
            }

            log::info!(
                "🔮 OCW: Processing pending requests at block {:?}",
                block_number
            );

            // 处理待处理请求
            Self::process_pending_requests(block_number);
        }
    }

    // ========================================================================
    // 交易接口
    // ========================================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建公开模式请求
        ///
        /// 用于 Public 隐私模式，输入数据明文提交。
        ///
        /// # 参数
        /// - `origin`: 交易发起者
        /// - `divination_type`: 占卜类型
        /// - `input_data`: 明文输入数据（编码后）
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn create_public_request(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            input_data: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证输入数据长度
            let bounded_input: BoundedVec<u8, T::MaxInputDataLen> = input_data
                .try_into()
                .map_err(|_| Error::<T>::InputDataTooLong)?;

            // 创建请求
            let request_id = Self::do_create_request(
                who.clone(),
                divination_type,
                InputData::Plaintext(bounded_input),
                None,
                PrivacyMode::Public,
            )?;

            // 触发事件
            Self::deposit_event(Event::RequestCreated {
                request_id,
                requester: who,
                divination_type,
                privacy_mode: PrivacyMode::Public,
            });

            Ok(())
        }

        /// 创建公开计算加密存储模式请求
        ///
        /// 用于 PublicEncrypted 隐私模式：
        /// - 输入数据明文提交（适用于非敏感输入如时间）
        /// - OCW 明文计算（无需 TEE）
        /// - 结果使用用户公钥加密后存储到 IPFS
        ///
        /// # 参数
        /// - `origin`: 交易发起者
        /// - `divination_type`: 占卜类型
        /// - `input_data`: 明文输入数据（编码后）
        /// - `user_pubkey`: 用户 X25519 公钥（用于加密结果）
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(55_000_000, 0))]
        pub fn create_public_encrypted_request(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            input_data: Vec<u8>,
            user_pubkey: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证用户公钥
            ensure!(
                user_pubkey != [0u8; 32],
                Error::<T>::UserPubkeyRequired
            );

            // 验证输入数据长度
            let bounded_input: BoundedVec<u8, T::MaxInputDataLen> = input_data
                .try_into()
                .map_err(|_| Error::<T>::InputDataTooLong)?;

            // 创建请求
            let request_id = Self::do_create_request(
                who.clone(),
                divination_type,
                InputData::Plaintext(bounded_input),
                Some(user_pubkey),  // 提供用户公钥用于加密结果
                PrivacyMode::PublicEncrypted,
            )?;

            // 触发事件
            Self::deposit_event(Event::RequestCreated {
                request_id,
                requester: who,
                divination_type,
                privacy_mode: PrivacyMode::PublicEncrypted,
            });

            Ok(())
        }

        /// 创建加密模式请求
        ///
        /// 用于 Encrypted/Private 隐私模式，输入数据加密提交。
        ///
        /// # 参数
        /// - `origin`: 交易发起者
        /// - `divination_type`: 占卜类型
        /// - `encrypted_input`: 加密的输入数据
        /// - `user_pubkey`: 用户公钥（用于加密返回结果）
        /// - `privacy_mode`: 隐私模式（Encrypted 或 Private）
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(60_000_000, 0))]
        pub fn create_encrypted_request(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            ciphertext: Vec<u8>,
            nonce: [u8; 24],
            sender_pubkey: [u8; 32],
            user_pubkey: [u8; 32],
            privacy_mode: PrivacyMode,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证隐私模式（只允许 Encrypted 和 Private）
            ensure!(
                privacy_mode != PrivacyMode::Public,
                Error::<T>::UsePublicExtrinsic
            );
            ensure!(
                privacy_mode != PrivacyMode::PublicEncrypted,
                Error::<T>::UsePublicEncryptedExtrinsic
            );

            // 验证用户公钥
            ensure!(
                user_pubkey != [0u8; 32],
                Error::<T>::UserPubkeyRequired
            );

            // 构建加密数据
            let bounded_ciphertext: BoundedVec<u8, T::MaxInputDataLen> = ciphertext
                .try_into()
                .map_err(|_| Error::<T>::InputDataTooLong)?;

            let encrypted_data = EncryptedData {
                ciphertext: bounded_ciphertext,
                nonce,
                sender_pubkey,
            };

            // 创建请求
            let request_id = Self::do_create_request(
                who.clone(),
                divination_type,
                InputData::Encrypted(encrypted_data),
                Some(user_pubkey),
                privacy_mode,
            )?;

            // 触发事件
            Self::deposit_event(Event::RequestCreated {
                request_id,
                requester: who,
                divination_type,
                privacy_mode,
            });

            Ok(())
        }

        /// 提交计算结果（OCW 调用）
        ///
        /// OCW 处理完成后调用此接口提交结果。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（OCW）
        /// - `request_id`: 请求 ID
        /// - `manifest_cid`: IPFS CID
        /// - `manifest_hash`: 清单哈希
        /// - `type_index`: 类型特定索引（编码后）
        /// - `proof`: 计算证明（TEE 模式）
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(80_000_000, 0))]
        pub fn submit_result(
            origin: OriginFor<T>,
            request_id: u64,
            manifest_cid: Vec<u8>,
            manifest_hash: [u8; 32],
            type_index: Option<Vec<u8>>,
            proof: Option<ComputationProof>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // 从 TeePrivacyIntegration 获取请求信息
            let (requester, compute_type_id, _input_hash, assigned_node) = 
                T::TeePrivacy::get_request(request_id)
                    .ok_or(Error::<T>::RequestNotFound)?;

            // 验证请求状态
            let status = T::TeePrivacy::get_request_status(request_id)
                .ok_or(Error::<T>::RequestNotFound)?;
            ensure!(
                status == RequestStatus::Processing || status == RequestStatus::Pending,
                Error::<T>::RequestAlreadyProcessed
            );

            // 获取本地元数据
            let (divination_type, privacy_mode) = RequestMetadata::<T>::get(request_id)
                .unwrap_or((DivinationType::from_u8(compute_type_id), PrivacyMode::Public));

            // 存储结果
            Self::do_store_result(
                request_id,
                requester,
                divination_type,
                privacy_mode,
                assigned_node,
                manifest_cid.clone(),
                manifest_hash,
                type_index,
                proof,
            )?;

            // 触发事件
            Self::deposit_event(Event::RequestCompleted {
                request_id,
                manifest_cid,
            });

            Ok(())
        }

        /// 更新请求状态（OCW 调用）
        ///
        /// 通过 TeePrivacyIntegration 分配节点
        #[pallet::call_index(11)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn update_request_status(
            origin: OriginFor<T>,
            request_id: u64,
            _status: RequestStatus,
            assigned_node: Option<T::AccountId>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // 如果指定了节点，通过 TeePrivacyIntegration 分配
            if let Some(node) = assigned_node.clone() {
                T::TeePrivacy::assign_node(request_id, node)?;
            }

            Self::deposit_event(Event::RequestProcessing {
                request_id,
                assigned_node,
            });

            Ok(())
        }

        /// 标记请求失败（OCW 调用）
        #[pallet::call_index(12)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn mark_request_failed(
            origin: OriginFor<T>,
            request_id: u64,
            reason: Vec<u8>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // 通过 TeePrivacyIntegration 标记失败
            T::TeePrivacy::mark_request_failed(request_id, FailureReason::ComputationError)?;

            Self::deposit_event(Event::RequestFailed {
                request_id,
                reason,
                retry_count: 0, // 重试计数由 tee-privacy 管理
            });

            Ok(())
        }

        /// 取消待处理的请求
        ///
        /// 只能取消 Pending 状态的请求
        #[pallet::call_index(20)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn cancel_request(
            origin: OriginFor<T>,
            request_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证是请求所有者
            let (requester, _, _, _) = T::TeePrivacy::get_request(request_id)
                .ok_or(Error::<T>::RequestNotFound)?;
            ensure!(requester == who, Error::<T>::NotOwner);

            // 只能取消 Pending 状态的请求
            let status = T::TeePrivacy::get_request_status(request_id)
                .ok_or(Error::<T>::RequestNotFound)?;
            ensure!(status == RequestStatus::Pending, Error::<T>::CannotCancel);

            // 标记为取消（使用 UserCancelled 原因）
            T::TeePrivacy::mark_request_failed(request_id, FailureReason::UserCancelled)?;

            // 清理本地数据
            RequestInputData::<T>::remove(request_id);
            RequestUserPubkey::<T>::remove(request_id);
            RequestMetadata::<T>::remove(request_id);
            RequestVersionInfo::<T>::remove(request_id);

            Self::deposit_event(Event::RequestCancelled {
                request_id,
                owner: who,
            });

            Ok(())
        }

        /// 更新占卜结果（创建新版本）
        ///
        /// 用于修正错误输入，创建新版本替换旧版本
        #[pallet::call_index(21)]
        #[pallet::weight(Weight::from_parts(100_000_000, 0))]
        pub fn update_divination(
            origin: OriginFor<T>,
            original_request_id: u64,
            divination_type: DivinationType,
            input_data: BoundedVec<u8, T::MaxInputDataLen>,
            user_pubkey: Option<[u8; 32]>,
            privacy_mode: PrivacyMode,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 1. 获取首版 ID
            let first_version_id = Self::get_first_version_id(original_request_id)?;

            // 2. 验证所有权
            let original = CompletedResults::<T>::get(first_version_id)
                .ok_or(Error::<T>::RequestNotFound)?;
            ensure!(original.owner == who, Error::<T>::NotOwner);

            // 3. 获取当前最新版本
            let current_latest_id = LatestVersion::<T>::get(first_version_id)
                .unwrap_or(first_version_id);
            let current_latest = CompletedResults::<T>::get(current_latest_id)
                .ok_or(Error::<T>::RequestNotFound)?;

            // 4. 构建输入数据
            let input = if privacy_mode == PrivacyMode::Public {
                InputData::Plaintext(input_data)
            } else {
                // 加密模式需要用户公钥
                ensure!(user_pubkey.is_some(), Error::<T>::UserPubkeyRequired);
                InputData::Encrypted(EncryptedData {
                    ciphertext: input_data,
                    nonce: [0u8; 24],  // 24 字节，兼容 XSalsa20
                    sender_pubkey: [0u8; 32],
                })
            };

            // 5. 创建新请求
            let new_request_id = Self::do_create_request(
                who.clone(),
                divination_type,
                input,
                user_pubkey,
                privacy_mode,
            )?;

            let new_version = current_latest.version.saturating_add(1);

            // 6. 设置版本信息
            RequestVersionInfo::<T>::insert(new_request_id, VersionInfo {
                first_version_id,
                version: new_version,
                previous_version: Some(current_latest_id),
            });

            // 7. 更新版本链
            VersionChain::<T>::try_mutate(first_version_id, |chain| {
                if chain.is_empty() {
                    // 首次更新，添加首版
                    chain.try_push(first_version_id)
                        .map_err(|_| Error::<T>::TooManyVersions)?;
                }
                chain.try_push(new_request_id)
                    .map_err(|_| Error::<T>::TooManyVersions)
            })?;

            // 8. 更新最新版本索引
            LatestVersion::<T>::insert(first_version_id, new_request_id);

            // 9. 标记旧版本为非最新
            CompletedResults::<T>::mutate(current_latest_id, |maybe_result| {
                if let Some(result) = maybe_result {
                    result.is_latest = false;
                }
            });

            Self::deposit_event(Event::DivinationUpdated {
                first_version_id,
                new_request_id,
                version: new_version,
                owner: who,
            });

            Ok(())
        }
    }

    // ========================================================================
    // 内部方法
    // ========================================================================

    impl<T: Config> Pallet<T> {
        /// 创建请求（内部）
        /// 
        /// 通过 TeePrivacyIntegration 提交请求到 tee-privacy 统一队列
        fn do_create_request(
            requester: T::AccountId,
            divination_type: DivinationType,
            input_data: InputData<T::MaxInputDataLen>,
            user_pubkey: Option<[u8; 32]>,
            privacy_mode: PrivacyMode,
        ) -> Result<u64, DispatchError> {
            // 计算输入数据哈希
            let input_hash = Self::compute_input_hash(&input_data);
            
            // 获取超时时间
            let timeout_blocks = divination_type.recommended_timeout();
            
            // 计算类型 ID
            let compute_type_id = divination_type as u8;

            // 通过 TeePrivacyIntegration 提交请求到 tee-privacy
            let request_id = T::TeePrivacy::submit_request(
                requester.clone(),
                compute_type_id,
                input_hash,
                timeout_blocks,
            )?;

            // 存储本地元数据
            RequestMetadata::<T>::insert(request_id, (divination_type, privacy_mode));
            RequestInputData::<T>::insert(request_id, input_data);
            
            if let Some(pubkey) = user_pubkey {
                RequestUserPubkey::<T>::insert(request_id, pubkey);
            }

            // 添加到用户请求列表
            UserRequests::<T>::try_mutate(&requester, |ids| {
                ids.try_push(request_id)
                    .map_err(|_| Error::<T>::UserRequestListFull)
            })?;

            Ok(request_id)
        }

        /// 计算输入数据哈希
        fn compute_input_hash(input_data: &InputData<T::MaxInputDataLen>) -> [u8; 32] {
            use sp_io::hashing::blake2_256;
            
            let encoded = input_data.encode();
            blake2_256(&encoded)
        }

        /// 存储结果（内部）
        fn do_store_result(
            request_id: u64,
            requester: T::AccountId,
            divination_type: DivinationType,
            privacy_mode: PrivacyMode,
            assigned_node: Option<T::AccountId>,
            manifest_cid: Vec<u8>,
            manifest_hash: [u8; 32],
            type_index: Option<Vec<u8>>,
            proof: Option<ComputationProof>,
        ) -> DispatchResult {
            let current_block = frame_system::Pallet::<T>::block_number();

            // 转换 CID
            let bounded_cid: BoundedVec<u8, ConstU32<64>> = manifest_cid
                .clone()
                .try_into()
                .map_err(|_| Error::<T>::CidTooLong)?;

            // 转换索引
            let bounded_index: Option<BoundedVec<u8, ConstU32<256>>> = type_index
                .map(|idx| idx.try_into().map_err(|_| Error::<T>::IndexTooLong))
                .transpose()?;

            // 构建生成信息
            let generation = match (assigned_node, proof) {
                (Some(node), Some(p)) => GenerationInfo::Tee {
                    node,
                    proof: p,
                },
                _ => GenerationInfo::Ocw,
            };

            // 获取版本信息
            let version_info = RequestVersionInfo::<T>::get(request_id)
                .unwrap_or(VersionInfo {
                    first_version_id: request_id,
                    version: 1,
                    previous_version: None,
                });

            // 创建链上存储
            let result = DivinationOnChain {
                owner: requester.clone(),
                divination_type,
                privacy_mode,
                type_index: bounded_index,
                manifest_cid: bounded_cid,
                manifest_hash,
                generation,
                version: version_info.version,
                first_version_id: version_info.first_version_id,
                previous_version: version_info.previous_version,
                is_latest: true,
                created_at: current_block,
                updated_at: current_block,
            };

            // 存储结果
            CompletedResults::<T>::insert(request_id, result);

            // Phase 15: 自动 Pin 到 IPFS（使用三级扣费机制）
            // 根据隐私模式选择 PinTier
            let pin_tier = match privacy_mode {
                PrivacyMode::Public => pallet_storage_service::PinTier::Temporary,
                PrivacyMode::PublicEncrypted => pallet_storage_service::PinTier::Standard, // 加密存储，使用标准级别
                PrivacyMode::Encrypted => pallet_storage_service::PinTier::Standard,
                PrivacyMode::Private => pallet_storage_service::PinTier::Critical,
            };
            
            // 调用 IpfsPinner 进行 Pin（使用 DivinationReport SubjectType）
            // 费用通过三级扣费机制处理：IpfsPool → SubjectFunding → Grace
            let _ = T::IpfsPinner::pin_cid_for_subject(
                requester.clone(),
                pallet_storage_service::SubjectType::DivinationReport,
                request_id,
                manifest_cid.clone(),
                Some(pin_tier),
            );
            // 注意：Pin 失败不阻塞结果存储，仅记录日志

            // 如果是首版，初始化版本链
            if version_info.version == 1 {
                VersionChain::<T>::try_mutate(request_id, |chain| {
                    chain.try_push(request_id)
                        .map_err(|_| Error::<T>::TooManyVersions)
                })?;
                LatestVersion::<T>::insert(request_id, request_id);
            }

            // 清理临时数据
            RequestInputData::<T>::remove(request_id);
            RequestUserPubkey::<T>::remove(request_id);
            RequestMetadata::<T>::remove(request_id);
            RequestVersionInfo::<T>::remove(request_id);

            // 触发事件
            Self::deposit_event(Event::ResultStored {
                request_id,
                owner: requester,
                divination_type,
                manifest_cid,
            });

            Ok(())
        }

        /// 获取首版请求 ID
        fn get_first_version_id(request_id: u64) -> Result<u64, DispatchError> {
            // 先检查是否有已完成的结果
            if let Some(result) = CompletedResults::<T>::get(request_id) {
                return Ok(result.first_version_id);
            }
            
            // 检查版本信息（可能是待处理的更新请求）
            if let Some(version_info) = RequestVersionInfo::<T>::get(request_id) {
                return Ok(version_info.first_version_id);
            }
            
            // 如果都没有，假设这是首版
            Err(Error::<T>::InvalidVersionId.into())
        }

        /// 获取所有版本列表
        pub fn get_all_versions(first_version_id: u64) -> Vec<u64> {
            VersionChain::<T>::get(first_version_id).into_inner()
        }

        /// 获取版本历史
        pub fn get_version_history(first_version_id: u64) -> Vec<VersionHistoryEntry<BlockNumberFor<T>>> {
            Self::get_all_versions(first_version_id)
                .iter()
                .filter_map(|id| {
                    CompletedResults::<T>::get(id).map(|r| VersionHistoryEntry {
                        request_id: *id,
                        version: r.version,
                        created_at: r.created_at,
                        is_latest: r.is_latest,
                    })
                })
                .collect()
        }

        /// 处理待处理请求（OCW）
        /// 
        /// 从 tee-privacy 获取待处理请求列表
        fn process_pending_requests(_block_number: BlockNumberFor<T>) {
            // 从 TeePrivacyIntegration 获取待处理请求
            let request_ids = T::TeePrivacy::get_pending_requests();
            let max_per_block = T::MaxRequestsPerBlock::get() as usize;

            for (idx, request_id) in request_ids.iter().enumerate() {
                if idx >= max_per_block {
                    break;
                }

                // 检查是否有本地元数据（确认是占卜请求）
                if let Some((divination_type, privacy_mode)) = RequestMetadata::<T>::get(request_id) {
                    // 获取请求状态
                    if let Some(status) = T::TeePrivacy::get_request_status(*request_id) {
                        if status == RequestStatus::Pending {
                            Self::process_single_request(*request_id, divination_type, privacy_mode);
                        }
                    }
                }
            }
        }

        /// 处理单个请求（OCW）
        fn process_single_request(
            request_id: u64,
            divination_type: DivinationType,
            privacy_mode: PrivacyMode,
        ) {
            log::info!(
                "🔮 OCW: Processing request {} (type: {:?}, mode: {:?})",
                request_id,
                divination_type,
                privacy_mode
            );

            // 根据隐私模式处理
            let result = match privacy_mode {
                PrivacyMode::Public => {
                    Self::process_public_request(request_id, divination_type)
                }
                PrivacyMode::PublicEncrypted => {
                    Self::process_public_encrypted_request(request_id, divination_type)
                }
                PrivacyMode::Encrypted | PrivacyMode::Private => {
                    Self::process_tee_request(request_id, divination_type, privacy_mode)
                }
            };

            match result {
                Ok(_) => {
                    log::info!("🔮 OCW: Request {} completed successfully", request_id);
                }
                Err(e) => {
                    log::warn!("🔮 OCW: Request {} failed: {:?}", request_id, e);
                    // 通过 TeePrivacyIntegration 标记失败
                    let _ = T::TeePrivacy::mark_request_failed(request_id, FailureReason::ComputationError);
                }
            }
        }

        /// 处理公开模式请求（OCW）
        /// 
        /// 完整实现 Public 模式的处理流程：
        /// 1. 获取并验证输入数据
        /// 2. 调用 ModuleRegistry 处理请求
        /// 3. 上传结果到 IPFS
        /// 4. 提交结果到链上
        fn process_public_request(
            request_id: u64,
            divination_type: DivinationType,
        ) -> Result<(), ModuleError> {
            log::info!(
                "🔮 OCW: Processing public request {} (type: {:?})",
                request_id,
                divination_type
            );

            // 1. 获取输入数据
            let input_data = RequestInputData::<T>::get(request_id)
                .ok_or(ModuleError::InvalidInput(b"Input data not found".to_vec().try_into().unwrap_or_default()))?;

            // 2. 确保是明文输入
            let plaintext = match input_data {
                InputData::Plaintext(data) => data,
                InputData::Encrypted(_) => {
                    return Err(ModuleError::InvalidInput(b"Expected plaintext for public mode".to_vec().try_into().unwrap_or_default()));
                }
            };

            log::info!(
                "🔮 OCW: Input data length: {} bytes",
                plaintext.len()
            );

            // 3. 检查模块是否已注册
            if !T::ModuleRegistry::is_registered(divination_type) {
                log::warn!(
                    "🔮 OCW: Module {:?} not registered, skipping",
                    divination_type
                );
                return Err(ModuleError::ModuleNotRegistered);
            }

            // 4. 调用 ModuleRegistry 处理请求
            let process_result = T::ModuleRegistry::process_public(
                divination_type,
                plaintext.as_slice(),
            )?;

            log::info!(
                "🔮 OCW: Computation completed, manifest_cid: {} bytes",
                process_result.manifest_cid.len()
            );

            // 5. 如果有清单数据但没有 CID，上传到 IPFS
            let manifest_cid = if process_result.manifest_cid.is_empty() {
                if let Some(manifest_data) = &process_result.manifest_data {
                    // 上传到 IPFS
                    let cid = T::IpfsClient::upload(manifest_data)?;
                    log::info!("🔮 OCW: Uploaded to IPFS, CID: {} bytes", cid.len());
                    cid
                } else {
                    return Err(ModuleError::IpfsUploadFailed);
                }
            } else {
                process_result.manifest_cid.clone()
            };

            // 6. 获取请求者信息
            let (requester, _compute_type_id, _input_hash, _assigned_node) = 
                T::TeePrivacy::get_request(request_id)
                    .ok_or(ModuleError::other(b"Request not found in tee-privacy"))?;

            // 7. 存储结果到链上
            Self::do_store_result(
                request_id,
                requester,
                divination_type,
                PrivacyMode::Public,
                None, // Public 模式无 TEE 节点
                manifest_cid.clone(),
                process_result.manifest_hash,
                process_result.type_index,
                None, // Public 模式无计算证明
            ).map_err(|e| {
                log::error!("🔮 OCW: Failed to store result: {:?}", e);
                ModuleError::other(b"Failed to store result")
            })?;

            log::info!(
                "🔮 OCW: Public request {} completed successfully",
                request_id
            );

            Ok(())
        }

        /// 处理公开计算加密存储模式请求（OCW）
        /// 
        /// PublicEncrypted 模式的处理流程：
        /// 1. 获取明文输入数据和用户公钥
        /// 2. 调用 ModuleRegistry 处理请求（明文计算）
        /// 3. 使用用户公钥加密 JSON 清单
        /// 4. 上传加密数据到 IPFS
        /// 5. 提交结果到链上
        /// 
        /// 优势：
        /// - 无需 TEE 节点
        /// - 结果隐私保护（只有用户能解密）
        /// - 链上存储索引便于查询
        fn process_public_encrypted_request(
            request_id: u64,
            divination_type: DivinationType,
        ) -> Result<(), ModuleError> {
            log::info!(
                "🔮 OCW: Processing public_encrypted request {} (type: {:?})",
                request_id,
                divination_type
            );

            // 1. 获取输入数据
            let input_data = RequestInputData::<T>::get(request_id)
                .ok_or(ModuleError::InvalidInput(b"Input data not found".to_vec().try_into().unwrap_or_default()))?;

            // 2. 确保是明文输入
            let plaintext = match input_data {
                InputData::Plaintext(data) => data,
                InputData::Encrypted(_) => {
                    return Err(ModuleError::InvalidInput(b"Expected plaintext for public_encrypted mode".to_vec().try_into().unwrap_or_default()));
                }
            };

            // 3. 获取用户公钥（必须提供）
            let user_pubkey = RequestUserPubkey::<T>::get(request_id)
                .ok_or(ModuleError::InvalidInput(b"User pubkey required for encryption".to_vec().try_into().unwrap_or_default()))?;

            if user_pubkey == [0u8; 32] {
                return Err(ModuleError::InvalidInput(b"Invalid user pubkey".to_vec().try_into().unwrap_or_default()));
            }

            log::info!(
                "🔮 OCW: Input data length: {} bytes, user_pubkey: {:?}",
                plaintext.len(),
                &user_pubkey[..4]
            );

            // 4. 检查模块是否已注册
            if !T::ModuleRegistry::is_registered(divination_type) {
                log::warn!(
                    "🔮 OCW: Module {:?} not registered, skipping",
                    divination_type
                );
                return Err(ModuleError::ModuleNotRegistered);
            }

            // 5. 调用 ModuleRegistry 处理请求（明文计算）
            let process_result = T::ModuleRegistry::process_public(
                divination_type,
                plaintext.as_slice(),
            )?;

            log::info!("🔮 OCW: Computation completed, encrypting result...");

            // 6. 获取明文 JSON 清单
            let manifest_data = process_result.manifest_data
                .ok_or(ModuleError::other(b"No manifest data to encrypt"))?;

            // 7. 加密 JSON 清单
            let encrypted_manifest = Self::encrypt_for_user(&manifest_data, &user_pubkey)?;

            log::info!(
                "🔮 OCW: Encrypted manifest: {} bytes -> {} bytes",
                manifest_data.len(),
                encrypted_manifest.len()
            );

            // 8. 上传加密数据到 IPFS
            let manifest_cid = T::IpfsClient::upload(&encrypted_manifest)?;
            log::info!("🔮 OCW: Uploaded encrypted manifest to IPFS");

            // 9. 获取请求者信息
            let (requester, _compute_type_id, _input_hash, _assigned_node) = 
                T::TeePrivacy::get_request(request_id)
                    .ok_or(ModuleError::other(b"Request not found in tee-privacy"))?;

            // 10. 存储结果到链上
            Self::do_store_result(
                request_id,
                requester,
                divination_type,
                PrivacyMode::PublicEncrypted,
                None, // PublicEncrypted 模式无 TEE 节点
                manifest_cid.clone(),
                process_result.manifest_hash, // 明文哈希，用于验证
                process_result.type_index,    // 链上索引
                None, // 无 TEE 计算证明
            ).map_err(|e| {
                log::error!("🔮 OCW: Failed to store result: {:?}", e);
                ModuleError::other(b"Failed to store result")
            })?;

            log::info!(
                "🔮 OCW: PublicEncrypted request {} completed successfully",
                request_id
            );

            Ok(())
        }

        /// 使用用户公钥加密数据 (标准 ECIES)
        /// 
        /// 加密流程：
        /// 1. 生成临时 X25519 密钥对
        /// 2. 使用 ECDH 计算共享密钥
        /// 3. 使用 ChaCha20-Poly1305 AEAD 加密
        /// 4. 返回：ephemeral_pubkey(32) + nonce(12) + ciphertext + auth_tag(16)
        /// 
        /// 使用标准加密库：x25519-dalek + chacha20poly1305
        fn encrypt_for_user(
            plaintext: &[u8],
            user_pubkey: &[u8; 32],
        ) -> Result<Vec<u8>, ModuleError> {
            crate::crypto::encrypt_ecies(plaintext, user_pubkey)
        }

        /// 处理 TEE 模式请求（OCW）
        fn process_tee_request(
            request_id: u64,
            divination_type: DivinationType,
            _privacy_mode: PrivacyMode,
        ) -> Result<(), ModuleError> {
            // 从 TeePrivacyIntegration 获取分配的节点
            let (requester, _compute_type_id, _input_hash, assigned_node) = 
                T::TeePrivacy::get_request(request_id)
                    .ok_or(ModuleError::other(b"Request not found in tee-privacy"))?;

            let node = assigned_node.ok_or(ModuleError::TeeNodeUnavailable)?;

            // 获取节点端点
            let endpoint = T::TeePrivacy::get_node_endpoint(&node)
                .ok_or(ModuleError::TeeNodeUnavailable)?;

            // 获取输入数据
            let input_data = RequestInputData::<T>::get(request_id)
                .ok_or(ModuleError::InvalidInput(b"Input data not found".to_vec().try_into().unwrap_or_default()))?;

            // 获取用户公钥
            let _user_pubkey = RequestUserPubkey::<T>::get(request_id)
                .unwrap_or([0u8; 32]);

            // 确保是加密输入
            let encrypted = match input_data {
                InputData::Encrypted(data) => data,
                InputData::Plaintext(_) => {
                    return Err(ModuleError::InvalidInput(b"Expected encrypted".to_vec().try_into().unwrap_or_default()));
                }
            };

            // 调用 TEE 节点
            let _endpoint_str = core::str::from_utf8(&endpoint)
                .map_err(|_| ModuleError::other(b"Invalid endpoint encoding"))?;

            // TODO: 实现 TEE 模式处理
            // 1. 调用 T::TeeClient::call_tee()
            // 2. 验证计算证明
            // 3. 上传到 IPFS
            // 4. 通过 TeePrivacyIntegration 提交结果

            log::info!(
                "🔮 OCW: TEE mode processing for {:?}, node: {:?}, encrypted_len: {}",
                divination_type,
                node,
                encrypted.ciphertext.len()
            );

            // 验证签名（示例）
            let _is_valid = T::TeePrivacy::verify_enclave_signature(
                &node,
                &[0u8; 32], // 实际应该是计算结果
                &[0u8; 64], // 实际应该是签名
            );

            // 提交结果到 tee-privacy（触发奖励）
            let output_hash = [0u8; 32]; // TODO: 实际计算
            let signature = [0u8; 64]; // TODO: 实际签名
            
            T::TeePrivacy::submit_result(
                request_id,
                node.clone(),
                output_hash,
                signature,
            ).map_err(|_| ModuleError::other(b"Failed to submit result to tee-privacy"))?;

            // 存储占卜结果
            Self::do_store_result(
                request_id,
                requester,
                divination_type,
                _privacy_mode,
                Some(node),
                Vec::new(), // TODO: 实际 CID
                [0u8; 32],  // TODO: 实际哈希
                None,
                None,
            ).map_err(|_| ModuleError::other(b"Failed to store result"))?;

            Ok(())
        }

        /// 清理超时请求
        /// 
        /// 超时处理由 tee-privacy 统一管理，这里只清理本地数据
        fn cleanup_timeout_requests(_current_block: BlockNumberFor<T>) {
            // 从 TeePrivacyIntegration 获取待处理请求
            let request_ids = T::TeePrivacy::get_pending_requests();

            for request_id in request_ids.iter() {
                // 检查请求状态
                if let Some(status) = T::TeePrivacy::get_request_status(*request_id) {
                    if status == RequestStatus::Timeout || status == RequestStatus::Failed {
                        // 清理本地临时数据
                        RequestInputData::<T>::remove(request_id);
                        RequestUserPubkey::<T>::remove(request_id);
                        
                        // 保留 RequestMetadata 用于历史查询
                        
                        Self::deposit_event(Event::RequestTimeout {
                            request_id: *request_id,
                        });
                    }
                }
            }
        }
    }
}
