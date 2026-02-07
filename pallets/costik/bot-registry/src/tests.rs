use crate::{mock::*, Error, Platform, BotStatus};
use frame_support::{assert_ok, assert_noop};

fn bot_hash(id: u8) -> [u8; 32] {
	let mut h = [0u8; 32];
	h[0] = id;
	h
}

fn community_hash(id: u8) -> [u8; 32] {
	let mut h = [0u8; 32];
	h[1] = id;
	h
}

fn pubkey(id: u8) -> [u8; 32] {
	[id; 32]
}

// ═══════════════════════════════════════════════════════════════
// Bot 注册
// ═══════════════════════════════════════════════════════════════

#[test]
fn register_bot_works() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		let ch = community_hash(1);
		let pk = pubkey(1);

		assert_ok!(BotRegistry::register_bot(
			RuntimeOrigin::signed(1),
			bh, ch, Platform::Telegram, pk,
		));

		let bot = BotRegistry::bots(bh).expect("bot should exist");
		assert_eq!(bot.owner, 1);
		assert_eq!(bot.platform, Platform::Telegram);
		assert_eq!(bot.bot_id_hash, bh);
		assert_eq!(bot.community_id_hash, ch);
		assert_eq!(bot.owner_public_key, pk);
		assert_eq!(bot.status, BotStatus::Active);

		let owner_list = BotRegistry::owner_bots(1u64);
		assert!(owner_list.contains(&bh));
	});
}

#[test]
fn register_bot_fails_duplicate() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		assert_ok!(BotRegistry::register_bot(
			RuntimeOrigin::signed(1),
			bh, community_hash(1), Platform::Telegram, pubkey(1),
		));

		assert_noop!(
			BotRegistry::register_bot(
				RuntimeOrigin::signed(2),
				bh, community_hash(2), Platform::Discord, pubkey(2),
			),
			Error::<Test>::BotAlreadyExists
		);
	});
}

// ═══════════════════════════════════════════════════════════════
// 公钥更新
// ═══════════════════════════════════════════════════════════════

#[test]
fn update_public_key_works() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		assert_ok!(BotRegistry::register_bot(
			RuntimeOrigin::signed(1),
			bh, community_hash(1), Platform::Telegram, pubkey(1),
		));

		let new_pk = pubkey(2);
		assert_ok!(BotRegistry::update_bot_public_key(
			RuntimeOrigin::signed(1),
			bh, new_pk,
		));

		let bot = BotRegistry::bots(bh).unwrap();
		assert_eq!(bot.owner_public_key, new_pk);
	});
}

#[test]
fn update_public_key_fails_not_owner() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		assert_ok!(BotRegistry::register_bot(
			RuntimeOrigin::signed(1),
			bh, community_hash(1), Platform::Telegram, pubkey(1),
		));

		assert_noop!(
			BotRegistry::update_bot_public_key(
				RuntimeOrigin::signed(2),
				bh, pubkey(3),
			),
			Error::<Test>::NotBotOwner
		);
	});
}

#[test]
fn update_public_key_fails_same_key() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		let pk = pubkey(1);
		assert_ok!(BotRegistry::register_bot(
			RuntimeOrigin::signed(1),
			bh, community_hash(1), Platform::Telegram, pk,
		));

		assert_noop!(
			BotRegistry::update_bot_public_key(
				RuntimeOrigin::signed(1),
				bh, pk,
			),
			Error::<Test>::PublicKeyUnchanged
		);
	});
}

// ═══════════════════════════════════════════════════════════════
// Bot 停用 / 暂停 / 恢复
// ═══════════════════════════════════════════════════════════════

#[test]
fn deactivate_bot_works() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		assert_ok!(BotRegistry::register_bot(
			RuntimeOrigin::signed(1),
			bh, community_hash(1), Platform::Telegram, pubkey(1),
		));

		assert_ok!(BotRegistry::deactivate_bot(RuntimeOrigin::signed(1), bh));

		let bot = BotRegistry::bots(bh).unwrap();
		assert_eq!(bot.status, BotStatus::Deactivated);

		// 从 owner 列表移除
		let owner_list = BotRegistry::owner_bots(1u64);
		assert!(!owner_list.contains(&bh));

		// 已停用不能再操作
		assert_noop!(
			BotRegistry::update_bot_public_key(RuntimeOrigin::signed(1), bh, pubkey(2)),
			Error::<Test>::BotAlreadyDeactivated
		);
	});
}

#[test]
fn suspend_and_reactivate_works() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		assert_ok!(BotRegistry::register_bot(
			RuntimeOrigin::signed(1),
			bh, community_hash(1), Platform::Telegram, pubkey(1),
		));

		// 暂停
		assert_ok!(BotRegistry::suspend_bot(RuntimeOrigin::signed(1), bh));
		assert_eq!(BotRegistry::bots(bh).unwrap().status, BotStatus::Suspended);

		// 恢复
		assert_ok!(BotRegistry::reactivate_bot(RuntimeOrigin::signed(1), bh));
		assert_eq!(BotRegistry::bots(bh).unwrap().status, BotStatus::Active);
	});
}

#[test]
fn reactivate_fails_if_not_suspended() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		assert_ok!(BotRegistry::register_bot(
			RuntimeOrigin::signed(1),
			bh, community_hash(1), Platform::Telegram, pubkey(1),
		));

		assert_noop!(
			BotRegistry::reactivate_bot(RuntimeOrigin::signed(1), bh),
			Error::<Test>::BotNotSuspended
		);
	});
}

