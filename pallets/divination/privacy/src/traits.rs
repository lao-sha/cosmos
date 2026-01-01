//! # 统一隐私授权模块 - Trait 定义
//!
//! 本模块定义了隐私授权系统的 Trait 接口，
//! 供各占卜模块和悬赏系统调用。
//!
//! ## Trait 接口
//!
//! - `DivinationPrivacy`: 供占卜模块使用的隐私管理接口
//! - `BountyPrivacy`: 供悬赏系统使用的授权管理接口

use crate::types::{AccessRole, AccessScope, PrivacyMode, ServiceProviderType};
use frame_support::dispatch::DispatchResult;
use pallet_divination_common::DivinationType;
use sp_std::vec::Vec;

// ============================================================================
// 占卜模块隐私管理 Trait
// ============================================================================

/// 占卜隐私管理 Trait
///
/// 各占卜模块通过此 trait 与隐私模块交互，
/// 用于检查和管理加密记录的访问权限。
pub trait DivinationPrivacy<AccountId, BlockNumber> {
    /// 检查记录是否为加密存储
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    ///
    /// # 返回
    /// - `true`: 记录存在且为加密存储
    /// - `false`: 记录不存在或为公开存储
    fn is_encrypted(divination_type: DivinationType, result_id: u64) -> bool;

    /// 获取记录的隐私模式
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    ///
    /// # 返回
    /// - `Some(PrivacyMode)`: 记录的隐私模式
    /// - `None`: 记录不存在
    fn get_privacy_mode(
        divination_type: DivinationType,
        result_id: u64,
    ) -> Option<PrivacyMode>;

    /// 检查账户是否有访问权限
    ///
    /// 检查逻辑：
    /// 1. 公开记录：所有人可访问
    /// 2. 私密记录：仅所有者可访问
    /// 3. 授权记录：所有者 + 被授权者可访问
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    /// - `account`: 要检查的账户
    ///
    /// # 返回
    /// - `true`: 有访问权限
    /// - `false`: 无访问权限或记录不存在
    fn has_access(
        divination_type: DivinationType,
        result_id: u64,
        account: &AccountId,
    ) -> bool;

    /// 获取账户的访问角色
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    /// - `account`: 要查询的账户
    ///
    /// # 返回
    /// - `Some(AccessRole)`: 账户的访问角色
    /// - `None`: 无访问权限或记录不存在
    fn get_access_role(
        divination_type: DivinationType,
        result_id: u64,
        account: &AccountId,
    ) -> Option<AccessRole>;

    /// 获取账户的访问范围
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    /// - `account`: 要查询的账户
    ///
    /// # 返回
    /// - `Some(AccessScope)`: 账户的访问范围
    /// - `None`: 无访问权限或记录不存在
    fn get_access_scope(
        divination_type: DivinationType,
        result_id: u64,
        account: &AccountId,
    ) -> Option<AccessScope>;

    /// 获取记录的所有授权账户
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    ///
    /// # 返回
    /// - 被授权账户列表（不含所有者）
    fn get_grantees(divination_type: DivinationType, result_id: u64) -> Vec<AccountId>;

    /// 获取记录的所有者
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    ///
    /// # 返回
    /// - `Some(AccountId)`: 记录的所有者
    /// - `None`: 记录不存在
    fn get_owner(divination_type: DivinationType, result_id: u64) -> Option<AccountId>;

    /// 获取用户的加密公钥
    ///
    /// # 参数
    /// - `account`: 用户账户
    ///
    /// # 返回
    /// - `Some([u8; 32])`: X25519 公钥
    /// - `None`: 用户未注册公钥
    fn get_user_public_key(account: &AccountId) -> Option<[u8; 32]>;

    /// 获取服务提供者类型
    ///
    /// # 参数
    /// - `account`: 服务提供者账户
    ///
    /// # 返回
    /// - `Some(ServiceProviderType)`: 提供者类型
    /// - `None`: 不是服务提供者
    fn get_provider_type(account: &AccountId) -> Option<ServiceProviderType>;

    /// 检查服务提供者是否活跃
    ///
    /// # 参数
    /// - `account`: 服务提供者账户
    ///
    /// # 返回
    /// - `true`: 是活跃的服务提供者
    /// - `false`: 不是服务提供者或已停用
    fn is_provider_active(account: &AccountId) -> bool;
}

