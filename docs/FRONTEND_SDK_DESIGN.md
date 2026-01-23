# Stardust 前端 SDK 设计文档

## 概述

本文档定义了 Stardust 区块链前端 SDK 的设计规范，用于与 OCW + TEE 隐私计算架构交互。

## 架构概览

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           前端应用                                       │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    @stardust/sdk                                 │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────┐  │   │
│  │  │ Divination  │ │ Encryption  │ │ Subscription│ │ Storage   │  │   │
│  │  │ Client      │ │ Utils       │ │ Manager     │ │ Client    │  │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └───────────┘  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                │                                        │
│                                ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    @polkadot/api                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        Stardust 区块链                                   │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────────────┐   │
│  │ ocw-tee pallet  │ │ tee-privacy     │ │ divination pallets      │   │
│  │                 │ │ pallet          │ │ (bazi, qimen, meihua)   │   │
│  └─────────────────┘ └─────────────────┘ └─────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

## 核心模块

### 1. DivinationClient - 占卜客户端

主要入口类，提供所有占卜功能的统一接口。

```typescript
import { ApiPromise } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';

interface DivinationClientConfig {
  api: ApiPromise;
  keyring?: Keyring;
  ipfsGateway?: string;
  defaultPrivacyMode?: PrivacyMode;
}

class DivinationClient {
  constructor(config: DivinationClientConfig);
  
  // 占卜模块
  readonly bazi: BaziModule;
  readonly qimen: QimenModule;
  readonly meihua: MeihuaModule;
  
  // 通用方法
  async getRequestStatus(requestId: bigint): Promise<RequestStatus>;
  async getResult(requestId: bigint): Promise<DivinationResult | null>;
  async getUserRequests(account: string): Promise<bigint[]>;
  
  // TEE 节点信息
  async getActiveNodes(): Promise<TeeNodeInfo[]>;
  async getNodeInfo(account: string): Promise<TeeNodeInfo | null>;
}
```

### 2. 占卜模块接口

每个占卜类型都有专门的模块类。

#### 2.1 BaziModule - 八字模块

```typescript
interface BaziInput {
  year: number;      // 出生年 (1900-2100)
  month: number;     // 出生月 (1-12)
  day: number;       // 出生日 (1-31)
  hour: number;      // 出生时 (0-23)
  minute?: number;   // 出生分 (0-59)
  gender: 'male' | 'female';
  timezone?: number; // 时区偏移 (默认 +8)
  location?: {       // 出生地 (用于真太阳时)
    longitude: number;
    latitude: number;
  };
}

interface BaziResult {
  siZhu: {
    year: { gan: string; zhi: string };
    month: { gan: string; zhi: string };
    day: { gan: string; zhi: string };
    hour: { gan: string; zhi: string };
  };
  wuXing: {
    jin: number;
    mu: number;
    shui: number;
    huo: number;
    tu: number;
  };
  riYuan: string;
  yongShen: string[];
  jiShen: string[];
  daYun: DaYunInfo[];
  shenSha: ShenShaEntry[];
}

class BaziModule {
  // 公开模式 - 明文提交
  async createPublic(input: BaziInput): Promise<bigint>;
  
  // 加密模式 - 链上存索引
  async createEncrypted(input: BaziInput): Promise<bigint>;
  
  // 私密模式 - 链上不存索引
  async createPrivate(input: BaziInput): Promise<bigint>;
  
  // 获取结果
  async getResult(requestId: bigint): Promise<BaziResult | null>;
  
  // 订阅结果
  subscribe(requestId: bigint, callback: (result: BaziResult) => void): Unsubscribe;
}
```

#### 2.2 QimenModule - 奇门遁甲模块

```typescript
interface QimenInput {
  year: number;
  month: number;
  day: number;
  hour: number;
  minute?: number;
  question?: string;  // 占问事宜
  method?: 'time' | 'number';  // 起局方式
  numbers?: number[]; // 数字起局时的数字
}

interface QimenResult {
  dunType: 'yang' | 'yin';
  sanYuan: 'shang' | 'zhong' | 'xia';
  ju: number;
  palaces: Palace[];
  yongShen: YongShenInfo;
  geJu: GeJuInfo;
  interpretation?: string;
}

class QimenModule {
  async createPublic(input: QimenInput): Promise<bigint>;
  async createEncrypted(input: QimenInput): Promise<bigint>;
  async createPrivate(input: QimenInput): Promise<bigint>;
  async getResult(requestId: bigint): Promise<QimenResult | null>;
  subscribe(requestId: bigint, callback: (result: QimenResult) => void): Unsubscribe;
}
```

