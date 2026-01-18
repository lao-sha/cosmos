# 直播模块前端页面设计方案

## 概述

本文档描述了基于 `pallet-livestream` 模块的前端页面设计方案，使用 React Native + Expo 框架，集成 LiveKit 实现实时音视频直播功能。

## 技术栈

- **框架**: React Native + Expo (Development Build)
- **状态管理**: Zustand
- **链交互**: @polkadot/api
- **直播服务**: LiveKit (@livekit/react-native + @livekit/react-native-webrtc)
- **内容存储**: IPFS

> ⚠️ **重要**: LiveKit 需要原生模块支持，必须使用 Expo Development Build，Expo Go 不支持。

## 目录结构

```
frontend/src/
├── features/
│   └── livestream/
│       ├── components/
│       │   ├── LivePlayer.tsx           # 直播播放器
│       │   ├── LiveControls.tsx         # 主播控制面板
│       │   ├── GiftPanel.tsx            # 礼物面板
│       │   ├── GiftAnimation.tsx        # 礼物动画
│       │   ├── LiveChat.tsx             # 直播聊天
│       │   ├── DanmakuOverlay.tsx       # 弹幕层
│       │   ├── ViewerList.tsx           # 观众列表
│       │   ├── CoHostPanel.tsx          # 连麦面板
│       │   ├── CoHostLayout.tsx         # 连麦布局
│       │   ├── RoomCard.tsx             # 直播间卡片
│       │   ├── RoomHeader.tsx           # 直播间头部
│       │   ├── TicketModal.tsx          # 购票弹窗
│       │   └── LiveStats.tsx            # 直播统计
│       ├── screens/
│       │   ├── LiveListScreen.tsx       # 直播列表页
│       │   ├── LiveRoomScreen.tsx       # 直播观看页
│       │   ├── LiveHostScreen.tsx       # 主播开播页
│       │   ├── CreateRoomScreen.tsx     # 创建直播间页
│       │   └── MyLiveScreen.tsx         # 我的直播页
│       ├── hooks/
│       │   ├── useLiveRoom.ts           # 直播间逻辑
│       │   ├── useLiveKit.ts            # LiveKit 集成
│       │   ├── useGifts.ts              # 礼物系统
│       │   ├── useDanmaku.ts            # 弹幕系统
│       │   └── useCoHost.ts             # 连麦功能
│       ├── services/
│       │   ├── livestream.service.ts    # 直播业务逻辑
│       │   ├── livekit.service.ts       # LiveKit 服务
│       │   └── gift.service.ts          # 礼物服务
│       └── types.ts                     # 类型定义
│
├── stores/
│   └── livestream.store.ts              # 直播状态管理
│
└── app/
    └── livestream/
        ├── index.tsx                    # 直播列表入口
        ├── [roomId].tsx                 # 直播观看页
        ├── host.tsx                     # 主播开播页
        └── create.tsx                   # 创建直播间页
```

## 核心类型定义

```typescript
// frontend/src/features/livestream/types.ts

/** 直播间状态 */
export enum LiveRoomStatus {
  Preparing = 'Preparing',   // 准备中
  Live = 'Live',             // 直播中
  Paused = 'Paused',         // 暂停中
  Ended = 'Ended',           // 已结束
  Banned = 'Banned',         // 被封禁
}

/** 直播间类型 */
export enum LiveRoomType {
  Normal = 'Normal',         // 普通直播
  Paid = 'Paid',             // 付费直播
  Private = 'Private',       // 私密直播
  MultiHost = 'MultiHost',   // 连麦直播
}

/** 直播间信息 */
export interface LiveRoom {
  id: number;
  host: string;              // 主播地址
  hostName?: string;         // 主播昵称
  hostAvatar?: string;       // 主播头像
  title: string;
  description?: string;
  roomType: LiveRoomType;
  status: LiveRoomStatus;
  coverCid?: string;         // 封面 IPFS CID
  totalViewers: number;      // 累计观众
  peakViewers: number;       // 峰值观众
  currentViewers: number;    // 当前观众 (链下)
  totalGifts: string;        // 累计礼物收入
  ticketPrice?: string;      // 门票价格
  createdAt: number;
  startedAt?: number;
  endedAt?: number;
}

/** 礼物定义 */
export interface Gift {
  id: number;
  name: string;
  price: string;
  iconCid: string;
  enabled: boolean;
}

/** 礼物记录 (事件) */
export interface GiftRecord {
  sender: string;
  senderName?: string;
  receiver: string;
  roomId: number;
  giftId: number;
  giftName: string;
  quantity: number;
  totalValue: string;
  timestamp: number;
}

/** 连麦者信息 */
export interface CoHost {
  address: string;
  name?: string;
  avatar?: string;
  type: 'audio' | 'video';
  isMuted: boolean;
}

/** 弹幕消息 */
export interface DanmakuMessage {
  id: string;
  content: string;
  sender: string;
  senderName?: string;
  color: string;
  position: 'scroll' | 'top' | 'bottom';
  timestamp: number;
}

/** 聊天消息 */
export interface LiveChatMessage {
  type: 'chat' | 'danmaku' | 'gift' | 'system' | 'like';
  content: string;
  sender: string;
  senderName: string;
  timestamp: number;
  giftId?: number;
  giftCount?: number;
  giftValue?: string;
  color?: string;
}
```

## 页面设计

### 1. 直播列表页 (LiveListScreen)

展示所有正在直播的房间列表。

```
┌─────────────────────────────────────┐
│  直播大厅                    🔍 开播 │
├─────────────────────────────────────┤
│  [全部] [付费] [连麦] [私密]         │
├─────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐          │
│  │ 封面图   │  │ 封面图   │          │
│  │         │  │         │          │
│  │ 👤 1.2k │  │ 👤 856  │          │
│  ├─────────┤  ├─────────┤          │
│  │ 主播名   │  │ 主播名   │          │
│  │ 直播标题 │  │ 直播标题 │          │
│  └─────────┘  └─────────┘          │
│                                     │
│  ┌─────────┐  ┌─────────┐          │
│  │ 封面图   │  │ 封面图   │          │
│  │ 💰 10   │  │         │          │
│  │ 👤 432  │  │ 👤 128  │          │
│  ├─────────┤  ├─────────┤          │
│  │ 主播名   │  │ 主播名   │          │
│  │ 付费直播 │  │ 直播标题 │          │
│  └─────────┘  └─────────┘          │
└─────────────────────────────────────┘
```

**功能点**:
- 分类筛选 (全部/付费/连麦/私密)
- 下拉刷新
- 无限滚动加载
- 点击进入直播间
- 右上角开播入口

### 2. 直播观看页 (LiveRoomScreen)

观众观看直播的主界面。

```
┌─────────────────────────────────────┐
│  ← 主播名 · 直播中        👤 1.2k   │
├─────────────────────────────────────┤
│                                     │
│         ┌─────────────────┐         │
│         │                 │         │
│         │   直播画面      │         │
│         │                 │         │
│         │   ～弹幕飘过～   │         │
│         │                 │         │
│         └─────────────────┘         │
│                                     │
├─────────────────────────────────────┤
│  聊天消息区域                        │
│  ┌─────────────────────────────────┐│
│  │ 用户A: 主播好厉害！              ││
│  │ 用户B: 666                       ││
│  │ 🎁 用户C 送出 火箭 x1            ││
│  │ 用户D: 求连麦                    ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  [输入消息...]        [弹幕] [礼物] │
└─────────────────────────────────────┘
```

**功能点**:
- 视频播放 (LiveKit)
- 弹幕显示
- 聊天消息
- 礼物发送
- 连麦申请
- 分享直播间

### 3. 礼物面板 (GiftPanel)

```
┌─────────────────────────────────────┐
│  礼物                          ✕    │
├─────────────────────────────────────┤
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐   │
│  │ 🌹  │ │ ❤️  │ │ 🎂  │ │ 🚀  │   │
│  │ 玫瑰 │ │ 爱心 │ │ 蛋糕 │ │ 火箭 │   │
│  │ 1    │ │ 5    │ │ 10   │ │ 100  │   │
│  └─────┘ └─────┘ └─────┘ └─────┘   │
│                                     │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐   │
│  │ 🏎️  │ │ ✈️  │ │ 🏰  │ │ 🌟  │   │
│  │ 跑车 │ │ 飞机 │ │ 城堡 │ │ 星星 │   │
│  │ 500  │ │ 1000 │ │ 5000 │ │ 10   │   │
│  └─────┘ └─────┘ └─────┘ └─────┘   │
├─────────────────────────────────────┤
│  数量: [-] 1 [+]     余额: 1000 DUST│
│                                     │
│  [      发送礼物 (100 DUST)      ]  │
└─────────────────────────────────────┘
```

### 4. 主播开播页 (LiveHostScreen)

主播推流界面。

```
┌─────────────────────────────────────┐
│  ← 我的直播间              [结束直播]│
├─────────────────────────────────────┤
│                                     │
│         ┌─────────────────┐         │
│         │                 │         │
│         │   摄像头预览    │         │
│         │                 │         │
│         │                 │         │
│         │                 │         │
│         └─────────────────┘         │
│                                     │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐   │
│  │ 🔄  │ │ 🎤  │ │ 📷  │ │ 💄  │   │
│  │翻转 │ │静音 │ │关闭 │ │美颜 │   │
│  └─────┘ └─────┘ └─────┘ └─────┘   │
├─────────────────────────────────────┤
│  👤 1.2k    💰 5,230 DUST    ⏱ 1:23:45│
├─────────────────────────────────────┤
│  聊天消息 / 连麦申请                 │
│  ┌─────────────────────────────────┐│
│  │ [聊天] [连麦申请(3)]            ││
│  │                                 ││
│  │ 用户A 申请连麦 [同意] [拒绝]    ││
│  │ 用户B 申请连麦 [同意] [拒绝]    ││
│  └─────────────────────────────────┘│
└─────────────────────────────────────┘
```

**功能点**:
- 摄像头预览/推流
- 翻转摄像头
- 静音/取消静音
- 开关摄像头
- 美颜滤镜
- 查看观众数/收益
- 管理连麦申请
- 踢出/拉黑观众

### 5. 创建直播间页 (CreateRoomScreen)

```
┌─────────────────────────────────────┐
│  ← 创建直播间                       │
├─────────────────────────────────────┤
│                                     │
│  ┌─────────────────────────────────┐│
│  │                                 ││
│  │      点击上传封面图             ││
│  │         📷                      ││
│  │                                 ││
│  └─────────────────────────────────┘│
│                                     │
│  直播标题                           │
│  ┌─────────────────────────────────┐│
│  │ 输入直播标题...                 ││
│  └─────────────────────────────────┘│
│                                     │
│  直播简介                           │
│  ┌─────────────────────────────────┐│
│  │ 输入直播简介...                 ││
│  │                                 ││
│  └─────────────────────────────────┘│
│                                     │
│  直播类型                           │
│  ○ 普通直播  ○ 付费直播            │
│  ○ 私密直播  ○ 连麦直播            │
│                                     │
│  门票价格 (付费直播)                │
│  ┌─────────────────────────────────┐│
│  │ 10                         DUST ││
│  └─────────────────────────────────┘│
│                                     │
│  [        创建直播间        ]       │
│                                     │
│  ⚠️ 创建直播间需要押金 100 DUST     │
└─────────────────────────────────────┘
```

### 6. 连麦布局

```
画中画模式:
┌─────────────────────────────────────┐
│                                     │
│         主播画面                    │
│                                     │
│                        ┌─────┐      │
│                        │连麦1│      │
│                        └─────┘      │
│                        ┌─────┐      │
│                        │连麦2│      │
│                        └─────┘      │
└─────────────────────────────────────┘

分屏模式:
┌─────────────────────────────────────┐
│ ┌───────────────┐ ┌───────────────┐ │
│ │               │ │               │ │
│ │   主播画面    │ │   连麦画面    │ │
│ │               │ │               │ │
│ └───────────────┘ └───────────────┘ │
└─────────────────────────────────────┘

网格模式 (多人连麦):
┌─────────────────────────────────────┐
│ ┌───────────┐ ┌───────────┐        │
│ │   主播    │ │   连麦1   │        │
│ └───────────┘ └───────────┘        │
│ ┌───────────┐ ┌───────────┐        │
│ │   连麦2   │ │   连麦3   │        │
│ └───────────┘ └───────────┘        │
└─────────────────────────────────────┘
```

## 状态管理

