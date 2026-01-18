/**
 * 星尘玄鉴 - 安全存储服务
 * 使用 expo-secure-store 存储敏感数据
 * iOS: Keychain (Secure Enclave)
 * Android: Keystore + EncryptedSharedPreferences
 */

import { storageAdapter } from './storage-adapter';
import { u8aToHex, hexToU8a, stringToU8a } from '@polkadot/util';
import { pbkdf2Encode } from '@polkadot/util-crypto';
import type {
  WalletAccount,
  WalletSettings,
  EncryptedWalletData,
} from '@/features/wallet/types';
import {
  SECURITY_PARAMS,
  DEFAULT_WALLET_SETTINGS,
} from '@/features/wallet/types';

const STORAGE_KEYS = {
  WALLET_DATA: 'stardust_wallet_data',
  WALLET_SETTINGS: 'stardust_wallet_settings',
  PIN_HASH: 'stardust_pin_hash',
  BIOMETRIC_KEY: 'stardust_biometric_key',
  PIN_ATTEMPTS: 'stardust_pin_attempts',
} as const;

/**
 * PIN 哈希存储结构
 */
interface PinHashData {
  salt: string;
  hash: string;
  iterations: number;
}

/**
 * 安全存储服务
 */
export class SecureStorageService {
  private cachedEncryptionKey: Uint8Array | null = null;

  /**
   * 检查是否已初始化钱包
   */
  async isWalletInitialized(): Promise<boolean> {
    const data = await storageAdapter.getItemAsync(STORAGE_KEYS.WALLET_DATA);
    return data !== null;
  }

  /**
   * 验证 PIN 复杂度
   */
  validatePinComplexity(pin: string): { valid: boolean; error?: string } {
    if (pin.length < 6) {
      return { valid: false, error: 'PIN 码至少需要 6 位' };
    }

    if (!/^\d+$/.test(pin)) {
      return { valid: false, error: 'PIN 码只能包含数字' };
    }

    // 检查简单序列
    const weakPatterns = [
      '000000', '111111', '222222', '333333', '444444',
      '555555', '666666', '777777', '888888', '999999',
      '123456', '654321', '123123', '121212', '112233',
    ];

    if (weakPatterns.includes(pin)) {
      return { valid: false, error: 'PIN 码过于简单，请使用更复杂的组合' };
    }

    // 检查连续递增/递减
    let isSequential = true;
    for (let i = 1; i < pin.length; i++) {
      const curr = pin[i];
      const prev = pin[i - 1];
      if (curr === undefined || prev === undefined) {
        isSequential = false;
        break;
      }
      const diff = parseInt(curr, 10) - parseInt(prev, 10);
      if (Math.abs(diff) !== 1) {
        isSequential = false;
        break;
      }
    }
    if (isSequential && pin.length >= 4) {
      return { valid: false, error: 'PIN 码不能是连续数字' };
    }

    return { valid: true };
  }

  /**
   * 设置 PIN 码
   */
  async setPin(pin: string): Promise<void> {
    const validation = this.validatePinComplexity(pin);
    if (!validation.valid) {
      throw new Error(validation.error);
    }

    const salt = this.generateRandomBytes(SECURITY_PARAMS.SALT_LENGTH);
    const { password: hash } = pbkdf2Encode(
      stringToU8a(pin),
      salt,
      SECURITY_PARAMS.PBKDF2_ITERATIONS
    );

    const pinData: PinHashData = {
      salt: u8aToHex(salt),
      hash: u8aToHex(hash),
      iterations: SECURITY_PARAMS.PBKDF2_ITERATIONS,
    };

    await storageAdapter.setItemAsync(
      STORAGE_KEYS.PIN_HASH,
      JSON.stringify(pinData)
    );
  }

  /**
   * 验证 PIN 码
   */
  async verifyPin(pin: string): Promise<boolean> {
    const stored = await storageAdapter.getItemAsync(STORAGE_KEYS.PIN_HASH);
    if (!stored) return false;

    const { salt, hash, iterations }: PinHashData = JSON.parse(stored);
    const { password: computedHash } = pbkdf2Encode(
      stringToU8a(pin),
      hexToU8a(salt),
      iterations
    );

    return u8aToHex(computedHash) === hash;
  }

  /**
   * 从 PIN 派生加密密钥
   */
  async deriveKeyFromPin(pin: string): Promise<Uint8Array> {
    const stored = await storageAdapter.getItemAsync(STORAGE_KEYS.PIN_HASH);
    if (!stored) throw new Error('PIN not set');

    const { salt, iterations }: PinHashData = JSON.parse(stored);

    // 使用不同的后缀派生加密密钥（与验证 hash 不同）
    const { password: key } = pbkdf2Encode(
      stringToU8a(pin + ':encryption'),
      hexToU8a(salt),
      iterations
    );

    // 缓存加密密钥
    this.cachedEncryptionKey = key.slice(0, SECURITY_PARAMS.KEY_LENGTH);
    return this.cachedEncryptionKey;
  }

