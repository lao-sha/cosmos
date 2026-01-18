/**
 * 做市商桥接页面
 * 选择做市商进行 DUST → USDT 兑换
 */

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import {
  SwapAmountInput,
  TronAddressInput,
  BridgeMakerCard,
} from '@/features/bridge/components';
import { BridgeMaker } from '@/features/bridge/types';
import { isWebEnvironment, isSignerUnlocked } from '@/lib/signer';

const MIN_AMOUNT = 10;

// 模拟做市商数据
const mockMakers: BridgeMaker[] = [
  {
    id: 1,
    account: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    tronAddress: 'TJYeasTPa6gpEEfYcPQgLHu9eGNj1FGrVK',
    isActive: true,
    rating: 4.8,
    completedSwaps: 156,
    avgResponseTime: 600,
    creditLevel: 'A+',
  },
  {
    id: 2,
    account: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
    tronAddress: 'TN3W4H6rK2ce4vX9YnFQHwKENnHjoxb3m9',
    isActive: true,
    rating: 4.5,
    completedSwaps: 89,
    avgResponseTime: 900,
    creditLevel: 'A',
  },
  {
    id: 3,
    account: '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y',
    tronAddress: 'TVj7RNVHy6thbM7BWdSe9G6gXwKhjhdNZS',
    isActive: false,
    rating: 4.2,
    completedSwaps: 45,
    avgResponseTime: 1200,
    creditLevel: 'B+',
  },
];

