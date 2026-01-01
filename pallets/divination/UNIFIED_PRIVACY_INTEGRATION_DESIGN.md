# 占卜系统统一隐私模式集成设计方案

**版本**: v3.4 (修复问题 + 完善细节)
**日期**: 2025-12-26
**状态**: 设计阶段
**变更**: 修复版本号、统一加密术语、完善授权逻辑、添加密钥备份策略

---

## 📋 执行摘要

本文档提供了将 `pallet-divination-privacy` 的 **PrivacyMode** 统一集成到所有占卜模块的完整技术方案。

### 核心设计目标

1. ✅ **统一隐私框架** - Public/Partial/Private 三级隐私模式
2. ✅ **零侵入式改造** - 保留现有明文结构，新增加密版本
3. ✅ **统一计算方案** - 前端传参 + Runtime API 计算
4. ✅ **零额外服务器** - 无需部署后端服务
5. ✅ **链端更新自动同步** - 前端无需修改

---

## 🎯 三种隐私模式定义

```rust
pub enum PrivacyMode {
    Public = 0,   // 公开 - 所有数据明文存储
    Partial = 1,  // 部分加密 - 计算数据明文 + 敏感数据加密 ⭐推荐
    Private = 2,  // 完全加密 - 所有数据加密
}
```

### 完整对比表

| 特性 | Public | Partial ⭐ | Private |
|------|--------|-----------|---------|
| **敏感数据** | 明文 | 加密 | 加密 |
| **计算数据** | 明文 | 明文 | 加密 |
| **链上存储** | 全部明文 | 计算数据明文 | 全部加密 |
| **计算方式** | Runtime API (chartId) | Runtime API (chartId) | Runtime API (前端传参) |
| **授权支持** | ❌ 无 | ✅ 支持 | ✅ 支持 |
| **存储开销** | 0 | +50B | +50B |
| **推荐场景** | 公开展示 | **奇门遁甲、命运档案** | 高度敏感数据 |

---

## 🔧 统一计算方案：前端传参 + Runtime API

### 核心架构

```
┌─────────────────────────────────────────────────────────────┐
│                    统一计算架构                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   Public/Partial 模式：                                      │
│   ┌─────────┐      ┌─────────┐      ┌─────────────┐        │
│   │  前端   │─(id)─>│  RPC   │─────>│ Runtime API │        │
│   └─────────┘      └─────────┘      └─────────────┘        │
│                                            │                │
│                                     读取链上明文数据         │
│                                            │                │
│                                            ▼                │
│                                     返回计算结果             │
│                                                             │
│   Private 模式：                                             │
│   ┌─────────┐      ┌─────────┐      ┌─────────────┐        │
│   │  前端   │─解密─>│  前端  │─参数─>│ Runtime API │        │
│   └─────────┘      └─────────┘      └─────────────┘        │
│        │                                   │                │
│   用私钥解密                          传入明文参数           │
│   敏感数据                                 │                │
│                                            ▼                │
│                                     返回计算结果             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 方案优势

| 优势 | 说明 |
|------|------|
| ✅ **零服务器成本** | 无需部署后端服务，直接调用 RPC 节点 |
| ✅ **零开发重复** | 复用链上 Runtime 算法，无需前端重写 |
| ✅ **自动同步更新** | 链端算法更新，前端自动生效 |
| ✅ **即时计算** | 无需等待（对比 ZK 需要 10-30 秒） |
| ✅ **开发成本低** | 无需 ZK 电路开发（节省 35 人日） |

### RPC 隐私风险与缓解

**风险**：Private 模式下，前端传参会经过 RPC 节点，存在隐私泄露风险。

**缓解措施**：

| 方案 | 隐私保护 | 成本 | 适用场景 |
|------|---------|------|---------|
| **自建 RPC 节点** | ✅ 完全隐私 | ¥100-500/月 | 企业用户 |
| **信任的 RPC 服务** | ⚠️ 需信任服务商 | ¥0 | 普通用户 |

**推荐**：
- 大多数用户使用 **Partial 模式**（计算数据明文，敏感数据加密）
- 高隐私需求企业用户可自建 RPC 节点

---

## 📦 各模式详细设计

### 1. Public 模式（公开）

**数据流**：
```
创建时：前端 ──(明文数据)──> 链上存储

查询时：前端 ──(chartId)──> Runtime API ──读取──> 链上明文 ──> 返回结果
```

**特点**：
- 所有数据明文存储
- 任何人可查看
- 无需授权

### 2. Partial 模式（部分加密）⭐推荐

**数据流**：
```
创建时：
├── 计算数据（四柱、九宫等）──> 链上明文存储
└── 敏感数据（姓名、问题）──加密──> EncryptedRecords 存储

解盘时：
前端 ──(chartId)──> Runtime API ──读取──> 链上明文计算数据 ──> 返回解盘结果

查看敏感数据时：
前端 ──(私钥解密)──> 显示姓名、问题等
```

**优势**：
- ✅ 免费链上解盘（计算数据明文）
- ✅ 隐私保护（敏感数据加密）
- ✅ 支持多方授权

### 3. Private 模式（完全加密）

**数据流**：
```
创建时：
└── 所有数据 ──加密──> EncryptedRecords 存储（链上仅存加密数据）

解盘时：
1. 前端用私钥解密敏感数据
2. 调用 Runtime API（传入明文参数）
3. Runtime 计算返回结果
4. 结果不存储，仅展示

┌─────────┐     ┌─────────┐     ┌─────────────┐
│  前端   │──解密──>│  传参  │────>│ Runtime API │
└─────────┘     └─────────┘     └─────────────┘
                                      │
                               计算并返回结果
                              （不存储在链上）
