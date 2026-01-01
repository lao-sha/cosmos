/**
 * 星尘玄鉴 - 密钥存储（原生版本）
 * 使用真实的 @polkadot/keyring 和加密
 */

import * as SecureStore from 'expo-secure-store';
import { Keyring } from '@polkadot/keyring';
import { mnemonicGenerate, mnemonicValidate, cryptoWaitReady } from '@polkadot/util-crypto';
import type { KeyringPair } from '@polkadot/keyring/types';
import { encryptMnemonic, decryptMnemonic } from './crypto';
import { WalletError, AuthenticationError } from './errors';

const STORAGE_KEYS = {
  ENCRYPTED: 'stardust_encrypted',
  SALT: 'stardust_salt',
  IV: 'stardust_iv',
  ADDRESS: 'stardust_address',
} as const;

/**
 * 初始化加密库
 */
export async function initializeCrypto(): Promise<void> {
  try {
    await cryptoWaitReady();
    console.log('[Keystore] Crypto initialized (native)');
  } catch (error) {
    throw new WalletError('加密库初始化失败', error);
  }
}

/**
 * 生成新助记词
 */
export function generateMnemonic(): string {
  return mnemonicGenerate();
}

/**
 * 验证助记词
 */
export function validateMnemonic(mnemonic: string): boolean {
  return mnemonicValidate(mnemonic);
}

/**
 * 从助记词创建密钥对
 */
export function createKeyPairFromMnemonic(mnemonic: string): KeyringPair {
  try {
    const keyring = new Keyring({ type: 'sr25519' });
    return keyring.addFromMnemonic(mnemonic);
  } catch (error) {
    throw new WalletError('创建密钥对失败', error);
  }
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
    const { encrypted, salt, iv } = await encryptMnemonic(mnemonic, password);

    await SecureStore.setItemAsync(STORAGE_KEYS.ENCRYPTED, encrypted);
    await SecureStore.setItemAsync(STORAGE_KEYS.SALT, salt);
    await SecureStore.setItemAsync(STORAGE_KEYS.IV, iv);
    await SecureStore.setItemAsync(STORAGE_KEYS.ADDRESS, address);

    console.log('[Keystore] Mnemonic stored securely');
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
    const salt = await SecureStore.getItemAsync(STORAGE_KEYS.SALT);
    const iv = await SecureStore.getItemAsync(STORAGE_KEYS.IV);

    if (!encrypted || !salt || !iv) {
      throw new WalletError('未找到钱包数据');
    }

    return await decryptMnemonic(encrypted, salt, iv, password);
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
    console.log('[Keystore] Wallet deleted');
  } catch (error) {
    throw new WalletError('删除钱包失败', error);
  }
}
