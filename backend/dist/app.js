"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.httpServer = void 0;
const express_1 = __importDefault(require("express"));
const cors_1 = __importDefault(require("cors"));
const helmet_1 = __importDefault(require("helmet"));
const http_1 = require("http");
const index_js_1 = require("./config/index.js");
const error_middleware_js_1 = require("./middleware/error.middleware.js");
const rateLimit_middleware_js_1 = require("./middleware/rateLimit.middleware.js");
const index_js_2 = __importDefault(require("./routes/index.js"));
const logger_js_1 = require("./utils/logger.js");
const chain_service_js_1 = require("./services/chain.service.js");
const cache_service_js_1 = require("./services/cache.service.js");
const livekit_service_js_1 = require("./services/livekit.service.js");
const app = (0, express_1.default)();
exports.httpServer = (0, http_1.createServer)(app);
// 基础中间件
app.use((0, helmet_1.default)());
app.use((0, cors_1.default)({ origin: index_js_1.config.cors.origin }));
app.use(express_1.default.json());
app.use(rateLimit_middleware_js_1.rateLimitMiddleware);
// 请求日志
app.use((req, _res, next) => {
    logger_js_1.logger.info({ method: req.method, path: req.path }, 'Request');
    next();
});
// 健康检查 (完整版)
app.get('/health', async (_req, res) => {
    const [redisOk, livekitOk] = await Promise.all([
        cache_service_js_1.cacheService.ping(),
        livekit_service_js_1.livekitService.healthCheck(),
    ]);
    const chainOk = chain_service_js_1.chainService.isConnected();
    const allOk = redisOk && livekitOk && chainOk;
    res.status(allOk ? 200 : 503).json({
        status: allOk ? 'ok' : 'degraded',
        timestamp: Date.now(),
        services: {
            redis: redisOk ? 'connected' : 'disconnected',
            livekit: livekitOk ? 'connected' : 'disconnected',
            chain: chainOk ? 'connected' : 'disconnected',
        },
    });
});
// API 路由
app.use('/api', index_js_2.default);
// 错误处理
app.use(error_middleware_js_1.errorMiddleware);
exports.default = app;
//# sourceMappingURL=app.js.map