# 设计文档

## 概述

Cosmos 移动端前端应用采用 React Native 技术栈，实现跨平台（iOS/Android）的 Web3 应用。应用通过 @polkadot/api 与 Substrate 区块链交互，通过 REST API 和 WebSocket 与后端服务通信，提供占卜、社交、直播、交易等核心功能。

## 技术栈选型

### 前端框架：React Native

**选型理由：**
- 跨平台开发，一套代码同时支持 iOS 和 Android
- 丰富的 Web3 生态支持（@polkadot/api、ethers.js 等）
- 成熟的社区和大量第三方库
- 热更新能力，便于快速迭代
- 与 Web 端代码复用度高

**核心依赖：**
```
react-native: ^0.73.x
@polkadot/api: ^10.x
@polkadot/keyring: ^12.x
@polkadot/util-crypto: ^12.x
react-native-keychain: 安全存储
react-native-biometrics: 生物识别
livekit-react-native: 直播 SDK
react-navigation: 导航
zustand: 状态管理
react-query: 数据请求
```

## 架构设计

### 整体架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Cosmos Mobile App                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                              UI Layer (React Native)                        │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │  占卜    │ │  聊天    │ │  直播    │ │  交易    │ │  婚恋    │          │
│  │  模块    │ │  模块    │ │  模块    │ │  模块    │ │  模块    │          │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘          │
│       │            │            │            │            │                 │
├───────┴────────────┴────────────┴────────────┴────────────┴─────────────────┤
│                           Business Logic Layer                              │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                        State Management (Zustand)                     │  │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐        │  │
│  │  │ Wallet  │ │ Chain   │ │ User    │ │ Cache   │ │ UI      │        │  │
│  │  │ Store   │ │ Store   │ │ Store   │ │ Store   │ │ Store   │        │  │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘        │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────────────┤
│                           Service Layer                                     │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐       │
│  │ ChainService │ │ ApiService   │ │ WalletService│ │ StorageService│      │
│  │ (Polkadot)   │ │ (REST/WS)    │ │ (Keychain)   │ │ (AsyncStorage)│      │
│  └──────┬───────┘ └──────┬───────┘ └──────┬───────┘ └──────┬───────┘       │
│         │                │                │                │                │
├─────────┴────────────────┴────────────────┴────────────────┴────────────────┤
│                           External Services                                 │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐       │
│  │  Cosmos    │ │   Backend    │ │   LiveKit    │ │    IPFS      │       │
│  │  Chain Node  │ │   Server     │ │   Server     │ │   Gateway    │       │
│  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘       │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 分层架构说明

| 层级 | 职责 | 技术选型 |
|------|------|----------|
| UI Layer | 界面展示、用户交互 | React Native + React Navigation |
| Business Logic | 状态管理、业务逻辑 | Zustand + React Query |
| Service Layer | 外部服务通信 | @polkadot/api + Axios + WebSocket |
| External Services | 区块链、后端、媒体 | Substrate Node + Express + LiveKit |

## 组件与接口

### 1. ChainService - 链上交互服务

```typescript
interface ChainService {
  // 连接管理
  connect(endpoint: string): Promise<ApiPromise>;
  disconnect(): Promise<void>;
  isConnected(): boolean;
  
  // 账户查询
  getBalance(address: string): Promise<Balance>;
  getAccountInfo(address: string): Promise<AccountInfo>;
  
  // 交易提交
  submitExtrinsic(
    pallet: string,
    method: string,
    params: any[],
    signer: KeyringPair
  ): Promise<TxResult>;
  
  // 批量交易
  batchExtrinsics(
    calls: ExtrinsicCall[],
    signer: KeyringPair
  ): Promise<TxResult>;
  
  // Runtime API 查询
  queryRuntimeApi<T>(
    api: string,
    method: string,
    params: any[]
  ): Promise<T>;
  
  // 事件监听
  subscribeEvents(
    callback: (events: EventRecord[]) => void
  ): Promise<UnsubscribeFn>;
}
```

