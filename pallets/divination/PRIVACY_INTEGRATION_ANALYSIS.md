# 占卜模块隐私模式集成方案分析

## 📋 文档概述

本文档分析所有占卜模块与 `pallet-divination-privacy` 隐私模式的集成方案，提供可行、合理的存储结构重构建议。

**创建时间**: 2024-12-26  
**目标**: 统一所有占卜模块的隐私保护机制，支持 Public/Partial/Private 三级隐私模式

---

## 🎯 核心目标

### 1. 统一隐私模式

将所有占卜模块的隐私控制统一到 `pallet-divination-privacy`：

```rust
pub enum PrivacyMode {
    Public = 0,   // 完全公开
    Partial = 1,  // 部分加密 + 可授权
    Private = 2,  // 完全私密 + 可授权
}
```

### 2. 数据分层原则

**明文层（链上计算）**：
- 四柱干支索引（八字）
- 九宫排盘数据（奇门遁甲）
- 卦象结构（梅花易数、六爻）
- 性别（用于大运计算）

**加密层（隐私保护）**：
- 姓名
- 出生时间（年月日时分）
- 问题/占问事宜
- 备注信息

### 3. 授权机制

Partial 和 Private 模式都支持多方授权：
- 所有者（Owner）
- 命理师（Master）
- 家族成员（Family）
- AI 服务（AiService）
- 悬赏回答者（BountyAnswerer）

---

## 📊 现有占卜模块分析

### 1. 八字模块 (pallet-bazi)

#### 当前存储结构

```rust
pub struct BaziChart<T: Config> {
    pub owner: T::AccountId,
    pub name: BoundedVec<u8, ConstU32<32>>,        // 明文姓名 ⚠️
    pub birth_time: BirthTime,                      // 明文出生时间 ⚠️
    pub gender: Gender,
    pub sizhu: SiZhu<T>,                            // 四柱数据
    pub dayun: DaYunInfo<T>,
    // ...
}
```

#### 隐私问题

- ❌ 姓名明文存储
- ❌ 完整出生时间明文存储
- ❌ 无隐私模式选择
- ❌ 无授权机制

#### 重构方案

**方案 A: 完全集成 Privacy Pallet（推荐）**

```rust
// 1. 修改 BaziChart 结构
pub struct BaziChart<T: Config> {
    pub owner: T::AccountId,
    pub sizhu_index: SiZhuIndex,      // 四柱索引（明文，8 bytes）
    pub gender: Gender,                // 性别（明文，用于大运）
    pub privacy_mode: PrivacyMode,     // 隐私模式
    pub dayun: DaYunInfo<T>,           // 大运信息（明文）
    pub wuxing_strength: WuXingStrength,
    pub timestamp: u64,
}

// 2. 敏感数据存储在 Privacy Pallet
// 通过 Privacy::store_encrypted_record() 存储：
// - name (姓名)
// - birth_time (出生时间)
// - notes (备注)
```

**存储大小对比**：
- 当前：~600 bytes（含明文姓名和时间）
- 重构后：~200 bytes（BaziChart） + ~300 bytes（EncryptedRecord）

**优势**：
- ✅ 支持三级隐私模式
- ✅ 统一授权机制
- ✅ Runtime API 仍可基于 sizhu_index 计算解盘
- ✅ 减少 BaziChart 存储大小

---

### 2. 奇门遁甲模块 (pallet-qimen)

#### 当前存储结构

```rust
pub struct QimenChart<AccountId, BlockNumber, MaxCidLen> {
    pub id: u64,
    pub diviner: AccountId,
    pub name: Option<BoundedVec<u8, MaxNameLen>>,      // 明文姓名 ⚠️
    pub gender: Option<Gender>,
    pub birth_year: Option<u16>,
    pub question: Option<BoundedVec<u8, MaxQuestionLen>>, // 明文问题 ⚠️
    pub question_type: Option<QuestionType>,
    
    // 排盘数据（明文，用于计算）
    pub year_ganzhi: GanZhi,
    pub month_ganzhi: GanZhi,
    pub day_ganzhi: GanZhi,
    pub hour_ganzhi: GanZhi,
    pub ju_number: u8,
    pub palaces: [Palace; 9],  // 九宫数据
    // ...
}
```

#### 隐私问题

- ❌ 姓名明文存储
- ❌ 问题明文存储
- ⚠️ 出生时间可从四柱反推（约2小时精度）

#### 重构方案

**Partial 模式特别适合奇门遁甲**：

