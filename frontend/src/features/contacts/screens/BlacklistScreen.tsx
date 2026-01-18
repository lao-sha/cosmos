/**
 * 黑名单管理页面
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
import { BlockedUserItem } from '../components/BlockedUserItem';
import type { BlockedUser } from '../types';

export function BlacklistScreen() {
  const router = useRouter();
  const insets = useSafeAreaInsets();
  const { blacklist, isLoading, loadBlacklist, unblockUser } =
    useContactsStore();

  useEffect(() => {
    loadBlacklist();
  }, []);

  const handleBack = useCallback(() => {
    router.back();
  }, [router]);

  const handleUnblock = useCallback(
    async (address: string) => {
      Alert.alert('确认解除', '确定要解除对该用户的拉黑吗？', [
        { text: '取消', style: 'cancel' },
        {
          text: '解除',
          onPress: async () => {
            try {
              await unblockUser(address);
              Alert.alert('成功', '已解除拉黑');
            } catch (error) {
              Alert.alert('失败', (error as Error).message);
            }
          },
        },
      ]);
    },
    [unblockUser]
  );

  const renderItem = useCallback(
    ({ item }: { item: BlockedUser }) => (
      <BlockedUserItem
        blockedUser={item}
        onUnblock={() => handleUnblock(item.address)}
      />
    ),
    [handleUnblock]
  );

  const renderEmpty = () => (
    <View style={styles.empty}>
      <Ionicons name="ban-outline" size={64} color="#ccc" />
      <Text style={styles.emptyText}>黑名单为空</Text>
      <Text style={styles.emptySubtext}>被拉黑的用户会显示在这里</Text>
    </View>
  );

  return (
    <View style={[styles.container, { paddingTop: insets.top }]}>
      <View style={styles.header}>
        <TouchableOpacity style={styles.backBtn} onPress={handleBack}>
          <Ionicons name="chevron-back" size={28} color="#007AFF" />
        </TouchableOpacity>
        <Text style={styles.title}>黑名单</Text>
        <View style={styles.placeholder} />
      </View>

      <View style={styles.tip}>
        <Ionicons name="information-circle-outline" size={18} color="#666" />
        <Text style={styles.tipText}>
          被拉黑的用户无法给您发送消息，您也无法收到对方的消息
        </Text>
      </View>

      <FlatList
        data={blacklist}
        renderItem={renderItem}
        keyExtractor={(item) => item.address}
        ListEmptyComponent={renderEmpty}
        refreshControl={
          <RefreshControl refreshing={isLoading} onRefresh={loadBlacklist} />
        }
        contentContainerStyle={
          blacklist.length === 0 ? styles.emptyContainer : undefined
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
  tip: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FFF9E6',
    padding: 12,
    gap: 8,
  },
  tipText: {
    flex: 1,
    fontSize: 13,
    color: '#666',
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
});
