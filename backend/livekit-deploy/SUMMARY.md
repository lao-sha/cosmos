# LiveKit 自部署方案总结

## 📦 已创建的文件

```
backend/livekit-deploy/
├── README.md              # 完整部署文档（详细）
├── QUICKSTART.md          # 快速开始指南（5分钟）
├── INSTALL.md             # 安装指南（前置要求）
├── SUMMARY.md             # 本文件（总结）
├── livekit.yaml           # LiveKit 配置文件
├── docker-compose.yml     # Docker Compose 配置
├── nginx.conf             # Nginx 反向代理配置
├── quick-start.sh         # 一键部署脚本（需要 docker-compose）
├── start-simple.sh        # 简单启动脚本（只需要 docker）
└── test-livekit.sh        # 测试脚本
```

---

## 🚀 快速开始

### 最简单的方式（推荐）

```bash
cd backend/livekit-deploy

# 如果有 docker 权限
./start-simple.sh

# 如果没有 docker 权限
sudo ./start-simple.sh
```

脚本会自动：
1. 生成 API 密钥
2. 启动 LiveKit 容器
3. 测试连接
4. 显示配置信息

---

## 📋 部署方案对比

| 方案 | 文件 | 适用场景 | 难度 |
|------|------|----------|------|
| 简单启动 | `start-simple.sh` | 开发测试 | ⭐ |
| Docker Compose | `docker-compose.yml` | 小规模生产 | ⭐⭐ |
| 完整部署 | `README.md` | 大规模生产 | ⭐⭐⭐ |

---

## 🔧 配置步骤

### 1. 启动 LiveKit

```bash
# 运行启动脚本
./start-simple.sh
```

**输出示例**:
```
API Key: 2918aa73fa0ea0a2d796c24930a797d7
API Secret: wGGpf9/hutJ2wllxSJDktPPxRFXy2K+Wzy4HjRwc4I4=
```

### 2. 更新后端配置

编辑 `backend/.env`:
```bash
LIVEKIT_URL=ws://localhost:7880
LIVEKIT_API_KEY=2918aa73fa0ea0a2d796c24930a797d7
LIVEKIT_API_SECRET=wGGpf9/hutJ2wllxSJDktPPxRFXy2K+Wzy4HjRwc4I4=
```

### 3. 重启后端

```bash
cd backend
npm run dev
```

### 4. 验证连接

```bash
curl http://localhost:3001/health
```

应该看到：
```json
{
  "services": {
    "livekit": "connected"
  }
}
```

---

## 📊 端口说明

| 端口 | 协议 | 用途 | 必需 |
|------|------|------|------|
| 7880 | TCP | HTTP/WebSocket API | ✅ |
| 7881 | TCP | TURN 服务器 | ✅ |
| 7882 | UDP | TURN 服务器 | ✅ |
| 50000-50100 | UDP | WebRTC 媒体 | ✅ |

---

## 🔍 常用命令

### 容器管理

```bash
# 查看状态
docker ps | grep livekit

# 查看日志
docker logs -f livekit-server

# 停止服务
docker stop livekit-server

# 启动服务
docker start livekit-server

# 重启服务
docker restart livekit-server

# 删除容器
docker rm -f livekit-server
```

### 测试

```bash
# 测试 HTTP 连接
curl http://localhost:7880

# 测试后端健康检查
curl http://localhost:3001/health

# 运行完整测试
./test-livekit.sh
```

---

## 🛠️ 配置文件说明

### livekit.yaml

核心配置文件，包含：
- API 密钥
- RTC 配置（端口、IP）
- 房间配置（最大参与者）
- 日志级别

**关键配置**:
```yaml
keys:
  APIxxxxxxx: secretxxxxxxx  # 替换为真实密钥

rtc:
  use_external_ip: true      # NAT 穿透
  port_range_start: 50000
  port_range_end: 50100
```

### docker-compose.yml

Docker Compose 配置，包含：
- LiveKit 服务器
- Redis（可选）
- Nginx（可选）

### nginx.conf

Nginx 反向代理配置，用于：
- HTTPS 支持
- 负载均衡
- SSL 终止

---

## 🔐 安全建议

### 1. 生成强密钥

