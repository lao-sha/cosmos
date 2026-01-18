# 点对点个人聊天前端方案设计

## 概述

本文档描述了基于 Stardust 区块链的点对点个人聊天前端实现方案，与后端 `pallet-chat` 模块完美对接，实现端到端加密的安全即时通讯。

## 技术栈

- **框架**: React Native + Expo
- **状态管理**: Zustand
- **链交互**: @polkadot/api
- **加密**: expo-crypto + SubtleCrypto
- **内容存储**: IPFS HTTP Client

## 目录结构

```
frontend/src/
├── features/
│   ├── chat/
│   │   ├── components/
│   │   │   ├── ChatBubble.tsx          # 消息气泡
│   │   │   ├── ChatInput.tsx           # 输入框组件
│   │   │   ├── SessionItem.tsx         # 会话列表项
│   │   │   ├── MessageList.tsx         # 消息列表
│   │   │   ├── ChatHeader.tsx          # 聊天头部
│   │   │   ├── UserProfileCard.tsx     # 用户资料卡片
│   │   │   ├── OnlineStatus.tsx        # 在线状态指示器
│   │   │   └── PrivacySettings.tsx     # 隐私设置组件
│   │   ├── screens/
│   │   │   ├── SessionListScreen.tsx   # 会话列表页
│   │   │   ├── ChatScreen.tsx          # 聊天详情页
│   │   │   ├── ProfileScreen.tsx       # 个人资料页
│   │   │   ├── UserSearchScreen.tsx    # 用户搜索页（按ChatUserId）
│   │   │   └── SettingsScreen.tsx      # 聊天设置页
│   │   ├── hooks/
│   │   │   ├── useChat.ts              # 聊天核心逻辑
│   │   │   ├── useChatEvents.ts        # 链上事件监听
│   │   │   ├── useChatUser.ts          # 用户资料管理
│   │   │   └── useOnlineStatus.ts      # 在线状态管理
│   │   └── types.ts                    # 类型定义
│   │
│   └── contacts/                       # 通讯录模块
│       ├── components/
│       │   ├── ContactItem.tsx         # 联系人列表项
│       │   ├── ContactCard.tsx         # 联系人详情卡片
│       │   ├── GroupItem.tsx           # 分组列表项
│       │   ├── FriendRequestItem.tsx   # 好友申请项
│       │   └── BlockedUserItem.tsx     # 黑名单用户项
│       ├── screens/
│       │   ├── ContactListScreen.tsx   # 联系人列表页
│       │   ├── ContactDetailScreen.tsx # 联系人详情页
│       │   ├── GroupListScreen.tsx     # 分组列表页
│       │   ├── GroupDetailScreen.tsx   # 分组详情页
│       │   ├── FriendRequestScreen.tsx # 好友申请页
│       │   ├── AddContactScreen.tsx    # 添加联系人页
│       │   └── BlacklistScreen.tsx     # 黑名单管理页
│       ├── hooks/
│       │   ├── useContacts.ts          # 联系人管理
│       │   ├── useGroups.ts            # 分组管理
│       │   └── useFriendRequests.ts    # 好友申请管理
│       └── types.ts                    # 类型定义
│
├── services/
│   ├── chat.service.ts                 # 聊天业务逻辑
│   ├── contacts.service.ts             # 通讯录服务
│   ├── user.service.ts                 # 用户资料服务
│   ├── crypto.service.ts               # 加密服务
│   └── ipfs.service.ts                 # IPFS服务
└── stores/
    ├── chat.store.ts                   # 聊天状态管理
    ├── contacts.store.ts               # 通讯录状态管理
    └── user.store.ts                   # 用户状态管理
```


## 核心类型定义

```typescript
// frontend/src/features/chat/types.ts

/**
 * 聊天用户ID - 11位数字（类似QQ号）
 * 范围：10000000000 - 99999999999
 */
export type ChatUserId = number;

export enum MessageType {
  Text = 0,
  Image = 1,
  File = 2,
  Voice = 3,
  System = 4,
}

/**
 * 用户在线状态
 */
export enum UserStatus {
  Online = 'Online',
  Offline = 'Offline',
  Busy = 'Busy',
  Away = 'Away',
  Invisible = 'Invisible',
}

/**
 * 隐私设置
 */
export interface PrivacySettings {
  /** 是否允许陌生人发送消息 */
  allowStrangerMessages: boolean;
  /** 是否显示在线状态 */
  showOnlineStatus: boolean;
  /** 是否显示最后活跃时间 */
  showLastActive: boolean;
}

/**
 * 聊天用户资料
 */
export interface ChatUserProfile {
  /** 11位聊天ID */
  chatUserId: ChatUserId;
  /** 关联的链上地址 */
  accountId: string;
  /** 昵称 */
  nickname?: string;
  /** 头像 IPFS CID */
  avatarCid?: string;
  /** 个性签名 */
  signature?: string;
  /** 在线状态 */
  status: UserStatus;
  /** 隐私设置 */
  privacySettings: PrivacySettings;
  /** 创建时间戳 */
  createdAt: number;
  /** 最后活跃时间戳 */
  lastActive: number;
}

export interface Message {
  id: number;
  sessionId: string;
  sender: string;
  receiver: string;
  /** 发送方聊天ID（用于显示） */
  senderChatId?: ChatUserId;
  /** 接收方聊天ID（用于显示） */
  receiverChatId?: ChatUserId;
  content: string;           // 解密后的内容
  contentCid: string;        // IPFS CID
  msgType: MessageType;
  sentAt: number;            // 区块高度
  isRead: boolean;
  isDeletedBySender: boolean;
  isDeletedByReceiver: boolean;
  isMine: boolean;           // 是否是自己发的
  status: 'sending' | 'sent' | 'failed';
}

export interface Session {
  id: string;
  participants: string[];
  peerAddress: string;       // 对方地址
  /** 对方聊天ID */
  peerChatId?: ChatUserId;
  peerAlias?: string;        // 对方昵称
  /** 对方用户资料 */
  peerProfile?: ChatUserProfile;
  lastMessage?: Message;
  lastActive: number;
  unreadCount: number;
  isArchived: boolean;
}

export interface ChatUser {
  address: string;
  chatUserId?: ChatUserId;
  profile?: ChatUserProfile;
  isBlocked: boolean;
}
```

## 状态管理 (Zustand Store)

```typescript
// frontend/src/stores/chat.store.ts

import { create } from 'zustand';
import type { Session, Message } from '@/features/chat/types';

interface ChatState {
  // 状态
  sessions: Session[];
  currentSession: Session | null;
  messages: Record<string, Message[]>;  // sessionId -> messages
  totalUnread: number;
  isLoading: boolean;
  
  // 会话操作
  loadSessions: () => Promise<void>;
  selectSession: (sessionId: string) => void;
  archiveSession: (sessionId: string) => Promise<void>;
  
  // 消息操作
  loadMessages: (sessionId: string, offset?: number) => Promise<void>;
  sendMessage: (receiver: string, content: string, msgType?: number) => Promise<void>;
  markAsRead: (messageIds: number[]) => Promise<void>;
  markSessionAsRead: (sessionId: string) => Promise<void>;
  deleteMessage: (msgId: number) => Promise<void>;
  
  // 黑名单
  blockUser: (address: string) => Promise<void>;
  unblockUser: (address: string) => Promise<void>;
  
  // 事件处理
  handleNewMessage: (message: Message) => void;
}

export const useChatStore = create<ChatState>()((set, get) => ({
  sessions: [],
  currentSession: null,
  messages: {},
  totalUnread: 0,
  isLoading: false,
  
  loadSessions: async () => {
    set({ isLoading: true });
    try {
      const chatService = getChatService();
      const sessions = await chatService.getSessions();
      set({ sessions });
    } finally {
      set({ isLoading: false });
    }
  },
  
  selectSession: (sessionId: string) => {
    const session = get().sessions.find(s => s.id === sessionId);
    set({ currentSession: session || null });
  },
  
  sendMessage: async (receiver, content, msgType = 0) => {
    const chatService = getChatService();
    const currentSession = get().currentSession;
    
    // 乐观更新 UI
    const tempMessage: Message = {
      id: Date.now(),
      sessionId: currentSession?.id || '',
      sender: chatService.myAddress,
      receiver,
      content,
      contentCid: '',
      msgType,
      sentAt: 0,
      isRead: false,
      isDeletedBySender: false,
      isDeletedByReceiver: false,
      isMine: true,
      status: 'sending',
    };
    
    set(state => ({
      messages: {
        ...state.messages,
        [currentSession?.id || '']: [
          ...(state.messages[currentSession?.id || ''] || []),
          tempMessage,
        ],
      },
    }));
    
    try {
      const result = await chatService.sendMessage(receiver, content, msgType);
      // 更新消息状态为已发送
      set(state => ({
        messages: {
          ...state.messages,
          [result.sessionId]: state.messages[result.sessionId]?.map(m =>
            m.id === tempMessage.id
              ? { ...m, id: result.msgId, status: 'sent' as const }
              : m
          ) || [],
        },
      }));
    } catch (error) {
      // 标记发送失败
      set(state => ({
        messages: {
          ...state.messages,
          [currentSession?.id || '']: state.messages[currentSession?.id || '']?.map(m =>
            m.id === tempMessage.id ? { ...m, status: 'failed' as const } : m
          ) || [],
        },
      }));
      throw error;
    }
  },
  
  handleNewMessage: (message: Message) => {
    set(state => {
      const sessionMessages = state.messages[message.sessionId] || [];
      return {
        messages: {
          ...state.messages,
          [message.sessionId]: [...sessionMessages, message],
        },
        totalUnread: state.totalUnread + (message.isMine ? 0 : 1),
      };
    });
  },
  
  // ... 其他方法实现
}));
```


## 加密服务

```typescript
// frontend/src/services/crypto.service.ts

import * as Crypto from 'expo-crypto';
import { u8aToHex, hexToU8a } from '@polkadot/util';
import { x25519 } from '@noble/curves/ed25519';

/**
 * X25519 ECDH 密钥交换
 * 使用 @noble/curves 的 x25519 实现椭圆曲线 Diffie-Hellman
 *
 * 注意：sr25519 本身不支持 ECDH，需要使用 x25519（Curve25519）
 * @noble/curves 是一个经过审计的纯 JavaScript 实现，安全可靠
 */
export async function deriveSharedKey(
  myPrivateKey: Uint8Array,  // 32 bytes x25519 private key
  peerPublicKey: Uint8Array  // 32 bytes x25519 public key
): Promise<Uint8Array> {
  // 使用 x25519 进行 ECDH 密钥协商
  const sharedSecret = x25519.getSharedSecret(myPrivateKey, peerPublicKey);

  // 使用 HKDF 派生最终的 AES 密钥
  return await hkdfDerive(sharedSecret, 32, 'stardust-chat-v1');
}

/**
 * 生成 x25519 密钥对
 * 用于端到端加密的密钥交换
 */
export function generateX25519KeyPair(): { privateKey: Uint8Array; publicKey: Uint8Array } {
  const privateKey = x25519.utils.randomPrivateKey();
  const publicKey = x25519.getPublicKey(privateKey);
  return { privateKey, publicKey };
}

/**
 * HKDF 密钥派生函数
 * 从共享密钥派生出指定长度的加密密钥
 */
async function hkdfDerive(
  inputKey: Uint8Array,
  length: number,
  info: string
): Promise<Uint8Array> {
  const cryptoKey = await crypto.subtle.importKey(
    'raw',
    inputKey,
    { name: 'HKDF' },
    false,
    ['deriveBits']
  );
  
  const derivedBits = await crypto.subtle.deriveBits(
    {
      name: 'HKDF',
      hash: 'SHA-256',
      salt: new Uint8Array(32), // 可使用随机 salt 增强安全性
      info: new TextEncoder().encode(info),
    },
    cryptoKey,
    length * 8
  );
  
  return new Uint8Array(derivedBits);
}

/**
 * 备选方案：使用 Web Crypto API 的 ECDH (P-256)
 * 适用于不支持 sr25519 的环境
 */
export async function deriveSharedKeyP256(
  myKeyPair: CryptoKeyPair,
  peerPublicKey: CryptoKey
): Promise<Uint8Array> {
  const sharedSecret = await crypto.subtle.deriveBits(
    {
      name: 'ECDH',
      public: peerPublicKey,
    },
    myKeyPair.privateKey,
    256
  );
  
  return new Uint8Array(sharedSecret);
}

/**
 * 生成临时 ECDH 密钥对 (P-256)
 */
export async function generateECDHKeyPair(): Promise<CryptoKeyPair> {
  return crypto.subtle.generateKey(
    {
      name: 'ECDH',
      namedCurve: 'P-256',
    },
    true,
    ['deriveBits']
  );
}

/**
 * 导出公钥为字节数组（用于传输）
 */
export async function exportPublicKey(publicKey: CryptoKey): Promise<Uint8Array> {
  const exported = await crypto.subtle.exportKey('raw', publicKey);
  return new Uint8Array(exported);
}

/**
 * 导入公钥（从字节数组）
 */
export async function importPublicKey(keyData: Uint8Array): Promise<CryptoKey> {
  return crypto.subtle.importKey(
    'raw',
    keyData,
    {
      name: 'ECDH',
      namedCurve: 'P-256',
    },
    true,
    []
  );
}

/**
 * AES-256-GCM 加密消息
 * IV: 12 bytes (96 bits) - GCM 推荐长度
 * Tag: 16 bytes (128 bits) - 认证标签，自动附加在密文后
 */
export async function encryptMessage(
  plaintext: string,
  sharedKey: Uint8Array
): Promise<Uint8Array> {
  // 生成随机 IV
  const iv = crypto.getRandomValues(new Uint8Array(12));
  
  // 导入 AES 密钥
  const aesKey = await crypto.subtle.importKey(
    'raw',
    sharedKey,
    { name: 'AES-GCM' },
    false,
    ['encrypt']
  );
  
  // 加密
  const encoder = new TextEncoder();
  const encrypted = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv, tagLength: 128 },
    aesKey,
    encoder.encode(plaintext)
  );
  
  // 返回 IV (12 bytes) + Ciphertext + Tag
  const result = new Uint8Array(iv.length + encrypted.byteLength);
  result.set(iv, 0);
  result.set(new Uint8Array(encrypted), iv.length);
  
  return result;
}

/**
 * AES-256-GCM 解密消息
 */
export async function decryptMessage(
  ciphertext: Uint8Array,
  sharedKey: Uint8Array
): Promise<string> {
  // 提取 IV 和密文
  const iv = ciphertext.slice(0, 12);
  const data = ciphertext.slice(12);
  
  // 导入 AES 密钥
  const aesKey = await crypto.subtle.importKey(
    'raw',
    sharedKey,
    { name: 'AES-GCM' },
    false,
    ['decrypt']
  );
  
  // 解密（自动验证认证标签）
  const decrypted = await crypto.subtle.decrypt(
    { name: 'AES-GCM', iv, tagLength: 128 },
    aesKey,
    data
  );
  
  return new TextDecoder().decode(decrypted);
}

/**
 * 使用 sr25519 签名消息
 */
export async function signMessage(
  message: Uint8Array,
  secretKey: Uint8Array
): Promise<Uint8Array> {
  const { sr25519Sign } = await import('@polkadot/util-crypto');
  return sr25519Sign(message, { secretKey });
}

/**
 * 验证 sr25519 签名
 */
export async function verifySignature(
  message: Uint8Array,
  signature: Uint8Array,
  publicKey: Uint8Array
): Promise<boolean> {
  const { sr25519Verify } = await import('@polkadot/util-crypto');
  return sr25519Verify(message, signature, publicKey);
}
```


## IPFS 服务

