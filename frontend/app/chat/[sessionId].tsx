import { useTransaction } from '@/src/hooks/useTransaction';
import { IpfsService } from '@/src/services/ipfs';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useCallback, useEffect, useRef, useState } from 'react';
import {
    Animated,
    FlatList,
    KeyboardAvoidingView,
    Platform,
    Pressable,
    StyleSheet,
    Text,
    TextInput,
    View
} from 'react-native';

interface Message {
  id: string;
  content: string;
  senderId: string;
  timestamp: string;
  isRead: boolean;
  cid?: string;
  status?: 'sending' | 'sent' | 'delivered' | 'read';
}

const MOCK_MESSAGES: Message[] = [
  { id: '1', content: '你好！', senderId: 'other', timestamp: '10:00', isRead: true, status: 'read' },
  { id: '2', content: '你好，有什么可以帮助你的吗？', senderId: 'me', timestamp: '10:01', isRead: true, status: 'read' },
  { id: '3', content: '我想咨询一下占卜服务', senderId: 'other', timestamp: '10:05', isRead: true, status: 'read' },
  { id: '4', content: '好的，请问你想了解哪方面的？', senderId: 'me', timestamp: '10:06', isRead: true, status: 'read' },
  { id: '5', content: '八字命理', senderId: 'other', timestamp: '10:10', isRead: false, status: 'delivered' },
];

const STATUS_ICONS = {
  sending: '⏳',
  sent: '✓',
  delivered: '✓✓',
  read: '✓✓',
};

