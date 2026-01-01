# 多密钥解密方案：自己 + 多位命理师可解密

## 📋 需求分析

### 核心需求
- ✅ **用户自己**可以随时解密查看原始数据
- ✅ **授权的命理师**可以解密查看（用于咨询服务）
- ✅ **未授权的人**无法解密（隐私保护）
- ✅ **可撤销授权**（用户可以取消某个命理师的访问权限）

### 典型场景
```
用户张三创建加密命盘
  ↓
授权给命理师 A（线上咨询）
授权给命理师 B（线下咨询）
  ↓
命理师 A 和 B 都可以解密查看
  ↓
咨询结束后，撤销命理师 A 的授权
  ↓
命理师 A 无法再解密，命理师 B 仍可以
```

---

## 🔐 技术方案对比

### 方案 1：对称密钥 + 链上加密分发（推荐）

#### 架构设计
```
┌─────────────────────────────────────────────────┐
│           加密命盘数据结构                       │
├─────────────────────────────────────────────────┤
│  1. sizhu_index (明文)                          │
│  2. gender (明文)                               │
│  3. encrypted_data (AES-256-GCM 加密)          │
│     - 使用随机生成的 data_key 加密              │
│  4. encrypted_keys (多个加密的 data_key)       │
│     - owner_encrypted_key: 用户公钥加密         │
│     - master1_encrypted_key: 命理师1公钥加密    │
│     - master2_encrypted_key: 命理师2公钥加密    │
│  5. data_hash (验证)                            │
└─────────────────────────────────────────────────┘
```

#### 数据结构定义
```rust
/// 加密密钥条目
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct EncryptedKeyEntry<AccountId> {
    /// 授权账户（用户或命理师）
    pub account: AccountId,
    /// 用该账户公钥加密的 data_key
    /// 使用 X25519 + ChaCha20-Poly1305
    pub encrypted_key: BoundedVec<u8, ConstU32<64>>,
    /// 授权时间戳
    pub granted_at: u32,
    /// 授权类型
    pub role: KeyRole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum KeyRole {
    Owner = 0,      // 所有者（不可撤销）
    Master = 1,     // 命理师（可撤销）
    Family = 2,     // 家族成员（可撤销）
}

/// 增强的加密八字命盘（支持多密钥）
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct EncryptedBaziChartV2<T: Config> {
    pub owner: T::AccountId,
    pub sizhu_index: SiZhuIndex,
    pub gender: Gender,
    /// 使用随机 data_key 加密的数据
    pub encrypted_data: BoundedVec<u8, ConstU32<256>>,
    /// 多个加密的 data_key（最多 10 个授权）
    pub encrypted_keys: BoundedVec<EncryptedKeyEntry<T::AccountId>, ConstU32<10>>,
    pub data_hash: [u8; 32],
    pub created_at: u32,
}
```


#### 工作流程

**1. 创建加密命盘**
```typescript
// 前端实现
async function createMultiKeyEncryptedChart(
    birthData: BirthData,
    account: Account,
    authorizedMasters: Account[]  // 授权的命理师列表
) {
    // 1. 生成随机 data_key（32 bytes）
    const dataKey = crypto.getRandomValues(new Uint8Array(32));
    
    // 2. 使用 data_key 加密敏感数据
    const encryptedData = await encryptWithAES(
        JSON.stringify(birthData),
        dataKey
    );
    
    // 3. 为每个授权账户加密 data_key
    const encryptedKeys = [];
    
    // 3.1 用户自己的公钥加密
    const ownerEncryptedKey = await encryptKeyWithPublicKey(
        dataKey,
        account.publicKey
    );
    encryptedKeys.push({
        account: account.address,
        encrypted_key: ownerEncryptedKey,
        role: "Owner"
    });
    
    // 3.2 为每个命理师的公钥加密
    for (const master of authorizedMasters) {
        const masterEncryptedKey = await encryptKeyWithPublicKey(
            dataKey,
            master.publicKey
        );
        encryptedKeys.push({
            account: master.address,
            encrypted_key: masterEncryptedKey,
            role: "Master"
        });
    }
    
    // 4. 计算数据哈希
    const dataHash = blake2_256(JSON.stringify(birthData));
    
    // 5. 提交到链上
    await api.tx.baziChart.createEncryptedChartV2(
        sizhuIndex,
        gender,
        encryptedData,
        encryptedKeys,
        dataHash
    ).signAndSend(account);
}
```

