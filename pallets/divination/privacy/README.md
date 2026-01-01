# pallet-divination-privacy

统一隐私授权模块 - 为所有占卜系统提供加密存储和多方授权功能。

## 概述

本模块解决了占卜系统中的隐私和授权问题：

- **问题**：加密的占卜数据（如八字出生时间）与悬赏系统之间缺乏授权集成，导致大师接单后无法访问加密数据
- **解决方案**：创建统一的隐私授权模块，为所有占卜类型提供标准化的加密存储和多方授权机制

## 功能

### 1. 密钥管理
- 用户注册 X25519 加密公钥
- 公钥更新
- 公钥查询

### 2. 服务提供者管理
- 命理师注册
- AI 服务注册
- 家族成员注册
- 研究机构注册
- 提供者状态管理（活跃/停用）

### 3. 加密数据存储
- 创建加密记录（AES-256-GCM）
- 更新加密数据
- 删除加密记录
- 隐私模式管理（公开/部分加密/完全私密）
- Partial 和 Private 模式支持授权访问

### 4. 授权管理
- 授权访问
- 撤销授权
- 批量撤销
- 授权范围控制（只读/可评论/完全访问）
- 授权角色控制（所有者/命理师/家族/AI/悬赏回答者）
- 授权过期机制

### 5. 悬赏系统集成
- 创建悬赏授权配置
- 为回答者授权
- 悬赏结束时批量撤销授权
- 自动授权支持

## 架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        pallet-divination-market                          │
│                         (悬赏问答 & 订单系统)                              │
└───────────────────────────────────────┬─────────────────────────────────┘
                                        │ BountyPrivacy trait
                                        ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                       pallet-divination-privacy                          │
│                         (统一隐私授权模块)                                │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────┐    │
│  │ 密钥管理    │ │ 授权管理    │ │ 加密存储    │ │ 服务提供者管理  │    │
│  │ - 公钥注册  │ │ - 授权/撤销 │ │ - 加密数据  │ │ - 命理师注册    │    │
│  │ - 密钥更新  │ │ - 角色/范围 │ │ - 解密凭证  │ │ - AI服务注册    │    │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────────┘    │
└───────────────────────────────────────┬─────────────────────────────────┘
                                        │ DivinationPrivacy trait
                  ┌─────────────────────┼─────────────────┐
                  ▼                     ▼                 ▼
         ┌──────────────┐      ┌──────────────┐   ┌──────────────┐
         │ pallet-bazi  │      │pallet-meihua │   │pallet-liuyao │  ...
         │   (八字)     │      │  (梅花易数)   │   │   (六爻)     │
         └──────────────┘      └──────────────┘   └──────────────┘
