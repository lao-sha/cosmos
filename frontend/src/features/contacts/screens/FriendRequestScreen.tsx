/**
 * 好友申请页面
 */

import React, { useEffect, useCallback } from 'react';
import {
  View,
  FlatList,
  StyleSheet,
  Text,
  RefreshControl,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { useContactsStore } from '@/stores/contacts.store';
import { FriendRequestItem } from '../components/FriendRequestItem';
import type { FriendRequest } from '../types';

export function FriendRequestScreen() {
  const router = useRouter();
  const insets = useSafeAreaInsets();
  const {
    friendRequests,
    isLoading,
    loadFriendRequests,
    acceptFriendRequest,
    rejectFriendRequest,
  } = useContactsStore();

  useEffect(() => {
    loadFriendRequests();
  }, []);

  const handleBack = useCallback(() => {
    router.back();
  }, [router]);

  const handleAccept = useCallback(
    async (requester: string) => {
      try {
        await acceptFriendRequest(requester);
        Alert.alert('成功', '已接受好友申请');
      } catch (error) {
        Alert.alert('失败', (error as Error).message);
      }
    },
    [acceptFriendRequest]
  );

  const handleReject = useCallback(
    async (requester: string) => {
      Alert.alert('确认拒绝', '确定要拒绝这个好友申请吗？', [
        { text: '取消', style: 'cancel' },
        {
          text: '拒绝',
          style: 'destructive',
          onPress: async () => {
            try {
              await rejectFriendRequest(requester);
            } catch (error) {
              Alert.alert('失败', (error as Error).message);
            }
          },
        },
      ]);
    },
    [rejectFriendRequest]
  );

  const renderItem = useCallback(
    ({ item }: { item: FriendRequest }) => (
      <FriendRequestItem
        request={item}
        onAccept={() => handleAccept(item.requester)}
        onReject={() => handleReject(item.requester)}
      />
    ),
    [handleAccept, handleReject]
  );

  const renderEmpty = () => (
    <View style={styles.empty}>
      <Ionicons name="person-add-outline" size={64} color="#ccc" />
      <Text style={styles.emptyText}>暂无好友申请</Text>
      <Text style={styles.emptySubtext}>新的好友申请会显示在这里</Text>
    </View>
  );

  // 分离未过期和已过期的申请
  const activeRequests = friendRequests.filter((r) => !r.isExpired);
  const expiredRequests = friendRequests.filter((r) => r.isExpired);

  return (
    <View style={[styles.container, { paddingTop: insets.top }]}>
      <View style={styles.header}>
        <TouchableOpacity style={styles.backBtn} onPress={handleBack}>
          <Ionicons name="chevron-back" size={28} color="#007AFF" />
        </TouchableOpacity>
        <Text style={styles.title}>好友申请</Text>
        <View style={styles.placeholder} />
      </View>

      <FlatList
        data={activeRequests}
        renderItem={renderItem}
        keyExtractor={(item) => item.requester}
        ListEmptyComponent={renderEmpty}
        ListFooterComponent={
          expiredRequests.length > 0 ? (
            <View style={styles.expiredSection}>
              <Text style={styles.expiredTitle}>
                已过期 ({expiredRequests.length})
              </Text>
              {expiredRequests.map((request) => (
                <FriendRequestItem
                  key={request.requester}
                  request={request}
                  onAccept={() => {}}
                  onReject={() => {}}
                />
              ))}
            </View>
          ) : null
        }
        refreshControl={
          <RefreshControl
            refreshing={isLoading}
            onRefresh={loadFriendRequests}
          />
        }
        contentContainerStyle={
          activeRequests.length === 0 && expiredRequests.length === 0
            ? styles.emptyContainer
            : undefined
        }
      />
    </View>
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
    paddingVertical: 10,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#e0e0e0',
  },
  backBtn: {
    padding: 8,
    marginLeft: 4,
  },
  title: {
    fontSize: 17,
    fontWeight: '600',
    color: '#333',
  },
  placeholder: {
    width: 44,
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
  },
  expiredSection: {
    marginTop: 20,
  },
  expiredTitle: {
    fontSize: 13,
    fontWeight: '600',
    color: '#999',
    paddingHorizontal: 16,
    paddingVertical: 8,
    backgroundColor: '#f5f5f5',
  },
});
