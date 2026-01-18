// frontend/src/features/livestream/types.ts

/** 直播间状态 */
export enum LiveRoomStatus {
  Preparing = 'Preparing',   // 准备中
  Live = 'Live',             // 直播中
  Paused = 'Paused',         // 暂停中
  Ended = 'Ended',           // 已结束
  Banned = 'Banned',         // 被封禁
}

/** 直播间类型 */
export enum LiveRoomType {
  Normal = 'Normal',         // 普通直播
  Paid = 'Paid',             // 付费直播
  Private = 'Private',       // 私密直播
  MultiHost = 'MultiHost',   // 连麦直播
}

/** 直播间信息 */
export interface LiveRoom {
  id: number;
  host: string;              // 主播地址
  hostName?: string;         // 主播昵称
  hostAvatar?: string;       // 主播头像
  title: string;
  description?: string;
  roomType: LiveRoomType;
  status: LiveRoomStatus;
  coverCid?: string;         // 封面 IPFS CID
  totalViewers: number;      // 累计观众
  peakViewers: number;       // 峰值观众
  currentViewers: number;    // 当前观众 (链下)
  totalGifts: string;        // 累计礼物收入
  ticketPrice?: string;      // 门票价格
  createdAt: number;
  startedAt?: number;
  endedAt?: number;
}

/** 礼物定义 */
export interface Gift {
  id: number;
  name: string;
  price: string;
  iconCid: string;
  enabled: boolean;
}

/** 礼物记录 (事件) */
export interface GiftRecord {
  sender: string;
  senderName?: string;
  receiver: string;
  roomId: number;
  giftId: number;
  giftName: string;
  quantity: number;
  totalValue: string;
  timestamp: number;
}

/** 连麦者信息 */
export interface CoHost {
  address: string;
  name?: string;
  avatar?: string;
  type: 'audio' | 'video';
  isMuted: boolean;
}

/** 连麦申请 */
export interface CoHostRequest {
  address: string;
  name?: string;
  type: 'audio' | 'video';
  timestamp: number;
}

/** 弹幕消息 */
export interface DanmakuMessage {
  id: string;
  content: string;
  sender: string;
  senderName?: string;
  color: string;
  position: 'scroll' | 'top' | 'bottom';
  timestamp: number;
}

/** 聊天消息 */
export interface LiveChatMessage {
  id: string;
  type: 'chat' | 'danmaku' | 'gift' | 'system' | 'like';
  content: string;
  sender: string;
  senderName: string;
  timestamp: number;
  giftId?: number;
  giftCount?: number;
  giftValue?: string;
  color?: string;
}

/** 观众信息 */
export interface Viewer {
  address: string;
  name: string;
  avatar?: string;
  level: number;
  joinedAt: number;
}

/** 创建直播间参数 */
export interface CreateRoomParams {
  title: string;
  description?: string;
  roomType: LiveRoomType;
  coverCid?: string;
  ticketPrice?: string;
}

/** 直播间筛选类型 */
export type RoomFilter = 'all' | 'Normal' | 'Paid' | 'MultiHost' | 'Private';

/** DataChannel 消息类型 */
export type DataChannelMessageType =
  | 'chat'
  | 'danmaku'
  | 'gift_notification'
  | 'cohost_request'
  | 'cohost_accept'
  | 'cohost_reject'
  | 'cohost_end'
  | 'viewer_join'
  | 'viewer_leave'
  | 'like';

/** DataChannel 消息 */
export interface DataChannelMessage {
  type: DataChannelMessageType;
  payload: unknown;
  sender: string;
  senderName?: string;
  timestamp: number;
}

/** 礼物动画项 */
export interface GiftAnimationItem {
  id: string;
  giftId: number;
  giftName: string;
  senderName: string;
  quantity: number;
  isFullScreen: boolean;
}
