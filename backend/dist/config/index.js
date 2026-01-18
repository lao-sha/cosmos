"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.config = void 0;
exports.validateConfig = validateConfig;
const dotenv_1 = __importDefault(require("dotenv"));
dotenv_1.default.config();
exports.config = {
    // 服务配置
    port: parseInt(process.env.PORT || '3001', 10),
    nodeEnv: process.env.NODE_ENV || 'development',
    // LiveKit 配置
    livekit: {
        url: process.env.LIVEKIT_URL || 'wss://localhost:7880',
        apiKey: process.env.LIVEKIT_API_KEY || '',
        apiSecret: process.env.LIVEKIT_API_SECRET || '',
        tokenTTL: process.env.LIVEKIT_TOKEN_TTL || '6h',
    },
    // Substrate 节点配置
    chain: {
        wsEndpoint: process.env.CHAIN_WS_ENDPOINT || 'ws://localhost:9944',
    },
    // Redis 配置
    redis: {
        url: process.env.REDIS_URL || 'redis://localhost:6379',
        keyPrefix: 'livestream:',
        ttl: {
            room: 60, // 直播间信息缓存 60 秒
            gift: 300, // 礼物信息缓存 5 分钟
            blacklist: 30, // 黑名单缓存 30 秒
            coHosts: 10, // 连麦者列表缓存 10 秒
        },
    },
    // 安全配置
    security: {
        signatureMaxAge: 5 * 60 * 1000, // 签名有效期 5 分钟
        rateLimitWindow: 60 * 1000, // 限流窗口 1 分钟
        rateLimitMax: 100, // 每窗口最大请求数
    },
    // CORS 配置
    cors: {
        origin: process.env.CORS_ORIGIN || '*',
    },
};
// 验证必要配置
function validateConfig() {
    const required = ['LIVEKIT_API_KEY', 'LIVEKIT_API_SECRET'];
    const missing = required.filter(key => !process.env[key]);
    if (missing.length > 0 && exports.config.nodeEnv === 'production') {
        throw new Error(`Missing required environment variables: ${missing.join(', ')}`);
    }
}
//# sourceMappingURL=index.js.map