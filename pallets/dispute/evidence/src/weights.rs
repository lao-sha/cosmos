//! 权重实现（临时手写版）。
//! 说明：后续可用 benchmark 自动生成替换此文件。

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// 函数级中文注释：定义 evidence Pallet 所需权重函数接口。
pub trait WeightInfo {
    /// 提交明文 CID 列表，权重与三类媒体数量线性相关
    fn commit(n_imgs: u32, n_vids: u32, n_docs: u32) -> Weight;
    /// 提交承诺哈希（常数级）
    fn commit_hash() -> Weight;
    /// 链接既有证据（常数级）
    fn link() -> Weight;
    /// 命名空间链接既有证据（常数级）
    fn link_by_ns() -> Weight;
    /// 取消链接（常数级）
    fn unlink() -> Weight;
    /// 命名空间取消链接（常数级）
    fn unlink_by_ns() -> Weight;
}

/// 函数级中文注释：参照 Substrate 推荐的 RocksDb 权重，提供通用实现。
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn commit(n_imgs: u32, n_vids: u32, n_docs: u32) -> Weight {
        // 读写：NextEvidenceId(w), Evidences(w), EvidenceByTarget(w) = 3 writes
        // 验证：遍历所有 CID 做去重与字符校验，按 O(n) 估算
        let per_cid_cost = 2_000_000; // 粗略：2ms/项（Wasm 估算），后续以基准替换
        let n_total = n_imgs.saturating_add(n_vids).saturating_add(n_docs);
        Weight::from_parts(8_000_000, 0)
            .saturating_add(
                Weight::from_parts(per_cid_cost as u64, 0).saturating_mul(n_total as u64),
            )
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    fn commit_hash() -> Weight {
        // 读写：NextEvidenceId(w), Evidences(w), EvidenceByNs(w), CommitIndex(w) = 4 writes
        // 读：CommitIndex(r) = 1 read
        Weight::from_parts(6_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
    }
    fn link() -> Weight {
        // 读：Evidences(r)；写：EvidenceByTarget(w)
        Weight::from_parts(4_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    fn link_by_ns() -> Weight {
        // 读：Evidences(r)；写：EvidenceByNs(w)
        Weight::from_parts(4_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    fn unlink() -> Weight {
        // 读：Evidences(r)；写：EvidenceByTarget(w)
        Weight::from_parts(4_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    fn unlink_by_ns() -> Weight {
        // 读：Evidences(r)；写：EvidenceByNs(w)
        Weight::from_parts(4_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
}

/// 函数级中文注释：为测试与未接线场景提供默认实现（基于 RocksDbWeight）。
impl WeightInfo for () {
    fn commit(n_imgs: u32, n_vids: u32, n_docs: u32) -> Weight {
        let per_cid_cost = 2_000_000u64;
        let n_total = n_imgs as u64 + n_vids as u64 + n_docs as u64;
        Weight::from_parts(8_000_000, 0)
            .saturating_add(Weight::from_parts(per_cid_cost, 0).saturating_mul(n_total))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
    }
    fn commit_hash() -> Weight {
        Weight::from_parts(6_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(4_u64))
    }
    fn link() -> Weight {
        Weight::from_parts(4_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    fn link_by_ns() -> Weight {
        Weight::from_parts(4_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    fn unlink() -> Weight {
        Weight::from_parts(4_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    fn unlink_by_ns() -> Weight {
        Weight::from_parts(4_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
}
