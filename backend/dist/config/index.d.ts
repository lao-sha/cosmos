export declare const config: {
    port: number;
    nodeEnv: string;
    livekit: {
        url: string;
        apiKey: string;
        apiSecret: string;
        tokenTTL: string;
    };
    chain: {
        wsEndpoint: string;
    };
    redis: {
        url: string;
        keyPrefix: string;
        ttl: {
            room: number;
            gift: number;
            blacklist: number;
            coHosts: number;
        };
    };
    security: {
        signatureMaxAge: number;
        rateLimitWindow: number;
        rateLimitMax: number;
    };
    cors: {
        origin: string;
    };
};
export declare function validateConfig(): void;
//# sourceMappingURL=index.d.ts.map