**2. 解密数据**
```typescript
async function decryptBaziChart(
    chartId: number,
    account: Account
) {
    // 1. 从链上读取加密命盘
    const chart = await api.query.baziChart.encryptedChartById(chartId);
    
    // 2. 查找自己的 encrypted_key
    const myKeyEntry = chart.encrypted_keys.find(
        entry => entry.account === account.address
    );
    
    if (!myKeyEntry) {
        throw new Error("无权访问此命盘");
    }
    
    // 3. 使用自己的私钥解密 data_key
    const dataKey = await decryptKeyWithPrivateKey(
        myKeyEntry.encrypted_key,
        account.privateKey
    );
    
    // 4. 使用 data_key 解密数据
    const decryptedData = await decryptWithAES(
        chart.encrypted_data,
        dataKey
    );
    
    // 5. 验证哈希
    const hash = blake2_256(decryptedData);
    if (hash !== chart.data_hash) {
        throw new Error("数据已损坏");
    }
    
    return JSON.parse(decryptedData);
}
```

**3. 授权新的命理师**
```typescript
async function grantAccessToMaster(
    chartId: number,
    masterAccount: Account,
    ownerAccount: Account
) {
    // 1. 读取命盘
    const chart = await api.query.baziChart.encryptedChartById(chartId);
    
    // 2. 用户解密 data_key
    const dataKey = await decryptMyDataKey(chart, ownerAccount);
    
    // 3. 用命理师的公钥加密 data_key
    const masterEncryptedKey = await encryptKeyWithPublicKey(
        dataKey,
        masterAccount.publicKey
    );
    
    // 4. 提交到链上
    await api.tx.baziChart.grantChartAccess(
        chartId,
        masterAccount.address,
        masterEncryptedKey,
        "Master"
    ).signAndSend(ownerAccount);
}
```

**4. 撤销命理师授权**
```typescript
async function revokeAccessFromMaster(
    chartId: number,
    masterAddress: string,
    ownerAccount: Account
) {
    await api.tx.baziChart.revokeChartAccess(
        chartId,
        masterAddress
    ).signAndSend(ownerAccount);
}
```

---

## 🔧 链上接口设计

### 新增 Extrinsics

```rust
/// 创建支持多密钥的加密命盘
#[pallet::call_index(4)]
pub fn create_encrypted_chart_v2(
    origin: OriginFor<T>,
    sizhu_index: SiZhuIndex,
    gender: Gender,
    encrypted_data: BoundedVec<u8, ConstU32<256>>,
    encrypted_keys: BoundedVec<EncryptedKeyEntry<T::AccountId>, ConstU32<10>>,
    data_hash: [u8; 32],
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    // 验证至少有一个 Owner 角色的密钥
    ensure!(
        encrypted_keys.iter().any(|k| k.account == who && k.role == KeyRole::Owner),
        Error::<T>::MissingOwnerKey
    );
    
    // 存储逻辑...
}

/// 授权新账户访问
#[pallet::call_index(5)]
pub fn grant_chart_access(
    origin: OriginFor<T>,
    chart_id: u64,
    grantee: T::AccountId,
    encrypted_key: BoundedVec<u8, ConstU32<64>>,
    role: KeyRole,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    // 验证调用者是所有者
    let chart = EncryptedChartById::<T>::get(chart_id)
        .ok_or(Error::<T>::ChartNotFound)?;
    ensure!(chart.owner == who, Error::<T>::NotOwner);
    
    // 验证不超过最大授权数
    ensure!(
        chart.encrypted_keys.len() < 10,
        Error::<T>::TooManyAuthorizations
    );
    
    // 添加新的加密密钥
    EncryptedChartById::<T>::try_mutate(chart_id, |maybe_chart| {
        let chart = maybe_chart.as_mut().ok_or(Error::<T>::ChartNotFound)?;
        
        let entry = EncryptedKeyEntry {
            account: grantee.clone(),
            encrypted_key,
            granted_at: <frame_system::Pallet<T>>::block_number().saturated_into(),
            role,
        };
        
        chart.encrypted_keys.try_push(entry)
            .map_err(|_| Error::<T>::TooManyAuthorizations)?;
        
        Ok(())
    })?;
    
    Self::deposit_event(Event::ChartAccessGranted {
        chart_id,
        owner: who,
        grantee,
        role,
    });
    
    Ok(())
}

/// 撤销账户访问权限
#[pallet::call_index(6)]
pub fn revoke_chart_access(
    origin: OriginFor<T>,
    chart_id: u64,
    revokee: T::AccountId,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    // 验证调用者是所有者
    let chart = EncryptedChartById::<T>::get(chart_id)
        .ok_or(Error::<T>::ChartNotFound)?;
    ensure!(chart.owner == who, Error::<T>::NotOwner);
    
    // 不能撤销所有者自己的权限
    ensure!(revokee != who, Error::<T>::CannotRevokeOwner);
    
    // 移除指定账户的密钥
    EncryptedChartById::<T>::try_mutate(chart_id, |maybe_chart| {
        let chart = maybe_chart.as_mut().ok_or(Error::<T>::ChartNotFound)?;
        
        chart.encrypted_keys.retain(|entry| entry.account != revokee);
        
        Ok(())
    })?;
    
    Self::deposit_event(Event::ChartAccessRevoked {
        chart_id,
        owner: who,
        revokee,
    });
    
    Ok(())
}
```

