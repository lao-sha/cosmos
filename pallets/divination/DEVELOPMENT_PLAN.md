# 占卜系统隐私模式集成 - 开发计划

**基于**: UNIFIED_PRIVACY_INTEGRATION_DESIGN.md v3.4
**创建日期**: 2025-12-26
**总工作量**: 38 人日

---

## 📋 开发阶段总览

| 阶段 | 模块 | 工作量 | 状态 |
|------|------|--------|------|
| Phase 1 | Qimen + Ziwei | 15 人日 | 待开始 |
| Phase 2 | Liuyao + Xiaoliuren + Daliuren + Meihua | 17 人日 | 待开始 |
| Phase 3 | Tarot + 前端优化 | 6 人日 | 待开始 |

---

## 🚀 Phase 1: 核心改造（15 人日）

### 1.1 Qimen 模块改造（7 人日）

#### 1.1.1 后端 - 数据结构修改（1 天）

- [ ] **QimenChart 结构添加隐私字段**
  - 添加 `privacy_mode: PrivacyMode`
  - 添加 `encrypted_fields: Option<u16>`
  - 添加 `sensitive_data_hash: Option<[u8; 32]>`

- [ ] **敏感字段改为 Option 类型**
  - `name: Option<BoundedVec<u8, MaxNameLen>>`
  - `gender: Option<Gender>`
  - `birth_year: Option<u16>`
  - `question: Option<BoundedVec<u8, MaxQuestionLen>>`

- [ ] **计算字段改为 Option 类型（Private 模式）**
  - `year_ganzhi: Option<GanZhi>`
  - `month_ganzhi: Option<GanZhi>`
  - `day_ganzhi: Option<GanZhi>`
  - `hour_ganzhi: Option<GanZhi>`
  - `palaces: Option<[Palace; 9]>`

#### 1.1.2 后端 - 新增接口（1.5 天）

- [ ] **create_chart_encrypted 接口**
  ```rust
  pub fn create_chart_encrypted(
      origin: OriginFor<T>,
      solar_year: u16,
      solar_month: u8,
      solar_day: u8,
      solar_hour: u8,
      solar_minute: u8,
      privacy_mode: PrivacyMode,
  ) -> DispatchResult
  ```

- [ ] **修改现有 create_chart 接口**
  - 添加 `privacy_mode` 参数（默认 Public）
  - 向后兼容处理

#### 1.1.3 后端 - Runtime API（1.5 天）

- [ ] **定义 QimenApi trait**
  ```rust
  sp_api::decl_runtime_apis! {
      pub trait QimenApi {
          fn interpret_chart(chart_id: u64) -> Option<ChartInterpretation>;
          fn compute_chart(
              solar_year: u16,
              solar_month: u8,
              solar_day: u8,
              solar_hour: u8,
              solar_minute: u8,
          ) -> QimenChartResult;
          fn batch_interpret(chart_ids: Vec<u64>) -> Vec<Option<ChartInterpretation>>;
      }
  }
  ```

- [ ] **实现 QimenApi**
  - `interpret_chart`: 读取链上数据解盘（Public/Partial）
  - `compute_chart`: 临时排盘（Private + 预览）
  - `batch_interpret`: 批量解盘

- [ ] **定义 QimenChartResult 类型**

#### 1.1.4 后端 - 单元测试（1 天）

- [ ] 测试 create_chart_encrypted (Public 模式)
- [ ] 测试 create_chart_encrypted (Partial 模式)
- [ ] 测试 create_chart_encrypted (Private 模式)
- [ ] 测试 interpret_chart API
- [ ] 测试 compute_chart API
- [ ] 测试向后兼容性

#### 1.1.5 前端 - Qimen 服务层（2 天）

- [ ] **QimenEncryptionService**
  - `createPartialChart()`: 使用 batchAll 原子化创建
  - `createPrivateChart()`: 全加密模式创建
  - `interpretChart()`: 调用 Runtime API 解盘

- [ ] **本地密钥存储管理**
  - `saveEncryptedKeyToLocal()`
  - `loadEncryptedKeyFromLocal()`

