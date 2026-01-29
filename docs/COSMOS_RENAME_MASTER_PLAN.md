# Cosmos 重命名最优方案

> 综合 `RENAME_STARCOS_TO_COSMOS.md` 和 `COSMOS_RENAME_FILE_LIST.md` 设计

## 变更概览

| 变更项 | 原值 | 新值 | 影响范围 |
|--------|------|------|----------|
| 项目名称 | Cosmos | Cosmos | 145 文件 / 355 处 |
| 代币符号 | COS | COS | 98 文件 / 770 处 |

---

## 执行策略

### 🎯 核心原则

1. **依赖顺序**：先修改 trait 定义，再修改实现
2. **编译验证**：每个阶段完成后执行 `cargo check`
3. **批量替换**：使用 sed/IDE 批量替换，减少手动错误
4. **分支隔离**：在 `feature/cosmos-rename` 分支进行

### 📋 执行顺序

```
阶段 P0 → P1 → P2 → P3 → P4 → P5
 ↓       ↓     ↓     ↓     ↓     ↓
Trait  Runtime Node Frontend Docs  Cargo.toml
```

---

## P0: Trading Common Trait（编译依赖根）

**必须最先修改**，否则后续文件无法编译。

### 文件: `pallets/trading/common/src/traits.rs`

| 修改项 | 原内容 | 新内容 | 行号 |
|--------|--------|--------|------|
| 函数名 | `get_dust_to_usd_rate` | `get_cos_to_usd_rate` | 29 |
| 参数名 | `dust_qty` | `cos_qty` | 36, 41 |
| 注释 | `COS/USD` | `COS/USD` | 14, 24, 27 |
| 变量名 | `dust_precision`, `dust_amount_u128` | `cos_precision`, `cos_amount_u128` | 295-296 |
| 注释 | 所有 `COS` | `COS` | 255-268 |

**批量替换命令**:
```bash
cd pallets/trading/common/src
sed -i 's/get_dust_to_usd_rate/get_cos_to_usd_rate/g' traits.rs
sed -i 's/dust_qty/cos_qty/g' traits.rs
sed -i 's/dust_precision/cos_precision/g' traits.rs
sed -i 's/dust_amount/cos_amount/g' traits.rs
sed -i 's/COS\/USD/COS\/USD/g' traits.rs
sed -i 's/COS 数量/COS 数量/g' traits.rs
sed -i 's/COS 精度/COS 精度/g' traits.rs
sed -i 's/COS 保证金/COS 保证金/g' traits.rs
sed -i 's/COS 金额/COS 金额/g' traits.rs
sed -i 's/0\.1 USD\/COS/0.1 USD\/COS/g' traits.rs
```

### 文件: `pallets/trading/common/README.md`

同步更新文档中的 `COS` → `COS`。

---

## P1: Trading Pallets 实现层

### 1.1 Pricing Pallet

**文件**: `pallets/trading/pricing/src/lib.rs`

| 修改项 | 原内容 | 新内容 |
|--------|--------|--------|
| 函数名 | `get_dust_market_price_weighted` | `get_cos_market_price_weighted` |
| 注释 | `USDT/COS` | `USDT/COS` |

**批量替换命令**:
```bash
cd pallets/trading/pricing/src
sed -i 's/get_dust_market_price_weighted/get_cos_market_price_weighted/g' lib.rs
sed -i 's/COS/COS/g' lib.rs  # 注意：需检查是否有误替换
```

**同步文件**:
- `pallets/trading/pricing/src/tests.rs`
- `pallets/trading/pricing/README.md`
- `pallets/trading/pricing/docs/EVALUATION.md`

### 1.2 OTC Pallet

**文件**: `pallets/trading/otc/src/lib.rs`

