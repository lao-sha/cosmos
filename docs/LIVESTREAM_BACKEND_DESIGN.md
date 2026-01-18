# 直播模块后端服务设计文档

## 概述

本文档描述直播模块的后端服务设计，主要负责：
- LiveKit Token 生成与管理
- 主播身份签名验证
- 链上数据查询代理
- 直播间状态同步
- WebSocket 实时通信

## 技术栈

- **运行时**: Node.js 20+
- **框架**: Express.js
- **实时通信**: Socket.IO
- **LiveKit SDK**: livekit-server-sdk
- **链交互**: @polkadot/api
- **签名验证**: @polkadot/util-crypto
- **缓存**: Redis (ioredis)
- **参数校验**: zod
- **日志**: pino
- **部署**: Docker

## 项目结构

```
backend/
├── src/
│   ├── index.ts                    # 入口文件
│   ├── app.ts                      # Express 应用配置
│   ├── websocket.ts                # WebSocket 服务
│   ├── config/
│   │   ├── index.ts                # 配置加载
│   │   └── livekit.ts              # LiveKit 配置
│   ├── routes/
│   │   ├── index.ts                # 路由汇总
│   │   ├── livestream.routes.ts    # 直播相关路由
│   │   └── health.routes.ts        # 健康检查
│   ├── controllers/
│   │   └── livestream.controller.ts
│   ├── services/
│   │   ├── livekit.service.ts      # LiveKit Token 生成
│   │   ├── chain.service.ts        # 链上数据查询
│   │   ├── chain-events.service.ts # 链上事件监听
│   │   ├── signature.service.ts    # 签名验证
│   │   └── cache.service.ts        # Redis 缓存服务
│   ├── middleware/
│   │   ├── error.middleware.ts     # 错误处理
│   │   ├── logger.middleware.ts    # 请求日志
│   │   ├── validate.middleware.ts  # 参数校验
│   │   └── rateLimit.middleware.ts # 限流
│   ├── schemas/
│   │   └── livestream.schema.ts    # Zod 校验模式
│   ├── types/
│   │   └── index.ts                # 类型定义
│   └── utils/
│       └── logger.ts               # 日志工具
├── package.json
├── tsconfig.json
├── Dockerfile
├── docker-compose.yml
└── .env.example
```

## API 设计

### 基础信息

- **Base URL**: `/api/livestream`
- **Content-Type**: `application/json`
- **错误响应格式**:
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message"
  }
}
```

### API 端点

#### 1. 获取主播推流 Token

主播开播前调用，需要签名验证身份。

```
POST /api/livestream/publisher-token
```

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

**错误码**:
| Code | Message |
|------|---------|
| SIGNATURE_EXPIRED | 签名已过期 (超过5分钟) |
| INVALID_SIGNATURE | 签名验证失败 |
| ROOM_NOT_FOUND | 直播间不存在 |
| NOT_ROOM_HOST | 不是直播间主播 |
| ROOM_NOT_AVAILABLE | 直播间状态不可用 |

#### 2. 获取观众观看 Token

观众进入直播间时调用，**需要签名验证身份**。

```
POST /api/livestream/viewer-token
```

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

**错误码**:
| Code | Message |
|------|---------|
| SIGNATURE_EXPIRED | 签名已过期 (超过5分钟) |
| INVALID_SIGNATURE | 签名验证失败 |
| ROOM_NOT_FOUND | 直播间不存在 |
| ROOM_NOT_LIVE | 直播间未开播 |
| TICKET_REQUIRED | 需要购买门票 |
| VIEWER_BANNED | 观众已被封禁 |

#### 3. 获取连麦者 Token

观众被主播同意连麦后调用，**需要链上验证连麦状态**。

```
POST /api/livestream/co-host-token
```

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

**响应**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "url": "wss://livekit.example.com",
  "roomName": "room-1",
  "expiresAt": 1704088800000
}
```

**错误码**:
| Code | Message |
|------|---------|
| SIGNATURE_EXPIRED | 签名已过期 |
| INVALID_SIGNATURE | 签名验证失败 |
| ROOM_NOT_FOUND | 直播间不存在 |
| ROOM_NOT_LIVE | 直播间未开播 |
| NOT_CO_HOST | 未被批准为连麦者 |

#### 4. 获取直播间信息

```
GET /api/livestream/room/:roomId
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

#### 5. 获取直播间列表

```
GET /api/livestream/rooms?status=Live&type=Normal&page=1&limit=20
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

#### 6. 健康检查

```
GET /health
```

**响应**:
```json
{
  "status": "ok",
  "timestamp": 1704067200000,
  "services": {
    "livekit": "connected",
    "chain": "connected"
  }
}
```

## 核心服务实现

### 配置文件 (config/index.ts)

```typescript
// backend/src/config/index.ts

import dotenv from 'dotenv';

dotenv.config();

export const config = {
  // 服务配置
  port: parseInt(process.env.PORT || '3001', 10),
  nodeEnv: process.env.NODE_ENV || 'development',
  
  // LiveKit 配置
  livekit: {
    url: process.env.LIVEKIT_URL || 'wss://localhost:7880',
    apiKey: process.env.LIVEKIT_API_KEY || '',
    apiSecret: process.env.LIVEKIT_API_SECRET || '',
    tokenTTL: process.env.LIVEKIT_TOKEN_TTL || '6h',
  },
  
  // Substrate 节点配置
  chain: {
    wsEndpoint: process.env.CHAIN_WS_ENDPOINT || 'ws://localhost:9944',
  },
  
  // Redis 配置
  redis: {
    url: process.env.REDIS_URL || 'redis://localhost:6379',
    keyPrefix: 'livestream:',
    ttl: {
      room: 60,           // 直播间信息缓存 60 秒
      gift: 300,          // 礼物信息缓存 5 分钟
      blacklist: 30,      // 黑名单缓存 30 秒
      coHosts: 10,        // 连麦者列表缓存 10 秒
    },
  },
  
  // 安全配置
  security: {
    signatureMaxAge: 5 * 60 * 1000, // 签名有效期 5 分钟
    rateLimitWindow: 60 * 1000,     // 限流窗口 1 分钟
    rateLimitMax: 100,              // 每窗口最大请求数
  },
  
  // CORS 配置
  cors: {
    origin: process.env.CORS_ORIGIN || '*',
  },
};

// 验证必要配置
export function validateConfig(): void {
  const required = ['LIVEKIT_API_KEY', 'LIVEKIT_API_SECRET', 'REDIS_URL'];
  const missing = required.filter(key => !process.env[key]);
  
  if (missing.length > 0) {
    throw new Error(`Missing required environment variables: ${missing.join(', ')}`);
  }
}
```

### Redis 缓存服务 (services/cache.service.ts)

