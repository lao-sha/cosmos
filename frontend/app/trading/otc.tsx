import { useTransaction } from '@/src/hooks/useTransaction';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    Alert,
    FlatList,
    Modal,
    Platform,
    Pressable,
    StyleSheet,
    Text,
    TextInput,
    View
} from 'react-native';

type OrderStatus = 'open' | 'locked' | 'paid' | 'completed' | 'disputed' | 'cancelled';
type OrderSide = 'buy' | 'sell';

interface OtcOrder {
  id: string;
  maker: string;
  side: OrderSide;
  amount: string;
  price: string;
  currency: string;
  status: OrderStatus;
  createdAt: string;
}

const MOCK_ORDERS: OtcOrder[] = [
  {
    id: '1',
    maker: '5GrwvaEF...JMakT',
    side: 'sell',
    amount: '1000',
    price: '7.2',
    currency: 'CNY',
    status: 'open',
    createdAt: '2026-01-28 10:00',
  },
  {
    id: '2',
    maker: '5FHneW46...rePXx',
    side: 'sell',
    amount: '500',
    price: '7.15',
    currency: 'CNY',
    status: 'open',
    createdAt: '2026-01-28 09:30',
  },
  {
    id: '3',
    maker: '5DAAnrj7...Cq7aG',
    side: 'buy',
    amount: '2000',
    price: '7.0',
    currency: 'CNY',
    status: 'open',
    createdAt: '2026-01-28 08:15',
  },
];

const STATUS_MAP: Record<OrderStatus, { label: string; color: string }> = {
  open: { label: '待交易', color: '#22c55e' },
  locked: { label: '已锁定', color: '#f59e0b' },
  paid: { label: '已付款', color: '#3b82f6' },
  completed: { label: '已完成', color: '#6b7280' },
  disputed: { label: '争议中', color: '#ef4444' },
  cancelled: { label: '已取消', color: '#9ca3af' },
};