  /**
   * 获取缓存的加密密钥
   */
  getCachedEncryptionKey(): Uint8Array | null {
    return this.cachedEncryptionKey;
  }

  /**
   * 清除缓存的加密密钥
   */
  clearCachedKey(): void {
    if (this.cachedEncryptionKey) {
      // 安全清除内存
      this.cachedEncryptionKey.fill(0);
      this.cachedEncryptionKey = null;
    }
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
      encryptedData: encrypted,
      version: SECURITY_PARAMS.VERSION,
      createdAt: Date.now(),
      iterations: SECURITY_PARAMS.PBKDF2_ITERATIONS,
    };

    await storageAdapter.setItemAsync(
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
    const stored = await storageAdapter.getItemAsync(STORAGE_KEYS.WALLET_DATA);
    if (!stored) return null;

    const walletData: EncryptedWalletData = JSON.parse(stored);

    try {
      const decrypted = await this.decrypt(
        walletData.encryptedData,
        encryptionKey
      );
      return JSON.parse(decrypted);
    } catch {
      return null;
    }
  }

  /**
   * 保存钱包设置
   */
  async saveSettings(settings: WalletSettings): Promise<void> {
    await storageAdapter.setItemAsync(
      STORAGE_KEYS.WALLET_SETTINGS,
      JSON.stringify(settings)
    );
  }

  /**
   * 读取钱包设置
   */
  async loadSettings(): Promise<WalletSettings> {
    const stored = await storageAdapter.getItemAsync(STORAGE_KEYS.WALLET_SETTINGS);
    return stored ? JSON.parse(stored) : DEFAULT_WALLET_SETTINGS;
  }

  /**
   * 存储生物识别密钥
   */
  async saveBiometricKey(key: Uint8Array): Promise<void> {
    await storageAdapter.setItemAsync(
      STORAGE_KEYS.BIOMETRIC_KEY,
      u8aToHex(key),
      {
        requireAuthentication: true,
      }
    );
  }

  /**
   * 读取生物识别密钥
   */
  async loadBiometricKey(): Promise<Uint8Array | null> {
    try {
      const stored = await storageAdapter.getItemAsync(
        STORAGE_KEYS.BIOMETRIC_KEY,
        {
          requireAuthentication: true,
        }
      );
      return stored ? hexToU8a(stored) : null;
    } catch {
      return null;
    }
  }

  /**
   * 清除所有钱包数据
   */
  async clearAll(): Promise<void> {
    this.clearCachedKey();
    await Promise.all([
      storageAdapter.deleteItemAsync(STORAGE_KEYS.WALLET_DATA),
      storageAdapter.deleteItemAsync(STORAGE_KEYS.WALLET_SETTINGS),
      storageAdapter.deleteItemAsync(STORAGE_KEYS.PIN_HASH),
      storageAdapter.deleteItemAsync(STORAGE_KEYS.BIOMETRIC_KEY),
      storageAdapter.deleteItemAsync(STORAGE_KEYS.PIN_ATTEMPTS),
    ]);
  }

  /**
   * 生成随机字节
   */
  private generateRandomBytes(length: number): Uint8Array {
    const bytes = new Uint8Array(length);
    if (typeof crypto !== 'undefined' && crypto.getRandomValues) {
      crypto.getRandomValues(bytes);
    } else {
      // Fallback for environments without crypto
      for (let i = 0; i < length; i++) {
        bytes[i] = Math.floor(Math.random() * 256);
      }
    }
    return bytes;
  }

  /**
   * AES-256-GCM 加密
   */
  private async encrypt(plaintext: string, key: Uint8Array): Promise<string> {
    const iv = this.generateRandomBytes(SECURITY_PARAMS.IV_LENGTH);
    const encoder = new TextEncoder();

    const keySlice = key.slice(0, SECURITY_PARAMS.KEY_LENGTH);
    const cryptoKey = await crypto.subtle.importKey(
      'raw',
      new Uint8Array(keySlice),
      { name: 'AES-GCM' },
      false,
      ['encrypt']
    );

    const encrypted = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv: new Uint8Array(iv) },
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
    const iv = data.slice(0, SECURITY_PARAMS.IV_LENGTH);
    const encrypted = data.slice(SECURITY_PARAMS.IV_LENGTH);

    const keySlice = key.slice(0, SECURITY_PARAMS.KEY_LENGTH);
    const cryptoKey = await crypto.subtle.importKey(
      'raw',
      new Uint8Array(keySlice),
      { name: 'AES-GCM' },
      false,
      ['decrypt']
    );

    const decrypted = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv: new Uint8Array(iv) },
      cryptoKey,
      new Uint8Array(encrypted)
    );

    return new TextDecoder().decode(decrypted);
  }
}

// 单例
export const secureStorage = new SecureStorageService();
