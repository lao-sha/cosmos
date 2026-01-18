// frontend/src/divination/market/services/e2e-encryption.service.ts

import { Platform } from 'react-native';

// 加密配置
const ENCRYPTION_ALGORITHM = 'AES-GCM';
const KEY_LENGTH = 256;
const IV_LENGTH = 12;
const SALT_LENGTH = 16;

/**
 * 生成随机字节
 */
function getRandomBytes(length: number): Uint8Array {
  if (Platform.OS === 'web' && typeof window !== 'undefined' && window.crypto) {
    return window.crypto.getRandomValues(new Uint8Array(length));
  }
  // React Native 环境
  const bytes = new Uint8Array(length);
  for (let i = 0; i < length; i++) {
    bytes[i] = Math.floor(Math.random() * 256);
  }
  return bytes;
}

/**
 * 从密码派生密钥
 */
async function deriveKey(password: string, salt: Uint8Array): Promise<CryptoKey> {
  const encoder = new TextEncoder();
  const passwordKey = await crypto.subtle.importKey(
    'raw',
    encoder.encode(password),
    'PBKDF2',
    false,
    ['deriveBits', 'deriveKey']
  );

  return crypto.subtle.deriveKey(
    {
      name: 'PBKDF2',
      salt,
      iterations: 100000,
      hash: 'SHA-256',
    },
    passwordKey,
    { name: ENCRYPTION_ALGORITHM, length: KEY_LENGTH },
    false,
    ['encrypt', 'decrypt']
  );
}

/**
 * 加密数据
 */
export async function encryptData(plaintext: string, password: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(plaintext);

  const salt = getRandomBytes(SALT_LENGTH);
  const iv = getRandomBytes(IV_LENGTH);
  const key = await deriveKey(password, salt);

  const encrypted = await crypto.subtle.encrypt(
    { name: ENCRYPTION_ALGORITHM, iv },
    key,
    data
  );

  // 组合: salt + iv + ciphertext
  const result = new Uint8Array(salt.length + iv.length + encrypted.byteLength);
  result.set(salt, 0);
  result.set(iv, salt.length);
  result.set(new Uint8Array(encrypted), salt.length + iv.length);

  // 转为 base64
  return btoa(String.fromCharCode(...result));
}

/**
 * 解密数据
 */
export async function decryptData(ciphertext: string, password: string): Promise<string> {
  // 从 base64 解码
  const data = Uint8Array.from(atob(ciphertext), (c) => c.charCodeAt(0));

  const salt = data.slice(0, SALT_LENGTH);
  const iv = data.slice(SALT_LENGTH, SALT_LENGTH + IV_LENGTH);
  const encrypted = data.slice(SALT_LENGTH + IV_LENGTH);

  const key = await deriveKey(password, salt);

  const decrypted = await crypto.subtle.decrypt(
    { name: ENCRYPTION_ALGORITHM, iv },
    key,
    encrypted
  );

  const decoder = new TextDecoder();
  return decoder.decode(decrypted);
}

/**
 * 使用共享密钥加密
 */
export async function encryptWithSharedKey(
  plaintext: string,
  sharedKey: Uint8Array
): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(plaintext);

  const iv = getRandomBytes(IV_LENGTH);

  const key = await crypto.subtle.importKey(
    'raw',
    sharedKey,
    { name: ENCRYPTION_ALGORITHM },
    false,
    ['encrypt']
  );

  const encrypted = await crypto.subtle.encrypt(
    { name: ENCRYPTION_ALGORITHM, iv },
    key,
    data
  );

  // 组合: iv + ciphertext
  const result = new Uint8Array(iv.length + encrypted.byteLength);
  result.set(iv, 0);
  result.set(new Uint8Array(encrypted), iv.length);

  return btoa(String.fromCharCode(...result));
}

/**
 * 使用共享密钥解密
 */
export async function decryptWithSharedKey(
  ciphertext: string,
  sharedKey: Uint8Array
): Promise<string> {
  const data = Uint8Array.from(atob(ciphertext), (c) => c.charCodeAt(0));

  const iv = data.slice(0, IV_LENGTH);
  const encrypted = data.slice(IV_LENGTH);

  const key = await crypto.subtle.importKey(
    'raw',
    sharedKey,
    { name: ENCRYPTION_ALGORITHM },
    false,
    ['decrypt']
  );

  const decrypted = await crypto.subtle.decrypt(
    { name: ENCRYPTION_ALGORITHM, iv },
    key,
    encrypted
  );

  const decoder = new TextDecoder();
  return decoder.decode(decrypted);
}

/**
 * 生成随机会话密钥
 */
export function generateSessionKey(): Uint8Array {
  return getRandomBytes(32); // 256 bits
}

/**
 * 将 Uint8Array 转换为十六进制字符串
 */
export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

/**
 * 将十六进制字符串转换为 Uint8Array
 */
export function hexToBytes(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(hex.substr(i * 2, 2), 16);
  }
  return bytes;
}