```typescript
// frontend/src/services/ipfs.service.ts

import AsyncStorage from '@react-native-async-storage/async-storage';

/**
 * IPFS 配置接口
 */
interface IpfsConfig {
  // 上传 API 端点（支持多个备选）
  apiEndpoints: string[];
  // 下载网关（支持多个备选）
  gateways: string[];
  // 请求超时（毫秒）
  timeout: number;
  // 重试次数
  retries: number;
}

/**
 * 默认配置 - 支持多环境
 */
const DEFAULT_CONFIG: IpfsConfig = {
  apiEndpoints: [
    process.env.EXPO_PUBLIC_IPFS_API || '',
    'https://api.pinata.cloud/pinning/pinFileToIPFS',  // Pinata
    'https://api.web3.storage/upload',                  // Web3.Storage
  ].filter(Boolean),
  gateways: [
    process.env.EXPO_PUBLIC_IPFS_GATEWAY || '',
    'https://gateway.pinata.cloud/ipfs/',
    'https://w3s.link/ipfs/',
    'https://cloudflare-ipfs.com/ipfs/',
    'https://ipfs.io/ipfs/',
  ].filter(Boolean),
  timeout: 30000,
  retries: 3,
};

let config: IpfsConfig = { ...DEFAULT_CONFIG };

/**
 * 初始化 IPFS 配置
 */
export async function initIpfsService(customConfig?: Partial<IpfsConfig>): Promise<void> {
  // 从本地存储加载配置
  const stored = await AsyncStorage.getItem('ipfs_config');
  if (stored) {
    config = { ...DEFAULT_CONFIG, ...JSON.parse(stored) };
  }
  
  // 合并自定义配置
  if (customConfig) {
    config = { ...config, ...customConfig };
  }
}

/**
 * 更新 IPFS 配置
 */
export async function updateIpfsConfig(newConfig: Partial<IpfsConfig>): Promise<void> {
  config = { ...config, ...newConfig };
  await AsyncStorage.setItem('ipfs_config', JSON.stringify(config));
}

/**
 * 获取当前配置
 */
export function getIpfsConfig(): IpfsConfig {
  return { ...config };
}

/**
 * 上传加密内容到 IPFS（带重试和故障转移）
 */
export async function uploadToIpfs(content: Uint8Array): Promise<string> {
  const errors: Error[] = [];
  
  for (const endpoint of config.apiEndpoints) {
    for (let attempt = 0; attempt < config.retries; attempt++) {
      try {
        const cid = await uploadToEndpoint(endpoint, content);
        return cid;
      } catch (error) {
        errors.push(error as Error);
        // 短暂延迟后重试
        await delay(1000 * (attempt + 1));
      }
    }
  }
  
  throw new AggregateError(errors, 'All IPFS upload attempts failed');
}

/**
 * 上传到指定端点
 */
async function uploadToEndpoint(endpoint: string, content: Uint8Array): Promise<string> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), config.timeout);
  
  try {
    // 检测端点类型并使用对应的上传方式
    if (endpoint.includes('pinata.cloud')) {
      return await uploadToPinata(endpoint, content, controller.signal);
    } else if (endpoint.includes('web3.storage')) {
      return await uploadToWeb3Storage(endpoint, content, controller.signal);
    } else {
      return await uploadToStandardIpfs(endpoint, content, controller.signal);
    }
  } finally {
    clearTimeout(timeoutId);
  }
}

/**
 * 标准 IPFS API 上传
 */
async function uploadToStandardIpfs(
  endpoint: string,
  content: Uint8Array,
  signal: AbortSignal
): Promise<string> {
  const formData = new FormData();
  formData.append('file', new Blob([content], { type: 'application/octet-stream' }));
  
  const response = await fetch(`${endpoint}/add?pin=true`, {
    method: 'POST',
    body: formData,
    signal,
  });
  
  if (!response.ok) {
    throw new Error(`IPFS upload failed: ${response.status}`);
  }
  
  const result = await response.json();
  return result.Hash;
}

/**
 * Pinata 上传
 */
async function uploadToPinata(
  endpoint: string,
  content: Uint8Array,
  signal: AbortSignal
): Promise<string> {
  const apiKey = process.env.EXPO_PUBLIC_PINATA_API_KEY;
  const apiSecret = process.env.EXPO_PUBLIC_PINATA_API_SECRET;
  
  if (!apiKey || !apiSecret) {
    throw new Error('Pinata API credentials not configured');
  }
  
  const formData = new FormData();
  formData.append('file', new Blob([content], { type: 'application/octet-stream' }));
  
  const response = await fetch(endpoint, {
    method: 'POST',
    headers: {
      'pinata_api_key': apiKey,
      'pinata_secret_api_key': apiSecret,
    },
    body: formData,
    signal,
  });
  
  if (!response.ok) {
    throw new Error(`Pinata upload failed: ${response.status}`);
  }
  
  const result = await response.json();
  return result.IpfsHash;
}

/**
 * Web3.Storage 上传
 */
async function uploadToWeb3Storage(
  endpoint: string,
  content: Uint8Array,
  signal: AbortSignal
): Promise<string> {
  const token = process.env.EXPO_PUBLIC_WEB3_STORAGE_TOKEN;
  
  if (!token) {
    throw new Error('Web3.Storage token not configured');
  }
  
  const response = await fetch(endpoint, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/octet-stream',
    },
    body: content,
    signal,
  });
  
  if (!response.ok) {
    throw new Error(`Web3.Storage upload failed: ${response.status}`);
  }
  
  const result = await response.json();
  return result.cid;
}

/**
 * 从 IPFS 下载内容（带故障转移）
 */
export async function downloadFromIpfs(cid: string): Promise<Uint8Array> {
  const errors: Error[] = [];
  
  for (const gateway of config.gateways) {
    try {
      return await downloadFromGateway(gateway, cid);
    } catch (error) {
      errors.push(error as Error);
    }
  }
  
  throw new AggregateError(errors, 'All IPFS download attempts failed');
}

/**
 * 从指定网关下载
 */
async function downloadFromGateway(gateway: string, cid: string): Promise<Uint8Array> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), config.timeout);
  
  try {
    const response = await fetch(`${gateway}${cid}`, {
      signal: controller.signal,
    });
    
    if (!response.ok) {
      throw new Error(`IPFS download failed: ${response.status}`);
    }
    
    const buffer = await response.arrayBuffer();
    return new Uint8Array(buffer);
  } finally {
    clearTimeout(timeoutId);
  }
}

/**
 * 检查 CID 是否可访问
 */
export async function checkCidAvailability(cid: string): Promise<boolean> {
  for (const gateway of config.gateways) {
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 5000);
      
      const response = await fetch(`${gateway}${cid}`, {
        method: 'HEAD',
        signal: controller.signal,
      });
      
      clearTimeout(timeoutId);
      
      if (response.ok) {
        return true;
      }
    } catch {
      // 继续尝试下一个网关
    }
  }
  return false;
}

function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}
```

### IPFS 环境变量配置

```bash
# .env.development
EXPO_PUBLIC_IPFS_API=http://localhost:5001/api/v0
EXPO_PUBLIC_IPFS_GATEWAY=http://localhost:8080/ipfs/

# .env.production
EXPO_PUBLIC_IPFS_API=https://your-ipfs-node.example.com/api/v0
EXPO_PUBLIC_IPFS_GATEWAY=https://your-gateway.example.com/ipfs/

# 可选：第三方服务
EXPO_PUBLIC_PINATA_API_KEY=your_pinata_key
EXPO_PUBLIC_PINATA_API_SECRET=your_pinata_secret
EXPO_PUBLIC_WEB3_STORAGE_TOKEN=your_web3storage_token
```

## 用户服务（ChatUserId 系统）

```typescript
// frontend/src/services/user.service.ts

import { ApiPromise } from '@polkadot/api';
import { getApi } from '@/api';
import { uploadToIpfs } from './ipfs.service';
import type {
  ChatUserId,
  ChatUserProfile,
  UserStatus,
  PrivacySettings,
} from '@/features/chat/types';

/**
 * 用户服务 - 管理 ChatUserId 和用户资料
 */
export class UserService {
  private api: ApiPromise | null = null;
  private myAddress: string;
  private myChatUserId: ChatUserId | null = null;
  private profileCache: Map<ChatUserId, ChatUserProfile> = new Map();
  
  constructor(myAddress: string) {
    this.myAddress = myAddress;
  }
  
  async init(): Promise<void> {
    this.api = await getApi();
    // 获取自己的 ChatUserId
    this.myChatUserId = await this.getChatUserIdByAccount(this.myAddress);
  }
  
  /**
   * 注册聊天用户，获取 11 位 ChatUserId
   * @param preferredId 可选的首选 ID（如果可用）
   */
  async registerChatUser(preferredId?: ChatUserId): Promise<ChatUserId> {
    if (!this.api) throw new Error('API not initialized');
    
    // 检查是否已注册
    const existing = await this.getChatUserIdByAccount(this.myAddress);
    if (existing) {
      this.myChatUserId = existing;
      return existing;
    }
    
    const tx = this.api.tx.chat.registerChatUser(preferredId || null);
    
    return new Promise((resolve, reject) => {
      tx.signAndSend(this.myAddress, ({ status, events }) => {
        if (status.isInBlock) {
          for (const { event } of events) {
            if (this.api!.events.chat.ChatUserCreated.is(event)) {
              const [, chatUserId] = event.data;
              this.myChatUserId = chatUserId.toNumber() as ChatUserId;
              resolve(this.myChatUserId);
              return;
            }
          }
          reject(new Error('ChatUserCreated event not found'));
        }
      }).catch(reject);
    });
  }
  
  /**
   * 通过账户地址获取 ChatUserId
   */
  async getChatUserIdByAccount(address: string): Promise<ChatUserId | null> {
    if (!this.api) return null;
    
    const result = await this.api.query.chat.accountToChatUserId(address);
    if (result.isNone) return null;
    
    return result.unwrap().toNumber() as ChatUserId;
  }
  
  /**
   * 通过 ChatUserId 获取账户地址
   */
  async getAccountByChatUserId(chatUserId: ChatUserId): Promise<string | null> {
    if (!this.api) return null;
    
    const result = await this.api.query.chat.chatUserIdToAccount(chatUserId);
    if (result.isNone) return null;
    
    return result.unwrap().toString();
  }
  
  /**
   * 获取用户资料
   */
  async getUserProfile(chatUserId: ChatUserId): Promise<ChatUserProfile | null> {
    if (!this.api) return null;
    
    // 检查缓存
    if (this.profileCache.has(chatUserId)) {
      return this.profileCache.get(chatUserId)!;
    }
    
    const result = await this.api.query.chat.chatUserProfiles(chatUserId);
    if (result.isNone) return null;
    
    const data = result.unwrap();
    const accountId = await this.getAccountByChatUserId(chatUserId);
    
    const profile: ChatUserProfile = {
      chatUserId,
      accountId: accountId || '',
      nickname: data.nickname.isSome ? data.nickname.unwrap().toUtf8() : undefined,
      avatarCid: data.avatarCid.isSome ? data.avatarCid.unwrap().toUtf8() : undefined,
      signature: data.signature.isSome ? data.signature.unwrap().toUtf8() : undefined,
      status: this.parseUserStatus(data.status.toString()),
      privacySettings: {
        allowStrangerMessages: data.privacySettings.allowStrangerMessages.valueOf(),
        showOnlineStatus: data.privacySettings.showOnlineStatus.valueOf(),
        showLastActive: data.privacySettings.showLastActive.valueOf(),
      },
      createdAt: data.createdAt.toNumber(),
      lastActive: data.lastActive.toNumber(),
    };
    
    // 缓存资料
    this.profileCache.set(chatUserId, profile);
    
    return profile;
  }
  
  /**
   * 更新个人资料
   */
  async updateProfile(params: {
    nickname?: string;
    avatarCid?: string;
    signature?: string;
  }): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.chat.updateChatProfile(
      params.nickname || null,
      params.avatarCid || null,
      params.signature || null
    );
    
    await tx.signAndSend(this.myAddress);
    
    // 清除缓存
    if (this.myChatUserId) {
      this.profileCache.delete(this.myChatUserId);
    }
  }
  
  /**
   * 上传头像并更新资料
   */
  async uploadAvatar(imageData: Uint8Array): Promise<string> {
    const cid = await uploadToIpfs(imageData);
    await this.updateProfile({ avatarCid: cid });
    return cid;
  }
  
  /**
   * 设置在线状态
   */
  async setStatus(status: UserStatus): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const statusCode = this.statusToCode(status);
    const tx = this.api.tx.chat.setUserStatus(statusCode);
    
    await tx.signAndSend(this.myAddress);
    
    // 更新缓存
    if (this.myChatUserId && this.profileCache.has(this.myChatUserId)) {
      const profile = this.profileCache.get(this.myChatUserId)!;
      profile.status = status;
    }
  }
  
  /**
   * 更新隐私设置
   */
  async updatePrivacySettings(settings: Partial<PrivacySettings>): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.chat.updatePrivacySettings(
      settings.allowStrangerMessages ?? null,
      settings.showOnlineStatus ?? null,
      settings.showLastActive ?? null
    );
    
    await tx.signAndSend(this.myAddress);
    
    // 更新缓存
    if (this.myChatUserId && this.profileCache.has(this.myChatUserId)) {
      const profile = this.profileCache.get(this.myChatUserId)!;
      profile.privacySettings = { ...profile.privacySettings, ...settings };
    }
  }
  
  /**
   * 通过 ChatUserId 搜索用户
   */
  async searchUserById(chatUserId: ChatUserId): Promise<ChatUserProfile | null> {
    // 验证 ID 格式（11位数字）
    if (chatUserId < 10_000_000_000 || chatUserId > 99_999_999_999) {
      return null;
    }
    
    return this.getUserProfile(chatUserId);
  }
  
  /**
   * 批量获取用户资料
   */
  async batchGetProfiles(chatUserIds: ChatUserId[]): Promise<Map<ChatUserId, ChatUserProfile>> {
    const results = new Map<ChatUserId, ChatUserProfile>();
    
    // 并行获取
    await Promise.all(
      chatUserIds.map(async (id) => {
        const profile = await this.getUserProfile(id);
        if (profile) {
          results.set(id, profile);
        }
      })
    );
    
    return results;
  }
  
  /**
   * 获取自己的 ChatUserId
   */
  getMyChatUserId(): ChatUserId | null {
    return this.myChatUserId;
  }
  
  /**
   * 获取自己的资料
   */
  async getMyProfile(): Promise<ChatUserProfile | null> {
    if (!this.myChatUserId) return null;
    return this.getUserProfile(this.myChatUserId);
  }
  
  /**
   * 清除资料缓存
   */
  clearCache(): void {
    this.profileCache.clear();
  }
  
  private parseUserStatus(status: string): UserStatus {
    const statusMap: Record<string, UserStatus> = {
      'Online': UserStatus.Online,
      'Offline': UserStatus.Offline,
      'Busy': UserStatus.Busy,
      'Away': UserStatus.Away,
      'Invisible': UserStatus.Invisible,
    };
    return statusMap[status] || UserStatus.Offline;
  }
  
  private statusToCode(status: UserStatus): number {
    const codeMap: Record<UserStatus, number> = {
      [UserStatus.Online]: 0,
      [UserStatus.Offline]: 1,
      [UserStatus.Busy]: 2,
      [UserStatus.Away]: 3,
      [UserStatus.Invisible]: 4,
    };
    return codeMap[status];
  }
}

// 单例
let userServiceInstance: UserService | null = null;

export function getUserService(): UserService {
  if (!userServiceInstance) {
    throw new Error('UserService not initialized');
  }
  return userServiceInstance;
}

export function initUserService(myAddress: string): UserService {
  userServiceInstance = new UserService(myAddress);
  return userServiceInstance;
}
```

## 用户状态管理 (Zustand Store)

