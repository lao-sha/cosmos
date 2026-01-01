//! # 统一隐私授权模块 - 权重定义
//!
//! 定义各交易的权重（gas 消耗估算）。
//! 在正式部署前应使用 benchmarking 进行精确测量。

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 trait
pub trait WeightInfo {
    // 密钥管理
    fn register_encryption_key() -> Weight;
    fn update_encryption_key() -> Weight;

    // 服务提供者管理
    fn register_provider() -> Weight;
    fn update_provider_key() -> Weight;
    fn set_provider_active() -> Weight;
    fn unregister_provider() -> Weight;

    // 加密数据管理
    fn create_encrypted_record() -> Weight;
    fn update_encrypted_record() -> Weight;
    fn change_privacy_mode() -> Weight;
    fn delete_encrypted_record() -> Weight;

    // 授权管理
    fn grant_access() -> Weight;
    fn revoke_access() -> Weight;
    fn revoke_all_access() -> Weight;
    fn update_access_scope() -> Weight;

    // 悬赏授权
    fn create_bounty_authorization() -> Weight;
    fn authorize_bounty_answerer() -> Weight;
    fn revoke_bounty_authorizations() -> Weight;
}

/// 默认权重实现（用于开发和测试）
///
/// 这些值是估算值，正式部署前应使用 benchmarking 进行精确测量。
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    // ========================================================================
    // 密钥管理
    // ========================================================================

    /// 注册加密公钥
    /// - 1 次存储读取（检查是否已存在）
    /// - 1 次存储写入
    fn register_encryption_key() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// 更新加密公钥
    /// - 1 次存储读取
    /// - 1 次存储写入
    fn update_encryption_key() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    // ========================================================================
    // 服务提供者管理
    // ========================================================================

    /// 注册服务提供者
    /// - 2 次存储读取（检查已存在 + 类型列表）
    /// - 2 次存储写入（提供者信息 + 类型索引）
    fn register_provider() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// 更新提供者公钥
    /// - 1 次存储读取
    /// - 1 次存储写入
    fn update_provider_key() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// 设置提供者活跃状态
    /// - 1 次存储读取
    /// - 1 次存储写入
    fn set_provider_active() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// 注销服务提供者
    /// - 1 次存储读取
    /// - 2 次存储写入（删除提供者 + 更新类型索引）
    fn unregister_provider() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    // ========================================================================
    // 加密数据管理
    // ========================================================================

    /// 创建加密记录
    /// - 1 次存储读取（检查已存在）
    /// - 4 次存储写入（记录 + 用户索引 + 授权 + 授权列表）
    fn create_encrypted_record() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    /// 更新加密记录
    /// - 1 次存储读取
    /// - 1 次存储写入
    fn update_encrypted_record() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// 更改隐私模式
    /// - 1 次存储读取
    /// - 1 次存储写入
    fn change_privacy_mode() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// 删除加密记录
    /// - 2 次存储读取（记录 + 授权列表）
    /// - N 次存储写入（记录 + 用户索引 + 所有授权）
    /// 假设最多 20 个授权
    fn delete_encrypted_record() -> Weight {
        Weight::from_parts(100_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(22))
    }

    // ========================================================================
    // 授权管理
    // ========================================================================

    /// 授权访问
    /// - 2 次存储读取（记录 + 检查授权已存在）
    /// - 3 次存储写入（授权 + 授权列表 + 提供者授权列表）
    fn grant_access() -> Weight {
        Weight::from_parts(45_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// 撤销授权
    /// - 2 次存储读取（记录 + 授权）
    /// - 3 次存储写入（删除授权 + 更新授权列表 + 更新提供者列表）
    fn revoke_access() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// 撤销所有授权
    /// - 2 次存储读取（记录 + 授权列表）
    /// - N 次存储写入（所有授权 + 提供者列表）
    /// 假设最多 20 个授权
    fn revoke_all_access() -> Weight {
        Weight::from_parts(80_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(21))
    }

    /// 更新授权范围
    /// - 2 次存储读取（记录 + 授权）
    /// - 1 次存储写入
    fn update_access_scope() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    // ========================================================================
    // 悬赏授权
    // ========================================================================

    /// 创建悬赏授权配置
    /// - 2 次存储读取（记录 + 检查已存在）
    /// - 1 次存储写入
    fn create_bounty_authorization() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// 为悬赏回答者授权
    /// - 3 次存储读取（悬赏信息 + 记录 + 检查授权已存在）
    /// - 4 次存储写入（授权 + 记录授权列表 + 提供者列表 + 悬赏授权列表）
    fn authorize_bounty_answerer() -> Weight {
        Weight::from_parts(55_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    /// 撤销悬赏所有授权
    /// - 3 次存储读取（悬赏信息 + 记录 + 悬赏授权列表）
    /// - N 次存储写入
    /// 假设最多 100 个回答者
    fn revoke_bounty_authorizations() -> Weight {
        Weight::from_parts(150_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(102))
    }
}

/// 用于测试的权重实现（所有权重为 0）
impl WeightInfo for () {
    fn register_encryption_key() -> Weight {
        Weight::zero()
    }
    fn update_encryption_key() -> Weight {
        Weight::zero()
    }
    fn register_provider() -> Weight {
        Weight::zero()
    }
    fn update_provider_key() -> Weight {
        Weight::zero()
    }
    fn set_provider_active() -> Weight {
        Weight::zero()
    }
    fn unregister_provider() -> Weight {
        Weight::zero()
    }
    fn create_encrypted_record() -> Weight {
        Weight::zero()
    }
    fn update_encrypted_record() -> Weight {
        Weight::zero()
    }
    fn change_privacy_mode() -> Weight {
        Weight::zero()
    }
    fn delete_encrypted_record() -> Weight {
        Weight::zero()
    }
    fn grant_access() -> Weight {
        Weight::zero()
    }
    fn revoke_access() -> Weight {
        Weight::zero()
    }
    fn revoke_all_access() -> Weight {
        Weight::zero()
    }
    fn update_access_scope() -> Weight {
        Weight::zero()
    }
    fn create_bounty_authorization() -> Weight {
        Weight::zero()
    }
    fn authorize_bounty_answerer() -> Weight {
        Weight::zero()
    }
    fn revoke_bounty_authorizations() -> Weight {
        Weight::zero()
    }
}
