# 内置钱包（Embedded Wallet）设计方案

## 概述

本文档描述了 Stardust 移动端内置钱包的设计方案，无需依赖浏览器扩展，用户可以直接在 App 内创建、导入和管理钱包，实现完整的区块链交互功能。

## 技术栈

- **框架**: React Native + Expo
- **状态管理**: Zustand
- **链交互**: @polkadot/api + @polkadot/keyring
- **安全存储**: expo-secure-store
- **生物识别**: expo-local-authentication
- **加密**: @polkadot/util-crypto

## 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                        UI Layer                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ UnlockScreen│  │ WalletSetup │  │ AccountManagement   │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                     State Layer (Zustand)                    │
│  ┌─────────────────────────────────────────────────────────┐│
│  │                    walletStore                           ││
│  │  - isUnlocked, accounts, activeAccount, balance         ││
│  └─────────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────┤
│                     Service Layer                            │
│  ┌───────────────┐ ┌───────────────┐ ┌───────────────────┐  │
│  │SecureStorage  │ │ KeyringService│ │ BiometricService  │  │
│  │   Service     │ │               │ │                   │  │
│  └───────────────┘ └───────────────┘ └───────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                     Security Layer                           │
│  ┌───────────────┐ ┌───────────────┐ ┌───────────────────┐  │
│  │expo-secure-   │ │@polkadot/     │ │expo-local-        │  │
│  │store          │ │keyring        │ │authentication     │  │
│  └───────────────┘ └───────────────┘ └───────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## 目录结构

```
frontend/src/
├── features/
│   └── wallet/
│       ├── components/
│       │   ├── AccountCard.tsx         # 账户卡片
│       │   ├── AccountSelector.tsx     # 账户选择器
│       │   ├── BalanceDisplay.tsx      # 余额显示
│       │   ├── MnemonicDisplay.tsx     # 助记词显示
│       │   ├── MnemonicInput.tsx       # 助记词输入
│       │   ├── PinInput.tsx            # PIN 输入组件
│       │   ├── BiometricPrompt.tsx     # 生物识别提示
│       │   └── TransactionConfirm.tsx  # 交易确认弹窗
│       ├── screens/
│       │   ├── WelcomeScreen.tsx       # 欢迎页（创建/导入选择）
│       │   ├── CreateWalletScreen.tsx  # 创建钱包
│       │   ├── ImportWalletScreen.tsx  # 导入钱包
│       │   ├── SetupPinScreen.tsx      # 设置 PIN
│       │   ├── BackupMnemonicScreen.tsx# 备份助记词
│       │   ├── UnlockScreen.tsx        # 解锁页面
│       │   ├── AccountListScreen.tsx   # 账户列表
│       │   └── AccountDetailScreen.tsx # 账户详情
│       ├── hooks/
│       │   ├── useWallet.ts            # 钱包核心逻辑
│       │   ├── useBalance.ts           # 余额查询
│       │   └── useTransaction.ts       # 交易签名
│       └── types.ts                    # 类型定义
│
├── services/
│   ├── secure-storage.service.ts       # 安全存储服务
│   ├── keyring.service.ts              # 密钥环服务
│   └── biometric.service.ts            # 生物识别服务
│
└── stores/
    └── wallet.store.ts                 # 钱包状态管理
```

## 核心类型定义

```typescript
// frontend/src/features/wallet/types.ts

/**
 * 钱包账户
 */
export interface WalletAccount {
  /** 账户地址 (SS58 格式) */
  address: string;
  /** 账户名称 */
  name: string;
  /** 账户类型 */
  type: 'sr25519' | 'ed25519';
  /** 是否为主账户 */
  isPrimary: boolean;
  /** 创建时间 */
  createdAt: number;
  /** 派生路径 (如 //0, //1) */
  derivePath?: string;
}

/**
 * 加密存储的钱包数据
 */
export interface EncryptedWalletData {
  /** 加密后的助记词 */
  encryptedMnemonic: string;
  /** 加密后的账户列表 */
  encryptedAccounts: string;
  /** 加密算法版本 */
  version: number;
  /** 创建时间 */
  createdAt: number;
}

/**
 * 钱包设置
 */
export interface WalletSettings {
  /** 是否启用生物识别 */
  biometricEnabled: boolean;
  /** 自动锁定时间（分钟，0 表示不自动锁定） */
  autoLockMinutes: number;
  /** 是否显示余额 */
  showBalance: boolean;
  /** 默认网络 */
  defaultNetwork: string;
}

/**
 * 钱包状态
 */
export type WalletStatus = 
  | 'uninitialized'  // 未初始化（首次使用）
  | 'locked'         // 已锁定
  | 'unlocked'       // 已解锁
  | 'creating'       // 创建中
  | 'importing';     // 导入中

/**
 * 交易请求
 */
export interface TransactionRequest {
  /** 交易模块 */
  module: string;
  /** 交易方法 */
  method: string;
  /** 交易参数 */
  args: unknown[];
  /** 交易描述 */
  description?: string;
}

/**
 * 签名结果
 */
export interface SignedTransaction {
  /** 签名后的交易 hex */
  signedTx: string;
  /** 交易 hash */
  txHash: string;
}
```


## 安全存储服务

