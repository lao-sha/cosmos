"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const app_js_1 = require("./app.js");
const index_js_1 = require("./config/index.js");
const logger_js_1 = require("./utils/logger.js");
const chain_service_js_1 = require("./services/chain.service.js");
const cache_service_js_1 = require("./services/cache.service.js");
const chain_events_service_js_1 = require("./services/chain-events.service.js");
const websocket_js_1 = require("./websocket.js");
async function main() {
    try {
        // 验证配置
        (0, index_js_1.validateConfig)();
        // 连接 Redis
        const redisOk = await cache_service_js_1.cacheService.ping();
        if (!redisOk) {
            logger_js_1.logger.warn('Redis not available, running without cache');
        }
        else {
            logger_js_1.logger.info('Redis connected');
        }
        // 连接链
        try {
            await chain_service_js_1.chainService.connect();
        }
        catch (error) {
            logger_js_1.logger.warn({ error }, 'Chain connection failed, some features may not work');
        }
        // 初始化 WebSocket 服务
        const wsService = (0, websocket_js_1.initWebSocketService)(app_js_1.httpServer);
        (0, chain_events_service_js_1.setWebSocketService)(wsService);
        // 启动链上事件监听 (仅在链连接成功时)
        if (chain_service_js_1.chainService.isConnected()) {
            await chain_events_service_js_1.chainEventsService.startListening();
        }
        // 启动 HTTP + WebSocket 服务
        app_js_1.httpServer.listen(index_js_1.config.port, () => {
            logger_js_1.logger.info({ port: index_js_1.config.port }, 'Server started (HTTP + WebSocket)');
        });
        // 优雅关闭
        process.on('SIGTERM', async () => {
            logger_js_1.logger.info('SIGTERM received, shutting down...');
            chain_events_service_js_1.chainEventsService.stopListening();
            await cache_service_js_1.cacheService.disconnect();
            app_js_1.httpServer.close(() => {
                logger_js_1.logger.info('Server closed');
                process.exit(0);
            });
        });
        process.on('SIGINT', async () => {
            logger_js_1.logger.info('SIGINT received, shutting down...');
            chain_events_service_js_1.chainEventsService.stopListening();
            await cache_service_js_1.cacheService.disconnect();
            app_js_1.httpServer.close(() => {
                logger_js_1.logger.info('Server closed');
                process.exit(0);
            });
        });
    }
    catch (error) {
        logger_js_1.logger.error({ error }, 'Failed to start server');
        process.exit(1);
    }
}
main();
//# sourceMappingURL=index.js.map