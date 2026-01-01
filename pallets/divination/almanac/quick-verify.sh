#!/bin/bash
# 快速验证节点侧 AppCode 配置功能

echo "========================================="
echo "  节点侧 AppCode 配置 - 快速验证"
echo "========================================="
echo ""

#颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "[1/4] 检查代码改动..."
if grep -q "almanac_appcode: Option<String>" node/src/cli.rs && \
   grep -q "std::env::var(\"ALMANAC_APPCODE\")" node/src/command.rs && \
   grep -q "almanac_appcode: Option<String>" node/src/service.rs; then
    echo -e "${GREEN}✅ 所有代码改动已完成${NC}"
else
    echo -e "${RED}❌ 代码改动不完整${NC}"
    exit 1
fi
echo ""

echo "[2/4] 检查编译..."
if cargo check -p stardust-node --message-format=short 2>&1 | grep -q "^error"; then
    echo -e "${RED}❌ 存在编译错误${NC}"
    exit 1
else
    echo -e "${GREEN}✅ 编译通过${NC}"
fi
echo ""

echo "[3/4] 检查文档..."
if [ -f "pallets/divination/almanac/NODE_APPCODE_USAGE.md" ] && \
   [ -f "pallets/divination/almanac/APPCODE_SECURITY.md" ] && \
   [ -f "pallets/divination/almanac/NODE_IMPLEMENTATION_SUMMARY.md" ]; then
    echo -e "${GREEN}✅ 文档已创建${NC}"
else
    echo -e "${RED}❌ 文档缺失${NC}"
    exit 1
fi
echo ""

echo "[4/4] 使用示例"
echo -e "${GREEN}✅ 可以通过以下方式配置 AppCode:${NC}"
echo ""
echo "   方式一 (环境变量,推荐):"
echo "   $ export ALMANAC_APPCODE=\"your_appcode\""
echo "   $ ./target/release/stardust-node --dev"
echo ""
echo "   方式二 (命令行参数):"
echo "   $ ./target/release/stardust-node --dev --almanac-appcode \"your_appcode\""
echo ""
echo "   方式三 (临时设置):"
echo "   $ ALMANAC_APPCODE=\"your_appcode\" ./target/release/stardust-node --dev"
echo ""

echo "========================================="
echo -e "${GREEN}  ✅ 所有测试通过！${NC}"
echo "========================================="
echo ""
echo "📚 详细文档:"
echo "   - pallets/divination/almanac/NODE_APPCODE_USAGE.md"
echo "   - pallets/divination/almanac/APPCODE_SECURITY.md"
echo "   - pallets/divination/almanac/NODE_IMPLEMENTATION_SUMMARY.md"
echo ""
echo "🚀 下一步:"
echo "   1. 实现 pallet-almanac 的 OCW 逻辑"
echo "   2. 测试 API 调用"
echo "   3. 实现前端集成"
echo ""