```typescript
// frontend/src/services/secure-storage.service.ts

import * as SecureStore from 'expo-secure-store';
import { pbkdf2Sync } from '@polkadot/util-crypto';
import { u8aToHex, hexToU8a, stringToU8a, u8aToString } from '@polkadot/util';

const STORAGE_KEYS = {
  WALLET_DATA: 'stardust_wallet_data',
  WALLET_SETTINGS: 'stardust_wallet_settings',
  PIN_HASH: 'stardust_pin_hash',
  BIOMETRIC_KEY: 'stardust_biometric_key',
} as const;

// 加密参数
const PBKDF2_ITERATIONS = 100000;
const SALT_LENGTH = 32;
const KEY_LENGTH = 32;

/**
 * 安全存储服务
 * 使用 expo-secure-store 存储敏感数据
 * iOS: Keychain
 * Android: Keystore + EncryptedSharedPreferences
 */
export class SecureStorageService {
  /**
   * 检查是否已初始化钱包
   */
  async isWalletInitialized(): Promise<boolean> {
    const data = await SecureStore.getItemAsync(STORAGE_KEYS.WALLET_DATA);
    return data !== null;
  }

  /**
   * 设置 PIN 码
   * 使用 PBKDF2 派生密钥，存储 hash 用于验证
   */
  async setPin(pin: string): Promise<void> {
    const salt = crypto.getRandomValues(new Uint8Array(SALT_LENGTH));
    const hash = pbkdf2Sync(
      stringToU8a(pin),
      salt,
      PBKDF2_ITERATIONS,
      KEY_LENGTH
    );
    
    const pinData = {
      salt: u8aToHex(salt),
      hash: u8aToHex(hash),
    };
    
    await SecureStore.setItemAsync(
      STORAGE_KEYS.PIN_HASH,
      JSON.stringify(pinData)
    );
  }

  /**
   * 验证 PIN 码
   */
  async verifyPin(pin: string): Promise<boolean> {
    const stored = await SecureStore.getItemAsync(STORAGE_KEYS.PIN_HASH);
    if (!stored) return false;
    
    const { salt, hash } = JSON.parse(stored);
    const computedHash = pbkdf2Sync(
      stringToU8a(pin),
      hexToU8a(salt),
      PBKDF2_ITERATIONS,
      KEY_LENGTH
    );
    
    return u8aToHex(computedHash) === hash;
  }

  /**
   * 从 PIN 派生加密密钥
   */
  async deriveKeyFromPin(pin: string): Promise<Uint8Array> {
    const stored = await SecureStore.getItemAsync(STORAGE_KEYS.PIN_HASH);
    if (!stored) throw new Error('PIN not set');
    
    const { salt } = JSON.parse(stored);
    
    // 使用不同的 info 派生加密密钥（与验证 hash 不同）
    return pbkdf2Sync(
      stringToU8a(pin + ':encryption'),
      hexToU8a(salt),
      PBKDF2_ITERATIONS,
      KEY_LENGTH
    );
  }

  /**
   * 加密并存储钱包数据
   */
  async saveWalletData(
    mnemonic: string,
    accounts: WalletAccount[],
    encryptionKey: Uint8Array
  ): Promise<void> {
    const data = {
      mnemonic,
      accounts,
    };
    
    const encrypted = await this.encrypt(
      JSON.stringify(data),
      encryptionKey
    );
    
    const walletData: EncryptedWalletData = {
      encryptedMnemonic: encrypted,
      encryptedAccounts: '', // 冗余字段，实际数据在 encryptedMnemonic 中
      version: 1,
      createdAt: Date.now(),
    };
    
    await SecureStore.setItemAsync(
      STORAGE_KEYS.WALLET_DATA,
      JSON.stringify(walletData)
    );
  }

  /**
   * 解密并读取钱包数据
   */
  async loadWalletData(
    encryptionKey: Uint8Array
  ): Promise<{ mnemonic: string; accounts: WalletAccount[] } | null> {
    const stored = await SecureStore.getItemAsync(STORAGE_KEYS.WALLET_DATA);
    if (!stored) return null;
    
    const walletData: EncryptedWalletData = JSON.parse(stored);
    
    try {
      const decrypted = await this.decrypt(
        walletData.encryptedMnemonic,
        encryptionKey
      );
      return JSON.parse(decrypted);
    } catch {
      return null; // 解密失败（密钥错误）
    }
  }

  /**
   * 保存钱包设置
   */
  async saveSettings(settings: WalletSettings): Promise<void> {
    await SecureStore.setItemAsync(
      STORAGE_KEYS.WALLET_SETTINGS,
      JSON.stringify(settings)
    );
  }

  /**
   * 读取钱包设置
   */
  async loadSettings(): Promise<WalletSettings | null> {
    const stored = await SecureStore.getItemAsync(STORAGE_KEYS.WALLET_SETTINGS);
    return stored ? JSON.parse(stored) : null;
  }

  /**
   * 存储生物识别密钥
   * 用于生物识别解锁时派生加密密钥
   */
  async saveBiometricKey(key: Uint8Array): Promise<void> {
    await SecureStore.setItemAsync(
      STORAGE_KEYS.BIOMETRIC_KEY,
      u8aToHex(key),
      {
        requireAuthentication: true, // 需要生物识别才能访问
      }
    );
  }

  /**
   * 读取生物识别密钥
   */
  async loadBiometricKey(): Promise<Uint8Array | null> {
    try {
      const stored = await SecureStore.getItemAsync(
        STORAGE_KEYS.BIOMETRIC_KEY,
        {
          requireAuthentication: true,
        }
      );
      return stored ? hexToU8a(stored) : null;
    } catch {
      return null; // 生物识别验证失败
    }
  }

  /**
   * 清除所有钱包数据（危险操作）
   */
  async clearAll(): Promise<void> {
    await Promise.all([
      SecureStore.deleteItemAsync(STORAGE_KEYS.WALLET_DATA),
      SecureStore.deleteItemAsync(STORAGE_KEYS.WALLET_SETTINGS),
      SecureStore.deleteItemAsync(STORAGE_KEYS.PIN_HASH),
      SecureStore.deleteItemAsync(STORAGE_KEYS.BIOMETRIC_KEY),
    ]);
  }

  /**
   * AES-256-GCM 加密
   */
  private async encrypt(plaintext: string, key: Uint8Array): Promise<string> {
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const encoder = new TextEncoder();
    
    const cryptoKey = await crypto.subtle.importKey(
      'raw',
      key,
      { name: 'AES-GCM' },
      false,
      ['encrypt']
    );
    
    const encrypted = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      cryptoKey,
      encoder.encode(plaintext)
    );
    
    // 返回 IV + 密文
    const result = new Uint8Array(iv.length + encrypted.byteLength);
    result.set(iv, 0);
    result.set(new Uint8Array(encrypted), iv.length);
    
    return u8aToHex(result);
  }

  /**
   * AES-256-GCM 解密
   */
  private async decrypt(ciphertext: string, key: Uint8Array): Promise<string> {
    const data = hexToU8a(ciphertext);
    const iv = data.slice(0, 12);
    const encrypted = data.slice(12);
    
    const cryptoKey = await crypto.subtle.importKey(
      'raw',
      key,
      { name: 'AES-GCM' },
      false,
      ['decrypt']
    );
    
    const decrypted = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv },
      cryptoKey,
      encrypted
    );
    
    return new TextDecoder().decode(decrypted);
  }
}

// 单例
export const secureStorage = new SecureStorageService();
```


## 密钥环服务

```typescript
// frontend/src/services/keyring.service.ts

import { Keyring } from '@polkadot/keyring';
import { mnemonicGenerate, mnemonicValidate, cryptoWaitReady } from '@polkadot/util-crypto';
import type { KeyringPair } from '@polkadot/keyring/types';
import type { WalletAccount } from '@/features/wallet/types';

/**
 * 密钥环服务
 * 管理账户的创建、导入和签名
 */
export class KeyringService {
  private keyring: Keyring | null = null;
  private pairs: Map<string, KeyringPair> = new Map();
  private mnemonic: string | null = null;

  /**
   * 初始化密钥环
   */
  async init(): Promise<void> {
    await cryptoWaitReady();
    this.keyring = new Keyring({ type: 'sr25519', ss58Format: 42 });
  }

  /**
   * 生成新的助记词
   */
  generateMnemonic(): string {
    return mnemonicGenerate(12);
  }

  /**
   * 验证助记词
   */
  validateMnemonic(mnemonic: string): boolean {
    return mnemonicValidate(mnemonic);
  }

  /**
   * 从助记词创建钱包
   */
  async createFromMnemonic(
    mnemonic: string,
    accountName: string = 'Account 1'
  ): Promise<WalletAccount> {
    if (!this.keyring) {
      await this.init();
    }

    if (!this.validateMnemonic(mnemonic)) {
      throw new Error('Invalid mnemonic');
    }

    this.mnemonic = mnemonic;

    // 创建主账户
    const pair = this.keyring!.addFromMnemonic(mnemonic, { name: accountName });
    this.pairs.set(pair.address, pair);

    return {
      address: pair.address,
      name: accountName,
      type: 'sr25519',
      isPrimary: true,
      createdAt: Date.now(),
    };
  }

  /**
   * 派生新账户
   */
  async deriveAccount(
    accountName: string,
    derivePath: string = ''
  ): Promise<WalletAccount> {
    if (!this.keyring || !this.mnemonic) {
      throw new Error('Wallet not initialized');
    }

    // 自动生成派生路径
    const path = derivePath || `//${this.pairs.size}`;
    const fullPath = `${this.mnemonic}${path}`;

    const pair = this.keyring.addFromUri(fullPath, { name: accountName });
    this.pairs.set(pair.address, pair);

    return {
      address: pair.address,
      name: accountName,
      type: 'sr25519',
      isPrimary: false,
      createdAt: Date.now(),
      derivePath: path,
    };
  }

  /**
   * 恢复钱包（从存储的数据）
   */
  async restore(mnemonic: string, accounts: WalletAccount[]): Promise<void> {
    if (!this.keyring) {
      await this.init();
    }

    this.mnemonic = mnemonic;
    this.pairs.clear();

    for (const account of accounts) {
      const uri = account.derivePath
        ? `${mnemonic}${account.derivePath}`
        : mnemonic;

      const pair = this.keyring!.addFromUri(uri, { name: account.name });
      this.pairs.set(pair.address, pair);
    }
  }

  /**
   * 获取密钥对
   */
  getPair(address: string): KeyringPair | undefined {
    return this.pairs.get(address);
  }

  /**
   * 获取所有地址
   */
  getAddresses(): string[] {
    return Array.from(this.pairs.keys());
  }

  /**
   * 签名消息
   */
  signMessage(address: string, message: Uint8Array): Uint8Array {
    const pair = this.pairs.get(address);
    if (!pair) {
      throw new Error('Account not found');
    }
    return pair.sign(message);
  }

  /**
   * 签名交易
   */
  async signTransaction(
    address: string,
    tx: any // SubmittableExtrinsic
  ): Promise<string> {
    const pair = this.pairs.get(address);
    if (!pair) {
      throw new Error('Account not found');
    }

    const signed = await tx.signAsync(pair);
    return signed.toHex();
  }

  /**
   * 获取助记词（用于备份）
   */
  getMnemonic(): string | null {
    return this.mnemonic;
  }

  /**
   * 锁定钱包（清除内存中的密钥）
   */
  lock(): void {
    this.pairs.clear();
    this.mnemonic = null;
  }

  /**
   * 检查是否已解锁
   */
  isUnlocked(): boolean {
    return this.mnemonic !== null && this.pairs.size > 0;
  }

  /**
   * 导出账户（JSON 格式）
   */
  exportAccount(address: string, password: string): string {
    const pair = this.pairs.get(address);
    if (!pair) {
      throw new Error('Account not found');
    }
    return JSON.stringify(pair.toJson(password));
  }

  /**
   * 导入账户（JSON 格式）
   */
  async importAccount(json: string, password: string): Promise<WalletAccount> {
    if (!this.keyring) {
      await this.init();
    }

    const pair = this.keyring!.addFromJson(JSON.parse(json));
    pair.unlock(password);
    this.pairs.set(pair.address, pair);

    return {
      address: pair.address,
      name: pair.meta.name as string || 'Imported Account',
      type: pair.type as 'sr25519' | 'ed25519',
      isPrimary: false,
      createdAt: Date.now(),
    };
  }
}

