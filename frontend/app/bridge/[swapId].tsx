/**
 * 兑换详情页面
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
  Linking,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { SwapStatusBadge } from '@/features/bridge/components';
import { MakerSwapRecord, SwapStatus } from '@/features/bridge/types';
import { CountdownTimer } from '@/features/trading/components';

export default function SwapDetailPage() {
  const router = useRouter();
  const { swapId } = useLocalSearchParams<{ swapId: string }>();
  const [record, setRecord] = useState<MakerSwapRecord | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // TODO: 从链上获取兑换详情
    setTimeout(() => {
      // 模拟数据
      setRecord({
        swapId: parseInt(swapId || '0'),
        makerId: 1,
        maker: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        user: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
        dustAmount: BigInt(500_000_000_000_000),
        usdtAmount: 50_000_000,
        usdtAddress: 'TJYeasTPa6gpEEfYcPQgLHu9eGNj1FGrVK',
        createdAt: 12345678,
        timeoutAt: 12345978,
        trc20TxHash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
        completedAt: 12345700,
        status: SwapStatus.Pending,
        priceUsdt: 100_000,
      });
      setLoading(false);
    }, 500);
  }, [swapId]);

  const formatDust = (amount: bigint): string => {
    return (Number(amount) / 1e12).toFixed(4);
  };

  const formatUsdt = (amount: number): string => {
    return (amount / 1e6).toFixed(2);
  };

  const formatAddress = (address: string): string => {
    if (address.length <= 16) return address;
    return `${address.slice(0, 8)}...${address.slice(-8)}`;
  };

  const handleReport = () => {
    Alert.alert(
      '举报兑换',
      '确定要举报此兑换吗？举报后将进入仲裁流程。',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确定举报',
          style: 'destructive',
          onPress: async () => {
            // TODO: 调用链上 bridge.report_swap() 方法
            Alert.alert('成功', '举报已提交，请等待仲裁处理');
          },
        },
      ]
    );
  };

  const handleViewTronTx = () => {
    if (record?.trc20TxHash) {
      const url = `https://tronscan.org/#/transaction/${record.trc20TxHash}`;
      Linking.openURL(url);
    }
  };

  const handleCopyAddress = async (address: string) => {
    // TODO: 复制到剪贴板
    Alert.alert('已复制', address);
  };

  if (loading) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="兑换详情" />
        <View style={styles.loading}>
          <ActivityIndicator size="large" color="#B2955D" />
          <Text style={styles.loadingText}>加载中...</Text>
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  if (!record) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="兑换详情" />
        <View style={styles.empty}>
          <Text style={styles.emptyText}>兑换记录不存在</Text>
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  const canReport =
    record.status === SwapStatus.Pending ||
    record.status === SwapStatus.Completed;

  return (
    <View style={styles.wrapper}>
      <PageHeader title={`兑换 #${record.swapId}`} />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 状态卡片 */}
        <View style={styles.section}>
          <View style={styles.statusCard}>
            <SwapStatusBadge status={record.status} size="large" />
            {record.status === SwapStatus.Pending && (
              <View style={styles.countdownContainer}>
                <Text style={styles.countdownLabel}>剩余时间</Text>
                <CountdownTimer
                  expireAt={Date.now() + 1800000}
                  onExpire={() => {}}
                />
              </View>
            )}
          </View>
        </View>

        {/* 金额信息 */}
        <View style={styles.section}>
          <View style={styles.amountCard}>
            <View style={styles.amountRow}>
              <View style={styles.amountItem}>
                <Text style={styles.amountLabel}>支付</Text>
                <Text style={styles.amountValue}>
                  {formatDust(record.dustAmount)} DUST
                </Text>
              </View>
              <Text style={styles.arrow}>→</Text>
              <View style={styles.amountItem}>
                <Text style={styles.amountLabel}>获得</Text>
                <Text style={styles.amountValueGreen}>
                  {formatUsdt(record.usdtAmount)} USDT
                </Text>
              </View>
            </View>
          </View>
        </View>

        {/* 详细信息 */}
        <View style={styles.section}>
          <View style={styles.detailCard}>
            <Text style={styles.detailTitle}>详细信息</Text>

            <View style={styles.detailRow}>
              <Text style={styles.detailLabel}>兑换 ID</Text>
              <Text style={styles.detailValue}>#{record.swapId}</Text>
            </View>

            <View style={styles.detailRow}>
              <Text style={styles.detailLabel}>做市商</Text>
              <Text style={styles.detailValue}>#{record.makerId}</Text>
            </View>

            <View style={styles.detailRow}>
              <Text style={styles.detailLabel}>汇率</Text>
              <Text style={styles.detailValue}>
                1 DUST = {(record.priceUsdt / 1e6).toFixed(6)} USDT
              </Text>
            </View>

            <View style={styles.detailRow}>
              <Text style={styles.detailLabel}>创建区块</Text>
              <Text style={styles.detailValue}>#{record.createdAt}</Text>
            </View>

            <View style={styles.detailRow}>
              <Text style={styles.detailLabel}>超时区块</Text>
              <Text style={styles.detailValue}>#{record.timeoutAt}</Text>
            </View>

            {record.completedAt && (
              <View style={styles.detailRow}>
                <Text style={styles.detailLabel}>完成区块</Text>
                <Text style={styles.detailValue}>#{record.completedAt}</Text>
              </View>
            )}
          </View>
        </View>

        {/* 地址信息 */}
        <View style={styles.section}>
          <View style={styles.detailCard}>
            <Text style={styles.detailTitle}>地址信息</Text>

            <View style={styles.addressRow}>
              <Text style={styles.addressLabel}>USDT 收款地址</Text>
              <TouchableOpacity
                onPress={() => handleCopyAddress(record.usdtAddress)}
              >
                <Text style={styles.addressValue}>{record.usdtAddress}</Text>
              </TouchableOpacity>
            </View>

            <View style={styles.addressRow}>
              <Text style={styles.addressLabel}>做市商地址</Text>
              <TouchableOpacity
                onPress={() => handleCopyAddress(record.maker)}
              >
                <Text style={styles.addressValue}>
                  {formatAddress(record.maker)}
                </Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>

        {/* 交易哈希 */}
        {record.trc20TxHash && (
          <View style={styles.section}>
            <View style={styles.detailCard}>
              <Text style={styles.detailTitle}>TRC20 交易</Text>
              <TouchableOpacity
                style={styles.txHashButton}
                onPress={handleViewTronTx}
              >
                <Text style={styles.txHash} numberOfLines={1} ellipsizeMode="middle">
                  {record.trc20TxHash}
                </Text>
                <Text style={styles.txHashLink}>查看 ›</Text>
              </TouchableOpacity>
            </View>
          </View>
        )}

        {/* 操作按钮 */}
        {canReport && (
          <View style={styles.section}>
            <TouchableOpacity
              style={styles.reportButton}
              onPress={handleReport}
            >
              <Text style={styles.reportButtonText}>⚠️ 举报问题</Text>
            </TouchableOpacity>
            <Text style={styles.reportHint}>
              如果做市商未按时转账或金额不符，可发起举报
            </Text>
          </View>
        )}
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
    paddingBottom: 20,
  },
  loading: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  loadingText: {
    fontSize: 14,
    color: '#666666',
    marginTop: 12,
  },
  empty: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  emptyText: {
    fontSize: 16,
    color: '#999999',
  },
  section: {
    padding: 16,
  },
  statusCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 20,
    alignItems: 'center',
  },
  countdownContainer: {
    marginTop: 16,
    alignItems: 'center',
  },
  countdownLabel: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 8,
  },
  amountCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 20,
  },
  amountRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  amountItem: {
    flex: 1,
    alignItems: 'center',
  },
  amountLabel: {
    fontSize: 14,
    color: '#999999',
    marginBottom: 8,
  },
  amountValue: {
    fontSize: 20,
    fontWeight: '600',
    color: '#000000',
  },
  amountValueGreen: {
    fontSize: 20,
    fontWeight: '600',
    color: '#4CD964',
  },
  arrow: {
    fontSize: 24,
    color: '#B2955D',
    marginHorizontal: 16,
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
  detailValue: {
    fontSize: 14,
    color: '#000000',
  },
  addressRow: {
    marginBottom: 12,
  },
  addressLabel: {
    fontSize: 12,
    color: '#999999',
    marginBottom: 4,
  },
  addressValue: {
    fontSize: 14,
    color: '#007AFF',
    fontFamily: 'monospace',
  },
  txHashButton: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    padding: 12,
  },
  txHash: {
    flex: 1,
    fontSize: 12,
    color: '#666666',
    fontFamily: 'monospace',
    marginRight: 8,
  },
  txHashLink: {
    fontSize: 14,
    color: '#007AFF',
  },
  reportButton: {
    backgroundColor: '#FFE5E5',
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#FF3B30',
  },
  reportButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FF3B30',
  },
  reportHint: {
    fontSize: 12,
    color: '#999999',
    textAlign: 'center',
    marginTop: 8,
  },
});
