# Cosmos 项目重命名修改文档

## 📋 修改概述

**项目名称变更**: `Cosmos` → `Cosmos`  
**原生代币变更**: `COS` → `COS`  
**修改日期**: 2024年  
**文档版本**: v1.0

---

## ⚠️ 重要风险提示

1. **品牌冲突风险**: "Cosmos" 在区块链领域已被 Cosmos Network 广泛使用，存在严重的品牌混淆和商标冲突风险
2. **代币符号冲突**: "COS" 可能与 Cosmos 生态相关代币产生冲突
3. **技术成本**: 需要修改大量代码、配置文件和文档
4. **链上数据**: 如果链已部署，需要迁移链上数据
5. **社区认知**: 需要重新建立品牌认知

---

## 📁 修改分类

### 1. Rust 链端代码修改

#### 1.1 Runtime 核心文件

| 文件路径 | 修改内容 | 优先级 |
|---------|---------|--------|
| `runtime/src/lib.rs` | `spec_name` 和 `impl_name`: `"cosmos"` → `"cosmos"` | 🔴 高 |
| `runtime/src/configs/mod.rs` | 所有 `COS` 注释和常量 → `COS` | 🔴 高 |
| `runtime/src/genesis_config_presets.rs` | `COS` 相关注释 → `COS` | 🔴 高 |

#### 1.2 Node 节点配置

| 文件路径 | 修改内容 | 优先级 |
|---------|---------|--------|
| `node/src/chain_spec.rs` | `tokenSymbol`: `"COS"` → `"COS"` | 🔴 高 |
| `node/src/chain_spec.rs` | 链名称: `"Cosmos Development"` → `"Cosmos Development"` | 🔴 高 |
| `node/src/chain_spec.rs` | 链 ID: `"cosmos_dev"` → `"cosmos_dev"` | 🔴 高 |
| `node/src/chain_spec.rs` | `"Cosmos Local Testnet"` → `"Cosmos Local Testnet"` | 🔴 高 |
| `node/src/chain_spec.rs` | `"cosmos_local"` → `"cosmos_local"` | 🔴 高 |

#### 1.3 Cargo 配置文件

| 文件路径 | 修改内容 | 优先级 |
|---------|---------|--------|
| `Cargo.toml` | 注释: `# Cosmos common libraries` → `# Cosmos common libraries` | 🟡 中 |
| `Cargo.lock` | 自动更新（运行 `cargo build` 后） | 🟢 低 |

#### 1.4 Pallets 模块修改

**需要修改的 Pallets（包含 "Cosmos" 或 "COS" 的注释和文档）:**

| Pallet 路径 | 修改内容 | 优先级 |
|------------|---------|--------|
| `pallets/evidence/README.md` | 所有 `Cosmos` → `Cosmos`, `COS` → `COS` | 🟡 中 |
| `pallets/evidence/src/lib.rs` | `cosmos-media-common` 相关注释 | 🟡 中 |
| `pallets/affiliate/README.md` | `Cosmos` → `Cosmos`, `COS` → `COS` | 🟡 中 |
| `pallets/chat/core/README.md` | `Cosmos` → `Cosmos` | 🟡 中 |
| `pallets/chat/core/Cargo.toml` | `authors`, `repository` 中的 `Cosmos` | 🟡 中 |
| `pallets/chat/livestream/src/lib.rs` | 注释中的 `Cosmos` | 🟡 中 |
| `pallets/chat/group/src/lib.rs` | 注释中的 `Cosmos` | 🟡 中 |
| `pallets/divination/membership/README.md` | `Cosmos` → `Cosmos`, `COS` → `COS` | 🟡 中 |
| `pallets/divination/ocw-tee/Cargo.toml` | `authors`, `repository` | 🟡 中 |
| `pallets/trading/otc/README.md` | `Cosmos` → `Cosmos`, `COS` → `COS` | 🟡 中 |
| `pallets/trading/otc/src/lib.rs` | `COS` 相关变量名和注释 | 🔴 高 |
| `pallets/trading/swap/README.md` | `StarDust` → `Cosmos`, `COS` → `COS` | 🟡 中 |
| `pallets/trading/swap/src/lib.rs` | `COS` 相关注释 | 🔴 高 |
| `pallets/trading/maker/README.md` | `Cosmos` → `Cosmos` | 🟡 中 |
| `pallets/trading/maker/Cargo.toml` | `authors`, `repository` | 🟡 中 |
| `pallets/trading/maker/src/lib.rs` | `COS` 相关注释 | 🔴 高 |
| `pallets/arbitration/README.md` | `Cosmos` → `Cosmos` | 🟡 中 |
| `pallets/referral/README.md` | `Cosmos` → `Cosmos`, `COS` → `COS` | 🟡 中 |
| `pallets/matchmaking/profile/README.md` | `COS` → `COS` | 🟡 中 |
| `pallets/matchmaking/interaction/README.md` | `COS` → `COS` | 🟡 中 |
| `pallets/matchmaking/common/src/types.rs` | 注释中的 `Cosmos` | 🟡 中 |
| `pallets/storage-service/src/types.rs` | `pallet-cosmos-ipfs` → `pallet-cosmos-ipfs` | 🔴 高 |

