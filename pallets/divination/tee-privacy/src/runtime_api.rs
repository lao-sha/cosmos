//! # TEE 隐私计算模块 - Runtime API 定义
//!
//! 本模块定义了 TEE Privacy 的 Runtime API，供 RPC 层调用。
//!
//! ## 功能说明
//!
//! ### TEE 节点管理
//! - `get_active_nodes`: 获取所有活跃 TEE 节点
//! - `get_node_info`: 获取指定节点信息
//! - `is_node_active`: 检查节点是否活跃
//!
//! ### 计算请求管理
//! - `get_request_status`: 获取请求状态
//! - `get_user_pending_requests`: 获取用户待处理请求
//! - `get_node_current_request`: 获取节点当前处理的请求
//!
//! ### 认证验证
//! - `verify_attestation`: 验证认证报告
//! - `get_allowed_mr_enclaves`: 获取允许的 MRENCLAVE 列表
//!
//! ### 经济激励查询
//! - `get_node_stake`: 获取节点质押信息
//! - `get_minimum_stake`: 获取最低质押要求
//!
//! ## 使用示例
//!
//! ```javascript
//! // 获取活跃节点
//! const nodes = await api.call.teePrivacyApi.getActiveNodes();
//!
//! // 检查节点是否活跃
//! const isActive = await api.call.teePrivacyApi.isNodeActive(accountId);
//!
//! // 获取请求状态
//! const status = await api.call.teePrivacyApi.getRequestStatus(requestId);
//! ```

use crate::types::{AttestationVerifyResult, RequestStatusInfo, TeeNodeInfo};
use codec::Codec;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    /// TEE Privacy Runtime API
    ///
    /// 提供查询 TEE 节点、计算请求和系统状态的接口
    pub trait TeePrivacyApi<AccountId, BlockNumber>
    where
        AccountId: Codec,
        BlockNumber: Codec,
    {
        // ============================================================
        // 节点查询接口
        // ============================================================

        /// 获取所有活跃 TEE 节点
        ///
        /// 返回当前所有活跃状态的 TEE 节点信息列表
        ///
        /// # 返回
        /// - `Vec<TeeNodeInfo>`: 活跃节点信息列表
        ///
        /// # 示例
        /// ```javascript
        /// const nodes = await api.call.teePrivacyApi.getActiveNodes();
        /// console.log(`活跃节点数: ${nodes.length}`);
        /// ```
        fn get_active_nodes() -> Vec<TeeNodeInfo>;

        /// 获取指定节点信息
        ///
        /// # 参数
        /// - `account`: 节点账户地址
        ///
        /// # 返回
        /// - `Some(TeeNodeInfo)`: 节点存在时返回节点信息
        /// - `None`: 节点不存在
        fn get_node_info(account: AccountId) -> Option<TeeNodeInfo>;

        /// 获取节点 Enclave 公钥
        ///
        /// # 参数
        /// - `account`: 节点账户地址
        ///
        /// # 返回
        /// - `Some([u8; 32])`: 节点存在时返回公钥
        /// - `None`: 节点不存在
        fn get_enclave_pubkey(account: AccountId) -> Option<[u8; 32]>;

        /// 检查节点是否活跃
        ///
        /// # 参数
        /// - `account`: 节点账户地址
        ///
        /// # 返回
        /// - `true`: 节点活跃且认证有效
        /// - `false`: 节点不活跃或不存在
        fn is_node_active(account: AccountId) -> bool;

        /// 获取活跃节点数量
        fn get_active_node_count() -> u32;

        /// 获取节点总数
        fn get_node_count() -> u32;

        // ============================================================
        // 请求查询接口
        // ============================================================

        /// 获取请求状态
        ///
        /// # 参数
        /// - `request_id`: 请求 ID
        ///
        /// # 返回
        /// - `Some(RequestStatusInfo)`: 请求存在时返回状态信息
        /// - `None`: 请求不存在
        fn get_request_status(request_id: u64) -> Option<RequestStatusInfo>;

        /// 获取用户的所有待处理请求
        ///
        /// # 参数
        /// - `account`: 用户账户地址
        ///
        /// # 返回
        /// 用户的待处理请求 ID 列表
        fn get_user_pending_requests(account: AccountId) -> Vec<u64>;

        /// 获取节点当前处理的请求
        ///
        /// # 参数
        /// - `node`: 节点账户地址
        ///
        /// # 返回
        /// - `Some(u64)`: 节点正在处理的请求 ID
        /// - `None`: 节点没有正在处理的请求
        fn get_node_current_request(node: AccountId) -> Option<u64>;

        /// 获取下一个请求 ID
        fn get_next_request_id() -> u64;

        /// 获取待处理请求数量
        fn get_pending_request_count() -> u32;

        // ============================================================
        // 认证验证接口
        // ============================================================

        /// 验证认证报告
        ///
        /// # 参数
        /// - `mr_enclave`: MRENCLAVE 值
        /// - `mr_signer`: MRSIGNER 值
        /// - `timestamp`: 认证时间戳
        ///
        /// # 返回
        /// 验证结果详情
        fn verify_attestation(
            mr_enclave: [u8; 32],
            mr_signer: [u8; 32],
            timestamp: u64,
        ) -> AttestationVerifyResult;

        /// 获取允许的 MRENCLAVE 列表
        fn get_allowed_mr_enclaves() -> Vec<[u8; 32]>;

        /// 获取允许的 MRSIGNER 列表
        fn get_allowed_mr_signers() -> Vec<[u8; 32]>;

        // ============================================================
        // 经济激励查询接口
        // ============================================================

        /// 获取节点质押信息
        ///
        /// # 参数
        /// - `account`: 节点账户地址
        ///
        /// # 返回
        /// - `Some((amount, unlock_at, is_unbonding))`: 质押详情
        /// - `None`: 未质押
        fn get_node_stake(account: AccountId) -> Option<(u128, Option<BlockNumber>, bool)>;

        /// 获取最低质押要求
        fn get_minimum_stake() -> u128;

        /// 获取累计惩罚金额
        fn get_total_slashed() -> u128;

        /// 获取奖励池余额
        fn get_reward_pool() -> u128;

        // ============================================================
        // 审计日志查询接口
        // ============================================================

        /// 获取审计日志是否启用
        fn is_audit_enabled() -> bool;

        /// 获取账户的审计日志数量
        ///
        /// # 参数
        /// - `account`: 账户地址
        fn get_account_audit_log_count(account: AccountId) -> u32;

        /// 获取下一个审计日志 ID
        fn get_next_audit_log_id() -> u64;
    }
}
