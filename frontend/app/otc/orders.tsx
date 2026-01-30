import { useBuyerOrders } from '@/src/hooks/useOtc';
import { otcService, OtcOrder, OrderState } from '@/src/services/otc';
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

export default function OtcOrdersScreen() {
  const router = useRouter();
  const { isLoggedIn } = useAuthStore();
  const { isConnected } = useChainStore();
  const { orders, loading, refresh } = useBuyerOrders();
  const [refreshing, setRefreshing] = useState(false);
  const [filter, setFilter] = useState<'all' | 'active' | 'completed'>('all');

  const handleRefresh = async () => {
    setRefreshing(true);
    await refresh();
    setRefreshing(false);
  };

  const filteredOrders = orders.filter(order => {
    if (filter === 'all') return true;
    if (filter === 'active') {
      return [OrderState.Created, OrderState.PaidOrCommitted, OrderState.Disputed].includes(order.state);
    }
    if (filter === 'completed') {
      return [OrderState.Released, OrderState.Canceled, OrderState.Refunded, OrderState.Expired].includes(order.state);
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
          <Text style={styles.headerTitle}>我的订单</Text>
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
        <Text style={styles.headerTitle}>我的订单</Text>
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
        {loading && orders.length === 0 ? (
          <View style={styles.loadingContainer}>
            <ActivityIndicator size="large" color="#6D28D9" />
            <Text style={styles.loadingText}>加载中...</Text>
          </View>
        ) : filteredOrders.length === 0 ? (
          <View style={styles.emptyOrders}>
            <Text style={styles.emptyOrdersIcon}>📭</Text>
            <Text style={styles.emptyOrdersText}>暂无订单</Text>
            <Pressable
              style={styles.createButton}
              onPress={() => router.push('/otc')}
            >
              <Text style={styles.createButtonText}>去购买</Text>
            </Pressable>
          </View>
        ) : (
          filteredOrders.map((order) => (
            <Pressable
              key={order.orderId}
              style={({ pressed }) => [
                styles.orderCard,
                pressed && styles.orderCardPressed,
              ]}
              onPress={() => router.push(`/otc/${order.orderId}`)}
            >
              <View style={styles.orderHeader}>
                <Text style={styles.orderId}>订单 #{order.orderId}</Text>
                <View style={[
                  styles.statusBadge,
                  { backgroundColor: otcService.getStateColor(order.state) }
                ]}>
                  <Text style={styles.statusText}>
                    {otcService.getStateText(order.state)}
                  </Text>
                </View>
              </View>

              <View style={styles.orderInfo}>
                <View style={styles.infoRow}>
                  <Text style={styles.infoLabel}>数量</Text>
                  <Text style={styles.infoValue}>{otcService.formatCos(order.qty)}</Text>
                </View>
                <View style={styles.infoRow}>
                  <Text style={styles.infoLabel}>金额</Text>
                  <Text style={styles.infoValue}>{otcService.formatUsdt(order.amount)}</Text>
                </View>
              </View>

              <View style={styles.orderFooter}>
                <Text style={styles.orderTime}>
                  {otcService.formatTime(order.createdAt)}
                </Text>
                {order.isFirstPurchase && (
                  <View style={styles.firstPurchaseBadge}>
                    <Text style={styles.firstPurchaseText}>首购</Text>
                  </View>
                )}
              </View>

              {order.state === OrderState.Created && !otcService.isExpired(order.expireAt) && (
                <View style={styles.countdownBar}>
                  <Text style={styles.countdownText}>
                    ⏱️ 剩余 {otcService.getRemainingTime(order.expireAt)}
                  </Text>
                </View>
              )}
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
    backgroundColor: '#6D28D9',
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
    backgroundColor: '#6D28D9',
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
  emptyOrders: {
    alignItems: 'center',
    paddingVertical: 60,
  },
  emptyOrdersIcon: {
    fontSize: 64,
    marginBottom: 16,
  },
  emptyOrdersText: {
    fontSize: 16,
    color: '#6b7280',
    marginBottom: 20,
  },
  createButton: {
    backgroundColor: '#6D28D9',
    paddingVertical: 12,
    paddingHorizontal: 32,
    borderRadius: 24,
  },
  createButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  orderCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 12,
    padding: 16,
    borderRadius: 12,
  },
  orderCardPressed: {
    backgroundColor: '#f9fafb',
  },
  orderHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  orderId: {
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
  orderInfo: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 12,
    borderTopWidth: 1,
    borderBottomWidth: 1,
    borderColor: '#f3f4f6',
  },
  infoRow: {
    alignItems: 'center',
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
  orderFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginTop: 12,
  },
  orderTime: {
    fontSize: 12,
    color: '#9ca3af',
  },
  firstPurchaseBadge: {
    backgroundColor: '#fef3c7',
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 8,
  },
  firstPurchaseText: {
    fontSize: 11,
    color: '#92400e',
    fontWeight: '500',
  },
  countdownBar: {
    backgroundColor: '#fef3c7',
    marginTop: 12,
    padding: 8,
    borderRadius: 8,
    alignItems: 'center',
  },
  countdownText: {
    fontSize: 13,
    color: '#92400e',
    fontWeight: '500',
  },
  bottomPadding: {
    height: 40,
  },
});