// ═══════════════════════════════════════════════════════════════
// 社区平台绑定
// ═══════════════════════════════════════════════════════════════

#[test]
fn bind_community_platform_works() {
	new_test_ext().execute_with(|| {
		let ch = community_hash(1);
		let platform_ch = [42u8; 32];

		assert_ok!(BotRegistry::bind_community_platform(
			RuntimeOrigin::signed(1),
			ch, Platform::Telegram, platform_ch, Some(bot_hash(1)),
		));

		let binding = BotRegistry::community_platforms(ch, Platform::Telegram).unwrap();
		assert_eq!(binding.binder, 1);
		assert_eq!(binding.platform_community_id_hash, platform_ch);
		assert_eq!(binding.bot_id_hash, Some(bot_hash(1)));
	});
}

#[test]
fn bind_multiple_platforms_to_community() {
	new_test_ext().execute_with(|| {
		let ch = community_hash(1);

		assert_ok!(BotRegistry::bind_community_platform(
			RuntimeOrigin::signed(1),
			ch, Platform::Telegram, [1u8; 32], None,
		));
		assert_ok!(BotRegistry::bind_community_platform(
			RuntimeOrigin::signed(1),
			ch, Platform::Discord, [2u8; 32], None,
		));

		assert!(BotRegistry::community_platforms(ch, Platform::Telegram).is_some());
		assert!(BotRegistry::community_platforms(ch, Platform::Discord).is_some());
		assert!(BotRegistry::community_platforms(ch, Platform::Slack).is_none());
	});
}

#[test]
fn bind_community_fails_duplicate() {
	new_test_ext().execute_with(|| {
		let ch = community_hash(1);
		assert_ok!(BotRegistry::bind_community_platform(
			RuntimeOrigin::signed(1),
			ch, Platform::Telegram, [1u8; 32], None,
		));

		assert_noop!(
			BotRegistry::bind_community_platform(
				RuntimeOrigin::signed(1),
				ch, Platform::Telegram, [2u8; 32], None,
			),
			Error::<Test>::CommunityBindingAlreadyExists
		);
	});
}

#[test]
fn unbind_community_platform_works() {
	new_test_ext().execute_with(|| {
		let ch = community_hash(1);
		assert_ok!(BotRegistry::bind_community_platform(
			RuntimeOrigin::signed(1),
			ch, Platform::Telegram, [1u8; 32], None,
		));

		assert_ok!(BotRegistry::unbind_community_platform(
			RuntimeOrigin::signed(1),
			ch, Platform::Telegram,
		));

		assert!(BotRegistry::community_platforms(ch, Platform::Telegram).is_none());
	});
}

// ═══════════════════════════════════════════════════════════════
// 用户平台绑定
// ═══════════════════════════════════════════════════════════════

#[test]
fn bind_user_platform_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(BotRegistry::bind_user_platform(
			RuntimeOrigin::signed(1),
			Platform::Telegram,
			[10u8; 32],
		));

		let binding = BotRegistry::user_platform_bindings(1u64, Platform::Telegram).unwrap();
		assert_eq!(binding.platform_user_id_hash, [10u8; 32]);
		assert!(!binding.verified);
	});
}

#[test]
fn bind_user_platform_fails_duplicate() {
	new_test_ext().execute_with(|| {
		assert_ok!(BotRegistry::bind_user_platform(
			RuntimeOrigin::signed(1),
			Platform::Telegram,
			[10u8; 32],
		));

		assert_noop!(
			BotRegistry::bind_user_platform(
				RuntimeOrigin::signed(1),
				Platform::Telegram,
				[20u8; 32],
			),
			Error::<Test>::UserBindingAlreadyExists
		);
	});
}

#[test]
fn unbind_user_platform_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(BotRegistry::bind_user_platform(
			RuntimeOrigin::signed(1),
			Platform::Discord,
			[10u8; 32],
		));

		assert_ok!(BotRegistry::unbind_user_platform(
			RuntimeOrigin::signed(1),
			Platform::Discord,
		));

		assert!(BotRegistry::user_platform_bindings(1u64, Platform::Discord).is_none());
	});
}

#[test]
fn unbind_user_platform_fails_not_found() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			BotRegistry::unbind_user_platform(
				RuntimeOrigin::signed(1),
				Platform::Matrix,
			),
			Error::<Test>::UserBindingNotFound
		);
	});
}

// ═══════════════════════════════════════════════════════════════
// 辅助函数
// ═══════════════════════════════════════════════════════════════

#[test]
fn helper_functions_work() {
	new_test_ext().execute_with(|| {
		let bh = bot_hash(1);
		let pk = pubkey(1);

		assert_ok!(BotRegistry::register_bot(
			RuntimeOrigin::signed(1),
			bh, community_hash(1), Platform::Telegram, pk,
		));

		// get_bot_public_key
		assert_eq!(BotRegistry::get_bot_public_key(&bh), Some(pk));
		assert_eq!(BotRegistry::get_bot_public_key(&bot_hash(99)), None);

		// is_bot_active
		assert!(BotRegistry::is_bot_active(&bh));
		assert!(!BotRegistry::is_bot_active(&bot_hash(99)));

		// get_bot_owner
		assert_eq!(BotRegistry::get_bot_owner(&bh), Some(1));
	});
}
