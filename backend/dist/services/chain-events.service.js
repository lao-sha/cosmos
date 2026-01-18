"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.chainEventsService = exports.ChainEventsService = void 0;
exports.setWebSocketService = setWebSocketService;
const chain_service_js_1 = require("./chain.service.js");
const cache_service_js_1 = require("./cache.service.js");
const logger_js_1 = require("../utils/logger.js");
// WebSocket 服务将在后面导入，避免循环依赖
let websocketService = null;
function setWebSocketService(ws) {
    websocketService = ws;
}
class ChainEventsService {
    unsubscribe = null;
    /**
     * 开始监听链上事件
     */
    async startListening() {
        const api = await chain_service_js_1.chainService.connect();
        logger_js_1.logger.info('Starting chain events listener...');
        // 订阅新区块事件
        const unsub = await api.query.system.events((events) => {
            events.forEach((record) => {
                const { event } = record;
                // 只处理 livestream 模块的事件
                if (event.section === 'livestream') {
                    this.handleLivestreamEvent(event);
                }
            });
        });
        this.unsubscribe = unsub;
        logger_js_1.logger.info('Chain events listener started');
    }
    /**
     * 停止监听
     */
    stopListening() {
        if (this.unsubscribe) {
            this.unsubscribe();
            this.unsubscribe = null;
            logger_js_1.logger.info('Chain events listener stopped');
        }
    }
    /**
     * 处理直播模块事件
     */
    async handleLivestreamEvent(event) {
        const eventName = event.method;
        const data = event.data.toJSON();
        logger_js_1.logger.info({ eventName, data }, 'Received livestream event');
        try {
            switch (eventName) {
                case 'RoomCreated':
                    await this.onRoomCreated(data);
                    break;
                case 'LiveStarted':
                    await this.onLiveStarted(data);
                    break;
                case 'LiveEnded':
                    await this.onLiveEnded(data);
                    break;
                case 'GiftSent':
                    await this.onGiftSent(data);
                    break;
                case 'CoHostStarted':
                    await this.onCoHostStarted(data);
                    break;
                case 'CoHostEnded':
                    await this.onCoHostEnded(data);
                    break;
                case 'ViewerKicked':
                    await this.onViewerKicked(data);
                    break;
                case 'ViewerUnbanned':
                    await this.onViewerUnbanned(data);
                    break;
                case 'RoomBanned':
                    await this.onRoomBanned(data);
                    break;
                default:
                    logger_js_1.logger.debug({ eventName }, 'Unhandled event');
            }
        }
        catch (error) {
            logger_js_1.logger.error({ error, eventName, data }, 'Error handling event');
        }
    }
    // ============ 事件处理器 ============
    async onRoomCreated(data) {
        const { host, roomId, roomType } = data;
        // 广播新直播间创建
        if (websocketService) {
            websocketService.broadcastToAll('room:created', {
                roomId,
                host,
                roomType,
            });
        }
    }
    async onLiveStarted(data) {
        const { roomId, startedAt } = data;
        // 清除缓存
        await cache_service_js_1.cacheService.invalidateRoom(roomId);
        // 广播开播通知
        if (websocketService) {
            websocketService.broadcastToRoom(roomId, 'live:started', {
                roomId,
                startedAt,
            });
            // 广播到大厅
            websocketService.broadcastToAll('room:live', { roomId });
        }
    }
    async onLiveEnded(data) {
        const { roomId, duration, totalViewers, peakViewers, totalGifts } = data;
        // 清除缓存
        await cache_service_js_1.cacheService.invalidateRoom(roomId);
        await cache_service_js_1.cacheService.resetViewerCount(roomId);
        // 广播结束通知
        if (websocketService) {
            websocketService.broadcastToRoom(roomId, 'live:ended', {
                roomId,
                duration,
                totalViewers,
                peakViewers,
                totalGifts,
            });
        }
    }
    async onGiftSent(data) {
        const { roomId, sender, receiver, giftId, quantity, value } = data;
        // 清除直播间缓存 (礼物总额变化)
        await cache_service_js_1.cacheService.invalidateRoom(roomId);
        // 广播礼物通知到直播间
        if (websocketService) {
            websocketService.broadcastToRoom(roomId, 'gift:received', {
                roomId,
                sender,
                receiver,
                giftId,
                quantity,
                value,
            });
        }
    }
    async onCoHostStarted(data) {
        const { roomId, coHost } = data;
        // 清除连麦者缓存
        await cache_service_js_1.cacheService.invalidateCoHosts(roomId);
        // 广播连麦开始
        if (websocketService) {
            websocketService.broadcastToRoom(roomId, 'cohost:started', {
                roomId,
                coHost,
            });
        }
    }
    async onCoHostEnded(data) {
        const { roomId, coHost } = data;
        // 清除连麦者缓存
        await cache_service_js_1.cacheService.invalidateCoHosts(roomId);
        // 广播连麦结束
        if (websocketService) {
            websocketService.broadcastToRoom(roomId, 'cohost:ended', {
                roomId,
                coHost,
            });
        }
    }
    async onViewerKicked(data) {
        const { roomId, viewer } = data;
        // 更新黑名单缓存
        await cache_service_js_1.cacheService.setBlacklisted(roomId, viewer, true);
        // 通知被踢用户
        if (websocketService) {
            websocketService.notifyUser(viewer, 'viewer:kicked', {
                roomId,
                message: 'You have been kicked from the room',
            });
        }
    }
    async onViewerUnbanned(data) {
        const { roomId, viewer } = data;
        // 更新黑名单缓存
        await cache_service_js_1.cacheService.setBlacklisted(roomId, viewer, false);
    }
    async onRoomBanned(data) {
        const { roomId, reason } = data;
        // 清除缓存
        await cache_service_js_1.cacheService.invalidateRoom(roomId);
        // 广播封禁通知
        if (websocketService) {
            websocketService.broadcastToRoom(roomId, 'room:banned', {
                roomId,
                reason,
            });
        }
    }
}
exports.ChainEventsService = ChainEventsService;
exports.chainEventsService = new ChainEventsService();
//# sourceMappingURL=chain-events.service.js.map