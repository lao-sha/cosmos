# Stardust 直播间模块架构设计

## 概述

直播间模块 (`pallet-livestream`) 为 Stardust 平台提供去中心化直播功能，支持主播开播、观众互动、礼物打赏等核心功能。

## 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        Frontend (React Native)                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ 直播播放器   │  │ 弹幕/聊天   │  │ 礼物/打赏面板           │  │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘  │
└─────────┼────────────────┼─────────────────────┼────────────────┘
          │                │                     │
          ▼                ▼                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Media Server (链下)                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ WebRTC SFU  │  │ HLS/DASH    │  │ 弹幕服务 (WebSocket)    │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
          │                │                     │
          ▼                ▼                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Substrate Blockchain                          │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                   pallet-livestream                          ││
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌─────────────┐  ││
│  │  │ 直播间管理 │ │ 观众管理  │ │ 礼物系统  │ │ 收益分配    │  ││
│  │  └───────────┘ └───────────┘ └───────────┘ └─────────────┘  ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────────┐│
│  │ pallet-tips │ │pallet-escrow│ │ pallet-stardust-ipfs        ││
│  └─────────────┘ └─────────────┘ └─────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### 模块目录结构

```
pallets/livestream/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs           # 主模块入口
│   ├── types.rs         # 类型定义
│   ├── weights.rs       # 权重定义
│   ├── mock.rs          # 测试mock
│   └── tests.rs         # 单元测试
```

## 核心类型定义

### 直播间状态

```rust
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum LiveRoomStatus {
    /// 准备中（未开播）
    Preparing,
    /// 直播中
    Live,
    /// 暂停中
    Paused,
    /// 已结束
    Ended,
    /// 被封禁
    Banned,
}
```

### 直播间类型

```rust
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum LiveRoomType {
    /// 普通直播
    Normal,
    /// 付费直播（需购票）
    Paid,
    /// 私密直播（仅邀请）
    Private,
    /// 连麦直播
    MultiHost,
}
```

### 直播间信息

```rust
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct LiveRoom<AccountId, BoundedTitle, BoundedDescription, Balance> {
    /// 直播间ID（自增）
    pub id: u64,
    /// 主播账户
    pub host: AccountId,
    /// 直播间标题
    pub title: BoundedTitle,
    /// 直播间描述
    pub description: Option<BoundedDescription>,
    /// 直播间类型
    pub room_type: LiveRoomType,
    /// 直播间状态
    pub status: LiveRoomStatus,
    /// 封面图CID (IPFS)
    pub cover_cid: Option<BoundedVec<u8, MaxCidLen>>,
    /// 累计观众数（直播结束时从 LiveKit 同步）
    pub total_viewers: u64,
    /// 峰值观众数
    pub peak_viewers: u32,
    /// 累计礼物收入
    pub total_gifts: Balance,
    /// 付费直播票价（仅Paid类型）
    pub ticket_price: Option<Balance>,
    /// 创建时间
    pub created_at: u64,
    /// 开播时间
    pub started_at: Option<u64>,
    /// 结束时间
    pub ended_at: Option<u64>,
    // 注意: 不存储 stream_key，改用签名验证机制
}
```

> **安全设计**: 不在链上存储 `stream_key`，主播推流时通过私钥签名验证身份，防止任何人读取链上数据后冒充主播推流。

### 礼物定义

```rust
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct Gift<Balance> {
    /// 礼物ID
    pub id: u32,
    /// 礼物名称
    pub name: BoundedVec<u8, MaxGiftNameLen>,
    /// 礼物价格
    pub price: Balance,
    /// 礼物图标CID
    pub icon_cid: BoundedVec<u8, MaxCidLen>,
    /// 是否启用
    pub enabled: bool,
}

/// 礼物记录
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct GiftRecord<AccountId, Balance> {
    /// 送礼者
    pub sender: AccountId,
    /// 接收者（主播）
    pub receiver: AccountId,
    /// 直播间ID
    pub room_id: u64,
    /// 礼物ID
    pub gift_id: u32,
    /// 数量
    pub quantity: u32,
    /// 总价值
    pub total_value: Balance,
    /// 时间戳
    pub timestamp: u64,
}
```

### 观众信息（链下管理）

> **设计变更**: 观众信息不再存储在链上，改为通过 LiveKit 管理。链上只存储付费直播的门票购买记录和黑名单。

```rust
// 观众实时状态由 LiveKit 管理，不存储在链上
// 链上只记录以下信息:

/// 付费直播门票购买记录
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct Ticket<AccountId> {
    /// 购买者
    pub buyer: AccountId,
    /// 直播间ID
    pub room_id: u64,
    /// 购买时间
    pub purchased_at: u64,
}
```

## 存储设计

> **设计原则**: 最小化链上存储，观众管理、聊天、弹幕等高频操作全部链下处理。链上只存储必要的状态和资金相关数据。

```rust
/// 直播间信息
#[pallet::storage]
pub type LiveRooms<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64,  // room_id
    LiveRoom<T::AccountId, BoundedTitle, BoundedDescription, BalanceOf<T>>,
>;

/// 主播的直播间（一个主播只能有一个活跃直播间）
#[pallet::storage]
pub type HostRoom<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,  // host
    u64,           // room_id
>;

/// 下一个直播间ID（自增）
#[pallet::storage]
pub type NextRoomId<T: Config> = StorageValue<_, u64, ValueQuery>;

/// 礼物定义
#[pallet::storage]
pub type Gifts<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u32,  // gift_id
    Gift<BalanceOf<T>>,
>;

/// 用户在直播间的累计打赏
#[pallet::storage]
pub type UserRoomGifts<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    u64,           // room_id
    Blake2_128Concat,
    T::AccountId,  // user
    BalanceOf<T>,  // total_gifted
>;

/// 主播累计收入
#[pallet::storage]
pub type HostEarnings<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,  // host
    BalanceOf<T>,  // total_earnings
    ValueQuery,
>;

/// 付费直播门票持有者
#[pallet::storage]
pub type TicketHolders<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    u64,           // room_id
    Blake2_128Concat,
    T::AccountId,  // buyer
    u64,           // purchase_time
>;

/// 直播间黑名单（被踢出的用户）
#[pallet::storage]
pub type RoomBlacklist<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    u64,           // room_id
    Blake2_128Concat,
    T::AccountId,  // banned_user
    (),
>;

/// 当前连麦者列表（简化存储）
#[pallet::storage]
pub type ActiveCoHosts<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64,  // room_id
    BoundedVec<T::AccountId, T::MaxCoHostsPerRoom>,
    ValueQuery,
>;
```

### 已删除的存储（改为链下或事件记录）

| 原存储 | 处理方式 | 原因 |
|--------|----------|------|
| `RoomViewers` | 链下 LiveKit | 10000观众存链上会导致存储爆炸 |
| `GiftRecords` | 事件记录 | 历史记录通过事件索引查询 |
| `CoHostRequests` | 链下 DataChannel | 申请流程无需上链 |

