# LiveKit 安装指南

## 前置要求

### 1. 安装 Docker

```bash
# Ubuntu/Debian
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# 添加当前用户到 docker 组（避免每次使用 sudo）
sudo usermod -aG docker $USER

# 重新登录或运行
newgrp docker

# 验证安装
docker --version
```

### 2. 安装 Docker Compose（可选）

```bash
# 方式 1: 使用 apt（推荐）
sudo apt update
sudo apt install docker-compose-plugin

# 方式 2: 手动安装
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# 验证安装
docker-compose --version
```

---

## 快速部署

### 方式 1: 使用简单脚本（推荐）

```bash
cd backend/livekit-deploy

# 如果没有 docker 权限，使用 sudo
sudo ./start-simple.sh

# 或者添加用户到 docker 组后
./start-simple.sh
```

### 方式 2: 手动启动

```bash
# 1. 生成 API 密钥
API_KEY=$(openssl rand -hex 16)
API_SECRET=$(openssl rand -base64 32)

echo "API Key: $API_KEY"
echo "API Secret: $API_SECRET"

# 2. 更新 livekit.yaml
# 将 devkey: secret 替换为你的密钥

# 3. 启动容器
docker run -d \
  --name livekit-server \
  -p 7880:7880 \
  -p 7881:7881 \
  -p 7882:7882/udp \
  -p 50000-50100:50000-50100/udp \
  -v $(pwd)/livekit.yaml:/etc/livekit.yaml:ro \
  livekit/livekit-server:latest \
  --config /etc/livekit.yaml

# 4. 查看日志
docker logs -f livekit-server
```

---

## 配置后端

### 1. 更新 backend/.env

```bash
cd ../../backend
nano .env
```

添加或更新：
```bash
LIVEKIT_URL=ws://localhost:7880
LIVEKIT_API_KEY=你的API_KEY
LIVEKIT_API_SECRET=你的API_SECRET
```

### 2. 重启后端服务

```bash
# 如果后端正在运行，先停止
# 然后重新启动
npm run dev
```

---

## 验证部署

### 1. 检查容器状态

```bash
docker ps | grep livekit
```

应该看到类似输出：
```
CONTAINER ID   IMAGE                          STATUS         PORTS
abc123def456   livekit/livekit-server:latest  Up 2 minutes   0.0.0.0:7880->7880/tcp, ...
```

### 2. 测试 HTTP 连接

```bash
curl http://localhost:7880
```

### 3. 查看日志

```bash
docker logs livekit-server
```

### 4. 测试后端连接

```bash
cd ../../backend
curl http://localhost:3001/health
```

应该看到 `livekit: "connected"`

---

## 常见问题

### Q1: Docker 权限错误

**错误**: `permission denied while trying to connect to the Docker daemon socket`

**解决**:
```bash
# 方式 1: 添加用户到 docker 组
sudo usermod -aG docker $USER
newgrp docker

# 方式 2: 使用 sudo
sudo docker ps
```

### Q2: 端口被占用

**错误**: `bind: address already in use`

**解决**:
```bash
# 查找占用端口的进程
sudo lsof -i :7880

# 停止占用端口的容器
docker stop $(docker ps -q --filter "publish=7880")
```

### Q3: 容器无法启动

**检查**:
```bash
# 查看详细日志
docker logs livekit-server

# 检查配置文件
cat livekit.yaml
```

### Q4: WebRTC 连接失败

**原因**: UDP 端口未开放

**解决**:
```bash
# 检查防火墙
sudo ufw status

# 开放端口
sudo ufw allow 50000:50100/udp
```

---

## 生产环境建议

### 1. 使用域名和 HTTPS

```bash
# 获取 SSL 证书
sudo certbot certonly --standalone -d your-domain.com

# 配置 Nginx 反向代理
# 参考 nginx.conf
```

### 2. 启用 Redis

```bash
# 启动 Redis
docker run -d --name livekit-redis -p 6379:6379 redis:7-alpine

# 更新 livekit.yaml
redis:
  address: localhost:6379
```

### 3. 配置监控

```bash
# 使用 Prometheus + Grafana
# 参考 docker-compose.yml
```

### 4. 自动重启

```bash
# 添加 --restart=unless-stopped
docker update --restart=unless-stopped livekit-server
```

---

## 卸载

```bash
# 停止并删除容器
docker stop livekit-server
docker rm livekit-server

# 删除镜像（可选）
docker rmi livekit/livekit-server:latest

# 删除数据卷（可选）
docker volume prune
```

---

## 下一步

1. ✅ LiveKit 已安装
2. ⏭️ 配置后端
3. ⏭️ 测试推流
4. ⏭️ 部署到生产环境

---

## 获取帮助

- [完整文档](./README.md)
- [快速开始](./QUICKSTART.md)
- [LiveKit 官方文档](https://docs.livekit.io/)