### 2. WalletService - 钱包管理服务

```typescript
interface WalletService {
  // 钱包创建
  createWallet(): Promise<{ mnemonic: string; address: string }>;
  
  // 钱包导入
  importFromMnemonic(mnemonic: string): Promise<KeyringPair>;
  importFromPrivateKey(privateKey: string): Promise<KeyringPair>;
  importFromJson(json: string, password: string): Promise<KeyringPair>;
  
  // 安全存储
  saveToSecureStorage(pair: KeyringPair, pin: string): Promise<void>;
  loadFromSecureStorage(pin: string): Promise<KeyringPair>;
  
  // 签名
  sign(message: Uint8Array, pair: KeyringPair): Uint8Array;
  signTransaction(tx: SubmittableExtrinsic, pair: KeyringPair): Promise<void>;
  
  // 生物识别
  enableBiometrics(): Promise<boolean>;
  authenticateWithBiometrics(): Promise<boolean>;
}
```

### 3. DivinationService - 占卜服务

```typescript
interface DivinationService {
  // 八字排盘
  createBaziChart(params: BaziParams): Promise<TxResult>;
  createBaziChartFromLunar(params: LunarBaziParams): Promise<TxResult>;
  createBaziChartFromSizhu(params: SizhuParams): Promise<TxResult>;
  
  // 查询（免费 Runtime API）
  getFullBaziChart(chartId: number): Promise<FullBaziChart | null>;
  getInterpretation(chartId: number): Promise<Interpretation | null>;
  calculateBaziTemp(params: BaziParams): Promise<FullBaziChart | null>;
  
  // 其他占卜类型
  createMeihuaHexagram(params: MeihuaParams): Promise<TxResult>;
  createLiuyaoHexagram(params: LiuyaoParams): Promise<TxResult>;
  createTarotReading(params: TarotParams): Promise<TxResult>;
  
  // 加密存储
  createEncryptedChart(params: EncryptedChartParams): Promise<TxResult>;
}

interface BaziParams {
  year: number;
  month: number;
  day: number;
  hour: number;
  minute: number;
  gender: 'Male' | 'Female';
  zishiMode: 'Traditional' | 'Modern';
  longitude?: number;
  latitude?: number;
}
```

### 4. TradingService - 交易服务

```typescript
interface TradingService {
  // 做市商查询
  getActiveMakers(): Promise<MakerInfo[]>;
  getMakerInfo(makerId: number): Promise<MakerInfo | null>;
  
  // OTC 订单
  createFirstPurchase(params: FirstPurchaseParams): Promise<TxResult>;
  createOrder(params: OrderParams): Promise<TxResult>;
  markPaid(orderId: number, tronTxHash?: string): Promise<TxResult>;
  cancelOrder(orderId: number): Promise<TxResult>;
  
  // Swap 兑换
  createSwap(params: SwapParams): Promise<TxResult>;
  reportSwap(swapId: number): Promise<TxResult>;
  
  // 查询
  getOrder(orderId: number): Promise<Order | null>;
  getBuyerOrders(buyer: string): Promise<number[]>;
  getSwap(swapId: number): Promise<SwapRecord | null>;
}

interface OrderParams {
  makerId: number;
  dustAmount: string;
  paymentCommit: string;  // SHA256 哈希
  contactCommit: string;  // SHA256 哈希
}
```

### 5. ChatService - 聊天服务

```typescript
interface ChatService {
  // 聊天 ID
  getChatUserId(address: string): Promise<string | null>;
  
  // 消息发送
  sendMessage(params: MessageParams): Promise<TxResult>;
  
  // 群组管理
  createGroup(params: GroupParams): Promise<TxResult>;
  joinGroup(groupId: number): Promise<TxResult>;
  leaveGroup(groupId: number): Promise<TxResult>;
  
  // 权限管理
  setPermission(params: PermissionParams): Promise<TxResult>;
  checkPermission(scene: SceneType, user: string): Promise<PermissionLevel>;
}

interface MessageParams {
  recipient: string;
  messageType: MessageType;
  contentCid: string;  // IPFS CID
  encryptionMode?: EncryptionMode;
}
```