```typescript
// backend/src/services/cache.service.ts

import Redis from 'ioredis';
import { config } from '../config';
import { logger } from '../utils/logger';

export class CacheService {
  private redis: Redis;
  private prefix: string;
  
  constructor() {
    this.redis = new Redis(config.redis.url);
    this.prefix = config.redis.keyPrefix;
    
    this.redis.on('connect', () => {
      logger.info('Redis connected');
    });
    
    this.redis.on('error', (err) => {
      logger.error({ error: err }, 'Redis error');
    });
  }
  
  private key(type: string, id: string | number): string {
    return `${this.prefix}${type}:${id}`;
  }
  
  // ============ 直播间缓存 ============
  
  async getRoom(roomId: number): Promise<any | null> {
    const cached = await this.redis.get(this.key('room', roomId));
    return cached ? JSON.parse(cached) : null;
  }
  
  async setRoom(roomId: number, room: any): Promise<void> {
    await this.redis.setex(
      this.key('room', roomId),
      config.redis.ttl.room,
      JSON.stringify(room)
    );
  }
  
  async invalidateRoom(roomId: number): Promise<void> {
    await this.redis.del(this.key('room', roomId));
  }
  
  // ============ 礼物缓存 ============
  
  async getGifts(): Promise<any[] | null> {
    const cached = await this.redis.get(this.key('gifts', 'all'));
    return cached ? JSON.parse(cached) : null;
  }
  
  async setGifts(gifts: any[]): Promise<void> {
    await this.redis.setex(
      this.key('gifts', 'all'),
      config.redis.ttl.gift,
      JSON.stringify(gifts)
    );
  }
  
  // ============ 黑名单缓存 ============
  
  async isBlacklisted(roomId: number, user: string): Promise<boolean | null> {
    const cached = await this.redis.get(this.key('blacklist', `${roomId}:${user}`));
    return cached !== null ? cached === 'true' : null;
  }
  
  async setBlacklisted(roomId: number, user: string, banned: boolean): Promise<void> {
    await this.redis.setex(
      this.key('blacklist', `${roomId}:${user}`),
      config.redis.ttl.blacklist,
      banned.toString()
    );
  }
  
  // ============ 连麦者缓存 ============
  
  async getCoHosts(roomId: number): Promise<string[] | null> {
    const cached = await this.redis.get(this.key('cohosts', roomId));
    return cached ? JSON.parse(cached) : null;
  }
  
  async setCoHosts(roomId: number, coHosts: string[]): Promise<void> {
    await this.redis.setex(
      this.key('cohosts', roomId),
      config.redis.ttl.coHosts,
      JSON.stringify(coHosts)
    );
  }
  
  async invalidateCoHosts(roomId: number): Promise<void> {
    await this.redis.del(this.key('cohosts', roomId));
  }
  
  // ============ 在线观众统计 ============
  
  async incrViewerCount(roomId: number): Promise<number> {
    return this.redis.incr(this.key('viewers', roomId));
  }
  
  async decrViewerCount(roomId: number): Promise<number> {
    const count = await this.redis.decr(this.key('viewers', roomId));
    return Math.max(0, count);
  }
  
  async getViewerCount(roomId: number): Promise<number> {
    const count = await this.redis.get(this.key('viewers', roomId));
    return count ? parseInt(count, 10) : 0;
  }
  
  // ============ 健康检查 ============
  
  async ping(): Promise<boolean> {
    try {
      const result = await this.redis.ping();
      return result === 'PONG';
    } catch {
      return false;
    }
  }
  
  async disconnect(): Promise<void> {
    await this.redis.quit();
  }
}

export const cacheService = new CacheService();
```

### LiveKit 服务 (services/livekit.service.ts)

```typescript
// backend/src/services/livekit.service.ts

import { AccessToken, VideoGrant, RoomServiceClient } from 'livekit-server-sdk';
import { config } from '../config';
import { logger } from '../utils/logger';

export interface TokenOptions {
  identity: string;
  name?: string;
  roomName: string;
  canPublish: boolean;
  canSubscribe: boolean;
  canPublishData: boolean;
}

export class LiveKitService {
  private apiKey: string;
  private apiSecret: string;
  private url: string;
  private roomService: RoomServiceClient;
  
  constructor() {
    this.apiKey = config.livekit.apiKey;
    this.apiSecret = config.livekit.apiSecret;
    this.url = config.livekit.url;
    
    // 初始化 Room Service 客户端
    const httpUrl = this.url.replace('wss://', 'https://').replace('ws://', 'http://');
    this.roomService = new RoomServiceClient(httpUrl, this.apiKey, this.apiSecret);
  }
  
  /**
   * 生成 LiveKit 访问 Token
   */
  generateToken(options: TokenOptions): string {
    const { identity, name, roomName, canPublish, canSubscribe, canPublishData } = options;
    
    const token = new AccessToken(this.apiKey, this.apiSecret, {
      identity,
      name: name || identity,
      ttl: config.livekit.tokenTTL,
    });
    
    const grant: VideoGrant = {
      room: roomName,
      roomJoin: true,
      canPublish,
      canSubscribe,
      canPublishData,
    };
    
    token.addGrant(grant);
    
    logger.info({ identity, roomName, canPublish }, 'Generated LiveKit token');
    
    return token.toJwt();
  }
  
  /**
   * 生成主播 Token
   */
  generatePublisherToken(roomId: number, hostAddress: string): string {
    return this.generateToken({
      identity: hostAddress,
      name: `Host-${roomId}`,
      roomName: `room-${roomId}`,
      canPublish: true,
      canSubscribe: true,
      canPublishData: true,
    });
  }
  
  /**
   * 生成观众 Token
   */
  generateViewerToken(roomId: number, viewerAddress: string): string {
    return this.generateToken({
      identity: viewerAddress,
      roomName: `room-${roomId}`,
      canPublish: false,
      canSubscribe: true,
      canPublishData: true,
    });
  }
  
  /**
   * 生成连麦者 Token
   */
  generateCoHostToken(roomId: number, coHostAddress: string): string {
    return this.generateToken({
      identity: coHostAddress,
      name: `CoHost-${coHostAddress.slice(0, 8)}`,
      roomName: `room-${roomId}`,
      canPublish: true,
      canSubscribe: true,
      canPublishData: true,
    });
  }
  
  /**
   * 获取 LiveKit 服务器 URL
   */
  getServerUrl(): string {
    return this.url;
  }
  
  /**
   * 计算 Token 过期时间
   */
  getTokenExpiry(): number {
    const ttl = config.livekit.tokenTTL;
    const hours = parseInt(ttl.replace('h', ''), 10) || 6;
    return Date.now() + hours * 60 * 60 * 1000;
  }
  
  /**
   * 获取房间参与者数量
   */
  async getRoomParticipantCount(roomName: string): Promise<number> {
    try {
      const rooms = await this.roomService.listRooms([roomName]);
      return rooms[0]?.numParticipants || 0;
    } catch (error) {
      logger.error({ error, roomName }, 'Failed to get room participant count');
      return 0;
    }
  }
  
  /**
   * 检查 LiveKit 服务连接状态
   */
  async healthCheck(): Promise<boolean> {
    try {
      await this.roomService.listRooms([]);
      return true;
    } catch {
      return false;
    }
  }
  
  /**
   * 踢出参与者
   */
  async removeParticipant(roomName: string, identity: string): Promise<void> {
    try {
      await this.roomService.removeParticipant(roomName, identity);
      logger.info({ roomName, identity }, 'Participant removed from room');
    } catch (error) {
      logger.error({ error, roomName, identity }, 'Failed to remove participant');
    }
  }
}

export const livekitService = new LiveKitService();
```

### 签名验证服务 (services/signature.service.ts)

