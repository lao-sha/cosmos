"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.cacheService = exports.CacheService = void 0;
const ioredis_1 = __importDefault(require("ioredis"));
const index_js_1 = require("../config/index.js");
const logger_js_1 = require("../utils/logger.js");
class CacheService {
    redis;
    prefix;
    constructor() {
        this.redis = new ioredis_1.default(index_js_1.config.redis.url);
        this.prefix = index_js_1.config.redis.keyPrefix;
        this.redis.on('connect', () => {
            logger_js_1.logger.info('Redis connected');
        });
        this.redis.on('error', (err) => {
            logger_js_1.logger.error({ error: err }, 'Redis error');
        });
    }
    key(type, id) {
        return `${this.prefix}${type}:${id}`;
    }
    // ============ 直播间缓存 ============
    async getRoom(roomId) {
        const cached = await this.redis.get(this.key('room', roomId));
        return cached ? JSON.parse(cached) : null;
    }
    async setRoom(roomId, room) {
        await this.redis.setex(this.key('room', roomId), index_js_1.config.redis.ttl.room, JSON.stringify(room));
    }
    async invalidateRoom(roomId) {
        await this.redis.del(this.key('room', roomId));
    }
    // ============ 礼物缓存 ============
    async getGifts() {
        const cached = await this.redis.get(this.key('gifts', 'all'));
        return cached ? JSON.parse(cached) : null;
    }
    async setGifts(gifts) {
        await this.redis.setex(this.key('gifts', 'all'), index_js_1.config.redis.ttl.gift, JSON.stringify(gifts));
    }
    // ============ 黑名单缓存 ============
    async isBlacklisted(roomId, user) {
        const cached = await this.redis.get(this.key('blacklist', `${roomId}:${user}`));
        return cached !== null ? cached === 'true' : null;
    }
    async setBlacklisted(roomId, user, banned) {
        await this.redis.setex(this.key('blacklist', `${roomId}:${user}`), index_js_1.config.redis.ttl.blacklist, banned.toString());
    }
    // ============ 连麦者缓存 ============
    async getCoHosts(roomId) {
        const cached = await this.redis.get(this.key('cohosts', roomId));
        return cached ? JSON.parse(cached) : null;
    }
    async setCoHosts(roomId, coHosts) {
        await this.redis.setex(this.key('cohosts', roomId), index_js_1.config.redis.ttl.coHosts, JSON.stringify(coHosts));
    }
    async invalidateCoHosts(roomId) {
        await this.redis.del(this.key('cohosts', roomId));
    }
    // ============ 在线观众统计 ============
    async incrViewerCount(roomId) {
        return this.redis.incr(this.key('viewers', roomId));
    }
    async decrViewerCount(roomId) {
        const count = await this.redis.decr(this.key('viewers', roomId));
        return Math.max(0, count);
    }
    async getViewerCount(roomId) {
        const count = await this.redis.get(this.key('viewers', roomId));
        return count ? parseInt(count, 10) : 0;
    }
    async resetViewerCount(roomId) {
        await this.redis.del(this.key('viewers', roomId));
    }
    // ============ 健康检查 ============
    async ping() {
        try {
            const result = await this.redis.ping();
            return result === 'PONG';
        }
        catch {
            return false;
        }
    }
    async disconnect() {
        await this.redis.quit();
    }
}
exports.CacheService = CacheService;
exports.cacheService = new CacheService();
//# sourceMappingURL=cache.service.js.map