```typescript
// frontend/src/stores/livestream.store.ts

import { create } from 'zustand';
import type { LiveRoom, Gift, GiftRecord, CoHost, LiveChatMessage } from '@/features/livestream/types';

interface LivestreamState {
  // 直播列表
  rooms: LiveRoom[];
  isLoadingRooms: boolean;
  
  // 当前直播间
  currentRoom: LiveRoom | null;
  isInRoom: boolean;
  
  // 礼物
  gifts: Gift[];
  giftRecords: GiftRecord[];
  
  // 连麦
  coHosts: CoHost[];
  coHostRequests: { address: string; name: string; type: 'audio' | 'video' }[];
  
  // 聊天
  messages: LiveChatMessage[];
  
  // 主播状态
  isHost: boolean;
  isLive: boolean;
  earnings: string;
  
  // 操作
  loadRooms: (filter?: string) => Promise<void>;
  joinRoom: (roomId: number) => Promise<void>;
  leaveRoom: () => Promise<void>;
  
  // 主播操作
  createRoom: (params: CreateRoomParams) => Promise<number>;
  startLive: (roomId: number) => Promise<void>;
  pauseLive: (roomId: number) => Promise<void>;
  endLive: (roomId: number) => Promise<void>;
  
  // 礼物操作
  loadGifts: () => Promise<void>;
  sendGift: (roomId: number, giftId: number, quantity: number) => Promise<void>;
  
  // 连麦操作
  requestCoHost: (type: 'audio' | 'video') => Promise<void>;
  acceptCoHost: (address: string) => Promise<void>;
  rejectCoHost: (address: string) => Promise<void>;
  endCoHost: (address?: string) => Promise<void>;
  
  // 聊天操作
  sendChat: (content: string) => Promise<void>;
  sendDanmaku: (content: string, color?: string) => Promise<void>;
  
  // 管理操作
  kickViewer: (address: string) => Promise<void>;
  buyTicket: (roomId: number) => Promise<void>;
}

interface CreateRoomParams {
  title: string;
  description?: string;
  roomType: LiveRoomType;
  coverCid?: string;
  ticketPrice?: string;
}

export const useLivestreamStore = create<LivestreamState>()((set, get) => ({
  rooms: [],
  isLoadingRooms: false,
  currentRoom: null,
  isInRoom: false,
  gifts: [],
  giftRecords: [],
  coHosts: [],
  coHostRequests: [],
  messages: [],
  isHost: false,
  isLive: false,
  earnings: '0',
  
  loadRooms: async (filter) => {
    set({ isLoadingRooms: true });
    try {
      const service = getLivestreamService();
      const rooms = await service.getLiveRooms(filter);
      set({ rooms });
    } finally {
      set({ isLoadingRooms: false });
    }
  },
  
  joinRoom: async (roomId) => {
    const service = getLivestreamService();
    const room = await service.getRoomInfo(roomId);
    
    // 检查是否需要购票
    if (room.roomType === 'Paid') {
      const hasTicket = await service.checkTicket(roomId);
      if (!hasTicket) {
        throw new Error('TICKET_REQUIRED');
      }
    }
    
    // 连接 LiveKit
    await service.connectLiveKit(roomId);
    
    set({ 
      currentRoom: room, 
      isInRoom: true,
      isHost: room.host === service.myAddress,
    });
  },
  
  sendGift: async (roomId, giftId, quantity) => {
    const service = getLivestreamService();
    await service.sendGift(roomId, giftId, quantity);
    
    // 发送礼物通知到聊天
    const gift = get().gifts.find(g => g.id === giftId);
    if (gift) {
      await service.sendGiftNotification(giftId, quantity, gift.price);
    }
  },
  
  // ... 其他方法实现
}));
```


## 核心服务

### IPFS 服务集成

直播模块使用项目中已有的 `@/services/ipfs.service.ts` 进行内容存储。

#### 使用场景

| 场景 | 说明 |
|------|------|
| 封面图上传 | 创建直播间时上传封面图到 IPFS |
| 礼物图标 | 管理员上传礼物图标到 IPFS |
| 直播回放 | 直播结束后保存回放到 IPFS (可选) |

#### 封面图上传示例

```typescript
// frontend/src/features/livestream/services/cover.service.ts

import { uploadToIpfs, getCidUrl } from '@/services/ipfs.service';
import * as ImagePicker from 'expo-image-picker';
import * as ImageManipulator from 'expo-image-manipulator';

/**
 * 选择并上传直播封面图
 */
export async function pickAndUploadCover(): Promise<{ cid: string; url: string }> {
  // 1. 选择图片
  const result = await ImagePicker.launchImageLibraryAsync({
    mediaTypes: ImagePicker.MediaTypeOptions.Images,
    allowsEditing: true,
    aspect: [16, 9],
    quality: 0.8,
  });

  if (result.canceled || !result.assets[0]) {
    throw new Error('User cancelled image picker');
  }

  const imageUri = result.assets[0].uri;

  // 2. 压缩图片 (推荐 1280x720)
  const manipulated = await ImageManipulator.manipulateAsync(
    imageUri,
    [{ resize: { width: 1280, height: 720 } }],
    { compress: 0.8, format: ImageManipulator.SaveFormat.JPEG }
  );

  // 3. 读取图片数据
  const response = await fetch(manipulated.uri);
  const blob = await response.blob();
  const arrayBuffer = await blob.arrayBuffer();
  const imageData = new Uint8Array(arrayBuffer);

  // 4. 上传到 IPFS
  const cid = await uploadToIpfs(imageData);

  return {
    cid,
    url: getCidUrl(cid),
  };
}

/**
 * 从 CID 获取封面图 URL
 */
export function getCoverUrl(cid: string | undefined): string | undefined {
  if (!cid) return undefined;
  return getCidUrl(cid);
}
```

#### 在创建直播间时使用

```typescript
// CreateRoomScreen.tsx 中使用

import { pickAndUploadCover } from '../services/cover.service';

const [coverCid, setCoverCid] = useState<string>();
const [coverUrl, setCoverUrl] = useState<string>();
const [isUploading, setIsUploading] = useState(false);

const handlePickCover = async () => {
  setIsUploading(true);
  try {
    const { cid, url } = await pickAndUploadCover();
    setCoverCid(cid);
    setCoverUrl(url);
  } catch (error) {
    console.error('上传封面失败:', error);
    Alert.alert('错误', '上传封面失败，请重试');
  } finally {
    setIsUploading(false);
  }
};

// 创建直播间时传入 coverCid
const handleCreateRoom = async () => {
  await createRoom({
    title,
    description,
    roomType,
    coverCid,  // IPFS CID
    ticketPrice,
  });
};
```

#### 环境变量配置

```bash
# .env - IPFS 配置

# 自建 IPFS 节点 (可选)
EXPO_PUBLIC_IPFS_API=http://localhost:5001/api/v0

# IPFS 网关
EXPO_PUBLIC_IPFS_GATEWAY=https://gateway.pinata.cloud/ipfs/

# Pinata 配置 (推荐生产环境使用)
EXPO_PUBLIC_PINATA_API_KEY=your-pinata-api-key
EXPO_PUBLIC_PINATA_API_SECRET=your-pinata-api-secret

# Web3.Storage 配置 (备选)
EXPO_PUBLIC_WEB3_STORAGE_TOKEN=your-web3-storage-token
```

#### 直播回放保存 (后续扩展)

```typescript
// 使用 LiveKit Egress 录制直播，结束后上传到 IPFS
// 这需要后端服务配合实现

interface RecordingService {
  // 开始录制
  startRecording(roomId: number): Promise<string>;
  
  // 停止录制并上传到 IPFS
  stopAndUpload(recordingId: string): Promise<{ cid: string; duration: number }>;
}
```

### 直播服务 (livestream.service.ts)

```typescript
// frontend/src/features/livestream/services/livestream.service.ts

import { ApiPromise } from '@polkadot/api';
import { getApi } from '@/api';
import { LiveKitService } from './livekit.service';
import type { LiveRoom, Gift, LiveRoomType, LiveRoomStatus } from '../types';

export class LivestreamService {
  private api: ApiPromise | null = null;
  private livekit: LiveKitService | null = null;
  public myAddress: string;
  
  constructor(myAddress: string) {
    this.myAddress = myAddress;
  }
  
  async init(): Promise<void> {
    this.api = await getApi();
    this.livekit = new LiveKitService();
  }
  
  /**
   * 获取直播间列表
   */
  async getLiveRooms(filter?: string): Promise<LiveRoom[]> {
    if (!this.api) throw new Error('API not initialized');
    
    const entries = await this.api.query.livestream.liveRooms.entries();
    const rooms: LiveRoom[] = [];
    
    for (const [key, value] of entries) {
      if (value.isNone) continue;
      const data = value.unwrap();
      
      // 只返回直播中的房间
      if (data.status.toString() !== 'Live') continue;
      
      // 筛选
      if (filter && data.roomType.toString() !== filter) continue;
      
      rooms.push(this.parseRoom(data));
    }
    
    // 按观众数排序
    return rooms.sort((a, b) => b.currentViewers - a.currentViewers);
  }
  
  /**
   * 获取直播间详情
   */
  async getRoomInfo(roomId: number): Promise<LiveRoom> {
    if (!this.api) throw new Error('API not initialized');
    
    const result = await this.api.query.livestream.liveRooms(roomId);
    if (result.isNone) throw new Error('Room not found');
    
    return this.parseRoom(result.unwrap());
  }
  
  /**
   * 创建直播间
   */
  async createRoom(params: {
    title: string;
    description?: string;
    roomType: LiveRoomType;
    coverCid?: string;
    ticketPrice?: string;
  }): Promise<number> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.livestream.createRoom(
      params.title,
      params.description || null,
      params.roomType,
      params.coverCid || null,
      params.ticketPrice || null
    );
    
    return new Promise((resolve, reject) => {
      tx.signAndSend(this.myAddress, ({ status, events }) => {
        if (status.isInBlock) {
          for (const { event } of events) {
            if (this.api!.events.livestream.RoomCreated.is(event)) {
              const [, roomId] = event.data;
              resolve(roomId.toNumber());
              return;
            }
          }
          reject(new Error('RoomCreated event not found'));
        }
      }).catch(reject);
    });
  }
  
  /**
   * 开始直播
   */
  async startLive(roomId: number): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.livestream.startLive(roomId);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 暂停直播
   */
  async pauseLive(roomId: number): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.livestream.pauseLive(roomId);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 结束直播
   */
  async endLive(roomId: number): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.livestream.endLive(roomId);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 购买门票
   */
  async buyTicket(roomId: number): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.livestream.buyTicket(roomId);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 检查是否有门票
   */
  async checkTicket(roomId: number): Promise<boolean> {
    if (!this.api) return false;
    
    const result = await this.api.query.livestream.ticketHolders(roomId, this.myAddress);
    return result.isSome;
  }
  
  /**
   * 发送礼物
   */
  async sendGift(roomId: number, giftId: number, quantity: number): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.livestream.sendGift(roomId, giftId, quantity);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 获取礼物列表
   */
  async getGifts(): Promise<Gift[]> {
    if (!this.api) throw new Error('API not initialized');
    
    const entries = await this.api.query.livestream.gifts.entries();
    const gifts: Gift[] = [];
    
    for (const [key, value] of entries) {
      if (value.isNone) continue;
      const data = value.unwrap();
      
      if (!data.enabled.valueOf()) continue;
      
      gifts.push({
        id: key.args[0].toNumber(),
        name: data.name.toUtf8(),
        price: data.price.toString(),
        iconCid: data.iconCid.toUtf8(),
        enabled: true,
      });
    }
    
    return gifts.sort((a, b) => Number(a.price) - Number(b.price));
  }
  
  /**
   * 提现收益
   */
  async withdrawEarnings(amount: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.livestream.withdrawEarnings(amount);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 获取主播收益
   */
  async getHostEarnings(): Promise<string> {
    if (!this.api) return '0';
    
    const result = await this.api.query.livestream.hostEarnings(this.myAddress);
    return result.toString();
  }
  
  /**
   * 踢出观众
   */
  async kickViewer(roomId: number, viewer: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.livestream.kickViewer(roomId, viewer);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 开始连麦 (链上记录)
   */
  async startCoHost(roomId: number, coHost: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.livestream.startCoHost(roomId, coHost);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 结束连麦
   */
  async endCoHost(roomId: number, coHost?: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.livestream.endCoHost(roomId, coHost || null);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 连接 LiveKit
   */
  async connectLiveKit(roomId: number): Promise<void> {
    if (!this.livekit) throw new Error('LiveKit not initialized');
    
    // 获取观众 Token
    const token = await this.getViewerToken(roomId);
    await this.livekit.connect(token);
  }
  
  /**
   * 主播推流
   */
  async startPublishing(roomId: number): Promise<void> {
    if (!this.livekit) throw new Error('LiveKit not initialized');
    
    // 获取主播 Token (需要签名验证)
    const token = await this.getPublisherToken(roomId);
    await this.livekit.connect(token);
    await this.livekit.enableCameraAndMicrophone();
  }
  
  /**
   * 获取观众 Token
   */
  private async getViewerToken(roomId: number): Promise<string> {
    const response = await fetch('/api/livestream/viewer-token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ roomId, viewerAddress: this.myAddress }),
    });
    
    if (!response.ok) throw new Error('Failed to get viewer token');
    const { token } = await response.json();
    return token;
  }
  
  /**
   * 获取主播 Token (需要签名)
   * 
   * ⚠️ 注意: React Native 不能使用 @polkadot/extension-dapp (浏览器扩展 API)
   * 必须使用项目中的 signer.native.ts 或 keyring.service.ts 进行签名
   */
  private async getPublisherToken(roomId: number): Promise<string> {
    const timestamp = Date.now();
    const message = `livestream:${roomId}:${timestamp}`;
    
    // 使用移动端签名器签名
    const signature = await this.signMessageNative(message);
    
    const response = await fetch('/api/livestream/publisher-token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ 
        roomId, 
        publicKey: this.myAddress, 
        signature, 
        timestamp 
      }),
    });
    
    if (!response.ok) throw new Error('Failed to get publisher token');
    const { token } = await response.json();
    return token;
  }
  
  /**
   * 使用移动端签名器签名消息
   * 
   * 方式一: 使用 MobileSigner (推荐，需要先解锁钱包)
   * 方式二: 使用 KeyringService
   */
  private async signMessageNative(message: string): Promise<string> {
    // 方式一: 使用 MobileSigner
    const { getCurrentPair } = await import('@/lib/signer.native');
    const pair = getCurrentPair();
    
    if (!pair) {
      throw new Error('Wallet is locked. Please unlock first.');
    }
    
    // 将消息转换为 Uint8Array 并签名
    const messageU8a = new TextEncoder().encode(message);
    const signatureU8a = pair.sign(messageU8a);
    
    // 转换为十六进制字符串
    const { u8aToHex } = await import('@polkadot/util');
    return u8aToHex(signatureU8a);
  }
  
  private parseRoom(data: any): LiveRoom {
    return {
      id: data.id.toNumber(),
      host: data.host.toString(),
      title: data.title.toUtf8(),
      description: data.description.isSome ? data.description.unwrap().toUtf8() : undefined,
      roomType: data.roomType.toString() as LiveRoomType,
      status: data.status.toString() as LiveRoomStatus,
      coverCid: data.coverCid.isSome ? data.coverCid.unwrap().toUtf8() : undefined,
      totalViewers: data.totalViewers.toNumber(),
      peakViewers: data.peakViewers.toNumber(),
      currentViewers: 0, // 从 LiveKit 获取
      totalGifts: data.totalGifts.toString(),
      ticketPrice: data.ticketPrice.isSome ? data.ticketPrice.unwrap().toString() : undefined,
      createdAt: data.createdAt.toNumber(),
      startedAt: data.startedAt.isSome ? data.startedAt.unwrap().toNumber() : undefined,
      endedAt: data.endedAt.isSome ? data.endedAt.unwrap().toNumber() : undefined,
    };
  }
}

// 单例
let serviceInstance: LivestreamService | null = null;

export function getLivestreamService(): LivestreamService {
  if (!serviceInstance) throw new Error('LivestreamService not initialized');
  return serviceInstance;
}

export function initLivestreamService(myAddress: string): LivestreamService {
  serviceInstance = new LivestreamService(myAddress);
  return serviceInstance;
}
```