## 可调用函数 (Extrinsics)

### 直播间管理

```rust
/// 创建直播间
#[pallet::call_index(0)]
pub fn create_room(
    origin: OriginFor<T>,
    title: Vec<u8>,
    description: Option<Vec<u8>>,
    room_type: LiveRoomType,
    cover_cid: Option<Vec<u8>>,
    ticket_price: Option<BalanceOf<T>>,
) -> DispatchResult;

/// 开始直播
#[pallet::call_index(1)]
pub fn start_live(
    origin: OriginFor<T>,
    room_id: u64,
) -> DispatchResult;

/// 暂停直播
#[pallet::call_index(2)]
pub fn pause_live(
    origin: OriginFor<T>,
    room_id: u64,
) -> DispatchResult;

/// 恢复直播
#[pallet::call_index(3)]
pub fn resume_live(
    origin: OriginFor<T>,
    room_id: u64,
) -> DispatchResult;

/// 结束直播
#[pallet::call_index(4)]
pub fn end_live(
    origin: OriginFor<T>,
    room_id: u64,
) -> DispatchResult;

/// 更新直播间信息
#[pallet::call_index(5)]
pub fn update_room(
    origin: OriginFor<T>,
    room_id: u64,
    title: Option<Vec<u8>>,
    description: Option<Vec<u8>>,
    cover_cid: Option<Vec<u8>>,
) -> DispatchResult;
```

### 观众操作

> **设计变更**: `join_room` 和 `leave_room` 已移至链下（直接连接/断开 LiveKit）。链上只保留付费直播购票功能。

```rust
/// 购买付费直播门票
#[pallet::call_index(10)]
pub fn buy_ticket(
    origin: OriginFor<T>,
    room_id: u64,
) -> DispatchResult;
```

### 礼物系统

```rust
/// 发送礼物（含防溢出和状态检查）
#[pallet::call_index(20)]
pub fn send_gift(
    origin: OriginFor<T>,
    room_id: u64,
    gift_id: u32,
    quantity: u32,
) -> DispatchResult {
    let sender = ensure_signed(origin)?;

    // 1. 检查直播间状态
    let mut room = LiveRooms::<T>::get(room_id).ok_or(Error::<T>::RoomNotFound)?;
    ensure!(room.status == LiveRoomStatus::Live, Error::<T>::RoomNotLive);

    // 2. 检查礼物存在且启用
    let gift = Gifts::<T>::get(gift_id).ok_or(Error::<T>::GiftNotFound)?;
    ensure!(gift.enabled, Error::<T>::GiftDisabled);

    // 3. 计算总价（防溢出）
    let total = gift.price
        .checked_mul(&quantity.into())
        .ok_or(Error::<T>::Overflow)?;

    // 4. 计算分成
    let platform_fee = total.saturating_mul(T::PlatformFeePercent::get().into()) / 100u32.into();
    let host_amount = total.saturating_sub(platform_fee);

    // 5. 转账给主播
    T::Currency::transfer(&sender, &room.host, host_amount, ExistenceRequirement::KeepAlive)?;

    // 6. 转账给国库
    T::Currency::transfer(&sender, &T::TreasuryAccount::get(), platform_fee, ExistenceRequirement::KeepAlive)?;

    // 7. 更新统计
    room.total_gifts = room.total_gifts.saturating_add(total);
    LiveRooms::<T>::insert(room_id, room.clone());

    HostEarnings::<T>::mutate(&room.host, |e| *e = e.saturating_add(host_amount));
    UserRoomGifts::<T>::mutate(room_id, &sender, |g| *g = g.saturating_add(total));

    // 8. 发出事件（替代存储记录）
    Self::deposit_event(Event::GiftSent {
        room_id,
        sender: sender.clone(),
        receiver: room.host,
        gift_id,
        quantity,
        value: total,
    });

    Ok(())
}

/// 主播提现收益
#[pallet::call_index(21)]
pub fn withdraw_earnings(
    origin: OriginFor<T>,
    amount: BalanceOf<T>,
) -> DispatchResult;

/// 同步直播统计数据（直播结束时由后端调用）
#[pallet::call_index(22)]
pub fn sync_live_stats(
    origin: OriginFor<T>,
    room_id: u64,
    total_viewers: u64,
    peak_viewers: u32,
) -> DispatchResult;
```

### 管理功能

> **设计变更**: 禁言功能移至链下（通过 LiveKit 权限控制）。链上只保留黑名单和封禁功能。

```rust
/// 踢出观众并加入黑名单（主播）
#[pallet::call_index(30)]
pub fn kick_viewer(
    origin: OriginFor<T>,
    room_id: u64,
    viewer: T::AccountId,
) -> DispatchResult;

/// 从黑名单移除
#[pallet::call_index(31)]
pub fn remove_from_blacklist(
    origin: OriginFor<T>,
    room_id: u64,
    viewer: T::AccountId,
) -> DispatchResult;

/// 封禁直播间（管理员）
#[pallet::call_index(40)]
pub fn ban_room(
    origin: OriginFor<T>,
    room_id: u64,
    reason: Vec<u8>,
) -> DispatchResult;
```

### 连麦功能（简化版）

> **设计变更**: 连麦申请/同意/拒绝流程移至链下（通过 DataChannel）。链上只记录连麦开始和结束。

```rust
/// 开始连麦（主播调用，记录连麦者）
#[pallet::call_index(50)]
pub fn start_co_host(
    origin: OriginFor<T>,
    room_id: u64,
    co_host: T::AccountId,
) -> DispatchResult;

/// 结束连麦（主播或连麦者调用）
#[pallet::call_index(51)]
pub fn end_co_host(
    origin: OriginFor<T>,
    room_id: u64,
    co_host: Option<T::AccountId>,  // None = 自己退出
) -> DispatchResult;
```

## 事件定义

> **设计原则**: 事件用于替代链上存储记录，前端/索引器通过监听事件获取历史数据。

```rust
#[pallet::event]
pub enum Event<T: Config> {
    /// 直播间已创建
    RoomCreated { host: T::AccountId, room_id: u64 },
    /// 直播已开始
    LiveStarted { room_id: u64, started_at: u64 },
    /// 直播已暂停
    LivePaused { room_id: u64 },
    /// 直播已恢复
    LiveResumed { room_id: u64 },
    /// 直播已结束（含统计数据）
    LiveEnded {
        room_id: u64,
        duration: u64,
        total_viewers: u64,
        peak_viewers: u32,
        total_gifts: BalanceOf<T>,
    },
    /// 门票已购买
    TicketPurchased { room_id: u64, buyer: T::AccountId, price: BalanceOf<T> },
    /// 礼物已发送（替代 GiftRecords 存储）
    GiftSent {
        room_id: u64,
        sender: T::AccountId,
        receiver: T::AccountId,
        gift_id: u32,
        quantity: u32,
        value: BalanceOf<T>,
    },
    /// 收益已提现
    EarningsWithdrawn { host: T::AccountId, amount: BalanceOf<T> },
    /// 观众被踢出（加入黑名单）
    ViewerKicked { room_id: u64, viewer: T::AccountId },
    /// 观众从黑名单移除
    ViewerUnbanned { room_id: u64, viewer: T::AccountId },
    /// 直播间被封禁
    RoomBanned { room_id: u64, reason: Vec<u8> },
    /// 连麦已开始
    CoHostStarted { room_id: u64, co_host: T::AccountId },
    /// 连麦已结束
    CoHostEnded { room_id: u64, co_host: T::AccountId, duration: u64 },
}
```

