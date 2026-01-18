import Redis from 'ioredis';
import { config } from '../config/index.js';
import { logger } from '../utils/logger.js';

export class CacheService {
  private redis: Redis;
  private prefix: string;

  constructor() {
    this.redis = new Redis(config.redis.url);
    this.prefix = config.redis.keyPrefix;

    this.redis.on('connect', () => {
      logger.info('Redis connected');
    });

    this.redis.on('error', (err) => {
      logger.error({ error: err }, 'Redis error');
    });
  }

  private key(type: string, id: string | number): string {
    return `${this.prefix}${type}:${id}`;
  }

  // ============ 直播间缓存 ============

  async getRoom(roomId: number): Promise<any | null> {
    const cached = await this.redis.get(this.key('room', roomId));
    return cached ? JSON.parse(cached) : null;
  }

  async setRoom(roomId: number, room: any): Promise<void> {
    await this.redis.setex(
      this.key('room', roomId),
      config.redis.ttl.room,
      JSON.stringify(room)
    );
  }

  async invalidateRoom(roomId: number): Promise<void> {
    await this.redis.del(this.key('room', roomId));
  }

  // ============ 礼物缓存 ============

  async getGifts(): Promise<any[] | null> {
    const cached = await this.redis.get(this.key('gifts', 'all'));
    return cached ? JSON.parse(cached) : null;
  }

  async setGifts(gifts: any[]): Promise<void> {
    await this.redis.setex(
      this.key('gifts', 'all'),
      config.redis.ttl.gift,
      JSON.stringify(gifts)
    );
  }

  // ============ 黑名单缓存 ============

  async isBlacklisted(roomId: number, user: string): Promise<boolean | null> {
    const cached = await this.redis.get(this.key('blacklist', `${roomId}:${user}`));
    return cached !== null ? cached === 'true' : null;
  }

  async setBlacklisted(roomId: number, user: string, banned: boolean): Promise<void> {
    await this.redis.setex(
      this.key('blacklist', `${roomId}:${user}`),
      config.redis.ttl.blacklist,
      banned.toString()
    );
  }

  // ============ 连麦者缓存 ============

  async getCoHosts(roomId: number): Promise<string[] | null> {
    const cached = await this.redis.get(this.key('cohosts', roomId));
    return cached ? JSON.parse(cached) : null;
  }

  async setCoHosts(roomId: number, coHosts: string[]): Promise<void> {
    await this.redis.setex(
      this.key('cohosts', roomId),
      config.redis.ttl.coHosts,
      JSON.stringify(coHosts)
    );
  }

  async invalidateCoHosts(roomId: number): Promise<void> {
    await this.redis.del(this.key('cohosts', roomId));
  }

  // ============ 在线观众统计 ============

  async incrViewerCount(roomId: number): Promise<number> {
    return this.redis.incr(this.key('viewers', roomId));
  }

  async decrViewerCount(roomId: number): Promise<number> {
    const count = await this.redis.decr(this.key('viewers', roomId));
    return Math.max(0, count);
  }

  async getViewerCount(roomId: number): Promise<number> {
    const count = await this.redis.get(this.key('viewers', roomId));
    return count ? parseInt(count, 10) : 0;
  }

  async resetViewerCount(roomId: number): Promise<void> {
    await this.redis.del(this.key('viewers', roomId));
  }

  // ============ 健康检查 ============

  async ping(): Promise<boolean> {
    try {
      const result = await this.redis.ping();
      return result === 'PONG';
    } catch {
      return false;
    }
  }

  async disconnect(): Promise<void> {
    await this.redis.quit();
  }
}

export const cacheService = new CacheService();