```

**注意**：通过公共 RPC 传参存在隐私风险，建议高隐私用户优先选择 Partial 模式，或企业用户自建 RPC 节点。

---

## 🔧 Runtime API 设计

### API 定义

```rust
sp_api::decl_runtime_apis! {
    pub trait QimenApi {
        /// 解盘（Public/Partial 模式 - 读取链上数据）
        fn interpret_chart(chart_id: u64) -> Option<ChartInterpretation>;

        /// 临时排盘（不存储，直接计算）⭐
        /// 用途：
        /// 1. Private 模式解盘（前端解密后调用）
        /// 2. 用户临时查看排盘（不想存储）
        fn compute_chart(
            solar_year: u16,
            solar_month: u8,
            solar_day: u8,
            solar_hour: u8,
            solar_minute: u8,
        ) -> QimenChartResult;

        /// 批量解盘
        fn batch_interpret(chart_ids: Vec<u64>) -> Vec<Option<ChartInterpretation>>;
    }
}
```

### 实现

```rust
impl_runtime_apis! {
    impl qimen_runtime_api::QimenApi<Block> for Runtime {
        fn interpret_chart(chart_id: u64) -> Option<ChartInterpretation> {
            let chart = Qimen::get_chart(chart_id)?;

            match chart.privacy_mode {
                // Public/Partial：计算数据明文可用
                PrivacyMode::Public | PrivacyMode::Partial => {
                    Some(Qimen::do_interpret(&chart))
                },
                // Private：计算数据加密，需使用 compute_chart
                PrivacyMode::Private => None,
            }
        }

        fn compute_chart(
            solar_year: u16,
            solar_month: u8,
            solar_day: u8,
            solar_hour: u8,
            solar_minute: u8,
        ) -> QimenChartResult {
            // 临时排盘：排盘 + 解读，不存储
            Qimen::do_compute_and_interpret(
                solar_year,
                solar_month,
                solar_day,
                solar_hour,
                solar_minute,
            )
        }
    }
}
```

### 临时排盘 API 的优势

| 优势 | 说明 |
|------|------|
| ✅ **代码复用** | 排盘逻辑只实现一次，多场景共用 |
| ✅ **API 精简** | 不需要专门的 `interpret_chart_with_params` |
| ✅ **多场景适用** | Private 解盘 + 临时排盘 + 预览功能 |
| ✅ **输入简单** | 只需公历时间，无需传复杂的干支结构 |

---

## 📱 前端实现

### Partial 模式调用

```typescript
// Partial 模式：直接传 chartId，链上有明文计算数据
async function interpretPartialChart(chartId: number) {
  const api = await getApi();

  // 调用 Runtime API（免费，无需签名）
  const interpretation = await api.call.qimenApi.interpretChart(chartId);

  // 解密敏感数据（可选，用于显示姓名、问题）
  const encryptedRecord = await api.query.privacy.encryptedRecords(
    DivinationType.Qimen,
    chartId
  );
  const sensitiveData = await decryptWithPrivateKey(encryptedRecord, privateKey);

  return {
    interpretation: interpretation.toJSON(),
    name: sensitiveData.name,
    question: sensitiveData.question,
  };
}
```

### Private 模式调用

```typescript
// Private 模式：前端解密后调用临时排盘 API
async function interpretPrivateChart(chartId: number, privateKey: Uint8Array) {
  const api = await getApi();

  // 1. 获取加密数据
  const encryptedRecord = await api.query.privacy.encryptedRecords(
    DivinationType.Qimen,
    chartId
  );

  // 2. 前端解密（使用用户私钥）
  const decryptedData = await decryptWithPrivateKey(encryptedRecord, privateKey);

  // 3. 调用临时排盘 API（只需传公历时间）⭐
  const chartResult = await api.call.qimenApi.computeChart(
    decryptedData.solarYear,
    decryptedData.solarMonth,
    decryptedData.solarDay,
    decryptedData.solarHour,
    decryptedData.solarMinute,
  );

  return {
    chart: chartResult.toJSON(),
    name: decryptedData.name,
    question: decryptedData.question,
  };
}
```

---

## 🔐 Privacy Pallet 集成详解

本节详细说明如何与 `pallet-divination-privacy` 模块集成，包括完整的 API 调用流程和前端实现示例。

### 集成架构图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    Privacy Pallet 集成架构                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   ┌──────────────┐                                                      │
│   │   用户注册   │                                                      │
│   └──────┬───────┘                                                      │
│          │                                                              │
│          ▼                                                              │
│   ┌──────────────────────────────────────────────────────────┐         │
│   │  1. register_encryption_key(public_key)                  │         │
│   │     - 前端生成 X25519 密钥对                              │         │
│   │     - 私钥安全存储在本地（浏览器/钱包）                    │         │
│   │     - 公钥上链，用于多方加密                              │         │
│   └──────────────────────────────────────────────────────────┘         │
│                                                                         │
│   ┌──────────────┐                                                      │
│   │   创建占卜   │                                                      │
│   └──────┬───────┘                                                      │
│          │                                                              │
│          ▼                                                              │
│   ┌──────────────────────────────────────────────────────────┐         │
│   │  2. create_encrypted_record(...)                         │         │
│   │     - 前端生成随机 DataKey                                │         │
│   │     - XChaCha20-Poly1305 加密敏感数据                  │         │
│   │     - DataKey 用所有者公钥加密                            │         │
│   │     - 加密数据 + 加密密钥上链                             │         │
│   └──────────────────────────────────────────────────────────┘         │
│                                                                         │
│   ┌──────────────┐                                                      │
│   │   授权访问   │                                                      │
│   └──────┬───────┘                                                      │
│          │                                                              │
│          ▼                                                              │
│   ┌──────────────────────────────────────────────────────────┐         │
│   │  3. grant_access(grantee, encrypted_key, role, scope)    │         │
│   │     - 所有者解密 DataKey                                  │         │
│   │     - 用被授权者公钥重新加密 DataKey                       │         │
│   │     - 加密后的密钥上链                                    │         │
│   └──────────────────────────────────────────────────────────┘         │
│                                                                         │
│   ┌──────────────┐                                                      │
│   │   访问数据   │                                                      │
│   └──────┬───────┘                                                      │
│          │                                                              │
│          ▼                                                              │
│   ┌──────────────────────────────────────────────────────────┐         │
│   │  4. 查询链上数据 + 本地解密                               │         │
│   │     - 查询 EncryptedRecords 获取加密数据                  │         │
│   │     - 查询 Authorizations 获取加密的 DataKey              │         │
│   │     - 用私钥解密 DataKey                                  │         │
│   │     - 用 DataKey 解密敏感数据                             │         │
│   └──────────────────────────────────────────────────────────┘         │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Step 1: 密钥管理

用户首次使用加密功能前，需注册加密公钥。

```typescript
import { xchacha20poly1305 } from '@noble/ciphers/chacha';
import { x25519 } from '@noble/curves/ed25519';
import { randomBytes } from '@noble/ciphers/webcrypto';