## 配置参数

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    /// 货币类型
    type Currency: ReservableCurrency<Self::AccountId>;

    /// 时间服务
    type TimeProvider: UnixTime;

    /// 直播间标题最大长度
    #[pallet::constant]
    type MaxTitleLen: Get<u32>;  // 建议: 100

    /// 直播间描述最大长度
    #[pallet::constant]
    type MaxDescriptionLen: Get<u32>;  // 建议: 500

    /// CID最大长度
    #[pallet::constant]
    type MaxCidLen: Get<u32>;  // 建议: 64

    /// 礼物名称最大长度
    #[pallet::constant]
    type MaxGiftNameLen: Get<u32>;  // 建议: 32

    /// 最大连麦人数
    #[pallet::constant]
    type MaxCoHostsPerRoom: Get<u32>;  // 建议: 4

    /// 平台抽成比例 (百分比)
    #[pallet::constant]
    type PlatformFeePercent: Get<u8>;  // 建议: 10-20%

    /// 最小提现金额
    #[pallet::constant]
    type MinWithdrawAmount: Get<BalanceOf<Self>>;

    /// 创建直播间押金
    #[pallet::constant]
    type RoomDeposit: Get<BalanceOf<Self>>;

    /// 国库账户
    #[pallet::constant]
    type TreasuryAccount: Get<Self::AccountId>;

    /// Pallet ID
    #[pallet::constant]
    type PalletId: Get<PalletId>;

    /// 权重信息
    type WeightInfo: WeightInfo;
}
```

### 已删除的配置项

| 原配置 | 原因 |
|--------|------|
| `MaxViewersPerRoom` | 观众管理移至链下，无需限制 |
| `MaxStreamKeyLen` | 不再存储 stream_key |
| `Randomness` | 改用自增 ID，无需随机数 |

## 收益分配机制

```
礼物收入分配:
┌─────────────────────────────────────────┐
│           用户支付礼物金额               │
└────────────────┬────────────────────────┘
                 │
        ┌────────┴────────┐
        ▼                 ▼
┌───────────────┐  ┌───────────────┐
│ 主播收益 (80%) │  │ 平台抽成 (20%)│
└───────────────┘  └───────┬───────┘
                           │
              ┌────────────┴────────────┐
              ▼                         ▼
       ┌─────────────┐          ┌─────────────┐
       │ 国库 (70%)  │          │ 推荐人 (30%)│
       └─────────────┘          └─────────────┘
```

## 与其他模块的集成

### 与 pallet-tips 集成
- 复用打赏逻辑
- 共享收益分配机制

### 与 pallet-escrow 集成
- 付费直播门票托管
- 主播收益托管

### 与 pallet-stardust-ipfs 集成
- 直播封面存储
- 礼物图标存储
- 直播回放存储

### 与 pallet-referral 集成
- 推荐人分成
- 邀请奖励

## 链下组件

### 媒体服务器
直播流媒体传输不适合在链上处理，需要链下媒体服务器：

```
推荐技术栈:
- WebRTC SFU: mediasoup / Janus / LiveKit
- 流媒体协议: HLS / DASH / WebRTC
- 弹幕服务: WebSocket + Redis
- CDN: 边缘节点分发
```

### 链上链下交互

```
1. 主播创建直播间 → 链上记录 → 返回 stream_key
2. 主播推流 → 媒体服务器 (使用 stream_key 验证)
3. 观众进入 → 链上记录 → 获取播放地址
4. 观众送礼 → 链上转账 → 触发弹幕特效
5. 直播结束 → 链上记录 → 可选保存回放到 IPFS
```

## 安全考虑

1. **防刷机制**: 限制创建直播间频率、送礼频率
2. **押金机制**: 创建直播间需要押金，违规扣除
3. **权限控制**: 只有主播可以管理自己的直播间
4. **金额验证**: 礼物金额、提现金额需要验证
5. **状态检查**: 只有直播中的房间才能送礼

## 后续扩展

1. **连麦功能**: 多主播同时直播
2. **PK功能**: 主播间PK对战
3. **直播回放**: 保存到IPFS
4. **直播预约**: 预约开播通知
5. **直播分类**: 游戏、音乐、聊天等分类
6. **粉丝等级**: 根据打赏累计升级
7. **守护功能**: 月度守护特权

---

## 媒体服务方案：LiveKit

本项目选用 **LiveKit** 作为直播媒体服务方案。

### 为什么选择 LiveKit

| 优势 | 说明 |
|------|------|
| **开源** | 完全开源 (Apache 2.0)，可随时迁移自建 |
| **无最低消费** | 免费开始，按量付费，适合早期项目 |
| **定价透明** | $0.01/分钟，无隐藏费用 |
| **现代架构** | Go 语言，云原生，支持 Kubernetes |
| **SDK 完善** | Web/iOS/Android/Flutter/React Native |
| **内置 DataChannel** | 可直接实现聊天/弹幕，无需额外服务 |

### 部署方案

#### 方案一：LiveKit Cloud (推荐早期使用)

```
零运维，按量付费:

价格:
├─ 视频: $0.01/分钟/参与者
├─ 音频: $0.004/分钟/参与者
└─ 免费额度: 1000 分钟/月

成本估算 (100观众看1小时):
100人 × 60分钟 × $0.01 = $60/场
```

#### 方案二：自建 LiveKit Server

```yaml
# docker-compose.yml
version: '3.8'

services:
  livekit:
    image: livekit/livekit-server:latest
    command: --config /etc/livekit.yaml
    ports:
      - "7880:7880"
      - "7881:7881"
      - "7882:7882/udp"
      - "50000-50100:50000-50100/udp"
    volumes:
      - ./livekit.yaml:/etc/livekit.yaml

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
```

```yaml
# livekit.yaml
port: 7880
rtc:
  port_range_start: 50000
  port_range_end: 50100
  use_external_ip: true

redis:
  address: redis:6379

keys:
  APIxxxxxxx: secretxxxxxxx

turn:
  enabled: true
  udp_port: 3478
```

### 前端集成 (React Native)

```bash
npm install @livekit/react-native @livekit/react-native-webrtc
```

#### 主播推流

```typescript
import { Room } from 'livekit-client';

class LiveKitPublisher {
  private room: Room;

  async startPublishing(roomName: string, token: string) {
    this.room = new Room();
    await this.room.connect('wss://your-livekit-server.com', token);
    await this.room.localParticipant.enableCameraAndMicrophone();
  }

