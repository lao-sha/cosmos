# LiveKit 自部署方案

完整的 LiveKit 服务器自部署指南，包括 Docker、Docker Compose 和生产环境配置。

---

## 方案概览

### 部署选项

| 方案 | 适用场景 | 难度 | 成本 |
|------|----------|------|------|
| Docker 单机 | 开发测试 | ⭐ | 免费 |
| Docker Compose | 小规模生产 | ⭐⭐ | 低 |
| Kubernetes | 大规模生产 | ⭐⭐⭐⭐ | 中高 |
| LiveKit Cloud | 快速上线 | ⭐ | 按量付费 |

**推荐**: 开发测试使用 Docker，生产环境使用 Docker Compose + Nginx。

---

## 方案 1: Docker 单机部署（开发测试）

### 1.1 快速启动

```bash
docker run -d \
  --name livekit \
  -p 7880:7880 \
  -p 7881:7881 \
  -p 7882:7882/udp \
  -p 50000-50100:50000-50100/udp \
  livekit/livekit-server:latest
```

**端口说明**:
- `7880`: HTTP API 和 WebSocket
- `7881`: TURN/STUN 服务器（TCP）
- `7882`: TURN/STUN 服务器（UDP）
- `50000-50100`: WebRTC 媒体端口（UDP）

### 1.2 使用配置文件

创建 `livekit.yaml`:
```yaml
port: 7880
rtc:
  port_range_start: 50000
  port_range_end: 50100
  use_external_ip: true
  
keys:
  # 生成方式: openssl rand -base64 32
  APIxxxxxxx: secretxxxxxxx
```

启动容器:
```bash
docker run -d \
  --name livekit \
  -p 7880:7880 \
  -p 7881:7881 \
  -p 7882:7882/udp \
  -p 50000-50100:50000-50100/udp \
  -v $(pwd)/livekit.yaml:/etc/livekit.yaml \
  livekit/livekit-server:latest \
  --config /etc/livekit.yaml
```

### 1.3 测试连接

```bash
# 检查服务状态
curl http://localhost:7880

# 查看日志
docker logs -f livekit
```

---

## 方案 2: Docker Compose 部署（推荐）

### 2.1 项目结构

```
livekit-deploy/
├── docker-compose.yml
├── livekit.yaml
├── nginx.conf
├── .env
└── README.md
```

### 2.2 创建配置文件

#### livekit.yaml
```yaml
# LiveKit 服务器配置
port: 7880
bind_addresses:
  - "0.0.0.0"

# RTC 配置
rtc:
  # UDP 端口范围
  port_range_start: 50000
  port_range_end: 50100
  
  # 使用外部 IP（重要！）
  use_external_ip: true
  
  # 如果有固定公网 IP，在这里配置
  # external_ip: "your.public.ip"
  
  # TCP 回退（防火墙限制时使用）
  tcp_port: 7881
  
  # TURN 服务器配置
  turn_servers:
    - host: 0.0.0.0
      port: 3478
      protocol: udp

# API 密钥配置
keys:
  # 格式: API_KEY: API_SECRET
  # 生成方式: openssl rand -base64 32
  devkey: secret
  
# 日志配置
logging:
  level: info
  # 可选: debug, info, warn, error
  
# Redis 配置（可选，用于多实例）
# redis:
#   address: redis:6379

# 房间配置
room:
  # 空房间自动关闭时间（秒）
  auto_create: true
  empty_timeout: 300
  
  # 最大参与者数量
  max_participants: 100

# 限流配置
limit:
  # 每秒最大请求数
  bytes_per_sec: 10000000  # 10MB/s
```

#### docker-compose.yml
```yaml
version: '3.8'

services:
  livekit:
    image: livekit/livekit-server:latest
    container_name: livekit-server
    command: --config /etc/livekit.yaml
    restart: unless-stopped
    ports:
      - "7880:7880"      # HTTP/WebSocket
      - "7881:7881"      # TURN TCP
      - "7882:7882/udp"  # TURN UDP
      - "50000-50100:50000-50100/udp"  # WebRTC 媒体
    volumes:
      - ./livekit.yaml:/etc/livekit.yaml:ro
    networks:
      - livekit-net
    environment:
      - LIVEKIT_CONFIG=/etc/livekit.yaml
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:7880"]
      interval: 30s
      timeout: 10s
      retries: 3

  # Nginx 反向代理（可选，用于 HTTPS）
  nginx:
    image: nginx:alpine
    container_name: livekit-nginx
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro  # SSL 证书目录
    networks:
      - livekit-net
    depends_on:
      - livekit

  # Redis（可选，用于多实例负载均衡）
  redis:
    image: redis:7-alpine
    container_name: livekit-redis
    restart: unless-stopped
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    networks:
      - livekit-net
    command: redis-server --appendonly yes

networks:
  livekit-net:
    driver: bridge

volumes:
  redis-data:
```

