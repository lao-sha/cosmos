import { Request, Response, NextFunction } from 'express';
export declare class AppError extends Error {
    code: string;
    statusCode: number;
    constructor(code: string, message: string, statusCode?: number);
}
export declare function errorMiddleware(err: Error, req: Request, res: Response, _next: NextFunction): Response<any, Record<string, any>> | undefined;
//# sourceMappingURL=error.middleware.d.ts.map