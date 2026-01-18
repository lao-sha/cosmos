import { Router } from 'express';
import { livestreamController } from '../controllers/livestream.controller.js';
import { validate } from '../middleware/validate.middleware.js';
import { tokenRateLimitMiddleware } from '../middleware/rateLimit.middleware.js';
import {
  publisherTokenSchema,
  viewerTokenSchema,
  coHostTokenSchema,
  viewerLeaveSchema,
} from '../schemas/livestream.schema.js';

const router = Router();

// Token 相关 (带限流和参数校验)
router.post(
  '/publisher-token',
  tokenRateLimitMiddleware,
  validate(publisherTokenSchema),
  (req, res, next) => livestreamController.getPublisherToken(req, res, next)
);

router.post(
  '/viewer-token',
  tokenRateLimitMiddleware,
  validate(viewerTokenSchema),
  (req, res, next) => livestreamController.getViewerToken(req, res, next)
);

router.post(
  '/co-host-token',
  tokenRateLimitMiddleware,
  validate(coHostTokenSchema),
  (req, res, next) => livestreamController.getCoHostToken(req, res, next)
);

router.post(
  '/viewer-leave',
  validate(viewerLeaveSchema),
  (req, res, next) => livestreamController.viewerLeave(req, res, next)
);

// 直播间信息
router.get('/room/:roomId', (req, res, next) =>
  livestreamController.getRoom(req, res, next)
);

router.get('/rooms', (req, res, next) =>
  livestreamController.getRooms(req, res, next)
);

export default router;