// ============================================================================
// 悬赏系统授权 Trait
// ============================================================================

/// 悬赏隐私管理 Trait
///
/// 悬赏系统通过此 trait 管理与加密数据相关的授权，
/// 确保回答者能够访问必要的数据来完成解读。
pub trait BountyPrivacy<AccountId, BlockNumber> {
    /// 检查悬赏关联的结果是否加密
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    ///
    /// # 返回
    /// - `true`: 关联的结果为加密存储
    /// - `false`: 关联的结果为公开存储或不存在
    fn is_bounty_encrypted(divination_type: DivinationType, result_id: u64) -> bool;

    /// 检查回答者是否有权限访问悬赏数据
    ///
    /// 用于在大师接单时检查是否已获得授权
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    /// - `answerer`: 回答者账户
    ///
    /// # 返回
    /// - `true`: 回答者有访问权限
    /// - `false`: 回答者无访问权限
    fn can_answer_bounty(
        divination_type: DivinationType,
        result_id: u64,
        answerer: &AccountId,
    ) -> bool;

    /// 获取悬赏的授权列表
    ///
    /// # 参数
    /// - `bounty_id`: 悬赏 ID
    ///
    /// # 返回
    /// - 已授权的回答者账户列表
    fn get_bounty_authorizations(bounty_id: u64) -> Vec<AccountId>;

    /// 检查悬赏是否需要授权
    ///
    /// 用于在创建悬赏时提示用户需要进行授权
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    ///
    /// # 返回
    /// - `true`: 悬赏关联加密数据，需要授权
    /// - `false`: 悬赏关联公开数据，无需授权
    fn bounty_requires_authorization(
        divination_type: DivinationType,
        result_id: u64,
    ) -> bool;

    /// 获取悬赏的授权过期时间
    ///
    /// # 参数
    /// - `bounty_id`: 悬赏 ID
    ///
    /// # 返回
    /// - `Some(BlockNumber)`: 授权过期区块号
    /// - `None`: 悬赏不存在或无授权配置
    fn get_bounty_authorization_expiry(bounty_id: u64) -> Option<BlockNumber>;

    /// 检查悬赏是否启用自动授权
    ///
    /// 自动授权：新回答者接单时自动获得临时访问权限
    ///
    /// # 参数
    /// - `bounty_id`: 悬赏 ID
    ///
    /// # 返回
    /// - `true`: 启用自动授权
    /// - `false`: 未启用或悬赏不存在
    fn is_auto_authorize_enabled(bounty_id: u64) -> bool;
}

// ============================================================================
// 隐私模块回调 Trait
// ============================================================================

/// 隐私事件回调 Trait
///
/// 允许其他模块监听隐私模块的关键事件
pub trait PrivacyEventHandler<AccountId> {
    /// 当授权被创建时调用
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    /// - `grantor`: 授权者
    /// - `grantee`: 被授权者
    /// - `role`: 授权角色
    fn on_access_granted(
        divination_type: DivinationType,
        result_id: u64,
        grantor: &AccountId,
        grantee: &AccountId,
        role: AccessRole,
    );

    /// 当授权被撤销时调用
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    /// - `grantor`: 授权者
    /// - `grantee`: 被授权者
    fn on_access_revoked(
        divination_type: DivinationType,
        result_id: u64,
        grantor: &AccountId,
        grantee: &AccountId,
    );

    /// 当加密记录被创建时调用
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    /// - `owner`: 所有者
    fn on_encrypted_record_created(
        divination_type: DivinationType,
        result_id: u64,
        owner: &AccountId,
    );

    /// 当加密记录被删除时调用
    ///
    /// # 参数
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    /// - `owner`: 所有者
    fn on_encrypted_record_deleted(
        divination_type: DivinationType,
        result_id: u64,
        owner: &AccountId,
    );
}

/// 空实现，用于不需要回调的场景
impl<AccountId> PrivacyEventHandler<AccountId> for () {
    fn on_access_granted(
        _divination_type: DivinationType,
        _result_id: u64,
        _grantor: &AccountId,
        _grantee: &AccountId,
        _role: AccessRole,
    ) {
    }

