# Cosmos Blockchain

Cosmos 是一个基于 Substrate 构建的区块链平台，集成玄学占卜、AI 宠物养成、社交通讯、去中心化交易等功能的 Web3 生态系统。

## 🌟 核心功能

### 🔮 玄学占卜
- **八字命理** - 根据出生时间分析命运
- **梅花易数** - 随机起卦预测
- **六爻占卜** - 传统易经占卜
- **奇门遁甲** - 时空预测术
- **紫微斗数** - 星象命理分析
- **塔罗牌** - 西方占卜系统

### 🐱 喵星宇宙 (Meowstar)
AI 宠物养成与 GameFi 生态系统：
- **宠物养成** - 孵化、升级、进化宠物
- **回合制战斗** - PVE/PVP 战斗系统
- **NFT 市场** - 宠物交易、拍卖
- **DAO 治理** - 社区提案与投票
- **AI 聊天** - 与宠物智能对话

### 💬 社交通讯
- 端到端加密聊天
- 实时直播功能
- 缘分匹配系统

### 💱 去中心化交易
- OTC 场外交易
- Swap 代币兑换
- Maker 做市商

## 📁 项目结构

```
cosmos/
├── node/                    # 区块链节点实现
├── runtime/                 # 运行时逻辑
├── pallets/                 # 自定义 Pallet 模块
│   ├── divination/          # 占卜模块
│   ├── meowstar/            # 喵星宇宙模块
│   ├── chat/                # 聊天和直播模块
│   ├── trading/             # 交易模块
│   ├── matchmaking/         # 匹配模块
│   ├── affiliate/           # 分销模块
│   ├── escrow/              # 托管模块
│   ├── evidence/            # 证据存储模块
│   └── arbitration/         # 仲裁模块
├── frontend/                # React Native + Expo 前端应用
│   ├── app/                 # 页面路由
│   │   ├── (tabs)/          # 底部导航页面
│   │   └── meowstar/        # 喵星宇宙页面
│   ├── services/            # 服务层
│   │   └── meowstar/        # 喵星宇宙数据服务
│   ├── components/          # 可复用组件
│   └── src/stores/          # 状态管理
├── ai-companion/            # AI 伴侣后端服务
│   ├── app/                 # FastAPI 应用
│   └── docker-compose.yml   # Docker 部署配置
├── backend/                 # 后端服务（LiveKit 直播等）
└── media-utils/             # 媒体工具库
```

## 🚀 快速开始

### 环境要求

- Rust 1.75+
- Node.js 18+
- Docker & Docker Compose
- protobuf-compiler

### 区块链节点

```bash
# 构建节点
cargo build --release

# 运行开发链
./target/release/cosmos-node --dev
```

### 前端应用

```bash
cd frontend
npm install
npm start
# 访问 http://localhost:8081
```

### AI 后端服务

```bash
cd ai-companion
cp .env.example .env
# 编辑 .env 配置 LLM 提供商

# 启动服务
docker-compose up -d
# API 地址: http://localhost:8000
```

## 🎮 喵星宇宙功能

### 宠物系统
| 功能 | 描述 |
|------|------|
| 孵化 | 消耗 10 COS 随机获得宠物 |
| 升级 | 消耗 10 COS，属性 +5/+3/+2/+2 |
| 进化 | 消耗 50 COS，全属性 +10% |

### 稀有度
- 🔘 普通 (Common)
- 🟢 稀有 (Rare) - 2x 价值
- 🟣 史诗 (Epic) - 5x 价值
- 🟡 传说 (Legendary) - 15x 价值
- 🔴 神话 (Mythic) - 50x 价值

### 元素属性
- 🔥 火焰 | 💧 水 | ☀️ 光明 | 🌙 暗影 | ⭐ 普通

## 💰 代币信息

- **代币符号**: COS
- **精度**: 12 位小数
- **SS58 格式**: 42

## 🔗 连接到链

启动节点后，可以通过以下方式连接：

- **Polkadot.js Apps**: https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944
- **RPC 端点**: ws://localhost:9944

## 🐳 Docker 部署

### 区块链节点

```bash
# 构建镜像
docker build . -t cosmos-blockchain

# 运行节点
docker run -p 9944:9944 -p 30333:30333 cosmos-blockchain --dev
```

### 完整服务栈

```bash
# AI 后端 + 向量数据库 + 缓存
cd ai-companion
docker-compose up -d
```

## 📱 前端页面

| 页面 | 路径 | 功能 |
|------|------|------|
| 首页 | `/` | 核心功能入口 |
| 聊天 | `/chat` | 即时通讯 |
| 占卜 | `/market` | 玄学占卜服务 |
| 合婚 | `/matchmaking` | 缘分匹配 |
| 我的 | `/profile` | 个人中心 |
| 喵星主页 | `/meowstar` | 喵星宇宙入口 |
| 宠物列表 | `/meowstar/pets` | 我的宠物 |
| 宠物详情 | `/meowstar/pet/[id]` | 升级/进化/出售 |
| 战斗大厅 | `/meowstar/battle` | PVE/PVP 选择 |
| 战斗场景 | `/meowstar/battle-arena` | 回合制战斗 |
| 市场 | `/meowstar/marketplace` | 购买/拍卖宠物 |
| 治理 | `/meowstar/governance` | 提案/投票 |
| AI 聊天 | `/meowstar/chat` | 与宠物对话 |

## 🛠 技术栈

### 区块链
- Substrate Framework
- Rust

### 前端
- React Native + Expo
- Expo Router
- TypeScript
- Lucide Icons
- AsyncStorage

### AI 后端
- FastAPI
- LangChain
- Ollama / OpenAI
- Qdrant (向量数据库)
- Redis (缓存)

## 📄 许可证

MIT-0