export default function OtcScreen() {
  const router = useRouter();
  const { isLoggedIn, address } = useAuthStore();
  const { isConnected } = useChainStore();
  const { createOtcOrder, lockOtcOrder, isLoading, status } = useTransaction();
  const [orders, setOrders] = useState<OtcOrder[]>(MOCK_ORDERS);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [selectedSide, setSelectedSide] = useState<OrderSide>('sell');
  const [amount, setAmount] = useState('');
  const [price, setPrice] = useState('');

  const handleOrderPress = async (order: OtcOrder) => {
    if (!isLoggedIn) {
      if (Platform.OS === 'web') {
        window.alert('请先登录钱包');
      } else {
        Alert.alert('提示', '请先登录钱包');
      }
      return;
    }

    const action = order.side === 'sell' ? '购买' : '出售';
    const msg = `确认${action} ${order.amount} STAR？\n单价: ${order.price} ${order.currency}\n\n将发起链上交易`;
    
    const doLock = async () => {
      const result = await lockOtcOrder(order.id);
      if (result?.success) {
        setOrders(prev => prev.map(o => 
          o.id === order.id ? { ...o, status: 'locked' as OrderStatus } : o
        ));
      }
    };

    if (Platform.OS === 'web') {
      if (window.confirm(msg)) {
        doLock();
      }
    } else {
      Alert.alert('确认交易', msg, [
        { text: '取消', style: 'cancel' },
        { text: '确认上链', onPress: doLock },
      ]);
    }
  };

  const handleCreateOrder = async () => {
    if (!amount || !price) {
      if (Platform.OS === 'web') {
        window.alert('请填写数量和价格');
      } else {
        Alert.alert('提示', '请填写数量和价格');
      }
      return;
    }

    const orderType = selectedSide === 'sell' ? 'Sell' : 'Buy';
    const amountInUnits = (parseFloat(amount) * 1e10).toString();
    const priceInUnits = (parseFloat(price) * 100).toString();

    const result = await createOtcOrder(orderType, amountInUnits, priceInUnits, 'CNY');
    
    if (result?.success) {
      const newOrder: OtcOrder = {
        id: result.txHash || Date.now().toString(),
        maker: address?.slice(0, 10) + '...' + address?.slice(-5) || 'Unknown',
        side: selectedSide,
        amount,
        price,
        currency: 'CNY',
        status: 'open',
        createdAt: new Date().toLocaleString('zh-CN'),
      };

      setOrders([newOrder, ...orders]);
      setShowCreateModal(false);
      setAmount('');
      setPrice('');
    }
  };

  const renderOrder = ({ item }: { item: OtcOrder }) => {
    const status = STATUS_MAP[item.status];
    const isBuy = item.side === 'buy';
    
    return (
      <Pressable
        style={({ pressed }) => [styles.orderCard, pressed && styles.orderCardPressed]}
        onPress={() => handleOrderPress(item)}
      >
        <View style={styles.orderHeader}>
          <View style={[styles.sideBadge, isBuy ? styles.buyBadge : styles.sellBadge]}>
            <Text style={styles.sideText}>{isBuy ? '买入' : '卖出'}</Text>
          </View>
          <View style={[styles.statusBadge, { backgroundColor: status.color + '20' }]}>
            <Text style={[styles.statusText, { color: status.color }]}>{status.label}</Text>
          </View>
        </View>

        <View style={styles.orderBody}>
          <View style={styles.orderRow}>
            <Text style={styles.orderLabel}>数量</Text>
            <Text style={styles.orderValue}>{item.amount} STAR</Text>
          </View>
          <View style={styles.orderRow}>
            <Text style={styles.orderLabel}>单价</Text>
            <Text style={styles.orderPrice}>{item.price} {item.currency}</Text>
          </View>
          <View style={styles.orderRow}>
            <Text style={styles.orderLabel}>总价</Text>
            <Text style={styles.orderTotal}>
              {(parseFloat(item.amount) * parseFloat(item.price)).toFixed(2)} {item.currency}
            </Text>
          </View>
        </View>

        <View style={styles.orderFooter}>
          <Text style={styles.makerText}>{item.maker}</Text>
          <Text style={styles.timeText}>{item.createdAt}</Text>
        </View>
      </Pressable>
    );
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>OTC 交易</Text>
        <Pressable
          style={styles.createButton}
          onPress={() => {
            if (!isLoggedIn) {
              if (Platform.OS === 'web') {
                window.alert('请先登录钱包');
              } else {
                Alert.alert('提示', '请先登录钱包');
              }
              return;
            }
            setShowCreateModal(true);
          }}
        >
          <Text style={styles.createButtonText}>发布</Text>
        </Pressable>
      </View>

      <View style={styles.infoBar}>
        <Text style={styles.infoText}>
          {isConnected ? '🟢 链上交易' : '🔴 链未连接'}
        </Text>
        <Text style={styles.infoText}>当前 {orders.length} 个订单</Text>
      </View>

      <FlatList
        data={orders}
        renderItem={renderOrder}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.listContainer}
        ListEmptyComponent={
          <View style={styles.emptyContainer}>
            <Text style={styles.emptyIcon}>📋</Text>
            <Text style={styles.emptyText}>暂无订单</Text>
          </View>
        }
      />

      <Modal
        visible={showCreateModal}
        transparent
        animationType="slide"
        onRequestClose={() => setShowCreateModal(false)}
      >
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <Text style={styles.modalTitle}>发布订单</Text>

            <View style={styles.sideSelector}>
              <Pressable
                style={[styles.sideOption, selectedSide === 'sell' && styles.sideOptionSelected]}
                onPress={() => setSelectedSide('sell')}
              >
                <Text style={[styles.sideOptionText, selectedSide === 'sell' && styles.sideOptionTextSelected]}>
                  我要卖出
                </Text>
              </Pressable>
              <Pressable
                style={[styles.sideOption, selectedSide === 'buy' && styles.sideOptionSelected]}
                onPress={() => setSelectedSide('buy')}
              >
                <Text style={[styles.sideOptionText, selectedSide === 'buy' && styles.sideOptionTextSelected]}>
                  我要买入
                </Text>
              </Pressable>
            </View>

            <View style={styles.inputGroup}>
              <Text style={styles.inputLabel}>数量 (STAR)</Text>
              <TextInput
                style={styles.input}
                value={amount}
                onChangeText={setAmount}
                placeholder="输入数量"
                keyboardType="numeric"
                placeholderTextColor="#9ca3af"
              />
            </View>

            <View style={styles.inputGroup}>
              <Text style={styles.inputLabel}>单价 (CNY)</Text>
              <TextInput
                style={styles.input}
                value={price}
                onChangeText={setPrice}
                placeholder="输入单价"
                keyboardType="numeric"
                placeholderTextColor="#9ca3af"
              />
            </View>

            {amount && price && (
              <Text style={styles.totalPreview}>
                总价: {(parseFloat(amount || '0') * parseFloat(price || '0')).toFixed(2)} CNY
              </Text>
            )}

            <View style={styles.modalActions}>
              <Pressable
                style={[styles.modalButton, styles.cancelButton]}
                onPress={() => setShowCreateModal(false)}
              >
                <Text style={styles.cancelButtonText}>取消</Text>
              </Pressable>
              <Pressable
                style={[styles.modalButton, styles.confirmButton]}
                onPress={handleCreateOrder}
              >
                <Text style={styles.confirmButtonText}>发布</Text>
              </Pressable>
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
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#fff',
    paddingTop: 50,
    paddingBottom: 12,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  backButton: {
    padding: 8,
  },
  backText: {
    fontSize: 16,
    color: '#6D28D9',
  },
  headerTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
  },
  createButton: {
    backgroundColor: '#6D28D9',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 8,
  },
  createButtonText: {
    color: '#fff',
    fontWeight: '600',
  },
  infoBar: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    backgroundColor: '#fff',
    paddingHorizontal: 16,
    paddingVertical: 10,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  infoText: {
    fontSize: 13,
    color: '#6b7280',
  },
  listContainer: {
    padding: 12,
  },
  orderCard: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  orderCardPressed: {
    backgroundColor: '#f9fafb',
  },
  orderHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  sideBadge: {
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 4,
  },
  buyBadge: {
    backgroundColor: '#dcfce7',
  },
  sellBadge: {
    backgroundColor: '#fee2e2',
  },
  sideText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#1f2937',
  },
  statusBadge: {
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
  },
  statusText: {
    fontSize: 11,
    fontWeight: '500',
  },
  orderBody: {
    marginBottom: 12,
  },
  orderRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 6,
  },
  orderLabel: {
    fontSize: 13,
    color: '#6b7280',
  },
  orderValue: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1f2937',
  },
  orderPrice: {
    fontSize: 14,
    fontWeight: '600',
    color: '#6D28D9',
  },
  orderTotal: {
    fontSize: 15,
    fontWeight: 'bold',
    color: '#ef4444',
  },
  orderFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    borderTopWidth: 1,
    borderTopColor: '#f3f4f6',
    paddingTop: 10,
  },
  makerText: {
    fontSize: 12,
    color: '#9ca3af',
  },
  timeText: {
    fontSize: 12,
    color: '#9ca3af',
  },
  emptyContainer: {
    alignItems: 'center',
    paddingVertical: 60,
  },
  emptyIcon: {
    fontSize: 48,
    marginBottom: 12,
  },
  emptyText: {
    fontSize: 14,
    color: '#9ca3af',
  },
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.5)',
    justifyContent: 'flex-end',
  },
  modalContent: {
    backgroundColor: '#fff',
    borderTopLeftRadius: 20,
    borderTopRightRadius: 20,
    padding: 20,
    paddingBottom: 40,
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
    textAlign: 'center',
    marginBottom: 20,
  },
  sideSelector: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 20,
  },
  sideOption: {
    flex: 1,
    paddingVertical: 12,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#e5e7eb',
    alignItems: 'center',
  },
  sideOptionSelected: {
    borderColor: '#6D28D9',
    backgroundColor: '#6D28D9',
  },
  sideOptionText: {
    fontSize: 14,
    fontWeight: '500',
    color: '#6b7280',
  },
  sideOptionTextSelected: {
    color: '#fff',
  },
  inputGroup: {
    marginBottom: 16,
  },
  inputLabel: {
    fontSize: 13,
    color: '#374151',
    marginBottom: 6,
  },
  input: {
    backgroundColor: '#f3f4f6',
    borderRadius: 8,
    paddingHorizontal: 14,
    paddingVertical: 12,
    fontSize: 15,
    color: '#1f2937',
  },
  totalPreview: {
    fontSize: 16,
    fontWeight: '600',
    color: '#ef4444',
    textAlign: 'center',
    marginBottom: 20,
  },
  modalActions: {
    flexDirection: 'row',
    gap: 12,
  },
  modalButton: {
    flex: 1,
    paddingVertical: 14,
    borderRadius: 10,
    alignItems: 'center',
  },
  cancelButton: {
    backgroundColor: '#f3f4f6',
  },
  cancelButtonText: {
    fontSize: 15,
    fontWeight: '600',
    color: '#6b7280',
  },
  confirmButton: {
    backgroundColor: '#6D28D9',
  },
  confirmButtonText: {
    fontSize: 15,
    fontWeight: '600',
    color: '#fff',
  },
});
