/**
 * 收益管理页面
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  TextInput,
  Alert,
  ActivityIndicator,
  RefreshControl,
} from 'react-native';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { WithdrawalRecord } from '@/features/diviner';

const THEME_COLOR = '#B2955D';

// Mock 数据
const mockWithdrawals: WithdrawalRecord[] = [
  {
    id: 1,
    provider: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    amount: BigInt(500 * 1e10),
    createdAt: Date.now() - 86400000 * 3,
    completedAt: Date.now() - 86400000 * 3,
  },
  {
    id: 2,
    provider: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    amount: BigInt(800 * 1e10),
    createdAt: Date.now() - 86400000 * 10,
    completedAt: Date.now() - 86400000 * 10,
  },
];

export default function EarningsPage() {
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [availableBalance, setAvailableBalance] = useState(BigInt(0));
  const [totalEarnings, setTotalEarnings] = useState(BigInt(0));
  const [withdrawals, setWithdrawals] = useState<WithdrawalRecord[]>([]);
  const [withdrawAmount, setWithdrawAmount] = useState('');

  const loadData = async () => {
    await new Promise(resolve => setTimeout(resolve, 500));
    setAvailableBalance(BigInt(1250 * 1e10));
    setTotalEarnings(BigInt(15680 * 1e10));
    setWithdrawals(mockWithdrawals);
  };

  useEffect(() => {
    loadData().finally(() => setLoading(false));
  }, []);

  const onRefresh = async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  };

  const formatDust = (amount: bigint) => (Number(amount) / 1e10).toFixed(2);

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp);
    return `${date.getFullYear()}/${date.getMonth() + 1}/${date.getDate()}`;
  };

  const handleWithdraw = async () => {
    const amount = parseFloat(withdrawAmount);
    if (isNaN(amount) || amount <= 0) {
      Alert.alert('提示', '请输入有效的提现金额');
      return;
    }

    const amountBigInt = BigInt(Math.floor(amount * 1e10));
    if (amountBigInt > availableBalance) {
      Alert.alert('提示', '提现金额超过可用余额');
      return;
    }

    setSubmitting(true);
    try {
      // TODO: 调用链上提现方法
      await new Promise(resolve => setTimeout(resolve, 1500));

      setAvailableBalance(prev => prev - amountBigInt);
      setWithdrawals(prev => [{
        id: Date.now(),
        provider: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        amount: amountBigInt,
        createdAt: Date.now(),
        completedAt: Date.now(),
      }, ...prev]);
      setWithdrawAmount('');

      Alert.alert('成功', `已提现 ${amount} DUST 到您的钱包`);
    } catch (error: any) {
      Alert.alert('提现失败', error.message || '请稍后重试');
    } finally {
      setSubmitting(false);
    }
  };

  const handleWithdrawAll = () => {
    setWithdrawAmount(formatDust(availableBalance));
  };

  if (loading) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="收益管理" />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  return (
    <View style={styles.wrapper}>
      <PageHeader title="收益管理" />

      <ScrollView
        style={styles.container}
        contentContainerStyle={styles.contentContainer}
        refreshControl={<RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor={THEME_COLOR} />}
      >
        {/* 余额卡片 */}
        <View style={styles.balanceCard}>
          <Text style={styles.balanceLabel}>可提现余额</Text>
          <Text style={styles.balanceValue}>{formatDust(availableBalance)} DUST</Text>
          <View style={styles.totalRow}>
            <Text style={styles.totalLabel}>累计收益：</Text>
            <Text style={styles.totalValue}>{formatDust(totalEarnings)} DUST</Text>
          </View>
        </View>

        {/* 提现表单 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>申请提现</Text>
          <View style={styles.withdrawCard}>
            <View style={styles.inputRow}>
              <TextInput
                style={styles.amountInput}
                value={withdrawAmount}
                onChangeText={setWithdrawAmount}
                placeholder="输入提现金额"
                placeholderTextColor="#999"
                keyboardType="decimal-pad"
              />
              <Pressable style={styles.allBtn} onPress={handleWithdrawAll}>
                <Text style={styles.allBtnText}>全部</Text>
              </Pressable>
            </View>
            <Pressable
              style={[styles.withdrawBtn, submitting && styles.withdrawBtnDisabled]}
              onPress={handleWithdraw}
              disabled={submitting}
            >
              {submitting ? (
                <ActivityIndicator color="#FFF" />
              ) : (
                <Text style={styles.withdrawBtnText}>立即提现</Text>
              )}
            </Pressable>
            <Text style={styles.withdrawNote}>提现将即时到账您的钱包</Text>
          </View>
        </View>

        {/* 提现记录 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>提现记录</Text>
          {withdrawals.length === 0 ? (
            <View style={styles.emptyContainer}>
              <Text style={styles.emptyText}>暂无提现记录</Text>
            </View>
          ) : (
            withdrawals.map(record => (
              <View key={record.id} style={styles.recordCard}>
                <View style={styles.recordLeft}>
                  <Text style={styles.recordAmount}>-{formatDust(record.amount)} DUST</Text>
                  <Text style={styles.recordTime}>{formatTime(record.createdAt)}</Text>
                </View>
                <View style={styles.recordStatus}>
                  <Text style={styles.recordStatusText}>已完成</Text>
                </View>
              </View>
            ))
          )}
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
  balanceCard: {
    backgroundColor: THEME_COLOR,
    margin: 16,
    borderRadius: 12,
    padding: 24,
    alignItems: 'center',
  },
  balanceLabel: {
    fontSize: 14,
    color: 'rgba(255,255,255,0.8)',
    marginBottom: 8,
  },
  balanceValue: {
    fontSize: 36,
    fontWeight: '700',
    color: '#FFF',
    marginBottom: 16,
  },
  totalRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  totalLabel: {
    fontSize: 14,
    color: 'rgba(255,255,255,0.8)',
  },
  totalValue: {
    fontSize: 14,
    color: '#FFF',
    fontWeight: '500',
  },
  section: {
    paddingHorizontal: 16,
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#000',
    marginBottom: 12,
  },
  withdrawCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
  },
  inputRow: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 12,
  },
  amountInput: {
    flex: 1,
    height: 48,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 8,
    paddingHorizontal: 12,
    fontSize: 16,
    color: '#333',
    backgroundColor: '#FAFAFA',
  },
  allBtn: {
    height: 48,
    paddingHorizontal: 16,
    justifyContent: 'center',
    alignItems: 'center',
    borderWidth: 1,
    borderColor: THEME_COLOR,
    borderRadius: 8,
  },
  allBtnText: {
    fontSize: 14,
    color: THEME_COLOR,
    fontWeight: '500',
  },
  withdrawBtn: {
    height: 48,
    backgroundColor: THEME_COLOR,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 8,
  },
  withdrawBtnDisabled: {
    opacity: 0.5,
  },
  withdrawBtnText: {
    fontSize: 16,
    color: '#FFF',
    fontWeight: '600',
  },
  withdrawNote: {
    fontSize: 12,
    color: '#999',
    textAlign: 'center',
  },
  emptyContainer: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 32,
    alignItems: 'center',
  },
  emptyText: {
    fontSize: 14,
    color: '#999',
  },
  recordCard: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 8,
  },
  recordLeft: {},
  recordAmount: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 4,
  },
  recordTime: {
    fontSize: 12,
    color: '#999',
  },
  recordStatus: {
    backgroundColor: '#E8F8EB',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
  },
  recordStatusText: {
    fontSize: 12,
    color: '#4CD964',
  },
});
