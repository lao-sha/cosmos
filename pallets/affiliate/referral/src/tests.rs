//! # Referral Pallet Tests
//!
//! 函数级详细中文注释：Referral Pallet 单元测试

use crate::{mock::*, Error, Event, pallet::*};
use frame_support::{assert_noop, assert_ok, BoundedVec};

// ========================================
// 推荐人绑定测试
// ========================================

#[test]
fn test_bind_sponsor_success() {
    new_test_ext().execute_with(|| {
        // 准备：为 Bob (2) 设置推荐码
        setup_code_for_account(2, b"BOBCODE1");

        // 执行：Alice (1) 绑定 Bob 为推荐人
        assert_ok!(Referral::bind_sponsor(
            RuntimeOrigin::signed(1),
            b"BOBCODE1".to_vec()
        ));

        // 验证：推荐关系已建立
        assert_eq!(Referral::sponsor_of(1), Some(2));

        // 验证：事件已发射
        System::assert_has_event(RuntimeEvent::Referral(Event::SponsorBound {
            who: 1,
            sponsor: 2,
        }));
    });
}

#[test]
fn test_bind_sponsor_already_bound() {
    new_test_ext().execute_with(|| {
        // 准备：设置推荐码
        setup_code_for_account(2, b"BOBCODE1");
        setup_code_for_account(3, b"CHARLIE1");

        // 执行：Alice 先绑定 Bob
        assert_ok!(Referral::bind_sponsor(
            RuntimeOrigin::signed(1),
            b"BOBCODE1".to_vec()
        ));

        // 验证：再次绑定失败
        assert_noop!(
            Referral::bind_sponsor(RuntimeOrigin::signed(1), b"CHARLIE1".to_vec()),
            Error::<Test>::AlreadyBound
        );
    });
}

#[test]
fn test_bind_sponsor_code_not_found() {
    new_test_ext().execute_with(|| {
        // 执行：使用不存在的推荐码
        assert_noop!(
            Referral::bind_sponsor(RuntimeOrigin::signed(1), b"NOTEXIST".to_vec()),
            Error::<Test>::CodeNotFound
        );
    });
}

#[test]
fn test_bind_sponsor_cannot_bind_self() {
    new_test_ext().execute_with(|| {
        // 准备：为 Alice 设置推荐码
        setup_code_for_account(1, b"ALICE001");

        // 执行：Alice 尝试绑定自己
        assert_noop!(
            Referral::bind_sponsor(RuntimeOrigin::signed(1), b"ALICE001".to_vec()),
            Error::<Test>::CannotBindSelf
        );
    });
}

#[test]
fn test_bind_sponsor_cycle_detection() {
    new_test_ext().execute_with(|| {
        // 准备：设置推荐码
        setup_code_for_account(1, b"ALICE001");
        setup_code_for_account(2, b"BOBCODE1");
        setup_code_for_account(3, b"CHARLIE1");

        // 建立链：Alice → Bob → Charlie
        assert_ok!(Referral::bind_sponsor(
            RuntimeOrigin::signed(1),
            b"BOBCODE1".to_vec()
        ));
        assert_ok!(Referral::bind_sponsor(
            RuntimeOrigin::signed(2),
            b"CHARLIE1".to_vec()
        ));

        // 验证：Charlie 尝试绑定 Alice（会形成循环）
        assert_noop!(
            Referral::bind_sponsor(RuntimeOrigin::signed(3), b"ALICE001".to_vec()),
            Error::<Test>::WouldCreateCycle
        );
    });
}

#[test]
fn test_bind_sponsor_code_too_short() {
    new_test_ext().execute_with(|| {
        // 准备：设置短推荐码（直接写入存储绕过验证）
        setup_code_for_account(2, b"ABC");

        // 执行：使用过短的推荐码
        assert_noop!(
            Referral::bind_sponsor(RuntimeOrigin::signed(1), b"ABC".to_vec()),
            Error::<Test>::CodeTooShort
        );
    });
}

// ========================================
// 推荐码认领测试
// ========================================

