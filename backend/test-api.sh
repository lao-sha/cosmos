#!/bin/bash

# Livestream Backend API 测试脚本

BASE_URL="http://localhost:3001"

echo "========================================="
echo "Livestream Backend API 测试"
echo "========================================="
echo ""

# 1. 健康检查
echo "1. 测试健康检查..."
curl -s "${BASE_URL}/health" | python3 -m json.tool
echo ""
echo ""

# 2. 获取直播间列表
echo "2. 测试获取直播间列表..."
curl -s "${BASE_URL}/api/livestream/rooms?status=Live&page=1&limit=10" | python3 -m json.tool
echo ""
echo ""

# 3. 获取单个直播间信息
echo "3. 测试获取直播间信息 (roomId=1)..."
curl -s "${BASE_URL}/api/livestream/room/1" | python3 -m json.tool
echo ""
echo ""

# 4. 测试获取主播 Token (需要真实签名)
echo "4. 测试获取主播 Token (会失败，因为需要真实签名)..."
curl -s -X POST "${BASE_URL}/api/livestream/publisher-token" \
  -H "Content-Type: application/json" \
  -d '{
    "roomId": 1,
    "publicKey": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    "signature": "0x1234567890abcdef",
    "timestamp": '$(date +%s000)'
  }' | python3 -m json.tool
echo ""
echo ""

echo "========================================="
echo "测试完成"
echo "========================================="
echo ""
echo "提示："
echo "- 健康检查应该返回 status: 'degraded' (因为没有 Redis 和 LiveKit)"
echo "- 直播间列表可能为空（如果链上没有直播间）"
echo "- Token 获取会失败（需要真实的签名）"
echo ""
echo "下一步："
echo "1. 配置 LiveKit 服务器"
echo "2. 启动 Redis (可选)"
echo "3. 在链上创建直播间"
echo "4. 使用前端应用测试完整流程"