```typescript
// frontend/src/stores/user.store.ts

import { create } from 'zustand';
import { initUserService, getUserService } from '@/services/user.service';
import type {
  ChatUserId,
  ChatUserProfile,
  UserStatus,
  PrivacySettings,
} from '@/features/chat/types';

interface UserState {
  // 状态
  isRegistered: boolean;
  myChatUserId: ChatUserId | null;
  myProfile: ChatUserProfile | null;
  isLoading: boolean;
  error: string | null;
  
  // 操作
  initialize: (address: string) => Promise<void>;
  register: (preferredId?: ChatUserId) => Promise<ChatUserId>;
  updateProfile: (params: { nickname?: string; signature?: string }) => Promise<void>;
  uploadAvatar: (imageData: Uint8Array) => Promise<string>;
  setStatus: (status: UserStatus) => Promise<void>;
  updatePrivacySettings: (settings: Partial<PrivacySettings>) => Promise<void>;
  searchUser: (chatUserId: ChatUserId) => Promise<ChatUserProfile | null>;
  refresh: () => Promise<void>;
}

export const useUserStore = create<UserState>()((set, get) => ({
  isRegistered: false,
  myChatUserId: null,
  myProfile: null,
  isLoading: false,
  error: null,
  
  initialize: async (address: string) => {
    set({ isLoading: true, error: null });
    
    try {
      const service = initUserService(address);
      await service.init();
      
      const chatUserId = service.getMyChatUserId();
      const profile = chatUserId ? await service.getMyProfile() : null;
      
      set({
        isRegistered: !!chatUserId,
        myChatUserId: chatUserId,
        myProfile: profile,
      });
    } catch (error) {
      set({ error: (error as Error).message });
    } finally {
      set({ isLoading: false });
    }
  },
  
  register: async (preferredId?: ChatUserId) => {
    set({ isLoading: true, error: null });
    
    try {
      const service = getUserService();
      const chatUserId = await service.registerChatUser(preferredId);
      const profile = await service.getMyProfile();
      
      set({
        isRegistered: true,
        myChatUserId: chatUserId,
        myProfile: profile,
      });
      
      return chatUserId;
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },
  
  updateProfile: async (params) => {
    set({ isLoading: true, error: null });
    
    try {
      const service = getUserService();
      await service.updateProfile(params);
      
      // 刷新资料
      const profile = await service.getMyProfile();
      set({ myProfile: profile });
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },
  
  uploadAvatar: async (imageData: Uint8Array) => {
    set({ isLoading: true, error: null });
    
    try {
      const service = getUserService();
      const cid = await service.uploadAvatar(imageData);
      
      // 刷新资料
      const profile = await service.getMyProfile();
      set({ myProfile: profile });
      
      return cid;
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },
  
  setStatus: async (status: UserStatus) => {
    try {
      const service = getUserService();
      await service.setStatus(status);
      
      set(state => ({
        myProfile: state.myProfile
          ? { ...state.myProfile, status }
          : null,
      }));
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },
  
  updatePrivacySettings: async (settings: Partial<PrivacySettings>) => {
    set({ isLoading: true, error: null });
    
    try {
      const service = getUserService();
      await service.updatePrivacySettings(settings);
      
      set(state => ({
        myProfile: state.myProfile
          ? {
              ...state.myProfile,
              privacySettings: { ...state.myProfile.privacySettings, ...settings },
            }
          : null,
      }));
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },
  
  searchUser: async (chatUserId: ChatUserId) => {
    try {
      const service = getUserService();
      return await service.searchUserById(chatUserId);
    } catch (error) {
      set({ error: (error as Error).message });
      return null;
    }
  },
  
  refresh: async () => {
    const service = getUserService();
    const profile = await service.getMyProfile();
    set({ myProfile: profile });
  },
}));
```

## 用户相关 UI 组件

### 用户资料卡片

```tsx
// frontend/src/features/chat/components/UserProfileCard.tsx

import React from 'react';
import { View, Text, Image, StyleSheet, TouchableOpacity } from 'react-native';
import { OnlineStatus } from './OnlineStatus';
import type { ChatUserProfile } from '../types';

interface Props {
  profile: ChatUserProfile;
  onPress?: () => void;
  showChatId?: boolean;
}

export function UserProfileCard({ profile, onPress, showChatId = true }: Props) {
  const avatarUri = profile.avatarCid
    ? `https://gateway.pinata.cloud/ipfs/${profile.avatarCid}`
    : null;
  
  return (
    <TouchableOpacity
      style={styles.container}
      onPress={onPress}
      disabled={!onPress}
    >
      <View style={styles.avatarContainer}>
        {avatarUri ? (
          <Image source={{ uri: avatarUri }} style={styles.avatar} />
        ) : (
          <View style={[styles.avatar, styles.avatarPlaceholder]}>
            <Text style={styles.avatarText}>
              {profile.nickname?.[0] || profile.chatUserId.toString().slice(-2)}
            </Text>
          </View>
        )}
        <OnlineStatus status={profile.status} style={styles.statusBadge} />
      </View>
      
      <View style={styles.info}>
        <Text style={styles.nickname} numberOfLines={1}>
          {profile.nickname || `用户${profile.chatUserId}`}
        </Text>
        
        {showChatId && (
          <Text style={styles.chatId}>
            ID: {profile.chatUserId}
          </Text>
        )}
        
        {profile.signature && (
          <Text style={styles.signature} numberOfLines={1}>
            {profile.signature}
          </Text>
        )}
      </View>
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
    backgroundColor: '#fff',
  },
  avatarContainer: {
    position: 'relative',
  },
  avatar: {
    width: 56,
    height: 56,
    borderRadius: 28,
  },
  avatarPlaceholder: {
    backgroundColor: '#007AFF',
    justifyContent: 'center',
    alignItems: 'center',
  },
  avatarText: {
    color: '#fff',
    fontSize: 20,
    fontWeight: '600',
  },
  statusBadge: {
    position: 'absolute',
    bottom: 0,
    right: 0,
  },
  info: {
    flex: 1,
    marginLeft: 12,
  },
  nickname: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  chatId: {
    fontSize: 13,
    color: '#666',
    marginTop: 2,
  },
  signature: {
    fontSize: 14,
    color: '#999',
    marginTop: 4,
  },
});
```

### 在线状态指示器

```tsx
// frontend/src/features/chat/components/OnlineStatus.tsx

import React from 'react';
import { View, StyleSheet, ViewStyle } from 'react-native';
import { UserStatus } from '../types';

interface Props {
  status: UserStatus;
  size?: 'small' | 'medium' | 'large';
  style?: ViewStyle;
}

const STATUS_COLORS: Record<UserStatus, string> = {
  [UserStatus.Online]: '#34C759',   // 绿色
  [UserStatus.Offline]: '#8E8E93',  // 灰色
  [UserStatus.Busy]: '#FF3B30',     // 红色
  [UserStatus.Away]: '#FF9500',     // 橙色
  [UserStatus.Invisible]: '#8E8E93', // 灰色（隐身显示为离线）
};

const SIZES = {
  small: 8,
  medium: 12,
  large: 16,
};

export function OnlineStatus({ status, size = 'medium', style }: Props) {
  const dotSize = SIZES[size];
  const color = STATUS_COLORS[status];
  
  return (
    <View
      style={[
        styles.dot,
        {
          width: dotSize,
          height: dotSize,
          borderRadius: dotSize / 2,
          backgroundColor: color,
        },
        style,
      ]}
    />
  );
}

const styles = StyleSheet.create({
  dot: {
    borderWidth: 2,
    borderColor: '#fff',
  },
});
```

### 隐私设置组件

```tsx
// frontend/src/features/chat/components/PrivacySettings.tsx

import React from 'react';
import { View, Text, Switch, StyleSheet } from 'react-native';
import { useUserStore } from '@/stores/user.store';
import type { PrivacySettings as PrivacySettingsType } from '../types';

export function PrivacySettingsPanel() {
  const { myProfile, updatePrivacySettings, isLoading } = useUserStore();
  
  if (!myProfile) return null;
  
  const settings = myProfile.privacySettings;
  
  const handleToggle = async (key: keyof PrivacySettingsType, value: boolean) => {
    await updatePrivacySettings({ [key]: value });
  };
  
  return (
    <View style={styles.container}>
      <Text style={styles.title}>隐私设置</Text>
      
      <View style={styles.item}>
        <View style={styles.itemText}>
          <Text style={styles.itemTitle}>允许陌生人消息</Text>
          <Text style={styles.itemDesc}>关闭后，非好友无法给您发送消息</Text>
        </View>
        <Switch
          value={settings.allowStrangerMessages}
          onValueChange={(v) => handleToggle('allowStrangerMessages', v)}
          disabled={isLoading}
        />
      </View>
      
      <View style={styles.item}>
        <View style={styles.itemText}>
          <Text style={styles.itemTitle}>显示在线状态</Text>
          <Text style={styles.itemDesc}>关闭后，他人无法看到您的在线状态</Text>
        </View>
        <Switch
          value={settings.showOnlineStatus}
          onValueChange={(v) => handleToggle('showOnlineStatus', v)}
          disabled={isLoading}
        />
      </View>
      
      <View style={styles.item}>
        <View style={styles.itemText}>
          <Text style={styles.itemTitle}>显示最后活跃时间</Text>
          <Text style={styles.itemDesc}>关闭后，他人无法看到您的最后活跃时间</Text>
        </View>
        <Switch
          value={settings.showLastActive}
          onValueChange={(v) => handleToggle('showLastActive', v)}
          disabled={isLoading}
        />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#fff',
    padding: 16,
  },
  title: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 16,
  },
  item: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 12,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#e0e0e0',
  },
  itemText: {
    flex: 1,
  },
  itemTitle: {
    fontSize: 16,
    color: '#333',
  },
  itemDesc: {
    fontSize: 13,
    color: '#999',
    marginTop: 2,
  },
});
```

### 用户搜索页面

```tsx
// frontend/src/features/chat/screens/UserSearchScreen.tsx