/**
 * 加密密钥管理服务
 */
export class EncryptionKeyService {
  private static readonly STORAGE_KEY = 'stardust_encryption_keypair';

  /**
   * 生成或获取用户的 X25519 密钥对
   * 私钥安全存储在本地，公钥上链
   */
  static async getOrCreateKeyPair(): Promise<{
    privateKey: Uint8Array;
    publicKey: Uint8Array;
  }> {
    // 尝试从本地存储加载
    const stored = localStorage.getItem(this.STORAGE_KEY);
    if (stored) {
      const { privateKey } = JSON.parse(stored);
      const privKeyBytes = new Uint8Array(Object.values(privateKey));
      return {
        privateKey: privKeyBytes,
        publicKey: x25519.getPublicKey(privKeyBytes),
      };
    }

    // 生成新密钥对
    const privateKey = randomBytes(32);
    const publicKey = x25519.getPublicKey(privateKey);

    // 安全存储私钥（生产环境应使用更安全的存储方式）
    localStorage.setItem(this.STORAGE_KEY, JSON.stringify({
      privateKey: Array.from(privateKey),
    }));

    return { privateKey, publicKey };
  }

  /**
   * 注册加密公钥到链上
   */
  static async registerEncryptionKey(
    api: ApiPromise,
    signer: KeyringPair
  ): Promise<void> {
    const { publicKey } = await this.getOrCreateKeyPair();

    // 检查是否已注册
    const existing = await api.query.privacy.userEncryptionKeys(signer.address);
    if (existing.isSome) {
      console.log('加密公钥已注册');
      return;
    }

    // 注册公钥
    await api.tx.privacy
      .registerEncryptionKey(Array.from(publicKey))
      .signAndSend(signer);

    console.log('加密公钥注册成功');
  }

  /**
   * 更新加密公钥（密钥轮换）
   */
  static async updateEncryptionKey(
    api: ApiPromise,
    signer: KeyringPair
  ): Promise<void> {
    // 生成新密钥对
    const privateKey = randomBytes(32);
    const publicKey = x25519.getPublicKey(privateKey);

    // 更新链上公钥
    await api.tx.privacy
      .updateEncryptionKey(Array.from(publicKey))
      .signAndSend(signer);

    // 更新本地存储
    localStorage.setItem(this.STORAGE_KEY, JSON.stringify({
      privateKey: Array.from(privateKey),
    }));

    console.log('加密公钥更新成功');
  }

  /**
   * 导出密钥备份（用于跨设备恢复）
   * 返回加密后的备份数据，需用户提供密码保护
   */
  static async exportKeyBackup(password: string): Promise<string> {
    const { privateKey } = await this.getOrCreateKeyPair();

    // 使用密码派生加密密钥
    const salt = randomBytes(16);
    const passwordKey = await deriveKeyFromPassword(password, salt);

    // 加密私钥
    const nonce = randomBytes(24);
    const cipher = xchacha20poly1305(passwordKey, nonce);
    const encryptedPrivKey = cipher.encrypt(privateKey);

    // 组装备份数据
    const backup = {
      version: 1,
      salt: Array.from(salt),
      nonce: Array.from(nonce),
      encryptedKey: Array.from(encryptedPrivKey),
    };

    return btoa(JSON.stringify(backup));
  }

  /**
   * 从备份恢复密钥
   */
  static async importKeyBackup(backupString: string, password: string): Promise<void> {
    const backup = JSON.parse(atob(backupString));

    // 派生解密密钥
    const salt = new Uint8Array(backup.salt);
    const passwordKey = await deriveKeyFromPassword(password, salt);

    // 解密私钥
    const nonce = new Uint8Array(backup.nonce);
    const encryptedKey = new Uint8Array(backup.encryptedKey);
    const cipher = xchacha20poly1305(passwordKey, nonce);
    const privateKey = cipher.decrypt(encryptedKey);

    // 保存到本地存储
    localStorage.setItem(this.STORAGE_KEY, JSON.stringify({
      privateKey: Array.from(privateKey),
    }));

    console.log('密钥恢复成功');
  }
}

/**
 * 从密码派生加密密钥（使用 PBKDF2）
 */
