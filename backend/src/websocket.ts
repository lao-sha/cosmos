import { Server, Socket } from 'socket.io';
import { Server as HttpServer } from 'http';
import { config } from './config/index.js';
import { logger } from './utils/logger.js';

export class WebSocketService {
  private io: Server;
  private userSockets: Map<string, Set<string>> = new Map(); // address -> socket ids

  constructor(httpServer: HttpServer) {
    this.io = new Server(httpServer, {
      cors: {
        origin: config.cors.origin,
        methods: ['GET', 'POST'],
      },
      pingTimeout: 60000,
      pingInterval: 25000,
    });

    this.setupEventHandlers();
  }

  private setupEventHandlers(): void {
    this.io.on('connection', (socket: Socket) => {
      logger.info({ socketId: socket.id }, 'Client connected');

      // 用户认证 (可选)
      socket.on('auth', (address: string) => {
        this.registerUser(address, socket.id);
        socket.data.address = address;
        logger.info({ socketId: socket.id, address }, 'User authenticated');
      });

      // 加入直播间
      socket.on('join-room', (roomId: number) => {
        const roomName = `room-${roomId}`;
        socket.join(roomName);
        logger.info({ socketId: socket.id, roomId }, 'Joined room');

        // 通知房间内其他人
        socket.to(roomName).emit('viewer:joined', {
          address: socket.data.address,
        });
      });

      // 离开直播间
      socket.on('leave-room', (roomId: number) => {
        const roomName = `room-${roomId}`;
        socket.leave(roomName);
        logger.info({ socketId: socket.id, roomId }, 'Left room');

        // 通知房间内其他人
        socket.to(roomName).emit('viewer:left', {
          address: socket.data.address,
        });
      });

      // 断开连接
      socket.on('disconnect', () => {
        if (socket.data.address) {
          this.unregisterUser(socket.data.address, socket.id);
        }
        logger.info({ socketId: socket.id }, 'Client disconnected');
      });
    });
  }

  private registerUser(address: string, socketId: string): void {
    if (!this.userSockets.has(address)) {
      this.userSockets.set(address, new Set());
    }
    this.userSockets.get(address)!.add(socketId);
  }

  private unregisterUser(address: string, socketId: string): void {
    const sockets = this.userSockets.get(address);
    if (sockets) {
      sockets.delete(socketId);
      if (sockets.size === 0) {
        this.userSockets.delete(address);
      }
    }
  }

  /**
   * 广播到指定直播间
   */
  broadcastToRoom(roomId: number, event: string, data: any): void {
    const roomName = `room-${roomId}`;
    this.io.to(roomName).emit(event, data);
    logger.debug({ roomId, event }, 'Broadcast to room');
  }

  /**
   * 广播到所有连接
   */
  broadcastToAll(event: string, data: any): void {
    this.io.emit(event, data);
    logger.debug({ event }, 'Broadcast to all');
  }

  /**
   * 通知特定用户
   */
  notifyUser(address: string, event: string, data: any): void {
    const socketIds = this.userSockets.get(address);
    if (socketIds) {
      socketIds.forEach((socketId) => {
        this.io.to(socketId).emit(event, data);
      });
      logger.debug({ address, event }, 'Notified user');
    }
  }

  /**
   * 获取直播间在线人数
   */
  async getRoomViewerCount(roomId: number): Promise<number> {
    const roomName = `room-${roomId}`;
    const sockets = await this.io.in(roomName).fetchSockets();
    return sockets.length;
  }

  /**
   * 踢出用户
   */
  async kickUserFromRoom(roomId: number, address: string): Promise<void> {
    const roomName = `room-${roomId}`;
    const socketIds = this.userSockets.get(address);

    if (socketIds) {
      for (const socketId of socketIds) {
        const socket = this.io.sockets.sockets.get(socketId);
        if (socket && socket.rooms.has(roomName)) {
          socket.leave(roomName);
          socket.emit('viewer:kicked', { roomId, message: 'You have been kicked' });
        }
      }
    }
  }
}

// 导出单例 (将在 app.ts 中初始化)
export let websocketService: WebSocketService;

export function initWebSocketService(httpServer: HttpServer): WebSocketService {
  websocketService = new WebSocketService(httpServer);
  return websocketService;
}
