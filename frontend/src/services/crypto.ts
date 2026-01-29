import { Buffer } from 'buffer';
import { Platform } from 'react-native';

const encoder = new TextEncoder();
const decoder = new TextDecoder();

async function getSubtleCrypto(): Promise<SubtleCrypto> {
  if (Platform.OS === 'web') {
    return window.crypto.subtle;
  }
  const { subtle } = await import('crypto');
  return subtle as unknown as SubtleCrypto;
}

async function generateRandomBytes(length: number): Promise<Uint8Array> {
  if (Platform.OS === 'web') {
    const bytes = new Uint8Array(length);
    window.crypto.getRandomValues(bytes);
    return bytes;
  }
  const { randomBytes } = await import('crypto');
  return new Uint8Array(randomBytes(length));
}

export class CryptoService {
  static async generateKeyPair(): Promise<CryptoKeyPair> {
    const subtle = await getSubtleCrypto();
    return await subtle.generateKey(
      {
        name: 'ECDH',
        namedCurve: 'P-256',
      },
      true,
      ['deriveKey', 'deriveBits']
    );
  }

  static async exportPublicKey(publicKey: CryptoKey): Promise<string> {
    const subtle = await getSubtleCrypto();
    const exported = await subtle.exportKey('spki', publicKey);
    return Buffer.from(exported).toString('base64');
  }

  static async importPublicKey(base64Key: string): Promise<CryptoKey> {
    const subtle = await getSubtleCrypto();
    const keyData = Buffer.from(base64Key, 'base64');
    return await subtle.importKey(
      'spki',
      keyData,
      {
        name: 'ECDH',
        namedCurve: 'P-256',
      },
      true,
      []
    );
  }

  static async deriveSharedKey(
    privateKey: CryptoKey,
    publicKey: CryptoKey
  ): Promise<CryptoKey> {
    const subtle = await getSubtleCrypto();
    return await subtle.deriveKey(
      {
        name: 'ECDH',
        public: publicKey,
      },
      privateKey,
      {
        name: 'AES-GCM',
        length: 256,
      },
      false,
      ['encrypt', 'decrypt']
    );
  }

  static async encrypt(plaintext: string, key: CryptoKey): Promise<string> {
    const subtle = await getSubtleCrypto();
    const iv = await generateRandomBytes(12);
    const data = encoder.encode(plaintext);

    const encrypted = await subtle.encrypt(
      {
        name: 'AES-GCM',
        iv,
      },
      key,
      data
    );

    const combined = new Uint8Array(iv.length + encrypted.byteLength);
    combined.set(iv);
    combined.set(new Uint8Array(encrypted), iv.length);

    return Buffer.from(combined).toString('base64');
  }

  static async decrypt(ciphertext: string, key: CryptoKey): Promise<string> {
    const subtle = await getSubtleCrypto();
    const combined = Buffer.from(ciphertext, 'base64');
    const iv = combined.slice(0, 12);
    const data = combined.slice(12);

    const decrypted = await subtle.decrypt(
      {
        name: 'AES-GCM',
        iv,
      },
      key,
      data
    );

    return decoder.decode(decrypted);
  }

  static async encryptWithPassword(plaintext: string, password: string): Promise<string> {
    const subtle = await getSubtleCrypto();
    const salt = await generateRandomBytes(16);
    const iv = await generateRandomBytes(12);

    const keyMaterial = await subtle.importKey(
      'raw',
      encoder.encode(password),
      'PBKDF2',
      false,
      ['deriveKey']
    );

    const key = await subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt,
        iterations: 100000,
        hash: 'SHA-256',
      },
      keyMaterial,
      { name: 'AES-GCM', length: 256 },
      false,
      ['encrypt']
    );

    const encrypted = await subtle.encrypt(
      { name: 'AES-GCM', iv },
      key,
      encoder.encode(plaintext)
    );

    const combined = new Uint8Array(salt.length + iv.length + encrypted.byteLength);
    combined.set(salt);
    combined.set(iv, salt.length);
    combined.set(new Uint8Array(encrypted), salt.length + iv.length);

    return Buffer.from(combined).toString('base64');
  }

  static async decryptWithPassword(ciphertext: string, password: string): Promise<string> {
    const subtle = await getSubtleCrypto();
    const combined = Buffer.from(ciphertext, 'base64');
    const salt = combined.slice(0, 16);
    const iv = combined.slice(16, 28);
    const data = combined.slice(28);

    const keyMaterial = await subtle.importKey(
      'raw',
      encoder.encode(password),
      'PBKDF2',
      false,
      ['deriveKey']
    );

    const key = await subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt,
        iterations: 100000,
        hash: 'SHA-256',
      },
      keyMaterial,
      { name: 'AES-GCM', length: 256 },
      false,
      ['decrypt']
    );

    const decrypted = await subtle.decrypt(
      { name: 'AES-GCM', iv },
      key,
      data
    );

    return decoder.decode(decrypted);
  }
}
