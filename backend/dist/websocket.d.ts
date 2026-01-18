import { Server as HttpServer } from 'http';
export declare class WebSocketService {
    private io;
    private userSockets;
    constructor(httpServer: HttpServer);
    private setupEventHandlers;
    private registerUser;
    private unregisterUser;
    /**
     * 广播到指定直播间
     */
    broadcastToRoom(roomId: number, event: string, data: any): void;
    /**
     * 广播到所有连接
     */
    broadcastToAll(event: string, data: any): void;
    /**
     * 通知特定用户
     */
    notifyUser(address: string, event: string, data: any): void;
    /**
     * 获取直播间在线人数
     */
    getRoomViewerCount(roomId: number): Promise<number>;
    /**
     * 踢出用户
     */
    kickUserFromRoom(roomId: number, address: string): Promise<void>;
}
export declare let websocketService: WebSocketService;
export declare function initWebSocketService(httpServer: HttpServer): WebSocketService;
//# sourceMappingURL=websocket.d.ts.map