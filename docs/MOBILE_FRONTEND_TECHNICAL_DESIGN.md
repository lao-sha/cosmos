# Cosmos 移动端前端技术设计文档

## 1. 引言
本文档概述了 Cosmos 移动应用程序的技术设计。Cosmos 是一个去中心化平台，融合了传统文化（占卜）、社交互动和 Web3 交易。该应用程序旨在实现跨平台支持、安全性以及高度的响应性能。

## 2. 技术栈 (Tech Stack)

### 2.1 核心框架与语言
- **框架**: [Expo](https://expo.dev/) (React Native) - 为跨平台开发提供强大的生态系统，支持访问原生 API。
- **导航**: [Expo Router](https://docs.expo.dev/router/introduction/) - 基于文件的 React Native 路由，确保深层链接支持和一致的导航模式。
- **语言**: [TypeScript](https://www.typescriptlang.org/) - 严格类型检查，提高可维护性和开发体验。

### 2.2 状态管理与数据获取
- **客户端状态**: [Zustand](https://github.com/pmndrs/zustand) - 轻量级、基于 Hook 的状态管理，用于本地 UI 状态和用户偏好。
- **服务器/链端状态**: [TanStack Query (React Query)](https://tanstack.com/query/latest) - 处理区块链数据的缓存、同步和后台更新。
- **持久化**: `zustand/middleware/persist` 结合 `idb-keyval` (Web) 和 `Expo SecureStore` (Native)。

### 2.3 区块链交互
- **核心库**: `@polkadot/api` - 与基于 Substrate 的 Cosmos 链交互的主要接口。
- **钱包/密钥管理**:
  - **Web**: 使用 `@polkadot/extension-dapp` 处理浏览器扩展。
  - **Native**: 使用 `polkadot-js/util-crypto` 和通过 `Expo SecureStore` 实现的硬件支持存储。
- **IPFS**: 使用 `ipfs-http-client` 或类似工具进行去中心化媒体存储。

### 2.4 UI 与样式
- **样式**: [NativeWind](https://www.nativewind.dev/) (基于 Tailwind CSS 的 React Native 实现) - 原子级样式，支持快速 UI 开发和跨平台一致性。
- **组件**: 遵循原子设计原则的自定义组件库，使用 `lucide-react-native` 作为图标库。
- **动画**: `React Native Reanimated` 用于实现流利、高性能的交互效果。

### 2.5 基础设施与工具
- **测试**: `Jest` + `React Native Testing Library`。
- **代码规范/格式化**: `ESLint` + `Prettier`。
- **CI/CD**: `GitHub Actions` + `Expo Application Services (EAS)` 用于构建和部署。

---

## 3. 前端架构 (Architecture)

### 3.1 模块化结构
项目遵循基于功能的模块化架构，以确保可扩展性和关注点分离。

```text
/
├── app/                  # 基于文件的路由 (Expo Router)
│   ├── (tabs)/           # 主导航选项卡 (首页, 聊天, 市场, 个人中心)
│   ├── divination/       # 占卜相关路由
│   ├── trading/          # 交易与兑换路由
│   └── matchmaking/      # 社交与婚恋路由
├── src/
│   ├── api/              # Polkadot API 提供者与基础配置
│   ├── components/       # 共享 UI 组件 (原子, 分子, 核心组件)
│   ├── features/         # 业务功能逻辑封装
│   │   ├── auth/         # 钱包连接与身份认证
│   │   ├── chat/         # 消息与社交逻辑
│   │   ├── divination/   # 计算与解盘逻辑
│   │   └── trading/      # OTC 与兑换业务逻辑
│   ├── hooks/            # 全局可复用 Hooks (useChain, useTheme 等)
│   ├── services/         # 特定领域链服务 (Query/Tx 封装)
│   ├── stores/           # Zustand Store 定义
│   ├── types/            # 全局 TypeScript 接口与枚举
│   └── lib/              # 工具库 (加密, IPFS, 格式化)
```

### 3.2 分层架构
1. **视图层 (View)**: 专注于展示的 React 组件。
2. **逻辑层 (Hooks)**: `features/` 中的自定义 Hooks，封装业务逻辑和副作用。
3. **服务层 (API/Chain)**: `services/` 中的标准化方法，用于与区块链和 IPFS 交互，抽象 Substrate extrinsics 的复杂性。
4. **数据层 (Stores)**: 应用程序状态和缓存的链端数据。

### 3.3 安全设计
- **敏感数据**: 私钥和助记词绝不以明文存储。原生版本使用 Keychain/Keystore；Web 版本使用带 AES-GCM 加密的 IndexedDB。
- **交易流程**: 标准化的“签名并发送”流程，每个交易都有明确的用户确认和 Gas 估算。
- **隐私**: 在将敏感占卜数据发送到链端或 IPFS 之前，进行客户端加密。

---

## 4. 设计模式
- **原子设计 (Atomic Design)**: 将 UI 拆分为原子、分子和核心组件，以实现最大程度的复用。
- **HOC/Providers**: 使用 React Context 提供全局 Provider (主题, 链, 钱包)。
- **错误边界 (Error Boundaries)**: 针对区块链连接失败或交易被拒绝进行细粒度的错误处理。

---

## 5. 核心优化与增强

### 5.1 离线策略与同步机制 (Offline & Sync)
为了提升在去中心化环境下的用户体验，系统需支持：
- **乐观 UI (Optimistic UI)**: 在发起链上请求或消息发送时，UI 立即更新为“已完成”状态，并在后台异步处理。
- **离线任务队列**: 使用 Zustand 持久化插件管理未完成的链上交易，在网络恢复后自动重试或提示用户。
- **状态回滚**: 当异步交易最终失败时，提供清晰的通知并自动回滚本地 UI 状态。

### 5.2 本地加密存储扩展
- **SQLite 加密数据库**: 针对聊天记录、占卜历史等大数据量场景，使用 `expo-sqlite` 配合加密层，确保存储在本地的业务数据安全。
- **端到端加密**: 存储的数据应使用用户私钥派生的本地密钥进行 AES 加密，防止设备丢失导致的数据外泄。

### 5.3 环境管理与 EAS 配置
- **多环境支持**: 在 `eas.json` 中配置 `development`, `preview`, `production` 配置文件，分别对应开发网、测试网和主网的 RPC 节点及合约地址。
- **RPC 故障转移**: 服务层实现多节点轮询逻辑，当主 RPC 节点不可用时，自动切换到备份节点。

### 5.4 主题与动效规范 (Design Tokens & Animation)
- **Design Tokens**: 使用 NativeWind 定义符合“东方玄学”风格的全局变量（如：`colors.zen-purple`, `colors.gold-leaf`）。
- **动效库**: 
  - **Lottie**: 用于占卜排盘、抽签等复杂的矢量动画。
  - **Reanimated 3**: 用于处理手势交互、卡片滑动等高性能物理动效。

