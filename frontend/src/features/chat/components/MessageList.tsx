/**
 * 消息列表组件
 */

import React, { useRef, useCallback } from 'react';
import {
  FlatList,
  View,
  StyleSheet,
  ActivityIndicator,
  Text,
} from 'react-native';
import { ChatBubble } from './ChatBubble';
import type { Message } from '../types';

interface Props {
  messages: Message[];
  isLoading?: boolean;
  onLoadMore?: () => void;
  hasMore?: boolean;
  onRetry?: (tempId: string) => void;
  onDelete?: (tempId: string) => void;
}

export function MessageList({
  messages,
  isLoading,
  onLoadMore,
  hasMore,
  onRetry,
  onDelete,
}: Props) {
  const listRef = useRef<FlatList>(null);

  const renderItem = useCallback(
    ({ item }: { item: Message }) => (
      <ChatBubble message={item} onRetry={onRetry} onDelete={onDelete} />
    ),
    [onRetry, onDelete]
  );

  const keyExtractor = useCallback(
    (item: Message) => (item.tempId || item.id).toString(),
    []
  );

  const renderHeader = () => {
    if (!isLoading) return null;
    return (
      <View style={styles.loading}>
        <ActivityIndicator size="small" color="#007AFF" />
      </View>
    );
  };

  const renderEmpty = () => {
    if (isLoading) return null;
    return (
      <View style={styles.empty}>
        <Text style={styles.emptyText}>暂无消息</Text>
        <Text style={styles.emptySubtext}>发送第一条消息开始聊天吧</Text>
      </View>
    );
  };

  return (
    <FlatList
      ref={listRef}
      data={messages}
      renderItem={renderItem}
      keyExtractor={keyExtractor}
      contentContainerStyle={[
        styles.content,
        messages.length === 0 && styles.emptyContent,
      ]}
      inverted={false}
      onEndReached={hasMore ? onLoadMore : undefined}
      onEndReachedThreshold={0.1}
      ListHeaderComponent={renderHeader}
      ListEmptyComponent={renderEmpty}
      showsVerticalScrollIndicator={false}
      maintainVisibleContentPosition={{
        minIndexForVisible: 0,
      }}
    />
  );
}

const styles = StyleSheet.create({
  content: {
    paddingVertical: 8,
  },
  emptyContent: {
    flex: 1,
    justifyContent: 'center',
  },
  loading: {
    padding: 16,
    alignItems: 'center',
  },
  empty: {
    alignItems: 'center',
    paddingVertical: 40,
  },
  emptyText: {
    fontSize: 16,
    color: '#666',
    marginBottom: 8,
  },
  emptySubtext: {
    fontSize: 14,
    color: '#999',
  },
});