#### 2.3 MeihuaModule - 梅花易数模块

```typescript
interface MeihuaInput {
  method: 'time' | 'number' | 'word' | 'direction';
  // 时间起卦
  time?: {
    year: number;
    month: number;
    day: number;
    hour: number;
  };
  // 数字起卦
  numbers?: number[];
  // 文字起卦
  text?: string;
  // 方位起卦
  direction?: string;
  question?: string;
}

interface MeihuaResult {
  benGua: HexagramInfo;
  huGua: HexagramInfo;
  bianGua: HexagramInfo;
  tiYong: TiYongRelation;
  dongYao: number;
  interpretation?: string;
}

class MeihuaModule {
  async createPublic(input: MeihuaInput): Promise<bigint>;
  async createEncrypted(input: MeihuaInput): Promise<bigint>;
  async createPrivate(input: MeihuaInput): Promise<bigint>;
  async getResult(requestId: bigint): Promise<MeihuaResult | null>;
  subscribe(requestId: bigint, callback: (result: MeihuaResult) => void): Unsubscribe;
}
```

### 3. EncryptionUtils - 加密工具

处理客户端加密，用于 Encrypted 和 Private 模式。

```typescript
class EncryptionUtils {
  // 获取 TEE 节点公钥
  static async getEnclavePublicKey(api: ApiPromise, nodeAccount?: string): Promise<Uint8Array>;
  
  // 生成临时密钥对
  static generateEphemeralKeyPair(): { publicKey: Uint8Array; secretKey: Uint8Array };
  
  // ECDH 密钥交换
  static deriveSharedSecret(
    ephemeralSecretKey: Uint8Array,
    enclavePublicKey: Uint8Array
  ): Uint8Array;
  
  // AES-256-GCM 加密
  static encrypt(
    plaintext: Uint8Array,
    sharedSecret: Uint8Array
  ): { ciphertext: Uint8Array; nonce: Uint8Array; authTag: Uint8Array };
  
  // 完整加密流程
  static async encryptForTee(
    api: ApiPromise,
    data: Uint8Array,
    nodeAccount?: string
  ): Promise<EncryptedData>;
  
  // 解密结果（使用用户私钥）
  static decrypt(
    encryptedData: EncryptedData,
    userSecretKey: Uint8Array
  ): Uint8Array;
}

interface EncryptedData {
  ciphertext: Uint8Array;
  nonce: Uint8Array;
  senderPubkey: Uint8Array;
  authTag?: Uint8Array;
}
```

### 4. SubscriptionManager - 订阅管理

管理事件订阅和结果轮询。

```typescript
class SubscriptionManager {
  constructor(api: ApiPromise);
  
  // 订阅请求状态变化
  subscribeRequestStatus(
    requestId: bigint,
    callback: (status: RequestStatus) => void
  ): Unsubscribe;
  
  // 订阅请求完成事件
  subscribeRequestCompleted(
    requestId: bigint,
    callback: (result: DivinationResult) => void
  ): Unsubscribe;
  
  // 订阅用户所有请求
  subscribeUserRequests(
    account: string,
    callback: (requestIds: bigint[]) => void
  ): Unsubscribe;
  
  // 轮询结果（备用方案）
  pollResult(
    requestId: bigint,
    intervalMs: number,
    timeoutMs: number
  ): Promise<DivinationResult>;
}

type Unsubscribe = () => void;
```

### 5. StorageClient - 存储客户端

与 IPFS 交互获取完整结果。

```typescript
class StorageClient {
  constructor(gatewayUrl: string);
  
  // 获取 JSON 清单
  async getManifest(cid: string): Promise<DivinationManifest>;
  
  // 获取加密清单并解密
  async getEncryptedManifest(
    cid: string,
    userSecretKey: Uint8Array
  ): Promise<DivinationManifest>;
  
  // 验证清单哈希
  verifyManifestHash(manifest: DivinationManifest, expectedHash: Uint8Array): boolean;
}

interface DivinationManifest {
  version: string;
  type: DivinationType;
  timestamp: number;
  input: any;
  result: any;
  metadata: {
    privacyMode: PrivacyMode;
    computedBy: 'ocw' | 'tee';
    teeNode?: string;
    proof?: ComputationProof;
  };
}
```