```

## 数据类型

### 隐私模式 (PrivacyMode)

| 值 | 说明 | 支持授权 | 推荐场景 |
|---|---|:---:|---|
| Public | 公开 - 所有人可见 | ❌ | 公开展示、教学案例 |
| Partial | 部分加密 - 计算数据明文 + 敏感数据加密 | ✅ | 奇门遁甲、命运档案 |
| Private | 完全私密 - 全部数据加密 | ✅ | 高度敏感的个人数据 |

**授权说明**: Partial 和 Private 模式下，所有者可以授权他人（咨询师/家人/AI服务）访问加密数据。

### 授权角色 (AccessRole)
| 值 | 说明 |
|---|---|
| Owner | 所有者 - 不可撤销 |
| Master | 命理师 - 专业解读者 |
| Family | 家族成员 - 家庭内部共享 |
| AiService | AI 服务 - 自动化解读 |
| BountyAnswerer | 悬赏回答者 - 临时授权 |

### 访问范围 (AccessScope)
| 值 | 说明 |
|---|---|
| ReadOnly | 只读 - 仅能查看 |
| CanComment | 可评论 - 可以查看并添加解读评论 |
| FullAccess | 完全访问 - 完全访问所有数据 |

## 交易接口

### 密钥管理
- `register_encryption_key(public_key)` - 注册加密公钥
- `update_encryption_key(new_public_key)` - 更新加密公钥

### 服务提供者管理
- `register_provider(provider_type, public_key)` - 注册为服务提供者
- `update_provider_key(new_public_key)` - 更新提供者公钥
- `set_provider_active(is_active)` - 设置活跃状态
- `unregister_provider()` - 注销服务提供者

### 加密数据管理
- `create_encrypted_record(...)` - 创建加密记录
- `update_encrypted_record(...)` - 更新加密记录
- `change_privacy_mode(...)` - 更改隐私模式
- `delete_encrypted_record(...)` - 删除加密记录

### 授权管理
- `grant_access(...)` - 授权访问
- `revoke_access(...)` - 撤销授权
- `revoke_all_access(...)` - 撤销所有授权
- `update_access_scope(...)` - 更新授权范围

### 悬赏授权
- `create_bounty_authorization(...)` - 创建悬赏授权配置
- `authorize_bounty_answerer(...)` - 为回答者授权
- `revoke_bounty_authorizations(...)` - 撤销悬赏所有授权

## Trait 接口

### DivinationPrivacy
供各占卜模块使用的隐私管理接口：
- `is_encrypted()` - 检查记录是否加密
- `get_privacy_mode()` - 获取隐私模式
- `has_access()` - 检查访问权限
- `get_access_role()` - 获取访问角色
- `get_grantees()` - 获取授权列表
- `get_owner()` - 获取所有者
- `get_user_public_key()` - 获取用户公钥
- `get_provider_type()` - 获取提供者类型
- `is_provider_active()` - 检查提供者是否活跃

### BountyPrivacy
供悬赏系统使用的授权管理接口：
- `is_bounty_encrypted()` - 检查悬赏关联数据是否加密
- `can_answer_bounty()` - 检查回答者是否有权限
- `get_bounty_authorizations()` - 获取悬赏授权列表
- `bounty_requires_authorization()` - 检查悬赏是否需要授权
- `get_bounty_authorization_expiry()` - 获取授权过期时间
- `is_auto_authorize_enabled()` - 检查是否启用自动授权

## 配置参数

| 参数 | 说明 | 推荐值 |
|---|---|---|
| MaxEncryptedDataLen | 加密数据最大长度 | 512 |
| MaxEncryptedKeyLen | 加密密钥最大长度 | 128 |
| MaxGranteesPerRecord | 单条记录最大授权数 | 20 |
| MaxRecordsPerUser | 用户最大记录数（按类型） | 1000 |
| MaxProvidersPerType | 服务提供者最大数量（按类型） | 1000 |
| MaxGrantsPerProvider | 提供者最大被授权记录数 | 500 |
| MaxAuthorizationsPerBounty | 单个悬赏最大授权数 | 100 |

## 使用示例

### 1. 用户注册公钥
```rust
Privacy::register_encryption_key(origin, [/* X25519 公钥 */])?;
```

### 2. 创建加密记录（Partial 模式）
```rust
Privacy::create_encrypted_record(
    origin,
    DivinationType::Bazi,
    result_id,
    PrivacyMode::Partial,  // 部分加密 - 计算数据明文，敏感数据加密
    encrypted_data,
    nonce,
    auth_tag,
    data_hash,
    owner_encrypted_key,
)?;
```

### 3. 授权大师访问
```rust
Privacy::grant_access(
    origin,
    DivinationType::Bazi,
    result_id,
    master_account,
    encrypted_key_for_master,
    AccessRole::Master,
    AccessScope::CanComment,
    0, // 永不过期
)?;
```

### 4. 创建悬赏授权
```rust
// 创建悬赏时
Privacy::create_bounty_authorization(
    origin,
    bounty_id,
    DivinationType::Bazi,
    result_id,
    expires_at,
    auto_authorize,
)?;

// 大师接单后
Privacy::authorize_bounty_answerer(
    origin,
    bounty_id,
    answerer,
    encrypted_key_for_answerer,
)?;
```

## 运行测试

```bash
cargo test -p pallet-divination-privacy
```

## 集成指南

### 1. 添加依赖
```toml
[dependencies]
pallet-divination-privacy = { path = "../privacy", default-features = false }
```

### 2. 配置 Runtime
```rust
impl pallet_divination_privacy::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxEncryptedDataLen = ConstU32<512>;
    type MaxEncryptedKeyLen = ConstU32<128>;
    type MaxGranteesPerRecord = ConstU32<20>;
    type MaxRecordsPerUser = ConstU32<1000>;
    type MaxProvidersPerType = ConstU32<1000>;
    type MaxGrantsPerProvider = ConstU32<500>;
    type MaxAuthorizationsPerBounty = ConstU32<100>;
    type EventHandler = ();
    type WeightInfo = pallet_divination_privacy::weights::SubstrateWeight<Runtime>;
}
```

### 3. 在占卜模块中使用
```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    type Privacy: DivinationPrivacy<Self::AccountId, BlockNumberFor<Self>>;
}

// 检查访问权限
if T::Privacy::has_access(divination_type, result_id, &who) {
    // 允许访问
}
```

### 4. 在悬赏模块中使用
```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    type Privacy: BountyPrivacy<Self::AccountId, BlockNumberFor<Self>>;
}

// 检查是否需要授权
if T::Privacy::bounty_requires_authorization(divination_type, result_id) {
    // 提示用户需要授权
}
```

## 安全考虑

1. **加密方案**：使用 AES-256-GCM 对称加密，X25519 密钥交换
2. **密钥管理**：私钥仅在用户本地存储，链上只存储公钥
3. **授权控制**：所有者授权不可撤销，其他授权可随时撤销
4. **过期机制**：支持授权过期，悬赏结束后自动撤销

## 许可证

MIT License
