/**
 * 消息搜索页面
 */

import React, { useState, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TextInput,
  FlatList,
  TouchableOpacity,
  ActivityIndicator,
  Pressable,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useChatStore } from '@/stores/chat.store';
import type { Message } from '../types';

const THEME_COLOR = '#B2955D';

export function MessageSearchScreen() {
  const router = useRouter();
  const { messages, sessions } = useChatStore();

  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<
    Array<{ message: Message; sessionId: string }>
  >([]);
  const [isSearching, setIsSearching] = useState(false);

  const handleSearch = useCallback(
    (query: string) => {
      setSearchQuery(query);

      if (!query.trim()) {
        setSearchResults([]);
        return;
      }

      setIsSearching(true);

      // 搜索所有会话中的消息
      const results: Array<{ message: Message; sessionId: string }> = [];
      const lowerQuery = query.toLowerCase();

      Object.entries(messages).forEach(([sessionId, sessionMessages]) => {
        sessionMessages.forEach((message) => {
          if (
            message.content.toLowerCase().includes(lowerQuery) &&
            message.msgType === 0 // 只搜索文本消息
          ) {
            results.push({ message, sessionId });
          }
        });
      });

      // 按时间倒序排序
      results.sort((a, b) => b.message.sentAt - a.message.sentAt);

      setSearchResults(results);
      setIsSearching(false);
    },
    [messages]
  );

  const handleClear = () => {
    setSearchQuery('');
    setSearchResults([]);
  };

  const handleResultPress = (sessionId: string, messageId: number | string) => {
    // 跳转到对应的聊天会话，并定位到该消息
    router.push(`/chat/${sessionId}?messageId=${messageId}`);
  };

  const getSessionName = (sessionId: string) => {
    const session = sessions.find((s) => s.id === sessionId);
    return session?.peerAlias || session?.peerProfile?.nickname || '未知用户';
  };

  const highlightText = (text: string, query: string) => {
    if (!query) return text;

    const parts = text.split(new RegExp(`(${query})`, 'gi'));
    return parts.map((part, index) =>
      part.toLowerCase() === query.toLowerCase() ? (
        <Text key={index} style={styles.highlight}>
          {part}
        </Text>
      ) : (
        <Text key={index}>{part}</Text>
      )
    );
  };

  const renderSearchResult = ({
    item,
  }: {
    item: { message: Message; sessionId: string };
  }) => {
    const { message, sessionId } = item;
    const sessionName = getSessionName(sessionId);

    return (
      <TouchableOpacity
        style={styles.resultItem}
        onPress={() => handleResultPress(sessionId, message.id)}
      >
        <View style={styles.resultHeader}>
          <Text style={styles.sessionName}>{sessionName}</Text>
          <Text style={styles.resultTime}>
            {new Date(message.sentAt * 1000).toLocaleDateString()}
          </Text>
        </View>
        <Text style={styles.resultContent} numberOfLines={2}>
          {highlightText(message.content, searchQuery)}
        </Text>
      </TouchableOpacity>
    );
  };

  const renderEmpty = () => {
    if (isSearching) {
      return (
        <View style={styles.emptyContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
          <Text style={styles.emptyText}>搜索中...</Text>
        </View>
      );
    }

    if (searchQuery && searchResults.length === 0) {
      return (
        <View style={styles.emptyContainer}>
          <Ionicons name="search-outline" size={64} color="#ccc" />
          <Text style={styles.emptyText}>未找到相关消息</Text>
        </View>
      );
    }

    return (
      <View style={styles.emptyContainer}>
        <Ionicons name="chatbubbles-outline" size={64} color="#ccc" />
        <Text style={styles.emptyText}>搜索聊天记录</Text>
        <Text style={styles.emptyHint}>输入关键词搜索消息内容</Text>
      </View>
    );
  };

  return (
    <View style={styles.container}>
      {/* 搜索栏 */}
      <View style={styles.searchBar}>
        <Pressable style={styles.backBtn} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>

        <View style={styles.searchInputContainer}>
          <Ionicons name="search" size={20} color="#999" />
          <TextInput
            style={styles.searchInput}
            value={searchQuery}
            onChangeText={handleSearch}
            placeholder="搜索消息"
            placeholderTextColor="#999"
            autoFocus
            returnKeyType="search"
          />
          {searchQuery.length > 0 && (
            <TouchableOpacity onPress={handleClear}>
              <Ionicons name="close-circle" size={20} color="#999" />
            </TouchableOpacity>
          )}
        </View>
      </View>

      {/* 搜索结果 */}
      <FlatList
        data={searchResults}
        renderItem={renderSearchResult}
        keyExtractor={(item) => `${item.sessionId}-${item.message.id}`}
        ListEmptyComponent={renderEmpty}
        contentContainerStyle={
          searchResults.length === 0 ? styles.emptyList : undefined
        }
      />

      {/* 搜索统计 */}
      {searchResults.length > 0 && (
        <View style={styles.footer}>
          <Text style={styles.footerText}>
            找到 {searchResults.length} 条相关消息
          </Text>
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  searchBar: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingTop: 50,
    paddingHorizontal: 8,
    paddingBottom: 8,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#E0E0E0',
  },
  backBtn: {
    padding: 8,
  },
  searchInputContainer: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#F5F5F5',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 8,
    gap: 8,
  },
  searchInput: {
    flex: 1,
    fontSize: 16,
    color: '#333',
  },
  emptyList: {
    flex: 1,
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    gap: 16,
    paddingHorizontal: 32,
  },
  emptyText: {
    fontSize: 16,
    color: '#999',
  },
  emptyHint: {
    fontSize: 14,
    color: '#ccc',
  },
  resultItem: {
    backgroundColor: '#FFF',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  resultHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  sessionName: {
    fontSize: 15,
    fontWeight: '600',
    color: '#333',
  },
  resultTime: {
    fontSize: 12,
    color: '#999',
  },
  resultContent: {
    fontSize: 14,
    color: '#666',
    lineHeight: 20,
  },
  highlight: {
    backgroundColor: THEME_COLOR + '30',
    color: THEME_COLOR,
    fontWeight: '600',
  },
  footer: {
    backgroundColor: '#FFF',
    paddingVertical: 12,
    paddingHorizontal: 16,
    borderTopWidth: 1,
    borderTopColor: '#E0E0E0',
    alignItems: 'center',
  },
  footerText: {
    fontSize: 13,
    color: '#999',
  },
});
