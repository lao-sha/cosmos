// frontend/app/market/provider/dashboard.tsx

import React, { useEffect, useState, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  RefreshControl,
  SafeAreaView,
  StatusBar,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useRouter } from 'expo-router';
import { useWalletStore } from '@/stores/wallet.store';
import { useProvider, useOrders, useChainTransaction } from '@/divination/market/hooks';
import {
  Avatar,
  TierBadge,
  PriceDisplay,
  LoadingSpinner,
  EmptyState,
  OrderStatusBadge,
  DivinationTypeBadge,
} from '@/divination/market/components';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { Order, ProviderDashboard } from '@/divination/market/types';
import { formatBalance, formatTimeAgo, truncateAddress } from '@/divination/market/utils/market.utils';
import { getIpfsUrl } from '@/divination/market/services/ipfs.service';
import { Alert } from 'react-native';

export default function ProviderDashboardScreen() {
  const router = useRouter();
  const { address } = useWalletStore();
  const { isProvider, providerInfo, loading, getDashboard } = useProvider();
  const { getReceivedOrders } = useOrders();
  const { 
    updateProviderProfile, 
    pauseProvider, 
    resumeProvider, 
    withdraw,
    isProcessing 
  } = useChainTransaction();

  const [dashboard, setDashboard] = useState<ProviderDashboard | null>(null);
  const [recentOrders, setRecentOrders] = useState<Order[]>([]);
  const [refreshing, setRefreshing] = useState(false);

  const loadData = useCallback(async () => {
    try {
      const [dashboardData, ordersData] = await Promise.all([
        getDashboard(),
        getReceivedOrders(),
      ]);
      setDashboard(dashboardData);
      setRecentOrders(ordersData.slice(0, 5));
    } catch (err) {
      console.error('Load dashboard error:', err);
    }
  }, [getDashboard, getReceivedOrders]);

  useEffect(() => {
    if (isProvider) {
      loadData();
    }
  }, [isProvider, loadData]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  }, [loadData]);

  const handleUpdateProfile = () => {
    Alert.prompt(
      '更新资料',
      '输入新的简介',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '提交',
          onPress: async (bio) => {
            if (!bio) return;
            await updateProviderProfile({ bio }, {
              onSuccess: () => {
                Alert.alert('成功', '资料已更新');
                loadData();
              }
            });
          }
        }
      ],
      'plain-text',
      providerInfo?.bio
    );
  };

  const handleToggleStatus = () => {
    const isActive = providerInfo?.status === 'Active';
    Alert.alert(
      isActive ? '暂停服务' : '恢复服务',
      isActive ? '暂停后您的套餐将不再出现在搜索结果中' : '恢复后用户可以继续下单',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确定',
          onPress: async () => {
            if (isActive) {
              await pauseProvider({
                onSuccess: () => {
                  Alert.alert('成功', '已暂停服务');
                  loadData();
                }
              });
            } else {
              await resumeProvider({
                onSuccess: () => {
                  Alert.alert('成功', '已恢复服务');
                  loadData();
                }
              });
            }
          }
        }
      ]
    );
  };

  const handleWithdraw = async () => {
    if (!dashboard?.balance || dashboard.balance <= 0n) {
      Alert.alert('提示', '当前没有可提现余额');
      return;
    }

    Alert.prompt(
      '申请提现',
      `当前余额: ${formatBalance(dashboard.balance)} DUST。请输入提现金额 (输入 0 提取全部)`,
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确认提现',
          onPress: async (amountStr) => {
            const amount = parseFloat(amountStr || '0');
            const amountBigInt = amount === 0 ? undefined : BigInt(Math.floor(amount * 1000000));
            
            await withdraw(amountBigInt, {
              onSuccess: () => {
                Alert.alert('成功', '提现申请已提交');
                loadData();
              }
            });
          }
        }
      ],
      'plain-text',
      '0'
    );
  };

  if (!address) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.header}>
          <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
            <Ionicons name="arrow-back" size={24} color={THEME.text} />
          </TouchableOpacity>
          <Text style={styles.headerTitle}>解卦师工作台</Text>
          <View style={styles.backBtn} />
        </View>
        <EmptyState
          icon="wallet-outline"
          title="请先连接钱包"
          actionText="返回"
          onAction={() => router.back()}
        />
      </SafeAreaView>
    );
  }

  if (loading && !providerInfo) {
    return (
      <SafeAreaView style={styles.container}>
        <LoadingSpinner text="加载中..." fullScreen />
      </SafeAreaView>
    );
  }

  if (!isProvider || !providerInfo) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.header}>
          <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
            <Ionicons name="arrow-back" size={24} color={THEME.text} />
          </TouchableOpacity>
          <Text style={styles.headerTitle}>解卦师工作台</Text>
          <View style={styles.backBtn} />
        </View>
        <EmptyState
          icon="person-add-outline"
          title="您还不是解卦师"
          description="立即注册成为解卦师，开始提供占卜服务"
          actionText="立即注册"
          onAction={() => router.push('/market/provider/register')}
        />
      </SafeAreaView>
    );
  }

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="dark-content" backgroundColor={THEME.card} />

      {/* 顶部导航 */}
      <View style={styles.header}>
        <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
          <Ionicons name="arrow-back" size={24} color={THEME.text} />
        </TouchableOpacity>
        <Text style={styles.headerTitle}>解卦师工作台</Text>
        <TouchableOpacity style={styles.backBtn} onPress={handleUpdateProfile}>
          <Ionicons name="settings-outline" size={22} color={THEME.text} />
        </TouchableOpacity>
      </View>

      <ScrollView
        style={styles.content}
        refreshControl={
          <RefreshControl
            refreshing={refreshing}
            onRefresh={onRefresh}
            colors={[THEME.primary]}
            tintColor={THEME.primary}
          />
        }
      >
        {/* 个人信息卡片 */}
        <View style={[styles.profileCard, SHADOWS.medium]}>
          <View style={styles.profileRow}>
            <Avatar
              uri={providerInfo.avatarCid ? getIpfsUrl(providerInfo.avatarCid) : undefined}
              name={providerInfo.name}
              size={56}
            />
            <View style={styles.profileInfo}>
              <View style={styles.nameRow}>
                <Text style={styles.name}>{providerInfo.name}</Text>
                <TierBadge tier={providerInfo.tier} size="small" />
              </View>
              <View style={styles.statusRowSmall}>
                <Text style={styles.ordersInfo}>
                  已完成 {providerInfo.completedOrders} 单
                </Text>
                <TouchableOpacity 
                  style={[styles.statusChip, providerInfo.status === 'Active' ? styles.activeChip : styles.pausedChip]}
                  onPress={handleToggleStatus}
                  disabled={isProcessing}
                >
                  <View style={[styles.statusDot, providerInfo.status === 'Active' ? styles.activeDot : styles.pausedDot]} />
                  <Text style={[styles.statusText, providerInfo.status === 'Active' ? styles.activeText : styles.pausedText]}>
                    {providerInfo.status === 'Active' ? '服务中' : '已暂停'}
                  </Text>
                </TouchableOpacity>
              </View>
            </View>
          </View>
        </View>

        {/* 数据统计 */}
        <View style={[styles.statsCard, SHADOWS.small]}>
          <View style={styles.statsRow}>
            <View style={styles.statItem}>
              <Text style={styles.statValue}>
                {dashboard?.todayStats.newOrders || 0}
              </Text>
              <Text style={styles.statLabel}>今日新单</Text>
            </View>
            <View style={styles.statDivider} />
            <View style={styles.statItem}>
              <Text style={styles.statValue}>
                {dashboard?.todayStats.completed || 0}
              </Text>
              <Text style={styles.statLabel}>今日完成</Text>
            </View>
            <View style={styles.statDivider} />
            <View style={styles.statItem}>
              <Text style={[styles.statValue, { color: THEME.primary }]}>
                {dashboard ? formatBalance(dashboard.todayStats.earnings) : '0'}
              </Text>
              <Text style={styles.statLabel}>今日收入</Text>
            </View>
          </View>
        </View>

        {/* 余额卡片 */}
        <View style={[styles.balanceCard, SHADOWS.small]}>
          <View style={styles.balanceHeader}>
            <Text style={styles.balanceLabel}>可提现余额</Text>
            <TouchableOpacity 
              style={styles.withdrawBtn} 
              onPress={handleWithdraw}
              disabled={isProcessing}
            >
              <Text style={styles.withdrawBtnText}>提现</Text>
            </TouchableOpacity>
          </View>
          <PriceDisplay
            amount={dashboard?.balance || 0n}
            size="large"
          />
        </View>

        {/* 快捷操作 */}
        <View style={styles.quickActions}>
          <TouchableOpacity
            style={[styles.actionBtn, SHADOWS.small]}
            onPress={() => router.push('/market/order/list?type=received&status=Paid')}
          >
            <View style={[styles.actionIcon, { backgroundColor: THEME.warning + '20' }]}>
              <Ionicons name="hourglass-outline" size={22} color={THEME.warning} />
            </View>
            <Text style={styles.actionLabel}>待接单</Text>
            {(dashboard?.pendingOrders.length || 0) > 0 && (
              <View style={styles.badge}>
                <Text style={styles.badgeText}>{dashboard?.pendingOrders.length}</Text>
              </View>
            )}
          </TouchableOpacity>

          <TouchableOpacity
            style={[styles.actionBtn, SHADOWS.small]}
            onPress={() => router.push('/market/order/list?type=received&status=Accepted')}
          >
            <View style={[styles.actionIcon, { backgroundColor: THEME.info + '20' }]}>
              <Ionicons name="document-text-outline" size={22} color={THEME.info} />
            </View>
            <Text style={styles.actionLabel}>进行中</Text>
            {(dashboard?.activeOrders.length || 0) > 0 && (
              <View style={styles.badge}>
                <Text style={styles.badgeText}>{dashboard?.activeOrders.length}</Text>
              </View>
            )}
          </TouchableOpacity>

          <TouchableOpacity
            style={[styles.actionBtn, SHADOWS.small]}
            onPress={() => router.push('/market/provider/packages')}
          >
            <View style={[styles.actionIcon, { backgroundColor: THEME.success + '20' }]}>
              <Ionicons name="pricetag-outline" size={22} color={THEME.success} />
            </View>
            <Text style={styles.actionLabel}>套餐管理</Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={[styles.actionBtn, SHADOWS.small]}
            onPress={() => router.push('/market/order/list?type=received')}
          >
            <View style={[styles.actionIcon, { backgroundColor: THEME.primary + '20' }]}>
              <Ionicons name="list-outline" size={22} color={THEME.primary} />
            </View>
            <Text style={styles.actionLabel}>全部订单</Text>
          </TouchableOpacity>
        </View>

        {/* 最近订单 */}
        <View style={styles.recentSection}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>最近订单</Text>
            <TouchableOpacity onPress={() => router.push('/market/order/list?type=received')}>
              <Text style={styles.sectionMore}>查看全部</Text>
            </TouchableOpacity>
          </View>

          {recentOrders.length === 0 ? (
            <View style={styles.emptyOrders}>
              <Text style={styles.emptyText}>暂无订单</Text>
            </View>
          ) : (
            recentOrders.map((order) => (
              <TouchableOpacity
                key={order.id}
                style={[styles.orderItem, SHADOWS.small]}
                onPress={() => router.push(`/market/order/${order.id}`)}
              >
                <View style={styles.orderHeader}>
                  <DivinationTypeBadge type={order.divinationType} size="small" />
                  <OrderStatusBadge status={order.status} size="small" />
                </View>
                <View style={styles.orderInfo}>
                  <Text style={styles.orderCustomer}>
                    客户: {truncateAddress(order.customer)}
                  </Text>
                  <Text style={styles.orderTime}>
                    {formatTimeAgo(order.createdAt)}
                  </Text>
                </View>
                <View style={styles.orderFooter}>
                  <PriceDisplay amount={order.amount} size="small" />
                  {order.isUrgent && (
                    <View style={styles.urgentTag}>
                      <Ionicons name="flash" size={10} color={THEME.warning} />
                      <Text style={styles.urgentText}>加急</Text>
                    </View>
                  )}
                </View>
              </TouchableOpacity>
            ))
          )}
        </View>

        <View style={styles.bottomSpace} />
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME.background,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: THEME.card,
    paddingHorizontal: 8,
    paddingVertical: 10,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.border,
  },
  backBtn: {
    padding: 8,
    width: 40,
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: THEME.text,
  },
  content: {
    flex: 1,
    padding: 16,
  },
  profileCard: {
    backgroundColor: THEME.card,
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  profileRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  profileInfo: {
    flex: 1,
    marginLeft: 12,
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  name: {
    fontSize: 18,
    fontWeight: '600',
    color: THEME.text,
  },
  ordersInfo: {
    fontSize: 13,
    color: THEME.textSecondary,
  },
  statusRowSmall: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginTop: 6,
  },
  statusChip: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 10,
    gap: 4,
  },
  activeChip: {
    backgroundColor: THEME.success + '15',
  },
  pausedChip: {
    backgroundColor: THEME.warning + '15',
  },
  statusDot: {
    width: 6,
    height: 6,
    borderRadius: 3,
  },
  activeDot: {
    backgroundColor: THEME.success,
  },
  pausedDot: {
    backgroundColor: THEME.warning,
  },
  statusText: {
    fontSize: 11,
    fontWeight: '500',
  },
  activeText: {
    color: THEME.success,
  },
  pausedText: {
    color: THEME.warning,
  },
  statsCard: {
    backgroundColor: THEME.card,
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  statsRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  statItem: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 22,
    fontWeight: '600',
    color: THEME.text,
  },
  statLabel: {
    fontSize: 12,
    color: THEME.textTertiary,
    marginTop: 4,
  },
  statDivider: {
    width: 1,
    height: 30,
    backgroundColor: THEME.border,
  },
  balanceCard: {
    backgroundColor: THEME.primary,
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  balanceHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  balanceLabel: {
    fontSize: 13,
    color: THEME.textInverse,
    opacity: 0.8,
  },
  withdrawBtn: {
    backgroundColor: 'rgba(255,255,255,0.2)',
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 12,
  },
  withdrawBtnText: {
    fontSize: 12,
    color: THEME.textInverse,
  },
  quickActions: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 12,
    marginBottom: 24,
  },
  actionBtn: {
    width: '47%',
    backgroundColor: THEME.card,
    borderRadius: 12,
    padding: 16,
    alignItems: 'center',
    position: 'relative',
  },
  actionIcon: {
    width: 44,
    height: 44,
    borderRadius: 22,
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 8,
  },
  actionLabel: {
    fontSize: 13,
    color: THEME.text,
  },
  badge: {
    position: 'absolute',
    top: 10,
    right: 10,
    backgroundColor: THEME.error,
    borderRadius: 10,
    minWidth: 20,
    height: 20,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 6,
  },
  badgeText: {
    fontSize: 11,
    color: THEME.textInverse,
    fontWeight: '600',
  },
  recentSection: {
    marginBottom: 16,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.text,
  },
  sectionMore: {
    fontSize: 13,
    color: THEME.primary,
  },
  emptyOrders: {
    backgroundColor: THEME.card,
    borderRadius: 12,
    padding: 32,
    alignItems: 'center',
  },
  emptyText: {
    fontSize: 14,
    color: THEME.textTertiary,
  },
  orderItem: {
    backgroundColor: THEME.card,
    borderRadius: 10,
    padding: 14,
    marginBottom: 10,
  },
  orderHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 10,
  },
  orderInfo: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  orderCustomer: {
    fontSize: 13,
    color: THEME.textSecondary,
  },
  orderTime: {
    fontSize: 12,
    color: THEME.textTertiary,
  },
  orderFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  urgentTag: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: THEME.warning + '15',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
    gap: 2,
  },
  urgentText: {
    fontSize: 10,
    color: THEME.warning,
    fontWeight: '500',
  },
  bottomSpace: {
    height: 32,
  },
});