- [ ] **授权功能集成**
  - `authorizeMaster()`: 授权命理师
  - `authorizeFamily()`: 授权家人

---

### 1.2 Ziwei 模块改造（8 人日）

#### 1.2.1 后端 - 数据结构修改（1 天）

- [ ] **ZiweiChart 结构添加隐私字段**
  - 添加 `privacy_mode: PrivacyMode`
  - 添加 `encrypted_fields: Option<u16>`
  - 添加 `sensitive_data_hash: Option<[u8; 32]>`

- [ ] **敏感字段改为 Option 类型**
  - 农历生日相关字段
  - 性别字段

- [ ] **计算字段改为 Option 类型**
  - 命宫、身宫等十二宫位数据

#### 1.2.2 后端 - 新增接口（1.5 天）

- [ ] **create_chart_encrypted 接口**
- [ ] **修改现有接口兼容性**

#### 1.2.3 后端 - Runtime API（2 天）

- [ ] **定义 ZiweiApi trait**
  ```rust
  pub trait ZiweiApi {
      fn interpret_chart(chart_id: u64) -> Option<ZiweiInterpretation>;
      fn compute_chart(
          lunar_year: u16,
          lunar_month: u8,
          lunar_day: u8,
          lunar_hour: u8,
          gender: Gender,
      ) -> ZiweiChartResult;
  }
  ```

- [ ] **实现 ZiweiApi**
- [ ] **定义 ZiweiChartResult 类型**

#### 1.2.4 后端 - 单元测试（1.5 天）

- [ ] 测试各隐私模式创建
- [ ] 测试 Runtime API
- [ ] 测试向后兼容性

#### 1.2.5 前端 - Ziwei 服务层（2 天）

- [ ] **ZiweiEncryptionService**
- [ ] **本地密钥存储管理**
- [ ] **授权功能集成**

---

## 🔧 Phase 2: 其他模块（17 人日）

### 2.1 Liuyao 模块改造（4 人日）

#### 2.1.1 后端改造（2 天）

- [ ] **数据结构添加隐私字段**
- [ ] **敏感字段改为 Option**
- [ ] **新增 create_gua_encrypted 接口**
- [ ] **实现 LiuyaoApi Runtime API**
- [ ] **迁移 IPFS 问题存储到 EncryptedRecords**

#### 2.1.2 前端改造（1 天）

- [ ] **LiuyaoEncryptionService**
- [ ] **问题加密存储**

#### 2.1.3 测试（1 天）

- [ ] 单元测试
- [ ] 集成测试

---

### 2.2 Xiaoliuren 模块改造（4 人日）

#### 2.2.1 后端改造（2 天）

- [ ] **数据结构添加隐私字段**
- [ ] **新增 create_reading_encrypted 接口**
- [ ] **实现 XiaoliurenApi Runtime API**
- [ ] **迁移 IPFS 问题存储**

#### 2.2.2 前端改造（1 天）

- [ ] **XiaoliurenEncryptionService**

#### 2.2.3 测试（1 天）

- [ ] 单元测试
- [ ] 集成测试

---

### 2.3 Daliuren 模块改造（4 人日）

#### 2.3.1 后端改造（2 天）

- [ ] **数据结构添加隐私字段**
- [ ] **新增 create_chart_encrypted 接口**
- [ ] **实现 DaliurenApi Runtime API**
- [ ] **迁移 IPFS 问题存储**

#### 2.3.2 前端改造（1 天）

- [ ] **DaliurenEncryptionService**

#### 2.3.3 测试（1 天）

- [ ] 单元测试
- [ ] 集成测试

---

### 2.4 Meihua 模块改造（5 人日）

#### 2.4.1 后端改造（2.5 天）

- [ ] **数据结构添加隐私字段**
- [ ] **敏感字段（性别、年份）改为 Option**
- [ ] **新增 create_gua_encrypted 接口**
- [ ] **实现 MeihuaApi Runtime API**
- [ ] **激活已有的加密结构**

