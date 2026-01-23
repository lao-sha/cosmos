//! # OCW + TEE 通用 Trait 定义
//!
//! 定义各占卜模块需要实现的 trait 和通用接口。

use crate::types::*;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_std::prelude::*;

// ==================== 占卜模块 Trait ====================

/// 占卜模块 Trait
///
/// 所有占卜模块必须实现此 trait 来接入 OCW + TEE 通用架构。
///
/// # 示例
///
/// ```ignore
/// impl<T: Config> DivinationModule<T> for Pallet<T> {
///     const MODULE_ID: DivinationType = DivinationType::BaZi;
///     const MODULE_NAME: &'static str = "BaZi";
///     const VERSION: u32 = 1;
///
///     type PlainInput = BaziInputPlain;
///     type Index = SiZhuIndex;
///     type Result = BaziChart;
///
///     fn compute(input: &Self::PlainInput) -> Result<Self::Result, ModuleError> {
///         // 实现计算逻辑
///     }
///
///     // ... 其他方法
/// }
/// ```
pub trait DivinationModule<T: frame_system::Config> {
    /// 模块唯一标识
    const MODULE_ID: DivinationType;

    /// 模块名称（用于日志和调试）
    const MODULE_NAME: &'static str;

    /// 模块版本
    const VERSION: u32;

    /// 输入类型（明文）
    type PlainInput: Clone + Encode + Decode + TypeInfo + MaxEncodedLen;

    /// 索引类型（链上存储的最小化数据）
    type Index: Clone + Encode + Decode + TypeInfo + MaxEncodedLen;

    /// 计算结果类型
    type Result: Clone + Encode + Decode;

    // ==================== 核心方法 ====================

    /// 执行计算
    ///
    /// 根据输入数据执行占卜计算，返回完整结果。
    fn compute(input: &Self::PlainInput) -> Result<Self::Result, ModuleError>;

    /// 从计算结果提取索引
    ///
    /// 根据隐私模式决定是否返回索引：
    /// - Public/Encrypted: 返回索引
    /// - Private: 返回 None
    fn extract_index(result: &Self::Result, privacy_mode: PrivacyMode) -> Option<Self::Index>;

    /// 生成 JSON 清单
    ///
    /// 将计算结果转换为 JSON 格式，用于 IPFS 存储。
    fn generate_manifest(
        input: &Self::PlainInput,
        result: &Self::Result,
        privacy_mode: PrivacyMode,
    ) -> Result<Vec<u8>, ModuleError>;

    /// 验证输入有效性
    ///
    /// 在计算前验证输入数据是否合法。
    fn validate_input(input: &Self::PlainInput) -> Result<(), ModuleError>;

    // ==================== 可选方法（有默认实现）====================

    /// 获取推荐超时时间（区块数）
    fn recommended_timeout() -> u32 {
        Self::MODULE_ID.recommended_timeout()
    }

    /// 获取最大输入大小（字节）
    fn max_input_size() -> u32 {
        Self::MODULE_ID.max_input_size()
    }

    /// 是否支持批量处理
    fn supports_batch() -> bool {
        false
    }

    /// 获取 TEE 端点路径
    fn tee_endpoint() -> &'static str {
        Self::MODULE_ID.tee_endpoint()
    }

    /// 模块初始化钩子（runtime 启动时调用）
    fn on_initialize() -> Weight {
        Weight::zero()
    }

    /// 模块清理钩子（区块结束时调用）
    fn on_finalize() {
        // 默认无操作
    }
}

// ==================== 模块处理器 Trait ====================

/// 模块处理器 Trait（类型擦除）
///
/// 用于在注册表中统一管理不同类型的占卜模块。
pub trait ModuleHandler<T: frame_system::Config>: Send + Sync {
    /// 获取模块 ID
    fn module_id(&self) -> DivinationType;

    /// 获取模块名称
    fn module_name(&self) -> &'static str;

    /// 获取模块版本
    fn version(&self) -> u32;