async function deriveKeyFromPassword(password: string, salt: Uint8Array): Promise<Uint8Array> {
  const encoder = new TextEncoder();
  const keyMaterial = await crypto.subtle.importKey(
    'raw',
    encoder.encode(password),
    'PBKDF2',
    false,
    ['deriveBits']
  );

  const derivedBits = await crypto.subtle.deriveBits(
    {
      name: 'PBKDF2',
      salt,
      iterations: 100000,
      hash: 'SHA-256',
    },
    keyMaterial,
    256
  );

  return new Uint8Array(derivedBits);
}
```

### ⚠️ 密钥安全与备份策略

**重要提醒**：加密密钥是访问您所有加密数据的唯一凭证，丢失将导致数据永久无法恢复。

#### 密钥存储风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| localStorage 清除 | 私钥丢失，数据无法解密 | 定期导出备份 |
| 设备丢失/损坏 | 同上 | 跨设备备份 |
| 浏览器更新/重置 | 同上 | 使用密码保护的导出功能 |

#### 推荐备份方案

**方案 1：密码保护备份（推荐普通用户）**

```typescript
// 导出备份
const backup = await EncryptionKeyService.exportKeyBackup('your-strong-password');
// 将 backup 字符串保存到安全位置（如密码管理器、云笔记等）

// 恢复备份
await EncryptionKeyService.importKeyBackup(backup, 'your-strong-password');
```

**方案 2：助记词派生（高级用户）**

```typescript
import { mnemonicToSeedSync } from '@scure/bip39';

// 从钱包助记词派生加密密钥（使用不同路径避免与签名密钥冲突）
const seed = mnemonicToSeedSync(mnemonic);
const encryptionKey = sha256(new Uint8Array([...seed, ...Buffer.from('stardust-encryption')]));
```

**方案 3：硬件钱包集成（企业用户）**

- 使用 Ledger/Trezor 存储加密密钥
- 需要硬件确认才能解密

#### 密钥轮换建议

1. **定期轮换**：建议每 6-12 个月更新密钥
2. **轮换步骤**：
   - 导出所有加密数据的密钥包备份
   - 调用 `updateEncryptionKey()` 生成新密钥
   - 旧数据仍可用旧密钥解密

### Step 2: 创建加密记录

创建 Partial/Private 模式的占卜记录时，需同时创建加密记录。

```typescript
import { xchacha20poly1305 } from '@noble/ciphers/chacha';
import { x25519 } from '@noble/curves/ed25519';
import { randomBytes } from '@noble/ciphers/webcrypto';
import { sha256 } from '@noble/hashes/sha256';

/**
 * 占卜加密服务
 */
export class DivinationEncryptionService {
  /**
   * 加密敏感数据并创建链上记录
   *
   * @param api Polkadot API 实例
   * @param signer 签名账户
   * @param divinationType 占卜类型（如 DivinationType.Qimen）
   * @param resultId 占卜结果 ID
   * @param privacyMode 隐私模式（Partial 或 Private）
   * @param sensitiveData 待加密的敏感数据
   * @param encryptedFields 加密字段标志位（仅 Partial 模式）
   * @returns 所有者的加密密钥包（用于后续授权操作，需本地安全保存）
   */
  static async createEncryptedRecord(
    api: ApiPromise,
    signer: KeyringPair,
    divinationType: number,
    resultId: number,
    privacyMode: 'Partial' | 'Private',
    sensitiveData: object,
    encryptedFields?: number
  ): Promise<Uint8Array> {  // 返回加密密钥包
    // 1. 获取用户密钥对
    const { privateKey, publicKey } = await EncryptionKeyService.getOrCreateKeyPair();

    // 2. 生成随机 DataKey（用于对称加密）
    const dataKey = randomBytes(32);
    const nonce = randomBytes(24);

    // 3. 加密敏感数据
    const plaintext = new TextEncoder().encode(JSON.stringify(sensitiveData));
    const cipher = xchacha20poly1305(dataKey, nonce);
    const ciphertext = cipher.encrypt(plaintext);

    // 分离密文和认证标签（最后 16 字节）
    const encryptedData = ciphertext.slice(0, -16);
    const authTag = ciphertext.slice(-16);

    // 4. 计算数据哈希（用于完整性验证）
    const dataHash = sha256(plaintext);

    // 5. 用所有者公钥加密 DataKey
    const ephemeralPrivKey = randomBytes(32);
    const ephemeralPubKey = x25519.getPublicKey(ephemeralPrivKey);
    const sharedSecret = x25519.getSharedSecret(ephemeralPrivKey, publicKey);
    const keyNonce = randomBytes(24);
    const keyCipher = xchacha20poly1305(sharedSecret, keyNonce);
    const encryptedKey = keyCipher.encrypt(dataKey);

    // 组装加密密钥包（临时公钥 + nonce + 加密后的 DataKey）
    const ownerEncryptedKey = new Uint8Array([
      ...ephemeralPubKey,
      ...keyNonce,
      ...encryptedKey,
    ]);

    // 6. 创建链上加密记录
    await api.tx.privacy
      .createEncryptedRecord(
        divinationType,
        resultId,
        privacyMode === 'Partial' ? 1 : 2,  // PrivacyMode 枚举值
        Array.from(encryptedData),
        Array.from(nonce),
        Array.from(authTag),
        Array.from(dataHash),
        Array.from(ownerEncryptedKey),
        encryptedFields ?? null
      )
      .signAndSend(signer);

    console.log('加密记录创建成功');

    // 返回加密密钥包，用于后续授权操作
    // ⚠️ 重要：此密钥包需本地安全保存，用于解密数据和授权他人
    return ownerEncryptedKey;
  }

