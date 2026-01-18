"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.signatureService = exports.SignatureService = void 0;
const util_crypto_1 = require("@polkadot/util-crypto");
const index_js_1 = require("../config/index.js");
const logger_js_1 = require("../utils/logger.js");
class SignatureService {
    /**
     * 验证签名
     */
    verifySignature(message, signature, publicKey) {
        try {
            // 验证地址格式
            (0, util_crypto_1.decodeAddress)(publicKey);
            // 验证签名
            const result = (0, util_crypto_1.signatureVerify)(message, signature, publicKey);
            if (!result.isValid) {
                logger_js_1.logger.warn({ publicKey, message }, 'Invalid signature');
                return { isValid: false, error: 'Invalid signature' };
            }
            return { isValid: true };
        }
        catch (error) {
            logger_js_1.logger.error({ error, publicKey }, 'Signature verification failed');
            return { isValid: false, error: 'Signature verification failed' };
        }
    }
    /**
     * 验证时间戳 (防重放攻击)
     */
    verifyTimestamp(timestamp) {
        const now = Date.now();
        const maxAge = index_js_1.config.security.signatureMaxAge;
        if (Math.abs(now - timestamp) > maxAge) {
            logger_js_1.logger.warn({ timestamp, now, maxAge }, 'Signature expired');
            return { isValid: false, error: 'Signature expired' };
        }
        return { isValid: true };
    }
    /**
     * 构造签名消息
     */
    buildSignatureMessage(roomId, timestamp) {
        return `livestream:${roomId}:${timestamp}`;
    }
    /**
     * 完整验证流程
     */
    verifyPublisherSignature(roomId, publicKey, signature, timestamp) {
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
exports.SignatureService = SignatureService;
exports.signatureService = new SignatureService();
//# sourceMappingURL=signature.service.js.map