### LiveKit 服务 (livekit.service.ts)

> ⚠️ **React Native 注意事项**:
> - 必须使用 `@livekit/react-native` 而非 `livekit-client` (Web SDK)
> - 需要 `@livekit/react-native-webrtc` 提供原生 WebRTC 支持
> - Expo Go 不支持，必须使用 Development Build 或运行 `expo prebuild`
> - iOS/Android 需要配置相机和麦克风权限

```typescript
// frontend/src/features/livestream/services/livekit.service.ts

import { 
  Room, 
  RoomEvent, 
  Track, 
  Participant, 
  LocalParticipant,
  AudioSession,
  registerGlobals,
} from '@livekit/react-native';

const LIVEKIT_URL = process.env.EXPO_PUBLIC_LIVEKIT_URL || 'wss://your-livekit-server.com';

// 必须在应用启动时调用一次
let isInitialized = false;

export async function initializeLiveKit(): Promise<void> {
  if (isInitialized) return;
  
  // 注册 LiveKit 全局依赖 (WebRTC polyfills)
  registerGlobals();
  
  // 配置音频会话 (iOS 需要)
  await AudioSession.configureAudio({
    android: {
      preferredOutputList: ['speaker'],
      audioTypeOptions: {
        manageAudioFocus: true,
        audioMode: 'normal',
        audioFocusMode: 'gain',
      },
    },
    ios: {
      defaultOutput: 'speaker',
    },
  });
  
  // 启动音频会话
  await AudioSession.startAudioSession();
  
  isInitialized = true;
}

export class LiveKitService {
  private room: Room | null = null;
  private onTrackSubscribed?: (track: Track, participant: Participant) => void;
  private onParticipantConnected?: (participant: Participant) => void;
  private onParticipantDisconnected?: (participant: Participant) => void;
  private onDataReceived?: (data: Uint8Array, participant: Participant) => void;
  
  /**
   * 连接到 LiveKit 房间
   */
  async connect(token: string): Promise<void> {
    // 确保 LiveKit 已初始化
    await initializeLiveKit();
    
    this.room = new Room();
    
    // 设置事件监听
    this.setupEventListeners();
    
    await this.room.connect(LIVEKIT_URL, token, {});
  }
  
  /**
   * 断开连接
   */
  async disconnect(): Promise<void> {
    if (this.room) {
      await this.room.disconnect();
      this.room = null;
    }
  }
  
  /**
   * 开启摄像头和麦克风 (主播)
   */
  async enableCameraAndMicrophone(): Promise<void> {
    if (!this.room) throw new Error('Not connected');
    await this.room.localParticipant.enableCameraAndMicrophone();
  }
  
  /**
   * 切换摄像头 (前置/后置)
   */
  async switchCamera(): Promise<void> {
    if (!this.room) return;
    
    const videoTrack = this.room.localParticipant
      .getTrackPublication(Track.Source.Camera)?.track;
    
    if (videoTrack) {
      // React Native 中使用 switchCamera 方法
      await (videoTrack as any).restartTrack({
        facingMode: (videoTrack as any).facingMode === 'user' ? 'environment' : 'user',
      });
    }
  }
  
  /**
   * 切换麦克风静音
   */
  async toggleMute(): Promise<boolean> {
    if (!this.room) return false;
    
    const enabled = this.room.localParticipant.isMicrophoneEnabled;
    await this.room.localParticipant.setMicrophoneEnabled(!enabled);
    return !enabled;
  }
  
  /**
   * 切换摄像头开关
   */
  async toggleCamera(): Promise<boolean> {
    if (!this.room) return false;
    
    const enabled = this.room.localParticipant.isCameraEnabled;
    await this.room.localParticipant.setCameraEnabled(!enabled);
    return !enabled;
  }
  
  /**
   * 发送数据消息 (聊天/弹幕)
   */
  async sendData(data: object): Promise<void> {
    if (!this.room) throw new Error('Not connected');
    
    const encoded = new TextEncoder().encode(JSON.stringify(data));
    await this.room.localParticipant.publishData(encoded, { reliable: true });
  }
  
  /**
   * 获取当前观众数
   */
  getParticipantCount(): number {
    if (!this.room) return 0;
    return this.room.remoteParticipants.size + 1;
  }
  
  /**
   * 获取本地参与者
   */
  getLocalParticipant(): LocalParticipant | null {
    return this.room?.localParticipant || null;
  }
  
  /**
   * 获取 Room 实例 (用于 React Hooks)
   */
  getRoom(): Room | null {
    return this.room;
  }
  
  /**
   * 设置轨道订阅回调
   */
  setOnTrackSubscribed(callback: (track: Track, participant: Participant) => void): void {
    this.onTrackSubscribed = callback;
  }
  
  /**
   * 设置参与者连接回调
   */
  setOnParticipantConnected(callback: (participant: Participant) => void): void {
    this.onParticipantConnected = callback;
  }
  
  /**
   * 设置参与者断开回调
   */
  setOnParticipantDisconnected(callback: (participant: Participant) => void): void {
    this.onParticipantDisconnected = callback;
  }
  
  /**
   * 设置数据接收回调
   */
  setOnDataReceived(callback: (data: Uint8Array, participant: Participant) => void): void {
    this.onDataReceived = callback;
  }
  
  private setupEventListeners(): void {
    if (!this.room) return;
    
    this.room.on(RoomEvent.TrackSubscribed, (track, publication, participant) => {
      this.onTrackSubscribed?.(track, participant);
    });
    
    this.room.on(RoomEvent.ParticipantConnected, (participant) => {
      this.onParticipantConnected?.(participant);
    });
    
    this.room.on(RoomEvent.ParticipantDisconnected, (participant) => {
      this.onParticipantDisconnected?.(participant);
    });
    
    this.room.on(RoomEvent.DataReceived, (payload, participant) => {
      this.onDataReceived?.(payload, participant!);
    });
    
    this.room.on(RoomEvent.Disconnected, () => {
      console.log('LiveKit disconnected');
    });
    
    this.room.on(RoomEvent.Reconnecting, () => {
      console.log('LiveKit reconnecting...');
    });
    
    this.room.on(RoomEvent.Reconnected, () => {
      console.log('LiveKit reconnected');
    });
  }
}
```

## 核心组件

### 直播播放器 (LivePlayer.tsx)

```typescript
// frontend/src/features/livestream/components/LivePlayer.tsx

import React, { useEffect, useState } from 'react';
import { View, StyleSheet, Dimensions, ActivityIndicator, Text } from 'react-native';
import { VideoView } from '@livekit/react-native';
import { Track } from 'livekit-client';
import { DanmakuOverlay } from './DanmakuOverlay';

const { width: SCREEN_WIDTH } = Dimensions.get('window');

interface LivePlayerProps {
  videoTrack: Track | null;
  isLoading?: boolean;
  showDanmaku?: boolean;
  roomId: number;
}

export function LivePlayer({ 
  videoTrack, 
  isLoading = false, 
  showDanmaku = true,
  roomId,
}: LivePlayerProps) {
  if (isLoading) {
    return (
      <View style={styles.container}>
        <ActivityIndicator size="large" color="#FF4757" />
        <Text style={styles.loadingText}>正在连接直播...</Text>
      </View>
    );
  }
  
  if (!videoTrack) {
    return (
      <View style={styles.container}>
        <Text style={styles.offlineText}>主播暂时离开</Text>
      </View>
    );
  }
  
  return (
    <View style={styles.container}>
      <VideoView 
        style={styles.video} 
        videoTrack={videoTrack}
        objectFit="contain"
      />
      {showDanmaku && <DanmakuOverlay roomId={roomId} />}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    width: SCREEN_WIDTH,
    aspectRatio: 16 / 9,
    backgroundColor: '#000',
    justifyContent: 'center',
    alignItems: 'center',
  },
  video: {
    width: '100%',
    height: '100%',
  },
  loadingText: {
    color: '#FFF',
    marginTop: 12,
    fontSize: 14,
  },
  offlineText: {
    color: '#999',
    fontSize: 16,
  },
});
```

### 礼物面板 (GiftPanel.tsx)