```typescript
// backend/src/services/signature.service.ts

import { signatureVerify, decodeAddress } from '@polkadot/util-crypto';
import { u8aToHex, hexToU8a } from '@polkadot/util';
import { config } from '../config';
import { logger } from '../utils/logger';

export interface SignatureVerifyResult {
  isValid: boolean;
  error?: string;
}

export class SignatureService {
  /**
   * 验证签名
   */
  verifySignature(
    message: string,
    signature: string,
    publicKey: string
  ): SignatureVerifyResult {
    try {
      // 验证地址格式
      decodeAddress(publicKey);
      
      // 验证签名
      const result = signatureVerify(message, signature, publicKey);
      
      if (!result.isValid) {
        logger.warn({ publicKey, message }, 'Invalid signature');
        return { isValid: false, error: 'Invalid signature' };
      }
      
      return { isValid: true };
    } catch (error) {
      logger.error({ error, publicKey }, 'Signature verification failed');
      return { isValid: false, error: 'Signature verification failed' };
    }
  }
  
  /**
   * 验证时间戳 (防重放攻击)
   */
  verifyTimestamp(timestamp: number): SignatureVerifyResult {
    const now = Date.now();
    const maxAge = config.security.signatureMaxAge;
    
    if (Math.abs(now - timestamp) > maxAge) {
      logger.warn({ timestamp, now, maxAge }, 'Signature expired');
      return { isValid: false, error: 'Signature expired' };
    }
    
    return { isValid: true };
  }
  
  /**
   * 构造签名消息
   */
  buildSignatureMessage(roomId: number, timestamp: number): string {
    return `livestream:${roomId}:${timestamp}`;
  }
  
  /**
   * 完整验证流程
   */
  verifyPublisherSignature(
    roomId: number,
    publicKey: string,
    signature: string,
    timestamp: number
  ): SignatureVerifyResult {
    // 1. 验证时间戳
    const timestampResult = this.verifyTimestamp(timestamp);
    if (!timestampResult.isValid) {
      return timestampResult;
    }
    
    // 2. 构造消息
    const message = this.buildSignatureMessage(roomId, timestamp);
    
    // 3. 验证签名
    return this.verifySignature(message, signature, publicKey);
  }
}

export const signatureService = new SignatureService();
```

### 链上数据服务 (services/chain.service.ts)

> **重要**: 修复了链上数据解析，正确处理 `BoundedVec<u8>` 类型转换。

```typescript
// backend/src/services/chain.service.ts

import { ApiPromise, WsProvider } from '@polkadot/api';
import { config } from '../config';
import { logger } from '../utils/logger';
import { cacheService } from './cache.service';

export interface LiveRoom {
  id: number;
  host: string;
  title: string;
  description?: string;
  roomType: string;
  status: string;
  coverCid?: string;
  totalViewers: number;
  peakViewers: number;
  totalGifts: string;
  ticketPrice?: string;
  createdAt: number;
  startedAt?: number;
  endedAt?: number;
}

// 直播间状态枚举映射
const ROOM_STATUS_MAP: Record<number, string> = {
  0: 'Preparing',
  1: 'Live',
  2: 'Paused',
  3: 'Ended',
  4: 'Banned',
};

// 直播间类型枚举映射
const ROOM_TYPE_MAP: Record<number, string> = {
  0: 'Normal',
  1: 'Paid',
  2: 'Private',
  3: 'MultiHost',
};

export class ChainService {
  private api: ApiPromise | null = null;
  private connecting: Promise<ApiPromise> | null = null;
  
  /**
   * 连接到 Substrate 节点
   */
  async connect(): Promise<ApiPromise> {
    if (this.api?.isConnected) {
      return this.api;
    }
    
    if (this.connecting) {
      return this.connecting;
    }
    
    this.connecting = this.doConnect();
    return this.connecting;
  }
  
  private async doConnect(): Promise<ApiPromise> {
    try {
      logger.info({ endpoint: config.chain.wsEndpoint }, 'Connecting to chain...');
      
      const provider = new WsProvider(config.chain.wsEndpoint);
      this.api = await ApiPromise.create({ provider });
      
      // 监听断开事件
      this.api.on('disconnected', () => {
        logger.warn('Chain disconnected, will reconnect...');
        this.api = null;
        this.connecting = null;
      });
      
      logger.info('Connected to chain');
      return this.api;
    } catch (error) {
      logger.error({ error }, 'Failed to connect to chain');
      this.connecting = null;
      throw error;
    }
  }
  
  /**
   * 获取直播间信息 (带缓存)
   */
  async getRoom(roomId: number): Promise<LiveRoom | null> {
    // 1. 先查缓存
    const cached = await cacheService.getRoom(roomId);
    if (cached) {
      return cached;
    }
    
    // 2. 查询链上
    const api = await this.connect();
    const result = await api.query.livestream.liveRooms(roomId);
    
    if (result.isNone) {
      return null;
    }
    
    const room = this.parseRoom(roomId, result.unwrap());
    
    // 3. 写入缓存
    await cacheService.setRoom(roomId, room);
    
    return room;
  }
  
  /**
   * 检查是否是直播间主播
   */
  async isRoomHost(roomId: number, address: string): Promise<boolean> {
    const room = await this.getRoom(roomId);
    return room?.host === address;
  }
  
  /**
   * 检查直播间状态
   */
  async getRoomStatus(roomId: number): Promise<string | null> {
    const room = await this.getRoom(roomId);
    return room?.status || null;
  }
  
  /**
   * 检查观众是否有门票
   */
  async hasTicket(roomId: number, viewerAddress: string): Promise<boolean> {
    const api = await this.connect();
    const result = await api.query.livestream.ticketHolders(roomId, viewerAddress);
    return result.isSome;
  }
  
  /**
   * 检查观众是否被封禁 (带缓存)
   */
  async isViewerBanned(roomId: number, viewerAddress: string): Promise<boolean> {
    // 1. 先查缓存
    const cached = await cacheService.isBlacklisted(roomId, viewerAddress);
    if (cached !== null) {
      return cached;
    }
    
    // 2. 查询链上
    const api = await this.connect();
    const result = await api.query.livestream.roomBlacklist(roomId, viewerAddress);
    const banned = result.isSome;
    
    // 3. 写入缓存
    await cacheService.setBlacklisted(roomId, viewerAddress, banned);
    
    return banned;
  }
  
  /**
   * 获取连麦者列表 (带缓存)
   */
  async getCoHosts(roomId: number): Promise<string[]> {
    // 1. 先查缓存
    const cached = await cacheService.getCoHosts(roomId);
    if (cached !== null) {
      return cached;
    }
    
    // 2. 查询链上
    const api = await this.connect();
    const result = await api.query.livestream.activeCoHosts(roomId);
    
    // 解析 BoundedVec<AccountId>
    const coHosts = result.toArray().map((account: any) => account.toString());
    
    // 3. 写入缓存
    await cacheService.setCoHosts(roomId, coHosts);
    
    return coHosts;
  }
  
  /**
   * 检查是否是连麦者
   */
  async isCoHost(roomId: number, address: string): Promise<boolean> {
    const coHosts = await this.getCoHosts(roomId);
    return coHosts.includes(address);
  }
  
  /**
   * 获取直播间列表
   */
  async getRooms(options: {
    status?: string;
    roomType?: string;
    page?: number;
    limit?: number;
  }): Promise<{ rooms: LiveRoom[]; total: number }> {
    const api = await this.connect();
    const { status, roomType, page = 1, limit = 20 } = options;
    
    const entries = await api.query.livestream.liveRooms.entries();
    let rooms: LiveRoom[] = [];
    
    for (const [key, value] of entries) {
      if (value.isNone) continue;
      
      const roomId = (key.args[0] as any).toNumber();
      const room = this.parseRoom(roomId, value.unwrap());
      
      // 筛选
      if (status && room.status !== status) continue;
      if (roomType && room.roomType !== roomType) continue;
      
      rooms.push(room);
    }
    
    // 按观众数排序
    rooms.sort((a, b) => b.totalViewers - a.totalViewers);
    
    const total = rooms.length;
    const start = (page - 1) * limit;
    rooms = rooms.slice(start, start + limit);
    
    return { rooms, total };
  }
  
  /**
   * 获取主播累计收益
   */
  async getHostEarnings(host: string): Promise<string> {
    const api = await this.connect();
    const result = await api.query.livestream.hostEarnings(host);
    return result.toString();
  }
  
  /**
   * 检查连接状态
   */
  isConnected(): boolean {
    return this.api?.isConnected || false;
  }
  
  /**
   * 解析链上直播间数据
   * 
   * 重要: 正确处理 BoundedVec<u8> 类型转换
   */
  private parseRoom(roomId: number, data: any): LiveRoom {
    // 解析 BoundedVec<u8> 为字符串
    const decodeBytes = (boundedVec: any): string => {
      if (!boundedVec) return '';
      try {
        // BoundedVec 需要先转为 Uint8Array
        const bytes = boundedVec.toU8a ? boundedVec.toU8a() : new Uint8Array(boundedVec);
        return new TextDecoder('utf-8').decode(bytes);
      } catch {
        return boundedVec.toString();
      }
    };
    
    // 解析 Option 类型
    const unwrapOption = <T>(option: any, decoder?: (v: any) => T): T | undefined => {
      if (option.isNone) return undefined;
      const value = option.unwrap();
      return decoder ? decoder(value) : value;
    };
    
    // 解析枚举类型
    const parseEnum = (enumValue: any, map: Record<number, string>): string => {
      // Substrate 枚举可能是对象或数字
      if (typeof enumValue.toNumber === 'function') {
        return map[enumValue.toNumber()] || 'Unknown';
      }
      // 尝试获取枚举变体名称
      if (enumValue.type) {
        return enumValue.type;
      }
      // 遍历枚举变体
      for (const [key, value] of Object.entries(map)) {
        if (enumValue[`is${value}`]) {
          return value;
        }
      }
      return 'Unknown';
    };
    
    return {
      id: roomId,
      host: data.host.toString(),
      title: decodeBytes(data.title),
      description: unwrapOption(data.description, decodeBytes),
      roomType: parseEnum(data.roomType, ROOM_TYPE_MAP),
      status: parseEnum(data.status, ROOM_STATUS_MAP),
      coverCid: unwrapOption(data.coverCid, decodeBytes),
      totalViewers: data.totalViewers.toNumber(),
      peakViewers: data.peakViewers.toNumber(),
      totalGifts: data.totalGifts.toString(),
      ticketPrice: unwrapOption(data.ticketPrice, (v) => v.toString()),
      createdAt: data.createdAt.toNumber(),
      startedAt: unwrapOption(data.startedAt, (v) => v.toNumber()),
      endedAt: unwrapOption(data.endedAt, (v) => v.toNumber()),
    };
  }
}

export const chainService = new ChainService();
```