    /// 处理请求（通用入口）
    ///
    /// 接收编码后的输入数据，返回处理结果。
    fn handle_request(
        &self,
        input_data: &[u8],
        privacy_mode: PrivacyMode,
    ) -> Result<ProcessResult, ModuleError>;

    /// 获取推荐超时
    fn recommended_timeout(&self) -> u32;

    /// 获取最大输入大小
    fn max_input_size(&self) -> u32;
}

// ==================== TEE 客户端 Trait ====================

/// TEE 客户端 Trait
///
/// 定义与 TEE 节点通信的接口。
pub trait TeeClient {
    /// 调用 TEE 节点执行计算
    ///
    /// # 参数
    /// - `endpoint`: TEE 节点 HTTP 端点
    /// - `divination_type`: 占卜类型
    /// - `encrypted_input`: 加密的输入数据
    /// - `user_pubkey`: 用户公钥（用于加密返回结果）
    /// - `privacy_mode`: 隐私模式
    ///
    /// # 返回
    /// TEE 计算响应或错误
    fn call_tee(
        endpoint: &str,
        divination_type: DivinationType,
        encrypted_input: &[u8],
        user_pubkey: &[u8; 32],
        privacy_mode: PrivacyMode,
    ) -> Result<TeeComputeResponse, ModuleError>;

    /// 获取 TEE 节点公钥
    fn get_enclave_pubkey(endpoint: &str) -> Result<[u8; 32], ModuleError>;

    /// 验证 TEE 远程认证
    fn verify_attestation(attestation: &[u8]) -> Result<bool, ModuleError>;
}

// ==================== IPFS 客户端 Trait ====================

/// IPFS 客户端 Trait
///
/// 定义与 IPFS 节点通信的接口。
pub trait IpfsClient {
    /// 上传数据到 IPFS
    ///
    /// # 参数
    /// - `data`: 要上传的数据
    ///
    /// # 返回
    /// IPFS CID 或错误
    fn upload(data: &[u8]) -> Result<Vec<u8>, ModuleError>;

    /// 请求 PIN 数据
    ///
    /// # 参数
    /// - `cid`: IPFS CID
    fn pin(cid: &[u8]) -> Result<(), ModuleError>;

    /// 取消 PIN 数据
    ///
    /// # 参数
    /// - `cid`: IPFS CID
    fn unpin(cid: &[u8]) -> Result<(), ModuleError>;

    /// 获取数据
    ///
    /// # 参数
    /// - `cid`: IPFS CID
    ///
    /// # 返回
    /// 数据内容或错误
    fn get(cid: &[u8]) -> Result<Vec<u8>, ModuleError>;
}

// ==================== TEE 节点管理 Trait ====================

/// TEE 节点运行时信息（用于 OCW 调度）
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct TeeNodeRuntime<AccountId: Clone + Encode + Decode + TypeInfo + MaxEncodedLen> {
    /// 节点账户
    pub account: AccountId,
    /// HTTP 端点
    pub endpoint: BoundedVec<u8, ConstU32<256>>,
    /// Enclave 公钥
    pub enclave_pubkey: [u8; 32],
    /// 是否在线
    pub is_online: bool,
    /// 当前负载（待处理请求数）
    pub current_load: u32,
    /// 最大负载
    pub max_load: u32,
}

/// TEE 节点管理 Trait
pub trait TeeNodeManager<AccountId: Clone + Encode + Decode + TypeInfo + MaxEncodedLen> {
    /// 获取可用的 TEE 节点
    ///
    /// 返回负载最低的在线节点。
    fn get_available_node() -> Option<TeeNodeRuntime<AccountId>>;

    /// 获取指定节点信息
    fn get_node(account: &AccountId) -> Option<TeeNodeRuntime<AccountId>>;

    /// 增加节点负载
    fn increment_load(account: &AccountId);

    /// 减少节点负载
    fn decrement_load(account: &AccountId);

    /// 标记节点离线
    fn mark_offline(account: &AccountId);

    /// 标记节点在线
    fn mark_online(account: &AccountId);
}

