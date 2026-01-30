# Pallet Contacts 通讯录模块

## 概述

`pallet-chat-contacts` 是 Cosmos 区块链的去中心化通讯录管理模块，提供联系人管理、分组管理、黑名单机制和好友申请等功能。

## 功能特性

### 核心功能

1. **联系人管理**
   - 添加/删除联系人
   - 修改联系人备注
   - 自动识别双向好友关系
   - 联系人数量限制

2. **分组管理**
   - 创建/删除分组
   - 重命名分组
   - 联系人多分组归属
   - 分组成员数量限制

3. **黑名单机制**
   - 添加/移除黑名单
   - 屏蔽原因记录
   - 自动清理被屏蔽联系人
   - 防止被黑名单用户添加

4. **好友申请系统**
   - 发送好友申请
   - 接受/拒绝申请
   - 申请过期机制
   - 申请留言功能

### 安全特性

- ✅ 不能添加自己为联系人
- ✅ 黑名单双向检查
- ✅ 容量限制防止滥用
- ✅ 好友申请防重复
- ✅ 字符串长度限制

## 存储结构

### 主要存储项

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `Contacts` | DoubleMap | (用户, 联系人) => 联系人信息 |
| `ContactCount` | Map | 用户 => 联系人数量 |
| `Groups` | DoubleMap | (用户, 分组名) => 分组信息 |
| `GroupMembers` | DoubleMap | (用户, 分组名) => 成员列表 |
| `Blacklist` | DoubleMap | (用户, 被屏蔽账户) => 黑名单记录 |
| `FriendRequests` | DoubleMap | (接收者, 申请者) => 申请时间 |

## 可调用函数

### 联系人管理

```rust
// 添加联系人
fn add_contact(
    origin: OriginFor<T>,
    contact: T::AccountId,
    alias: Option<BoundedVec<u8, T::MaxAliasLen>>,
    groups: BoundedVec<BoundedVec<u8, T::MaxGroupNameLen>, T::MaxGroupsPerContact>,
) -> DispatchResult;

// 删除联系人
fn remove_contact(
    origin: OriginFor<T>,
    contact: T::AccountId,
) -> DispatchResult;

// 更新联系人信息
fn update_contact(
    origin: OriginFor<T>,
    contact: T::AccountId,
    alias: Option<BoundedVec<u8, T::MaxAliasLen>>,
    groups: BoundedVec<BoundedVec<u8, T::MaxGroupNameLen>, T::MaxGroupsPerContact>,
) -> DispatchResult;
```

### 分组管理

```rust
// 创建分组
fn create_group(
    origin: OriginFor<T>,
    name: BoundedVec<u8, T::MaxGroupNameLen>,
) -> DispatchResult;

// 删除分组
fn delete_group(
    origin: OriginFor<T>,
    name: BoundedVec<u8, T::MaxGroupNameLen>,
) -> DispatchResult;

// 重命名分组
fn rename_group(
    origin: OriginFor<T>,
    old_name: BoundedVec<u8, T::MaxGroupNameLen>,
    new_name: BoundedVec<u8, T::MaxGroupNameLen>,
) -> DispatchResult;
```

### 黑名单管理

```rust
// 添加到黑名单
fn block_account(
    origin: OriginFor<T>,
    account: T::AccountId,
    reason: Option<BoundedVec<u8, T::MaxReasonLen>>,
) -> DispatchResult;

// 从黑名单移除
fn unblock_account(
    origin: OriginFor<T>,
    account: T::AccountId,
) -> DispatchResult;
```

### 好友申请

```rust
// 发送好友申请
fn send_friend_request(
    origin: OriginFor<T>,
    target: T::AccountId,
    message: Option<BoundedVec<u8, T::MaxMessageLen>>,
) -> DispatchResult;

// 接受好友申请
fn accept_friend_request(
    origin: OriginFor<T>,
    requester: T::AccountId,
) -> DispatchResult;

// 拒绝好友申请
fn reject_friend_request(
    origin: OriginFor<T>,
    requester: T::AccountId,
) -> DispatchResult;
```

