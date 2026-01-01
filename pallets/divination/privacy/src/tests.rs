//! # 统一隐私授权模块 - 单元测试
//!
//! 测试覆盖：
//! 1. 密钥管理
//! 2. 服务提供者管理
//! 3. 加密记录管理
//! 4. 授权管理
//! 5. 悬赏授权集成

use crate::{mock::*, types::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use pallet_divination_common::DivinationType;

// ============================================================================
// 密钥管理测试
// ============================================================================

mod key_management {
    use super::*;

    #[test]
    fn register_encryption_key_works() {
        new_test_ext().execute_with(|| {
            let public_key = test_public_key(1);

            // 注册公钥
            assert_ok!(Privacy::register_encryption_key(
                RuntimeOrigin::signed(ALICE),
                public_key
            ));

            // 验证存储
            let info = Privacy::user_encryption_keys(ALICE).unwrap();
            assert_eq!(info.public_key, public_key);
            assert_eq!(info.registered_at, 1);

            // 验证事件
            System::assert_last_event(
                Event::EncryptionKeyRegistered {
                    account: ALICE,
                    public_key,
                }
                .into(),
            );
        });
    }

    #[test]
    fn register_encryption_key_fails_if_already_registered() {
        new_test_ext().execute_with(|| {
            let public_key = test_public_key(1);

            // 第一次注册成功
            assert_ok!(Privacy::register_encryption_key(
                RuntimeOrigin::signed(ALICE),
                public_key
            ));

            // 第二次注册失败
            assert_noop!(
                Privacy::register_encryption_key(RuntimeOrigin::signed(ALICE), public_key),
                Error::<Test>::EncryptionKeyAlreadyRegistered
            );
        });
    }

    #[test]
    fn register_encryption_key_fails_with_invalid_key() {
        new_test_ext().execute_with(|| {
            let zero_key = [0u8; 32];

            // 全零公钥无效
            assert_noop!(
                Privacy::register_encryption_key(RuntimeOrigin::signed(ALICE), zero_key),
                Error::<Test>::InvalidPublicKey
            );
        });
    }

    #[test]
    fn update_encryption_key_works() {
        new_test_ext().execute_with(|| {
            let old_key = test_public_key(1);
            let new_key = test_public_key(2);

            // 先注册
            assert_ok!(Privacy::register_encryption_key(
                RuntimeOrigin::signed(ALICE),
                old_key
            ));

            // 更新
            assert_ok!(Privacy::update_encryption_key(
                RuntimeOrigin::signed(ALICE),
                new_key
            ));

            // 验证
            let info = Privacy::user_encryption_keys(ALICE).unwrap();
            assert_eq!(info.public_key, new_key);

            // 验证事件
            System::assert_last_event(
                Event::EncryptionKeyUpdated {
                    account: ALICE,
                    old_key,
                    new_key,
                }
                .into(),
            );
        });
    }

    #[test]
    fn update_encryption_key_fails_if_not_registered() {
        new_test_ext().execute_with(|| {
            let new_key = test_public_key(2);

            // 未注册时更新失败
            assert_noop!(
                Privacy::update_encryption_key(RuntimeOrigin::signed(ALICE), new_key),
                Error::<Test>::EncryptionKeyNotRegistered
            );
        });
    }
}

// ============================================================================
// 服务提供者管理测试
// ============================================================================

mod provider_management {
    use super::*;

    #[test]
    fn register_provider_works() {
        new_test_ext().execute_with(|| {
            let public_key = test_public_key(10);

            // 注册为命理师
            assert_ok!(Privacy::register_provider(
                RuntimeOrigin::signed(MASTER),
                ServiceProviderType::MingLiShi,
                public_key
            ));

            // 验证存储
            let provider = Privacy::service_providers(MASTER).unwrap();
            assert_eq!(provider.provider_type, ServiceProviderType::MingLiShi);
            assert_eq!(provider.public_key, public_key);
            assert_eq!(provider.reputation, 50);
            assert!(provider.is_active);

            // 验证类型索引
            let providers = Privacy::providers_by_type(ServiceProviderType::MingLiShi);
            assert!(providers.contains(&MASTER));

            // 验证事件
            System::assert_last_event(
                Event::ProviderRegistered {
                    account: MASTER,
                    provider_type: ServiceProviderType::MingLiShi,
                }
                .into(),
            );
        });
    }

    #[test]
    fn register_provider_fails_if_already_registered() {
        new_test_ext().execute_with(|| {
            let public_key = test_public_key(10);

            // 第一次注册成功
            assert_ok!(Privacy::register_provider(
                RuntimeOrigin::signed(MASTER),
                ServiceProviderType::MingLiShi,
                public_key
            ));

            // 第二次注册失败
            assert_noop!(
                Privacy::register_provider(
                    RuntimeOrigin::signed(MASTER),
                    ServiceProviderType::AiService,
                    public_key
                ),
                Error::<Test>::AlreadyRegisteredAsProvider
            );
        });
    }

    #[test]
    fn set_provider_active_works() {
        new_test_ext().execute_with(|| {
            let public_key = test_public_key(10);

            // 注册
            assert_ok!(Privacy::register_provider(
                RuntimeOrigin::signed(MASTER),
                ServiceProviderType::MingLiShi,
                public_key
            ));

            // 设置为不活跃
            assert_ok!(Privacy::set_provider_active(
                RuntimeOrigin::signed(MASTER),
                false
            ));

            // 验证
            let provider = Privacy::service_providers(MASTER).unwrap();
            assert!(!provider.is_active);

            // 验证事件
            System::assert_last_event(
                Event::ProviderStatusChanged {
                    account: MASTER,
                    is_active: false,
                }
                .into(),
            );
        });
    }

    #[test]
    fn unregister_provider_works() {
        new_test_ext().execute_with(|| {
            let public_key = test_public_key(10);

            // 注册
            assert_ok!(Privacy::register_provider(
                RuntimeOrigin::signed(MASTER),
                ServiceProviderType::MingLiShi,
                public_key
            ));

            // 注销
            assert_ok!(Privacy::unregister_provider(RuntimeOrigin::signed(MASTER)));

            // 验证已删除
            assert!(Privacy::service_providers(MASTER).is_none());

            // 验证从类型索引中移除
            let providers = Privacy::providers_by_type(ServiceProviderType::MingLiShi);
            assert!(!providers.contains(&MASTER));

            // 验证事件
            System::assert_last_event(Event::ProviderUnregistered { account: MASTER }.into());
        });
    }
}

// ============================================================================
// 加密记录管理测试
// ============================================================================

mod encrypted_record {
    use super::*;

    #[test]
    fn create_encrypted_record_works() {
        new_test_ext().execute_with(|| {
            let encrypted_data = test_encrypted_data(256);
            let nonce = test_nonce();
            let auth_tag = test_auth_tag();
            let data_hash = test_data_hash();
            let owner_key = test_encrypted_key(1);

            // 创建加密记录（使用 Partial 模式测试授权功能）
            assert_ok!(Privacy::create_encrypted_record(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Bazi,
                1,
                PrivacyMode::Partial,
                encrypted_data.clone(),
                nonce,
                auth_tag,
                data_hash,
                owner_key,
            ));

            // 验证记录存储
            let record = Privacy::encrypted_records(DivinationType::Bazi, 1).unwrap();
            assert_eq!(record.owner, ALICE);
            assert_eq!(record.privacy_mode, PrivacyMode::Partial);
            assert_eq!(record.nonce, nonce);

            // 验证用户索引
            let user_records = Privacy::user_encrypted_records(ALICE, DivinationType::Bazi);
            assert!(user_records.contains(&1));

            // 验证所有者授权
            let auth = Privacy::authorizations((DivinationType::Bazi, 1, ALICE)).unwrap();
            assert_eq!(auth.role, AccessRole::Owner);

            // 验证事件
            System::assert_last_event(
                Event::EncryptedRecordCreated {
                    divination_type: DivinationType::Bazi,
                    result_id: 1,
                    owner: ALICE,
                    privacy_mode: PrivacyMode::Partial,
                }
                .into(),
            );
        });
    }

    #[test]
    fn create_encrypted_record_fails_if_exists() {
        new_test_ext().execute_with(|| {
            let encrypted_data = test_encrypted_data(256);
            let nonce = test_nonce();
            let auth_tag = test_auth_tag();
            let data_hash = test_data_hash();
            let owner_key = test_encrypted_key(1);

            // 第一次创建成功
            assert_ok!(Privacy::create_encrypted_record(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Bazi,
                1,
                PrivacyMode::Partial,
                encrypted_data.clone(),
                nonce,
                auth_tag,
                data_hash,
                owner_key.clone(),
            ));

            // 第二次创建失败
            assert_noop!(
                Privacy::create_encrypted_record(
                    RuntimeOrigin::signed(ALICE),
                    DivinationType::Bazi,
                    1,
                    PrivacyMode::Partial,
                    encrypted_data,
                    nonce,
                    auth_tag,
                    data_hash,
                    owner_key,
                ),
                Error::<Test>::EncryptedRecordAlreadyExists
            );
        });
    }

    #[test]
    fn change_privacy_mode_works() {
        new_test_ext().execute_with(|| {
            // 创建记录
            assert_ok!(Privacy::create_encrypted_record(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Bazi,
                1,
                PrivacyMode::Private,
                test_encrypted_data(256),
                test_nonce(),
                test_auth_tag(),
                test_data_hash(),
                test_encrypted_key(1),
            ));

            // 更改隐私模式
            assert_ok!(Privacy::change_privacy_mode(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Bazi,
                1,
                PrivacyMode::Public,
            ));

            // 验证
            let record = Privacy::encrypted_records(DivinationType::Bazi, 1).unwrap();
            assert_eq!(record.privacy_mode, PrivacyMode::Public);

            // 验证事件
            System::assert_last_event(
                Event::PrivacyModeChanged {
                    divination_type: DivinationType::Bazi,
                    result_id: 1,
                    old_mode: PrivacyMode::Private,
                    new_mode: PrivacyMode::Public,
                }
                .into(),
            );
        });
    }

    #[test]
    fn change_privacy_mode_fails_if_not_owner() {
        new_test_ext().execute_with(|| {
            // ALICE 创建记录
            assert_ok!(Privacy::create_encrypted_record(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Bazi,
                1,
                PrivacyMode::Private,
                test_encrypted_data(256),
                test_nonce(),
                test_auth_tag(),
                test_data_hash(),
                test_encrypted_key(1),
            ));

            // BOB 尝试更改失败
            assert_noop!(
                Privacy::change_privacy_mode(
                    RuntimeOrigin::signed(BOB),
                    DivinationType::Bazi,
                    1,
                    PrivacyMode::Public,
                ),
                Error::<Test>::NotRecordOwner
            );
        });
    }

    #[test]
    fn delete_encrypted_record_works() {
        new_test_ext().execute_with(|| {
            // 创建记录
            assert_ok!(Privacy::create_encrypted_record(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Bazi,
                1,
                PrivacyMode::Partial,
                test_encrypted_data(256),
                test_nonce(),
                test_auth_tag(),
                test_data_hash(),
                test_encrypted_key(1),
            ));

            // 删除记录
            assert_ok!(Privacy::delete_encrypted_record(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Bazi,
                1,
            ));

            // 验证已删除
            assert!(Privacy::encrypted_records(DivinationType::Bazi, 1).is_none());

            // 验证用户索引已清理
            let user_records = Privacy::user_encrypted_records(ALICE, DivinationType::Bazi);
            assert!(!user_records.contains(&1));

            // 验证事件
            System::assert_last_event(
                Event::EncryptedRecordDeleted {
                    divination_type: DivinationType::Bazi,
                    result_id: 1,
                }
                .into(),
            );
        });
    }
}

// ============================================================================
// 授权管理测试
// ============================================================================

mod authorization {
    use super::*;

    fn setup_record() {
        assert_ok!(Privacy::create_encrypted_record(
            RuntimeOrigin::signed(ALICE),
            DivinationType::Bazi,
            1,
            PrivacyMode::Partial,
            test_encrypted_data(256),
            test_nonce(),
            test_auth_tag(),
            test_data_hash(),
            test_encrypted_key(1),
        ));
    }

    #[test]
    fn grant_access_works() {
        new_test_ext().execute_with(|| {
            setup_record();

            let encrypted_key = test_encrypted_key(2);

            // 授权给 MASTER
            assert_ok!(Privacy::grant_access(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Bazi,
                1,
                MASTER,
                encrypted_key,
                AccessRole::Master,
                AccessScope::CanComment,
                100, // 过期区块
            ));

            // 验证授权存储
            let auth = Privacy::authorizations((DivinationType::Bazi, 1, MASTER)).unwrap();
            assert_eq!(auth.role, AccessRole::Master);
            assert_eq!(auth.scope, AccessScope::CanComment);
            assert_eq!(auth.expires_at, 100);

            // 验证授权列表
            let grantees = Privacy::record_grantees(DivinationType::Bazi, 1);
            assert!(grantees.contains(&MASTER));

            // 验证提供者授权列表
            let grants = Privacy::provider_grants(MASTER);
            assert!(grants
                .iter()
                .any(|k| k.divination_type == DivinationType::Bazi && k.result_id == 1));

            // 验证事件
            System::assert_last_event(
                Event::AccessGranted {
                    divination_type: DivinationType::Bazi,
                    result_id: 1,
                    grantee: MASTER,
                    role: AccessRole::Master,
                    scope: AccessScope::CanComment,
                    expires_at: 100,
                }
                .into(),
            );
        });
    }

    #[test]
    fn grant_access_fails_if_already_authorized() {
        new_test_ext().execute_with(|| {
            setup_record();

            let encrypted_key = test_encrypted_key(2);

            // 第一次授权成功
            assert_ok!(Privacy::grant_access(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Bazi,
                1,
                MASTER,
                encrypted_key.clone(),
                AccessRole::Master,
                AccessScope::CanComment,
                100,
            ));

            // 第二次授权失败
            assert_noop!(
                Privacy::grant_access(
                    RuntimeOrigin::signed(ALICE),
                    DivinationType::Bazi,
                    1,
                    MASTER,
                    encrypted_key,
                    AccessRole::Master,
                    AccessScope::FullAccess,
                    200,
                ),
                Error::<Test>::AuthorizationAlreadyExists
            );
        });
    }

    #[test]
    fn revoke_access_works() {
        new_test_ext().execute_with(|| {
            setup_record();

            // 先授权
            assert_ok!(Privacy::grant_access(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Bazi,
                1,
                MASTER,
                test_encrypted_key(2),
                AccessRole::Master,
                AccessScope::CanComment,
                100,
            ));

            // 撤销授权
            assert_ok!(Privacy::revoke_access(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Bazi,
                1,
                MASTER,
            ));

            // 验证已撤销
            assert!(Privacy::authorizations((DivinationType::Bazi, 1, MASTER)).is_none());

            // 验证从授权列表中移除
            let grantees = Privacy::record_grantees(DivinationType::Bazi, 1);
            assert!(!grantees.contains(&MASTER));

            // 验证事件
            System::assert_last_event(
                Event::AccessRevoked {
                    divination_type: DivinationType::Bazi,
                    result_id: 1,
                    grantee: MASTER,
                }
                .into(),
            );
        });
    }

    #[test]
    fn revoke_access_fails_for_owner() {
        new_test_ext().execute_with(|| {
            setup_record();

            // 尝试撤销所有者授权失败
            assert_noop!(
                Privacy::revoke_access(
                    RuntimeOrigin::signed(ALICE),
                    DivinationType::Bazi,
                    1,
                    ALICE,
                ),
                Error::<Test>::CannotRevokeOwnerAccess
            );
        });
    }

    #[test]
    fn revoke_all_access_works() {
        new_test_ext().execute_with(|| {
            setup_record();

            // 授权多个账户
            assert_ok!(Privacy::grant_access(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Bazi,
                1,
                MASTER,
                test_encrypted_key(2),
                AccessRole::Master,
                AccessScope::CanComment,
                100,
            ));

            assert_ok!(Privacy::grant_access(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Bazi,
                1,
                BOB,
                test_encrypted_key(3),
                AccessRole::Family,
                AccessScope::ReadOnly,
                100,
            ));

            // 撤销所有授权
            assert_ok!(Privacy::revoke_all_access(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Bazi,
                1,
            ));

            // 验证 MASTER 和 BOB 的授权已撤销
            assert!(Privacy::authorizations((DivinationType::Bazi, 1, MASTER)).is_none());
            assert!(Privacy::authorizations((DivinationType::Bazi, 1, BOB)).is_none());

            // 验证所有者授权仍在
            assert!(Privacy::authorizations((DivinationType::Bazi, 1, ALICE)).is_some());

            // 验证事件
            System::assert_last_event(
                Event::AllAccessRevoked {
                    divination_type: DivinationType::Bazi,
                    result_id: 1,
                    count: 2,
                }
                .into(),
            );
        });
    }

    #[test]
    fn update_access_scope_works() {
        new_test_ext().execute_with(|| {
            setup_record();

            // 先授权
            assert_ok!(Privacy::grant_access(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Bazi,
                1,
                MASTER,
                test_encrypted_key(2),
                AccessRole::Master,
                AccessScope::ReadOnly,
                100,
            ));

            // 更新范围
            assert_ok!(Privacy::update_access_scope(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Bazi,
                1,
                MASTER,
                AccessScope::FullAccess,
            ));

            // 验证
            let auth = Privacy::authorizations((DivinationType::Bazi, 1, MASTER)).unwrap();
            assert_eq!(auth.scope, AccessScope::FullAccess);

            // 验证事件
            System::assert_last_event(
                Event::AccessScopeUpdated {
                    divination_type: DivinationType::Bazi,
                    result_id: 1,
                    grantee: MASTER,
                    new_scope: AccessScope::FullAccess,
                }
                .into(),
            );
        });
    }
}

// ============================================================================
// 悬赏授权测试
// ============================================================================

mod bounty_authorization {
    use super::*;

    fn setup_record() {
        assert_ok!(Privacy::create_encrypted_record(
            RuntimeOrigin::signed(ALICE),
            DivinationType::Bazi,
            1,
            PrivacyMode::Partial,
            test_encrypted_data(256),
            test_nonce(),
            test_auth_tag(),
            test_data_hash(),
            test_encrypted_key(1),
        ));
    }

    #[test]
    fn create_bounty_authorization_works() {
        new_test_ext().execute_with(|| {
            setup_record();

            // 创建悬赏授权配置
            assert_ok!(Privacy::create_bounty_authorization(
                RuntimeOrigin::signed(ALICE),
                100, // bounty_id
                DivinationType::Bazi,
                1,
                1000, // expires_at
                true, // auto_authorize
            ));

            // 验证存储
            let auth_info = Privacy::bounty_auth_info(100).unwrap();
            assert_eq!(auth_info.divination_type, DivinationType::Bazi);
            assert_eq!(auth_info.result_id, 1);
            assert_eq!(auth_info.expires_at, 1000);
            assert!(auth_info.auto_authorize);

            // 验证事件
            System::assert_last_event(
                Event::BountyAuthorizationCreated {
                    bounty_id: 100,
                    divination_type: DivinationType::Bazi,
                    result_id: 1,
                    auto_authorize: true,
                }
                .into(),
            );
        });
    }

    #[test]
    fn authorize_bounty_answerer_works() {
        new_test_ext().execute_with(|| {
            setup_record();

            // 创建悬赏授权配置
            assert_ok!(Privacy::create_bounty_authorization(
                RuntimeOrigin::signed(ALICE),
                100,
                DivinationType::Bazi,
                1,
                1000,
                true,
            ));

            // 为回答者授权
            assert_ok!(Privacy::authorize_bounty_answerer(
                RuntimeOrigin::signed(ALICE),
                100,
                MASTER,
                test_encrypted_key(10),
            ));

            // 验证授权
            let auth = Privacy::authorizations((DivinationType::Bazi, 1, MASTER)).unwrap();
            assert_eq!(auth.role, AccessRole::BountyAnswerer);
            assert_eq!(auth.scope, AccessScope::ReadOnly);
            assert_eq!(auth.bounty_id, Some(100));

            // 验证悬赏授权列表
            let answerers = Privacy::bounty_authorizations(100);
            assert!(answerers.contains(&MASTER));

            // 验证事件
            System::assert_last_event(
                Event::BountyAnswererAuthorized {
                    bounty_id: 100,
                    answerer: MASTER,
                }
                .into(),
            );
        });
    }

    #[test]
    fn revoke_bounty_authorizations_works() {
        new_test_ext().execute_with(|| {
            setup_record();

            // 创建悬赏授权配置
            assert_ok!(Privacy::create_bounty_authorization(
                RuntimeOrigin::signed(ALICE),
                100,
                DivinationType::Bazi,
                1,
                1000,
                true,
            ));

            // 为多个回答者授权
            assert_ok!(Privacy::authorize_bounty_answerer(
                RuntimeOrigin::signed(ALICE),
                100,
                MASTER,
                test_encrypted_key(10),
            ));

            assert_ok!(Privacy::authorize_bounty_answerer(
                RuntimeOrigin::signed(ALICE),
                100,
                BOB,
                test_encrypted_key(11),
            ));

            // 撤销所有悬赏授权
            assert_ok!(Privacy::revoke_bounty_authorizations(
                RuntimeOrigin::signed(ALICE),
                100,
            ));

            // 验证授权已撤销
            assert!(Privacy::authorizations((DivinationType::Bazi, 1, MASTER)).is_none());
            assert!(Privacy::authorizations((DivinationType::Bazi, 1, BOB)).is_none());

            // 验证悬赏授权信息已删除
            assert!(Privacy::bounty_auth_info(100).is_none());

            // 验证悬赏授权列表已清空
            let answerers = Privacy::bounty_authorizations(100);
            assert!(answerers.is_empty());

            // 验证事件
            System::assert_last_event(
                Event::BountyAuthorizationsRevoked {
                    bounty_id: 100,
                    count: 2,
                }
                .into(),
            );
        });
    }
}

// ============================================================================
// Trait 实现测试
// ============================================================================

mod trait_implementation {
    use super::*;
    use crate::traits::{BountyPrivacy, DivinationPrivacy};

    fn setup_record_with_authorizations() {
        // 创建记录
        assert_ok!(Privacy::create_encrypted_record(
            RuntimeOrigin::signed(ALICE),
            DivinationType::Bazi,
            1,
            PrivacyMode::Partial,
            test_encrypted_data(256),
            test_nonce(),
            test_auth_tag(),
            test_data_hash(),
            test_encrypted_key(1),
        ));

        // 授权 MASTER
        assert_ok!(Privacy::grant_access(
            RuntimeOrigin::signed(ALICE),
            DivinationType::Bazi,
            1,
            MASTER,
            test_encrypted_key(2),
            AccessRole::Master,
            AccessScope::CanComment,
            0, // 永不过期
        ));
    }

    #[test]
    fn divination_privacy_trait_works() {
        new_test_ext().execute_with(|| {
            setup_record_with_authorizations();

            // is_encrypted
            assert!(Privacy::is_encrypted(DivinationType::Bazi, 1));
            assert!(!Privacy::is_encrypted(DivinationType::Bazi, 999));

            // get_privacy_mode
            assert_eq!(
                Privacy::get_privacy_mode(DivinationType::Bazi, 1),
                Some(PrivacyMode::Partial)
            );

            // has_access
            assert!(Privacy::has_access(DivinationType::Bazi, 1, &ALICE)); // 所有者
            assert!(Privacy::has_access(DivinationType::Bazi, 1, &MASTER)); // 被授权者
            assert!(!Privacy::has_access(DivinationType::Bazi, 1, &BOB)); // 未授权

            // get_access_role
            assert_eq!(
                Privacy::get_access_role(DivinationType::Bazi, 1, &ALICE),
                Some(AccessRole::Owner)
            );
            assert_eq!(
                Privacy::get_access_role(DivinationType::Bazi, 1, &MASTER),
                Some(AccessRole::Master)
            );
            assert_eq!(
                Privacy::get_access_role(DivinationType::Bazi, 1, &BOB),
                None
            );

            // get_grantees
            let grantees = Privacy::get_grantees(DivinationType::Bazi, 1);
            assert!(grantees.contains(&ALICE));
            assert!(grantees.contains(&MASTER));

            // get_owner
            assert_eq!(
                Privacy::get_owner(DivinationType::Bazi, 1),
                Some(ALICE)
            );
        });
    }

    #[test]
    fn bounty_privacy_trait_works() {
        new_test_ext().execute_with(|| {
            setup_record_with_authorizations();

            // is_bounty_encrypted
            assert!(Privacy::is_bounty_encrypted(DivinationType::Bazi, 1));

            // can_answer_bounty
            assert!(Privacy::can_answer_bounty(
                DivinationType::Bazi,
                1,
                &MASTER
            )); // 已授权
            assert!(!Privacy::can_answer_bounty(DivinationType::Bazi, 1, &BOB)); // 未授权

            // bounty_requires_authorization
            assert!(Privacy::bounty_requires_authorization(DivinationType::Bazi, 1));

            // 创建悬赏授权
            assert_ok!(Privacy::create_bounty_authorization(
                RuntimeOrigin::signed(ALICE),
                100,
                DivinationType::Bazi,
                1,
                1000,
                true,
            ));

            // get_bounty_authorization_expiry
            assert_eq!(Privacy::get_bounty_authorization_expiry(100), Some(1000));

            // is_auto_authorize_enabled
            assert!(Privacy::is_auto_authorize_enabled(100));
        });
    }

    #[test]
    fn public_record_access() {
        new_test_ext().execute_with(|| {
            // 创建公开记录
            assert_ok!(Privacy::create_encrypted_record(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Meihua,
                1,
                PrivacyMode::Public,
                test_encrypted_data(256),
                test_nonce(),
                test_auth_tag(),
                test_data_hash(),
                vec![], // 公开记录不需要所有者密钥
            ));

            // 所有人都可以访问公开记录
            assert!(Privacy::has_access(DivinationType::Meihua, 1, &ALICE));
            assert!(Privacy::has_access(DivinationType::Meihua, 1, &BOB));
            assert!(Privacy::has_access(DivinationType::Meihua, 1, &CHARLIE));

            // 公开记录不需要悬赏授权
            assert!(!Privacy::bounty_requires_authorization(
                DivinationType::Meihua,
                1
            ));
        });
    }

    #[test]
    fn private_record_access() {
        new_test_ext().execute_with(|| {
            // 创建私密记录
            assert_ok!(Privacy::create_encrypted_record(
                RuntimeOrigin::signed(ALICE),
                DivinationType::Liuyao,
                1,
                PrivacyMode::Private,
                test_encrypted_data(256),
                test_nonce(),
                test_auth_tag(),
                test_data_hash(),
                test_encrypted_key(1),
            ));

            // 只有所有者可以访问
            assert!(Privacy::has_access(DivinationType::Liuyao, 1, &ALICE));
            assert!(!Privacy::has_access(DivinationType::Liuyao, 1, &BOB));

            // 私密记录需要悬赏授权
            assert!(Privacy::bounty_requires_authorization(
                DivinationType::Liuyao,
                1
            ));
        });
    }
}
