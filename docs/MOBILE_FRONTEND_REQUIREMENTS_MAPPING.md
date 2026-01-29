# Cosmos 移动端前端需求与接口映射文档 (输入文档)

## 1. 概述
本文档将 Cosmos 区块链模块 (Pallets) 和接口映射到特定的移动端前端功能。它是前端开发团队的主要需求参考文档。

## 2. 模块与功能映射

### 2.1 占卜模块 (Divination)
**链端 Pallets**: `pallet-divination-bazi`, `pallet-divination-market`, `pallet-divination-privacy`

| 功能名称 | 链端接口 (Extrinsics/Queries) | UI/UX 需求 |
| :--- | :--- | :--- |
| 八字命盘创建 | `create_bazi_chart` | 包含出生日期、时间、性别、地点的多步骤表单。 |
| 加密记录 | `create_encrypted_chart` | 上传前进行本地加密；记录上显示“锁定”图标。 |
| 解盘市场 | `register_provider`, `create_package` | 占卜师用于设置服务、定价和管理套餐的控制面板。 |
| 下单服务 | `create_order`, `submit_interpretation` | 市场列表、支持状态追踪的订单详情视图。 |
| 历史记录追踪 | `divinationRecords` (Query) | 可按类型筛选的无限滚动历史记录列表。 |

### 2.2 社交与婚恋 (Social & Matchmaking)
**链端 Pallets**: `pallet-matchmaking-profile`, `pallet-matchmaking-interaction`, `pallet-chat-core`

| 功能名称 | 链端接口 (Extrinsics/Queries) | UI/UX 需求 |
| :--- | :--- | :--- |
| 资料管理 | `create_profile`, `update_profile` | 支持图片上传（IPFS 集成）的富文本资料编辑器。 |
| 发现/匹配 | `like`, `super_like`, `pass` | 类似 Tinder 的滑动卡片，用于发现潜在匹配对象。 |
| 即时通讯 | `send_message`, `mark_batch_as_read` | 包含消息状态（已发送/已读）的实时聊天界面。 |
| 匹配报告 | `generate_report` | 可视化的“亲密度评分”和用于匹配分析的 PDF 导出功能。 |

### 2.3 交易与资产 (Trading & Finance)
**链端 Pallets**: `pallet-trading-swap`, `pallet-trading-otc`

| 功能名称 | 链端接口 (Extrinsics/Queries) | UI/UX 需求 |
| :--- | :--- | :--- |
| 代币兑换 | `create_official_swap`, `create_maker_swap` | 币种选择（主权/原生）、汇率显示及实时计算。 |
| OTC 市场 | `create_order`, `take_order` | 包含买入/卖出按钮的广告列表；托管状态指示器。 |
| 争议解决 | `dispute_order`, `submit_evidence` | 包含图片/文本证据上传到 IPFS 的争议处理表单。 |

### 2.4 治理与基础设施 (Governance & Infrastructure)
**链端 Pallets**: `pallet-arbitration`, `pallet-evidence`, `pallet-cosmos-ipfs`

| 功能名称 | 链端接口 (Extrinsics/Queries) | UI/UX 需求 |
| :--- | :--- | :--- |
| 证据库 | `submit_evidence` | 将文档安全上传到 IPFS 并在链上记录哈希。 |
| 仲裁中心 | `create_dispute`, `arbitrate` | 仲裁员审查并对争议案件进行投票的案件列表。 |
| 存储管理 | `request_pin_for_subject` | 钱包选项卡显示 IPFS 存储使用情况和配额。 |

---

## 3. 非功能性需求

### 3.1 用户体验与性能
- **连接**: 自动连接到最优的 RPC 节点。
- **反馈**: 每次“签名”操作都提供触觉反馈。
- **延迟**: 对聊天消息使用乐观 UI 更新；对链上交易显示“处理中”状态。

### 3.2 安全性
- **生物识别**: 解锁钱包或签署大额交易时需要 FaceID/指纹识别。
- **隐私模式**: 允许在公共场合模糊敏感的占卜结果或钱包余额。

### 3.3 国际化
- **语言**: 首批支持简体中文、繁体中文和英文。
- **本地历法**: 占卜输入支持公历和农历切换。

---

## 4. 验收标准
1. 映射表中列出的所有 “P0” 级接口必须有对应的、功能完备的 UI 组件。
2. 应用程序必须能通过 `@polkadot/api` 成功连接到 Cosmos 测试网/主网。
3. 媒体资源（照片、文档）必须能通过 IPFS 正确解析。
4. 用户私钥必须保持加密状态，且第三方脚本无法访问。
