// frontend/src/divination/market/services/key-exchange.service.ts

/**
 * 密钥交换服务
 * 基于 X25519 ECDH 进行密钥协商
 */

// X25519 密钥对
export interface KeyPair {
  publicKey: Uint8Array;
  privateKey: Uint8Array;
}

/**
 * 生成 X25519 密钥对
 * 注意：实际实现需要使用 tweetnacl 或 @noble/curves 库
 */
export async function generateKeyPair(): Promise<KeyPair> {
  // 使用 Web Crypto API 生成 ECDH 密钥对
  const keyPair = await crypto.subtle.generateKey(
    {
      name: 'ECDH',
      namedCurve: 'P-256', // X25519 需要专用库，这里使用 P-256 作为替代
    },
    true,
    ['deriveBits']
  );

  const publicKeyBuffer = await crypto.subtle.exportKey('raw', keyPair.publicKey);
  const privateKeyBuffer = await crypto.subtle.exportKey('pkcs8', keyPair.privateKey);

  return {
    publicKey: new Uint8Array(publicKeyBuffer),
    privateKey: new Uint8Array(privateKeyBuffer),
  };
}

/**
 * 计算共享密钥
 */
export async function computeSharedSecret(
  privateKey: Uint8Array,
  peerPublicKey: Uint8Array
): Promise<Uint8Array> {
  // 导入私钥
  const importedPrivateKey = await crypto.subtle.importKey(
    'pkcs8',
    privateKey,
    { name: 'ECDH', namedCurve: 'P-256' },
    false,
    ['deriveBits']
  );

  // 导入对方公钥
  const importedPeerPublicKey = await crypto.subtle.importKey(
    'raw',
    peerPublicKey,
    { name: 'ECDH', namedCurve: 'P-256' },
    false,
    []
  );

  // 计算共享密钥
  const sharedBits = await crypto.subtle.deriveBits(
    {
      name: 'ECDH',
      public: importedPeerPublicKey,
    },
    importedPrivateKey,
    256
  );

  return new Uint8Array(sharedBits);
}

/**
 * 密钥派生 - 从共享密钥派生加密密钥
 */
export async function deriveEncryptionKey(
  sharedSecret: Uint8Array,
  context: string
): Promise<Uint8Array> {
  const encoder = new TextEncoder();

  // 使用 HKDF 派生密钥
  const baseKey = await crypto.subtle.importKey(
    'raw',
    sharedSecret,
    'HKDF',
    false,
    ['deriveBits']
  );

  const derivedBits = await crypto.subtle.deriveBits(
    {
      name: 'HKDF',
      hash: 'SHA-256',
      salt: encoder.encode('stardust-market'),
      info: encoder.encode(context),
    },
    baseKey,
    256
  );

  return new Uint8Array(derivedBits);
}

/**
 * 公钥转为十六进制字符串（用于链上存储）
 */
export function publicKeyToHex(publicKey: Uint8Array): string {
  return Array.from(publicKey)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

/**
 * 从十六进制字符串恢复公钥
 */
export function hexToPublicKey(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(hex.substr(i * 2, 2), 16);
  }
  return bytes;
}

/**
 * 会话密钥协商上下文
 */
export interface KeyExchangeSession {
  sessionId: string;
  localKeyPair: KeyPair;
  peerPublicKey?: Uint8Array;
  sharedSecret?: Uint8Array;
  encryptionKey?: Uint8Array;
  createdAt: number;
}

/**
 * 创建密钥交换会话
 */
export async function createKeyExchangeSession(): Promise<KeyExchangeSession> {
  const keyPair = await generateKeyPair();
  const sessionId = crypto.randomUUID();

  return {
    sessionId,
    localKeyPair: keyPair,
    createdAt: Date.now(),
  };
}

/**
 * 完成密钥交换
 */
export async function completeKeyExchange(
  session: KeyExchangeSession,
  peerPublicKey: Uint8Array
): Promise<KeyExchangeSession> {
  const sharedSecret = await computeSharedSecret(
    session.localKeyPair.privateKey,
    peerPublicKey
  );

  const encryptionKey = await deriveEncryptionKey(sharedSecret, session.sessionId);

  return {
    ...session,
    peerPublicKey,
    sharedSecret,
    encryptionKey,
  };
}
