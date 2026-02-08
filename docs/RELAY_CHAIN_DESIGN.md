# 独立中继链设计方案

> 基于 Polkadot SDK 从零构建独立中继链，支持平行链接入、共享安全、XCM 跨链。

---

## 1. 项目定位

从零创建**完全独立的中继链**（非 Cosmos 升级），具备：

- 自主验证人网络，提供共享安全
- 多条平行链接入，区块验证与终局性
- XCM 跨链消息协议
- 独立经济模型（原生代币、质押、通胀）
- 不依赖 Polkadot/Kusama 网络

| 维度 | Polkadot | 本项目 |
|------|----------|--------|
| 网络 | 公共中继链 | 独立中继链 |
| 验证人 | 数百个 NPoS | 初期 4-20，逐步扩展 |
| 平行链接入 | 拍卖插槽 | 治理注册（初期） |
| 治理 | OpenGov | Sudo → Council（渐进） |

---

## 2. 技术选型

**SDK 版本：** polkadot-sdk stable2409 或更新稳定版

```toml
[workspace.dependencies]
polkadot-sdk = { git = "https://github.com/nickelshack/polkadot-sdk", tag = "stable2409" }
```

| 组件 | 选型 | 说明 |
|------|------|------|
| 出块共识 | BABE | VRF 随机出块 |
| 终局性 | GRANDPA | 拜占庭容错 |
| 验证人选举 | NPoS | Phragmén 算法 |
| 平行链验证 | polkadot-runtime-parachains | 核心平行链逻辑 |
| 跨链协议 | XCM v4 | 跨共识消息 |
| 网络层 | libp2p | 默认 |
| 数据库 | RocksDB | 状态存储 |

---

## 3. 项目结构

```
my-relay-chain/
├── Cargo.toml                      # Workspace
├── rust-toolchain.toml
│
├── relay/
│   ├── runtime/                    # 中继链 Runtime
│   │   ├── src/
│   │   │   ├── lib.rs              # pallet 注册、类型定义
│   │   │   ├── xcm_config.rs       # XCM 执行器与路由
│   │   │   ├── genesis.rs          # 创世配置
│   │   │   └── weights/
│   │   ├── Cargo.toml
│   │   └── build.rs
│   └── node/                       # 中继链节点
│       ├── src/
│       │   ├── main.rs
│       │   ├── service.rs          # BABE + GRANDPA + Overseer
│       │   ├── chain_spec.rs       # Dev/Local/Testnet
│       │   ├── cli.rs
│       │   └── rpc.rs
│       └── Cargo.toml
│
├── primitives/                     # 共享原语
│   └── src/lib.rs                  # AccountId, Balance, SessionKeys 等
│
├── parachain-template/             # 平行链模板
│   ├── runtime/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── xcm_config.rs
│   │   └── Cargo.toml
│   └── node/
│       ├── src/
│       │   ├── main.rs
│       │   └── service.rs          # Cumulus collator
│       └── Cargo.toml
│
├── zombienet/                      # 测试网编排
│   ├── relay-local.toml
│   └── relay-with-parachains.toml
│
├── docker/
│   ├── Dockerfile.relay
│   └── Dockerfile.parachain
│
└── scripts/
    ├── build.sh
    └── launch-local.sh
```

---

## 4. 共享原语 (`primitives/`)

```rust
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Signature = sp_runtime::MultiSignature;
pub type Balance = u128;
pub type BlockNumber = u32;
pub type Hash = sp_core::H256;
pub type Nonce = u32;
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
pub type Block = generic::Block<Header, OpaqueExtrinsic>;

pub const TOKEN_SYMBOL: &str = "RELAY";
pub const TOKEN_DECIMALS: u32 = 12;
pub const UNIT: Balance = 1_000_000_000_000;
pub const EXISTENTIAL_DEPOSIT: Balance = UNIT / 1_000;

pub const BLOCK_TIME_MS: u64 = 6_000;
pub const MINUTES: BlockNumber = 60_000 / (BLOCK_TIME_MS as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

pub const EPOCH_DURATION_IN_SLOTS: BlockNumber = 4 * HOURS;
pub const SESSIONS_PER_ERA: u32 = 6; // 1 Era ≈ 24h
```

