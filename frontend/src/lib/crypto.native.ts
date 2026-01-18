/**
 * 星尘玄鉴 - 加密工具库（原生版本）
 * 完全使用 expo-crypto，不依赖 @polkadot
 */

import * as ExpoCrypto from 'expo-crypto';
import { CryptoError } from './errors';

// 工具函数：Uint8Array 转 hex
function u8aToHex(bytes: Uint8Array): string {
  return '0x' + Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

// 工具函数：hex 转 Uint8Array
function hexToU8a(hex: string): Uint8Array {
  const cleanHex = hex.startsWith('0x') ? hex.slice(2) : hex;
  const bytes = new Uint8Array(cleanHex.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(cleanHex.slice(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

// 工具函数：string 转 Uint8Array
function stringToU8a(str: string): Uint8Array {
  const encoder = new TextEncoder();
  return encoder.encode(str);
}

// 工具函数：Uint8Array 转 string
function u8aToString(bytes: Uint8Array): string {
  const decoder = new TextDecoder();
  return decoder.decode(bytes);
}

/**
 * 简单的密钥派生函数
 */
async function deriveKey(password: string, salt: Uint8Array): Promise<Uint8Array> {
  const passwordBytes = stringToU8a(password);
  let result = new Uint8Array([...passwordBytes, ...salt]);
  
  // 多轮哈希
  for (let i = 0; i < 1000; i++) {
    const hash = await ExpoCrypto.digestStringAsync(
      ExpoCrypto.CryptoDigestAlgorithm.SHA256,
      u8aToHex(result)
    );
    result = hexToU8a(hash);
  }
  
  return result;
}

/**
 * 加密助记词
 */
export async function encryptMnemonic(
  mnemonic: string,
  password: string
): Promise<{ encrypted: string; salt: string; iv: string }> {
  try {
    if (!password || password.length < 8) {
      throw new CryptoError('密码至少需要 8 位');
    }

    const salt = ExpoCrypto.getRandomBytes(32);
    const key = await deriveKey(password, salt);
    const iv = ExpoCrypto.getRandomBytes(12);
    const plaintext = stringToU8a(mnemonic);
    const ciphertext = new Uint8Array(plaintext.length);

    for (let i = 0; i < plaintext.length; i++) {
      ciphertext[i] = plaintext[i]! ^ key[i % key.length]!;
    }

    console.log('[Crypto] Mnemonic encrypted');

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
    const key = await deriveKey(password, hexToU8a(salt));
    const ciphertext = hexToU8a(encrypted);
    const plaintext = new Uint8Array(ciphertext.length);

    for (let i = 0; i < ciphertext.length; i++) {
      plaintext[i] = ciphertext[i]! ^ key[i % key.length]!;
    }

    console.log('[Crypto] Mnemonic decrypted');
    return u8aToString(plaintext);
  } catch (error) {
    throw new CryptoError('解密失败', error);
  }
}

export function generateRandomBytes(length: number): Uint8Array {
  return ExpoCrypto.getRandomBytes(length);
}

export async function sha256(data: string): Promise<string> {
  return await ExpoCrypto.digestStringAsync(
    ExpoCrypto.CryptoDigestAlgorithm.SHA256,
    data
  );
}
