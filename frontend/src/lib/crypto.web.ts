/**
 * 星尘玄鉴 - 加密工具库（Mock 版本）
 * Web 测试用
 */

import * as Crypto from 'expo-crypto';
import { CryptoError } from './errors';

/**
 * 使用简单加密（Mock）
 */
export async function encryptMnemonic(
  mnemonic: string,
  password: string
): Promise<{
  encrypted: string;
  salt: string;
  iv: string;
}> {
  try {
    if (!password || password.length < 8) {
      throw new CryptoError('密码至少需要 8 位');
    }

    // Mock: 简单 base64 编码
    const encrypted = btoa(mnemonic);
    const salt = 'salt_' + Date.now();
    const iv = 'iv_' + Date.now();

    return { encrypted, salt, iv };
  } catch (error) {
    throw new CryptoError('加密失败', error);
  }
}

/**
 * 解密助记词（Mock）
 */
export async function decryptMnemonic(
  encrypted: string,
  salt: string,
  iv: string,
  password: string
): Promise<string> {
  try {
    // Mock: 简单 base64 解码
    return atob(encrypted);
  } catch (error) {
    throw new CryptoError('解密失败', error);
  }
}

/**
 * 生成安全的随机字节
 */
export function generateRandomBytes(length: number): Uint8Array {
  const arr = new Uint8Array(length);
  for (let i = 0; i < length; i++) {
    arr[i] = Math.floor(Math.random() * 256);
  }
  return arr;
}

/**
 * 计算 SHA-256 哈希
 */
export async function sha256(data: string): Promise<string> {
  return await Crypto.digestStringAsync(
    Crypto.CryptoDigestAlgorithm.SHA256,
    data
  );
}

/**
 * 验证密码
 */
export function verifyPassword(input: string, stored: string): boolean {
  return input === stored;
}
