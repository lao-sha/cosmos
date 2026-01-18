# Livestream Backend 使用指南

## 当前状态

✅ **后端服务已实现并可用**

### 已启动的服务
- HTTP API 服务器：`http://localhost:3001`
- WebSocket 服务器：`ws://localhost:3001`
- 链上事件监听：已连接到 Substrate 节点
- 状态：降级运行（无 Redis 缓存）

### 服务状态
```bash
curl http://localhost:3001/health
```

响应示例：
```json
{
  "status": "degraded",
  "timestamp": 1768640147630,
  "services": {
    "redis": "disconnected",
    "livekit": "disconnected",
    "chain": "connected"
  }
}
```

---

## API 端点

### 1. 获取主播推流 Token

**端点**: `POST /api/livestream/publisher-token`

**请求体**:
```json
{
  "roomId": 1,
  "publicKey": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
  "signature": "0x...",
  "timestamp": 1704067200000
}
```

**响应**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "url": "wss://livekit.example.com",
  "roomName": "room-1",
  "expiresAt": 1704088800000
}
```

**测试命令**:
```bash
curl -X POST http://localhost:3001/api/livestream/publisher-token \
  -H "Content-Type: application/json" \
  -d '{
    "roomId": 1,
    "publicKey": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    "signature": "0x1234...",
    "timestamp": '$(date +%s000)'
  }'
```

---

### 2. 获取观众观看 Token

**端点**: `POST /api/livestream/viewer-token`

**请求体**:
```json
{
  "roomId": 1,
  "viewerAddress": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
  "signature": "0x...",
  "timestamp": 1704067200000
}
```

**响应**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "url": "wss://livekit.example.com",
  "roomName": "room-1",
  "expiresAt": 1704088800000
}
```

---

### 3. 获取连麦者 Token

**端点**: `POST /api/livestream/co-host-token`

**请求体**:
```json
{
  "roomId": 1,
  "coHostAddress": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
  "signature": "0x...",
  "timestamp": 1704067200000,
  "type": "video"
}
```

---

### 4. 获取直播间信息

**端点**: `GET /api/livestream/room/:roomId`

**测试命令**:
```bash
curl http://localhost:3001/api/livestream/room/1
```

**响应**:
```json
{
  "id": 1,
  "host": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
  "title": "直播标题",
  "status": "Live",
  "roomType": "Normal",
  "currentViewers": 128,
  "totalGifts": "10000000000000",
  "startedAt": 1704067200000
}
```

---

### 5. 获取直播间列表

**端点**: `GET /api/livestream/rooms`

**查询参数**:
- `status`: 直播间状态（Live, Preparing, Ended）
- `type`: 直播间类型（Normal, Paid, Private）
- `page`: 页码（默认 1）
- `limit`: 每页数量（默认 20）

**测试命令**:
```bash
curl "http://localhost:3001/api/livestream/rooms?status=Live&page=1&limit=10"
```

**响应**:
```json
{
  "rooms": [...],
  "total": 100,
  "page": 1,
  "limit": 20
}
```

---

### 6. 观众离开直播间

**端点**: `POST /api/livestream/viewer-leave`

**请求体**:
```json
{
  "roomId": 1,
  "viewerAddress": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
}
```

---

## WebSocket 事件

### 连接
```javascript
const socket = io('http://localhost:3001');
```

### 客户端发送事件

#### 1. 用户认证
```javascript
socket.emit('auth', {
  address: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
  signature: '0x...',
  timestamp: Date.now()
});
```

#### 2. 加入直播间
```javascript
socket.emit('join-room', {
  roomId: 1
});
```

#### 3. 离开直播间
```javascript
socket.emit('leave-room', {
  roomId: 1
});
```

### 服务端推送事件

#### 1. 新直播间创建
```javascript
socket.on('room:created', (data) => {
  console.log('新直播间:', data);
  // { roomId: 1, host: '5G...', roomType: 'Normal' }
});
```

#### 2. 直播开始
```javascript
socket.on('live:started', (data) => {
  console.log('直播开始:', data);
  // { roomId: 1, startedAt: 1704067200000 }
});
```

#### 3. 直播结束
```javascript
socket.on('live:ended', (data) => {
  console.log('直播结束:', data);
  // { roomId: 1, duration: 3600, totalViewers: 1000, ... }
});
```

#### 4. 收到礼物
```javascript
socket.on('gift:received', (data) => {
  console.log('收到礼物:', data);
  // { roomId: 1, sender: '5F...', giftId: 1, quantity: 1, value: '1000000' }
});
```

#### 5. 连麦开始
```javascript
socket.on('cohost:started', (data) => {
  console.log('连麦开始:', data);
  // { roomId: 1, coHost: '5F...' }
});
```