## 类型定义

### 枚举类型

```typescript
enum DivinationType {
  Meihua = 0,
  BaZi = 1,
  LiuYao = 2,
  QiMen = 3,
  ZiWei = 4,
  TaiYi = 5,
  DaLiuRen = 6,
  XiaoLiuRen = 7,
  Tarot = 8,
}

enum PrivacyMode {
  Public = 0,    // 公开模式
  Encrypted = 1, // 加密模式
  Private = 2,   // 私密模式
}

enum RequestStatus {
  Pending = 0,
  Processing = 1,
  Completed = 2,
  Failed = 3,
  Timeout = 4,
}
```

### 通用类型

```typescript
interface TeeNodeInfo {
  account: string;
  enclavePubkey: Uint8Array;
  teeType: TeeType;
  status: TeeNodeStatus;
  registeredAt: number;
  mrEnclave: Uint8Array;
  attestationTimestamp: number;
}

interface DivinationResult {
  requestId: bigint;
  owner: string;
  divinationType: DivinationType;
  privacyMode: PrivacyMode;
  manifestCid: string;
  manifestHash: Uint8Array;
  typeIndex?: Uint8Array;
  generation: GenerationInfo;
  createdAt: number;
}

interface GenerationInfo {
  type: 'ocw' | 'tee';
  node?: string;
  proof?: ComputationProof;
}

interface ComputationProof {
  mrEnclave: Uint8Array;
  inputHash: Uint8Array;
  outputHash: Uint8Array;
  timestamp: number;
  signature: Uint8Array;
}
```

## 使用示例

### 基本使用

```typescript
import { ApiPromise, WsProvider } from '@polkadot/api';
import { DivinationClient, PrivacyMode } from '@stardust/sdk';

// 初始化
const provider = new WsProvider('wss://rpc.stardust.network');
const api = await ApiPromise.create({ provider });
const client = new DivinationClient({ api });

// 八字排盘 - 公开模式
const requestId = await client.bazi.createPublic({
  year: 1990,
  month: 5,
  day: 15,
  hour: 10,
  gender: 'male',
});

// 等待结果
const result = await client.bazi.getResult(requestId);
console.log('八字结果:', result);
```

### 加密模式

```typescript
import { DivinationClient, EncryptionUtils } from '@stardust/sdk';

// 生成用户密钥对（用于解密结果）
const userKeyPair = EncryptionUtils.generateEphemeralKeyPair();

// 创建加密请求
const requestId = await client.bazi.createEncrypted({
  year: 1990,
  month: 5,
  day: 15,
  hour: 10,
  gender: 'male',
});

// 订阅结果
client.bazi.subscribe(requestId, async (result) => {
  // 从 IPFS 获取加密清单
  const manifest = await client.storage.getEncryptedManifest(
    result.manifestCid,
    userKeyPair.secretKey
  );
  console.log('解密后的结果:', manifest.result);
});
```

### 批量查询

```typescript
// 获取用户所有请求
const requestIds = await client.getUserRequests(userAccount);

// 批量获取结果
const results = await Promise.all(
  requestIds.map(id => client.getResult(id))
);
```

### 订阅事件

```typescript
import { SubscriptionManager } from '@stardust/sdk';

const subscriptionManager = new SubscriptionManager(api);

// 订阅请求状态
const unsubscribe = subscriptionManager.subscribeRequestStatus(
  requestId,
  (status) => {
    console.log('状态更新:', RequestStatus[status]);
    if (status === RequestStatus.Completed) {
      unsubscribe();
    }
  }
);
```

## 错误处理