export default function MakerBridgePage() {
  const router = useRouter();
  const [dustAmount, setDustAmount] = useState('');
  const [tronAddress, setTronAddress] = useState('');
  const [selectedMaker, setSelectedMaker] = useState<BridgeMaker | null>(null);
  const [dustPrice, setDustPrice] = useState(0.10);
  const [balance, setBalance] = useState('1000');
  const [makers, setMakers] = useState<BridgeMaker[]>([]);
  const [loadingMakers, setLoadingMakers] = useState(true);
  const [loading, setLoading] = useState(false);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('准备中...');

  useEffect(() => {
    // TODO: 从链上获取做市商列表
    setTimeout(() => {
      setMakers(mockMakers);
      setLoadingMakers(false);
    }, 500);
  }, []);

  const validateForm = (): boolean => {
    const amount = parseFloat(dustAmount);
    if (isNaN(amount) || amount < MIN_AMOUNT) {
      Alert.alert('提示', `最小兑换金额为 ${MIN_AMOUNT} DUST`);
      return false;
    }

    if (amount > parseFloat(balance)) {
      Alert.alert('提示', 'DUST 余额不足');
      return false;
    }

    // 验证 TRON 地址
    const tronRegex = /^T[A-Za-z1-9]{33}$/;
    if (!tronRegex.test(tronAddress)) {
      Alert.alert('提示', '请输入有效的 TRON 地址');
      return false;
    }

    if (!selectedMaker) {
      Alert.alert('提示', '请选择做市商');
      return false;
    }

    if (!selectedMaker.isActive) {
      Alert.alert('提示', '该做市商当前离线，请选择其他做市商');
      return false;
    }

    return true;
  };

  const handleSwap = async () => {
    if (!validateForm()) return;

    // 检查是否需要解锁钱包
    if (!isWebEnvironment() && !isSignerUnlocked()) {
      setShowUnlockDialog(true);
      return;
    }

    await executeSwap();
  };

  const handleWalletUnlocked = async () => {
    setShowUnlockDialog(false);
    await executeSwap();
  };

  const executeSwap = async () => {
    if (!selectedMaker) return;

    try {
      setShowTxStatus(true);
      setTxStatus('正在创建兑换请求...');

      // TODO: 调用链上 bridge.maker_swap() 方法
      // const api = await getApi();
      // const tx = api.tx.bridge.makerSwap(
      //   selectedMaker.id,
      //   dustAmountBigInt,
      //   tronAddressBytes
      // );
      // await signAndSend(tx);

      // 模拟交易
      await new Promise(resolve => setTimeout(resolve, 2000));
      setTxStatus('交易已提交，等待确认...');
      await new Promise(resolve => setTimeout(resolve, 1500));

      setShowTxStatus(false);

      Alert.alert(
        '成功',
        '兑换请求已创建，做市商将在 30 分钟内转账',
        [
          {
            text: '查看记录',
            onPress: () => router.push('/bridge/history' as any),
          },
          {
            text: '确定',
            style: 'cancel',
          },
        ]
      );
    } catch (error) {
      setShowTxStatus(false);
      const errorMessage = error instanceof Error ? error.message : '创建兑换失败';
      Alert.alert('错误', errorMessage);
    }
  };

  const usdtEstimate = (parseFloat(dustAmount) || 0) * dustPrice;
  const activeMakersCount = makers.filter(m => m.isActive).length;

  return (
    <View style={styles.wrapper}>
      <PageHeader title="做市商桥接" />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 说明卡片 */}
        <View style={styles.section}>
          <View style={styles.infoCard}>
            <Text style={styles.infoTitle}>👥 做市商桥接</Text>
            <Text style={styles.infoText}>
              选择做市商进行兑换，通常 30 分钟内到账。
              超时未完成将自动退款。
            </Text>
          </View>
        </View>

        {/* 金额输入 */}
        <View style={styles.section}>
          <SwapAmountInput
            value={dustAmount}
            onChangeText={setDustAmount}
            dustPrice={dustPrice}
            balance={balance}
            minAmount={MIN_AMOUNT}
          />
        </View>

        {/* TRON 地址输入 */}
        <View style={styles.section}>
          <TronAddressInput
            value={tronAddress}
            onChangeText={setTronAddress}
          />
        </View>

        {/* 选择做市商 */}
        <View style={styles.section}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>选择做市商</Text>
            <Text style={styles.sectionSubtitle}>
              {activeMakersCount} 位做市商在线
            </Text>
          </View>

          {loadingMakers ? (
            <View style={styles.loading}>
              <ActivityIndicator size="large" color="#B2955D" />
              <Text style={styles.loadingText}>加载中...</Text>
            </View>
          ) : makers.length === 0 ? (
            <View style={styles.empty}>
              <Text style={styles.emptyText}>暂无可用做市商</Text>
            </View>
          ) : (
            makers.map((maker) => (
              <BridgeMakerCard
                key={maker.id}
                maker={maker}
                selected={selectedMaker?.id === maker.id}
                onPress={() => setSelectedMaker(maker)}
              />
            ))
          )}
        </View>

        {/* 兑换详情 */}
        {selectedMaker && (
          <View style={styles.section}>
            <View style={styles.detailCard}>
              <Text style={styles.detailTitle}>兑换详情</Text>
              <View style={styles.detailRow}>
                <Text style={styles.detailLabel}>支付</Text>
                <Text style={styles.detailValue}>
                  {dustAmount || '0'} DUST
                </Text>
              </View>
              <View style={styles.detailRow}>
                <Text style={styles.detailLabel}>汇率</Text>
                <Text style={styles.detailValue}>
                  1 DUST = {dustPrice.toFixed(4)} USDT
                </Text>
              </View>
              <View style={styles.detailRow}>
                <Text style={styles.detailLabel}>做市商</Text>
                <Text style={styles.detailValue}>
                  #{selectedMaker.id} ({selectedMaker.creditLevel})
                </Text>
              </View>
              <View style={styles.detailRow}>
                <Text style={styles.detailLabel}>超时时间</Text>
                <Text style={styles.detailValue}>30 分钟</Text>
              </View>
              <View style={styles.divider} />
              <View style={styles.detailRow}>
                <Text style={styles.detailLabelBold}>预计获得</Text>
                <Text style={styles.detailValueGreen}>
                  ≈ {usdtEstimate.toFixed(2)} USDT
                </Text>
              </View>
            </View>
          </View>
        )}

        {/* 提交按钮 */}
        <View style={styles.section}>
          <TouchableOpacity
            style={[
              styles.submitButton,
              (!dustAmount || !tronAddress || !selectedMaker) && styles.submitButtonDisabled,
            ]}
            onPress={handleSwap}
            disabled={!dustAmount || !tronAddress || !selectedMaker || loading}
          >
            {loading ? (
              <ActivityIndicator color="#FFFFFF" />
            ) : (
              <Text style={styles.submitButtonText}>确认兑换</Text>
            )}
          </TouchableOpacity>
        </View>

        {/* 注意事项 */}
        <View style={styles.section}>
          <Text style={styles.noticeTitle}>⚠️ 注意事项</Text>
          <Text style={styles.noticeText}>• 兑换请求提交后，DUST 将被锁定</Text>
          <Text style={styles.noticeText}>• 做市商需在 30 分钟内完成转账</Text>
          <Text style={styles.noticeText}>• 超时未完成将自动退款</Text>
          <Text style={styles.noticeText}>• 如遇问题可发起举报</Text>
        </View>
      </ScrollView>

      <BottomNavBar activeTab="profile" />

      {/* 解锁钱包对话框 */}
      <UnlockWalletDialog
        visible={showUnlockDialog}
        onUnlock={handleWalletUnlocked}
        onCancel={() => setShowUnlockDialog(false)}
      />

      {/* 交易状态对话框 */}
      <TransactionStatusDialog
        visible={showTxStatus}
        status={txStatus}
        title="创建兑换中"
      />
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
    paddingBottom: 20,
  },
  section: {
    padding: 16,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#000000',
  },
  sectionSubtitle: {
    fontSize: 14,
    color: '#666666',
  },
  infoCard: {
    backgroundColor: '#FFF9F0',
    borderRadius: 12,
    padding: 16,
    borderWidth: 1,
    borderColor: '#B2955D',
  },
  infoTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  infoText: {
    fontSize: 14,
    color: '#666666',
    lineHeight: 20,
  },
  loading: {
    alignItems: 'center',
    paddingVertical: 40,
  },
  loadingText: {
    fontSize: 14,
    color: '#666666',
    marginTop: 12,
  },
  empty: {
    alignItems: 'center',
    paddingVertical: 40,
  },
  emptyText: {
    fontSize: 14,
    color: '#999999',
  },
  detailCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
  },
  detailTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 12,
  },
  detailRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  detailLabel: {
    fontSize: 14,
    color: '#666666',
  },
  detailLabelBold: {
    fontSize: 14,
    fontWeight: '600',
    color: '#000000',
  },
  detailValue: {
    fontSize: 14,
    color: '#000000',
  },
  detailValueGreen: {
    fontSize: 16,
    fontWeight: '600',
    color: '#4CD964',
  },
  divider: {
    height: 1,
    backgroundColor: '#F0F0F0',
    marginVertical: 8,
  },
  submitButton: {
    backgroundColor: '#B2955D',
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
  },
  submitButtonDisabled: {
    backgroundColor: '#CCCCCC',
  },
  submitButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  noticeTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  noticeText: {
    fontSize: 13,
    color: '#666666',
    marginBottom: 4,
  },
});
