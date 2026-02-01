import { useRouter } from 'expo-router';
import { Package, Truck, CheckCircle, Clock, AlertCircle } from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
  Alert,
  TextInput,
  Modal,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import { sharemallTxService } from '@/src/services/sharemall-tx';
import type { MallOrder, Shop, OrderStatus } from '@/src/types/sharemall';
import { useWalletStore } from '@/src/stores/wallet';

type TabKey = 'all' | 'pending' | 'shipped' | 'completed';

export default function SellerOrdersScreen() {
  const router = useRouter();
  const { currentAccount } = useWalletStore();
  const [orders, setOrders] = useState<MallOrder[]>([]);
  const [myShop, setMyShop] = useState<Shop | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [activeTab, setActiveTab] = useState<TabKey>('all');
  const [shipModalVisible, setShipModalVisible] = useState(false);
  const [selectedOrder, setSelectedOrder] = useState<MallOrder | null>(null);
  const [trackingCid, setTrackingCid] = useState('');

  const loadData = async () => {
    if (!currentAccount) return;
    try {
      const shops = await sharemallService.getShopList(100);
      const userShop = shops.find((s) => s.owner === currentAccount.address);
      setMyShop(userShop || null);

      if (userShop) {
        const list = await sharemallService.getShopOrders(userShop.id);
        setOrders(list);
      }
    } catch (error) {
      console.error('Failed to load orders:', error);
    }
  };

  useEffect(() => {
    loadData();
  }, [currentAccount]);

  const onRefresh = async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  };

  const formatPrice = (price: string) => {
    const num = BigInt(price);
    return (Number(num) / 1e10).toFixed(2);
  };

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const handleShip = (order: MallOrder) => {
    setSelectedOrder(order);
    setTrackingCid('');
    setShipModalVisible(true);
  };

  const confirmShip = async () => {
    if (!selectedOrder || !trackingCid.trim()) {
      Alert.alert('提示', '请输入物流信息');
      return;
    }

    try {
      await sharemallTxService.shipOrder(selectedOrder.id, trackingCid.trim());
      Alert.alert('成功', '发货成功');
      setShipModalVisible(false);
      loadData();
    } catch (error: any) {
      Alert.alert('错误', error.message || '发货失败');
    }
  };

  const handleApproveRefund = async (order: MallOrder) => {
    Alert.alert('确认退款', '确定同意退款申请吗？', [
      { text: '取消', style: 'cancel' },
      {
        text: '确定',
        onPress: async () => {
          try {
            await sharemallTxService.approveRefund(order.id);
            Alert.alert('成功', '退款已处理');
            loadData();
          } catch (error: any) {
            Alert.alert('错误', error.message || '操作失败');
          }
        },
      },
    ]);
  };

  const filteredOrders = orders.filter((order) => {
    switch (activeTab) {
      case 'pending':
        return order.status === 'Paid';
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
    { key: 'pending', label: '待发货' },
    { key: 'shipped', label: '已发货' },
    { key: 'completed', label: '已完成' },
  ];

  const getStatusConfig = (status: OrderStatus) => {
    switch (status) {
      case 'Paid':
        return { label: '待发货', color: '#F57C00', icon: Clock };
      case 'Shipped':
        return { label: '已发货', color: '#7B1FA2', icon: Truck };
      case 'Completed':
        return { label: '已完成', color: '#388E3C', icon: CheckCircle };
      case 'Disputed':
        return { label: '争议中', color: '#D32F2F', icon: AlertCircle };
      default:
        return { label: status, color: '#757575', icon: Package };
    }
  };

  const renderOrderItem = ({ item }: { item: MallOrder }) => {
    const statusConfig = getStatusConfig(item.status);
    const StatusIcon = statusConfig.icon;

    return (
      <View style={styles.orderCard}>
        <View style={styles.orderHeader}>
          <Text style={styles.orderId}>订单 #{item.id}</Text>
          <View style={[styles.statusBadge, { backgroundColor: statusConfig.color + '20' }]}>
            <StatusIcon size={12} color={statusConfig.color} />
            <Text style={[styles.statusText, { color: statusConfig.color }]}>
              {statusConfig.label}
            </Text>
          </View>
        </View>

        <View style={styles.orderContent}>
          <View style={styles.productInfo}>
            <Text style={styles.productName}>商品 #{item.productId}</Text>
            <Text style={styles.productQuantity}>x{item.quantity}</Text>
          </View>
          <Text style={styles.orderPrice}>¥{formatPrice(item.totalAmount)}</Text>
        </View>

        <View style={styles.buyerInfo}>
          <Text style={styles.buyerLabel}>买家:</Text>
          <Text style={styles.buyerAddress}>
            {item.buyer.slice(0, 12)}...{item.buyer.slice(-6)}
          </Text>
        </View>

        <View style={styles.orderFooter}>
          <Text style={styles.orderTime}>{formatTime(item.createdAt)}</Text>
          <View style={styles.orderActions}>
            {item.status === 'Paid' && (
              <TouchableOpacity
                style={styles.actionBtn}
                onPress={() => handleShip(item)}
              >
                <Text style={styles.actionBtnText}>发货</Text>
              </TouchableOpacity>
            )}
            {item.status === 'Disputed' && (
              <TouchableOpacity
                style={[styles.actionBtn, styles.refundBtn]}
                onPress={() => handleApproveRefund(item)}
              >
                <Text style={styles.actionBtnText}>同意退款</Text>
              </TouchableOpacity>
            )}
          </View>
        </View>
      </View>
    );
  };

  return (
    <View style={styles.container}>
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

      {/* 发货弹窗 */}
      <Modal visible={shipModalVisible} transparent animationType="fade">
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <Text style={styles.modalTitle}>发货</Text>
            <Text style={styles.modalLabel}>物流信息 (IPFS CID)</Text>
            <TextInput
              style={styles.modalInput}
              placeholder="输入物流信息的 IPFS CID"
              value={trackingCid}
              onChangeText={setTrackingCid}
            />
            <View style={styles.modalActions}>
              <TouchableOpacity
                style={styles.modalCancelBtn}
                onPress={() => setShipModalVisible(false)}
              >
                <Text style={styles.modalCancelText}>取消</Text>
              </TouchableOpacity>
              <TouchableOpacity style={styles.modalConfirmBtn} onPress={confirmShip}>
                <Text style={styles.modalConfirmText}>确认发货</Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>
      </Modal>
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
    fontSize: 14,
    fontWeight: '500',
    color: '#333',
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
  },
  productInfo: {
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
  buyerInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 8,
  },
  buyerLabel: {
    fontSize: 12,
    color: '#999',
  },
  buyerAddress: {
    fontSize: 12,
    color: '#666',
    marginLeft: 4,
  },
  orderFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginTop: 8,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#f5f5f5',
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
  refundBtn: {
    backgroundColor: '#F57C00',
  },
  actionBtnText: {
    color: '#fff',
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
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.5)',
    justifyContent: 'center',
    alignItems: 'center',
  },
  modalContent: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 20,
    width: '85%',
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 16,
    textAlign: 'center',
  },
  modalLabel: {
    fontSize: 14,
    color: '#666',
    marginBottom: 8,
  },
  modalInput: {
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 12,
    fontSize: 14,
    marginBottom: 20,
  },
  modalActions: {
    flexDirection: 'row',
  },
  modalCancelBtn: {
    flex: 1,
    paddingVertical: 12,
    alignItems: 'center',
    marginRight: 8,
    borderRadius: 8,
    backgroundColor: '#f5f5f5',
  },
  modalCancelText: {
    color: '#666',
    fontSize: 14,
  },
  modalConfirmBtn: {
    flex: 1,
    paddingVertical: 12,
    alignItems: 'center',
    marginLeft: 8,
    borderRadius: 8,
    backgroundColor: '#1976D2',
  },
  modalConfirmText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
});
