# Pallet Matchmaking Interaction（婚恋模块 - 互动功能）

## 📋 模块概述

`pallet-matchmaking-interaction` 是婚恋系统的互动功能模块，采用隐私保护设计。

### 核心特性

- ✅ **点赞/超级喜欢**：表达好感，超级喜欢为付费功能
- ✅ **跳过/屏蔽**：跳过或屏蔽不想看到的用户
- ✅ **匹配检测**：检测互相喜欢，自动建立匹配
- ✅ **隐私保护**：互动记录哈希化存储
- ✅ **配额系统**：每日点赞、超级喜欢、查看配额
- ✅ **聊天发起**：匹配后可发起聊天

---

## 🔐 隐私保护机制

### 哈希化存储

- **互动记录**：`hash(from || to || from_salt)`
- **接收记录**：`hash(from || to || to_salt)`
- **屏蔽记录**：`hash(blocker || blocked || blocker_salt)`

### 隐私盐值

每个用户拥有唯一的 16 字节盐值，用于生成哈希：

```rust
pub struct PrivacySalt {
    pub salt: [u8; 16],
}
```

### 隐私保护效果

- 第三方无法直接查询谁喜欢谁
- 仅当事人可验证互动关系
- 事件中使用哈希而非明文账户

---

## 🔑 核心功能

### 1. 点赞

```rust
pub fn like(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult;
```

- 消耗每日点赞配额
- 检查是否被对方屏蔽
- 自动检测互相喜欢并建立匹配

### 2. 超级喜欢（付费）

```rust
pub fn super_like(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult;
```

- 需支付 `SuperLikeCost` 费用
- 添加到接收者的优先队列
- 在推荐列表中优先展示
- 可获得聊天发起特权

### 3. 屏蔽/取消屏蔽

```rust
pub fn block_user(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult;
pub fn unblock_user(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult;
```

### 4. 查看资料

```rust
pub fn view_profile(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult;
```

- 消耗每日查看配额
- 同一天重复查看不消耗配额
- 记录查看历史（谁看过我）

### 5. 发起聊天

```rust
pub fn initiate_matchmaking_chat(
    origin: OriginFor<T>,
    receiver: T::AccountId,
) -> DispatchResult;
```

**权限规则**：
- 已匹配用户可发起（消耗配额）
- 收到超级喜欢后可发起（不消耗配额）
- 已有会话可继续（不消耗配额）
- 被动回复不消耗配额

---

## 📊 配额系统

### 每日配额类型

| 配额类型 | 免费用户 | 会员用户 |
|----------|----------|----------|
| 点赞 | `FreeDailyLikes` | 更多/无限 |
| 超级喜欢 | `FreeDailySuperLikes` | `MemberDailySuperLikes` |
| 查看资料 | `FreeDailyViews` | `MemberDailyViews` |
| 发起聊天 | `FreeDailyChatInitiations` | 更多/无限 |

### 配额重置

每日自动重置（基于区块号计算日期）

---

## 📊 数据结构

### 互动记录

```rust
pub struct InteractionRecord<T: Config> {
    pub interaction_type: InteractionType,
    pub timestamp: BlockNumberFor<T>,
}
```

### 超级喜欢记录

```rust
pub struct SuperLikeRecord {
    pub sender_hash: [u8; 32],
    pub sent_at: u64,
    pub viewed: bool,
}
```

### 聊天会话信息

```rust
pub struct ChatSessionInfo {
    pub created_at: u64,
    pub initiation_type: ChatInitiationType,
}

pub enum ChatInitiationType {
    InitiatedByMe,       // 我主动发起
    InitiatedByOther,    // 对方先发起
    SuperLikePrivilege,  // 超级喜欢特权
}
```

### 每日配额

```rust
pub struct DailyQuota {
    pub likes_used: u32,
    pub super_likes_used: u32,
    pub views_used: u32,
    pub last_reset_day: u32,
}
```