```typescript
// frontend/src/features/livestream/components/GiftPanel.tsx

import React, { useState } from 'react';
import { 
  View, 
  Text, 
  StyleSheet, 
  TouchableOpacity, 
  FlatList,
  Image,
  Modal,
} from 'react-native';
import { useLivestreamStore } from '@/stores/livestream.store';
import type { Gift } from '../types';

interface GiftPanelProps {
  visible: boolean;
  onClose: () => void;
  roomId: number;
  balance: string;
}

export function GiftPanel({ visible, onClose, roomId, balance }: GiftPanelProps) {
  const { gifts, sendGift } = useLivestreamStore();
  const [selectedGift, setSelectedGift] = useState<Gift | null>(null);
  const [quantity, setQuantity] = useState(1);
  const [isSending, setIsSending] = useState(false);
  
  const handleSend = async () => {
    if (!selectedGift) return;
    
    setIsSending(true);
    try {
      await sendGift(roomId, selectedGift.id, quantity);
      onClose();
    } catch (error) {
      console.error('发送礼物失败:', error);
    } finally {
      setIsSending(false);
    }
  };
  
  const totalPrice = selectedGift 
    ? (Number(selectedGift.price) * quantity).toString() 
    : '0';
  
  const renderGift = ({ item }: { item: Gift }) => (
    <TouchableOpacity
      style={[
        styles.giftItem,
        selectedGift?.id === item.id && styles.giftItemSelected,
      ]}
      onPress={() => setSelectedGift(item)}
    >
      <Image 
        source={{ uri: `https://ipfs.io/ipfs/${item.iconCid}` }}
        style={styles.giftIcon}
      />
      <Text style={styles.giftName}>{item.name}</Text>
      <Text style={styles.giftPrice}>{item.price} DUST</Text>
    </TouchableOpacity>
  );
  
  return (
    <Modal
      visible={visible}
      transparent
      animationType="slide"
      onRequestClose={onClose}
    >
      <View style={styles.overlay}>
        <View style={styles.container}>
          <View style={styles.header}>
            <Text style={styles.title}>礼物</Text>
            <TouchableOpacity onPress={onClose}>
              <Text style={styles.closeBtn}>✕</Text>
            </TouchableOpacity>
          </View>
          
          <FlatList
            data={gifts}
            renderItem={renderGift}
            keyExtractor={(item) => item.id.toString()}
            numColumns={4}
            style={styles.giftList}
          />
          
          <View style={styles.footer}>
            <View style={styles.quantityRow}>
              <Text style={styles.label}>数量:</Text>
              <TouchableOpacity 
                style={styles.quantityBtn}
                onPress={() => setQuantity(Math.max(1, quantity - 1))}
              >
                <Text style={styles.quantityBtnText}>-</Text>
              </TouchableOpacity>
              <Text style={styles.quantity}>{quantity}</Text>
              <TouchableOpacity 
                style={styles.quantityBtn}
                onPress={() => setQuantity(quantity + 1)}
              >
                <Text style={styles.quantityBtnText}>+</Text>
              </TouchableOpacity>
              <Text style={styles.balance}>余额: {balance} DUST</Text>
            </View>
            
            <TouchableOpacity
              style={[styles.sendBtn, !selectedGift && styles.sendBtnDisabled]}
              onPress={handleSend}
              disabled={!selectedGift || isSending}
            >
              <Text style={styles.sendBtnText}>
                {isSending ? '发送中...' : `发送礼物 (${totalPrice} DUST)`}
              </Text>
            </TouchableOpacity>
          </View>
        </View>
      </View>
    </Modal>
  );
}

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.5)',
    justifyContent: 'flex-end',
  },
  container: {
    backgroundColor: '#1A1A2E',
    borderTopLeftRadius: 20,
    borderTopRightRadius: 20,
    paddingBottom: 34,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#333',
  },
  title: {
    color: '#FFF',
    fontSize: 18,
    fontWeight: 'bold',
  },
  closeBtn: {
    color: '#999',
    fontSize: 20,
  },
  giftList: {
    padding: 8,
    maxHeight: 300,
  },
  giftItem: {
    flex: 1,
    alignItems: 'center',
    padding: 12,
    margin: 4,
    borderRadius: 8,
    backgroundColor: '#252540',
  },
  giftItemSelected: {
    backgroundColor: '#FF4757',
  },
  giftIcon: {
    width: 48,
    height: 48,
    marginBottom: 8,
  },
  giftName: {
    color: '#FFF',
    fontSize: 12,
    marginBottom: 4,
  },
  giftPrice: {
    color: '#FFD700',
    fontSize: 11,
  },
  footer: {
    padding: 16,
    borderTopWidth: 1,
    borderTopColor: '#333',
  },
  quantityRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 16,
  },
  label: {
    color: '#FFF',
    marginRight: 12,
  },
  quantityBtn: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#333',
    justifyContent: 'center',
    alignItems: 'center',
  },
  quantityBtnText: {
    color: '#FFF',
    fontSize: 18,
  },
  quantity: {
    color: '#FFF',
    fontSize: 18,
    marginHorizontal: 16,
  },
  balance: {
    color: '#999',
    marginLeft: 'auto',
  },
  sendBtn: {
    backgroundColor: '#FF4757',
    borderRadius: 24,
    paddingVertical: 14,
    alignItems: 'center',
  },
  sendBtnDisabled: {
    backgroundColor: '#555',
  },
  sendBtnText: {
    color: '#FFF',
    fontSize: 16,
    fontWeight: 'bold',
  },
});
```

### 弹幕层 (DanmakuOverlay.tsx)

```typescript
// frontend/src/features/livestream/components/DanmakuOverlay.tsx

import React, { useState, useEffect, useRef } from 'react';
import { View, Text, Animated, StyleSheet, Dimensions } from 'react-native';
import { useLivestreamStore } from '@/stores/livestream.store';

const { width: SCREEN_WIDTH } = Dimensions.get('window');

interface Danmaku {
  id: string;
  content: string;
  color: string;
  translateX: Animated.Value;
  top: number;
}

interface DanmakuOverlayProps {
  roomId: number;
}