  /**
   * 解密敏感数据
   *
   * @param encryptedRecord 链上加密记录
   * @param encryptedKey 加密的 DataKey（创建时保存的或从授权获取的）
   * @param privateKey 用户私钥
   */
  static async decryptSensitiveData(
    encryptedRecord: {
      encryptedData: Uint8Array;
      nonce: Uint8Array;
      authTag: Uint8Array;
    },
    encryptedKey: Uint8Array,
    privateKey: Uint8Array
  ): Promise<object> {
    // 1. 解析加密密钥包
    const ephemeralPubKey = encryptedKey.slice(0, 32);
    const keyNonce = encryptedKey.slice(32, 56);
    const encryptedDataKey = encryptedKey.slice(56);

    // 2. 恢复共享密钥并解密 DataKey
    const sharedSecret = x25519.getSharedSecret(privateKey, ephemeralPubKey);
    const keyCipher = xchacha20poly1305(sharedSecret, keyNonce);
    const dataKey = keyCipher.decrypt(encryptedDataKey);

    // 3. 解密敏感数据
    const ciphertext = new Uint8Array([
      ...encryptedRecord.encryptedData,
      ...encryptedRecord.authTag,
    ]);
    const dataCipher = xchacha20poly1305(dataKey, encryptedRecord.nonce);
    const plaintext = dataCipher.decrypt(ciphertext);

    return JSON.parse(new TextDecoder().decode(plaintext));
  }
}
```

### Step 3: 授权管理

所有者可授权他人（咨询师、家人、AI 服务）访问加密数据。

```typescript
/**
 * 授权管理服务
 */
export class AuthorizationService {
  /**
   * 授权角色枚举
   */
  static readonly AccessRole = {
    Owner: 0,
    Master: 1,        // 命理师
    Family: 2,        // 家族成员
    AiService: 3,     // AI 服务
    BountyAnswerer: 4 // 悬赏回答者
  };

  /**
   * 访问范围枚举
   */
  static readonly AccessScope = {
    ReadOnly: 0,      // 只读
    CanComment: 1,    // 可评论
    FullAccess: 2     // 完全访问
  };

  /**
   * 授权他人访问加密数据
   *
   * @param api Polkadot API 实例
   * @param signer 签名账户（必须是所有者）
   * @param divinationType 占卜类型
   * @param resultId 占卜结果 ID
   * @param granteeAddress 被授权者地址
   * @param role 授权角色
   * @param scope 访问范围
   * @param expiresAt 过期区块号（0 表示永久）
   * @param ownerEncryptedKey 所有者的加密 DataKey（创建记录时本地保存）
   */
  static async grantAccess(
    api: ApiPromise,
    signer: KeyringPair,
    divinationType: number,
    resultId: number,
    granteeAddress: string,
    role: number,
    scope: number,
    expiresAt: number = 0,
    ownerEncryptedKey: Uint8Array  // 创建记录时本地保存的加密密钥
  ): Promise<void> {
    // 1. 获取所有者的私钥
    const { privateKey } = await EncryptionKeyService.getOrCreateKeyPair();

    // 2. 解密 DataKey（使用创建时保存的加密密钥）
    const ephemeralPubKey = ownerEncryptedKey.slice(0, 32);
    const keyNonce = ownerEncryptedKey.slice(32, 56);
    const encryptedDataKey = ownerEncryptedKey.slice(56);

    const sharedSecret = x25519.getSharedSecret(privateKey, ephemeralPubKey);
    const keyCipher = xchacha20poly1305(sharedSecret, keyNonce);
    const dataKey = keyCipher.decrypt(encryptedDataKey);

    // 3. 获取被授权者的公钥
    const granteeKeyInfo = await api.query.privacy.userEncryptionKeys(granteeAddress);
    if (granteeKeyInfo.isNone) {
      throw new Error('被授权者尚未注册加密公钥');
    }
    const granteePublicKey = new Uint8Array(granteeKeyInfo.unwrap().publicKey);

    // 4. 用被授权者公钥加密 DataKey
    const newEphemeralPrivKey = randomBytes(32);
    const newEphemeralPubKey = x25519.getPublicKey(newEphemeralPrivKey);
    const newSharedSecret = x25519.getSharedSecret(newEphemeralPrivKey, granteePublicKey);
    const newKeyNonce = randomBytes(24);
    const newKeyCipher = xchacha20poly1305(newSharedSecret, newKeyNonce);
    const granteeEncryptedKey = newKeyCipher.encrypt(dataKey);

    // 组装加密密钥包
    const encryptedKeyForGrantee = new Uint8Array([
      ...newEphemeralPubKey,
      ...newKeyNonce,
      ...granteeEncryptedKey,
    ]);

    // 5. 提交授权交易
    await api.tx.privacy
      .grantAccess(
        divinationType,
        resultId,
        granteeAddress,
        Array.from(encryptedKeyForGrantee),
        role,
        scope,
        expiresAt
      )
      .signAndSend(signer);

    console.log(`已授权 ${granteeAddress} 访问数据`);
  }

  /**
   * 撤销授权
   */
  static async revokeAccess(
    api: ApiPromise,
    signer: KeyringPair,
    divinationType: number,
    resultId: number,
    granteeAddress: string
  ): Promise<void> {
    await api.tx.privacy
      .revokeAccess(divinationType, resultId, granteeAddress)
      .signAndSend(signer);

    console.log(`已撤销 ${granteeAddress} 的访问权限`);
  }

  /**
   * 查询授权列表
   */
  static async listAuthorizations(
    api: ApiPromise,
    divinationType: number,
    resultId: number
  ): Promise<Array<{
    grantee: string;
    role: number;
    scope: number;
    grantedAt: number;
    expiresAt: number;
  }>> {
    const authorizations = await api.query.privacy.authorizations(
      divinationType,
      resultId
    );

    return authorizations.map((auth: any) => ({
      grantee: auth.grantee.toString(),
      role: auth.role.toNumber(),
      scope: auth.scope.toNumber(),
      grantedAt: auth.grantedAt.toNumber(),
      expiresAt: auth.expiresAt.toNumber(),
    }));
  }
}
```

### 完整工作流示例

```typescript
import { randomBytes } from '@noble/ciphers/webcrypto';
import { xchacha20poly1305 } from '@noble/ciphers/chacha';
import { x25519 } from '@noble/curves/ed25519';
import { sha256 } from '@noble/hashes/sha256';

/**
 * 本地加密数据存储接口
 * 用于保存加密密钥包，支持后续授权和解密操作
 */