---

## 5. 中继链 Runtime Pallet 清单

### 5.1 基础系统 (index 0-9)

| Index | Pallet | 说明 |
|:---:|--------|------|
| 0 | `frame_system` | 核心框架 |
| 1 | `pallet_timestamp` | 时间戳 |
| 2 | `pallet_babe` | BABE 出块 |
| 3 | `pallet_grandpa` | GRANDPA 终局性 |
| 4 | `pallet_balances` | 余额管理 |
| 5 | `pallet_transaction_payment` | 交易费 |
| 6 | `pallet_sudo` | 超级用户（初期） |
| 7 | `pallet_authorship` | 出块人追踪 |
| 8 | `pallet_utility` | 批量调用 |

### 5.2 验证人与质押 (index 10-19)

| Index | Pallet | 说明 |
|:---:|--------|------|
| 10 | `pallet_session` | 会话管理 |
| 11 | `pallet_staking` | NPoS 质押 |
| 12 | `pallet_authority_discovery` | 验证人发现 |
| 13 | `pallet_offences` | 违规记录 |
| 14 | `pallet_historical` | 历史会话 |
| 15 | `pallet_election_provider_multi_phase` | NPoS 选举 |
| 16 | `pallet_bags_list` | 提名人排序 |

### 5.3 治理 (index 20-29)

| Index | Pallet | 说明 |
|:---:|--------|------|
| 20 | `pallet_collective` | 理事会 |
| 21 | `pallet_membership` | 理事会成员 |
| 22 | `pallet_treasury` | 国库 |

### 5.4 平行链管理 (index 50-69)

| Index | Pallet | 说明 |
|:---:|--------|------|
| 50 | `parachains_configuration` | 全局配置 |
| 51 | `parachains_shared` | 共享状态 |
| 52 | `parachains_inclusion` | 候选纳入 |
| 53 | `parachains_paras_inherent` | Inherent 数据 |
| 54 | `parachains_scheduler` | 核心调度 |
| 55 | `parachains_paras` | 平行链注册/生命周期 |
| 56 | `parachains_initializer` | 会话初始化 |
| 57 | `parachains_dmp` | 下行消息 |
| 58 | `parachains_hrmp` | 水平中继消息 |
| 59 | `parachains_session_info` | 会话信息 |
| 60 | `parachains_disputes` | 争议处理 |
| 61 | `parachains_slashing` | 争议惩罚 |
| 62 | `pallet_message_queue` | 消息队列 |
| 63 | `paras_registrar` | 注册器 |
| 64 | `slots` | 插槽管理（可选） |

### 5.5 XCM (index 70+)

| Index | Pallet | 说明 |
|:---:|--------|------|
| 70 | `pallet_xcm` | XCM 接口 |
| 71 | `parachains_origin` | 平行链 Origin |

---

## 6. 核心配置参数

### 6.1 共识

| 参数 | 值 | 说明 |
|------|-----|------|
| 出块时间 | 6 秒 | BABE slot duration |
| Epoch 时长 | 4 小时 (2400 slots) | VRF 密钥轮换周期 |
| MaxAuthorities | 100 | 最大验证人数 |

### 6.2 Staking

| 参数 | 值 | 说明 |
|------|-----|------|
| Session 周期 | 4 小时 | 验证人集合轮换 |
| Era 周期 | 24 小时 (6 session) | 奖励分发周期 |
| 解绑期 | 28 Era ≈ 28 天 | 防攻击后撤出 |
| 惩罚延迟 | 14 Era ≈ 14 天 | 给治理时间撤销误判 |
| 最低验证人质押 | 10,000 RELAY | 验证人门槛 |
| 最低提名质押 | 100 RELAY | 提名人门槛 |
| 验证人上限 | 100 | 根据平行链数调整 |
| 理想质押率 | 50% | 通胀曲线最优点 |
| 通胀范围 | 2%-10% 年化 | 质押率低→高通胀激励质押 |

