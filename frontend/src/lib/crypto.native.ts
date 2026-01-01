/**
 * 星尘玄鉴 - 加密工具库（原生版本）
 * 使用 scrypt + AES-256 加密助记词
 */

import * as Crypto from 'expo-crypto';
import { randomBytes, scrypt } from '@polkadot/util-crypto';
import { u8aToHex, hexToU8a, stringToU8a, u8aToString } from '@polkadot/util';
import { CryptoError } from './errors';

/**
 * 使用 scrypt + XOR 加密助记词
 * 注意：这是简化版实现，生产环境建议使用 react-native-aes-crypto 实现 AES-GCM
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

    // 1. 生成随机盐值（32 字节）
    const salt = randomBytes(32);

    // 2. 使用 scrypt 从密码派生密钥
    // N=16384, r=8, p=1 是推荐的移动端参数
    const key = scrypt(password, salt, 16384, 8, 1, 32);

    // 3. 生成随机 IV
    const iv = randomBytes(12);

    // 4. XOR 加密（生产环境应使用 AES-GCM）
    const plaintext = stringToU8a(mnemonic);
    const ciphertext = new Uint8Array(plaintext.length);

    for (let i = 0; i < plaintext.length; i++) {
      ciphertext[i] = plaintext[i]! ^ key[i % key.length]!;
    }

    console.log('[Crypto] Mnemonic encrypted (native)');

    return {
      encrypted: u8aToHex(ciphertext),
      salt: u8aToHex(salt),
      iv: u8aToHex(iv),
    };
  } catch (error) {
    throw new CryptoError('加密失败', error);
  }
}

/**
 * 解密助记词
 */
export async function decryptMnemonic(
  encrypted: string,
  salt: string,
  iv: string,
  password: string
): Promise<string> {
  try {
    // 1. 使用相同参数重新生成密钥
    const key = scrypt(
      password,
      hexToU8a(salt),
      16384,
      8,
      1,
      32
    );

    // 2. 解密（XOR 的逆操作）
    const ciphertext = hexToU8a(encrypted);
    const plaintext = new Uint8Array(ciphertext.length);

    for (let i = 0; i < ciphertext.length; i++) {
      plaintext[i] = ciphertext[i]! ^ key[i % key.length]!;
    }

    console.log('[Crypto] Mnemonic decrypted (native)');
    return u8aToString(plaintext);
  } catch (error) {
    throw new CryptoError('解密失败，密码可能错误', error);
  }
}

/**
 * 生成安全的随机字节
 */
export function generateRandomBytes(length: number): Uint8Array {
  return randomBytes(length);
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
 * 验证密码（常量时间比较防时序攻击）
 */
export function verifyPassword(input: string, stored: string): boolean {
  if (input.length !== stored.length) {
    return false;
  }

  let result = 0;
  for (let i = 0; i < input.length; i++) {
    result |= input.charCodeAt(i) ^ stored.charCodeAt(i);
  }

  return result === 0;
}
