# Pallet Chat

## 模块概述

去中心化聊天功能模块，采用混合存储架构（链上元数据 + IPFS内容存储），为Stardust纪念平台提供安全、隐私、可扩展的即时通讯服务。该模块实现了完整的私聊功能，包括会话管理、消息已读/未读状态追踪、软删除机制、黑名单系统、频率限制防护等核心功能。

### 版本历史

- **v0.1.0 (2024)**: 初始版本，基础私聊功能
- **v1.0.0 (P1)**: 生产级优化
  - 移除BoundedVec限制，支持无限消息和会话
  - 添加黑名单功能
  - 添加频率限制防护
  - 添加分别软删除机制
  - 添加CID加密验证
- **v1.1.0 (P2)**: 运维功能增强
  - 添加旧消息清理功能
  - 优化查询性能
  - 完善错误处理

### 设计理念

1. **链上元数据，链下内容**：链上只存储消息元数据（发送方、接收方、时间戳、CID等），消息内容加密后存储在IPFS，平衡存储成本和数据可用性
2. **端到端加密**：消息内容在前端加密后上传IPFS，只有发送方和接收方能解密，保证通讯隐私
3. **软删除机制**：发送方和接收方可独立删除消息，互不影响，提升用户体验
4. **防垃圾攻击**：通过频率限制和黑名单机制防止恶意用户发送垃圾消息
5. **无限扩展**：使用DoubleMap替代BoundedVec，支持无限数量的会话和消息

### 与其他模块的关系

- **pallet-stardust-ipfs**: 依赖IPFS模块存储加密的消息内容
- **pallet-otc-order**: OTC订单系统中的买卖双方沟通渠道
- **前端DApp**: 通过Polkadot-JS API调用，实现实时通讯功能

## 核心功能

### 1. 私聊功能

#### 1.1 消息发送

支持用户之间一对一的私聊通讯，消息内容通过IPFS存储，链上只记录元数据。

```rust
pub fn send_message(
    origin: OriginFor<T>,
    receiver: T::AccountId,      // 接收方地址
    content_cid: Vec<u8>,         // IPFS CID（加密的消息内容）
    msg_type_code: u8,            // 消息类型代码
    session_id: Option<T::Hash>, // 会话ID（可选）
) -> DispatchResult
```

**流程图**：

```text
┌───────────────────────────────────────────────────────────────────┐
│ 用户A (发送方)                                                      │
└────────────┬──────────────────────────────────────────────────────┘
             │
             ▼
    ┌─────────────────┐
    │ 1. 加密消息内容  │  (前端实现)
    └────────┬────────┘
             │
             ▼
    ┌─────────────────┐
    │ 2. 上传到IPFS    │  (获取CID)
    └────────┬────────┘
             │
             ▼
    ┌──────────────────────────────────────────┐
    │ 3. 调用 send_message                      │
    │    - 检查接收方是否拉黑发送方              │
    │    - 检查频率限制                          │
    │    - 验证CID长度和加密状态                 │
    │    - 获取或创建会话                        │
    │    - 生成消息ID并存储元数据                │
    │    - 更新会话最后活跃时间                  │
    │    - 增加接收方未读计数                    │
    │    - 触发MessageSent事件                  │
    └────────┬─────────────────────────────────┘
             │
             ▼
    ┌─────────────────┐
    │ 4. 链上存储元数据 │
    └────────┬────────┘
             │
             ▼
┌────────────┴──────────────────────────────────────────────────────┐
│ 用户B (接收方)                                                      │
│   - 监听MessageSent事件                                            │
│   - 获取CID并从IPFS下载                                             │
│   - 解密消息内容                                                    │
│   - 显示消息                                                        │
└───────────────────────────────────────────────────────────────────┘
```

**关键设计点**：

1. **CID加密验证**：链上验证CID必须是加密后的（长度>50字节或不以"Qm"开头），防止明文内容泄露
2. **频率限制**：每个用户在时间窗口（100个区块 ≈ 10分钟）内最多发送10条消息，防止垃圾消息
3. **黑名单检查**：发送前检查接收方是否拉黑了发送方，提升用户体验
4. **自动会话创建**：如果未提供session_id，系统会自动创建新会话，简化用户操作

#### 1.2 消息类型

支持多种消息类型，满足不同场景需求：

```rust
pub enum MessageType {
    Text,    // 0: 文本消息
    Image,   // 1: 图片消息
    File,    // 2: 文件消息
    Voice,   // 3: 语音消息
    System,  // 4: 系统消息（如订单状态变更）
}
```

**应用场景**：

- **Text**: 普通文本聊天
- **Image**: 图片分享
- **File**: 文件传输
- **Voice**: 语音留言
- **System**: OTC订单状态变更通知、系统公告等

### 2. 会话管理

#### 2.1 会话创建

会话（Session）是两个用户之间所有消息的集合，会话ID基于两个用户账户地址的哈希值确定，保证一对用户只有一个会话。

```rust
pub fn create_session(
    user1: &T::AccountId,
    user2: &T::AccountId,
) -> Result<T::Hash, DispatchError>
```

**会话ID生成逻辑**：

```rust
// 1. 对两个用户地址排序（保证一致性）
let mut participants = vec![user1.clone(), user2.clone()];
participants.sort();

// 2. 基于排序后的地址生成哈希
let session_id = T::Hashing::hash_of(&participants);
```

**特点**：

