# 需求文档

## 简介

Cosmos 移动端前端应用是一个基于 Substrate 区块链的综合性 Web3 应用，为用户提供占卜服务、社交聊天、直播互动、OTC 交易、婚恋匹配等功能。本应用需要与 Cosmos 区块链节点和后端服务进行交互，实现去中心化的用户体验。

## 术语表

- **Cosmos_Chain**: Cosmos 区块链网络，基于 Substrate 框架构建
- **Mobile_App**: Cosmos 移动端前端应用
- **Polkadot_API**: @polkadot/api 库，用于与 Substrate 链交互
- **LiveKit**: 实时音视频通信服务
- **IPFS**: 去中心化存储网络，用于存储媒体文件和加密数据
- **COS**: Cosmos 链原生代币
- **Pallet**: Substrate 链上的功能模块
- **Extrinsic**: 链上交易/调用
- **Runtime_API**: 链上免费查询接口

## 需求

### 需求 1：钱包与账户管理

**用户故事：** 作为用户，我希望能够创建和管理我的区块链钱包，以便安全地存储资产和进行链上操作。

#### 验收标准

1. WHEN 用户首次打开应用 THEN Mobile_App SHALL 提供创建新钱包或导入已有钱包的选项
2. WHEN 用户创建新钱包 THEN Mobile_App SHALL 生成助记词并要求用户安全备份
3. WHEN 用户导入钱包 THEN Mobile_App SHALL 支持助记词、私钥和 JSON 文件三种导入方式
4. THE Mobile_App SHALL 使用设备安全存储（Keychain/Keystore）加密保存私钥
5. WHEN 用户进行敏感操作 THEN Mobile_App SHALL 要求生物识别或 PIN 码验证
6. THE Mobile_App SHALL 显示账户余额、交易历史和资产列表
7. WHEN 用户切换网络 THEN Mobile_App SHALL 支持主网和测试网切换

### 需求 2：占卜服务模块

**用户故事：** 作为用户，我希望能够使用各种占卜服务（八字、梅花易数、六爻、塔罗等），以便获取命理分析和人生指导。

#### 验收标准

1. THE Mobile_App SHALL 支持以下占卜类型：八字排盘、梅花易数、六爻、奇门遁甲、紫微斗数、塔罗牌、小六壬、大六壬
2. WHEN 用户选择八字排盘 THEN Mobile_App SHALL 支持公历、农历和四柱直接输入三种方式
3. WHEN 用户输入出生信息 THEN Mobile_App SHALL 支持真太阳时修正（基于经度）
4. WHEN 用户创建占卜 THEN Mobile_App SHALL 调用对应 Pallet 的 extrinsic 并支付 Gas 费用
5. THE Mobile_App SHALL 通过 Runtime_API 免费查询占卜结果和解盘信息
6. WHEN 用户查看占卜结果 THEN Mobile_App SHALL 展示四柱、大运、五行强度、神煞、空亡等完整信息
7. THE Mobile_App SHALL 支持占卜结果的加密存储（隐私模式）
8. WHEN 用户选择隐私模式 THEN Mobile_App SHALL 在本地加密敏感数据后上传

### 需求 3：占卜服务市场

**用户故事：** 作为用户，我希望能够购买专业占卜师的解读服务，以便获得更深入的命理分析。

#### 验收标准

1. THE Mobile_App SHALL 展示服务提供者列表，包含等级、评分、擅长领域和服务套餐
2. WHEN 用户浏览服务提供者 THEN Mobile_App SHALL 支持按占卜类型、评分、价格筛选
3. WHEN 用户下单 THEN Mobile_App SHALL 锁定订单金额到托管账户
4. THE Mobile_App SHALL 展示订单状态流转：待支付→已支付→已接单→已完成→已评价
5. WHEN 订单完成 THEN Mobile_App SHALL 支持多维度评价（总体、准确度、态度、响应速度）
6. THE Mobile_App SHALL 支持订单追问功能（根据套餐次数限制）
7. IF 订单超时未接单 THEN Mobile_App SHALL 自动退款给用户

### 需求 4：聊天与社交模块

**用户故事：** 作为用户，我希望能够与其他用户进行即时通讯，以便社交互动和咨询服务。

#### 验收标准

