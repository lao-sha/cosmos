//! 直播间模块 Benchmarks

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_support::traits::Currency;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    fn setup_account<T: Config>(name: &'static str) -> T::AccountId {
        let account: T::AccountId = account(name, 0, 0);
        let amount = T::RoomDeposit::get() * 100u32.into();
        T::Currency::make_free_balance_be(&account, amount);
        account
    }

    fn create_test_room<T: Config>(host: &T::AccountId) -> u64 {
        let title: Vec<u8> = b"Test Room".to_vec();
        let _ = Pallet::<T>::create_room(
            RawOrigin::Signed(host.clone()).into(),
            title,
            None,
            LiveRoomType::Normal,
            None,
            None,
        );
        Pallet::<T>::host_room(host).unwrap_or(0)
    }

    fn create_test_gift<T: Config>() -> u32 {
        let name: Vec<u8> = b"Rose".to_vec();
        let price: BalanceOf<T> = 1_000_000_000_000u128.try_into().ok().unwrap();
        let icon_cid: Vec<u8> = b"QmRose".to_vec();
        let _ = Pallet::<T>::create_gift(RawOrigin::Root.into(), name, price, icon_cid);
        0
    }

    #[benchmark]
    fn create_room() {
        let host = setup_account::<T>("host");
        let title: Vec<u8> = b"Benchmark Room".to_vec();
        let description: Vec<u8> = b"Description".to_vec();

        #[extrinsic_call]
        create_room(
            RawOrigin::Signed(host.clone()),
            title,
            Some(description),
            LiveRoomType::Normal,
            None,
            None,
        );

        assert!(HostRoom::<T>::contains_key(&host));
    }

    #[benchmark]
    fn start_live() {
        let host = setup_account::<T>("host");
        let room_id = create_test_room::<T>(&host);

        #[extrinsic_call]
        start_live(RawOrigin::Signed(host.clone()), room_id);

        let room = LiveRooms::<T>::get(room_id).unwrap();
        assert_eq!(room.status, LiveRoomStatus::Live);
    }

    #[benchmark]
    fn pause_live() {
        let host = setup_account::<T>("host");
        let room_id = create_test_room::<T>(&host);
        let _ = Pallet::<T>::start_live(RawOrigin::Signed(host.clone()).into(), room_id);

        #[extrinsic_call]
        pause_live(RawOrigin::Signed(host.clone()), room_id);

        let room = LiveRooms::<T>::get(room_id).unwrap();
        assert_eq!(room.status, LiveRoomStatus::Paused);
    }

    #[benchmark]
    fn resume_live() {
        let host = setup_account::<T>("host");
        let room_id = create_test_room::<T>(&host);
        let _ = Pallet::<T>::start_live(RawOrigin::Signed(host.clone()).into(), room_id);
        let _ = Pallet::<T>::pause_live(RawOrigin::Signed(host.clone()).into(), room_id);

        #[extrinsic_call]
        resume_live(RawOrigin::Signed(host.clone()), room_id);

        let room = LiveRooms::<T>::get(room_id).unwrap();
        assert_eq!(room.status, LiveRoomStatus::Live);
    }

    #[benchmark]
    fn end_live() {
        let host = setup_account::<T>("host");
        let room_id = create_test_room::<T>(&host);
        let _ = Pallet::<T>::start_live(RawOrigin::Signed(host.clone()).into(), room_id);

        #[extrinsic_call]
        end_live(RawOrigin::Signed(host.clone()), room_id);

        let room = LiveRooms::<T>::get(room_id).unwrap();
        assert_eq!(room.status, LiveRoomStatus::Ended);
    }

    #[benchmark]
    fn update_room() {
        let host = setup_account::<T>("host");
        let room_id = create_test_room::<T>(&host);
        let new_title: Vec<u8> = b"Updated Title".to_vec();

        #[extrinsic_call]
        update_room(
            RawOrigin::Signed(host.clone()),
            room_id,
            Some(new_title),
            None,
            None,
        );

        assert!(LiveRooms::<T>::contains_key(room_id));
    }

    #[benchmark]
    fn buy_ticket() {
        let host = setup_account::<T>("host");
        let buyer = setup_account::<T>("buyer");
        let ticket_price: BalanceOf<T> = 10_000_000_000_000u128.try_into().ok().unwrap();

        let title: Vec<u8> = b"Paid Room".to_vec();
        let _ = Pallet::<T>::create_room(
            RawOrigin::Signed(host.clone()).into(),
            title,
            None,
            LiveRoomType::Paid,
            None,
            Some(ticket_price),
        );
        let room_id = Pallet::<T>::host_room(&host).unwrap();

        #[extrinsic_call]
        buy_ticket(RawOrigin::Signed(buyer.clone()), room_id);

        assert!(TicketHolders::<T>::contains_key(room_id, &buyer));
    }

    #[benchmark]
    fn send_gift() {
        let host = setup_account::<T>("host");
        let sender = setup_account::<T>("sender");
        let room_id = create_test_room::<T>(&host);
        let _ = Pallet::<T>::start_live(RawOrigin::Signed(host.clone()).into(), room_id);
        let gift_id = create_test_gift::<T>();

        #[extrinsic_call]
        send_gift(RawOrigin::Signed(sender.clone()), room_id, gift_id, 1);

        let room = LiveRooms::<T>::get(room_id).unwrap();
        assert!(!room.total_gifts.is_zero());
    }

    #[benchmark]
    fn withdraw_earnings() {
        let host = setup_account::<T>("host");
        let sender = setup_account::<T>("sender");
        let room_id = create_test_room::<T>(&host);
        let _ = Pallet::<T>::start_live(RawOrigin::Signed(host.clone()).into(), room_id);
        let gift_id = create_test_gift::<T>();
        let _ = Pallet::<T>::send_gift(
            RawOrigin::Signed(sender.clone()).into(),
            room_id,
            gift_id,
            10,
        );

        let earnings = HostEarnings::<T>::get(&host);

        #[extrinsic_call]
        withdraw_earnings(RawOrigin::Signed(host.clone()), earnings);

        assert!(HostEarnings::<T>::get(&host).is_zero());
    }

    #[benchmark]
    fn sync_live_stats() {
        let host = setup_account::<T>("host");
        let room_id = create_test_room::<T>(&host);
        let _ = Pallet::<T>::start_live(RawOrigin::Signed(host.clone()).into(), room_id);

        #[extrinsic_call]
        sync_live_stats(RawOrigin::Signed(host.clone()), room_id, 1000, 500);

        let room = LiveRooms::<T>::get(room_id).unwrap();
        assert_eq!(room.total_viewers, 1000);
        assert_eq!(room.peak_viewers, 500);
    }

    #[benchmark]
    fn kick_viewer() {
        let host = setup_account::<T>("host");
        let viewer = setup_account::<T>("viewer");
        let room_id = create_test_room::<T>(&host);

        #[extrinsic_call]
        kick_viewer(RawOrigin::Signed(host.clone()), room_id, viewer.clone());

        assert!(RoomBlacklist::<T>::contains_key(room_id, &viewer));
    }

    #[benchmark]
    fn remove_from_blacklist() {
        let host = setup_account::<T>("host");
        let viewer = setup_account::<T>("viewer");
        let room_id = create_test_room::<T>(&host);
        let _ = Pallet::<T>::kick_viewer(
            RawOrigin::Signed(host.clone()).into(),
            room_id,
            viewer.clone(),
        );

        #[extrinsic_call]
        remove_from_blacklist(RawOrigin::Signed(host.clone()), room_id, viewer.clone());

        assert!(!RoomBlacklist::<T>::contains_key(room_id, &viewer));
    }

    #[benchmark]
    fn ban_room() {
        let host = setup_account::<T>("host");
        let room_id = create_test_room::<T>(&host);
        let reason: Vec<u8> = b"Violation".to_vec();

        #[extrinsic_call]
        ban_room(RawOrigin::Root, room_id, reason);

        let room = LiveRooms::<T>::get(room_id).unwrap();
        assert_eq!(room.status, LiveRoomStatus::Banned);
    }

    #[benchmark]
    fn start_co_host() {
        let host = setup_account::<T>("host");
        let co_host = setup_account::<T>("co_host");
        let room_id = create_test_room::<T>(&host);
        let _ = Pallet::<T>::start_live(RawOrigin::Signed(host.clone()).into(), room_id);

        #[extrinsic_call]
        start_co_host(RawOrigin::Signed(host.clone()), room_id, co_host.clone());

        let co_hosts = ActiveCoHosts::<T>::get(room_id);
        assert!(co_hosts.contains(&co_host));
    }

    #[benchmark]
    fn end_co_host() {
        let host = setup_account::<T>("host");
        let co_host = setup_account::<T>("co_host");
        let room_id = create_test_room::<T>(&host);
        let _ = Pallet::<T>::start_live(RawOrigin::Signed(host.clone()).into(), room_id);
        let _ = Pallet::<T>::start_co_host(
            RawOrigin::Signed(host.clone()).into(),
            room_id,
            co_host.clone(),
        );

        #[extrinsic_call]
        end_co_host(RawOrigin::Signed(co_host.clone()), room_id, None);

        let co_hosts = ActiveCoHosts::<T>::get(room_id);
        assert!(!co_hosts.contains(&co_host));
    }

    #[benchmark]
    fn create_gift() {
        let name: Vec<u8> = b"Diamond".to_vec();
        let price: BalanceOf<T> = 100_000_000_000_000u128.try_into().ok().unwrap();
        let icon_cid: Vec<u8> = b"QmDiamond".to_vec();

        #[extrinsic_call]
        create_gift(RawOrigin::Root, name, price, icon_cid);

        assert!(Gifts::<T>::contains_key(0));
    }

    #[benchmark]
    fn update_gift() {
        let _ = create_test_gift::<T>();

        #[extrinsic_call]
        update_gift(RawOrigin::Root, 0, false);

        let gift = Gifts::<T>::get(0).unwrap();
        assert!(!gift.enabled);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
