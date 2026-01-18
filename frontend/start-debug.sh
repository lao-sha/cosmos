#!/bin/bash
# 调试启动脚本

echo "清理旧进程..."
pkill -f "expo start" || true

echo "启动 Expo (离线模式，避免网络错误)..."
cd "$(dirname "$0")"

# 设置环境变量避免代理检测和网络错误
export NODE_NO_WARNINGS=1
export GLOBAL_AGENT_NO_PROXY=1
export EXPO_NO_TELEMETRY=1

# 使用离线模式启动
npx expo start --offline --android --clear
