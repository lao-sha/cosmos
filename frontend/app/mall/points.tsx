import { useRouter } from 'expo-router';
import { Coins, ArrowRightLeft, Store, ChevronRight } from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
  TextInput,
  Alert,
  Modal,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import { sharemallTxService } from '@/src/services/sharemall-tx';
import type { Shop } from '@/src/types/sharemall';
import { useWalletStore } from '@/src/stores/wallet';

interface ShopPoints {
  shop: Shop;
  balance: string;
}

export default function PointsScreen() {
  const router = useRouter();
  const { currentAccount } = useWalletStore();
  const [shopPoints, setShopPoints] = useState<ShopPoints[]>([]);
  const [refreshing, setRefreshing] = useState(false);
  const [transferModalVisible, setTransferModalVisible] = useState(false);
  const [selectedShop, setSelectedShop] = useState<ShopPoints | null>(null);
  const [transferTo, setTransferTo] = useState('');
  const [transferAmount, setTransferAmount] = useState('');
  const [loading, setLoading] = useState(false);

  const loadData = async () => {
    if (!currentAccount) return;
    try {
      const shops = await sharemallService.getShopList(50);
      const pointsList: ShopPoints[] = [];

      for (const shop of shops) {
        const config = await sharemallService.getTokenConfig(shop.id);
        if (config?.enabled) {
          const balance = await sharemallService.getTokenBalance(
            shop.id,
            currentAccount.address
          );
          if (BigInt(balance) > 0) {
            pointsList.push({ shop, balance });
          }
        }
      }

      setShopPoints(pointsList);
    } catch (error) {
      console.error('Failed to load points:', error);
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

  const formatBalance = (balance: string) => {
    const num = BigInt(balance);
    return (Number(num) / 1e10).toFixed(2);
  };

  const openTransferModal = (item: ShopPoints) => {
    setSelectedShop(item);
    setTransferTo('');
    setTransferAmount('');
    setTransferModalVisible(true);
  };

  const handleTransfer = async () => {
    if (!selectedShop || !transferTo.trim() || !transferAmount) {
      Alert.alert('提示', '请填写完整信息');
      return;
    }

    setLoading(true);
    try {
      const amount = (parseFloat(transferAmount) * 1e10).toString();
      await sharemallTxService.transferToken(
        selectedShop.shop.id,
        transferTo.trim(),
        amount
      );
      Alert.alert('成功', '积分转让成功');
      setTransferModalVisible(false);
      loadData();
    } catch (error: any) {
      Alert.alert('错误', error.message || '转让失败');
    } finally {
      setLoading(false);
    }
  };

  const totalPoints = shopPoints.reduce(
    (total, item) => total + BigInt(item.balance),
    BigInt(0)
  );

  const renderPointsItem = ({ item }: { item: ShopPoints }) => (
    <View style={styles.pointsCard}>
      <TouchableOpacity
        style={styles.shopInfo}
        onPress={() => router.push(`/mall/shop/${item.shop.id}` as any)}
      >
        <View style={styles.shopIcon}>
          <Store size={24} color="#1976D2" />
        </View>
        <View style={styles.shopDetail}>
          <Text style={styles.shopName}>{item.shop.name}</Text>
          <Text style={styles.pointsBalance}>
            {formatBalance(item.balance)} 积分
          </Text>
        </View>
        <ChevronRight size={20} color="#ccc" />
      </TouchableOpacity>

      <View style={styles.actions}>
        <TouchableOpacity
          style={styles.actionBtn}
          onPress={() => openTransferModal(item)}
        >
          <ArrowRightLeft size={16} color="#1976D2" />
          <Text style={styles.actionText}>转让</Text>
        </TouchableOpacity>
        <TouchableOpacity
          style={styles.actionBtn}
          onPress={() => router.push(`/mall/market?shopId=${item.shop.id}` as any)}
        >
          <Coins size={16} color="#388E3C" />
          <Text style={[styles.actionText, { color: '#388E3C' }]}>交易</Text>
        </TouchableOpacity>
      </View>
    </View>
  );

  return (
    <View style={styles.container}>
      {/* 总积分 */}
      <View style={styles.totalCard}>
        <Coins size={32} color="#FFB300" />
        <View style={styles.totalInfo}>
          <Text style={styles.totalLabel}>我的积分总额</Text>
          <Text style={styles.totalValue}>{formatBalance(totalPoints.toString())}</Text>
        </View>
      </View>

      {/* 积分列表 */}
      <FlatList
        data={shopPoints}
        renderItem={renderPointsItem}
        keyExtractor={(item) => item.shop.id.toString()}
        contentContainerStyle={styles.listContent}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
        ListEmptyComponent={
          <View style={styles.emptyContainer}>
            <Coins size={48} color="#ccc" />
            <Text style={styles.emptyText}>暂无积分</Text>
            <Text style={styles.emptyHint}>购物后可获得店铺积分</Text>
          </View>
        }
      />

      {/* 转让弹窗 */}
      <Modal visible={transferModalVisible} transparent animationType="fade">
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <Text style={styles.modalTitle}>转让积分</Text>
            <Text style={styles.modalShopName}>{selectedShop?.shop.name}</Text>
            <Text style={styles.modalBalance}>
              可用: {selectedShop ? formatBalance(selectedShop.balance) : '0'} 积分
            </Text>

            <Text style={styles.modalLabel}>接收地址</Text>
            <TextInput
              style={styles.modalInput}
              placeholder="输入接收人地址"
              value={transferTo}
              onChangeText={setTransferTo}
            />

            <Text style={styles.modalLabel}>转让数量</Text>
            <TextInput
              style={styles.modalInput}
              placeholder="输入转让数量"
              keyboardType="decimal-pad"
              value={transferAmount}
              onChangeText={setTransferAmount}
            />

            <View style={styles.modalActions}>
              <TouchableOpacity
                style={styles.modalCancelBtn}
                onPress={() => setTransferModalVisible(false)}
              >
                <Text style={styles.modalCancelText}>取消</Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={[styles.modalConfirmBtn, loading && styles.modalBtnDisabled]}
                onPress={handleTransfer}
                disabled={loading}
              >
                <Text style={styles.modalConfirmText}>
                  {loading ? '转让中...' : '确认转让'}
                </Text>
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
  totalCard: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    margin: 12,
    borderRadius: 12,
    padding: 20,
    alignItems: 'center',
  },
  totalInfo: {
    marginLeft: 16,
  },
  totalLabel: {
    fontSize: 13,
    color: '#999',
  },
  totalValue: {
    fontSize: 28,
    fontWeight: '700',
    color: '#333',
    marginTop: 4,
  },
  listContent: {
    paddingHorizontal: 12,
    paddingBottom: 12,
  },
  pointsCard: {
    backgroundColor: '#fff',
    borderRadius: 12,
    marginBottom: 12,
    overflow: 'hidden',
  },
  shopInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 16,
  },
  shopIcon: {
    width: 48,
    height: 48,
    borderRadius: 24,
    backgroundColor: '#E3F2FD',
    alignItems: 'center',
    justifyContent: 'center',
  },
  shopDetail: {
    flex: 1,
    marginLeft: 12,
  },
  shopName: {
    fontSize: 15,
    fontWeight: '500',
    color: '#333',
  },
  pointsBalance: {
    fontSize: 18,
    fontWeight: '600',
    color: '#FFB300',
    marginTop: 4,
  },
  actions: {
    flexDirection: 'row',
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
  },
  actionBtn: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 12,
  },
  actionText: {
    fontSize: 13,
    color: '#1976D2',
    marginLeft: 6,
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
  emptyHint: {
    marginTop: 4,
    fontSize: 12,
    color: '#bbb',
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
    textAlign: 'center',
  },
  modalShopName: {
    fontSize: 14,
    color: '#666',
    textAlign: 'center',
    marginTop: 4,
  },
  modalBalance: {
    fontSize: 13,
    color: '#FFB300',
    textAlign: 'center',
    marginTop: 4,
    marginBottom: 20,
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
    marginBottom: 16,
  },
  modalActions: {
    flexDirection: 'row',
    marginTop: 8,
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
  modalBtnDisabled: {
    opacity: 0.6,
  },
  modalConfirmText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
});
