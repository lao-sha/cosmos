# pallet-livestream

去中心化直播间模块，为 Stardust 平台提供直播功能。

## 功能特性

### 直播间管理
- 创建直播间 (普通/付费/私密/连麦)
- 开始/暂停/恢复/结束直播
- 更新直播间信息

### 礼物系统
- 发送礼物打赏
- 平台抽成 (可配置)
- 主播收益提现

### 付费直播
- 设置门票价格
- 购买门票
- 门票验证

### 连麦功能
- 开始/结束连麦
- 最多支持 4 人同时连麦

### 管理功能
- 踢出观众/黑名单
- 封禁直播间 (管理员)

## 设计原则

### 链上链下分离

| 链上 | 链下 (LiveKit) |
|------|----------------|
| 直播间创建/状态 | 视频/音频流传输 |
| 礼物打赏/分成 | 观众进入/离开 |
| 门票购买 | 聊天/弹幕 |
| 黑名单 | 连麦申请/同意 |
| 连麦记录 | 禁言/静音 |

### 存储优化

- 观众列表: 链下 LiveKit 管理
- 礼物记录: 用事件替代存储
- 连麦申请: 链下 DataChannel

### 安全设计

- 不存储 stream_key，使用签名验证
- 押金机制防止滥用
- 权限控制严格

## 配置参数

```rust
type MaxTitleLen = ConstU32<100>;        // 标题最大长度
type MaxDescriptionLen = ConstU32<500>;  // 描述最大长度
type MaxCidLen = ConstU32<64>;           // CID 最大长度
type MaxGiftNameLen = ConstU32<32>;      // 礼物名称最大长度
type MaxCoHostsPerRoom = ConstU32<4>;    // 最大连麦人数
type PlatformFeePercent = ConstU8<20>;   // 平台抽成 20%
type MinWithdrawAmount = ConstU128<...>; // 最小提现金额
type RoomDeposit = ConstU128<...>;       // 创建直播间押金
```

## 调用函数

### 直播间管理
- `create_room` - 创建直播间
- `start_live` - 开始直播
- `pause_live` - 暂停直播
- `resume_live` - 恢复直播
- `end_live` - 结束直播
- `update_room` - 更新信息

### 门票系统
- `buy_ticket` - 购买门票

### 礼物系统
- `send_gift` - 发送礼物
- `withdraw_earnings` - 提现收益
- `sync_live_stats` - 同步统计

### 管理功能
- `kick_viewer` - 踢出观众
- `remove_from_blacklist` - 移除黑名单
- `ban_room` - 封禁直播间

### 连麦功能
- `start_co_host` - 开始连麦
- `end_co_host` - 结束连麦

### 礼物管理
- `create_gift` - 创建礼物
- `update_gift` - 更新礼物

## 事件

- `RoomCreated` - 直播间创建
- `LiveStarted` / `LivePaused` / `LiveResumed` / `LiveEnded` - 直播状态
- `TicketPurchased` - 门票购买
- `GiftSent` - 礼物发送
- `EarningsWithdrawn` - 收益提现
- `ViewerKicked` / `ViewerUnbanned` - 黑名单操作
- `RoomBanned` - 直播间封禁
- `CoHostStarted` / `CoHostEnded` - 连麦状态

## 收益分配

```
礼物收入:
├── 主播: 80%
└── 平台: 20% → 国库
```

## 测试

```bash
cargo test -p pallet-livestream
```

## License

Apache-2.0
