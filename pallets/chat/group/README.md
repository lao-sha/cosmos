# Pallet Chat Group

智能群聊系统 - 支持四种加密模式

## 概述

本模块提供群聊功能：

- **群组管理**：创建、解散群组
- **成员管理**：加入、离开、踢出成员
- **消息发送**：群组消息广播
- **四种加密模式**：Military/Business/Selective/Transparent

## 加密模式

| 模式 | 描述 | 适用场景 |
|------|------|----------|
| Military | 军用级量子抗性加密 | 高度机密群组 |
| Business | 商用级AES-256加密 | 普通私密群组（默认）|
| Selective | 选择性加密 | 部分消息需加密 |
| Transparent | 透明公开 | 公开群组 |

## 核心功能

### 群组创建

```rust
// 创建群组
Chat::create_group(
    origin,
    name,           // 群组名称
    description,    // 描述（可选）
    encryption_mode,// 加密模式
    is_public,      // 是否公开
)?;
```

### 消息发送

```rust
// 发送群组消息
Chat::send_group_message(
    origin,
    group_id,       // 群组ID
    content,        // 消息内容（CID）
    message_type,   // 消息类型
)?;
```

## 存储结构

- `Groups`: 群组信息
- `GroupMembers`: 群组成员
- `UserGroups`: 用户的群组列表
- `GroupMessages`: 群组消息
- `NextMessageId`: 消息ID计数器

## 依赖

- `pallet-chat-common`: 共享类型（MessageType, EncryptionMode等）
- `pallet-chat-permission`: 权限检查
- `cosmos-media-common`: 媒体验证
