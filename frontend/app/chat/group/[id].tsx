import { useTransaction } from '@/src/hooks/useTransaction';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useState, useRef, useEffect } from 'react';
import {
  FlatList,
  KeyboardAvoidingView,
  Platform,
  Pressable,
  StyleSheet,
  Text,
  TextInput,
  View
} from 'react-native';

interface GroupMessage {
  id: string;
  senderId: string;
  senderName: string;
  content: string;
  timestamp: string;
  isMe: boolean;
}

interface GroupInfo {
  id: string;
  name: string;
  memberCount: number;
  description: string;
}

const MOCK_GROUP_INFO: Record<string, GroupInfo> = {
  g1: { id: 'g1', name: '八字命理交流群', memberCount: 128, description: '八字命理爱好者交流群' },
  g2: { id: 'g2', name: '奇门遁甲研习社', memberCount: 56, description: '奇门遁甲学习交流' },
  g3: { id: 'g3', name: '塔罗牌爱好者', memberCount: 89, description: '塔罗牌解读分享' },
  g4: { id: 'g4', name: '紫微斗数学习群', memberCount: 45, description: '紫微斗数入门学习' },
};

const MOCK_MESSAGES: GroupMessage[] = [
  { id: '1', senderId: 'u1', senderName: '张三', content: '大家好，今天排了个有意思的盘', timestamp: '10:00', isMe: false },
  { id: '2', senderId: 'u2', senderName: '李四', content: '什么盘？分享一下呗', timestamp: '10:02', isMe: false },
  { id: '3', senderId: 'me', senderName: '我', content: '我也想看看', timestamp: '10:03', isMe: true },
  { id: '4', senderId: 'u1', senderName: '张三', content: '一个日主为甲木的八字，地支全是水', timestamp: '10:05', isMe: false },
  { id: '5', senderId: 'u3', senderName: '王五', content: '那不是水多木漂吗？', timestamp: '10:06', isMe: false },
  { id: '6', senderId: 'u1', senderName: '张三', content: '对，但是有个特殊格局', timestamp: '10:08', isMe: false },
];

