/**
 * 聊天状态管理
 * 使用 Zustand 实现，支持乐观更新
 */

import { create } from 'zustand';
import { getChatService, initChatService } from '@/services/chat.service';
import type {
  Session,
  Message,
  ChatUserProfile,
  ChatUserId,
  UserStatus,
  PrivacySettings,
  MessageType,
} from '@/features/chat/types';

interface ChatState {
  // 状态
  sessions: Session[];
  currentSession: Session | null;
  messages: Record<string, Message[]>; // sessionId -> messages
  totalUnread: number;
  isLoading: boolean;
  error: string | null;
  blockedUsers: Set<string>;

  // 初始化
  initialize: (address: string) => Promise<void>;

  // 会话操作
  loadSessions: () => Promise<void>;
  selectSession: (sessionId: string) => void;
  archiveSession: (sessionId: string) => Promise<void>;

  // 消息操作
  loadMessages: (sessionId: string, offset?: number) => Promise<Message[]>;
  sendMessage: (
    receiver: string,
    content: string,
    msgType?: number
  ) => Promise<void>;
  retryMessage: (tempId: string) => Promise<void>;
  removeFailedMessage: (tempId: string) => void;
  markAsRead: (messageIds: number[]) => Promise<void>;
  markSessionAsRead: (sessionId: string) => Promise<void>;
  deleteMessage: (msgId: number) => Promise<void>;

  // 黑名单
  blockUser: (address: string) => Promise<void>;
  unblockUser: (address: string) => Promise<void>;

  // 事件处理
  handleNewMessage: (message: Message) => void;
  handleMessageRead: (msgId: number, reader: string) => void;
  handleMessageDeleted: (msgId: number) => void;
  handleSessionCreated: (session: Session) => void;
  handleUserBlocked: (blocker: string, blocked: string) => void;
  handleUserUnblocked: (unblocker: string, unblocked: string) => void;
}

