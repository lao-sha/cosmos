import { ApiPromise, WsProvider } from '@polkadot/api';
import { config } from '../config/index.js';
import { logger } from '../utils/logger.js';
import { cacheService } from './cache.service.js';

export interface LiveRoom {
  id: number;
  host: string;
  title: string;
  description?: string;
  roomType: string;
  status: string;
  coverCid?: string;
  totalViewers: number;
  peakViewers: number;
  totalGifts: string;
  ticketPrice?: string;
  createdAt: number;
  startedAt?: number;
  endedAt?: number;
}

// 直播间状态枚举映射
const ROOM_STATUS_MAP: Record<number, string> = {
  0: 'Preparing',
  1: 'Live',
  2: 'Paused',
  3: 'Ended',
  4: 'Banned',
};

// 直播间类型枚举映射
const ROOM_TYPE_MAP: Record<number, string> = {
  0: 'Normal',
  1: 'Paid',
  2: 'Private',
  3: 'MultiHost',
};

export class ChainService {
  private api: ApiPromise | null = null;
  private connecting: Promise<ApiPromise> | null = null;

  /**
   * 连接到 Substrate 节点
   */
  async connect(): Promise<ApiPromise> {
    if (this.api?.isConnected) {
      return this.api;
    }

    if (this.connecting) {
      return this.connecting;
    }

    this.connecting = this.doConnect();
    return this.connecting;
  }

  private async doConnect(): Promise<ApiPromise> {
    try {
      logger.info({ endpoint: config.chain.wsEndpoint }, 'Connecting to chain...');

      const provider = new WsProvider(config.chain.wsEndpoint);
      this.api = await ApiPromise.create({ provider });

      // 监听断开事件
      this.api.on('disconnected', () => {
        logger.warn('Chain disconnected, will reconnect...');
        this.api = null;
        this.connecting = null;
      });

      logger.info('Connected to chain');
      return this.api;
    } catch (error) {
      logger.error({ error }, 'Failed to connect to chain');
      this.connecting = null;
      throw error;
    }
  }

  /**
   * 获取直播间信息 (带缓存)
   */
  async getRoom(roomId: number): Promise<LiveRoom | null> {
    // 1. 先查缓存
    const cached = await cacheService.getRoom(roomId);
    if (cached) {
      return cached;
    }

    // 2. 查询链上
    const api = await this.connect();
    const result = await (api.query as any).livestream.liveRooms(roomId);

    if (result.isNone) {
      return null;
    }

    const room = this.parseRoom(roomId, result.unwrap());

    // 3. 写入缓存
    await cacheService.setRoom(roomId, room);

    return room;
  }

  /**
   * 检查是否是直播间主播
   */
  async isRoomHost(roomId: number, address: string): Promise<boolean> {
    const room = await this.getRoom(roomId);
    return room?.host === address;
  }

  /**
   * 检查直播间状态
   */
  async getRoomStatus(roomId: number): Promise<string | null> {
    const room = await this.getRoom(roomId);
    return room?.status || null;
  }

  /**
   * 检查观众是否有门票
   */
  async hasTicket(roomId: number, viewerAddress: string): Promise<boolean> {
    const api = await this.connect();
    const result = await (api.query as any).livestream.ticketHolders(roomId, viewerAddress);
    return result.isSome;
  }

  /**
   * 检查观众是否被封禁 (带缓存)
   */
  async isViewerBanned(roomId: number, viewerAddress: string): Promise<boolean> {
    // 1. 先查缓存
    const cached = await cacheService.isBlacklisted(roomId, viewerAddress);
    if (cached !== null) {
      return cached;
    }

    // 2. 查询链上
    const api = await this.connect();
    const result = await (api.query as any).livestream.roomBlacklist(roomId, viewerAddress);
    const banned = result.isSome;

    // 3. 写入缓存
    await cacheService.setBlacklisted(roomId, viewerAddress, banned);

    return banned;
  }

  /**
   * 获取连麦者列表 (带缓存)
   */
  async getCoHosts(roomId: number): Promise<string[]> {
    // 1. 先查缓存
    const cached = await cacheService.getCoHosts(roomId);
    if (cached !== null) {
      return cached;
    }

    // 2. 查询链上
    const api = await this.connect();
    const result = await (api.query as any).livestream.activeCoHosts(roomId);

    // 解析 BoundedVec<AccountId>
    const coHosts = result.toArray().map((account: any) => account.toString());

    // 3. 写入缓存
    await cacheService.setCoHosts(roomId, coHosts);

    return coHosts;
  }

  /**
   * 检查是否是连麦者
   */
  async isCoHost(roomId: number, address: string): Promise<boolean> {
    const coHosts = await this.getCoHosts(roomId);
    return coHosts.includes(address);
  }

  /**
   * 获取直播间列表
   */
  async getRooms(options: {
    status?: string;
    roomType?: string;
    page?: number;
    limit?: number;
  }): Promise<{ rooms: LiveRoom[]; total: number }> {
    const api = await this.connect();
    const { status, roomType, page = 1, limit = 20 } = options;

    const entries = await (api.query as any).livestream.liveRooms.entries();
    let rooms: LiveRoom[] = [];

    for (const [key, value] of entries) {
      if (value.isNone) continue;

      const roomId = (key.args[0] as any).toNumber();
      const room = this.parseRoom(roomId, value.unwrap());

      // 筛选
      if (status && room.status !== status) continue;
      if (roomType && room.roomType !== roomType) continue;

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
  async getHostEarnings(host: string): Promise<string> {
    const api = await this.connect();
    const result = await (api.query as any).livestream.hostEarnings(host);
    return result.toString();
  }

  /**
   * 检查连接状态
   */
  isConnected(): boolean {
    return this.api?.isConnected || false;
  }

  /**
   * 获取 API 实例 (用于事件监听)
   */
  getApi(): ApiPromise | null {
    return this.api;
  }

  /**
   * 解析链上直播间数据
   */
  private parseRoom(roomId: number, data: any): LiveRoom {
    // 解析 BoundedVec<u8> 为字符串
    const decodeBytes = (boundedVec: any): string => {
      if (!boundedVec) return '';
      try {
        // BoundedVec 需要先转为 Uint8Array
        const bytes = boundedVec.toU8a ? boundedVec.toU8a() : new Uint8Array(boundedVec);
        return new TextDecoder('utf-8').decode(bytes);
      } catch {
        return boundedVec.toString();
      }
    };

    // 解析 Option 类型
    const unwrapOption = <T>(option: any, decoder?: (v: any) => T): T | undefined => {
      if (option.isNone) return undefined;
      const value = option.unwrap();
      return decoder ? decoder(value) : value;
    };

    // 解析枚举类型
    const parseEnum = (enumValue: any, map: Record<number, string>): string => {
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

export const chainService = new ChainService();
