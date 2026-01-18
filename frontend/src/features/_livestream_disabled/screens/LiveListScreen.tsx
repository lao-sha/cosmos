// frontend/src/features/livestream/screens/LiveListScreen.tsx

import React, { useEffect, useCallback, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
  RefreshControl,
  ActivityIndicator,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useLivestreamStore } from '@/stores/livestream.store';
import { RoomCard } from '../components/RoomCard';
import type { LiveRoom, RoomFilter } from '../types';
import { getLivestreamService, initLivestreamService } from '../services/livestream.service';
import { useWalletStore } from '@/stores/wallet.store';

const FILTERS: { key: RoomFilter; label: string }[] = [
  { key: 'all', label: '全部' },
  { key: 'Paid', label: '付费' },
  { key: 'MultiHost', label: '连麦' },
  { key: 'Private', label: '私密' },
];

export function LiveListScreen() {
  const router = useRouter();
  const { currentWallet } = useWalletStore();
  const {
    rooms,
    isLoadingRooms,
    roomFilter,
    setRoomFilter,
    setRooms,
    setIsLoadingRooms,
  } = useLivestreamStore();

  const [isRefreshing, setIsRefreshing] = useState(false);

  const loadRooms = useCallback(async () => {
    if (!currentWallet?.address) return;

    setIsLoadingRooms(true);
    try {
      const service = initLivestreamService(currentWallet.address);
      await service.init();
      const data = await service.getLiveRooms(roomFilter === 'all' ? undefined : roomFilter);
      setRooms(data);
    } catch (error) {
      console.error('加载直播列表失败:', error);
    } finally {
      setIsLoadingRooms(false);
    }
  }, [currentWallet?.address, roomFilter, setRooms, setIsLoadingRooms]);

  useEffect(() => {
    loadRooms();
  }, [loadRooms]);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    await loadRooms();
    setIsRefreshing(false);
  };

  const handleRoomPress = (room: LiveRoom) => {
    router.push(`/livestream/${room.id}`);
  };

  const handleStartLive = () => {
    router.push('/livestream/create');
  };

  const renderRoom = ({ item }: { item: LiveRoom }) => (
    <RoomCard room={item} onPress={() => handleRoomPress(item)} />
  );

  const renderHeader = () => (
    <View style={styles.filterContainer}>
      {FILTERS.map((filter) => (
        <TouchableOpacity
          key={filter.key}
          style={[
            styles.filterBtn,
            roomFilter === filter.key && styles.filterBtnActive,
          ]}
          onPress={() => setRoomFilter(filter.key)}
        >
          <Text
            style={[
              styles.filterText,
              roomFilter === filter.key && styles.filterTextActive,
            ]}
          >
            {filter.label}
          </Text>
        </TouchableOpacity>
      ))}
    </View>
  );

  const renderEmpty = () => (
    <View style={styles.emptyContainer}>
      <Ionicons name="videocam-off-outline" size={64} color="#666" />
      <Text style={styles.emptyText}>暂无直播</Text>
      <Text style={styles.emptySubtext}>快来开启第一场直播吧</Text>
    </View>
  );

  return (
    <SafeAreaView style={styles.container} edges={['top']}>
      <View style={styles.header}>
        <Text style={styles.title}>直播大厅</Text>
        <View style={styles.headerRight}>
          <TouchableOpacity style={styles.searchBtn}>
            <Ionicons name="search" size={22} color="#FFF" />
          </TouchableOpacity>
          <TouchableOpacity style={styles.startBtn} onPress={handleStartLive}>
            <Text style={styles.startBtnText}>开播</Text>
          </TouchableOpacity>
        </View>
      </View>

      {isLoadingRooms && rooms.length === 0 ? (
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color="#FF4757" />
          <Text style={styles.loadingText}>加载中...</Text>
        </View>
      ) : (
        <FlatList
          data={rooms}
          renderItem={renderRoom}
          keyExtractor={(item) => item.id.toString()}
          numColumns={2}
          columnWrapperStyle={styles.row}
          contentContainerStyle={styles.listContent}
          ListHeaderComponent={renderHeader}
          ListEmptyComponent={renderEmpty}
          refreshControl={
            <RefreshControl
              refreshing={isRefreshing}
              onRefresh={handleRefresh}
              tintColor="#FF4757"
              colors={['#FF4757']}
            />
          }
          showsVerticalScrollIndicator={false}
        />
      )}
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0F0F1A',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
  },
  title: {
    color: '#FFF',
    fontSize: 24,
    fontWeight: 'bold',
  },
  headerRight: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  searchBtn: {
    padding: 8,
  },
  startBtn: {
    backgroundColor: '#FF4757',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 16,
  },
  startBtnText: {
    color: '#FFF',
    fontSize: 14,
    fontWeight: '600',
  },
  filterContainer: {
    flexDirection: 'row',
    paddingHorizontal: 12,
    paddingVertical: 12,
    gap: 8,
  },
  filterBtn: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 16,
    backgroundColor: '#252540',
  },
  filterBtnActive: {
    backgroundColor: '#FF4757',
  },
  filterText: {
    color: '#999',
    fontSize: 14,
  },
  filterTextActive: {
    color: '#FFF',
    fontWeight: '600',
  },
  listContent: {
    paddingHorizontal: 12,
    paddingBottom: 20,
  },
  row: {
    justifyContent: 'space-between',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  loadingText: {
    color: '#999',
    marginTop: 12,
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingTop: 100,
  },
  emptyText: {
    color: '#FFF',
    fontSize: 18,
    marginTop: 16,
  },
  emptySubtext: {
    color: '#666',
    fontSize: 14,
    marginTop: 8,
  },
});
