/**
 * ECIES 解密库 - 用于解密 PublicEncrypted 模式的占卜结果
 *
 * 加密格式：
 * +------------------+------------------+------------------+
 * | ephemeral_pubkey |      nonce       | ciphertext + tag |
 * |    (32 bytes)    |   (12 bytes)     |    (N bytes)     |
 * +------------------+------------------+------------------+
 *
 * 算法：
 * - 密钥交换: X25519 ECDH
 * - 对称加密: ChaCha20-Poly1305 AEAD
 * - 密钥派生: HKDF (blake2b-256)
 */

import { blake2b } from '@noble/hashes/blake2b';
import { x25519 } from '@noble/curves/ed25519';
import { chacha20poly1305 } from '@noble/ciphers/chacha';

/** 加密数据头部长度 */
const HEADER_LEN = 32 + 12; // ephemeral_pubkey + nonce
/** 认证标签长度 */
const AUTH_TAG_LEN = 16;

/**
 * 解密后的清单数据
 */
export interface DivinationManifest {
  type: string;
  version: number;
  input: Record<string, unknown>;
  result: Record<string, unknown>;
}

/**
 * 解密 ECIES 加密的数据
 *
 * @param encryptedData - 加密数据 (Uint8Array 或 hex 字符串)
 * @param privateKey - 用户 X25519 私钥 (32 bytes)
 * @returns 解密后的明文
 */
export function decryptEcies(
  encryptedData: Uint8Array | string,
  privateKey: Uint8Array
): Uint8Array {
  // 转换输入
  const data =
    typeof encryptedData === 'string'
      ? hexToBytes(encryptedData)
      : encryptedData;

  // 验证长度
  if (data.length < HEADER_LEN + AUTH_TAG_LEN) {
    throw new Error(
      `Encrypted data too short: ${data.length} bytes, minimum ${HEADER_LEN + AUTH_TAG_LEN}`
    );
  }

  // 解析加密数据
  const ephemeralPubkey = data.slice(0, 32);
  const nonce = data.slice(32, 44);
  const ciphertext = data.slice(44);

  // 计算共享密钥 (X25519 ECDH)
  const sharedSecret = x25519.getSharedSecret(privateKey, ephemeralPubkey);

  // 派生对称密钥
  const symmetricKey = deriveSymmetricKey(sharedSecret, ephemeralPubkey);

  // 使用 ChaCha20-Poly1305 解密
  const cipher = chacha20poly1305(symmetricKey, nonce);
  const plaintext = cipher.decrypt(ciphertext);

  return plaintext;
}

/**
 * 解密并解析 JSON 清单
 *
 * @param encryptedData - 加密数据
 * @param privateKey - 用户私钥
 * @returns 解析后的清单对象
 */
export function decryptManifest(
  encryptedData: Uint8Array | string,
  privateKey: Uint8Array
): DivinationManifest {
  const plaintext = decryptEcies(encryptedData, privateKey);
  const jsonStr = new TextDecoder().decode(plaintext);
  return JSON.parse(jsonStr) as DivinationManifest;
}

/**
 * 生成 X25519 密钥对
 *
 * @returns { privateKey, publicKey }
 */
export function generateKeyPair(): {
  privateKey: Uint8Array;
  publicKey: Uint8Array;
} {
  const privateKey = x25519.utils.randomPrivateKey();
  const publicKey = x25519.getPublicKey(privateKey);
  return { privateKey, publicKey };
}

/**
 * 从私钥派生公钥
 *
 * @param privateKey - X25519 私钥
 * @returns 公钥
 */
export function getPublicKey(privateKey: Uint8Array): Uint8Array {
  return x25519.getPublicKey(privateKey);
}

/**
 * 派生对称密钥 (与 Rust 端保持一致)
 */
function deriveSymmetricKey(
  sharedSecret: Uint8Array,
  ephemeralPubkey: Uint8Array
): Uint8Array {
  const input = new Uint8Array(
    sharedSecret.length + ephemeralPubkey.length + 16
  );
  input.set(sharedSecret, 0);
  input.set(ephemeralPubkey, sharedSecret.length);
  input.set(
    new TextEncoder().encode('cosmos-ecies-key'),
    sharedSecret.length + ephemeralPubkey.length
  );
  return blake2b(input, { dkLen: 32 });
}

/**
 * Hex 字符串转 Uint8Array
 */
function hexToBytes(hex: string): Uint8Array {
  const cleanHex = hex.startsWith('0x') ? hex.slice(2) : hex;
  const bytes = new Uint8Array(cleanHex.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(cleanHex.substr(i * 2, 2), 16);
  }
  return bytes;
}

/**
 * Uint8Array 转 Hex 字符串
 */
export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

/**
 * 使用示例
 *
 * ```typescript
 * import { generateKeyPair, decryptManifest, getPublicKey } from './ecies-decrypt';
 *
 * // 1. 生成密钥对（用户首次使用时）
 * const { privateKey, publicKey } = generateKeyPair();
 * // 保存 privateKey 到安全存储
 * // 使用 publicKey 提交请求
 *
 * // 2. 提交请求时
 * await api.tx.divinationOcwTee.createPublicEncryptedRequest(
 *   DivinationType.Meihua,
 *   inputData,
 *   publicKey
 * ).signAndSend(account);
 *
 * // 3. 获取结果后解密
 * const result = await api.query.divinationOcwTee.completedResults(requestId);
 * const encryptedData = await fetchFromIpfs(result.manifestCid);
 * const manifest = decryptManifest(encryptedData, privateKey);
 *
 * console.log('占卜结果:', manifest);
 * // {
 * //   type: 'meihua',
 * //   version: 1,
 * //   input: { method: 'Time', question: '...' },
 * //   result: { shang_gua: 5, xia_gua: 3, ... }
 * // }
 * ```
 */
