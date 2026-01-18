/**
 * 加密服务
 * 使用 X25519 ECDH 进行密钥交换，AES-256-GCM 进行消息加密
 */

import { x25519 } from '@noble/curves/ed25519';
import { u8aToHex, hexToU8a } from '@polkadot/util';

/**
 * X25519 ECDH 密钥交换
 * 使用 @noble/curves 的 x25519 实现椭圆曲线 Diffie-Hellman
 *
 * 注意：sr25519 用于签名，x25519 用于密钥交换（两者基于相同曲线）
 * @noble/curves 是一个经过审计的纯 JavaScript 实现，安全可靠
 */
export async function deriveSharedKey(
  myPrivateKey: Uint8Array,
  peerPublicKey: Uint8Array
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
export function generateX25519KeyPair(): {
  privateKey: Uint8Array;
  publicKey: Uint8Array;
} {
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
      salt: new Uint8Array(32),
      info: new TextEncoder().encode(info),
    },
    cryptoKey,
    length * 8
  );

  return new Uint8Array(derivedBits);
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
 * 将公钥转换为十六进制字符串（用于存储和传输）
 */
export function publicKeyToHex(publicKey: Uint8Array): string {
  return u8aToHex(publicKey);
}

/**
 * 从十六进制字符串恢复公钥
 */
export function hexToPublicKey(hex: string): Uint8Array {
  return hexToU8a(hex);
}

/**
 * 将私钥转换为十六进制字符串（用于安全存储）
 */
export function privateKeyToHex(privateKey: Uint8Array): string {
  return u8aToHex(privateKey);
}

/**
 * 从十六进制字符串恢复私钥
 */
export function hexToPrivateKey(hex: string): Uint8Array {
  return hexToU8a(hex);
}

/**
 * 生成随机字节
 */
export function randomBytes(length: number): Uint8Array {
  return crypto.getRandomValues(new Uint8Array(length));
}

/**
 * 计算 SHA-256 哈希
 */
export async function sha256(data: Uint8Array): Promise<Uint8Array> {
  const hash = await crypto.subtle.digest('SHA-256', data);
  return new Uint8Array(hash);
}
