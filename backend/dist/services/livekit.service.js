"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.livekitService = exports.LiveKitService = void 0;
const livekit_server_sdk_1 = require("livekit-server-sdk");
const index_js_1 = require("../config/index.js");
const logger_js_1 = require("../utils/logger.js");
class LiveKitService {
    apiKey;
    apiSecret;
    url;
    roomService = null;
    constructor() {
        this.apiKey = index_js_1.config.livekit.apiKey;
        this.apiSecret = index_js_1.config.livekit.apiSecret;
        this.url = index_js_1.config.livekit.url;
        // 初始化 Room Service 客户端 (仅在配置完整时)
        if (this.apiKey && this.apiSecret) {
            const httpUrl = this.url.replace('wss://', 'https://').replace('ws://', 'http://');
            this.roomService = new livekit_server_sdk_1.RoomServiceClient(httpUrl, this.apiKey, this.apiSecret);
        }
    }
    /**
     * 生成 LiveKit 访问 Token
     */
    async generateToken(options) {
        const { identity, name, roomName, canPublish, canSubscribe, canPublishData } = options;
        const token = new livekit_server_sdk_1.AccessToken(this.apiKey, this.apiSecret, {
            identity,
            name: name || identity,
            ttl: index_js_1.config.livekit.tokenTTL,
        });
        const grant = {
            room: roomName,
            roomJoin: true,
            canPublish,
            canSubscribe,
            canPublishData,
        };
        token.addGrant(grant);
        logger_js_1.logger.info({ identity, roomName, canPublish }, 'Generated LiveKit token');
        return await token.toJwt();
    }
    /**
     * 生成主播 Token
     */
    async generatePublisherToken(roomId, hostAddress) {
        return this.generateToken({
            identity: hostAddress,
            name: `Host-${roomId}`,
            roomName: `room-${roomId}`,
            canPublish: true,
            canSubscribe: true,
            canPublishData: true,
        });
    }
    /**
     * 生成观众 Token
     */
    async generateViewerToken(roomId, viewerAddress) {
        return this.generateToken({
            identity: viewerAddress,
            roomName: `room-${roomId}`,
            canPublish: false,
            canSubscribe: true,
            canPublishData: true,
        });
    }
    /**
     * 生成连麦者 Token
     */
    async generateCoHostToken(roomId, coHostAddress) {
        return this.generateToken({
            identity: coHostAddress,
            name: `CoHost-${coHostAddress.slice(0, 8)}`,
            roomName: `room-${roomId}`,
            canPublish: true,
            canSubscribe: true,
            canPublishData: true,
        });
    }
    /**
     * 获取 LiveKit 服务器 URL
     */
    getServerUrl() {
        return this.url;
    }
    /**
     * 计算 Token 过期时间
     */
    getTokenExpiry() {
        const ttl = index_js_1.config.livekit.tokenTTL;
        const hours = parseInt(ttl.replace('h', ''), 10) || 6;
        return Date.now() + hours * 60 * 60 * 1000;
    }
    /**
     * 获取房间参与者数量
     */
    async getRoomParticipantCount(roomName) {
        if (!this.roomService) {
            return 0;
        }
        try {
            const rooms = await this.roomService.listRooms([roomName]);
            return rooms[0]?.numParticipants || 0;
        }
        catch (error) {
            logger_js_1.logger.error({ error, roomName }, 'Failed to get room participant count');
            return 0;
        }
    }
    /**
     * 检查 LiveKit 服务连接状态
     */
    async healthCheck() {
        if (!this.roomService) {
            return false;
        }
        try {
            await this.roomService.listRooms([]);
            return true;
        }
        catch {
            return false;
        }
    }
    /**
     * 踢出参与者
     */
    async removeParticipant(roomName, identity) {
        if (!this.roomService) {
            return;
        }
        try {
            await this.roomService.removeParticipant(roomName, identity);
            logger_js_1.logger.info({ roomName, identity }, 'Participant removed from room');
        }
        catch (error) {
            logger_js_1.logger.error({ error, roomName, identity }, 'Failed to remove participant');
        }
    }
}
exports.LiveKitService = LiveKitService;
exports.livekitService = new LiveKitService();
//# sourceMappingURL=livekit.service.js.map