| 修改项 | 原内容 | 新内容 | 行号 |
|--------|--------|--------|------|
| 函数调用 | `get_dust_to_usd_rate` | `get_cos_to_usd_rate` | 1280, 1452, 2704, 2774, 2815 |
| 变量名 | `dust_to_usd_rate` | `cos_to_usd_rate` | 多处 |
| 变量名 | `dust_amount` | `cos_amount` | 多处 |
| Config类型 | `MinFirstPurchaseDustAmount` | `MinFirstPurchaseCosAmount` | - |
| Config类型 | `MaxFirstPurchaseDustAmount` | `MaxFirstPurchaseCosAmount` | - |

**批量替换命令**:
```bash
cd pallets/trading/otc/src
sed -i 's/get_dust_to_usd_rate/get_cos_to_usd_rate/g' lib.rs
sed -i 's/dust_to_usd_rate/cos_to_usd_rate/g' lib.rs
sed -i 's/dust_amount/cos_amount/g' lib.rs
sed -i 's/MinFirstPurchaseDustAmount/MinFirstPurchaseCosAmount/g' lib.rs
sed -i 's/MaxFirstPurchaseDustAmount/MaxFirstPurchaseCosAmount/g' lib.rs
```

### 1.3 Swap Pallet

**文件**: `pallets/trading/swap/src/lib.rs`

```bash
cd pallets/trading/swap/src
sed -i 's/get_dust_to_usd_rate/get_cos_to_usd_rate/g' lib.rs
```

### 1.4 Maker Pallet

**文件**: `pallets/trading/maker/src/lib.rs`

| 修改项 | 原内容 | 新内容 |
|--------|--------|--------|
| 函数名 | `calculate_dust_amount_for_usd` | `calculate_cos_amount_for_usd` |
| 变量名 | `dust_to_usd_rate` | `cos_to_usd_rate` |

```bash
cd pallets/trading/maker/src
sed -i 's/calculate_dust_amount_for_usd/calculate_cos_amount_for_usd/g' lib.rs
sed -i 's/dust_to_usd_rate/cos_to_usd_rate/g' lib.rs
```

### ✅ P1 验证点

```bash
cargo check -p pallet-trading-common
cargo check -p pallet-trading-pricing
cargo check -p pallet-trading-otc
cargo check -p pallet-trading-swap
cargo check -p pallet-trading-maker
```

---

## P2: Runtime 核心配置

### 2.1 Runtime lib.rs

**文件**: `runtime/src/lib.rs`

```rust
// 修改前
spec_name: alloc::borrow::Cow::Borrowed("cosmos"),
impl_name: alloc::borrow::Cow::Borrowed("cosmos"),

// 修改后
spec_name: alloc::borrow::Cow::Borrowed("cosmos"),
impl_name: alloc::borrow::Cow::Borrowed("cosmos"),
```

### 2.2 Runtime configs/mod.rs

**文件**: `runtime/src/configs/mod.rs`

| 行号 | 修改内容 |
|------|----------|
| 269 | `10 COS` → `10 COS` |
| 273 | `1亿 COS` → `1亿 COS` |
| 595 | `50 COS` → `50 COS` |
| 637 | `1 COS` → `1 COS` |
| 638 | `0.05 COS` → `0.05 COS` |
| 685 | `get_dust_to_usd_rate` → `get_cos_to_usd_rate` |
| 686 | `get_dust_market_price_weighted` → `get_cos_market_price_weighted` |
| 694 | `dust_qty` → `cos_qty` |
| 779 | `10 COS` → `10 COS` |
| 862-863 | Config 类型名同步修改 |
| 870 | `0.1 COS` → `0.1 COS` |
| 927-953 | 所有 `COS` → `COS`，变量名 `min_dust` → `min_cos` |
| 1261 | `0.1 COS` → `0.1 COS` |
| 1462 | `1 COS` → `1 COS` |
| 1497-1504 | 所有金额注释 `COS` → `COS` |