### 链上事件监听服务 (services/chain-events.service.ts)

```typescript
// backend/src/services/chain-events.service.ts

import { ApiPromise } from '@polkadot/api';
import { chainService } from './chain.service';
import { cacheService } from './cache.service';
import { websocketService } from '../websocket';
import { logger } from '../utils/logger';

export interface LivestreamEvent {
  type: string;
  roomId?: number;
  data: any;
  blockNumber: number;
  timestamp: number;
}

export class ChainEventsService {
  private unsubscribe: (() => void) | null = null;
  
  /**
   * 开始监听链上事件
   */
  async startListening(): Promise<void> {
    const api = await chainService.connect();
    
    logger.info('Starting chain events listener...');
    
    // 订阅新区块事件
    this.unsubscribe = await api.query.system.events((events) => {
      const blockNumber = api.runtimeVersion.specVersion.toNumber();
      
      events.forEach((record) => {
        const { event } = record;
        
        // 只处理 livestream 模块的事件
        if (event.section === 'livestream') {
          this.handleLivestreamEvent(event, blockNumber);
        }
      });
    }) as unknown as () => void;
    
    logger.info('Chain events listener started');
  }
  
  /**
   * 停止监听
   */
  stopListening(): void {
    if (this.unsubscribe) {
      this.unsubscribe();
      this.unsubscribe = null;
      logger.info('Chain events listener stopped');
    }
  }
  
  /**
   * 处理直播模块事件
   */
  private async handleLivestreamEvent(event: any, blockNumber: number): Promise<void> {
    const eventName = event.method;
    const data = event.data.toJSON();
    
    logger.info({ eventName, data }, 'Received livestream event');
    
    try {
      switch (eventName) {
        case 'RoomCreated':
          await this.onRoomCreated(data);
          break;
          
        case 'LiveStarted':
          await this.onLiveStarted(data);
          break;
          
        case 'LiveEnded':
          await this.onLiveEnded(data);
          break;
          
        case 'GiftSent':
          await this.onGiftSent(data);
          break;
          
        case 'CoHostStarted':
          await this.onCoHostStarted(data);
          break;
          
        case 'CoHostEnded':
          await this.onCoHostEnded(data);
          break;
          
        case 'ViewerKicked':
          await this.onViewerKicked(data);
          break;
          
        case 'ViewerUnbanned':
          await this.onViewerUnbanned(data);
          break;
          
        case 'RoomBanned':
          await this.onRoomBanned(data);
          break;
          
        default:
          logger.debug({ eventName }, 'Unhandled event');
      }
    } catch (error) {
      logger.error({ error, eventName, data }, 'Error handling event');
    }
  }
  
  // ============ 事件处理器 ============
  
  private async onRoomCreated(data: any): Promise<void> {
    const { host, roomId, roomType } = data;
    
    // 广播新直播间创建
    websocketService.broadcastToAll('room:created', {
      roomId,
      host,
      roomType,
    });
  }
  
  private async onLiveStarted(data: any): Promise<void> {
    const { roomId, startedAt } = data;
    
    // 清除缓存
    await cacheService.invalidateRoom(roomId);
    
    // 广播开播通知
    websocketService.broadcastToRoom(roomId, 'live:started', {
      roomId,
      startedAt,
    });
    
    // 广播到大厅
    websocketService.broadcastToAll('room:live', { roomId });
  }
  
  private async onLiveEnded(data: any): Promise<void> {
    const { roomId, duration, totalViewers, peakViewers, totalGifts } = data;
    
    // 清除缓存
    await cacheService.invalidateRoom(roomId);
    
    // 广播结束通知
    websocketService.broadcastToRoom(roomId, 'live:ended', {
      roomId,
      duration,
      totalViewers,
      peakViewers,
      totalGifts,
    });
  }
  
  private async onGiftSent(data: any): Promise<void> {
    const { roomId, sender, receiver, giftId, quantity, value } = data;
    
    // 清除直播间缓存 (礼物总额变化)
    await cacheService.invalidateRoom(roomId);
    
    // 广播礼物通知到直播间
    websocketService.broadcastToRoom(roomId, 'gift:received', {
      roomId,
      sender,
      receiver,
      giftId,
      quantity,
      value,
    });
  }
  
  private async onCoHostStarted(data: any): Promise<void> {
    const { roomId, coHost } = data;
    
    // 清除连麦者缓存
    await cacheService.invalidateCoHosts(roomId);
    
    // 广播连麦开始
    websocketService.broadcastToRoom(roomId, 'cohost:started', {
      roomId,
      coHost,
    });
  }
  
  private async onCoHostEnded(data: any): Promise<void> {
    const { roomId, coHost } = data;
    
    // 清除连麦者缓存
    await cacheService.invalidateCoHosts(roomId);
    
    // 广播连麦结束
    websocketService.broadcastToRoom(roomId, 'cohost:ended', {
      roomId,
      coHost,
    });
  }
  
  private async onViewerKicked(data: any): Promise<void> {
    const { roomId, viewer } = data;
    
    // 更新黑名单缓存
    await cacheService.setBlacklisted(roomId, viewer, true);
    
    // 通知被踢用户
    websocketService.notifyUser(viewer, 'viewer:kicked', {
      roomId,
      message: 'You have been kicked from the room',
    });
  }
  
  private async onViewerUnbanned(data: any): Promise<void> {
    const { roomId, viewer } = data;
    
    // 更新黑名单缓存
    await cacheService.setBlacklisted(roomId, viewer, false);
  }
  
  private async onRoomBanned(data: any): Promise<void> {
    const { roomId, reason } = data;
    
    // 清除缓存
    await cacheService.invalidateRoom(roomId);
    
    // 广播封禁通知
    websocketService.broadcastToRoom(roomId, 'room:banned', {
      roomId,
      reason,
    });
  }
}

export const chainEventsService = new ChainEventsService();
```