export function DanmakuOverlay({ roomId }: DanmakuOverlayProps) {
  const [danmakus, setDanmakus] = useState<Danmaku[]>([]);
  const trackRef = useRef(0);
  const { messages } = useLivestreamStore();
  
  useEffect(() => {
    // 监听弹幕消息
    const danmakuMessages = messages.filter(m => m.type === 'danmaku');
    const lastMessage = danmakuMessages[danmakuMessages.length - 1];
    
    if (lastMessage) {
      addDanmaku(lastMessage.content, lastMessage.color || '#FFFFFF');
    }
  }, [messages]);
  
  const addDanmaku = (content: string, color: string) => {
    const id = `${Date.now()}-${Math.random()}`;
    const translateX = new Animated.Value(SCREEN_WIDTH);
    const top = (trackRef.current % 10) * 30 + 20;
    trackRef.current++;
    
    setDanmakus(prev => [...prev, { id, content, color, translateX, top }]);
    
    Animated.timing(translateX, {
      toValue: -500,
      duration: 8000,
      useNativeDriver: true,
    }).start(() => {
      setDanmakus(prev => prev.filter(d => d.id !== id));
    });
  };
  
  return (
    <View style={StyleSheet.absoluteFill} pointerEvents="none">
      {danmakus.map(d => (
        <Animated.Text
          key={d.id}
          style={[
            styles.danmaku,
            { 
              color: d.color, 
              top: d.top, 
              transform: [{ translateX: d.translateX }] 
            }
          ]}
        >
          {d.content}
        </Animated.Text>
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  danmaku: {
    position: 'absolute',
    fontSize: 16,
    fontWeight: 'bold',
    textShadowColor: '#000',
    textShadowOffset: { width: 1, height: 1 },
    textShadowRadius: 2,
  },
});
```


## 路由配置

```typescript
// frontend/app/livestream/index.tsx - 直播列表页

import React from 'react';
import { LiveListScreen } from '@/features/livestream/screens/LiveListScreen';

export default function LivestreamIndex() {
  return <LiveListScreen />;
}
```

```typescript
// frontend/app/livestream/[roomId].tsx - 直播观看页

import React from 'react';
import { useLocalSearchParams } from 'expo-router';
import { LiveRoomScreen } from '@/features/livestream/screens/LiveRoomScreen';

export default function LiveRoom() {
  const { roomId } = useLocalSearchParams<{ roomId: string }>();
  return <LiveRoomScreen roomId={Number(roomId)} />;
}
```

```typescript
// frontend/app/livestream/host.tsx - 主播开播页

import React from 'react';
import { LiveHostScreen } from '@/features/livestream/screens/LiveHostScreen';

export default function LiveHost() {
  return <LiveHostScreen />;
}
```

```typescript
// frontend/app/livestream/create.tsx - 创建直播间页

import React from 'react';
import { CreateRoomScreen } from '@/features/livestream/screens/CreateRoomScreen';

export default function CreateRoom() {
  return <CreateRoomScreen />;
}
```

## 环境变量配置

```bash
# .env
EXPO_PUBLIC_LIVEKIT_URL=wss://your-livekit-server.com
EXPO_PUBLIC_API_URL=https://your-api-server.com
EXPO_PUBLIC_IPFS_GATEWAY=https://ipfs.io/ipfs/
```

## 依赖安装与项目配置

### 1. 安装依赖

```bash
cd frontend

# LiveKit React Native SDK (不要安装 livekit-client，那是 Web SDK)
npm install @livekit/react-native @livekit/react-native-webrtc

# 其他依赖
npm install zustand @polkadot/api @polkadot/extension-dapp

# Expo Development Build 支持
npx expo install expo-dev-client expo-camera expo-av
```

### 2. 生成原生项目

> ⚠️ **Expo Go 不支持 LiveKit**，必须使用 Development Build

```bash
# 生成 iOS 和 Android 原生项目
npx expo prebuild

# 或者只生成特定平台
npx expo prebuild --platform ios
npx expo prebuild --platform android
```

### 3. iOS 权限配置

在 `ios/[项目名]/Info.plist` 中添加：

```xml
<key>NSCameraUsageDescription</key>
<string>需要访问相机进行直播</string>
<key>NSMicrophoneUsageDescription</key>
<string>需要访问麦克风进行直播</string>
<key>UIBackgroundModes</key>
<array>
  <string>audio</string>
  <string>voip</string>
</array>
```

### 4. Android 权限配置

在 `android/app/src/main/AndroidManifest.xml` 中添加：

```xml
<uses-permission android:name="android.permission.CAMERA" />
<uses-permission android:name="android.permission.RECORD_AUDIO" />
<uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
<uses-permission android:name="android.permission.BLUETOOTH" android:maxSdkVersion="30" />
<uses-permission android:name="android.permission.BLUETOOTH_CONNECT" />

<!-- 后台音频 -->
<uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
<uses-permission android:name="android.permission.WAKE_LOCK" />
```

### 5. 运行 Development Build

```bash
# iOS (需要 Mac + Xcode)
npx expo run:ios

# Android
npx expo run:android

# 或者构建 development client
eas build --profile development --platform ios
eas build --profile development --platform android
```

### 6. app.json 配置

```json
{
  "expo": {
    "plugins": [
      [
        "expo-camera",
        {
          "cameraPermission": "需要访问相机进行直播"
        }
      ],
      [
        "expo-av",
        {
          "microphonePermission": "需要访问麦克风进行直播"
        }
      ]
    ],
    "ios": {
      "infoPlist": {
        "UIBackgroundModes": ["audio", "voip"]
      }
    },
    "android": {
      "permissions": [
        "android.permission.CAMERA",
        "android.permission.RECORD_AUDIO",
        "android.permission.MODIFY_AUDIO_SETTINGS"
      ]
    }
  }
}
```

## 功能清单

| 功能 | 页面 | 链上/链下 | 状态 |
|------|------|----------|------|
| 直播列表 | LiveListScreen | 链上查询 | 待开发 |
| 创建直播间 | CreateRoomScreen | 链上交易 | 待开发 |
| 开始直播 | LiveHostScreen | 链上 + LiveKit | 待开发 |
| 观看直播 | LiveRoomScreen | LiveKit | 待开发 |
| 发送礼物 | GiftPanel | 链上交易 | 待开发 |
| 购买门票 | TicketModal | 链上交易 | 待开发 |
| 聊天/弹幕 | LiveChat/DanmakuOverlay | LiveKit DataChannel | 待开发 |
| 连麦申请 | CoHostPanel | DataChannel | 待开发 |
| 连麦布局 | CoHostLayout | LiveKit | 待开发 |
| 踢出观众 | ViewerList | 链上交易 | 待开发 |
| 提现收益 | MyLiveScreen | 链上交易 | 待开发 |

## 链上/链下交互流程

### 观众进入直播间

```
1. 用户点击直播间卡片
2. 查询链上直播间状态
3. 如果是付费直播，检查是否有门票
   - 无门票 → 显示购票弹窗 → 链上购票
4. 请求后端生成 LiveKit Token
5. 连接 LiveKit 房间
6. 订阅主播视频/音频轨道
7. 监听 DataChannel 接收聊天/弹幕
```

### 主播开播流程

```
1. 创建直播间 (链上交易)
2. 获取 roomId
3. 签名验证身份
4. 请求后端生成主播 Token
5. 连接 LiveKit 房间
6. 开启摄像头/麦克风推流
7. 调用 startLive (链上交易)
8. 监听礼物事件更新收益
```

### 礼物发送流程

```
1. 用户选择礼物和数量
2. 调用 sendGift (链上交易)
3. 交易成功后，通过 DataChannel 广播礼物通知
4. 所有观众收到通知，播放礼物动画
5. 主播收益自动更新
```

### 连麦流程

```
1. 观众点击申请连麦
2. 通过 DataChannel 发送申请
3. 主播收到申请，选择同意/拒绝
4. 同意后，通过 DataChannel 通知观众
5. 观众开启摄像头/麦克风
6. 主播调用 startCoHost (链上记录)
7. 连麦结束时调用 endCoHost
```

## 性能优化

### 1. 弹幕限流与对象池

大量弹幕会导致频繁创建/销毁 Animated.Value，造成卡顿。使用限流和对象池优化：

```typescript
// frontend/src/features/livestream/components/DanmakuOverlay.optimized.tsx

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { View, Animated, StyleSheet, Dimensions } from 'react-native';

const { width: SCREEN_WIDTH } = Dimensions.get('window');

// 性能配置
const MAX_DANMAKU_ON_SCREEN = 50;      // 屏幕上最大弹幕数
const MAX_DANMAKU_PER_SECOND = 10;     // 每秒最大弹幕数
const DANMAKU_POOL_SIZE = 60;          // 对象池大小
const TRACK_COUNT = 12;                // 弹幕轨道数

interface DanmakuItem {
  id: number;
  content: string;
  color: string;
  translateX: Animated.Value;
  opacity: Animated.Value;
  track: number;
  active: boolean;
}

export function DanmakuOverlayOptimized({ roomId }: { roomId: number }) {
  // 对象池 - 预创建 Animated.Value 避免频繁 GC
  const poolRef = useRef<DanmakuItem[]>([]);
  const activeCountRef = useRef(0);
  const lastSecondCountRef = useRef(0);
  const lastSecondTimeRef = useRef(Date.now());
  const trackLastUseRef = useRef<number[]>(new Array(TRACK_COUNT).fill(0));
  const pendingQueueRef = useRef<{ content: string; color: string }[]>([]);
  
  const [, forceUpdate] = useState(0);

  // 初始化对象池
  useEffect(() => {
    poolRef.current = Array.from({ length: DANMAKU_POOL_SIZE }, (_, i) => ({
      id: i,
      content: '',
      color: '#FFFFFF',
      translateX: new Animated.Value(SCREEN_WIDTH),
      opacity: new Animated.Value(0),
      track: 0,
      active: false,
    }));
  }, []);

  // 获取最佳轨道 (避免重叠)
  const getBestTrack = useCallback((): number => {
    const now = Date.now();
    let bestTrack = 0;
    let oldestTime = trackLastUseRef.current[0];

    for (let i = 1; i < TRACK_COUNT; i++) {
      if (trackLastUseRef.current[i] < oldestTime) {
        oldestTime = trackLastUseRef.current[i];
        bestTrack = i;
      }
    }

    trackLastUseRef.current[bestTrack] = now;
    return bestTrack;
  }, []);

  // 从对象池获取可用项
  const acquireFromPool = useCallback((): DanmakuItem | null => {
    const item = poolRef.current.find(d => !d.active);
    if (item) {
      item.active = true;
      activeCountRef.current++;
    }
    return item || null;
  }, []);

  // 归还到对象池
  const releaseToPool = useCallback((item: DanmakuItem) => {
    item.active = false;
    item.translateX.setValue(SCREEN_WIDTH);
    item.opacity.setValue(0);
    activeCountRef.current--;
    
    // 处理等待队列
    if (pendingQueueRef.current.length > 0) {
      const pending = pendingQueueRef.current.shift()!;
      setTimeout(() => addDanmaku(pending.content, pending.color), 50);
    }
  }, []);

  // 添加弹幕 (带限流)
  const addDanmaku = useCallback((content: string, color: string) => {
    const now = Date.now();

    // 重置每秒计数器
    if (now - lastSecondTimeRef.current > 1000) {
      lastSecondCountRef.current = 0;
      lastSecondTimeRef.current = now;
    }

    // 限流检查
    if (lastSecondCountRef.current >= MAX_DANMAKU_PER_SECOND) {
      // 加入等待队列 (最多缓存 20 条)
      if (pendingQueueRef.current.length < 20) {
        pendingQueueRef.current.push({ content, color });
      }
      return;
    }

    // 屏幕弹幕数量检查
    if (activeCountRef.current >= MAX_DANMAKU_ON_SCREEN) {
      return;
    }

    const item = acquireFromPool();
    if (!item) return;

    lastSecondCountRef.current++;

    // 配置弹幕
    item.content = content;
    item.color = color;
    item.track = getBestTrack();
    item.translateX.setValue(SCREEN_WIDTH);
    item.opacity.setValue(1);

    forceUpdate(n => n + 1);

    // 动画
    Animated.timing(item.translateX, {
      toValue: -content.length * 20, // 根据内容长度调整终点
      duration: 8000 + Math.random() * 2000, // 随机速度
      useNativeDriver: true,
    }).start(() => {
      releaseToPool(item);
      forceUpdate(n => n + 1);
    });
  }, [acquireFromPool, releaseToPool, getBestTrack]);

  // 暴露方法给父组件
  useEffect(() => {
    // 可以通过 ref 或 context 暴露 addDanmaku 方法
  }, [addDanmaku]);

  const activeDanmakus = poolRef.current.filter(d => d.active);

  return (
    <View style={StyleSheet.absoluteFill} pointerEvents="none">
      {activeDanmakus.map(d => (
        <Animated.Text
          key={d.id}
          style={[
            styles.danmaku,
            {
              color: d.color,
              top: d.track * 32 + 20,
              opacity: d.opacity,
              transform: [{ translateX: d.translateX }],
            },
          ]}
        >
          {d.content}
        </Animated.Text>
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  danmaku: {
    position: 'absolute',
    fontSize: 16,
    fontWeight: 'bold',
    textShadowColor: '#000',
    textShadowOffset: { width: 1, height: 1 },
    textShadowRadius: 2,
  },
});
```

### 2. 礼物动画队列

避免多个全屏动画同时播放导致卡顿：

```typescript
// frontend/src/features/livestream/hooks/useGiftAnimationQueue.ts

import { useState, useCallback, useRef, useEffect } from 'react';

interface GiftAnimationItem {
  id: string;
  giftId: number;
  giftName: string;
  senderName: string;
  quantity: number;
  isFullScreen: boolean; // 大礼物全屏动画
}

interface UseGiftAnimationQueueOptions {
  maxConcurrent?: number;        // 同时播放的最大动画数
  fullScreenDuration?: number;   // 全屏动画持续时间
  normalDuration?: number;       // 普通动画持续时间
}

export function useGiftAnimationQueue(options: UseGiftAnimationQueueOptions = {}) {
  const {
    maxConcurrent = 3,
    fullScreenDuration = 3000,
    normalDuration = 1500,
  } = options;

  const [currentAnimations, setCurrentAnimations] = useState<GiftAnimationItem[]>([]);
  const queueRef = useRef<GiftAnimationItem[]>([]);
  const playingCountRef = useRef(0);

  // 播放下一个动画
  const playNext = useCallback(() => {
    if (queueRef.current.length === 0) return;
    if (playingCountRef.current >= maxConcurrent) return;

    // 全屏动画独占
    const hasFullScreen = currentAnimations.some(a => a.isFullScreen);
    if (hasFullScreen) return;

    const next = queueRef.current.shift()!;
    
    // 如果是全屏动画，等待其他动画结束
    if (next.isFullScreen && playingCountRef.current > 0) {
      queueRef.current.unshift(next);
      return;
    }

    playingCountRef.current++;
    setCurrentAnimations(prev => [...prev, next]);

    // 动画结束后移除
    const duration = next.isFullScreen ? fullScreenDuration : normalDuration;
    setTimeout(() => {
      playingCountRef.current--;
      setCurrentAnimations(prev => prev.filter(a => a.id !== next.id));
      playNext();
    }, duration);
  }, [currentAnimations, maxConcurrent, fullScreenDuration, normalDuration]);

  // 添加礼物到队列
  const enqueueGift = useCallback((gift: Omit<GiftAnimationItem, 'id'>) => {
    const item: GiftAnimationItem = {
      ...gift,
      id: `${Date.now()}-${Math.random()}`,
    };

    // 大礼物优先级更高
    if (item.isFullScreen) {
      queueRef.current.unshift(item);
    } else {
      queueRef.current.push(item);
    }

    playNext();
  }, [playNext]);

  // 合并相同礼物 (连击效果)
  const enqueueGiftWithCombo = useCallback((gift: Omit<GiftAnimationItem, 'id'>) => {
    // 查找队列中相同发送者的相同礼物
    const existingIndex = queueRef.current.findIndex(
      g => g.giftId === gift.giftId && g.senderName === gift.senderName
    );

    if (existingIndex !== -1) {
      // 合并数量
      queueRef.current[existingIndex].quantity += gift.quantity;
    } else {
      enqueueGift(gift);
    }
  }, [enqueueGift]);

  return {
    currentAnimations,
    enqueueGift,
    enqueueGiftWithCombo,
    queueLength: queueRef.current.length,
  };
}
```

### 3. 观众列表虚拟滚动

```typescript
// frontend/src/features/livestream/components/ViewerList.optimized.tsx

import React, { useCallback, memo } from 'react';
import { View, Text, FlatList, StyleSheet, Image } from 'react-native';

interface Viewer {
  address: string;
  name: string;
  avatar?: string;
  level: number;
  joinedAt: number;
}

interface ViewerListProps {
  viewers: Viewer[];
  onLoadMore: () => void;
  hasMore: boolean;
  isLoading: boolean;
}

// 使用 memo 避免不必要的重渲染
const ViewerItem = memo(({ viewer }: { viewer: Viewer }) => (
  <View style={styles.viewerItem}>
    <Image
      source={{ uri: viewer.avatar || 'https://via.placeholder.com/40' }}
      style={styles.avatar}
    />
    <View style={styles.viewerInfo}>
      <Text style={styles.viewerName} numberOfLines={1}>
        {viewer.name}
      </Text>
      <Text style={styles.viewerLevel}>Lv.{viewer.level}</Text>
    </View>
  </View>
));

export function ViewerListOptimized({
  viewers,
  onLoadMore,
  hasMore,
  isLoading,
}: ViewerListProps) {
  // 稳定的 keyExtractor
  const keyExtractor = useCallback((item: Viewer) => item.address, []);

  // 稳定的 renderItem
  const renderItem = useCallback(
    ({ item }: { item: Viewer }) => <ViewerItem viewer={item} />,
    []
  );

  // 获取 item 布局 (优化滚动性能)
  const getItemLayout = useCallback(
    (_: any, index: number) => ({
      length: 60, // item 高度
      offset: 60 * index,
      index,
    }),
    []
  );

  return (
    <FlatList
      data={viewers}
      keyExtractor={keyExtractor}
      renderItem={renderItem}
      getItemLayout={getItemLayout}
      // 虚拟滚动优化
      windowSize={10}              // 渲染窗口大小
      maxToRenderPerBatch={20}     // 每批渲染数量
      updateCellsBatchingPeriod={50} // 批量更新间隔
      removeClippedSubviews={true} // 移除屏幕外的视图
      initialNumToRender={15}      // 初始渲染数量
      // 加载更多
      onEndReached={hasMore ? onLoadMore : undefined}
      onEndReachedThreshold={0.5}
      // 加载指示器
      ListFooterComponent={
        isLoading ? (
          <View style={styles.loading}>
            <Text style={styles.loadingText}>加载中...</Text>
          </View>
        ) : null
      }
      // 空状态
      ListEmptyComponent={
        <View style={styles.empty}>
          <Text style={styles.emptyText}>暂无观众</Text>
        </View>
      }
    />
  );
}

const styles = StyleSheet.create({
  viewerItem: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 10,
    height: 60,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#333',
  },
  avatar: {
    width: 40,
    height: 40,
    borderRadius: 20,
  },
  viewerInfo: {
    marginLeft: 12,
    flex: 1,
  },
  viewerName: {
    color: '#FFF',
    fontSize: 14,
    fontWeight: '500',
  },
  viewerLevel: {
    color: '#FFD700',
    fontSize: 12,
    marginTop: 2,
  },
  loading: {
    padding: 16,
    alignItems: 'center',
  },
  loadingText: {
    color: '#999',
  },
  empty: {
    padding: 32,
    alignItems: 'center',
  },
  emptyText: {
    color: '#666',
  },
});
```

### 4. 链上数据缓存

```typescript
// frontend/src/features/livestream/hooks/useChainDataCache.ts

import { useRef, useCallback } from 'react';

interface CacheEntry<T> {
  data: T;
  timestamp: number;
  ttl: number;
}

interface CacheOptions {
  defaultTTL?: number;  // 默认缓存时间 (毫秒)
}

/**
 * 链上数据缓存 Hook
 * 减少重复的链上查询
 */
export function useChainDataCache<T>(options: CacheOptions = {}) {
  const { defaultTTL = 10000 } = options; // 默认 10 秒
  const cacheRef = useRef<Map<string, CacheEntry<T>>>(new Map());

  // 获取缓存
  const get = useCallback((key: string): T | null => {
    const entry = cacheRef.current.get(key);
    if (!entry) return null;

    // 检查是否过期
    if (Date.now() - entry.timestamp > entry.ttl) {
      cacheRef.current.delete(key);
      return null;
    }

    return entry.data;
  }, []);

  // 设置缓存
  const set = useCallback((key: string, data: T, ttl?: number) => {
    cacheRef.current.set(key, {
      data,
      timestamp: Date.now(),
      ttl: ttl ?? defaultTTL,
    });
  }, [defaultTTL]);

  // 带缓存的查询
  const cachedQuery = useCallback(
    async (
      key: string,
      queryFn: () => Promise<T>,
      ttl?: number
    ): Promise<T> => {
      // 先检查缓存
      const cached = get(key);
      if (cached !== null) {
        return cached;
      }

      // 执行查询
      const data = await queryFn();
      set(key, data, ttl);
      return data;
    },
    [get, set]
  );

  // 清除缓存
  const clear = useCallback((key?: string) => {
    if (key) {
      cacheRef.current.delete(key);
    } else {
      cacheRef.current.clear();
    }
  }, []);

  // 预热缓存
  const prefetch = useCallback(
    async (queries: { key: string; queryFn: () => Promise<T>; ttl?: number }[]) => {
      await Promise.all(
        queries.map(({ key, queryFn, ttl }) => cachedQuery(key, queryFn, ttl))
      );
    },
    [cachedQuery]
  );

  return {
    get,
    set,
    cachedQuery,
    clear,
    prefetch,
  };
}

// 使用示例
/*
const { cachedQuery } = useChainDataCache<LiveRoom>();

// 查询直播间信息 (10秒缓存)
const room = await cachedQuery(
  `room:${roomId}`,
  () => chainService.getRoom(roomId),
  10000
);

// 查询礼物列表 (5分钟缓存，礼物列表变化少)
const gifts = await cachedQuery(
  'gifts:all',
  () => chainService.getGifts(),
  5 * 60 * 1000
);
*/
```

### 5. 性能监控

```typescript
// frontend/src/features/livestream/utils/performanceMonitor.ts

/**
 * 直播性能监控
 */
class LivestreamPerformanceMonitor {
  private metrics = {
    fps: 0,
    danmakuCount: 0,
    animationCount: 0,
    memoryUsage: 0,
    chainQueryCount: 0,
    chainQueryTime: 0,
  };

  private frameCount = 0;
  private lastFpsTime = Date.now();

  // FPS 监控
  measureFps() {
    this.frameCount++;
    const now = Date.now();
    if (now - this.lastFpsTime >= 1000) {
      this.metrics.fps = this.frameCount;
      this.frameCount = 0;
      this.lastFpsTime = now;

      // FPS 过低警告
      if (this.metrics.fps < 30) {
        console.warn('[Performance] Low FPS:', this.metrics.fps);
      }
    }
  }

  // 记录链上查询
  recordChainQuery(duration: number) {
    this.metrics.chainQueryCount++;
    this.metrics.chainQueryTime += duration;
  }

  // 获取指标
  getMetrics() {
    return { ...this.metrics };
  }

  // 重置
  reset() {
    this.metrics = {
      fps: 0,
      danmakuCount: 0,
      animationCount: 0,
      memoryUsage: 0,
      chainQueryCount: 0,
      chainQueryTime: 0,
    };
  }
}

export const performanceMonitor = new LivestreamPerformanceMonitor();
```

### 性能优化总结

| 优化项 | 方案 | 效果 |
|--------|------|------|
| 弹幕渲染 | 对象池 + 限流 (10条/秒) | 避免频繁 GC，防止卡顿 |
| 礼物动画 | 队列管理 + 全屏独占 | 避免动画堆积 |
| 观众列表 | 虚拟滚动 + memo | 支持大量观众 |
| 链上查询 | 缓存 + TTL | 减少 RPC 请求 |
| 整体监控 | FPS 监控 + 指标收集 | 及时发现性能问题 |

## 错误处理

### 1. LiveKit 连接状态管理

```typescript
// frontend/src/features/livestream/hooks/useLiveKitConnection.ts

import { useState, useEffect, useCallback, useRef } from 'react';
import { Room, RoomEvent, ConnectionState } from '@livekit/react-native';
import { Alert } from 'react-native';
import Toast from 'react-native-toast-message';

interface ConnectionStatus {
  state: 'disconnected' | 'connecting' | 'connected' | 'reconnecting';
  error?: string;
  retryCount: number;
}

interface UseLiveKitConnectionOptions {
  maxRetries?: number;
  retryDelay?: number;
  onFatalError?: () => void;
}

export function useLiveKitConnection(
  room: Room | null,
  options: UseLiveKitConnectionOptions = {}
) {
  const { maxRetries = 3, retryDelay = 2000, onFatalError } = options;
  
  const [status, setStatus] = useState<ConnectionStatus>({
    state: 'disconnected',
    retryCount: 0,
  });
  
  const retryCountRef = useRef(0);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout>();

  useEffect(() => {
    if (!room) return;

    // 连接成功
    const handleConnected = () => {
      retryCountRef.current = 0;
      setStatus({ state: 'connected', retryCount: 0 });
      Toast.show({
        type: 'success',
        text1: '连接成功',
        visibilityTime: 2000,
      });
    };

    // 断开连接
    const handleDisconnected = (reason?: string) => {
      setStatus(prev => ({
        state: 'disconnected',
        error: reason,
        retryCount: prev.retryCount,
      }));
      
      // 非主动断开时提示
      if (reason && reason !== 'client_initiated') {
        Toast.show({
          type: 'error',
          text1: '连接已断开',
          text2: getDisconnectReason(reason),
        });
      }
    };

    // 正在重连
    const handleReconnecting = () => {
      retryCountRef.current++;
      setStatus(prev => ({
        state: 'reconnecting',
        retryCount: retryCountRef.current,
      }));
      
      Toast.show({
        type: 'info',
        text1: '正在重新连接...',
        text2: `第 ${retryCountRef.current} 次尝试`,
        autoHide: false,
      });
    };

    // 重连成功
    const handleReconnected = () => {
      retryCountRef.current = 0;
      setStatus({ state: 'connected', retryCount: 0 });
      
      Toast.hide();
      Toast.show({
        type: 'success',
        text1: '重新连接成功',
        visibilityTime: 2000,
      });
    };

    // 连接质量变化
    const handleConnectionQualityChanged = (quality: string) => {
      if (quality === 'poor') {
        Toast.show({
          type: 'warning',
          text1: '网络质量较差',
          text2: '可能会影响直播体验',
        });
      }
    };

    room.on(RoomEvent.Connected, handleConnected);
    room.on(RoomEvent.Disconnected, handleDisconnected);
    room.on(RoomEvent.Reconnecting, handleReconnecting);
    room.on(RoomEvent.Reconnected, handleReconnected);
    room.on(RoomEvent.ConnectionQualityChanged, handleConnectionQualityChanged);

    return () => {
      room.off(RoomEvent.Connected, handleConnected);
      room.off(RoomEvent.Disconnected, handleDisconnected);
      room.off(RoomEvent.Reconnecting, handleReconnecting);
      room.off(RoomEvent.Reconnected, handleReconnected);
      room.off(RoomEvent.ConnectionQualityChanged, handleConnectionQualityChanged);
      
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
    };
  }, [room, maxRetries, onFatalError]);

  // 手动重连
  const reconnect = useCallback(async () => {
    if (!room) return;
    
    if (retryCountRef.current >= maxRetries) {
      Alert.alert(
        '连接失败',
        '多次重连失败，请检查网络后重试',
        [
          { text: '退出直播间', onPress: onFatalError },
          { text: '继续重试', onPress: () => {
            retryCountRef.current = 0;
            reconnect();
          }},
        ]
      );
      return;
    }

    setStatus(prev => ({ ...prev, state: 'reconnecting' }));
    
    try {
      // LiveKit 会自动处理重连，这里只是触发
      // 如果需要手动重连，可以断开后重新连接
    } catch (error) {
      console.error('Reconnect failed:', error);
    }
  }, [room, maxRetries, onFatalError]);

  return {
    status,
    reconnect,
    isConnected: status.state === 'connected',
    isReconnecting: status.state === 'reconnecting',
  };
}

// 断开原因映射
function getDisconnectReason(reason: string): string {
  const reasons: Record<string, string> = {
    'duplicate_identity': '账号在其他设备登录',
    'participant_removed': '您已被移出直播间',
    'room_deleted': '直播间已关闭',
    'state_mismatch': '连接状态异常',
    'join_failure': '加入直播间失败',
    'signal_close': '信号连接断开',
  };
  return reasons[reason] || '网络连接异常';
}
```

### 2. 链上交易错误处理

```typescript
// frontend/src/features/livestream/utils/chainErrorHandler.ts

import { Alert } from 'react-native';
import Toast from 'react-native-toast-message';

/**
 * 链上错误类型
 */
export enum ChainErrorType {
  InsufficientBalance = 'InsufficientBalance',
  RoomNotFound = 'RoomNotFound',
  NotRoomHost = 'NotRoomHost',
  RoomNotLive = 'RoomNotLive',
  AlreadyLive = 'AlreadyLive',
  TicketRequired = 'TicketRequired',
  AlreadyHasTicket = 'AlreadyHasTicket',
  ViewerBanned = 'ViewerBanned',
  GiftNotFound = 'GiftNotFound',
  GiftDisabled = 'GiftDisabled',
  InvalidQuantity = 'InvalidQuantity',
  CoHostLimitReached = 'CoHostLimitReached',
  NotCoHost = 'NotCoHost',
  NetworkError = 'NetworkError',
  SignatureError = 'SignatureError',
  Unknown = 'Unknown',
}

/**
 * 错误信息映射
 */
const ERROR_MESSAGES: Record<ChainErrorType, { title: string; message: string; action?: string }> = {
  [ChainErrorType.InsufficientBalance]: {
    title: '余额不足',
    message: '您的 DUST 余额不足，请先充值',
    action: 'recharge',
  },
  [ChainErrorType.RoomNotFound]: {
    title: '直播间不存在',
    message: '该直播间已关闭或不存在',
  },
  [ChainErrorType.NotRoomHost]: {
    title: '权限不足',
    message: '您不是该直播间的主播',
  },
  [ChainErrorType.RoomNotLive]: {
    title: '直播未开始',
    message: '主播还未开始直播',
  },
  [ChainErrorType.AlreadyLive]: {
    title: '操作失败',
    message: '直播已经开始',
  },
  [ChainErrorType.TicketRequired]: {
    title: '需要门票',
    message: '这是付费直播，请先购买门票',
    action: 'buyTicket',
  },
  [ChainErrorType.AlreadyHasTicket]: {
    title: '已有门票',
    message: '您已经购买过门票了',
  },
  [ChainErrorType.ViewerBanned]: {
    title: '无法进入',
    message: '您已被主播禁止进入该直播间',
  },
  [ChainErrorType.GiftNotFound]: {
    title: '礼物不存在',
    message: '该礼物已下架',
  },
  [ChainErrorType.GiftDisabled]: {
    title: '礼物不可用',
    message: '该礼物暂时不可用',
  },
  [ChainErrorType.InvalidQuantity]: {
    title: '数量无效',
    message: '请输入有效的数量',
  },
  [ChainErrorType.CoHostLimitReached]: {
    title: '连麦人数已满',
    message: '当前直播间连麦人数已达上限',
  },
  [ChainErrorType.NotCoHost]: {
    title: '操作失败',
    message: '您不是当前连麦者',
  },
  [ChainErrorType.NetworkError]: {
    title: '网络错误',
    message: '网络连接失败，请检查网络后重试',
  },
  [ChainErrorType.SignatureError]: {
    title: '签名失败',
    message: '钱包签名失败，请重试',
  },
  [ChainErrorType.Unknown]: {
    title: '操作失败',
    message: '发生未知错误，请稍后重试',
  },
};

/**
 * 解析链上错误
 */
export function parseChainError(error: any): ChainErrorType {
  const errorString = error?.message || error?.toString() || '';
  
  // Substrate 模块错误格式: "module.ErrorName"
  if (errorString.includes('InsufficientBalance') || errorString.includes('Arithmetic')) {
    return ChainErrorType.InsufficientBalance;
  }
  if (errorString.includes('RoomNotFound')) {
    return ChainErrorType.RoomNotFound;
  }
  if (errorString.includes('NotRoomHost') || errorString.includes('NotHost')) {
    return ChainErrorType.NotRoomHost;
  }
  if (errorString.includes('RoomNotLive') || errorString.includes('NotLive')) {
    return ChainErrorType.RoomNotLive;
  }
  if (errorString.includes('AlreadyLive')) {
    return ChainErrorType.AlreadyLive;
  }
  if (errorString.includes('TicketRequired') || errorString.includes('NoTicket')) {
    return ChainErrorType.TicketRequired;
  }
  if (errorString.includes('AlreadyHasTicket')) {
    return ChainErrorType.AlreadyHasTicket;
  }
  if (errorString.includes('Banned') || errorString.includes('Blacklisted')) {
    return ChainErrorType.ViewerBanned;
  }
  if (errorString.includes('GiftNotFound')) {
    return ChainErrorType.GiftNotFound;
  }
  if (errorString.includes('GiftDisabled')) {
    return ChainErrorType.GiftDisabled;
  }
  if (errorString.includes('InvalidQuantity') || errorString.includes('ZeroQuantity')) {
    return ChainErrorType.InvalidQuantity;
  }
  if (errorString.includes('CoHostLimit')) {
    return ChainErrorType.CoHostLimitReached;
  }
  if (errorString.includes('NotCoHost')) {
    return ChainErrorType.NotCoHost;
  }
  if (errorString.includes('network') || errorString.includes('timeout') || errorString.includes('ECONNREFUSED')) {
    return ChainErrorType.NetworkError;
  }
  if (errorString.includes('signature') || errorString.includes('sign')) {
    return ChainErrorType.SignatureError;
  }
  
  return ChainErrorType.Unknown;
}

/**
 * 处理链上错误 (显示提示)
 */
export function handleChainError(
  error: any,
  options?: {
    onRecharge?: () => void;
    onBuyTicket?: () => void;
    onRetry?: () => void;
  }
): ChainErrorType {
  const errorType = parseChainError(error);
  const errorInfo = ERROR_MESSAGES[errorType];
  
  console.error('[ChainError]', errorType, error);
  
  // 根据错误类型显示不同的提示
  if (errorInfo.action === 'recharge' && options?.onRecharge) {
    Alert.alert(errorInfo.title, errorInfo.message, [
      { text: '取消', style: 'cancel' },
      { text: '去充值', onPress: options.onRecharge },
    ]);
  } else if (errorInfo.action === 'buyTicket' && options?.onBuyTicket) {
    Alert.alert(errorInfo.title, errorInfo.message, [
      { text: '取消', style: 'cancel' },
      { text: '购买门票', onPress: options.onBuyTicket },
    ]);
  } else if (errorType === ChainErrorType.NetworkError && options?.onRetry) {
    Alert.alert(errorInfo.title, errorInfo.message, [
      { text: '取消', style: 'cancel' },
      { text: '重试', onPress: options.onRetry },
    ]);
  } else {
    Toast.show({
      type: 'error',
      text1: errorInfo.title,
      text2: errorInfo.message,
    });
  }
  
  return errorType;
}

/**
 * 包装链上交易的错误处理
 */
export async function withChainErrorHandling<T>(
  operation: () => Promise<T>,
  options?: {
    onRecharge?: () => void;
    onBuyTicket?: () => void;
    onRetry?: () => void;
    loadingMessage?: string;
  }
): Promise<T | null> {
  try {
    if (options?.loadingMessage) {
      Toast.show({
        type: 'info',
        text1: options.loadingMessage,
        autoHide: false,
      });
    }
    
    const result = await operation();
    
    Toast.hide();
    return result;
  } catch (error) {
    Toast.hide();
    handleChainError(error, options);
    return null;
  }
}
```

### 3. 礼物发送错误处理示例

```typescript
// 在 GiftPanel 中使用错误处理

import { withChainErrorHandling, ChainErrorType } from '../utils/chainErrorHandler';
import { useRouter } from 'expo-router';

function GiftPanel({ roomId, onClose }: GiftPanelProps) {
  const router = useRouter();
  const { sendGift } = useLivestreamStore();
  const [selectedGift, setSelectedGift] = useState<Gift | null>(null);
  const [quantity, setQuantity] = useState(1);
  const [isSending, setIsSending] = useState(false);

  const handleSend = async () => {
    if (!selectedGift) return;

    setIsSending(true);
    
    const result = await withChainErrorHandling(
      () => sendGift(roomId, selectedGift.id, quantity),
      {
        loadingMessage: '正在发送礼物...',
        onRecharge: () => {
          onClose();
          router.push('/wallet/buy-dust');
        },
        onRetry: () => handleSend(),
      }
    );

    setIsSending(false);

    if (result !== null) {
      Toast.show({
        type: 'success',
        text1: '礼物发送成功',
        text2: `送出 ${selectedGift.name} x${quantity}`,
      });
      onClose();
    }
  };

  // ...
}
```

### 4. 进入直播间错误处理

```typescript
// frontend/src/features/livestream/hooks/useJoinRoom.ts

import { useState, useCallback } from 'react';
import { Alert } from 'react-native';
import { useRouter } from 'expo-router';
import { useLivestreamStore } from '@/stores/livestream.store';
import { handleChainError, ChainErrorType, parseChainError } from '../utils/chainErrorHandler';

interface UseJoinRoomOptions {
  onTicketRequired?: (roomId: number, price: string) => void;
}

export function useJoinRoom(options: UseJoinRoomOptions = {}) {
  const router = useRouter();
  const { joinRoom, buyTicket } = useLivestreamStore();
  const [isJoining, setIsJoining] = useState(false);
  const [error, setError] = useState<ChainErrorType | null>(null);

  const join = useCallback(async (roomId: number) => {
    setIsJoining(true);
    setError(null);

    try {
      await joinRoom(roomId);
      router.push(`/livestream/${roomId}`);
    } catch (err: any) {
      const errorType = parseChainError(err);
      setError(errorType);

      if (errorType === ChainErrorType.TicketRequired) {
        // 需要购票 - 显示购票弹窗
        if (options.onTicketRequired) {
          // 获取票价
          const room = await useLivestreamStore.getState().currentRoom;
          options.onTicketRequired(roomId, room?.ticketPrice || '0');
        } else {
          Alert.alert(
            '需要门票',
            '这是付费直播，是否购买门票？',
            [
              { text: '取消', style: 'cancel' },
              { 
                text: '购买', 
                onPress: async () => {
                  try {
                    await buyTicket(roomId);
                    // 购票成功后重新加入
                    await join(roomId);
                  } catch (buyError) {
                    handleChainError(buyError, {
                      onRecharge: () => router.push('/wallet/buy-dust'),
                    });
                  }
                }
              },
            ]
          );
        }
      } else if (errorType === ChainErrorType.ViewerBanned) {
        Alert.alert('无法进入', '您已被主播禁止进入该直播间');
      } else if (errorType === ChainErrorType.RoomNotLive) {
        Alert.alert('直播未开始', '主播还未开始直播，请稍后再试');
      } else {
        handleChainError(err);
      }
    } finally {
      setIsJoining(false);
    }
  }, [joinRoom, buyTicket, router, options]);

  return {
    join,
    isJoining,
    error,
  };
}
```

### 5. 全局错误边界

```typescript
// frontend/src/features/livestream/components/LivestreamErrorBoundary.tsx

import React, { Component, ReactNode } from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';

interface Props {
  children: ReactNode;
  onReset?: () => void;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class LivestreamErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('[LivestreamErrorBoundary]', error, errorInfo);
    // 可以上报错误到监控服务
  }

  handleReset = () => {
    this.setState({ hasError: false, error: undefined });
    this.props.onReset?.();
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <View style={styles.container}>
          <Text style={styles.emoji}>😵</Text>
          <Text style={styles.title}>直播出现问题</Text>
          <Text style={styles.message}>
            {this.state.error?.message || '发生了未知错误'}
          </Text>
          <TouchableOpacity style={styles.button} onPress={this.handleReset}>
            <Text style={styles.buttonText}>重新加载</Text>
          </TouchableOpacity>
        </View>
      );
    }

    return this.props.children;
  }
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#1A1A2E',
    padding: 20,
  },
  emoji: {
    fontSize: 64,
    marginBottom: 16,
  },
  title: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#FFF',
    marginBottom: 8,
  },
  message: {
    fontSize: 14,
    color: '#999',
    textAlign: 'center',
    marginBottom: 24,
  },
  button: {
    backgroundColor: '#FF4757',
    paddingHorizontal: 32,
    paddingVertical: 12,
    borderRadius: 24,
  },
  buttonText: {
    color: '#FFF',
    fontSize: 16,
    fontWeight: '600',
  },
});
```

### 错误处理总结

| 场景 | 处理方式 |
|------|----------|
| LiveKit 断线 | 自动重连 + Toast 提示 + 重连次数限制 |
| 网络质量差 | 警告提示，不中断直播 |
| 余额不足 | Alert 弹窗，引导去充值 |
| 需要门票 | Alert 弹窗，引导购买 |
| 被封禁 | Alert 提示，无法进入 |
| 交易失败 | Toast 提示具体原因 |
| 组件崩溃 | ErrorBoundary 捕获，显示重试按钮 |

## 权限管理

### 1. 权限请求 Hook

```typescript
// frontend/src/features/livestream/hooks/useMediaPermissions.ts