  async switchCamera() {
    const track = this.room.localParticipant.getTrackPublication('camera');
    if (track?.videoTrack) {
      await track.videoTrack.switchCamera();
    }
  }

  async toggleMute() {
    const enabled = this.room.localParticipant.isMicrophoneEnabled;
    await this.room.localParticipant.setMicrophoneEnabled(!enabled);
  }

  async stopPublishing() {
    await this.room.disconnect();
  }
}
```

#### 观众拉流

```typescript
import { Room, RoomEvent, Track } from 'livekit-client';

class LiveKitViewer {
  private room: Room;

  async startViewing(token: string, onVideoTrack: (track: Track) => void) {
    this.room = new Room();
    
    this.room.on(RoomEvent.TrackSubscribed, (track) => {
      if (track.kind === Track.Kind.Video) {
        onVideoTrack(track);
      }
    });

    await this.room.connect('wss://your-livekit-server.com', token);
  }

  async stopViewing() {
    await this.room.disconnect();
  }
}
```

### 后端 Token 生成（含签名验证）

> **安全设计**: 主播推流前需用私钥签名，后端验证签名后才生成 LiveKit Token，防止冒充推流。

```typescript
import { AccessToken } from 'livekit-server-sdk';
import { signatureVerify, decodeAddress } from '@polkadot/util-crypto';
import { ApiPromise, WsProvider } from '@polkadot/api';

// 连接 Substrate 节点
const api = await ApiPromise.create({ provider: new WsProvider('ws://localhost:9944') });

/**
 * 验证主播身份并生成推流 Token
 */
async function generatePublisherToken(
  roomId: string,
  publicKey: string,    // 主播公钥 (SS58 地址)
  signature: string,    // 主播签名
  timestamp: number     // 签名时间戳
): Promise<string> {
  // 1. 检查时间戳有效性（防重放攻击，5分钟内有效）
  const now = Date.now();
  if (Math.abs(now - timestamp) > 5 * 60 * 1000) {
    throw new Error('Signature expired');
  }

  // 2. 构造签名消息
  const message = `livestream:${roomId}:${timestamp}`;

  // 3. 验证签名
  const { isValid } = signatureVerify(message, signature, publicKey);
  if (!isValid) {
    throw new Error('Invalid signature');
  }

  // 4. 查询链上确认是房主
  const room = await api.query.livestream.liveRooms(roomId);
  if (room.isNone) {
    throw new Error('Room not found');
  }
  const roomData = room.unwrap();
  if (roomData.host.toString() !== publicKey) {
    throw new Error('Not room host');
  }

  // 5. 检查直播间状态
  if (roomData.status.toString() !== 'Live' && roomData.status.toString() !== 'Preparing') {
    throw new Error('Room not available for streaming');
  }

  // 6. 生成 LiveKit Token（发布者权限）
  const token = new AccessToken(
    process.env.LIVEKIT_API_KEY!,
    process.env.LIVEKIT_API_SECRET!,
    { identity: publicKey, name: `Host-${roomId}`, ttl: '6h' }
  );

  token.addGrant({
    room: `room-${roomId}`,
    roomJoin: true,
    canPublish: true,       // 可以推流
    canSubscribe: true,
    canPublishData: true,   // 可以发送聊天消息
  });

  return token.toJwt();
}

/**
 * 生成观众观看 Token
 */
async function generateViewerToken(
  roomId: string,
  viewerAddress: string
): Promise<string> {
  // 1. 查询直播间
  const room = await api.query.livestream.liveRooms(roomId);
  if (room.isNone) {
    throw new Error('Room not found');
  }
  const roomData = room.unwrap();

  // 2. 检查直播间状态
  if (roomData.status.toString() !== 'Live') {
    throw new Error('Room not live');
  }

  // 3. 如果是付费直播，检查是否购票
  if (roomData.roomType.toString() === 'Paid') {
    const ticket = await api.query.livestream.ticketHolders(roomId, viewerAddress);
    if (ticket.isNone) {
      throw new Error('Ticket required');
    }
  }

  // 4. 检查是否在黑名单
  const banned = await api.query.livestream.roomBlacklist(roomId, viewerAddress);
  if (banned.isSome) {
    throw new Error('You are banned from this room');
  }

  // 5. 生成 LiveKit Token（观众权限）
  const token = new AccessToken(
    process.env.LIVEKIT_API_KEY!,
    process.env.LIVEKIT_API_SECRET!,
    { identity: viewerAddress, ttl: '6h' }
  );

  token.addGrant({
    room: `room-${roomId}`,
    roomJoin: true,
    canPublish: false,      // 不能推流
    canSubscribe: true,     // 可以观看
    canPublishData: true,   // 可以发送聊天消息
  });

  return token.toJwt();
}
```

### 前端签名流程

> ⚠️ **React Native 注意**: 不能使用 `@polkadot/extension-dapp`（浏览器扩展 API），必须使用移动端签名器。

```typescript
// React Native 移动端签名实现
import { getCurrentPair } from '@/lib/signer.native';
import { u8aToHex } from '@polkadot/util';

async function getPublisherToken(roomId: string, address: string): Promise<string> {
  // 1. 获取当前密钥对 (需要先解锁钱包)
  const pair = getCurrentPair();
  if (!pair) {
    throw new Error('Wallet is locked. Please unlock first.');
  }

  // 2. 构造签名消息
  const timestamp = Date.now();
  const message = `livestream:${roomId}:${timestamp}`;

  // 3. 签名
  const messageU8a = new TextEncoder().encode(message);
  const signatureU8a = pair.sign(messageU8a);
  const signature = u8aToHex(signatureU8a);

  // 4. 请求后端生成 Token
  const response = await fetch('/api/livestream/publisher-token', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ roomId, publicKey: address, signature, timestamp }),
  });

  if (!response.ok) {
    throw new Error('Failed to get publisher token');
  }

  const { token } = await response.json();
  return token;
}

// 使用前需要先解锁钱包
import { unlockWallet } from '@/lib/signer.native';
await unlockWallet(userPassword);
```

#### Web 端签名 (仅供参考，本项目为 React Native)

```typescript
// 浏览器环境可使用 Polkadot.js 扩展
import { web3FromAddress } from '@polkadot/extension-dapp';

const injector = await web3FromAddress(address);
const { signature } = await injector.signer.signRaw!({
  address,
  data: message,
  type: 'bytes',
});
```

---

## 直播间聊天系统 (LiveKit DataChannel)

使用 LiveKit 内置 DataChannel 实现实时聊天和弹幕，无需额外服务器。

### 消息类型定义

```typescript
type MessageType = 'chat' | 'danmaku' | 'gift' | 'system' | 'emoji' | 'like';

interface LiveChatMessage {
  type: MessageType;
  content: string;
  sender: string;        // 账户地址
  senderName: string;    // 显示名称
  timestamp: number;
  // 礼物消息
  giftId?: number;
  giftCount?: number;
  giftValue?: number;
  // 弹幕样式
  color?: string;
  position?: 'scroll' | 'top' | 'bottom';
}
```

### 聊天服务封装

```typescript
import { Room, RoomEvent } from 'livekit-client';

