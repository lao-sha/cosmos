export interface SignatureVerifyResult {
    isValid: boolean;
    error?: string;
}
export declare class SignatureService {
    /**
     * 验证签名
     */
    verifySignature(message: string, signature: string, publicKey: string): SignatureVerifyResult;
    /**
     * 验证时间戳 (防重放攻击)
     */
    verifyTimestamp(timestamp: number): SignatureVerifyResult;
    /**
     * 构造签名消息
     */
    buildSignatureMessage(roomId: number, timestamp: number): string;
    /**
     * 完整验证流程
     */
    verifyPublisherSignature(roomId: number, publicKey: string, signature: string, timestamp: number): SignatureVerifyResult;
}
export declare const signatureService: SignatureService;
//# sourceMappingURL=signature.service.d.ts.map