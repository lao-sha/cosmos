/**
 * 通讯录模块类型定义
 * 与后端 pallet-contacts 对应
 */

import type { ChatUserId, ChatUserProfile } from '@/features/chat/types';

/**
 * 好友关系状态
 */
export enum FriendStatus {
  /** 单向添加（仅我添加了对方） */
  OneWay = 0,
  /** 双向好友（互相添加） */
  Mutual = 1,
  /** 待确认（已发送好友申请） */
  Pending = 2,
}

/**
 * 联系人信息
 */
export interface Contact {
  /** 联系人地址 */
  address: string;
  /** 联系人 ChatUserId */
  chatUserId?: ChatUserId;
  /** 备注名 */
  alias?: string;
  /** 所属分组 */
  groups: string[];
  /** 好友状态 */
  friendStatus: FriendStatus;
  /** 添加时间（区块高度） */
  addedAt: number;
  /** 最后更新时间（区块高度） */
  updatedAt: number;
  /** 用户资料（缓存） */
  profile?: ChatUserProfile;
}

/**
 * 分组信息
 */
export interface ContactGroup {
  /** 分组名称 */
  name: string;
  /** 成员数量 */
  memberCount: number;
  /** 创建时间（区块高度） */
  createdAt: number;
}

/**
 * 黑名单记录
 */
export interface BlockedUser {
  /** 被屏蔽的地址 */
  address: string;
  /** 被屏蔽用户的 ChatUserId */
  chatUserId?: ChatUserId;
  /** 屏蔽原因 */
  reason?: string;
  /** 屏蔽时间（区块高度） */
  blockedAt: number;
  /** 用户资料（缓存） */
  profile?: ChatUserProfile;
}

/**
 * 好友申请
 */
export interface FriendRequest {
  /** 申请者地址 */
  requester: string;
  /** 申请者 ChatUserId */
  requesterChatId?: ChatUserId;
  /** 申请留言 */
  message?: string;
  /** 申请时间（区块高度） */
  requestedAt: number;
  /** 过期时间（区块高度） */
  expiresAt: number;
  /** 是否已过期 */
  isExpired: boolean;
  /** 申请者资料 */
  profile?: ChatUserProfile;
}

/**
 * 通讯录统计
 */
export interface ContactsStats {
  /** 联系人总数 */
  contactCount: number;
  /** 分组总数 */
  groupCount: number;
  /** 黑名单数量 */
  blacklistCount: number;
  /** 待处理好友申请数量 */
  pendingRequestCount: number;
}

/**
 * 通讯录事件类型
 */
export interface ContactsEvents {
  ContactAdded: {
    owner: string;
    contact: string;
  };
  ContactRemoved: {
    owner: string;
    contact: string;
  };
  ContactUpdated: {
    owner: string;
    contact: string;
  };
  GroupCreated: {
    owner: string;
    groupName: string;
  };
  GroupDeleted: {
    owner: string;
    groupName: string;
  };
  GroupRenamed: {
    owner: string;
    oldName: string;
    newName: string;
  };
  UserBlocked: {
    blocker: string;
    blocked: string;
  };
  UserUnblocked: {
    unblocker: string;
    unblocked: string;
  };
  FriendRequestSent: {
    sender: string;
    receiver: string;
  };
  FriendRequestAccepted: {
    accepter: string;
    requester: string;
  };
  FriendRequestRejected: {
    rejecter: string;
    requester: string;
  };
  FriendStatusChanged: {
    account1: string;
    account2: string;
    isMutual: boolean;
  };
}

/**
 * 好友申请有效期（区块数）
 * 约 7 天（假设 6 秒出块）
 */
export const FRIEND_REQUEST_EXPIRY_BLOCKS = 100800;

/**
 * 通讯录错误代码
 */
export enum ContactsErrorCode {
  CONTACT_NOT_FOUND = 'CONTACT_NOT_FOUND',
  CONTACT_ALREADY_EXISTS = 'CONTACT_ALREADY_EXISTS',
  GROUP_NOT_FOUND = 'GROUP_NOT_FOUND',
  GROUP_ALREADY_EXISTS = 'GROUP_ALREADY_EXISTS',
  CANNOT_ADD_SELF = 'CANNOT_ADD_SELF',
  CANNOT_BLOCK_SELF = 'CANNOT_BLOCK_SELF',
  REQUEST_EXPIRED = 'REQUEST_EXPIRED',
  REQUEST_NOT_FOUND = 'REQUEST_NOT_FOUND',
  ALREADY_FRIENDS = 'ALREADY_FRIENDS',
}

/**
 * 通讯录错误消息映射
 */
export const ContactsErrorMessages: Record<ContactsErrorCode, string> = {
  [ContactsErrorCode.CONTACT_NOT_FOUND]: '联系人不存在',
  [ContactsErrorCode.CONTACT_ALREADY_EXISTS]: '联系人已存在',
  [ContactsErrorCode.GROUP_NOT_FOUND]: '分组不存在',
  [ContactsErrorCode.GROUP_ALREADY_EXISTS]: '分组已存在',
  [ContactsErrorCode.CANNOT_ADD_SELF]: '不能添加自己为联系人',
  [ContactsErrorCode.CANNOT_BLOCK_SELF]: '不能拉黑自己',
  [ContactsErrorCode.REQUEST_EXPIRED]: '好友申请已过期',
  [ContactsErrorCode.REQUEST_NOT_FOUND]: '好友申请不存在',
  [ContactsErrorCode.ALREADY_FRIENDS]: '已经是好友了',
};
