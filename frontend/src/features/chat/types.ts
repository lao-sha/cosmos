/**
 * 聊天模块类型定义
 * 与后端 pallet-chat 对应
 */

/**
 * 聊天用户ID - 11位数字（类似QQ号）
 * 范围：10000000000 - 99999999999
 */
export type ChatUserId = number;

/**
 * 消息类型
 */
export enum MessageType {
  Text = 0,
  Image = 1,
  File = 2,
  Voice = 3,
  System = 4,
}

/**
 * 用户在线状态
 */
export enum UserStatus {
  Online = 'Online',
  Offline = 'Offline',
  Busy = 'Busy',
  Away = 'Away',
  Invisible = 'Invisible',
}

/**
 * 消息发送状态（用于乐观更新）
 */
export type MessageSendStatus = 'sending' | 'sent' | 'failed';

/**
 * 隐私设置
 */
export interface PrivacySettings {
  /** 是否允许陌生人发送消息 */
  allowStrangerMessages: boolean;
  /** 是否显示在线状态 */
  showOnlineStatus: boolean;
  /** 是否显示最后活跃时间 */
  showLastActive: boolean;
}

/**
 * 聊天用户资料
 */
export interface ChatUserProfile {
  /** 11位聊天ID */
  chatUserId: ChatUserId;
  /** 关联的链上地址 */
  accountId: string;
  /** 昵称 */
  nickname?: string;
  /** 头像 IPFS CID */
  avatarCid?: string;
  /** 个性签名 */
  signature?: string;
  /** 在线状态 */
  status: UserStatus;
  /** 隐私设置 */
  privacySettings: PrivacySettings;
  /** 创建时间戳 */
  createdAt: number;
  /** 最后活跃时间戳 */
  lastActive: number;
  /** x25519 公钥（用于端到端加密） */
  encryptionPublicKey?: string;
}

/**
 * 消息
 */
export interface Message {
  /** 消息ID（链上ID或临时ID） */
  id: number | string;
  /** 临时ID（用于乐观更新） */
  tempId?: string;
  /** 会话ID */
  sessionId: string;
  /** 发送方地址 */
  sender: string;
  /** 接收方地址 */
  receiver: string;
  /** 发送方聊天ID（用于显示） */
  senderChatId?: ChatUserId;
  /** 接收方聊天ID（用于显示） */
  receiverChatId?: ChatUserId;
  /** 解密后的内容 */
  content: string;
  /** IPFS CID */
  contentCid: string;
  /** 消息类型 */
  msgType: MessageType;
  /** 发送时间（区块高度） */
  sentAt: number;
  /** 是否已读 */
  isRead: boolean;
  /** 是否被发送方删除 */
  isDeletedBySender: boolean;
  /** 是否被接收方删除 */
  isDeletedByReceiver: boolean;
  /** 是否是自己发的 */
  isMine: boolean;
  /** 发送状态 */
  status: MessageSendStatus;
  /** 错误信息（发送失败时） */
  error?: string;
  /** 重试次数 */
  retryCount?: number;
  /** 回复的消息ID */
  replyTo?: number;
}

/**
 * 会话
 */
export interface Session {
  /** 会话ID */
  id: string;
  /** 参与者地址列表 */
  participants: string[];
  /** 对方地址 */
  peerAddress: string;
  /** 对方聊天ID */
  peerChatId?: ChatUserId;
  /** 对方昵称/备注 */
  peerAlias?: string;
  /** 对方用户资料 */
  peerProfile?: ChatUserProfile;
  /** 最后一条消息 */
  lastMessage?: Message;
  /** 最后活跃时间（区块高度） */
  lastActive: number;
  /** 未读消息数 */
  unreadCount: number;
  /** 是否已归档 */
  isArchived: boolean;
  /** 创建时间（区块高度） */
  createdAt: number;
}

/**
 * 聊天用户（简化版，用于列表显示）
 */
export interface ChatUser {
  /** 链上地址 */
  address: string;
  /** 聊天ID */
  chatUserId?: ChatUserId;
  /** 用户资料 */
  profile?: ChatUserProfile;
  /** 是否被拉黑 */
  isBlocked: boolean;
}

/**
 * 链上事件类型
 */
export interface ChatEvents {
  MessageSent: {
    msgId: number;
    sessionId: string;
    sender: string;
    receiver: string;
  };
  MessageSentWithChatId: {
    msgId: number;
    senderChatId: ChatUserId | null;
    receiverChatId: ChatUserId | null;
    contentCid: string;
  };
  MessageRead: {
    msgId: number;
    reader: string;
  };
  MessageDeleted: {
    msgId: number;
    deleter: string;
  };
  SessionCreated: {
    sessionId: string;
    participants: string[];
  };
  SessionMarkedAsRead: {
    sessionId: string;
    user: string;
  };
  SessionArchived: {
    sessionId: string;
    operator: string;
  };
  UserBlocked: {
    blocker: string;
    blocked: string;
  };
  UserUnblocked: {
    unblocker: string;
    unblocked: string;
  };
  ChatUserCreated: {
    accountId: string;
    chatUserId: ChatUserId;
  };
  ChatUserProfileUpdated: {
    chatUserId: ChatUserId;
  };
  ChatUserStatusChanged: {
    chatUserId: ChatUserId;
    newStatus: number;
  };
  PrivacySettingsUpdated: {
    chatUserId: ChatUserId;
  };
}

/**
 * 错误代码
 */
export enum ChatErrorCode {
  NETWORK_ERROR = 'NETWORK_ERROR',
  ENCRYPTION_FAILED = 'ENCRYPTION_FAILED',
  DECRYPTION_FAILED = 'DECRYPTION_FAILED',
  IPFS_UPLOAD_FAILED = 'IPFS_UPLOAD_FAILED',
  IPFS_DOWNLOAD_FAILED = 'IPFS_DOWNLOAD_FAILED',
  TRANSACTION_FAILED = 'TRANSACTION_FAILED',
  RATE_LIMIT_EXCEEDED = 'RATE_LIMIT_EXCEEDED',
  USER_BLOCKED = 'USER_BLOCKED',
  STRANGER_NOT_ALLOWED = 'STRANGER_NOT_ALLOWED',
  SESSION_NOT_FOUND = 'SESSION_NOT_FOUND',
  MESSAGE_NOT_FOUND = 'MESSAGE_NOT_FOUND',
}

/**
 * 错误消息映射
 */
export const ChatErrorMessages: Record<ChatErrorCode, string> = {
  [ChatErrorCode.NETWORK_ERROR]: '网络连接失败，请检查网络设置',
  [ChatErrorCode.ENCRYPTION_FAILED]: '消息加密失败',
  [ChatErrorCode.DECRYPTION_FAILED]: '消息解密失败',
  [ChatErrorCode.IPFS_UPLOAD_FAILED]: '内容上传失败，请重试',
  [ChatErrorCode.IPFS_DOWNLOAD_FAILED]: '内容下载失败',
  [ChatErrorCode.TRANSACTION_FAILED]: '发送失败，请重试',
  [ChatErrorCode.RATE_LIMIT_EXCEEDED]: '发送过于频繁，请稍后再试',
  [ChatErrorCode.USER_BLOCKED]: '对方已将您拉黑',
  [ChatErrorCode.STRANGER_NOT_ALLOWED]: '对方不接收陌生人消息',
  [ChatErrorCode.SESSION_NOT_FOUND]: '会话不存在',
  [ChatErrorCode.MESSAGE_NOT_FOUND]: '消息不存在',
};