import { useState, useCallback, useEffect } from 'react';
import { Alert, Linking, Platform } from 'react-native';
import { Camera } from 'expo-camera';
import { Audio } from 'expo-av';
import * as IntentLauncher from 'expo-intent-launcher';

export type PermissionStatus = 'undetermined' | 'granted' | 'denied';

interface MediaPermissions {
  camera: PermissionStatus;
  microphone: PermissionStatus;
}

interface UseMediaPermissionsResult {
  permissions: MediaPermissions;
  isLoading: boolean;
  hasAllPermissions: boolean;
  checkPermissions: () => Promise<MediaPermissions>;
  requestPermissions: () => Promise<boolean>;
  openSettings: () => Promise<void>;
}

export function useMediaPermissions(): UseMediaPermissionsResult {
  const [permissions, setPermissions] = useState<MediaPermissions>({
    camera: 'undetermined',
    microphone: 'undetermined',
  });
  const [isLoading, setIsLoading] = useState(true);

  // 检查权限状态
  const checkPermissions = useCallback(async (): Promise<MediaPermissions> => {
    try {
      const [cameraStatus, audioStatus] = await Promise.all([
        Camera.getCameraPermissionsAsync(),
        Audio.getPermissionsAsync(),
      ]);

      const result: MediaPermissions = {
        camera: cameraStatus.granted 
          ? 'granted' 
          : cameraStatus.canAskAgain 
            ? 'undetermined' 
            : 'denied',
        microphone: audioStatus.granted 
          ? 'granted' 
          : audioStatus.canAskAgain 
            ? 'undetermined' 
            : 'denied',
      };

      setPermissions(result);
      return result;
    } catch (error) {
      console.error('Check permissions error:', error);
      return { camera: 'undetermined', microphone: 'undetermined' };
    }
  }, []);

  // 请求权限
  const requestPermissions = useCallback(async (): Promise<boolean> => {
    setIsLoading(true);

    try {
      // 先检查当前状态
      const current = await checkPermissions();

      // 如果已经全部授权
      if (current.camera === 'granted' && current.microphone === 'granted') {
        return true;
      }

      // 如果有权限被永久拒绝
      if (current.camera === 'denied' || current.microphone === 'denied') {
        showPermissionDeniedAlert(current);
        return false;
      }

      // 请求相机权限
      let cameraGranted = current.camera === 'granted';
      if (!cameraGranted) {
        const cameraResult = await Camera.requestCameraPermissionsAsync();
        cameraGranted = cameraResult.granted;
        
        if (!cameraGranted && !cameraResult.canAskAgain) {
          setPermissions(prev => ({ ...prev, camera: 'denied' }));
        }
      }

      // 请求麦克风权限
      let microphoneGranted = current.microphone === 'granted';
      if (!microphoneGranted) {
        const audioResult = await Audio.requestPermissionsAsync();
        microphoneGranted = audioResult.granted;
        
        if (!microphoneGranted && !audioResult.canAskAgain) {
          setPermissions(prev => ({ ...prev, microphone: 'denied' }));
        }
      }

      // 更新状态
      await checkPermissions();

      // 检查结果
      if (!cameraGranted || !microphoneGranted) {
        const deniedPermissions: string[] = [];
        if (!cameraGranted) deniedPermissions.push('相机');
        if (!microphoneGranted) deniedPermissions.push('麦克风');

        Alert.alert(
          '权限不足',
          `需要${deniedPermissions.join('和')}权限才能开播`,
          [{ text: '知道了' }]
        );
        return false;
      }

      return true;
    } catch (error) {
      console.error('Request permissions error:', error);
      Alert.alert('错误', '请求权限时发生错误');
      return false;
    } finally {
      setIsLoading(false);
    }
  }, [checkPermissions]);

  // 打开系统设置
  const openSettings = useCallback(async () => {
    try {
      if (Platform.OS === 'ios') {
        await Linking.openSettings();
      } else {
        // Android 打开应用设置页
        await IntentLauncher.startActivityAsync(
          IntentLauncher.ActivityAction.APPLICATION_DETAILS_SETTINGS,
          { data: 'package:' + (await import('expo-application')).applicationId }
        );
      }
    } catch (error) {
      console.error('Open settings error:', error);
      // 降级方案
      await Linking.openSettings();
    }
  }, []);

  // 显示权限被拒绝的提示
  const showPermissionDeniedAlert = useCallback((current: MediaPermissions) => {
    const deniedPermissions: string[] = [];
    if (current.camera === 'denied') deniedPermissions.push('相机');
    if (current.microphone === 'denied') deniedPermissions.push('麦克风');

    Alert.alert(
      '权限被拒绝',
      `${deniedPermissions.join('和')}权限已被拒绝，请在系统设置中开启`,
      [
        { text: '取消', style: 'cancel' },
        { text: '去设置', onPress: openSettings },
      ]
    );
  }, [openSettings]);

  // 初始化时检查权限
  useEffect(() => {
    checkPermissions().finally(() => setIsLoading(false));
  }, [checkPermissions]);

  return {
    permissions,
    isLoading,
    hasAllPermissions: permissions.camera === 'granted' && permissions.microphone === 'granted',
    checkPermissions,
    requestPermissions,
    openSettings,
  };
}
```

### 2. 开播前权限检查组件

```typescript
// frontend/src/features/livestream/components/PermissionGate.tsx

