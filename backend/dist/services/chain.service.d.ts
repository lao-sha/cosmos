import { ApiPromise } from '@polkadot/api';
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
export declare class ChainService {
    private api;
    private connecting;
    /**
     * 连接到 Substrate 节点
     */
    connect(): Promise<ApiPromise>;
    private doConnect;
    /**
     * 获取直播间信息 (带缓存)
     */
    getRoom(roomId: number): Promise<LiveRoom | null>;
    /**
     * 检查是否是直播间主播
     */
    isRoomHost(roomId: number, address: string): Promise<boolean>;
    /**
     * 检查直播间状态
     */
    getRoomStatus(roomId: number): Promise<string | null>;
    /**
     * 检查观众是否有门票
     */
    hasTicket(roomId: number, viewerAddress: string): Promise<boolean>;
    /**
     * 检查观众是否被封禁 (带缓存)
     */
    isViewerBanned(roomId: number, viewerAddress: string): Promise<boolean>;
    /**
     * 获取连麦者列表 (带缓存)
     */
    getCoHosts(roomId: number): Promise<string[]>;
    /**
     * 检查是否是连麦者
     */
    isCoHost(roomId: number, address: string): Promise<boolean>;
    /**
     * 获取直播间列表
     */
    getRooms(options: {
        status?: string;
        roomType?: string;
        page?: number;
        limit?: number;
    }): Promise<{
        rooms: LiveRoom[];
        total: number;
    }>;
    /**
     * 获取主播累计收益
     */
    getHostEarnings(host: string): Promise<string>;
    /**
     * 检查连接状态
     */
    isConnected(): boolean;
    /**
     * 获取 API 实例 (用于事件监听)
     */
    getApi(): ApiPromise | null;
    /**
     * 解析链上直播间数据
     */
    private parseRoom;
}
export declare const chainService: ChainService;
//# sourceMappingURL=chain.service.d.ts.map