**注意**: 所有 pallet 的 `Cargo.toml` 中的 `authors` 和 `repository` 字段如果包含 "Cosmos" 都需要修改。

---

### 2. 前端代码修改

#### 2.1 配置文件

| 文件路径 | 修改内容 | 优先级 |
|---------|---------|--------|
| `frontend/package.json` | `name`: `"frontend"` → `"cosmos-frontend"` (可选) | 🟢 低 |
| `frontend/app.json` | `name`: `"frontend"` → `"Cosmos"` | 🔴 高 |
| `frontend/app.json` | `slug`: `"frontend"` → `"cosmos"` | 🔴 高 |

#### 2.2 核心业务代码

| 文件路径 | 修改内容 | 优先级 |
|---------|---------|--------|
| `frontend/src/lib/wallet.ts` | `MNEMONIC_KEY`: `'cosmos_mnemonic'` → `'cosmos_mnemonic'` | 🔴 高 |
| `frontend/src/lib/wallet.ts` | `ACCOUNTS_KEY`: `'cosmos_accounts'` → `'cosmos_accounts'` | 🔴 高 |
| `frontend/src/lib/wallet.ts` | `ACTIVE_ACCOUNT_KEY`: `'cosmos_active_account'` → `'cosmos_active_account'` | 🔴 高 |
| `frontend/src/lib/wallet.ts` | `HD_PATH_PREFIX`: `'//cosmos//'` → `'//cosmos//'` | 🔴 高 |
| `frontend/src/services/ipfs.ts` | `name: \`cosmos-${Date.now()}\`` → `name: \`cosmos-${Date.now()}\`` | 🟡 中 |

#### 2.3 UI 页面修改

| 文件路径 | 修改内容 | 优先级 |
|---------|---------|--------|
| `frontend/app/(tabs)/index.tsx` | `✨ Cosmos` → `✨ Cosmos` | 🔴 高 |
| `frontend/app/(tabs)/index.tsx` | `关于 Cosmos` → `关于 Cosmos` | 🔴 高 |
| `frontend/app/(tabs)/index.tsx` | `Cosmos 是一个...` → `Cosmos 是一个...` | 🔴 高 |
| `frontend/app/(tabs)/profile.tsx` | `关于 Cosmos` → `关于 Cosmos` | 🔴 高 |
| `frontend/app/wallet/index.tsx` | `Cosmos Token` → `Cosmos Token` | 🔴 高 |
| `frontend/app/wallet/receive.tsx` | `Cosmos Token` → `Cosmos Token` | 🔴 高 |
| `frontend/app/help/index.tsx` | `COS奖励` → `COS奖励` | 🔴 高 |
| `frontend/app/help/index.tsx` | `COS是平台积分` → `COS是平台积分` | 🔴 高 |
| `frontend/app/help/index.tsx` | `support@cosmos.app` → `support@cosmos.app` | 🔴 高 |
| `frontend/app/settings/index.tsx` | `Cosmos © 2024` → `Cosmos © 2024` | 🔴 高 |
| `frontend/app/legal/privacy.tsx` | 所有 `Cosmos` → `Cosmos` | 🔴 高 |
| `frontend/app/legal/privacy.tsx` | `privacy@cosmos.app` → `privacy@cosmos.app` | 🔴 高 |
| `frontend/app/legal/terms.tsx` | 所有 `Cosmos` → `Cosmos` | 🔴 高 |
| `frontend/app/legal/terms.tsx` | `support@cosmos.app` → `support@cosmos.app` | 🔴 高 |
| `frontend/app/notifications/index.tsx` | `20 COS` → `20 COS` | 🔴 高 |
| `frontend/app/membership/rewards.tsx` | `COS` → `COS` | 🔴 高 |
| `frontend/app/membership/index.tsx` | `COS` → `COS` | 🔴 高 |
| `frontend/src/components/CheckInCard.tsx` | `COS` → `COS` | 🔴 高 |