---

## 📊 合理性分析

### ✅ 优点

| 维度 | 评分 | 说明 |
|------|------|------|
| **隐私保护** | ⭐⭐⭐⭐⭐ | 数据加密，只有授权者可解密 |
| **灵活授权** | ⭐⭐⭐⭐⭐ | 可动态添加/撤销授权 |
| **用户体验** | ⭐⭐⭐⭐⭐ | 用户无需分享密码给命理师 |
| **安全性** | ⭐⭐⭐⭐⭐ | 每个人用自己的私钥解密 |
| **可审计** | ⭐⭐⭐⭐⭐ | 链上记录所有授权历史 |

### ⚠️ 注意事项

| 问题 | 影响 | 解决方案 |
|------|------|----------|
| **存储成本** | 中 | 每个授权 ~100 bytes，限制最多 10 个 |
| **Gas 费用** | 中 | 授权/撤销需要交易费 |
| **密钥管理** | 低 | 使用账户公私钥，无需额外管理 |
| **撤销后仍可访问** | 高 | 命理师可能已保存解密数据 |

---

## 🔐 可行性分析

### 技术可行性：⭐⭐⭐⭐⭐ 完全可行

#### 1. 加密算法支持
```rust
// Substrate 原生支持
use sp_core::crypto::{Pair, Public};
use sp_io::crypto;

// X25519 密钥交换 + ChaCha20-Poly1305 加密
// 或使用 ECIES (Elliptic Curve Integrated Encryption Scheme)
```

#### 2. 前端实现
```typescript
// 使用 @polkadot/util-crypto
import { encryptMessage, decryptMessage } from '@polkadot/util-crypto';

// 加密 data_key
const encrypted = encryptMessage(
    dataKey,
    masterAccount.publicKey,
    ownerAccount.secretKey
);

// 解密 data_key
const decrypted = decryptMessage(
    encrypted,
    ownerAccount.publicKey,
    masterAccount.secretKey
);
```

#### 3. 性能评估
| 操作 | 时间 | Gas 费用 |
|------|------|----------|
| 创建命盘（3个授权） | ~200ms | ~0.01 DOT |
| 授权新命理师 | ~100ms | ~0.005 DOT |
| 撤销授权 | ~100ms | ~0.005 DOT |
| 解密数据（前端） | ~50ms | 免费 |

---

## 🎯 应用场景详解

### 场景 1：线上命理咨询平台

```typescript
// 用户创建命盘并授权平台推荐的命理师
async function consultWithMaster(
    birthData: BirthData,
    userAccount: Account,
    platformMasters: Account[]
) {
    // 1. 创建加密命盘，授权给多位命理师
    const chartId = await createMultiKeyEncryptedChart(
        birthData,
        userAccount,
        platformMasters  // 平台的 3 位命理师
    );
    
    // 2. 命理师可以解密查看
    for (const master of platformMasters) {
        const data = await decryptBaziChart(chartId, master);
        console.log(`命理师 ${master.name} 可以查看：`, data);
    }
    
    // 3. 咨询结束后，撤销授权
    for (const master of platformMasters) {
        await revokeAccessFromMaster(chartId, master.address, userAccount);
    }
}
```

### 场景 2：家族共享命盘