// 单例
export const keyringService = new KeyringService();
```


## 生物识别服务

```typescript
// frontend/src/services/biometric.service.ts

import * as LocalAuthentication from 'expo-local-authentication';

/**
 * 生物识别类型
 */
export enum BiometricType {
  None = 'none',
  Fingerprint = 'fingerprint',
  FaceId = 'face_id',
  Iris = 'iris',
}

/**
 * 生物识别服务
 * 封装 expo-local-authentication
 */
export class BiometricService {
  /**
   * 检查设备是否支持生物识别
   */
  async isSupported(): Promise<boolean> {
    const compatible = await LocalAuthentication.hasHardwareAsync();
    const enrolled = await LocalAuthentication.isEnrolledAsync();
    return compatible && enrolled;
  }

  /**
   * 获取支持的生物识别类型
   */
  async getSupportedTypes(): Promise<BiometricType[]> {
    const types = await LocalAuthentication.supportedAuthenticationTypesAsync();
    
    return types.map(type => {
      switch (type) {
        case LocalAuthentication.AuthenticationType.FINGERPRINT:
          return BiometricType.Fingerprint;
        case LocalAuthentication.AuthenticationType.FACIAL_RECOGNITION:
          return BiometricType.FaceId;
        case LocalAuthentication.AuthenticationType.IRIS:
          return BiometricType.Iris;
        default:
          return BiometricType.None;
      }
    }).filter(t => t !== BiometricType.None);
  }

  /**
   * 获取生物识别显示名称
   */
  async getBiometricName(): Promise<string> {
    const types = await this.getSupportedTypes();
    
    if (types.includes(BiometricType.FaceId)) {
      return 'Face ID';
    } else if (types.includes(BiometricType.Fingerprint)) {
      return '指纹';
    } else if (types.includes(BiometricType.Iris)) {
      return '虹膜';
    }
    
    return '生物识别';
  }

  /**
   * 请求生物识别认证
   */
  async authenticate(options?: {
    promptMessage?: string;
    cancelLabel?: string;
    fallbackLabel?: string;
    disableDeviceFallback?: boolean;
  }): Promise<{ success: boolean; error?: string }> {
    try {
      const result = await LocalAuthentication.authenticateAsync({
        promptMessage: options?.promptMessage || '验证身份以解锁钱包',
        cancelLabel: options?.cancelLabel || '取消',
        fallbackLabel: options?.fallbackLabel || '使用 PIN',
        disableDeviceFallback: options?.disableDeviceFallback ?? false,
      });

      if (result.success) {
        return { success: true };
      }

      // 处理错误
      let errorMessage = '认证失败';
      switch (result.error) {
        case 'user_cancel':
          errorMessage = '用户取消';
          break;
        case 'user_fallback':
          errorMessage = '用户选择备用方式';
          break;
        case 'system_cancel':
          errorMessage = '系统取消';
          break;
        case 'not_enrolled':
          errorMessage = '未设置生物识别';
          break;
        case 'lockout':
          errorMessage = '尝试次数过多，请稍后再试';
          break;
      }

      return { success: false, error: errorMessage };
    } catch (error) {
      return { success: false, error: (error as Error).message };
    }
  }

  /**
   * 检查安全级别
   */
  async getSecurityLevel(): Promise<'none' | 'weak' | 'strong'> {
    const level = await LocalAuthentication.getEnrolledLevelAsync();
    
    switch (level) {
      case LocalAuthentication.SecurityLevel.NONE:
        return 'none';
      case LocalAuthentication.SecurityLevel.SECRET:
        return 'weak';
      case LocalAuthentication.SecurityLevel.BIOMETRIC_STRONG:
      case LocalAuthentication.SecurityLevel.BIOMETRIC_WEAK:
        return 'strong';
      default:
        return 'none';
    }
  }
}

// 单例
export const biometricService = new BiometricService();
```

## 钱包状态管理

```typescript
// frontend/src/stores/wallet.store.ts

import { create } from 'zustand';
import { secureStorage } from '@/services/secure-storage.service';
import { keyringService } from '@/services/keyring.service';
import { biometricService } from '@/services/biometric.service';
import type {
  WalletAccount,
  WalletSettings,
  WalletStatus,
  TransactionRequest,
} from '@/features/wallet/types';

interface WalletState {
  // 状态
  status: WalletStatus;
  accounts: WalletAccount[];
  activeAccount: WalletAccount | null;
  settings: WalletSettings;
  balance: string;
  isLoading: boolean;
  error: string | null;

  // 初始化
  initialize: () => Promise<void>;

  // 钱包创建/导入
  createWallet: (pin: string) => Promise<string>; // 返回助记词
  importWallet: (mnemonic: string, pin: string) => Promise<void>;

  // 解锁/锁定
  unlockWithPin: (pin: string) => Promise<boolean>;
  unlockWithBiometric: () => Promise<boolean>;
  lock: () => void;

  // 账户管理
  addAccount: (name: string) => Promise<WalletAccount>;
  setActiveAccount: (address: string) => void;
  renameAccount: (address: string, name: string) => Promise<void>;

  // 设置
  updateSettings: (settings: Partial<WalletSettings>) => Promise<void>;
  enableBiometric: (pin: string) => Promise<void>;
  disableBiometric: () => Promise<void>;

  // 交易
  signAndSend: (request: TransactionRequest) => Promise<string>;

  // 备份
  getMnemonic: (pin: string) => Promise<string | null>;

  // 重置
  resetWallet: () => Promise<void>;
}

