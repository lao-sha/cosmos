# Node Operator Agent

LLM 驱动的 Substrate 节点运维助手 - MVP 概念验证版本。

## 概述

使用大语言模型（Claude/GPT-4）自动化节点运维任务：

- 🔍 **节点诊断** - 自动检测常见问题
- 📊 **状态监控** - 获取同步状态、对等节点等信息
- 📝 **日志分析** - 智能分析节点日志
- ⚙️ **配置建议** - 根据节点类型生成最佳配置

## 快速开始

### 1. 配置环境变量

```bash
cp .env.example .env
# 编辑 .env 填入你的 API KEY
```

### 2. 编译

```bash
cd node-operator
cargo build --release
```

### 3. 使用

```bash
# 单次对话
./target/release/node-operator chat "检查节点状态"

# 交互式模式
./target/release/node-operator interactive

# 快速诊断
./target/release/node-operator diagnose --endpoint http://127.0.0.1:9944

# 查看可用工具
./target/release/node-operator tools
```

## 可用工具

| 工具 | 描述 |
|------|------|
| `get_node_status` | 获取节点运行状态 |
| `diagnose_node` | 诊断节点问题 |
| `get_node_logs` | 获取节点日志 |
| `generate_chain_spec` | 生成配置建议 |
| `check_system_resources` | 检查系统资源 |
| `list_nodes` | 列出管理的节点 |

## 使用示例

### 诊断节点问题

```
👤 You: 我的节点同步很慢，请帮我诊断一下

🤖 Agent: 让我检查一下节点状态...

经过诊断，发现以下问题：
1. ⚠️ 对等节点数量较少（仅 2 个），建议检查网络配置
2. ✓ RPC 端口正常响应
3. ✓ 磁盘空间充足

建议操作：
- 添加更多 bootnodes 到配置中
- 检查防火墙是否阻止了 P2P 端口（默认 30333）
- 考虑增加 --out-peers 参数值
```

### 生成 Validator 配置

```
👤 You: 我想部署一个 validator 节点，需要什么配置？

🤖 Agent: 为您生成 Validator 节点配置建议...

硬件要求：
- CPU: 8+ 核心
- 内存: 32GB+
- 存储: 1TB+ NVMe SSD
- 网络: 1Gbps

推荐配置：
- pruning: archive
- validator: true
- rpc_external: false（安全考虑）

安全建议：
1. 禁用外部 RPC/WS 访问
2. 使用防火墙限制入站连接
3. 定期备份 keystore
```

## 架构

```
┌─────────────────────────────────────────────────────────────┐
│                     Node Operator Agent                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                    LLM Client                         │  │
│  │         (Claude API / OpenAI API)                     │  │
│  └───────────────────────────────────────────────────────┘  │
│                            │                                │
│                            ▼                                │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                    Tool Registry                       │  │
│  │                                                       │  │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐     │  │
│  │  │ Status  │ │Diagnose │ │  Logs   │ │ Config  │     │  │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘     │  │
│  └───────────────────────────────────────────────────────┘  │
│                            │                                │
│                            ▼                                │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Substrate Node (RPC)                      │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 安全设计

### 当前安全措施

1. **工具白名单** - LLM 只能调用预定义的安全工具
2. **只读操作** - MVP 版本只支持查询操作，不执行变更
3. **无命令执行** - 不支持执行任意 shell 命令
4. **API KEY 隔离** - 通过环境变量配置，不硬编码

### 生产环境建议

1. **TEE 保护** - 使用 Intel SGX 保护 API KEY
2. **操作审批** - 危险操作需人工确认
3. **审计日志** - 记录所有工具调用
4. **Rate Limiting** - 限制 API 调用频率

## 后续开发计划

### Phase 2 - 配置管理
- [ ] 生成 Docker Compose / K8s YAML
- [ ] 节点启动命令生成
- [ ] 多节点批量操作

### Phase 3 - 自动化运维
- [ ] 监控告警集成
- [ ] 自动扩缩容建议
- [ ] 故障自动恢复（需审批）

### Phase 4 - 生产加固
- [ ] TEE 集成
- [ ] 操作审批流程
- [ ] 完整审计日志

## 远程部署功能

使用 `-r` 或 `--remote` 标志启用远程部署工具：

```bash
# 启用远程模式的交互式对话
./target/release/node-operator -r interactive

# 列出配置的服务器
./target/release/node-operator servers

# 远程部署节点
./target/release/node-operator deploy --server prod1 --chain nexus --node-type validator

# 列出所有工具（含远程工具）
./target/release/node-operator -r tools
```

### 远程工具列表

| 工具 | 描述 | 风险级别 |
|------|------|----------|
| `list_servers` | 列出配置的服务器 | 安全 |
| `remote_status` | 检查远程服务器状态 | 安全 |
| `ssh_execute` | 执行远程命令 | 需审批 |
| `deploy_node` | 部署节点 | 高风险 |
| `start_node` | 启动节点服务 | 中风险 |
| `stop_node` | 停止节点服务 | 高风险 |
| `generate_ansible_playbook` | 生成 Ansible 脚本 | 安全 |

### 审批机制

危险操作会触发审批提示：

```
╔══════════════════════════════════════════════════════════════╗
║                    ⚠️  操作审批请求                            ║
╠══════════════════════════════════════════════════════════════╣
║ 操作ID:   a1b2c3d4                                           ║
║ 类型:     deploy_node                                        ║
║ 目标:     prod-server-1                                      ║
║ 风险级别: 高风险 - 可能导致数据丢失                            ║
╠══════════════════════════════════════════════════════════════╣
║ 描述: Deploy nexus validator node on prod-server-1         ║
╚══════════════════════════════════════════════════════════════╝