#### 2.4.2 前端改造（1.5 天）

- [ ] **MeihuaEncryptionService**

#### 2.4.3 测试（1 天）

- [ ] 单元测试
- [ ] 集成测试

---

## 🎨 Phase 3: 收尾（6 人日）

### 3.1 Tarot 模块改造（1 天）

- [ ] **替换 is_public 为 privacy_mode**
- [ ] **保持问题哈希存储（已满足隐私需求）**
- [ ] **单元测试**

---

### 3.2 前端通用组件（3 天）

#### 3.2.1 加密服务层（1.5 天）

- [ ] **EncryptionKeyService 实现**
  - `getOrCreateKeyPair()`
  - `registerEncryptionKey()`
  - `updateEncryptionKey()`
  - `exportKeyBackup()`
  - `importKeyBackup()`

- [ ] **DivinationEncryptionService 实现**
  - `createEncryptedRecord()`
  - `decryptSensitiveData()`

- [ ] **AuthorizationService 实现**
  - `grantAccess()`
  - `revokeAccess()`
  - `listAuthorizations()`

#### 3.2.2 UI 组件（1.5 天）

- [ ] **PrivacyModeSelector 组件**
  - 三种模式选择器
  - 各模式说明提示

- [ ] **EncryptedFieldsSelector 组件**
  - Partial 模式加密字段选择

- [ ] **KeyBackupDialog 组件**
  - 密钥导出界面
  - 密钥恢复界面

- [ ] **AuthorizationManager 组件**
  - 授权列表展示
  - 添加/撤销授权

---

### 3.3 测试与文档（2 天）

#### 3.3.1 集成测试（1 天）

- [ ] **端到端测试**
  - Public 模式完整流程
  - Partial 模式完整流程
  - Private 模式完整流程
  - 授权流程测试

- [ ] **跨模块测试**
  - Privacy Pallet 与各占卜模块集成

#### 3.3.2 文档更新（1 天）

- [ ] **更新各模块 README**
- [ ] **API 文档更新**
- [ ] **前端使用指南**
- [ ] **用户隐私模式选择指南**

---

## 📦 依赖关系

```
Phase 1.1 (Qimen)
    └── Phase 1.2 (Ziwei) [可并行]

Phase 1 完成
    └── Phase 2.1 (Liuyao)
    └── Phase 2.2 (Xiaoliuren) [可并行]
    └── Phase 2.3 (Daliuren) [可并行]
    └── Phase 2.4 (Meihua) [可并行]

Phase 2 完成
    └── Phase 3.1 (Tarot)
    └── Phase 3.2 (前端组件) [可并行]
    └── Phase 3.3 (测试文档) [依赖 3.1 + 3.2]
```

---

## ✅ 验收标准

### Phase 1 验收

- [ ] Qimen 支持三种隐私模式创建
- [ ] Qimen Runtime API 可用
- [ ] Ziwei 支持三种隐私模式创建
- [ ] Ziwei Runtime API 可用
- [ ] 前端可创建 Partial 模式占卜
- [ ] 单元测试覆盖率 > 80%

### Phase 2 验收

- [ ] 所有占卜模块支持隐私模式
- [ ] IPFS 问题存储迁移完成
- [ ] 前端各模块服务层完成
- [ ] 单元测试覆盖率 > 80%

### Phase 3 验收

- [ ] Tarot 隐私模式改造完成
- [ ] 前端通用组件可用
- [ ] 端到端测试通过
- [ ] 文档更新完成
- [ ] 整体单元测试覆盖率 > 90%

---

## 🚨 风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| Runtime API 性能问题 | 低 | 高 | 提前进行基准测试 |
| 前端加密库兼容性 | 中 | 中 | 使用成熟的 @noble 系列 |
| 数据迁移复杂度 | 中 | 中 | 保持向后兼容，渐进式迁移 |
| 密钥管理用户体验 | 中 | 高 | 提供完善的备份恢复功能 |

---

**创建时间**: 2025-12-26
**版本**: v1.0
**维护者**: Stardust 技术团队
