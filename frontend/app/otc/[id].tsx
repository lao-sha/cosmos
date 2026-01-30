import { useOtcOrder } from '@/src/hooks/useOtc';
import { useTransaction } from '@/src/hooks/useTransaction';
import { otcService, OrderState } from '@/src/services/otc';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useEffect, useState } from 'react';
import {
  ActivityIndicator,
  Platform,
  Pressable,
  RefreshControl,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View
} from 'react-native';
import * as Clipboard from 'expo-clipboard';

export default function OtcOrderDetailScreen() {
  const router = useRouter();
  const params = useLocalSearchParams<{ id: string }>();
  const orderId = params.id ? parseInt(params.id) : null;
  
  const { isLoggedIn, mnemonic, address } = useAuthStore();
  const { isConnected } = useChainStore();
  const { order, loading, refresh } = useOtcOrder(orderId);
  const { markOtcPaid, releaseOtcCos, cancelOtcOrder, disputeOtcOrder, isTxLoading } = useTransaction();

  const [refreshing, setRefreshing] = useState(false);
  const [tronTxHash, setTronTxHash] = useState('');
  const [showTxInput, setShowTxInput] = useState(false);

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

  const isBuyer = order?.taker === address;
  const isMaker = order?.maker === address;

  const handleMarkPaid = async () => {
    if (!mnemonic || !orderId) return;

    await markOtcPaid(mnemonic, orderId, tronTxHash || undefined, {
      onSuccess: () => {
        showAlert('成功', '已标记付款，请等待做市商确认');
        setShowTxInput(false);
        setTronTxHash('');
        refresh();
      },
      onError: (error) => {
        showAlert('失败', error);
      },
    });
  };

  const handleReleaseCos = async () => {
    if (!mnemonic || !orderId) return;

    const doRelease = async () => {
      await releaseOtcCos(mnemonic, orderId, {
        onSuccess: () => {
          showAlert('成功', 'COS 已释放给买家');
          refresh();
        },
        onError: (error) => {
          showAlert('失败', error);
        },
      });
    };

    if (Platform.OS === 'web') {
      if (window.confirm('确认已收到买家付款？释放后 COS 将转给买家。')) {
        doRelease();
      }
    } else {
      const { Alert } = require('react-native');
      Alert.alert('确认放币', '确认已收到买家付款？释放后 COS 将转给买家。', [
        { text: '取消', style: 'cancel' },
        { text: '确认放币', onPress: doRelease },
      ]);
    }
  };

  const handleCancelOrder = async () => {
    if (!mnemonic || !orderId) return;

    const doCancel = async () => {
      await cancelOtcOrder(mnemonic, orderId, {
        onSuccess: () => {
          showAlert('成功', '订单已取消');
          refresh();
        },
        onError: (error) => {
          showAlert('失败', error);
        },
      });
    };

    if (Platform.OS === 'web') {
      if (window.confirm('确定要取消订单吗？')) {
        doCancel();
      }
    } else {
      const { Alert } = require('react-native');
      Alert.alert('取消订单', '确定要取消订单吗？', [
        { text: '取消', style: 'cancel' },
        { text: '确定', onPress: doCancel, style: 'destructive' },
      ]);
    }
  };

  const handleDispute = async () => {
    if (!mnemonic || !orderId) return;

    const doDispute = async () => {
      await disputeOtcOrder(mnemonic, orderId, {
        onSuccess: () => {
          showAlert('成功', '已发起争议，请等待仲裁');
          refresh();
        },
        onError: (error) => {
          showAlert('失败', error);
        },
      });
    };

    if (Platform.OS === 'web') {
      if (window.confirm('确定要发起争议吗？争议将进入仲裁流程。')) {
        doDispute();
      }
    } else {
      const { Alert } = require('react-native');
      Alert.alert('发起争议', '确定要发起争议吗？争议将进入仲裁流程。', [
        { text: '取消', style: 'cancel' },
        { text: '发起争议', onPress: doDispute, style: 'destructive' },
      ]);
    }
  };

  if (!isConnected) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>订单详情</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>🔌</Text>
          <Text style={styles.emptyTitle}>未连接网络</Text>
        </View>
      </View>
    );
  }

  if (loading && !order) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>订单详情</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color="#6D28D9" />
          <Text style={styles.loadingText}>加载中...</Text>
        </View>
      </View>
    );
  }

  if (!order) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>订单详情</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>❌</Text>
          <Text style={styles.emptyTitle}>订单不存在</Text>
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
        <Text style={styles.headerTitle}>订单 #{order.orderId}</Text>
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
            { backgroundColor: otcService.getStateColor(order.state) }
          ]}>
            <Text style={styles.statusText}>
              {otcService.getStateText(order.state)}
            </Text>
          </View>
          
          {order.state === OrderState.Created && !otcService.isExpired(order.expireAt) && (
            <View style={styles.countdownBox}>
              <Text style={styles.countdownLabel}>付款剩余时间</Text>
              <Text style={styles.countdownValue}>
                {otcService.getRemainingTime(order.expireAt)}
              </Text>
            </View>
          )}
        </View>

        <View style={styles.card}>
          <Text style={styles.cardTitle}>💰 订单信息</Text>
          
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>COS 数量</Text>
            <Text style={styles.infoValue}>{otcService.formatCos(order.qty)}</Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>USDT 金额</Text>
            <Text style={styles.infoValueLarge}>{otcService.formatUsdt(order.amount)}</Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>创建时间</Text>
            <Text style={styles.infoValue}>{otcService.formatTime(order.createdAt)}</Text>
          </View>
          {order.completedAt && (
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>完成时间</Text>
              <Text style={styles.infoValue}>{otcService.formatTime(order.completedAt)}</Text>
            </View>
          )}
          {order.isFirstPurchase && (
            <View style={styles.firstPurchaseBadge}>
              <Text style={styles.firstPurchaseText}>🎁 首购订单</Text>
            </View>
          )}
        </View>

        <View style={styles.card}>
          <Text style={styles.cardTitle}>👤 做市商信息</Text>
          
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>做市商 ID</Text>
            <Text style={styles.infoValue}>#{order.makerId}</Text>
          </View>
          <Pressable 
            style={styles.copyRow}
            onPress={() => copyToClipboard(order.makerTronAddress)}
          >
            <Text style={styles.infoLabel}>TRON 收款地址</Text>
            <Text style={styles.copyText}>{order.makerTronAddress}</Text>
            <Text style={styles.copyIcon}>📋</Text>
          </Pressable>
        </View>

        {order.state === OrderState.Created && isBuyer && !otcService.isExpired(order.expireAt) && (
          <View style={styles.actionCard}>
            <Text style={styles.actionTitle}>📱 付款操作</Text>
            <Text style={styles.actionDesc}>
              请向做市商的 TRON 地址转账 {otcService.formatUsdt(order.amount)}，完成后点击"已付款"
            </Text>
            
            {showTxInput ? (
              <View style={styles.txInputContainer}>
                <TextInput
                  style={styles.txInput}
                  value={tronTxHash}
                  onChangeText={setTronTxHash}
                  placeholder="输入 TRON 交易哈希（可选）"
                  placeholderTextColor="#9ca3af"
                  autoCapitalize="none"
                />
                <View style={styles.txInputButtons}>
                  <Pressable
                    style={styles.txCancelButton}
                    onPress={() => {
                      setShowTxInput(false);
                      setTronTxHash('');
                    }}
                  >
                    <Text style={styles.txCancelButtonText}>取消</Text>
                  </Pressable>
                  <Pressable
                    style={[styles.txConfirmButton, isTxLoading && styles.buttonDisabled]}
                    onPress={handleMarkPaid}
                    disabled={isTxLoading}
                  >
                    {isTxLoading ? (
                      <ActivityIndicator color="#fff" size="small" />
                    ) : (
                      <Text style={styles.txConfirmButtonText}>确认</Text>
                    )}
                  </Pressable>
                </View>
              </View>
            ) : (
              <Pressable
                style={styles.primaryButton}
                onPress={() => setShowTxInput(true)}
              >
                <Text style={styles.primaryButtonText}>我已付款</Text>
              </Pressable>
            )}
            
            <Pressable
              style={styles.cancelButton}
              onPress={handleCancelOrder}
              disabled={isTxLoading}
            >
              <Text style={styles.cancelButtonText}>取消订单</Text>
            </Pressable>
          </View>
        )}

        {order.state === OrderState.PaidOrCommitted && isMaker && (
          <View style={styles.actionCard}>
            <Text style={styles.actionTitle}>✅ 确认收款</Text>
            <Text style={styles.actionDesc}>
              买家已标记付款，请确认收到 {otcService.formatUsdt(order.amount)} 后释放 COS
            </Text>
            
            <Pressable
              style={[styles.primaryButton, isTxLoading && styles.buttonDisabled]}
              onPress={handleReleaseCos}
              disabled={isTxLoading}
            >
              {isTxLoading ? (
                <ActivityIndicator color="#fff" />
              ) : (
                <Text style={styles.primaryButtonText}>确认收款，释放 COS</Text>
              )}
            </Pressable>
            
            <Pressable
              style={styles.disputeButton}
              onPress={handleDispute}
              disabled={isTxLoading}
            >
              <Text style={styles.disputeButtonText}>发起争议</Text>
            </Pressable>
          </View>
        )}

        {order.state === OrderState.PaidOrCommitted && isBuyer && (
          <View style={styles.actionCard}>
            <Text style={styles.actionTitle}>⏳ 等待放币</Text>
            <Text style={styles.actionDesc}>
              您已标记付款，请等待做市商确认收款后释放 COS
            </Text>
            
            <Pressable
              style={styles.disputeButton}
              onPress={handleDispute}
              disabled={isTxLoading}
            >
              <Text style={styles.disputeButtonText}>发起争议</Text>
            </Pressable>
          </View>
        )}

        {order.state === OrderState.Disputed && (
          <View style={styles.disputeCard}>
            <Text style={styles.disputeTitle}>⚠️ 订单争议中</Text>
            <Text style={styles.disputeDesc}>
              订单已进入争议状态，请等待仲裁结果
            </Text>
          </View>
        )}

        {order.state === OrderState.Released && (
          <View style={styles.successCard}>
            <Text style={styles.successIcon}>🎉</Text>
            <Text style={styles.successTitle}>交易完成</Text>
            <Text style={styles.successDesc}>
              COS 已成功转入您的账户
            </Text>
          </View>
        )}

        {order.state === OrderState.Canceled && (
          <View style={styles.canceledCard}>
            <Text style={styles.canceledIcon}>❌</Text>
            <Text style={styles.canceledTitle}>订单已取消</Text>
          </View>
        )}

        {order.state === OrderState.Expired && (
          <View style={styles.expiredCard}>
            <Text style={styles.expiredIcon}>⏰</Text>
            <Text style={styles.expiredTitle}>订单已过期</Text>
            <Text style={styles.expiredDesc}>
              订单超时未付款，已自动取消
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
    backgroundColor: '#6D28D9',
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
  countdownBox: {
    marginTop: 16,
    alignItems: 'center',
  },
  countdownLabel: {
    fontSize: 12,
    color: '#6b7280',
    marginBottom: 4,
  },
  countdownValue: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#f59e0b',
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
  infoValueLarge: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#16a34a',
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
  firstPurchaseBadge: {
    backgroundColor: '#fef3c7',
    padding: 10,
    borderRadius: 8,
    marginTop: 12,
    alignItems: 'center',
  },
  firstPurchaseText: {
    fontSize: 14,
    color: '#92400e',
    fontWeight: '500',
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
  txInputContainer: {
    marginBottom: 12,
  },
  txInput: {
    backgroundColor: '#f9fafb',
    borderWidth: 1,
    borderColor: '#e5e7eb',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 12,
    fontSize: 14,
    color: '#1f2937',
    marginBottom: 12,
  },
  txInputButtons: {
    flexDirection: 'row',
    gap: 12,
  },
  txCancelButton: {
    flex: 1,
    backgroundColor: '#f3f4f6',
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
  },
  txCancelButtonText: {
    color: '#6b7280',
    fontSize: 14,
    fontWeight: '600',
  },
  txConfirmButton: {
    flex: 2,
    backgroundColor: '#6D28D9',
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
  },
  txConfirmButtonText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
  },
  primaryButton: {
    backgroundColor: '#6D28D9',
    paddingVertical: 14,
    borderRadius: 10,
    alignItems: 'center',
    marginBottom: 12,
  },
  primaryButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  cancelButton: {
    backgroundColor: '#f3f4f6',
    paddingVertical: 14,
    borderRadius: 10,
    alignItems: 'center',
  },
  cancelButtonText: {
    color: '#6b7280',
    fontSize: 16,
    fontWeight: '600',
  },
  disputeButton: {
    backgroundColor: '#fef2f2',
    paddingVertical: 14,
    borderRadius: 10,
    alignItems: 'center',
  },
  disputeButtonText: {
    color: '#dc2626',
    fontSize: 16,
    fontWeight: '600',
  },
  buttonDisabled: {
    opacity: 0.6,
  },
  disputeCard: {
    backgroundColor: '#fef2f2',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 20,
    borderRadius: 12,
    alignItems: 'center',
  },
  disputeTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#dc2626',
    marginBottom: 8,
  },
  disputeDesc: {
    fontSize: 14,
    color: '#991b1b',
    textAlign: 'center',
  },
  successCard: {
    backgroundColor: '#f0fdf4',
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
    color: '#16a34a',
    marginBottom: 8,
  },
  successDesc: {
    fontSize: 14,
    color: '#15803d',
  },
  canceledCard: {
    backgroundColor: '#f3f4f6',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 24,
    borderRadius: 12,
    alignItems: 'center',
  },
  canceledIcon: {
    fontSize: 48,
    marginBottom: 12,
  },
  canceledTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#6b7280',
  },
  expiredCard: {
    backgroundColor: '#fef3c7',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 24,
    borderRadius: 12,
    alignItems: 'center',
  },
  expiredIcon: {
    fontSize: 48,
    marginBottom: 12,
  },
  expiredTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#92400e',
    marginBottom: 8,
  },
  expiredDesc: {
    fontSize: 14,
    color: '#92400e',
  },
  bottomPadding: {
    height: 40,
  },
});
