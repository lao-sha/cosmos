# Cosmos 聊天系统模块

统一的聊天系统模块集合，提供完整的即时通讯和AI对话功能。

## 模块概览

```
pallets/chat/
├── common/       # 共享类型和工具库
├── permission/   # 权限系统（场景授权+黑白名单）
├── core/         # 核心私聊模块
├── group/        # 智能群聊模块（四种加密模式）
└── ai/           # AI对话模块（与逝者数字代理对话）
```

## 模块依赖关系

```
                    ┌─────────────┐
                    │   common    │  ← 共享类型（无pallet依赖）
                    └──────┬──────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
       ┌────────────┐ ┌─────────┐ ┌─────────────┐
       │ permission │ │  core   │ │    group    │
       └──────┬─────┘ └────┬────┘ └──────┬──────┘
              │            │             │
              └────────────┼─────────────┘
                           ▼
                    ┌─────────────┐
                    │     ai      │
                    └─────────────┘
```

## 各模块功能

### common - 共享类型库

提供聊天系统的基础类型和工具：

- **MessageType**: 消息类型（文本/图片/文件/语音/视频/系统/AI）
- **EncryptionMode**: 加密模式（Military/Business/Selective/Transparent）
- **ChatUserId**: 11位数字聊天ID（10000000000-99999999999）
- **Traits**: ChatPermissionCheck, FriendshipCheck, ChatUserIdProvider
- **Validation**: CID格式验证、加密CID验证
- **RateLimit**: 防刷机制

### permission - 权限系统

场景化权限控制：

- **SceneType**: 支持多种场景（私聊/群聊/AI对话/逝者纪念等）
- **PermissionLevel**: 权限级别（Block/ReadOnly/Normal/Premium/Admin）
- **白名单/黑名单**: 灵活的访问控制
- **临时授权**: 带过期时间的访问令牌

### core - 核心私聊

基础的一对一聊天功能：

- **ChatUserId分配**: 唯一11位数字ID
- **消息发送**: 支持多种消息类型
- **消息状态**: 已发送/已送达/已读/已撤回
- **权限集成**: 基于permission模块的访问控制

### group - 智能群聊

四种加密模式的群组聊天：

| 模式 | 描述 | 适用场景 |
|------|------|----------|
| Military | 量子抗性加密 | 高度机密群组 |
| Business | AES-256加密 | 普通私密群组（默认）|
| Selective | 选择性加密 | 部分消息需加密 |
| Transparent | 透明公开 | 公开群组 |

功能：
- 群组创建/解散
- 成员管理（加入/离开/踢出）
- 群组消息广播
- 管理员权限管理

### ai - AI对话

与逝者数字代理的智能对话：

- **会话管理**: 创建、暂停、归档、恢复
- **消息交互**: 用户消息 ↔ AI响应
- **质量评估**: 多维度评分（相关性、人格匹配、情感真实性等）
- **OCW集成**: 支持多AI服务商（OpenAI/Anthropic/阿里云/百度）

## Runtime 配置

在 `runtime/Cargo.toml` 中添加依赖：

```toml
pallet-chat-common = { path = "../pallets/chat/common", default-features = false }
pallet-chat-permission = { path = "../pallets/chat/permission", default-features = false }
pallet-chat-core = { path = "../pallets/chat/core", default-features = false }
pallet-chat-group = { path = "../pallets/chat/group", default-features = false }
pallet-chat-ai = { path = "../pallets/chat/ai", default-features = false }
```

在 `std` feature 中添加：

```toml
"pallet-chat-common/std",
"pallet-chat-permission/std",
"pallet-chat-core/std",
"pallet-chat-group/std",
"pallet-chat-ai/std",
```

## 设计原则

1. **低耦合**: common模块不依赖任何pallet，各子模块通过traits交互
2. **类型统一**: 所有消息类型、加密模式在common中定义
3. **权限集中**: 统一的权限检查通过permission模块
4. **可扩展**: 新增聊天场景只需扩展SceneType枚举

## 迁移历史

- 2025-12-29: 统一整合以下模块到 `pallets/chat/` 目录：
  - `pallets/chat` → `pallets/chat/core`
  - `pallets/chat-permission` → `pallets/chat/permission`
  - `pallets/smart-group-chat` → `pallets/chat/group`
  - `pallets/ai-chat` → `pallets/chat/ai`
  - 新建 `pallets/chat/common` 共享类型库
