import { useRouter } from 'expo-router';
import { Package, Truck, CheckCircle, XCircle, Clock, AlertCircle } from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import type { MallOrder, OrderStatus } from '@/src/types/sharemall';
import { useWalletStore } from '@/src/stores/wallet';

const STATUS_CONFIG: Record<OrderStatus, { label: string; color: string; icon: any }> = {
  Created: { label: '待支付', color: '#F57C00', icon: Clock },
  Paid: { label: '待发货', color: '#1976D2', icon: Package },
  Shipped: { label: '已发货', color: '#7B1FA2', icon: Truck },
  Completed: { label: '已完成', color: '#388E3C', icon: CheckCircle },
  Cancelled: { label: '已取消', color: '#757575', icon: XCircle },
  Disputed: { label: '争议中', color: '#D32F2F', icon: AlertCircle },
  Refunded: { label: '已退款', color: '#616161', icon: XCircle },
  Expired: { label: '已过期', color: '#9E9E9E', icon: Clock },
};

type TabKey = 'all' | 'pending' | 'shipped' | 'completed';

export default function OrdersScreen() {
  const router = useRouter();
  const { currentAccount } = useWalletStore();
  const [orders, setOrders] = useState<MallOrder[]>([]);
  const [refreshing, setRefreshing] = useState(false);
  const [activeTab, setActiveTab] = useState<TabKey>('all');

  const loadOrders = async () => {
    if (!currentAccount) return;
    try {
      const list = await sharemallService.getBuyerOrders(currentAccount.address);
      setOrders(list);
    } catch (error) {
      console.error('Failed to load orders:', error);
    }
  };

  useEffect(() => {
    loadOrders();
  }, [currentAccount]);

  const onRefresh = async () => {
    setRefreshing(true);
    await loadOrders();
    setRefreshing(false);
  };

  const formatPrice = (price: string) => {
    const num = BigInt(price);
    return (Number(num) / 1e10).toFixed(2);
  };

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const filteredOrders = orders.filter((order) => {
    switch (activeTab) {
      case 'pending':
        return order.status === 'Created' || order.status === 'Paid';
      case 'shipped':
        return order.status === 'Shipped';
      case 'completed':
        return order.status === 'Completed';
      default:
        return true;
    }
  });

  const tabs: { key: TabKey; label: string }[] = [
    { key: 'all', label: '全部' },
    { key: 'pending', label: '待处理' },
    { key: 'shipped', label: '待收货' },
    { key: 'completed', label: '已完成' },
  ];

  const renderOrderItem = ({ item }: { item: MallOrder }) => {
    const statusConfig = STATUS_CONFIG[item.status];
    const StatusIcon = statusConfig.icon;

    return (
      <TouchableOpacity
        style={styles.orderCard}
        onPress={() => router.push(`/mall/order/${item.id}` as any)}
      >
        <View style={styles.orderHeader}>
          <Text style={styles.orderId}>订单号: {item.id}</Text>
          <View style={[styles.statusBadge, { backgroundColor: statusConfig.color + '20' }]}>
            <StatusIcon size={12} color={statusConfig.color} />
            <Text style={[styles.statusText, { color: statusConfig.color }]}>
              {statusConfig.label}
            </Text>
          </View>
        </View>

        <View style={styles.orderContent}>
          <View style={styles.productInfo}>
            <View style={styles.productImage}>
              <Package size={24} color="#ccc" />
            </View>
            <View style={styles.productDetail}>
              <Text style={styles.productName}>商品 #{item.productId}</Text>
              <Text style={styles.productQuantity}>x{item.quantity}</Text>
            </View>
          </View>
          <Text style={styles.orderPrice}>¥{formatPrice(item.totalAmount)}</Text>
        </View>

        <View style={styles.orderFooter}>
          <Text style={styles.orderTime}>{formatTime(item.createdAt)}</Text>
          <View style={styles.orderActions}>
            {item.status === 'Shipped' && (
              <TouchableOpacity style={styles.actionBtn}>
                <Text style={styles.actionBtnText}>确认收货</Text>
              </TouchableOpacity>
            )}
            {item.status === 'Completed' && (
              <TouchableOpacity style={styles.actionBtnOutline}>
                <Text style={styles.actionBtnOutlineText}>评价</Text>
              </TouchableOpacity>
            )}
            {item.status === 'Created' && (
              <TouchableOpacity style={styles.actionBtn}>
                <Text style={styles.actionBtnText}>去支付</Text>
              </TouchableOpacity>
            )}
          </View>
        </View>
      </TouchableOpacity>
    );
  };

  return (
    <View style={styles.container}>
      {/* 标签栏 */}
      <View style={styles.tabBar}>
        {tabs.map((tab) => (
          <TouchableOpacity
            key={tab.key}
            style={[styles.tab, activeTab === tab.key && styles.activeTab]}
            onPress={() => setActiveTab(tab.key)}
          >
            <Text style={[styles.tabText, activeTab === tab.key && styles.activeTabText]}>
              {tab.label}
            </Text>
          </TouchableOpacity>
        ))}
      </View>

      <FlatList
        data={filteredOrders}
        renderItem={renderOrderItem}
        keyExtractor={(item) => item.id.toString()}
        contentContainerStyle={styles.listContent}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
        ListEmptyComponent={
          <View style={styles.emptyContainer}>
            <Package size={48} color="#ccc" />
            <Text style={styles.emptyText}>暂无订单</Text>
          </View>
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
  tabBar: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#eee',
  },
  tab: {
    flex: 1,
    paddingVertical: 14,
    alignItems: 'center',
  },
  activeTab: {
    borderBottomWidth: 2,
    borderBottomColor: '#1976D2',
  },
  tabText: {
    fontSize: 14,
    color: '#666',
  },
  activeTabText: {
    color: '#1976D2',
    fontWeight: '500',
  },
  listContent: {
    padding: 12,
  },
  orderCard: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  orderHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  orderId: {
    fontSize: 13,
    color: '#666',
  },
  statusBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
  },
  statusText: {
    fontSize: 12,
    marginLeft: 4,
  },
  orderContent: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 12,
    borderTopWidth: 1,
    borderTopColor: '#f5f5f5',
    borderBottomWidth: 1,
    borderBottomColor: '#f5f5f5',
  },
  productInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    flex: 1,
  },
  productImage: {
    width: 60,
    height: 60,
    borderRadius: 8,
    backgroundColor: '#f5f5f5',
    alignItems: 'center',
    justifyContent: 'center',
  },
  productDetail: {
    marginLeft: 12,
    flex: 1,
  },
  productName: {
    fontSize: 14,
    color: '#333',
    marginBottom: 4,
  },
  productQuantity: {
    fontSize: 12,
    color: '#999',
  },
  orderPrice: {
    fontSize: 16,
    fontWeight: '600',
    color: '#E53935',
  },
  orderFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginTop: 12,
  },
  orderTime: {
    fontSize: 12,
    color: '#999',
  },
  orderActions: {
    flexDirection: 'row',
  },
  actionBtn: {
    backgroundColor: '#1976D2',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 16,
    marginLeft: 8,
  },
  actionBtnText: {
    color: '#fff',
    fontSize: 13,
  },
  actionBtnOutline: {
    borderWidth: 1,
    borderColor: '#1976D2',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 16,
    marginLeft: 8,
  },
  actionBtnOutlineText: {
    color: '#1976D2',
    fontSize: 13,
  },
  emptyContainer: {
    alignItems: 'center',
    paddingVertical: 60,
  },
  emptyText: {
    marginTop: 12,
    fontSize: 14,
    color: '#999',
  },
});
