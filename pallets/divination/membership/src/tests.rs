//! Unit tests for the membership pallet.

use crate::{mock::*, types::*, Error, Event};
use frame_support::{assert_noop, assert_ok, BoundedVec};

// ============ Subscription Tests ============

#[test]
fn subscribe_bronze_works() {
    new_test_ext().execute_with(|| {
        let user = 1;
        let initial_balance = balance(user);

        // Subscribe to Bronze (5 DUST/month)
        assert_ok!(Membership::subscribe(
            RuntimeOrigin::signed(user),
            MemberTier::Bronze,
            SubscriptionDuration::Monthly,
            false,
        ));

        // Check balance deducted
        assert_eq!(balance(user), initial_balance - 5 * DUST);

        // Check membership created
        let member = Membership::members(user).unwrap();
        assert_eq!(member.tier, MemberTier::Bronze);
        assert_eq!(member.total_paid, 5 * DUST);

        // Check expiration (300 blocks = 1 month in test)
        let now = System::block_number();
        assert_eq!(member.expires_at, now + 300);

        // Check treasury received 90% (treasury started with existential deposit of 1)
        assert_eq!(treasury_balance(), 1 + 5 * DUST * 9 / 10);

        // Check reward pool received 10%
        let pool_initial = 100_000 * DUST;
        assert_eq!(reward_pool_balance(), pool_initial + 5 * DUST / 10);

        // Check event
        System::assert_last_event(
            Event::Subscribed {
                who: user,
                tier: MemberTier::Bronze,
                duration: SubscriptionDuration::Monthly,
                amount_paid: 5 * DUST,
                expires_at: now + 300,
            }
            .into(),
        );
    });
}

#[test]
fn subscribe_yearly_with_discount_works() {
    new_test_ext().execute_with(|| {
        let user = 1;

        // Subscribe to Silver yearly (25 * 10 = 250 DUST instead of 25 * 12 = 300)
        assert_ok!(Membership::subscribe(
            RuntimeOrigin::signed(user),
            MemberTier::Silver,
            SubscriptionDuration::Yearly,
            false,
        ));

        let member = Membership::members(user).unwrap();
        assert_eq!(member.total_paid, 250 * DUST); // 10 months for 12

        // Check expiration (300 * 12 blocks = 1 year in test)
        let now = System::block_number();
        assert_eq!(member.expires_at, now + 300 * 12);
    });
}

#[test]
fn subscribe_to_free_tier_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::subscribe(
                RuntimeOrigin::signed(1),
                MemberTier::Free,
                SubscriptionDuration::Monthly,
                false,
            ),
            Error::<Test>::InvalidTier
        );
    });
}

#[test]
fn subscribe_when_already_subscribed_fails() {
    new_test_ext().execute_with(|| {
        let user = 1;

        // First subscription
        assert_ok!(Membership::subscribe(
            RuntimeOrigin::signed(user),
            MemberTier::Bronze,
            SubscriptionDuration::Monthly,
            false,
        ));

        // Second subscription fails
        assert_noop!(
            Membership::subscribe(
                RuntimeOrigin::signed(user),
                MemberTier::Silver,
                SubscriptionDuration::Monthly,
                false,
            ),
            Error::<Test>::AlreadySubscribed
        );
    });
}

#[test]
fn subscribe_with_insufficient_balance_fails() {
    new_test_ext().execute_with(|| {
        let poor_user = 4; // 0 balance

        assert_noop!(
            Membership::subscribe(
                RuntimeOrigin::signed(poor_user),
                MemberTier::Bronze,
                SubscriptionDuration::Monthly,
                false,
            ),
            Error::<Test>::InsufficientBalance
        );
    });
}

// ============ Upgrade Tests ============

#[test]
fn upgrade_tier_works() {
    new_test_ext().execute_with(|| {
        let user = 1;

        // Subscribe to Bronze
        assert_ok!(Membership::subscribe(
            RuntimeOrigin::signed(user),
            MemberTier::Bronze,
            SubscriptionDuration::Monthly,
            false,
        ));

        let balance_after_bronze = balance(user);

        // Upgrade to Silver (prorated)
        assert_ok!(Membership::upgrade_tier(
            RuntimeOrigin::signed(user),
            MemberTier::Silver,
        ));

        // Check tier changed
        let member = Membership::members(user).unwrap();
        assert_eq!(member.tier, MemberTier::Silver);

        // Check prorated cost deducted (approximately 20 DUST for full month)
        // Silver(25) - Bronze(5) = 20 DUST/month difference
        let upgrade_cost = balance_after_bronze - balance(user);
        assert!(upgrade_cost > 0);
        assert!(upgrade_cost <= 20 * DUST);
    });
}