export const useChatStore = create<ChatState>()((set, get) => ({
  sessions: [],
  currentSession: null,
  messages: {},
  totalUnread: 0,
  isLoading: false,
  error: null,
  blockedUsers: new Set(),

  initialize: async (address: string) => {
    set({ isLoading: true, error: null });

    try {
      const service = initChatService(address);
      await service.init();
      await get().loadSessions();

      // 订阅新消息
      service.subscribeMessages(get().handleNewMessage);
    } catch (error) {
      set({ error: (error as Error).message });
    } finally {
      set({ isLoading: false });
    }
  },

  loadSessions: async () => {
    set({ isLoading: true });
    try {
      const chatService = getChatService();
      const sessions = await chatService.getSessions();

      // 计算总未读数
      const totalUnread = sessions.reduce((sum, s) => sum + s.unreadCount, 0);

      set({ sessions, totalUnread, error: null });
    } catch (error) {
      set({ error: (error as Error).message });
    } finally {
      set({ isLoading: false });
    }
  },

  selectSession: (sessionId: string) => {
    const session = get().sessions.find((s) => s.id === sessionId);
    set({ currentSession: session || null });
  },

  archiveSession: async (sessionId: string) => {
    try {
      const chatService = getChatService();
      await chatService.archiveSession(sessionId);

      set((state) => ({
        sessions: state.sessions.map((s) =>
          s.id === sessionId ? { ...s, isArchived: true } : s
        ),
      }));
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  loadMessages: async (sessionId: string, offset: number = 0) => {
    set({ isLoading: true });
    try {
      const chatService = getChatService();
      const newMessages = await chatService.getMessages(sessionId, offset);

      set((state) => {
        const existingMessages = state.messages[sessionId] || [];
        const mergedMessages =
          offset === 0
            ? newMessages
            : [...newMessages, ...existingMessages];

        return {
          messages: {
            ...state.messages,
            [sessionId]: mergedMessages,
          },
          error: null,
        };
      });

      return newMessages;
    } catch (error) {
      set({ error: (error as Error).message });
      return [];
    } finally {
      set({ isLoading: false });
    }
  },

  sendMessage: async (receiver: string, content: string, msgType = 0) => {
    const chatService = getChatService();
    const currentSession = get().currentSession;
    const sessionId = currentSession?.id || '';

    // 1. 生成临时消息，立即显示（乐观更新）
    const tempId = `temp-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    const tempMessage: Message = {
      id: tempId,
      tempId,
      sessionId,
      sender: chatService.myAddress,
      receiver,
      content,
      contentCid: '',
      msgType: msgType as MessageType,
      sentAt: 0,
      isRead: false,
      isDeletedBySender: false,
      isDeletedByReceiver: false,
      isMine: true,
      status: 'sending',
      retryCount: 0,
    };

    // 立即添加到消息列表
    set((state) => ({
      messages: {
        ...state.messages,
        [sessionId]: [...(state.messages[sessionId] || []), tempMessage],
      },
    }));

    try {
      // 2. 后台执行实际发送
      const result = await chatService.sendMessage(
        receiver,
        content,
        msgType,
        sessionId || undefined
      );

      // 3. 成功：用真实数据替换临时消息
      set((state) => ({
        messages: {
          ...state.messages,
          [result.sessionId]:
            state.messages[result.sessionId]?.map((msg) =>
              msg.tempId === tempId
                ? {
                    ...msg,
                    id: result.msgId,
                    sessionId: result.sessionId,
                    contentCid: result.cid,
                    sentAt: result.blockNumber,
                    status: 'sent' as const,
                  }
                : msg
            ) || [],
        },
      }));
    } catch (error) {
      // 4. 失败：标记错误状态
      set((state) => ({
        messages: {
          ...state.messages,
          [sessionId]:
            state.messages[sessionId]?.map((msg) =>
              msg.tempId === tempId
                ? {
                    ...msg,
                    status: 'failed' as const,
                    error: (error as Error).message,
                  }
                : msg
            ) || [],
        },
      }));
      throw error;
    }
  },

  retryMessage: async (tempId: string) => {
    const state = get();
    let targetMessage: Message | null = null;
    let targetSessionId: string | null = null;

    // 找到失败的消息
    for (const [sessionId, messages] of Object.entries(state.messages)) {
      const msg = messages.find(
        (m) => m.tempId === tempId && m.status === 'failed'
      );
      if (msg) {
        targetMessage = msg;
        targetSessionId = sessionId;
        break;
      }
    }

    if (!targetMessage || !targetSessionId) return;

    // 检查重试次数
    if ((targetMessage.retryCount || 0) >= 3) {
      throw new Error('重试次数已达上限');
    }

    // 更新状态为发送中
    set((state) => ({
      messages: {
        ...state.messages,
        [targetSessionId!]:
          state.messages[targetSessionId!]?.map((msg) =>
            msg.tempId === tempId
              ? {
                  ...msg,
                  status: 'sending' as const,
                  retryCount: (msg.retryCount || 0) + 1,
                }
              : msg
          ) || [],
      },
    }));

    // 重新发送
    const chatService = getChatService();
    try {
      const result = await chatService.sendMessage(
        targetMessage.receiver,
        targetMessage.content,
        targetMessage.msgType,
        targetSessionId
      );

      set((state) => ({
        messages: {
          ...state.messages,
          [targetSessionId!]:
            state.messages[targetSessionId!]?.map((msg) =>
              msg.tempId === tempId
                ? { ...msg, id: result.msgId, status: 'sent' as const }
                : msg
            ) || [],
        },
      }));
    } catch (error) {
      set((state) => ({
        messages: {
          ...state.messages,
          [targetSessionId!]:
            state.messages[targetSessionId!]?.map((msg) =>
              msg.tempId === tempId
                ? {
                    ...msg,
                    status: 'failed' as const,
                    error: (error as Error).message,
                  }
                : msg
            ) || [],
        },
      }));
      throw error;
    }
  },

  removeFailedMessage: (tempId: string) => {
    set((state) => {
      const newMessages = { ...state.messages };
      for (const sessionId in newMessages) {
        newMessages[sessionId] = newMessages[sessionId].filter(
          (msg) => msg.tempId !== tempId || msg.status !== 'failed'
        );
      }
      return { messages: newMessages };
    });
  },

  markAsRead: async (messageIds: number[]) => {
    if (messageIds.length === 0) return;

    try {
      const chatService = getChatService();
      await chatService.markAsRead(messageIds);

      // 更新本地状态
      set((state) => {
        const newMessages = { ...state.messages };
        for (const sessionId in newMessages) {
          newMessages[sessionId] = newMessages[sessionId].map((msg) =>
            messageIds.includes(msg.id as number)
              ? { ...msg, isRead: true }
              : msg
          );
        }
        return { messages: newMessages };
      });
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  markSessionAsRead: async (sessionId: string) => {
    try {
      const chatService = getChatService();
      await chatService.markSessionAsRead(sessionId);

      set((state) => ({
        sessions: state.sessions.map((s) =>
          s.id === sessionId ? { ...s, unreadCount: 0 } : s
        ),
        totalUnread: state.totalUnread - (state.sessions.find(s => s.id === sessionId)?.unreadCount || 0),
      }));
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  deleteMessage: async (msgId: number) => {
    try {
      const chatService = getChatService();
      await chatService.deleteMessage(msgId);

      set((state) => {
        const newMessages = { ...state.messages };
        for (const sessionId in newMessages) {
          newMessages[sessionId] = newMessages[sessionId].filter(
            (msg) => msg.id !== msgId
          );
        }
        return { messages: newMessages };
      });
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  blockUser: async (address: string) => {
    try {
      const chatService = getChatService();
      await chatService.blockUser(address);

      set((state) => {
        const newBlockedUsers = new Set(state.blockedUsers);
        newBlockedUsers.add(address);
        return { blockedUsers: newBlockedUsers };
      });
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  unblockUser: async (address: string) => {
    try {
      const chatService = getChatService();
      await chatService.unblockUser(address);

      set((state) => {
        const newBlockedUsers = new Set(state.blockedUsers);
        newBlockedUsers.delete(address);
        return { blockedUsers: newBlockedUsers };
      });
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  // 事件处理方法
  handleNewMessage: (message: Message) => {
    set((state) => {
      const sessionMessages = state.messages[message.sessionId] || [];
      return {
        messages: {
          ...state.messages,
          [message.sessionId]: [...sessionMessages, message],
        },
        totalUnread: state.totalUnread + (message.isMine ? 0 : 1),
      };
    });
  },

  handleMessageRead: (msgId: number, reader: string) => {
    set((state) => {
      const newMessages = { ...state.messages };

      for (const sessionId in newMessages) {
        newMessages[sessionId] = newMessages[sessionId].map((msg) =>
          msg.id === msgId ? { ...msg, isRead: true } : msg
        );
      }

      return { messages: newMessages };
    });
  },

  handleMessageDeleted: (msgId: number) => {
    set((state) => {
      const newMessages = { ...state.messages };

      for (const sessionId in newMessages) {
        newMessages[sessionId] = newMessages[sessionId].filter(
          (msg) => msg.id !== msgId
        );
      }

      return { messages: newMessages };
    });
  },

  handleSessionCreated: (session: Session) => {
    set((state) => ({
      sessions: [session, ...state.sessions],
    }));
  },

  handleUserBlocked: (blocker: string, blocked: string) => {
    set((state) => {
      const newBlockedUsers = new Set(state.blockedUsers);
      newBlockedUsers.add(`${blocker}:${blocked}`);
      return { blockedUsers: newBlockedUsers };
    });
  },

  handleUserUnblocked: (unblocker: string, unblocked: string) => {
    set((state) => {
      const newBlockedUsers = new Set(state.blockedUsers);
      newBlockedUsers.delete(`${unblocker}:${unblocked}`);
      return { blockedUsers: newBlockedUsers };
    });
  },
}));
