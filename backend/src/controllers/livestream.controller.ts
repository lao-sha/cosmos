import { Request, Response, NextFunction } from 'express';
import { livekitService } from '../services/livekit.service.js';
import { signatureService } from '../services/signature.service.js';
import { chainService } from '../services/chain.service.js';
import { cacheService } from '../services/cache.service.js';
import { AppError } from '../middleware/error.middleware.js';
import { logger } from '../utils/logger.js';

export class LivestreamController {
  /**
   * 获取主播推流 Token
   * POST /api/livestream/publisher-token
   */
  async getPublisherToken(req: Request, res: Response, next: NextFunction) {
    try {
      const { roomId, publicKey, signature, timestamp } = req.body;

      // 1. 验证签名
      const signResult = signatureService.verifyPublisherSignature(
        roomId,
        publicKey,
        signature,
        timestamp
      );

      if (!signResult.isValid) {
        const errorCode = signResult.error?.includes('expired')
          ? 'SIGNATURE_EXPIRED'
          : 'INVALID_SIGNATURE';
        throw new AppError(errorCode, signResult.error || 'Signature verification failed', 401);
      }

      // 2. 查询链上直播间
      const room = await chainService.getRoom(roomId);
      if (!room) {
        throw new AppError('ROOM_NOT_FOUND', 'Room not found', 404);
      }

      // 3. 验证是否是主播
      if (room.host !== publicKey) {
        throw new AppError('NOT_ROOM_HOST', 'You are not the host of this room', 403);
      }

      // 4. 验证直播间状态
      if (room.status !== 'Live' && room.status !== 'Preparing') {
        throw new AppError('ROOM_NOT_AVAILABLE', 'Room is not available for streaming', 400);
      }

      // 5. 生成 Token
      const token = await livekitService.generatePublisherToken(roomId, publicKey);

      logger.info({ roomId, host: publicKey }, 'Publisher token generated');

      res.json({
        token,
        url: livekitService.getServerUrl(),
        roomName: `room-${roomId}`,
        expiresAt: livekitService.getTokenExpiry(),
      });
    } catch (error) {
      next(error);
    }
  }

  /**
   * 获取观众观看 Token
   * POST /api/livestream/viewer-token
   */
  async getViewerToken(req: Request, res: Response, next: NextFunction) {
    try {
      const { roomId, viewerAddress, signature, timestamp } = req.body;

      // 1. 验证签名 (防止身份伪造)
      const signResult = signatureService.verifyPublisherSignature(
        roomId,
        viewerAddress,
        signature,
        timestamp
      );

      if (!signResult.isValid) {
        const errorCode = signResult.error?.includes('expired')
          ? 'SIGNATURE_EXPIRED'
          : 'INVALID_SIGNATURE';
        throw new AppError(errorCode, signResult.error || 'Signature verification failed', 401);
      }

      // 2. 查询链上直播间
      const room = await chainService.getRoom(roomId);
      if (!room) {
        throw new AppError('ROOM_NOT_FOUND', 'Room not found', 404);
      }

      // 3. 验证直播间状态
      if (room.status !== 'Live') {
        throw new AppError('ROOM_NOT_LIVE', 'Room is not live', 400);
      }

      // 4. 如果是付费直播，检查门票
      if (room.roomType === 'Paid') {
        const hasTicket = await chainService.hasTicket(roomId, viewerAddress);
        if (!hasTicket) {
          throw new AppError('TICKET_REQUIRED', 'You need to buy a ticket first', 402);
        }
      }

      // 5. 检查是否被封禁
      const isBanned = await chainService.isViewerBanned(roomId, viewerAddress);
      if (isBanned) {
        throw new AppError('VIEWER_BANNED', 'You are banned from this room', 403);
      }

      // 6. 生成 Token
      const token = await livekitService.generateViewerToken(roomId, viewerAddress);

      // 7. 增加在线观众计数
      await cacheService.incrViewerCount(roomId);

      logger.info({ roomId, viewer: viewerAddress }, 'Viewer token generated');

      res.json({
        token,
        url: livekitService.getServerUrl(),
        roomName: `room-${roomId}`,
        expiresAt: livekitService.getTokenExpiry(),
      });
    } catch (error) {
      next(error);
    }
  }

