import { getApi } from './api';

export type MessageType = 'text' | 'image' | 'file' | 'system' | 'trade';
export type MessageStatus = 'sending' | 'sent' | 'delivered' | 'read' | 'failed';

export interface Message {
  id: string;
  chatId: string;
  senderId: string;
  content: string;
  type: MessageType;
  status: MessageStatus;
  createdAt: number;
  readAt?: number;
  metadata?: {
    fileName?: string;
    fileSize?: number;
    imageWidth?: number;
    imageHeight?: number;
    tradeType?: string;
    tradeAmount?: string;
  };
}

export interface Conversation {
  id: string;
  participantId: string;
  participantName: string;
  participantAvatar?: string;
  lastMessage: string;
  lastMessageType: MessageType;
  lastMessageTime: number;
  unreadCount: number;
  isOnline: boolean;
  isPinned: boolean;
  isMuted: boolean;
}

export interface ChatUser {
  id: string;
  address: string;
  name: string;
  avatar?: string;
  isOnline: boolean;
  lastSeen?: number;
}

export async function getConversations(address: string): Promise<Conversation[]> {
  try {
    const api = await getApi();
    const data = await api.query.chatCore.conversations(address);
    
    if (data.isEmpty) return [];
    
    const conversations = data.toJSON() as any[];
    return conversations.map((conv: any) => ({
      id: conv.id,
      participantId: conv.participantId,
      participantName: conv.participantName || '用户',
      participantAvatar: conv.participantAvatar,
      lastMessage: conv.lastMessage || '',
      lastMessageType: conv.lastMessageType || 'text',
      lastMessageTime: conv.lastMessageTime || Date.now(),
      unreadCount: conv.unreadCount || 0,
      isOnline: conv.isOnline || false,
      isPinned: conv.isPinned || false,
      isMuted: conv.isMuted || false,
    }));
  } catch (error) {
    console.error('Failed to get conversations:', error);
    return getMockConversations();
  }
}

export async function getMessages(
  chatId: string,
  limit = 50,
  before?: string
): Promise<Message[]> {
  try {
    const api = await getApi();
    const data = await api.query.chatCore.messages(chatId);
    
    if (data.isEmpty) return [];
    
    const messages = data.toJSON() as any[];
    return messages.map((msg: any) => ({
      id: msg.id,
      chatId: msg.chatId,
      senderId: msg.senderId,
      content: msg.content,
      type: msg.type || 'text',
      status: msg.status || 'sent',
      createdAt: msg.createdAt || Date.now(),
      readAt: msg.readAt,
      metadata: msg.metadata,
    }));
  } catch (error) {
    console.error('Failed to get messages:', error);
    return [];
  }
}

export async function sendMessage(
  targetId: string,
  content: string,
  type: MessageType,
  mnemonic: string,
  metadata?: any
): Promise<string> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  // Encrypt content (simplified - should use proper encryption)
  const encryptedContent = Buffer.from(content).toString('base64');

  return new Promise((resolve, reject) => {
    api.tx.chatCore
      .sendMessage(targetId, encryptedContent, type, metadata || null)
      .signAndSend(pair, ({ status, events, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) {
            reject(new Error(dispatchError.toString()));
          } else {
            const msgEvent = events.find(
              ({ event }) => event.method === 'MessageSent'
            );
            resolve(msgEvent?.event.data[0]?.toString() || status.asFinalized.toHex());
          }
        }
      })
      .catch(reject);
  });
}

export async function markAsRead(chatId: string, mnemonic: string): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.chatCore
      .markAsRead(chatId)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) reject(new Error(dispatchError.toString()));
          else resolve();
        }
      })
      .catch(reject);
  });
}

export async function getChatUser(userId: string): Promise<ChatUser | null> {
  try {
    const api = await getApi();
    const data = await api.query.chatCore.users(userId);
    
    if (data.isEmpty) return null;
    
    const user = data.toJSON() as any;
    return {
      id: userId,
      address: user.address,
      name: user.name || '用户',
      avatar: user.avatar,
      isOnline: user.isOnline || false,
      lastSeen: user.lastSeen,
    };
  } catch (error) {
    return null;
  }
}

function getMockConversations(): Conversation[] {
  return [
    {
      id: '1',
      participantId: 'user1',
      participantName: '金牌商家',
      lastMessage: '好的，已放币',
      lastMessageType: 'text',
      lastMessageTime: Date.now() - 60000,
      unreadCount: 0,
      isOnline: true,
      isPinned: false,
      isMuted: false,
    },
    {
      id: '2',
      participantId: 'system',
      participantName: '系统通知',
      lastMessage: '欢迎使用 COSMOS',
      lastMessageType: 'system',
      lastMessageTime: Date.now() - 3600000,
      unreadCount: 1,
      isOnline: true,
      isPinned: true,
      isMuted: false,
    },
  ];
}