#[test]
fn test_claim_code_success() {
    new_test_ext().execute_with(|| {
        // 执行：Alice (有效会员) 认领推荐码
        assert_ok!(Referral::claim_code(
            RuntimeOrigin::signed(1),
            b"MYCODE01".to_vec()
        ));

        // 验证：推荐码已关联
        let code: BoundedVec<u8, MaxCodeLen> = b"MYCODE01".to_vec().try_into().unwrap();
        assert_eq!(Referral::account_by_code(&code), Some(1));
        assert_eq!(Referral::code_by_account(1), Some(code.clone()));

        // 验证：事件已发射
        System::assert_has_event(RuntimeEvent::Referral(Event::CodeClaimed {
            who: 1,
            code,
        }));
    });
}

#[test]
fn test_claim_code_not_member() {
    new_test_ext().execute_with(|| {
        // 执行：非会员 (901) 尝试认领推荐码
        assert_noop!(
            Referral::claim_code(RuntimeOrigin::signed(901), b"MYCODE01".to_vec()),
            Error::<Test>::NotMember
        );
    });
}

#[test]
fn test_claim_code_already_taken() {
    new_test_ext().execute_with(|| {
        // 准备：Alice 先认领
        assert_ok!(Referral::claim_code(
            RuntimeOrigin::signed(1),
            b"MYCODE01".to_vec()
        ));

        // 执行：Bob 尝试认领同一推荐码
        assert_noop!(
            Referral::claim_code(RuntimeOrigin::signed(2), b"MYCODE01".to_vec()),
            Error::<Test>::CodeAlreadyTaken
        );
    });
}

#[test]
fn test_claim_code_already_has_code() {
    new_test_ext().execute_with(|| {
        // 准备：Alice 先认领一个推荐码
        assert_ok!(Referral::claim_code(
            RuntimeOrigin::signed(1),
            b"MYCODE01".to_vec()
        ));

        // 执行：Alice 尝试认领另一个推荐码
        assert_noop!(
            Referral::claim_code(RuntimeOrigin::signed(1), b"MYCODE02".to_vec()),
            Error::<Test>::AlreadyHasCode
        );
    });
}

#[test]
fn test_claim_code_too_short() {
    new_test_ext().execute_with(|| {
        // 执行：使用过短的推荐码
        assert_noop!(
            Referral::claim_code(RuntimeOrigin::signed(1), b"ABC".to_vec()),
            Error::<Test>::CodeTooShort
        );
    });
}

#[test]
fn test_claim_code_too_long() {
    new_test_ext().execute_with(|| {
        // 执行：使用过长的推荐码（超过 32 字符）
        let long_code = vec![b'A'; 40];
        assert_noop!(
            Referral::claim_code(RuntimeOrigin::signed(1), long_code),
            Error::<Test>::CodeTooLong
        );
    });
}

// ========================================
// 推荐链查询测试
// ========================================

#[test]
fn test_get_referral_chain_empty() {
    new_test_ext().execute_with(|| {
        // 执行：查询未绑定推荐人的账户
        let chain = Referral::get_referral_chain(&1);

        // 验证：推荐链为空
        assert!(chain.is_empty());
    });
}

#[test]
fn test_get_referral_chain_single_level() {
    new_test_ext().execute_with(|| {
        // 准备：Alice → Bob
        setup_code_for_account(2, b"BOBCODE1");
        assert_ok!(Referral::bind_sponsor(
            RuntimeOrigin::signed(1),
            b"BOBCODE1".to_vec()
        ));

        // 执行：查询 Alice 的推荐链
        let chain = Referral::get_referral_chain(&1);

        // 验证：推荐链包含 Bob
        assert_eq!(chain, vec![2]);
    });
}

#[test]
fn test_get_referral_chain_multi_level() {
    new_test_ext().execute_with(|| {
        // 准备：建立链 1 → 2 → 3 → 4 → 5
        setup_code_for_account(2, b"BOBCODE1");
        setup_code_for_account(3, b"CHARLIE1");
        setup_code_for_account(4, b"DAVEXXXX");
        setup_code_for_account(5, b"EVEXXXXX");

        assert_ok!(Referral::bind_sponsor(RuntimeOrigin::signed(1), b"BOBCODE1".to_vec()));
        assert_ok!(Referral::bind_sponsor(RuntimeOrigin::signed(2), b"CHARLIE1".to_vec()));
        assert_ok!(Referral::bind_sponsor(RuntimeOrigin::signed(3), b"DAVEXXXX".to_vec()));
        assert_ok!(Referral::bind_sponsor(RuntimeOrigin::signed(4), b"EVEXXXXX".to_vec()));

        // 执行：查询账户 1 的推荐链
        let chain = Referral::get_referral_chain(&1);

        // 验证：推荐链正确
        assert_eq!(chain, vec![2, 3, 4, 5]);
    });
}