### 控制器 (controllers/livestream.controller.ts)

> **更新**: 添加了观众身份验证和连麦者链上状态检查。

```typescript
// backend/src/controllers/livestream.controller.ts

import { Request, Response, NextFunction } from 'express';
import { livekitService } from '../services/livekit.service';
import { signatureService } from '../services/signature.service';
import { chainService } from '../services/chain.service';
import { cacheService } from '../services/cache.service';
import { AppError } from '../middleware/error.middleware';
import { logger } from '../utils/logger';

export class LivestreamController {
  /**
   * 获取主播推流 Token
   * POST /api/livestream/publisher-token
   */
  async getPublisherToken(req: Request, res: Response, next: NextFunction) {
    try {
      const { roomId, publicKey, signature, timestamp } = req.body;
      
      // 1. 参数验证 (由 zod 中间件完成)
      
      // 2. 验证签名
      const signResult = signatureService.verifyPublisherSignature(
        roomId,
        publicKey,
        signature,
        timestamp
      );
      
      if (!signResult.isValid) {
        const errorCode = signResult.error?.includes('expired') 
          ? 'SIGNATURE_EXPIRED' 
          : 'INVALID_SIGNATURE';
        throw new AppError(errorCode, signResult.error || 'Signature verification failed', 401);
      }
      
      // 3. 查询链上直播间
      const room = await chainService.getRoom(roomId);
      if (!room) {
        throw new AppError('ROOM_NOT_FOUND', 'Room not found', 404);
      }
      
      // 4. 验证是否是主播
      if (room.host !== publicKey) {
        throw new AppError('NOT_ROOM_HOST', 'You are not the host of this room', 403);
      }
      
      // 5. 验证直播间状态
      if (room.status !== 'Live' && room.status !== 'Preparing') {
        throw new AppError('ROOM_NOT_AVAILABLE', 'Room is not available for streaming', 400);
      }
      
      // 6. 生成 Token
      const token = livekitService.generatePublisherToken(roomId, publicKey);
      
      logger.info({ roomId, host: publicKey }, 'Publisher token generated');
      
      res.json({
        token,
        url: livekitService.getServerUrl(),
        roomName: `room-${roomId}`,
        expiresAt: livekitService.getTokenExpiry(),
      });
    } catch (error) {
      next(error);
    }
  }
  
  /**
   * 获取观众观看 Token
   * POST /api/livestream/viewer-token
   * 
   * 更新: 添加签名验证，防止身份伪造
   */
  async getViewerToken(req: Request, res: Response, next: NextFunction) {
    try {
      const { roomId, viewerAddress, signature, timestamp } = req.body;
      
      // 1. 验证签名 (防止身份伪造)
      const signResult = signatureService.verifyPublisherSignature(
        roomId,
        viewerAddress,
        signature,
        timestamp
      );
      
      if (!signResult.isValid) {
        const errorCode = signResult.error?.includes('expired') 
          ? 'SIGNATURE_EXPIRED' 
          : 'INVALID_SIGNATURE';
        throw new AppError(errorCode, signResult.error || 'Signature verification failed', 401);
      }
      
      // 2. 查询链上直播间
      const room = await chainService.getRoom(roomId);
      if (!room) {
        throw new AppError('ROOM_NOT_FOUND', 'Room not found', 404);
      }
      
      // 3. 验证直播间状态
      if (room.status !== 'Live') {
        throw new AppError('ROOM_NOT_LIVE', 'Room is not live', 400);
      }
      
      // 4. 如果是付费直播，检查门票
      if (room.roomType === 'Paid') {
        const hasTicket = await chainService.hasTicket(roomId, viewerAddress);
        if (!hasTicket) {
          throw new AppError('TICKET_REQUIRED', 'You need to buy a ticket first', 402);
        }
      }
      
      // 5. 检查是否被封禁
      const isBanned = await chainService.isViewerBanned(roomId, viewerAddress);
      if (isBanned) {
        throw new AppError('VIEWER_BANNED', 'You are banned from this room', 403);
      }
      
      // 6. 生成 Token
      const token = livekitService.generateViewerToken(roomId, viewerAddress);
      
      // 7. 增加在线观众计数
      await cacheService.incrViewerCount(roomId);
      
      logger.info({ roomId, viewer: viewerAddress }, 'Viewer token generated');
      
      res.json({
        token,
        url: livekitService.getServerUrl(),
        roomName: `room-${roomId}`,
        expiresAt: livekitService.getTokenExpiry(),
      });
    } catch (error) {
      next(error);
    }
  }
  
  /**
   * 获取连麦者 Token
   * POST /api/livestream/co-host-token
   * 
   * 更新: 添加链上连麦状态验证
   */
  async getCoHostToken(req: Request, res: Response, next: NextFunction) {
    try {
      const { roomId, coHostAddress, signature, timestamp, type } = req.body;
      
      // 1. 验证签名
      const signResult = signatureService.verifyPublisherSignature(
        roomId,
        coHostAddress,
        signature,
        timestamp
      );
      
      if (!signResult.isValid) {
        const errorCode = signResult.error?.includes('expired') 
          ? 'SIGNATURE_EXPIRED' 
          : 'INVALID_SIGNATURE';
        throw new AppError(errorCode, signResult.error || 'Signature verification failed', 401);
      }
      
      // 2. 查询链上直播间
      const room = await chainService.getRoom(roomId);
      if (!room) {
        throw new AppError('ROOM_NOT_FOUND', 'Room not found', 404);
      }
      
      // 3. 验证直播间状态
      if (room.status !== 'Live') {
        throw new AppError('ROOM_NOT_LIVE', 'Room is not live', 400);
      }
      
      // 4. 验证是否在连麦者列表中 (链上验证)
      const isCoHost = await chainService.isCoHost(roomId, coHostAddress);
      if (!isCoHost) {
        throw new AppError('NOT_CO_HOST', 'You are not approved as co-host', 403);
      }
      
      // 5. 生成 Token
      const token = livekitService.generateCoHostToken(roomId, coHostAddress);
      
      logger.info({ roomId, coHost: coHostAddress, type }, 'Co-host token generated');
      
      res.json({
        token,
        url: livekitService.getServerUrl(),
        roomName: `room-${roomId}`,
        expiresAt: livekitService.getTokenExpiry(),
      });
    } catch (error) {
      next(error);
    }
  }
  
  /**
   * 获取直播间信息
   * GET /api/livestream/room/:roomId
   */
  async getRoom(req: Request, res: Response, next: NextFunction) {
    try {
      const roomId = parseInt(req.params.roomId, 10);
      
      if (isNaN(roomId)) {
        throw new AppError('INVALID_PARAMS', 'Invalid room ID', 400);
      }
      
      const room = await chainService.getRoom(roomId);
      if (!room) {
        throw new AppError('ROOM_NOT_FOUND', 'Room not found', 404);
      }
      
      // 获取实时观众数
      const currentViewers = await cacheService.getViewerCount(roomId);
      
      res.json({
        ...room,
        currentViewers,
      });
    } catch (error) {
      next(error);
    }
  }
  
  /**
   * 获取直播间列表
   * GET /api/livestream/rooms
   */
  async getRooms(req: Request, res: Response, next: NextFunction) {
    try {
      const { status, type, page, limit } = req.query;
      
      const result = await chainService.getRooms({
        status: status as string,
        roomType: type as string,
        page: page ? parseInt(page as string, 10) : 1,
        limit: limit ? parseInt(limit as string, 10) : 20,
      });
      
      // 为每个直播间添加实时观众数
      const roomsWithViewers = await Promise.all(
        result.rooms.map(async (room) => ({
          ...room,
          currentViewers: await cacheService.getViewerCount(room.id),
        }))
      );
      
      res.json({
        rooms: roomsWithViewers,
        total: result.total,
        page: page ? parseInt(page as string, 10) : 1,
        limit: limit ? parseInt(limit as string, 10) : 20,
      });
    } catch (error) {
      next(error);
    }
  }
  
  /**
   * 观众离开直播间
   * POST /api/livestream/viewer-leave
   */
  async viewerLeave(req: Request, res: Response, next: NextFunction) {
    try {
      const { roomId, viewerAddress, signature, timestamp } = req.body;
      
      // 验证签名
      const signResult = signatureService.verifyPublisherSignature(
        roomId,
        viewerAddress,
        signature,
        timestamp
      );
      
      if (!signResult.isValid) {
        throw new AppError('INVALID_SIGNATURE', 'Signature verification failed', 401);
      }
      
      // 减少在线观众计数
      await cacheService.decrViewerCount(roomId);
      
      logger.info({ roomId, viewer: viewerAddress }, 'Viewer left');
      
      res.json({ success: true });
    } catch (error) {
      next(error);
    }
  }
}

export const livestreamController = new LivestreamController();
```