是否批准此操作? [y/N]:
```

## 环境变量

| 变量 | 描述 | 默认值 |
|------|------|--------|
| `LLM_PROVIDER` | LLM 提供商 (claude/openai/deepseek) | claude |
| `ANTHROPIC_API_KEY` | Claude API 密钥 | - |
| `OPENAI_API_KEY` | OpenAI API 密钥 | - |
| `DEEPSEEK_API_KEY` | DeepSeek API 密钥 | - |
| `DEEPSEEK_BASE_URL` | DeepSeek API 地址 | api.deepseek.com |
| `DEEPSEEK_MODEL` | DeepSeek 模型 | deepseek-chat |
| `MANAGED_NODES` | 管理的节点列表（逗号分隔） | localhost:9944 |
| `SSH_SERVERS` | 远程服务器配置 | - |
| `SSH_KEY_PATH` | SSH 私钥路径 | ~/.ssh/id_rsa |
| `RUST_LOG` | 日志级别 | node_operator=info |

### SSH 服务器配置格式

```bash
# 格式: name:host:user,name:host:user,...
export SSH_SERVERS="prod1:192.168.1.10:root,prod2:192.168.1.11:ubuntu"
export SSH_KEY_PATH="/home/user/.ssh/id_rsa"
```

## 云服务器自动购买

支持自动购买 **Vultr**、**DigitalOcean** 或 **LNVPS** 服务器并部署节点。

### 支持的服务商

| 服务商 | 最低价格 | 支付方式 | KYC |
|--------|----------|----------|-----|
| **LNVPS** | €1.70/月 (~¥13) | ⚡ Bitcoin Lightning | ❌ 不需要 |
| Vultr | $24/月 | 信用卡 | 需要 |
| DigitalOcean | $24/月 | 信用卡/PayPal | 需要 |

### 配置

```bash
# 推荐: LNVPS (最便宜，无需 KYC)
export CLOUD_PROVIDER=lnvps
export LNVPS_API_KEY=your-api-key  # 从 https://lnvps.net 获取

# 或者: Vultr
# export CLOUD_PROVIDER=vultr
# export VULTR_API_KEY=your-api-key

# 月度预算上限（超出自动拒绝）
export CLOUD_BUDGET_MONTHLY=100
```

### 云服务工具

| 工具 | 功能 | 风险级别 |
|------|------|----------|
| `list_cloud_regions` | 列出可用区域 | 安全 |
| `get_lightning_balance` | 查询 LNbits 钱包余额 | 安全 |
| `pay_lightning_invoice` | 支付 Lightning Invoice | 高风险 |
| `list_cloud_plans` | 列出套餐和价格 | 安全 |
| `estimate_cloud_cost` | 估算成本 | 安全 |
| `create_cloud_server` | **购买服务器** | **极高** |
| `destroy_cloud_server` | **销毁服务器** | **极高** |
| `list_cloud_servers` | 列出已创建服务器 | 安全 |
| `auto_deploy_node` | 自动规划部署 | 安全 |

### Lightning 自动支付 (LNVPS)

配置 LNbits 实现全自动购买 VPS：

```bash
# LNbits 配置
export LNBITS_URL=https://legend.lnbits.com
export LNBITS_ADMIN_KEY=your-admin-key

# 开启自动支付 (可选)
export LIGHTNING_AUTO_PAY=true
export LIGHTNING_MAX_AUTO_PAY_SATS=100000  # 最多自动支付 ~$30
```

**支付流程:**
```
1. Agent 创建 LNVPS 订单 → 获取 Lightning Invoice
2. 自动支付 (如已开启) 或手动确认
3. 支付完成 → VPS 自动创建
4. 返回服务器 IP，可立即部署节点
```

### 使用示例

```
👤 You: 帮我在新加坡买一台服务器部署 validator 节点

🤖 Agent: 让我查看可用的区域和套餐...

📋 自动部署计划

节点类型: validator (需要高性能)
区域: sgp (新加坡)
推荐套餐: vc2-4c-16gb (4 vCPU, 16GB RAM)
费用: $80.00/月

╔══════════════════════════════════════════════════════════════╗
║                    ⚠️  操作审批请求                            ║
╠══════════════════════════════════════════════════════════════╣
║ 操作ID:   a1b2                                                ║
║ 类型:     create_cloud_server                                 ║
║ 风险级别: 极高风险 - 需要多重确认                               ║
╚══════════════════════════════════════════════════════════════╝

⚠️  极高风险操作！请输入确认码 [a1b2] 以继续: 
```

### 安全控制

1. **预算限制** - 超出月度预算自动拒绝
2. **多重确认** - 购买操作需要输入确认码
3. **审计日志** - 所有操作记录可追溯
4. **标签追踪** - 自动添加 `node-operator` 标签

## 依赖

- Rust 1.70+
- 网络访问（调用 LLM API）
- 系统 SSH 客户端（用于远程部署）
- 可选：本地运行的 Substrate 节点

## License

MIT