#[test]
fn downgrade_tier_fails() {
    new_test_ext().execute_with(|| {
        let user = 1;

        // Subscribe to Gold
        assert_ok!(Membership::subscribe(
            RuntimeOrigin::signed(user),
            MemberTier::Gold,
            SubscriptionDuration::Monthly,
            false,
        ));

        // Try to downgrade to Bronze
        assert_noop!(
            Membership::upgrade_tier(RuntimeOrigin::signed(user), MemberTier::Bronze),
            Error::<Test>::CannotDowngrade
        );
    });
}

// ============ Tier Configuration Tests ============

#[test]
fn tier_monthly_fees_are_correct() {
    new_test_ext().execute_with(|| {
        assert_eq!(Membership::get_tier_monthly_fee(MemberTier::Free), 0);
        assert_eq!(Membership::get_tier_monthly_fee(MemberTier::Bronze), 5 * DUST);
        assert_eq!(Membership::get_tier_monthly_fee(MemberTier::Silver), 25 * DUST);
        assert_eq!(Membership::get_tier_monthly_fee(MemberTier::Gold), 80 * DUST);
        assert_eq!(Membership::get_tier_monthly_fee(MemberTier::Platinum), 200 * DUST);
        assert_eq!(Membership::get_tier_monthly_fee(MemberTier::Diamond), 500 * DUST);
    });
}

#[test]
fn tier_discounts_are_correct() {
    new_test_ext().execute_with(|| {
        // Storage discounts
        assert_eq!(Membership::get_storage_discount(MemberTier::Free), 0);
        assert_eq!(Membership::get_storage_discount(MemberTier::Bronze), 3000); // 30%
        assert_eq!(Membership::get_storage_discount(MemberTier::Silver), 3000); // 30%
        assert_eq!(Membership::get_storage_discount(MemberTier::Gold), 4000); // 40%
        assert_eq!(Membership::get_storage_discount(MemberTier::Platinum), 5000); // 50%
        assert_eq!(Membership::get_storage_discount(MemberTier::Diamond), 6000); // 60%

        // AI discounts
        assert_eq!(Membership::get_ai_discount(MemberTier::Free), 0);
        assert_eq!(Membership::get_ai_discount(MemberTier::Bronze), 1500); // 15%
        assert_eq!(Membership::get_ai_discount(MemberTier::Silver), 2000); // 20%
        assert_eq!(Membership::get_ai_discount(MemberTier::Gold), 5000); // 50%
        assert_eq!(Membership::get_ai_discount(MemberTier::Platinum), 7000); // 70%
        assert_eq!(Membership::get_ai_discount(MemberTier::Diamond), 8000); // 80%
    });
}

#[test]
fn tier_free_ai_quotas_are_correct() {
    new_test_ext().execute_with(|| {
        assert_eq!(Membership::get_monthly_free_ai_quota(MemberTier::Free), 0);
        assert_eq!(Membership::get_monthly_free_ai_quota(MemberTier::Bronze), 1);
        assert_eq!(Membership::get_monthly_free_ai_quota(MemberTier::Silver), 5);
        assert_eq!(Membership::get_monthly_free_ai_quota(MemberTier::Gold), 5);
        assert_eq!(Membership::get_monthly_free_ai_quota(MemberTier::Platinum), 20);
        assert_eq!(Membership::get_monthly_free_ai_quota(MemberTier::Diamond), 50);
    });
}

// ============ Check-in Tests ============

#[test]
fn check_in_requires_cooldown_period() {
    new_test_ext().execute_with(|| {
        let new_user = 2;

        // First, register activity to start cooldown tracking
        assert_ok!(Membership::update_profile(
            RuntimeOrigin::signed(new_user),
            BoundedVec::try_from(b"NewUser".to_vec()).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
        ));

        // New user cannot check in immediately (within cooldown)
        assert_noop!(
            Membership::check_in(RuntimeOrigin::signed(new_user)),
            Error::<Test>::AccountInCooldown
        );

        // Advance past cooldown (100 blocks in test)
        advance_blocks(101);

        // Now check-in should work
        assert_ok!(Membership::check_in(RuntimeOrigin::signed(new_user)));
    });
}

#[test]
fn check_in_requires_minimum_balance() {
    new_test_ext().execute_with(|| {
        let poor_user = 4; // 0 balance

        // Create account activity to start cooldown
        assert_ok!(Membership::update_profile(
            RuntimeOrigin::signed(poor_user),
            BoundedVec::try_from(b"Poor".to_vec()).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
        ));

        // Advance past cooldown
        advance_blocks(101);

        // Check-in fails due to low balance
        assert_noop!(
            Membership::check_in(RuntimeOrigin::signed(poor_user)),
            Error::<Test>::BalanceTooLow
        );
    });
}

