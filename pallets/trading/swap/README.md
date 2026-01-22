# Swap Pallet（做市商兑换模块）

## 模块概述

`pallet-trading-swap` 是 StarDust 链上的做市商兑换服务模块，提供 **DUST → USDT** 的兑换功能。用户可以通过做市商将链上的 DUST 代币兑换为 TRC20 USDT。

### 核心特性

- 🔄 **做市商兑换**：市场化的 DUST → USDT 兑换服务
- 🔍 **OCW 自动验证**：链下工作机（Off-Chain Worker）自动验证 TRC20 交易
- ⏰ **超时退款机制**：做市商未及时完成转账时自动退款给用户
- 🔒 **TRC20 交易哈希防重放**：防止同一笔 USDT 交易被重复使用
- 🚨 **用户举报机制**：用户可举报未履约的做市商，进入仲裁流程

### 版本历史

| 版本 | 日期 | 说明 |
|------|------|------|
| v0.1.0 | 2025-11-03 | 从 pallet-trading 拆分而来 |
| v0.2.0 | 2026-01-18 | 移除官方桥接功能，仅保留做市商兑换 |
| v0.3.0 | 2026-01-18 | 重命名 bridge → swap |
| v0.4.0 | 2026-01-20 | 添加 OCW TRC20 验证机制 |

---

## 核心功能

### 1. 做市商兑换流程

```
用户                    链上                    做市商
 │                       │                       │
 │ ① 创建兑换请求         │                       │
 │ ─────────────────────>│                       │
 │   (锁定 DUST)          │                       │
 │                       │                       │
 │                       │ ② 通知做市商           │
 │                       │ ─────────────────────>│
 │                       │                       │
 │                       │ ③ 转账 USDT (TRC20)   │
 │ <─────────────────────────────────────────────│
 │                       │                       │
 │                       │ ④ 提交交易哈希         │
 │                       │ <─────────────────────│
 │                       │                       │
 │                       │ ⑤ OCW 验证 TRC20 交易  │
 │                       │ ─────────────────────>│
 │                       │                       │
 │                       │ ⑥ 验证成功，释放 DUST  │
 │                       │ ─────────────────────>│
 │                       │                       │
```

### 2. OCW 自动验证

做市商提交 TRC20 交易哈希后，系统通过链下工作机（OCW）自动验证：

- 调用 TronGrid API 查询交易信息
- 验证交易状态（SUCCESS）
- 验证收款地址匹配
- 验证转账金额（允许 0.5% 误差）
- 检查确认数（≥19 个确认）

### 3. 超时退款机制

- **兑换超时**：做市商在规定时间内（默认 1 天）未提交交易哈希，自动退款给用户
- **验证超时**：OCW 验证超时（默认 2 小时）后，自动退款给用户

### 4. 用户举报机制

用户可以对以下状态的兑换发起举报：
- `Pending`：做市商未响应
- `Completed`：对已完成的兑换有异议

举报后进入仲裁流程，由仲裁委员会裁决。

---

## 数据结构

### SwapStatus（兑换状态）

```rust
pub enum SwapStatus {
    /// 待处理 - 等待做市商转账 USDT
    Pending,
    /// 等待验证 - 做市商已提交交易哈希，等待 OCW 验证
    AwaitingVerification,
    /// 已完成 - 验证成功，DUST 已释放给做市商
    Completed,
    /// 验证失败 - OCW 验证 TRC20 交易失败
    VerificationFailed,
    /// 用户举报 - 用户已举报，等待仲裁
    UserReported,
    /// 仲裁中 - 正在进行仲裁
    Arbitrating,
    /// 仲裁通过 - 做市商胜诉
    ArbitrationApproved,
    /// 仲裁拒绝 - 用户胜诉
    ArbitrationRejected,
    /// 已退款 - 超时或仲裁后退款给用户
    Refunded,
}
```

### MakerSwapRecord（做市商兑换记录）

```rust
pub struct MakerSwapRecord<T: Config> {
    /// 兑换ID
    pub swap_id: u64,
    /// 做市商ID
    pub maker_id: u64,
    /// 做市商账户
    pub maker: T::AccountId,
    /// 用户账户
    pub user: T::AccountId,
    /// DUST 数量
    pub dust_amount: BalanceOf<T>,
    /// USDT 金额（精度 10^6）
    pub usdt_amount: u64,
    /// USDT 接收地址（TRC20）
    pub usdt_address: TronAddress,
    /// 创建时间（区块号）
    pub created_at: BlockNumberFor<T>,
    /// 超时时间（区块号）
    pub timeout_at: BlockNumberFor<T>,
    /// TRC20 交易哈希
    pub trc20_tx_hash: Option<BoundedVec<u8, ConstU32<128>>>,
    /// 完成时间（区块号）
    pub completed_at: Option<BlockNumberFor<T>>,
    /// 证据 CID（用于仲裁）
    pub evidence_cid: Option<BoundedVec<u8, ConstU32<256>>>,
    /// 兑换状态
    pub status: SwapStatus,
    /// 兑换价格（精度 10^6）
    pub price_usdt: u64,
}
```