#### nginx.conf
```nginx
events {
    worker_connections 1024;
}

http {
    upstream livekit {
        server livekit:7880;
    }

    # HTTP 重定向到 HTTPS
    server {
        listen 80;
        server_name your-domain.com;
        
        location / {
            return 301 https://$server_name$request_uri;
        }
    }

    # HTTPS 配置
    server {
        listen 443 ssl http2;
        server_name your-domain.com;

        # SSL 证书（使用 Let's Encrypt）
        ssl_certificate /etc/nginx/ssl/fullchain.pem;
        ssl_certificate_key /etc/nginx/ssl/privkey.pem;
        
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers HIGH:!aNULL:!MD5;
        ssl_prefer_server_ciphers on;

        # WebSocket 支持
        location / {
            proxy_pass http://livekit;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            
            # 超时设置
            proxy_connect_timeout 60s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;
        }
    }
}
```

### 2.3 启动服务

```bash
# 启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f livekit

# 查看状态
docker-compose ps

# 停止服务
docker-compose down
```

### 2.4 生成 API 密钥

```bash
# 生成 API Key
openssl rand -hex 16

# 生成 API Secret
openssl rand -base64 32
```

更新 `livekit.yaml` 中的 `keys` 部分：
```yaml
keys:
  APIa1b2c3d4e5f6g7h8: c2VjcmV0a2V5MTIzNDU2Nzg5MGFiY2RlZg==
```

---

## 方案 3: 生产环境部署

### 3.1 域名和 SSL 证书

#### 使用 Let's Encrypt（免费）

```bash
# 安装 certbot
sudo apt install certbot

# 获取证书
sudo certbot certonly --standalone -d your-domain.com

# 证书位置
# /etc/letsencrypt/live/your-domain.com/fullchain.pem
# /etc/letsencrypt/live/your-domain.com/privkey.pem

# 复制到项目目录
sudo cp /etc/letsencrypt/live/your-domain.com/fullchain.pem ./ssl/
sudo cp /etc/letsencrypt/live/your-domain.com/privkey.pem ./ssl/
```

#### 自动续期

```bash
# 添加 cron 任务
sudo crontab -e

# 每月 1 号凌晨 2 点续期
0 2 1 * * certbot renew --quiet && docker-compose restart nginx
```

### 3.2 防火墙配置

```bash
# UFW 防火墙
sudo ufw allow 80/tcp      # HTTP
sudo ufw allow 443/tcp     # HTTPS
sudo ufw allow 7881/tcp    # TURN TCP
sudo ufw allow 7882/udp    # TURN UDP
sudo ufw allow 50000:50100/udp  # WebRTC 媒体

# 启用防火墙
sudo ufw enable
```

### 3.3 系统优化

#### 增加文件描述符限制

```bash
# 编辑 /etc/security/limits.conf
sudo nano /etc/security/limits.conf

# 添加以下内容
* soft nofile 65536
* hard nofile 65536
```

#### 优化网络参数

```bash
# 编辑 /etc/sysctl.conf
sudo nano /etc/sysctl.conf

# 添加以下内容
net.core.rmem_max = 134217728
net.core.wmem_max = 134217728
net.ipv4.tcp_rmem = 4096 87380 67108864
net.ipv4.tcp_wmem = 4096 65536 67108864
net.ipv4.udp_mem = 65536 131072 262144

# 应用配置
sudo sysctl -p
```

### 3.4 监控和日志

#### 使用 Prometheus + Grafana

```yaml
# 添加到 docker-compose.yml
  prometheus:
    image: prom/prometheus:latest
    container_name: livekit-prometheus
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    ports:
      - "9090:9090"
    networks:
      - livekit-net

  grafana:
    image: grafana/grafana:latest
    container_name: livekit-grafana
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
    networks:
      - livekit-net
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
```

#### prometheus.yml
```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'livekit'
    static_configs:
      - targets: ['livekit:7880']
```

---

## 方案 4: 多实例负载均衡

### 4.1 架构图

```
                    ┌─────────────┐
                    │   Nginx     │
                    │ Load Balancer│
                    └──────┬──────┘
                           │
        ┏━━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━━━━━┓
        ▼                                      ▼
┌───────────────┐                    ┌───────────────┐
│  LiveKit 1    │                    │  LiveKit 2    │
│  (7880)       │◄──────Redis───────►│  (7880)       │
└───────────────┘                    └───────────────┘
```

### 4.2 配置 Redis

在 `livekit.yaml` 中启用 Redis：
```yaml
redis:
  address: redis:6379
  # 可选: 密码
  # password: your-redis-password
```

### 4.3 Nginx 负载均衡配置

```nginx
upstream livekit_cluster {
    least_conn;  # 最少连接算法
    server livekit1:7880 max_fails=3 fail_timeout=30s;
    server livekit2:7880 max_fails=3 fail_timeout=30s;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;
    
    location / {
        proxy_pass http://livekit_cluster;
        # ... 其他配置
    }
}
```

