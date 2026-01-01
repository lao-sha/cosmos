//! # 统一隐私授权模块 - Runtime API 定义
//!
//! 为前端提供查询接口，包括：
//! - 用户加密公钥查询
//! - 服务提供者查询
//! - 加密记录查询
//! - 授权状态查询
//! - 悬赏授权状态查询

use crate::types::{
    AuthorizationInfo, BountyAuthorizationStatus, EncryptedRecordInfo, ServiceProviderInfo,
};
use codec::Codec;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    /// 隐私模块 Runtime API
    ///
    /// 提供链上隐私数据的查询接口
    pub trait DivinationPrivacyApi<AccountId, BlockNumber>
    where
        AccountId: Codec,
        BlockNumber: Codec,
    {
        // ====================================================================
        // 用户密钥查询
        // ====================================================================

        /// 获取用户加密公钥
        ///
        /// # 参数
        /// - `account`: 用户账户
        ///
        /// # 返回
        /// - `Some([u8; 32])`: X25519 公钥
        /// - `None`: 用户未注册公钥
        fn get_user_encryption_key(account: AccountId) -> Option<[u8; 32]>;

        /// 检查用户是否已注册公钥
        ///
        /// # 参数
        /// - `account`: 用户账户
        ///
        /// # 返回
        /// - `true`: 已注册
        /// - `false`: 未注册
        fn has_encryption_key(account: AccountId) -> bool;

        // ====================================================================
        // 服务提供者查询
        // ====================================================================

        /// 获取服务提供者信息
        ///
        /// # 参数
        /// - `account`: 服务提供者账户
        ///
        /// # 返回
        /// - `Some(ServiceProviderInfo)`: 提供者信息
        /// - `None`: 不是服务提供者
        fn get_service_provider(account: AccountId) -> Option<ServiceProviderInfo>;

        /// 按类型获取服务提供者列表
        ///
        /// # 参数
        /// - `provider_type`: 服务提供者类型（0-3）
        ///
        /// # 返回
        /// - 该类型的服务提供者账户列表
        fn get_providers_by_type(provider_type: u8) -> Vec<AccountId>;

        /// 检查账户是否为活跃的服务提供者
        ///
        /// # 参数
        /// - `account`: 账户
        ///
        /// # 返回
        /// - `true`: 是活跃的服务提供者
        /// - `false`: 不是或已停用
        fn is_active_provider(account: AccountId) -> bool;

        // ====================================================================
        // 加密记录查询
        // ====================================================================

        /// 获取加密记录信息
        ///
        /// # 参数
        /// - `divination_type`: 占卜类型（0-4）
        /// - `result_id`: 结果 ID
        ///
        /// # 返回
        /// - `Some(EncryptedRecordInfo)`: 记录信息
        /// - `None`: 记录不存在
        fn get_encrypted_record_info(
            divination_type: u8,
            result_id: u64,
        ) -> Option<EncryptedRecordInfo>;

        /// 检查记录是否为加密存储
        ///
        /// # 参数
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 结果 ID
        ///
        /// # 返回
        /// - `true`: 是加密记录
        /// - `false`: 不是或不存在
        fn is_encrypted_record(divination_type: u8, result_id: u64) -> bool;

        /// 获取用户的加密记录列表
        ///
        /// # 参数
        /// - `account`: 用户账户
        /// - `divination_type`: 占卜类型（可选，None 表示所有类型）
        ///
        /// # 返回
        /// - 结果 ID 列表
        fn get_user_encrypted_records(
            account: AccountId,
            divination_type: Option<u8>,
        ) -> Vec<u64>;

        // ====================================================================
        // 授权查询
        // ====================================================================

        /// 检查是否有访问权限
        ///
        /// # 参数
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 结果 ID
        /// - `account`: 要检查的账户
        ///
        /// # 返回
        /// - `true`: 有访问权限
        /// - `false`: 无访问权限
        fn has_access(divination_type: u8, result_id: u64, account: AccountId) -> bool;

        /// 获取授权列表
        ///
        /// # 参数
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 结果 ID
        ///
        /// # 返回
        /// - 授权信息列表
        fn get_authorizations(
            divination_type: u8,
            result_id: u64,
        ) -> Vec<AuthorizationInfo>;

        /// 获取提供者被授权的记录
        ///
        /// # 参数
        /// - `account`: 服务提供者账户
        ///
        /// # 返回
        /// - (divination_type, result_id) 列表
        fn get_provider_grants(account: AccountId) -> Vec<(u8, u64)>;

        /// 获取特定授权的详细信息
        ///
        /// # 参数
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 结果 ID
        /// - `grantee`: 被授权账户
        ///
        /// # 返回
        /// - `Some(AuthorizationInfo)`: 授权信息
        /// - `None`: 授权不存在
        fn get_authorization_info(
            divination_type: u8,
            result_id: u64,
            grantee: AccountId,
        ) -> Option<AuthorizationInfo>;

        // ====================================================================
        // 悬赏授权查询
        // ====================================================================

        /// 获取悬赏授权状态
        ///
        /// # 参数
        /// - `bounty_id`: 悬赏 ID
        ///
        /// # 返回
        /// - 悬赏授权状态信息
        fn get_bounty_authorization_status(bounty_id: u64) -> BountyAuthorizationStatus;

        /// 检查悬赏是否需要授权
        ///
        /// # 参数
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 结果 ID
        ///
        /// # 返回
        /// - `true`: 需要授权
        /// - `false`: 不需要
        fn bounty_requires_authorization(divination_type: u8, result_id: u64) -> bool;

        /// 获取悬赏已授权的回答者列表
        ///
        /// # 参数
        /// - `bounty_id`: 悬赏 ID
        ///
        /// # 返回
        /// - 已授权的回答者账户列表
        fn get_bounty_authorized_answerers(bounty_id: u64) -> Vec<AccountId>;
    }
}
