import { chainService } from './chain.service.js';
import { cacheService } from './cache.service.js';
import { logger } from '../utils/logger.js';

// WebSocket 服务将在后面导入，避免循环依赖
let websocketService: any = null;

export function setWebSocketService(ws: any) {
  websocketService = ws;
}

export interface LivestreamEvent {
  type: string;
  roomId?: number;
  data: any;
  blockNumber: number;
  timestamp: number;
}

export class ChainEventsService {
  private unsubscribe: (() => void) | null = null;

  /**
   * 开始监听链上事件
   */
  async startListening(): Promise<void> {
    const api = await chainService.connect();

    logger.info('Starting chain events listener...');

    // 订阅新区块事件
    const unsub = await api.query.system.events((events: any) => {
      events.forEach((record: any) => {
        const { event } = record;

        // 只处理 livestream 模块的事件
        if (event.section === 'livestream') {
          this.handleLivestreamEvent(event);
        }
      });
    });

    this.unsubscribe = unsub as unknown as () => void;

    logger.info('Chain events listener started');
  }

  /**
   * 停止监听
   */
  stopListening(): void {
    if (this.unsubscribe) {
      this.unsubscribe();
      this.unsubscribe = null;
      logger.info('Chain events listener stopped');
    }
  }

  /**
   * 处理直播模块事件
   */
  private async handleLivestreamEvent(event: any): Promise<void> {
    const eventName = event.method;
    const data = event.data.toJSON();

    logger.info({ eventName, data }, 'Received livestream event');

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
          logger.debug({ eventName }, 'Unhandled event');
      }
    } catch (error) {
      logger.error({ error, eventName, data }, 'Error handling event');
    }
  }

  // ============ 事件处理器 ============

  private async onRoomCreated(data: any): Promise<void> {
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

  private async onLiveStarted(data: any): Promise<void> {
    const { roomId, startedAt } = data;

    // 清除缓存
    await cacheService.invalidateRoom(roomId);

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

  private async onLiveEnded(data: any): Promise<void> {
    const { roomId, duration, totalViewers, peakViewers, totalGifts } = data;

    // 清除缓存
    await cacheService.invalidateRoom(roomId);
    await cacheService.resetViewerCount(roomId);

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

  private async onGiftSent(data: any): Promise<void> {
    const { roomId, sender, receiver, giftId, quantity, value } = data;

    // 清除直播间缓存 (礼物总额变化)
    await cacheService.invalidateRoom(roomId);

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

  private async onCoHostStarted(data: any): Promise<void> {
    const { roomId, coHost } = data;

    // 清除连麦者缓存
    await cacheService.invalidateCoHosts(roomId);

    // 广播连麦开始
    if (websocketService) {
      websocketService.broadcastToRoom(roomId, 'cohost:started', {
        roomId,
        coHost,
      });
    }
  }

  private async onCoHostEnded(data: any): Promise<void> {
    const { roomId, coHost } = data;

    // 清除连麦者缓存
    await cacheService.invalidateCoHosts(roomId);

    // 广播连麦结束
    if (websocketService) {
      websocketService.broadcastToRoom(roomId, 'cohost:ended', {
        roomId,
        coHost,
      });
    }
  }

  private async onViewerKicked(data: any): Promise<void> {
    const { roomId, viewer } = data;

    // 更新黑名单缓存
    await cacheService.setBlacklisted(roomId, viewer, true);

    // 通知被踢用户
    if (websocketService) {
      websocketService.notifyUser(viewer, 'viewer:kicked', {
        roomId,
        message: 'You have been kicked from the room',
      });
    }
  }

  private async onViewerUnbanned(data: any): Promise<void> {
    const { roomId, viewer } = data;

    // 更新黑名单缓存
    await cacheService.setBlacklisted(roomId, viewer, false);
  }

  private async onRoomBanned(data: any): Promise<void> {
    const { roomId, reason } = data;

    // 清除缓存
    await cacheService.invalidateRoom(roomId);

    // 广播封禁通知
    if (websocketService) {
      websocketService.broadcastToRoom(roomId, 'room:banned', {
        roomId,
        reason,
      });
    }
  }
}

export const chainEventsService = new ChainEventsService();