#[test]
fn test_get_referral_chain_max_depth() {
    new_test_ext().execute_with(|| {
        // 准备：建立 20 层推荐链（超过 MAX_REFERRAL_CHAIN = 15）
        for i in 2..=20 {
            let code = format!("CODE{:04}", i);
            setup_code_for_account(i, code.as_bytes());
        }

        // 建立链：1 → 2 → 3 → ... → 20
        for i in 1..20 {
            let code = format!("CODE{:04}", i + 1);
            assert_ok!(Referral::bind_sponsor(
                RuntimeOrigin::signed(i),
                code.into_bytes()
            ));
        }

        // 执行：查询账户 1 的推荐链
        let chain = Referral::get_referral_chain(&1);

        // 验证：推荐链最多 15 层
        assert_eq!(chain.len(), 15);
        assert_eq!(chain[0], 2);
        assert_eq!(chain[14], 16);
    });
}

// ========================================
// 循环检测测试
// ========================================

#[test]
fn test_would_create_cycle_no_cycle() {
    new_test_ext().execute_with(|| {
        // 准备：Bob 没有推荐人
        // 执行：检查 Alice 绑定 Bob 是否会形成循环
        let result = Referral::would_create_cycle(&1, &2);

        // 验证：不会形成循环
        assert!(!result);
    });
}

#[test]
fn test_would_create_cycle_direct_cycle() {
    new_test_ext().execute_with(|| {
        // 准备：Alice → Bob
        Sponsors::<Test>::insert(1, 2);

        // 执行：检查 Bob 绑定 Alice 是否会形成循环
        let result = Referral::would_create_cycle(&2, &1);

        // 验证：会形成循环
        assert!(result);
    });
}

#[test]
fn test_would_create_cycle_indirect_cycle() {
    new_test_ext().execute_with(|| {
        // 准备：Alice → Bob → Charlie
        Sponsors::<Test>::insert(1, 2);
        Sponsors::<Test>::insert(2, 3);

        // 执行：检查 Charlie 绑定 Alice 是否会形成循环
        let result = Referral::would_create_cycle(&3, &1);

        // 验证：会形成循环
        assert!(result);
    });
}

// ========================================
// 自动认领推荐码测试
// ========================================

#[test]
fn test_try_auto_claim_code_success() {
    new_test_ext().execute_with(|| {
        // 执行：为有效会员自动认领推荐码
        let result = Referral::try_auto_claim_code(&1);

        // 验证：认领成功
        assert!(result);
        assert!(Referral::code_by_account(1).is_some());
    });
}

#[test]
fn test_try_auto_claim_code_already_has_code() {
    new_test_ext().execute_with(|| {
        // 准备：先手动认领
        assert_ok!(Referral::claim_code(
            RuntimeOrigin::signed(1),
            b"MYCODE01".to_vec()
        ));

        // 执行：尝试自动认领
        let result = Referral::try_auto_claim_code(&1);

        // 验证：认领失败（已有推荐码）
        assert!(!result);
    });
}

#[test]
fn test_try_auto_claim_code_not_member() {
    new_test_ext().execute_with(|| {
        // 执行：为非会员自动认领推荐码
        let result = Referral::try_auto_claim_code(&901);

        // 验证：认领失败
        assert!(!result);
    });
}

// ========================================
// ReferralProvider Trait 测试
// ========================================

#[test]
fn test_referral_provider_get_sponsor() {
    new_test_ext().execute_with(|| {
        use crate::ReferralProvider;

        // 准备：Alice → Bob
        setup_code_for_account(2, b"BOBCODE1");
        assert_ok!(Referral::bind_sponsor(
            RuntimeOrigin::signed(1),
            b"BOBCODE1".to_vec()
        ));

        // 执行：通过 trait 查询
        let sponsor = <Referral as ReferralProvider<u64>>::get_sponsor(&1);

        // 验证
        assert_eq!(sponsor, Some(2));
    });
}