// ==================== 空实现（用于测试）====================

/// 空 TEE 客户端实现
pub struct NullTeeClient;

impl TeeClient for NullTeeClient {
    fn call_tee(
        _endpoint: &str,
        _divination_type: DivinationType,
        _encrypted_input: &[u8],
        _user_pubkey: &[u8; 32],
        _privacy_mode: PrivacyMode,
    ) -> Result<TeeComputeResponse, ModuleError> {
        Err(ModuleError::TeeNodeUnavailable)
    }

    fn get_enclave_pubkey(_endpoint: &str) -> Result<[u8; 32], ModuleError> {
        Err(ModuleError::TeeNodeUnavailable)
    }

    fn verify_attestation(_attestation: &[u8]) -> Result<bool, ModuleError> {
        Ok(false)
    }
}

/// 空 IPFS 客户端实现
pub struct NullIpfsClient;

impl IpfsClient for NullIpfsClient {
    fn upload(_data: &[u8]) -> Result<Vec<u8>, ModuleError> {
        Err(ModuleError::IpfsUploadFailed)
    }

    fn pin(_cid: &[u8]) -> Result<(), ModuleError> {
        Err(ModuleError::IpfsUploadFailed)
    }

    fn unpin(_cid: &[u8]) -> Result<(), ModuleError> {
        Err(ModuleError::IpfsUploadFailed)
    }

    fn get(_cid: &[u8]) -> Result<Vec<u8>, ModuleError> {
        Err(ModuleError::IpfsUploadFailed)
    }
}

// ==================== TEE Privacy 集成 Trait ====================

/// TEE Privacy 集成 Trait
///
/// 定义 ocw-tee 与 tee-privacy 的交互接口。
/// 通过此 trait，ocw-tee 可以：
/// - 提交计算请求到 tee-privacy 的统一队列
/// - 获取可用的 TEE 节点
/// - 提交计算结果并触发奖励
/// - 验证 Enclave 签名
pub trait TeePrivacyIntegration<AccountId, BlockNumber> {
    /// 提交计算请求到 tee-privacy 队列
    ///
    /// # 参数
    /// - `requester`: 请求者账户
    /// - `compute_type_id`: 计算类型 ID (0=Meihua, 1=BaZi, 等)
    /// - `input_hash`: 输入数据哈希
    /// - `timeout_blocks`: 超时区块数
    ///
    /// # 返回
    /// 请求 ID 或错误
    fn submit_request(
        requester: AccountId,
        compute_type_id: u8,
        input_hash: [u8; 32],
        timeout_blocks: u32,
    ) -> Result<u64, sp_runtime::DispatchError>;

    /// 获取请求状态
    fn get_request_status(request_id: u64) -> Option<RequestStatus>;

    /// 获取请求信息
    fn get_request(request_id: u64) -> Option<(AccountId, u8, [u8; 32], Option<AccountId>)>;

    /// 获取待处理请求列表
    fn get_pending_requests() -> Vec<u64>;

    /// 获取可用 TEE 节点
    ///
    /// 返回负载最低的活跃节点账户
    fn get_available_node() -> Option<AccountId>;

    /// 获取节点 Enclave 公钥
    fn get_node_enclave_pubkey(node: &AccountId) -> Option<[u8; 32]>;

    /// 获取节点 HTTP 端点
    fn get_node_endpoint(node: &AccountId) -> Option<Vec<u8>>;

    /// 分配节点给请求
    fn assign_node(request_id: u64, node: AccountId) -> Result<(), sp_runtime::DispatchError>;

    /// 提交计算结果
    ///
    /// # 参数
    /// - `request_id`: 请求 ID
    /// - `executor`: 执行节点账户
    /// - `output_hash`: 输出数据哈希
    /// - `signature`: Enclave 签名
    ///
    /// # 返回
    /// 成功或错误
    fn submit_result(
        request_id: u64,
        executor: AccountId,
        output_hash: [u8; 32],
        signature: [u8; 64],
    ) -> Result<(), sp_runtime::DispatchError>;

