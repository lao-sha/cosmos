/**
 * 占卜师仪表盘页面
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  ActivityIndicator,
  RefreshControl,
} from 'react-native';
import { useRouter } from 'expo-router';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import {
  DashboardStats,
  DivinerOrderCard,
  Provider,
  ProviderStatus,
  ProviderTier,
  Order,
  OrderStatus,
  DivinationType,
} from '@/features/diviner';

const THEME_COLOR = '#B2955D';

// Mock 数据
const mockProvider: Provider = {
  account: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
  name: '玄机子',
  bio: '从业20年，专注事业财运分析',
  specialties: 0b0000_0101,
  supportedTypes: 0b0000_0011,
  status: ProviderStatus.Active,
  tier: ProviderTier.Certified,
  totalOrders: 156,
  completedOrders: 148,
  totalEarnings: BigInt(15680 * 1e10),
  averageRating: 4.8,
  ratingCount: 142,
  acceptsUrgent: true,
  registeredAt: Date.now() - 86400000 * 180,
};

const mockOrders: Order[] = [
  {
    id: 1001,
    customer: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
    provider: mockProvider.account,
    packageId: 1,
    divinationType: DivinationType.Meihua,
    questionCid: 'QmXxx...',
    totalAmount: BigInt(10 * 1e10),
    platformFee: BigInt(1.5 * 1e10),
    providerEarnings: BigInt(8.5 * 1e10),
    isUrgent: false,
    status: OrderStatus.Paid,
    createdAt: Date.now() - 3600000,
    followUpsUsed: 0,
    followUpsTotal: 3,
  },
  {
    id: 1002,
    customer: '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy',
    provider: mockProvider.account,
    packageId: 2,
    divinationType: DivinationType.Bazi,
    questionCid: 'QmYyy...',
    totalAmount: BigInt(25 * 1e10),
    platformFee: BigInt(3.75 * 1e10),
    providerEarnings: BigInt(21.25 * 1e10),
    isUrgent: true,
    status: OrderStatus.Accepted,
    createdAt: Date.now() - 7200000,
    acceptedAt: Date.now() - 3600000,
    followUpsUsed: 1,
    followUpsTotal: 5,
  },
];

export default function DivinerDashboardPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [provider, setProvider] = useState<Provider | null>(null);
  const [pendingOrders, setPendingOrders] = useState<Order[]>([]);
  const [availableBalance, setAvailableBalance] = useState(BigInt(0));

  const loadData = async () => {
    // TODO: 从链上加载数据
    await new Promise(resolve => setTimeout(resolve, 800));
    setProvider(mockProvider);
    setPendingOrders(mockOrders);
    setAvailableBalance(BigInt(1250 * 1e10));
  };

  useEffect(() => {
    loadData().finally(() => setLoading(false));
  }, []);

  const onRefresh = async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  };

  const handleAcceptOrder = (orderId: number) => {
    // TODO: 接单逻辑
    console.log('Accept order:', orderId);
  };

  const handleRejectOrder = (orderId: number) => {
    // TODO: 拒单逻辑
    console.log('Reject order:', orderId);
  };

  const handleViewOrder = (orderId: number) => {
    router.push(`/diviner/orders/${orderId}` as any);
  };

  if (loading) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="占卜师中心" />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  if (!provider) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="占卜师中心" />
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyText}>您还不是占卜师</Text>
          <Pressable style={styles.registerBtn} onPress={() => router.push('/diviner' as any)}>
            <Text style={styles.registerBtnText}>立即注册</Text>
          </Pressable>
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  return (
    <View style={styles.wrapper}>
      <PageHeader title="占卜师中心" />

      <ScrollView
        style={styles.container}
        contentContainerStyle={styles.contentContainer}
        refreshControl={<RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor={THEME_COLOR} />}
      >
        {/* 统计卡片 */}
        <View style={styles.section}>
          <DashboardStats
            provider={provider}
            pendingOrders={pendingOrders.filter(o => o.status === OrderStatus.Paid).length}
            todayEarnings={BigInt(85 * 1e10)}
            monthlyEarnings={BigInt(2350 * 1e10)}
            availableBalance={availableBalance}
          />
        </View>

        {/* 快捷操作 */}
        <View style={styles.section}>
          <View style={styles.quickActions}>
            <Pressable style={styles.actionItem} onPress={() => router.push('/diviner/orders' as any)}>
              <Text style={styles.actionIcon}>📋</Text>
              <Text style={styles.actionLabel}>订单管理</Text>
            </Pressable>
            <Pressable style={styles.actionItem} onPress={() => router.push('/diviner/packages' as any)}>
              <Text style={styles.actionIcon}>📦</Text>
              <Text style={styles.actionLabel}>套餐管理</Text>
            </Pressable>
            <Pressable style={styles.actionItem} onPress={() => router.push('/diviner/reviews' as any)}>
              <Text style={styles.actionIcon}>⭐</Text>
              <Text style={styles.actionLabel}>评价管理</Text>
            </Pressable>
            <Pressable style={styles.actionItem} onPress={() => router.push('/diviner/earnings' as any)}>
              <Text style={styles.actionIcon}>💰</Text>
              <Text style={styles.actionLabel}>收益提现</Text>
            </Pressable>
          </View>
        </View>

        {/* 待处理订单 */}
        <View style={styles.section}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>待处理订单</Text>
            <Pressable onPress={() => router.push('/diviner/orders' as any)}>
              <Text style={styles.viewAllText}>查看全部 ›</Text>
            </Pressable>
          </View>

          {pendingOrders.length === 0 ? (
            <View style={styles.emptyOrders}>
              <Text style={styles.emptyOrdersText}>暂无待处理订单</Text>
            </View>
          ) : (
            pendingOrders.map(order => (
              <DivinerOrderCard
                key={order.id}
                order={order}
                onAccept={() => handleAcceptOrder(order.id)}
                onReject={() => handleRejectOrder(order.id)}
                onViewDetail={() => handleViewOrder(order.id)}
              />
            ))
          )}
        </View>

        {/* 更多操作 */}
        <View style={styles.section}>
          <Pressable style={styles.menuItem} onPress={() => router.push('/diviner/profile' as any)}>
            <Text style={styles.menuIcon}>👤</Text>
            <Text style={styles.menuLabel}>编辑资料</Text>
            <Text style={styles.menuArrow}>›</Text>
          </Pressable>
          <Pressable style={styles.menuItem} onPress={() => router.push(`/diviner/${provider.account}` as any)}>
            <Text style={styles.menuIcon}>🔗</Text>
            <Text style={styles.menuLabel}>查看公开主页</Text>
            <Text style={styles.menuArrow}>›</Text>
          </Pressable>
        </View>
      </ScrollView>

      <BottomNavBar activeTab="profile" />
    </View>
  );
}

