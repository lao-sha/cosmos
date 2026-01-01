# Pallet Divination Privacy - 设计文档

## 概述

**pallet-divination-privacy** 是一个统一的占卜隐私和授权管理层，为所有占卜类型（八字、梅花易数、六爻、紫微斗数等）提供加密存储、多方授权、权限管理和密钥分发功能。

## 设计目标

1. **统一管理**：所有占卜类型使用相同的加密和授权机制
2. **避免重复**：消除各占卜模块的重复代码
3. **易于扩展**：新增占卜类型只需实现简单的trait
4. **安全可靠**：集中管理加密密钥和权限控制
5. **自动授权**：支持悬赏问答等场景的自动授权

## 核心功能

### 1. 加密数据管理

支持三种存储模式：
- **明文模式**：公开可见，无加密
- **单方加密**：仅所有者可见
- **多方授权加密**：支持授权多方访问

### 2. 授权角色系统

- **Owner**（所有者）：不可撤销，完全控制权
- **Master**（命理师）：可撤销，用于专业解读
- **Family**（家族成员）：可撤销，家庭内部共享
- **AiService**（AI服务）：可撤销，AI解读服务

### 3. 访问范围控制

- **ReadOnly**：只读权限，仅可查看
- **CanComment**：可评论权限，可查看并添加解读
- **FullAccess**：完全访问权限，包含所有元数据

### 4. 自动授权机制

支持悬赏问答等场景的自动授权：
- 大师接单时自动授权访问权限
- 订单完成后可选择撤销授权
- 支持临时授权和永久授权

## 数据结构设计


### 核心类型定义

```rust
/// 占卜结果引用（通用标识符）
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct DivinationRef {
    /// 占卜类型
    pub divination_type: DivinationType,
    /// 结果ID（卦象ID、命盘ID等）
    pub result_id: u64,
}

/// 加密模式
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum EncryptionMode {
    /// 明文（公开可见）
    Plain = 0,
    /// 单方加密（仅所有者）
    SingleKey = 1,
    /// 多方授权加密
    MultiKey = 2,
}

/// 授权角色
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum AccessRole {
    Owner = 0,      // 所有者（不可撤销）
    Master = 1,     // 命理师（可撤销）
    Family = 2,     // 家族成员（可撤销）
    AiService = 3,  // AI服务（可撤销）
}

/// 访问范围
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum AccessScope {
    ReadOnly = 0,      // 只读
    CanComment = 1,    // 可评论
    FullAccess = 2,    // 完全访问
}

/// 加密密钥条目
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct EncryptedKeyEntry<AccountId> {
    /// 授权账户
    pub account: AccountId,
    /// 用该账户X25519公钥加密的DataKey
    pub encrypted_key: BoundedVec<u8, ConstU32<72>>,
    /// 授权角色
    pub role: AccessRole,
    /// 访问范围
    pub scope: AccessScope,
    /// 授权开始时间
    pub granted_at: BlockNumber,
    /// 授权结束时间（0=永久）
    pub expires_at: BlockNumber,
}

/// 加密占卜数据
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct EncryptedDivinationData<AccountId> {
    /// 占卜结果引用
    pub divination_ref: DivinationRef,
    /// 所有者
    pub owner: AccountId,
    /// 加密模式
    pub mode: EncryptionMode,
    /// AES-256-GCM加密的敏感数据
    pub encrypted_data: BoundedVec<u8, ConstU32<512>>,
    /// 加密nonce
    pub nonce: [u8; 12],
    /// 认证标签
    pub auth_tag: [u8; 16],
    /// 多个加密的DataKey（最多10个授权）
    pub encrypted_keys: BoundedVec<EncryptedKeyEntry<AccountId>, ConstU32<10>>,
    /// 数据哈希（验证解密正确性）
    pub data_hash: [u8; 32],
    /// 创建时间
    pub created_at: BlockNumber,
}
```

## 存储设计

```rust
/// 用户加密公钥（X25519）
pub type UserEncryptionKeys<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<u8, ConstU32<32>>,
>;

/// 加密数据存储（按占卜引用索引）
pub type EncryptedDataByRef<T> = StorageMap<
    _,
    Blake2_128Concat,
    DivinationRef,
    EncryptedDivinationData<T::AccountId>,
>;

/// 用户拥有的加密数据索引
pub type UserEncryptedData<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<DivinationRef, ConstU32<100>>,
    ValueQuery,
>;

/// 用户被授权的数据索引（作为Master等角色）
pub type UserGrantedData<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<DivinationRef, ConstU32<500>>,
    ValueQuery,
>;
```

## 核心接口设计

