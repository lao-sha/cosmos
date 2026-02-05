import React, { useState, useRef, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TextInput,
  TouchableOpacity,
  KeyboardAvoidingView,
  Platform,
  Alert,
} from 'react-native';
import { useLocalSearchParams, useRouter, Stack } from 'expo-router';
import { Send, Image as ImageIcon, Plus, Phone, MoreVertical } from 'lucide-react-native';
import * as Haptics from 'expo-haptics';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import {
  getMessages,
  sendMessage,
  markAsRead,
  getChatUser,
  type Message,
  type ChatUser,
} from '@/services/chat';
import { Colors } from '@/constants/colors';

export default function ChatDetailScreen() {
  const colors = useColors();
  const router = useRouter();
  const { id } = useLocalSearchParams<{ id: string }>();
  const { address, mnemonic } = useWalletStore();
  const flatListRef = useRef<FlatList>(null);

  const [messages, setMessages] = useState<Message[]>([]);
  const [chatUser, setChatUser] = useState<ChatUser | null>(null);
  const [inputText, setInputText] = useState('');
  const [sending, setSending] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
    const interval = setInterval(loadMessages, 5000);
    return () => clearInterval(interval);
  }, [id]);

  const loadData = async () => {
    setLoading(true);
    await Promise.all([loadMessages(), loadChatUser()]);
    setLoading(false);
  };

  const loadMessages = async () => {
    if (!id) return;
    try {
      const data = await getMessages(id);
      setMessages(data.reverse());
      if (mnemonic) {
        await markAsRead(id, mnemonic).catch(() => {});
      }
    } catch (error) {
      console.error('Failed to load messages:', error);
    }
  };

  const loadChatUser = async () => {
    if (!id) return;
    try {
      const user = await getChatUser(id);
      setChatUser(user);
    } catch (error) {
      console.error('Failed to load chat user:', error);
    }
  };

  const handleSend = async () => {
    if (!inputText.trim() || !mnemonic || !id) return;

    const text = inputText.trim();
    setInputText('');
    setSending(true);

    // Optimistic update
    const tempMessage: Message = {
      id: `temp_${Date.now()}`,
      chatId: id,
      senderId: address || '',
      content: text,
      type: 'text',
      status: 'sending',
      createdAt: Date.now(),
    };
    setMessages((prev) => [...prev, tempMessage]);

    try {
      const msgId = await sendMessage(id, text, 'text', mnemonic);
      Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light);
      
      // Update message status
      setMessages((prev) =>
        prev.map((m) =>
          m.id === tempMessage.id
            ? { ...m, id: msgId, status: 'sent' }
            : m
        )
      );
    } catch (error: any) {
      setMessages((prev) =>
        prev.map((m) =>
          m.id === tempMessage.id ? { ...m, status: 'failed' } : m
        )
      );
      Alert.alert('发送失败', error.message);
    } finally {
      setSending(false);
    }
  };

  const renderMessage = ({ item }: { item: Message }) => {
    const isMe = item.senderId === address;

    return (
      <View
        style={[
          styles.messageRow,
          isMe ? styles.messageRowRight : styles.messageRowLeft,
        ]}
      >
        {!isMe && (
          <View style={[styles.avatar, { backgroundColor: Colors.primary + '20' }]}>
            <Text style={[styles.avatarText, { color: Colors.primary }]}>
              {chatUser?.name?.charAt(0) || 'U'}
            </Text>
          </View>
        )}
        <View
          style={[
            styles.bubble,
            isMe
              ? [styles.bubbleRight, { backgroundColor: Colors.chat.myBubble }]
              : [styles.bubbleLeft, { backgroundColor: colors.isDark ? Colors.chat.theirBubbleDark : Colors.chat.theirBubbleLight }],
          ]}
        >
          <Text
            style={[
              styles.messageText,
              { color: isMe ? '#FFFFFF' : colors.textPrimary },
            ]}
          >
            {item.content}
          </Text>
          <View style={styles.messageFooter}>
            <Text
              style={[
                styles.messageTime,
                { color: isMe ? 'rgba(255,255,255,0.7)' : colors.textTertiary },
              ]}
            >
              {formatTime(item.createdAt)}
            </Text>
            {isMe && (
              <Text
                style={[
                  styles.messageStatus,
                  { color: item.status === 'failed' ? Colors.error : 'rgba(255,255,255,0.7)' },
                ]}
              >
                {item.status === 'sending'
                  ? '发送中'
                  : item.status === 'failed'
                  ? '失败'
                  : item.status === 'read'
                  ? '已读'
                  : '已发送'}
              </Text>
            )}
          </View>
        </View>
      </View>
    );
  };

  return (
    <>
      <Stack.Screen
        options={{
          title: chatUser?.name || '聊天',
          headerRight: () => (
            <TouchableOpacity style={styles.headerButton}>
              <MoreVertical size={24} color={colors.textPrimary} />
            </TouchableOpacity>
          ),
        }}
      />
      <KeyboardAvoidingView
        style={[styles.container, { backgroundColor: colors.background }]}
        behavior={Platform.OS === 'ios' ? 'padding' : undefined}
        keyboardVerticalOffset={90}
      >
        <FlatList
          ref={flatListRef}
          data={messages}
          renderItem={renderMessage}
          keyExtractor={(item) => item.id}
          contentContainerStyle={styles.messageList}
          onContentSizeChange={() =>
            flatListRef.current?.scrollToEnd({ animated: false })
          }
          ListEmptyComponent={
            <View style={styles.empty}>
              <Text style={[styles.emptyText, { color: colors.textTertiary }]}>
                {loading ? '加载中...' : '暂无消息'}
              </Text>
            </View>
          }
        />

        {/* Input Area */}
        <View style={[styles.inputArea, { backgroundColor: colors.surface, borderTopColor: colors.border }]}>
          <TouchableOpacity style={styles.inputButton}>
            <Plus size={24} color={colors.textSecondary} />
          </TouchableOpacity>
          
          <View style={[styles.inputWrapper, { backgroundColor: colors.background }]}>
            <TextInput
              style={[styles.input, { color: colors.textPrimary }]}
              placeholder="输入消息..."
              placeholderTextColor={colors.textTertiary}
              value={inputText}
              onChangeText={setInputText}
              multiline
              maxLength={1000}
            />
          </View>

          <TouchableOpacity
            style={[
              styles.sendButton,
              { backgroundColor: inputText.trim() ? Colors.primary : colors.border },
            ]}
            onPress={handleSend}
            disabled={!inputText.trim() || sending}
          >
            <Send size={20} color={inputText.trim() ? '#FFFFFF' : colors.textTertiary} />
          </TouchableOpacity>
        </View>
      </KeyboardAvoidingView>
    </>
  );
}

