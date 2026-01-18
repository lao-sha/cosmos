"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.tokenRateLimitMiddleware = exports.rateLimitMiddleware = void 0;
const express_rate_limit_1 = __importDefault(require("express-rate-limit"));
const index_js_1 = require("../config/index.js");
exports.rateLimitMiddleware = (0, express_rate_limit_1.default)({
    windowMs: index_js_1.config.security.rateLimitWindow,
    max: index_js_1.config.security.rateLimitMax,
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
exports.tokenRateLimitMiddleware = (0, express_rate_limit_1.default)({
    windowMs: 60 * 1000, // 1 分钟
    max: 10, // 每分钟最多 10 次
    message: {
        error: {
            code: 'RATE_LIMIT_EXCEEDED',
            message: 'Too many token requests, please try again later',
        },
    },
});
//# sourceMappingURL=rateLimit.middleware.js.map