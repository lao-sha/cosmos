"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.AppError = void 0;
exports.errorMiddleware = errorMiddleware;
const logger_js_1 = require("../utils/logger.js");
class AppError extends Error {
    code;
    statusCode;
    constructor(code, message, statusCode = 500) {
        super(message);
        this.code = code;
        this.statusCode = statusCode;
        this.name = 'AppError';
    }
}
exports.AppError = AppError;
function errorMiddleware(err, req, res, _next) {
    if (err instanceof AppError) {
        logger_js_1.logger.warn({
            code: err.code,
            message: err.message,
            path: req.path,
        }, 'Application error');
        return res.status(err.statusCode).json({
            error: {
                code: err.code,
                message: err.message,
            },
        });
    }
    // 未知错误
    logger_js_1.logger.error({
        error: err,
        path: req.path,
        method: req.method,
    }, 'Unexpected error');
    res.status(500).json({
        error: {
            code: 'INTERNAL_ERROR',
            message: 'An unexpected error occurred',
        },
    });
}
//# sourceMappingURL=error.middleware.js.map