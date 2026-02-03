//! KYC 模块测试

use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};

#[test]
fn register_provider_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(EntityKyc::register_provider(
            RuntimeOrigin::root(),
            PROVIDER,
            b"Test Provider".to_vec(),
            ProviderType::ThirdParty,
            KycLevel::Enhanced,
        ));

        let provider = Providers::<Test>::get(PROVIDER).unwrap();
        assert_eq!(provider.max_level, KycLevel::Enhanced);
        assert!(provider.active);
    });
}

#[test]
fn submit_kyc_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(EntityKyc::submit_kyc(
            RuntimeOrigin::signed(USER),
            KycLevel::Standard,
            b"QmKycData".to_vec(),
            *b"CN",
        ));

        let record = KycRecords::<Test>::get(USER).unwrap();
        assert_eq!(record.level, KycLevel::Standard);
        assert_eq!(record.status, KycStatus::Pending);
    });
}

#[test]
fn approve_kyc_works() {
    new_test_ext().execute_with(|| {
        // 注册提供者
        assert_ok!(EntityKyc::register_provider(
            RuntimeOrigin::root(),
            PROVIDER,
            b"Test Provider".to_vec(),
            ProviderType::ThirdParty,
            KycLevel::Enhanced,
        ));

        // 提交 KYC
        assert_ok!(EntityKyc::submit_kyc(
            RuntimeOrigin::signed(USER),
            KycLevel::Standard,
            b"QmKycData".to_vec(),
            *b"CN",
        ));

        // 批准 KYC
        assert_ok!(EntityKyc::approve_kyc(
            RuntimeOrigin::signed(PROVIDER),
            USER,
            20, // risk_score
        ));

        let record = KycRecords::<Test>::get(USER).unwrap();
        assert_eq!(record.status, KycStatus::Approved);
        assert_eq!(record.risk_score, 20);
    });
}

#[test]
fn get_kyc_level_works() {
    new_test_ext().execute_with(|| {
        // 未认证用户
        assert_eq!(Pallet::<Test>::get_kyc_level(&USER), KycLevel::None);

        // 注册提供者并批准 KYC
        assert_ok!(EntityKyc::register_provider(
            RuntimeOrigin::root(),
            PROVIDER,
            b"Provider".to_vec(),
            ProviderType::Internal,
            KycLevel::Standard,
        ));

        assert_ok!(EntityKyc::submit_kyc(
            RuntimeOrigin::signed(USER),
            KycLevel::Standard,
            b"data".to_vec(),
            *b"US",
        ));

        assert_ok!(EntityKyc::approve_kyc(
            RuntimeOrigin::signed(PROVIDER),
            USER,
            10,
        ));

        assert_eq!(Pallet::<Test>::get_kyc_level(&USER), KycLevel::Standard);
    });
}
