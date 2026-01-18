//! # TEE 隐私计算模块 - 权重定义
//!
//! 本模块定义了所有 extrinsic 的权重估算。

use frame_support::weights::Weight;
use frame_support::traits::Get;
use core::marker::PhantomData;

/// 权重信息 trait
pub trait WeightInfo {
    // ==================== TEE 节点管理 ====================

    /// 注册 TEE 节点
    fn register_tee_node() -> Weight;

    /// 更新认证报告
    fn update_attestation() -> Weight;

    /// 更新节点状态
    fn update_node_status() -> Weight;

    /// 注销 TEE 节点
    fn deregister_tee_node() -> Weight;

    // ==================== 计算请求管理 ====================

    /// 提交计算请求
    fn submit_compute_request() -> Weight;

    /// 提交计算结果
    fn submit_compute_result() -> Weight;

    /// 取消计算请求
    fn cancel_compute_request() -> Weight;

    // ==================== 批处理优化 ====================

    /// 批量提交计算请求
    fn submit_batch_compute_requests(n: u32) -> Weight;

    /// 批量提交计算结果
    fn submit_batch_compute_results(n: u32) -> Weight;

    // ==================== 经济激励 ====================

    /// 质押
    fn stake() -> Weight;

    /// 申请解除质押
    fn request_unstake() -> Weight;

    /// 提取解除质押的金额
    fn withdraw_unstaked() -> Weight;

    // ==================== 管理功能 ====================

    /// 添加允许的 MR_ENCLAVE
    fn add_allowed_mr_enclave() -> Weight;

    /// 移除允许的 MR_ENCLAVE
    fn remove_allowed_mr_enclave() -> Weight;

    /// 添加允许的 MR_SIGNER
    fn add_allowed_mr_signer() -> Weight;

    /// 移除允许的 MR_SIGNER
    fn remove_allowed_mr_signer() -> Weight;

    /// 设置审计启用状态
    fn set_audit_enabled() -> Weight;

    // ==================== 超时处理 ====================

    /// 处理超时请求（on_finalize）
    fn process_timeouts(n: u32) -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn register_tee_node() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn update_attestation() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn update_node_status() -> Weight {
        Weight::from_parts(15_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn deregister_tee_node() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn submit_compute_request() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    fn submit_compute_result() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    fn cancel_compute_request() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn submit_batch_compute_requests(n: u32) -> Weight {
        Weight::from_parts(30_000_000 + 35_000_000 * n as u64, 0)
            .saturating_add(T::DbWeight::get().reads(3 + n as u64))
            .saturating_add(T::DbWeight::get().writes(2 + 2 * n as u64))
    }

    fn submit_batch_compute_results(n: u32) -> Weight {
        Weight::from_parts(30_000_000 + 50_000_000 * n as u64, 0)
            .saturating_add(T::DbWeight::get().reads(1 + 2 * n as u64))
            .saturating_add(T::DbWeight::get().writes(3 * n as u64))
    }

    fn stake() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn request_unstake() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn withdraw_unstaked() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn add_allowed_mr_enclave() -> Weight {
        Weight::from_parts(15_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn remove_allowed_mr_enclave() -> Weight {
        Weight::from_parts(15_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn add_allowed_mr_signer() -> Weight {
        Weight::from_parts(15_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn remove_allowed_mr_signer() -> Weight {
        Weight::from_parts(15_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn set_audit_enabled() -> Weight {
        Weight::from_parts(10_000_000, 0)
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn process_timeouts(n: u32) -> Weight {
        Weight::from_parts(10_000_000 * n as u64, 0)
            .saturating_add(T::DbWeight::get().reads(n as u64))
            .saturating_add(T::DbWeight::get().writes(n as u64))
    }
}

/// 默认权重实现 (用于测试)
impl WeightInfo for () {
    fn register_tee_node() -> Weight {
        Weight::from_parts(50_000_000, 0)
    }

    fn update_attestation() -> Weight {
        Weight::from_parts(30_000_000, 0)
    }

    fn update_node_status() -> Weight {
        Weight::from_parts(15_000_000, 0)
    }

    fn deregister_tee_node() -> Weight {
        Weight::from_parts(20_000_000, 0)
    }

    fn submit_compute_request() -> Weight {
        Weight::from_parts(40_000_000, 0)
    }

    fn submit_compute_result() -> Weight {
        Weight::from_parts(60_000_000, 0)
    }

    fn cancel_compute_request() -> Weight {
        Weight::from_parts(25_000_000, 0)
    }

    fn submit_batch_compute_requests(n: u32) -> Weight {
        Weight::from_parts(30_000_000 + 35_000_000 * n as u64, 0)
    }

    fn submit_batch_compute_results(n: u32) -> Weight {
        Weight::from_parts(30_000_000 + 50_000_000 * n as u64, 0)
    }

    fn stake() -> Weight {
        Weight::from_parts(35_000_000, 0)
    }

    fn request_unstake() -> Weight {
        Weight::from_parts(25_000_000, 0)
    }

    fn withdraw_unstaked() -> Weight {
        Weight::from_parts(30_000_000, 0)
    }

    fn add_allowed_mr_enclave() -> Weight {
        Weight::from_parts(15_000_000, 0)
    }

    fn remove_allowed_mr_enclave() -> Weight {
        Weight::from_parts(15_000_000, 0)
    }

    fn add_allowed_mr_signer() -> Weight {
        Weight::from_parts(15_000_000, 0)
    }

    fn remove_allowed_mr_signer() -> Weight {
        Weight::from_parts(15_000_000, 0)
    }

    fn set_audit_enabled() -> Weight {
        Weight::from_parts(10_000_000, 0)
    }

    fn process_timeouts(n: u32) -> Weight {
        Weight::from_parts(10_000_000 * n as u64, 0)
    }
}