class LiveChatService {
  private room: Room;
  private messageHandlers: Map<MessageType, (msg: LiveChatMessage) => void> = new Map();

  constructor(room: Room) {
    this.room = room;
    this.setupMessageListener();
  }

  private setupMessageListener() {
    this.room.on(RoomEvent.DataReceived, (payload) => {
      try {
        const message: LiveChatMessage = JSON.parse(new TextDecoder().decode(payload));
        const handler = this.messageHandlers.get(message.type);
        handler?.(message);
      } catch (e) {
        console.error('消息解析失败:', e);
      }
    });
  }

  onMessage(type: MessageType, handler: (msg: LiveChatMessage) => void) {
    this.messageHandlers.set(type, handler);
  }

  // 发送聊天消息
  async sendChat(content: string) {
    await this.sendMessage({
      type: 'chat',
      content,
      sender: this.room.localParticipant.identity,
      senderName: this.room.localParticipant.name || '匿名用户',
      timestamp: Date.now(),
    });
  }

  // 发送弹幕
  async sendDanmaku(content: string, color = '#FFFFFF') {
    await this.sendMessage({
      type: 'danmaku',
      content,
      sender: this.room.localParticipant.identity,
      senderName: this.room.localParticipant.name || '匿名用户',
      timestamp: Date.now(),
      color,
      position: 'scroll',
    });
  }

  // 发送礼物通知 (链上打赏成功后调用)
  async sendGiftNotification(giftId: number, giftCount: number, giftValue: number) {
    await this.sendMessage({
      type: 'gift',
      content: `送出了 ${giftCount} 个礼物`,
      sender: this.room.localParticipant.identity,
      senderName: this.room.localParticipant.name || '匿名用户',
      timestamp: Date.now(),
      giftId,
      giftCount,
      giftValue,
    });
  }

  // 发送点赞
  async sendLike() {
    await this.sendMessage({
      type: 'like',
      content: '❤️',
      sender: this.room.localParticipant.identity,
      senderName: '',
      timestamp: Date.now(),
    });
  }

  private async sendMessage(message: LiveChatMessage) {
    const data = new TextEncoder().encode(JSON.stringify(message));
    await this.room.localParticipant.publishData(data, { reliable: true });
  }
}
```

### React Native 聊天组件

```typescript
import React, { useState, useEffect, useRef } from 'react';
import { View, FlatList, TextInput, TouchableOpacity, Text, StyleSheet } from 'react-native';
import { Room } from 'livekit-client';

export function LiveChat({ room }: { room: Room }) {
  const [messages, setMessages] = useState<LiveChatMessage[]>([]);
  const [inputText, setInputText] = useState('');
  const chatService = useRef<LiveChatService | null>(null);

  useEffect(() => {
    chatService.current = new LiveChatService(room);
    
    chatService.current.onMessage('chat', (msg) => {
      setMessages(prev => [...prev, msg].slice(-100));
    });

    chatService.current.onMessage('gift', (msg) => {
      setMessages(prev => [...prev, msg].slice(-100));
    });

    return () => { chatService.current = null; };
  }, [room]);

  const handleSend = async () => {
    if (!inputText.trim() || !chatService.current) return;
    await chatService.current.sendChat(inputText.trim());
    setInputText('');
  };

  const renderMessage = ({ item }: { item: LiveChatMessage }) => {
    if (item.type === 'gift') {
      return (
        <View style={styles.giftMessage}>
          <Text style={styles.giftText}>🎁 {item.senderName} {item.content}</Text>
        </View>
      );
    }
    return (
      <View style={styles.chatMessage}>
        <Text style={styles.senderName}>{item.senderName}:</Text>
        <Text style={styles.messageContent}>{item.content}</Text>
      </View>
    );
  };

  return (
    <View style={styles.container}>
      <FlatList
        data={messages}
        renderItem={renderMessage}
        keyExtractor={(_, index) => index.toString()}
        style={styles.messageList}
      />
      <View style={styles.inputContainer}>
        <TextInput
          style={styles.input}
          value={inputText}
          onChangeText={setInputText}
          placeholder="发送消息..."
          maxLength={100}
        />
        <TouchableOpacity style={styles.sendButton} onPress={handleSend}>
          <Text style={styles.sendButtonText}>发送</Text>
        </TouchableOpacity>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1 },
  messageList: { flex: 1, padding: 10 },
  chatMessage: { flexDirection: 'row', marginBottom: 8 },
  senderName: { color: '#FFD700', fontWeight: 'bold', marginRight: 8 },
  messageContent: { color: '#FFF', flex: 1 },
  giftMessage: { backgroundColor: 'rgba(255,215,0,0.2)', padding: 8, borderRadius: 4, marginBottom: 8 },
  giftText: { color: '#FFD700' },
  inputContainer: { flexDirection: 'row', padding: 10, borderTopWidth: 1, borderTopColor: '#333' },
  input: { flex: 1, backgroundColor: '#222', borderRadius: 20, paddingHorizontal: 15, color: '#FFF' },
  sendButton: { marginLeft: 10, backgroundColor: '#FF4757', borderRadius: 20, paddingHorizontal: 20, justifyContent: 'center' },
  sendButtonText: { color: '#FFF', fontWeight: 'bold' },
});
```

### 弹幕组件

```typescript
import React, { useState, useEffect, useRef } from 'react';
import { View, Text, Animated, StyleSheet, Dimensions } from 'react-native';

const { width: SCREEN_WIDTH } = Dimensions.get('window');

