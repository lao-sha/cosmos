import { httpServer } from './app.js';
import { config, validateConfig } from './config/index.js';
import { logger } from './utils/logger.js';
import { chainService } from './services/chain.service.js';
import { cacheService } from './services/cache.service.js';
import { chainEventsService, setWebSocketService } from './services/chain-events.service.js';
import { initWebSocketService } from './websocket.js';

async function main() {
  try {
    // 验证配置
    validateConfig();

    // 连接 Redis
    const redisOk = await cacheService.ping();
    if (!redisOk) {
      logger.warn('Redis not available, running without cache');
    } else {
      logger.info('Redis connected');
    }

    // 连接链
    try {
      await chainService.connect();
    } catch (error) {
      logger.warn({ error }, 'Chain connection failed, some features may not work');
    }

    // 初始化 WebSocket 服务
    const wsService = initWebSocketService(httpServer);
    setWebSocketService(wsService);

    // 启动链上事件监听 (仅在链连接成功时)
    if (chainService.isConnected()) {
      await chainEventsService.startListening();
    }

    // 启动 HTTP + WebSocket 服务
    httpServer.listen(config.port, () => {
      logger.info({ port: config.port }, 'Server started (HTTP + WebSocket)');
    });

    // 优雅关闭
    process.on('SIGTERM', async () => {
      logger.info('SIGTERM received, shutting down...');
      chainEventsService.stopListening();
      await cacheService.disconnect();
      httpServer.close(() => {
        logger.info('Server closed');
        process.exit(0);
      });
    });

    process.on('SIGINT', async () => {
      logger.info('SIGINT received, shutting down...');
      chainEventsService.stopListening();
      await cacheService.disconnect();
      httpServer.close(() => {
        logger.info('Server closed');
        process.exit(0);
      });
    });

  } catch (error) {
    logger.error({ error }, 'Failed to start server');
    process.exit(1);
  }
}

main();
