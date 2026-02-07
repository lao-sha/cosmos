# Costik Agent — 群主本地代理

去中心化多节点验证架构（方案 D）的 Local Agent 组件。

## 功能

- 接收 Telegram Webhook 推送
- Ed25519 签名（密钥永不上链）
- 确定性随机多播到 K 个项目节点
- 健康检查端点
- 序列号持久化（防重放）

## 快速开始

### 环境变量

| 变量 | 必填 | 默认值 | 说明 |
|------|------|--------|------|
| `BOT_TOKEN` | ✅ | - | Telegram Bot Token（永不上链） |
| `WEBHOOK_URL` | ✅ | - | Webhook 外部 URL，如 `https://my-server.com:8443` |
| `WEBHOOK_PORT` | - | `8443` | Webhook 监听端口 |
| `WEBHOOK_SECRET` | - | 随机生成 | Telegram Webhook 验证密钥 |
| `CHAIN_RPC` | - | `ws://127.0.0.1:9944` | 链上 RPC 端点 |
| `DATA_DIR` | - | `/data` | 数据目录（密钥、序列号） |
| `MULTICAST_TIMEOUT_MS` | - | `3000` | 多播超时（毫秒） |
| `NODES` | - | - | 静态节点列表，格式: `id1@http://host1:port,id2@http://host2:port` |
| `RUST_LOG` | - | `costik_agent=info` | 日志级别 |

### Docker 运行

```bash
docker build -t costik-agent .

docker run -d \
  --name costik-agent \
  -p 8443:8443 \
  -v costik-data:/data \
  -e BOT_TOKEN="your_bot_token" \
  -e WEBHOOK_URL="https://your-server.com:8443" \
  -e NODES="node1@http://node1:8080,node2@http://node2:8080,node3@http://node3:8080" \
  costik-agent
```

### 本地开发

```bash
# 设置环境变量
export BOT_TOKEN="your_bot_token"
export WEBHOOK_URL="https://your-ngrok-url"
export DATA_DIR="./data"
export NODES="node1@http://localhost:8081,node2@http://localhost:8082"

# 运行
cargo run
```

## API 端点

| 端点 | 方法 | 说明 |
|------|------|------|
| `/webhook` | POST | Telegram Webhook 接收 |
| `/v1/execute` | POST | Leader 节点管理指令（Sprint 4） |
| `/health` | GET | 健康检查 |

## 首次启动

1. Agent 自动生成 Ed25519 密钥对，保存到 `$DATA_DIR/agent.key`
2. 控制台输出公钥（hex），复制此公钥
3. 在链上调用 `pallet-bot-registry::register_bot()` 注册公钥
4. Agent 自动调用 Telegram `setWebhook` 注册 Webhook

## 消息流程

```
Telegram 用户发消息
       ↓
Telegram 服务器 → POST /webhook → Agent
       ↓
Agent: 验证 secret → 解析 Update → 签名(Ed25519) → 构造 SignedMessage
       ↓
Agent: 确定性选择 K 个节点 → 并发 POST /v1/message
       ↓
K 个节点收到 → Gossip 共识 → Leader 执行 → 回传指令
```

## 文件结构

```
costik-agent/
├── src/
│   ├── main.rs          # 入口: 初始化 + Axum 服务器
│   ├── config.rs        # 配置管理（环境变量）
│   ├── signer.rs        # Ed25519 密钥管理 + 序列号管理
│   ├── types.rs         # 数据类型定义
│   ├── webhook.rs       # Webhook 处理 + Health check
│   └── multicaster.rs   # 确定性多播 + TG Webhook 注册
├── Cargo.toml
├── Dockerfile
└── README.md
```
