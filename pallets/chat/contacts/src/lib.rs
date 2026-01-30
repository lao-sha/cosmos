#![cfg_attr(not(feature = "std"), no_std)]

//! # Pallet Contacts - 通讯录管理模块
//!
//! ## 功能概述
//!
//! `pallet-chat-contacts` 是 Cosmos 区块链的去中心化通讯录管理模块，提供以下功能：
//!
//! - **联系人管理**：添加、删除、修改联系人信息
//! - **分组管理**：创建分组、管理分组成员
//! - **黑名单机制**：屏蔽不需要的用户
//! - **好友关系**：双向好友关系验证
//! - **备注系统**：为联系人添加个性化备注
//!
//! ## 接口说明
//!
//! ### 可调用函数
//!
//! - `add_contact` - 添加联系人
//! - `remove_contact` - 删除联系人
//! - `update_contact` - 更新联系人信息
//! - `create_group` - 创建分组
//! - `delete_group` - 删除分组
//! - `rename_group` - 重命名分组
//! - `block_account` - 添加到黑名单
//! - `unblock_account` - 从黑名单移除
//! - `send_friend_request` - 发送好友申请
//! - `accept_friend_request` - 接受好友申请
//! - `reject_friend_request` - 拒绝好友申请

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::WeightInfo;
	use frame_support::{
		pallet_prelude::*,
		traits::Get,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::Saturating;
	use sp_std::vec::Vec;

	/// 好友关系状态
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
	pub enum FriendStatus {
		/// 单向添加（仅我添加了对方）
		#[default]
		OneWay,
		/// 双向好友（互相添加）
		Mutual,
		/// 待确认（已发送好友申请）
		Pending,
	}

	impl FriendStatus {
		/// 转换为 u8 代码用于事件
		pub fn as_u8(&self) -> u8 {
			match self {
				FriendStatus::OneWay => 0,
				FriendStatus::Mutual => 1,
				FriendStatus::Pending => 2,
			}
		}
	}

	/// 联系人信息
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct ContactInfo<T: Config> {
		/// 联系人账户
		pub account: T::AccountId,
		/// 备注名称（可选）
		pub alias: Option<BoundedVec<u8, T::MaxAliasLen>>,
		/// 所属分组列表
		pub groups: BoundedVec<BoundedVec<u8, T::MaxGroupNameLen>, T::MaxGroupsPerContact>,
		/// 好友状态（单向添加 vs 双向好友）
		pub friend_status: FriendStatus,
		/// 添加时间
		pub added_at: BlockNumberFor<T>,
		/// 最后更新时间
		pub updated_at: BlockNumberFor<T>,
	}

	/// 分组信息
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct GroupInfo<T: Config> {
		/// 分组名称
		pub name: BoundedVec<u8, T::MaxGroupNameLen>,
		/// 分组成员数量
		pub member_count: u32,
		/// 创建时间
		pub created_at: BlockNumberFor<T>,
	}

	/// 黑名单记录
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct BlockedInfo<T: Config> {
		/// 被屏蔽的账户
		pub account: T::AccountId,
		/// 屏蔽原因（可选）
		pub reason: Option<BoundedVec<u8, T::MaxReasonLen>>,
		/// 屏蔽时间
		pub blocked_at: BlockNumberFor<T>,
	}

	/// 好友申请信息
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct FriendRequestInfo<T: Config> {
		/// 申请者账户
		pub from: T::AccountId,
		/// 申请留言（可选）
		pub message: Option<BoundedVec<u8, T::MaxMessageLen>>,
		/// 申请时间（区块号）
		pub requested_at: BlockNumberFor<T>,
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// 配置接口
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// 运行时事件类型
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// 权重信息
		type WeightInfo: WeightInfo;

		/// 最大联系人数量（每个用户）
		#[pallet::constant]
		type MaxContacts: Get<u32>;

		/// 最大分组数量（每个用户）
		#[pallet::constant]
		type MaxGroups: Get<u32>;

		/// 每个分组的最大成员数
		#[pallet::constant]
		type MaxContactsPerGroup: Get<u32>;

		/// 每个联系人可归属的最大分组数
		#[pallet::constant]
		type MaxGroupsPerContact: Get<u32>;

		/// 最大黑名单数量
		#[pallet::constant]
		type MaxBlacklist: Get<u32>;

		/// 备注名最大长度（字节）
		#[pallet::constant]
		type MaxAliasLen: Get<u32>;

		/// 分组名最大长度（字节）
		#[pallet::constant]
		type MaxGroupNameLen: Get<u32>;

		/// 屏蔽原因最大长度（字节）
		#[pallet::constant]
		type MaxReasonLen: Get<u32>;

		/// 好友申请留言最大长度
		#[pallet::constant]
		type MaxMessageLen: Get<u32>;

		/// 好友申请有效期（区块数）
		#[pallet::constant]
		type FriendRequestExpiry: Get<BlockNumberFor<Self>>;
	}

	// ====== 存储项 ======

	/// 用户的联系人列表
	/// 存储映射：(用户账户, 联系人账户) => 联系人信息
	#[pallet::storage]
	#[pallet::getter(fn contacts)]
	pub type Contacts<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId, // 用户
		Blake2_128Concat,
		T::AccountId, // 联系人
		ContactInfo<T>,
		OptionQuery,
	>;

	/// 用户的联系人数量统计
	#[pallet::storage]
	#[pallet::getter(fn contact_count)]
	pub type ContactCount<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	/// 用户的分组数量统计
	#[pallet::storage]
	#[pallet::getter(fn group_count)]
	pub type GroupCount<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	/// 用户的黑名单数量统计
	#[pallet::storage]
	#[pallet::getter(fn blacklist_count)]
	pub type BlacklistCount<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	/// 用户的分组信息
	/// 存储映射：(用户账户, 分组名) => 分组信息
	#[pallet::storage]
	#[pallet::getter(fn groups)]
	pub type Groups<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxGroupNameLen>,
		GroupInfo<T>,
		OptionQuery,
	>;

	/// 分组中的成员列表
	/// 存储映射：(用户账户, 分组名) => 成员账户列表
	#[pallet::storage]
	#[pallet::getter(fn group_members)]
	pub type GroupMembers<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxGroupNameLen>,
		BoundedVec<T::AccountId, T::MaxContactsPerGroup>,
		ValueQuery,
	>;

	/// 用户的黑名单
	/// 存储映射：(用户账户, 被屏蔽账户) => 黑名单记录
	#[pallet::storage]
	#[pallet::getter(fn blacklist)]
	pub type Blacklist<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		BlockedInfo<T>,
		OptionQuery,
	>;

	/// 好友申请记录
	/// 存储映射：(接收者账户, 申请者账户) => 申请详情
	#[pallet::storage]
	#[pallet::getter(fn friend_requests)]
	pub type FriendRequests<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId, // 接收者
		Blake2_128Concat,
		T::AccountId, // 申请者
		FriendRequestInfo<T>, // 申请详情（包含留言和时间）
		OptionQuery,
	>;

	/// 用户的待处理好友申请数量
	#[pallet::storage]
	#[pallet::getter(fn pending_request_count)]
	pub type PendingRequestCount<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	// ====== 事件 ======

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// 联系人已添加
		/// [添加者, 联系人账户, 好友状态（0=单向，1=双向，2=待确认）]
		ContactAdded { who: T::AccountId, contact: T::AccountId, friend_status: u8 },

		/// 联系人已删除
		/// [删除者, 联系人账户]
		ContactRemoved { who: T::AccountId, contact: T::AccountId },

		/// 联系人信息已更新
		/// [更新者, 联系人账户]
		ContactUpdated { who: T::AccountId, contact: T::AccountId },

		/// 分组已创建
		/// [创建者, 分组名]
		GroupCreated { who: T::AccountId, name: BoundedVec<u8, T::MaxGroupNameLen> },

		/// 分组已删除
		/// [删除者, 分组名]
		GroupDeleted { who: T::AccountId, name: BoundedVec<u8, T::MaxGroupNameLen> },

		/// 分组已重命名
		/// [操作者, 旧名称, 新名称]
		GroupRenamed {
			who: T::AccountId,
			old_name: BoundedVec<u8, T::MaxGroupNameLen>,
			new_name: BoundedVec<u8, T::MaxGroupNameLen>,
		},

		/// 账户已加入黑名单
		/// [操作者, 被屏蔽账户]
		AccountBlocked { who: T::AccountId, blocked: T::AccountId },

		/// 账户已从黑名单移除
		/// [操作者, 解除屏蔽账户]
		AccountUnblocked { who: T::AccountId, unblocked: T::AccountId },

		/// 好友申请已发送
		/// [申请者, 目标账户]
		FriendRequestSent { from: T::AccountId, to: T::AccountId },

		/// 好友申请已接受
		/// [接受者, 申请者]
		FriendRequestAccepted { who: T::AccountId, requester: T::AccountId },

		/// 好友申请已拒绝
		/// [拒绝者, 申请者]
		FriendRequestRejected { who: T::AccountId, requester: T::AccountId },

		/// 好友关系状态变更（单向 -> 双向）
		/// [账户1, 账户2, 新状态（0=单向，1=双向，2=待确认）]
		FriendStatusChanged {
			account1: T::AccountId,
			account2: T::AccountId,
			new_status: u8,
		},
	}

	// ====== 错误类型 ======

	#[pallet::error]
	pub enum Error<T> {
		/// 联系人已存在
		ContactAlreadyExists,
		/// 联系人不存在
		ContactNotFound,
		/// 联系人数量已达上限
		TooManyContacts,
		/// 不能添加自己为联系人
		CannotAddSelf,
		/// 已被对方加入黑名单
		BlockedByOther,
		/// 分组已存在
		GroupAlreadyExists,
		/// 分组不存在
		GroupNotFound,
		/// 分组数量已达上限
		TooManyGroups,
		/// 分组成员数量已达上限
		GroupMembersFull,
		/// 分组名称为空
		EmptyGroupName,
		/// 账户已在黑名单中
		AlreadyBlocked,
		/// 账户不在黑名单中
		NotBlocked,
		/// 黑名单数量已达上限
		TooManyBlocked,
		/// 好友申请已存在
		FriendRequestAlreadyExists,
		/// 好友申请不存在
		FriendRequestNotFound,
		/// 好友申请已过期
		FriendRequestExpired,
		/// 备注名称过长
		AliasTooLong,
		/// 屏蔽原因过长
		ReasonTooLong,
		/// 联系人分组超出上限
		TooManyGroupsForContact,
		/// 无效的分组名称
		InvalidGroupName,
	}

	// ====== 可调用函数 ======

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// 添加联系人
		///
		/// # 参数
		/// - `origin`: 调用者来源
		/// - `contact`: 联系人账户地址
		/// - `alias`: 可选的备注名称
		/// - `groups`: 所属分组列表（可为空）
		///
		/// # 功能说明
		/// - 验证调用者身份
		/// - 检查是否在对方黑名单中
		/// - 检查联系人数量是否超限
		/// - 检查联系人是否已存在
		/// - 验证分组是否存在（如果指定）
		/// - 检查好友关系状态（如果对方也添加了我，则为双向好友）
		/// - 创建联系人记录
		/// - 更新分组成员列表
		/// - 触发事件
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::add_contact())]
		pub fn add_contact(
			origin: OriginFor<T>,
			contact: T::AccountId,
			alias: Option<BoundedVec<u8, T::MaxAliasLen>>,
			groups: BoundedVec<BoundedVec<u8, T::MaxGroupNameLen>, T::MaxGroupsPerContact>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 不能添加自己
			ensure!(who != contact, Error::<T>::CannotAddSelf);

			// 2. 检查是否在对方黑名单中
			ensure!(!Self::is_blocked(&contact, &who), Error::<T>::BlockedByOther);

			// 3. 检查联系人是否已存在
			ensure!(!Contacts::<T>::contains_key(&who, &contact), Error::<T>::ContactAlreadyExists);

			// 4. 检查联系人数量是否超限
			let count = ContactCount::<T>::get(&who);
			ensure!(count < T::MaxContacts::get(), Error::<T>::TooManyContacts);

			// 5. 验证所有分组是否存在
			for group_name in groups.iter() {
				ensure!(Groups::<T>::contains_key(&who, group_name), Error::<T>::GroupNotFound);
			}

			// 6. 检查好友关系状态
			let friend_status = if Contacts::<T>::contains_key(&contact, &who) {
				FriendStatus::Mutual
			} else {
				FriendStatus::OneWay
			};

			let current_block = frame_system::Pallet::<T>::block_number();

			// 7. 创建联系人记录
			let contact_info = ContactInfo {
				account: contact.clone(),
				alias,
				groups: groups.clone(),
				friend_status: friend_status.clone(),
				added_at: current_block,
				updated_at: current_block,
			};

			Contacts::<T>::insert(&who, &contact, contact_info);

			// 8. 更新联系人数量
			ContactCount::<T>::insert(&who, count + 1);

			// 9. 添加到分组成员列表
			for group_name in groups.iter() {
				GroupMembers::<T>::try_mutate(&who, group_name, |members| -> DispatchResult {
					members
						.try_push(contact.clone())
						.map_err(|_| Error::<T>::GroupMembersFull)?;
					Ok(())
				})?;

				// 更新分组成员数量
				Groups::<T>::mutate(&who, group_name, |maybe_group| {
					if let Some(group) = maybe_group {
						group.member_count = group.member_count.saturating_add(1);
					}
				});
			}

			// 10. 如果形成双向好友，更新对方的状态
			if friend_status == FriendStatus::Mutual {
				Contacts::<T>::mutate(&contact, &who, |maybe_info| {
					if let Some(info) = maybe_info {
						info.friend_status = FriendStatus::Mutual;
						info.updated_at = current_block;
					}
				});

				Self::deposit_event(Event::FriendStatusChanged {
					account1: who.clone(),
					account2: contact.clone(),
					new_status: FriendStatus::Mutual.as_u8(),
				});
			}

			// 11. 触发事件
			Self::deposit_event(Event::ContactAdded { who, contact, friend_status: friend_status.as_u8() });

			Ok(())
		}

		/// 删除联系人
		///
		/// # 参数
		/// - `origin`: 调用者来源
		/// - `contact`: 要删除的联系人账户
		///
		/// # 功能说明
		/// - 验证调用者身份
		/// - 检查联系人是否存在
		/// - 从所有分组中移除该联系人
		/// - 删除联系人记录
		/// - 更新计数
		/// - 如果对方也添加了我，更新对方的好友状态为单向
		/// - 触发事件
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::remove_contact())]
		pub fn remove_contact(origin: OriginFor<T>, contact: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 检查联系人是否存在
			let contact_info =
				Contacts::<T>::get(&who, &contact).ok_or(Error::<T>::ContactNotFound)?;

			// 2. 从所有分组中移除该联系人
			for group_name in contact_info.groups.iter() {
				GroupMembers::<T>::mutate(&who, group_name, |members| {
					members.retain(|acc| acc != &contact);
				});

				// 更新分组成员数量
				Groups::<T>::mutate(&who, group_name, |maybe_group| {
					if let Some(group) = maybe_group {
						group.member_count = group.member_count.saturating_sub(1);
					}
				});
			}

			// 3. 删除联系人记录
			Contacts::<T>::remove(&who, &contact);

			// 4. 更新计数
			ContactCount::<T>::mutate(&who, |count| {
				*count = count.saturating_sub(1);
			});

			// 5. 如果对方也添加了我，更新对方的好友状态为单向
			if Contacts::<T>::contains_key(&contact, &who) {
				let current_block = frame_system::Pallet::<T>::block_number();
				Contacts::<T>::mutate(&contact, &who, |maybe_info| {
					if let Some(info) = maybe_info {
						info.friend_status = FriendStatus::OneWay;
						info.updated_at = current_block;
					}
				});

				Self::deposit_event(Event::FriendStatusChanged {
					account1: contact.clone(),
					account2: who.clone(),
					new_status: FriendStatus::OneWay.as_u8(),
				});
			}

			// 6. 触发事件
			Self::deposit_event(Event::ContactRemoved { who, contact });

			Ok(())
		}

		/// 更新联系人信息（备注、分组）
		///
		/// # 参数
		/// - `origin`: 调用者来源
		/// - `contact`: 联系人账户
		/// - `alias`: 新的备注名称（None 表示清除备注）
		/// - `groups`: 新的分组列表
		///
		/// # 功能说明
		/// - 验证调用者身份
		/// - 检查联系人是否存在
		/// - 验证新分组是否存在
		/// - 从旧分组中移除
		/// - 添加到新分组
		/// - 更新联系人信息
		/// - 触发事件
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::update_contact())]
		pub fn update_contact(
			origin: OriginFor<T>,
			contact: T::AccountId,
			alias: Option<BoundedVec<u8, T::MaxAliasLen>>,
			groups: BoundedVec<BoundedVec<u8, T::MaxGroupNameLen>, T::MaxGroupsPerContact>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 检查联系人是否存在
			Contacts::<T>::try_mutate(&who, &contact, |maybe_info| -> DispatchResult {
				let info = maybe_info.as_mut().ok_or(Error::<T>::ContactNotFound)?;

				// 2. 验证新分组是否存在
				for group_name in groups.iter() {
					ensure!(
						Groups::<T>::contains_key(&who, group_name),
						Error::<T>::GroupNotFound
					);
				}

				// 3. 从旧分组中移除
				for old_group in info.groups.iter() {
					if !groups.contains(old_group) {
						GroupMembers::<T>::mutate(&who, old_group, |members| {
							members.retain(|acc| acc != &contact);
						});

						// 更新分组成员数量
						Groups::<T>::mutate(&who, old_group, |maybe_group| {
							if let Some(group) = maybe_group {
								group.member_count = group.member_count.saturating_sub(1);
							}
						});
					}
				}

				// 4. 添加到新分组
				for new_group in groups.iter() {
					if !info.groups.contains(new_group) {
						GroupMembers::<T>::try_mutate(
							&who,
							new_group,
							|members| -> DispatchResult {
								members
									.try_push(contact.clone())
									.map_err(|_| Error::<T>::GroupMembersFull)?;
								Ok(())
							},
						)?;

						// 更新分组成员数量
						Groups::<T>::mutate(&who, new_group, |maybe_group| {
							if let Some(group) = maybe_group {
								group.member_count = group.member_count.saturating_add(1);
							}
						});
					}
				}

				// 5. 更新联系人信息
				info.alias = alias;
				info.groups = groups;
				info.updated_at = frame_system::Pallet::<T>::block_number();

				Ok(())
			})?;

			// 6. 触发事件
			Self::deposit_event(Event::ContactUpdated { who, contact });

			Ok(())
		}

		/// 创建分组
		///
		/// # 参数
		/// - `origin`: 调用者来源
		/// - `name`: 分组名称
		///
		/// # 功能说明
		/// - 验证调用者身份
		/// - 检查分组名称是否为空
		/// - 检查分组是否已存在
		/// - 检查分组数量是否超限
		/// - 创建分组记录
		/// - 初始化空成员列表
		/// - 触发事件
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::create_group())]
		pub fn create_group(
			origin: OriginFor<T>,
			name: BoundedVec<u8, T::MaxGroupNameLen>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 检查分组名称是否为空
			ensure!(!name.is_empty(), Error::<T>::EmptyGroupName);

			// 2. 检查分组是否已存在
			ensure!(!Groups::<T>::contains_key(&who, &name), Error::<T>::GroupAlreadyExists);

			// 3. 检查分组数量（使用计数器，O(1) 复杂度）
			let count = GroupCount::<T>::get(&who);
			ensure!(count < T::MaxGroups::get(), Error::<T>::TooManyGroups);

			// 4. 创建分组记录
			let group_info = GroupInfo {
				name: name.clone(),
				member_count: 0,
				created_at: frame_system::Pallet::<T>::block_number(),
			};

			Groups::<T>::insert(&who, &name, group_info);

			// 5. 更新分组计数
			GroupCount::<T>::insert(&who, count + 1);

			// 6. 初始化空成员列表
			GroupMembers::<T>::insert(&who, &name, BoundedVec::default());

			// 7. 触发事件
			Self::deposit_event(Event::GroupCreated { who, name });

			Ok(())
		}

		/// 删除分组
		///
		/// # 参数
		/// - `origin`: 调用者来源
		/// - `name`: 分组名称
		///
		/// # 功能说明
		/// - 验证调用者身份
		/// - 检查分组是否存在
		/// - 从所有联系人中移除该分组标记
		/// - 删除分组成员列表
		/// - 删除分组记录
		/// - 更新分组计数
		/// - 触发事件
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::delete_group())]
		pub fn delete_group(
			origin: OriginFor<T>,
			name: BoundedVec<u8, T::MaxGroupNameLen>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 检查分组是否存在
			ensure!(Groups::<T>::contains_key(&who, &name), Error::<T>::GroupNotFound);

			// 2. 获取分组成员
			let members = GroupMembers::<T>::get(&who, &name);

			// 3. 从所有联系人中移除该分组标记
			for member in members.iter() {
				Contacts::<T>::mutate(&who, member, |maybe_info| {
					if let Some(info) = maybe_info {
						info.groups.retain(|g| g != &name);
						info.updated_at = frame_system::Pallet::<T>::block_number();
					}
				});
			}

			// 4. 删除分组成员列表
			GroupMembers::<T>::remove(&who, &name);

			// 5. 删除分组记录
			Groups::<T>::remove(&who, &name);

			// 6. 更新分组计数
			GroupCount::<T>::mutate(&who, |count| {
				*count = count.saturating_sub(1);
			});

			// 7. 触发事件
			Self::deposit_event(Event::GroupDeleted { who, name });

			Ok(())
		}

		/// 重命名分组
		///
		/// # 参数
		/// - `origin`: 调用者来源
		/// - `old_name`: 旧分组名
		/// - `new_name`: 新分组名
		///
		/// # 功能说明
		/// - 验证调用者身份
		/// - 检查新名称是否为空
		/// - 检查旧分组是否存在
		/// - 检查新名称是否已被使用
		/// - 更新所有联系人的分组标记
		/// - 迁移分组成员列表
		/// - 删除旧分组，创建新分组
		/// - 触发事件
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::rename_group())]
		pub fn rename_group(
			origin: OriginFor<T>,
			old_name: BoundedVec<u8, T::MaxGroupNameLen>,
			new_name: BoundedVec<u8, T::MaxGroupNameLen>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 检查新名称是否为空
			ensure!(!new_name.is_empty(), Error::<T>::EmptyGroupName);

			// 2. 检查旧分组是否存在
			let mut group_info =
				Groups::<T>::get(&who, &old_name).ok_or(Error::<T>::GroupNotFound)?;

			// 3. 检查新名称是否已被使用
			ensure!(!Groups::<T>::contains_key(&who, &new_name), Error::<T>::GroupAlreadyExists);

			// 4. 获取分组成员
			let members = GroupMembers::<T>::get(&who, &old_name);

			// 5. 更新所有联系人的分组标记
			for member in members.iter() {
				Contacts::<T>::mutate(&who, member, |maybe_info| {
					if let Some(info) = maybe_info {
						// 找到旧分组并替换为新分组
						if let Some(pos) = info.groups.iter().position(|g| g == &old_name) {
							// 移除旧分组
							info.groups.remove(pos);
							// 添加新分组（忽略错误，因为已经检查过容量）
							let _ = info.groups.try_push(new_name.clone());
						}
						info.updated_at = frame_system::Pallet::<T>::block_number();
					}
				});
			}

			// 6. 更新分组信息中的名称
			group_info.name = new_name.clone();

			// 7. 创建新分组记录
			Groups::<T>::insert(&who, &new_name, group_info);

			// 8. 迁移分组成员列表
			GroupMembers::<T>::insert(&who, &new_name, members);

			// 9. 删除旧分组记录和成员列表
			Groups::<T>::remove(&who, &old_name);
			GroupMembers::<T>::remove(&who, &old_name);

			// 10. 触发事件
			Self::deposit_event(Event::GroupRenamed { who, old_name, new_name });

			Ok(())
		}

		/// 添加账户到黑名单
		///
		/// # 参数
		/// - `origin`: 调用者来源
		/// - `account`: 要屏蔽的账户
		/// - `reason`: 屏蔽原因（可选）
		///
		/// # 功能说明
		/// - 验证调用者身份
		/// - 检查是否已在黑名单
		/// - 检查黑名单数量是否超限
		/// - 创建黑名单记录
		/// - 更新黑名单计数
		/// - 如果该账户在联系人列表中，自动删除
		/// - 触发事件
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::block_account())]
		pub fn block_account(
			origin: OriginFor<T>,
			account: T::AccountId,
			reason: Option<BoundedVec<u8, T::MaxReasonLen>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 不能屏蔽自己
			ensure!(who != account, Error::<T>::CannotAddSelf);

			// 2. 检查是否已在黑名单
			ensure!(!Blacklist::<T>::contains_key(&who, &account), Error::<T>::AlreadyBlocked);

			// 3. 检查黑名单数量（使用计数器，O(1) 复杂度）
			let count = BlacklistCount::<T>::get(&who);
			ensure!(count < T::MaxBlacklist::get(), Error::<T>::TooManyBlocked);

			// 4. 创建黑名单记录
			let blocked_info = BlockedInfo {
				account: account.clone(),
				reason,
				blocked_at: frame_system::Pallet::<T>::block_number(),
			};

			Blacklist::<T>::insert(&who, &account, blocked_info);

			// 5. 更新黑名单计数
			BlacklistCount::<T>::insert(&who, count + 1);

			// 6. 如果该账户在联系人列表中，自动删除
			if Contacts::<T>::contains_key(&who, &account) {
				// 调用内部删除逻辑（不触发额外事件）
				let contact_info = Contacts::<T>::get(&who, &account).expect("已检查存在性");

				// 从所有分组中移除
				for group_name in contact_info.groups.iter() {
					GroupMembers::<T>::mutate(&who, group_name, |members| {
						members.retain(|acc| acc != &account);
					});

					Groups::<T>::mutate(&who, group_name, |maybe_group| {
						if let Some(group) = maybe_group {
							group.member_count = group.member_count.saturating_sub(1);
						}
					});
				}

				// 删除联系人记录
				Contacts::<T>::remove(&who, &account);

				// 更新计数
				ContactCount::<T>::mutate(&who, |count| {
					*count = count.saturating_sub(1);
				});
			}

			// 7. 触发事件
			Self::deposit_event(Event::AccountBlocked { who, blocked: account });

			Ok(())
		}

		/// 从黑名单移除账户
		///
		/// # 参数
		/// - `origin`: 调用者来源
		/// - `account`: 要解除屏蔽的账户
		///
		/// # 功能说明
		/// - 验证调用者身份
		/// - 检查黑名单记录是否存在
		/// - 删除黑名单记录
		/// - 更新黑名单计数
		/// - 触发事件
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::unblock_account())]
		pub fn unblock_account(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 检查黑名单记录是否存在
			ensure!(Blacklist::<T>::contains_key(&who, &account), Error::<T>::NotBlocked);

			// 2. 删除黑名单记录
			Blacklist::<T>::remove(&who, &account);

			// 3. 更新黑名单计数
			BlacklistCount::<T>::mutate(&who, |count| {
				*count = count.saturating_sub(1);
			});

			// 4. 触发事件
			Self::deposit_event(Event::AccountUnblocked { who, unblocked: account });

			Ok(())
		}

		/// 发送好友申请
		///
		/// # 参数
		/// - `origin`: 调用者来源
		/// - `target`: 目标账户
		/// - `message`: 申请留言（可选）
		///
		/// # 功能说明
		/// - 验证调用者身份
		/// - 检查是否在对方黑名单中
		/// - 检查是否已发送过申请
		/// - 检查是否已经是联系人
		/// - 创建申请记录（包含留言）
		/// - 更新待处理申请计数
		/// - 触发事件（对方可通过事件接收通知）
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::send_friend_request())]
		pub fn send_friend_request(
			origin: OriginFor<T>,
			target: T::AccountId,
			message: Option<BoundedVec<u8, T::MaxMessageLen>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 不能给自己发送申请
			ensure!(who != target, Error::<T>::CannotAddSelf);

			// 2. 检查是否在对方黑名单中
			ensure!(!Self::is_blocked(&target, &who), Error::<T>::BlockedByOther);

			// 3. 检查是否已发送过申请
			ensure!(
				!FriendRequests::<T>::contains_key(&target, &who),
				Error::<T>::FriendRequestAlreadyExists
			);

			// 4. 检查是否已经是联系人
			ensure!(!Contacts::<T>::contains_key(&who, &target), Error::<T>::ContactAlreadyExists);

			// 5. 创建申请记录（包含留言）
			let current_block = frame_system::Pallet::<T>::block_number();
			let request_info = FriendRequestInfo {
				from: who.clone(),
				message,
				requested_at: current_block,
			};
			FriendRequests::<T>::insert(&target, &who, request_info);

			// 6. 更新待处理申请计数
			PendingRequestCount::<T>::mutate(&target, |count| {
				*count = count.saturating_add(1);
			});

			// 7. 触发事件
			Self::deposit_event(Event::FriendRequestSent { from: who, to: target });

			Ok(())
		}

		/// 接受好友申请
		///
		/// # 参数
		/// - `origin`: 调用者来源
		/// - `requester`: 申请者账户
		///
		/// # 功能说明
		/// - 验证调用者身份
		/// - 检查申请记录是否存在
		/// - 检查申请是否过期
		/// - 自动添加对方为联系人（双向）
		/// - 删除申请记录
		/// - 更新待处理申请计数
		/// - 触发事件
		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::accept_friend_request())]
		pub fn accept_friend_request(
			origin: OriginFor<T>,
			requester: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 检查申请记录是否存在
			let request_info = FriendRequests::<T>::get(&who, &requester)
				.ok_or(Error::<T>::FriendRequestNotFound)?;

			// 2. 检查申请是否过期
			let current_block = frame_system::Pallet::<T>::block_number();
			let expiry_blocks: BlockNumberFor<T> = T::FriendRequestExpiry::get();
			ensure!(
				current_block.saturating_sub(request_info.requested_at) <= expiry_blocks,
				Error::<T>::FriendRequestExpired
			);

			// 3. 检查是否在对方黑名单中
			ensure!(!Self::is_blocked(&requester, &who), Error::<T>::BlockedByOther);

			// 4. 检查联系人数量
			let count = ContactCount::<T>::get(&who);
			ensure!(count < T::MaxContacts::get(), Error::<T>::TooManyContacts);

			// 5. 自动添加对方为联系人（双向好友）
			let contact_info = ContactInfo {
				account: requester.clone(),
				alias: None,
				groups: BoundedVec::default(),
				friend_status: FriendStatus::Mutual,
				added_at: current_block,
				updated_at: current_block,
			};

			Contacts::<T>::insert(&who, &requester, contact_info);
			ContactCount::<T>::insert(&who, count + 1);

			// 6. 更新申请者的好友状态（如果存在）
			if Contacts::<T>::contains_key(&requester, &who) {
				Contacts::<T>::mutate(&requester, &who, |maybe_info| {
					if let Some(info) = maybe_info {
						info.friend_status = FriendStatus::Mutual;
						info.updated_at = current_block;
					}
				});
			}

			// 7. 删除申请记录
			FriendRequests::<T>::remove(&who, &requester);

			// 8. 更新待处理申请计数
			PendingRequestCount::<T>::mutate(&who, |count| {
				*count = count.saturating_sub(1);
			});

			// 9. 触发事件
			Self::deposit_event(Event::FriendRequestAccepted { who: who.clone(), requester: requester.clone() });
			Self::deposit_event(Event::FriendStatusChanged {
				account1: who,
				account2: requester,
				new_status: FriendStatus::Mutual.as_u8(),
			});

			Ok(())
		}

		/// 拒绝好友申请
		///
		/// # 参数
		/// - `origin`: 调用者来源
		/// - `requester`: 申请者账户
		///
		/// # 功能说明
		/// - 验证调用者身份
		/// - 检查申请记录是否存在
		/// - 删除申请记录
		/// - 更新待处理申请计数
		/// - 触发事件
		#[pallet::call_index(10)]
		#[pallet::weight(T::WeightInfo::reject_friend_request())]
		pub fn reject_friend_request(
			origin: OriginFor<T>,
			requester: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 1. 检查申请记录是否存在
			ensure!(
				FriendRequests::<T>::contains_key(&who, &requester),
				Error::<T>::FriendRequestNotFound
			);

			// 2. 删除申请记录
			FriendRequests::<T>::remove(&who, &requester);

			// 3. 更新待处理申请计数
			PendingRequestCount::<T>::mutate(&who, |count| {
				*count = count.saturating_sub(1);
			});

			// 4. 触发事件
			Self::deposit_event(Event::FriendRequestRejected { who, requester });

			Ok(())
		}
	}

	// ====== 辅助方法 ======

	impl<T: Config> Pallet<T> {
		/// 检查两个账户是否为双向好友
		pub fn are_mutual_friends(account1: &T::AccountId, account2: &T::AccountId) -> bool {
			if let Some(info1) = Contacts::<T>::get(account1, account2) {
				if let Some(info2) = Contacts::<T>::get(account2, account1) {
					return info1.friend_status == FriendStatus::Mutual &&
						info2.friend_status == FriendStatus::Mutual
				}
			}
			false
		}

		/// 检查账户是否在黑名单中
		pub fn is_blocked(blocker: &T::AccountId, account: &T::AccountId) -> bool {
			Blacklist::<T>::contains_key(blocker, account)
		}

		/// 获取用户的所有联系人账户列表
		pub fn get_all_contacts(account: &T::AccountId) -> Vec<T::AccountId> {
			Contacts::<T>::iter_prefix(account).map(|(contact, _)| contact).collect()
		}

		/// 获取分组的所有成员
		pub fn get_group_members(
			account: &T::AccountId,
			group: &BoundedVec<u8, T::MaxGroupNameLen>,
		) -> Vec<T::AccountId> {
			GroupMembers::<T>::get(account, group).to_vec()
		}

		/// 检查好友申请是否过期
		pub fn is_request_expired(requested_at: BlockNumberFor<T>) -> bool {
			let current_block = frame_system::Pallet::<T>::block_number();
			let expiry_blocks: BlockNumberFor<T> = T::FriendRequestExpiry::get();
			current_block.saturating_sub(requested_at) > expiry_blocks
		}

		/// 获取用户的所有待处理好友申请
		/// 返回申请者账户和申请详情的列表
		pub fn get_pending_requests(account: &T::AccountId) -> Vec<(T::AccountId, FriendRequestInfo<T>)> {
			FriendRequests::<T>::iter_prefix(account)
				.filter(|(_, info)| !Self::is_request_expired(info.requested_at))
				.collect()
		}

		/// 获取用户的所有待处理好友申请（包含过期的）
		/// 返回申请者账户和申请详情的列表
		pub fn get_all_pending_requests(account: &T::AccountId) -> Vec<(T::AccountId, FriendRequestInfo<T>)> {
			FriendRequests::<T>::iter_prefix(account).collect()
		}

		/// 获取指定好友申请的详情
		pub fn get_friend_request(
			receiver: &T::AccountId,
			requester: &T::AccountId,
		) -> Option<FriendRequestInfo<T>> {
			FriendRequests::<T>::get(receiver, requester)
		}
	}
}
