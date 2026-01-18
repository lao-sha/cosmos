//! Benchmarking setup for pallet-contacts

#![cfg(feature = "runtime-benchmarks")]

use super::*;

#[allow(unused)]
use crate::Pallet as Contacts;
use frame_benchmarking::v2::*;
use frame_support::BoundedVec;
use frame_system::RawOrigin;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn add_contact() {
		let caller: T::AccountId = whitelisted_caller();
		let contact: T::AccountId = account("contact", 0, 0);
		let alias: BoundedVec<u8, T::MaxAliasLen> =
			BoundedVec::try_from(b"Test Contact".to_vec()).unwrap();
		let groups: BoundedVec<BoundedVec<u8, T::MaxGroupNameLen>, T::MaxGroupsPerContact> =
			BoundedVec::default();

		#[extrinsic_call]
		add_contact(RawOrigin::Signed(caller.clone()), contact.clone(), Some(alias), groups);

		assert!(Contacts::<T>::contains_key(&caller, &contact));
	}

	#[benchmark]
	fn remove_contact() {
		let caller: T::AccountId = whitelisted_caller();
		let contact: T::AccountId = account("contact", 0, 0);
		let alias: BoundedVec<u8, T::MaxAliasLen> =
			BoundedVec::try_from(b"Test Contact".to_vec()).unwrap();

		// 先添加联系人
		Contacts::<T>::add_contact(
			RawOrigin::Signed(caller.clone()).into(),
			contact.clone(),
			Some(alias),
			BoundedVec::default(),
		)
		.unwrap();

		#[extrinsic_call]
		remove_contact(RawOrigin::Signed(caller.clone()), contact.clone());

		assert!(!Contacts::<T>::contains_key(&caller, &contact));
	}

	#[benchmark]
	fn update_contact() {
		let caller: T::AccountId = whitelisted_caller();
		let contact: T::AccountId = account("contact", 0, 0);
		let alias1: BoundedVec<u8, T::MaxAliasLen> =
			BoundedVec::try_from(b"Old Name".to_vec()).unwrap();
		let alias2: BoundedVec<u8, T::MaxAliasLen> =
			BoundedVec::try_from(b"New Name".to_vec()).unwrap();

		// 先添加联系人
		Contacts::<T>::add_contact(
			RawOrigin::Signed(caller.clone()).into(),
			contact.clone(),
			Some(alias1),
			BoundedVec::default(),
		)
		.unwrap();

		#[extrinsic_call]
		update_contact(
			RawOrigin::Signed(caller.clone()),
			contact.clone(),
			Some(alias2),
			BoundedVec::default(),
		);

		assert!(Contacts::<T>::contains_key(&caller, &contact));
	}

	#[benchmark]
	fn create_group() {
		let caller: T::AccountId = whitelisted_caller();
		let group_name: BoundedVec<u8, T::MaxGroupNameLen> =
			BoundedVec::try_from(b"Friends".to_vec()).unwrap();

		#[extrinsic_call]
		create_group(RawOrigin::Signed(caller.clone()), group_name.clone());

		assert!(Groups::<T>::contains_key(&caller, &group_name));
	}

	#[benchmark]
	fn delete_group() {
		let caller: T::AccountId = whitelisted_caller();
		let group_name: BoundedVec<u8, T::MaxGroupNameLen> =
			BoundedVec::try_from(b"Friends".to_vec()).unwrap();

		// 先创建分组
		Contacts::<T>::create_group(RawOrigin::Signed(caller.clone()).into(), group_name.clone())
			.unwrap();

		#[extrinsic_call]
		delete_group(RawOrigin::Signed(caller.clone()), group_name.clone());

		assert!(!Groups::<T>::contains_key(&caller, &group_name));
	}

	#[benchmark]
	fn rename_group() {
		let caller: T::AccountId = whitelisted_caller();
		let old_name: BoundedVec<u8, T::MaxGroupNameLen> =
			BoundedVec::try_from(b"OldGroup".to_vec()).unwrap();
		let new_name: BoundedVec<u8, T::MaxGroupNameLen> =
			BoundedVec::try_from(b"NewGroup".to_vec()).unwrap();

		// 先创建分组
		Contacts::<T>::create_group(RawOrigin::Signed(caller.clone()).into(), old_name.clone())
			.unwrap();

		#[extrinsic_call]
		rename_group(RawOrigin::Signed(caller.clone()), old_name.clone(), new_name.clone());

		assert!(!Groups::<T>::contains_key(&caller, &old_name));
		assert!(Groups::<T>::contains_key(&caller, &new_name));
	}

	#[benchmark]
	fn block_account() {
		let caller: T::AccountId = whitelisted_caller();
		let blocked: T::AccountId = account("blocked", 0, 0);
		let reason: BoundedVec<u8, T::MaxReasonLen> =
			BoundedVec::try_from(b"Spam".to_vec()).unwrap();

		#[extrinsic_call]
		block_account(RawOrigin::Signed(caller.clone()), blocked.clone(), Some(reason));

		assert!(Blacklist::<T>::contains_key(&caller, &blocked));
	}

	#[benchmark]
	fn unblock_account() {
		let caller: T::AccountId = whitelisted_caller();
		let blocked: T::AccountId = account("blocked", 0, 0);
		let reason: BoundedVec<u8, T::MaxReasonLen> =
			BoundedVec::try_from(b"Spam".to_vec()).unwrap();

		// 先屏蔽账户
		Contacts::<T>::block_account(
			RawOrigin::Signed(caller.clone()).into(),
			blocked.clone(),
			Some(reason),
		)
		.unwrap();

		#[extrinsic_call]
		unblock_account(RawOrigin::Signed(caller.clone()), blocked.clone());

		assert!(!Blacklist::<T>::contains_key(&caller, &blocked));
	}

	#[benchmark]
	fn send_friend_request() {
		let caller: T::AccountId = whitelisted_caller();
		let target: T::AccountId = account("target", 0, 0);
		let message: BoundedVec<u8, T::MaxMessageLen> =
			BoundedVec::try_from(b"Hi, let's be friends!".to_vec()).unwrap();

		#[extrinsic_call]
		send_friend_request(RawOrigin::Signed(caller.clone()), target.clone(), Some(message));

		assert!(FriendRequests::<T>::contains_key(&target, &caller));
	}

	#[benchmark]
	fn accept_friend_request() {
		let caller: T::AccountId = whitelisted_caller();
		let requester: T::AccountId = account("requester", 0, 0);

		// 先发送好友申请
		Contacts::<T>::send_friend_request(
			RawOrigin::Signed(requester.clone()).into(),
			caller.clone(),
			None,
		)
		.unwrap();

		#[extrinsic_call]
		accept_friend_request(RawOrigin::Signed(caller.clone()), requester.clone());

		assert!(Contacts::<T>::contains_key(&caller, &requester));
		assert!(!FriendRequests::<T>::contains_key(&caller, &requester));
	}

	#[benchmark]
	fn reject_friend_request() {
		let caller: T::AccountId = whitelisted_caller();
		let requester: T::AccountId = account("requester", 0, 0);

		// 先发送好友申请
		Contacts::<T>::send_friend_request(
			RawOrigin::Signed(requester.clone()).into(),
			caller.clone(),
			None,
		)
		.unwrap();

		#[extrinsic_call]
		reject_friend_request(RawOrigin::Signed(caller.clone()), requester.clone());

		assert!(!FriendRequests::<T>::contains_key(&caller, &requester));
	}

	impl_benchmark_test_suite!(Contacts, crate::mock::new_test_ext(), crate::mock::Test);
}
