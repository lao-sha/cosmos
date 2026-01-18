#!/bin/bash

# LiveKit 快速启动脚本

set -e

echo "========================================="
echo "LiveKit 快速部署脚本"
echo "========================================="
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查 Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}错误: Docker 未安装${NC}"
    echo "请先安装 Docker: https://docs.docker.com/get-docker/"
    exit 1
fi

# 检查 Docker Compose
if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}错误: Docker Compose 未安装${NC}"
    echo "请先安装 Docker Compose: https://docs.docker.com/compose/install/"
    exit 1
fi

echo -e "${GREEN}✓ Docker 和 Docker Compose 已安装${NC}"
echo ""

# 生成 API 密钥
echo "========================================="
echo "生成 API 密钥"
echo "========================================="
echo ""

API_KEY=$(openssl rand -hex 16)
API_SECRET=$(openssl rand -base64 32)

echo -e "${GREEN}API Key:${NC} $API_KEY"
echo -e "${GREEN}API Secret:${NC} $API_SECRET"
echo ""

# 询问是否更新配置文件
read -p "是否自动更新 livekit.yaml 配置文件? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    # 备份原配置
    if [ -f "livekit.yaml" ]; then
        cp livekit.yaml livekit.yaml.backup
        echo -e "${YELLOW}已备份原配置到 livekit.yaml.backup${NC}"
    fi
    
    # 更新配置
    sed -i.bak "s/devkey: secret/$API_KEY: $API_SECRET/" livekit.yaml
    echo -e "${GREEN}✓ 已更新 livekit.yaml${NC}"
fi
echo ""

# 询问是否启动服务
echo "========================================="
echo "启动服务"
echo "========================================="
echo ""

read -p "是否立即启动 LiveKit 服务? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "正在启动服务..."
    docker-compose up -d
    
    echo ""
    echo -e "${GREEN}✓ 服务启动成功！${NC}"
    echo ""
    
    # 等待服务就绪
    echo "等待服务就绪..."
    sleep 5
    
    # 检查服务状态
    echo ""
    echo "========================================="
    echo "服务状态"
    echo "========================================="
    docker-compose ps
    
    echo ""
    echo "========================================="
    echo "测试连接"
    echo "========================================="
    
    if curl -s http://localhost:7880 > /dev/null; then
        echo -e "${GREEN}✓ LiveKit 服务运行正常${NC}"
    else
        echo -e "${RED}✗ LiveKit 服务连接失败${NC}"
        echo "请检查日志: docker-compose logs livekit"
    fi
fi

echo ""
echo "========================================="
echo "部署完成"
echo "========================================="
echo ""
echo "LiveKit 服务信息:"
echo "  - HTTP/WebSocket: http://localhost:7880"
echo "  - TURN TCP: localhost:7881"
echo "  - TURN UDP: localhost:7882"
echo "  - WebRTC 端口: 50000-50100 (UDP)"
echo ""
echo "API 密钥:"
echo "  - API Key: $API_KEY"
echo "  - API Secret: $API_SECRET"
echo ""
echo "后端配置 (backend/.env):"
echo "  LIVEKIT_URL=ws://localhost:7880"
echo "  LIVEKIT_API_KEY=$API_KEY"
echo "  LIVEKIT_API_SECRET=$API_SECRET"
echo ""
echo "常用命令:"
echo "  - 查看日志: docker-compose logs -f livekit"
echo "  - 停止服务: docker-compose down"
echo "  - 重启服务: docker-compose restart"
echo "  - 查看状态: docker-compose ps"
echo ""
echo "下一步:"
echo "  1. 更新 backend/.env 配置"
echo "  2. 重启后端服务: cd backend && npm run dev"
echo "  3. 测试推流和观看功能"
echo ""
echo -e "${GREEN}部署完成！${NC}"
