# 项目重命名设计文档

## 概述

| 变更项 | 原值 | 新值 |
|--------|------|------|
| 项目名称 | Cosmos | Cosmos |
| 代币符号 | COS | COS |

## 影响范围统计

| 类别 | cosmos 匹配 | COS 匹配 |
|------|---------------|-----------|
| 文件数 | 145个 | 98个 |
| 匹配数 | 355处 | 770处 |

---

## 第一阶段：核心配置文件（必须修改）

### 1.1 链规范配置
**文件**: `node/src/chain_spec.rs`

```rust
// 修改前
fn chain_properties() -> sc_service::Properties {
    json!({
        "tokenSymbol": "COS",  // ← 改为 "COS"
        ...
    })
}

.with_name("Cosmos Development")   // ← 改为 "Cosmos Development"
.with_id("cosmos_dev")             // ← 改为 "cosmos_dev"

.with_name("Cosmos Local Testnet") // ← 改为 "Cosmos Local Testnet"
.with_id("cosmos_local")           // ← 改为 "cosmos_local"
```

### 1.2 Runtime 版本信息
**文件**: `runtime/src/lib.rs`

```rust
// 修改前
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: alloc::borrow::Cow::Borrowed("cosmos"),  // ← 改为 "cosmos"
    impl_name: alloc::borrow::Cow::Borrowed("cosmos"),  // ← 改为 "cosmos"
    ...
};
```

### 1.3 创世配置
**文件**: `runtime/src/genesis_config_presets.rs`

```rust
// 修改前
/// Total initial supply: 100,000,000,000 COS  // ← 改为 COS
const INITIAL_SUPPLY: u128 = 100_000_000_000 * UNIT;
```

### 1.4 根 Cargo.toml
**文件**: `Cargo.toml`

```toml
# 修改前
# Cosmos common libraries  # ← 改为 Cosmos
media-utils = { path = "./media-utils", default-features = false }
```

---

## 第二阶段：Runtime 配置注释

### 2.1 文件: `runtime/src/configs/mod.rs`

需要修改的注释（COS → COS）：

| 行号 | 原内容 | 新内容 |
|------|--------|--------|
| 269 | `// 最低保证金 10 COS` | `// 最低保证金 10 COS` |
| 273 | `// 最大服务价格 1亿 COS` | `// 最大服务价格 1亿 COS` |
| 595 | `// 创建群组保证金兜底值 50 COS` | `// 创建群组保证金兜底值 50 COS` |
| 637 | `// 最小提现 1 COS` | `// 最小提现 1 COS` |
| 638 | `// 保证金兜底值 0.05 COS` | `// 保证金兜底值 0.05 COS` |
| 779 | `// 最小兑换10 COS` | `// 最小兑换10 COS` |
| 862 | `// 最小1 COS` | `// 最小1 COS` |
| 863 | `// 最大1亿COS` | `// 最大1亿COS` |
| 870 | `// 最小押金 0.1 COS` | `// 最小押金 0.1 COS` |
| 927-928 | `COS 才有资格` / `COS/USDT 价格` | `COS 才有资格` / `COS/USDT 价格` |
| 936 | `// 获取 COS/USDT 价格` | `// 获取 COS/USDT 价格` |
| 944-949 | `COS 数量` / `min_dust` / `COS 精度` | `COS 数量` / `min_cos` / `COS 精度` |
| 953 | `balance >= min_dust` | `balance >= min_cos` |
| 1261 | `// 投诉押金兜底值 0.1 COS` | `// 投诉押金兜底值 0.1 COS` |
| 1462 | `// 最低 1 COS` | `// 最低 1 COS` |
| 1497 | `// 兜底值 10 COS` | `// 兜底值 10 COS` |
| 1499 | `// 兜底值 500 COS` | `// 兜底值 500 COS` |
| 1502 | `// 兜底值 500 COS` | `// 兜底值 500 COS` |
| 1504 | `// 兜底值 20 COS` | `// 兜底值 20 COS` |

---

## 第三阶段：Trading Pallets 核心代码

### 3.1 PricingProvider Trait
**文件**: `pallets/trading/common/src/traits.rs`

```rust
// 修改前
pub trait PricingProvider<Balance> {
    /// 获取 COS/USD 汇率                    // ← 改为 COS/USD
    fn get_dust_to_usd_rate() -> Option<Balance>;  // ← 改为 get_cos_to_usd_rate
    
    /// - `dust_qty`: COS 数量              // ← 改为 COS
    fn report_swap_order(timestamp: u64, price_usdt: u64, dust_qty: u128) -> ...;
                                                    // ↑ 改为 cos_qty
}

// 空实现也需要同步修改
impl<Balance> PricingProvider<Balance> for () {
    fn get_dust_to_usd_rate() -> Option<Balance> { ... }  // ← 改函数名
    fn report_swap_order(..., _dust_qty: u128) -> ... { ... }  // ← 改参数名
}
```

**影响的注释**（同文件）：
- 行 14: `提供 COS/USD 实时汇率` → `提供 COS/USD 实时汇率`
- 行 24-28: 所有 `COS` → `COS`
- 行 36: `dust_qty: COS 数量` → `cos_qty: COS 数量`
- 行 255-268: `DepositCalculator` 注释中所有 `COS` → `COS`
- 行 288-296: 变量名 `dust_precision`, `dust_amount_u128` → `cos_precision`, `cos_amount_u128`

### 3.2 Pricing Pallet
**文件**: `pallets/trading/pricing/src/lib.rs`