import React, { useState, useCallback } from 'react';
import {
  View,
  Text,
  TextInput,
  StyleSheet,
  TouchableOpacity,
  ActivityIndicator,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { useUserStore } from '@/stores/user.store';
import { useChatStore } from '@/stores/chat.store';
import { UserProfileCard } from '../components/UserProfileCard';
import type { ChatUserProfile, ChatUserId } from '../types';

export function UserSearchScreen() {
  const router = useRouter();
  const { searchUser } = useUserStore();
  const { startNewChat } = useChatStore();
  
  const [searchId, setSearchId] = useState('');
  const [isSearching, setIsSearching] = useState(false);
  const [result, setResult] = useState<ChatUserProfile | null>(null);
  const [notFound, setNotFound] = useState(false);
  
  const handleSearch = useCallback(async () => {
    const id = parseInt(searchId, 10);
    
    // 验证 11 位数字
    if (isNaN(id) || id < 10_000_000_000 || id > 99_999_999_999) {
      Alert.alert('提示', '请输入有效的 11 位聊天 ID');
      return;
    }
    
    setIsSearching(true);
    setNotFound(false);
    setResult(null);
    
    try {
      const profile = await searchUser(id as ChatUserId);
      if (profile) {
        setResult(profile);
      } else {
        setNotFound(true);
      }
    } finally {
      setIsSearching(false);
    }
  }, [searchId, searchUser]);
  
  const handleStartChat = useCallback(async () => {
    if (!result) return;
    
    try {
      await startNewChat(result.accountId);
      router.push(`/chat/${result.accountId}`);
    } catch (error) {
      Alert.alert('错误', '无法开始聊天');
    }
  }, [result, startNewChat, router]);
  
  return (
    <View style={styles.container}>
      <View style={styles.searchBox}>
        <TextInput
          style={styles.input}
          value={searchId}
          onChangeText={setSearchId}
          placeholder="输入 11 位聊天 ID"
          keyboardType="number-pad"
          maxLength={11}
        />
        <TouchableOpacity
          style={styles.searchBtn}
          onPress={handleSearch}
          disabled={isSearching || searchId.length !== 11}
        >
          {isSearching ? (
            <ActivityIndicator color="#fff" size="small" />
          ) : (
            <Text style={styles.searchBtnText}>搜索</Text>
          )}
        </TouchableOpacity>
      </View>
      
      {result && (
        <View style={styles.resultContainer}>
          <UserProfileCard profile={result} />
          <TouchableOpacity style={styles.chatBtn} onPress={handleStartChat}>
            <Text style={styles.chatBtnText}>发起聊天</Text>
          </TouchableOpacity>
        </View>
      )}
      
      {notFound && (
        <View style={styles.notFound}>
          <Text style={styles.notFoundText}>未找到该用户</Text>
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  searchBox: {
    flexDirection: 'row',
    padding: 16,
    backgroundColor: '#fff',
  },
  input: {
    flex: 1,
    height: 44,
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 16,
    fontSize: 16,
  },
  searchBtn: {
    marginLeft: 12,
    backgroundColor: '#007AFF',
    paddingHorizontal: 20,
    borderRadius: 8,
    justifyContent: 'center',
  },
  searchBtnText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '500',
  },
  resultContainer: {
    marginTop: 16,
    backgroundColor: '#fff',
  },
  chatBtn: {
    margin: 16,
    backgroundColor: '#007AFF',
    height: 44,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  chatBtnText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  notFound: {
    alignItems: 'center',
    marginTop: 40,
  },
  notFoundText: {
    fontSize: 16,
    color: '#999',
  },
});
```

## 聊天服务

```typescript
// frontend/src/services/chat.service.ts

import { ApiPromise } from '@polkadot/api';
import { getApi } from '@/api';
import { deriveSharedKey, encryptMessage, decryptMessage } from './crypto.service';
import { uploadToIpfs, downloadFromIpfs } from './ipfs.service';
import type { Message, Session, MessageType } from '@/features/chat/types';

export class ChatService {
  private api: ApiPromise | null = null;
  public myAddress: string;
  private sharedKeys: Map<string, Uint8Array> = new Map();  // peerAddress -> AES key
  private unsubscribe: (() => void) | null = null;

  constructor(myAddress: string) {
    this.myAddress = myAddress;
  }

  async init(): Promise<void> {
    this.api = await getApi();
  }
  
  /**
   * 发送消息
   */
  async sendMessage(
    receiver: string,
    content: string,
    msgType: number = 0,
    sessionId?: string
  ): Promise<{ msgId: number; sessionId: string }> {
    if (!this.api) throw new Error('API not initialized');
    
    // 1. 获取或派生共享密钥
    const sharedKey = await this.getSharedKey(receiver);
    
    // 2. 加密消息
    const encrypted = await encryptMessage(content, sharedKey);
    
    // 3. 上传到 IPFS
    const cid = await uploadToIpfs(encrypted);
    
    // 4. 发送链上交易
    const tx = this.api.tx.chat.sendMessage(
      receiver,
      cid,
      msgType,
      sessionId || null
    );
    
    return new Promise((resolve, reject) => {
      tx.signAndSend(this.myAddress, ({ status, events }) => {
        if (status.isInBlock) {
          for (const { event } of events) {
            if (this.api!.events.chat.MessageSent.is(event)) {
              const [msgId, sessionId] = event.data;
              resolve({
                msgId: msgId.toNumber(),
                sessionId: sessionId.toHex()
              });
              return;
            }
          }
        }
        if (status.isFinalized) {
          reject(new Error('Transaction finalized without MessageSent event'));
        }
      }).catch(reject);
    });
  }
  
  /**
   * 获取会话列表
   */
  async getSessions(): Promise<Session[]> {
    if (!this.api) throw new Error('API not initialized');
    
    const entries = await this.api.query.chat.userSessions.entries(this.myAddress);
    const sessions: Session[] = [];
    
    for (const [key] of entries) {
      const sessionId = key.args[1].toHex();
      const session = await this.getSessionDetail(sessionId);
      if (session) sessions.push(session);
    }
    
    // 按最后活跃时间排序
    return sessions.sort((a, b) => b.lastActive - a.lastActive);
  }
  
  /**
   * 获取会话详情
   */
  async getSessionDetail(sessionId: string): Promise<Session | null> {
    if (!this.api) return null;
    
    const sessionData = await this.api.query.chat.sessions(sessionId);
    if (sessionData.isNone) return null;
    
    const session = sessionData.unwrap();
    const participants = session.participants.map((p: any) => p.toString());
    const peerAddress = participants.find((p: string) => p !== this.myAddress) || '';
    
    // 获取未读数
    const unreadCount = await this.api.query.chat.unreadCount([this.myAddress, sessionId]);
    
    return {
      id: sessionId,
      participants,
      peerAddress,
      lastActive: session.lastActive.toNumber(),
      unreadCount: unreadCount.toNumber(),
      isArchived: session.isArchived.valueOf(),
    };
  }
  
  /**
   * 获取会话消息（分页）
   */
  async getMessages(
    sessionId: string,
    offset: number = 0,
    limit: number = 20
  ): Promise<Message[]> {
    if (!this.api) throw new Error('API not initialized');
    
    const entries = await this.api.query.chat.sessionMessages.entries(sessionId);
    const msgIds = entries.map(([key]) => key.args[1].toNumber());
    
    // 按 ID 倒序排列（最新的在前）
    msgIds.sort((a, b) => b - a);
    
    const messages: Message[] = [];
    const targetIds = msgIds.slice(offset, offset + limit);
    
    for (const msgId of targetIds) {
      const msg = await this.getMessageDetail(msgId);
      if (msg) messages.push(msg);
    }
    
    // 返回时按时间正序（旧的在前）
    return messages.reverse();
  }
  
  /**
   * 获取消息详情并解密
   */
  async getMessageDetail(msgId: number): Promise<Message | null> {
    if (!this.api) return null;
    
    const msgData = await this.api.query.chat.messages(msgId);
    if (msgData.isNone) return null;
    
    const msg = msgData.unwrap();
    const sender = msg.sender.toString();
    const receiver = msg.receiver.toString();
    const isMine = sender === this.myAddress;
    const peerAddress = isMine ? receiver : sender;
    
    // 检查是否被删除
    if (isMine && msg.isDeletedBySender.valueOf()) return null;
    if (!isMine && msg.isDeletedByReceiver.valueOf()) return null;
    
    // 解密消息内容
    const cid = msg.contentCid.toUtf8();
    let content = '';
    
    try {
      const encrypted = await downloadFromIpfs(cid);
      const sharedKey = await this.getSharedKey(peerAddress);
      content = await decryptMessage(encrypted, sharedKey);
    } catch (error) {
      console.error('Failed to decrypt message:', error);
      content = '[无法解密]';
    }
    
    return {
      id: msgId,
      sessionId: msg.sessionId.toHex(),
      sender,
      receiver,
      content,
      contentCid: cid,
      msgType: msg.msgType.toNumber() as MessageType,
      sentAt: msg.sentAt.toNumber(),
      isRead: msg.isRead.valueOf(),
      isDeletedBySender: msg.isDeletedBySender.valueOf(),
      isDeletedByReceiver: msg.isDeletedByReceiver.valueOf(),
      isMine,
      status: 'sent',
    };
  }
  
  /**
   * 标记消息已读
   */
  async markAsRead(messageIds: number[]): Promise<void> {
    if (!this.api || messageIds.length === 0) return;
    
    const tx = this.api.tx.chat.markBatchAsRead(messageIds);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 标记整个会话已读
   */
  async markSessionAsRead(sessionId: string): Promise<void> {
    if (!this.api) return;
    
    const tx = this.api.tx.chat.markSessionAsRead(sessionId);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 删除消息（软删除）
   */
  async deleteMessage(msgId: number): Promise<void> {
    if (!this.api) return;
    
    const tx = this.api.tx.chat.deleteMessage(msgId);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 拉黑用户
   */
  async blockUser(address: string): Promise<void> {
    if (!this.api) return;
    
    const tx = this.api.tx.chat.blockUser(address);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 解除拉黑
   */
  async unblockUser(address: string): Promise<void> {
    if (!this.api) return;
    
    const tx = this.api.tx.chat.unblockUser(address);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 检查是否被拉黑
   */
  async isBlocked(address: string): Promise<boolean> {
    if (!this.api) return false;
    
    const result = await this.api.query.chat.blacklist(address, this.myAddress);
    return result.isSome;
  }
  
  /**
   * 监听新消息事件
   */
  subscribeMessages(callback: (msg: Message) => void): () => void {
    if (!this.api) return () => {};
    
    const unsub = this.api.query.system.events((events: any[]) => {
      for (const { event } of events) {
        if (this.api!.events.chat.MessageSent.is(event)) {
          const [msgId, , , receiver] = event.data;
          
          // 只处理发给自己的消息
          if (receiver.toString() === this.myAddress) {
            this.getMessageDetail(msgId.toNumber()).then(msg => {
              if (msg) callback(msg);
            });
          }
        }
      }
    });
    
    this.unsubscribe = unsub as any;
    return () => {
      if (this.unsubscribe) {
        this.unsubscribe();
        this.unsubscribe = null;
      }
    };
  }
  
  /**
   * 获取或派生共享密钥
   * 使用 x25519 ECDH 进行密钥交换
   */
  private async getSharedKey(peerAddress: string): Promise<Uint8Array> {
    if (!this.sharedKeys.has(peerAddress)) {
      // 1. 获取自己的 x25519 私钥（从安全存储）
      const myPrivateKey = await this.getMyX25519PrivateKey();

      // 2. 获取对方的 x25519 公钥（从链上或缓存）
      const peerPublicKey = await this.getPeerX25519PublicKey(peerAddress);

      // 3. 使用 x25519 ECDH 派生共享密钥
      const key = await deriveSharedKey(myPrivateKey, peerPublicKey);
      this.sharedKeys.set(peerAddress, key);
    }
    return this.sharedKeys.get(peerAddress)!;
  }

  /**
   * 获取自己的 x25519 私钥
   * 从安全存储中读取，如果不存在则生成新的
   */
  private async getMyX25519PrivateKey(): Promise<Uint8Array> {
    // TODO: 从 expo-secure-store 读取
    // 如果不存在，调用 generateX25519KeyPair() 生成并存储
    throw new Error('Not implemented');
  }

  /**
   * 获取对方的 x25519 公钥
   * 从链上查询或本地缓存
   */
  private async getPeerX25519PublicKey(peerAddress: string): Promise<Uint8Array> {
    // TODO: 从链上 ChatUserProfiles 查询对方的公钥
    // 或从本地缓存读取
    throw new Error('Not implemented');
  }
  
  /**
   * 清理资源
   */
  destroy(): void {
    if (this.unsubscribe) {
      this.unsubscribe();
    }
    this.sharedKeys.clear();
  }
}

// 单例
let chatServiceInstance: ChatService | null = null;

export function getChatService(): ChatService {
  if (!chatServiceInstance) {
    throw new Error('ChatService not initialized');
  }
  return chatServiceInstance;
}

export function initChatService(myAddress: string): ChatService {
  chatServiceInstance = new ChatService(myAddress);
  return chatServiceInstance;
}
```


## UI 组件

### 会话列表项

```tsx
// frontend/src/features/chat/components/SessionItem.tsx

import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import type { Session } from '../types';

interface Props {
  session: Session;
  onPress: () => void;
}

/**
 * 区块时间工具 - 需要当前区块高度来计算相对时间
 */
import { useBlockTime } from '@/hooks/useBlockTime';

/**
 * 将区块高度转换为相对时间显示
 * @param targetBlock 目标区块高度
 * @param currentBlock 当前区块高度
 * @param blockTime 出块时间（毫秒），默认 6000ms
 */
function formatBlockTime(
  targetBlock: number,
  currentBlock: number,
  blockTime: number = 6000
): string {
  const blockDiff = currentBlock - targetBlock;
  const timeDiff = blockDiff * blockTime; // 毫秒
  
  if (timeDiff < 0) return '刚刚'; // 未来区块，显示为刚刚
  if (timeDiff < 60000) return '刚刚';
  if (timeDiff < 3600000) return `${Math.floor(timeDiff / 60000)}分钟前`;
  if (timeDiff < 86400000) return `${Math.floor(timeDiff / 3600000)}小时前`;
  if (timeDiff < 604800000) return `${Math.floor(timeDiff / 86400000)}天前`;
  
  // 超过一周，显示具体日期
  const date = new Date(Date.now() - timeDiff);
  return `${date.getMonth() + 1}/${date.getDate()}`;
}

/**
 * 将区块高度转换为具体时间
 * @param targetBlock 目标区块高度
 * @param currentBlock 当前区块高度
 * @param blockTime 出块时间（毫秒）
 */
function blockToDate(
  targetBlock: number,
  currentBlock: number,
  blockTime: number = 6000
): Date {
  const blockDiff = currentBlock - targetBlock;
  const timeDiff = blockDiff * blockTime;
  return new Date(Date.now() - timeDiff);
}

/**
 * 格式化消息时间（用于聊天气泡）
 */
function formatMessageTime(
  targetBlock: number,
  currentBlock: number,
  blockTime: number = 6000
): string {
  const date = blockToDate(targetBlock, currentBlock, blockTime);
  const hours = date.getHours().toString().padStart(2, '0');
  const minutes = date.getMinutes().toString().padStart(2, '0');
  return `${hours}:${minutes}`;
}

export function SessionItem({ session, onPress }: Props) {
  // 获取当前区块高度和出块时间
  const { currentBlock, blockTime } = useBlockTime();
  
  return (
    <TouchableOpacity style={styles.container} onPress={onPress}>
      <View style={styles.avatar}>
        <Text style={styles.avatarText}>
          {session.peerAlias?.[0] || session.peerAddress.slice(0, 2).toUpperCase()}
        </Text>
      </View>
      
      <View style={styles.content}>
        <View style={styles.header}>
          <Text style={styles.name} numberOfLines={1}>
            {session.peerAlias || `${session.peerAddress.slice(0, 8)}...${session.peerAddress.slice(-4)}`}
          </Text>
          <Text style={styles.time}>
            {formatBlockTime(session.lastActive, currentBlock, blockTime)}
          </Text>
        </View>
        
        <Text style={styles.preview} numberOfLines={1}>
          {session.lastMessage?.content || '暂无消息'}
        </Text>
      </View>
      
      {session.unreadCount > 0 && (
        <View style={styles.badge}>
          <Text style={styles.badgeText}>
            {session.unreadCount > 99 ? '99+' : session.unreadCount}
          </Text>
        </View>
      )}
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
    backgroundColor: '#fff',
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#e0e0e0',
  },
  avatar: {
    width: 48,
    height: 48,
    borderRadius: 24,
    backgroundColor: '#007AFF',
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  avatarText: {
    color: '#fff',
    fontSize: 18,
    fontWeight: '600',
  },
  content: {
    flex: 1,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 4,
  },
  name: {
    fontSize: 16,
    fontWeight: '500',
    color: '#333',
    flex: 1,
    marginRight: 8,
  },
  time: {
    fontSize: 12,
    color: '#999',
  },
  preview: {
    fontSize: 14,
    color: '#666',
  },
  badge: {
    minWidth: 20,
    height: 20,
    borderRadius: 10,
    backgroundColor: '#FF3B30',
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 6,
    marginLeft: 8,
  },
  badgeText: {
    color: '#fff',
    fontSize: 12,
    fontWeight: '600',
  },
});
```

### 消息气泡

```tsx
// frontend/src/features/chat/components/ChatBubble.tsx

import React from 'react';
import { View, Text, Image, StyleSheet } from 'react-native';
import { MessageType, type Message } from '../types';
import { useBlockTime } from '@/hooks/useBlockTime';

interface Props {
  message: Message;
}

export function ChatBubble({ message }: Props) {
  const isMine = message.isMine;
  const { currentBlock, blockTime } = useBlockTime();
  
  // 计算消息时间
  const messageTime = React.useMemo(() => {
    const blockDiff = currentBlock - message.sentAt;
    const timeDiff = blockDiff * blockTime;
    const date = new Date(Date.now() - timeDiff);
    return `${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
  }, [message.sentAt, currentBlock, blockTime]);
  
  const renderContent = () => {
    switch (message.msgType) {
      case MessageType.Text:
        return (
          <Text style={[styles.text, isMine && styles.textMine]}>
            {message.content}
          </Text>
        );
      
      case MessageType.Image:
        return (
          <Image
            source={{ uri: message.content }}
            style={styles.image}
            resizeMode="cover"
          />
        );
      
      case MessageType.Voice:
        return (
          <View style={styles.voice}>
            <Text style={[styles.text, isMine && styles.textMine]}>
              🎤 语音消息
            </Text>
          </View>
        );
      
      case MessageType.File:
        return (
          <View style={styles.file}>
            <Text style={[styles.text, isMine && styles.textMine]}>
              📎 文件
            </Text>
          </View>
        );
      
      case MessageType.System:
        return (
          <Text style={styles.systemText}>
            {message.content}
          </Text>
        );
      
      default:
        return (
          <Text style={[styles.text, isMine && styles.textMine]}>
            {message.content}
          </Text>
        );
    }
  };
  
  // 系统消息居中显示
  if (message.msgType === MessageType.System) {
    return (
      <View style={styles.systemContainer}>
        {renderContent()}
      </View>
    );
  }
  
  return (
    <View style={[styles.container, isMine ? styles.mine : styles.theirs]}>
      <View style={[styles.bubble, isMine ? styles.bubbleMine : styles.bubbleTheirs]}>
        {renderContent()}
      </View>
      
      <View style={[styles.meta, isMine && styles.metaMine]}>
        <Text style={styles.time}>{messageTime}</Text>
        {isMine && (
          <Text style={styles.status}>
            {message.status === 'sending' ? '发送中...' : message.isRead ? '已读' : '已发送'}
          </Text>
        )}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    marginVertical: 4,
    marginHorizontal: 12,
    maxWidth: '80%',
  },
  mine: {
    alignSelf: 'flex-end',
  },
  theirs: {
    alignSelf: 'flex-start',
  },
  bubble: {
    padding: 10,
    borderRadius: 16,
  },
  bubbleMine: {
    backgroundColor: '#007AFF',
    borderBottomRightRadius: 4,
  },
  bubbleTheirs: {
    backgroundColor: '#E9E9EB',
    borderBottomLeftRadius: 4,
  },
  text: {
    fontSize: 16,
    color: '#333',
    lineHeight: 22,
  },
  textMine: {
    color: '#fff',
  },
  image: {
    width: 200,
    height: 150,
    borderRadius: 8,
  },
  voice: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  file: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  meta: {
    flexDirection: 'row',
    marginTop: 4,
    gap: 8,
  },
  metaMine: {
    justifyContent: 'flex-end',
  },
  time: {
    fontSize: 11,
    color: '#999',
  },
  status: {
    fontSize: 11,
    color: '#999',
  },
  systemContainer: {
    alignItems: 'center',
    marginVertical: 8,
  },
  systemText: {
    fontSize: 12,
    color: '#999',
    backgroundColor: 'rgba(0,0,0,0.05)',
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 12,
  },
});
```


### 输入框组件

```tsx
// frontend/src/features/chat/components/ChatInput.tsx

import React, { useState } from 'react';
import { View, TextInput, TouchableOpacity, StyleSheet, Platform } from 'react-native';
import { Ionicons } from '@expo/vector-icons';

interface Props {
  onSend: (content: string) => void;
  onAttach?: () => void;
  disabled?: boolean;
  placeholder?: string;
}

export function ChatInput({ onSend, onAttach, disabled, placeholder = '输入消息...' }: Props) {
  const [text, setText] = useState('');
  
  const handleSend = () => {
    const trimmed = text.trim();
    if (trimmed) {
      onSend(trimmed);
      setText('');
    }
  };
  
  const canSend = text.trim().length > 0 && !disabled;
  
  return (
    <View style={styles.container}>
      {onAttach && (
        <TouchableOpacity style={styles.iconBtn} onPress={onAttach} disabled={disabled}>
          <Ionicons name="add-circle-outline" size={28} color={disabled ? '#ccc' : '#666'} />
        </TouchableOpacity>
      )}
      
      <View style={styles.inputWrapper}>
        <TextInput
          style={styles.input}
          value={text}
          onChangeText={setText}
          placeholder={placeholder}
          placeholderTextColor="#999"
          multiline
          maxLength={1000}
          editable={!disabled}
        />
      </View>
      
      <TouchableOpacity
        style={[styles.sendBtn, !canSend && styles.sendBtnDisabled]}
        onPress={handleSend}
        disabled={!canSend}
      >
        <Ionicons name="send" size={24} color={canSend ? '#007AFF' : '#ccc'} />
      </TouchableOpacity>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    paddingHorizontal: 8,
    paddingVertical: 8,
    backgroundColor: '#fff',
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: '#e0e0e0',
  },
  iconBtn: {
    padding: 8,
  },
  inputWrapper: {
    flex: 1,
    backgroundColor: '#f5f5f5',
    borderRadius: 20,
    paddingHorizontal: 16,
    paddingVertical: Platform.OS === 'ios' ? 10 : 6,
    marginHorizontal: 8,
    maxHeight: 120,
  },
  input: {
    fontSize: 16,
    color: '#333',
    maxHeight: 100,
  },
  sendBtn: {
    padding: 8,
  },
  sendBtnDisabled: {
    opacity: 0.5,
  },
});
```

### 消息列表

```tsx
// frontend/src/features/chat/components/MessageList.tsx

import React, { useRef, useCallback } from 'react';
import { FlatList, View, StyleSheet, ActivityIndicator } from 'react-native';
import { ChatBubble } from './ChatBubble';
import type { Message } from '../types';

interface Props {
  messages: Message[];
  isLoading?: boolean;
  onLoadMore?: () => void;
  hasMore?: boolean;
}

export function MessageList({ messages, isLoading, onLoadMore, hasMore }: Props) {
  const listRef = useRef<FlatList>(null);
  
  const renderItem = useCallback(({ item }: { item: Message }) => (
    <ChatBubble message={item} />
  ), []);
  
  const keyExtractor = useCallback((item: Message) => item.id.toString(), []);
  
  const renderFooter = () => {
    if (!isLoading) return null;
    return (
      <View style={styles.loading}>
        <ActivityIndicator size="small" color="#007AFF" />
      </View>
    );
  };
  
  return (
    <FlatList
      ref={listRef}
      data={messages}
      renderItem={renderItem}
      keyExtractor={keyExtractor}
      contentContainerStyle={styles.content}
      inverted={false}
      onEndReached={hasMore ? onLoadMore : undefined}
      onEndReachedThreshold={0.1}
      ListHeaderComponent={renderFooter}
      showsVerticalScrollIndicator={false}
      maintainVisibleContentPosition={{
        minIndexForVisible: 0,
      }}
    />
  );
}

const styles = StyleSheet.create({
  content: {
    paddingVertical: 8,
  },
  loading: {
    padding: 16,
    alignItems: 'center',
  },
});
```


## 页面组件

### 会话列表页

```tsx
// frontend/src/features/chat/screens/SessionListScreen.tsx

import React, { useEffect, useCallback } from 'react';
import { View, FlatList, StyleSheet, Text, RefreshControl } from 'react-native';
import { useRouter } from 'expo-router';
import { useChatStore } from '@/stores/chat.store';
import { SessionItem } from '../components/SessionItem';
import type { Session } from '../types';

export function SessionListScreen() {
  const router = useRouter();
  const { sessions, isLoading, totalUnread, loadSessions, selectSession } = useChatStore();
  
  useEffect(() => {
    loadSessions();
  }, []);
  
  const handleSessionPress = useCallback((session: Session) => {
    selectSession(session.id);
    router.push(`/chat/${session.id}`);
  }, [selectSession, router]);
  
  const renderItem = useCallback(({ item }: { item: Session }) => (
    <SessionItem session={item} onPress={() => handleSessionPress(item)} />
  ), [handleSessionPress]);
  
  const keyExtractor = useCallback((item: Session) => item.id, []);
  
  const renderEmpty = () => (
    <View style={styles.empty}>
      <Text style={styles.emptyText}>暂无会话</Text>
      <Text style={styles.emptySubtext}>开始一段新的对话吧</Text>
    </View>
  );
  
  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>消息</Text>
        {totalUnread > 0 && (
          <View style={styles.totalBadge}>
            <Text style={styles.totalBadgeText}>{totalUnread}</Text>
          </View>
        )}
      </View>
      
      <FlatList
        data={sessions}
        renderItem={renderItem}
        keyExtractor={keyExtractor}
        ListEmptyComponent={renderEmpty}
        refreshControl={
          <RefreshControl refreshing={isLoading} onRefresh={loadSessions} />
        }
        contentContainerStyle={sessions.length === 0 && styles.emptyContainer}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#e0e0e0',
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    color: '#333',
  },
  totalBadge: {
    marginLeft: 8,
    backgroundColor: '#FF3B30',
    borderRadius: 10,
    paddingHorizontal: 8,
    paddingVertical: 2,
  },
  totalBadgeText: {
    color: '#fff',
    fontSize: 12,
    fontWeight: '600',
  },
  emptyContainer: {
    flex: 1,
  },
  empty: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingVertical: 60,
  },
  emptyText: {
    fontSize: 18,
    color: '#333',
    marginBottom: 8,
  },
  emptySubtext: {
    fontSize: 14,
    color: '#999',
  },
});
```

### 聊天详情页

```tsx
// frontend/src/features/chat/screens/ChatScreen.tsx

import React, { useEffect, useCallback, useState } from 'react';
import { View, StyleSheet, KeyboardAvoidingView, Platform } from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useChatStore } from '@/stores/chat.store';
import { ChatHeader } from '../components/ChatHeader';
import { MessageList } from '../components/MessageList';
import { ChatInput } from '../components/ChatInput';

export function ChatScreen() {
  const { sessionId } = useLocalSearchParams<{ sessionId: string }>();
  const router = useRouter();
  
  const {
    currentSession,
    messages,
    isLoading,
    selectSession,
    loadMessages,
    sendMessage,
    markSessionAsRead,
  } = useChatStore();
  
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  
  const sessionMessages = messages[sessionId || ''] || [];
  
  useEffect(() => {
    if (sessionId) {
      selectSession(sessionId);
      loadMessages(sessionId);
      markSessionAsRead(sessionId);
    }
  }, [sessionId]);
  
  const handleSend = useCallback(async (content: string) => {
    if (!currentSession) return;
    
    try {
      await sendMessage(currentSession.peerAddress, content);
    } catch (error) {
      console.error('Send message failed:', error);
      // TODO: 显示错误提示
    }
  }, [currentSession, sendMessage]);
  
  const handleLoadMore = useCallback(async () => {
    if (!sessionId || isLoading || !hasMore) return;
    
    const newOffset = offset + 20;
    const newMessages = await loadMessages(sessionId, newOffset);
    
    if (newMessages.length < 20) {
      setHasMore(false);
    }
    setOffset(newOffset);
  }, [sessionId, offset, isLoading, hasMore, loadMessages]);
  
  const handleBack = useCallback(() => {
    router.back();
  }, [router]);
  
  const handleMore = useCallback(() => {
    // TODO: 显示更多操作菜单（拉黑、清空记录等）
  }, []);
  
  if (!currentSession) {
    return null;
  }
  
  return (
    <KeyboardAvoidingView
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}
      keyboardVerticalOffset={Platform.OS === 'ios' ? 88 : 0}
    >
      <ChatHeader
        title={currentSession.peerAlias || currentSession.peerAddress}
        onBack={handleBack}
        onMore={handleMore}
      />
      
      <View style={styles.content}>
        <MessageList
          messages={sessionMessages}
          isLoading={isLoading}
          onLoadMore={handleLoadMore}
          hasMore={hasMore}
        />
      </View>
      
      <ChatInput onSend={handleSend} />
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  content: {
    flex: 1,
  },
});
```


## Hooks

### useBlockTime Hook

```typescript
// frontend/src/hooks/useBlockTime.ts

import { useState, useEffect } from 'react';
import { getApi } from '@/api';

interface BlockTimeState {
  /** 当前区块高度 */
  currentBlock: number;
  /** 出块时间（毫秒） */
  blockTime: number;
  /** 是否已加载 */
  isLoaded: boolean;
}

/**
 * 获取当前区块高度和出块时间
 * 用于将区块高度转换为实际时间
 */
export function useBlockTime(): BlockTimeState {
  const [state, setState] = useState<BlockTimeState>({
    currentBlock: 0,
    blockTime: 6000, // 默认 6 秒
    isLoaded: false,
  });
  
  useEffect(() => {
    let unsubscribe: (() => void) | null = null;
    
    const init = async () => {
      try {
        const api = await getApi();
        
        // 获取出块时间（从链配置）
        const expectedBlockTime = api.consts.babe?.expectedBlockTime
          || api.consts.timestamp?.minimumPeriod?.muln(2);
        const blockTime = expectedBlockTime
          ? expectedBlockTime.toNumber()
          : 6000;
        
        // 订阅最新区块
        unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
          setState({
            currentBlock: header.number.toNumber(),
            blockTime,
            isLoaded: true,
          });
        });
      } catch (error) {
        console.error('Failed to subscribe to block headers:', error);
        // 使用估算值
        setState(prev => ({ ...prev, isLoaded: true }));
      }
    };
    
    init();
    
    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }, []);
  
  return state;
}

/**
 * 区块时间工具函数
 */
export const BlockTimeUtils = {
  /**
   * 将区块高度差转换为毫秒
   */
  blocksToMs(blocks: number, blockTime: number = 6000): number {
    return blocks * blockTime;
  },
  
  /**
   * 将毫秒转换为区块数
   */
  msToBlocks(ms: number, blockTime: number = 6000): number {
    return Math.floor(ms / blockTime);
  },
  
  /**
   * 计算目标区块对应的时间戳
   */
  blockToTimestamp(
    targetBlock: number,
    currentBlock: number,
    blockTime: number = 6000
  ): number {
    const blockDiff = currentBlock - targetBlock;
    return Date.now() - (blockDiff * blockTime);
  },
  
  /**
   * 格式化区块时间为相对时间
   */
  formatRelative(
    targetBlock: number,
    currentBlock: number,
    blockTime: number = 6000
  ): string {
    const timeDiff = (currentBlock - targetBlock) * blockTime;
    
    if (timeDiff < 0) return '刚刚';
    if (timeDiff < 60000) return '刚刚';
    if (timeDiff < 3600000) return `${Math.floor(timeDiff / 60000)}分钟前`;
    if (timeDiff < 86400000) return `${Math.floor(timeDiff / 3600000)}小时前`;
    if (timeDiff < 604800000) return `${Math.floor(timeDiff / 86400000)}天前`;
    
    const date = new Date(Date.now() - timeDiff);
    return `${date.getMonth() + 1}/${date.getDate()}`;
  },
  
  /**
   * 格式化区块时间为 HH:mm
   */
  formatTime(
    targetBlock: number,
    currentBlock: number,
    blockTime: number = 6000
  ): string {
    const timestamp = BlockTimeUtils.blockToTimestamp(targetBlock, currentBlock, blockTime);
    const date = new Date(timestamp);
    return `${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
  },
  
  /**
   * 格式化区块时间为完整日期时间
   */
  formatDateTime(
    targetBlock: number,
    currentBlock: number,
    blockTime: number = 6000
  ): string {
    const timestamp = BlockTimeUtils.blockToTimestamp(targetBlock, currentBlock, blockTime);
    const date = new Date(timestamp);
    return `${date.getFullYear()}/${date.getMonth() + 1}/${date.getDate()} ${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
  },
};
```

### useChat Hook

```typescript
// frontend/src/features/chat/hooks/useChat.ts

import { useEffect, useCallback } from 'react';
import { useChatStore } from '@/stores/chat.store';
import { useWalletStore } from '@/stores/wallet.store';
import { initChatService, getChatService } from '@/services/chat.service';

export function useChat() {
  const { address, isLocked } = useWalletStore();
  const {
    sessions,
    currentSession,
    messages,
    totalUnread,
    isLoading,
    loadSessions,
    selectSession,
    sendMessage,
    markAsRead,
    markSessionAsRead,
    deleteMessage,
    blockUser,
    unblockUser,
    handleNewMessage,
  } = useChatStore();
  
  // 初始化聊天服务
  useEffect(() => {
    if (address && !isLocked) {
      const service = initChatService(address);
      service.init().then(() => {
        loadSessions();
        
        // 订阅新消息
        const unsub = service.subscribeMessages(handleNewMessage);
        return unsub;
      });
    }
  }, [address, isLocked]);
  
  // 发送文本消息
  const sendTextMessage = useCallback(async (receiver: string, content: string) => {
    await sendMessage(receiver, content, 0);
  }, [sendMessage]);
  
  // 发送图片消息
  const sendImageMessage = useCallback(async (receiver: string, imageUri: string) => {
    await sendMessage(receiver, imageUri, 1);
  }, [sendMessage]);
  
  // 开始新会话
  const startNewChat = useCallback(async (peerAddress: string) => {
    // 检查是否已有会话
    const existingSession = sessions.find(s => s.peerAddress === peerAddress);
    if (existingSession) {
      selectSession(existingSession.id);
      return existingSession;
    }
    
    // 发送一条消息来创建会话
    // 或者直接返回 null，让用户发送第一条消息时自动创建
    return null;
  }, [sessions, selectSession]);
  
  return {
    // 状态
    sessions,
    currentSession,
    messages,
    totalUnread,
    isLoading,
    
    // 方法
    loadSessions,
    selectSession,
    sendTextMessage,
    sendImageMessage,
    startNewChat,
    markAsRead,
    markSessionAsRead,
    deleteMessage,
    blockUser,
    unblockUser,
  };
}
```

### useChatEvents Hook

```typescript
// frontend/src/features/chat/hooks/useChatEvents.ts

import { useEffect, useRef, useCallback } from 'react';
import { AppState, AppStateStatus } from 'react-native';
import { getApi } from '@/api';
import { useChatStore } from '@/stores/chat.store';
import { useUserStore } from '@/stores/user.store';
import { getChatService } from '@/services/chat.service';
import { getUserService } from '@/services/user.service';
import type { ChatUserId, Message, Session } from '../types';

/**
 * 链上事件类型定义
 */
interface ChatEvents {
  // 消息事件
  MessageSent: {
    msgId: number;
    sessionId: string;
    sender: string;
    receiver: string;
  };
  MessageSentWithChatId: {
    msgId: number;
    senderChatId: ChatUserId | null;
    receiverChatId: ChatUserId | null;
    contentCid: string;
  };
  MessageRead: {
    msgId: number;
    reader: string;
  };
  MessageDeleted: {
    msgId: number;
    deleter: string;
  };
  
  // 会话事件
  SessionCreated: {
    sessionId: string;
    participants: string[];
  };
  SessionMarkedAsRead: {
    sessionId: string;
    user: string;
  };
  SessionArchived: {
    sessionId: string;
    operator: string;
  };
  
  // 黑名单事件
  UserBlocked: {
    blocker: string;
    blocked: string;
  };
  UserUnblocked: {
    unblocker: string;
    unblocked: string;
  };
  
  // 用户事件
  ChatUserCreated: {
    accountId: string;
    chatUserId: ChatUserId;
  };
  ChatUserProfileUpdated: {
    chatUserId: ChatUserId;
  };
  ChatUserStatusChanged: {
    chatUserId: ChatUserId;
    newStatus: number;
  };
  PrivacySettingsUpdated: {
    chatUserId: ChatUserId;
  };
}

/**
 * 事件回调类型
 */
interface EventCallbacks {
  onNewMessage?: (message: Message) => void;
  onMessageRead?: (msgId: number, reader: string) => void;
  onMessageDeleted?: (msgId: number) => void;
  onSessionCreated?: (session: Session) => void;
  onUserBlocked?: (blocker: string, blocked: string) => void;
  onUserUnblocked?: (unblocker: string, unblocked: string) => void;
  onUserRegistered?: (accountId: string, chatUserId: ChatUserId) => void;
  onProfileUpdated?: (chatUserId: ChatUserId) => void;
  onStatusChanged?: (chatUserId: ChatUserId, status: number) => void;
}

/**
 * 完整的聊天事件监听 Hook
 * 监听所有后端事件并更新前端状态
 */
export function useChatEvents(callbacks?: EventCallbacks) {
  const {
    handleNewMessage,
    handleMessageRead,
    handleMessageDeleted,
    handleSessionCreated,
    handleUserBlocked,
    handleUserUnblocked,
    loadSessions,
  } = useChatStore();
  
  const { refresh: refreshUserProfile } = useUserStore();
  
  const appState = useRef(AppState.currentState);
  const myAddress = useRef<string | null>(null);
  
  // 获取当前用户地址
  useEffect(() => {
    try {
      const chatService = getChatService();
      myAddress.current = chatService.myAddress;
    } catch {
      // 服务未初始化
    }
  }, []);
  
  // 应用前后台切换监听
  useEffect(() => {
    const subscription = AppState.addEventListener('change', (nextAppState: AppStateStatus) => {
      if (appState.current.match(/inactive|background/) && nextAppState === 'active') {
        // 从后台回到前台，刷新数据
        loadSessions();
      }
      appState.current = nextAppState;
    });
    
    return () => subscription.remove();
  }, [loadSessions]);
  
  // 链上事件订阅
  useEffect(() => {
    let unsubscribe: (() => void) | null = null;
    
    const subscribeEvents = async () => {
      try {
        const api = await getApi();
        const chatService = getChatService();
        const userService = getUserService();
        
        unsubscribe = api.query.system.events((events: any[]) => {
          events.forEach(({ event }) => {
            const { section, method, data } = event;
            
            if (section !== 'chat') return;
            
            switch (method) {
              // ========== 消息事件 ==========
              
              case 'MessageSent': {
                const [msgId, sessionId, sender, receiver] = data;
                const receiverAddr = receiver.toString();
                
                // 只处理发给自己的消息
                if (receiverAddr === myAddress.current) {
                  chatService.getMessageDetail(msgId.toNumber()).then(msg => {
                    if (msg) {
                      handleNewMessage(msg);
                      callbacks?.onNewMessage?.(msg);
                    }
                  });
                }
                break;
              }
              
              case 'MessageSentWithChatId': {
                // 增强版消息事件，包含 ChatUserId
                const [msgId, senderChatId, receiverChatId, contentCid] = data;
                // 可用于显示发送方的 ChatUserId 而非地址
                console.log('[Event] MessageSentWithChatId:', {
                  msgId: msgId.toNumber(),
                  senderChatId: senderChatId.isSome ? senderChatId.unwrap().toNumber() : null,
                  receiverChatId: receiverChatId.isSome ? receiverChatId.unwrap().toNumber() : null,
                });
                break;
              }
              
              case 'MessageRead': {
                const [msgId, reader] = data;
                handleMessageRead(msgId.toNumber(), reader.toString());
                callbacks?.onMessageRead?.(msgId.toNumber(), reader.toString());
                break;
              }
              
              case 'MessageDeleted': {
                const [msgId, deleter] = data;
                handleMessageDeleted(msgId.toNumber());
                callbacks?.onMessageDeleted?.(msgId.toNumber());
                break;
              }
              
              // ========== 会话事件 ==========
              
              case 'SessionCreated': {
                const [sessionId, participants] = data;
                const participantAddrs = participants.map((p: any) => p.toString());
                
                // 如果自己是参与者，刷新会话列表
                if (participantAddrs.includes(myAddress.current)) {
                  chatService.getSessionDetail(sessionId.toHex()).then(session => {
                    if (session) {
                      handleSessionCreated(session);
                      callbacks?.onSessionCreated?.(session);
                    }
                  });
                }
                break;
              }
              
              case 'SessionMarkedAsRead': {
                const [sessionId, user] = data;
                // 如果是对方标记已读，更新消息状态
                if (user.toString() !== myAddress.current) {
                  // 刷新该会话的消息状态
                  loadSessions();
                }
                break;
              }
              
              case 'SessionArchived': {
                const [sessionId, operator] = data;
                if (operator.toString() === myAddress.current) {
                  loadSessions();
                }
                break;
              }
              
              // ========== 黑名单事件 ==========
              
              case 'UserBlocked': {
                const [blocker, blocked] = data;
                handleUserBlocked(blocker.toString(), blocked.toString());
                callbacks?.onUserBlocked?.(blocker.toString(), blocked.toString());
                break;
              }
              
              case 'UserUnblocked': {
                const [unblocker, unblocked] = data;
                handleUserUnblocked(unblocker.toString(), unblocked.toString());
                callbacks?.onUserUnblocked?.(unblocker.toString(), unblocked.toString());
                break;
              }
              
              // ========== 用户事件 ==========
              
              case 'ChatUserCreated': {
                const [accountId, chatUserId] = data;
                const addr = accountId.toString();
                const id = chatUserId.toNumber() as ChatUserId;
                
                // 如果是自己注册成功，刷新用户状态
                if (addr === myAddress.current) {
                  refreshUserProfile();
                }
                callbacks?.onUserRegistered?.(addr, id);
                break;
              }
              
              case 'ChatUserProfileUpdated': {
                const [chatUserId] = data;
                const id = chatUserId.toNumber() as ChatUserId;
                
                // 清除该用户的缓存，下次获取时会重新加载
                userService.clearCache();
                callbacks?.onProfileUpdated?.(id);
                break;
              }
              
              case 'ChatUserStatusChanged': {
                const [chatUserId, newStatus] = data;
                const id = chatUserId.toNumber() as ChatUserId;
                const status = newStatus.toNumber();
                
                callbacks?.onStatusChanged?.(id, status);
                break;
              }
              
              case 'PrivacySettingsUpdated': {
                const [chatUserId] = data;
                const id = chatUserId.toNumber() as ChatUserId;
                
                // 清除缓存
                userService.clearCache();
                break;
              }
              
              default:
                // 未知事件，忽略
                break;
            }
          });
        }) as any;
        
      } catch (error) {
        console.error('[useChatEvents] Failed to subscribe:', error);
      }
    };
    
    subscribeEvents();
    
    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }, [
    handleNewMessage,
    handleMessageRead,
    handleMessageDeleted,
    handleSessionCreated,
    handleUserBlocked,
    handleUserUnblocked,
    loadSessions,
    refreshUserProfile,
    callbacks,
  ]);
}

/**
 * 简化版事件监听 - 仅监听新消息
 */
export function useNewMessageListener(onMessage: (message: Message) => void) {
  useChatEvents({
    onNewMessage: onMessage,
  });
}

/**
 * 在线状态监听 Hook
 */
export function useOnlineStatusListener(
  chatUserIds: ChatUserId[],
  onStatusChange: (chatUserId: ChatUserId, status: number) => void
) {
  const statusMap = useRef(new Map<ChatUserId, number>());
  
  useChatEvents({
    onStatusChanged: (chatUserId, status) => {
      if (chatUserIds.includes(chatUserId)) {
        const prevStatus = statusMap.current.get(chatUserId);
        if (prevStatus !== status) {
          statusMap.current.set(chatUserId, status);
          onStatusChange(chatUserId, status);
        }
      }
    },
  });
  
  return statusMap.current;
}
```

## 聊天 Store 事件处理方法补充

```typescript
// 在 chat.store.ts 中添加以下方法

interface ChatState {
  // ... 现有状态
  blockedUsers: Set<string>;
  
  // ... 现有方法
  
  // 事件处理方法
  handleNewMessage: (message: Message) => void;
  handleMessageRead: (msgId: number, reader: string) => void;
  handleMessageDeleted: (msgId: number) => void;
  handleSessionCreated: (session: Session) => void;
  handleUserBlocked: (blocker: string, blocked: string) => void;
  handleUserUnblocked: (unblocker: string, unblocked: string) => void;
}

// 实现
handleMessageRead: (msgId: number, reader: string) => {
  set(state => {
    const newMessages = { ...state.messages };
    
    // 遍历所有会话找到该消息并更新
    for (const sessionId in newMessages) {
      newMessages[sessionId] = newMessages[sessionId].map(msg =>
        msg.id === msgId ? { ...msg, isRead: true } : msg
      );
    }
    
    return { messages: newMessages };
  });
},

handleMessageDeleted: (msgId: number) => {
  set(state => {
    const newMessages = { ...state.messages };
    
    for (const sessionId in newMessages) {
      newMessages[sessionId] = newMessages[sessionId].filter(msg => msg.id !== msgId);
    }
    
    return { messages: newMessages };
  });
},

handleSessionCreated: (session: Session) => {
  set(state => ({
    sessions: [session, ...state.sessions],
  }));
},

handleUserBlocked: (blocker: string, blocked: string) => {
  set(state => {
    const newBlockedUsers = new Set(state.blockedUsers);
    // 如果是别人拉黑了我，记录下来
    newBlockedUsers.add(`${blocker}:${blocked}`);
    return { blockedUsers: newBlockedUsers };
  });
},

handleUserUnblocked: (unblocker: string, unblocked: string) => {
  set(state => {
    const newBlockedUsers = new Set(state.blockedUsers);
    newBlockedUsers.delete(`${unblocker}:${unblocked}`);
    return { blockedUsers: newBlockedUsers };
  });
},
```

## 数据流架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         UI Layer                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ SessionList  │  │  ChatScreen  │  │  ChatInput   │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
└─────────┼─────────────────┼─────────────────┼───────────────────┘
          │                 │                 │
          ▼                 ▼                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Zustand Store                               │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ sessions | messages | currentSession | totalUnread      │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Service Layer                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ ChatService  │  │ CryptoService│  │ IpfsService  │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
└─────────┼─────────────────┼─────────────────┼───────────────────┘
          │                 │                 │
          ▼                 │                 ▼
┌─────────────────────┐     │     ┌─────────────────────┐
│   Polkadot Chain    │     │     │    IPFS Network     │
│   (元数据存储)       │     │     │    (内容存储)        │
└─────────────────────┘     │     └─────────────────────┘
                            │
                    ┌───────┴───────┐
                    │  E2E 加密/解密  │
                    └───────────────┘
```


## 关键流程

### 发送消息流程（乐观更新）

```
┌──────────┐    ┌──────────┐    ┌──────────────────────────────────────────┐
│ 用户输入  │ -> │ 立即显示  │    │           后台异步处理                    │
│          │    │ (乐观更新) │    │  ┌────────┐  ┌────────┐  ┌────────┐    │
└──────────┘    └──────────┘    │  │AES加密 │->│上传IPFS│->│链上交易│    │
                    │           │  └────────┘  └────────┘  └────────┘    │
                    ▼           └──────────────────┬───────────────────────┘
              status: 'sending'                    │
              用户立即看到消息                       ▼
                                            ┌──────────────┐
                                            │ 成功/失败回调 │
                                            └──────┬───────┘
                                                   │
                              ┌────────────────────┴────────────────────┐
                              ▼                                         ▼
                    ┌──────────────┐                          ┌──────────────┐
                    │ status: 'sent'│                          │status: 'failed'│
                    │ 更新为真实ID   │                          │ 显示重试按钮  │
                    └──────────────┘                          └──────────────┘
```

**延时对比：**
- 传统方式：用户等待 4-10 秒才能看到消息
- 乐观更新：用户立即（<100ms）看到消息

### 乐观更新实现

```typescript
// frontend/src/stores/chat.store.ts

interface Message {
  id: number | string;      // 临时ID为字符串，链上ID为数字
  tempId?: string;          // 保留临时ID用于匹配
  status: 'sending' | 'sent' | 'failed';
  error?: string;           // 失败原因
  retryCount?: number;      // 重试次数
  // ... 其他字段
}

const useChatStore = create<ChatState>()((set, get) => ({
  // ...
  
  /**
   * 发送消息（乐观更新）
   */
  sendMessage: async (receiver: string, content: string, msgType = 0) => {
    const chatService = getChatService();
    const currentSession = get().currentSession;
    const sessionId = currentSession?.id || '';
    
    // 1️⃣ 生成临时消息，立即显示
    const tempId = `temp-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    const tempMessage: Message = {
      id: tempId,
      tempId,
      sessionId,
      sender: chatService.myAddress,
      receiver,
      content,
      contentCid: '',
      msgType,
      sentAt: 0,  // 待确认
      isRead: false,
      isDeletedBySender: false,
      isDeletedByReceiver: false,
      isMine: true,
      status: 'sending',
      retryCount: 0,
    };
    
    // 立即添加到消息列表（用户马上看到）
    set(state => ({
      messages: {
        ...state.messages,
        [sessionId]: [...(state.messages[sessionId] || []), tempMessage],
      },
    }));
    
    try {
      // 2️⃣ 后台执行实际发送（加密 -> IPFS -> 链上）
      const result = await chatService.sendMessage(receiver, content, msgType, sessionId);
      
      // 3️⃣ 成功：用真实数据替换临时消息
      set(state => ({
        messages: {
          ...state.messages,
          [sessionId]: state.messages[sessionId]?.map(msg =>
            msg.tempId === tempId
              ? {
                  ...msg,
                  id: result.msgId,
                  sessionId: result.sessionId,
                  contentCid: result.cid,
                  sentAt: result.blockNumber,
                  status: 'sent' as const,
                }
              : msg
          ) || [],
        },
      }));
      
      return result;
      
    } catch (error) {
      // 4️⃣ 失败：标记错误状态，保留消息供重试
      set(state => ({
        messages: {
          ...state.messages,
          [sessionId]: state.messages[sessionId]?.map(msg =>
            msg.tempId === tempId
              ? {
                  ...msg,
                  status: 'failed' as const,
                  error: (error as Error).message,
                }
              : msg
          ) || [],
        },
      }));
      
      throw error;
    }
  },
  
  /**
   * 重试发送失败的消息
   */
  retryMessage: async (tempId: string) => {
    const state = get();
    let targetMessage: Message | null = null;
    let targetSessionId: string | null = null;
    
    // 找到失败的消息
    for (const [sessionId, messages] of Object.entries(state.messages)) {
      const msg = messages.find(m => m.tempId === tempId && m.status === 'failed');
      if (msg) {
        targetMessage = msg;
        targetSessionId = sessionId;
        break;
      }
    }
    
    if (!targetMessage || !targetSessionId) return;
    
    // 检查重试次数
    if ((targetMessage.retryCount || 0) >= 3) {
      throw new Error('重试次数已达上限');
    }
    
    // 更新状态为发送中
    set(state => ({
      messages: {
        ...state.messages,
        [targetSessionId!]: state.messages[targetSessionId!]?.map(msg =>
          msg.tempId === tempId
            ? { ...msg, status: 'sending' as const, retryCount: (msg.retryCount || 0) + 1 }
            : msg
        ) || [],
      },
    }));
    
    // 重新发送
    const chatService = getChatService();
    try {
      const result = await chatService.sendMessage(
        targetMessage.receiver,
        targetMessage.content,
        targetMessage.msgType,
        targetSessionId
      );
      
      set(state => ({
        messages: {
          ...state.messages,
          [targetSessionId!]: state.messages[targetSessionId!]?.map(msg =>
            msg.tempId === tempId
              ? { ...msg, id: result.msgId, status: 'sent' as const }
              : msg
          ) || [],
        },
      }));
    } catch (error) {
      set(state => ({
        messages: {
          ...state.messages,
          [targetSessionId!]: state.messages[targetSessionId!]?.map(msg =>
            msg.tempId === tempId
              ? { ...msg, status: 'failed' as const, error: (error as Error).message }
              : msg
          ) || [],
        },
      }));
      throw error;
    }
  },
  
  /**
   * 删除发送失败的消息
   */
  removeFailedMessage: (tempId: string) => {
    set(state => {
      const newMessages = { ...state.messages };
      for (const sessionId in newMessages) {
        newMessages[sessionId] = newMessages[sessionId].filter(
          msg => msg.tempId !== tempId || msg.status !== 'failed'
        );
      }
      return { messages: newMessages };
    });
  },
}));
```

### 消息气泡状态展示

```tsx
// frontend/src/features/chat/components/ChatBubble.tsx

export function ChatBubble({ message, onRetry, onDelete }: Props) {
  const isMine = message.isMine;
  
  return (
    <View style={[styles.container, isMine ? styles.mine : styles.theirs]}>
      <View style={[
        styles.bubble,
        isMine ? styles.bubbleMine : styles.bubbleTheirs,
        message.status === 'failed' && styles.bubbleFailed,
      ]}>
        <Text style={[styles.text, isMine && styles.textMine]}>
          {message.content}
        </Text>
      </View>
      
      <View style={[styles.meta, isMine && styles.metaMine]}>
        {/* 发送状态指示 */}
        {isMine && (
          <MessageStatus
            status={message.status}
            isRead={message.isRead}
            error={message.error}
            onRetry={() => onRetry?.(message.tempId!)}
            onDelete={() => onDelete?.(message.tempId!)}
          />
        )}
      </View>
    </View>
  );
}

function MessageStatus({ status, isRead, error, onRetry, onDelete }: StatusProps) {
  switch (status) {
    case 'sending':
      return (
        <View style={styles.statusRow}>
          <ActivityIndicator size="small" color="#999" />
          <Text style={styles.statusText}>发送中</Text>
        </View>
      );
    
    case 'sent':
      return (
        <Text style={styles.statusText}>
          {isRead ? '已读 ✓✓' : '已发送 ✓'}
        </Text>
      );
    
    case 'failed':
      return (
        <View style={styles.failedRow}>
          <Text style={styles.errorText}>发送失败</Text>
          <TouchableOpacity onPress={onRetry} style={styles.retryBtn}>
            <Text style={styles.retryText}>重试</Text>
          </TouchableOpacity>
          <TouchableOpacity onPress={onDelete} style={styles.deleteBtn}>
            <Text style={styles.deleteText}>删除</Text>
          </TouchableOpacity>
        </View>
      );
    
    default:
      return null;
  }
}

const styles = StyleSheet.create({
  // ... 其他样式
  bubbleFailed: {
    opacity: 0.6,
    borderColor: '#FF3B30',
    borderWidth: 1,
  },
  statusRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  statusText: {
    fontSize: 11,
    color: '#999',
  },
  failedRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  errorText: {
    fontSize: 11,
    color: '#FF3B30',
  },
  retryBtn: {
    paddingHorizontal: 8,
    paddingVertical: 2,
    backgroundColor: '#007AFF',
    borderRadius: 4,
  },
  retryText: {
    fontSize: 11,
    color: '#fff',
  },
  deleteBtn: {
    paddingHorizontal: 8,
    paddingVertical: 2,
  },
  deleteText: {
    fontSize: 11,
    color: '#999',
  },
});
```

### 接收消息流程

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│ 监听事件  │ -> │ 获取CID  │ -> │ 下载IPFS │ -> │ AES解密  │ -> │ 更新Store│
└──────────┘    └──────────┘    └──────────┘    └──────────┘    └──────────┘
      │               │               │               │               │
      ▼               ▼               ▼               ▼               ▼
 MessageSent      从链上获取      下载加密内容    使用共享密钥     刷新UI
   事件           消息元数据                      解密消息
```

**优化：并行预取**

```typescript
// 收到事件后立即开始下载，不等待 UI 更新
onMessageEvent(async (event) => {
  // 并行执行：获取元数据 + 预取 IPFS 内容
  const [metadata, content] = await Promise.all([
    getMessageMetadata(event.msgId),
    prefetchFromIpfs(event.cid),  // 多网关并行
  ]);
  
  // 解密并更新 UI
  const decrypted = await decrypt(content);
  updateMessages(metadata, decrypted);
});
```

### 消息延时分析

| 场景 | 传统方式 | 乐观更新 |
|------|----------|----------|
| 发送方看到消息 | 4-10秒 | **<100ms** |
| 发送方确认成功 | 4-10秒 | 4-10秒 |
| 接收方看到消息 | 4-10秒 | 4-10秒 |

乐观更新不能减少实际的链上确认时间，但能让发送方**立即看到自己发送的消息**，大幅提升用户体验。

### 密钥交换流程

```
┌─────────────────────────────────────────────────────────────────┐
│                   X25519 ECDH 密钥交换                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Alice (x25519 keypair)               Bob (x25519 keypair)      │
│    │                                       │                     │
│    │  privateA, publicA                    │  privateB, publicB  │
│    │                                       │                     │
│    │  1. 获取 Bob 的公钥 ──────────────>   │                     │
│    │     (从链上或首次消息)                 │                     │
│    │                                       │                     │
│    │  2. ECDH 计算共享密钥                  │                     │
│    │     sharedSecret = x25519.getSharedSecret(privateA, publicB)│
│    │                                       │                     │
│    │  3. HKDF 派生 AES 密钥                 │                     │
│    │     aesKey = HKDF(sharedSecret, 'stardust-chat-v1')         │
│    │                                       │                     │
│    │  ═══════════════════════════════════  │                     │
│    │                                       │                     │
│    │   <─────────────── Bob 收到消息后     │                     │
│    │                    同样计算:          │                     │
│    │                    sharedSecret = x25519.getSharedSecret(privateB, publicA)
│    │                    aesKey = HKDF(sharedSecret, 'stardust-chat-v1')
│    │                                       │                     │
│    │  ═══════════════════════════════════  │                     │
│    │         双方得到相同的 aesKey          │                     │
│    │  ═══════════════════════════════════  │                     │
│                                                                  │
│  注意：使用 @noble/curves 的 x25519 实现 ECDH                    │
│  基于 Curve25519，提供 128-bit 安全强度                          │
│  sr25519 用于签名，x25519 用于密钥交换（两者基于相同曲线）        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## 安全设计

### 1. 端到端加密

- **密钥交换**: X25519 ECDH (基于 Curve25519，使用 @noble/curves)
- **对称加密**: AES-256-GCM
- **密钥派生**: HKDF-SHA256
- **每条消息使用随机 12 字节 IV**
- **128-bit 认证标签防篡改**
- **链上只存储加密后的 CID**

### 2. 密钥管理

```typescript
// 密钥存储策略
interface KeyStorage {
  // 聊天密钥：使用 expo-secure-store 加密存储
  // x25519 密钥对，专用于端到端加密
  chatKeyPair: {
    privateKey: Uint8Array;  // 32 bytes x25519 private key
    publicKey: Uint8Array;   // 32 bytes x25519 public key
  };
  
  // 会话密钥缓存：内存中，应用退出后清除
  // key: peerAddress, value: derived AES key
  sessionKeys: Map<string, Uint8Array>;
  
  // 密钥派生流程
  deriveSessionKey: async (peerPublicKey: Uint8Array) => {
    // 1. sr25519Agreement 计算共享密钥
    // 2. HKDF 派生 32 字节 AES 密钥
    // 3. 缓存到 sessionKeys
  };
}
```

### 3. CID 验证

后端会验证 CID 必须是加密后的格式：
- 长度 > 50 字节
- 或不以 "Qm" 开头（非标准 CIDv0）

### 4. 频率限制

- 后端限制：10 条消息 / 10 分钟
- 前端防抖：500ms 发送间隔
- 超出限制时显示友好提示

## 性能优化

### 1. 消息分页

```typescript
// 分页加载策略
const MESSAGES_PER_PAGE = 20;

// 首次加载最新 20 条
// 上拉加载更多历史消息
// 使用虚拟列表优化长列表性能
```

### 2. 会话缓存

```typescript
// 本地缓存策略
interface CacheStrategy {
  // 会话列表：AsyncStorage 持久化
  sessions: AsyncStorage;
  
  // 最近消息：内存 + AsyncStorage
  recentMessages: Map<string, Message[]>;
  
  // 缓存过期时间：5 分钟
  ttl: 300000;
}
```

### 3. 乐观更新

发送消息时立即更新 UI，交易确认后更新状态：

```typescript
// 1. 立即显示消息（status: 'sending'）
// 2. 交易打包后更新（status: 'sent'）
// 3. 失败时标记（status: 'failed'）并提供重试
```

## 错误处理

```typescript
// 错误类型定义
enum ChatErrorCode {
  NETWORK_ERROR = 'NETWORK_ERROR',
  ENCRYPTION_FAILED = 'ENCRYPTION_FAILED',
  DECRYPTION_FAILED = 'DECRYPTION_FAILED',
  IPFS_UPLOAD_FAILED = 'IPFS_UPLOAD_FAILED',
  IPFS_DOWNLOAD_FAILED = 'IPFS_DOWNLOAD_FAILED',
  TRANSACTION_FAILED = 'TRANSACTION_FAILED',
  RATE_LIMIT_EXCEEDED = 'RATE_LIMIT_EXCEEDED',
  USER_BLOCKED = 'USER_BLOCKED',
}

// 用户友好的错误提示
const errorMessages: Record<ChatErrorCode, string> = {
  NETWORK_ERROR: '网络连接失败，请检查网络设置',
  ENCRYPTION_FAILED: '消息加密失败',
  DECRYPTION_FAILED: '消息解密失败',
  IPFS_UPLOAD_FAILED: '内容上传失败，请重试',
  IPFS_DOWNLOAD_FAILED: '内容下载失败',
  TRANSACTION_FAILED: '发送失败，请重试',
  RATE_LIMIT_EXCEEDED: '发送过于频繁，请稍后再试',
  USER_BLOCKED: '对方已将您拉黑',
};
```

## 路由配置

```typescript
// app/(tabs)/chat/_layout.tsx
export default function ChatLayout() {
  return (
    <Stack>
      <Stack.Screen name="index" options={{ title: '消息' }} />
      <Stack.Screen name="[sessionId]" options={{ headerShown: false }} />
      <Stack.Screen name="new" options={{ title: '新建会话' }} />
    </Stack>
  );
}
```

## 通讯录模块

### 类型定义

```typescript
// frontend/src/features/contacts/types.ts

import type { ChatUserId, ChatUserProfile } from '@/features/chat/types';

/**
 * 好友关系状态
 */
export enum FriendStatus {
  /** 单向添加（仅我添加了对方） */
  OneWay = 0,
  /** 双向好友（互相添加） */
  Mutual = 1,
  /** 待确认（已发送好友申请） */
  Pending = 2,
}

/**
 * 联系人信息
 */
export interface Contact {
  /** 联系人地址 */
  address: string;
  /** 联系人 ChatUserId */
  chatUserId?: ChatUserId;
  /** 备注名 */
  alias?: string;
  /** 所属分组 */
  groups: string[];
  /** 好友状态 */
  friendStatus: FriendStatus;
  /** 添加时间（区块高度） */
  addedAt: number;
  /** 最后更新时间（区块高度） */
  updatedAt: number;
  /** 用户资料（缓存） */
  profile?: ChatUserProfile;
}

/**
 * 分组信息
 */
export interface ContactGroup {
  /** 分组名称 */
  name: string;
  /** 成员数量 */
  memberCount: number;
  /** 创建时间（区块高度） */
  createdAt: number;
}

/**
 * 黑名单记录
 */
export interface BlockedUser {
  /** 被屏蔽的地址 */
  address: string;
  /** 屏蔽原因 */
  reason?: string;
  /** 屏蔽时间（区块高度） */
  blockedAt: number;
}

/**
 * 好友申请
 */
export interface FriendRequest {
  /** 申请者地址 */
  requester: string;
  /** 申请者 ChatUserId */
  requesterChatId?: ChatUserId;
  /** 申请留言 */
  message?: string;
  /** 申请时间（区块高度） */
  requestedAt: number;
  /** 是否已过期 */
  isExpired: boolean;
  /** 申请者资料 */
  profile?: ChatUserProfile;
}

/**
 * 通讯录统计
 */
export interface ContactsStats {
  /** 联系人总数 */
  contactCount: number;
  /** 分组总数 */
  groupCount: number;
  /** 黑名单数量 */
  blacklistCount: number;
  /** 待处理好友申请数量 */
  pendingRequestCount: number;
}
```

### 通讯录服务

```typescript
// frontend/src/services/contacts.service.ts

import { ApiPromise } from '@polkadot/api';
import { getApi } from '@/api';
import type {
  Contact,
  ContactGroup,
  BlockedUser,
  FriendRequest,
  FriendStatus,
  ContactsStats,
} from '@/features/contacts/types';

// 好友申请有效期（区块数），与后端配置一致
const FRIEND_REQUEST_EXPIRY = 100800; // 约 7 天

export class ContactsService {
  private api: ApiPromise | null = null;
  private myAddress: string;
  
  constructor(myAddress: string) {
    this.myAddress = myAddress;
  }
  
  async init(): Promise<void> {
    this.api = await getApi();
  }
  
  // ========== 统计信息 ==========
  
  /**
   * 获取通讯录统计信息
   */
  async getStats(): Promise<ContactsStats> {
    if (!this.api) throw new Error('API not initialized');
    
    const [contactCount, groupCount, blacklistCount, pendingRequestCount] = await Promise.all([
      this.api.query.contacts.contactCount(this.myAddress),
      this.api.query.contacts.groupCount(this.myAddress),
      this.api.query.contacts.blacklistCount(this.myAddress),
      this.api.query.contacts.pendingRequestCount(this.myAddress),
    ]);
    
    return {
      contactCount: contactCount.toNumber(),
      groupCount: groupCount.toNumber(),
      blacklistCount: blacklistCount.toNumber(),
      pendingRequestCount: pendingRequestCount.toNumber(),
    };
  }
  
  // ========== 联系人管理 ==========
  
  /**
   * 添加联系人
   */
  async addContact(
    contact: string,
    alias?: string,
    groups?: string[]
  ): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.contacts.addContact(
      contact,
      alias || null,
      groups || []
    );
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 删除联系人
   */
  async removeContact(contact: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.contacts.removeContact(contact);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 更新联系人信息
   */
  async updateContact(
    contact: string,
    alias?: string,
    groups?: string[]
  ): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.contacts.updateContact(
      contact,
      alias || null,
      groups || []
    );
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 获取所有联系人
   */
  async getAllContacts(): Promise<Contact[]> {
    if (!this.api) return [];
    
    const entries = await this.api.query.contacts.contacts.entries(this.myAddress);
    const contacts: Contact[] = [];
    
    for (const [key, value] of entries) {
      const contactAddr = key.args[1].toString();
      const data = value.unwrap();
      
      // 解析好友状态
      const friendStatusStr = data.friendStatus.toString();
      let friendStatus: FriendStatus;
      switch (friendStatusStr) {
        case 'Mutual':
          friendStatus = FriendStatus.Mutual;
          break;
        case 'Pending':
          friendStatus = FriendStatus.Pending;
          break;
        default:
          friendStatus = FriendStatus.OneWay;
      }
      
      contacts.push({
        address: contactAddr,
        alias: data.alias.isSome ? data.alias.unwrap().toUtf8() : undefined,
        groups: data.groups.map((g: any) => g.toUtf8()),
        friendStatus,
        addedAt: data.addedAt.toNumber(),
        updatedAt: data.updatedAt.toNumber(),
      });
    }
    
    return contacts;
  }
  
  /**
   * 检查是否为双向好友
   */
  areMutualFriends(contact: Contact): boolean {
    return contact.friendStatus === FriendStatus.Mutual;
  }
  
  /**
   * 获取双向好友列表
   */
  async getMutualFriends(): Promise<Contact[]> {
    const contacts = await this.getAllContacts();
    return contacts.filter(c => c.friendStatus === FriendStatus.Mutual);
  }
  
  // ========== 分组管理 ==========
  
  /**
   * 创建分组
   */
  async createGroup(name: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.contacts.createGroup(name);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 删除分组
   */
  async deleteGroup(name: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.contacts.deleteGroup(name);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 重命名分组
   */
  async renameGroup(oldName: string, newName: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.contacts.renameGroup(oldName, newName);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 获取所有分组
   */
  async getAllGroups(): Promise<ContactGroup[]> {
    if (!this.api) return [];
    
    const entries = await this.api.query.contacts.groups.entries(this.myAddress);
    const groups: ContactGroup[] = [];
    
    for (const [key, value] of entries) {
      const name = key.args[1].toUtf8();
      const data = value.unwrap();
      
      groups.push({
        name,
        memberCount: data.memberCount.toNumber(),
        createdAt: data.createdAt.toNumber(),
      });
    }
    
    return groups;
  }
  
  /**
   * 获取分组成员
   */
  async getGroupMembers(groupName: string): Promise<string[]> {
    if (!this.api) return [];
    
    const result = await this.api.query.contacts.groupMembers(this.myAddress, groupName);
    if (result.isNone) return [];
    
    return result.unwrap().map((addr: any) => addr.toString());
  }
  
  // ========== 黑名单管理 ==========
  
  /**
   * 添加到黑名单
   */
  async blockAccount(account: string, reason?: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.contacts.blockAccount(account, reason || null);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 从黑名单移除
   */
  async unblockAccount(account: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.contacts.unblockAccount(account);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 获取黑名单列表
   */
  async getBlacklist(): Promise<BlockedUser[]> {
    if (!this.api) return [];
    
    const entries = await this.api.query.contacts.blacklist.entries(this.myAddress);
    const blockedUsers: BlockedUser[] = [];
    
    for (const [key, value] of entries) {
      const blockedAddr = key.args[1].toString();
      const data = value.unwrap();
      
      blockedUsers.push({
        address: blockedAddr,
        reason: data.reason.isSome ? data.reason.unwrap().toUtf8() : undefined,
        blockedAt: data.blockedAt.toNumber(),
      });
    }
    
    return blockedUsers;
  }
  
  /**
   * 检查是否在黑名单中
   */
  async isBlocked(account: string): Promise<boolean> {
    if (!this.api) return false;
    
    const result = await this.api.query.contacts.blacklist(this.myAddress, account);
    return result.isSome;
  }
  
  // ========== 好友申请 ==========
  
  /**
   * 发送好友申请
   */
  async sendFriendRequest(target: string, message?: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.contacts.sendFriendRequest(target, message || null);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 接受好友申请
   */
  async acceptFriendRequest(requester: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.contacts.acceptFriendRequest(requester);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 拒绝好友申请
   */
  async rejectFriendRequest(requester: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');
    
    const tx = this.api.tx.contacts.rejectFriendRequest(requester);
    await tx.signAndSend(this.myAddress);
  }
  
  /**
   * 获取收到的好友申请
   * @param includeExpired 是否包含已过期的申请，默认 false
   */
  async getReceivedFriendRequests(includeExpired = false): Promise<FriendRequest[]> {
    if (!this.api) return [];
    
    const entries = await this.api.query.contacts.friendRequests.entries(this.myAddress);
    const requests: FriendRequest[] = [];
    const currentBlock = (await this.api.query.system.number()).toNumber();
    
    for (const [key, value] of entries) {
      const requesterAddr = key.args[1].toString();
      const data = value.unwrap();
      const requestedAt = data.requestedAt.toNumber();
      const isExpired = (currentBlock - requestedAt) > FRIEND_REQUEST_EXPIRY;
      
      // 如果不包含过期申请，跳过已过期的
      if (!includeExpired && isExpired) continue;
      
      requests.push({
        requester: requesterAddr,
        message: data.message.isSome ? data.message.unwrap().toUtf8() : undefined,
        requestedAt,
        isExpired,
      });
    }
    
    // 按申请时间倒序排列（最新的在前）
    return requests.sort((a, b) => b.requestedAt - a.requestedAt);
  }
  
  /**
   * 获取待处理好友申请数量
   */
  async getPendingRequestCount(): Promise<number> {
    if (!this.api) return 0;
    
    const count = await this.api.query.contacts.pendingRequestCount(this.myAddress);
    return count.toNumber();
  }
  
  /**
   * 检查好友申请是否过期
   */
  async isRequestExpired(requestedAt: number): Promise<boolean> {
    if (!this.api) return true;
    
    const currentBlock = (await this.api.query.system.number()).toNumber();
    return (currentBlock - requestedAt) > FRIEND_REQUEST_EXPIRY;
  }
}

// 单例
let contactsServiceInstance: ContactsService | null = null;

export function getContactsService(): ContactsService {
  if (!contactsServiceInstance) {
    throw new Error('ContactsService not initialized');
  }
  return contactsServiceInstance;
}

export function initContactsService(myAddress: string): ContactsService {
  contactsServiceInstance = new ContactsService(myAddress);
  return contactsServiceInstance;
}
```

### 通讯录状态管理

```typescript
// frontend/src/stores/contacts.store.ts

import { create } from 'zustand';
import { initContactsService, getContactsService } from '@/services/contacts.service';
import type { Contact, ContactGroup, BlockedUser, FriendRequest } from '@/features/contacts/types';

interface ContactsState {
  // 状态
  contacts: Contact[];
  groups: ContactGroup[];
  blacklist: BlockedUser[];
  friendRequests: FriendRequest[];
  isLoading: boolean;
  error: string | null;
  
  // 初始化
  initialize: (address: string) => Promise<void>;
  
  // 联系人操作
  loadContacts: () => Promise<void>;
  addContact: (address: string, alias?: string, groups?: string[]) => Promise<void>;
  removeContact: (address: string) => Promise<void>;
  updateContact: (address: string, alias?: string, groups?: string[]) => Promise<void>;
  
  // 分组操作
  loadGroups: () => Promise<void>;
  createGroup: (name: string) => Promise<void>;
  deleteGroup: (name: string) => Promise<void>;
  renameGroup: (oldName: string, newName: string) => Promise<void>;
  
  // 黑名单操作
  loadBlacklist: () => Promise<void>;
  blockUser: (address: string, reason?: string) => Promise<void>;
  unblockUser: (address: string) => Promise<void>;
  
  // 好友申请操作
  loadFriendRequests: () => Promise<void>;
  sendFriendRequest: (target: string, message?: string) => Promise<void>;
  acceptFriendRequest: (requester: string) => Promise<void>;
  rejectFriendRequest: (requester: string) => Promise<void>;
}

export const useContactsStore = create<ContactsState>()((set, get) => ({
  contacts: [],
  groups: [],
  blacklist: [],
  friendRequests: [],
  isLoading: false,
  error: null,
  
  initialize: async (address: string) => {
    const service = initContactsService(address);
    await service.init();
    
    // 并行加载所有数据
    await Promise.all([
      get().loadContacts(),
      get().loadGroups(),
      get().loadBlacklist(),
      get().loadFriendRequests(),
    ]);
  },
  
  loadContacts: async () => {
    set({ isLoading: true });
    try {
      const service = getContactsService();
      const contacts = await service.getAllContacts();
      set({ contacts, error: null });
    } catch (error) {
      set({ error: (error as Error).message });
    } finally {
      set({ isLoading: false });
    }
  },
  
  addContact: async (address, alias, groups) => {
    set({ isLoading: true });
    try {
      const service = getContactsService();
      await service.addContact(address, alias, groups);
      await get().loadContacts();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },
  
  removeContact: async (address) => {
    set({ isLoading: true });
    try {
      const service = getContactsService();
      await service.removeContact(address);
      set(state => ({
        contacts: state.contacts.filter(c => c.address !== address),
      }));
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },
  
  updateContact: async (address, alias, groups) => {
    try {
      const service = getContactsService();
      await service.updateContact(address, alias, groups);
      await get().loadContacts();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },
  
  loadGroups: async () => {
    try {
      const service = getContactsService();
      const groups = await service.getAllGroups();
      set({ groups });
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },
  
  createGroup: async (name) => {
    try {
      const service = getContactsService();
      await service.createGroup(name);
      await get().loadGroups();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },
  
  deleteGroup: async (name) => {
    try {
      const service = getContactsService();
      await service.deleteGroup(name);
      set(state => ({
        groups: state.groups.filter(g => g.name !== name),
      }));
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },
  
  renameGroup: async (oldName, newName) => {
    try {
      const service = getContactsService();
      await service.renameGroup(oldName, newName);
      await get().loadGroups();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },
  
  loadBlacklist: async () => {
    try {
      const service = getContactsService();
      const blacklist = await service.getBlacklist();
      set({ blacklist });
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },
  
  blockUser: async (address, reason) => {
    try {
      const service = getContactsService();
      await service.blockAccount(address, reason);
      await get().loadBlacklist();
      // 同时从联系人列表移除
      set(state => ({
        contacts: state.contacts.filter(c => c.address !== address),
      }));
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },
  
  unblockUser: async (address) => {
    try {
      const service = getContactsService();
      await service.unblockAccount(address);
      set(state => ({
        blacklist: state.blacklist.filter(b => b.address !== address),
      }));
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },
  
  loadFriendRequests: async () => {
    try {
      const service = getContactsService();
      const friendRequests = await service.getReceivedFriendRequests();
      set({ friendRequests });
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },
  
  sendFriendRequest: async (target, message) => {
    try {
      const service = getContactsService();
      await service.sendFriendRequest(target, message);
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },
  
  acceptFriendRequest: async (requester) => {
    try {
      const service = getContactsService();
      await service.acceptFriendRequest(requester);
      // 刷新好友申请和联系人列表
      await Promise.all([
        get().loadFriendRequests(),
        get().loadContacts(),
      ]);
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },
  
  rejectFriendRequest: async (requester) => {
    try {
      const service = getContactsService();
      await service.rejectFriendRequest(requester);
      set(state => ({
        friendRequests: state.friendRequests.filter(r => r.requester !== requester),
      }));
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },
}));
```

### 通讯录 UI 组件

#### 联系人列表项

```tsx
// frontend/src/features/contacts/components/ContactItem.tsx

import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { OnlineStatus } from '@/features/chat/components/OnlineStatus';
import { FriendStatus, type Contact } from '../types';

interface Props {
  contact: Contact;
  onPress: () => void;
  onLongPress?: () => void;
}

// 好友状态标签配置
const FRIEND_STATUS_CONFIG = {
  [FriendStatus.Mutual]: { text: '好友', color: '#34C759' },
  [FriendStatus.OneWay]: { text: '已添加', color: '#8E8E93' },
  [FriendStatus.Pending]: { text: '待确认', color: '#FF9500' },
};

export function ContactItem({ contact, onPress, onLongPress }: Props) {
  const statusConfig = FRIEND_STATUS_CONFIG[contact.friendStatus];
  
  return (
    <TouchableOpacity
      style={styles.container}
      onPress={onPress}
      onLongPress={onLongPress}
    >
      <View style={styles.avatar}>
        <Text style={styles.avatarText}>
          {(contact.alias || contact.address).slice(0, 2).toUpperCase()}
        </Text>
        {contact.profile && (
          <OnlineStatus status={contact.profile.status} style={styles.statusBadge} />
        )}
      </View>
      
      <View style={styles.info}>
        <View style={styles.nameRow}>
          <Text style={styles.name} numberOfLines={1}>
            {contact.alias || `${contact.address.slice(0, 8)}...`}
          </Text>
          <View style={[styles.friendBadge, { backgroundColor: statusConfig.color }]}>
            <Text style={styles.friendBadgeText}>{statusConfig.text}</Text>
          </View>
        </View>
        
        {contact.groups.length > 0 && (
          <Text style={styles.groups} numberOfLines={1}>
            {contact.groups.join(', ')}
          </Text>
        )}
      </View>
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
    backgroundColor: '#fff',
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#e0e0e0',
  },
  avatar: {
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: '#007AFF',
    justifyContent: 'center',
    alignItems: 'center',
    position: 'relative',
  },
  avatarText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  statusBadge: {
    position: 'absolute',
    bottom: 0,
    right: 0,
  },
  info: {
    flex: 1,
    marginLeft: 12,
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  name: {
    fontSize: 16,
    fontWeight: '500',
    color: '#333',
  },
  friendBadge: {
    marginLeft: 8,
    backgroundColor: '#34C759',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
  },
  friendBadgeText: {
    fontSize: 10,
    color: '#fff',
  },
  groups: {
    fontSize: 13,
    color: '#999',
    marginTop: 2,
  },
});
```

#### 好友申请项

```tsx
// frontend/src/features/contacts/components/FriendRequestItem.tsx

import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { useBlockTime, BlockTimeUtils } from '@/hooks/useBlockTime';
import type { FriendRequest } from '../types';

interface Props {
  request: FriendRequest;
  onAccept: () => void;
  onReject: () => void;
}

export function FriendRequestItem({ request, onAccept, onReject }: Props) {
  const { currentBlock, blockTime } = useBlockTime();
  
  const timeAgo = BlockTimeUtils.formatRelative(
    request.requestedAt,
    currentBlock,
    blockTime
  );
  
  const isExpiringSoon = request.expiresAt - currentBlock < 14400; // 24小时内过期
  
  return (
    <View style={styles.container}>
      <View style={styles.avatar}>
        <Text style={styles.avatarText}>
          {request.requester.slice(0, 2).toUpperCase()}
        </Text>
      </View>
      
      <View style={styles.content}>
        <Text style={styles.name} numberOfLines={1}>
          {request.requesterChatId
            ? `ID: ${request.requesterChatId}`
            : `${request.requester.slice(0, 12)}...`}
        </Text>
        
        {request.message && (
          <Text style={styles.message} numberOfLines={2}>
            {request.message}
          </Text>
        )}
        
        <Text style={[styles.time, isExpiringSoon && styles.expiring]}>
          {timeAgo} {isExpiringSoon && '· 即将过期'}
        </Text>
      </View>
      
      <View style={styles.actions}>
        <TouchableOpacity style={styles.acceptBtn} onPress={onAccept}>
          <Text style={styles.acceptText}>接受</Text>
        </TouchableOpacity>
        <TouchableOpacity style={styles.rejectBtn} onPress={onReject}>
          <Text style={styles.rejectText}>拒绝</Text>
        </TouchableOpacity>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
    backgroundColor: '#fff',
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#e0e0e0',
  },
  avatar: {
    width: 48,
    height: 48,
    borderRadius: 24,
    backgroundColor: '#FF9500',
    justifyContent: 'center',
    alignItems: 'center',
  },
  avatarText: {
    color: '#fff',
    fontSize: 18,
    fontWeight: '600',
  },
  content: {
    flex: 1,
    marginLeft: 12,
  },
  name: {
    fontSize: 16,
    fontWeight: '500',
    color: '#333',
  },
  message: {
    fontSize: 14,
    color: '#666',
    marginTop: 4,
  },
  time: {
    fontSize: 12,
    color: '#999',
    marginTop: 4,
  },
  expiring: {
    color: '#FF9500',
  },
  actions: {
    flexDirection: 'row',
    gap: 8,
  },
  acceptBtn: {
    backgroundColor: '#007AFF',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 6,
  },
  acceptText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
  rejectBtn: {
    backgroundColor: '#f5f5f5',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 6,
  },
  rejectText: {
    color: '#666',
    fontSize: 14,
  },
});
```

### 通讯录事件监听

```typescript
// 在 useChatEvents.ts 中添加通讯录相关事件

// 通讯录事件
case 'ContactAdded': {
  const [owner, contact] = data;
  if (owner.toString() === myAddress.current) {
    // 刷新联系人列表
    contactsStore.loadContacts();
  }
  break;
}

case 'FriendRequestSent': {
  const [sender, receiver] = data;
  if (receiver.toString() === myAddress.current) {
    // 收到新的好友申请
    contactsStore.loadFriendRequests();
    // 可以显示通知
  }
  break;
}

case 'FriendRequestAccepted': {
  const [accepter, requester] = data;
  if (requester.toString() === myAddress.current) {
    // 我的好友申请被接受了
    contactsStore.loadContacts();
  }
  break;
}

case 'FriendStatusChanged': {
  const [account1, account2, isMutual] = data;
  // 好友关系变更，刷新联系人
  contactsStore.loadContacts();
  break;
}
```

## 后续扩展

1. **群聊支持**: 扩展 Session 支持多人
2. **消息撤回**: 在时间窗口内支持撤回
3. **消息转发**: 支持转发消息到其他会话
4. **语音/视频通话**: 集成 WebRTC
5. **消息搜索**: 本地全文搜索
6. **消息同步**: 多设备消息同步
