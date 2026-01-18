"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.websocketService = exports.WebSocketService = void 0;
exports.initWebSocketService = initWebSocketService;
const socket_io_1 = require("socket.io");
const index_js_1 = require("./config/index.js");
const logger_js_1 = require("./utils/logger.js");
class WebSocketService {
    io;
    userSockets = new Map(); // address -> socket ids
    constructor(httpServer) {
        this.io = new socket_io_1.Server(httpServer, {
            cors: {
                origin: index_js_1.config.cors.origin,
                methods: ['GET', 'POST'],
            },
            pingTimeout: 60000,
            pingInterval: 25000,
        });
        this.setupEventHandlers();
    }
    setupEventHandlers() {
        this.io.on('connection', (socket) => {
            logger_js_1.logger.info({ socketId: socket.id }, 'Client connected');
            // 用户认证 (可选)
            socket.on('auth', (address) => {
                this.registerUser(address, socket.id);
                socket.data.address = address;
                logger_js_1.logger.info({ socketId: socket.id, address }, 'User authenticated');
            });
            // 加入直播间
            socket.on('join-room', (roomId) => {
                const roomName = `room-${roomId}`;
                socket.join(roomName);
                logger_js_1.logger.info({ socketId: socket.id, roomId }, 'Joined room');
                // 通知房间内其他人
                socket.to(roomName).emit('viewer:joined', {
                    address: socket.data.address,
                });
            });
            // 离开直播间
            socket.on('leave-room', (roomId) => {
                const roomName = `room-${roomId}`;
                socket.leave(roomName);
                logger_js_1.logger.info({ socketId: socket.id, roomId }, 'Left room');
                // 通知房间内其他人
                socket.to(roomName).emit('viewer:left', {
                    address: socket.data.address,
                });
            });
            // 断开连接
            socket.on('disconnect', () => {
                if (socket.data.address) {
                    this.unregisterUser(socket.data.address, socket.id);
                }
                logger_js_1.logger.info({ socketId: socket.id }, 'Client disconnected');
            });
        });
    }
    registerUser(address, socketId) {
        if (!this.userSockets.has(address)) {
            this.userSockets.set(address, new Set());
        }
        this.userSockets.get(address).add(socketId);
    }
    unregisterUser(address, socketId) {
        const sockets = this.userSockets.get(address);
        if (sockets) {
            sockets.delete(socketId);
            if (sockets.size === 0) {
                this.userSockets.delete(address);
            }
        }
    }
    /**
     * 广播到指定直播间
     */
    broadcastToRoom(roomId, event, data) {
        const roomName = `room-${roomId}`;
        this.io.to(roomName).emit(event, data);
        logger_js_1.logger.debug({ roomId, event }, 'Broadcast to room');
    }
    /**
     * 广播到所有连接
     */
    broadcastToAll(event, data) {
        this.io.emit(event, data);
        logger_js_1.logger.debug({ event }, 'Broadcast to all');
    }
    /**
     * 通知特定用户
     */
    notifyUser(address, event, data) {
        const socketIds = this.userSockets.get(address);
        if (socketIds) {
            socketIds.forEach((socketId) => {
                this.io.to(socketId).emit(event, data);
            });
            logger_js_1.logger.debug({ address, event }, 'Notified user');
        }
    }
    /**
     * 获取直播间在线人数
     */
    async getRoomViewerCount(roomId) {
        const roomName = `room-${roomId}`;
        const sockets = await this.io.in(roomName).fetchSockets();
        return sockets.length;
    }
    /**
     * 踢出用户
     */
    async kickUserFromRoom(roomId, address) {
        const roomName = `room-${roomId}`;
        const socketIds = this.userSockets.get(address);
        if (socketIds) {
            for (const socketId of socketIds) {
                const socket = this.io.sockets.sockets.get(socketId);
                if (socket && socket.rooms.has(roomName)) {
                    socket.leave(roomName);
                    socket.emit('viewer:kicked', { roomId, message: 'You have been kicked' });
                }
            }
        }
    }
}
exports.WebSocketService = WebSocketService;
function initWebSocketService(httpServer) {
    exports.websocketService = new WebSocketService(httpServer);
    return exports.websocketService;
}
//# sourceMappingURL=websocket.js.map