import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import { createServer } from 'http';
import { config } from './config/index.js';
import { errorMiddleware } from './middleware/error.middleware.js';
import { rateLimitMiddleware } from './middleware/rateLimit.middleware.js';
import routes from './routes/index.js';
import { logger } from './utils/logger.js';
import { chainService } from './services/chain.service.js';
import { cacheService } from './services/cache.service.js';
import { livekitService } from './services/livekit.service.js';

const app = express();
export const httpServer = createServer(app);

// 基础中间件
app.use(helmet());
app.use(cors({ origin: config.cors.origin }));
app.use(express.json());
app.use(rateLimitMiddleware);

// 请求日志
app.use((req, _res, next) => {
  logger.info({ method: req.method, path: req.path }, 'Request');
  next();
});

// 健康检查 (完整版)
app.get('/health', async (_req, res) => {
  const [redisOk, livekitOk] = await Promise.all([
    cacheService.ping(),
    livekitService.healthCheck(),
  ]);

  const chainOk = chainService.isConnected();
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
app.use('/api', routes);

// 错误处理
app.use(errorMiddleware);

export default app;