**批量替换命令**:
```bash
cd runtime/src/configs
# 函数名替换
sed -i 's/get_dust_to_usd_rate/get_cos_to_usd_rate/g' mod.rs
sed -i 's/get_dust_market_price_weighted/get_cos_market_price_weighted/g' mod.rs
# 变量名替换
sed -i 's/dust_qty/cos_qty/g' mod.rs
sed -i 's/min_dust/min_cos/g' mod.rs
# 注释替换（谨慎执行，建议手动检查）
sed -i 's/ COS/ COS/g' mod.rs
sed -i 's/COS\//COS\//g' mod.rs
```

### 2.3 Genesis Config

**文件**: `runtime/src/genesis_config_presets.rs`

```rust
// 修改前
/// Total initial supply: 100,000,000,000 COS

// 修改后
/// Total initial supply: 100,000,000,000 COS
```

### ✅ P2 验证点

```bash
cargo check -p solochain-template-runtime
```

---

## P3: Node 配置

**文件**: `node/src/chain_spec.rs`

| 行号 | 修改内容 |
|------|----------|
| 10 | `"tokenSymbol": "COS"` → `"tokenSymbol": "COS"` |
| 24 | `"Cosmos Development"` → `"Cosmos Development"` |
| 25 | `"cosmos_dev"` → `"cosmos_dev"` |
| 37 | `"Cosmos Local Testnet"` → `"Cosmos Local Testnet"` |
| 38 | `"cosmos_local"` → `"cosmos_local"` |

**批量替换命令**:
```bash
cd node/src
sed -i 's/"COS"/"COS"/g' chain_spec.rs
sed -i 's/Cosmos/Cosmos/g' chain_spec.rs
sed -i 's/cosmos_/cosmos_/g' chain_spec.rs
```

### ✅ P3 验证点

```bash
cargo build --release
./target/release/solochain-template-node --version
```

---

## P4: 前端代码

### 4.1 核心配置

| 文件 | 修改内容 |
|------|----------|
| `frontend/src/lib/wallet.ts` | 存储键名 `cosmos_*` → `cosmos_*` |
| `frontend/app.json` | `name`, `slug` |

**wallet.ts 替换**:
```bash
cd frontend/src/lib
sed -i "s/cosmos_mnemonic/cosmos_mnemonic/g" wallet.ts
sed -i "s/cosmos_accounts/cosmos_accounts/g" wallet.ts
sed -i "s/cosmos_active_account/cosmos_active_account/g" wallet.ts
sed -i "s/\/\/cosmos\/\//\/\/cosmos\/\//g" wallet.ts
```

### 4.2 UI 页面

| 文件 | 替换模式 |
|------|----------|
| `frontend/app/(tabs)/index.tsx` | `Cosmos` → `Cosmos` |
| `frontend/app/(tabs)/profile.tsx` | `Cosmos` → `Cosmos` |
| `frontend/app/wallet/*.tsx` | `Cosmos Token` → `Cosmos Token` |
| `frontend/app/help/index.tsx` | `COS` → `COS` |
| `frontend/app/legal/*.tsx` | `Cosmos` → `Cosmos`, 邮箱域名 |
| `frontend/app/membership/*.tsx` | `COS` → `COS` |
| `frontend/src/components/CheckInCard.tsx` | `COS` → `COS` |

**批量替换命令**:
```bash
cd frontend
find . -name "*.tsx" -o -name "*.ts" | xargs sed -i 's/Cosmos/Cosmos/g'
find . -name "*.tsx" -o -name "*.ts" | xargs sed -i 's/COS/COS/g'
find . -name "*.tsx" -o -name "*.ts" | xargs sed -i 's/cosmos\.app/cosmos.app/g'
```

### ⚠️ 注意事项

**存储键迁移**：如果链已部署，需要创建前端迁移脚本：

