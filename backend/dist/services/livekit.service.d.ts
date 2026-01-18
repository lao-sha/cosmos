export interface TokenOptions {
    identity: string;
    name?: string;
    roomName: string;
    canPublish: boolean;
    canSubscribe: boolean;
    canPublishData: boolean;
}
export declare class LiveKitService {
    private apiKey;
    private apiSecret;
    private url;
    private roomService;
    constructor();
    /**
     * 生成 LiveKit 访问 Token
     */
    generateToken(options: TokenOptions): Promise<string>;
    /**
     * 生成主播 Token
     */
    generatePublisherToken(roomId: number, hostAddress: string): Promise<string>;
    /**
     * 生成观众 Token
     */
    generateViewerToken(roomId: number, viewerAddress: string): Promise<string>;
    /**
     * 生成连麦者 Token
     */
    generateCoHostToken(roomId: number, coHostAddress: string): Promise<string>;
    /**
     * 获取 LiveKit 服务器 URL
     */
    getServerUrl(): string;
    /**
     * 计算 Token 过期时间
     */
    getTokenExpiry(): number;
    /**
     * 获取房间参与者数量
     */
    getRoomParticipantCount(roomName: string): Promise<number>;
    /**
     * 检查 LiveKit 服务连接状态
     */
    healthCheck(): Promise<boolean>;
    /**
     * 踢出参与者
     */
    removeParticipant(roomName: string, identity: string): Promise<void>;
}
export declare const livekitService: LiveKitService;
//# sourceMappingURL=livekit.service.d.ts.map