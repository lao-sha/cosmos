import { Request, Response, NextFunction } from 'express';
export declare class LivestreamController {
    /**
     * 获取主播推流 Token
     * POST /api/livestream/publisher-token
     */
    getPublisherToken(req: Request, res: Response, next: NextFunction): Promise<void>;
    /**
     * 获取观众观看 Token
     * POST /api/livestream/viewer-token
     */
    getViewerToken(req: Request, res: Response, next: NextFunction): Promise<void>;
    /**
     * 获取连麦者 Token
     * POST /api/livestream/co-host-token
     */
    getCoHostToken(req: Request, res: Response, next: NextFunction): Promise<void>;
    /**
     * 获取直播间信息
     * GET /api/livestream/room/:roomId
     */
    getRoom(req: Request, res: Response, next: NextFunction): Promise<void>;
    /**
     * 获取直播间列表
     * GET /api/livestream/rooms
     */
    getRooms(req: Request, res: Response, next: NextFunction): Promise<void>;
    /**
     * 观众离开直播间
     * POST /api/livestream/viewer-leave
     */
    viewerLeave(req: Request, res: Response, next: NextFunction): Promise<void>;
}
export declare const livestreamController: LivestreamController;
//# sourceMappingURL=livestream.controller.d.ts.map