import React, { useState, useRef } from 'react';
import { View, Text, ScrollView, TouchableOpacity, StyleSheet, TextInput, KeyboardAvoidingView, Platform } from 'react-native';
import { Send, Smile, Heart, Sparkles } from 'lucide-react-native';

interface Message {
  id: number;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
  emotion?: string;
}

export default function ChatScreen() {
  const [messages, setMessages] = useState<Message[]>([
    {
      id: 1,
      role: 'assistant',
      content: '喵~ 主人好呀！🐱 今天想聊点什么呢？',
      timestamp: new Date(),
      emotion: 'happy',
    },
  ]);
  const [inputText, setInputText] = useState('');
  const [isTyping, setIsTyping] = useState(false);
  const scrollViewRef = useRef<ScrollView>(null);

  const petInfo = {
    name: '小火',
    element: 'fire',
    level: 15,
    personality: {
      extroversion: 70,
      warmth: 80,
      humor: 60,
    },
  };

  const handleSend = async () => {
    if (!inputText.trim()) return;

    const userMessage: Message = {
      id: messages.length + 1,
      role: 'user',
      content: inputText,
      timestamp: new Date(),
    };

    setMessages(prev => [...prev, userMessage]);
    setInputText('');
    setIsTyping(true);

    // 模拟 AI 响应
    setTimeout(() => {
      const responses = [
        { content: '喵喵~ 原来是这样呀！主人说的真有趣呢~ 🎉', emotion: 'happy' },
        { content: '嗯嗯，小火明白了喵~ 主人今天心情怎么样呀？💕', emotion: 'caring' },
        { content: '哇！这个听起来好棒喵！小火也想试试看~ ✨', emotion: 'excited' },
        { content: '主人辛苦了喵~ 要好好休息哦！小火会一直陪着你的~ 🌙', emotion: 'caring' },
      ];
      
      const randomResponse = responses[Math.floor(Math.random() * responses.length)];
      
      const assistantMessage: Message = {
        id: messages.length + 2,
        role: 'assistant',
        content: randomResponse.content,
        timestamp: new Date(),
        emotion: randomResponse.emotion,
      };

      setMessages(prev => [...prev, assistantMessage]);
      setIsTyping(false);
    }, 1500);
  };

  const quickReplies = [
    '今天过得怎么样？',
    '给我讲个笑话',
    '我有点累了',
    '陪我聊聊天',
  ];

  return (
    <KeyboardAvoidingView
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
      keyboardVerticalOffset={90}
    >
      {/* 宠物信息栏 */}
      <View style={styles.petHeader}>
        <View style={styles.petAvatar}>
          <Text style={styles.petEmoji}>🐱</Text>
        </View>
        <View style={styles.petInfo}>
          <Text style={styles.petName}>{petInfo.name}</Text>
          <Text style={styles.petMeta}>Lv.{petInfo.level} · 火系</Text>
        </View>
        <View style={styles.moodIndicator}>
          <Heart size={16} color="#FF6B6B" fill="#FF6B6B" />
          <Text style={styles.moodText}>开心</Text>
        </View>
      </View>

      {/* 消息列表 */}
      <ScrollView
        ref={scrollViewRef}
        style={styles.messageList}
        contentContainerStyle={styles.messageListContent}
        onContentSizeChange={() => scrollViewRef.current?.scrollToEnd()}
      >
        {messages.map((message) => (
          <View
            key={message.id}
            style={[
              styles.messageContainer,
              message.role === 'user' ? styles.userMessage : styles.assistantMessage,
            ]}
          >
            {message.role === 'assistant' && (
              <View style={styles.assistantAvatar}>
                <Text style={styles.avatarEmoji}>🐱</Text>
              </View>
            )}
            <View
              style={[
                styles.messageBubble,
                message.role === 'user' ? styles.userBubble : styles.assistantBubble,
              ]}
            >
              <Text style={[
                styles.messageText,
                message.role === 'user' ? styles.userText : styles.assistantText,
              ]}>
                {message.content}
              </Text>
            </View>
          </View>
        ))}
        
        {isTyping && (
          <View style={[styles.messageContainer, styles.assistantMessage]}>
            <View style={styles.assistantAvatar}>
              <Text style={styles.avatarEmoji}>🐱</Text>
            </View>
            <View style={[styles.messageBubble, styles.assistantBubble]}>
              <View style={styles.typingIndicator}>
                <View style={[styles.typingDot, styles.typingDot1]} />
                <View style={[styles.typingDot, styles.typingDot2]} />
                <View style={[styles.typingDot, styles.typingDot3]} />
              </View>
            </View>
          </View>
        )}
      </ScrollView>

      {/* 快捷回复 */}
      <ScrollView
        horizontal
        showsHorizontalScrollIndicator={false}
        style={styles.quickReplies}
        contentContainerStyle={styles.quickRepliesContent}
      >
        {quickReplies.map((reply, index) => (
          <TouchableOpacity
            key={index}
            style={styles.quickReplyButton}
            onPress={() => setInputText(reply)}
          >
            <Text style={styles.quickReplyText}>{reply}</Text>
          </TouchableOpacity>
        ))}
      </ScrollView>

      {/* 输入框 */}
      <View style={styles.inputContainer}>
        <TouchableOpacity style={styles.emojiButton}>
          <Smile size={24} color="#888" />
        </TouchableOpacity>
        <TextInput
          style={styles.input}
          placeholder="和宠物聊天..."
          placeholderTextColor="#666"
          value={inputText}
          onChangeText={setInputText}
          multiline
          maxLength={500}
        />
        <TouchableOpacity
          style={[styles.sendButton, !inputText.trim() && styles.sendButtonDisabled]}
          onPress={handleSend}
          disabled={!inputText.trim()}
        >
          <Send size={20} color={inputText.trim() ? '#fff' : '#666'} />
        </TouchableOpacity>
      </View>
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0f0f1a',
  },
  petHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#1a1a2e',
    padding: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#252540',
  },
  petAvatar: {
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: '#252540',
    justifyContent: 'center',
    alignItems: 'center',
    borderWidth: 2,
    borderColor: '#FF6B6B',
  },
  petEmoji: {
    fontSize: 24,
  },
  petInfo: {
    flex: 1,
    marginLeft: 12,
  },
  petName: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#fff',
  },
  petMeta: {
    fontSize: 12,
    color: '#888',
  },
  moodIndicator: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FF6B6B20',
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 12,
  },
  moodText: {
    fontSize: 12,
    color: '#FF6B6B',
    marginLeft: 4,
  },
  messageList: {
    flex: 1,
  },
  messageListContent: {
    padding: 16,
  },
  messageContainer: {
    flexDirection: 'row',
    marginBottom: 16,
  },
  userMessage: {
    justifyContent: 'flex-end',
  },
  assistantMessage: {
    justifyContent: 'flex-start',
  },
  assistantAvatar: {
    width: 36,
    height: 36,
    borderRadius: 18,
    backgroundColor: '#252540',
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 8,
  },
  avatarEmoji: {
    fontSize: 20,
  },
  messageBubble: {
    maxWidth: '75%',
    padding: 12,
    borderRadius: 16,
  },
  userBubble: {
    backgroundColor: '#4ECDC4',
    borderBottomRightRadius: 4,
  },
  assistantBubble: {
    backgroundColor: '#1a1a2e',
    borderBottomLeftRadius: 4,
  },
  messageText: {
    fontSize: 15,
    lineHeight: 22,
  },
  userText: {
    color: '#fff',
  },
  assistantText: {
    color: '#fff',
  },
  typingIndicator: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 4,
  },
  typingDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: '#666',
    marginHorizontal: 2,
  },
  typingDot1: {
    opacity: 0.4,
  },
  typingDot2: {
    opacity: 0.6,
  },
  typingDot3: {
    opacity: 0.8,
  },
  quickReplies: {
    maxHeight: 50,
    borderTopWidth: 1,
    borderTopColor: '#252540',
  },
  quickRepliesContent: {
    paddingHorizontal: 12,
    paddingVertical: 8,
  },
  quickReplyButton: {
    backgroundColor: '#252540',
    paddingHorizontal: 14,
    paddingVertical: 8,
    borderRadius: 16,
    marginRight: 8,
  },
  quickReplyText: {
    fontSize: 13,
    color: '#888',
  },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    backgroundColor: '#1a1a2e',
    padding: 12,
    borderTopWidth: 1,
    borderTopColor: '#252540',
  },
  emojiButton: {
    padding: 8,
  },
  input: {
    flex: 1,
    backgroundColor: '#252540',
    borderRadius: 20,
    paddingHorizontal: 16,
    paddingVertical: 10,
    fontSize: 15,
    color: '#fff',
    maxHeight: 100,
    marginHorizontal: 8,
  },
  sendButton: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#4ECDC4',
    justifyContent: 'center',
    alignItems: 'center',
  },
  sendButtonDisabled: {
    backgroundColor: '#333',
  },
});