### VerificationRequest（验证请求）

```rust
pub struct VerificationRequest<T: Config> {
    /// 兑换ID
    pub swap_id: u64,
    /// TRC20 交易哈希
    pub tx_hash: BoundedVec<u8, ConstU32<128>>,
    /// 预期收款地址
    pub expected_to: TronAddress,
    /// 预期 USDT 金额（精度 10^6）
    pub expected_amount: u64,
    /// 提交时间（区块号）
    pub submitted_at: BlockNumberFor<T>,
    /// 验证超时时间（区块号）
    pub verification_timeout_at: BlockNumberFor<T>,
    /// 重试次数
    pub retry_count: u8,
}
```

---

## 存储项

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `NextSwapId` | `u64` | 下一个兑换 ID |
| `MakerSwaps` | `Map<u64, MakerSwapRecord>` | 做市商兑换记录（swap_id → 记录） |
| `UserSwaps` | `Map<AccountId, Vec<u64>>` | 用户兑换列表（每用户最多 100 个） |
| `MakerSwapList` | `Map<u64, Vec<u64>>` | 做市商兑换列表（每做市商最多 200 个活跃兑换） |
| `UsedTronTxHashes` | `Map<Vec<u8>, BlockNumber>` | 已使用的 TRC20 交易哈希（防重放，30 天 TTL） |
| `PendingVerifications` | `Map<u64, VerificationRequest>` | 待验证队列 |
| `ArchivedSwapsL1` | `Map<u64, ArchivedSwapL1>` | L1 归档兑换（精简版） |
| `ArchivedSwapsL2` | `Map<u64, ArchivedSwapL2>` | L2 归档兑换（最小版，~16 字节） |
| `SwapStats` | `SwapPermanentStats` | 永久统计数据 |

---

## Extrinsics（可调用函数）

### 1. `maker_swap` - 创建做市商兑换

用户发起 DUST → USDT 兑换请求。

```rust
pub fn maker_swap(
    origin: OriginFor<T>,
    maker_id: u64,           // 做市商 ID
    dust_amount: BalanceOf<T>, // DUST 数量
    usdt_address: Vec<u8>,   // USDT 接收地址（TRC20）
) -> DispatchResult
```

**流程**：
1. 验证兑换金额 ≥ 最小金额（100 DUST）
2. 验证做市商存在且激活
3. 验证 USDT 地址格式（TRC20）
4. 获取当前 DUST/USD 汇率
5. 计算 USDT 金额（至少 1 USDT）
6. 锁定用户的 DUST 到托管
7. 创建兑换记录

### 2. `mark_swap_complete` - 提交 TRC20 交易哈希

做市商完成 USDT 转账后，提交交易哈希。

```rust
pub fn mark_swap_complete(
    origin: OriginFor<T>,
    swap_id: u64,            // 兑换 ID
    trc20_tx_hash: Vec<u8>,  // TRC20 交易哈希
) -> DispatchResult
```

**流程**：
1. 验证调用者是兑换的做市商
2. 验证兑换状态为 `Pending`
3. 检查交易哈希未被使用（防重放）
4. 记录交易哈希
5. 创建验证请求，等待 OCW 验证
6. 更新状态为 `AwaitingVerification`

### 3. `report_swap` - 举报做市商

用户举报未履约的做市商。

```rust
pub fn report_swap(
    origin: OriginFor<T>,
    swap_id: u64,  // 兑换 ID
) -> DispatchResult
```

**流程**：
1. 验证调用者是兑换的用户
2. 验证状态为 `Pending` 或 `Completed`
3. 更新状态为 `UserReported`
4. 进入仲裁流程

### 4. `confirm_verification` - 确认验证结果

由 OCW 或委员会调用，确认 TRC20 交易验证结果。

```rust
pub fn confirm_verification(
    origin: OriginFor<T>,
    swap_id: u64,                  // 兑换 ID
    verified: bool,                // 验证结果
    reason: Option<Vec<u8>>,       // 失败原因
) -> DispatchResult
```

**权限**：仅 `VerificationOrigin`（OCW 或委员会）可调用

### 5. `handle_verification_timeout` - 处理验证超时

任何人可调用，处理验证超时的兑换。

```rust
pub fn handle_verification_timeout(
    origin: OriginFor<T>,
    swap_id: u64,  // 兑换 ID
) -> DispatchResult
```

**流程**：
1. 验证已超过验证超时时间
2. 自动退款给用户
3. 记录做市商超时（影响信用分）

### 6. `ocw_submit_verification` - OCW 提交验证结果

OCW 无签名交易，提交验证结果。

```rust
pub fn ocw_submit_verification(
    origin: OriginFor<T>,
    swap_id: u64,
    verified: bool,
    reason: Option<Vec<u8>>,
) -> DispatchResult
```

**权限**：仅 OCW 可调用（通过 ValidateUnsigned 验证）

---

## 事件

