export declare function setWebSocketService(ws: any): void;
export interface LivestreamEvent {
    type: string;
    roomId?: number;
    data: any;
    blockNumber: number;
    timestamp: number;
}
export declare class ChainEventsService {
    private unsubscribe;
    /**
     * 开始监听链上事件
     */
    startListening(): Promise<void>;
    /**
     * 停止监听
     */
    stopListening(): void;
    /**
     * 处理直播模块事件
     */
    private handleLivestreamEvent;
    private onRoomCreated;
    private onLiveStarted;
    private onLiveEnded;
    private onGiftSent;
    private onCoHostStarted;
    private onCoHostEnded;
    private onViewerKicked;
    private onViewerUnbanned;
    private onRoomBanned;
}
export declare const chainEventsService: ChainEventsService;
//# sourceMappingURL=chain-events.service.d.ts.map