- **确定性**：无论A→B还是B→A，生成的session_id相同
- **唯一性**：每对用户只有一个会话
- **自动创建**：首次发送消息时自动创建

#### 2.2 会话查询

```rust
// 查询用户的所有会话（按最后活跃时间倒序）
pub fn list_sessions(user: T::AccountId) -> Vec<T::Hash>

// 查询会话详情
pub fn get_session(session_id: T::Hash) -> Option<Session<T>>

// 查询会话中的消息列表（分页）
pub fn list_messages_by_session(
    session_id: T::Hash,
    offset: u32,
    limit: u32,
) -> Vec<u64>
```

**分页机制**：

- 默认按消息ID倒序返回（最新的在前）
- 每页最多100条消息
- 支持offset和limit参数，适配移动端无限滚动加载

#### 2.3 会话归档

用户可以归档不常用的会话，清理会话列表：

```rust
pub fn archive_session(
    origin: OriginFor<T>,
    session_id: T::Hash,
) -> DispatchResult
```

**注意**：归档不会删除消息，只是标记会话为归档状态，前端可选择性隐藏。

### 3. 已读/未读状态管理

#### 3.1 单条消息标记已读

```rust
pub fn mark_as_read(
    origin: OriginFor<T>,
    msg_id: u64,
) -> DispatchResult
```

**流程**：

1. 验证调用者是接收方
2. 检查消息是否已读（避免重复标记）
3. 标记消息为已读
4. 减少未读计数
5. 触发MessageRead事件

#### 3.2 批量标记已读

```rust
// 批量标记指定消息列表为已读
pub fn mark_batch_as_read(
    origin: OriginFor<T>,
    message_ids: Vec<u64>,
) -> DispatchResult

// 批量标记整个会话为已读
pub fn mark_session_as_read(
    origin: OriginFor<T>,
    session_id: T::Hash,
) -> DispatchResult
```

**性能优化**：

- `mark_batch_as_read`: 适用于已知消息ID列表的场景
- `mark_session_as_read`: 适用于"标记全部已读"的场景，更高效

#### 3.3 未读计数查询

```rust
pub fn get_unread_count(
    user: T::AccountId,
    session_id: Option<T::Hash>,
) -> u32
```

**两种查询模式**：

1. **指定会话**：返回该会话的未读数
2. **全部会话**：返回用户所有会话的未读总数（适用于应用图标角标）

### 4. 软删除机制

#### 4.1 分别删除

发送方和接收方可以独立删除消息，互不影响：

```rust
pub fn delete_message(
    origin: OriginFor<T>,
    msg_id: u64,
) -> DispatchResult
```

**删除标记**：

- `is_deleted_by_sender`: 发送方是否已删除
- `is_deleted_by_receiver`: 接收方是否已删除

**示例场景**：

```text
Alice -> Bob: "Hello"

1. Alice删除消息后：
   - Alice看不到这条消息
   - Bob仍然可以看到

2. Bob也删除消息后：
   - Alice和Bob都看不到这条消息
   - 链上记录仍存在（双方都删除且过期后可被清理）
```

#### 4.2 消息清理

支持清理过期且双方都删除的消息，释放链上存储空间：

```rust
pub fn cleanup_old_messages(
    origin: OriginFor<T>,
    limit: u32,  // 每次清理的最大消息数（1-1000）
) -> DispatchResult
```

**清理条件**：

1. 消息发送时间超过`MessageExpirationTime`（如180天）
2. 发送方和接收方都标记为删除

**安全措施**：

- 一次最多清理1000条，防止区块过载
- 只清理满足条件的消息，保护用户数据
- 建议由治理或定期任务调用

### 5. 黑名单系统

#### 5.1 拉黑用户

```rust
pub fn block_user(
    origin: OriginFor<T>,
    blocked_user: T::AccountId,
) -> DispatchResult
```

**功能**：

- 拉黑后，被拉黑的用户无法向您发送消息
- 支持查询黑名单列表
- 拉黑是单向的（A拉黑B不影响B拉黑A）

**限制**：

- 不能拉黑自己

#### 5.2 解除拉黑

```rust
pub fn unblock_user(
    origin: OriginFor<T>,
    unblocked_user: T::AccountId,
) -> DispatchResult
```

#### 5.3 黑名单查询

```rust
// 检查是否被拉黑
pub fn is_blocked(
    blocker: T::AccountId,
    potential_blocked: T::AccountId,
) -> bool

// 查询用户的黑名单列表
pub fn list_blocked_users(user: T::AccountId) -> Vec<T::AccountId>
```

### 6. 频率限制

防止用户短时间内发送大量消息，防护垃圾消息和DoS攻击：

```rust
fn check_rate_limit(sender: &T::AccountId) -> DispatchResult
```

**限制规则**：

- **时间窗口**：`RateLimitWindow` 个区块（如100个区块 ≈ 10分钟）
- **最大消息数**：`MaxMessagesPerWindow` 条消息（如10条）
- **超出限制**：返回`RateLimitExceeded`错误

**窗口重置**：

- 当前区块与上次发送区块的差值超过窗口期时，自动重置计数

### 7. CID加密验证

确保消息内容已加密，保护用户隐私：

```rust
pub fn is_cid_encrypted(cid: &[u8]) -> bool
```

**验证逻辑**：

1. **标准CIDv0检测**：如果CID长度=46字节且以"Qm"开头，判定为未加密
2. **长度检测**：加密后的CID通常>50字节
3. **拒绝未加密CID**：`send_message`调用时会验证CID加密状态

