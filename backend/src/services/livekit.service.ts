import { AccessToken, VideoGrant, RoomServiceClient } from 'livekit-server-sdk';
import { config } from '../config/index.js';
import { logger } from '../utils/logger.js';

export interface TokenOptions {
  identity: string;
  name?: string;
  roomName: string;
  canPublish: boolean;
  canSubscribe: boolean;
  canPublishData: boolean;
}

export class LiveKitService {
  private apiKey: string;
  private apiSecret: string;
  private url: string;
  private roomService: RoomServiceClient | null = null;

  constructor() {
    this.apiKey = config.livekit.apiKey;
    this.apiSecret = config.livekit.apiSecret;
    this.url = config.livekit.url;

    // 初始化 Room Service 客户端 (仅在配置完整时)
    if (this.apiKey && this.apiSecret) {
      const httpUrl = this.url.replace('wss://', 'https://').replace('ws://', 'http://');
      this.roomService = new RoomServiceClient(httpUrl, this.apiKey, this.apiSecret);
    }
  }

  /**
   * 生成 LiveKit 访问 Token
   */
  async generateToken(options: TokenOptions): Promise<string> {
    const { identity, name, roomName, canPublish, canSubscribe, canPublishData } = options;

    const token = new AccessToken(this.apiKey, this.apiSecret, {
      identity,
      name: name || identity,
      ttl: config.livekit.tokenTTL,
    });

    const grant: VideoGrant = {
      room: roomName,
      roomJoin: true,
      canPublish,
      canSubscribe,
      canPublishData,
    };

    token.addGrant(grant);

    logger.info({ identity, roomName, canPublish }, 'Generated LiveKit token');

    return await token.toJwt();
  }

  /**
   * 生成主播 Token
   */
  async generatePublisherToken(roomId: number, hostAddress: string): Promise<string> {
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
  async generateViewerToken(roomId: number, viewerAddress: string): Promise<string> {
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
  async generateCoHostToken(roomId: number, coHostAddress: string): Promise<string> {
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
  getServerUrl(): string {
    return this.url;
  }

  /**
   * 计算 Token 过期时间
   */
  getTokenExpiry(): number {
    const ttl = config.livekit.tokenTTL;
    const hours = parseInt(ttl.replace('h', ''), 10) || 6;
    return Date.now() + hours * 60 * 60 * 1000;
  }

  /**
   * 获取房间参与者数量
   */
  async getRoomParticipantCount(roomName: string): Promise<number> {
    if (!this.roomService) {
      return 0;
    }
    try {
      const rooms = await this.roomService.listRooms([roomName]);
      return rooms[0]?.numParticipants || 0;
    } catch (error) {
      logger.error({ error, roomName }, 'Failed to get room participant count');
      return 0;
    }
  }

  /**
   * 检查 LiveKit 服务连接状态
   */
  async healthCheck(): Promise<boolean> {
    if (!this.roomService) {
      return false;
    }
    try {
      await this.roomService.listRooms([]);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * 踢出参与者
   */
  async removeParticipant(roomName: string, identity: string): Promise<void> {
    if (!this.roomService) {
      return;
    }
    try {
      await this.roomService.removeParticipant(roomName, identity);
      logger.info({ roomName, identity }, 'Participant removed from room');
    } catch (error) {
      logger.error({ error, roomName, identity }, 'Failed to remove participant');
    }
  }
}

export const livekitService = new LiveKitService();