import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity, ActivityIndicator } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useMediaPermissions, PermissionStatus } from '../hooks/useMediaPermissions';

interface PermissionGateProps {
  children: React.ReactNode;
  onPermissionDenied?: () => void;
}

export function PermissionGate({ children, onPermissionDenied }: PermissionGateProps) {
  const { 
    permissions, 
    isLoading, 
    hasAllPermissions, 
    requestPermissions, 
    openSettings 
  } = useMediaPermissions();

  if (isLoading) {
    return (
      <View style={styles.container}>
        <ActivityIndicator size="large" color="#FF4757" />
        <Text style={styles.text}>检查权限中...</Text>
      </View>
    );
  }

  if (hasAllPermissions) {
    return <>{children}</>;
  }

  const hasDenied = permissions.camera === 'denied' || permissions.microphone === 'denied';

  return (
    <View style={styles.container}>
      <Ionicons name="videocam-off" size={64} color="#666" />
      <Text style={styles.title}>需要相机和麦克风权限</Text>
      <Text style={styles.description}>
        开播需要使用相机和麦克风，请授予相关权限
      </Text>

      <View style={styles.permissionList}>
        <PermissionItem 
          icon="camera" 
          label="相机" 
          status={permissions.camera} 
        />
        <PermissionItem 
          icon="mic" 
          label="麦克风" 
          status={permissions.microphone} 
        />
      </View>

      {hasDenied ? (
        <TouchableOpacity style={styles.button} onPress={openSettings}>
          <Ionicons name="settings-outline" size={20} color="#FFF" />
          <Text style={styles.buttonText}>去系统设置开启</Text>
        </TouchableOpacity>
      ) : (
        <TouchableOpacity style={styles.button} onPress={requestPermissions}>
          <Text style={styles.buttonText}>授予权限</Text>
        </TouchableOpacity>
      )}

      <TouchableOpacity 
        style={styles.cancelButton} 
        onPress={onPermissionDenied}
      >
        <Text style={styles.cancelText}>暂不开播</Text>
      </TouchableOpacity>
    </View>
  );
}

