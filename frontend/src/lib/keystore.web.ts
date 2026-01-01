/**
 * 星尘玄鉴 - 密钥存储（Mock 版本）
 * Web 测试用，移除 polkadot 依赖
 */

import * as SecureStore from './secureStore';
import { WalletError, AuthenticationError } from './errors';

const STORAGE_KEYS = {
  ENCRYPTED: 'stardust_encrypted',
  SALT: 'stardust_salt',
  IV: 'stardust_iv',
  ADDRESS: 'stardust_address',
} as const;

/**
 * 初始化加密库（Mock）
 */
export async function initializeCrypto(): Promise<void> {
  // Mock: 直接返回
  return Promise.resolve();
}

/**
 * 生成新助记词（Mock）
 */
export function generateMnemonic(): string {
  const words = [
    'abandon', 'ability', 'able', 'about', 'above', 'absent',
    'absorb', 'abstract', 'absurd', 'abuse', 'access', 'accident'
  ];
  return words.join(' ');
}

/**
 * 验证助记词（Mock）
 */
export function validateMnemonic(mnemonic: string): boolean {
  return mnemonic.split(' ').length >= 12;
}

/**
 * 从助记词创建密钥对（Mock）
 */
export function createKeyPairFromMnemonic(mnemonic: string): { address: string } {
  const address = '5' + Array.from({ length: 47 }, () =>
    'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'[Math.floor(Math.random() * 62)]
  ).join('');
  return { address };
}

/**
 * 安全存储加密的助记词
 */
export async function storeEncryptedMnemonic(
  mnemonic: string,
  password: string,
  address: string
): Promise<void> {
  try {
    // Mock: 简单 base64 编码
    const encrypted = btoa(mnemonic);
    const salt = 'mock_salt_' + Date.now();
    const iv = 'mock_iv_' + Date.now();

    await SecureStore.setItemAsync(STORAGE_KEYS.ENCRYPTED, encrypted);
    await SecureStore.setItemAsync(STORAGE_KEYS.SALT, salt);
    await SecureStore.setItemAsync(STORAGE_KEYS.IV, iv);
    await SecureStore.setItemAsync(STORAGE_KEYS.ADDRESS, address);
  } catch (error) {
    throw new WalletError('存储失败', error);
  }
}

/**
 * 恢复加密的助记词
 */
export async function retrieveEncryptedMnemonic(
  password: string
): Promise<string> {
  try {
    const encrypted = await SecureStore.getItemAsync(STORAGE_KEYS.ENCRYPTED);

    if (!encrypted) {
      throw new WalletError('未找到钱包数据');
    }

    // Mock: 简单 base64 解码
    return atob(encrypted);
  } catch (error) {
    if (error instanceof WalletError) {
      throw error;
    }
    throw new AuthenticationError('密码错误');
  }
}

/**
 * 检查是否存在钱包
 */
export async function hasWallet(): Promise<boolean> {
  try {
    const encrypted = await SecureStore.getItemAsync(STORAGE_KEYS.ENCRYPTED);
    return !!encrypted;
  } catch {
    return false;
  }
}

/**
 * 获取存储的地址
 */
export async function getStoredAddress(): Promise<string | null> {
  try {
    return await SecureStore.getItemAsync(STORAGE_KEYS.ADDRESS);
  } catch {
    return null;
  }
}

/**
 * 删除钱包数据
 */
export async function deleteWallet(): Promise<void> {
  try {
    await Promise.all([
      SecureStore.deleteItemAsync(STORAGE_KEYS.ENCRYPTED),
      SecureStore.deleteItemAsync(STORAGE_KEYS.SALT),
      SecureStore.deleteItemAsync(STORAGE_KEYS.IV),
      SecureStore.deleteItemAsync(STORAGE_KEYS.ADDRESS),
    ]);
  } catch (error) {
    throw new WalletError('删除钱包失败', error);
  }
}