**安全设计**：

- 根据项目规则，聊天消息必须加密（除证据类数据外）
- 前端负责加密，链端负责验证
- 避免明文内容泄露到IPFS网络

## 数据结构

### 核心结构

#### MessageMeta - 消息元数据

```rust
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct MessageMeta<T: Config> {
    /// 发送方账户
    pub sender: T::AccountId,
    /// 接收方账户
    pub receiver: T::AccountId,
    /// IPFS CID（加密的消息内容）
    pub content_cid: BoundedVec<u8, T::MaxCidLen>,
    /// 会话ID（用于分组消息）
    pub session_id: T::Hash,
    /// 消息类型
    pub msg_type: MessageType,
    /// 发送时间（区块高度）
    pub sent_at: BlockNumberFor<T>,
    /// 是否已读
    pub is_read: bool,
    /// 发送方是否已删除（软删除）
    pub is_deleted_by_sender: bool,
    /// 接收方是否已删除（软删除）
    pub is_deleted_by_receiver: bool,
}
```

**字段说明**：

- `sender`: 发送方账户地址
- `receiver`: 接收方账户地址
- `content_cid`: 加密的消息内容的IPFS CID（最长100字节）
- `session_id`: 会话唯一标识符
- `msg_type`: 消息类型（Text/Image/File/Voice/System）
- `sent_at`: 消息发送时的区块高度
- `is_read`: 接收方是否已读
- `is_deleted_by_sender`: 发送方是否删除（软删除）
- `is_deleted_by_receiver`: 接收方是否删除（软删除）

#### Session - 会话信息

```rust
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Session<T: Config> {
    /// 会话ID
    pub id: T::Hash,
    /// 参与者列表（最多2人，私聊）
    pub participants: BoundedVec<T::AccountId, ConstU32<2>>,
    /// 最后一条消息ID
    pub last_message_id: u64,
    /// 最后活跃时间
    pub last_active: BlockNumberFor<T>,
    /// 创建时间
    pub created_at: BlockNumberFor<T>,
    /// 是否归档
    pub is_archived: bool,
}
```

**字段说明**：

- `id`: 会话ID（基于参与者地址的哈希）
- `participants`: 参与者列表（私聊固定为2人）
- `last_message_id`: 最后一条消息的ID（用于快速定位）
- `last_active`: 最后活跃时间（用于排序会话列表）
- `created_at`: 会话创建时间
- `is_archived`: 是否已归档

#### MessageType - 消息类型枚举

```rust
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub enum MessageType {
    /// 文本消息
    Text,
    /// 图片消息
    Image,
    /// 文件消息
    File,
    /// 语音消息
    Voice,
    /// 系统消息（如订单状态变更）
    System,
}
```

### 存储项

#### 核心存储

```rust
/// 消息元数据存储
/// Key: 消息ID
/// Value: 消息元数据
pub type Messages<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64,                // 消息ID
    MessageMeta<T>,     // 消息元数据
>;

/// 下一个消息ID
pub type NextMessageId<T: Config> = StorageValue<_, u64, ValueQuery>;

/// 会话存储
/// Key: 会话ID
/// Value: 会话信息
pub type Sessions<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::Hash,            // 会话ID
    Session<T>,         // 会话信息
>;
```

#### 索引存储

```rust
/// 用户会话索引
/// Key1: 账户地址
/// Key2: 会话ID
/// Value: () 标记（只用于索引）
/// 支持无限会话数量
pub type UserSessions<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AccountId,       // 用户账户
    Blake2_128Concat,
    T::Hash,            // 会话ID
    (),
    OptionQuery,
>;

/// 会话消息索引
/// Key1: 会话ID
/// Key2: 消息ID
/// Value: () 标记（只用于索引）
/// 支持无限消息数量
pub type SessionMessages<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::Hash,            // 会话ID
    Blake2_128Concat,
    u64,                // 消息ID
    (),
    OptionQuery,
>;
```

#### 未读计数

```rust
/// 未读消息计数
/// Key: (接收方, 会话ID)
/// Value: 未读数量
pub type UnreadCount<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    (T::AccountId, T::Hash),  // (接收方, 会话ID)
    u32,                       // 未读数量
    ValueQuery,
>;
```

#### 黑名单

```rust
/// 黑名单
/// Key1: 用户
/// Key2: 被拉黑的用户
/// Value: () 标记
pub type Blacklist<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AccountId,       // 用户
    Blake2_128Concat,
    T::AccountId,       // 被拉黑的用户
    (),
    OptionQuery,
>;
```

#### 频率限制

```rust
/// 消息发送频率限制
/// Key: 用户账户
/// Value: (最后发送时间, 时间窗口内发送次数)
pub type MessageRateLimit<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    (BlockNumberFor<T>, u32),  // (最后发送时间, 发送次数)
    ValueQuery,
>;
```

## 主要调用方法

### 消息发送类

#### `send_message` - 发送消息

发送一条消息给指定用户。

```rust
#[pallet::call_index(0)]
pub fn send_message(
    origin: OriginFor<T>,
    receiver: T::AccountId,         // 接收方地址
    content_cid: Vec<u8>,            // IPFS CID（加密的消息内容）
    msg_type_code: u8,               // 消息类型代码 (0=Text, 1=Image, 2=File, 3=Voice, 4=System)
    session_id: Option<T::Hash>,    // 会话ID（可选，如果为None则自动创建新会话）
) -> DispatchResult
```