### 路由配置 (routes/livestream.routes.ts)

```typescript
// backend/src/routes/livestream.routes.ts

import { Router } from 'express';
import { livestreamController } from '../controllers/livestream.controller';
import { validate } from '../middleware/validate.middleware';
import { tokenRateLimitMiddleware } from '../middleware/rateLimit.middleware';
import {
  publisherTokenSchema,
  viewerTokenSchema,
  coHostTokenSchema,
  viewerLeaveSchema,
} from '../schemas/livestream.schema';

const router = Router();

// Token 相关 (带限流和参数校验)
router.post(
  '/publisher-token',
  tokenRateLimitMiddleware,
  validate(publisherTokenSchema),
  (req, res, next) => livestreamController.getPublisherToken(req, res, next)
);

router.post(
  '/viewer-token',
  tokenRateLimitMiddleware,
  validate(viewerTokenSchema),
  (req, res, next) => livestreamController.getViewerToken(req, res, next)
);

router.post(
  '/co-host-token',
  tokenRateLimitMiddleware,
  validate(coHostTokenSchema),
  (req, res, next) => livestreamController.getCoHostToken(req, res, next)
);

router.post(
  '/viewer-leave',
  validate(viewerLeaveSchema),
  (req, res, next) => livestreamController.viewerLeave(req, res, next)
);

// 直播间信息
router.get('/room/:roomId', (req, res, next) => 
  livestreamController.getRoom(req, res, next)
);

router.get('/rooms', (req, res, next) => 
  livestreamController.getRooms(req, res, next)
);

export default router;
```

### 参数校验模式 (schemas/livestream.schema.ts)

```typescript
// backend/src/schemas/livestream.schema.ts

import { z } from 'zod';

// SS58 地址正则 (Substrate 地址格式)
const ss58AddressRegex = /^[1-9A-HJ-NP-Za-km-z]{47,48}$/;

// 签名正则 (0x 开头的十六进制)
const signatureRegex = /^0x[a-fA-F0-9]+$/;

/**
 * 主播推流 Token 请求
 */
export const publisherTokenSchema = z.object({
  body: z.object({
    roomId: z.number().int().positive('Room ID must be a positive integer'),
    publicKey: z.string().regex(ss58AddressRegex, 'Invalid SS58 address format'),
    signature: z.string().regex(signatureRegex, 'Invalid signature format'),
    timestamp: z.number().int().positive('Timestamp must be a positive integer'),
  }),
});

/**
 * 观众观看 Token 请求
 */
export const viewerTokenSchema = z.object({
  body: z.object({
    roomId: z.number().int().positive('Room ID must be a positive integer'),
    viewerAddress: z.string().regex(ss58AddressRegex, 'Invalid SS58 address format'),
    signature: z.string().regex(signatureRegex, 'Invalid signature format'),
    timestamp: z.number().int().positive('Timestamp must be a positive integer'),
  }),
});

/**
 * 连麦者 Token 请求
 */
export const coHostTokenSchema = z.object({
  body: z.object({
    roomId: z.number().int().positive('Room ID must be a positive integer'),
    coHostAddress: z.string().regex(ss58AddressRegex, 'Invalid SS58 address format'),
    signature: z.string().regex(signatureRegex, 'Invalid signature format'),
    timestamp: z.number().int().positive('Timestamp must be a positive integer'),
    type: z.enum(['audio', 'video']).optional().default('audio'),
  }),
});

/**
 * 观众离开请求
 */
export const viewerLeaveSchema = z.object({
  body: z.object({
    roomId: z.number().int().positive('Room ID must be a positive integer'),
    viewerAddress: z.string().regex(ss58AddressRegex, 'Invalid SS58 address format'),
    signature: z.string().regex(signatureRegex, 'Invalid signature format'),
    timestamp: z.number().int().positive('Timestamp must be a positive integer'),
  }),
});

export type PublisherTokenRequest = z.infer<typeof publisherTokenSchema>;
export type ViewerTokenRequest = z.infer<typeof viewerTokenSchema>;
export type CoHostTokenRequest = z.infer<typeof coHostTokenSchema>;
export type ViewerLeaveRequest = z.infer<typeof viewerLeaveSchema>;
```

### 参数校验中间件 (middleware/validate.middleware.ts)

```typescript
// backend/src/middleware/validate.middleware.ts

import { Request, Response, NextFunction } from 'express';
import { AnyZodObject, ZodError } from 'zod';
import { AppError } from './error.middleware';

export const validate = (schema: AnyZodObject) => {
  return async (req: Request, res: Response, next: NextFunction) => {
    try {
      await schema.parseAsync({
        body: req.body,
        query: req.query,
        params: req.params,
      });
      next();
    } catch (error) {
      if (error instanceof ZodError) {
        const message = error.errors
          .map((e) => `${e.path.join('.')}: ${e.message}`)
          .join(', ');
        next(new AppError('VALIDATION_ERROR', message, 400));
      } else {
        next(error);
      }
    }
  };
};
```

### 错误处理中间件 (middleware/error.middleware.ts)

