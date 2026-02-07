#!/bin/bash
# ══════════════════════════════════════════════════════════
# Costik 端到端集成测试脚本
#
# 前置条件:
#   - docker-compose up -d 已运行
#   - 所有服务健康
#
# 用法: ./test-e2e.sh
# ══════════════════════════════════════════════════════════

set -e

AGENT_URL="http://localhost:8443"
NODE1_URL="http://localhost:8081"
NODE2_URL="http://localhost:8082"
NODE3_URL="http://localhost:8083"
WEBHOOK_SECRET="dev_secret_12345"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass=0
fail=0

check() {
    local name="$1"
    local result="$2"
    if [ "$result" = "0" ]; then
        echo -e "  ${GREEN}✓${NC} $name"
        ((pass++))
    else
        echo -e "  ${RED}✗${NC} $name"
        ((fail++))
    fi
}

echo "══════════════════════════════════════════"
echo "  Costik E2E Integration Tests"
echo "══════════════════════════════════════════"
echo ""

# ════════════════════════════════════════════
# 1. Health Checks
# ════════════════════════════════════════════
echo -e "${YELLOW}[1] Health Checks${NC}"

# Agent health
agent_health=$(curl -sf "$AGENT_URL/health" 2>/dev/null | jq -r '.status' 2>/dev/null || echo "fail")
check "Agent health" "$([ "$agent_health" = "ok" ] && echo 0 || echo 1)"

# Node 1 health
node1_health=$(curl -sf "$NODE1_URL/health" 2>/dev/null | jq -r '.status' 2>/dev/null || echo "fail")
check "Node 1 health" "$([ "$node1_health" = "ok" ] && echo 0 || echo 1)"

# Node 2 health
node2_health=$(curl -sf "$NODE2_URL/health" 2>/dev/null | jq -r '.status' 2>/dev/null || echo "fail")
check "Node 2 health" "$([ "$node2_health" = "ok" ] && echo 0 || echo 1)"

# Node 3 health
node3_health=$(curl -sf "$NODE3_URL/health" 2>/dev/null | jq -r '.status' 2>/dev/null || echo "fail")
check "Node 3 health" "$([ "$node3_health" = "ok" ] && echo 0 || echo 1)"

echo ""

# ════════════════════════════════════════════
# 2. Webhook → Agent → Nodes 流程
# ════════════════════════════════════════════
echo -e "${YELLOW}[2] Webhook → Agent → Nodes Pipeline${NC}"

# 模拟 Telegram Webhook 推送
webhook_status=$(curl -sf -o /dev/null -w "%{http_code}" \
    -X POST "$AGENT_URL/webhook" \
    -H "Content-Type: application/json" \
    -H "X-Telegram-Bot-Api-Secret-Token: $WEBHOOK_SECRET" \
    -d '{
        "update_id": 100001,
        "message": {
            "message_id": 1,
            "from": {"id": 12345, "is_bot": false, "first_name": "Test"},
            "chat": {"id": -100999, "type": "supergroup", "title": "Test Group"},
            "date": 1700000000,
            "text": "Hello from E2E test"
        }
    }' 2>/dev/null || echo "000")
check "Webhook accepted (200)" "$([ "$webhook_status" = "200" ] && echo 0 || echo 1)"

# 等待多播完成
sleep 2

# 检查 Agent 序列号递增
agent_seq=$(curl -sf "$AGENT_URL/health" 2>/dev/null | jq -r '.sequence' 2>/dev/null || echo "0")
check "Agent sequence incremented (>0)" "$([ "$agent_seq" -gt 0 ] 2>/dev/null && echo 0 || echo 1)"

echo ""

# ════════════════════════════════════════════
# 3. Webhook Secret 验证
# ════════════════════════════════════════════
echo -e "${YELLOW}[3] Security: Webhook Secret Validation${NC}"

# 无 secret → 401
no_secret_status=$(curl -sf -o /dev/null -w "%{http_code}" \
    -X POST "$AGENT_URL/webhook" \
    -H "Content-Type: application/json" \
    -d '{"update_id": 999}' 2>/dev/null || echo "000")