**参数说明**：

- `receiver`: 接收方账户地址
- `content_cid`: 加密的消息内容的IPFS CID（长度≤100字节）
- `msg_type_code`: 消息类型代码（0-4）
- `session_id`: 可选，指定会话ID；如果为None，系统会自动创建新会话

**返回**：

- `Ok(())`: 消息发送成功
- `Err(ReceiverBlockedSender)`: 接收方已拉黑发送方
- `Err(RateLimitExceeded)`: 超过频率限制
- `Err(CidTooLong)`: CID长度超过限制
- `Err(CidNotEncrypted)`: CID未加密

**事件**：

- `MessageSent`: 消息已发送
- `SessionCreated`: 新会话已创建（如果是首次对话）

### 已读/未读管理类

#### `mark_as_read` - 标记消息已读

标记单条消息为已读。

```rust
#[pallet::call_index(1)]
pub fn mark_as_read(
    origin: OriginFor<T>,
    msg_id: u64,                     // 消息ID
) -> DispatchResult
```

**参数说明**：

- `msg_id`: 要标记的消息ID

**返回**：

- `Ok(())`: 标记成功
- `Err(MessageNotFound)`: 消息不存在
- `Err(NotReceiver)`: 调用者不是接收方

**事件**：

- `MessageRead`: 消息已读

#### `mark_batch_as_read` - 批量标记已读（指定消息列表）

批量标记多条消息为已读。

```rust
#[pallet::call_index(3)]
pub fn mark_batch_as_read(
    origin: OriginFor<T>,
    message_ids: Vec<u64>,           // 消息ID列表
) -> DispatchResult
```

**参数说明**：

- `message_ids`: 要标记的消息ID列表

**返回**：

- `Ok(())`: 批量标记成功
- `Err(EmptyMessageList)`: 消息列表为空

**事件**：

- `MessageRead`: 每条消息触发一次事件

#### `mark_session_as_read` - 批量标记已读（按会话）

标记整个会话的所有未读消息为已读。

```rust
#[pallet::call_index(4)]
pub fn mark_session_as_read(
    origin: OriginFor<T>,
    session_id: T::Hash,             // 会话ID
) -> DispatchResult
```

**参数说明**：

- `session_id`: 会话ID

**返回**：

- `Ok(())`: 会话标记成功
- `Err(SessionNotFound)`: 会话不存在
- `Err(NotSessionParticipant)`: 调用者不是会话参与者

**事件**：

- `SessionMarkedAsRead`: 会话已标记为已读

### 删除管理类

#### `delete_message` - 删除消息（软删除）

删除消息（仅对调用者隐藏，不影响对方）。

```rust
#[pallet::call_index(2)]
pub fn delete_message(
    origin: OriginFor<T>,
    msg_id: u64,                     // 消息ID
) -> DispatchResult
```

**参数说明**：

- `msg_id`: 要删除的消息ID

**返回**：

- `Ok(())`: 删除成功
- `Err(MessageNotFound)`: 消息不存在
- `Err(NotAuthorized)`: 调用者既不是发送方也不是接收方

**事件**：

- `MessageDeleted`: 消息已删除

**说明**：

- 发送方删除：只对发送方隐藏，接收方仍可见
- 接收方删除：只对接收方隐藏，发送方仍可见
- 双方都删除且过期后：可通过`cleanup_old_messages`清理

### 会话管理类

#### `archive_session` - 归档会话

归档会话（前端可选择性隐藏）。

```rust
#[pallet::call_index(5)]
pub fn archive_session(
    origin: OriginFor<T>,
    session_id: T::Hash,             // 会话ID
) -> DispatchResult
```

**参数说明**：

- `session_id`: 要归档的会话ID

**返回**：

- `Ok(())`: 归档成功
- `Err(SessionNotFound)`: 会话不存在
- `Err(NotSessionParticipant)`: 调用者不是会话参与者

**事件**：

- `SessionArchived`: 会话已归档

### 黑名单管理类

#### `block_user` - 拉黑用户

拉黑指定用户，拉黑后对方无法向您发送消息。

```rust
#[pallet::call_index(6)]
pub fn block_user(
    origin: OriginFor<T>,
    blocked_user: T::AccountId,      // 要拉黑的用户
) -> DispatchResult
```

**参数说明**：

- `blocked_user`: 要拉黑的用户账户地址

**返回**：

- `Ok(())`: 拉黑成功
- `Err(CannotBlockSelf)`: 不能拉黑自己

**事件**：

- `UserBlocked`: 用户已被拉黑

#### `unblock_user` - 解除拉黑

解除对指定用户的拉黑。

```rust
#[pallet::call_index(7)]
pub fn unblock_user(
    origin: OriginFor<T>,
    unblocked_user: T::AccountId,    // 要解除拉黑的用户
) -> DispatchResult
```

**参数说明**：

- `unblocked_user`: 要解除拉黑的用户账户地址

**返回**：

- `Ok(())`: 解除成功

**事件**：

- `UserUnblocked`: 用户已解除拉黑

### 运维管理类

#### `cleanup_old_messages` - 清理过期消息

清理过期且双方都删除的消息，释放链上存储空间。

```rust
#[pallet::call_index(8)]
pub fn cleanup_old_messages(
    origin: OriginFor<T>,
    limit: u32,                      // 每次清理的最大消息数（1-1000）
) -> DispatchResult
```