function PermissionItem({ 
  icon, 
  label, 
  status 
}: { 
  icon: string; 
  label: string; 
  status: PermissionStatus;
}) {
  const getStatusIcon = () => {
    switch (status) {
      case 'granted':
        return <Ionicons name="checkmark-circle" size={20} color="#4CAF50" />;
      case 'denied':
        return <Ionicons name="close-circle" size={20} color="#F44336" />;
      default:
        return <Ionicons name="help-circle" size={20} color="#FFC107" />;
    }
  };

  const getStatusText = () => {
    switch (status) {
      case 'granted':
        return '已授权';
      case 'denied':
        return '已拒绝';
      default:
        return '未授权';
    }
  };

  return (
    <View style={styles.permissionItem}>
      <Ionicons name={icon as any} size={24} color="#FFF" />
      <Text style={styles.permissionLabel}>{label}</Text>
      <View style={styles.permissionStatus}>
        {getStatusIcon()}
        <Text style={[
          styles.permissionStatusText,
          status === 'granted' && styles.granted,
          status === 'denied' && styles.denied,
        ]}>
          {getStatusText()}
        </Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#1A1A2E',
    padding: 24,
  },
  title: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#FFF',
    marginTop: 16,
    marginBottom: 8,
  },
  description: {
    fontSize: 14,
    color: '#999',
    textAlign: 'center',
    marginBottom: 24,
  },
  text: {
    color: '#999',
    marginTop: 12,
  },
  permissionList: {
    width: '100%',
    marginBottom: 24,
  },
  permissionItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#252540',
    padding: 16,
    borderRadius: 12,
    marginBottom: 12,
  },
  permissionLabel: {
    flex: 1,
    color: '#FFF',
    fontSize: 16,
    marginLeft: 12,
  },
  permissionStatus: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  permissionStatusText: {
    color: '#FFC107',
    fontSize: 14,
    marginLeft: 4,
  },
  granted: {
    color: '#4CAF50',
  },
  denied: {
    color: '#F44336',
  },
  button: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#FF4757',
    paddingHorizontal: 32,
    paddingVertical: 14,
    borderRadius: 24,
    width: '100%',
    gap: 8,
  },
  buttonText: {
    color: '#FFF',
    fontSize: 16,
    fontWeight: '600',
  },
  cancelButton: {
    marginTop: 16,
    padding: 12,
  },
  cancelText: {
    color: '#999',
    fontSize: 14,
  },
});
```

### 3. 在开播页面使用

```typescript
// frontend/src/features/livestream/screens/LiveHostScreen.tsx

import React from 'react';
import { useRouter } from 'expo-router';
import { PermissionGate } from '../components/PermissionGate';
import { LiveHostContent } from '../components/LiveHostContent';

export function LiveHostScreen() {
  const router = useRouter();

  return (
    <PermissionGate onPermissionDenied={() => router.back()}>
      <LiveHostContent />
    </PermissionGate>
  );
}
```

### 4. 后台运行权限 (主播切后台保持推流)

#### iOS 后台模式配置

已在 `Info.plist` 中配置：
```xml
<key>UIBackgroundModes</key>
<array>
  <string>audio</string>
  <string>voip</string>
</array>
```

#### Android 前台服务

```typescript
// frontend/src/features/livestream/services/foregroundService.ts

import * as TaskManager from 'expo-task-manager';
import * as BackgroundFetch from 'expo-background-fetch';
import { Platform } from 'react-native';

const LIVESTREAM_TASK = 'LIVESTREAM_BACKGROUND_TASK';

// 注册后台任务 (仅 Android 需要)
if (Platform.OS === 'android') {
  TaskManager.defineTask(LIVESTREAM_TASK, async () => {
    // 保持直播连接活跃
    console.log('[BackgroundTask] Keeping livestream alive');
    return BackgroundFetch.BackgroundFetchResult.NewData;
  });
}

/**
 * 启动前台服务 (Android)
 * 在开始直播时调用
 */
export async function startForegroundService(): Promise<void> {
  if (Platform.OS !== 'android') return;

  try {
    // 使用 expo-notifications 显示持续通知
    const { default: Notifications } = await import('expo-notifications');
    
    await Notifications.setNotificationChannelAsync('livestream', {
      name: '直播中',
      importance: Notifications.AndroidImportance.LOW,
      sound: null,
      vibrationPattern: null,
    });

    await Notifications.scheduleNotificationAsync({
      content: {
        title: '正在直播',
        body: '点击返回直播间',
        data: { type: 'livestream' },
        sticky: true,
      },
      trigger: null, // 立即显示
    });

    console.log('[ForegroundService] Started');
  } catch (error) {
    console.error('[ForegroundService] Start error:', error);
  }
}

/**
 * 停止前台服务 (Android)
 * 在结束直播时调用
 */
export async function stopForegroundService(): Promise<void> {
  if (Platform.OS !== 'android') return;

  try {
    const { default: Notifications } = await import('expo-notifications');
    await Notifications.dismissAllNotificationsAsync();
    console.log('[ForegroundService] Stopped');
  } catch (error) {
    console.error('[ForegroundService] Stop error:', error);
  }
}
```

### 5. 应用状态监听 (切后台/前台)

```typescript
// frontend/src/features/livestream/hooks/useAppStateHandler.ts

import { useEffect, useRef } from 'react';
import { AppState, AppStateStatus } from 'react-native';
import Toast from 'react-native-toast-message';

interface UseAppStateHandlerOptions {
  onBackground?: () => void;
  onForeground?: () => void;
  isHost?: boolean;
}

export function useAppStateHandler(options: UseAppStateHandlerOptions = {}) {
  const { onBackground, onForeground, isHost = false } = options;
  const appStateRef = useRef(AppState.currentState);

  useEffect(() => {
    const subscription = AppState.addEventListener('change', (nextAppState: AppStateStatus) => {
      const prevState = appStateRef.current;

      // 从前台切到后台
      if (prevState === 'active' && nextAppState.match(/inactive|background/)) {
        console.log('[AppState] App went to background');
        
        if (isHost) {
          Toast.show({
            type: 'info',
            text1: '直播继续中',
            text2: '切回应用可查看直播画面',
            visibilityTime: 3000,
          });
        }
        
        onBackground?.();
      }

      // 从后台切到前台
      if (prevState.match(/inactive|background/) && nextAppState === 'active') {
        console.log('[AppState] App came to foreground');
        onForeground?.();
      }

      appStateRef.current = nextAppState;
    });

    return () => {
      subscription.remove();
    };
  }, [onBackground, onForeground, isHost]);
}

// 使用示例
/*
function LiveHostScreen() {
  const { reconnect } = useLiveKitConnection(room);

  useAppStateHandler({
    isHost: true,
    onBackground: () => {
      // 主播切后台，保持推流
      startForegroundService();
    },
    onForeground: () => {
      // 主播切回前台，检查连接状态
      stopForegroundService();
      // 如果断开了，尝试重连
      if (!room?.state === 'connected') {
        reconnect();
      }
    },
  });
}
*/
```

### 权限管理总结

| 场景 | 处理方式 |
|------|----------|
| 首次请求权限 | 显示权限说明，请求授权 |
| 权限被拒绝 (可再次请求) | 提示用户，再次请求 |
| 权限被永久拒绝 | 引导用户去系统设置开启 |
| 主播切后台 | iOS 使用 background modes，Android 使用前台服务 |
| 主播切回前台 | 检查连接状态，必要时重连 |

## 注意事项

1. **LiveKit 原生依赖**: 
   - 必须使用 `@livekit/react-native` + `@livekit/react-native-webrtc`
   - 不要使用 `livekit-client`，那是 Web SDK
   - Expo Go 不支持，必须运行 `expo prebuild` 生成原生项目
   - 使用 `expo-dev-client` 进行开发调试
2. **权限配置**: iOS 需要 Info.plist，Android 需要 AndroidManifest.xml 配置相机/麦克风权限
3. **音频会话**: 应用启动时需要调用 `registerGlobals()` 和配置 `AudioSession`
4. **签名验证**: 主播推流需要私钥签名，确保安全
5. **礼物动画**: 大礼物需要全屏动画效果
6. **弹幕性能**: 大量弹幕时需要优化渲染
7. **连麦布局**: 支持画中画/分屏/网格多种布局
8. **断线重连**: LiveKit 断线后自动重连
9. **后台处理**: 主播切后台时保持推流，需要配置 `UIBackgroundModes`

## 后续扩展

1. **美颜滤镜**: 集成美颜 SDK
2. **直播回放**: 保存到 IPFS
3. **直播预约**: 预约开播通知
4. **PK 功能**: 主播间 PK 对战
5. **粉丝等级**: 根据打赏累计升级
6. **守护功能**: 月度守护特权
