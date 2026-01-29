# Cosmos 重命名文件清单

本文档列出所有需要修改的文件及其具体修改内容。

## 🔴 高优先级文件（必须修改）

### Runtime 核心

| 文件路径 | 修改内容 | 行号（示例） |
|---------|---------|-------------|
| `runtime/src/lib.rs` | `spec_name: "cosmos"` → `"cosmos"` | ~65 |
| `runtime/src/lib.rs` | `impl_name: "cosmos"` → `"cosmos"` | ~66 |
| `runtime/src/configs/mod.rs` | 所有 `COS` 注释 → `COS` | 多处 |
| `runtime/src/genesis_config_presets.rs` | `COS` 注释 → `COS` | ~50 |

### Node 配置

| 文件路径 | 修改内容 | 行号（示例） |
|---------|---------|-------------|
| `node/src/chain_spec.rs` | `"tokenSymbol": "COS"` → `"COS"` | ~10 |
| `node/src/chain_spec.rs` | `"Cosmos Development"` → `"Cosmos Development"` | ~24 |
| `node/src/chain_spec.rs` | `"cosmos_dev"` → `"cosmos_dev"` | ~25 |
| `node/src/chain_spec.rs` | `"Cosmos Local Testnet"` → `"Cosmos Local Testnet"` | ~37 |
| `node/src/chain_spec.rs` | `"cosmos_local"` → `"cosmos_local"` | ~38 |

### 前端核心

| 文件路径 | 修改内容 | 行号（示例） |
|---------|---------|-------------|
| `frontend/src/lib/wallet.ts` | `MNEMONIC_KEY = 'cosmos_mnemonic'` → `'cosmos_mnemonic'` | ~12 |
| `frontend/src/lib/wallet.ts` | `ACCOUNTS_KEY = 'cosmos_accounts'` → `'cosmos_accounts'` | ~13 |
| `frontend/src/lib/wallet.ts` | `ACTIVE_ACCOUNT_KEY = 'cosmos_active_account'` → `'cosmos_active_account'` | ~14 |
| `frontend/src/lib/wallet.ts` | `HD_PATH_PREFIX = '//cosmos//'` → `'//cosmos//'` | ~15 |
| `frontend/app.json` | `"name": "frontend"` → `"Cosmos"` | ~3 |
| `frontend/app.json` | `"slug": "frontend"` → `"cosmos"` | ~4 |

### Trading Pallets（包含 COS 变量）

| 文件路径 | 修改内容 | 优先级 |
|---------|---------|--------|
| `pallets/trading/otc/src/lib.rs` | `dust_amount` → `cos_amount` | 🔴 高 |
| `pallets/trading/otc/src/lib.rs` | `COS` 注释 → `COS` | 🔴 高 |
| `pallets/trading/swap/src/lib.rs` | `COS` 注释 → `COS` | 🔴 高 |
| `pallets/trading/maker/src/lib.rs` | `COS` 注释 → `COS` | 🔴 高 |
| `runtime/src/configs/mod.rs` | `get_dust_to_usd_rate` → `get_cos_to_usd_rate` | 🔴 高 |

---

## 🟡 中优先级文件（建议修改）

### UI 页面文件

| 文件路径 | 修改内容 |
|---------|---------|
| `frontend/app/(tabs)/index.tsx` | `✨ Cosmos` → `✨ Cosmos` |
| `frontend/app/(tabs)/index.tsx` | `关于 Cosmos` → `关于 Cosmos` |
| `frontend/app/(tabs)/index.tsx` | `Cosmos 是一个...` → `Cosmos 是一个...` |
| `frontend/app/(tabs)/profile.tsx` | `关于 Cosmos` → `关于 Cosmos` |
| `frontend/app/wallet/index.tsx` | `Cosmos Token` → `Cosmos Token` |
| `frontend/app/wallet/receive.tsx` | `Cosmos Token` → `Cosmos Token` |
| `frontend/app/help/index.tsx` | `COS奖励` → `COS奖励` |
| `frontend/app/help/index.tsx` | `COS是平台积分` → `COS是平台积分` |
| `frontend/app/help/index.tsx` | `support@cosmos.app` → `support@cosmos.app` |
| `frontend/app/settings/index.tsx` | `Cosmos © 2024` → `Cosmos © 2024` |
| `frontend/app/legal/privacy.tsx` | 所有 `Cosmos` → `Cosmos` |
| `frontend/app/legal/privacy.tsx` | `privacy@cosmos.app` → `privacy@cosmos.app` |
| `frontend/app/legal/terms.tsx` | 所有 `Cosmos` → `Cosmos` |
| `frontend/app/legal/terms.tsx` | `support@cosmos.app` → `support@cosmos.app` |
| `frontend/app/notifications/index.tsx` | `20 COS` → `20 COS` |
| `frontend/app/membership/rewards.tsx` | `COS` → `COS` |
| `frontend/app/membership/index.tsx` | `COS` → `COS` |
| `frontend/src/components/CheckInCard.tsx` | `COS` → `COS` |
| `frontend/src/services/ipfs.ts` | `cosmos-${Date.now()}` → `cosmos-${Date.now()}` |