1. THE Mobile_App SHALL 为每个用户分配唯一的 11 位数字聊天 ID
2. WHEN 用户发送消息 THEN Mobile_App SHALL 支持文本、图片、语音、视频、文件等消息类型
3. THE Mobile_App SHALL 支持四种群聊加密模式：军事级（量子抗性）、商业级（AES-256）、选择性加密、透明公开
4. WHEN 用户创建群组 THEN Mobile_App SHALL 支持设置加密模式和成员权限
5. THE Mobile_App SHALL 实现消息状态追踪：已发送、已送达、已读、已撤回
6. WHEN 用户被拉黑 THEN Mobile_App SHALL 阻止该用户发送消息
7. THE Mobile_App SHALL 支持场景化权限控制（私聊、群聊、AI 对话等）

### 需求 5：直播功能模块

**用户故事：** 作为用户，我希望能够观看和参与直播，以便获取实时占卜服务和娱乐内容。

#### 验收标准

1. THE Mobile_App SHALL 支持四种直播间类型：普通、付费、私密、连麦
2. WHEN 用户进入付费直播间 THEN Mobile_App SHALL 验证门票持有状态
3. WHEN 用户观看直播 THEN Mobile_App SHALL 通过后端服务获取 LiveKit Token
4. THE Mobile_App SHALL 支持礼物打赏功能，礼物金额实时上链
5. WHEN 主播开启连麦 THEN Mobile_App SHALL 支持最多 4 人同时连麦
6. THE Mobile_App SHALL 展示直播间列表，支持按状态、类型、观众数筛选
7. IF 用户被踢出或拉黑 THEN Mobile_App SHALL 阻止其进入该直播间

### 需求 6：OTC 交易模块

**用户故事：** 作为用户，我希望能够通过 OTC 方式购买或出售 COS 代币，以便进行法币与加密货币的兑换。

#### 验收标准

1. THE Mobile_App SHALL 展示活跃做市商列表，包含溢价率、最小金额、支付方式
2. WHEN 用户创建首购订单 THEN Mobile_App SHALL 固定 10 USD 价值，动态计算 COS 数量
3. WHEN 用户创建普通订单 THEN Mobile_App SHALL 限制金额在 20-200 USD 范围内
4. THE Mobile_App SHALL 生成支付承诺哈希和联系方式承诺哈希（本地加密）
5. WHEN 买家标记已付款 THEN Mobile_App SHALL 通知做市商确认并释放 COS
6. IF 订单发生争议 THEN Mobile_App SHALL 支持发起仲裁流程
7. THE Mobile_App SHALL 展示订单状态：已创建→已付款→已释放/已取消/争议中

### 需求 7：做市商兑换（Swap）模块

**用户故事：** 作为用户，我希望能够将 COS 兑换为 USDT，以便实现资产变现。

#### 验收标准

1. WHEN 用户发起兑换 THEN Mobile_App SHALL 锁定 COS 到托管账户
2. THE Mobile_App SHALL 展示做市商的 TRON 地址用于接收 USDT
3. WHEN 做市商提交 TRC20 交易哈希 THEN Mobile_App SHALL 等待 OCW 自动验证
4. IF 兑换超时 THEN Mobile_App SHALL 自动退款给用户
5. WHEN 用户认为做市商未发送 USDT THEN Mobile_App SHALL 支持举报功能
6. THE Mobile_App SHALL 展示兑换状态：待处理→等待验证→已完成/已退款

### 需求 8：仲裁与争议处理模块

**用户故事：** 作为用户，我希望在交易发生争议时能够提交证据并获得公正裁决，以便保护我的权益。

#### 验收标准

1. WHEN 用户发起仲裁 THEN Mobile_App SHALL 要求锁定 15% 订单金额作为押金
2. THE Mobile_App SHALL 支持上传证据到 IPFS 并提交证据 ID 到链上
3. WHEN 应诉方响应 THEN Mobile_App SHALL 同样锁定 15% 押金并提交反驳证据
4. THE Mobile_App SHALL 展示仲裁状态和双方证据列表
5. WHEN 仲裁裁决完成 THEN Mobile_App SHALL 展示裁决结果和押金处理情况
6. THE Mobile_App SHALL 支持投诉功能，覆盖 12 个业务域的 56 种投诉类型

### 需求 9：婚恋匹配模块

**用户故事：** 作为用户，我希望能够基于八字命理进行婚恋匹配，以便找到合适的伴侣。

#### 验收标准