## 辅助查询函数

```rust
// 检查是否为双向好友
pub fn are_mutual_friends(account1: &T::AccountId, account2: &T::AccountId) -> bool;

// 检查是否在黑名单中
pub fn is_blocked(blocker: &T::AccountId, account: &T::AccountId) -> bool;

// 获取所有联系人
pub fn get_all_contacts(account: &T::AccountId) -> Vec<T::AccountId>;

// 获取分组成员
pub fn get_group_members(
    account: &T::AccountId,
    group: &BoundedVec<u8, T::MaxGroupNameLen>,
) -> Vec<T::AccountId>;
```

## 配置参数

在 Runtime 中配置以下参数：

```rust
impl pallet_contacts::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_contacts::weights::SubstrateWeight<Runtime>;

    // 每个用户最多 500 个联系人
    type MaxContacts = ConstU32<500>;

    // 每个用户最多 50 个分组
    type MaxGroups = ConstU32<50>;

    // 每个分组最多 100 个成员
    type MaxContactsPerGroup = ConstU32<100>;

    // 每个联系人最多属于 10 个分组
    type MaxGroupsPerContact = ConstU32<10>;

    // 黑名单最多 200 个
    type MaxBlacklist = ConstU32<200>;

    // 备注名最长 64 字节
    type MaxAliasLen = ConstU32<64>;

    // 分组名最长 32 字节
    type MaxGroupNameLen = ConstU32<32>;

    // 屏蔽原因最长 256 字节
    type MaxReasonLen = ConstU32<256>;

    // 好友申请留言最长 512 字节
    type MaxMessageLen = ConstU32<512>;

    // 好友申请有效期 7 天
    type FriendRequestExpiry = ConstU32<100800>;
}
```

## 事件

| 事件 | 说明 |
|------|------|
| `ContactAdded` | 联系人已添加 |
| `ContactRemoved` | 联系人已删除 |
| `ContactUpdated` | 联系人信息已更新 |
| `GroupCreated` | 分组已创建 |
| `GroupDeleted` | 分组已删除 |
| `GroupRenamed` | 分组已重命名 |
| `AccountBlocked` | 账户已加入黑名单 |
| `AccountUnblocked` | 账户已从黑名单移除 |
| `FriendRequestSent` | 好友申请已发送 |
| `FriendRequestAccepted` | 好友申请已接受 |
| `FriendRequestRejected` | 好友申请已拒绝 |
| `FriendStatusChanged` | 好友关系状态变更 |

## 错误类型

| 错误 | 说明 |
|------|------|
| `ContactAlreadyExists` | 联系人已存在 |
| `ContactNotFound` | 联系人不存在 |
| `TooManyContacts` | 联系人数量已达上限 |
| `CannotAddSelf` | 不能添加自己为联系人 |
| `BlockedByOther` | 已被对方加入黑名单 |
| `GroupAlreadyExists` | 分组已存在 |
| `GroupNotFound` | 分组不存在 |
| `TooManyGroups` | 分组数量已达上限 |
| `GroupMembersFull` | 分组成员数量已达上限 |
| `EmptyGroupName` | 分组名称为空 |
| `AlreadyBlocked` | 账户已在黑名单中 |
| `NotBlocked` | 账户不在黑名单中 |
| `TooManyBlocked` | 黑名单数量已达上限 |
| `FriendRequestAlreadyExists` | 好友申请已存在 |
| `FriendRequestNotFound` | 好友申请不存在 |
| `FriendRequestExpired` | 好友申请已过期 |

## 与其他模块的集成

### 与 pallet-smart-group-chat 集成

可以通过 `are_mutual_friends` 函数检查用户是否为好友，用于群聊权限控制：

```rust
// 在 smart-group-chat 中检查好友关系
if group.privacy == GroupPrivacy::FriendsOnly {
    ensure!(
        pallet_contacts::Pallet::<T>::are_mutual_friends(&sender, &group_owner),
        Error::<T>::NotAFriend
    );
}
```

## 测试

运行单元测试：

```bash
cargo test -p pallet-chat-contacts
```

## License

Unlicense
