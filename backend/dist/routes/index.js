"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const express_1 = require("express");
const livestream_routes_js_1 = __importDefault(require("./livestream.routes.js"));
const router = (0, express_1.Router)();
router.use('/livestream', livestream_routes_js_1.default);
exports.default = router;
//# sourceMappingURL=index.js.map