### 6. LivestreamService - 直播服务

```typescript
interface LivestreamService {
  // 直播间管理
  createRoom(params: RoomParams): Promise<TxResult>;
  startLive(roomId: number): Promise<TxResult>;
  endLive(roomId: number): Promise<TxResult>;
  
  // Token 获取（通过后端）
  getPublisherToken(roomId: number, signature: string): Promise<TokenResponse>;
  getViewerToken(roomId: number, signature: string): Promise<TokenResponse>;
  getCoHostToken(roomId: number, signature: string): Promise<TokenResponse>;
  
  // 礼物打赏
  sendGift(roomId: number, giftId: number, count: number): Promise<TxResult>;
  
  // 门票
  buyTicket(roomId: number): Promise<TxResult>;
  
  // 查询
  getRoom(roomId: number): Promise<LiveRoom | null>;
  getRooms(filters: RoomFilters): Promise<{ rooms: LiveRoom[]; total: number }>;
}
```

### 7. ArbitrationService - 仲裁服务

```typescript
interface ArbitrationService {
  // 仲裁发起
  disputeWithTwoWayDeposit(
    domain: string,
    id: number,
    evidenceId: number
  ): Promise<TxResult>;
  
  // 应诉
  respondToDispute(
    domain: string,
    id: number,
    counterEvidenceId: number
  ): Promise<TxResult>;
  
  // 证据管理
  appendEvidenceId(
    domain: string,
    id: number,
    evidenceId: number
  ): Promise<TxResult>;
  
  // 投诉
  fileComplaint(params: ComplaintParams): Promise<TxResult>;
  respondToComplaint(complaintId: number, responseCid: string): Promise<TxResult>;
  withdrawComplaint(complaintId: number): Promise<TxResult>;
}
```

## 数据模型

### 钱包数据模型

```typescript
interface WalletAccount {
  address: string;
  name: string;
  isDefault: boolean;
  createdAt: number;
}

interface WalletState {
  accounts: WalletAccount[];
  currentAccount: string | null;
  isLocked: boolean;
  network: 'mainnet' | 'testnet';
}
```

### 占卜数据模型

```typescript
interface BaziChart {
  id: number;
  owner: string;
  sizhu: SiZhu;
  dayun: DaYunInfo;
  wuxingStrength: WuXingStrength;
  kongwang: KongWangInfo;
  shensha: ShenShaEntry[];
  xingyun: XingYunInfo;
  createdAt: number;
}

interface SiZhu {
  yearZhu: Zhu;
  monthZhu: Zhu;
  dayZhu: Zhu;
  hourZhu: Zhu;
  rizhu: TianGan;
}

interface Zhu {
  ganzhi: GanZhi;
  canggan: CangGan[];
  nayin: NaYin;
  shishen: ShiShen;
}

interface Interpretation {
  core: CoreInterpretation;
  xingge?: XingGeAnalysis;
  extendedJishen: WuXing[];
}

interface CoreInterpretation {
  geju: GeJu;
  qiangruo: QiangRuo;
  yongshen: WuXing;
  xishen: WuXing;
  jishen: WuXing;
  score: number;
  confidence: number;
}
```

### 交易数据模型

```typescript
interface Order {
  orderId: number;
  makerId: number;
  maker: string;
  taker: string;
  price: string;
  qty: string;
  amount: string;
  state: OrderState;
  createdAt: number;
  expireAt: number;
  makerTronAddress: string;
  isFirstPurchase: boolean;
}

type OrderState = 
  | 'Created'
  | 'PaidOrCommitted'
  | 'Released'
  | 'Refunded'
  | 'Canceled'
  | 'Disputed'
  | 'Closed'
  | 'Expired';

interface MakerInfo {
  makerId: number;
  owner: string;
  status: MakerStatus;
  direction: Direction;
  tronAddress: string;
  buyPremiumBps: number;
  sellPremiumBps: number;
  minAmount: string;
  usersServed: number;
  maskedFullName: string;
  wechatId: string;
}
```