```rust
pub struct QimenChart<AccountId, BlockNumber> {
    pub id: u64,
    pub diviner: AccountId,
    pub privacy_mode: PrivacyMode,
    
    // === 明文层（用于链上计算） ===
    pub gender: Option<Gender>,
    pub birth_year: Option<u16>,
    pub question_type: Option<QuestionType>,
    
    // 排盘数据（明文）
    pub year_ganzhi: GanZhi,
    pub month_ganzhi: GanZhi,
    pub day_ganzhi: GanZhi,
    pub hour_ganzhi: GanZhi,
    pub ju_number: u8,
    pub palaces: [Palace; 9],
    
    pub created_at: BlockNumber,
}

// 敏感数据存储在 Privacy Pallet（Partial 模式）：
// - name (姓名)
// - question (问题文本)
// - solar_date (公历日期)
// - solar_time (公历时间)
```

**Partial 模式优势**：
- ✅ 排盘数据明文，支持 Runtime API 实时解盘
- ✅ 姓名和问题加密，保护隐私
- ✅ 可授权命理师/AI 访问加密数据
- ✅ 兼顾链上计算能力和隐私保护

**存储大小**：
- QimenChart: ~400 bytes（明文排盘数据）
- EncryptedRecord: ~200 bytes（加密姓名和问题）

---

### 3. 梅花易数模块 (pallet-meihua)

#### 当前存储结构

```rust
pub struct Hexagram<AccountId, BlockNumber> {
    pub id: u64,
    pub diviner: AccountId,
    pub shang_gua: SingleGua,
    pub xia_gua: SingleGua,
    pub dong_yao: u8,
    pub question_hash: [u8; 32],  // 问题哈希 ⚠️
    pub gender: u8,
    pub birth_year: Option<u16>,
    pub is_public: bool,          // 简单的公开/私密 ⚠️
    // ...
}
```

#### 隐私问题

- ❌ 只有 is_public 二元选择，无细粒度控制
- ❌ 问题只存哈希，无法授权查看原文
- ❌ 无授权机制

#### 重构方案

```rust
pub struct Hexagram<AccountId, BlockNumber> {
    pub id: u64,
    pub diviner: AccountId,
    pub privacy_mode: PrivacyMode,  // 替换 is_public
    
    // === 明文层（卦象数据） ===
    pub shang_gua: SingleGua,
    pub xia_gua: SingleGua,
    pub dong_yao: u8,
    pub gender: u8,
    pub birth_year: Option<u16>,
    
    pub created_at: BlockNumber,
}

// 敏感数据存储在 Privacy Pallet：
// - name (姓名)
// - question (问题原文，不再只存哈希)
// - birth_date (完整出生日期)
// - notes (备注)
```

**改进**：
- ✅ 三级隐私模式
- ✅ 问题原文可授权查看（不再只是哈希）
- ✅ 支持命理师解读授权
- ✅ 保持卦象数据明文，便于分析

---

### 4. 六爻模块 (pallet-liuyao)

#### 当前存储结构

```rust
pub struct LiuYaoGua<AccountId, BlockNumber, MaxCidLen> {
    pub id: u64,
    pub creator: AccountId,
    pub question_cid: Option<BoundedVec<u8, MaxCidLen>>, // IPFS CID ⚠️
    
    // 时间信息（明文）
    pub year_gz: (TianGan, DiZhi),
    pub month_gz: (TianGan, DiZhi),
    pub day_gz: (TianGan, DiZhi),
    pub hour_gz: (TianGan, DiZhi),
    
    // 卦象数据
    pub original_yaos: [YaoInfo; 6],
    pub is_public: bool,  // 简单的公开/私密 ⚠️
    // ...
}
```

#### 隐私问题

- ❌ 问题存储在 IPFS，无链上授权控制
- ❌ 只有 is_public 二元选择
- ❌ 无授权机制

#### 重构方案

```rust
pub struct LiuYaoGua<AccountId, BlockNumber> {
    pub id: u64,
    pub creator: AccountId,
    pub privacy_mode: PrivacyMode,
    
    // === 明文层（卦象数据） ===
    pub year_gz: (TianGan, DiZhi),
    pub month_gz: (TianGan, DiZhi),
    pub day_gz: (TianGan, DiZhi),
    pub hour_gz: (TianGan, DiZhi),
    pub original_yaos: [YaoInfo; 6],
    pub gong: Trigram,
    pub gua_xu: GuaXu,
    
    pub created_at: BlockNumber,
}

// 敏感数据存储在 Privacy Pallet：
// - name (姓名)
// - question (问题原文，不再用 IPFS)
// - birth_info (出生信息)
// - notes (备注)
```

**改进**：
- ✅ 问题从 IPFS 迁移到链上加密存储
- ✅ 统一授权机制
- ✅ 三级隐私模式
- ✅ 卦象数据保持明文，便于分析

---

### 5. 其他模块

