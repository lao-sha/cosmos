//! 直播间模块单元测试

use crate::{mock::*, Error, Event, LiveRoomStatus, LiveRoomType};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::BuildStorage;

// ============ 直播间创建测试 ============

#[test]
fn create_room_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            Some(b"Description".to_vec()),
            LiveRoomType::Normal,
            None,
            None,
        ));

        // 检查直播间创建
        let room = Livestream::live_rooms(0).unwrap();
        assert_eq!(room.host, ALICE);
        assert_eq!(room.status, LiveRoomStatus::Preparing);
        assert_eq!(room.room_type, LiveRoomType::Normal);

        // 检查主播映射
        assert_eq!(Livestream::host_room(ALICE), Some(0));

        // 检查事件
        System::assert_has_event(
            Event::RoomCreated {
                host: ALICE,
                room_id: 0,
                room_type: LiveRoomType::Normal,
            }
            .into(),
        );
    });
}

#[test]
fn create_room_fails_if_already_has_room() {
    new_test_ext().execute_with(|| {
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Room 1".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));

        assert_noop!(
            Livestream::create_room(
                RuntimeOrigin::signed(ALICE),
                b"Room 2".to_vec(),
                None,
                LiveRoomType::Normal,
                None,
                None,
            ),
            Error::<Test>::HostAlreadyHasRoom
        );
    });
}

#[test]
fn create_paid_room_requires_ticket_price() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Livestream::create_room(
                RuntimeOrigin::signed(ALICE),
                b"Paid Room".to_vec(),
                None,
                LiveRoomType::Paid,
                None,
                None, // 没有设置票价
            ),
            Error::<Test>::InvalidTicketPrice
        );

        // 设置票价后可以创建
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Paid Room".to_vec(),
            None,
            LiveRoomType::Paid,
            None,
            Some(10_000_000_000_000), // 10 DUST
        ));
    });
}

// ============ 直播状态测试 ============

#[test]
fn start_live_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));

        assert_ok!(Livestream::start_live(RuntimeOrigin::signed(ALICE), 0));

        let room = Livestream::live_rooms(0).unwrap();
        assert_eq!(room.status, LiveRoomStatus::Live);
        assert!(room.started_at.is_some());
    });
}

#[test]
fn start_live_fails_if_not_host() {
    new_test_ext().execute_with(|| {
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));

        assert_noop!(
            Livestream::start_live(RuntimeOrigin::signed(BOB), 0),
            Error::<Test>::NotRoomHost
        );
    });
}

#[test]
fn pause_and_resume_live_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));
        assert_ok!(Livestream::start_live(RuntimeOrigin::signed(ALICE), 0));

        // 暂停
        assert_ok!(Livestream::pause_live(RuntimeOrigin::signed(ALICE), 0));
        let room = Livestream::live_rooms(0).unwrap();
        assert_eq!(room.status, LiveRoomStatus::Paused);

        // 恢复
        assert_ok!(Livestream::resume_live(RuntimeOrigin::signed(ALICE), 0));
        let room = Livestream::live_rooms(0).unwrap();
        assert_eq!(room.status, LiveRoomStatus::Live);
    });
}

#[test]
fn end_live_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));
        assert_ok!(Livestream::start_live(RuntimeOrigin::signed(ALICE), 0));

        System::set_block_number(100);
        assert_ok!(Livestream::end_live(RuntimeOrigin::signed(ALICE), 0));

        let room = Livestream::live_rooms(0).unwrap();
        assert_eq!(room.status, LiveRoomStatus::Ended);
        assert!(room.ended_at.is_some());

        // 主播映射已清除
        assert_eq!(Livestream::host_room(ALICE), None);
    });
}

// ============ 门票测试 ============

#[test]
fn buy_ticket_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let ticket_price = 10_000_000_000_000u128; // 10 DUST

        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Paid Room".to_vec(),
            None,
            LiveRoomType::Paid,
            None,
            Some(ticket_price),
        ));

        let alice_balance_before = Balances::free_balance(ALICE);
        let bob_balance_before = Balances::free_balance(BOB);

        assert_ok!(Livestream::buy_ticket(RuntimeOrigin::signed(BOB), 0));

        // 检查门票记录
        assert!(Livestream::has_ticket(0, &BOB));

        // 检查余额变化
        assert_eq!(
            Balances::free_balance(ALICE),
            alice_balance_before + ticket_price
        );
        assert_eq!(
            Balances::free_balance(BOB),
            bob_balance_before - ticket_price
        );
    });
}

