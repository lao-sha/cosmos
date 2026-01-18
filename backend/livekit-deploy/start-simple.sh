#!/bin/bash

# LiveKit 简单启动脚本（不依赖 docker-compose）

set -e

echo "========================================="
echo "LiveKit 简单部署"
echo "========================================="
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 检查 Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}错误: Docker 未安装${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Docker 已安装${NC}"
echo ""

# 生成 API 密钥
echo "生成 API 密钥..."
API_KEY=$(openssl rand -hex 16)
API_SECRET=$(openssl rand -base64 32)

echo -e "${GREEN}API Key:${NC} $API_KEY"
echo -e "${GREEN}API Secret:${NC} $API_SECRET"
echo ""

# 停止并删除旧容器
echo "清理旧容器..."
docker stop livekit-server 2>/dev/null || true
docker rm livekit-server 2>/dev/null || true
echo ""

# 启动 LiveKit
echo "启动 LiveKit 服务器..."
docker run -d \
  --name livekit-server \
  -p 7880:7880 \
  -p 7881:7881 \
  -p 7882:7882/udp \
  -p 50000-50100:50000-50100/udp \
  -v $(pwd)/livekit.yaml:/etc/livekit.yaml:ro \
  livekit/livekit-server:latest \
  --config /etc/livekit.yaml

echo ""
echo "等待服务启动..."
sleep 5

# 检查服务状态
if docker ps | grep -q livekit-server; then
    echo -e "${GREEN}✓ LiveKit 服务启动成功！${NC}"
else
    echo -e "${RED}✗ LiveKit 服务启动失败${NC}"
    echo "查看日志: docker logs livekit-server"
    exit 1
fi

# 测试连接
echo ""
echo "测试连接..."
if curl -s http://localhost:7880 > /dev/null; then
    echo -e "${GREEN}✓ HTTP 连接正常${NC}"
else
    echo -e "${YELLOW}⚠ HTTP 连接失败，请检查日志${NC}"
fi

echo ""
echo "========================================="
echo "部署完成"
echo "========================================="
echo ""
echo "LiveKit 服务信息:"
echo "  - URL: ws://localhost:7880"
echo "  - API Key: $API_KEY"
echo "  - API Secret: $API_SECRET"
echo ""
echo "更新后端配置 (backend/.env):"
echo "  LIVEKIT_URL=ws://localhost:7880"
echo "  LIVEKIT_API_KEY=$API_KEY"
echo "  LIVEKIT_API_SECRET=$API_SECRET"
echo ""
echo "常用命令:"
echo "  - 查看日志: docker logs -f livekit-server"
echo "  - 停止服务: docker stop livekit-server"
echo "  - 启动服务: docker start livekit-server"
echo "  - 删除容器: docker rm -f livekit-server"
echo ""
echo -e "${GREEN}部署完成！${NC}"