### 6.3 平行链

| 参数 | 值 | 说明 |
|------|-----|------|
| max_code_size | 5 MB | WASM 代码上限 |
| max_head_data_size | 32 KB | 区块头上限 |
| max_pov_size | 5 MB | PoV 块上限 |
| group_rotation_frequency | 20 blocks | 验证组轮换 |
| paras_availability_period | 4 blocks | 可用性超时 |
| max_validators_per_core | 5 | 每核心验证人 |
| hrmp_channel_max_capacity | 1000 | HRMP 消息容量 |
| hrmp_channel_max_message_size | 100 KB | 单条消息上限 |
| 注册押金 | 100 RELAY | 平行链注册押金 |

---

## 7. Session Keys

每个验证人需要 5 种密钥：

```rust
impl_opaque_keys! {
    pub struct SessionKeys {
        pub babe: Babe,                          // Sr25519 — 出块 VRF
        pub grandpa: Grandpa,                    // Ed25519 — 终局性投票
        pub para_validator: Initializer,         // Sr25519 — 平行链验证
        pub para_assignment: ParaSessionInfo,    // Sr25519 — 核心分配
        pub authority_discovery: AuthorityDiscovery, // Sr25519 — 节点发现
    }
}
```

---

## 8. 节点服务层

### 8.1 与单链节点的差异

| 组件 | 单链 | 中继链 |
|------|------|--------|
| 出块 | Aura (轮询) | BABE (VRF) |
| 终局性 | GRANDPA | GRANDPA (保留) |
| 链选择 | LongestChain | SelectRelayChain (平行链感知) |
| 子系统 | 无 | Overseer + 20 子系统 |
| 网络协议 | 基础 p2p | + Collation + Validation |
| 存储 | 单链 DB | + PoV 可用性存储 |

### 8.2 Overseer 子系统

中继链核心 — 管理平行链验证的 20+ 子系统：

- **CandidateValidation** — 验证平行链候选区块
- **CandidateBacking** — 背书候选区块
- **StatementDistribution** — 分发背书声明
- **AvailabilityDistribution** — 分发可用性数据
- **AvailabilityRecovery** — 恢复可用性数据
- **BitfieldDistribution/Signing** — 可用性位域
- **Provisioner** — 为出块人提供平行链数据
- **CollatorProtocol** — 与整理人通信
- **ApprovalVoting/Distribution** — 审批投票
- **DisputeCoordinator/Distribution** — 争议处理
- **ChainSelection** — 含平行链分叉的链选择
- **GossipSupport** — 网络拓扑
- **NetworkBridge Rx/Tx** — 网络桥接

---

## 9. XCM 跨链设计

### 9.1 消息流向

| 方向 | 协议 | 场景 |
|------|------|------|
| Relay → Para | DMP (下行) | 配置下发、治理指令 |
| Para → Relay | UMP (上行) | 资产转回、治理参与 |
| Para ↔ Para | HRMP (水平) | 平行链间资产/消息互通 |

### 9.2 中继链 XCM 角色

- **储备链**：原生代币 RELAY 的储备所有权在中继链
- **路由器**：平行链间 HRMP 消息通过中继链路由
- **不执行业务逻辑**：中继链只处理资产转移和系统级 XCM

### 9.3 核心配置

```rust
// XCM 路由器 — 发送到子平行链
pub type XcmRouter = xcm_sender::ChildParachainRouter<Runtime, XcmPallet, ()>;

// Barrier — 谁能在中继链执行 XCM
pub type Barrier = (
    TakeWeightCredit,                          // 本地 XCM 免费
    AllowTopLevelPaidExecutionFrom<Everything>, // 外部 XCM 收费
    AllowSubscriptionsFrom<Everything>,         // 版本订阅
);

// 资产处理 — FungibleAdapter 处理原生代币
pub type LocalAssetTransactor = FungibleAdapter<
    Balances,
    IsConcrete<RelayTokenLocation>,
    LocationToAccountId,
    AccountId,
    (),
>;
```