**参数说明**：

- `limit`: 每次清理的最大消息数，范围：1-1000

**返回**：

- `Ok(())`: 清理成功
- `Err(InvalidCleanupLimit)`: limit参数无效（必须在1-1000之间）

**事件**：

- `OldMessagesCleanedUp`: 旧消息已清理

**清理条件**：

1. 消息发送时间超过`MessageExpirationTime`（如180天）
2. 发送方和接收方都标记为删除

**建议**：

- 由治理或定期任务调用
- 每次清理数量不超过1000条，防止区块过载

### 查询方法（公共函数）

#### `get_message` - 查询单条消息

```rust
pub fn get_message(message_id: u64) -> Option<MessageMeta<T>>
```

**参数**：

- `message_id`: 消息ID

**返回**：

- `Some(MessageMeta)`: 消息元数据
- `None`: 消息不存在

#### `list_messages_by_session` - 分页查询会话消息

```rust
pub fn list_messages_by_session(
    session_id: T::Hash,
    offset: u32,
    limit: u32,
) -> Vec<u64>
```

**参数**：

- `session_id`: 会话ID
- `offset`: 偏移量（从0开始）
- `limit`: 每页数量（最多100条）

**返回**：

- `Vec<u64>`: 消息ID列表（按时间倒序，最新的在前）

**说明**：

- 返回消息ID列表，前端需再次查询消息详情
- 自动限制每页最多100条
- 适配移动端无限滚动加载

#### `get_session` - 查询会话信息

```rust
pub fn get_session(session_id: T::Hash) -> Option<Session<T>>
```

**参数**：

- `session_id`: 会话ID

**返回**：

- `Some(Session)`: 会话信息
- `None`: 会话不存在

#### `list_sessions` - 查询用户的所有会话

```rust
pub fn list_sessions(user: T::AccountId) -> Vec<T::Hash>
```

**参数**：

- `user`: 用户账户地址

**返回**：

- `Vec<T::Hash>`: 会话ID列表（按最后活跃时间倒序）

#### `get_unread_count` - 查询未读消息数

```rust
pub fn get_unread_count(
    user: T::AccountId,
    session_id: Option<T::Hash>,
) -> u32
```

**参数**：

- `user`: 用户账户地址
- `session_id`: 可选，指定会话ID

**返回**：

- `u32`: 未读消息数

**两种查询模式**：

1. **指定会话**（`session_id = Some(...)`）：返回该会话的未读数
2. **全部会话**（`session_id = None`）：返回用户所有会话的未读总数

#### `is_blocked` - 检查是否被拉黑

```rust
pub fn is_blocked(
    blocker: T::AccountId,
    potential_blocked: T::AccountId,
) -> bool
```

**参数**：

- `blocker`: 可能拉黑的用户
- `potential_blocked`: 可能被拉黑的用户

**返回**：

- `true`: 已被拉黑
- `false`: 未被拉黑

#### `list_blocked_users` - 查询黑名单列表

```rust
pub fn list_blocked_users(user: T::AccountId) -> Vec<T::AccountId>
```

**参数**：

- `user`: 用户账户地址

**返回**：

- `Vec<T::AccountId>`: 被该用户拉黑的账户列表

#### `is_cid_encrypted` - 检查CID是否加密

```rust
pub fn is_cid_encrypted(cid: &[u8]) -> bool
```

**参数**：

- `cid`: IPFS CID字节数组

**返回**：

- `true`: CID已加密
- `false`: CID未加密

**验证逻辑**：

- 标准CIDv0（46字节，以"Qm"开头）判定为未加密
- 其他情况（长度>50字节或不符合CIDv0格式）判定为已加密

## 事件定义

```rust
pub enum Event<T: Config> {
    /// 消息已发送
    /// [msg_id, session_id, sender, receiver]
    MessageSent {
        msg_id: u64,
        session_id: T::Hash,
        sender: T::AccountId,
        receiver: T::AccountId,
    },

    /// 消息已读
    /// [msg_id, reader]
    MessageRead {
        msg_id: u64,
        reader: T::AccountId,
    },

    /// 消息已删除
    /// [msg_id, deleter]
    MessageDeleted {
        msg_id: u64,
        deleter: T::AccountId,
    },

    /// 会话已创建
    /// [session_id, participants]
    SessionCreated {
        session_id: T::Hash,
        participants: BoundedVec<T::AccountId, ConstU32<2>>,
    },

    /// 会话已标记为已读
    /// [session_id, user]
    SessionMarkedAsRead {
        session_id: T::Hash,
        user: T::AccountId,
    },

    /// 会话已归档
    /// [session_id, operator]
    SessionArchived {
        session_id: T::Hash,
        operator: T::AccountId,
    },

    /// 用户已被拉黑
    /// [blocker, blocked]
    UserBlocked {
        blocker: T::AccountId,
        blocked: T::AccountId,
    },

    /// 用户已被解除拉黑
    /// [unblocker, unblocked]
    UserUnblocked {
        unblocker: T::AccountId,
        unblocked: T::AccountId,
    },

    /// 旧消息已清理
    /// [operator, count]
    OldMessagesCleanedUp {
        operator: T::AccountId,
        count: u32,
    },
}
```

## 错误定义