const DEFAULT_SETTINGS: WalletSettings = {
  biometricEnabled: false,
  autoLockMinutes: 5,
  showBalance: true,
  defaultNetwork: 'stardust',
};

export const useWalletStore = create<WalletState>()((set, get) => ({
  status: 'uninitialized',
  accounts: [],
  activeAccount: null,
  settings: DEFAULT_SETTINGS,
  balance: '0',
  isLoading: false,
  error: null,

  initialize: async () => {
    set({ isLoading: true, error: null });

    try {
      await keyringService.init();

      const isInitialized = await secureStorage.isWalletInitialized();
      const settings = await secureStorage.loadSettings();

      set({
        status: isInitialized ? 'locked' : 'uninitialized',
        settings: settings || DEFAULT_SETTINGS,
      });
    } catch (error) {
      set({ error: (error as Error).message });
    } finally {
      set({ isLoading: false });
    }
  },

  createWallet: async (pin: string) => {
    set({ isLoading: true, status: 'creating', error: null });

    try {
      // 生成助记词
      const mnemonic = keyringService.generateMnemonic();

      // 创建主账户
      const account = await keyringService.createFromMnemonic(mnemonic);

      // 设置 PIN
      await secureStorage.setPin(pin);

      // 派生加密密钥并保存钱包数据
      const encryptionKey = await secureStorage.deriveKeyFromPin(pin);
      await secureStorage.saveWalletData(mnemonic, [account], encryptionKey);

      set({
        status: 'unlocked',
        accounts: [account],
        activeAccount: account,
      });

      return mnemonic;
    } catch (error) {
      set({ error: (error as Error).message, status: 'uninitialized' });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  importWallet: async (mnemonic: string, pin: string) => {
    set({ isLoading: true, status: 'importing', error: null });

    try {
      // 验证助记词
      if (!keyringService.validateMnemonic(mnemonic)) {
        throw new Error('无效的助记词');
      }

      // 创建账户
      const account = await keyringService.createFromMnemonic(mnemonic);

      // 设置 PIN
      await secureStorage.setPin(pin);

      // 保存钱包数据
      const encryptionKey = await secureStorage.deriveKeyFromPin(pin);
      await secureStorage.saveWalletData(mnemonic, [account], encryptionKey);

      set({
        status: 'unlocked',
        accounts: [account],
        activeAccount: account,
      });
    } catch (error) {
      set({ error: (error as Error).message, status: 'uninitialized' });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  unlockWithPin: async (pin: string) => {
    set({ isLoading: true, error: null });

    try {
      // 验证 PIN
      const isValid = await secureStorage.verifyPin(pin);
      if (!isValid) {
        set({ error: 'PIN 码错误' });
        return false;
      }

      // 解密钱包数据
      const encryptionKey = await secureStorage.deriveKeyFromPin(pin);
      const walletData = await secureStorage.loadWalletData(encryptionKey);

      if (!walletData) {
        set({ error: '无法解密钱包数据' });
        return false;
      }

      // 恢复密钥环
      await keyringService.restore(walletData.mnemonic, walletData.accounts);

      set({
        status: 'unlocked',
        accounts: walletData.accounts,
        activeAccount: walletData.accounts[0] || null,
      });

      return true;
    } catch (error) {
      set({ error: (error as Error).message });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  unlockWithBiometric: async () => {
    const { settings } = get();

    if (!settings.biometricEnabled) {
      set({ error: '未启用生物识别' });
      return false;
    }

    set({ isLoading: true, error: null });

    try {
      // 生物识别认证
      const result = await biometricService.authenticate();
      if (!result.success) {
        set({ error: result.error });
        return false;
      }

      // 获取生物识别密钥
      const biometricKey = await secureStorage.loadBiometricKey();
      if (!biometricKey) {
        set({ error: '生物识别密钥不存在' });
        return false;
      }

      // 解密钱包数据
      const walletData = await secureStorage.loadWalletData(biometricKey);
      if (!walletData) {
        set({ error: '无法解密钱包数据' });
        return false;
      }

      // 恢复密钥环
      await keyringService.restore(walletData.mnemonic, walletData.accounts);

      set({
        status: 'unlocked',
        accounts: walletData.accounts,
        activeAccount: walletData.accounts[0] || null,
      });

      return true;
    } catch (error) {
      set({ error: (error as Error).message });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  lock: () => {
    keyringService.lock();
    set({
      status: 'locked',
      activeAccount: null,
    });
  },

  addAccount: async (name: string) => {
    const account = await keyringService.deriveAccount(name);
    const { accounts } = get();
    const newAccounts = [...accounts, account];

    // 更新存储
    const mnemonic = keyringService.getMnemonic();
    if (mnemonic) {
      // 需要重新加密保存（这里简化处理，实际需要保存加密密钥）
      // 建议在解锁时缓存加密密钥
    }

    set({ accounts: newAccounts });
    return account;
  },

  setActiveAccount: (address: string) => {
    const { accounts } = get();
    const account = accounts.find(a => a.address === address);
    if (account) {
      set({ activeAccount: account });
    }
  },

  renameAccount: async (address: string, name: string) => {
    const { accounts } = get();
    const newAccounts = accounts.map(a =>
      a.address === address ? { ...a, name } : a
    );
    set({ accounts: newAccounts });
  },

  updateSettings: async (newSettings: Partial<WalletSettings>) => {
    const { settings } = get();
    const updated = { ...settings, ...newSettings };
    await secureStorage.saveSettings(updated);
    set({ settings: updated });
  },

  enableBiometric: async (pin: string) => {
    // 验证 PIN
    const isValid = await secureStorage.verifyPin(pin);
    if (!isValid) {
      throw new Error('PIN 码错误');
    }

    // 派生加密密钥
    const encryptionKey = await secureStorage.deriveKeyFromPin(pin);

    // 存储为生物识别密钥
    await secureStorage.saveBiometricKey(encryptionKey);

    // 更新设置
    await get().updateSettings({ biometricEnabled: true });
  },

  disableBiometric: async () => {
    await get().updateSettings({ biometricEnabled: false });
  },

  signAndSend: async (request: TransactionRequest) => {
    const { activeAccount } = get();
    if (!activeAccount) {
      throw new Error('未选择账户');
    }

    // 这里需要与 @polkadot/api 集成
    // 简化示例
    const { getApi } = await import('@/api');
    const api = await getApi();

    const tx = api.tx[request.module][request.method](...request.args);
    const signedTx = await keyringService.signTransaction(
      activeAccount.address,
      tx
    );

    // 发送交易
    const hash = await api.rpc.author.submitExtrinsic(signedTx);
    return hash.toHex();
  },

  getMnemonic: async (pin: string) => {
    const isValid = await secureStorage.verifyPin(pin);
    if (!isValid) return null;

    return keyringService.getMnemonic();
  },

  resetWallet: async () => {
    keyringService.lock();
    await secureStorage.clearAll();
    set({
      status: 'uninitialized',
      accounts: [],
      activeAccount: null,
      settings: DEFAULT_SETTINGS,
    });
  },
}));
```


## UI 组件示例

### 解锁页面

```typescript
// frontend/src/features/wallet/screens/UnlockScreen.tsx

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  Vibration,
} from 'react-native';
import { useWalletStore } from '@/stores/wallet.store';
import { biometricService } from '@/services/biometric.service';
import { PinInput } from '../components/PinInput';

export function UnlockScreen() {
  const [pin, setPin] = useState('');
  const [error, setError] = useState('');
  const [biometricName, setBiometricName] = useState('');

  const {
    settings,
    unlockWithPin,
    unlockWithBiometric,
    isLoading,
  } = useWalletStore();

  useEffect(() => {
    // 获取生物识别名称
    biometricService.getBiometricName().then(setBiometricName);

    // 自动触发生物识别
    if (settings.biometricEnabled) {
      handleBiometricUnlock();
    }
  }, []);

  const handlePinComplete = async (enteredPin: string) => {
    setError('');
    const success = await unlockWithPin(enteredPin);

    if (!success) {
      setError('PIN 码错误');
      setPin('');
      Vibration.vibrate(200);
    }
  };

  const handleBiometricUnlock = async () => {
    setError('');
    const success = await unlockWithBiometric();

    if (!success) {
      setError('生物识别失败');
    }
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.logo}>✨</Text>
        <Text style={styles.title}>Stardust</Text>
        <Text style={styles.subtitle}>输入 PIN 码解锁钱包</Text>
      </View>

      <PinInput
        value={pin}
        onChange={setPin}
        onComplete={handlePinComplete}
        length={6}
        secureTextEntry
        disabled={isLoading}
      />

      {error ? (
        <Text style={styles.error}>{error}</Text>
      ) : null}

      {settings.biometricEnabled && (
        <TouchableOpacity
          style={styles.biometricButton}
          onPress={handleBiometricUnlock}
          disabled={isLoading}
        >
          <Text style={styles.biometricIcon}>
            {biometricName === 'Face ID' ? '👤' : '👆'}
          </Text>
          <Text style={styles.biometricText}>
            使用{biometricName}解锁
          </Text>
        </TouchableOpacity>
      )}

      <TouchableOpacity style={styles.forgotButton}>
        <Text style={styles.forgotText}>忘记 PIN？使用助记词恢复</Text>
      </TouchableOpacity>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0a0a0f',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
  },
  header: {
    alignItems: 'center',
    marginBottom: 40,
  },
  logo: {
    fontSize: 48,
    marginBottom: 16,
  },
  title: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 16,
    color: '#888',
  },
  error: {
    color: '#ff4444',
    marginTop: 16,
    fontSize: 14,
  },
  biometricButton: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 32,
    padding: 16,
    backgroundColor: '#1a1a2e',
    borderRadius: 12,
  },
  biometricIcon: {
    fontSize: 24,
    marginRight: 12,
  },
  biometricText: {
    color: '#fff',
    fontSize: 16,
  },
  forgotButton: {
    marginTop: 24,
  },
  forgotText: {
    color: '#666',
    fontSize: 14,
  },
});
```

### PIN 输入组件

```typescript
// frontend/src/features/wallet/components/PinInput.tsx

import React, { useRef, useEffect } from 'react';
import {
  View,
  TextInput,
  StyleSheet,
  TouchableOpacity,
  Text,
} from 'react-native';

interface PinInputProps {
  value: string;
  onChange: (value: string) => void;
  onComplete: (pin: string) => void;
  length?: number;
  secureTextEntry?: boolean;
  disabled?: boolean;
}

export function PinInput({
  value,
  onChange,
  onComplete,
  length = 6,
  secureTextEntry = true,
  disabled = false,
}: PinInputProps) {
  const inputRef = useRef<TextInput>(null);

  useEffect(() => {
    if (value.length === length) {
      onComplete(value);
    }
  }, [value, length, onComplete]);

  const handlePress = () => {
    inputRef.current?.focus();
  };

  const handleChange = (text: string) => {
    // 只允许数字
    const cleaned = text.replace(/[^0-9]/g, '').slice(0, length);
    onChange(cleaned);
  };

  return (
    <TouchableOpacity
      style={styles.container}
      onPress={handlePress}
      activeOpacity={1}
    >
      <View style={styles.dotsContainer}>
        {Array.from({ length }).map((_, index) => (
          <View
            key={index}
            style={[
              styles.dot,
              index < value.length && styles.dotFilled,
            ]}
          >
            {!secureTextEntry && index < value.length && (
              <Text style={styles.dotText}>{value[index]}</Text>
            )}
          </View>
        ))}
      </View>

      <TextInput
        ref={inputRef}
        style={styles.hiddenInput}
        value={value}
        onChangeText={handleChange}
        keyboardType="number-pad"
        maxLength={length}
        autoFocus
        editable={!disabled}
      />

      {/* 数字键盘（可选，用于自定义键盘） */}
      <View style={styles.keypad}>
        {[1, 2, 3, 4, 5, 6, 7, 8, 9, null, 0, 'del'].map((key, index) => (
          <TouchableOpacity
            key={index}
            style={[styles.key, key === null && styles.keyEmpty]}
            onPress={() => {
              if (key === 'del') {
                onChange(value.slice(0, -1));
              } else if (key !== null && value.length < length) {
                onChange(value + key);
              }
            }}
            disabled={disabled || key === null}
          >
            <Text style={styles.keyText}>
              {key === 'del' ? '⌫' : key}
            </Text>
          </TouchableOpacity>
        ))}
      </View>
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
  },
  dotsContainer: {
    flexDirection: 'row',
    gap: 16,
    marginBottom: 32,
  },
  dot: {
    width: 16,
    height: 16,
    borderRadius: 8,
    borderWidth: 2,
    borderColor: '#444',
    alignItems: 'center',
    justifyContent: 'center',
  },
  dotFilled: {
    backgroundColor: '#8b5cf6',
    borderColor: '#8b5cf6',
  },
  dotText: {
    color: '#fff',
    fontSize: 12,
    fontWeight: 'bold',
  },
  hiddenInput: {
    position: 'absolute',
    opacity: 0,
  },
  keypad: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    width: 280,
    justifyContent: 'center',
  },
  key: {
    width: 80,
    height: 60,
    alignItems: 'center',
    justifyContent: 'center',
    margin: 4,
    borderRadius: 8,
    backgroundColor: '#1a1a2e',
  },
  keyEmpty: {
    backgroundColor: 'transparent',
  },
  keyText: {
    color: '#fff',
    fontSize: 24,
    fontWeight: '500',
  },
});
```

### 创建钱包页面

```typescript
// frontend/src/features/wallet/screens/CreateWalletScreen.tsx

import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { useWalletStore } from '@/stores/wallet.store';
import { PinInput } from '../components/PinInput';

type Step = 'pin' | 'confirm' | 'backup';

export function CreateWalletScreen() {
  const [step, setStep] = useState<Step>('pin');
  const [pin, setPin] = useState('');
  const [confirmPin, setConfirmPin] = useState('');
  const [mnemonic, setMnemonic] = useState('');

  const { createWallet, isLoading } = useWalletStore();

  const handlePinComplete = (enteredPin: string) => {
    setPin(enteredPin);
    setStep('confirm');
  };

  const handleConfirmComplete = async (enteredPin: string) => {
    if (enteredPin !== pin) {
      Alert.alert('错误', 'PIN 码不匹配，请重新输入');
      setConfirmPin('');
      setStep('pin');
      setPin('');
      return;
    }

    try {
      const generatedMnemonic = await createWallet(pin);
      setMnemonic(generatedMnemonic);
      setStep('backup');
    } catch (error) {
      Alert.alert('错误', (error as Error).message);
    }
  };

  if (step === 'backup') {
    return (
      <View style={styles.container}>
        <Text style={styles.title}>备份助记词</Text>
        <Text style={styles.warning}>
          ⚠️ 请将以下 12 个单词按顺序抄写并妥善保管。
          这是恢复钱包的唯一方式！
        </Text>

        <View style={styles.mnemonicContainer}>
          {mnemonic.split(' ').map((word, index) => (
            <View key={index} style={styles.wordBox}>
              <Text style={styles.wordIndex}>{index + 1}</Text>
              <Text style={styles.word}>{word}</Text>
            </View>
          ))}
        </View>

        <TouchableOpacity
          style={styles.button}
          onPress={() => {
            Alert.alert(
              '确认',
              '您确定已经安全备份了助记词吗？',
              [
                { text: '取消', style: 'cancel' },
                {
                  text: '确定',
                  onPress: () => {
                    // 导航到主页
                  },
                },
              ]
            );
          }}
        >
          <Text style={styles.buttonText}>我已安全备份</Text>
        </TouchableOpacity>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <Text style={styles.title}>
        {step === 'pin' ? '设置 PIN 码' : '确认 PIN 码'}
      </Text>
      <Text style={styles.subtitle}>
        {step === 'pin'
          ? '设置 6 位数字 PIN 码保护您的钱包'
          : '请再次输入 PIN 码确认'}
      </Text>

      <PinInput
        value={step === 'pin' ? pin : confirmPin}
        onChange={step === 'pin' ? setPin : setConfirmPin}
        onComplete={step === 'pin' ? handlePinComplete : handleConfirmComplete}
        length={6}
        secureTextEntry
        disabled={isLoading}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0a0a0f',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 14,
    color: '#888',
    marginBottom: 32,
    textAlign: 'center',
  },
  warning: {
    fontSize: 14,
    color: '#ffa500',
    marginBottom: 24,
    textAlign: 'center',
    paddingHorizontal: 20,
  },
  mnemonicContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'center',
    gap: 8,
    marginBottom: 32,
  },
  wordBox: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#1a1a2e',
    paddingVertical: 8,
    paddingHorizontal: 12,
    borderRadius: 8,
    minWidth: 100,
  },
  wordIndex: {
    color: '#666',
    fontSize: 12,
    marginRight: 8,
  },
  word: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
  button: {
    backgroundColor: '#8b5cf6',
    paddingVertical: 16,
    paddingHorizontal: 32,
    borderRadius: 12,
  },
  buttonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
```


## 用户流程

### 首次使用流程

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  欢迎页面    │────▶│  创建钱包   │────▶│  设置 PIN   │
│             │     │  或导入钱包  │     │             │
└─────────────┘     └─────────────┘     └─────────────┘
                                               │
                                               ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   主页面    │◀────│  启用生物   │◀────│  备份助记词  │
│             │     │  识别(可选)  │     │             │
└─────────────┘     └─────────────┘     └─────────────┘
```

### 解锁流程

```
┌─────────────┐
│  App 启动   │
└──────┬──────┘
       │
       ▼
┌─────────────┐     ┌─────────────┐
│  检查钱包   │────▶│  首次使用   │──▶ 创建/导入流程
│  是否存在   │     │             │
└──────┬──────┘     └─────────────┘
       │ 存在
       ▼
┌─────────────┐     ┌─────────────┐
│  生物识别   │────▶│  解锁成功   │──▶ 进入主页
│  已启用?    │ 是  │             │
└──────┬──────┘     └─────────────┘
       │ 否                ▲
       ▼                   │
┌─────────────┐            │
│  输入 PIN   │────────────┘
│             │
└─────────────┘
```

### 交易签名流程

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  发起交易   │────▶│  显示交易   │────▶│  生物识别   │
│             │     │  详情确认   │     │  或 PIN     │
└─────────────┘     └─────────────┘     └──────┬──────┘
                                               │
                                               ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  交易完成   │◀────│  广播交易   │◀────│  签名交易   │
│             │     │             │     │             │
└─────────────┘     └─────────────┘     └─────────────┘
```

## 安全要点

### 密钥存储

| 平台 | 存储方式 | 安全级别 |
|------|----------|----------|
| iOS | Keychain (Secure Enclave) | 硬件级别 |
| Android | Keystore + EncryptedSharedPreferences | 硬件级别 |

### 加密方案

- **PIN 验证**: PBKDF2 (100,000 iterations) + SHA-256
- **数据加密**: AES-256-GCM
- **密钥派生**: PBKDF2 从 PIN 派生加密密钥

### 安全建议

1. **PIN 码要求**
   - 最少 6 位数字
   - 不允许简单序列（123456, 111111）
   - 错误次数限制（5 次后锁定）

2. **助记词保护**
   - 仅在备份时显示一次
   - 不存储明文
   - 导出需要 PIN 验证

3. **自动锁定**
   - 默认 5 分钟无操作自动锁定
   - 切换到后台立即锁定（可配置）

4. **交易确认**
   - 每笔交易需要生物识别或 PIN 确认
   - 显示交易详情（金额、接收方、手续费）

## 依赖安装

```bash
# 核心依赖
npx expo install expo-secure-store
npx expo install expo-local-authentication

# Polkadot 依赖
npm install @polkadot/api @polkadot/keyring @polkadot/util @polkadot/util-crypto

# 注意：需要配置 metro.config.js 支持 crypto polyfill
```

### Metro 配置

```javascript
// metro.config.js
const { getDefaultConfig } = require('expo/metro-config');

const config = getDefaultConfig(__dirname);

config.resolver.extraNodeModules = {
  crypto: require.resolve('react-native-crypto'),
  stream: require.resolve('stream-browserify'),
  buffer: require.resolve('buffer'),
};

module.exports = config;
```

### Babel 配置

```javascript
// babel.config.js
module.exports = function(api) {
  api.cache(true);
  return {
    presets: ['babel-preset-expo'],
    plugins: [
      // 支持 BigInt
      '@babel/plugin-syntax-bigint',
    ],
  };
};
```

## 与聊天模块集成

内置钱包与聊天模块的集成点：

```typescript
// 在 App 初始化时
import { useWalletStore } from '@/stores/wallet.store';
import { useChatStore } from '@/stores/chat.store';
import { useUserStore } from '@/stores/user.store';

function AppInitializer() {
  const { status, activeAccount } = useWalletStore();
  const { initialize: initChat } = useChatStore();
  const { initialize: initUser } = useUserStore();

  useEffect(() => {
    if (status === 'unlocked' && activeAccount) {
      // 钱包解锁后初始化聊天和用户服务
      initUser(activeAccount.address);
      initChat(activeAccount.address);
    }
  }, [status, activeAccount]);

  // ...
}
```

## 总结

内置钱包方案的核心优势：

1. **无需浏览器扩展** - 用户体验更流畅
2. **原生安全存储** - 利用 iOS Keychain / Android Keystore
3. **生物识别支持** - 快速解锁，安全便捷
4. **完整的账户管理** - 创建、导入、派生、备份
5. **与链上模块无缝集成** - 聊天、交易、签名一体化


---

## 补充：会话管理模块

```typescript
// frontend/src/services/session.service.ts

import { AppState, AppStateStatus } from 'react-native';
import { useWalletStore } from '@/stores/wallet.store';

/**
 * 会话管理服务
 * 处理自动锁定、后台切换、超时等场景
 */
export class SessionService {
  private lastActiveTime: number = Date.now();
  private lockTimer: NodeJS.Timeout | null = null;
  private appStateSubscription: any = null;

  /**
   * 启动会话管理
   */
  start(): void {
    // 监听 App 状态变化
    this.appStateSubscription = AppState.addEventListener(
      'change',
      this.handleAppStateChange
    );

    // 启动活动检测
    this.resetActivityTimer();
  }

  /**
   * 停止会话管理
   */
  stop(): void {
    if (this.appStateSubscription) {
      this.appStateSubscription.remove();
      this.appStateSubscription = null;
    }
    this.clearLockTimer();
  }

  /**
   * 记录用户活动（重置计时器）
   */
  recordActivity(): void {
    this.lastActiveTime = Date.now();
    this.resetActivityTimer();
  }

  /**
   * 处理 App 状态变化
   */
  private handleAppStateChange = (nextState: AppStateStatus): void => {
    const { status, lock, settings } = useWalletStore.getState();

    if (status !== 'unlocked') return;

    switch (nextState) {
      case 'background':
      case 'inactive':
        // 进入后台时的处理
        if (settings.lockOnBackground) {
          // 立即锁定
          lock();
        } else {
          // 记录进入后台时间，用于计算后台时长
          this.lastActiveTime = Date.now();
        }
        break;

      case 'active':
        // 从后台返回
        const backgroundDuration = Date.now() - this.lastActiveTime;
        const maxBackgroundMs = (settings.maxBackgroundMinutes || 1) * 60 * 1000;

        if (backgroundDuration > maxBackgroundMs) {
          lock();
        } else {
          this.resetActivityTimer();
        }
        break;
    }
  };

  /**
   * 重置自动锁定计时器
   */
  private resetActivityTimer(): void {
    this.clearLockTimer();

    const { settings, status } = useWalletStore.getState();

    if (status !== 'unlocked' || settings.autoLockMinutes <= 0) {
      return;
    }

    const timeoutMs = settings.autoLockMinutes * 60 * 1000;

    this.lockTimer = setTimeout(() => {
      const { lock } = useWalletStore.getState();
      lock();
    }, timeoutMs);
  }

  /**
   * 清除计时器
   */
  private clearLockTimer(): void {
    if (this.lockTimer) {
      clearTimeout(this.lockTimer);
      this.lockTimer = null;
    }
  }

  /**
   * 获取剩余解锁时间（秒）
   */
  getRemainingTime(): number {
    const { settings } = useWalletStore.getState();
    if (settings.autoLockMinutes <= 0) return Infinity;

    const elapsed = Date.now() - this.lastActiveTime;
    const remaining = settings.autoLockMinutes * 60 * 1000 - elapsed;

    return Math.max(0, Math.floor(remaining / 1000));
  }
}

// 单例
export const sessionService = new SessionService();
```

### 扩展钱包设置

```typescript
// 更新 WalletSettings 类型
export interface WalletSettings {
  biometricEnabled: boolean;
  autoLockMinutes: number;        // 无操作自动锁定（分钟，0=禁用）
  lockOnBackground: boolean;      // 切换后台立即锁定
  maxBackgroundMinutes: number;   // 后台最大时长（分钟）
  showBalance: boolean;
  defaultNetwork: string;
  requireAuthForTx: boolean;      // 交易需要认证
}

const DEFAULT_SETTINGS: WalletSettings = {
  biometricEnabled: false,
  autoLockMinutes: 5,
  lockOnBackground: false,
  maxBackgroundMinutes: 1,
  showBalance: true,
  defaultNetwork: 'stardust',
  requireAuthForTx: true,
};
```

### 会话管理 Hook

```typescript
// frontend/src/features/wallet/hooks/useSession.ts

import { useEffect, useCallback } from 'react';
import { sessionService } from '@/services/session.service';
import { useWalletStore } from '@/stores/wallet.store';

/**
 * 会话管理 Hook
 * 在根组件中使用，自动处理会话生命周期
 */
export function useSession() {
  const { status } = useWalletStore();

  useEffect(() => {
    if (status === 'unlocked') {
      sessionService.start();
    } else {
      sessionService.stop();
    }

    return () => {
      sessionService.stop();
    };
  }, [status]);

  // 记录用户活动（在触摸事件中调用）
  const recordActivity = useCallback(() => {
    sessionService.recordActivity();
  }, []);

  return { recordActivity };
}
```

### 在 App 根组件中使用

```typescript
// App.tsx
import { View, TouchableWithoutFeedback } from 'react-native';
import { useSession } from '@/features/wallet/hooks/useSession';

export default function App() {
  const { recordActivity } = useSession();

  return (
    <TouchableWithoutFeedback onPress={recordActivity}>
      <View style={{ flex: 1 }}>
        {/* App 内容 */}
      </View>
    </TouchableWithoutFeedback>
  );
}
```

---

## 补充：增强安全参数

### 提升 PBKDF2 迭代次数

```typescript
// frontend/src/services/secure-storage.service.ts

// 2026 年推荐的安全参数
const SECURITY_PARAMS = {
  // PBKDF2 迭代次数：OWASP 2024 建议 600,000+
  // 移动设备考虑性能，使用 600,000（约 300-500ms）
  PBKDF2_ITERATIONS: 600_000,
  
  // 盐长度
  SALT_LENGTH: 32,
  
  // 派生密钥长度
  KEY_LENGTH: 32,
  
  // AES-GCM IV 长度
  IV_LENGTH: 12,
  
  // 版本号（用于未来升级）
  VERSION: 2,
} as const;

/**
 * 根据设备性能动态调整迭代次数
 * 目标：派生时间 300-500ms
 */
async function calibrateIterations(): Promise<number> {
  const testIterations = 10_000;
  const testPassword = 'calibration_test';
  const testSalt = crypto.getRandomValues(new Uint8Array(32));

  const start = performance.now();
  await pbkdf2Derive(testPassword, testSalt, testIterations);
  const elapsed = performance.now() - start;

  // 计算达到 400ms 需要的迭代次数
  const targetMs = 400;
  const calibrated = Math.floor((targetMs / elapsed) * testIterations);

  // 限制范围：最少 300,000，最多 1,000,000
  return Math.max(300_000, Math.min(1_000_000, calibrated));
}
```

### PIN 错误限制机制

```typescript
// frontend/src/services/pin-guard.service.ts

import * as SecureStore from 'expo-secure-store';

const STORAGE_KEY = 'stardust_pin_attempts';

interface PinAttemptData {
  /** 连续错误次数 */
  failedAttempts: number;
  /** 最后一次尝试时间 */
  lastAttemptTime: number;
  /** 锁定解除时间（如果被锁定） */
  lockoutUntil: number | null;
}

/**
 * PIN 错误限制配置
 */
const PIN_GUARD_CONFIG = {
  /** 最大连续错误次数 */
  MAX_ATTEMPTS: 5,
  
  /** 锁定时间梯度（分钟） */
  LOCKOUT_MINUTES: [1, 5, 15, 60, 1440], // 1分钟, 5分钟, 15分钟, 1小时, 24小时
  
  /** 错误次数重置时间（小时） */
  RESET_AFTER_HOURS: 24,
  
  /** 永久锁定阈值（连续错误次数） */
  PERMANENT_LOCKOUT_THRESHOLD: 15,
} as const;

/**
 * PIN 保护服务
 * 防止暴力破解攻击
 */
export class PinGuardService {
  /**
   * 检查是否可以尝试 PIN
   * @returns 可以尝试返回 true，被锁定返回剩余秒数
   */
  async canAttempt(): Promise<{ allowed: boolean; remainingSeconds?: number; attemptsLeft?: number }> {
    const data = await this.loadAttemptData();

    // 检查是否永久锁定
    if (data.failedAttempts >= PIN_GUARD_CONFIG.PERMANENT_LOCKOUT_THRESHOLD) {
      return {
        allowed: false,
        remainingSeconds: -1, // -1 表示永久锁定
      };
    }

    // 检查是否在锁定期
    if (data.lockoutUntil && Date.now() < data.lockoutUntil) {
      const remainingSeconds = Math.ceil((data.lockoutUntil - Date.now()) / 1000);
      return {
        allowed: false,
        remainingSeconds,
      };
    }

    // 检查是否应该重置计数（超过重置时间）
    const resetTime = PIN_GUARD_CONFIG.RESET_AFTER_HOURS * 60 * 60 * 1000;
    if (Date.now() - data.lastAttemptTime > resetTime) {
      await this.resetAttempts();
      return {
        allowed: true,
        attemptsLeft: PIN_GUARD_CONFIG.MAX_ATTEMPTS,
      };
    }

    return {
      allowed: true,
      attemptsLeft: PIN_GUARD_CONFIG.MAX_ATTEMPTS - data.failedAttempts,
    };
  }

  /**
   * 记录 PIN 验证失败
   */
  async recordFailedAttempt(): Promise<{
    attemptsLeft: number;
    lockoutSeconds?: number;
    isPermanentlyLocked: boolean;
  }> {
    const data = await this.loadAttemptData();

    data.failedAttempts += 1;
    data.lastAttemptTime = Date.now();

    // 检查是否达到永久锁定阈值
    if (data.failedAttempts >= PIN_GUARD_CONFIG.PERMANENT_LOCKOUT_THRESHOLD) {
      await this.saveAttemptData(data);
      return {
        attemptsLeft: 0,
        isPermanentlyLocked: true,
      };
    }

    // 检查是否需要临时锁定
    if (data.failedAttempts >= PIN_GUARD_CONFIG.MAX_ATTEMPTS) {
      const lockoutIndex = Math.min(
        Math.floor((data.failedAttempts - PIN_GUARD_CONFIG.MAX_ATTEMPTS) / PIN_GUARD_CONFIG.MAX_ATTEMPTS),
        PIN_GUARD_CONFIG.LOCKOUT_MINUTES.length - 1
      );
      const lockoutMinutes = PIN_GUARD_CONFIG.LOCKOUT_MINUTES[lockoutIndex];
      data.lockoutUntil = Date.now() + lockoutMinutes * 60 * 1000;

      await this.saveAttemptData(data);

      return {
        attemptsLeft: 0,
        lockoutSeconds: lockoutMinutes * 60,
        isPermanentlyLocked: false,
      };
    }

    await this.saveAttemptData(data);

    return {
      attemptsLeft: PIN_GUARD_CONFIG.MAX_ATTEMPTS - data.failedAttempts,
      isPermanentlyLocked: false,
    };
  }

  /**
   * 记录 PIN 验证成功（重置计数）
   */
  async recordSuccess(): Promise<void> {
    await this.resetAttempts();
  }

  /**
   * 重置尝试次数
   */
  async resetAttempts(): Promise<void> {
    await this.saveAttemptData({
      failedAttempts: 0,
      lastAttemptTime: Date.now(),
      lockoutUntil: null,
    });
  }

  /**
   * 获取当前状态
   */
  async getStatus(): Promise<{
    failedAttempts: number;
    attemptsLeft: number;
    isLocked: boolean;
    lockoutRemainingSeconds?: number;
    isPermanentlyLocked: boolean;
  }> {
    const data = await this.loadAttemptData();
    const isPermanentlyLocked = data.failedAttempts >= PIN_GUARD_CONFIG.PERMANENT_LOCKOUT_THRESHOLD;

    let isLocked = isPermanentlyLocked;
    let lockoutRemainingSeconds: number | undefined;

    if (!isPermanentlyLocked && data.lockoutUntil && Date.now() < data.lockoutUntil) {
      isLocked = true;
      lockoutRemainingSeconds = Math.ceil((data.lockoutUntil - Date.now()) / 1000);
    }

    return {
      failedAttempts: data.failedAttempts,
      attemptsLeft: Math.max(0, PIN_GUARD_CONFIG.MAX_ATTEMPTS - data.failedAttempts),
      isLocked,
      lockoutRemainingSeconds,
      isPermanentlyLocked,
    };
  }

  private async loadAttemptData(): Promise<PinAttemptData> {
    const stored = await SecureStore.getItemAsync(STORAGE_KEY);
    if (!stored) {
      return {
        failedAttempts: 0,
        lastAttemptTime: Date.now(),
        lockoutUntil: null,
      };
    }
    return JSON.parse(stored);
  }

  private async saveAttemptData(data: PinAttemptData): Promise<void> {
    await SecureStore.setItemAsync(STORAGE_KEY, JSON.stringify(data));
  }
}

// 单例
export const pinGuard = new PinGuardService();
```

### 集成到解锁流程

```typescript
// 更新 wallet.store.ts 中的 unlockWithPin

unlockWithPin: async (pin: string) => {
  // 检查是否可以尝试
  const canAttempt = await pinGuard.canAttempt();
  
  if (!canAttempt.allowed) {
    if (canAttempt.remainingSeconds === -1) {
      set({ error: '钱包已永久锁定，请使用助记词恢复' });
    } else {
      const minutes = Math.ceil(canAttempt.remainingSeconds! / 60);
      set({ error: `请等待 ${minutes} 分钟后再试` });
    }
    return false;
  }

  set({ isLoading: true, error: null });

  try {
    const isValid = await secureStorage.verifyPin(pin);
    
    if (!isValid) {
      // 记录失败
      const result = await pinGuard.recordFailedAttempt();
      
      if (result.isPermanentlyLocked) {
        set({ error: '错误次数过多，钱包已永久锁定' });
      } else if (result.lockoutSeconds) {
        const minutes = Math.ceil(result.lockoutSeconds / 60);
        set({ error: `PIN 错误，请等待 ${minutes} 分钟后再试` });
      } else {
        set({ error: `PIN 错误，还剩 ${result.attemptsLeft} 次机会` });
      }
      return false;
    }

    // 验证成功，重置计数
    await pinGuard.recordSuccess();

    // ... 继续解锁流程
  } catch (error) {
    set({ error: (error as Error).message });
    return false;
  } finally {
    set({ isLoading: false });
  }
},
```

### 解锁页面显示锁定状态

```typescript
// 更新 UnlockScreen.tsx

export function UnlockScreen() {
  const [lockStatus, setLockStatus] = useState<{
    isLocked: boolean;
    remainingSeconds?: number;
    attemptsLeft: number;
  }>({ isLocked: false, attemptsLeft: 5 });

  useEffect(() => {
    // 检查锁定状态
    pinGuard.getStatus().then(status => {
      setLockStatus({
        isLocked: status.isLocked,
        remainingSeconds: status.lockoutRemainingSeconds,
        attemptsLeft: status.attemptsLeft,
      });
    });

    // 如果被锁定，启动倒计时
    if (lockStatus.isLocked && lockStatus.remainingSeconds) {
      const timer = setInterval(() => {
        setLockStatus(prev => {
          if (!prev.remainingSeconds || prev.remainingSeconds <= 1) {
            clearInterval(timer);
            return { ...prev, isLocked: false, remainingSeconds: undefined };
          }
          return { ...prev, remainingSeconds: prev.remainingSeconds - 1 };
        });
      }, 1000);

      return () => clearInterval(timer);
    }
  }, [lockStatus.isLocked]);

  // 渲染锁定倒计时
  if (lockStatus.isLocked && lockStatus.remainingSeconds) {
    const minutes = Math.floor(lockStatus.remainingSeconds / 60);
    const seconds = lockStatus.remainingSeconds % 60;

    return (
      <View style={styles.container}>
        <Text style={styles.lockIcon}>🔒</Text>
        <Text style={styles.lockTitle}>钱包已锁定</Text>
        <Text style={styles.lockTimer}>
          {minutes}:{seconds.toString().padStart(2, '0')}
        </Text>
        <Text style={styles.lockHint}>
          错误次数过多，请稍后再试
        </Text>
      </View>
    );
  }

  // ... 正常解锁界面
}
```

---

## 安全参数汇总

| 参数 | 值 | 说明 |
|------|-----|------|
| PBKDF2 迭代次数 | 600,000 | OWASP 2024 推荐 |
| 盐长度 | 32 bytes | 256 bits |
| AES 密钥长度 | 32 bytes | AES-256 |
| GCM IV 长度 | 12 bytes | 96 bits (推荐) |
| PIN 最大错误次数 | 5 次 | 触发临时锁定 |
| 锁定时间梯度 | 1/5/15/60/1440 分钟 | 递增惩罚 |
| 永久锁定阈值 | 15 次 | 需要助记词恢复 |
| 错误计数重置 | 24 小时 | 无错误后重置 |
