export declare class CacheService {
    private redis;
    private prefix;
    constructor();
    private key;
    getRoom(roomId: number): Promise<any | null>;
    setRoom(roomId: number, room: any): Promise<void>;
    invalidateRoom(roomId: number): Promise<void>;
    getGifts(): Promise<any[] | null>;
    setGifts(gifts: any[]): Promise<void>;
    isBlacklisted(roomId: number, user: string): Promise<boolean | null>;
    setBlacklisted(roomId: number, user: string, banned: boolean): Promise<void>;
    getCoHosts(roomId: number): Promise<string[] | null>;
    setCoHosts(roomId: number, coHosts: string[]): Promise<void>;
    invalidateCoHosts(roomId: number): Promise<void>;
    incrViewerCount(roomId: number): Promise<number>;
    decrViewerCount(roomId: number): Promise<number>;
    getViewerCount(roomId: number): Promise<number>;
    resetViewerCount(roomId: number): Promise<void>;
    ping(): Promise<boolean>;
    disconnect(): Promise<void>;
}
export declare const cacheService: CacheService;
//# sourceMappingURL=cache.service.d.ts.map