---

## 测试和验证

### 1. 健康检查

```bash
# 检查服务状态
curl http://localhost:7880

# 预期响应: 200 OK
```

### 2. 生成测试 Token

创建 `test-token.js`:
```javascript
const { AccessToken } = require('livekit-server-sdk');

const apiKey = 'APIxxxxxxx';
const apiSecret = 'secretxxxxxxx';

const token = new AccessToken(apiKey, apiSecret, {
  identity: 'test-user',
  name: 'Test User',
  ttl: '1h'
});

token.addGrant({
  room: 'test-room',
  roomJoin: true,
  canPublish: true,
  canSubscribe: true
});

console.log('Token:', token.toJwt());
```

运行:
```bash
node test-token.js
```

### 3. 使用 LiveKit CLI 测试

```bash
# 安装 LiveKit CLI
brew install livekit

# 或
curl -sSL https://get.livekit.io/cli | bash

# 创建测试房间
livekit-cli create-room \
  --url http://localhost:7880 \
  --api-key APIxxxxxxx \
  --api-secret secretxxxxxxx \
  --room test-room

# 列出房间
livekit-cli list-rooms \
  --url http://localhost:7880 \
  --api-key APIxxxxxxx \
  --api-secret secretxxxxxxx
```

---

## 更新后端配置

更新 `backend/.env`:
```bash
# 本地部署
LIVEKIT_URL=ws://localhost:7880

# 或使用域名（生产环境）
LIVEKIT_URL=wss://your-domain.com

LIVEKIT_API_KEY=APIxxxxxxx
LIVEKIT_API_SECRET=secretxxxxxxx
LIVEKIT_TOKEN_TTL=6h
```

重启后端服务:
```bash
cd backend
npm run dev
```

---

## 故障排查

### 问题 1: 无法连接到 LiveKit

**检查**:
```bash
# 检查容器状态
docker ps | grep livekit

# 查看日志
docker logs livekit

# 检查端口
netstat -tuln | grep 7880
```

### 问题 2: WebRTC 连接失败

**原因**: UDP 端口未开放或防火墙阻止

**解决**:
```bash
# 检查防火墙
sudo ufw status

# 开放 UDP 端口
sudo ufw allow 50000:50100/udp
```

### 问题 3: TURN 服务器不工作

**检查配置**:
```yaml
rtc:
  use_external_ip: true
  # 如果有公网 IP，明确指定
  external_ip: "your.public.ip"
```

### 问题 4: SSL 证书错误

**检查证书**:
```bash
# 验证证书
openssl x509 -in ./ssl/fullchain.pem -text -noout

# 检查证书有效期
openssl x509 -in ./ssl/fullchain.pem -noout -dates
```

---

## 性能优化

### 1. 调整并发连接数

```yaml
# livekit.yaml
room:
  max_participants: 100  # 根据服务器性能调整
```

### 2. 启用 Redis 缓存

```yaml
redis:
  address: redis:6379
```

### 3. 使用 CDN

将静态资源（如 SDK）托管到 CDN，减轻服务器压力。

### 4. 监控资源使用

```bash
# 查看容器资源使用
docker stats livekit

# 查看系统资源
htop
```

---

## 成本估算

### 服务器配置建议

| 并发用户 | CPU | 内存 | 带宽 | 月成本（AWS） |
|---------|-----|------|------|--------------|
| 10-50   | 2核 | 4GB  | 100Mbps | $20-40 |
| 50-200  | 4核 | 8GB  | 500Mbps | $80-150 |
| 200-500 | 8核 | 16GB | 1Gbps | $200-400 |

### 带宽计算

```
单个视频流: 1-3 Mbps
音频流: 50-100 Kbps

100 个观众观看 720p 直播:
100 × 2 Mbps = 200 Mbps
```

---

## 安全建议

1. **使用强密码**: API Key 和 Secret 使用随机生成
2. **启用 HTTPS**: 生产环境必须使用 SSL
3. **限制访问**: 使用防火墙限制不必要的端口
4. **定期更新**: 及时更新 LiveKit 版本
5. **监控日志**: 定期检查异常访问

---

## 相关资源

- [LiveKit 官方文档](https://docs.livekit.io/)
- [LiveKit GitHub](https://github.com/livekit/livekit)
- [LiveKit 示例](https://github.com/livekit/livekit-examples)
- [Docker Hub](https://hub.docker.com/r/livekit/livekit-server)

---

## 总结

**推荐部署方案**:
- 开发测试: Docker 单机
- 小规模生产: Docker Compose + Nginx
- 大规模生产: Kubernetes + 负载均衡

**关键配置**:
- 正确配置外部 IP
- 开放必要的 UDP 端口
- 使用 HTTPS（生产环境）
- 启用 Redis（多实例）

**下一步**:
1. 选择合适的部署方案
2. 配置域名和 SSL
3. 生成 API 密钥
4. 更新后端配置
5. 测试推流和观看
