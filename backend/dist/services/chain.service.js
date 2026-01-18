"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.chainService = exports.ChainService = void 0;
const api_1 = require("@polkadot/api");
const index_js_1 = require("../config/index.js");
const logger_js_1 = require("../utils/logger.js");
const cache_service_js_1 = require("./cache.service.js");
// 直播间状态枚举映射
const ROOM_STATUS_MAP = {
    0: 'Preparing',
    1: 'Live',
    2: 'Paused',
    3: 'Ended',
    4: 'Banned',
};
// 直播间类型枚举映射
const ROOM_TYPE_MAP = {
    0: 'Normal',
    1: 'Paid',
    2: 'Private',
    3: 'MultiHost',
};
class ChainService {
    api = null;
    connecting = null;
    /**
     * 连接到 Substrate 节点
     */
    async connect() {
        if (this.api?.isConnected) {
            return this.api;
        }
        if (this.connecting) {
            return this.connecting;
        }
        this.connecting = this.doConnect();
        return this.connecting;
    }
    async doConnect() {
        try {
            logger_js_1.logger.info({ endpoint: index_js_1.config.chain.wsEndpoint }, 'Connecting to chain...');
            const provider = new api_1.WsProvider(index_js_1.config.chain.wsEndpoint);
            this.api = await api_1.ApiPromise.create({ provider });
            // 监听断开事件
            this.api.on('disconnected', () => {
                logger_js_1.logger.warn('Chain disconnected, will reconnect...');
                this.api = null;
                this.connecting = null;
            });
            logger_js_1.logger.info('Connected to chain');
            return this.api;
        }
        catch (error) {
            logger_js_1.logger.error({ error }, 'Failed to connect to chain');
            this.connecting = null;
            throw error;
        }
    }
    /**
     * 获取直播间信息 (带缓存)
     */
    async getRoom(roomId) {
        // 1. 先查缓存
        const cached = await cache_service_js_1.cacheService.getRoom(roomId);
        if (cached) {
            return cached;
        }
        // 2. 查询链上
        const api = await this.connect();
        const result = await api.query.livestream.liveRooms(roomId);
        if (result.isNone) {
            return null;
        }
        const room = this.parseRoom(roomId, result.unwrap());
        // 3. 写入缓存
        await cache_service_js_1.cacheService.setRoom(roomId, room);
        return room;
    }
    /**
     * 检查是否是直播间主播
     */
    async isRoomHost(roomId, address) {
        const room = await this.getRoom(roomId);
        return room?.host === address;
    }
    /**
     * 检查直播间状态
     */
    async getRoomStatus(roomId) {
        const room = await this.getRoom(roomId);
        return room?.status || null;
    }
    /**
     * 检查观众是否有门票
     */
    async hasTicket(roomId, viewerAddress) {
        const api = await this.connect();
        const result = await api.query.livestream.ticketHolders(roomId, viewerAddress);
        return result.isSome;
    }
    /**
     * 检查观众是否被封禁 (带缓存)
     */
    async isViewerBanned(roomId, viewerAddress) {
        // 1. 先查缓存
        const cached = await cache_service_js_1.cacheService.isBlacklisted(roomId, viewerAddress);
        if (cached !== null) {
            return cached;
        }
        // 2. 查询链上
        const api = await this.connect();
        const result = await api.query.livestream.roomBlacklist(roomId, viewerAddress);
        const banned = result.isSome;
        // 3. 写入缓存
        await cache_service_js_1.cacheService.setBlacklisted(roomId, viewerAddress, banned);
        return banned;
    }
    /**
     * 获取连麦者列表 (带缓存)
     */
    async getCoHosts(roomId) {
        // 1. 先查缓存
        const cached = await cache_service_js_1.cacheService.getCoHosts(roomId);
        if (cached !== null) {
            return cached;
        }
        // 2. 查询链上
        const api = await this.connect();
        const result = await api.query.livestream.activeCoHosts(roomId);
        // 解析 BoundedVec<AccountId>
        const coHosts = result.toArray().map((account) => account.toString());
        // 3. 写入缓存
        await cache_service_js_1.cacheService.setCoHosts(roomId, coHosts);
        return coHosts;
    }
    /**
     * 检查是否是连麦者
     */
    async isCoHost(roomId, address) {
        const coHosts = await this.getCoHosts(roomId);
        return coHosts.includes(address);
    }
    /**
     * 获取直播间列表
     */
    async getRooms(options) {
        const api = await this.connect();
        const { status, roomType, page = 1, limit = 20 } = options;
        const entries = await api.query.livestream.liveRooms.entries();
        let rooms = [];
        for (const [key, value] of entries) {
            if (value.isNone)
                continue;
            const roomId = key.args[0].toNumber();
            const room = this.parseRoom(roomId, value.unwrap());
            // 筛选
            if (status && room.status !== status)
                continue;
            if (roomType && room.roomType !== roomType)
                continue;
            rooms.push(room);
        }
        // 按观众数排序
        rooms.sort((a, b) => b.totalViewers - a.totalViewers);
        const total = rooms.length;
        const start = (page - 1) * limit;
        rooms = rooms.slice(start, start + limit);
        return { rooms, total };
    }
    /**
     * 获取主播累计收益
     */
    async getHostEarnings(host) {
        const api = await this.connect();
        const result = await api.query.livestream.hostEarnings(host);
        return result.toString();
    }
    /**
     * 检查连接状态
     */
    isConnected() {
        return this.api?.isConnected || false;
    }
    /**
     * 获取 API 实例 (用于事件监听)
     */
    getApi() {
        return this.api;
    }
    /**
     * 解析链上直播间数据
     */
    parseRoom(roomId, data) {
        // 解析 BoundedVec<u8> 为字符串
        const decodeBytes = (boundedVec) => {
            if (!boundedVec)
                return '';
            try {
                // BoundedVec 需要先转为 Uint8Array
                const bytes = boundedVec.toU8a ? boundedVec.toU8a() : new Uint8Array(boundedVec);
                return new TextDecoder('utf-8').decode(bytes);
            }
            catch {
                return boundedVec.toString();
            }
        };
        // 解析 Option 类型
        const unwrapOption = (option, decoder) => {
            if (option.isNone)
                return undefined;
            const value = option.unwrap();
            return decoder ? decoder(value) : value;
        };
        // 解析枚举类型
        const parseEnum = (enumValue, map) => {
            // Substrate 枚举可能是对象或数字
            if (typeof enumValue.toNumber === 'function') {
                return map[enumValue.toNumber()] || 'Unknown';
            }
            // 尝试获取枚举变体名称
            if (enumValue.type) {
                return enumValue.type;
            }
            // 遍历枚举变体
            for (const [, value] of Object.entries(map)) {
                if (enumValue[`is${value}`]) {
                    return value;
                }
            }
            return 'Unknown';
        };
        return {
            id: roomId,
            host: data.host.toString(),
            title: decodeBytes(data.title),
            description: unwrapOption(data.description, decodeBytes),
            roomType: parseEnum(data.roomType, ROOM_TYPE_MAP),
            status: parseEnum(data.status, ROOM_STATUS_MAP),
            coverCid: unwrapOption(data.coverCid, decodeBytes),
            totalViewers: data.totalViewers.toNumber(),
            peakViewers: data.peakViewers.toNumber(),
            totalGifts: data.totalGifts.toString(),
            ticketPrice: unwrapOption(data.ticketPrice, (v) => v.toString()),
            createdAt: data.createdAt.toNumber(),
            startedAt: unwrapOption(data.startedAt, (v) => v.toNumber()),
            endedAt: unwrapOption(data.endedAt, (v) => v.toNumber()),
        };
    }
}
exports.ChainService = ChainService;
exports.chainService = new ChainService();
//# sourceMappingURL=chain.service.js.map