  /**
   * 获取连麦者 Token
   * POST /api/livestream/co-host-token
   */
  async getCoHostToken(req: Request, res: Response, next: NextFunction) {
    try {
      const { roomId, coHostAddress, signature, timestamp, type } = req.body;

      // 1. 验证签名
      const signResult = signatureService.verifyPublisherSignature(
        roomId,
        coHostAddress,
        signature,
        timestamp
      );

      if (!signResult.isValid) {
        const errorCode = signResult.error?.includes('expired')
          ? 'SIGNATURE_EXPIRED'
          : 'INVALID_SIGNATURE';
        throw new AppError(errorCode, signResult.error || 'Signature verification failed', 401);
      }

      // 2. 查询链上直播间
      const room = await chainService.getRoom(roomId);
      if (!room) {
        throw new AppError('ROOM_NOT_FOUND', 'Room not found', 404);
      }

      // 3. 验证直播间状态
      if (room.status !== 'Live') {
        throw new AppError('ROOM_NOT_LIVE', 'Room is not live', 400);
      }

      // 4. 验证是否在连麦者列表中 (链上验证)
      const isCoHost = await chainService.isCoHost(roomId, coHostAddress);
      if (!isCoHost) {
        throw new AppError('NOT_CO_HOST', 'You are not approved as co-host', 403);
      }

      // 5. 生成 Token
      const token = await livekitService.generateCoHostToken(roomId, coHostAddress);

      logger.info({ roomId, coHost: coHostAddress, type }, 'Co-host token generated');

      res.json({
        token,
        url: livekitService.getServerUrl(),
        roomName: `room-${roomId}`,
        expiresAt: livekitService.getTokenExpiry(),
      });
    } catch (error) {
      next(error);
    }
  }

  /**
   * 获取直播间信息
   * GET /api/livestream/room/:roomId
   */
  async getRoom(req: Request, res: Response, next: NextFunction) {
    try {
      const roomId = parseInt(req.params.roomId, 10);

      if (isNaN(roomId)) {
        throw new AppError('INVALID_PARAMS', 'Invalid room ID', 400);
      }

      const room = await chainService.getRoom(roomId);
      if (!room) {
        throw new AppError('ROOM_NOT_FOUND', 'Room not found', 404);
      }

      // 获取实时观众数
      const currentViewers = await cacheService.getViewerCount(roomId);

      res.json({
        ...room,
        currentViewers,
      });
    } catch (error) {
      next(error);
    }
  }

  /**
   * 获取直播间列表
   * GET /api/livestream/rooms
   */
  async getRooms(req: Request, res: Response, next: NextFunction) {
    try {
      const { status, type, page, limit } = req.query;

      const result = await chainService.getRooms({
        status: status as string,
        roomType: type as string,
        page: page ? parseInt(page as string, 10) : 1,
        limit: limit ? parseInt(limit as string, 10) : 20,
      });

      // 为每个直播间添加实时观众数
      const roomsWithViewers = await Promise.all(
        result.rooms.map(async (room) => ({
          ...room,
          currentViewers: await cacheService.getViewerCount(room.id),
        }))
      );

      res.json({
        rooms: roomsWithViewers,
        total: result.total,
        page: page ? parseInt(page as string, 10) : 1,
        limit: limit ? parseInt(limit as string, 10) : 20,
      });
    } catch (error) {
      next(error);
    }
  }

  /**
   * 观众离开直播间
   * POST /api/livestream/viewer-leave
   */
  async viewerLeave(req: Request, res: Response, next: NextFunction) {
    try {
      const { roomId, viewerAddress, signature, timestamp } = req.body;

      // 验证签名
      const signResult = signatureService.verifyPublisherSignature(
        roomId,
        viewerAddress,
        signature,
        timestamp
      );

      if (!signResult.isValid) {
        throw new AppError('INVALID_SIGNATURE', 'Signature verification failed', 401);
      }

      // 减少在线观众计数
      await cacheService.decrViewerCount(roomId);

      logger.info({ roomId, viewer: viewerAddress }, 'Viewer left');

      res.json({ success: true });
    } catch (error) {
      next(error);
    }
  }
}

export const livestreamController = new LivestreamController();