1. THE Mobile_App SHALL 支持创建婚恋资料，包含昵称、性别、出生日期、位置、简介
2. WHEN 用户设置择偶条件 THEN Mobile_App SHALL 支持年龄范围、位置偏好、教育水平等筛选
3. THE Mobile_App SHALL 支持绑定八字命盘用于合婚分析
4. WHEN 用户发起合婚请求 THEN Mobile_App SHALL 要求双方授权后生成报告
5. THE Mobile_App SHALL 展示匹配评分和建议（天作之合、良缘佳配、中等缘分等）
6. THE Mobile_App SHALL 支持互动功能：点赞、超级喜欢、跳过、屏蔽
7. WHEN 双方互相喜欢 THEN Mobile_App SHALL 自动匹配并通知双方

### 需求 10：会员与签到系统

**用户故事：** 作为用户，我希望能够订阅会员服务并通过签到获取奖励，以便享受更多权益。

#### 验收标准

1. THE Mobile_App SHALL 展示 6 级会员体系：Free、Bronze、Silver、Gold、Platinum、Diamond
2. WHEN 用户订阅会员 THEN Mobile_App SHALL 支持月付和年付（年付享 16.7% 折扣）
3. THE Mobile_App SHALL 支持每日签到领取 COS 奖励
4. WHEN 用户连续签到 7 天及以上 THEN Mobile_App SHALL 给予 1.5 倍奖励
5. THE Mobile_App SHALL 展示会员权益：存储折扣、免费 AI 解读次数等
6. IF 用户为新账户（7 天内） THEN Mobile_App SHALL 限制领取奖励功能

### 需求 11：用户档案与隐私管理

**用户故事：** 作为用户，我希望能够管理我的个人档案和隐私设置，以便控制信息的可见性。

#### 验收标准

1. THE Mobile_App SHALL 支持用户档案分层存储：公开层、占卜层、加密层
2. WHEN 用户更新档案 THEN Mobile_App SHALL 支持设置昵称、性别、出生信息、经纬度
3. THE Mobile_App SHALL 支持敏感数据（姓名、地址）的端到端加密
4. WHEN 用户清除敏感数据 THEN Mobile_App SHALL 从链上删除加密数据
5. THE Mobile_App SHALL 支持隐私模式设置：公开、仅匹配可见、完全私密

### 需求 12：链上交互与签名

**用户故事：** 作为用户，我希望应用能够安全地与区块链交互，以便执行各种链上操作。

#### 验收标准

1. THE Mobile_App SHALL 使用 @polkadot/api 库与 Cosmos_Chain 交互
2. WHEN 用户发起交易 THEN Mobile_App SHALL 显示交易详情并要求确认签名
3. THE Mobile_App SHALL 支持交易状态追踪：已提交→已入块→已确认
4. WHEN 交易失败 THEN Mobile_App SHALL 显示错误信息和失败原因
5. THE Mobile_App SHALL 支持批量交易（utility.batch）以优化 Gas 费用
6. THE Mobile_App SHALL 监听链上事件并实时更新 UI 状态

### 需求 13：后端服务集成

**用户故事：** 作为用户，我希望应用能够与后端服务无缝集成，以便获取实时数据和媒体服务。

#### 验收标准

1. THE Mobile_App SHALL 通过 REST API 与后端服务通信
2. WHEN 用户进入直播间 THEN Mobile_App SHALL 调用后端获取 LiveKit Token
3. THE Mobile_App SHALL 通过 WebSocket 接收实时事件推送
4. WHEN 后端服务不可用 THEN Mobile_App SHALL 显示友好的错误提示并支持重试
5. THE Mobile_App SHALL 缓存常用数据以减少网络请求

### 需求 14：离线支持与数据同步

**用户故事：** 作为用户，我希望在网络不稳定时仍能使用部分功能，以便获得流畅的使用体验。

#### 验收标准

1. THE Mobile_App SHALL 本地缓存用户的占卜结果和聊天记录
2. WHEN 网络恢复 THEN Mobile_App SHALL 自动同步离线期间的操作
3. THE Mobile_App SHALL 在离线状态下支持查看已缓存的数据
4. WHEN 用户在离线状态发起交易 THEN Mobile_App SHALL 提示需要网络连接

### 需求 15：多语言与本地化

**用户故事：** 作为用户，我希望应用支持多种语言，以便使用我熟悉的语言操作。

#### 验收标准

1. THE Mobile_App SHALL 默认支持中文（简体）和英文
2. WHEN 用户切换语言 THEN Mobile_App SHALL 立即更新所有界面文本
3. THE Mobile_App SHALL 根据系统语言自动选择默认语言
4. THE Mobile_App SHALL 支持占卜术语的专业翻译（如天干地支、神煞名称）
