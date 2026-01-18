// frontend/src/features/livestream/components/LiveChat.tsx

import React, { useRef, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TextInput,
  TouchableOpacity,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useLivestreamStore } from '@/stores/livestream.store';
import type { LiveChatMessage } from '../types';

interface LiveChatProps {
  onSendMessage: (content: string) => void;
  onSendDanmaku: (content: string) => void;
  onOpenGiftPanel: () => void;
}

export function LiveChat({ onSendMessage, onSendDanmaku, onOpenGiftPanel }: LiveChatProps) {
  const { messages } = useLivestreamStore();
  const [inputText, setInputText] = React.useState('');
  const [isDanmaku, setIsDanmaku] = React.useState(false);
  const flatListRef = useRef<FlatList>(null);

  // 自动滚动到底部
  useEffect(() => {
    if (messages.length > 0) {
      setTimeout(() => {
        flatListRef.current?.scrollToEnd({ animated: true });
      }, 100);
    }
  }, [messages.length]);

  const handleSend = () => {
    if (!inputText.trim()) return;

    if (isDanmaku) {
      onSendDanmaku(inputText.trim());
    } else {
      onSendMessage(inputText.trim());
    }
    setInputText('');
  };

  const renderMessage = ({ item }: { item: LiveChatMessage }) => {
    if (item.type === 'gift') {
      return (
        <View style={styles.giftMessage}>
          <Text style={styles.giftIcon}>🎁</Text>
          <Text style={styles.giftText}>
            <Text style={styles.senderName}>{item.senderName}</Text>
            {' 送出 '}
            <Text style={styles.giftName}>{item.content}</Text>
            {item.giftCount && item.giftCount > 1 && (
              <Text style={styles.giftCount}> x{item.giftCount}</Text>
            )}
          </Text>
        </View>
      );
    }

    if (item.type === 'system') {
      return (
        <View style={styles.systemMessage}>
          <Text style={styles.systemText}>{item.content}</Text>
        </View>
      );
    }

    return (
      <View style={styles.chatMessage}>
        <Text style={styles.senderName}>{item.senderName}: </Text>
        <Text style={styles.messageContent}>{item.content}</Text>
      </View>
    );
  };

  return (
    <View style={styles.container}>
      <FlatList
        ref={flatListRef}
        data={messages}
        renderItem={renderMessage}
        keyExtractor={(item) => item.id}
        style={styles.messageList}
        showsVerticalScrollIndicator={false}
      />

      <View style={styles.inputContainer}>
        <TextInput
          style={styles.input}
          placeholder={isDanmaku ? '发送弹幕...' : '发送消息...'}
          placeholderTextColor="#666"
          value={inputText}
          onChangeText={setInputText}
          onSubmitEditing={handleSend}
          returnKeyType="send"
        />

        <TouchableOpacity
          style={[styles.modeBtn, isDanmaku && styles.modeBtnActive]}
          onPress={() => setIsDanmaku(!isDanmaku)}
        >
          <Text style={[styles.modeBtnText, isDanmaku && styles.modeBtnTextActive]}>
            弹幕
          </Text>
        </TouchableOpacity>

        <TouchableOpacity style={styles.giftBtn} onPress={onOpenGiftPanel}>
          <Ionicons name="gift" size={24} color="#FFD700" />
        </TouchableOpacity>

        <TouchableOpacity style={styles.sendBtn} onPress={handleSend}>
          <Ionicons name="send" size={20} color="#FFF" />
        </TouchableOpacity>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#1A1A2E',
  },
  messageList: {
    flex: 1,
    paddingHorizontal: 12,
  },
  chatMessage: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    paddingVertical: 6,
  },
  senderName: {
    color: '#FF4757',
    fontSize: 14,
    fontWeight: '500',
  },
  messageContent: {
    color: '#FFF',
    fontSize: 14,
  },
  giftMessage: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: 'rgba(255, 215, 0, 0.1)',
    paddingVertical: 8,
    paddingHorizontal: 12,
    borderRadius: 8,
    marginVertical: 4,
  },
  giftIcon: {
    fontSize: 16,
    marginRight: 8,
  },
  giftText: {
    color: '#FFF',
    fontSize: 14,
  },
  giftName: {
    color: '#FFD700',
    fontWeight: 'bold',
  },
  giftCount: {
    color: '#FF4757',
    fontWeight: 'bold',
  },
  systemMessage: {
    paddingVertical: 6,
    alignItems: 'center',
  },
  systemText: {
    color: '#666',
    fontSize: 12,
  },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
    borderTopWidth: 1,
    borderTopColor: '#333',
    gap: 8,
  },
  input: {
    flex: 1,
    height: 40,
    backgroundColor: '#252540',
    borderRadius: 20,
    paddingHorizontal: 16,
    color: '#FFF',
    fontSize: 14,
  },
  modeBtn: {
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 16,
    backgroundColor: '#252540',
  },
  modeBtnActive: {
    backgroundColor: '#FF4757',
  },
  modeBtnText: {
    color: '#999',
    fontSize: 12,
  },
  modeBtnTextActive: {
    color: '#FFF',
  },
  giftBtn: {
    padding: 8,
  },
  sendBtn: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#FF4757',
    justifyContent: 'center',
    alignItems: 'center',
  },
});