---

## 💾 存储项

```rust
/// 用户隐私盐值
pub type UserSalt<T> = StorageMap<_, Blake2_128Concat, T::AccountId, PrivacySalt>;

/// 互动记录（哈希化）
pub type Interactions<T> = StorageMap<_, Identity, [u8; 32], InteractionRecord<T>>;

/// 用户发出的互动哈希列表
pub type MyInteractions<T> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<[u8; 32], T::MaxInteractionsPerUser>>;

/// 收到的点赞数量
pub type LikesReceivedCount<T> = StorageMap<_, Blake2_128Concat, T::AccountId, u32>;

/// 匹配列表（加密存储）
pub type Matches<T> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<EncryptedMatchRecord, T::MaxInteractionsPerUser>>;

/// 屏蔽列表（哈希化）
pub type BlockedHashes<T> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<[u8; 32], T::MaxInteractionsPerUser>>;

/// 收到的超级喜欢队列
pub type SuperLikesReceived<T> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<SuperLikeRecord, T::MaxSuperLikesReceived>>;

/// 每日配额
pub type DailyQuotas<T> = StorageMap<_, Blake2_128Concat, T::AccountId, DailyQuota>;

/// 聊天会话
pub type ChatSessions<T> = StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, [u8; 32], ChatSessionInfo>;

/// 查看历史
pub type ViewHistory<T> = StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, T::AccountId, BlockNumberFor<T>>;

/// 谁看过我
pub type ProfileViewers<T> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<(T::AccountId, BlockNumberFor<T>), ConstU32<100>>>;
```

---

## 🎯 外部调用（Extrinsics）

| 调用 | 描述 |
|------|------|
| `initialize_salt` | 初始化隐私盐值 |
| `like` | 点赞 |
| `super_like` | 超级喜欢（付费） |
| `pass` | 跳过 |
| `block_user` | 屏蔽用户 |
| `unblock_user` | 取消屏蔽 |
| `verify_interaction` | 验证互动关系 |
| `mark_super_like_viewed` | 标记超级喜欢已查看 |
| `initiate_matchmaking_chat` | 发起聊天 |
| `view_profile` | 查看资料 |

---

## 📡 事件定义

```rust
InteractionSent { from, interaction_hash, interaction_type }
InteractionReceived { to, interaction_hash, interaction_type }
MatchSuccess { match_hash }
UserBlocked { from, block_hash }
UserUnblocked { from, unblock_hash }
SaltInitialized { user }
SuperLikeSent { from, to, cost }
SuperLikeReceived { to, sender_hash }
ChatSessionEstablished { user, target_hash, initiation_type }
ChatInitiationQuotaConsumed { user, remaining, limit }
```

---

## ⚙️ 配置参数

```rust
impl pallet_matchmaking_interaction::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxInteractionsPerUser = ConstU32<1000>;
    type MaxSuperLikesReceived = ConstU32<100>;
    type SuperLikeCost = ConstU128<1_000_000_000_000_000_000>;  // 1 COS
    type FreeDailyLikes = ConstU32<50>;
    type FreeDailySuperLikes = ConstU32<0>;
    type MemberDailySuperLikes = ConstU32<5>;
    type FreeDailyViews = ConstU32<50>;
    type MemberDailyViews = ConstU32<0>;  // 0 = 无限
    type FreeDailyChatInitiations = ConstU32<10>;
    type MonthlyMemberDailyChatInitiations = ConstU32<50>;
    type YearlyMemberDailyChatInitiations = ConstU32<0>;  // 0 = 无限
    type BlocksPerDay = ConstU32<14400>;
    type Fungible = Balances;
    type TreasuryAccount = TreasuryAccountId;
    type WeightInfo = ();
}
```

---

## 📚 相关文档

- [婚恋模块主文档](../README.md)
- [用户资料模块](../profile/README.md)
