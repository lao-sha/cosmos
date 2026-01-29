import { CryptoService } from './crypto';

const IPFS_GATEWAY = 'https://ipfs.io/ipfs/';
const IPFS_API = 'https://api.pinata.cloud/pinning/pinJSONToIPFS';

export interface EncryptedMessage {
  ciphertext: string;
  senderPubKey: string;
  timestamp: number;
  nonce: string;
}

export interface ChatMessagePayload {
  content: string;
  msgType: 'text' | 'image' | 'file';
  replyTo?: string;
  metadata?: Record<string, any>;
}

export class IpfsService {
  private static pinataApiKey?: string;
  private static pinataSecretKey?: string;

  static configure(apiKey: string, secretKey: string) {
    this.pinataApiKey = apiKey;
    this.pinataSecretKey = secretKey;
  }

  static async uploadToIpfs(data: any): Promise<string> {
    if (!this.pinataApiKey || !this.pinataSecretKey) {
      console.warn('IPFS not configured, using mock CID');
      const mockCid = 'Qm' + Math.random().toString(36).substring(2, 15) + 
                      Math.random().toString(36).substring(2, 15);
      return mockCid;
    }

    try {
      const response = await fetch(IPFS_API, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'pinata_api_key': this.pinataApiKey,
          'pinata_secret_api_key': this.pinataSecretKey,
        },
        body: JSON.stringify({
          pinataContent: data,
          pinataMetadata: {
            name: `cosmos-${Date.now()}`,
          },
        }),
      });

      if (!response.ok) {
        throw new Error(`IPFS upload failed: ${response.statusText}`);
      }

      const result = await response.json();
      return result.IpfsHash;
    } catch (error) {
      console.error('IPFS upload error:', error);
      throw error;
    }
  }

  static async fetchFromIpfs<T>(cid: string): Promise<T> {
    try {
      const response = await fetch(`${IPFS_GATEWAY}${cid}`);
      if (!response.ok) {
        throw new Error(`IPFS fetch failed: ${response.statusText}`);
      }
      return await response.json();
    } catch (error) {
      console.error('IPFS fetch error:', error);
      throw error;
    }
  }

  static async encryptAndUpload(
    message: ChatMessagePayload,
    sharedKey: CryptoKey
  ): Promise<string> {
    const plaintext = JSON.stringify(message);
    const ciphertext = await CryptoService.encrypt(plaintext, sharedKey);
    
    const encryptedMessage: EncryptedMessage = {
      ciphertext,
      senderPubKey: '',
      timestamp: Date.now(),
      nonce: Math.random().toString(36).substring(2),
    };

    return await this.uploadToIpfs(encryptedMessage);
  }

  static async fetchAndDecrypt(
    cid: string,
    sharedKey: CryptoKey
  ): Promise<ChatMessagePayload> {
    const encryptedMessage = await this.fetchFromIpfs<EncryptedMessage>(cid);
    const plaintext = await CryptoService.decrypt(encryptedMessage.ciphertext, sharedKey);
    return JSON.parse(plaintext);
  }

  static async uploadSimpleMessage(content: string): Promise<string> {
    const payload: ChatMessagePayload = {
      content,
      msgType: 'text',
      timestamp: Date.now(),
    } as any;
    
    return await this.uploadToIpfs(payload);
  }

  static getGatewayUrl(cid: string): string {
    return `${IPFS_GATEWAY}${cid}`;
  }
}