### Pallet README 文档

| 文件路径 | 修改内容 |
|---------|---------|
| `pallets/evidence/README.md` | 所有 `Cosmos` → `Cosmos`, `COS` → `COS` |
| `pallets/affiliate/README.md` | `Cosmos` → `Cosmos`, `COS` → `COS` |
| `pallets/chat/core/README.md` | `Cosmos` → `Cosmos` |
| `pallets/divination/membership/README.md` | `Cosmos` → `Cosmos`, `COS` → `COS` |
| `pallets/trading/otc/README.md` | `Cosmos` → `Cosmos`, `COS` → `COS` |
| `pallets/trading/swap/README.md` | `StarDust` → `Cosmos`, `COS` → `COS` |
| `pallets/trading/maker/README.md` | `Cosmos` → `Cosmos` |
| `pallets/arbitration/README.md` | `Cosmos` → `Cosmos` |
| `pallets/referral/README.md` | `Cosmos` → `Cosmos`, `COS` → `COS` |
| `pallets/matchmaking/profile/README.md` | `COS` → `COS` |
| `pallets/matchmaking/interaction/README.md` | `COS` → `COS` |

### Pallet Cargo.toml 文件

需要检查并修改以下文件中的 `authors` 和 `repository` 字段：

| 文件路径 | 修改字段 |
|---------|---------|
| `pallets/chat/core/Cargo.toml` | `authors`, `repository` |
| `pallets/chat/livestream/Cargo.toml` | `authors`, `repository` |
| `pallets/chat/group/Cargo.toml` | `authors`, `repository` |
| `pallets/divination/ocw-tee/Cargo.toml` | `authors`, `repository` |
| `pallets/trading/maker/Cargo.toml` | `authors`, `repository` |
| `pallets/trading/swap/Cargo.toml` | `authors`, `repository` |
| `pallets/trading/otc/Cargo.toml` | `authors`, `repository` |
| `pallets/divination/nft/Cargo.toml` | `authors`, `repository` |
| `pallets/matchmaking/profile/Cargo.toml` | `authors`, `repository` |
| `pallets/matchmaking/membership/Cargo.toml` | `authors`, `repository` |
| `pallets/matchmaking/interaction/Cargo.toml` | `authors`, `repository` |
| `pallets/matchmaking/common/Cargo.toml` | `authors`, `repository` |
| `pallets/matchmaking/matching/Cargo.toml` | `authors`, `repository` |
| `pallets/matchmaking/recommendation/Cargo.toml` | `authors`, `repository` |
| `pallets/divination/membership/Cargo.toml` | `authors`, `repository` |
| `pallets/divination/ziwei/Cargo.toml` | `authors` |
| `pallets/divination/xiaoliuren/Cargo.toml` | `authors`, `repository` |
| `pallets/divination/bazi/Cargo.toml` | `authors`, `repository` |
| `pallets/divination/qimen/Cargo.toml` | `authors`, `repository` |

### 移动端文档

| 文件路径 | 修改内容 |
|---------|---------|
| `docs/MOBILE_DEVELOPMENT_ROADMAP.md` | 标题和所有 `Cosmos` → `Cosmos` |
| `docs/MOBILE_FRONTEND_TECHNICAL_DESIGN.md` | 标题和所有 `Cosmos` → `Cosmos` |
| `docs/MOBILE_FRONTEND_REQUIREMENTS_MAPPING.md` | 标题和所有 `Cosmos` → `Cosmos` |

### Pallet 源代码注释

