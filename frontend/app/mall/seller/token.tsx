import { useRouter } from 'expo-router';
import { Coins, Settings, TrendingUp, Users, ArrowRightLeft } from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
  TextInput,
  Alert,
  Switch,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import { sharemallTxService } from '@/src/services/sharemall-tx';
import type { Shop, ShopTokenConfig } from '@/src/types/sharemall';
import { useWalletStore } from '@/src/stores/wallet';

export default function TokenManageScreen() {
  const router = useRouter();
  const { currentAccount } = useWalletStore();

  const [myShop, setMyShop] = useState<Shop | null>(null);
  const [tokenConfig, setTokenConfig] = useState<ShopTokenConfig | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  // 配置表单
  const [rewardRate, setRewardRate] = useState('100'); // 1% = 100 基点
  const [exchangeRate, setExchangeRate] = useState('100');
  const [minRedeem, setMinRedeem] = useState('10');
  const [maxRedeemPerOrder, setMaxRedeemPerOrder] = useState('1000');
  const [transferable, setTransferable] = useState(true);
  const [loading, setLoading] = useState(false);

  const loadData = async () => {
    if (!currentAccount) return;
    try {
      const shops = await sharemallService.getShopList(100);
      const userShop = shops.find((s) => s.owner === currentAccount.address);
      setMyShop(userShop || null);

      if (userShop) {
        const config = await sharemallService.getTokenConfig(userShop.id);
        setTokenConfig(config);
        if (config) {
          setRewardRate(config.rewardRate.toString());
          setExchangeRate(config.exchangeRate.toString());
          setMinRedeem((Number(BigInt(config.minRedeem)) / 1e10).toString());
          setMaxRedeemPerOrder((Number(BigInt(config.maxRedeemPerOrder)) / 1e10).toString());
          setTransferable(config.transferable);
        }
      }
    } catch (error) {
      console.error('Failed to load token config:', error);
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

  const handleEnableToken = async () => {
    if (!myShop) return;

    setLoading(true);
    try {
      const minRedeemValue = (parseFloat(minRedeem) * 1e10).toString();
      const maxRedeemValue = (parseFloat(maxRedeemPerOrder) * 1e10).toString();

      await sharemallTxService.enableToken(
        myShop.id,
        parseInt(rewardRate, 10),
        parseInt(exchangeRate, 10),
        minRedeemValue,
        maxRedeemValue,
        transferable
      );

      Alert.alert('成功', '代币功能已启用');
      loadData();
    } catch (error: any) {
      Alert.alert('错误', error.message || '操作失败');
    } finally {
      setLoading(false);
    }
  };

  const handleSetInitialPrice = async () => {
    if (!myShop) return;

    Alert.prompt(
      '设置初始价格',
      '输入代币的初始价格 (COS)',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确定',
          onPress: async (price) => {
            if (!price) return;
            try {
              const priceValue = (parseFloat(price) * 1e10).toString();
              await sharemallTxService.setInitialPrice(myShop.id, priceValue);
              Alert.alert('成功', '初始价格已设置');
            } catch (error: any) {
              Alert.alert('错误', error.message || '设置失败');
            }
          },
        },
      ],
      'plain-text',
      '1'
    );
  };

  if (!myShop) {
    return (
      <View style={styles.loadingContainer}>
        <Text>加载中...</Text>
      </View>
    );
  }

  return (
    <ScrollView
      style={styles.container}
      refreshControl={<RefreshControl refreshing={refreshing} onRefresh={onRefresh} />}
    >
      {/* 代币状态 */}
      <View style={styles.statusCard}>
        <View style={styles.statusHeader}>
          <View style={styles.statusIcon}>
            <Coins size={32} color={tokenConfig?.enabled ? '#388E3C' : '#999'} />
          </View>
          <View style={styles.statusInfo}>
            <Text style={styles.statusTitle}>店铺代币</Text>
            <Text
              style={[
                styles.statusText,
                { color: tokenConfig?.enabled ? '#388E3C' : '#999' },
              ]}
            >
              {tokenConfig?.enabled ? '已启用' : '未启用'}
            </Text>
          </View>
        </View>

        {tokenConfig?.enabled && (
          <View style={styles.statsRow}>
            <View style={styles.statItem}>
              <Text style={styles.statValue}>{tokenConfig.rewardRate / 100}%</Text>
              <Text style={styles.statLabel}>返积分比例</Text>
            </View>
            <View style={styles.statItem}>
              <Text style={styles.statValue}>{tokenConfig.exchangeRate / 100}%</Text>
              <Text style={styles.statLabel}>兑换比例</Text>
            </View>
            <View style={styles.statItem}>
              <Text style={styles.statValue}>
                {tokenConfig.transferable ? '是' : '否'}
              </Text>
              <Text style={styles.statLabel}>可转让</Text>
            </View>
          </View>
        )}
      </View>

      {/* 快捷操作 */}
      {tokenConfig?.enabled && (
        <View style={styles.actionsCard}>
          <TouchableOpacity
            style={styles.actionItem}
            onPress={() => router.push(`/mall/market?shopId=${myShop.id}` as any)}
          >
            <View style={[styles.actionIcon, { backgroundColor: '#E3F2FD' }]}>
              <TrendingUp size={20} color="#1976D2" />
            </View>
            <Text style={styles.actionText}>代币交易</Text>
          </TouchableOpacity>

          <TouchableOpacity style={styles.actionItem} onPress={handleSetInitialPrice}>
            <View style={[styles.actionIcon, { backgroundColor: '#FFF3E0' }]}>
              <Settings size={20} color="#F57C00" />
            </View>
            <Text style={styles.actionText}>设置初始价格</Text>
          </TouchableOpacity>

          <TouchableOpacity style={styles.actionItem}>
            <View style={[styles.actionIcon, { backgroundColor: '#E8F5E9' }]}>
              <ArrowRightLeft size={20} color="#388E3C" />
            </View>
            <Text style={styles.actionText}>发放积分</Text>
          </TouchableOpacity>
        </View>
      )}

      {/* 配置表单 */}
      <View style={styles.configCard}>
        <Text style={styles.cardTitle}>
          {tokenConfig?.enabled ? '更新配置' : '启用代币'}
        </Text>

        <View style={styles.inputGroup}>
          <Text style={styles.label}>返积分比例 (基点, 100 = 1%)</Text>
          <TextInput
            style={styles.input}
            placeholder="100"
            keyboardType="number-pad"
            value={rewardRate}
            onChangeText={setRewardRate}
          />
          <Text style={styles.hint}>
            用户购物时获得的积分比例，如 100 表示 1%
          </Text>
        </View>

        <View style={styles.inputGroup}>
          <Text style={styles.label}>兑换比例 (基点, 100 = 1%)</Text>
          <TextInput
            style={styles.input}
            placeholder="100"
            keyboardType="number-pad"
            value={exchangeRate}
            onChangeText={setExchangeRate}
          />
          <Text style={styles.hint}>
            积分抵扣订单金额的比例，如 100 表示 1%
          </Text>
        </View>

        <View style={styles.inputGroup}>
          <Text style={styles.label}>最小兑换数量 (积分)</Text>
          <TextInput
            style={styles.input}
            placeholder="10"
            keyboardType="decimal-pad"
            value={minRedeem}
            onChangeText={setMinRedeem}
          />
        </View>

        <View style={styles.inputGroup}>
          <Text style={styles.label}>单笔最大兑换 (积分)</Text>
          <TextInput
            style={styles.input}
            placeholder="1000"
            keyboardType="decimal-pad"
            value={maxRedeemPerOrder}
            onChangeText={setMaxRedeemPerOrder}
          />
        </View>

        <View style={styles.switchRow}>
          <Text style={styles.label}>允许积分转让</Text>
          <Switch value={transferable} onValueChange={setTransferable} />
        </View>

        <TouchableOpacity
          style={[styles.submitBtn, loading && styles.submitBtnDisabled]}
          onPress={handleEnableToken}
          disabled={loading}
        >
          <Text style={styles.submitBtnText}>
            {loading
              ? '处理中...'
              : tokenConfig?.enabled
              ? '更新配置'
              : '启用代币'}
          </Text>
        </TouchableOpacity>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  loadingContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  statusCard: {
    backgroundColor: '#fff',
    margin: 12,
    borderRadius: 12,
    padding: 16,
  },
  statusHeader: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  statusIcon: {
    width: 56,
    height: 56,
    borderRadius: 28,
    backgroundColor: '#f5f5f5',
    alignItems: 'center',
    justifyContent: 'center',
  },
  statusInfo: {
    marginLeft: 16,
  },
  statusTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  statusText: {
    fontSize: 14,
    marginTop: 4,
  },
  statsRow: {
    flexDirection: 'row',
    marginTop: 16,
    paddingTop: 16,
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
  },
  statItem: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  statLabel: {
    fontSize: 12,
    color: '#999',
    marginTop: 4,
  },
  actionsCard: {
    backgroundColor: '#fff',
    marginHorizontal: 12,
    borderRadius: 12,
    padding: 16,
    flexDirection: 'row',
  },
  actionItem: {
    flex: 1,
    alignItems: 'center',
  },
  actionIcon: {
    width: 44,
    height: 44,
    borderRadius: 22,
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 8,
  },
  actionText: {
    fontSize: 12,
    color: '#333',
  },
  configCard: {
    backgroundColor: '#fff',
    margin: 12,
    borderRadius: 12,
    padding: 16,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 16,
  },
  inputGroup: {
    marginBottom: 16,
  },
  label: {
    fontSize: 14,
    color: '#333',
    marginBottom: 8,
  },
  input: {
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 12,
    fontSize: 14,
  },
  hint: {
    fontSize: 12,
    color: '#999',
    marginTop: 4,
  },
  switchRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 20,
  },
  submitBtn: {
    backgroundColor: '#1976D2',
    borderRadius: 8,
    paddingVertical: 14,
    alignItems: 'center',
  },
  submitBtnDisabled: {
    opacity: 0.6,
  },
  submitBtnText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