#### 6. 连麦结束
```javascript
socket.on('cohost:ended', (data) => {
  console.log('连麦结束:', data);
  // { roomId: 1, coHost: '5F...' }
});
```

#### 7. 观众被踢出
```javascript
socket.on('viewer:kicked', (data) => {
  console.log('被踢出:', data);
  // { roomId: 1, message: 'You have been kicked from the room' }
});
```

#### 8. 直播间被封禁
```javascript
socket.on('room:banned', (data) => {
  console.log('直播间被封禁:', data);
  // { roomId: 1, reason: '违规内容' }
});
```

---

## 前端集成

### 1. 配置后端地址

在前端 `.env` 文件中添加：
```bash
EXPO_PUBLIC_BACKEND_URL=http://192.168.1.28:3001
```

### 2. 创建 API 服务

```typescript
// frontend/src/services/livestream-api.service.ts
const BACKEND_URL = process.env.EXPO_PUBLIC_BACKEND_URL || 'http://localhost:3001';

export async function getPublisherToken(
  roomId: number,
  publicKey: string,
  signature: string,
  timestamp: number
): Promise<{ token: string; url: string }> {
  const response = await fetch(`${BACKEND_URL}/api/livestream/publisher-token`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ roomId, publicKey, signature, timestamp }),
  });
  
  if (!response.ok) {
    throw new Error('Failed to get publisher token');
  }
  
  return response.json();
}
```

### 3. 使用 WebSocket

```typescript
import io from 'socket.io-client';

const socket = io(BACKEND_URL);

// 监听事件
socket.on('live:started', (data) => {
  console.log('直播开始:', data);
});

// 加入直播间
socket.emit('join-room', { roomId: 1 });
```

---

## 依赖服务

### 必需
- ✅ **Substrate 节点** (ws://localhost:9944) - 已连接
- ⚠️ **LiveKit 服务器** - 需要配置真实的 LiveKit 服务器

### 可选
- ⚠️ **Redis** (localhost:6379) - 用于缓存，没有会降级运行

---

## 启动 Redis（可选）

如果需要缓存功能，可以启动 Redis：

### Docker 方式
```bash
docker run -d -p 6379:6379 redis:7-alpine
```

### 系统安装
```bash
sudo apt install redis-server
sudo systemctl start redis
```

---

## 配置 LiveKit

### 1. 使用 LiveKit Cloud（推荐测试）

访问 https://livekit.io/ 注册账号，获取：
- API Key
- API Secret
- Server URL

### 2. 自建 LiveKit Server

```bash
docker run -d \
  -p 7880:7880 \
  -p 7881:7881 \
  -p 7882:7882/udp \
  -p 50000-50100:50000-50100/udp \
  -v $(pwd)/livekit.yaml:/etc/livekit.yaml \
  livekit/livekit-server:latest \
  --config /etc/livekit.yaml
```

### 3. 更新 .env 配置

```bash
LIVEKIT_URL=wss://your-livekit-server.com
LIVEKIT_API_KEY=your-api-key
LIVEKIT_API_SECRET=your-api-secret
```

---

## 测试流程

### 1. 测试健康检查
```bash
curl http://localhost:3001/health
```

### 2. 测试获取直播间列表
```bash
curl http://localhost:3001/api/livestream/rooms
```

### 3. 测试 WebSocket 连接
```javascript
const socket = io('http://localhost:3001');
socket.on('connect', () => {
  console.log('WebSocket 连接成功');
});
```

---

## 常见问题

### Q: Redis 连接失败怎么办？
A: 服务会在没有 Redis 的情况下降级运行，只是没有缓存功能。如果需要缓存，请启动 Redis。

### Q: LiveKit 连接失败怎么办？
A: 需要配置真实的 LiveKit 服务器。可以使用 LiveKit Cloud 或自建服务器。

### Q: 如何测试签名验证？
A: 需要使用真实的钱包私钥签名。参考前端的 `signer.native.ts` 实现。

### Q: 如何查看详细日志？
A: 后端使用 pino 日志，会输出到控制台。可以使用 `npm run dev` 查看实时日志。

---

## 下一步

1. **配置 LiveKit** - 获取真实的 LiveKit 服务器配置
2. **启动 Redis**（可选）- 提升性能
3. **前端集成** - 在 React Native 应用中调用后端 API
4. **测试推流** - 完整测试主播推流和观众观看流程

---

## 相关文档

- [LiveKit 文档](https://docs.livekit.io/)
- [Polkadot.js API](https://polkadot.js.org/docs/api/)
- [Socket.IO 文档](https://socket.io/docs/v4/)
