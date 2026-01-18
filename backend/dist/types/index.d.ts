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
export interface Gift {
    id: number;
    name: string;
    price: string;
    iconCid: string;
    enabled: boolean;
}
export interface TokenResponse {
    token: string;
    url: string;
    roomName: string;
    expiresAt: number;
}
export interface ErrorResponse {
    error: {
        code: string;
        message: string;
    };
}
export interface RoomListResponse {
    rooms: (LiveRoom & {
        currentViewers: number;
    })[];
    total: number;
    page: number;
    limit: number;
}
export interface HealthResponse {
    status: 'ok' | 'degraded';
    timestamp: number;
    services: {
        redis: 'connected' | 'disconnected';
        livekit: 'connected' | 'disconnected';
        chain: 'connected' | 'disconnected';
    };
}
//# sourceMappingURL=index.d.ts.map