| 事件 | 说明 |
|------|------|
| `MakerSwapCreated` | 做市商兑换已创建 |
| `MakerSwapCompleted` | 做市商兑换已完成 |
| `MakerSwapMarkedComplete` | 做市商已提交交易哈希 |
| `SwapReported` | 用户已举报兑换 |
| `SwapTimeout` | 兑换已超时退款 |
| `VerificationSubmitted` | TRC20 验证已提交，等待验证 |
| `VerificationConfirmed` | TRC20 验证成功，DUST 已释放 |
| `VerificationFailed` | TRC20 验证失败 |
| `VerificationTimeout` | 验证超时，已退款 |

---

## 错误

| 错误 | 说明 |
|------|------|
| `SwapNotFound` | 兑换不存在 |
| `MakerNotFound` | 做市商不存在 |
| `MakerNotActive` | 做市商未激活 |
| `InvalidSwapStatus` | 兑换状态不正确 |
| `NotAuthorized` | 未授权 |
| `StorageLimitReached` | 存储限制已达到 |
| `SwapAmountTooLow` | 兑换金额太低 |
| `InvalidTronAddress` | 无效的 TRON 地址 |
| `AlreadyCompleted` | 兑换已完成 |
| `NotMaker` | 不是做市商 |
| `InvalidStatus` | 状态无效 |
| `InvalidTxHash` | 交易哈希无效 |
| `TooManySwaps` | 兑换太多 |
| `BelowMinimumAmount` | 低于最小金额 |
| `InvalidAddress` | 地址无效 |
| `NotSwapUser` | 不是兑换的用户 |
| `CannotReport` | 无法举报 |
| `PriceNotAvailable` | 价格不可用 |
| `AmountOverflow` | 金额溢出 |
| `UsdtAmountTooSmall` | USDT 金额太小 |
| `TronTxHashAlreadyUsed` | TRON 交易哈希已被使用（防重放） |
| `NotYetTimeout` | 尚未超时 |
| `VerificationNotFound` | 验证请求不存在 |
| `VerificationNotYetTimeout` | 验证尚未超时 |

---

## 配置参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `OcwSwapTimeoutBlocks` | `BlockNumber` | 14400（约 1 天） | 做市商兑换超时时间 |
| `VerificationTimeoutBlocks` | `BlockNumber` | 1200（约 2 小时） | TRC20 验证超时时间 |
| `MinSwapAmount` | `Balance` | 100 DUST | 最小兑换金额 |
| `TxHashTtlBlocks` | `BlockNumber` | 432000（约 30 天） | 交易哈希 TTL（防重放窗口） |

---

## 使用示例

### 用户发起兑换

```rust
// 用户将 1000 DUST 兑换为 USDT
let maker_id = 1;
let dust_amount = 1_000_000_000_000_000u128; // 1000 DUST (12位精度)
let usdt_address = b"TRC20_ADDRESS_HERE".to_vec();

Swap::maker_swap(
    RuntimeOrigin::signed(user),
    maker_id,
    dust_amount,
    usdt_address,
)?;
```

### 做市商完成兑换

```rust
// 做市商转账 USDT 后，提交交易哈希
let swap_id = 1;
let trc20_tx_hash = b"abc123...".to_vec();

Swap::mark_swap_complete(
    RuntimeOrigin::signed(maker),
    swap_id,
    trc20_tx_hash,
)?;
```

### 用户举报

```rust
// 用户举报未履约的做市商
let swap_id = 1;

Swap::report_swap(
    RuntimeOrigin::signed(user),
    swap_id,
)?;
```

### 查询兑换信息

```rust
// 获取兑换详情（含可读时间）
let swap_info = Swap::get_swap_with_time(swap_id);

// 获取用户所有兑换
let user_swaps = Swap::get_user_swaps(&user);

// 获取做市商所有兑换
let maker_swaps = Swap::get_maker_swaps(maker_id);
```

---

## 存储膨胀防护

### 归档机制

模块实现了三级存储归档机制，防止链上存储无限增长：

1. **活跃存储**：`MakerSwaps` - 完整的兑换记录
2. **L1 归档**：`ArchivedSwapsL1` - 精简版（30 天后归档）
3. **L2 归档**：`ArchivedSwapsL2` - 最小版（90 天后归档，~16 字节）

### TTL 清理

- **交易哈希 TTL**：30 天后自动清理已使用的交易哈希
- **on_idle 清理**：利用区块空闲时间进行清理，不影响正常交易

---

## 依赖模块

| 模块 | 用途 |
|------|------|
| `pallet-escrow` | 资金托管服务 |
| `pallet-arbitration` | 仲裁服务 |
| `pallet-trading-common` | 公共类型和接口 |
| `pallet-timestamp` | 时间戳服务 |
| `pallet-storage-lifecycle` | 存储生命周期管理 |
| `pallet-stardust-ipfs` | CID 锁定管理（证据存储） |

---

## 安全考虑

1. **防重放攻击**：每个 TRC20 交易哈希只能使用一次
2. **超时保护**：做市商未及时响应时自动退款给用户
3. **OCW 验证**：自动验证 TRC20 交易真实性
4. **仲裁机制**：争议情况下由仲裁委员会裁决
5. **信用分系统**：记录做市商履约情况，影响其信用评分

---

## License

Unlicense
