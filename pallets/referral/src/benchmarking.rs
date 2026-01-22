//! Benchmarking setup for pallet-referral

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use frame_support::BoundedVec;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn bind_sponsor() {
        let caller: T::AccountId = whitelisted_caller();
        
        // 创建推荐人账户并设置推荐码
        let sponsor: T::AccountId = account("sponsor", 0, 0);
        let code: BoundedVec<u8, T::MaxCodeLen> = 
            b"TESTCODE".to_vec().try_into().unwrap();
        AccountByCode::<T>::insert(&code, &sponsor);
        CodeByAccount::<T>::insert(&sponsor, &code);

        #[extrinsic_call]
        bind_sponsor(RawOrigin::Signed(caller), b"TESTCODE".to_vec());
    }

    #[benchmark]
    fn claim_code() {
        let caller: T::AccountId = whitelisted_caller();
        
        // 确保调用者是有效会员（通过 MembershipProvider）
        // 注意：需要在 mock 中正确设置

        #[extrinsic_call]
        claim_code(RawOrigin::Signed(caller), b"MYCODE01".to_vec());
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test,
    );
}
