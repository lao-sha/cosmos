# Costik Node — 项目节点

去中心化多节点验证架构（方案 D）的 Project Node 组件。

## 功能

- 接收 Agent 多播的签名消息
- 四层验证（Ed25519 验签 → Bot 活跃检查 → 公钥匹配 → 目标节点验证）
- Gossip 共识（8 种消息类型 + M/K 投票）
- 消息状态机（HeardViaSeen → Received → Confirmed → Executing → Completed）
- Equivocation 检测
- Pull 补偿机制
- Leader 执行（Sprint 4）

## 环境变量

| 变量 | 必填 | 默认值 | 说明 |
|------|------|--------|------|
| `NODE_ID` | ✅ | - | 节点唯一标识 |
| `LISTEN_PORT` | - | `8080` | HTTP API 端口 |
| `GOSSIP_PORT` | - | `9090` | Gossip WebSocket 端口 |
| `CHAIN_RPC` | - | `ws://127.0.0.1:9944` | 链上 RPC |
| `SIGNING_KEY_PATH` | - | `/data/node.key` | 节点私钥路径 |
| `DATA_DIR` | - | `/data` | 数据目录 |
| `BOT_REGISTRATIONS` | - | - | Bot 缓存: `hash:pubkey:platform,...` |
| `NODE_LIST` | - | - | 节点列表: `id@endpoint,...` |

## Docker 运行

```bash
docker build -t costik-node .

docker run -d \
  --name costik-node-1 \
  -p 8080:8080 -p 9090:9090 \
  -v node1-data:/data \
  -e NODE_ID="node_001" \
  -e NODE_LIST="node_001@http://node1:8080,node_002@http://node2:8080,node_003@http://node3:8080" \
  costik-node
```

## API 端点

| 端点 | 方法 | 说明 |
|------|------|------|
| `/v1/message` | POST | 接收 Agent 签名消息 |
| `/v1/status/{msg_id}` | GET | 查询消息状态 |
| `/health` | GET | 健康检查 |

## 消息流程

```
Agent POST /v1/message → 四层验证 → GossipEngine
                                        ↓
                              广播 MessageSeen → 其他节点
                                        ↓
                              收集 Seen → 检查 M/K 共识
                                        ↓
                              共识达成 → Leader 执行 (Sprint 4)
```

## 文件结构

```
costik-node/
├── src/
│   ├── main.rs           # 入口 + Axum 服务器
│   ├── config.rs         # 配置管理
│   ├── types.rs          # 所有数据类型 + Gossip 消息类型
│   ├── api.rs            # HTTP API 处理
│   ├── verifier.rs       # 四层验证 + Ed25519 验签
│   ├── chain_cache.rs    # 链上数据缓存
│   └── gossip/
│       ├── mod.rs
│       ├── state.rs      # 消息状态机 + M/K 共识
│       └── engine.rs     # Gossip 引擎 + 消息路由
├── Cargo.toml
├── Dockerfile
└── README.md
```