```typescript
// backend/src/middleware/error.middleware.ts

import { Request, Response, NextFunction } from 'express';
import { logger } from '../utils/logger';

export class AppError extends Error {
  constructor(
    public code: string,
    message: string,
    public statusCode: number = 500
  ) {
    super(message);
    this.name = 'AppError';
  }
}

export function errorMiddleware(
  err: Error,
  req: Request,
  res: Response,
  next: NextFunction
) {
  if (err instanceof AppError) {
    logger.warn({
      code: err.code,
      message: err.message,
      path: req.path,
    }, 'Application error');
    
    return res.status(err.statusCode).json({
      error: {
        code: err.code,
        message: err.message,
      },
    });
  }
  
  // 未知错误
  logger.error({
    error: err,
    path: req.path,
    method: req.method,
  }, 'Unexpected error');
  
  res.status(500).json({
    error: {
      code: 'INTERNAL_ERROR',
      message: 'An unexpected error occurred',
    },
  });
}
```

### 限流中间件 (middleware/rateLimit.middleware.ts)

```typescript
// backend/src/middleware/rateLimit.middleware.ts

import rateLimit from 'express-rate-limit';
import { config } from '../config';

export const rateLimitMiddleware = rateLimit({
  windowMs: config.security.rateLimitWindow,
  max: config.security.rateLimitMax,
  message: {
    error: {
      code: 'RATE_LIMIT_EXCEEDED',
      message: 'Too many requests, please try again later',
    },
  },
  standardHeaders: true,
  legacyHeaders: false,
});

// Token 接口专用限流 (更严格)
export const tokenRateLimitMiddleware = rateLimit({
  windowMs: 60 * 1000, // 1 分钟
  max: 10,             // 每分钟最多 10 次
  message: {
    error: {
      code: 'RATE_LIMIT_EXCEEDED',
      message: 'Too many token requests, please try again later',
    },
  },
});
```

### 日志工具 (utils/logger.ts)

```typescript
// backend/src/utils/logger.ts

import pino from 'pino';
import { config } from '../config';

export const logger = pino({
  level: config.nodeEnv === 'production' ? 'info' : 'debug',
  transport: config.nodeEnv === 'development' 
    ? { target: 'pino-pretty', options: { colorize: true } }
    : undefined,
});
```

### 应用入口 (app.ts)

```typescript
// backend/src/app.ts

import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import { createServer } from 'http';
import { config } from './config';
import { errorMiddleware } from './middleware/error.middleware';
import { rateLimitMiddleware } from './middleware/rateLimit.middleware';
import livestreamRoutes from './routes/livestream.routes';
import { logger } from './utils/logger';
import { chainService } from './services/chain.service';
import { cacheService } from './services/cache.service';
import { livekitService } from './services/livekit.service';

const app = express();
export const httpServer = createServer(app);

// 基础中间件
app.use(helmet());
app.use(cors({ origin: config.cors.origin }));
app.use(express.json());
app.use(rateLimitMiddleware);

// 请求日志
app.use((req, res, next) => {
  logger.info({ method: req.method, path: req.path }, 'Request');
  next();
});

// 健康检查 (完整版)
app.get('/health', async (req, res) => {
  const [redisOk, livekitOk] = await Promise.all([
    cacheService.ping(),
    livekitService.healthCheck(),
  ]);
  
  const chainOk = chainService.isConnected();
  const allOk = redisOk && livekitOk && chainOk;
  
  res.status(allOk ? 200 : 503).json({
    status: allOk ? 'ok' : 'degraded',
    timestamp: Date.now(),
    services: {
      redis: redisOk ? 'connected' : 'disconnected',
      livekit: livekitOk ? 'connected' : 'disconnected',
      chain: chainOk ? 'connected' : 'disconnected',
    },
  });
});

// API 路由
app.use('/api/livestream', livestreamRoutes);

// 错误处理
app.use(errorMiddleware);

export default app;
```

### WebSocket 服务 (websocket.ts)

```typescript
// backend/src/websocket.ts

import { Server, Socket } from 'socket.io';
import { httpServer } from './app';
import { config } from './config';
import { logger } from './utils/logger';

export class WebSocketService {
  private io: Server;
  private userSockets: Map<string, Set<string>> = new Map(); // address -> socket ids
  
  constructor() {
    this.io = new Server(httpServer, {
      cors: {
        origin: config.cors.origin,
        methods: ['GET', 'POST'],
      },
      pingTimeout: 60000,
      pingInterval: 25000,
    });
    
    this.setupEventHandlers();
  }
  
  private setupEventHandlers(): void {
    this.io.on('connection', (socket: Socket) => {
      logger.info({ socketId: socket.id }, 'Client connected');
      
      // 用户认证 (可选)
      socket.on('auth', (address: string) => {
        this.registerUser(address, socket.id);
        socket.data.address = address;
        logger.info({ socketId: socket.id, address }, 'User authenticated');
      });
      
      // 加入直播间
      socket.on('join-room', (roomId: number) => {
        const roomName = `room-${roomId}`;
        socket.join(roomName);
        logger.info({ socketId: socket.id, roomId }, 'Joined room');
        
        // 通知房间内其他人
        socket.to(roomName).emit('viewer:joined', {
          address: socket.data.address,
        });
      });
      
      // 离开直播间
      socket.on('leave-room', (roomId: number) => {
        const roomName = `room-${roomId}`;
        socket.leave(roomName);
        logger.info({ socketId: socket.id, roomId }, 'Left room');
        
        // 通知房间内其他人
        socket.to(roomName).emit('viewer:left', {
          address: socket.data.address,
        });
      });
      
      // 断开连接
      socket.on('disconnect', () => {
        if (socket.data.address) {
          this.unregisterUser(socket.data.address, socket.id);
        }
        logger.info({ socketId: socket.id }, 'Client disconnected');
      });
    });
  }
  
  private registerUser(address: string, socketId: string): void {
    if (!this.userSockets.has(address)) {
      this.userSockets.set(address, new Set());
    }
    this.userSockets.get(address)!.add(socketId);
  }
  
  private unregisterUser(address: string, socketId: string): void {
    const sockets = this.userSockets.get(address);
    if (sockets) {
      sockets.delete(socketId);
      if (sockets.size === 0) {
        this.userSockets.delete(address);
      }
    }
  }
  
  /**
   * 广播到指定直播间
   */
  broadcastToRoom(roomId: number, event: string, data: any): void {
    const roomName = `room-${roomId}`;
    this.io.to(roomName).emit(event, data);
    logger.debug({ roomId, event }, 'Broadcast to room');
  }
  
  /**
   * 广播到所有连接
   */
  broadcastToAll(event: string, data: any): void {
    this.io.emit(event, data);
    logger.debug({ event }, 'Broadcast to all');
  }
  
  /**
   * 通知特定用户
   */
  notifyUser(address: string, event: string, data: any): void {
    const socketIds = this.userSockets.get(address);
    if (socketIds) {
      socketIds.forEach((socketId) => {
        this.io.to(socketId).emit(event, data);
      });
      logger.debug({ address, event }, 'Notified user');
    }
  }
  
  /**
   * 获取直播间在线人数
   */
  async getRoomViewerCount(roomId: number): Promise<number> {
    const roomName = `room-${roomId}`;
    const sockets = await this.io.in(roomName).fetchSockets();
    return sockets.length;
  }
  
  /**
   * 踢出用户
   */
  async kickUserFromRoom(roomId: number, address: string): Promise<void> {
    const roomName = `room-${roomId}`;
    const socketIds = this.userSockets.get(address);
    
    if (socketIds) {
      for (const socketId of socketIds) {
        const socket = this.io.sockets.sockets.get(socketId);
        if (socket && socket.rooms.has(roomName)) {
          socket.leave(roomName);
          socket.emit('viewer:kicked', { roomId, message: 'You have been kicked' });
        }
      }
    }
  }
}

export const websocketService = new WebSocketService();
```

### 启动文件 (index.ts)