export default function GroupChatScreen() {
  const router = useRouter();
  const { id } = useLocalSearchParams<{ id: string }>();
  const { address, isLoggedIn } = useAuthStore();
  const { isConnected } = useChainStore();
  const { sendGroupMessage, leaveGroup, isLoading: isTxLoading } = useTransaction();
  const flatListRef = useRef<FlatList>(null);
  
  const [messages, setMessages] = useState<GroupMessage[]>(MOCK_MESSAGES);
  const [inputText, setInputText] = useState('');
  const [groupInfo] = useState<GroupInfo | null>(MOCK_GROUP_INFO[id || 'g1'] || null);
  const groupIdNum = parseInt(id || '0');

  useEffect(() => {
    setTimeout(() => {
      flatListRef.current?.scrollToEnd({ animated: false });
    }, 100);
  }, []);

  const showAlert = (title: string, message: string) => {
    if (Platform.OS === 'web') {
      window.alert(`${title}\n${message}`);
    } else {
      const { Alert } = require('react-native');
      Alert.alert(title, message);
    }
  };

  const handleSend = async () => {
    if (!inputText.trim()) return;
    
    if (!isLoggedIn) {
      showAlert('提示', '请先登录钱包');
      return;
    }
    if (!isConnected) {
      showAlert('提示', '请先连接区块链网络');
      return;
    }
    
    const content = inputText.trim();
    setInputText('');
    
    // 先在本地显示消息（乐观更新）
    const newMessage: GroupMessage = {
      id: Date.now().toString(),
      senderId: 'me',
      senderName: '我',
      content,
      timestamp: new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }),
      isMe: true,
    };
    setMessages(prev => [...prev, newMessage]);
    
    setTimeout(() => {
      flatListRef.current?.scrollToEnd({ animated: true });
    }, 100);
    
    // 发送到链上
    const txResult = await sendGroupMessage(groupIdNum, content, 0);
    if (!txResult?.success) {
      // 发送失败，移除本地消息
      setMessages(prev => prev.filter(m => m.id !== newMessage.id));
      setInputText(content);
    }
  };
  
  const handleLeaveGroup = async () => {
    if (!isLoggedIn || !isConnected) {
      showAlert('提示', '请先登录并连接网络');
      return;
    }
    
    const txResult = await leaveGroup(groupIdNum);
    if (txResult?.success) {
      router.back();
    }
  };

  const handleGroupInfo = () => {
    router.push(`/chat/group/info/${id}` as any);
  };

  const renderMessage = ({ item }: { item: GroupMessage }) => (
    <View style={[styles.messageContainer, item.isMe && styles.messageContainerMe]}>
      {!item.isMe && (
        <View style={styles.avatarSmall}>
          <Text style={styles.avatarSmallText}>{item.senderName[0]}</Text>
        </View>
      )}
      <View style={styles.messageContent}>
        {!item.isMe && (
          <Text style={styles.senderName}>{item.senderName}</Text>
        )}
        <View style={[styles.messageBubble, item.isMe && styles.messageBubbleMe]}>
          <Text style={[styles.messageText, item.isMe && styles.messageTextMe]}>
            {item.content}
          </Text>
        </View>
        <Text style={[styles.messageTime, item.isMe && styles.messageTimeMe]}>
          {item.timestamp}
        </Text>
      </View>
    </View>
  );

  if (!groupInfo) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹</Text>
          </Pressable>
          <Text style={styles.headerTitle}>群聊不存在</Text>
          <View style={styles.headerRight} />
        </View>
      </View>
    );
  }

  return (
    <KeyboardAvoidingView 
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}
      keyboardVerticalOffset={0}
    >
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹</Text>
        </Pressable>
        <Pressable style={styles.headerCenter} onPress={handleGroupInfo}>
          <Text style={styles.headerTitle}>{groupInfo.name}</Text>
          <Text style={styles.headerSubtitle}>{groupInfo.memberCount}人</Text>
        </Pressable>
        <Pressable style={styles.headerRight} onPress={handleGroupInfo}>
          <Text style={styles.moreIcon}>⋯</Text>
        </Pressable>
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
          placeholder="输入消息..."
          placeholderTextColor="#9ca3af"
          value={inputText}
          onChangeText={setInputText}
          multiline
          maxLength={500}
        />
        <Pressable 
          style={[styles.sendButton, !inputText.trim() && styles.sendButtonDisabled]}
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
    backgroundColor: '#fff',
    paddingTop: 50,
    paddingBottom: 12,
    paddingHorizontal: 16,
    flexDirection: 'row',
    alignItems: 'center',
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  backButton: {
    padding: 8,
    marginRight: 8,
  },
  backText: {
    fontSize: 28,
    color: '#6D28D9',
    fontWeight: '300',
  },
  headerCenter: {
    flex: 1,
    alignItems: 'center',
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#1f2937',
  },
  headerSubtitle: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 2,
  },
  headerRight: {
    padding: 8,
    marginLeft: 8,
  },
  moreIcon: {
    fontSize: 20,
    color: '#6b7280',
  },
  messageList: {
    padding: 16,
    paddingBottom: 8,
  },
  messageContainer: {
    flexDirection: 'row',
    marginBottom: 16,
    alignItems: 'flex-start',
  },
  messageContainerMe: {
    flexDirection: 'row-reverse',
  },
  avatarSmall: {
    width: 36,
    height: 36,
    borderRadius: 18,
    backgroundColor: '#10b981',
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 8,
  },
  avatarSmallText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: 'bold',
  },
  messageContent: {
    maxWidth: '75%',
  },
  senderName: {
    fontSize: 12,
    color: '#6b7280',
    marginBottom: 4,
    marginLeft: 4,
  },
  messageBubble: {
    backgroundColor: '#fff',
    borderRadius: 16,
    borderTopLeftRadius: 4,
    padding: 12,
  },
  messageBubbleMe: {
    backgroundColor: '#6D28D9',
    borderTopLeftRadius: 16,
    borderTopRightRadius: 4,
  },
  messageText: {
    fontSize: 15,
    color: '#1f2937',
    lineHeight: 22,
  },
  messageTextMe: {
    color: '#fff',
  },
  messageTime: {
    fontSize: 11,
    color: '#9ca3af',
    marginTop: 4,
    marginLeft: 4,
  },
  messageTimeMe: {
    textAlign: 'right',
    marginRight: 4,
    marginLeft: 0,
  },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    backgroundColor: '#fff',
    paddingHorizontal: 16,
    paddingVertical: 12,
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
    backgroundColor: '#6D28D9',
    borderRadius: 20,
    paddingHorizontal: 20,
    paddingVertical: 10,
    marginLeft: 8,
  },
  sendButtonDisabled: {
    backgroundColor: '#d1d5db',
  },
  sendButtonText: {
    color: '#fff',
    fontSize: 15,
    fontWeight: '600',
  },
});