```bash
# 使用随机生成的密钥
openssl rand -hex 16        # API Key
openssl rand -base64 32     # API Secret
```

### 2. 使用 HTTPS（生产环境）

```bash
# 获取免费 SSL 证书
sudo certbot certonly --standalone -d your-domain.com
```

### 3. 配置防火墙

```bash
# 只开放必要的端口
sudo ufw allow 7880/tcp
sudo ufw allow 7881/tcp
sudo ufw allow 7882/udp
sudo ufw allow 50000:50100/udp
```

### 4. 定期更新

```bash
# 更新 LiveKit 镜像
docker pull livekit/livekit-server:latest
docker stop livekit-server
docker rm livekit-server
./start-simple.sh
```

---

## 📈 性能优化

### 1. 调整并发数

```yaml
# livekit.yaml
room:
  max_participants: 100  # 根据服务器性能调整
```

### 2. 启用 Redis

```bash
# 启动 Redis
docker run -d --name livekit-redis -p 6379:6379 redis:7-alpine

# 更新 livekit.yaml
redis:
  address: localhost:6379
```

### 3. 系统优化

```bash
# 增加文件描述符
echo "* soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65536" | sudo tee -a /etc/security/limits.conf
```

---

## 🐛 故障排查

### 问题 1: Docker 权限错误

```bash
# 添加用户到 docker 组
sudo usermod -aG docker $USER
newgrp docker
```

### 问题 2: 端口被占用

```bash
# 查找占用端口的进程
sudo lsof -i :7880

# 停止容器
docker stop $(docker ps -q --filter "publish=7880")
```

### 问题 3: 容器无法启动

```bash
# 查看日志
docker logs livekit-server

# 检查配置
docker inspect livekit-server
```

### 问题 4: WebRTC 连接失败

```bash
# 检查 UDP 端口
sudo ufw status

# 开放端口
sudo ufw allow 50000:50100/udp
```

---

## 📚 文档索引

### 快速开始
- [QUICKSTART.md](./QUICKSTART.md) - 5 分钟快速部署
- [INSTALL.md](./INSTALL.md) - 详细安装步骤

### 完整文档
- [README.md](./README.md) - 完整部署方案
  - Docker 单机部署
  - Docker Compose 部署
  - 生产环境部署
  - 多实例负载均衡

### 脚本
- `start-simple.sh` - 简单启动（推荐）
- `quick-start.sh` - 一键部署（需要 docker-compose）
- `test-livekit.sh` - 测试脚本

---

## ✅ 部署检查清单

- [ ] Docker 已安装
- [ ] LiveKit 容器已启动
- [ ] API 密钥已生成
- [ ] 后端配置已更新
- [ ] 后端服务已重启
- [ ] 健康检查通过
- [ ] 防火墙已配置（生产环境）
- [ ] SSL 证书已配置（生产环境）

---

## 🎯 下一步

1. ✅ LiveKit 已部署
2. ⏭️ 在链上创建直播间
3. ⏭️ 前端获取 Token
4. ⏭️ 测试推流功能
5. ⏭️ 测试观看功能

---

## 💡 提示

### 开发环境
- 使用 `ws://localhost:7880`
- 不需要 HTTPS
- 可以使用测试密钥

### 生产环境
- 使用 `wss://your-domain.com`
- 必须使用 HTTPS
- 使用强密钥
- 配置防火墙
- 启用监控

---

## 📞 获取帮助

- **完整文档**: [README.md](./README.md)
- **快速开始**: [QUICKSTART.md](./QUICKSTART.md)
- **安装指南**: [INSTALL.md](./INSTALL.md)
- **LiveKit 官方**: https://docs.livekit.io/
- **GitHub Issues**: https://github.com/livekit/livekit/issues

---

## 🎉 总结

LiveKit 自部署方案已完整准备：

✅ **配置文件**: 已创建并优化
✅ **启动脚本**: 一键部署
✅ **测试脚本**: 自动验证
✅ **完整文档**: 详细说明

**只需运行**:
```bash
./start-simple.sh
```

**然后更新后端配置，即可开始使用！**

---

**更新时间**: 2026-01-17
**版本**: 1.0.0
**状态**: ✅ 就绪
