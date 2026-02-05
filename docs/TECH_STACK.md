# Frontend 技术栈

> 路径：`frontend/`

COSMOS 前端采用 React Native + Expo 跨平台方案，支持 iOS、Android 和 Web。

## 核心框架

| 技术 | 版本 | 说明 |
|------|------|------|
| **React Native** | 0.81.5 | 跨平台移动框架 |
| **Expo** | 54.0 | 开发工具链 + 原生模块 |
| **React** | 19.1 | UI 库 |
| **TypeScript** | 5.9 | 类型系统 |

## 路由与导航

| 技术 | 版本 | 说明 |
|------|------|------|
| **Expo Router** | 6.0 | 文件系统路由 |
| **React Navigation** | 7.x | 底层导航库 |

## 状态管理

| 技术 | 版本 | 说明 |
|------|------|------|
| **Zustand** | 5.0 | 轻量级状态管理 |
| **TanStack Query** | 5.90 | 服务端状态 + 缓存 |

## 区块链集成

| 技术 | 版本 | 说明 |
|------|------|------|
| **@polkadot/api** | 16.5 | Substrate RPC 客户端 |
| **@polkadot/keyring** | 14.0 | 密钥管理 |
| **@polkadot/util-crypto** | 14.0 | 加密工具 |
| **@noble/curves** | 1.8 | 椭圆曲线加密 |
| **@noble/hashes** | 1.7 | 哈希算法 |

## UI 组件

| 技术 | 版本 | 说明 |
|------|------|------|
| **Lucide React Native** | 0.563 | 图标库 |
| **Reanimated** | 4.1 | 高性能动画 |
| **Gesture Handler** | 2.28 | 手势处理 |

## 原生能力 (Expo 模块)

| 模块 | 功能 |
|------|------|
| `expo-secure-store` | 安全存储（密钥、Token）|
| `expo-local-authentication` | 生物识别（指纹/面容）|
| `expo-clipboard` | 剪贴板 |
| `expo-haptics` | 触觉反馈 |
| `expo-image` | 高性能图片 |
| `expo-web-browser` | 内置浏览器 |



## 特性

- **跨平台**: 一套代码支持 iOS / Android / Web
- **文件路由**: 基于目录结构自动生成路由
- **区块链钱包**: 原生集成 Polkadot 签名
- **安全存储**: 私钥使用 SecureStore 加密存储
- **生物识别**: 支持指纹/面容解锁