```rust
pub enum Error<T> {
    /// CID 太长，超过了最大长度限制
    CidTooLong,
    /// 消息未找到，请检查消息ID是否正确
    MessageNotFound,
    /// 会话未找到，请检查会话ID是否正确
    SessionNotFound,
    /// 不是接收方，只有消息接收方才能执行此操作
    NotReceiver,
    /// 未授权，您没有权限执行此操作
    NotAuthorized,
    /// 不是会话参与者，只有会话参与者才能执行此操作
    NotSessionParticipant,
    /// 会话消息太多，已达到单个会话的消息数量上限（已废弃）
    TooManyMessages,
    /// 用户会话太多，已达到单个用户的会话数量上限（已废弃）
    TooManySessions,
    /// 参与者太多，会话只支持2个参与者
    TooManyParticipants,
    /// CID未加密，根据系统规则，聊天消息必须加密后上传到IPFS
    CidNotEncrypted,
    /// 消息ID列表为空
    EmptyMessageList,
    /// 分页参数无效，offset或limit超出合理范围
    InvalidPagination,
    /// 接收方已将您拉黑，无法发送消息
    ReceiverBlockedSender,
    /// 发送消息过于频繁，请稍后再试
    RateLimitExceeded,
    /// 不能拉黑自己
    CannotBlockSelf,
    /// 清理数量参数无效（必须大于0且小于等于1000）
    InvalidCleanupLimit,
}
```

## 配置参数

```rust
pub trait Config: frame_system::Config {
    /// 事件类型
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    /// 权重信息
    type WeightInfo: WeightInfo;

    /// IPFS CID最大长度（通常为46-59字节）
    #[pallet::constant]
    type MaxCidLen: Get<u32>;

    /// 每个用户最多会话数（已废弃，但保留以兼容）
    #[pallet::constant]
    type MaxSessionsPerUser: Get<u32>;

    /// 每个会话最多消息数（已废弃，但保留以兼容）
    #[pallet::constant]
    type MaxMessagesPerSession: Get<u32>;

    /// 频率限制：时间窗口（区块数）
    /// 例如：100个区块 ≈ 10分钟（假设6秒一个块）
    #[pallet::constant]
    type RateLimitWindow: Get<BlockNumberFor<Self>>;

    /// 频率限制：时间窗口内最大消息数
    /// 例如：10条消息/10分钟
    #[pallet::constant]
    type MaxMessagesPerWindow: Get<u32>;

    /// 消息过期时间（区块数）
    /// 例如：2_592_000个区块 ≈ 180天（假设6秒一个块）
    /// 过期后可被清理
    #[pallet::constant]
    type MessageExpirationTime: Get<BlockNumberFor<Self>>;
}
```

**配置建议**：

- `MaxCidLen`: 100字节（足够容纳加密后的CID）
- `RateLimitWindow`: 100个区块（约10分钟）
- `MaxMessagesPerWindow`: 10条消息
- `MessageExpirationTime`: 2_592_000个区块（约180天）

## 使用示例

### TypeScript前端示例

#### 示例1：发送消息（完整流程）

```typescript
import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import CryptoJS from 'crypto-js';
import { create as ipfsClient } from 'ipfs-http-client';

// 初始化连接
const provider = new WsProvider('ws://localhost:9944');
const api = await ApiPromise.create({ provider });
const keyring = new Keyring({ type: 'sr25519' });

// 创建账户
const alice = keyring.addFromUri('//Alice');
const bob = keyring.addFromUri('//Bob');

// IPFS客户端
const ipfs = ipfsClient({ url: 'http://localhost:5001' });

// 1. 加密消息内容
const encryptMessage = (message: string, sharedKey: string): string => {
  return CryptoJS.AES.encrypt(message, sharedKey).toString();
};

// 2. 上传到IPFS
const uploadToIpfs = async (encryptedContent: string): Promise<string> => {
  const { cid } = await ipfs.add(encryptedContent);
  return cid.toString();
};

// 3. 发送消息
const sendMessage = async (
  sender: any,
  receiver: string,
  message: string,
  msgType: number = 0
) => {
  // 生成共享密钥（实际应用中应使用ECDH等协议）
  const sharedKey = 'shared_secret_key';

  // 加密消息
  const encrypted = encryptMessage(message, sharedKey);

  // 上传到IPFS
  const cid = await uploadToIpfs(encrypted);

  // 发送交易
  const tx = api.tx.chat.sendMessage(
    receiver,
    cid,
    msgType,
    null // session_id自动创建
  );

  return new Promise((resolve, reject) => {
    tx.signAndSend(sender, ({ status, events }) => {
      if (status.isInBlock) {
        console.log(`交易已打包: ${status.asInBlock}`);

        // 查找MessageSent事件
        events.forEach(({ event }) => {
          if (api.events.chat.MessageSent.is(event)) {
            const [msgId, sessionId, senderAddr, receiverAddr] = event.data;
            console.log(`消息已发送: ID=${msgId}, Session=${sessionId}`);
            resolve({ msgId, sessionId });
          }
        });
      } else if (status.isFinalized) {
        console.log(`交易已确认: ${status.asFinalized}`);
      }
    }).catch(reject);
  });
};

// 使用示例
try {
  const result = await sendMessage(
    alice,
    bob.address,
    'Hello Bob, this is Alice!'
  );
  console.log('发送成功:', result);
} catch (error) {
  console.error('发送失败:', error);
}
```

#### 示例2：接收和解密消息