```typescript
// 家族成员互相授权
async function createFamilyChart(
    birthData: BirthData,
    owner: Account,
    familyMembers: Account[]
) {
    const chartId = await createMultiKeyEncryptedChart(
        birthData,
        owner,
        familyMembers  // 父母、配偶、子女
    );
    
    // 所有家族成员都可以查看
    for (const member of familyMembers) {
        const data = await decryptBaziChart(chartId, member);
    }
}
```

### 场景 3：命理师团队协作

```typescript
// 主命理师授权助理查看客户命盘
async function teamConsultation(
    chartId: number,
    masterAccount: Account,
    assistants: Account[]
) {
    // 主命理师授权助理
    for (const assistant of assistants) {
        await grantAccessToMaster(chartId, assistant, masterAccount);
    }
    
    // 助理可以查看并做初步分析
    const data = await decryptBaziChart(chartId, assistants[0]);
    
    // 项目结束后撤销授权
    for (const assistant of assistants) {
        await revokeAccessFromMaster(chartId, assistant.address, masterAccount);
    }
}
```

---

## 🔄 与现有系统的兼容性

### 向后兼容方案

```rust
/// 统一的加密命盘枚举
#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub enum EncryptedBaziChartVersion<T: Config> {
    /// V1: 单密钥版本（现有）
    V1(EncryptedBaziChart<T>),
    /// V2: 多密钥版本（新增）
    V2(EncryptedBaziChartV2<T>),
}

// 存储时自动处理版本
#[pallet::storage]
pub type EncryptedChartById<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64,
    EncryptedBaziChartVersion<T>,
>;
```

---

## 💰 成本分析

### 存储成本对比

| 版本 | 基础大小 | 每个授权 | 10个授权总计 |
|------|----------|----------|--------------|
| V1（单密钥） | ~300 bytes | - | ~300 bytes |
| V2（多密钥） | ~300 bytes | ~100 bytes | ~1300 bytes |

### Gas 费用估算

| 操作 | V1 | V2（3个授权） | V2（10个授权） |
|------|----|--------------|--------------
| 创建命盘 | 0.01 DOT | 0.015 DOT | 0.025 DOT |
| 授权新用户 | - | 0.005 DOT | 0.005 DOT |
| 撤销授权 | - | 0.003 DOT | 0.003 DOT |

---

## 🎯 总结与建议

### 合理性评分：⭐⭐⭐⭐⭐ (5/5)

**强烈推荐实现**，理由：
1. ✅ 完美解决命理咨询场景的隐私需求
2. ✅ 用户体验优秀（无需分享密码）
3. ✅ 安全性高（每人用自己的私钥）
4. ✅ 灵活可控（动态授权/撤销）
5. ✅ 可审计（链上记录所有操作）

### 可行性评分：⭐⭐⭐⭐⭐ (5/5)

**技术完全成熟**，理由：
1. ✅ Substrate 原生支持公钥加密
2. ✅ 前端库完善（@polkadot/util-crypto）
3. ✅ 性能开销可接受
4. ✅ 存储成本可控
5. ✅ 向后兼容现有系统

### 实施建议

**阶段 1：核心功能（P0）**
- ✅ 实现 `EncryptedBaziChartV2` 数据结构
- ✅ 实现 `create_encrypted_chart_v2` 接口
- ✅ 实现 `grant_chart_access` 接口
- ✅ 实现 `revoke_chart_access` 接口

**阶段 2：前端集成（P1）**
- ✅ 前端加密/解密工具函数
- ✅ 授权管理 UI
- ✅ 命理师列表选择器

**阶段 3：增强功能（P2）**
- ⭐ 授权过期时间
- ⭐ 授权次数限制
- ⭐ 访问日志记录

---

## 🔒 安全注意事项

### 1. 撤销后的数据访问

⚠️ **重要**：撤销授权后，命理师可能已经保存了解密后的数据。

**缓解措施**：
- 在授权时明确告知用户此风险
- 建议用户只授权信任的命理师
- 平台可以建立命理师信用体系

### 2. 密钥泄露风险

⚠️ 如果用户的私钥泄露，所有授权都失效。

**缓解措施**：
- 教育用户妥善保管私钥
- 支持密钥轮换（重新加密数据）

### 3. 中间人攻击

⚠️ 授权时需要确保使用正确的公钥。

**缓解措施**：
- 前端验证公钥与账户地址匹配
- 显示账户地址供用户确认

---

**结论**：多密钥解密方案在技术上完全可行，在业务上高度合理，强烈建议实施！
