import { useUserSwaps } from '@/src/hooks/useSwap';
import { swapService, SwapRecord, SwapStatus } from '@/src/services/swap';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
  ActivityIndicator,
  Pressable,
  RefreshControl,
  ScrollView,
  StyleSheet,
  Text,
  View
} from 'react-native';

export default function SwapHistoryScreen() {
  const router = useRouter();
  const { isLoggedIn } = useAuthStore();
  const { isConnected } = useChainStore();
  const { swaps, loading, refresh } = useUserSwaps();
  const [refreshing, setRefreshing] = useState(false);
  const [filter, setFilter] = useState<'all' | 'active' | 'completed'>('all');

  const handleRefresh = async () => {
    setRefreshing(true);
    await refresh();
    setRefreshing(false);
  };

  const filteredSwaps = swaps.filter(swap => {
    if (filter === 'all') return true;
    if (filter === 'active') {
      return [SwapStatus.Pending, SwapStatus.AwaitingVerification, SwapStatus.UserReported, SwapStatus.Arbitrating].includes(swap.status);
    }
    if (filter === 'completed') {
      return [SwapStatus.Completed, SwapStatus.Refunded, SwapStatus.VerificationFailed, SwapStatus.ArbitrationApproved, SwapStatus.ArbitrationRejected].includes(swap.status);
    }
    return true;
  });

  if (!isLoggedIn || !isConnected) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>兑换记录</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>🔌</Text>
          <Text style={styles.emptyTitle}>请先登录并连接网络</Text>
        </View>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>兑换记录</Text>
        <View style={styles.headerRight} />
      </View>

      <View style={styles.filterBar}>
        <Pressable
          style={[styles.filterButton, filter === 'all' && styles.filterButtonActive]}
          onPress={() => setFilter('all')}
        >
          <Text style={[styles.filterText, filter === 'all' && styles.filterTextActive]}>
            全部
          </Text>
        </Pressable>
        <Pressable
          style={[styles.filterButton, filter === 'active' && styles.filterButtonActive]}
          onPress={() => setFilter('active')}
        >
          <Text style={[styles.filterText, filter === 'active' && styles.filterTextActive]}>
            进行中
          </Text>
        </Pressable>
        <Pressable
          style={[styles.filterButton, filter === 'completed' && styles.filterButtonActive]}
          onPress={() => setFilter('completed')}
        >
          <Text style={[styles.filterText, filter === 'completed' && styles.filterTextActive]}>
            已完成
          </Text>
        </Pressable>
      </View>

      <ScrollView
        style={styles.content}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} />
        }
      >
        {loading && swaps.length === 0 ? (
          <View style={styles.loadingContainer}>
            <ActivityIndicator size="large" color="#10b981" />
            <Text style={styles.loadingText}>加载中...</Text>
          </View>
        ) : filteredSwaps.length === 0 ? (
          <View style={styles.emptySwaps}>
            <Text style={styles.emptySwapsIcon}>📭</Text>
            <Text style={styles.emptySwapsText}>暂无兑换记录</Text>
            <Pressable
              style={styles.createButton}
              onPress={() => router.push('/swap')}
            >
              <Text style={styles.createButtonText}>去兑换</Text>
            </Pressable>
          </View>
        ) : (
          filteredSwaps.map((swap) => (
            <Pressable
              key={swap.swapId}
              style={({ pressed }) => [
                styles.swapCard,
                pressed && styles.swapCardPressed,
              ]}
              onPress={() => router.push(`/swap/${swap.swapId}`)}
            >
              <View style={styles.swapHeader}>
                <Text style={styles.swapId}>兑换 #{swap.swapId}</Text>
                <View style={[
                  styles.statusBadge,
                  { backgroundColor: swapService.getStatusColor(swap.status) }
                ]}>
                  <Text style={styles.statusText}>
                    {swapService.getStatusText(swap.status)}
                  </Text>
                </View>
              </View>

              <View style={styles.swapInfo}>
                <View style={styles.infoRow}>
                  <Text style={styles.infoLabel}>兑出</Text>
                  <Text style={styles.infoValue}>{swapService.formatCos(swap.cosAmount)}</Text>
                </View>
                <View style={styles.infoArrow}>
                  <Text style={styles.arrowText}>→</Text>
                </View>
                <View style={styles.infoRow}>
                  <Text style={styles.infoLabel}>收到</Text>
                  <Text style={styles.infoValueGreen}>{swapService.formatUsdt(swap.usdtAmount)}</Text>
                </View>
              </View>

              <View style={styles.swapFooter}>
                <Text style={styles.swapTime}>
                  区块 #{swap.createdAt}
                </Text>
                <Text style={styles.makerInfo}>
                  做市商 #{swap.makerId}
                </Text>
              </View>
            </Pressable>
          ))
        )}

        <View style={styles.bottomPadding} />
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    backgroundColor: '#10b981',
    paddingTop: 50,
    paddingBottom: 16,
    paddingHorizontal: 16,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  backButton: {
    padding: 4,
  },
  backText: {
    color: '#fff',
    fontSize: 18,
  },
  headerTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#fff',
  },
  headerRight: {
    width: 40,
  },
  filterBar: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    paddingHorizontal: 16,
    paddingVertical: 12,
    gap: 12,
  },
  filterButton: {
    paddingVertical: 8,
    paddingHorizontal: 16,
    borderRadius: 20,
    backgroundColor: '#f3f4f6',
  },
  filterButtonActive: {
    backgroundColor: '#10b981',
  },
  filterText: {
    fontSize: 14,
    color: '#6b7280',
  },
  filterTextActive: {
    color: '#fff',
    fontWeight: '500',
  },
  content: {
    flex: 1,
  },
  loadingContainer: {
    alignItems: 'center',
    paddingVertical: 60,
  },
  loadingText: {
    marginTop: 12,
    fontSize: 14,
    color: '#6b7280',
  },
  emptyContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
  },
  emptyIcon: {
    fontSize: 64,
    marginBottom: 16,
  },
  emptyTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#1f2937',
  },
  emptySwaps: {
    alignItems: 'center',
    paddingVertical: 60,
  },
  emptySwapsIcon: {
    fontSize: 64,
    marginBottom: 16,
  },
  emptySwapsText: {
    fontSize: 16,
    color: '#6b7280',
    marginBottom: 20,
  },
  createButton: {
    backgroundColor: '#10b981',
    paddingVertical: 12,
    paddingHorizontal: 32,
    borderRadius: 24,
  },
  createButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  swapCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 12,
    padding: 16,
    borderRadius: 12,
  },
  swapCardPressed: {
    backgroundColor: '#f9fafb',
  },
  swapHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  swapId: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
  },
  statusBadge: {
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
  },
  statusText: {
    fontSize: 12,
    color: '#fff',
    fontWeight: '500',
  },
  swapInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 12,
    borderTopWidth: 1,
    borderBottomWidth: 1,
    borderColor: '#f3f4f6',
  },
  infoRow: {
    alignItems: 'center',
    flex: 1,
  },
  infoArrow: {
    paddingHorizontal: 12,
  },
  arrowText: {
    fontSize: 20,
    color: '#9ca3af',
  },
  infoLabel: {
    fontSize: 12,
    color: '#9ca3af',
    marginBottom: 4,
  },
  infoValue: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
  },
  infoValueGreen: {
    fontSize: 16,
    fontWeight: '600',
    color: '#10b981',
  },
  swapFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginTop: 12,
  },
  swapTime: {
    fontSize: 12,
    color: '#9ca3af',
  },
  makerInfo: {
    fontSize: 12,
    color: '#6b7280',
  },
  bottomPadding: {
    height: 40,
  },
});
