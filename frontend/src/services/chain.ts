import { useChainStore } from '@/src/stores/chain';
import { ApiPromise } from '@polkadot/api';

type AnyCodec = any;

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
  contentCid?: string;
}

export interface DivinationProvider {
  id: string;
  name: string;
  avatar?: string;
  rating: number;
  orderCount: number;
  specialties: string[];
  price: number;
  isOnline: boolean;
  deposit: string;
}

export class ChainService {
  private api: ApiPromise | null = null;

  setApi(api: ApiPromise) {
    this.api = api;
  }

  private getApi(): ApiPromise {
    if (!this.api) {
      const { api } = useChainStore.getState();
      if (!api) {
        throw new Error('Chain not connected');
      }
      this.api = api;
    }
    return this.api;
  }

  async getChatSessions(accountId: string): Promise<ChatSession[]> {
    try {
      const api = this.getApi();
      const sessionIds = await api.query.chatCore.userSessions.keys(accountId);
      
      const sessions: ChatSession[] = [];
      for (const key of sessionIds) {
        const sessionId = key.args[1].toHex();
        const session = await api.query.chatCore.sessions(sessionId) as AnyCodec;
        
        if (session?.isSome) {
          const sessionData = session.unwrap();
          const unreadCount = await api.query.chatCore.unreadCount([accountId, sessionId]) as AnyCodec;
          
          sessions.push({
            id: sessionId,
            participantName: sessionData.participants[0].toString().slice(0, 8) + '...',
            lastMessage: '加密消息',
            lastMessageTime: sessionData.lastActive.toString(),
            unreadCount: unreadCount.toNumber(),
            isOnline: false,
          });
        }
      }
      return sessions;
    } catch (error) {
      console.error('Failed to get chat sessions:', error);
      return [];
    }
  }

  async getChatMessages(sessionId: string): Promise<ChatMessage[]> {
    try {
      const api = this.getApi();
      const messageIds = await api.query.chatCore.sessionMessages.keys(sessionId);
      
      const messages: ChatMessage[] = [];
      for (const key of messageIds) {
        const msgId = key.args[1].toString();
        const message = await api.query.chatCore.messages(msgId) as AnyCodec;
        
        if (message?.isSome) {
          const msgData = message.unwrap();
          messages.push({
            id: msgId,
            content: '加密消息 (CID: ' + msgData.contentCid.toHuman() + ')',
            senderId: msgData.sender.toString(),
            timestamp: msgData.sentAt.toString(),
            isRead: msgData.isRead.isTrue,
            contentCid: msgData.contentCid.toHuman() as string,
          });
        }
      }
      return messages;
    } catch (error) {
      console.error('Failed to get chat messages:', error);
      return [];
    }
  }

  async sendMessage(
    receiver: string,
    contentCid: string,
    msgType: number = 0,
    sessionId?: string
  ): Promise<string | null> {
    try {
      const api = this.getApi();
      const tx = api.tx.chatCore.sendMessage(
        receiver,
        contentCid,
        msgType,
        sessionId || null
      );
      return tx.toHex();
    } catch (error) {
      console.error('Failed to create send message tx:', error);
      return null;
    }
  }

  async getDivinationProviders(): Promise<DivinationProvider[]> {
    try {
      const api = this.getApi();
      const providerEntries = await api.query.divinationMarket.providers.entries();
      
      const providers: DivinationProvider[] = [];
      for (const [key, value] of providerEntries) {
        if ((value as AnyCodec)?.isSome) {
          const provider = (value as AnyCodec).unwrap();
          providers.push({
            id: key.args[0].toString(),
            name: provider.name?.toHuman() as string || '未知',
            rating: (provider.rating?.toNumber() || 0) / 10,
            orderCount: provider.totalOrders?.toNumber() || 0,
            specialties: provider.supportedTypes?.toHuman() as string[] || [],
            price: provider.minPrice?.toNumber() || 0,
            isOnline: provider.status?.toHuman() === 'Active',
            deposit: provider.deposit?.toString() || '0',
          });
        }
      }
      return providers;
    } catch (error) {
      console.error('Failed to get divination providers:', error);
      return [];
    }
  }

  async getBalance(accountId: string): Promise<string> {
    try {
      const api = this.getApi();
      const account = await api.query.system.account(accountId) as AnyCodec;
      return account.data.free.toString();
    } catch (error) {
      console.error('Failed to get balance:', error);
      return '0';
    }
  }

  async getChainInfo(): Promise<{ chain: string; nodeName: string; nodeVersion: string }> {
    try {
      const api = this.getApi();
      const [chain, nodeName, nodeVersion] = await Promise.all([
        api.rpc.system.chain(),
        api.rpc.system.name(),
        api.rpc.system.version(),
      ]);
      return {
        chain: chain.toString(),
        nodeName: nodeName.toString(),
        nodeVersion: nodeVersion.toString(),
      };
    } catch (error) {
      console.error('Failed to get chain info:', error);
      return { chain: 'Unknown', nodeName: 'Unknown', nodeVersion: 'Unknown' };
    }
  }
}

export const chainService = new ChainService();