function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  
  if (date.toDateString() === now.toDateString()) {
    return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
  }
  return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' });
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  headerButton: {
    padding: 8,
  },
  messageList: {
    padding: 16,
    flexGrow: 1,
  },
  messageRow: {
    flexDirection: 'row',
    marginBottom: 12,
    alignItems: 'flex-end',
  },
  messageRowLeft: {
    justifyContent: 'flex-start',
  },
  messageRowRight: {
    justifyContent: 'flex-end',
  },
  avatar: {
    width: 36,
    height: 36,
    borderRadius: 18,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 8,
  },
  avatarText: {
    fontSize: 14,
    fontWeight: '600',
  },
  bubble: {
    maxWidth: '75%',
    paddingHorizontal: 14,
    paddingVertical: 10,
    borderRadius: 18,
  },
  bubbleLeft: {
    borderBottomLeftRadius: 4,
  },
  bubbleRight: {
    borderBottomRightRadius: 4,
  },
  messageText: {
    fontSize: 16,
    lineHeight: 22,
  },
  messageFooter: {
    flexDirection: 'row',
    justifyContent: 'flex-end',
    marginTop: 4,
    gap: 6,
  },
  messageTime: {
    fontSize: 11,
  },
  messageStatus: {
    fontSize: 11,
  },
  empty: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    paddingTop: 100,
  },
  emptyText: {
    fontSize: 14,
  },
  inputArea: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    padding: 12,
    borderTopWidth: 1,
  },
  inputButton: {
    padding: 8,
  },
  inputWrapper: {
    flex: 1,
    borderRadius: 20,
    paddingHorizontal: 16,
    paddingVertical: 8,
    marginHorizontal: 8,
    maxHeight: 120,
  },
  input: {
    fontSize: 16,
    maxHeight: 100,
  },
  sendButton: {
    width: 40,
    height: 40,
    borderRadius: 20,
    alignItems: 'center',
    justifyContent: 'center',
  },
});
