import { signatureVerify, decodeAddress } from '@polkadot/util-crypto';
import { config } from '../config/index.js';
import { logger } from '../utils/logger.js';

export interface SignatureVerifyResult {
  isValid: boolean;
  error?: string;
}

export class SignatureService {
  /**
   * 验证签名
   */
  verifySignature(
    message: string,
    signature: string,
    publicKey: string
  ): SignatureVerifyResult {
    try {
      // 验证地址格式
      decodeAddress(publicKey);

      // 验证签名
      const result = signatureVerify(message, signature, publicKey);

      if (!result.isValid) {
        logger.warn({ publicKey, message }, 'Invalid signature');
        return { isValid: false, error: 'Invalid signature' };
      }

      return { isValid: true };
    } catch (error) {
      logger.error({ error, publicKey }, 'Signature verification failed');
      return { isValid: false, error: 'Signature verification failed' };
    }
  }

  /**
   * 验证时间戳 (防重放攻击)
   */
  verifyTimestamp(timestamp: number): SignatureVerifyResult {
    const now = Date.now();
    const maxAge = config.security.signatureMaxAge;

    if (Math.abs(now - timestamp) > maxAge) {
      logger.warn({ timestamp, now, maxAge }, 'Signature expired');
      return { isValid: false, error: 'Signature expired' };
    }

    return { isValid: true };
  }

  /**
   * 构造签名消息
   */
  buildSignatureMessage(roomId: number, timestamp: number): string {
    return `livestream:${roomId}:${timestamp}`;
  }

  /**
   * 完整验证流程
   */
  verifyPublisherSignature(
    roomId: number,
    publicKey: string,
    signature: string,
    timestamp: number
  ): SignatureVerifyResult {
    // 1. 验证时间戳
    const timestampResult = this.verifyTimestamp(timestamp);
    if (!timestampResult.isValid) {
      return timestampResult;
    }

    // 2. 构造消息
    const message = this.buildSignatureMessage(roomId, timestamp);

    // 3. 验证签名
    return this.verifySignature(message, signature, publicKey);
  }
}

export const signatureService = new SignatureService();