#[test]
fn buy_ticket_fails_if_already_has() {
    new_test_ext().execute_with(|| {
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Paid Room".to_vec(),
            None,
            LiveRoomType::Paid,
            None,
            Some(10_000_000_000_000),
        ));

        assert_ok!(Livestream::buy_ticket(RuntimeOrigin::signed(BOB), 0));

        assert_noop!(
            Livestream::buy_ticket(RuntimeOrigin::signed(BOB), 0),
            Error::<Test>::AlreadyHasTicket
        );
    });
}

// ============ 礼物测试 ============

#[test]
fn send_gift_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // 创建礼物
        assert_ok!(Livestream::create_gift(
            RuntimeOrigin::root(),
            b"Rose".to_vec(),
            1_000_000_000_000, // 1 DUST
            b"QmRose".to_vec(),
        ));

        // 创建直播间并开播
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));
        assert_ok!(Livestream::start_live(RuntimeOrigin::signed(ALICE), 0));

        let alice_balance_before = Balances::free_balance(ALICE);
        let bob_balance_before = Balances::free_balance(BOB);

        // 发送礼物
        assert_ok!(Livestream::send_gift(
            RuntimeOrigin::signed(BOB),
            0,
            0,  // gift_id
            10, // quantity
        ));

        // 检查余额变化 (10 DUST 礼物, 20% 平台抽成)
        let total = 10_000_000_000_000u128; // 10 DUST
        let platform_fee = total * 20 / 100; // 2 DUST
        let host_amount = total - platform_fee; // 8 DUST

        assert_eq!(
            Balances::free_balance(ALICE),
            alice_balance_before + host_amount
        );
        assert_eq!(Balances::free_balance(BOB), bob_balance_before - total);

        // 检查统计
        let room = Livestream::live_rooms(0).unwrap();
        assert_eq!(room.total_gifts, total);

        assert_eq!(Livestream::host_earnings(ALICE), host_amount);
        assert_eq!(Livestream::user_room_gifts(0, BOB), total);
    });
}

#[test]
fn send_gift_fails_if_room_not_live() {
    new_test_ext().execute_with(|| {
        assert_ok!(Livestream::create_gift(
            RuntimeOrigin::root(),
            b"Rose".to_vec(),
            1_000_000_000_000,
            b"QmRose".to_vec(),
        ));

        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));

        // 直播间未开播
        assert_noop!(
            Livestream::send_gift(RuntimeOrigin::signed(BOB), 0, 0, 1),
            Error::<Test>::RoomNotLive
        );
    });
}

// ============ 黑名单测试 ============

#[test]
fn kick_viewer_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));

        assert_ok!(Livestream::kick_viewer(
            RuntimeOrigin::signed(ALICE),
            0,
            BOB
        ));

        assert!(Livestream::is_blacklisted(0, &BOB));
    });
}

#[test]
fn remove_from_blacklist_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));

        assert_ok!(Livestream::kick_viewer(
            RuntimeOrigin::signed(ALICE),
            0,
            BOB
        ));
        assert!(Livestream::is_blacklisted(0, &BOB));

        assert_ok!(Livestream::remove_from_blacklist(
            RuntimeOrigin::signed(ALICE),
            0,
            BOB
        ));
        assert!(!Livestream::is_blacklisted(0, &BOB));
    });
}

// ============ 连麦测试 ============

#[test]
fn co_host_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));
        assert_ok!(Livestream::start_live(RuntimeOrigin::signed(ALICE), 0));

        // 开始连麦
        assert_ok!(Livestream::start_co_host(
            RuntimeOrigin::signed(ALICE),
            0,
            BOB
        ));

        let co_hosts = Livestream::active_co_hosts(0);
        assert_eq!(co_hosts.len(), 1);
        assert!(co_hosts.contains(&BOB));

        // 结束连麦 (连麦者自己退出)
        assert_ok!(Livestream::end_co_host(
            RuntimeOrigin::signed(BOB),
            0,
            None
        ));

        let co_hosts = Livestream::active_co_hosts(0);
        assert_eq!(co_hosts.len(), 0);
    });
}

#[test]
fn co_host_max_limit() {
    new_test_ext().execute_with(|| {
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));
        assert_ok!(Livestream::start_live(RuntimeOrigin::signed(ALICE), 0));

        // 添加 4 个连麦者 (最大限制)
        assert_ok!(Livestream::start_co_host(RuntimeOrigin::signed(ALICE), 0, BOB));
        assert_ok!(Livestream::start_co_host(RuntimeOrigin::signed(ALICE), 0, CHARLIE));
        assert_ok!(Livestream::start_co_host(RuntimeOrigin::signed(ALICE), 0, DAVE));
        assert_ok!(Livestream::start_co_host(RuntimeOrigin::signed(ALICE), 0, EVE));

        // 第 5 个应该失败
        assert_noop!(
            Livestream::start_co_host(RuntimeOrigin::signed(ALICE), 0, 6),
            Error::<Test>::TooManyCoHosts
        );
    });
}

