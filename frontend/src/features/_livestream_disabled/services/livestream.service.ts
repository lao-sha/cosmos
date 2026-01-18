// frontend/src/features/livestream/services/livestream.service.ts

import { ApiPromise } from '@polkadot/api';
import { getApi } from '@/lib/api';
import type {
  LiveRoom,
  Gift,
  LiveRoomType,
  LiveRoomStatus,
  CreateRoomParams,
} from '../types';

const API_BASE_URL = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:3000';

export class LivestreamService {
  private api: ApiPromise | null = null;
  public myAddress: string;

  constructor(myAddress: string) {
    this.myAddress = myAddress;
  }

  async init(): Promise<void> {
    this.api = getApi();
  }

  /**
   * 获取直播间列表
   */
  async getLiveRooms(filter?: string): Promise<LiveRoom[]> {
    if (!this.api) throw new Error('API not initialized');

    try {
      const entries = await this.api.query.livestream.liveRooms.entries();
      const rooms: LiveRoom[] = [];

      for (const [key, value] of entries) {
        if ((value as any).isNone) continue;
        const data = (value as any).unwrap();

        // 只返回直播中的房间
        if (data.status.toString() !== 'Live') continue;

        // 筛选
        if (filter && filter !== 'all' && data.roomType.toString() !== filter) continue;

        rooms.push(this.parseRoom(key.args[0].toNumber(), data));
      }

      // 按观众数排序
      return rooms.sort((a, b) => b.currentViewers - a.currentViewers);
    } catch (error) {
      console.error('[LivestreamService] getLiveRooms error:', error);
      return [];
    }
  }

  /**
   * 获取直播间详情
   */
  async getRoomInfo(roomId: number): Promise<LiveRoom> {
    if (!this.api) throw new Error('API not initialized');

    const result = await this.api.query.livestream.liveRooms(roomId);
    if ((result as any).isNone) throw new Error('Room not found');

    return this.parseRoom(roomId, (result as any).unwrap());
  }

  /**
   * 创建直播间
   */
  async createRoom(params: CreateRoomParams): Promise<number> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.livestream.createRoom(
      params.title,
      params.description || null,
      params.roomType,
      params.coverCid || null,
      params.ticketPrice || null
    );

