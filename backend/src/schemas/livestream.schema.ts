import { z } from 'zod';

// SS58 地址正则 (Substrate 地址格式)
const ss58AddressRegex = /^[1-9A-HJ-NP-Za-km-z]{47,48}$/;

// 签名正则 (0x 开头的十六进制)
const signatureRegex = /^0x[a-fA-F0-9]+$/;

/**
 * 主播推流 Token 请求
 */
export const publisherTokenSchema = z.object({
  body: z.object({
    roomId: z.number().int().positive('Room ID must be a positive integer'),
    publicKey: z.string().regex(ss58AddressRegex, 'Invalid SS58 address format'),
    signature: z.string().regex(signatureRegex, 'Invalid signature format'),
    timestamp: z.number().int().positive('Timestamp must be a positive integer'),
  }),
});

/**
 * 观众观看 Token 请求
 */
export const viewerTokenSchema = z.object({
  body: z.object({
    roomId: z.number().int().positive('Room ID must be a positive integer'),
    viewerAddress: z.string().regex(ss58AddressRegex, 'Invalid SS58 address format'),
    signature: z.string().regex(signatureRegex, 'Invalid signature format'),
    timestamp: z.number().int().positive('Timestamp must be a positive integer'),
  }),
});

/**
 * 连麦者 Token 请求
 */
export const coHostTokenSchema = z.object({
  body: z.object({
    roomId: z.number().int().positive('Room ID must be a positive integer'),
    coHostAddress: z.string().regex(ss58AddressRegex, 'Invalid SS58 address format'),
    signature: z.string().regex(signatureRegex, 'Invalid signature format'),
    timestamp: z.number().int().positive('Timestamp must be a positive integer'),
    type: z.enum(['audio', 'video']).optional().default('audio'),
  }),
});

/**
 * 观众离开请求
 */
export const viewerLeaveSchema = z.object({
  body: z.object({
    roomId: z.number().int().positive('Room ID must be a positive integer'),
    viewerAddress: z.string().regex(ss58AddressRegex, 'Invalid SS58 address format'),
    signature: z.string().regex(signatureRegex, 'Invalid signature format'),
    timestamp: z.number().int().positive('Timestamp must be a positive integer'),
  }),
});

export type PublisherTokenRequest = z.infer<typeof publisherTokenSchema>;
export type ViewerTokenRequest = z.infer<typeof viewerTokenSchema>;
export type CoHostTokenRequest = z.infer<typeof coHostTokenSchema>;
export type ViewerLeaveRequest = z.infer<typeof viewerLeaveSchema>;