#[test]
fn check_in_once_per_day() {
    new_test_ext().execute_with(|| {
        let user = 1;

        // Start cooldown tracking
        assert_ok!(Membership::update_profile(
            RuntimeOrigin::signed(user),
            BoundedVec::try_from(b"Test".to_vec()).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
        ));

        // Advance past cooldown
        advance_blocks(101);

        // First check-in works
        assert_ok!(Membership::check_in(RuntimeOrigin::signed(user)));

        // Second check-in same day fails
        assert_noop!(
            Membership::check_in(RuntimeOrigin::signed(user)),
            Error::<Test>::AlreadyCheckedIn
        );

        // Advance to next day (10 blocks = 1 day in test)
        advance_blocks(10);

        // Check-in works again
        assert_ok!(Membership::check_in(RuntimeOrigin::signed(user)));
    });
}

#[test]
fn check_in_streak_increases() {
    new_test_ext().execute_with(|| {
        let user = 1;

        // Setup
        assert_ok!(Membership::update_profile(
            RuntimeOrigin::signed(user),
            BoundedVec::try_from(b"Test".to_vec()).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
        ));
        advance_blocks(101);

        // Day 1
        assert_ok!(Membership::check_in(RuntimeOrigin::signed(user)));
        let record = Membership::check_in_records(user);
        assert_eq!(record.streak, 1);

        // Day 2
        advance_blocks(10);
        assert_ok!(Membership::check_in(RuntimeOrigin::signed(user)));
        let record = Membership::check_in_records(user);
        assert_eq!(record.streak, 2);

        // Day 3
        advance_blocks(10);
        assert_ok!(Membership::check_in(RuntimeOrigin::signed(user)));
        let record = Membership::check_in_records(user);
        assert_eq!(record.streak, 3);
    });
}

#[test]
fn check_in_streak_resets_on_missed_day() {
    new_test_ext().execute_with(|| {
        let user = 1;

        // Setup
        assert_ok!(Membership::update_profile(
            RuntimeOrigin::signed(user),
            BoundedVec::try_from(b"Test".to_vec()).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
        ));
        advance_blocks(101);

        // Day 1-3: consecutive check-ins
        assert_ok!(Membership::check_in(RuntimeOrigin::signed(user)));
        advance_blocks(10);
        assert_ok!(Membership::check_in(RuntimeOrigin::signed(user)));
        advance_blocks(10);
        assert_ok!(Membership::check_in(RuntimeOrigin::signed(user)));

        let record = Membership::check_in_records(user);
        assert_eq!(record.streak, 3);

        // Skip day 4, check in on day 5
        advance_blocks(20); // 2 days

        assert_ok!(Membership::check_in(RuntimeOrigin::signed(user)));
        let record = Membership::check_in_records(user);
        assert_eq!(record.streak, 1); // Reset
    });
}

// ============ Profile Tests ============

#[test]
fn update_profile_works() {
    new_test_ext().execute_with(|| {
        let user = 1;

        let birth_date = BirthDate {
            year: 1990,
            month: 5,
            day: 15,
        };

        assert_ok!(Membership::update_profile(
            RuntimeOrigin::signed(user),
            BoundedVec::try_from(b"TestUser".to_vec()).unwrap(),
            Some(Gender::Male),
            Some(birth_date.clone()),
            Some(14), // 2pm
            Some(1164532), // 116.4532 E
            Some(399042),  // 39.9042 N
            None,
        ));

        let profile = Membership::member_profiles(user).unwrap();
        assert_eq!(profile.display_name.as_slice(), b"TestUser");
        assert_eq!(profile.gender, Some(Gender::Male));
        assert_eq!(profile.birth_date, Some(birth_date));
        assert_eq!(profile.birth_hour, Some(14));
        assert_eq!(profile.longitude, Some(1164532));
        assert_eq!(profile.latitude, Some(399042));
        assert_eq!(profile.is_provider, false);
        assert_eq!(profile.provider_verified, false);
    });
}

#[test]
fn apply_provider_works() {
    new_test_ext().execute_with(|| {
        let user = 1;

        // First create profile
        assert_ok!(Membership::update_profile(
            RuntimeOrigin::signed(user),
            BoundedVec::try_from(b"Provider".to_vec()).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
        ));

        // Apply as provider
        assert_ok!(Membership::apply_provider(RuntimeOrigin::signed(user)));

        let profile = Membership::member_profiles(user).unwrap();
        assert_eq!(profile.is_provider, true);
        assert_eq!(profile.provider_verified, false);
    });
}