    fn on_access_revoked(
        _divination_type: DivinationType,
        _result_id: u64,
        _grantor: &AccountId,
        _grantee: &AccountId,
    ) {
    }

    fn on_encrypted_record_created(
        _divination_type: DivinationType,
        _result_id: u64,
        _owner: &AccountId,
    ) {
    }

    fn on_encrypted_record_deleted(
        _divination_type: DivinationType,
        _result_id: u64,
        _owner: &AccountId,
    ) {
    }
}

// ============================================================================
// 加密记录管理 Trait（内部使用）
// ============================================================================

/// 加密记录管理 Trait
///
/// 提供创建和管理加密记录的接口，
/// 主要供 Pallet 内部使用
pub trait EncryptedRecordManager<AccountId, BlockNumber> {
    /// 创建加密记录
    ///
    /// # 参数
    /// - `owner`: 所有者
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    /// - `privacy_mode`: 隐私模式
    /// - `encrypted_data`: 加密数据
    /// - `nonce`: 加密随机数
    /// - `auth_tag`: 认证标签
    /// - `data_hash`: 数据哈希
    /// - `owner_encrypted_key`: 所有者的加密密钥
    ///
    /// # 返回
    /// - `Ok(())`: 创建成功
    /// - `Err`: 创建失败
    fn create_record(
        owner: &AccountId,
        divination_type: DivinationType,
        result_id: u64,
        privacy_mode: PrivacyMode,
        encrypted_data: Vec<u8>,
        nonce: [u8; 24],
        auth_tag: [u8; 16],
        data_hash: [u8; 32],
        owner_encrypted_key: Vec<u8>,
    ) -> DispatchResult;

    /// 删除加密记录
    ///
    /// # 参数
    /// - `owner`: 所有者
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    ///
    /// # 返回
    /// - `Ok(())`: 删除成功
    /// - `Err`: 删除失败（无权限或记录不存在）
    fn delete_record(
        owner: &AccountId,
        divination_type: DivinationType,
        result_id: u64,
    ) -> DispatchResult;

    /// 添加授权
    ///
    /// # 参数
    /// - `grantor`: 授权者（必须是所有者）
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    /// - `grantee`: 被授权者
    /// - `encrypted_key`: 为被授权者加密的密钥
    /// - `role`: 授权角色
    /// - `scope`: 访问范围
    /// - `expires_at`: 过期时间（0 表示永久）
    /// - `bounty_id`: 关联的悬赏 ID（可选）
    ///
    /// # 返回
    /// - `Ok(())`: 授权成功
    /// - `Err`: 授权失败
    fn grant_access(
        grantor: &AccountId,
        divination_type: DivinationType,
        result_id: u64,
        grantee: &AccountId,
        encrypted_key: Vec<u8>,
        role: AccessRole,
        scope: AccessScope,
        expires_at: BlockNumber,
        bounty_id: Option<u64>,
    ) -> DispatchResult;

    /// 撤销授权
    ///
    /// # 参数
    /// - `grantor`: 授权者（必须是所有者）
    /// - `divination_type`: 占卜类型
    /// - `result_id`: 结果 ID
    /// - `grantee`: 被授权者
    ///
    /// # 返回
    /// - `Ok(())`: 撤销成功
    /// - `Err`: 撤销失败（无权限或授权不存在）
    fn revoke_access(
        grantor: &AccountId,
        divination_type: DivinationType,
        result_id: u64,
        grantee: &AccountId,
    ) -> DispatchResult;
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // 测试空实现编译通过
    #[test]
    fn test_empty_event_handler_compiles() {
        // 确保空实现可以正常调用
        <() as PrivacyEventHandler<u64>>::on_access_granted(
            DivinationType::Bazi,
            1,
            &1u64,
            &2u64,
            AccessRole::Master,
        );

        <() as PrivacyEventHandler<u64>>::on_access_revoked(
            DivinationType::Bazi,
            1,
            &1u64,
            &2u64,
        );

        <() as PrivacyEventHandler<u64>>::on_encrypted_record_created(
            DivinationType::Bazi,
            1,
            &1u64,
        );

        <() as PrivacyEventHandler<u64>>::on_encrypted_record_deleted(
            DivinationType::Bazi,
            1,
            &1u64,
        );
    }
}
