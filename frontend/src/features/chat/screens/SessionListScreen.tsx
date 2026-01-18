/**
 * 会话列表页面
 */

import React, { useEffect, useCallback } from 'react';
import {
  View,
  FlatList,
  StyleSheet,
  Text,
  RefreshControl,
  TouchableOpacity,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { useChatStore } from '@/stores/chat.store';
import { SessionItem } from '../components/SessionItem';
import { BottomNavBar } from '@/components/BottomNavBar';
import type { Session } from '../types';

export function SessionListScreen() {
  const router = useRouter();
  const insets = useSafeAreaInsets();
  const { sessions, isLoading, totalUnread, loadSessions, selectSession } =
    useChatStore();

  useEffect(() => {
    loadSessions();
  }, [loadSessions]);

  const handleSessionPress = useCallback(
    (session: Session) => {
      selectSession(session.id);
      router.push(`/chat/${session.id}`);
    },
    [selectSession, router]
  );

  const handleNewChat = useCallback(() => {
    router.push('/chat/new');
  }, [router]);

  const handleSearch = useCallback(() => {
    router.push('/chat/search');
  }, [router]);

  const renderItem = useCallback(
    ({ item }: { item: Session }) => (
      <SessionItem session={item} onPress={() => handleSessionPress(item)} />
    ),
    [handleSessionPress]
  );

  const keyExtractor = useCallback((item: Session) => item.id, []);

  const renderEmpty = () => (
    <View style={styles.empty}>
      <Ionicons name="chatbubbles-outline" size={64} color="#ccc" />
      <Text style={styles.emptyText}>暂无会话</Text>
      <Text style={styles.emptySubtext}>开始一段新的对话吧</Text>
      <TouchableOpacity style={styles.newChatBtn} onPress={handleNewChat}>
        <Text style={styles.newChatBtnText}>发起聊天</Text>
      </TouchableOpacity>
    </View>
  );

  // 过滤掉已归档的会话
  const activeSessions = sessions.filter((s) => !s.isArchived);

  return (
    <View style={[styles.container, { paddingTop: insets.top }]}>
      <View style={styles.header}>
        <Text style={styles.title}>消息</Text>
        {totalUnread > 0 && (
          <View style={styles.totalBadge}>
            <Text style={styles.totalBadgeText}>
              {totalUnread > 99 ? '99+' : totalUnread}
            </Text>
          </View>
        )}
        <View style={styles.headerRight}>
          <TouchableOpacity style={styles.headerBtn} onPress={handleSearch}>
            <Ionicons name="search-outline" size={24} color="#007AFF" />
          </TouchableOpacity>
          <TouchableOpacity style={styles.headerBtn} onPress={handleNewChat}>
            <Ionicons name="create-outline" size={24} color="#007AFF" />
          </TouchableOpacity>
        </View>
      </View>

      <FlatList
        data={activeSessions}
        renderItem={renderItem}
        keyExtractor={keyExtractor}
        ListEmptyComponent={renderEmpty}
        refreshControl={
          <RefreshControl refreshing={isLoading} onRefresh={loadSessions} />
        }
        contentContainerStyle={
          activeSessions.length === 0 && styles.emptyContainer
        }
      />

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="chat" />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#e0e0e0',
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    color: '#333',
  },
  totalBadge: {
    marginLeft: 8,
    backgroundColor: '#FF3B30',
    borderRadius: 10,
    paddingHorizontal: 8,
    paddingVertical: 2,
  },
  totalBadgeText: {
    color: '#fff',
    fontSize: 12,
    fontWeight: '600',
  },
  headerRight: {
    flex: 1,
    flexDirection: 'row',
    justifyContent: 'flex-end',
  },
  headerBtn: {
    padding: 4,
  },
  emptyContainer: {
    flex: 1,
  },
  empty: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingVertical: 60,
  },
  emptyText: {
    fontSize: 18,
    color: '#333',
    marginTop: 16,
    marginBottom: 8,
  },
  emptySubtext: {
    fontSize: 14,
    color: '#999',
    marginBottom: 24,
  },
  newChatBtn: {
    backgroundColor: '#007AFF',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  newChatBtnText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