#### 紫微斗数 (pallet-ziwei)
- 类似八字，需要存储命盘数据
- 建议采用与八字相同的方案

#### 大六壬 (pallet-daliuren)
- 类似奇门遁甲，需要排盘数据
- 建议采用 Partial 模式

#### 小六壬 (pallet-xiaoliuren)
- 较简单的占卜，数据量小
- 建议采用完全集成方案

#### 塔罗牌 (pallet-tarot)
- 主要是牌阵数据
- 建议采用完全集成方案

---

## 🏗️ 统一重构方案

### 方案设计原则

1. **数据分层**：明文计算数据 + 加密敏感数据
2. **统一接口**：所有模块使用相同的隐私 API
3. **向后兼容**：提供数据迁移路径
4. **性能优化**：减少不必要的存储

### 核心 API 设计

```rust
// 1. 存储加密数据
Privacy::store_encrypted_record(
    origin,
    divination_type: DivinationType,
    result_id: u64,
    privacy_mode: PrivacyMode,
    encrypted_data: Vec<u8>,
    nonce: [u8; 24],
    auth_tag: [u8; 16],
    data_hash: [u8; 32],
    encrypted_fields: Option<u16>,  // Partial 模式使用
)

// 2. 授权访问
Privacy::grant_authorization(
    origin,
    divination_type: DivinationType,
    result_id: u64,
    grantee: AccountId,
    role: AccessRole,
    scope: AccessScope,
    encrypted_key: Vec<u8>,
    expires_at: BlockNumber,
)

// 3. 检查访问权限
Privacy::can_access(
    account: &AccountId,
    divination_type: DivinationType,
    result_id: u64,
) -> bool

// 4. 获取加密数据
Privacy::get_encrypted_record(
    divination_type: DivinationType,
    result_id: u64,
) -> Option<EncryptedRecord>
```

---

## 📝 实施步骤

### Phase 1: Privacy Pallet 完善（1-2周）

1. **重构 PrivacyMode 枚举**
   ```rust
   pub enum PrivacyMode {
       Public = 0,
       Partial = 1,  // 新增
       Private = 2,
   }
   ```

2. **添加 Partial 模式支持**
   - 实现 `encrypted_fields` 标志位
   - 更新权限检查逻辑
   - 添加字段级加密控制

3. **完善授权机制**
   - 支持 5 种授权角色
   - 实现授权过期机制
   - 添加授权撤销功能

### Phase 2: 八字模块重构（1周）

1. **修改 BaziChart 结构**
   - 移除明文姓名和出生时间
   - 添加 `privacy_mode` 字段
   - 保留 `sizhu_index` 和 `gender`

2. **集成 Privacy API**
   - 修改 `create_chart` 函数
   - 添加隐私数据存储逻辑
   - 更新权限检查

3. **数据迁移**
   - 编写迁移脚本
   - 测试迁移流程

### Phase 3: 奇门遁甲模块重构（1周）

1. **修改 QimenChart 结构**
   - 移除明文姓名和问题
   - 添加 `privacy_mode` 字段
   - 保留排盘数据明文

2. **实现 Partial 模式**
   - 配置 `encrypted_fields`
   - 仅加密姓名和问题
   - 保持排盘数据可计算

3. **测试 Runtime API**
   - 验证解盘功能
   - 测试授权访问

### Phase 4: 其他模块重构（2-3周）

按优先级依次重构：
1. 梅花易数
2. 六爻
3. 紫微斗数
4. 大六壬
5. 小六壬
6. 塔罗牌

### Phase 5: 前端适配（1-2周）

1. **更新 UI 组件**
   - 添加隐私模式选择器
   - 实现授权管理界面
   - 更新数据展示逻辑

2. **加密逻辑**
   - 实现前端加密
   - 密钥管理
   - 授权流程

---

## 🔒 安全考虑

### 1. 密钥管理

**前端加密方案**：
```typescript
// 1. 用户钱包签名派生 DataKey
const signature = await wallet.signMessage("derive-encryption-key");
const dataKey = deriveKey(signature);

// 2. 加密敏感数据
const encrypted = aes256gcm.encrypt(sensitiveData, dataKey);

// 3. 用接收者公钥封装 DataKey
const encryptedKey = x25519.seal(dataKey, recipientPublicKey);

// 4. 提交到链上
await privacy.storeEncryptedRecord({
    encryptedData: encrypted.ciphertext,
    nonce: encrypted.nonce,
    authTag: encrypted.tag,
    dataHash: blake2b(sensitiveData),
    encryptedKey: encryptedKey,
});
```

### 2. 授权安全

- ✅ 所有者授权不可撤销
- ✅ 其他授权可设置过期时间
- ✅ 授权撤销立即生效
- ✅ 密钥独立，撤销不影响其他授权