---

## 10. 平行链模板

提供标准平行链模板，基于 Cumulus：

```rust
// parachain-template/runtime — 核心 pallet
#[runtime::pallet_index(0)]  pub type System = frame_system;        // ParachainDefaultConfig
#[runtime::pallet_index(1)]  pub type ParachainSystem = cumulus_pallet_parachain_system;
#[runtime::pallet_index(2)]  pub type ParachainInfo = parachain_info;
#[runtime::pallet_index(3)]  pub type Timestamp = pallet_timestamp;
#[runtime::pallet_index(4)]  pub type Balances = pallet_balances;
#[runtime::pallet_index(5)]  pub type TransactionPayment = pallet_transaction_payment;
#[runtime::pallet_index(10)] pub type Aura = pallet_aura;            // 平行链用 Aura 出块
#[runtime::pallet_index(11)] pub type AuraExt = cumulus_pallet_aura_ext;
#[runtime::pallet_index(20)] pub type XcmpQueue = cumulus_pallet_xcmp_queue;
#[runtime::pallet_index(21)] pub type CumulusXcm = cumulus_pallet_xcm;
#[runtime::pallet_index(22)] pub type MessageQueue = pallet_message_queue;
```

**平行链节点**使用 Cumulus collator 服务，不需要 GRANDPA（终局性由中继链保障）。

---

## 11. 经济模型

### 11.1 代币

| 属性 | 值 |
|------|-----|
| 符号 | RELAY |
| 精度 | 12 位小数 |
| 初始供应 | 100,000,000 RELAY |
| 存在性押金 | 0.001 RELAY |

### 11.2 通胀曲线

```
通胀率
10% ┤·
    │  ·
    │    ·
 5% ┤      · (最优点: 50% 质押率)
    │        ·
 2% ┤          · · · · · · ·
    └──────────────────────────
    0%       50%          100%  ← 全网质押率
```

- 质押率低于 50% → 高通胀激励质押
- 质押率 50% → 5% 通胀（最优平衡点）
- 质押率高于 50% → 通胀降低，释放流动性

### 11.3 收入分配

| 来源 | 流向 |
|------|------|
| 区块奖励（通胀） | 验证人 + 提名人（按质押比例） |
| 交易手续费 80% | 销毁（通缩） |
| 交易手续费 20% | 国库 |
| 平行链注册押金 | 锁定（注销时退还） |

---

## 12. 实施路线图

### Phase 1：基础中继链（4-6 周）

- [ ] 创建项目骨架（workspace、primitives、relay/runtime、relay/node）
- [ ] 配置 BABE + GRANDPA + Session + Staking
- [ ] 配置基础 pallet（System, Balances, TransactionPayment, Sudo）
- [ ] Genesis config presets（dev/local/testnet）
- [ ] 节点服务层（BABE import queue, GRANDPA voter）
- [ ] 本地 4 验证人启动测试
- [ ] Staking 质押/提名/奖励流程验证

### Phase 2：平行链支持（4-6 周）

- [ ] 集成 polkadot-runtime-parachains（全套 13 个模块）
- [ ] 集成 Overseer + 子系统
- [ ] 配置 Registrar（治理注册）
- [ ] HRMP/DMP/UMP 消息通道
- [ ] 创建平行链模板（Cumulus collator）
- [ ] 注册测试平行链，验证出块 + 终局性

### Phase 3：XCM 跨链（2-3 周）

- [ ] 中继链 XCM 执行器配置
- [ ] 平行链 XCM 配置
- [ ] 中继链 ↔ 平行链代币转移测试
- [ ] 平行链 ↔ 平行链 HRMP 测试
- [ ] XCM emulator 集成测试

### Phase 4：生产化（3-4 周）

