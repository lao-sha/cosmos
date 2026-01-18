"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.livestreamController = exports.LivestreamController = void 0;
const livekit_service_js_1 = require("../services/livekit.service.js");
const signature_service_js_1 = require("../services/signature.service.js");
const chain_service_js_1 = require("../services/chain.service.js");
const cache_service_js_1 = require("../services/cache.service.js");
const error_middleware_js_1 = require("../middleware/error.middleware.js");
const logger_js_1 = require("../utils/logger.js");
class LivestreamController {
    /**
     * 获取主播推流 Token
     * POST /api/livestream/publisher-token
     */
    async getPublisherToken(req, res, next) {
        try {
            const { roomId, publicKey, signature, timestamp } = req.body;
            // 1. 验证签名
            const signResult = signature_service_js_1.signatureService.verifyPublisherSignature(roomId, publicKey, signature, timestamp);
            if (!signResult.isValid) {
                const errorCode = signResult.error?.includes('expired')
                    ? 'SIGNATURE_EXPIRED'
                    : 'INVALID_SIGNATURE';
                throw new error_middleware_js_1.AppError(errorCode, signResult.error || 'Signature verification failed', 401);
            }
            // 2. 查询链上直播间
            const room = await chain_service_js_1.chainService.getRoom(roomId);
            if (!room) {
                throw new error_middleware_js_1.AppError('ROOM_NOT_FOUND', 'Room not found', 404);
            }
            // 3. 验证是否是主播
            if (room.host !== publicKey) {
                throw new error_middleware_js_1.AppError('NOT_ROOM_HOST', 'You are not the host of this room', 403);
            }
            // 4. 验证直播间状态
            if (room.status !== 'Live' && room.status !== 'Preparing') {
                throw new error_middleware_js_1.AppError('ROOM_NOT_AVAILABLE', 'Room is not available for streaming', 400);
            }
            // 5. 生成 Token
            const token = await livekit_service_js_1.livekitService.generatePublisherToken(roomId, publicKey);
            logger_js_1.logger.info({ roomId, host: publicKey }, 'Publisher token generated');
            res.json({
                token,
                url: livekit_service_js_1.livekitService.getServerUrl(),
                roomName: `room-${roomId}`,
                expiresAt: livekit_service_js_1.livekitService.getTokenExpiry(),
            });
        }
        catch (error) {
            next(error);
        }
    }
    /**
     * 获取观众观看 Token
     * POST /api/livestream/viewer-token
     */
    async getViewerToken(req, res, next) {
        try {
            const { roomId, viewerAddress, signature, timestamp } = req.body;
            // 1. 验证签名 (防止身份伪造)
            const signResult = signature_service_js_1.signatureService.verifyPublisherSignature(roomId, viewerAddress, signature, timestamp);
            if (!signResult.isValid) {
                const errorCode = signResult.error?.includes('expired')
                    ? 'SIGNATURE_EXPIRED'
                    : 'INVALID_SIGNATURE';
                throw new error_middleware_js_1.AppError(errorCode, signResult.error || 'Signature verification failed', 401);
            }
            // 2. 查询链上直播间
            const room = await chain_service_js_1.chainService.getRoom(roomId);
            if (!room) {
                throw new error_middleware_js_1.AppError('ROOM_NOT_FOUND', 'Room not found', 404);
            }
            // 3. 验证直播间状态
            if (room.status !== 'Live') {
                throw new error_middleware_js_1.AppError('ROOM_NOT_LIVE', 'Room is not live', 400);
            }
            // 4. 如果是付费直播，检查门票
            if (room.roomType === 'Paid') {
                const hasTicket = await chain_service_js_1.chainService.hasTicket(roomId, viewerAddress);
                if (!hasTicket) {
                    throw new error_middleware_js_1.AppError('TICKET_REQUIRED', 'You need to buy a ticket first', 402);
                }
            }
            // 5. 检查是否被封禁
            const isBanned = await chain_service_js_1.chainService.isViewerBanned(roomId, viewerAddress);
            if (isBanned) {
                throw new error_middleware_js_1.AppError('VIEWER_BANNED', 'You are banned from this room', 403);
            }
            // 6. 生成 Token
            const token = await livekit_service_js_1.livekitService.generateViewerToken(roomId, viewerAddress);
            // 7. 增加在线观众计数
            await cache_service_js_1.cacheService.incrViewerCount(roomId);
            logger_js_1.logger.info({ roomId, viewer: viewerAddress }, 'Viewer token generated');
            res.json({
                token,
                url: livekit_service_js_1.livekitService.getServerUrl(),
                roomName: `room-${roomId}`,
                expiresAt: livekit_service_js_1.livekitService.getTokenExpiry(),
            });
        }
        catch (error) {
            next(error);
        }
    }
    /**
     * 获取连麦者 Token
     * POST /api/livestream/co-host-token
     */
    async getCoHostToken(req, res, next) {
        try {
            const { roomId, coHostAddress, signature, timestamp, type } = req.body;
            // 1. 验证签名
            const signResult = signature_service_js_1.signatureService.verifyPublisherSignature(roomId, coHostAddress, signature, timestamp);
            if (!signResult.isValid) {
                const errorCode = signResult.error?.includes('expired')
                    ? 'SIGNATURE_EXPIRED'
                    : 'INVALID_SIGNATURE';
                throw new error_middleware_js_1.AppError(errorCode, signResult.error || 'Signature verification failed', 401);
            }
            // 2. 查询链上直播间
            const room = await chain_service_js_1.chainService.getRoom(roomId);
            if (!room) {
                throw new error_middleware_js_1.AppError('ROOM_NOT_FOUND', 'Room not found', 404);
            }
            // 3. 验证直播间状态
            if (room.status !== 'Live') {
                throw new error_middleware_js_1.AppError('ROOM_NOT_LIVE', 'Room is not live', 400);
            }
            // 4. 验证是否在连麦者列表中 (链上验证)
            const isCoHost = await chain_service_js_1.chainService.isCoHost(roomId, coHostAddress);
            if (!isCoHost) {
                throw new error_middleware_js_1.AppError('NOT_CO_HOST', 'You are not approved as co-host', 403);
            }
            // 5. 生成 Token
            const token = await livekit_service_js_1.livekitService.generateCoHostToken(roomId, coHostAddress);
            logger_js_1.logger.info({ roomId, coHost: coHostAddress, type }, 'Co-host token generated');
            res.json({
                token,
                url: livekit_service_js_1.livekitService.getServerUrl(),
                roomName: `room-${roomId}`,
                expiresAt: livekit_service_js_1.livekitService.getTokenExpiry(),
            });
        }
        catch (error) {
            next(error);
        }
    }
    /**
     * 获取直播间信息
     * GET /api/livestream/room/:roomId
     */
    async getRoom(req, res, next) {
        try {
            const roomId = parseInt(req.params.roomId, 10);
            if (isNaN(roomId)) {
                throw new error_middleware_js_1.AppError('INVALID_PARAMS', 'Invalid room ID', 400);
            }
            const room = await chain_service_js_1.chainService.getRoom(roomId);
            if (!room) {
                throw new error_middleware_js_1.AppError('ROOM_NOT_FOUND', 'Room not found', 404);
            }
            // 获取实时观众数
            const currentViewers = await cache_service_js_1.cacheService.getViewerCount(roomId);
            res.json({
                ...room,
                currentViewers,
            });
        }
        catch (error) {
            next(error);
        }
    }
    /**
     * 获取直播间列表
     * GET /api/livestream/rooms
     */
    async getRooms(req, res, next) {
        try {
            const { status, type, page, limit } = req.query;
            const result = await chain_service_js_1.chainService.getRooms({
                status: status,
                roomType: type,
                page: page ? parseInt(page, 10) : 1,
                limit: limit ? parseInt(limit, 10) : 20,
            });
            // 为每个直播间添加实时观众数
            const roomsWithViewers = await Promise.all(result.rooms.map(async (room) => ({
                ...room,
                currentViewers: await cache_service_js_1.cacheService.getViewerCount(room.id),
            })));
            res.json({
                rooms: roomsWithViewers,
                total: result.total,
                page: page ? parseInt(page, 10) : 1,
                limit: limit ? parseInt(limit, 10) : 20,
            });
        }
        catch (error) {
            next(error);
        }
    }
    /**
     * 观众离开直播间
     * POST /api/livestream/viewer-leave
     */
    async viewerLeave(req, res, next) {
        try {
            const { roomId, viewerAddress, signature, timestamp } = req.body;
            // 验证签名
            const signResult = signature_service_js_1.signatureService.verifyPublisherSignature(roomId, viewerAddress, signature, timestamp);
            if (!signResult.isValid) {
                throw new error_middleware_js_1.AppError('INVALID_SIGNATURE', 'Signature verification failed', 401);
            }
            // 减少在线观众计数
            await cache_service_js_1.cacheService.decrViewerCount(roomId);
            logger_js_1.logger.info({ roomId, viewer: viewerAddress }, 'Viewer left');
            res.json({ success: true });
        }
        catch (error) {
            next(error);
        }
    }
}
exports.LivestreamController = LivestreamController;
exports.livestreamController = new LivestreamController();
//# sourceMappingURL=livestream.controller.js.map