    return new Promise((resolve, reject) => {
      tx.signAndSend(this.myAddress, ({ status, events }) => {
        if (status.isInBlock) {
          for (const { event } of events) {
            if (this.api!.events.livestream.RoomCreated.is(event)) {
              const [, roomId] = event.data;
              resolve((roomId as any).toNumber());
              return;
            }
          }
          reject(new Error('RoomCreated event not found'));
        }
      }).catch(reject);
    });
  }

  /**
   * 开始直播
   */
  async startLive(roomId: number): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.livestream.startLive(roomId);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 暂停直播
   */
  async pauseLive(roomId: number): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.livestream.pauseLive(roomId);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 结束直播
   */
  async endLive(roomId: number): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.livestream.endLive(roomId);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 购买门票
   */
  async buyTicket(roomId: number): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.livestream.buyTicket(roomId);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 检查是否有门票
   */
  async checkTicket(roomId: number): Promise<boolean> {
    if (!this.api) return false;

    const result = await this.api.query.livestream.ticketHolders(roomId, this.myAddress);
    return (result as any).isSome;
  }

  /**
   * 发送礼物
   */
  async sendGift(roomId: number, giftId: number, quantity: number): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.livestream.sendGift(roomId, giftId, quantity);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 获取礼物列表
   */
  async getGifts(): Promise<Gift[]> {
    if (!this.api) throw new Error('API not initialized');

    try {
      const entries = await this.api.query.livestream.gifts.entries();
      const gifts: Gift[] = [];

      for (const [key, value] of entries) {
        if ((value as any).isNone) continue;
        const data = (value as any).unwrap();

        if (!data.enabled.valueOf()) continue;

        gifts.push({
          id: (key.args[0] as any).toNumber(),
          name: data.name.toUtf8(),
          price: data.price.toString(),
          iconCid: data.iconCid.toUtf8(),
          enabled: true,
        });
      }

      return gifts.sort((a, b) => Number(a.price) - Number(b.price));
    } catch (error) {
      console.error('[LivestreamService] getGifts error:', error);
      return [];
    }
  }

  /**
   * 提现收益
   */
  async withdrawEarnings(amount: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.livestream.withdrawEarnings(amount);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 获取主播收益
   */
  async getHostEarnings(): Promise<string> {
    if (!this.api) return '0';

    const result = await this.api.query.livestream.hostEarnings(this.myAddress);
    return result.toString();
  }

  /**
   * 踢出观众
   */
  async kickViewer(roomId: number, viewer: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.livestream.kickViewer(roomId, viewer);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 开始连麦 (链上记录)
   */
  async startCoHost(roomId: number, coHost: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.livestream.startCoHost(roomId, coHost);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 结束连麦
   */
  async endCoHost(roomId: number, coHost?: string): Promise<void> {
    if (!this.api) throw new Error('API not initialized');

    const tx = this.api.tx.livestream.endCoHost(roomId, coHost || null);
    await tx.signAndSend(this.myAddress);
  }

  /**
   * 获取观众 Token
   */
  async getViewerToken(roomId: number): Promise<string> {
    const response = await fetch(`${API_BASE_URL}/api/livestream/viewer-token`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ roomId, viewerAddress: this.myAddress }),
    });

    if (!response.ok) throw new Error('Failed to get viewer token');
    const { token } = await response.json();
    return token;
  }

  /**
   * 获取主播 Token (需要签名)
   */
  async getPublisherToken(roomId: number): Promise<string> {
    const timestamp = Date.now();
    const message = `livestream:${roomId}:${timestamp}`;

    // 使用移动端签名器签名
    const signature = await this.signMessageNative(message);

    const response = await fetch(`${API_BASE_URL}/api/livestream/publisher-token`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        roomId,
        publicKey: this.myAddress,
        signature,
        timestamp,
      }),
    });

    if (!response.ok) throw new Error('Failed to get publisher token');
    const { token } = await response.json();
    return token;
  }

  /**
   * 使用移动端签名器签名消息
   */
  private async signMessageNative(message: string): Promise<string> {
    const { getCurrentPair } = await import('@/lib/signer.native');
    const pair = getCurrentPair();

    if (!pair) {
      throw new Error('Wallet is locked. Please unlock first.');
    }

    // 将消息转换为 Uint8Array 并签名
    const messageU8a = new TextEncoder().encode(message);
    const signatureU8a = pair.sign(messageU8a);

    // 转换为十六进制字符串
    const { u8aToHex } = await import('@polkadot/util');
    return u8aToHex(signatureU8a);
  }

  private parseRoom(id: number, data: any): LiveRoom {
    return {
      id,
      host: data.host.toString(),
      title: data.title.toUtf8(),
      description: data.description.isSome ? data.description.unwrap().toUtf8() : undefined,
      roomType: data.roomType.toString() as LiveRoomType,
      status: data.status.toString() as LiveRoomStatus,
      coverCid: data.coverCid.isSome ? data.coverCid.unwrap().toUtf8() : undefined,
      totalViewers: data.totalViewers.toNumber(),
      peakViewers: data.peakViewers.toNumber(),
      currentViewers: 0, // 从 LiveKit 获取
      totalGifts: data.totalGifts.toString(),
      ticketPrice: data.ticketPrice.isSome ? data.ticketPrice.unwrap().toString() : undefined,
      createdAt: data.createdAt.toNumber(),
      startedAt: data.startedAt.isSome ? data.startedAt.unwrap().toNumber() : undefined,
      endedAt: data.endedAt.isSome ? data.endedAt.unwrap().toNumber() : undefined,
    };
  }
}

// 单例
let serviceInstance: LivestreamService | null = null;

export function getLivestreamService(): LivestreamService {
  if (!serviceInstance) throw new Error('LivestreamService not initialized');
  return serviceInstance;
}

export function initLivestreamService(myAddress: string): LivestreamService {
  serviceInstance = new LivestreamService(myAddress);
  return serviceInstance;
}