const styles = StyleSheet.create({
  wrapper: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  container: {
    flex: 1,
  },
  contentContainer: {
    paddingBottom: 100,
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  emptyText: {
    fontSize: 16,
    color: '#666',
    marginBottom: 20,
  },
  registerBtn: {
    backgroundColor: THEME_COLOR,
    paddingHorizontal: 32,
    paddingVertical: 12,
    borderRadius: 8,
  },
  registerBtnText: {
    fontSize: 16,
    color: '#FFF',
    fontWeight: '600',
  },
  section: {
    padding: 16,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#000',
  },
  viewAllText: {
    fontSize: 14,
    color: THEME_COLOR,
  },
  quickActions: {
    flexDirection: 'row',
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
  },
  actionItem: {
    flex: 1,
    alignItems: 'center',
  },
  actionIcon: {
    fontSize: 28,
    marginBottom: 8,
  },
  actionLabel: {
    fontSize: 12,
    color: '#333',
  },
  emptyOrders: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 32,
    alignItems: 'center',
  },
  emptyOrdersText: {
    fontSize: 14,
    color: '#999',
  },
  menuItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 8,
  },
  menuIcon: {
    fontSize: 20,
    marginRight: 12,
  },
  menuLabel: {
    flex: 1,
    fontSize: 16,
    color: '#333',
  },
  menuArrow: {
    fontSize: 20,
    color: '#C7C7CC',
  },
});