```typescript
// backend/src/index.ts

import { httpServer } from './app';
import { config, validateConfig } from './config';
import { logger } from './utils/logger';
import { chainService } from './services/chain.service';
import { cacheService } from './services/cache.service';
import { chainEventsService } from './services/chain-events.service';
import './websocket'; // 初始化 WebSocket

async function main() {
  try {
    // 验证配置
    validateConfig();
    
    // 连接 Redis
    const redisOk = await cacheService.ping();
    if (!redisOk) {
      throw new Error('Failed to connect to Redis');
    }
    logger.info('Redis connected');
    
    // 连接链
    await chainService.connect();
    
    // 启动链上事件监听
    await chainEventsService.startListening();
    
    // 启动 HTTP + WebSocket 服务
    httpServer.listen(config.port, () => {
      logger.info({ port: config.port }, 'Server started (HTTP + WebSocket)');
    });
    
    // 优雅关闭
    process.on('SIGTERM', async () => {
      logger.info('SIGTERM received, shutting down...');
      chainEventsService.stopListening();
      await cacheService.disconnect();
      httpServer.close(() => {
        logger.info('Server closed');
        process.exit(0);
      });
    });
    
  } catch (error) {
    logger.error({ error }, 'Failed to start server');
    process.exit(1);
  }
}

main();
```

## 部署配置

### package.json

```json
{
  "name": "livestream-backend",
  "version": "1.0.0",
  "scripts": {
    "dev": "tsx watch src/index.ts",
    "build": "tsc",
    "start": "node dist/index.js",
    "lint": "eslint src --ext .ts",
    "test": "vitest"
  },
  "dependencies": {
    "@polkadot/api": "^10.11.1",
    "@polkadot/util-crypto": "^12.6.1",
    "cors": "^2.8.5",
    "dotenv": "^16.3.1",
    "express": "^4.18.2",
    "express-rate-limit": "^7.1.5",
    "helmet": "^7.1.0",
    "ioredis": "^5.3.2",
    "livekit-server-sdk": "^2.0.0",
    "pino": "^8.17.2",
    "socket.io": "^4.7.4",
    "zod": "^3.22.4"
  },
  "devDependencies": {
    "@types/cors": "^2.8.17",
    "@types/express": "^4.17.21",
    "@types/node": "^20.10.6",
    "pino-pretty": "^10.3.1",
    "tsx": "^4.7.0",
    "typescript": "^5.3.3",
    "vitest": "^1.2.0"
  }
}
```

### tsconfig.json

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

### .env.example

```bash
# 服务配置
PORT=3001
NODE_ENV=development

# LiveKit 配置
LIVEKIT_URL=wss://your-livekit-server.com
LIVEKIT_API_KEY=your-api-key
LIVEKIT_API_SECRET=your-api-secret
LIVEKIT_TOKEN_TTL=6h

# Substrate 节点
CHAIN_WS_ENDPOINT=ws://localhost:9944

# Redis 配置 (必需)
REDIS_URL=redis://localhost:6379

# CORS
CORS_ORIGIN=http://localhost:8081
```

### Dockerfile

```dockerfile
FROM node:20-alpine AS builder

WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM node:20-alpine

WORKDIR /app
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
COPY package*.json ./

ENV NODE_ENV=production
EXPOSE 3001

CMD ["node", "dist/index.js"]
```

### docker-compose.yml

```yaml
version: '3.8'

services:
  livestream-backend:
    build: .
    ports:
      - "3001:3001"
    environment:
      - NODE_ENV=production
      - PORT=3001
      - LIVEKIT_URL=${LIVEKIT_URL}
      - LIVEKIT_API_KEY=${LIVEKIT_API_KEY}
      - LIVEKIT_API_SECRET=${LIVEKIT_API_SECRET}
      - CHAIN_WS_ENDPOINT=${CHAIN_WS_ENDPOINT}
      - REDIS_URL=redis://redis:6379
      - CORS_ORIGIN=${CORS_ORIGIN}
    depends_on:
      - redis
    restart: unless-stopped
    
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    command: redis-server --appendonly yes
    restart: unless-stopped
    
  # LiveKit 服务器 (开发环境)
  livekit:
    image: livekit/livekit-server:latest
    ports:
      - "7880:7880"
      - "7881:7881"
      - "7882:7882/udp"
    environment:
      - LIVEKIT_KEYS=devkey: secret
    command: --dev --bind 0.0.0.0

volumes:
  redis-data:
```

## 安全考虑

1. **签名验证**: 主播和观众都需要签名验证，防止身份伪造
2. **时间戳防重放**: 签名消息包含时间戳，5分钟内有效
3. **连麦者链上验证**: 连麦者必须在链上 `ActiveCoHosts` 列表中
4. **限流保护**: 防止 Token 接口被滥用
5. **CORS 配置**: 生产环境限制允许的来源
6. **Helmet**: 设置安全相关的 HTTP 头
7. **链上验证**: 所有权限检查都查询链上数据
8. **参数校验**: 使用 Zod 进行严格的参数校验

## 监控与日志

1. **请求日志**: 记录所有 API 请求
2. **错误日志**: 详细记录错误信息
3. **健康检查**: `/health` 端点检查 Redis、LiveKit、Chain 连接状态
4. **链连接状态**: 监控 Substrate 节点连接
5. **Redis 监控**: 缓存命中率、连接状态

## WebSocket 事件

### 客户端 → 服务端

| 事件 | 参数 | 说明 |
|------|------|------|
| `auth` | `address: string` | 用户认证 |
| `join-room` | `roomId: number` | 加入直播间 |
| `leave-room` | `roomId: number` | 离开直播间 |

### 服务端 → 客户端

| 事件 | 数据 | 说明 |
|------|------|------|
| `room:created` | `{ roomId, host, roomType }` | 新直播间创建 |
| `room:live` | `{ roomId }` | 直播间开播 |
| `live:started` | `{ roomId, startedAt }` | 直播开始 |
| `live:ended` | `{ roomId, duration, ... }` | 直播结束 |
| `gift:received` | `{ roomId, sender, giftId, ... }` | 收到礼物 |
| `cohost:started` | `{ roomId, coHost }` | 连麦开始 |
| `cohost:ended` | `{ roomId, coHost }` | 连麦结束 |
| `viewer:joined` | `{ address }` | 观众加入 |
| `viewer:left` | `{ address }` | 观众离开 |
| `viewer:kicked` | `{ roomId, message }` | 被踢出 |
| `room:banned` | `{ roomId, reason }` | 直播间被封禁 |

## 缓存策略

| 数据类型 | 缓存时间 | 失效策略 |
|----------|----------|----------|
| 直播间信息 | 60 秒 | 状态变化时主动失效 |
| 礼物列表 | 5 分钟 | 礼物更新时主动失效 |
| 黑名单状态 | 30 秒 | 踢人/解封时主动失效 |
| 连麦者列表 | 10 秒 | 连麦变化时主动失效 |
| 在线观众数 | 实时 | 无缓存，直接计数 |

## 后续扩展

1. ~~**Redis 缓存**: 缓存链上数据减少查询~~ ✅ 已实现
2. ~~**WebSocket**: 实时推送直播间状态变化~~ ✅ 已实现
3. ~~**观众统计**: 记录观众进出，统计在线人数~~ ✅ 已实现
4. ~~**礼物通知**: 接收链上礼物事件，推送给前端~~ ✅ 已实现
5. **录制服务**: 集成 LiveKit Egress 录制直播
6. **消息队列**: 使用 Redis Pub/Sub 或 RabbitMQ 处理高并发事件
7. **分布式部署**: 多实例部署时使用 Redis Adapter 同步 Socket.IO 状态
