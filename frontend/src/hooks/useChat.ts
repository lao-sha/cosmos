import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useCallback, useEffect, useState } from 'react';

export interface ChatSession {
  id: string;
  participantName: string;
  participantAvatar?: string;
  lastMessage: string;
  lastMessageTime: string;
  unreadCount: number;
  isOnline: boolean;
}

export interface ChatMessage {
  id: string;
  content: string;
  senderId: string;
  timestamp: string;
  isRead: boolean;
}

const MOCK_SESSIONS: ChatSession[] = [
  {
    id: '1',
    participantName: '张三',
    lastMessage: '你好，请问在吗？',
    lastMessageTime: '10:30',
    unreadCount: 2,
    isOnline: true,
  },
  {
    id: '2',
    participantName: '李四',
    lastMessage: '好的，谢谢',
    lastMessageTime: '昨天',
    unreadCount: 0,
    isOnline: false,
  },
];

export function useChatSessions() {
  const { isLoggedIn, address } = useAuthStore();
  const { api, isConnected } = useChainStore();
  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchSessions = useCallback(async () => {
    if (!isLoggedIn || !address) {
      setSessions([]);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      if (api && isConnected) {
        const sessionKeys = await (api.query as any).chatCore?.userSessions?.keys(address);
        if (sessionKeys && sessionKeys.length > 0) {
          const chainSessions: ChatSession[] = [];
          for (const key of sessionKeys) {
            const sessionId = key.args[1].toHex();
            const session = await (api.query as any).chatCore.sessions(sessionId);
            if (session && session.isSome) {
              const data = session.unwrap();
              const unread = await (api.query as any).chatCore.unreadCount([address, sessionId]);
              chainSessions.push({
                id: sessionId,
                participantName: data.participants[0]?.toString().slice(0, 8) + '...',
                lastMessage: '加密消息',
                lastMessageTime: new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }),
                unreadCount: unread?.toNumber() || 0,
                isOnline: false,
              });
            }
          }
          if (chainSessions.length > 0) {
            setSessions(chainSessions);
            return;
          }
        }
      }
      setSessions(MOCK_SESSIONS);
    } catch (err) {
      console.error('Failed to fetch sessions:', err);
      setSessions(MOCK_SESSIONS);
      setError('链上查询失败，使用模拟数据');
    } finally {
      setLoading(false);
    }
  }, [isLoggedIn, address, api, isConnected]);

  useEffect(() => {
    fetchSessions();
  }, [fetchSessions]);

  return { sessions, loading, error, refetch: fetchSessions };
}

export function useChatMessages(sessionId: string) {
  const { api, isConnected } = useChainStore();
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchMessages = useCallback(async () => {
    if (!sessionId) return;

    setLoading(true);
    try {
      if (api && isConnected) {
        const msgKeys = await (api.query as any).chatCore?.sessionMessages?.keys(sessionId);
        if (msgKeys && msgKeys.length > 0) {
          const chainMessages: ChatMessage[] = [];
          for (const key of msgKeys) {
            const msgId = key.args[1].toString();
            const msg = await (api.query as any).chatCore.messages(msgId);
            if (msg && msg.isSome) {
              const data = msg.unwrap();
              chainMessages.push({
                id: msgId,
                content: `[加密] CID: ${data.contentCid?.toHuman() || 'unknown'}`,
                senderId: data.sender?.toString() || 'unknown',
                timestamp: new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }),
                isRead: data.isRead?.isTrue || false,
              });
            }
          }
          if (chainMessages.length > 0) {
            setMessages(chainMessages);
            return;
          }
        }
      }
      setMessages([
        { id: '1', content: '你好！', senderId: 'other', timestamp: '10:00', isRead: true },
        { id: '2', content: '你好，有什么可以帮助你的吗？', senderId: 'me', timestamp: '10:01', isRead: true },
      ]);
    } catch (err) {
      console.error('Failed to fetch messages:', err);
    } finally {
      setLoading(false);
    }
  }, [sessionId, api, isConnected]);

  useEffect(() => {
    fetchMessages();
  }, [fetchMessages]);

  const sendMessage = useCallback((content: string) => {
    const newMsg: ChatMessage = {
      id: Date.now().toString(),
      content,
      senderId: 'me',
      timestamp: new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }),
      isRead: false,
    };
    setMessages(prev => [...prev, newMsg]);
  }, []);

  return { messages, loading, sendMessage, refetch: fetchMessages };
}
