//! Mock runtime for pallet-chat-contacts tests

use crate as pallet_contacts;
use frame_support::{
	derive_impl, parameter_types, BoundedVec,
};
use sp_runtime::{traits::IdentityLookup, BuildStorage};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Contacts: pallet_contacts,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
}

parameter_types! {
	pub const MaxContacts: u32 = 100;
	pub const MaxGroups: u32 = 20;
	pub const MaxContactsPerGroup: u32 = 50;
	pub const MaxGroupsPerContact: u32 = 10;
	pub const MaxBlacklist: u32 = 50;
	pub const MaxAliasLen: u32 = 64;
	pub const MaxGroupNameLen: u32 = 32;
	pub const MaxReasonLen: u32 = 256;
	pub const MaxMessageLen: u32 = 512;
	pub const FriendRequestExpiry: u64 = 100800; // 约7天（按6秒出块）
}

impl pallet_contacts::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MaxContacts = MaxContacts;
	type MaxGroups = MaxGroups;
	type MaxContactsPerGroup = MaxContactsPerGroup;
	type MaxGroupsPerContact = MaxGroupsPerContact;
	type MaxBlacklist = MaxBlacklist;
	type MaxAliasLen = MaxAliasLen;
	type MaxGroupNameLen = MaxGroupNameLen;
	type MaxReasonLen = MaxReasonLen;
	type MaxMessageLen = MaxMessageLen;
	type FriendRequestExpiry = FriendRequestExpiry;
}

// 类型别名，用于测试
pub type AliasVec = BoundedVec<u8, MaxAliasLen>;
pub type GroupNameVec = BoundedVec<u8, MaxGroupNameLen>;
pub type ReasonVec = BoundedVec<u8, MaxReasonLen>;
pub type MessageVec = BoundedVec<u8, MaxMessageLen>;
pub type GroupsVec = BoundedVec<GroupNameVec, MaxGroupsPerContact>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
