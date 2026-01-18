"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validate = void 0;
const zod_1 = require("zod");
const error_middleware_js_1 = require("./error.middleware.js");
const validate = (schema) => {
    return async (req, _res, next) => {
        try {
            await schema.parseAsync({
                body: req.body,
                query: req.query,
                params: req.params,
            });
            next();
        }
        catch (error) {
            if (error instanceof zod_1.ZodError) {
                const message = error.errors
                    .map((e) => `${e.path.join('.')}: ${e.message}`)
                    .join(', ');
                next(new error_middleware_js_1.AppError('VALIDATION_ERROR', message, 400));
            }
            else {
                next(error);
            }
        }
    };
};
exports.validate = validate;
//# sourceMappingURL=validate.middleware.js.map