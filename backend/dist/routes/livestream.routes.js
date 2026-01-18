"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const express_1 = require("express");
const livestream_controller_js_1 = require("../controllers/livestream.controller.js");
const validate_middleware_js_1 = require("../middleware/validate.middleware.js");
const rateLimit_middleware_js_1 = require("../middleware/rateLimit.middleware.js");
const livestream_schema_js_1 = require("../schemas/livestream.schema.js");
const router = (0, express_1.Router)();
// Token 相关 (带限流和参数校验)
router.post('/publisher-token', rateLimit_middleware_js_1.tokenRateLimitMiddleware, (0, validate_middleware_js_1.validate)(livestream_schema_js_1.publisherTokenSchema), (req, res, next) => livestream_controller_js_1.livestreamController.getPublisherToken(req, res, next));
router.post('/viewer-token', rateLimit_middleware_js_1.tokenRateLimitMiddleware, (0, validate_middleware_js_1.validate)(livestream_schema_js_1.viewerTokenSchema), (req, res, next) => livestream_controller_js_1.livestreamController.getViewerToken(req, res, next));
router.post('/co-host-token', rateLimit_middleware_js_1.tokenRateLimitMiddleware, (0, validate_middleware_js_1.validate)(livestream_schema_js_1.coHostTokenSchema), (req, res, next) => livestream_controller_js_1.livestreamController.getCoHostToken(req, res, next));
router.post('/viewer-leave', (0, validate_middleware_js_1.validate)(livestream_schema_js_1.viewerLeaveSchema), (req, res, next) => livestream_controller_js_1.livestreamController.viewerLeave(req, res, next));
// 直播间信息
router.get('/room/:roomId', (req, res, next) => livestream_controller_js_1.livestreamController.getRoom(req, res, next));
router.get('/rooms', (req, res, next) => livestream_controller_js_1.livestreamController.getRooms(req, res, next));
exports.default = router;
//# sourceMappingURL=livestream.routes.js.map