### 3. 隐私保护

**Partial 模式隐私分析**：
- ⚠️ 四柱干支可反推出生时间（约2小时精度）
- ✅ 姓名和问题完全加密
- ✅ 排盘数据可公开计算
- ✅ 适合需要专业解读的场景

**Private 模式隐私分析**：
- ✅ 所有数据完全加密
- ✅ 最高隐私保护级别
- ❌ 无法使用 Runtime API 计算
- ✅ 适合高度敏感数据

---

## 📊 存储成本分析

### 当前存储（以八字为例）

```
BaziChart: ~600 bytes
- 姓名: 32 bytes (明文)
- 出生时间: 10 bytes (明文)
- 四柱数据: ~200 bytes
- 大运数据: ~300 bytes
- 其他: ~58 bytes
```

### 重构后存储

```
BaziChart: ~200 bytes
- sizhu_index: 8 bytes
- gender: 1 byte
- privacy_mode: 1 byte
- dayun: ~150 bytes
- 其他: ~40 bytes

EncryptedRecord: ~300 bytes
- encrypted_data: ~150 bytes
- nonce: 24 bytes
- auth_tag: 16 bytes
- data_hash: 32 bytes
- metadata: ~78 bytes

总计: ~500 bytes (节省 ~100 bytes)
```

### 授权存储

```
每个授权条目: ~120 bytes
- grantee: 32 bytes
- encrypted_key: 72 bytes
- role + scope: 2 bytes
- 时间戳: 8 bytes
- 其他: ~6 bytes

最多 10 个授权: ~1200 bytes
```

---

## ✅ 可行性评估

### 技术可行性: ⭐⭐⭐⭐⭐

- ✅ Privacy Pallet 已实现核心功能
- ✅ 加密方案成熟（AES-256-GCM + X25519）
- ✅ 授权机制完善
- ✅ Runtime API 兼容

### 实施难度: ⭐⭐⭐⭐

- ✅ 模块结构清晰，易于重构
- ⚠️ 需要数据迁移
- ⚠️ 前端需要适配
- ✅ 可分阶段实施

### 性能影响: ⭐⭐⭐⭐⭐

- ✅ 存储成本降低
- ✅ 计算性能无影响（明文数据保留）
- ✅ 授权检查高效（O(1) 查询）
- ✅ 加密在前端完成，不占用链上资源

### 用户体验: ⭐⭐⭐⭐⭐

- ✅ 三级隐私模式，灵活选择
- ✅ 授权机制便于协作
- ✅ Partial 模式兼顾隐私和功能
- ✅ 向后兼容，平滑迁移

---

## 🎯 推荐方案总结

### 1. 八字/紫微斗数

**推荐**: 完全集成 Privacy Pallet

- 明文: sizhu_index, gender, dayun
- 加密: name, birth_time, notes
- 模式: 支持 Public/Partial/Private

### 2. 奇门遁甲/大六壬

**推荐**: Partial 模式

- 明文: 排盘数据（九宫、四柱）
- 加密: name, question, solar_date, solar_time
- 优势: 兼顾链上计算和隐私保护

### 3. 梅花易数/六爻

**推荐**: 完全集成 Privacy Pallet

- 明文: 卦象数据
- 加密: name, question, birth_info, notes
- 改进: 问题从 IPFS/哈希迁移到链上加密

### 4. 小六壬/塔罗牌

**推荐**: 完全集成 Privacy Pallet

- 明文: 占卜结果数据
- 加密: name, question, notes
- 简化: 数据量小，实施简单

---

## 📅 实施时间表

| 阶段 | 任务 | 时间 | 优先级 |
|------|------|------|--------|
| Phase 1 | Privacy Pallet 重构 | 1-2周 | P0 |
| Phase 2 | 八字模块重构 | 1周 | P0 |
| Phase 3 | 奇门遁甲模块重构 | 1周 | P1 |
| Phase 4 | 梅花易数模块重构 | 3天 | P1 |
| Phase 5 | 六爻模块重构 | 3天 | P1 |
| Phase 6 | 其他模块重构 | 1-2周 | P2 |
| Phase 7 | 前端适配 | 1-2周 | P1 |
| Phase 8 | 测试和优化 | 1周 | P0 |

**总计**: 6-9周

---

## 🔗 相关文档

- [Privacy Pallet 设计文档](./privacy/DESIGN.md)
- [Privacy Pallet 可行性分析](./privacy/FEASIBILITY_ANALYSIS.md)
- [八字模块文档](./bazi/README.md)
- [奇门遁甲模块文档](./qimen/README.md)

---

## 📞 联系方式

如有问题或建议，请联系开发团队。

**文档版本**: v1.0  
**最后更新**: 2024-12-26
