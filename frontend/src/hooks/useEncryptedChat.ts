import { IpfsService } from '@/src/services/ipfs';
import { useCallback, useState } from 'react';
import { useTransaction } from './useTransaction';

export interface ChatMessage {
  id: string;
  content: string;
  senderId: string;
  timestamp: string;
  isRead: boolean;
  cid?: string;
  isEncrypted?: boolean;
}

export function useEncryptedChat(sessionId: string) {
  const { sendChatMessage, isLoading, status } = useTransaction();
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isSending, setIsSending] = useState(false);

  const sendMessage = useCallback(async (content: string, receiver: string) => {
    if (!content.trim()) return null;

    setIsSending(true);
    
    try {
      const cid = await IpfsService.uploadSimpleMessage(content);
      
      const result = await sendChatMessage(receiver, cid, 0, sessionId);
      
      if (result?.success) {
        const newMessage: ChatMessage = {
          id: result.txHash || Date.now().toString(),
          content,
          senderId: 'me',
          timestamp: new Date().toLocaleTimeString('zh-CN', { 
            hour: '2-digit', 
            minute: '2-digit' 
          }),
          isRead: false,
          cid,
          isEncrypted: false,
        };
        
        setMessages(prev => [...prev, newMessage]);
        return newMessage;
      }
      
      return null;
    } catch (error) {
      console.error('Failed to send message:', error);
      return null;
    } finally {
      setIsSending(false);
    }
  }, [sendChatMessage, sessionId]);

  const addLocalMessage = useCallback((content: string) => {
    const newMessage: ChatMessage = {
      id: Date.now().toString(),
      content,
      senderId: 'me',
      timestamp: new Date().toLocaleTimeString('zh-CN', { 
        hour: '2-digit', 
        minute: '2-digit' 
      }),
      isRead: false,
    };
    setMessages(prev => [...prev, newMessage]);
    return newMessage;
  }, []);

  const fetchMessageContent = useCallback(async (cid: string): Promise<string> => {
    try {
      const payload = await IpfsService.fetchFromIpfs<{ content: string }>(cid);
      return payload.content;
    } catch (error) {
      console.error('Failed to fetch message:', error);
      return '[无法加载消息]';
    }
  }, []);

  return {
    messages,
    setMessages,
    sendMessage,
    addLocalMessage,
    fetchMessageContent,
    isSending: isSending || isLoading,
    status,
  };
}