    /// 标记请求失败
    fn mark_request_failed(
        request_id: u64,
        reason: FailureReason,
    ) -> Result<(), sp_runtime::DispatchError>;

    /// 验证 Enclave 签名
    ///
    /// # 参数
    /// - `node`: 节点账户
    /// - `data`: 签名数据
    /// - `signature`: 签名
    ///
    /// # 返回
    /// 签名是否有效
    fn verify_enclave_signature(
        node: &AccountId,
        data: &[u8],
        signature: &[u8; 64],
    ) -> bool;

    /// 获取节点统计信息
    fn get_node_stats(node: &AccountId) -> Option<NodeStatistics>;

    /// 检查节点是否活跃
    fn is_node_active(node: &AccountId) -> bool;
}

/// 空 TEE Privacy 集成实现（用于测试）
pub struct NullTeePrivacyIntegration<AccountId, BlockNumber>(
    sp_std::marker::PhantomData<(AccountId, BlockNumber)>,
);

impl<AccountId, BlockNumber> TeePrivacyIntegration<AccountId, BlockNumber>
    for NullTeePrivacyIntegration<AccountId, BlockNumber>
{
    fn submit_request(
        _requester: AccountId,
        _compute_type_id: u8,
        _input_hash: [u8; 32],
        _timeout_blocks: u32,
    ) -> Result<u64, sp_runtime::DispatchError> {
        Err(sp_runtime::DispatchError::Other("TEE Privacy not available"))
    }

    fn get_request_status(_request_id: u64) -> Option<RequestStatus> {
        None
    }

    fn get_request(_request_id: u64) -> Option<(AccountId, u8, [u8; 32], Option<AccountId>)> {
        None
    }

    fn get_pending_requests() -> Vec<u64> {
        Vec::new()
    }

    fn get_available_node() -> Option<AccountId> {
        None
    }

    fn get_node_enclave_pubkey(_node: &AccountId) -> Option<[u8; 32]> {
        None
    }

    fn get_node_endpoint(_node: &AccountId) -> Option<Vec<u8>> {
        None
    }

    fn assign_node(_request_id: u64, _node: AccountId) -> Result<(), sp_runtime::DispatchError> {
        Err(sp_runtime::DispatchError::Other("TEE Privacy not available"))
    }

    fn submit_result(
        _request_id: u64,
        _executor: AccountId,
        _output_hash: [u8; 32],
        _signature: [u8; 64],
    ) -> Result<(), sp_runtime::DispatchError> {
        Err(sp_runtime::DispatchError::Other("TEE Privacy not available"))
    }

    fn mark_request_failed(
        _request_id: u64,
        _reason: FailureReason,
    ) -> Result<(), sp_runtime::DispatchError> {
        Err(sp_runtime::DispatchError::Other("TEE Privacy not available"))
    }

    fn verify_enclave_signature(
        _node: &AccountId,
        _data: &[u8],
        _signature: &[u8; 64],
    ) -> bool {
        false
    }

    fn get_node_stats(_node: &AccountId) -> Option<NodeStatistics> {
        None
    }

    fn is_node_active(_node: &AccountId) -> bool {
        false
    }
}

// ==================== 空实现（用于测试）====================

/// 空 TEE 节点管理实现
pub struct NullTeeNodeManager<AccountId>(sp_std::marker::PhantomData<AccountId>);

impl<AccountId: Clone + Encode + Decode + TypeInfo + MaxEncodedLen> TeeNodeManager<AccountId>
    for NullTeeNodeManager<AccountId>
{
    fn get_available_node() -> Option<TeeNodeRuntime<AccountId>> {
        None
    }

    fn get_node(_account: &AccountId) -> Option<TeeNodeRuntime<AccountId>> {
        None
    }

    fn increment_load(_account: &AccountId) {}

    fn decrement_load(_account: &AccountId) {}

    fn mark_offline(_account: &AccountId) {}

    fn mark_online(_account: &AccountId) {}
}
