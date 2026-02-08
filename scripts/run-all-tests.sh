#!/bin/bash

# Nexus 链上功能测试脚本
# 使用方法: ./run-all-tests.sh

set -e

echo "=============================================="
echo "  Nexus 链上功能测试"
echo "=============================================="

cd "$(dirname "$0")"

# 检查依赖
if [ ! -d "node_modules" ]; then
  echo "📦 安装依赖..."
  npm install
fi

# 运行测试
echo ""
echo "🚀 开始测试..."
echo ""

npx tsx run-all-tests.ts