---

### 3. 文档修改

#### 3.1 移动端文档

| 文件路径 | 修改内容 | 优先级 |
|---------|---------|--------|
| `docs/MOBILE_DEVELOPMENT_ROADMAP.md` | 标题和所有 `Cosmos` → `Cosmos` | 🟡 中 |
| `docs/MOBILE_FRONTEND_TECHNICAL_DESIGN.md` | 标题和所有 `Cosmos` → `Cosmos` | 🟡 中 |
| `docs/MOBILE_FRONTEND_REQUIREMENTS_MAPPING.md` | 标题和所有 `Cosmos` → `Cosmos` | 🟡 中 |

#### 3.2 Pallet 文档

所有 pallet 的 `README.md` 文件中的 `Cosmos` → `Cosmos`, `COS` → `COS`。

---

### 4. 目录和文件名修改（可选）

**注意**: 以下修改需要谨慎，可能影响 Git 历史和依赖关系。

| 项目 | 原名称 | 新名称 | 优先级 |
|------|--------|--------|--------|
| 项目根目录 | `cosmos` | `cosmos` | 🟢 低（可选） |
| Pallet 名称 | `pallet-cosmos-ipfs` | `pallet-cosmos-ipfs` | 🔴 高（如果存在） |

---

## 🔧 执行步骤

### 阶段 1: 准备工作

1. **创建备份分支**
   ```bash
   git checkout -b backup-before-cosmos-rename
   git push origin backup-before-cosmos-rename
   ```

2. **创建修改分支**
   ```bash
   git checkout -b cosmos-rename
   ```

### 阶段 2: 批量替换（使用脚本）

#### 2.1 创建替换脚本

创建 `scripts/rename-to-cosmos.sh`:

```bash
#!/bin/bash

# 项目名称替换
find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" -o -name "*.md" -o -name "*.toml" -o -name "*.json" \) \
  ! -path "*/node_modules/*" \
  ! -path "*/target/*" \
  ! -path "*/.git/*" \
  ! -path "*/Cargo.lock" \
  -exec sed -i 's/Cosmos/Cosmos/g' {} +

# 代币符号替换（注意大小写）
find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" -o -name "*.md" -o -name "*.toml" -o -name "*.json" \) \
  ! -path "*/node_modules/*" \
  ! -path "*/target/*" \
  ! -path "*/.git/*" \
  ! -path "*/Cargo.lock" \
  -exec sed -i 's/COS/COS/g' {} +

# 小写替换（用于变量名和键名）
find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" \) \
  ! -path "*/node_modules/*" \
  ! -path "*/target/*" \
  ! -path "*/.git/*" \
  ! -path "*/Cargo.lock" \
  -exec sed -i "s/'cosmos_/'cosmos_/g" {} +

find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" \) \
  ! -path "*/node_modules/*" \
  ! -path "*/target/*" \
  ! -path "*/.git/*" \
  ! -path "*/Cargo.lock" \
  -exec sed -i 's/"cosmos_/"cosmos_/g' {} +

# 特殊替换：pallet-cosmos-ipfs
find . -type f \( -name "*.rs" -o -name "*.md" -o -name "*.toml" \) \
  ! -path "*/node_modules/*" \
  ! -path "*/target/*" \
  ! -path "*/.git/*" \
  ! -path "*/Cargo.lock" \
  -exec sed -i 's/pallet-cosmos-ipfs/pallet-cosmos-ipfs/g' {} +

find . -type f \( -name "*.rs" -o -name "*.md" -o -name "*.toml" \) \
  ! -path "*/node_modules/*" \
  ! -path "*/target/*" \
  ! -path "*/.git/*" \
  ! -path "*/Cargo.lock" \
  -exec sed -i 's/CosmosIpfs/CosmosIpfs/g' {} +

echo "批量替换完成！请检查修改结果。"
```

#### 2.2 执行脚本

```bash
chmod +x scripts/rename-to-cosmos.sh
./scripts/rename-to-cosmos.sh
```

### 阶段 3: 手动检查和修复

#### 3.1 关键文件手动检查

1. **`runtime/src/lib.rs`**
   ```rust
   // 确保以下行已修改：
   spec_name: alloc::borrow::Cow::Borrowed("cosmos"),
   impl_name: alloc::borrow::Cow::Borrowed("cosmos"),
   ```

