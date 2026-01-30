import { useSwapRecord } from '@/src/hooks/useSwap';
import { useTransaction } from '@/src/hooks/useTransaction';
import { swapService, SwapStatus } from '@/src/services/swap';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useState } from 'react';
import {
  ActivityIndicator,
  Platform,
  Pressable,
  RefreshControl,
  ScrollView,
  StyleSheet,
  Text,
  View
} from 'react-native';
import * as Clipboard from 'expo-clipboard';

export default function SwapDetailScreen() {
  const router = useRouter();
  const params = useLocalSearchParams<{ id: string }>();
  const swapId = params.id ? parseInt(params.id) : null;
  
  const { isLoggedIn, mnemonic, address } = useAuthStore();
  const { isConnected } = useChainStore();
  const { swap, loading, refresh } = useSwapRecord(swapId);
  const { reportSwap, handleSwapVerificationTimeout, isTxLoading } = useTransaction();

  const [refreshing, setRefreshing] = useState(false);

  const handleRefresh = async () => {
    setRefreshing(true);
    await refresh();
    setRefreshing(false);
  };

  const showAlert = (title: string, message: string) => {
    if (Platform.OS === 'web') {
      window.alert(`${title}\n${message}`);
    } else {
      const { Alert } = require('react-native');
      Alert.alert(title, message);
    }
  };

  const copyToClipboard = async (text: string) => {
    await Clipboard.setStringAsync(text);
    showAlert('已复制', text);
  };

  const isUser = swap?.user === address;

  const handleReport = async () => {
    if (!mnemonic || !swapId) return;

    const doReport = async () => {
      await reportSwap(mnemonic, swapId, {
        onSuccess: () => {
          showAlert('成功', '已提交举报，等待仲裁');
          refresh();
        },
        onError: (error) => {
          showAlert('失败', error);
        },
      });
    };

    if (Platform.OS === 'web') {
      if (window.confirm('确定要举报此兑换吗？举报后将进入仲裁流程。')) {
        doReport();
      }
    } else {
      const { Alert } = require('react-native');
      Alert.alert('举报兑换', '确定要举报此兑换吗？举报后将进入仲裁流程。', [
        { text: '取消', style: 'cancel' },
        { text: '确定举报', onPress: doReport, style: 'destructive' },
      ]);
    }
  };

  const handleTimeout = async () => {
    if (!mnemonic || !swapId) return;

    await handleSwapVerificationTimeout(mnemonic, swapId, {
      onSuccess: () => {
        showAlert('成功', '已处理超时，COS 将退还给您');
        refresh();
      },
      onError: (error) => {
        showAlert('失败', error);
      },
    });
  };

  if (!isConnected) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>兑换详情</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>🔌</Text>
          <Text style={styles.emptyTitle}>未连接网络</Text>
        </View>
      </View>
    );
  }

  if (loading && !swap) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>兑换详情</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color="#10b981" />
          <Text style={styles.loadingText}>加载中...</Text>
        </View>
      </View>
    );
  }

  if (!swap) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>兑换详情</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>❌</Text>
          <Text style={styles.emptyTitle}>兑换不存在</Text>
        </View>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>兑换 #{swap.swapId}</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView
        style={styles.content}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} />
        }
      >
        <View style={styles.statusCard}>
          <View style={[
            styles.statusBadge,
            { backgroundColor: swapService.getStatusColor(swap.status) }
          ]}>
            <Text style={styles.statusText}>
              {swapService.getStatusText(swap.status)}
            </Text>
          </View>
          
          {swap.status === SwapStatus.Pending && (
            <Text style={styles.statusHint}>
              等待做市商转账 USDT 到您的地址
            </Text>
          )}
          {swap.status === SwapStatus.AwaitingVerification && (
            <Text style={styles.statusHint}>
              做市商已提交交易哈希，系统正在验证中
            </Text>
          )}
        </View>

        <View style={styles.card}>
          <Text style={styles.cardTitle}>💱 兑换信息</Text>
          
          <View style={styles.swapAmount}>
            <View style={styles.amountBox}>
              <Text style={styles.amountLabel}>兑出</Text>
              <Text style={styles.amountValue}>{swapService.formatCos(swap.cosAmount)}</Text>
            </View>
            <Text style={styles.amountArrow}>→</Text>
            <View style={styles.amountBox}>
              <Text style={styles.amountLabel}>收到</Text>
              <Text style={styles.amountValueGreen}>{swapService.formatUsdt(swap.usdtAmount)}</Text>
            </View>
          </View>

          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>兑换价格</Text>
            <Text style={styles.infoValue}>${(swap.priceUsdt / 1e6).toFixed(4)}</Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>创建区块</Text>
            <Text style={styles.infoValue}>#{swap.createdAt}</Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>超时区块</Text>
            <Text style={styles.infoValue}>#{swap.timeoutAt}</Text>
          </View>
          {swap.completedAt && (
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>完成区块</Text>
              <Text style={styles.infoValue}>#{swap.completedAt}</Text>
            </View>
          )}
        </View>

        <View style={styles.card}>
          <Text style={styles.cardTitle}>📍 收款地址</Text>
          <Pressable 
            style={styles.addressBox}
            onPress={() => copyToClipboard(swap.usdtAddress)}
          >
            <Text style={styles.addressText}>{swap.usdtAddress}</Text>
            <Text style={styles.copyIcon}>📋</Text>
          </Pressable>
          <Text style={styles.addressHint}>TRC20 USDT 收款地址</Text>
        </View>

        <View style={styles.card}>
          <Text style={styles.cardTitle}>👤 做市商信息</Text>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>做市商 ID</Text>
            <Text style={styles.infoValue}>#{swap.makerId}</Text>
          </View>
          <Pressable 
            style={styles.copyRow}
            onPress={() => copyToClipboard(swap.maker)}
          >
            <Text style={styles.infoLabel}>做市商地址</Text>
            <Text style={styles.copyText} numberOfLines={1}>{swap.maker}</Text>
            <Text style={styles.copyIcon}>📋</Text>
          </Pressable>
        </View>

        {swap.trc20TxHash && (
          <View style={styles.card}>
            <Text style={styles.cardTitle}>🔗 TRC20 交易</Text>
            <Pressable 
              style={styles.addressBox}
              onPress={() => copyToClipboard(swap.trc20TxHash!)}
            >
              <Text style={styles.addressText} numberOfLines={1}>{swap.trc20TxHash}</Text>
              <Text style={styles.copyIcon}>📋</Text>
            </Pressable>
          </View>
        )}

        {swap.status === SwapStatus.Pending && isUser && (
          <View style={styles.actionCard}>
            <Text style={styles.actionTitle}>⏳ 等待做市商转账</Text>
            <Text style={styles.actionDesc}>
              做市商需在超时前向您的 TRC20 地址转账 {swapService.formatUsdt(swap.usdtAmount)}
            </Text>
            
            <Pressable
              style={styles.reportButton}
              onPress={handleReport}
              disabled={isTxLoading}
            >
              <Text style={styles.reportButtonText}>举报做市商</Text>
            </Pressable>
          </View>
        )}

        {swap.status === SwapStatus.AwaitingVerification && isUser && (
          <View style={styles.actionCard}>
            <Text style={styles.actionTitle}>🔍 验证中</Text>
            <Text style={styles.actionDesc}>
              系统正在验证 TRC20 交易，验证成功后 COS 将释放给做市商
            </Text>
            
            <Pressable
              style={[styles.timeoutButton, isTxLoading && styles.buttonDisabled]}
              onPress={handleTimeout}
              disabled={isTxLoading}
            >
              {isTxLoading ? (
                <ActivityIndicator color="#f59e0b" size="small" />
              ) : (
                <Text style={styles.timeoutButtonText}>处理验证超时</Text>
              )}
            </Pressable>
            <Text style={styles.timeoutHint}>如验证超时，点击此按钮可触发退款</Text>
          </View>
        )}

        {swap.status === SwapStatus.Completed && (
          <View style={styles.successCard}>
            <Text style={styles.successIcon}>🎉</Text>
            <Text style={styles.successTitle}>兑换完成</Text>
            <Text style={styles.successDesc}>
              USDT 已转入您的 TRC20 地址
            </Text>
          </View>
        )}

        {swap.status === SwapStatus.Refunded && (
          <View style={styles.refundedCard}>
            <Text style={styles.refundedIcon}>↩️</Text>
            <Text style={styles.refundedTitle}>已退款</Text>
            <Text style={styles.refundedDesc}>
              COS 已退还到您的账户
            </Text>
          </View>
        )}

        {swap.status === SwapStatus.VerificationFailed && (
          <View style={styles.failedCard}>
            <Text style={styles.failedIcon}>❌</Text>
            <Text style={styles.failedTitle}>验证失败</Text>
            <Text style={styles.failedDesc}>
              TRC20 交易验证失败，请联系做市商或发起举报
            </Text>
            
            {isUser && (
              <Pressable
                style={styles.reportButton}
                onPress={handleReport}
                disabled={isTxLoading}
              >
                <Text style={styles.reportButtonText}>举报做市商</Text>
              </Pressable>
            )}
          </View>
        )}

        {(swap.status === SwapStatus.UserReported || swap.status === SwapStatus.Arbitrating) && (
          <View style={styles.arbitratingCard}>
            <Text style={styles.arbitratingIcon}>⚖️</Text>
            <Text style={styles.arbitratingTitle}>仲裁中</Text>
            <Text style={styles.arbitratingDesc}>
              您的举报已提交，请等待仲裁结果
            </Text>
          </View>
        )}

        <View style={styles.bottomPadding} />
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    backgroundColor: '#10b981',
    paddingTop: 50,
    paddingBottom: 16,
    paddingHorizontal: 16,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  backButton: {
    padding: 4,
  },
  backText: {
    color: '#fff',
    fontSize: 18,
  },
  headerTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#fff',
  },
  headerRight: {
    width: 40,
  },
  content: {
    flex: 1,
  },
  loadingContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  loadingText: {
    marginTop: 12,
    fontSize: 14,
    color: '#6b7280',
  },
  emptyContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
  },
  emptyIcon: {
    fontSize: 64,
    marginBottom: 16,
  },
  emptyTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#1f2937',
  },
  statusCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 20,
    borderRadius: 12,
    alignItems: 'center',
  },
  statusBadge: {
    paddingHorizontal: 20,
    paddingVertical: 8,
    borderRadius: 20,
  },
  statusText: {
    fontSize: 16,
    color: '#fff',
    fontWeight: '600',
  },
  statusHint: {
    marginTop: 12,
    fontSize: 14,
    color: '#6b7280',
    textAlign: 'center',
  },
  card: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 12,
  },
  swapAmount: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 12,
    marginBottom: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  amountBox: {
    alignItems: 'center',
    flex: 1,
  },
  amountLabel: {
    fontSize: 12,
    color: '#9ca3af',
    marginBottom: 4,
  },
  amountValue: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#1f2937',
  },
  amountValueGreen: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#10b981',
  },
  amountArrow: {
    fontSize: 24,
    color: '#9ca3af',
    paddingHorizontal: 12,
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 10,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  infoLabel: {
    fontSize: 14,
    color: '#6b7280',
  },
  infoValue: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1f2937',
  },
  addressBox: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#f9fafb',
    padding: 12,
    borderRadius: 8,
  },
  addressText: {
    flex: 1,
    fontSize: 13,
    color: '#1f2937',
    fontFamily: 'monospace',
  },
  addressHint: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 8,
  },
  copyRow: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 10,
  },
  copyText: {
    flex: 1,
    fontSize: 12,
    color: '#1f2937',
    marginLeft: 8,
    fontFamily: 'monospace',
  },
  copyIcon: {
    fontSize: 16,
    marginLeft: 8,
  },
  actionCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  actionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 8,
  },
  actionDesc: {
    fontSize: 14,
    color: '#6b7280',
    marginBottom: 16,
    lineHeight: 20,
  },
  reportButton: {
    backgroundColor: '#fef2f2',
    paddingVertical: 14,
    borderRadius: 10,
    alignItems: 'center',
  },
  reportButtonText: {
    color: '#dc2626',
    fontSize: 16,
    fontWeight: '600',
  },
  timeoutButton: {
    backgroundColor: '#fef3c7',
    paddingVertical: 14,
    borderRadius: 10,
    alignItems: 'center',
    marginBottom: 8,
  },
  timeoutButtonText: {
    color: '#92400e',
    fontSize: 16,
    fontWeight: '600',
  },
  timeoutHint: {
    fontSize: 12,
    color: '#9ca3af',
    textAlign: 'center',
  },
  buttonDisabled: {
    opacity: 0.6,
  },
  successCard: {
    backgroundColor: '#ecfdf5',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 24,
    borderRadius: 12,
    alignItems: 'center',
  },
  successIcon: {
    fontSize: 48,
    marginBottom: 12,
  },
  successTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#065f46',
    marginBottom: 8,
  },
  successDesc: {
    fontSize: 14,
    color: '#047857',
  },
  refundedCard: {
    backgroundColor: '#f3f4f6',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 24,
    borderRadius: 12,
    alignItems: 'center',
  },
  refundedIcon: {
    fontSize: 48,
    marginBottom: 12,
  },
  refundedTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#6b7280',
    marginBottom: 8,
  },
  refundedDesc: {
    fontSize: 14,
    color: '#6b7280',
  },
  failedCard: {
    backgroundColor: '#fef2f2',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 24,
    borderRadius: 12,
    alignItems: 'center',
  },
  failedIcon: {
    fontSize: 48,
    marginBottom: 12,
  },
  failedTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#dc2626',
    marginBottom: 8,
  },
  failedDesc: {
    fontSize: 14,
    color: '#991b1b',
    textAlign: 'center',
    marginBottom: 16,
  },
  arbitratingCard: {
    backgroundColor: '#f5f3ff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 24,
    borderRadius: 12,
    alignItems: 'center',
  },
  arbitratingIcon: {
    fontSize: 48,
    marginBottom: 12,
  },
  arbitratingTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#6d28d9',
    marginBottom: 8,
  },
  arbitratingDesc: {
    fontSize: 14,
    color: '#7c3aed',
  },
  bottomPadding: {
    height: 40,
  },
});