interface LocalEncryptedDataStore {
  chartId: number;
  divinationType: number;
  ownerEncryptedKey: Uint8Array;
  createdAt: number;
}

/**
 * 完整的 Partial 模式创建和授权流程
 * 使用 batchAll 原子化执行，确保数据一致性
 */
async function createPartialQimenChart(
  api: ApiPromise,
  signer: KeyringPair,
  chartData: {
    name: string;
    question: string;
    solarYear: number;
    solarMonth: number;
    solarDay: number;
    solarHour: number;
    solarMinute: number;
  }
): Promise<{ chartId: number; ownerEncryptedKey: Uint8Array }> {
  // Step 1: 确保已注册加密公钥
  await EncryptionKeyService.registerEncryptionKey(api, signer);

  // Step 2: 准备加密数据
  const { publicKey } = await EncryptionKeyService.getOrCreateKeyPair();

  const sensitiveData = {
    name: chartData.name,
    question: chartData.question,
  };

  // 生成 DataKey 和加密数据
  const dataKey = randomBytes(32);
  const nonce = randomBytes(24);
  const plaintext = new TextEncoder().encode(JSON.stringify(sensitiveData));
  const cipher = xchacha20poly1305(dataKey, nonce);
  const ciphertext = cipher.encrypt(plaintext);
  const encryptedData = ciphertext.slice(0, -16);
  const authTag = ciphertext.slice(-16);
  const dataHash = sha256(plaintext);

  // 用所有者公钥加密 DataKey
  const ephemeralPrivKey = randomBytes(32);
  const ephemeralPubKey = x25519.getPublicKey(ephemeralPrivKey);
  const sharedSecret = x25519.getSharedSecret(ephemeralPrivKey, publicKey);
  const keyNonce = randomBytes(24);
  const keyCipher = xchacha20poly1305(sharedSecret, keyNonce);
  const encryptedKey = keyCipher.encrypt(dataKey);
  const ownerEncryptedKey = new Uint8Array([
    ...ephemeralPubKey,
    ...keyNonce,
    ...encryptedKey,
  ]);

  // Step 3: 使用 batchAll 原子化执行两个交易
  // 预估 chartId（基于当前 NextChartId）
  const nextChartId = (await api.query.qimen.nextChartId()).toNumber();

  const batch = api.tx.utility.batchAll([
    // 交易 1: 创建占卜记录（计算数据明文存储）
    api.tx.qimen.createChartEncrypted(
      chartData.solarYear,
      chartData.solarMonth,
      chartData.solarDay,
      chartData.solarHour,
      chartData.solarMinute,
      1  // PrivacyMode::Partial
    ),
    // 交易 2: 创建加密记录（敏感数据加密存储）
    api.tx.privacy.createEncryptedRecord(
      0,  // DivinationType::Qimen
      nextChartId,
      1,  // PrivacyMode::Partial
      Array.from(encryptedData),
      Array.from(nonce),
      Array.from(authTag),
      Array.from(dataHash),
      Array.from(ownerEncryptedKey),
      0x0003  // EncryptedFields::NAME | EncryptedFields::QUESTION
    ),
  ]);

  await batch.signAndSend(signer);

  // Step 4: 保存加密密钥包到本地存储（用于后续授权和解密）
  const localStore: LocalEncryptedDataStore = {
    chartId: nextChartId,
    divinationType: 0,
    ownerEncryptedKey,
    createdAt: Date.now(),
  };
  saveToLocalStorage(`encrypted_key_${nextChartId}`, localStore);

  console.log(`占卜记录创建成功，chartId: ${nextChartId}`);

  return { chartId: nextChartId, ownerEncryptedKey };
}

/**
 * 授权命理师访问（使用保存的加密密钥包）
 */
async function authorizeMaster(
  api: ApiPromise,
  signer: KeyringPair,
  chartId: number,
  masterAddress: string
): Promise<void> {
  // 从本地存储获取加密密钥包
  const localStore = loadFromLocalStorage(`encrypted_key_${chartId}`);
  if (!localStore) {
    throw new Error('未找到加密密钥包，请确认您是该记录的所有者');
  }

  await AuthorizationService.grantAccess(
    api,
    signer,
    0,  // DivinationType::Qimen
    chartId,
    masterAddress,
    AuthorizationService.AccessRole.Master,
    AuthorizationService.AccessScope.CanComment,
    0,  // 永久授权
    new Uint8Array(localStore.ownerEncryptedKey)
  );

  console.log(`已授权 ${masterAddress} 访问 chartId: ${chartId}`);
}

// 辅助函数：本地存储
function saveToLocalStorage(key: string, data: LocalEncryptedDataStore): void {
  localStorage.setItem(key, JSON.stringify({
    ...data,
    ownerEncryptedKey: Array.from(data.ownerEncryptedKey),
  }));
}

function loadFromLocalStorage(key: string): LocalEncryptedDataStore | null {
  const stored = localStorage.getItem(key);
  if (!stored) return null;
  const data = JSON.parse(stored);
  return {
    ...data,
    ownerEncryptedKey: new Uint8Array(data.ownerEncryptedKey),
  };
}
```

### Privacy Pallet 存储结构

```rust
// pallet-divination-privacy 存储项

/// 用户加密公钥
#[pallet::storage]
pub type UserEncryptionKeys<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    UserEncryptionInfo<BlockNumberFor<T>>,
>;

/// 加密记录（按占卜类型和结果ID索引）
#[pallet::storage]
pub type EncryptedRecords<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    DivinationType,
    Blake2_128Concat,
    u64,  // result_id
    EncryptedRecord<T::AccountId, BlockNumberFor<T>, T::MaxDataLen>,
>;