```rust
// 修改前
pub fn get_dust_market_price_weighted() -> u64 { ... }  // ← 改为 get_cos_market_price_weighted

// 注释修改
/// - `u64`: USDT/COS 价格  // ← 改为 USDT/COS
```

### 3.3 OTC Pallet
**文件**: `pallets/trading/otc/src/lib.rs`

需要修改的调用（约 10 处）：
```rust
// 修改前
let price = T::Pricing::get_dust_to_usd_rate()  // ← 改为 get_cos_to_usd_rate

// Config 类型名
type MinFirstPurchaseDustAmount  // ← 改为 MinFirstPurchaseCosAmount
type MaxFirstPurchaseDustAmount  // ← 改为 MaxFirstPurchaseCosAmount
```

### 3.4 Swap Pallet
**文件**: `pallets/trading/swap/src/lib.rs`

```rust
// 修改前
let price_balance = T::Pricing::get_dust_to_usd_rate()  // ← 改为 get_cos_to_usd_rate
```

### 3.5 Maker Pallet
**文件**: `pallets/trading/maker/src/lib.rs`

```rust
// 修改前
pub fn calculate_dust_amount_for_usd(...) -> ... {  // ← 改为 calculate_cos_amount_for_usd
    let dust_to_usd_rate = T::Pricing::get_dust_to_usd_rate()  // ← 改变量名和函数名
}
```

---

## 第四阶段：其他 Pallets

### 4.1 需要修改的 Pallet 列表

| Pallet | 文件 | 修改类型 |
|--------|------|----------|
| divination/common | src/deposit.rs | 注释中 COS → COS |
| divination/membership | src/lib.rs, tests.rs | 注释和测试数据 |
| affiliate | src/mock.rs, README.md | 注释和文档 |
| storage-service | README.md | 文档 |
| referral | src/mock.rs | 测试数据注释 |
| chat/group | src/storage.rs, crypto.rs | 注释 |
| livestream | REPORT_*.md | 文档 |
| matchmaking/profile | README.md, src/lib.rs | 文档和注释 |
| matchmaking/membership | src/lib.rs, tests.rs | 注释 |

---

## 第五阶段：文档和脚本

### 5.1 README 文件
| 文件路径 | 修改内容 |
|----------|----------|
| `README.md` | 项目名称（如有） |
| `script/README.md` | cosmos → cosmos |
| `pallets/*/README.md` | COS → COS |
| `media-utils/README.md` | cosmos → cosmos |
| `docs/*.md` | 所有引用 |

### 5.2 脚本文件
| 文件路径 | 修改内容 |
|----------|----------|
| `script/test-*.ts` | 注释和日志中的 cosmos/COS |

### 5.3 前端代码
| 文件路径 | 修改内容 |
|----------|----------|
| `frontend/src/lib/wallet.ts` | cosmos 相关配置 |
| `frontend/app/legal/privacy.tsx` | 品牌名称 |
| `frontend/app/legal/terms.tsx` | 品牌名称 |
| `frontend/app/(tabs)/index.tsx` | 显示名称 |

---

## 第六阶段：Cargo.toml 元数据

### 6.1 需要修改的 Cargo.toml
| 文件路径 | 修改字段 |
|----------|----------|
| `pallets/trading/common/Cargo.toml` | `authors`, `repository` |
| `pallets/chat/group/Cargo.toml` | 如有 cosmos 引用 |
| `pallets/divination/bazi/Cargo.toml` | 如有 cosmos 引用 |
| `pallets/divination/membership/Cargo.toml` | 如有 cosmos 引用 |
| `media-utils/Cargo.toml` | 如有 cosmos 引用 |

---

## 风险评估

### ⚠️ 命名冲突风险

| 风险项 | 严重程度 | 说明 |
|--------|----------|------|
| Cosmos SDK 品牌冲突 | **高** | Cosmos 是知名区块链生态，可能导致混淆 |
| COS 代币符号冲突 | **中** | Contentos (COS) 已存在 |
| SEO 劣势 | **中** | 搜索"cosmos blockchain"会被淹没 |

### 💡 替代建议

如需避免冲突，建议考虑：
- **Nebula** / **NEB** - 星云主题
- **Celestia** / **CEL** - 天体主题（需确认商标）
- **Astral** / **AST** - 星界主题

---

## 执行顺序建议

1. **阶段一**：核心配置（chain_spec, runtime）- **必须首先完成**
2. **阶段三**：Trading Pallets - **编译依赖**
3. **阶段二**：Runtime 配置注释 - 可并行
4. **阶段四**：其他 Pallets - 可并行
5. **阶段五**：文档和脚本 - 最后完成
6. **阶段六**：Cargo.toml 元数据 - 可选

---

## 验证清单

- [ ] `cargo build --release` 编译通过
- [ ] `cargo test` 所有测试通过
- [ ] `grep -r "cosmos" --include="*.rs"` 无遗漏
- [ ] `grep -r "COS" --include="*.rs"` 无遗漏（注意排除 `DustRemoval` 等框架类型）
- [ ] 前端钱包连接正常显示 COS
- [ ] Polkadot.js Apps 显示正确代币符号

---

## 注意事项

1. **不要修改** `type DustRemoval = ()` - 这是 Substrate 框架的类型名，与代币无关
2. **保留** `pallet_balances` 中的 `DustRemoval` 配置
3. **测试文件中的常量** 如 `const COS: u128 = ...` 需要改为 `const COS: u128 = ...`
4. **Git 分支建议**：在新分支进行全部修改，充分测试后再合并
