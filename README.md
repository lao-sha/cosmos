# Cosmos Blockchain

Cosmos 是一个基于 Substrate 构建的区块链平台，提供占卜、社交、交易等功能。

## 项目结构

- `node/` - 区块链节点实现
- `runtime/` - 运行时逻辑
- `pallets/` - 自定义 Pallet 模块
  - `divination/` - 占卜相关模块（八字、梅花、六爻、奇门、紫微、塔罗等）
  - `chat/` - 聊天和直播模块
  - `trading/` - 交易模块（OTC、Swap、Maker）
  - `matchmaking/` - 匹配模块
  - `affiliate/` - 分销模块
  - `escrow/` - 托管模块
  - `evidence/` - 证据存储模块
  - `arbitration/` - 仲裁模块
- `frontend/` - React Native 移动端应用
- `backend/` - 后端服务（LiveKit 直播等）
- `media-utils/` - 媒体工具库

## 快速开始

### 环境要求

- Rust 1.75+
- Node.js 18+
- protobuf-compiler

### 构建

```bash
# 构建节点
cargo build --release

# 运行开发链
./target/release/cosmos-node --dev
```

### 前端开发

```bash
cd frontend
npm install
npm start
```

## 代币信息

- 代币符号: COS
- 精度: 12 位小数
- SS58 格式: 42

## 连接到链

启动节点后，可以通过以下方式连接：

- Polkadot.js Apps: https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944
- RPC 端点: ws://localhost:9944

## Docker

```bash
# 构建镜像
docker build . -t cosmos-blockchain

# 运行节点
docker run -p 9944:9944 -p 30333:30333 cosmos-blockchain --dev
```

## 许可证

MIT-0