// ============ 封禁测试 ============

#[test]
fn ban_room_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));

        assert_ok!(Livestream::ban_room(
            RuntimeOrigin::root(),
            0,
            b"Violation".to_vec()
        ));

        let room = Livestream::live_rooms(0).unwrap();
        assert_eq!(room.status, LiveRoomStatus::Banned);

        // 主播映射已清除
        assert_eq!(Livestream::host_room(ALICE), None);
    });
}

// ============ 提现测试 ============

#[test]
fn withdraw_earnings_works() {
    new_test_ext().execute_with(|| {
        // 创建礼物和直播间
        assert_ok!(Livestream::create_gift(
            RuntimeOrigin::root(),
            b"Rose".to_vec(),
            1_000_000_000_000,
            b"QmRose".to_vec(),
        ));
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));
        assert_ok!(Livestream::start_live(RuntimeOrigin::signed(ALICE), 0));

        // 发送礼物
        assert_ok!(Livestream::send_gift(RuntimeOrigin::signed(BOB), 0, 0, 10));

        // 提现
        let earnings = Livestream::host_earnings(ALICE);
        assert_ok!(Livestream::withdraw_earnings(
            RuntimeOrigin::signed(ALICE),
            earnings
        ));

        assert_eq!(Livestream::host_earnings(ALICE), 0);
    });
}

#[test]
fn withdraw_earnings_fails_if_amount_too_low() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Livestream::withdraw_earnings(
                RuntimeOrigin::signed(ALICE),
                100_000_000_000 // 0.1 DUST, 低于最小提现金额
            ),
            Error::<Test>::WithdrawAmountTooLow
        );
    });
}

// ============ GenesisConfig 测试 ============

#[test]
fn genesis_config_initializes_gifts() {
    // 使用带有初始礼物的 GenesisConfig
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 1_000_000_000_000_000)],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    // 初始化 3 个礼物
    crate::GenesisConfig::<Test> {
        gifts: vec![
            (b"Rose".to_vec(), 1_000_000_000_000, b"QmRose".to_vec()),
            (b"Heart".to_vec(), 5_000_000_000_000, b"QmHeart".to_vec()),
            (b"Rocket".to_vec(), 10_000_000_000_000, b"QmRocket".to_vec()),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        // 验证礼物已创建
        assert_eq!(Livestream::next_gift_id(), 3);

        // 验证第一个礼物
        let gift0 = Livestream::gifts(0).unwrap();
        assert_eq!(gift0.id, 0);
        assert_eq!(gift0.price, 1_000_000_000_000);
        assert!(gift0.enabled);

        // 验证第二个礼物
        let gift1 = Livestream::gifts(1).unwrap();
        assert_eq!(gift1.id, 1);
        assert_eq!(gift1.price, 5_000_000_000_000);

        // 验证第三个礼物
        let gift2 = Livestream::gifts(2).unwrap();
        assert_eq!(gift2.id, 2);
        assert_eq!(gift2.price, 10_000_000_000_000);
    });
}

// ============ RuntimeApi 辅助函数测试 ============

#[test]
fn get_room_info_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // 创建直播间
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Test Room".to_vec(),
            Some(b"Description".to_vec()),
            LiveRoomType::Normal,
            None,
            None,
        ));

        // 获取直播间信息
        let room_info = Livestream::get_room_info(0).unwrap();
        assert_eq!(room_info.id, 0);
        assert_eq!(room_info.host, ALICE);
        assert_eq!(room_info.title, b"Test Room".to_vec());
        assert_eq!(room_info.room_type, 0); // Normal
        assert_eq!(room_info.status, 0);    // Preparing
    });
}

#[test]
fn get_live_room_ids_works() {
    new_test_ext().execute_with(|| {
        // 创建多个直播间
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(ALICE),
            b"Room 1".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));
        assert_ok!(Livestream::create_room(
            RuntimeOrigin::signed(BOB),
            b"Room 2".to_vec(),
            None,
            LiveRoomType::Normal,
            None,
            None,
        ));

        // 只有 ALICE 开播
        assert_ok!(Livestream::start_live(RuntimeOrigin::signed(ALICE), 0));

        // 获取活跃直播间
        let live_rooms = Livestream::get_live_room_ids();
        assert_eq!(live_rooms.len(), 1);
        assert!(live_rooms.contains(&0));
    });
}