```typescript
// 解密消息
const decryptMessage = (encryptedContent: string, sharedKey: string): string => {
  const bytes = CryptoJS.AES.decrypt(encryptedContent, sharedKey);
  return bytes.toString(CryptoJS.enc.Utf8);
};

// 从IPFS下载内容
const downloadFromIpfs = async (cid: string): Promise<string> => {
  const chunks = [];
  for await (const chunk of ipfs.cat(cid)) {
    chunks.push(chunk);
  }
  return Buffer.concat(chunks).toString();
};

// 监听消息事件
const listenMessages = async (userAddress: string) => {
  // 订阅MessageSent事件
  api.query.system.events((events) => {
    events.forEach((record) => {
      const { event } = record;

      if (api.events.chat.MessageSent.is(event)) {
        const [msgId, sessionId, sender, receiver] = event.data;

        // 检查是否是发给我的消息
        if (receiver.toString() === userAddress) {
          console.log(`收到新消息: ID=${msgId}`);

          // 获取消息详情
          handleNewMessage(msgId.toNumber());
        }
      }
    });
  });
};

// 处理新消息
const handleNewMessage = async (msgId: number) => {
  // 查询消息元数据
  const msg = await api.query.chat.messages(msgId);

  if (msg.isSome) {
    const msgData = msg.unwrap();
    const cid = msgData.contentCid.toUtf8();

    // 从IPFS下载加密内容
    const encryptedContent = await downloadFromIpfs(cid);

    // 解密消息
    const sharedKey = 'shared_secret_key';
    const decryptedMessage = decryptMessage(encryptedContent, sharedKey);

    console.log('消息内容:', decryptedMessage);

    // 标记已读
    await markAsRead(msgId);
  }
};

// 标记消息已读
const markAsRead = async (msgId: number) => {
  const tx = api.tx.chat.markAsRead(msgId);
  await tx.signAndSend(bob, ({ status }) => {
    if (status.isInBlock) {
      console.log(`消息${msgId}已标记为已读`);
    }
  });
};

// 使用示例
await listenMessages(bob.address);
```

#### 示例3：查询会话列表

```typescript
// 查询用户的所有会话
const listSessions = async (userAddress: string) => {
  const sessions: any[] = [];

  // 遍历UserSessions存储
  const entries = await api.query.chat.userSessions.entries(userAddress);

  for (const [key, value] of entries) {
    const sessionId = key.args[1]; // 第二个参数是session_id

    // 查询会话详情
    const session = await api.query.chat.sessions(sessionId);

    if (session.isSome) {
      const sessionData = session.unwrap();
      sessions.push({
        sessionId: sessionId.toHex(),
        participants: sessionData.participants.map((p: any) => p.toString()),
        lastMessageId: sessionData.lastMessageId.toNumber(),
        lastActive: sessionData.lastActive.toNumber(),
        isArchived: sessionData.isArchived.valueOf(),
      });
    }
  }

  // 按最后活跃时间排序
  sessions.sort((a, b) => b.lastActive - a.lastActive);

  return sessions;
};

// 使用示例
const sessions = await listSessions(alice.address);
console.log('会话列表:', sessions);
```

#### 示例4：查询未读消息数

```typescript
// 查询总未读数
const getTotalUnreadCount = async (userAddress: string): Promise<number> => {
  let totalUnread = 0;

  // 获取所有会话
  const sessions = await listSessions(userAddress);

  // 累加每个会话的未读数
  for (const session of sessions) {
    const unread = await api.query.chat.unreadCount([
      userAddress,
      session.sessionId
    ]);
    totalUnread += unread.toNumber();
  }

  return totalUnread;
};

// 查询单个会话的未读数
const getSessionUnreadCount = async (
  userAddress: string,
  sessionId: string
): Promise<number> => {
  const unread = await api.query.chat.unreadCount([userAddress, sessionId]);
  return unread.toNumber();
};

// 使用示例
const totalUnread = await getTotalUnreadCount(alice.address);
console.log('总未读消息数:', totalUnread);

const sessionUnread = await getSessionUnreadCount(alice.address, sessionId);
console.log('会话未读消息数:', sessionUnread);
```

#### 示例5：黑名单管理

```typescript
// 拉黑用户
const blockUser = async (blocker: any, blockedAddress: string) => {
  const tx = api.tx.chat.blockUser(blockedAddress);

  return new Promise((resolve, reject) => {
    tx.signAndSend(blocker, ({ status, events }) => {
      if (status.isInBlock) {
        console.log(`已拉黑用户: ${blockedAddress}`);

        events.forEach(({ event }) => {
          if (api.events.chat.UserBlocked.is(event)) {
            const [blockerAddr, blockedAddr] = event.data;
            resolve({ blocker: blockerAddr, blocked: blockedAddr });
          }
        });
      }
    }).catch(reject);
  });
};

// 解除拉黑
const unblockUser = async (unblocker: any, unblockedAddress: string) => {
  const tx = api.tx.chat.unblockUser(unblockedAddress);
  await tx.signAndSend(unblocker);
  console.log(`已解除拉黑: ${unblockedAddress}`);
};

// 查询是否被拉黑
const isBlocked = async (
  blockerAddress: string,
  potentialBlockedAddress: string
): Promise<boolean> => {
  const result = await api.query.chat.blacklist(
    blockerAddress,
    potentialBlockedAddress
  );
  return result.isSome;
};

// 查询黑名单列表
const listBlockedUsers = async (userAddress: string): Promise<string[]> => {
  const blockedList: string[] = [];

  const entries = await api.query.chat.blacklist.entries(userAddress);

  for (const [key, value] of entries) {
    const blockedUser = key.args[1]; // 第二个参数是被拉黑的用户
    blockedList.push(blockedUser.toString());
  }

  return blockedList;
};

// 使用示例
await blockUser(bob, alice.address);
const blocked = await isBlocked(bob.address, alice.address);
console.log('Alice是否被Bob拉黑:', blocked);

const blacklist = await listBlockedUsers(bob.address);
console.log('Bob的黑名单:', blacklist);
```

