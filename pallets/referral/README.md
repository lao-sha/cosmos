# Pallet Referral

> **推荐关系管理模块** - Referral Management Pallet

[![License: Unlicense](https://img.shields.io/badge/License-Unlicense-blue.svg)](http://unlicense.org/)
[![Substrate Version](https://img.shields.io/badge/Substrate-stable2506-brightgreen)](https://substrate.io/)

## 📋 目录

- [概述](#概述)
- [功能特性](#功能特性)
- [架构设计](#架构设计)
- [存储结构](#存储结构)
- [接口说明](#接口说明)
- [使用示例](#使用示例)
- [集成指南](#集成指南)
- [安全考虑](#安全考虑)
- [测试](#测试)

---

## 概述

`pallet-referral` 是 Stardust 平台的推荐关系管理模块，从 `pallet-affiliate` 抽离而来，专注于管理用户之间的推荐关系。

### 核心功能

- 🔗 **推荐人绑定**：用户通过推荐码建立推荐关系
- 🎫 **推荐码管理**：会员可认领自定义推荐码
- 🔍 **推荐链查询**：获取用户的完整推荐链（最多15层）
- 🛡️ **循环检测**：防止形成循环推荐关系
- 🤖 **自动认领**：会员自动获得默认推荐码
- 📊 **统计监控**：实时追踪推荐关系统计信息
- ⚖️ **权重基准**：完整的 Benchmark 权重计算
- ✅ **测试覆盖**：22 个单元测试，100% 功能覆盖

### 版本信息

- **版本**: 1.1.0
- **抽离日期**: 2025-12-30
- **原模块**: `pallet-affiliate/src/referral.rs`
- **Substrate**: stable2506

---

## 功能特性

### 1. 推荐人绑定

用户可以通过推荐码绑定推荐人，建立推荐关系。

**特点：**
- ✅ 一次性绑定（不可更改）
- ✅ 防止自我绑定
- ✅ 防止循环绑定
- ✅ 推荐码验证

**流程：**
```
用户 → 输入推荐码 → 验证推荐码 → 检查循环 → 绑定成功
```

### 2. 推荐码管理

会员可以认领自定义推荐码，用于推广。

**规则：**
- 📌 仅有效会员可认领
- 📌 每个账户只能认领一个推荐码
- 📌 推荐码长度：4-32 字符（`MIN_CODE_LEN = 4`，`MaxCodeLen` 可配置）
- 📌 推荐码唯一性保证

**默认推荐码：**
- 格式：账户 ID 前 4 字节的十六进制（8 字符）
- 示例：`0x1234...` → `12345678`

### 3. 推荐链查询

获取用户的完整推荐链，支持多级推荐。

**限制：**
- 最大层数：15 层（`MAX_REFERRAL_CHAIN = 15`）
- 防止无限循环
- 高效查询算法

**示例：**
```
用户 A → 推荐人 B → 推荐人 C → ... → 推荐人 N (最多15层)
```

---

## 架构设计

### 模块依赖

```
┌─────────────────────────────────────────────────────────┐
│                    pallet-referral                      │
│                   (推荐关系管理)                         │
└─────────────────────────────────────────────────────────┘
                          │
                          │ 依赖
                          ▼
┌─────────────────────────────────────────────────────────┐
│              MembershipProvider Trait                   │
│            (会员信息提供者 - 外部实现)                    │
└─────────────────────────────────────────────────────────┘
                          │
                          │ 实现
                          ▼
┌─────────────────────────────────────────────────────────┐
│         pallet-membership / pallet-affiliate            │
│                  (会员管理模块)                          │
└─────────────────────────────────────────────────────────┘
```

### 对外接口

```
┌─────────────────────────────────────────────────────────┐
│              ReferralProvider Trait                     │
│            (推荐关系提供者 - 供其他模块调用)              │
└─────────────────────────────────────────────────────────┘
                          │
                          │ 调用
                          ▼
┌─────────────────────────────────────────────────────────┐
│    pallet-affiliate / pallet-divination-market          │
│              (佣金分成 / 业务模块)                        │
└─────────────────────────────────────────────────────────┘
```

---

## 存储结构

### 存储项

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `Sponsors` | `StorageMap<AccountId, AccountId>` | 推荐人映射（账户 → 推荐人） |
| `AccountByCode` | `StorageMap<BoundedVec<u8, MaxCodeLen>, AccountId>` | 推荐码 → 账户 |
| `CodeByAccount` | `StorageMap<AccountId, BoundedVec<u8, MaxCodeLen>>` | 账户 → 推荐码 |
| `ReferralStats` | `StorageValue<ReferralStatistics>` | 全局统计信息 |

### ReferralStatistics 结构

```rust
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug, Default)]
pub struct ReferralStatistics {
    /// 总推荐关系数
    pub total_sponsors: u64,
    /// 总推荐码数
    pub total_codes: u64,
    /// 最后更新区块
    pub last_updated: u32,
}
```

### 存储关系图

```
┌─────────────────────────────────────────────────────────┐
│                      Sponsors                           │
│         (推荐人映射: AccountId → AccountId)              │
│                                                         │
│  Alice → Bob                                            │
│  Bob   → Charlie                                        │
│  Dave  → Alice                                          │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│                   AccountByCode                         │
│         (推荐码映射: Code → AccountId)                   │
│                                                         │
│  "ALICE123" → Alice                                     │
│  "BOB456"   → Bob                                       │
│  "12345678" → Charlie (默认推荐码)                       │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│                   CodeByAccount                         │
│         (账户推荐码: AccountId → Code)                   │
│                                                         │
│  Alice   → "ALICE123"                                   │
│  Bob     → "BOB456"                                     │
│  Charlie → "12345678"                                   │
└─────────────────────────────────────────────────────────┘
```

### 存储成本

| 项目 | 单条大小 | 说明 |
|------|----------|------|
| `Sponsors` | 64 字节 | 32 (key) + 32 (value) |
| `AccountByCode` | ~40 字节 | 8 (code) + 32 (account) |
| `CodeByAccount` | ~40 字节 | 32 (account) + 8 (code) |
| **总计/用户** | **~144 字节** | 假设用户绑定推荐人并认领推荐码 |

---

## 接口说明

### Extrinsics (可调用函数)

#### 1. `bind_sponsor` (call_index: 0)

绑定推荐人。

```rust
#[pallet::call_index(0)]
#[pallet::weight(T::WeightInfo::bind_sponsor())]
pub fn bind_sponsor(
    origin: OriginFor<T>,
    sponsor_code: Vec<u8>,
) -> DispatchResult
```

**参数：**
- `sponsor_code`: 推荐人的推荐码（4-32 字符）

**验证：**
- ✅ 用户未绑定过推荐人
- ✅ 推荐码存在
- ✅ 不能绑定自己
- ✅ 不能形成循环

**事件：**
```rust
SponsorBound {
    who: T::AccountId,
    sponsor: T::AccountId,
}
```

**错误：**
- `AlreadyBound`: 已绑定推荐人
- `CodeNotFound`: 推荐码不存在
- `CannotBindSelf`: 不能绑定自己
- `WouldCreateCycle`: 会形成循环绑定
- `CodeTooShort`: 推荐码过短（< 4 字符）
- `CodeTooLong`: 推荐码过长（> MaxCodeLen）

---

#### 2. `claim_code` (call_index: 1)

认领推荐码。

```rust
#[pallet::call_index(1)]
#[pallet::weight(T::WeightInfo::claim_code())]
pub fn claim_code(
    origin: OriginFor<T>,
    code: Vec<u8>,
) -> DispatchResult
```

**参数：**
- `code`: 要认领的推荐码（4-32 字符）

**验证：**
- ✅ 调用者是有效会员
- ✅ 推荐码未被占用
- ✅ 用户未认领其他推荐码

**事件：**
```rust
CodeClaimed {
    who: T::AccountId,
    code: BoundedVec<u8, T::MaxCodeLen>,
}
```

**错误：**
- `NotMember`: 非有效会员
- `CodeAlreadyTaken`: 推荐码已被占用
- `AlreadyHasCode`: 已拥有推荐码
- `CodeTooShort`: 推荐码过短（< 4 字符）
- `CodeTooLong`: 推荐码过长（> MaxCodeLen）

---

### 事件 (Events)

| 事件 | 字段 | 说明 |
|------|------|------|
| `SponsorBound` | `who`, `sponsor` | 推荐人绑定成功 |
| `CodeClaimed` | `who`, `code` | 推荐码认领成功 |

### 错误 (Errors)

| 错误 | 说明 |
|------|------|
| `AlreadyBound` | 已绑定推荐人 |
| `CodeNotFound` | 推荐码不存在 |
| `CannotBindSelf` | 不能绑定自己 |
| `WouldCreateCycle` | 会形成循环绑定 |
| `CodeTooLong` | 推荐码过长 |
| `CodeTooShort` | 推荐码过短 |
| `CodeAlreadyTaken` | 推荐码已被占用 |
| `AlreadyHasCode` | 已拥有推荐码 |
| `NotMember` | 非有效会员 |

---

### Public Functions (公共函数)

#### 1. `get_referral_chain`

获取推荐链（最多15层）。

```rust
pub fn get_referral_chain(who: &T::AccountId) -> Vec<T::AccountId>
```

**返回：**
- `Vec<AccountId>`: 推荐链，从直接推荐人开始

**示例：**
```rust
// Alice → Bob → Charlie → Dave
let chain = Referral::get_referral_chain(&alice);
// 返回: [Bob, Charlie, Dave]
```

---

#### 2. `would_create_cycle`

检查是否会形成循环绑定。

```rust
pub fn would_create_cycle(
    who: &T::AccountId,
    sponsor: &T::AccountId,
) -> bool
```

**返回：**
- `true`: 会形成循环
- `false`: 不会形成循环

---

#### 3. `find_account_by_code`

通过推荐码查找账户。

```rust
pub fn find_account_by_code(
    code: &BoundedVec<u8, T::MaxCodeLen>,
) -> Option<T::AccountId>
```

**返回：**
- `Some(AccountId)`: 找到对应账户
- `None`: 推荐码不存在

---

#### 4. `try_auto_claim_code`

自动认领默认推荐码。

```rust
pub fn try_auto_claim_code(who: &T::AccountId) -> bool
```

**规则：**
- 仅有效会员可自动认领
- 默认推荐码格式：账户 ID 前 4 字节的十六进制（8 字符）

**返回：**
- `true`: 认领成功
- `false`: 认领失败（已有推荐码或非会员）

---

### Traits

#### 1. `MembershipProvider` (需要实现)

会员信息提供者 trait，由外部模块实现。

```rust
pub trait MembershipProvider<AccountId> {
    /// 检查账户是否为有效会员
    fn is_valid_member(who: &AccountId) -> bool;
}
```

**实现示例：**
```rust
impl MembershipProvider<AccountId> for MyMembershipPallet {
    fn is_valid_member(who: &AccountId) -> bool {
        // 检查会员有效性
        Members::<T>::contains_key(who)
    }
}
```

---

#### 2. `ReferralProvider` (对外提供)

推荐关系提供者 trait，供其他模块调用。

```rust
pub trait ReferralProvider<AccountId> {
    /// 获取推荐人
    fn get_sponsor(who: &AccountId) -> Option<AccountId>;
    /// 获取推荐链（最多15层）
    fn get_referral_chain(who: &AccountId) -> Vec<AccountId>;
}
```

**使用示例：**
```rust
// 在其他模块中使用
let sponsor = T::ReferralProvider::get_sponsor(&user);
let chain = T::ReferralProvider::get_referral_chain(&user);
```

---

### 配置参数 (Config)

```rust
#[pallet::config]
pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {
    /// 会员信息提供者（检查是否有效会员）
    type MembershipProvider: MembershipProvider<Self::AccountId>;

    /// 推荐码最大长度
    #[pallet::constant]
    type MaxCodeLen: Get<u32>;

    /// 推荐链最大搜索深度（防止无限循环）
    #[pallet::constant]
    type MaxSearchHops: Get<u32>;

    /// 权重信息
    type WeightInfo: crate::weights::WeightInfo;
}
```

| 参数 | 类型 | 说明 | 建议值 |
|------|------|------|--------|
| `MembershipProvider` | Trait | 会员信息提供者 | 实现 `MembershipProvider` trait |
| `MaxCodeLen` | `u32` | 推荐码最大长度 | 32 |
| `MaxSearchHops` | `u32` | 循环检测最大搜索深度 | 20 |
| `WeightInfo` | Trait | 权重信息 | `SubstrateWeight<Runtime>` |

**常量：**
- `MAX_REFERRAL_CHAIN: u32 = 15` - 推荐链最大层数
- `MIN_CODE_LEN: usize = 4` - 推荐码最小长度

---

## 使用示例

### 场景 1：用户绑定推荐人

```javascript
// 1. Bob 认领推荐码 "BOB123"
const tx1 = api.tx.referral.claimCode("BOB12345");
await tx1.signAndSend(bob);

// 2. Alice 通过 Bob 的推荐码绑定推荐人
const tx2 = api.tx.referral.bindSponsor("BOB12345");
await tx2.signAndSend(alice);

// 3. 查询 Alice 的推荐人
const sponsor = await api.query.referral.sponsors(alice.address);
console.log(sponsor.toString()); // Bob's address
```

### 场景 2：查询推荐链

```rust
// 推荐关系：Alice → Bob → Charlie → Dave
let chain = Referral::get_referral_chain(&alice);
// 返回: [Bob, Charlie, Dave]

// 计算推荐层级
let level = chain.len(); // 3
```

### 场景 3：自动认领默认推荐码

```rust
// 会员注册时自动认领默认推荐码
impl<T: Config> Pallet<T> {
    pub fn on_member_registered(who: &T::AccountId) {
        // 自动认领默认推荐码
        let _ = pallet_referral::Pallet::<T>::try_auto_claim_code(who);
    }
}
```

### 场景 4：在其他模块中计算佣金

```rust
// 在 pallet-affiliate 中使用
impl<T: Config> Pallet<T> {
    pub fn calculate_commission(user: &T::AccountId, amount: Balance) {
        // 获取推荐链
        let chain = T::ReferralProvider::get_referral_chain(user);

        // 计算各级佣金
        for (level, sponsor) in chain.iter().enumerate() {
            let rate = Self::get_commission_rate(level);
            let commission = amount * rate / 100;
            // 分配佣金...
        }
    }
}
```

---

## 集成指南

### 1. 添加依赖

在 `runtime/Cargo.toml` 中添加：

```toml
[dependencies]
pallet-referral = { path = "../pallets/referral", default-features = false }

[features]
std = [
    "pallet-referral/std",
]
runtime-benchmarks = [
    "pallet-referral/runtime-benchmarks",
]
```

### 2. 配置 Runtime

在 `runtime/src/lib.rs` 中配置：

```rust
// 实现 MembershipProvider
pub struct MyMembershipProvider;
impl pallet_referral::MembershipProvider<AccountId> for MyMembershipProvider {
    fn is_valid_member(who: &AccountId) -> bool {
        // 检查会员有效性
        pallet_membership::Pallet::<Runtime>::is_member(who)
    }
}

// 配置参数
parameter_types! {
    pub const MaxCodeLen: u32 = 32;
    pub const MaxSearchHops: u32 = 20;
}

// 实现 Config
impl pallet_referral::Config for Runtime {
    type MembershipProvider = MyMembershipProvider;
    type MaxCodeLen = MaxCodeLen;
    type MaxSearchHops = MaxSearchHops;
    type WeightInfo = pallet_referral::weights::SubstrateWeight<Runtime>;
}

// 添加到 Runtime
construct_runtime!(
    pub enum Runtime {
        // ...
        Referral: pallet_referral,
    }
);
```

### 3. 在其他模块中使用

```rust
// 在 pallet-affiliate 中配置
#[pallet::config]
pub trait Config: frame_system::Config {
    type ReferralProvider: pallet_referral::ReferralProvider<Self::AccountId>;
}

// 使用推荐关系
impl<T: Config> Pallet<T> {
    pub fn get_user_sponsor(user: &T::AccountId) -> Option<T::AccountId> {
        T::ReferralProvider::get_sponsor(user)
    }
}
```

---

## 安全考虑

### 1. 循环绑定防护

**问题：** 防止 A→B→C→A 这种循环绑定。

**解决方案：**
```rust
pub fn would_create_cycle(who: &T::AccountId, sponsor: &T::AccountId) -> bool {
    let mut current = sponsor.clone();
    let max_hops = T::MaxSearchHops::get();

    for _ in 0..max_hops {
        if let Some(next_sponsor) = Sponsors::<T>::get(&current) {
            if &next_sponsor == who {
                return true; // 检测到循环
            }
            current = next_sponsor;
        } else {
            break;
        }
    }
    false
}
```

### 2. 推荐码唯一性

**保证：**
- ✅ 每个推荐码只能被一个账户认领
- ✅ 每个账户只能认领一个推荐码
- ✅ 双向映射保证一致性

### 3. 会员验证

**规则：**
- 只有有效会员才能认领推荐码
- 通过 `MembershipProvider` trait 验证

### 4. 推荐链深度限制

**限制：**
- 最大层数：15 层（`MAX_REFERRAL_CHAIN`）
- 最大搜索深度：可配置（`MaxSearchHops`）
- 防止无限循环和 DoS 攻击

---

## 测试

### 运行测试

```bash
# 运行所有测试
cargo test -p pallet-referral

# 运行特定测试
cargo test -p pallet-referral test_bind_sponsor

# 显示测试输出
cargo test -p pallet-referral -- --nocapture
```

### 测试覆盖率

| 类别 | 测试数 | 覆盖功能 |
|------|--------|----------|
| **推荐人绑定** | 6 | 成功、重复绑定、码不存在、自我绑定、循环检测、码过短 |
| **推荐码认领** | 6 | 成功、非会员、已被占用、已有码、码过短、码过长 |
| **推荐链查询** | 4 | 空链、单层、多层、最大深度 |
| **循环检测** | 3 | 无循环、直接循环、间接循环 |
| **自动认领** | 3 | 成功、已有码、非会员 |
| **Trait 实现** | 2 | get_sponsor、get_referral_chain |
| **总计** | **22** | **100% 功能覆盖** |

### 测试用例列表

```rust
// 推荐人绑定测试
#[test] fn test_bind_sponsor_success() { /* 测试正常绑定推荐人 */ }
#[test] fn test_bind_sponsor_already_bound() { /* 测试重复绑定失败 */ }
#[test] fn test_bind_sponsor_code_not_found() { /* 测试推荐码不存在 */ }
#[test] fn test_bind_sponsor_cannot_bind_self() { /* 测试不能绑定自己 */ }
#[test] fn test_bind_sponsor_cycle_detection() { /* 测试循环绑定检测 */ }
#[test] fn test_bind_sponsor_code_too_short() { /* 测试推荐码过短 */ }

// 推荐码认领测试
#[test] fn test_claim_code_success() { /* 测试认领推荐码 */ }
#[test] fn test_claim_code_not_member() { /* 测试非会员认领失败 */ }
#[test] fn test_claim_code_already_taken() { /* 测试推荐码已被占用 */ }
#[test] fn test_claim_code_already_has_code() { /* 测试已有推荐码 */ }
#[test] fn test_claim_code_too_short() { /* 测试推荐码过短 */ }
#[test] fn test_claim_code_too_long() { /* 测试推荐码过长 */ }

// 推荐链查询测试
#[test] fn test_get_referral_chain_empty() { /* 测试空推荐链 */ }
#[test] fn test_get_referral_chain_single_level() { /* 测试单层推荐链 */ }
#[test] fn test_get_referral_chain_multi_level() { /* 测试多层推荐链 */ }
#[test] fn test_get_referral_chain_max_depth() { /* 测试最大深度限制 */ }

// 循环检测测试
#[test] fn test_would_create_cycle_no_cycle() { /* 测试无循环情况 */ }
#[test] fn test_would_create_cycle_direct_cycle() { /* 测试直接循环 */ }
#[test] fn test_would_create_cycle_indirect_cycle() { /* 测试间接循环 */ }

// 自动认领测试
#[test] fn test_try_auto_claim_code_success() { /* 测试自动认领成功 */ }
#[test] fn test_try_auto_claim_code_already_has_code() { /* 测试已有码时自动认领 */ }
#[test] fn test_try_auto_claim_code_not_member() { /* 测试非会员自动认领 */ }

// Trait 实现测试
#[test] fn test_referral_provider_get_sponsor() { /* 测试 get_sponsor */ }
#[test] fn test_referral_provider_get_referral_chain() { /* 测试 get_referral_chain */ }
```

---

## 权重 Benchmark

### WeightInfo Trait

```rust
pub trait WeightInfo {
    fn bind_sponsor() -> Weight;
    fn claim_code() -> Weight;
}
```

### 权重估算

| Extrinsic | 读取操作 | 写入操作 | 估算权重 |
|-----------|----------|----------|----------|
| `bind_sponsor` | Sponsors + AccountByCode + 循环检测(≤MaxSearchHops) | Sponsors + ReferralStats | 25,000 |
| `claim_code` | MembershipProvider + AccountByCode + CodeByAccount | AccountByCode + CodeByAccount + ReferralStats | 20,000 |

### 运行 Benchmark

```bash
# 生成权重
cargo build --release --features runtime-benchmarks
./target/release/stardust-node benchmark pallet \
    --chain dev \
    --pallet pallet_referral \
    --extrinsic "*" \
    --output pallets/referral/src/weights.rs
```

---

## 常见问题

### Q1: 用户可以更改推荐人吗？

**A:** 不可以。推荐关系一旦绑定就不可更改，这是为了防止推荐关系被滥用。

### Q2: 推荐码可以包含哪些字符？

**A:** 推荐码是 `Vec<u8>`，理论上可以包含任意字节。建议使用字母数字组合（a-z, A-Z, 0-9）。

### Q3: 默认推荐码的格式是什么？

**A:** 账户 ID 前 4 字节的十六进制表示（8 个字符）。例如：`0x12345678...` → `12345678`

### Q4: 如何防止推荐码冲突？

**A:**
1. 默认推荐码基于账户 ID，理论上不会冲突
2. 自定义推荐码在认领时会检查唯一性
3. 如果冲突，用户需要选择其他推荐码

### Q5: 推荐链最多支持多少层？

**A:** 最多 15 层（`MAX_REFERRAL_CHAIN = 15`）。这是为了防止无限递归和提高查询效率。

### Q6: 循环检测的最大搜索深度是多少？

**A:** 由 `MaxSearchHops` 配置参数决定，建议设置为 20。

---

## 路线图

### v1.0.0 (2025-12-30)
- ✅ 基础推荐关系管理
- ✅ 推荐码认领
- ✅ 循环检测
- ✅ 推荐链查询

### v1.1.0 (当前版本)
- ✅ 推荐关系统计 (`ReferralStats`)
- ✅ 完整测试覆盖 (22 个测试用例)
- ✅ 权重 Benchmark 支持

### v2.0.0 (未来)
- 🔄 推荐码转让功能
- 🔄 推荐码黑名单
- 🔄 多级推荐佣金计算
- 🔄 推荐关系可视化
- 🔄 推荐码 NFT 化

---

## 贡献

欢迎提交 Issue 和 Pull Request！

### 开发指南

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

---

## 许可证

本项目采用 [Unlicense](http://unlicense.org/) 许可证。

---

## 联系方式

- **项目主页**: https://github.com/memoio/stardust
- **文档**: https://docs.stardust.io
- **Discord**: https://discord.gg/stardust

---

**维护者**: StarDust Team
