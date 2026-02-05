//! 基准实现（v2 宏）。覆盖主要外部调用，便于自动生成权重。

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_support::{traits::Get, BoundedVec};
use frame_system::RawOrigin;
use sp_core::H256;

#[benchmarks]
mod benches {
    use super::*;

    #[benchmark]
    fn commit() {
        let caller: T::AccountId = whitelisted_caller();
        let domain: u8 = 1;
        let target: u64 = 1;
        let make_cid = || -> BoundedVec<u8, T::MaxCidLen> {
            BoundedVec::try_from(Vec::from("bafy...".as_bytes())).unwrap()
        };
        let imgs = vec![make_cid(); 2];
        let vids = vec![];
        let docs = vec![];
        let memo: Option<BoundedVec<u8, T::MaxMemoLen>> = None;
        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            domain,
            target,
            imgs,
            vids,
            docs,
            memo,
        );
    }

    #[benchmark]
    fn commit_hash() {
        let caller: T::AccountId = whitelisted_caller();
        let ns = T::EvidenceNsBytes::get();
        let subject: u64 = 1;
        let hash_h256 = H256::repeat_byte(7);
        let memo: Option<BoundedVec<u8, T::MaxMemoLen>> = None;
        #[extrinsic_call]
        _(RawOrigin::Signed(caller), ns, subject, hash_h256, memo);
    }

    #[benchmark]
    fn link() {
        let caller: T::AccountId = whitelisted_caller();
        // 先提交一条
        let ns = T::EvidenceNsBytes::get();
        let subject: u64 = 42;
        let hash_h256 = H256::repeat_byte(9);
        let memo: Option<BoundedVec<u8, T::MaxMemoLen>> = None;
        Pallet::<T>::commit_hash(
            RawOrigin::Signed(caller.clone()).into(),
            ns,
            subject,
            hash_h256,
            memo,
        )
        .unwrap();
        let id = NextEvidenceId::<T>::get().saturating_sub(1);
        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 1u8, 1u64, id);
    }

    #[benchmark]
    fn link_by_ns() {
        let caller: T::AccountId = whitelisted_caller();
        let ns = T::EvidenceNsBytes::get();
        let subject: u64 = 43;
        let hash_h256 = H256::repeat_byte(11);
        let memo: Option<BoundedVec<u8, T::MaxMemoLen>> = None;
        Pallet::<T>::commit_hash(
            RawOrigin::Signed(caller.clone()).into(),
            ns,
            subject,
            hash_h256,
            memo,
        )
        .unwrap();
        let id = NextEvidenceId::<T>::get().saturating_sub(1);
        #[extrinsic_call]
        _(RawOrigin::Signed(caller), ns, subject, id);
    }

    #[benchmark]
    fn unlink() {
        let caller: T::AccountId = whitelisted_caller();
        let ns = T::EvidenceNsBytes::get();
        let subject: u64 = 44;
        let hash_h256 = H256::repeat_byte(12);
        let memo: Option<BoundedVec<u8, T::MaxMemoLen>> = None;
        Pallet::<T>::commit_hash(
            RawOrigin::Signed(caller.clone()).into(),
            ns,
            subject,
            hash_h256,
            memo,
        )
        .unwrap();
        let id = NextEvidenceId::<T>::get().saturating_sub(1);
        Pallet::<T>::link(RawOrigin::Signed(caller.clone()).into(), 1, 1, id).unwrap();
        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 1u8, 1u64, id);
    }

    #[benchmark]
    fn unlink_by_ns() {
        let caller: T::AccountId = whitelisted_caller();
        let ns = T::EvidenceNsBytes::get();
        let subject: u64 = 45;
        let hash_h256 = H256::repeat_byte(13);
        let memo: Option<BoundedVec<u8, T::MaxMemoLen>> = None;
        Pallet::<T>::commit_hash(
            RawOrigin::Signed(caller.clone()).into(),
            ns,
            subject,
            hash_h256,
            memo,
        )
        .unwrap();
        let id = NextEvidenceId::<T>::get().saturating_sub(1);
        Pallet::<T>::link_by_ns(RawOrigin::Signed(caller.clone()).into(), ns, subject, id).unwrap();
        #[extrinsic_call]
        _(RawOrigin::Signed(caller), ns, subject, id);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