```typescript
import { StardustError, ErrorCode } from '@stardust/sdk';

try {
  const result = await client.bazi.createPublic(input);
} catch (error) {
  if (error instanceof StardustError) {
    switch (error.code) {
      case ErrorCode.InvalidInput:
        console.error('输入数据无效:', error.message);
        break;
      case ErrorCode.TeeNodeUnavailable:
        console.error('TEE 节点不可用');
        break;
      case ErrorCode.RequestTimeout:
        console.error('请求超时');
        break;
      case ErrorCode.EncryptionFailed:
        console.error('加密失败');
        break;
      default:
        console.error('未知错误:', error);
    }
  }
}

enum ErrorCode {
  InvalidInput = 'INVALID_INPUT',
  TeeNodeUnavailable = 'TEE_NODE_UNAVAILABLE',
  RequestTimeout = 'REQUEST_TIMEOUT',
  EncryptionFailed = 'ENCRYPTION_FAILED',
  DecryptionFailed = 'DECRYPTION_FAILED',
  IpfsError = 'IPFS_ERROR',
  NetworkError = 'NETWORK_ERROR',
  UnknownError = 'UNKNOWN_ERROR',
}
```

## 安全考虑

### 1. 密钥管理

- 用户私钥应安全存储（如浏览器 IndexedDB 加密存储）
- 临时密钥对应在使用后立即销毁
- 不要在日志中输出敏感数据

### 2. 加密最佳实践

```typescript
// ✅ 正确：使用后清除敏感数据
const sharedSecret = EncryptionUtils.deriveSharedSecret(secretKey, pubKey);
try {
  const encrypted = EncryptionUtils.encrypt(data, sharedSecret);
  // 使用加密数据...
} finally {
  // 清除内存中的密钥
  sharedSecret.fill(0);
}

// ❌ 错误：密钥泄露到日志
console.log('Shared secret:', sharedSecret); // 不要这样做！
```

### 3. 验证 TEE 证明

```typescript
// 验证计算确实在 TEE 中执行
function verifyTeeProof(result: DivinationResult): boolean {
  if (result.generation.type !== 'tee') return false;
  
  const proof = result.generation.proof;
  if (!proof) return false;
  
  // 验证 MRENCLAVE 在白名单中
  const allowedEnclaves = await client.getAllowedMrEnclaves();
  if (!allowedEnclaves.some(e => arraysEqual(e, proof.mrEnclave))) {
    return false;
  }
  
  // 验证签名
  return EncryptionUtils.verifyEnclaveSignature(
    result.generation.node!,
    proof
  );
}
```

## 包结构

```
@stardust/sdk/
├── src/
│   ├── index.ts              # 主入口
│   ├── client.ts             # DivinationClient
│   ├── modules/
│   │   ├── bazi.ts           # BaziModule
│   │   ├── qimen.ts          # QimenModule
│   │   ├── meihua.ts         # MeihuaModule
│   │   └── index.ts
│   ├── encryption/
│   │   ├── utils.ts          # EncryptionUtils
│   │   ├── x25519.ts         # X25519 实现
│   │   └── aes-gcm.ts        # AES-GCM 实现
│   ├── subscription/
│   │   └── manager.ts        # SubscriptionManager
│   ├── storage/
│   │   └── client.ts         # StorageClient
│   ├── types/
│   │   ├── divination.ts     # 占卜类型
│   │   ├── tee.ts            # TEE 类型
│   │   └── index.ts
│   └── errors/
│       └── index.ts          # 错误定义
├── package.json
├── tsconfig.json
└── README.md
```

## 依赖

```json
{
  "dependencies": {
    "@polkadot/api": "^10.0.0",
    "@polkadot/keyring": "^12.0.0",
    "@polkadot/util": "^12.0.0",
    "@polkadot/util-crypto": "^12.0.0",
    "tweetnacl": "^1.0.3",
    "ipfs-http-client": "^60.0.0"
  },
  "devDependencies": {
    "typescript": "^5.0.0",
    "@types/node": "^20.0.0"
  }
}
```

## 版本兼容性

| SDK 版本 | Stardust Runtime 版本 | Polkadot API 版本 |
|---------|----------------------|-------------------|
| 1.0.x   | 100+                 | 10.x              |

## 后续计划

1. **v1.1**: 添加 LiuYao（六爻）模块
2. **v1.2**: 添加 ZiWei（紫微斗数）模块
3. **v1.3**: 添加批量请求支持
4. **v2.0**: React Hooks 封装 (`@stardust/react-sdk`)
