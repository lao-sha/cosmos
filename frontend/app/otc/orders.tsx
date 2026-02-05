import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
  RefreshControl,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Clock, CheckCircle, XCircle, AlertTriangle } from 'lucide-react-native';
import { useColors } from '@/hooks/useColors';
import { useOrders, formatCos, formatUsdt } from '@/hooks/useOtc';
import { Card } from '@/components/ui';
import { Colors } from '@/constants/colors';
import type { OtcOrder, OrderStatus } from '@/services/otc';

const STATUS_CONFIG: Record<OrderStatus, { label: string; color: string; icon: any }> = {
  pending: { label: '待支付', color: Colors.warning, icon: Clock },
  paid: { label: '已支付', color: Colors.info, icon: Clock },
  released: { label: '已完成', color: Colors.success, icon: CheckCircle },
  cancelled: { label: '已取消', color: Colors.trading.cancelled, icon: XCircle },
  disputed: { label: '争议中', color: Colors.error, icon: AlertTriangle },
};

export default function OrdersScreen() {
  const colors = useColors();
  const router = useRouter();
  const { data: orders, isLoading, refetch } = useOrders();

  const renderOrder = ({ item }: { item: OtcOrder }) => {
    const config = STATUS_CONFIG[item.status];
    const StatusIcon = config.icon;

    return (
      <TouchableOpacity
        activeOpacity={0.8}
        onPress={() => router.push(`/otc/order/${item.id}`)}
      >
        <Card style={styles.orderCard}>
          <View style={styles.orderHeader}>
            <View style={styles.orderInfo}>
              <Text style={[styles.orderType, { color: colors.textPrimary }]}>
                {item.orderType === 'buy' ? '购买' : '出售'} COS
              </Text>
              <Text style={[styles.orderId, { color: colors.textTertiary }]}>
                #{item.id.slice(0, 8)}
              </Text>
            </View>
            <View style={[styles.statusBadge, { backgroundColor: config.color + '20' }]}>
              <StatusIcon size={14} color={config.color} />
              <Text style={[styles.statusText, { color: config.color }]}>
                {config.label}
              </Text>
            </View>
          </View>

          <View style={styles.orderBody}>
            <View style={styles.amountItem}>
              <Text style={[styles.amountLabel, { color: colors.textSecondary }]}>
                数量
              </Text>
              <Text style={[styles.amountValue, { color: colors.textPrimary }]}>
                {formatCos(item.cosAmount)} COS
              </Text>
            </View>
            <View style={styles.amountItem}>
              <Text style={[styles.amountLabel, { color: colors.textSecondary }]}>
                金额
              </Text>
              <Text style={[styles.amountValue, { color: colors.textPrimary }]}>
                {formatUsdt(item.usdtAmount)} USDT
              </Text>
            </View>
          </View>

          <View style={styles.orderFooter}>
            <Text style={[styles.makerName, { color: colors.textSecondary }]}>
              商家: {item.makerName}
            </Text>
            <Text style={[styles.orderTime, { color: colors.textTertiary }]}>
              {new Date(item.createdAt).toLocaleDateString()}
            </Text>
          </View>
        </Card>
      </TouchableOpacity>
    );
  };

  const pendingOrders = orders?.filter(o => o.status === 'pending' || o.status === 'paid') || [];
  const completedOrders = orders?.filter(o => o.status !== 'pending' && o.status !== 'paid') || [];

  return (
    <View style={[styles.container, { backgroundColor: colors.background }]}>
      <FlatList
        data={orders}
        renderItem={renderOrder}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.list}
        refreshControl={
          <RefreshControl refreshing={isLoading} onRefresh={refetch} />
        }
        ListHeaderComponent={
          pendingOrders.length > 0 ? (
            <View style={styles.sectionHeader}>
              <Text style={[styles.sectionTitle, { color: colors.textPrimary }]}>
                进行中 ({pendingOrders.length})
              </Text>
            </View>
          ) : null
        }
        ListEmptyComponent={
          <View style={styles.empty}>
            <Text style={[styles.emptyText, { color: colors.textSecondary }]}>
              暂无订单
            </Text>
            <TouchableOpacity
              style={[styles.createButton, { backgroundColor: Colors.primary }]}
              onPress={() => router.push('/otc')}
            >
              <Text style={styles.createButtonText}>去购买</Text>
            </TouchableOpacity>
          </View>
        }
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  list: {
    padding: 16,
    gap: 12,
  },
  sectionHeader: {
    marginBottom: 8,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
  },
  orderCard: {
    marginBottom: 0,
  },
  orderHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 12,
  },
  orderInfo: {},
  orderType: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 2,
  },
  orderId: {
    fontSize: 12,
  },
  statusBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
  },
  statusText: {
    fontSize: 12,
    fontWeight: '500',
  },
  orderBody: {
    flexDirection: 'row',
    marginBottom: 12,
  },
  amountItem: {
    flex: 1,
  },
  amountLabel: {
    fontSize: 12,
    marginBottom: 4,
  },
  amountValue: {
    fontSize: 16,
    fontWeight: '600',
  },
  orderFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: 'rgba(0,0,0,0.05)',
  },
  makerName: {
    fontSize: 13,
  },
  orderTime: {
    fontSize: 12,
  },
  empty: {
    alignItems: 'center',
    paddingTop: 80,
  },
  emptyText: {
    fontSize: 16,
    marginBottom: 16,
  },
  createButton: {
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 12,
  },
  createButtonText: {
    color: '#FFFFFF',
    fontSize: 16,
    fontWeight: '600',
  },
});
