import { z } from 'zod';
/**
 * 主播推流 Token 请求
 */
export declare const publisherTokenSchema: z.ZodObject<{
    body: z.ZodObject<{
        roomId: z.ZodNumber;
        publicKey: z.ZodString;
        signature: z.ZodString;
        timestamp: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        publicKey: string;
        timestamp: number;
        roomId: number;
        signature: string;
    }, {
        publicKey: string;
        timestamp: number;
        roomId: number;
        signature: string;
    }>;
}, "strip", z.ZodTypeAny, {
    body: {
        publicKey: string;
        timestamp: number;
        roomId: number;
        signature: string;
    };
}, {
    body: {
        publicKey: string;
        timestamp: number;
        roomId: number;
        signature: string;
    };
}>;
/**
 * 观众观看 Token 请求
 */
export declare const viewerTokenSchema: z.ZodObject<{
    body: z.ZodObject<{
        roomId: z.ZodNumber;
        viewerAddress: z.ZodString;
        signature: z.ZodString;
        timestamp: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        timestamp: number;
        roomId: number;
        signature: string;
        viewerAddress: string;
    }, {
        timestamp: number;
        roomId: number;
        signature: string;
        viewerAddress: string;
    }>;
}, "strip", z.ZodTypeAny, {
    body: {
        timestamp: number;
        roomId: number;
        signature: string;
        viewerAddress: string;
    };
}, {
    body: {
        timestamp: number;
        roomId: number;
        signature: string;
        viewerAddress: string;
    };
}>;
/**
 * 连麦者 Token 请求
 */
export declare const coHostTokenSchema: z.ZodObject<{
    body: z.ZodObject<{
        roomId: z.ZodNumber;
        coHostAddress: z.ZodString;
        signature: z.ZodString;
        timestamp: z.ZodNumber;
        type: z.ZodDefault<z.ZodOptional<z.ZodEnum<["audio", "video"]>>>;
    }, "strip", z.ZodTypeAny, {
        timestamp: number;
        type: "audio" | "video";
        roomId: number;
        signature: string;
        coHostAddress: string;
    }, {
        timestamp: number;
        roomId: number;
        signature: string;
        coHostAddress: string;
        type?: "audio" | "video" | undefined;
    }>;
}, "strip", z.ZodTypeAny, {
    body: {
        timestamp: number;
        type: "audio" | "video";
        roomId: number;
        signature: string;
        coHostAddress: string;
    };
}, {
    body: {
        timestamp: number;
        roomId: number;
        signature: string;
        coHostAddress: string;
        type?: "audio" | "video" | undefined;
    };
}>;
/**
 * 观众离开请求
 */
export declare const viewerLeaveSchema: z.ZodObject<{
    body: z.ZodObject<{
        roomId: z.ZodNumber;
        viewerAddress: z.ZodString;
        signature: z.ZodString;
        timestamp: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        timestamp: number;
        roomId: number;
        signature: string;
        viewerAddress: string;
    }, {
        timestamp: number;
        roomId: number;
        signature: string;
        viewerAddress: string;
    }>;
}, "strip", z.ZodTypeAny, {
    body: {
        timestamp: number;
        roomId: number;
        signature: string;
        viewerAddress: string;
    };
}, {
    body: {
        timestamp: number;
        roomId: number;
        signature: string;
        viewerAddress: string;
    };
}>;
export type PublisherTokenRequest = z.infer<typeof publisherTokenSchema>;
export type ViewerTokenRequest = z.infer<typeof viewerTokenSchema>;
export type CoHostTokenRequest = z.infer<typeof coHostTokenSchema>;
export type ViewerLeaveRequest = z.infer<typeof viewerLeaveSchema>;
//# sourceMappingURL=livestream.schema.d.ts.map