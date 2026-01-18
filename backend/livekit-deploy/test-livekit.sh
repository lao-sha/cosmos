#!/bin/bash

# LiveKit 测试脚本

echo "========================================="
echo "LiveKit 服务测试"
echo "========================================="
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 测试 1: 检查服务是否运行
echo "1. 检查 Docker 容器状态..."
if docker ps | grep -q livekit-server; then
    echo -e "${GREEN}✓ LiveKit 容器运行中${NC}"
else
    echo -e "${RED}✗ LiveKit 容器未运行${NC}"
    echo "请先启动服务: docker-compose up -d"
    exit 1
fi
echo ""

# 测试 2: 检查端口
echo "2. 检查端口监听..."
ports=(7880 7881 7882)
for port in "${ports[@]}"; do
    if netstat -tuln 2>/dev/null | grep -q ":$port " || ss -tuln 2>/dev/null | grep -q ":$port "; then
        echo -e "${GREEN}✓ 端口 $port 正在监听${NC}"
    else
        echo -e "${YELLOW}⚠ 端口 $port 未监听${NC}"
    fi
done
echo ""

# 测试 3: HTTP 连接
echo "3. 测试 HTTP 连接..."
if curl -s -o /dev/null -w "%{http_code}" http://localhost:7880 | grep -q "200\|404"; then
    echo -e "${GREEN}✓ HTTP 连接成功${NC}"
else
    echo -e "${RED}✗ HTTP 连接失败${NC}"
fi
echo ""

# 测试 4: 查看日志
echo "4. 最近的日志 (最后 10 行)..."
docker-compose logs --tail=10 livekit
echo ""

# 测试 5: 容器资源使用
echo "5. 容器资源使用..."
docker stats --no-stream livekit-server
echo ""

# 测试 6: 健康检查
echo "6. 健康检查..."
if docker inspect livekit-server | grep -q '"Status": "healthy"'; then
    echo -e "${GREEN}✓ 容器健康状态正常${NC}"
else
    echo -e "${YELLOW}⚠ 容器健康状态未知或不健康${NC}"
fi
echo ""

echo "========================================="
echo "测试完成"
echo "========================================="
echo ""
echo "如果所有测试通过，LiveKit 服务已正常运行。"
echo ""
echo "下一步:"
echo "  1. 更新后端配置 (backend/.env)"
echo "  2. 重启后端服务"
echo "  3. 使用前端应用测试推流"
echo ""
