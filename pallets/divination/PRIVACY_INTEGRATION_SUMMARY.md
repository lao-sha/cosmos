# 占卜系统隐私模式集成 - 执行摘要

**完整设计文档**: [UNIFIED_PRIVACY_INTEGRATION_DESIGN.md](./UNIFIED_PRIVACY_INTEGRATION_DESIGN.md)

---

## 🎯 核心目标

将 `pallet-divination-privacy` 的 **PrivacyMode** (Public/Partial/Private) 统一集成到所有占卜模块，采用 **前端传参 + Runtime API 计算** 方案。

---

## 📊 当前问题

| 模块 | 问题 | 风险等级 |
|------|-----|---------|
| **qimen** | 姓名、问题明文存储 | 🔴 高 |
| **ziwei** | 完整出生时间明文 | 🔴 高 |
| **liuyao/daliuren/xiaoliuren** | 依赖 IPFS 链下存储 | 🟡 中 |
| **meihua** | 有加密结构未使用 | 🟡 中 |
| **bazi** | ✅ 已完整集成 | 🟢 已解决 |
| **tarot** | 仅存问题哈希 | 🟢 较好 |

---

## ✨ 解决方案：前端传参 + Runtime API

### 核心架构

```
┌─────────────────────────────────────────────────────────────┐
│                    统一计算架构                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   Public/Partial 模式：                                      │
│   前端 ──(chartId)──> RPC ──> Runtime API ──> 返回结果      │
│                              （读取链上明文数据）             │
│                                                             │
│   Private 模式：                                             │
│   前端 ──解密──> 前端 ──(参数)──> Runtime API ──> 返回结果   │
│                              （传入解密后的参数）             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 三种模式对比

| 模式 | 敏感数据 | 计算数据 | 计算方式 | 存储增量 | 推荐场景 |
|------|---------|---------|---------|---------|---------|
| **Public** | 明文 | 明文 | Runtime API (chartId) | 0 | 公开展示 |
| **Partial** | 加密 | 明文 | Runtime API (chartId) | +50B | **推荐大多数场景** ⭐ |
| **Private** | 加密 | 加密 | Runtime API (前端传参) | +50B | 高度敏感数据 |

### 方案优势

| 优势 | 说明 |
|------|------|
| ✅ **零服务器成本** | 无需部署后端服务，直接调用 RPC 节点 |
| ✅ **零开发重复** | 复用链上 Runtime 算法，无需前端重写 |
| ✅ **自动同步更新** | 链端算法更新，前端自动生效 |
| ✅ **即时计算** | 无需等待 |
| ✅ **开发成本低** | 38 人日完成全部改造 |

---

## 📦 Partial 模式详解（推荐）

```
┌─────────────────────────────────────────────────────────────┐
│                      QimenChart 存储                         │
├─────────────────────────────────────────────────────────────┤
│ privacy_mode: Partial                                        │
│                                                              │
│ 计算数据（明文，支持免费链上解盘）                             │
│ ├── year_ganzhi: (甲, 子)                                   │
│ ├── month_ganzhi: (乙, 丑)                                  │
│ ├── palaces: [九宫排盘数据...]                              │
│ └── ju_number: 阳遁三局                                     │
│                                                              │
│ 敏感数据（已移除，加密存储在 EncryptedRecords）               │
│ ├── name: None                                              │
│ └── question: None                                          │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│         Privacy::EncryptedRecords<Qimen, chart_id>          │
├─────────────────────────────────────────────────────────────┤
│ encrypted_fields: NAME | QUESTION (0x0003)                  │
│                                                              │
│ encrypted_data: AES-256-GCM({                               │
│   name: "张三",                                             │
│   question: "何时能找到工作？"                               │
│ })                                                          │
│                                                              │
│ 授权列表:                                                    │
│ ├── Owner (Alice)    → encrypted_key_for_alice             │
│ ├── Master (Bob)     → encrypted_key_for_bob  [可授权]     │
│ └── Family (Charlie) → encrypted_key_for_charlie [可授权]  │
└─────────────────────────────────────────────────────────────┘
```

**核心优势**:
- ✅ 免费链上解盘（计算数据明文）
- ✅ 隐私保护（姓名、问题加密）
- ✅ 多方授权（咨询师、家人可访问）

---

## 🚀 实施路径

### Phase 1: 高优先级（2周）
- **qimen** (奇门遁甲) - 7人日
- **ziwei** (紫微斗数) - 8人日

### Phase 2: 中优先级（2周）
- **liuyao** (六爻) - 4人日
- **xiaoliuren** (小六壬) - 4人日
- **daliuren** (大六壬) - 4人日
- **meihua** (梅花易数) - 5人日

### Phase 3: 收尾（1周）
- **tarot** (塔罗牌) - 1人日
- 前端 UI 组件 - 5人日

**总计**: 38 人日

---

## 📈 技术实现

### Runtime API 设计

```rust
sp_api::decl_runtime_apis! {
    pub trait QimenApi {
        /// 解盘（Public/Partial 模式）
        fn interpret_chart(chart_id: u64) -> Option<ChartInterpretation>;

        /// 临时排盘（Private 模式 + 临时查看）⭐
        fn compute_chart(
            solar_year: u16,
            solar_month: u8,
            solar_day: u8,
            solar_hour: u8,
            solar_minute: u8,
        ) -> QimenChartResult;
    }
}
```

### 前端调用示例

```typescript
// Partial 模式：直接传 chartId
const interpretation = await api.call.qimenApi.interpretChart(chartId);