## 集成说明

### 与其他模块的集成

#### 1. 与 pallet-stardust-ipfs 集成

Chat模块依赖IPFS模块存储加密的消息内容：

```rust
// 在runtime/src/lib.rs中配置
impl pallet_chat::Config for Runtime {
    // ... 其他配置
}

impl pallet_stardust_ipfs::Config for Runtime {
    // ... IPFS配置
}
```

**集成流程**：

1. 前端加密消息内容
2. 上传到IPFS节点，获取CID
3. 调用`chat::send_message`，传入CID
4. 链上存储元数据
5. 接收方监听事件，下载IPFS内容并解密

**注意事项**：

- CID必须是加密后的（链上会验证）
- IPFS节点需要配置为运营者，确保内容持久化

## 最佳实践

### 1. 消息内容加密

**推荐加密方案**：

- 使用AES-256-GCM进行对称加密
- 使用ECDH协议派生共享密钥
- 每条消息使用随机IV（初始化向量）

**前端实现示例**：

```typescript
import CryptoJS from 'crypto-js';
import { randomBytes } from 'crypto';

// 加密消息
const encryptMessage = (message: string, sharedKey: string): string => {
  const iv = randomBytes(16).toString('hex');
  const encrypted = CryptoJS.AES.encrypt(
    message,
    CryptoJS.enc.Hex.parse(sharedKey),
    {
      iv: CryptoJS.enc.Hex.parse(iv),
      mode: CryptoJS.mode.CBC,
      padding: CryptoJS.pad.Pkcs7
    }
  );

  // 返回 IV + 密文
  return iv + encrypted.toString();
};

// 解密消息
const decryptMessage = (encryptedData: string, sharedKey: string): string => {
  const iv = encryptedData.slice(0, 32);
  const ciphertext = encryptedData.slice(32);

  const decrypted = CryptoJS.AES.decrypt(
    ciphertext,
    CryptoJS.enc.Hex.parse(sharedKey),
    {
      iv: CryptoJS.enc.Hex.parse(iv),
      mode: CryptoJS.mode.CBC,
      padding: CryptoJS.pad.Pkcs7
    }
  );

  return decrypted.toString(CryptoJS.enc.Utf8);
};
```

### 2. IPFS内容管理

**推荐实践**：

- 使用私有IPFS节点或Pinata等托管服务
- 定期Pin重要消息，防止内容丢失
- 设置合理的过期策略，清理临时消息

### 3. 性能优化

**消息分页加载**：

- 首次加载最新20-50条消息
- 支持上拉加载更多历史消息
- 使用虚拟滚动优化长列表性能

**会话列表优化**：

- 只加载最近活跃的会话（前100个）
- 缓存会话列表，定期更新
- 使用未读计数排序，未读会话置顶

### 4. 安全建议

**密钥管理**：

- 私钥不要存储在浏览器LocalStorage
- 使用安全的密钥派生函数（如PBKDF2）
- 支持硬件钱包（如Ledger）

**内容验证**：

- 验证消息签名，防止伪造
- 检查消息时间戳，防止重放攻击
- 限制消息大小，防止DoS攻击

## 注意事项

1. **链上存储成本**：链上只存储元数据（约200字节/消息），成本可控
2. **IPFS内容持久化**：重要消息应Pin到IPFS节点，防止内容丢失
3. **频率限制**：默认10条消息/10分钟，超过限制会被拒绝
4. **黑名单机制**：拉黑是单向的（A拉黑B ≠ B拉黑A）
5. **消息删除**：软删除不会从链上移除数据，双方都删除且过期后才可被清理
6. **会话管理**：每对用户只有一个会话，会话ID基于用户地址生成
7. **性能考虑**：大量消息时使用分页加载，避免一次性查询所有会话
8. **安全风险**：务必加密消息内容，使用安全的密钥派生协议

## 路线图

### 已完成

- ✅ 基础私聊功能
- ✅ 会话管理
- ✅ 已读/未读状态
- ✅ 软删除机制
- ✅ 黑名单系统
- ✅ 频率限制防护
- ✅ CID加密验证
- ✅ 无限消息和会话支持
- ✅ 旧消息清理功能
- ✅ 分页查询优化

### 未来规划

- 🔄 **群聊功能**：支持多人群聊（3-100人）
- 🔄 **消息回复**：支持回复特定消息
- 🔄 **消息撤回**：发送后一定时间内可撤回
- 🔄 **消息转发**：支持转发消息到其他会话
- 🔄 **阅后即焚**：设置消息自动销毁时间
- 🔄 **富文本支持**：支持Markdown格式
- 🔄 **消息搜索**：全文搜索历史消息
- 🔄 **在线状态**：显示用户在线/离线状态
- 🔄 **输入状态**：显示"对方正在输入..."
- 🔄 **端到端加密群聊**：使用Signal Protocol

---

**版本**: v1.3.0
**最后更新**: 2025-11-04
**维护者**: Stardust 开发团队
