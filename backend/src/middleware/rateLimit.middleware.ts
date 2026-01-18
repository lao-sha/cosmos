import rateLimit from 'express-rate-limit';
import { config } from '../config/index.js';

export const rateLimitMiddleware = rateLimit({
  windowMs: config.security.rateLimitWindow,
  max: config.security.rateLimitMax,
  message: {
    error: {
      code: 'RATE_LIMIT_EXCEEDED',
      message: 'Too many requests, please try again later',
    },
  },
  standardHeaders: true,
  legacyHeaders: false,
});

// Token 接口专用限流 (更严格)
export const tokenRateLimitMiddleware = rateLimit({
  windowMs: 60 * 1000, // 1 分钟
  max: 10,             // 每分钟最多 10 次
  message: {
    error: {
      code: 'RATE_LIMIT_EXCEEDED',
      message: 'Too many token requests, please try again later',
    },
  },
});