check "No secret → 401" "$([ "$no_secret_status" = "401" ] && echo 0 || echo 1)"

# 错误 secret → 401
bad_secret_status=$(curl -sf -o /dev/null -w "%{http_code}" \
    -X POST "$AGENT_URL/webhook" \
    -H "Content-Type: application/json" \
    -H "X-Telegram-Bot-Api-Secret-Token: wrong_secret" \
    -d '{"update_id": 999}' 2>/dev/null || echo "000")
check "Wrong secret → 401" "$([ "$bad_secret_status" = "401" ] && echo 0 || echo 1)"

echo ""

# ════════════════════════════════════════════
# 4. 命令消息测试
# ════════════════════════════════════════════
echo -e "${YELLOW}[4] Command Messages${NC}"

# /ban 命令
ban_status=$(curl -sf -o /dev/null -w "%{http_code}" \
    -X POST "$AGENT_URL/webhook" \
    -H "Content-Type: application/json" \
    -H "X-Telegram-Bot-Api-Secret-Token: $WEBHOOK_SECRET" \
    -d '{
        "update_id": 100002,
        "message": {
            "message_id": 2,
            "from": {"id": 11111, "is_bot": false, "first_name": "Admin"},
            "chat": {"id": -100999, "type": "supergroup"},
            "text": "/ban",
            "reply_to_message": {
                "message_id": 1,
                "from": {"id": 99999, "is_bot": false, "first_name": "Spammer"}
            }
        }
    }' 2>/dev/null || echo "000")
check "/ban command accepted" "$([ "$ban_status" = "200" ] && echo 0 || echo 1)"

# 入群申请
join_status=$(curl -sf -o /dev/null -w "%{http_code}" \
    -X POST "$AGENT_URL/webhook" \
    -H "Content-Type: application/json" \
    -H "X-Telegram-Bot-Api-Secret-Token: $WEBHOOK_SECRET" \
    -d '{
        "update_id": 100003,
        "chat_join_request": {
            "chat": {"id": -100999, "type": "supergroup"},
            "from": {"id": 77777, "is_bot": false, "first_name": "NewUser"},
            "date": 1700000100
        }
    }' 2>/dev/null || echo "000")
check "Join request accepted" "$([ "$join_status" = "200" ] && echo 0 || echo 1)"

echo ""

# ════════════════════════════════════════════
# 5. Node API 测试
# ════════════════════════════════════════════
echo -e "${YELLOW}[5] Node API${NC}"

# 直接向节点发送消息（模拟 Agent 多播）
node_direct_status=$(curl -sf -o /dev/null -w "%{http_code}" \
    -X POST "$NODE1_URL/v1/message" \
    -H "Content-Type: application/json" \
    -d '{
        "owner_public_key": "aaaa",
        "bot_id_hash": "bbbb",
        "sequence": 1,
        "timestamp": 1700000000,
        "message_hash": "cccc",
        "telegram_update": {"update_id": 1},
        "owner_signature": "dddd",
        "platform": "telegram"
    }' 2>/dev/null || echo "000")
# 期望 403 (验证失败 — 假数据) 或 200
check "Node accepts/rejects message" "$([ "$node_direct_status" != "000" ] && echo 0 || echo 1)"

# 消息状态查询
msg_status=$(curl -sf -o /dev/null -w "%{http_code}" \
    "$NODE1_URL/v1/status/test_msg_1" 2>/dev/null || echo "000")
check "Message status endpoint" "$([ "$msg_status" = "200" ] && echo 0 || echo 1)"

echo ""

# ════════════════════════════════════════════
# 结果汇总
# ════════════════════════════════════════════
total=$((pass + fail))
echo "══════════════════════════════════════════"
echo -e "  Results: ${GREEN}${pass} passed${NC}, ${RED}${fail} failed${NC}, ${total} total"
echo "══════════════════════════════════════════"

if [ "$fail" -gt 0 ]; then
    exit 1
fi