- [ ] 移除 Sudo → Council 治理
- [ ] Staking slash 机制验证
- [ ] 安全审计 XCM 配置
- [ ] Zombienet 测试网编排
- [ ] Docker 镜像 + 部署文档
- [ ] 监控面板（Prometheus + Grafana）

**总工期：13-19 周（约 3.5-5 个月）**

---

## 13. 核心依赖清单

```toml
# === 基础框架 ===
frame-support = "45.0.0"
frame-system = "45.0.0"
frame-executive = "45.0.0"
sp-runtime = "45.0.0"
sp-core = "39.0.0"
sp-io = "44.0.0"
codec = { package = "parity-scale-codec", version = "3.7" }
scale-info = "2.11"

# === 共识 ===
pallet-babe = "45.0.0"
pallet-grandpa = "45.0.0"
sc-consensus-babe = "0.52.0"
sc-consensus-grandpa = "0.40.0"
sp-consensus-babe = "0.46.0"
sp-consensus-grandpa = "27.0.0"

# === 质押 ===
pallet-session = "45.0.0"
pallet-staking = "45.0.0"
pallet-authority-discovery = "45.0.0"
pallet-offences = "44.0.0"
pallet-election-provider-multi-phase = "44.0.0"
pallet-bags-list = "44.0.0"
frame-election-provider-support = "45.0.0"

# === 平行链 ===
polkadot-runtime-parachains = "17.0.0"
polkadot-primitives = "15.0.0"
polkadot-parachain-primitives = "14.0.0"

# === 节点 Overseer ===
polkadot-overseer = "17.0.0"
polkadot-node-core-candidate-validation = "..."
polkadot-node-core-backing = "..."
# ... 其余 Overseer 子系统 crate

# === XCM ===
staging-xcm = "14.0.0"
staging-xcm-builder = "17.0.0"
staging-xcm-executor = "17.0.0"
pallet-xcm = "17.0.0"

# === Cumulus（平行链模板用）===
cumulus-pallet-parachain-system = "0.17.0"
cumulus-pallet-xcm = "0.17.0"
cumulus-pallet-xcmp-queue = "0.17.0"
cumulus-client-consensus-aura = "0.17.0"
cumulus-client-collator = "0.17.0"
cumulus-primitives-core = "0.17.0"

# === 通用 ===
pallet-balances = "46.0.0"
pallet-timestamp = "44.0.0"
pallet-transaction-payment = "45.0.0"
pallet-sudo = "45.0.0"
pallet-utility = "45.0.0"
pallet-collective = "45.0.0"
pallet-treasury = "44.0.0"
pallet-authorship = "45.0.0"
pallet-message-queue = "48.0.0"
```

> ⚠️ 版本号需要根据选定的 polkadot-sdk tag 精确对齐。建议直接使用 git 依赖统一 tag。

---

## 14. 风险评估

| 风险 | 等级 | 缓解 |
|------|:---:|------|
| SDK 版本对齐 | 🔴高 | 统一使用 polkadot-sdk git tag |
| Overseer 复杂度 | 🔴高 | 参考 Rococo 实现逐步集成 |
| 验证人运维 | 🟡中 | Docker + 自动化部署 |
| XCM 安全 | 🟡中 | 严格 Barrier，分阶段开放 |
| 经济模型调优 | 🟡中 | 先 testnet 长时间验证 |

---

## 15. 参考实现

| 项目 | 说明 |
|------|------|
| [Rococo Runtime](https://github.com/nickelshack/polkadot-sdk/tree/master/polkadot/runtime/rococo) | 测试中继链，最佳起步参考 |
| [Polkadot Runtime](https://github.com/polkadot-fellows/runtimes) | 生产中继链，完整但复杂 |
| [Parachain Template](https://github.com/nickelshack/polkadot-sdk/tree/master/templates/parachain) | 官方平行链模板 |
| [Zombienet](https://github.com/nickelshack/zombienet) | 多链测试网编排 |
| [Paseo Network](https://github.com/paseo-network) | 社区测试中继链 |
