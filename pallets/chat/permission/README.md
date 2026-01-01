# Pallet Chat Permission

聊天权限系统模块 - 基于场景的多场景共存权限控制

## 概述

本模块提供聊天权限管理功能：

- **场景授权**：业务模块可为用户授予基于场景的聊天权限
- **好友关系**：用户间可建立双向好友关系
- **黑白名单**：用户可屏蔽特定用户或设置白名单
- **权限级别**：Open/FriendsOnly/Whitelist/Closed

## 核心功能

### 场景授权

业务模块通过 `SceneAuthorizationManager` trait 管理场景授权：

```rust
T::ChatPermission::grant_bidirectional_scene_authorization(
    *b"otc_ordr",
    &buyer,
    &seller,
    SceneType::Order,
    SceneId::Numeric(order_id),
    Some(30 * 24 * 60 * 10), // 30天
    "订单#123".as_bytes().to_vec(),
)?;
```

### 权限检查优先级

1. **黑名单检查**（最高优先级拒绝）
2. **好友关系检查**
3. **场景授权检查**
4. **隐私设置检查**

## 存储结构

- `PrivacySettingsOf`: 用户隐私设置
- `Friendships`: 好友关系（双向存储）
- `SceneAuthorizations`: 场景授权（排序后的用户对）

## 事件

- `PrivacySettingsUpdated`
- `UserBlocked` / `UserUnblocked`
- `FriendshipCreated` / `FriendshipRemoved`
- `SceneAuthorizationGranted` / `SceneAuthorizationRevoked`

## 依赖

- `pallet-chat-common`: 共享类型和工具