#[test]
fn verify_provider_requires_root() {
    new_test_ext().execute_with(|| {
        let provider = 1;

        // Setup
        assert_ok!(Membership::update_profile(
            RuntimeOrigin::signed(provider),
            BoundedVec::try_from(b"Provider".to_vec()).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
        ));
        assert_ok!(Membership::apply_provider(RuntimeOrigin::signed(provider)));

        // Non-root cannot verify
        assert_noop!(
            Membership::verify_provider(RuntimeOrigin::signed(2), provider, true),
            sp_runtime::DispatchError::BadOrigin
        );

        // Root can verify
        assert_ok!(Membership::verify_provider(
            RuntimeOrigin::root(),
            provider,
            true
        ));

        let profile = Membership::member_profiles(provider).unwrap();
        assert_eq!(profile.provider_verified, true);
    });
}

// ============ Free AI Quota Tests ============

#[test]
fn use_free_ai_works_for_bronze() {
    new_test_ext().execute_with(|| {
        let user = 1;

        // Subscribe to Bronze (1 free AI/month)
        assert_ok!(Membership::subscribe(
            RuntimeOrigin::signed(user),
            MemberTier::Bronze,
            SubscriptionDuration::Monthly,
            false,
        ));

        // Use free AI
        assert_ok!(Membership::use_free_ai(RuntimeOrigin::signed(user)));

        // Second use fails (quota exhausted)
        assert_noop!(
            Membership::use_free_ai(RuntimeOrigin::signed(user)),
            Error::<Test>::FreeAiQuotaExceeded
        );
    });
}

#[test]
fn free_ai_resets_monthly() {
    new_test_ext().execute_with(|| {
        let user = 1;

        // Subscribe to Bronze (use Yearly so it doesn't expire during test)
        assert_ok!(Membership::subscribe(
            RuntimeOrigin::signed(user),
            MemberTier::Bronze,
            SubscriptionDuration::Yearly,
            false,
        ));

        // Use free AI
        assert_ok!(Membership::use_free_ai(RuntimeOrigin::signed(user)));
        assert_noop!(
            Membership::use_free_ai(RuntimeOrigin::signed(user)),
            Error::<Test>::FreeAiQuotaExceeded
        );

        // Advance to next month (300 blocks in test)
        advance_blocks(300);

        // Can use free AI again
        assert_ok!(Membership::use_free_ai(RuntimeOrigin::signed(user)));
    });
}

// ============ Reward Tests ============

#[test]
fn reward_multiplier_applies_correctly() {
    new_test_ext().execute_with(|| {
        // Free user: 1.0x
        assert_eq!(
            Membership::get_reward_multiplier_base(MemberTier::Free),
            10000
        );

        // Diamond user: 5.0x
        assert_eq!(
            Membership::get_reward_multiplier_base(MemberTier::Diamond),
            50000
        );
    });
}

#[test]
fn check_in_reward_with_member_bonus() {
    new_test_ext().execute_with(|| {
        let user = 1;

        // Subscribe to Gold (2.0x reward multiplier)
        assert_ok!(Membership::subscribe(
            RuntimeOrigin::signed(user),
            MemberTier::Gold,
            SubscriptionDuration::Monthly,
            false,
        ));

        // Setup for check-in
        advance_blocks(101);

        let balance_before = balance(user);

        // Check-in
        assert_ok!(Membership::check_in(RuntimeOrigin::signed(user)));

        // Base reward: 0.001 DUST
        // Gold multiplier: 2.0x
        // Pool adjustment: 1.0x (pool is well-funded)
        // Expected: 0.001 * 2.0 * 1.0 = 0.002 DUST
        let balance_after = balance(user);
        let reward_received = balance_after - balance_before;

        assert!(reward_received > 0);
        // Allow for some rounding
        assert!(reward_received <= DUST / 500); // ~0.002 DUST
    });
}

// ============ Membership Expiration Tests ============

#[test]
fn expired_membership_returns_free_tier() {
    new_test_ext().execute_with(|| {
        let user = 1;

        // Subscribe to Bronze
        assert_ok!(Membership::subscribe(
            RuntimeOrigin::signed(user),
            MemberTier::Bronze,
            SubscriptionDuration::Monthly,
            false,
        ));

        // Currently Bronze
        assert_eq!(Membership::get_tier(&user), MemberTier::Bronze);

        // Advance past expiration (300 blocks in test)
        advance_blocks(301);

        // Now Free
        assert_eq!(Membership::get_tier(&user), MemberTier::Free);
    });
}

// ============ Global Stats Tests ============

#[test]
fn global_stats_update_on_subscription() {
    new_test_ext().execute_with(|| {
        // Initial stats
        let stats_before = Membership::global_stats();
        assert_eq!(stats_before.total_revenue, 0);

        // Subscribe
        assert_ok!(Membership::subscribe(
            RuntimeOrigin::signed(1),
            MemberTier::Bronze,
            SubscriptionDuration::Monthly,
            false,
        ));

        let stats_after = Membership::global_stats();
        assert_eq!(stats_after.total_revenue, 5 * DUST);
        assert_eq!(stats_after.tier_counts[MemberTier::Bronze as usize], 1);
    });
}
