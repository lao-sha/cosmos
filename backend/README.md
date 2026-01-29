# Livestream Backend

直播模块后端服务，为 Cosmos 平台提供 LiveKit Token 生成、链上数据查询代理、实时通信等功能。

## 快速开始

### 开发环境

1. 安装依赖：
```bash
npm install
```

2. 复制环境变量配置：
```bash
cp .env.example .env
```

3. 编辑 `.env` 文件，配置必要的环境变量。

4. 启动开发服务器：
```bash
npm run dev
```

### Docker 部署

```bash
docker-compose up -d
```

## API 端点

### Token 相关

- `POST /api/livestream/publisher-token` - 获取主播推流 Token
- `POST /api/livestream/viewer-token` - 获取观众观看 Token
- `POST /api/livestream/co-host-token` - 获取连麦者 Token
- `POST /api/livestream/viewer-leave` - 观众离开直播间

### 直播间信息

- `GET /api/livestream/room/:roomId` - 获取直播间信息
- `GET /api/livestream/rooms` - 获取直播间列表

### 健康检查

- `GET /health` - 服务健康检查

## WebSocket 事件

### 客户端发送

- `auth` - 用户认证
- `join-room` - 加入直播间
- `leave-room` - 离开直播间

### 服务端推送

- `room:created` - 新直播间创建
- `live:started` - 直播开始
- `live:ended` - 直播结束
- `gift:received` - 收到礼物
- `cohost:started` - 连麦开始
- `cohost:ended` - 连麦结束
- `viewer:kicked` - 观众被踢出
- `room:banned` - 直播间被封禁

## 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| PORT | 服务端口 | 3001 |
| NODE_ENV | 运行环境 | development |
| LIVEKIT_URL | LiveKit 服务器地址 | wss://localhost:7880 |
| LIVEKIT_API_KEY | LiveKit API Key | - |
| LIVEKIT_API_SECRET | LiveKit API Secret | - |
| CHAIN_WS_ENDPOINT | Substrate 节点地址 | ws://localhost:9944 |
| REDIS_URL | Redis 连接地址 | redis://localhost:6379 |
| CORS_ORIGIN | CORS 允许的来源 | * |