| 文件路径 | 修改内容 |
|---------|---------|
| `pallets/evidence/src/lib.rs` | `cosmos-media-common` 相关注释 |
| `pallets/chat/livestream/src/lib.rs` | 注释中的 `Cosmos` |
| `pallets/chat/group/src/lib.rs` | 注释中的 `Cosmos` |
| `pallets/matchmaking/common/src/types.rs` | 注释中的 `Cosmos` |
| `pallets/storage-service/src/types.ts` | `pallet-cosmos-ipfs` → `pallet-cosmos-ipfs` |

---

## 🟢 低优先级文件（可选修改）

### 配置文件

| 文件路径 | 修改内容 | 说明 |
|---------|---------|------|
| `Cargo.toml` | `# Cosmos common libraries` → `# Cosmos common libraries` | 注释，不影响功能 |
| `frontend/package.json` | `"name": "frontend"` → `"cosmos-frontend"` | 可选，不影响功能 |
| `Cargo.lock` | 自动更新 | 运行 `cargo build` 后自动更新 |

---

## 📝 特殊处理说明

### 1. 变量名替换规则

**需要替换的变量名模式：**
- `dust_amount` → `cos_amount`
- `dust_qty` → `cos_qty`
- `get_dust_to_usd_rate` → `get_cos_to_usd_rate`
- `MinFirstPurchaseDustAmount` → `MinFirstPurchaseCosAmount`
- `MaxFirstPurchaseDustAmount` → `MaxFirstPurchaseCosAmount`

**注意**: 只替换业务逻辑中的变量名，不要替换 Rust 标准库或第三方库中的 `dust` 相关名称。

### 2. 存储键名替换

**前端存储键名：**
- `'cosmos_mnemonic'` → `'cosmos_mnemonic'`
- `'cosmos_accounts'` → `'cosmos_accounts'`
- `'cosmos_active_account'` → `'cosmos_active_account'`

**注意**: 如果链已部署且有用户数据，需要创建迁移脚本更新现有用户的存储键名。

### 3. Pallet 名称替换

**如果存在 `pallet-cosmos-ipfs`：**
- 目录名: `pallets/cosmos-ipfs` → `pallets/cosmos-ipfs`
- 模块名: `pallet_cosmos_ipfs` → `pallet_cosmos_ipfs`
- 类型名: `CosmosIpfs` → `CosmosIpfs`

**注意**: 这需要修改所有引用该 pallet 的文件。

### 4. 邮箱域名替换

**如果使用自定义邮箱域名：**
- `support@cosmos.app` → `support@cosmos.app`
- `privacy@cosmos.app` → `privacy@cosmos.app`

**注意**: 需要先注册新域名或使用其他可用域名。

---

## 🔍 检查命令

### 查找所有包含 "Cosmos" 的文件

```bash
grep -r "Cosmos" --include="*.rs" --include="*.ts" --include="*.tsx" --include="*.md" --include="*.toml" --include="*.json" \
  --exclude-dir=node_modules --exclude-dir=target --exclude-dir=.git .
```

### 查找所有包含 "COS" 的文件

```bash
grep -r "COS" --include="*.rs" --include="*.ts" --include="*.tsx" --include="*.md" --include="*.toml" --include="*.json" \
  --exclude-dir=node_modules --exclude-dir=target --exclude-dir=.git .
```

### 查找所有包含 "cosmos_" 的文件（存储键名）

```bash
grep -r "cosmos_" --include="*.rs" --include="*.ts" --include="*.tsx" \
  --exclude-dir=node_modules --exclude-dir=target --exclude-dir=.git .
```

---

## ✅ 验证清单

执行批量替换后，请验证以下关键文件：

- [ ] `runtime/src/lib.rs` - `spec_name` 和 `impl_name` 为 `"cosmos"`
- [ ] `node/src/chain_spec.rs` - `tokenSymbol` 为 `"COS"`
- [ ] `node/src/chain_spec.rs` - 链名称包含 `"Cosmos"`
- [ ] `frontend/src/lib/wallet.ts` - 所有存储键名以 `cosmos_` 开头
- [ ] `frontend/app.json` - `name` 为 `"Cosmos"`
- [ ] `frontend/app/(tabs)/index.tsx` - UI 文本显示 `Cosmos`
- [ ] `pallets/trading/otc/src/lib.rs` - 变量名使用 `cos_amount` 而非 `dust_amount`

---

**最后更新**: 2024年