export function DanmakuOverlay({ room }: { room: Room }) {
  const [danmakus, setDanmakus] = useState<any[]>([]);
  const trackRef = useRef(0);

  useEffect(() => {
    const chatService = new LiveChatService(room);
    
    chatService.onMessage('danmaku', (msg) => {
      const id = `${Date.now()}-${Math.random()}`;
      const translateX = new Animated.Value(SCREEN_WIDTH);
      const top = (trackRef.current % 10) * 30 + 50;
      trackRef.current++;

      setDanmakus(prev => [...prev, { id, content: msg.content, color: msg.color, translateX, top }]);

      Animated.timing(translateX, {
        toValue: -500,
        duration: 8000,
        useNativeDriver: true,
      }).start(() => {
        setDanmakus(prev => prev.filter(d => d.id !== id));
      });
    });
  }, [room]);

  return (
    <View style={StyleSheet.absoluteFill} pointerEvents="none">
      {danmakus.map(d => (
        <Animated.Text
          key={d.id}
          style={[styles.danmaku, { color: d.color, top: d.top, transform: [{ translateX: d.translateX }] }]}
        >
          {d.content}
        </Animated.Text>
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  danmaku: {
    position: 'absolute',
    fontSize: 16,
    fontWeight: 'bold',
    textShadowColor: '#000',
    textShadowOffset: { width: 1, height: 1 },
    textShadowRadius: 2,
  },
});
```

### 消息限流

```typescript
class RateLimiter {
  private timestamps: number[] = [];
  
  constructor(private limit = 5, private windowMs = 1000) {}

  canSend(): boolean {
    const now = Date.now();
    this.timestamps = this.timestamps.filter(t => now - t < this.windowMs);
    if (this.timestamps.length >= this.limit) return false;
    this.timestamps.push(now);
    return true;
  }
}

// 使用: 每秒最多5条消息
const rateLimiter = new RateLimiter(5, 1000);

async function sendChat(content: string) {
  if (!rateLimiter.canSend()) {
    alert('发送太频繁');
    return;
  }
  await chatService.sendChat(content);
}
```

### 聊天功能特性

| 功能 | 实现 | 说明 |
|------|------|------|
| 实时聊天 | DataChannel | 毫秒级延迟 |
| 弹幕 | DataChannel + 动画 | 滚动/顶部/底部 |
| 礼物通知 | 链上事件 + DataChannel | 链上打赏后广播 |
| 点赞 | DataChannel | 轻量级互动 |
| 禁言 | 链上记录 + 前端过滤 | 被禁言用户无法发送 |

### 成本

**聊天功能零额外成本** - DataChannel 包含在 LiveKit 视频流费用中，不单独计费。


---

## 连麦功能（简化版）

> **设计变更**: 连麦申请/同意/拒绝流程全部移至链下（通过 DataChannel），链上只记录连麦开始和结束状态。这样可以避免频繁的链上交易，提升用户体验。

LiveKit 原生支持多人同时推流，非常适合实现连麦功能。

### 连麦模式

```
┌─────────────────────────────────────────────────────────────────┐
│                        连麦模式                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. 语音连麦 (Audio Only)                                        │
│     主播视频 + 连麦者语音                                         │
│     适合: 聊天、问答、电台                                        │
│                                                                  │
│  2. 视频连麦 (Video + Audio)                                     │
│     主播视频 + 连麦者视频 (画中画/分屏)                           │
│     适合: 访谈、PK、多人互动                                      │
│                                                                  │
│  3. 多人连麦 (Multi-Guest)                                       │
│     最多支持 4 人同时连麦                                         │
│     适合: 圆桌讨论、多人游戏                                      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 连麦流程（链下申请 + 链上记录）

```
┌─────────┐          ┌─────────┐          ┌─────────┐          ┌─────────┐
│  观众   │          │  主播   │          │ LiveKit │          │Substrate│
└────┬────┘          └────┬────┘          └────┬────┘          └────┬────┘
     │                    │                    │                    │
     │ 1. 申请连麦 (DataChannel)               │                    │
     │───────────────────→│                    │                    │
     │                    │                    │                    │
     │ 2. 同意连麦 (DataChannel)               │                    │
     │←───────────────────│                    │                    │
     │                    │                    │                    │
     │ 3. 请求连麦者 Token │                    │                    │
     │────────────────────────────────────────→│                    │
     │                    │                    │                    │
     │ 4. 开启摄像头/麦克风 │                    │                    │
     │────────────────────────────────────────→│                    │
     │                    │                    │                    │
     │                    │ 5. 记录连麦开始 (可选)                   │
     │                    │───────────────────────────────────────→│
     │                    │                    │                    │
     │←────────────────── 6. 连麦中 ──────────→│                    │
     │                    │                    │                    │
```

### 链上类型（简化版）

```rust
/// 连麦类型
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum CoHostType {
    /// 语音连麦
    AudioOnly,
    /// 视频连麦
    VideoAndAudio,
}
```

> **已删除的类型**: `CoHostStatus`、`CoHostRequest`、`CoHost` 结构体 - 这些信息改为链下管理。

### 链上存储（简化版）

```rust
/// 当前连麦者列表（仅记录账户，详细信息链下管理）
#[pallet::storage]
pub type ActiveCoHosts<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64,  // room_id
    BoundedVec<T::AccountId, T::MaxCoHostsPerRoom>,
    ValueQuery,
>;
```

> **已删除的存储**: `CoHostRequests`、`MaxCoHostsPerRoom` StorageValue - 申请队列改为链下管理，最大人数改为 Config 常量。

### 链上调用函数（简化版）

```rust
/// 开始连麦（主播调用，在链下同意后记录到链上）
#[pallet::call_index(50)]
pub fn start_co_host(
    origin: OriginFor<T>,
    room_id: u64,
    co_host: T::AccountId,
) -> DispatchResult {
    let host = ensure_signed(origin)?;

    // 1. 验证是房主
    let room = LiveRooms::<T>::get(room_id).ok_or(Error::<T>::RoomNotFound)?;
    ensure!(room.host == host, Error::<T>::NotRoomHost);
    ensure!(room.status == LiveRoomStatus::Live, Error::<T>::RoomNotLive);

    // 2. 检查连麦人数限制
    ActiveCoHosts::<T>::try_mutate(room_id, |co_hosts| {
        ensure!(co_hosts.len() < T::MaxCoHostsPerRoom::get() as usize, Error::<T>::TooManyCoHosts);
        ensure!(!co_hosts.contains(&co_host), Error::<T>::AlreadyCoHost);
        co_hosts.try_push(co_host.clone()).map_err(|_| Error::<T>::TooManyCoHosts)?;
        Ok::<(), DispatchError>(())
    })?;

    // 3. 发出事件
    Self::deposit_event(Event::CoHostStarted { room_id, co_host });

    Ok(())
}

/// 结束连麦（主播或连麦者调用）
#[pallet::call_index(51)]
pub fn end_co_host(
    origin: OriginFor<T>,
    room_id: u64,
    co_host: Option<T::AccountId>,
) -> DispatchResult {
    let caller = ensure_signed(origin)?;

    let room = LiveRooms::<T>::get(room_id).ok_or(Error::<T>::RoomNotFound)?;

    // 确定要移除的连麦者
    let target = co_host.unwrap_or(caller.clone());

    // 验证权限：房主可以移除任何人，连麦者只能移除自己
    if caller != room.host {
        ensure!(caller == target, Error::<T>::NotAuthorized);
    }

    // 移除连麦者
    ActiveCoHosts::<T>::try_mutate(room_id, |co_hosts| {
        let pos = co_hosts.iter().position(|x| x == &target).ok_or(Error::<T>::NotCoHost)?;
        co_hosts.remove(pos);
        Ok::<(), DispatchError>(())
    })?;

    // 发出事件
    Self::deposit_event(Event::CoHostEnded { room_id, co_host: target, duration: 0 });

    Ok(())
}
```

> **已删除的函数**: `request_co_host`、`accept_co_host`、`reject_co_host`、`mute_co_host`、`cancel_co_host_request` - 这些操作改为链下处理。

### 前端实现

#### 连麦服务

```typescript
import { Room, RoomEvent, Track, Participant } from 'livekit-client';

interface CoHostInfo {
  identity: string;
  name: string;
  type: 'audio' | 'video';
  videoTrack?: Track;
  audioTrack?: Track;
}

class CoHostService {
  private room: Room;
  private coHosts: Map<string, CoHostInfo> = new Map();
  private onCoHostChange?: (coHosts: CoHostInfo[]) => void;

  constructor(room: Room) {
    this.room = room;
    this.setupListeners();
  }

  private setupListeners() {
    // 监听新参与者 (连麦者加入)
    this.room.on(RoomEvent.ParticipantConnected, (participant: Participant) => {
      // 检查是否是连麦者 (通过 metadata 判断)
      if (participant.metadata?.includes('co_host')) {
        this.addCoHost(participant);
      }
    });

    // 监听参与者离开
    this.room.on(RoomEvent.ParticipantDisconnected, (participant: Participant) => {
      this.removeCoHost(participant.identity);
    });

    // 监听轨道订阅
    this.room.on(RoomEvent.TrackSubscribed, (track, publication, participant) => {
      const coHost = this.coHosts.get(participant.identity);
      if (coHost) {
        if (track.kind === Track.Kind.Video) {
          coHost.videoTrack = track;
        } else if (track.kind === Track.Kind.Audio) {
          coHost.audioTrack = track;
        }
        this.notifyChange();
      }
    });
  }

  private addCoHost(participant: Participant) {
    const metadata = JSON.parse(participant.metadata || '{}');
    this.coHosts.set(participant.identity, {
      identity: participant.identity,
      name: participant.name || '连麦用户',
      type: metadata.coHostType || 'audio',
    });
    this.notifyChange();
  }

  private removeCoHost(identity: string) {
    this.coHosts.delete(identity);
    this.notifyChange();
  }

  private notifyChange() {
    this.onCoHostChange?.(Array.from(this.coHosts.values()));
  }

  // 设置连麦者变化回调
  setOnCoHostChange(callback: (coHosts: CoHostInfo[]) => void) {
    this.onCoHostChange = callback;
  }

  // 获取当前连麦者列表
  getCoHosts(): CoHostInfo[] {
    return Array.from(this.coHosts.values());
  }
}
```

#### 观众申请连麦

```typescript
class CoHostApplicant {
  private room: Room;
  private chatService: LiveChatService;

  constructor(room: Room) {
    this.room = room;
    this.chatService = new LiveChatService(room);
  }

  // 申请连麦 (通过 DataChannel 发送请求)
  async requestCoHost(type: 'audio' | 'video') {
    await this.chatService.sendMessage({
      type: 'system',
      content: JSON.stringify({
        action: 'co_host_request',
        coHostType: type,
      }),
      sender: this.room.localParticipant.identity,
      senderName: this.room.localParticipant.name || '',
      timestamp: Date.now(),
    });
  }

  // 收到主播同意后，开启摄像头/麦克风
  async startCoHost(type: 'audio' | 'video') {
    if (type === 'video') {
      await this.room.localParticipant.enableCameraAndMicrophone();
    } else {
      await this.room.localParticipant.setMicrophoneEnabled(true);
    }

    // 更新 metadata 标记为连麦者
    await this.room.localParticipant.setMetadata(JSON.stringify({
      co_host: true,
      coHostType: type,
    }));
  }

  // 结束连麦
  async endCoHost() {
    await this.room.localParticipant.setCameraEnabled(false);
    await this.room.localParticipant.setMicrophoneEnabled(false);
    await this.room.localParticipant.setMetadata('{}');
  }
}
```

#### 主播管理连麦

```typescript
class CoHostManager {
  private room: Room;
  private chatService: LiveChatService;
  private pendingRequests: Map<string, { type: 'audio' | 'video'; timestamp: number }> = new Map();

  constructor(room: Room) {
    this.room = room;
    this.chatService = new LiveChatService(room);
    this.listenForRequests();
  }

  private listenForRequests() {
    this.chatService.onMessage('system', (msg) => {
      try {
        const data = JSON.parse(msg.content);
        if (data.action === 'co_host_request') {
          this.pendingRequests.set(msg.sender, {
            type: data.coHostType,
            timestamp: msg.timestamp,
          });
          this.onRequestReceived?.(msg.sender, msg.senderName, data.coHostType);
        }
      } catch {}
    });
  }

  private onRequestReceived?: (identity: string, name: string, type: 'audio' | 'video') => void;

  // 设置收到申请的回调
  setOnRequestReceived(callback: (identity: string, name: string, type: 'audio' | 'video') => void) {
    this.onRequestReceived = callback;
  }

  // 获取待处理的连麦申请
  getPendingRequests() {
    return Array.from(this.pendingRequests.entries()).map(([identity, data]) => ({
      identity,
      ...data,
    }));
  }

  // 同意连麦
  async acceptCoHost(identity: string) {
    const request = this.pendingRequests.get(identity);
    if (!request) return;

    // 通知申请者
    await this.chatService.sendMessage({
      type: 'system',
      content: JSON.stringify({
        action: 'co_host_accepted',
        target: identity,
        coHostType: request.type,
      }),
      sender: 'host',
      senderName: '主播',
      timestamp: Date.now(),
    });

    this.pendingRequests.delete(identity);
  }

  // 拒绝连麦
  async rejectCoHost(identity: string) {
    await this.chatService.sendMessage({
      type: 'system',
      content: JSON.stringify({
        action: 'co_host_rejected',
        target: identity,
      }),
      sender: 'host',
      senderName: '主播',
      timestamp: Date.now(),
    });

    this.pendingRequests.delete(identity);
  }

  // 结束某人的连麦
  async endCoHost(identity: string) {
    await this.chatService.sendMessage({
      type: 'system',
      content: JSON.stringify({
        action: 'co_host_ended',
        target: identity,
      }),
      sender: 'host',
      senderName: '主播',
      timestamp: Date.now(),
    });
  }

  // 静音连麦者
  async muteCoHost(identity: string, muted: boolean) {
    await this.chatService.sendMessage({
      type: 'system',
      content: JSON.stringify({
        action: 'co_host_mute',
        target: identity,
        muted,
      }),
      sender: 'host',
      senderName: '主播',
      timestamp: Date.now(),
    });
  }
}
```

#### 连麦布局组件

```typescript
import React from 'react';
import { View, StyleSheet, Dimensions } from 'react-native';
import { VideoView } from '@livekit/react-native';

const { width: SCREEN_WIDTH, height: SCREEN_HEIGHT } = Dimensions.get('window');

interface CoHostLayoutProps {
  hostTrack: Track;
  coHosts: CoHostInfo[];
  layout: 'pip' | 'split' | 'grid';  // 画中画 / 分屏 / 网格
}

export function CoHostLayout({ hostTrack, coHosts, layout }: CoHostLayoutProps) {
  // 画中画模式: 主播大画面 + 连麦者小窗
  if (layout === 'pip') {
    return (
      <View style={styles.container}>
        <VideoView style={styles.fullScreen} videoTrack={hostTrack} />
        <View style={styles.pipContainer}>
          {coHosts.map((coHost, index) => (
            coHost.videoTrack && (
              <VideoView
                key={coHost.identity}
                style={[styles.pipWindow, { bottom: 100 + index * 120 }]}
                videoTrack={coHost.videoTrack}
              />
            )
          ))}
        </View>
      </View>
    );
  }

  // 分屏模式: 左右/上下分屏
  if (layout === 'split') {
    const firstCoHost = coHosts[0];
    return (
      <View style={styles.splitContainer}>
        <VideoView style={styles.splitHalf} videoTrack={hostTrack} />
        {firstCoHost?.videoTrack && (
          <VideoView style={styles.splitHalf} videoTrack={firstCoHost.videoTrack} />
        )}
      </View>
    );
  }

  // 网格模式: 多人平分画面
  const allTracks = [hostTrack, ...coHosts.filter(c => c.videoTrack).map(c => c.videoTrack!)];
  const gridSize = Math.ceil(Math.sqrt(allTracks.length));
  
  return (
    <View style={styles.gridContainer}>
      {allTracks.map((track, index) => (
        <VideoView
          key={index}
          style={[styles.gridItem, { width: `${100 / gridSize}%`, height: `${100 / gridSize}%` }]}
          videoTrack={track}
        />
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1 },
  fullScreen: { width: SCREEN_WIDTH, height: SCREEN_HEIGHT },
  pipContainer: { position: 'absolute', right: 16 },
  pipWindow: { width: 100, height: 150, borderRadius: 8, overflow: 'hidden' },
  splitContainer: { flex: 1, flexDirection: 'row' },
  splitHalf: { flex: 1 },
  gridContainer: { flex: 1, flexDirection: 'row', flexWrap: 'wrap' },
  gridItem: { aspectRatio: 16 / 9 },
});
```

### 连麦功能配置

```rust
// runtime 配置（简化版）
parameter_types! {
    pub const MaxCoHostsPerRoom: u32 = 4;  // 最多4人连麦
}

// 已删除的配置:
// - CoHostRequestTimeout: 申请流程移至链下，无需超时配置
// - MinCoHostDuration: 链下管理，无需链上限制
```

### 连麦功能特性

| 功能 | 实现位置 | 说明 |
|------|----------|------|
| 语音连麦 | 链下 | 仅音频，带宽占用低 |
| 视频连麦 | 链下 | 音视频，画中画/分屏显示 |
| 多人连麦 | 链上限制 | 最多4人同时连麦 |
| 申请/同意/拒绝 | **链下 DataChannel** | 无需链上交易，即时响应 |
| 开始/结束记录 | 链上 | 用于统计和事件记录 |
| 静音控制 | 链下 LiveKit | 通过 LiveKit 权限控制 |
| 布局切换 | 链下前端 | 画中画/分屏/网格 |

### 成本

连麦功能使用 LiveKit 多人房间能力，按参与者计费：
- 主播 + 1连麦者 + 100观众 = 102人 × 60分钟 × $0.01 = $61.2/小时
- 相比无连麦仅增加连麦者的费用

---

## 设计修改总结

本文档经过优化，实施了以下核心修改：

### 1. 观众管理移至链下

| 修改项 | 原设计 | 新设计 |
|--------|--------|--------|
| 观众列表 | `RoomViewers` 链上存储 | LiveKit 管理 |
| 进入/离开 | `join_room`/`leave_room` extrinsic | 直接连接/断开 LiveKit |
| 观众计数 | 链上 `viewer_count` | LiveKit API 实时获取 |
| 禁言功能 | 链上 `mute_viewer` | LiveKit 权限控制 |

**收益**: 避免 10000 观众存储爆炸，减少链上交易费用

### 2. 删除 stream_key，改用签名验证

| 修改项 | 原设计 | 新设计 |
|--------|--------|--------|
| 推流认证 | 链上存储 `stream_key` | 私钥签名验证 |
| 安全性 | 任何人可读取 key | 只有私钥持有者可推流 |
| Token 生成 | 简单生成 | 验证签名 + 查询链上房主 |

**收益**: 防止冒充主播推流的安全漏洞

### 3. 简化链上存储，用事件替代详细记录

| 修改项 | 原设计 | 新设计 |
|--------|--------|--------|
| 礼物记录 | `GiftRecords` 存储 | `GiftSent` 事件 |
| 历史查询 | 链上遍历 | 索引器监听事件 |
| room_id | 随机数生成 | 自增 `NextRoomId` |

**收益**: 减少链上存储成本，提高查询效率

### 4. 连麦申请流程移至链下

| 修改项 | 原设计 | 新设计 |
|--------|--------|--------|
| 申请连麦 | `request_co_host` extrinsic | DataChannel 消息 |
| 同意/拒绝 | `accept_co_host`/`reject_co_host` | DataChannel 消息 |
| 静音控制 | `mute_co_host` extrinsic | LiveKit 权限控制 |
| 链上记录 | 完整状态 | 仅开始/结束事件 |

**收益**: 连麦操作即时响应，无需等待区块确认

### 链上/链下职责划分

```
┌─────────────────────────────────────────────────────────────────┐
│                         链上 (Substrate)                         │
├─────────────────────────────────────────────────────────────────┤
│  ✓ 直播间创建/开播/结束                                          │
│  ✓ 礼物打赏（资金转账）                                          │
│  ✓ 付费直播门票购买                                              │
│  ✓ 主播收益提现                                                  │
│  ✓ 黑名单管理                                                    │
│  ✓ 连麦开始/结束记录                                             │
│  ✓ 直播统计数据同步                                              │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                         链下 (LiveKit)                           │
├─────────────────────────────────────────────────────────────────┤
│  ✓ 视频/音频流传输                                               │
│  ✓ 观众进入/离开                                                 │
│  ✓ 实时观众计数                                                  │
│  ✓ 聊天/弹幕 (DataChannel)                                       │
│  ✓ 连麦申请/同意/拒绝                                            │
│  ✓ 禁言/静音控制                                                 │
│  ✓ 房管权限管理                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 删除的链上组件汇总

**存储**:
- `RoomViewers` - 观众列表
- `GiftRecords` - 礼物记录
- `CoHostRequests` - 连麦申请队列

**Extrinsics**:
- `join_room` / `leave_room` - 观众进出
- `mute_viewer` / `unmute_viewer` - 禁言
- `set_moderator` - 设置房管
- `request_co_host` / `accept_co_host` / `reject_co_host` - 连麦申请流程
- `mute_co_host` / `cancel_co_host_request` - 连麦管理

**配置**:
- `MaxViewersPerRoom` - 观众数限制
- `MaxStreamKeyLen` - 流密钥长度
- `Randomness` - 随机数生成器
- `CoHostRequestTimeout` / `MinCoHostDuration` - 连麦超时配置

**类型**:
- `Viewer` 结构体
- `CoHostStatus` / `CoHostRequest` / `CoHost` 结构体
