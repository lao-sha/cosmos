"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.viewerLeaveSchema = exports.coHostTokenSchema = exports.viewerTokenSchema = exports.publisherTokenSchema = void 0;
const zod_1 = require("zod");
// SS58 地址正则 (Substrate 地址格式)
const ss58AddressRegex = /^[1-9A-HJ-NP-Za-km-z]{47,48}$/;
// 签名正则 (0x 开头的十六进制)
const signatureRegex = /^0x[a-fA-F0-9]+$/;
/**
 * 主播推流 Token 请求
 */
exports.publisherTokenSchema = zod_1.z.object({
    body: zod_1.z.object({
        roomId: zod_1.z.number().int().positive('Room ID must be a positive integer'),
        publicKey: zod_1.z.string().regex(ss58AddressRegex, 'Invalid SS58 address format'),
        signature: zod_1.z.string().regex(signatureRegex, 'Invalid signature format'),
        timestamp: zod_1.z.number().int().positive('Timestamp must be a positive integer'),
    }),
});
/**
 * 观众观看 Token 请求
 */
exports.viewerTokenSchema = zod_1.z.object({
    body: zod_1.z.object({
        roomId: zod_1.z.number().int().positive('Room ID must be a positive integer'),
        viewerAddress: zod_1.z.string().regex(ss58AddressRegex, 'Invalid SS58 address format'),
        signature: zod_1.z.string().regex(signatureRegex, 'Invalid signature format'),
        timestamp: zod_1.z.number().int().positive('Timestamp must be a positive integer'),
    }),
});
/**
 * 连麦者 Token 请求
 */
exports.coHostTokenSchema = zod_1.z.object({
    body: zod_1.z.object({
        roomId: zod_1.z.number().int().positive('Room ID must be a positive integer'),
        coHostAddress: zod_1.z.string().regex(ss58AddressRegex, 'Invalid SS58 address format'),
        signature: zod_1.z.string().regex(signatureRegex, 'Invalid signature format'),
        timestamp: zod_1.z.number().int().positive('Timestamp must be a positive integer'),
        type: zod_1.z.enum(['audio', 'video']).optional().default('audio'),
    }),
});
/**
 * 观众离开请求
 */
exports.viewerLeaveSchema = zod_1.z.object({
    body: zod_1.z.object({
        roomId: zod_1.z.number().int().positive('Room ID must be a positive integer'),
        viewerAddress: zod_1.z.string().regex(ss58AddressRegex, 'Invalid SS58 address format'),
        signature: zod_1.z.string().regex(signatureRegex, 'Invalid signature format'),
        timestamp: zod_1.z.number().int().positive('Timestamp must be a positive integer'),
    }),
});
//# sourceMappingURL=livestream.schema.js.map