/// 授权列表（按占卜类型和结果ID索引）
#[pallet::storage]
pub type Authorizations<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    DivinationType,
    Blake2_128Concat,
    u64,  // result_id
    BoundedVec<AuthorizationEntry<T::AccountId, BlockNumberFor<T>, T::MaxKeyLen>, T::MaxAuthorizations>,
>;
```

---

## 📊 数据结构修改

### QimenChart 结构

```rust
pub struct QimenChart<AccountId, BlockNumber, MaxCidLen: Get<u32>> {
    pub id: u64,
    pub diviner: AccountId,

    // ==================== 隐私字段 ====================
    pub privacy_mode: PrivacyMode,
    pub encrypted_fields: Option<u16>,
    pub sensitive_data_hash: Option<[u8; 32]>,

    // ==================== 敏感数据（Partial/Private 时为 None）====================
    pub name: Option<BoundedVec<u8, MaxNameLen>>,
    pub gender: Option<Gender>,
    pub birth_year: Option<u16>,
    pub question: Option<BoundedVec<u8, MaxQuestionLen>>,
    pub question_hash: [u8; 32],

    // ==================== 计算数据 ====================
    // Public/Partial：明文存储
    // Private：为 None（加密存储在 EncryptedRecords）
    pub year_ganzhi: Option<GanZhi>,
    pub month_ganzhi: Option<GanZhi>,
    pub day_ganzhi: Option<GanZhi>,
    pub hour_ganzhi: Option<GanZhi>,
    pub jie_qi: Option<JieQi>,
    pub dun_type: Option<DunType>,
    pub ju_number: Option<u8>,
    pub palaces: Option<[Palace; 9]>,

    // ==================== 元数据 ====================
    pub created_at: BlockNumber,
    pub timestamp: u64,
}
```

### 存储对比

| 模式 | 链上 QimenChart | EncryptedRecords |
|------|----------------|------------------|
| **Public** | 全部字段明文 | 无 |
| **Partial** | 计算数据明文，敏感数据=None | 敏感数据加密 |
| **Private** | 全部=None（仅保留id和元数据） | 全部数据加密 |

### Private 模式加密数据结构

Private 模式下，所有数据加密存储在 `EncryptedRecords` 中。`encrypted_data` 字段的解密后结构如下：

```rust
/// Private 模式加密数据内容
///
/// 存储在 EncryptedRecord.encrypted_data 中（XChaCha20-Poly1305 加密）
/// 前端解密后用于调用 compute_chart API
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivateEncryptedData {
    // ==================== 敏感数据 ====================
    /// 命主姓名
    pub name: Option<String>,
    /// 占问事宜
    pub question: Option<String>,
    /// 性别
    pub gender: Option<u8>,
    /// 出生年份
    pub birth_year: Option<u16>,

    // ==================== 计算所需数据（Private 模式专用）====================
    /// 公历年份（用于 compute_chart API）
    pub solar_year: u16,
    /// 公历月份（1-12）
    pub solar_month: u8,
    /// 公历日期（1-31）
    pub solar_day: u8,
    /// 公历小时（0-23）
    pub solar_hour: u8,
    /// 公历分钟（0-59）
    pub solar_minute: u8,
}
```

**前端使用流程**：

```typescript
// 1. 获取加密记录
const encryptedRecord = await api.query.privacy.encryptedRecords(DivinationType.Qimen, chartId);

// 2. 解密获得 PrivateEncryptedData
const data: PrivateEncryptedData = await decryptWithPrivateKey(encryptedRecord, privateKey);