### 聊天数据模型

```typescript
interface Message {
  id: number;
  sender: string;
  recipient: string;
  messageType: MessageType;
  contentCid: string;
  status: MessageStatus;
  timestamp: number;
}

type MessageType = 
  | 'Text'
  | 'Image'
  | 'File'
  | 'Voice'
  | 'Video'
  | 'System'
  | 'AI';

type MessageStatus = 
  | 'Sent'
  | 'Delivered'
  | 'Read'
  | 'Recalled';

interface ChatGroup {
  id: number;
  name: string;
  owner: string;
  encryptionMode: EncryptionMode;
  members: string[];
  admins: string[];
  createdAt: number;
}

type EncryptionMode = 
  | 'Military'
  | 'Business'
  | 'Selective'
  | 'Transparent';
```

### 直播数据模型

```typescript
interface LiveRoom {
  id: number;
  host: string;
  title: string;
  description?: string;
  roomType: RoomType;
  status: RoomStatus;
  coverCid?: string;
  totalViewers: number;
  peakViewers: number;
  totalGifts: string;
  ticketPrice?: string;
  createdAt: number;
  startedAt?: number;
  endedAt?: number;
}

type RoomType = 'Normal' | 'Paid' | 'Private' | 'MultiHost';
type RoomStatus = 'Preparing' | 'Live' | 'Paused' | 'Ended' | 'Banned';
```



## 错误处理

### 错误分类

| 错误类型 | 处理策略 | 用户提示 |
|----------|----------|----------|
| 网络错误 | 自动重试 3 次，显示重试按钮 | "网络连接失败，请检查网络设置" |
| 链上错误 | 解析错误码，显示具体原因 | "交易失败：余额不足" |
| 签名错误 | 提示重新验证身份 | "签名验证失败，请重新登录" |
| 业务错误 | 根据错误码显示对应提示 | "订单已过期" |
| 服务器错误 | 显示通用错误，记录日志 | "服务暂时不可用，请稍后重试" |

### 链上错误码映射

```typescript
const ChainErrorMessages: Record<string, string> = {
  // Trading 模块
  'trading.InsufficientBalance': '余额不足',
  'trading.OrderNotFound': '订单不存在',
  'trading.InvalidOrderStatus': '订单状态无效',
  'trading.MakerNotActive': '做市商未激活',
  
  // Escrow 模块
  'escrow.Insufficient': '托管余额不足',
  'escrow.NoLock': '托管记录不存在',
  'escrow.InDispute': '订单处于争议状态',
  
  // Arbitration 模块
  'arbitration.AlreadyDisputed': '争议已存在',
  'arbitration.NotDisputed': '争议不存在',
  'arbitration.ResponseDeadlinePassed': '应诉期已过',
  
  // Chat 模块
  'chat.NotAuthorized': '无权限发送消息',
  'chat.UserBlocked': '用户已被拉黑',
  
  // Livestream 模块
  'livestream.RoomNotFound': '直播间不存在',
  'livestream.NotRoomHost': '非直播间主播',
  'livestream.ViewerBanned': '您已被禁止进入该直播间',
};
```

## 测试策略

### 测试分层

| 测试类型 | 覆盖范围 | 工具 |
|----------|----------|------|
| 单元测试 | Service 层、工具函数 | Jest |
| 组件测试 | UI 组件 | React Native Testing Library |
| 集成测试 | 服务间交互 | Jest + Mock |
| E2E 测试 | 完整用户流程 | Detox |

### 测试重点

1. **钱包安全测试**
   - 助记词生成的随机性
   - 私钥加密存储的安全性
   - 签名验证的正确性

2. **链上交互测试**
   - 交易构建的正确性
   - 事件解析的准确性
   - 错误处理的完整性

3. **业务流程测试**
   - OTC 订单完整流程
   - 占卜创建和查询流程
   - 直播进入和互动流程

