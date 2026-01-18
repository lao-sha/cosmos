# LiveKit 快速开始

5 分钟内完成 LiveKit 自部署。

---

## 方式 1: 一键部署（推荐）

```bash
cd backend/livekit-deploy
./quick-start.sh
```

脚本会自动：
1. ✅ 检查 Docker 环境
2. ✅ 生成 API 密钥
3. ✅ 更新配置文件
4. ✅ 启动服务
5. ✅ 测试连接

---

## 方式 2: 手动部署

### 1. 生成 API 密钥

```bash
# API Key
openssl rand -hex 16

# API Secret
openssl rand -base64 32
```

### 2. 更新配置

编辑 `livekit.yaml`，替换密钥：
```yaml
keys:
  APIxxxxxxx: secretxxxxxxx
```

### 3. 启动服务

```bash
docker-compose up -d
```

### 4. 验证服务

```bash
# 检查状态
docker-compose ps

# 查看日志
docker-compose logs -f livekit

# 测试连接
curl http://localhost:7880
```

---

## 更新后端配置

编辑 `backend/.env`:
```bash
LIVEKIT_URL=ws://localhost:7880
LIVEKIT_API_KEY=你的API_KEY
LIVEKIT_API_SECRET=你的API_SECRET
```

重启后端:
```bash
cd backend
npm run dev
```

---

## 测试 LiveKit

```bash
./test-livekit.sh
```

---

## 常用命令

```bash
# 启动服务
docker-compose up -d

# 停止服务
docker-compose down

# 重启服务
docker-compose restart

# 查看日志
docker-compose logs -f livekit

# 查看状态
docker-compose ps

# 查看资源使用
docker stats livekit-server
```

---

## 端口说明

| 端口 | 协议 | 用途 |
|------|------|------|
| 7880 | TCP | HTTP/WebSocket API |
| 7881 | TCP | TURN 服务器 |
| 7882 | UDP | TURN 服务器 |
| 50000-50100 | UDP | WebRTC 媒体 |

---

## 防火墙配置

如果部署在云服务器，需要开放端口：

```bash
# UFW
sudo ufw allow 7880/tcp
sudo ufw allow 7881/tcp
sudo ufw allow 7882/udp
sudo ufw allow 50000:50100/udp

# 或使用云服务商的安全组配置
```

---

## 故障排查

### 问题 1: 容器无法启动

```bash
# 查看详细日志
docker-compose logs livekit

# 检查配置文件
docker-compose config
```

### 问题 2: 无法连接

```bash
# 检查端口
netstat -tuln | grep 7880

# 检查防火墙
sudo ufw status
```

### 问题 3: WebRTC 连接失败

检查 `livekit.yaml` 配置：
```yaml
rtc:
  use_external_ip: true
  # 如果有公网 IP，明确指定
  external_ip: "your.public.ip"
```

---

## 生产环境部署

### 1. 使用域名和 HTTPS

1. 获取 SSL 证书（Let's Encrypt）:
```bash
sudo certbot certonly --standalone -d your-domain.com
```

2. 复制证书:
```bash
mkdir -p ssl
sudo cp /etc/letsencrypt/live/your-domain.com/fullchain.pem ./ssl/
sudo cp /etc/letsencrypt/live/your-domain.com/privkey.pem ./ssl/
```

3. 启用 Nginx:
编辑 `docker-compose.yml`，取消注释 nginx 部分

4. 更新 `nginx.conf` 中的域名

5. 重启服务:
```bash
docker-compose up -d
```

### 2. 启用 Redis（多实例）

Redis 已包含在 `docker-compose.yml` 中，默认启动。

在 `livekit.yaml` 中启用：
```yaml
redis:
  address: redis:6379
```

---

## 性能优化

### 1. 调整并发数

```yaml
# livekit.yaml
room:
  max_participants: 100  # 根据服务器性能调整
```

### 2. 系统优化

```bash
# 增加文件描述符
echo "* soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65536" | sudo tee -a /etc/security/limits.conf

# 优化网络
sudo sysctl -w net.core.rmem_max=134217728
sudo sysctl -w net.core.wmem_max=134217728
```

---

## 监控

### 查看实时日志

```bash
docker-compose logs -f livekit
```

### 查看资源使用

```bash
docker stats livekit-server
```

### 健康检查

```bash
curl http://localhost:7880/health
```

---

## 下一步

1. ✅ LiveKit 服务已启动
2. ⏭️ 更新后端配置
3. ⏭️ 重启后端服务
4. ⏭️ 测试推流功能
5. ⏭️ 测试观看功能

---

## 获取帮助

- [完整文档](./README.md)
- [LiveKit 官方文档](https://docs.livekit.io/)
- [故障排查指南](./README.md#故障排查)

---

**部署完成！开始使用 LiveKit 吧！** 🎉