// 3. 调用 compute_chart API
const result = await api.call.qimenApi.computeChart(
  data.solar_year,
  data.solar_month,
  data.solar_day,
  data.solar_hour,
  data.solar_minute,
);
```

### QimenChartResult 类型定义

`compute_chart` API 的返回类型，包含完整的排盘和解读结果：

```rust
/// 临时排盘结果
///
/// compute_chart API 的返回类型
/// 不含敏感数据（姓名、问题等），仅包含计算结果
#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct QimenChartResult {
    // ==================== 四柱 ====================
    /// 年柱
    pub year_ganzhi: GanZhi,
    /// 月柱
    pub month_ganzhi: GanZhi,
    /// 日柱
    pub day_ganzhi: GanZhi,
    /// 时柱
    pub hour_ganzhi: GanZhi,

    // ==================== 局数信息 ====================
    /// 节气
    pub jie_qi: JieQi,
    /// 阴阳遁
    pub dun_type: DunType,
    /// 三元
    pub san_yuan: SanYuan,
    /// 局数（1-9）
    pub ju_number: u8,

    // ==================== 盘面数据 ====================
    /// 值符星
    pub zhi_fu_xing: JiuXing,
    /// 值使门
    pub zhi_shi_men: BaMen,
    /// 九宫排盘结果
    pub palaces: [Palace; 9],

    // ==================== 解读结果（可选）====================
    /// 格局分析
    pub ge_ju: Option<GeJuType>,
    /// 综合吉凶
    pub fortune: Option<Fortune>,
    /// 用神得力状态
    pub yong_shen_status: Option<DeLiStatus>,
}
```

**存储大小估算**：
- 四柱：8 bytes（4 × 2 bytes）
- 局数信息：5 bytes
- 盘面数据：约 180 bytes（9 宫 × 20 bytes）
- 解读结果：约 10 bytes
- **总计**：约 **200 bytes**

---

## 📈 实施路线图

### Phase 1: 核心改造（2 周）

**模块**: Qimen, Ziwei

**任务**：
1. 添加 `privacy_mode`, `encrypted_fields` 字段
2. 敏感/计算字段改为 `Option`
3. 新增 `create_chart_encrypted` 接口
4. 实现 Runtime API（含传参版本）
5. 前端加密/解密服务

**工作量**: 15 人日

### Phase 2: 其他模块（2 周）

**模块**: Liuyao, Xiaoliuren, Daliuren, Meihua

**任务**：
1. 同 Phase 1 改造模式
2. 迁移 IPFS 问题存储到 EncryptedRecords

**工作量**: 17 人日

### Phase 3: 收尾（1 周）

**模块**: Tarot, 前端优化

**任务**：
1. Tarot 替换 is_public 为 privacy_mode
2. 前端 UI 组件完善
3. 测试和文档

**工作量**: 6 人日

### 总计

| 阶段 | 工作量 | 累计 |
|------|--------|------|
| Phase 1 | 15 人日 | 15 人日 |
| Phase 2 | 17 人日 | 32 人日 |
| Phase 3 | 6 人日 | **38 人日** |

**对比**: 原 ZK 方案需要 95+ 人日，节省 **57 人日（60%）**

---

## ⚖️ 可行性评估

### 技术可行性 ⭐⭐⭐⭐⭐

| 维度 | 评分 | 说明 |
|------|------|------|
| **架构简洁性** | ⭐⭐⭐⭐⭐ | 复用现有 Runtime，无需新增复杂组件 |
| **开发成本** | ⭐⭐⭐⭐⭐ | 38 人日（对比 ZK 95+ 人日） |
| **维护成本** | ⭐⭐⭐⭐⭐ | 单套代码，链端更新自动同步 |
| **服务器成本** | ⭐⭐⭐⭐⭐ | ¥0（无需额外服务器） |
| **用户体验** | ⭐⭐⭐⭐⭐ | 即时计算，无等待 |

### 隐私保护评估

| 模式 | 隐私级别 | 说明 |
|------|---------|------|
| **Public** | ❌ 无 | 全部明文 |
| **Partial** ⭐ | ⭐⭐⭐⭐ | 敏感数据加密，计算数据公开（推荐大多数用户） |
| **Private + 公共RPC** | ⭐⭐⭐ | RPC节点可见明文参数 |
| **Private + 自建RPC** | ⭐⭐⭐⭐⭐ | 完全隐私（企业用户推荐） |

---

## 🔒 安全性分析

### Partial 模式安全性

**保护的数据**：
- ✅ 姓名
- ✅ 问题文本
- ✅ 性别、出生年份

**公开的数据**：
- ⚠️ 四柱干支（可反推出生时间范围）
- ⚠️ 九宫排盘数据

**适用场景**：接受计算数据公开，但需保护个人身份信息

### Private 模式安全性

**使用公共 RPC**：
- ⚠️ RPC 节点可记录传入的明文参数
- ⚠️ 需要信任 RPC 服务提供商
- 💡 建议：优先考虑使用 Partial 模式

**使用自建 RPC**：
- ✅ 数据完全在可控环境内处理
- ✅ 不经过任何第三方
- ✅ 最高隐私保护级别
- 💰 成本：¥100-500/月

---

## 📚 附录

### A. 加密字段定义

```rust
#[allow(non_snake_case)]
pub mod EncryptedFields {
    pub const NAME: u16           = 0b0000_0000_0000_0001;
    pub const QUESTION: u16       = 0b0000_0000_0000_0010;
    pub const SOLAR_DATE: u16     = 0b0000_0000_0000_0100;
    pub const SOLAR_TIME: u16     = 0b0000_0000_0000_1000;
    pub const NOTES: u16          = 0b0000_0000_0001_0000;
    pub const BIRTH_YEAR: u16     = 0b0000_0000_0010_0000;
    pub const GENDER: u16         = 0b0000_0000_0100_0000;

    // 推荐配置
    pub const QIMEN_RECOMMENDED: u16 = NAME | QUESTION;
    pub const ALL: u16 = NAME | QUESTION | SOLAR_DATE | SOLAR_TIME | NOTES | BIRTH_YEAR | GENDER;
}
```

### B. 各模块改造清单

| 模块 | 敏感字段 | 推荐加密 | 改造难度 | 工作量 |
|------|---------|---------|---------|--------|
| qimen | 姓名、问题、性别、年份 | NAME \| QUESTION | 中 | 7 人日 |
| ziwei | 农历生日、性别 | 全部 | 中 | 8 人日 |
| liuyao | 问题 CID | QUESTION | 低 | 4 人日 |
| xiaoliuren | 问题 CID | QUESTION | 低 | 4 人日 |
| daliuren | 问题 CID | QUESTION | 低 | 4 人日 |
| meihua | 性别、年份 | 全部 | 低 | 5 人日 |
| tarot | 无（仅哈希） | 无需改造 | 极低 | 1 人日 |
| bazi | ✅ 已完成 | - | - | 0 |

### C. 前端加密库

| 库名 | 用途 | 大小 |
|------|-----|------|
| `@noble/ciphers` | XChaCha20-Poly1305 | 8 KB |
| `@noble/curves` | X25519 密钥交换 | 15 KB |

---

## 🎯 总结

### 核心优势

1. **零服务器成本** - 直接调用 RPC 节点
2. **零代码重复** - 复用链上 Runtime 算法
3. **自动同步更新** - 链端更新，前端自动生效
4. **开发成本低** - 38 人日（对比 ZK 95+ 人日，节省 60%）
5. **用户体验好** - 即时计算，无等待

### 推荐配置

| 用户类型 | 推荐模式 | RPC 选择 |
|---------|---------|---------|
| 普通用户 | Partial ⭐ | 公共 RPC |
| 高隐私需求 | Partial | 公共 RPC |
| 企业用户 | Private | 自建 RPC |

### 实施建议

1. ✅ **立即启动 Phase 1** - Qimen + Ziwei 改造
2. ✅ **优先 Partial 模式** - 满足大多数场景
3. ✅ **企业用户自建 RPC** - 实现完全隐私

---

**文档版本**: v3.4
**最后更新**: 2025-12-26
**维护者**: Stardust 技术团队