2. **`node/src/chain_spec.rs`**
   ```rust
   // 确保以下行已修改：
   "tokenSymbol": "COS",
   .with_name("Cosmos Development")
   .with_id("cosmos_dev")
   ```

3. **`frontend/src/lib/wallet.ts`**
   ```typescript
   // 确保以下常量已修改：
   const MNEMONIC_KEY = 'cosmos_mnemonic';
   const ACCOUNTS_KEY = 'cosmos_accounts';
   const ACTIVE_ACCOUNT_KEY = 'cosmos_active_account';
   const HD_PATH_PREFIX = '//cosmos//';
   ```

#### 3.2 特殊处理

1. **邮箱域名**: 如果项目使用 `@cosmos.app` 邮箱，需要：
   - 注册新域名 `cosmos.app`（或使用其他域名）
   - 更新所有邮箱引用

2. **GitHub Repository**: 如果项目托管在 GitHub，需要：
   - 创建新仓库或重命名现有仓库
   - 更新所有 `repository` 字段

3. **链上数据迁移**: 如果链已部署：
   - 创建数据迁移脚本
   - 更新链上存储的键名（如果使用 `cosmos_` 前缀）

### 阶段 4: 编译和测试

#### 4.1 Rust 编译

```bash
# 清理旧构建
cargo clean

# 重新编译
cargo build --release

# 运行测试
cargo test
```

#### 4.2 前端编译

```bash
cd frontend
npm install
npm run build
```

#### 4.3 链启动测试

```bash
# 启动开发链
./target/release/cosmos-node --dev

# 检查链信息
curl http://localhost:9944 -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_properties"}'
```

### 阶段 5: 提交和审查

```bash
# 查看所有修改
git status
git diff

# 提交修改
git add .
git commit -m "refactor: rename project from Cosmos to Cosmos, token from COS to COS"

# 推送到远程
git push origin cosmos-rename
```

---

## 📊 修改统计

### 预计修改文件数量

| 类别 | 文件数（估算） |
|------|--------------|
| Rust 源文件 (.rs) | ~150 |
| TypeScript/TSX 文件 | ~50 |
| Markdown 文档 | ~40 |
| Cargo.toml | ~60 |
| JSON 配置文件 | ~10 |
| **总计** | **~310** |

### 预计修改行数

- **代码行**: ~2000+ 行
- **注释和文档**: ~5000+ 行
- **配置行**: ~500+ 行

---

## ✅ 检查清单

### 代码层面

- [ ] Runtime `spec_name` 和 `impl_name` 已修改
- [ ] 链配置中的 `tokenSymbol` 已修改为 `"COS"`
- [ ] 所有链名称和 ID 已更新
- [ ] 所有 pallet 的 `Cargo.toml` 已更新
- [ ] 前端钱包存储键名已更新
- [ ] 所有 UI 文本中的 `Cosmos` → `Cosmos`
- [ ] 所有 UI 文本中的 `COS` → `COS`

### 配置层面

- [ ] `frontend/app.json` 已更新
- [ ] `package.json` 已更新（如需要）
- [ ] 所有邮箱域名已更新（如需要）
- [ ] GitHub repository 链接已更新（如需要）

### 文档层面

- [ ] 所有 README.md 已更新
- [ ] 技术设计文档已更新
- [ ] 开发路线图已更新

### 测试层面

- [ ] Rust 代码编译通过
- [ ] 前端代码编译通过
- [ ] 链可以正常启动
- [ ] 钱包功能正常
- [ ] UI 显示正确

---

## 🚨 注意事项

1. **不要一次性提交所有修改**: 建议分阶段提交，便于审查和回滚
2. **保留备份**: 确保有完整的备份分支
3. **测试充分**: 每个阶段都要进行充分测试
4. **文档同步**: 确保所有文档都已更新
5. **团队沟通**: 通知团队成员修改内容，避免冲突
6. **链上数据**: 如果链已部署，需要制定数据迁移计划

---

## 📝 后续工作

1. **品牌资产更新**
   - Logo 设计
   - 官网更新
   - 社交媒体账号

2. **社区通知**
   - 发布公告
   - 更新社区文档
   - 通知合作伙伴

3. **交易所对接**
   - 更新代币符号
   - 更新链信息
   - 重新提交上币申请

4. **法律合规**
   - 商标注册（如需要）
   - 法律风险评估
   - 合规审查

---

## 📞 联系信息

如有问题或需要帮助，请联系：
- 技术负责人: [待填写]
- 项目经理: [待填写]

---

**文档最后更新**: 2024年  
**文档维护者**: Cosmos 开发团队