// Private 模式：前端解密后调用临时排盘 API
const decrypted = await decryptWithPrivateKey(encryptedRecord, privateKey);
const chartResult = await api.call.qimenApi.computeChart(
  decrypted.solarYear,
  decrypted.solarMonth,
  decrypted.solarDay,
  decrypted.solarHour,
  decrypted.solarMinute,
);
```

### 核心数据结构

**Private 模式加密数据** (`PrivateEncryptedData`)：
```rust
pub struct PrivateEncryptedData {
    pub name: Option<String>,        // 敏感数据
    pub question: Option<String>,
    pub solar_year: u16,             // 计算所需（公历时间）
    pub solar_month: u8,
    pub solar_day: u8,
    pub solar_hour: u8,
    pub solar_minute: u8,
}
```

**临时排盘结果** (`QimenChartResult`)：
```rust
pub struct QimenChartResult {
    pub year_ganzhi: GanZhi,         // 四柱
    pub month_ganzhi: GanZhi,
    pub day_ganzhi: GanZhi,
    pub hour_ganzhi: GanZhi,
    pub jie_qi: JieQi,               // 局数信息
    pub dun_type: DunType,
    pub ju_number: u8,
    pub palaces: [Palace; 9],        // 盘面
    pub fortune: Option<Fortune>,    // 解读
}
```

---

## ⚖️ 可行性评估

### 技术可行性 ⭐⭐⭐⭐⭐

- ✅ 复用现有 Runtime，无需新增复杂组件
- ✅ 零服务器成本
- ✅ 单套代码，链端更新自动同步
- ✅ 向后完全兼容（Public 模式保持原有行为）

### 隐私保护评估

| 模式 | 隐私级别 | 说明 |
|------|---------|------|
| **Public** | ❌ 无 | 全部明文 |
| **Partial** ⭐ | ⭐⭐⭐⭐ | 敏感数据加密，计算数据公开（推荐大多数用户） |
| **Private + 公共RPC** | ⭐⭐⭐ | RPC节点可见明文参数 |
| **Private + 自建RPC** | ⭐⭐⭐⭐⭐ | 完全隐私（企业用户） |

### 实施成本

| 维度 | 工作量 | 风险 |
|------|--------|-----|
| 后端改造 | 32人日 | 低 |
| 前端适配 | 6人日 | 低 |
| **总计** | **38人日** | 低 |

---

## 🎯 立即行动

### Step 1: 改造 Qimen 模块（7天）

1. 添加 `privacy_mode`, `encrypted_fields` 字段
2. 敏感字段改为 `Option` 类型
3. 新增 `create_chart_encrypted` 接口
4. 实现 Runtime API（含传参版本）
5. 单元测试

### Step 2: 前端加密服务（5天）

```typescript
// 密钥管理
export class EncryptionKeyService {
  static async getOrCreateKeyPair(): Promise<{ privateKey, publicKey }>;
  static async registerEncryptionKey(api, signer): Promise<void>;
}

// 加密记录创建
export class DivinationEncryptionService {
  static async createEncryptedRecord(api, signer, ...): Promise<void>;
  static async decryptSensitiveData(record, key, privKey): Promise<object>;
}

// 授权管理
export class AuthorizationService {
  static async grantAccess(api, signer, ...): Promise<void>;
  static async revokeAccess(api, signer, ...): Promise<void>;
  static async listAuthorizations(api, ...): Promise<Array>;
}
```

> 详细实现请参考完整设计文档中的 "🔐 Privacy Pallet 集成详解" 章节

### Step 3: Runtime API 调用封装（3天）

```typescript
export class DivinationService {
  // Partial 模式
  async interpretChart(chartId: number): Promise<Interpretation>;

  // Private 模式
  async interpretPrivateChart(chartId: number, privateKey: Uint8Array): Promise<Interpretation>;
}
```

---

## 📚 参考资源

- **完整设计**: [UNIFIED_PRIVACY_INTEGRATION_DESIGN.md](./UNIFIED_PRIVACY_INTEGRATION_DESIGN.md)
- **Privacy 模块**: [privacy/README.md](./privacy/README.md)
- **Bazi 集成案例**: [bazi/docs/BAZI_CHART_STRUCT_MODIFICATIONS.md](./bazi/docs/BAZI_CHART_STRUCT_MODIFICATIONS.md)

---

**生成时间**: 2025-12-26
**版本**: v3.4
**状态**: 设计完成，待实施
**优先级**: 🔴 高（隐私合规和数据安全）