export default function ChatDetailScreen() {
  const { sessionId } = useLocalSearchParams<{ sessionId: string }>();
  const router = useRouter();
  const { address, mnemonic } = useAuthStore();
  const { isConnected } = useChainStore();
  const { sendChatMessage, isLoading } = useTransaction();
  const [messages, setMessages] = useState<Message[]>(MOCK_MESSAGES);
  const [inputText, setInputText] = useState('');
  const [isSending, setIsSending] = useState(false);
  const flatListRef = useRef<FlatList>(null);
  const fadeAnim = useRef(new Animated.Value(0)).current;

  useEffect(() => {
    flatListRef.current?.scrollToEnd({ animated: true });
  }, [messages]);

  useEffect(() => {
    Animated.timing(fadeAnim, {
      toValue: 1,
      duration: 300,
      useNativeDriver: true,
    }).start();
  }, []);

  const markAsRead = useCallback((messageId: string) => {
    setMessages(prev => prev.map(msg => 
      msg.id === messageId ? { ...msg, isRead: true, status: 'read' as const } : msg
    ));
  }, []);

  useEffect(() => {
    const unreadMessages = messages.filter(m => m.senderId === 'other' && !m.isRead);
    unreadMessages.forEach(msg => {
      setTimeout(() => markAsRead(msg.id), 1000);
    });
  }, [messages, markAsRead]);

  const handleSend = async () => {
    if (!inputText.trim() || isSending) return;

    const tempId = Date.now().toString();
    const content = inputText.trim();
    
    const newMessage: Message = {
      id: tempId,
      content,
      senderId: 'me',
      timestamp: new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }),
      isRead: false,
      status: 'sending',
    };

    setMessages(prev => [...prev, newMessage]);
    setInputText('');
    setIsSending(true);

    try {
      const cid = await IpfsService.uploadSimpleMessage(content);
      
      setMessages(prev => prev.map(msg => 
        msg.id === tempId ? { ...msg, cid, status: 'sent' as const } : msg
      ));

      setTimeout(() => {
        setMessages(prev => prev.map(msg => 
          msg.id === tempId ? { ...msg, status: 'delivered' as const } : msg
        ));
      }, 1500);

      setTimeout(() => {
        setMessages(prev => prev.map(msg => 
          msg.id === tempId ? { ...msg, isRead: true, status: 'read' as const } : msg
        ));
      }, 3000);

    } catch (error) {
      console.error('Failed to send message:', error);
    } finally {
      setIsSending(false);
    }
  };

  const renderMessage = ({ item }: { item: Message }) => {
    const isMe = item.senderId === 'me';
    return (
      <View style={[styles.messageRow, isMe && styles.messageRowMe]}>
        <View style={[styles.messageBubble, isMe ? styles.bubbleMe : styles.bubbleOther]}>
          <Text style={[styles.messageText, isMe && styles.messageTextMe]}>
            {item.content}
          </Text>
          <View style={styles.messageFooter}>
            <Text style={[styles.messageTime, isMe && styles.messageTimeMe]}>
              {item.timestamp}
            </Text>
            {isMe && item.status && (
              <Text style={[
                styles.statusIcon, 
                item.status === 'read' && styles.statusRead
              ]}>
                {STATUS_ICONS[item.status]}
              </Text>
            )}
          </View>
        </View>
      </View>
    );
  };

  return (
    <KeyboardAvoidingView
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}
      keyboardVerticalOffset={90}
    >
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>聊天</Text>
        <View style={styles.headerRight} />
      </View>

      <FlatList
        ref={flatListRef}
        data={messages}
        renderItem={renderMessage}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.messageList}
        onContentSizeChange={() => flatListRef.current?.scrollToEnd({ animated: false })}
      />

      <View style={styles.inputContainer}>
        <TextInput
          style={styles.input}
          value={inputText}
          onChangeText={setInputText}
          placeholder="输入消息..."
          placeholderTextColor="#9ca3af"
          multiline
          maxLength={500}
        />
        <Pressable
          style={({ pressed }) => [
            styles.sendButton,
            !inputText.trim() && styles.sendButtonDisabled,
            pressed && styles.sendButtonPressed,
          ]}
          onPress={handleSend}
          disabled={!inputText.trim()}
        >
          <Text style={styles.sendButtonText}>发送</Text>
        </Pressable>
      </View>
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#fff',
    paddingTop: 50,
    paddingBottom: 12,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  backButton: {
    padding: 8,
  },
  backText: {
    fontSize: 16,
    color: '#6D28D9',
  },
  headerTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
  },
  headerRight: {
    width: 60,
  },
  messageList: {
    padding: 16,
    paddingBottom: 8,
  },
  messageRow: {
    flexDirection: 'row',
    marginBottom: 12,
  },
  messageRowMe: {
    justifyContent: 'flex-end',
  },
  messageBubble: {
    maxWidth: '75%',
    padding: 12,
    borderRadius: 16,
  },
  bubbleOther: {
    backgroundColor: '#fff',
    borderTopLeftRadius: 4,
  },
  bubbleMe: {
    backgroundColor: '#6D28D9',
    borderTopRightRadius: 4,
  },
  messageText: {
    fontSize: 15,
    color: '#1f2937',
    lineHeight: 20,
  },
  messageTextMe: {
    color: '#fff',
  },
  messageFooter: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'flex-end',
    marginTop: 4,
    gap: 4,
  },
  messageTime: {
    fontSize: 11,
    color: '#9ca3af',
  },
  messageTimeMe: {
    color: 'rgba(255,255,255,0.7)',
  },
  statusIcon: {
    fontSize: 10,
    color: 'rgba(255,255,255,0.5)',
  },
  statusRead: {
    color: '#60a5fa',
  },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    backgroundColor: '#fff',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderTopWidth: 1,
    borderTopColor: '#e5e7eb',
  },
  input: {
    flex: 1,
    backgroundColor: '#f3f4f6',
    borderRadius: 20,
    paddingHorizontal: 16,
    paddingVertical: 10,
    fontSize: 15,
    maxHeight: 100,
    color: '#1f2937',
  },
  sendButton: {
    marginLeft: 8,
    backgroundColor: '#6D28D9',
    borderRadius: 20,
    paddingHorizontal: 20,
    paddingVertical: 10,
  },
  sendButtonDisabled: {
    backgroundColor: '#d1d5db',
  },
  sendButtonPressed: {
    opacity: 0.8,
  },
  sendButtonText: {
    color: '#fff',
    fontSize: 15,
    fontWeight: '600',
  },
});