#[test]
fn test_referral_provider_get_referral_chain() {
    new_test_ext().execute_with(|| {
        use crate::ReferralProvider;

        // 准备：1 → 2 → 3
        setup_code_for_account(2, b"BOBCODE1");
        setup_code_for_account(3, b"CHARLIE1");
        assert_ok!(Referral::bind_sponsor(RuntimeOrigin::signed(1), b"BOBCODE1".to_vec()));
        assert_ok!(Referral::bind_sponsor(RuntimeOrigin::signed(2), b"CHARLIE1".to_vec()));

        // 执行：通过 trait 查询
        let chain = <Referral as ReferralProvider<u64>>::get_referral_chain(&1);

        // 验证
        assert_eq!(chain, vec![2, 3]);
    });
}

// ========================================
// 🆕 下线功能测试
// ========================================

#[test]
fn test_downlines_updated_on_bind() {
    new_test_ext().execute_with(|| {
        // 准备：Bob (2) 设置推荐码
        setup_code_for_account(2, b"BOBCODE1");

        // 执行：Alice (1) 绑定 Bob 为推荐人
        assert_ok!(Referral::bind_sponsor(
            RuntimeOrigin::signed(1),
            b"BOBCODE1".to_vec()
        ));

        // 验证：Bob 的下线列表包含 Alice
        let downlines = Referral::downlines_of(2);
        assert_eq!(downlines.len(), 1);
        assert_eq!(downlines[0], 1);
    });
}

#[test]
fn test_downlines_multiple() {
    new_test_ext().execute_with(|| {
        // 准备：Bob (2) 设置推荐码
        setup_code_for_account(2, b"BOBCODE1");

        // 执行：多个用户绑定 Bob 为推荐人
        assert_ok!(Referral::bind_sponsor(RuntimeOrigin::signed(1), b"BOBCODE1".to_vec()));
        assert_ok!(Referral::bind_sponsor(RuntimeOrigin::signed(3), b"BOBCODE1".to_vec()));
        assert_ok!(Referral::bind_sponsor(RuntimeOrigin::signed(4), b"BOBCODE1".to_vec()));

        // 验证：Bob 的下线列表包含所有人
        let downlines = Referral::downlines_of(2);
        assert_eq!(downlines.len(), 3);
        assert!(downlines.contains(&1));
        assert!(downlines.contains(&3));
        assert!(downlines.contains(&4));
    });
}

#[test]
fn test_referral_provider_get_downlines() {
    new_test_ext().execute_with(|| {
        use crate::ReferralProvider;

        // 准备：Bob (2) 设置推荐码，Alice (1) 和 Charlie (3) 绑定
        setup_code_for_account(2, b"BOBCODE1");
        assert_ok!(Referral::bind_sponsor(RuntimeOrigin::signed(1), b"BOBCODE1".to_vec()));
        assert_ok!(Referral::bind_sponsor(RuntimeOrigin::signed(3), b"BOBCODE1".to_vec()));

        // 执行：通过 trait 查询下线
        let downlines = <Referral as ReferralProvider<u64>>::get_downlines(&2);

        // 验证
        assert_eq!(downlines.len(), 2);
        assert!(downlines.contains(&1));
        assert!(downlines.contains(&3));
    });
}

#[test]
fn test_referral_provider_get_downline_count() {
    new_test_ext().execute_with(|| {
        use crate::ReferralProvider;

        // 准备：Bob (2) 设置推荐码
        setup_code_for_account(2, b"BOBCODE1");

        // 验证：初始下线数为 0
        assert_eq!(<Referral as ReferralProvider<u64>>::get_downline_count(&2), 0);

        // 执行：绑定多个下线
        assert_ok!(Referral::bind_sponsor(RuntimeOrigin::signed(1), b"BOBCODE1".to_vec()));
        assert_ok!(Referral::bind_sponsor(RuntimeOrigin::signed(3), b"BOBCODE1".to_vec()));

        // 验证：下线数为 2
        assert_eq!(<Referral as ReferralProvider<u64>>::get_downline_count(&2), 2);
    });
}