```typescript
// 迁移脚本示例
const MIGRATION_MAP = {
  'cosmos_mnemonic': 'cosmos_mnemonic',
  'cosmos_accounts': 'cosmos_accounts',
  'cosmos_active_account': 'cosmos_active_account',
};

function migrateStorageKeys() {
  for (const [oldKey, newKey] of Object.entries(MIGRATION_MAP)) {
    const value = localStorage.getItem(oldKey);
    if (value) {
      localStorage.setItem(newKey, value);
      localStorage.removeItem(oldKey);
    }
  }
}
```

---

## P5: 文档和元数据

### 5.1 README 文档

```bash
# 批量替换所有 README
find . -name "README.md" | xargs sed -i 's/Cosmos/Cosmos/g'
find . -name "README.md" | xargs sed -i 's/COS/COS/g'
find . -name "README.md" | xargs sed -i 's/StarDust/Cosmos/g'
```

### 5.2 设计文档

| 文件 | 修改 |
|------|------|
| `docs/MOBILE_*.md` | `Cosmos` → `Cosmos` |
| `pallets/*/docs/*.md` | `COS` → `COS` |
| `.kiro/specs/*/` | `cosmos` → `cosmos` |

### 5.3 Cargo.toml 元数据

```bash
# 更新 authors 和 repository
find . -name "Cargo.toml" | xargs sed -i 's/StarDust Team/Cosmos Team/g'
find . -name "Cargo.toml" | xargs sed -i 's/memoio\/cosmos/memoio\/cosmos/g'
```

### 5.4 根 Cargo.toml

```toml
# 修改前
# Cosmos common libraries

# 修改后
# Cosmos common libraries
```

---

## 验证清单

### 编译验证

```bash
# 完整编译
cargo build --release

# 运行测试
cargo test --all

# 检查残留
grep -r "cosmos" --include="*.rs" . | grep -v target | grep -v ".git"
grep -r '"COS"' --include="*.rs" . | grep -v target | grep -v ".git"
```

### 功能验证

- [ ] 节点启动正常：`./target/release/solochain-template-node --dev`
- [ ] Polkadot.js Apps 显示代币符号为 `COS`
- [ ] 前端钱包连接正常
- [ ] 交易发送成功

### 排除项（不修改）

| 项目 | 原因 |
|------|------|
| `type DustRemoval = ()` | Substrate 框架类型名 |
| `pallet_balances::DustRemoval` | 框架配置 |
| `Cargo.lock` | 自动生成 |
| `node_modules/` | 第三方依赖 |
| `target/` | 构建产物 |

---

## 风险提示

### ⚠️ Cosmos 品牌冲突

| 风险 | 等级 | 说明 |
|------|------|------|
| Cosmos SDK 混淆 | **高** | 同名知名区块链生态 |
| COS 符号冲突 | **中** | Contentos 已使用 |
| SEO 劣势 | **中** | 搜索结果被淹没 |

### 💡 替代方案

如需避免冲突：
- **Nebula / NEB**
- **Celestia / CEL**
- **Astral / AST**
- **Nova / NOVA**

---

## 回滚方案

```bash
# 如需回滚，恢复 git 状态
git checkout .
git clean -fd

# 或从备份分支恢复
git checkout main
git branch -D feature/cosmos-rename
```

---

## 时间估算

| 阶段 | 预计耗时 | 风险 |
|------|----------|------|
| P0: Trading Common | 15 分钟 | 低 |
| P1: Trading Pallets | 30 分钟 | 中 |
| P2: Runtime | 20 分钟 | 低 |
| P3: Node | 5 分钟 | 低 |
| P4: 前端 | 30 分钟 | 中 |
| P5: 文档 | 20 分钟 | 低 |
| 验证测试 | 30 分钟 | - |
| **总计** | **~2.5 小时** | - |

---

**文档版本**: v1.0  
**创建日期**: 2026-01-29  
**综合自**: `RENAME_STARCOS_TO_COSMOS.md`, `COSMOS_RENAME_FILE_LIST.md`
