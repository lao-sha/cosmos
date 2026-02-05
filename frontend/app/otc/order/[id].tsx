import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import {
  Clock,
  Copy,
  CheckCircle,
  AlertTriangle,
  ArrowRight,
} from 'lucide-react-native';
import * as Clipboard from 'expo-clipboard';
import * as Haptics from 'expo-haptics';
import { useColors } from '@/hooks/useColors';
import {
  useOrder,
  useConfirmPayment,
  useCancelOrder,
  formatCos,
  formatUsdt,
  formatPrice,
} from '@/hooks/useOtc';
import { useWalletStore } from '@/stores/wallet';
import { Button, Card, Input } from '@/components/ui';
import { Colors } from '@/constants/colors';

const STATUS_CONFIG = {
  pending: { label: '待支付', color: Colors.warning, icon: Clock },
  paid: { label: '已支付', color: Colors.info, icon: Clock },
  released: { label: '已完成', color: Colors.success, icon: CheckCircle },
  cancelled: { label: '已取消', color: Colors.trading.cancelled, icon: AlertTriangle },
  disputed: { label: '争议中', color: Colors.error, icon: AlertTriangle },
};

export default function OrderDetailScreen() {
  const colors = useColors();
  const router = useRouter();
  const { id } = useLocalSearchParams<{ id: string }>();
  const { data: order, isLoading } = useOrder(id || '');
  const { address } = useWalletStore();
  const confirmPayment = useConfirmPayment();
  const cancelOrder = useCancelOrder();

  const [timeLeft, setTimeLeft] = useState(0);
  const [txHash, setTxHash] = useState('');
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!order || order.status !== 'pending') return;

    const updateTimer = () => {
      const remaining = Math.max(0, order.expiresAt - Date.now());
      setTimeLeft(remaining);
    };

    updateTimer();
    const interval = setInterval(updateTimer, 1000);
    return () => clearInterval(interval);
  }, [order]);

  const formatTime = (ms: number) => {
    const minutes = Math.floor(ms / 60000);
    const seconds = Math.floor((ms % 60000) / 1000);
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
  };

  const handleCopy = async (text: string) => {
    await Clipboard.setStringAsync(text);
    setCopied(true);
    Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleConfirmPayment = async () => {
    if (!txHash.trim()) {
      Alert.alert('提示', '请输入支付凭证/交易号');
      return;
    }

    Alert.alert('确认支付', '请确保您已完成支付，虚假确认将影响信用分', [
      { text: '取消', style: 'cancel' },
      {
        text: '确认',
        onPress: async () => {
          try {
            await confirmPayment.mutateAsync({ orderId: id!, txHash });
            Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
          } catch (error: any) {
            Alert.alert('操作失败', error.message);
          }
        },
      },
    ]);
  };

  const handleCancel = () => {
    Alert.alert('取消订单', '确定要取消此订单吗？', [
      { text: '返回', style: 'cancel' },
      {
        text: '取消订单',
        style: 'destructive',
        onPress: async () => {
          try {
            await cancelOrder.mutateAsync(id!);
            Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
            router.back();
          } catch (error: any) {
            Alert.alert('操作失败', error.message);
          }
        },
      },
    ]);
  };

  const handleDispute = () => {
    router.push(`/disputes/create?orderId=${id}`);
  };

  if (isLoading || !order) {
    return (
      <View style={[styles.container, { backgroundColor: colors.background }]}>
        <Text style={{ color: colors.textSecondary, textAlign: 'center', marginTop: 100 }}>
          加载中...
        </Text>
      </View>
    );
  }

  const statusConfig = STATUS_CONFIG[order.status];
  const isBuyer = order.takerAddress === address;
  const showPaymentInfo = order.status === 'pending' && isBuyer;
  const showConfirmButton = order.status === 'pending' && isBuyer;
  const showCancelButton = order.status === 'pending';
  const showDisputeButton = order.status === 'paid';

  return (
    <ScrollView
      style={[styles.container, { backgroundColor: colors.background }]}
      contentContainerStyle={styles.content}
    >
      {/* Status Header */}
      <View style={[styles.statusHeader, { backgroundColor: statusConfig.color + '15' }]}>
        <statusConfig.icon size={32} color={statusConfig.color} />
        <Text style={[styles.statusText, { color: statusConfig.color }]}>
          {statusConfig.label}
        </Text>
        {order.status === 'pending' && timeLeft > 0 && (
          <View style={styles.timerRow}>
            <Clock size={16} color={Colors.warning} />
            <Text style={[styles.timerText, { color: Colors.warning }]}>
              剩余 {formatTime(timeLeft)}
            </Text>
          </View>
        )}
      </View>

      {/* Order Info */}
      <Card style={styles.orderCard}>
        <Text style={[styles.sectionTitle, { color: colors.textPrimary }]}>
          订单信息
        </Text>

        <View style={styles.infoRow}>
          <Text style={[styles.infoLabel, { color: colors.textSecondary }]}>
            订单号
          </Text>
          <TouchableOpacity
            style={styles.copyRow}
            onPress={() => handleCopy(order.id)}
          >
            <Text style={[styles.infoValue, { color: colors.textPrimary }]}>
              {order.id.slice(0, 8)}...{order.id.slice(-8)}
            </Text>
            <Copy size={14} color={colors.textTertiary} />
          </TouchableOpacity>
        </View>

        <View style={styles.infoRow}>
          <Text style={[styles.infoLabel, { color: colors.textSecondary }]}>
            商家
          </Text>
          <Text style={[styles.infoValue, { color: colors.textPrimary }]}>
            {order.makerName}
          </Text>
        </View>

        <View style={styles.infoRow}>
          <Text style={[styles.infoLabel, { color: colors.textSecondary }]}>
            单价
          </Text>
          <Text style={[styles.infoValue, { color: colors.textPrimary }]}>
            {formatPrice(order.price)} USDT/COS
          </Text>
        </View>

        <View style={styles.divider} />

        <View style={styles.amountRow}>
          <View style={styles.amountItem}>
            <Text style={[styles.amountLabel, { color: colors.textSecondary }]}>
              支付金额
            </Text>
            <Text style={[styles.amountValue, { color: Colors.error }]}>
              {formatUsdt(order.usdtAmount)} USDT
            </Text>
          </View>
          <ArrowRight size={20} color={colors.textTertiary} />
          <View style={styles.amountItem}>
            <Text style={[styles.amountLabel, { color: colors.textSecondary }]}>
              获得数量
            </Text>
            <Text style={[styles.amountValue, { color: Colors.success }]}>
              {formatCos(order.cosAmount)} COS
            </Text>
          </View>
        </View>
      </Card>

      {/* Payment Info */}
      {showPaymentInfo && order.paymentInfo && (
        <Card style={styles.paymentCard}>
          <Text style={[styles.sectionTitle, { color: colors.textPrimary }]}>
            收款信息
          </Text>

          {order.paymentInfo.bankName && (
            <View style={styles.infoRow}>
              <Text style={[styles.infoLabel, { color: colors.textSecondary }]}>
                银行
              </Text>
              <Text style={[styles.infoValue, { color: colors.textPrimary }]}>
                {order.paymentInfo.bankName}
              </Text>
            </View>
          )}

          {order.paymentInfo.accountNumber && (
            <View style={styles.infoRow}>
              <Text style={[styles.infoLabel, { color: colors.textSecondary }]}>
                账号
              </Text>
              <TouchableOpacity
                style={styles.copyRow}
                onPress={() => handleCopy(order.paymentInfo!.accountNumber!)}
              >
                <Text style={[styles.infoValue, { color: colors.textPrimary }]}>
                  {order.paymentInfo.accountNumber}
                </Text>
                <Copy size={14} color={colors.textTertiary} />
              </TouchableOpacity>
            </View>
          )}

          {order.paymentInfo.accountName && (
            <View style={styles.infoRow}>
              <Text style={[styles.infoLabel, { color: colors.textSecondary }]}>
                户名
              </Text>
              <TouchableOpacity
                style={styles.copyRow}
                onPress={() => handleCopy(order.paymentInfo!.accountName!)}
              >
                <Text style={[styles.infoValue, { color: colors.textPrimary }]}>
                  {order.paymentInfo.accountName}
                </Text>
                <Copy size={14} color={colors.textTertiary} />
              </TouchableOpacity>
            </View>
          )}

          <View style={[styles.warningBox, { backgroundColor: Colors.warning + '15' }]}>
            <AlertTriangle size={16} color={Colors.warning} />
            <Text style={[styles.warningText, { color: Colors.warning }]}>
              请务必使用本人实名账户转账，备注订单号
            </Text>
          </View>
        </Card>
      )}

      {/* Confirm Payment Input */}
      {showConfirmButton && (
        <Card style={styles.confirmCard}>
          <Text style={[styles.sectionTitle, { color: colors.textPrimary }]}>
            确认支付
          </Text>
          <Input
            placeholder="输入支付凭证/交易号"
            value={txHash}
            onChangeText={setTxHash}
            containerStyle={styles.txInput}
          />
          <Button
            title="我已支付"
            onPress={handleConfirmPayment}
            loading={confirmPayment.isPending}
            disabled={!txHash.trim()}
          />
        </Card>
      )}

      {/* Status: Waiting for release */}
      {order.status === 'paid' && (
        <Card style={styles.waitingCard}>
          <Clock size={40} color={Colors.info} />
          <Text style={[styles.waitingTitle, { color: colors.textPrimary }]}>
            等待商家放币
          </Text>
          <Text style={[styles.waitingSubtitle, { color: colors.textSecondary }]}>
            商家正在确认您的付款，请耐心等待
          </Text>
        </Card>
      )}

      {/* Action Buttons */}
      <View style={styles.actions}>
        {showCancelButton && (
          <Button
            title="取消订单"
            variant="outline"
            onPress={handleCancel}
            loading={cancelOrder.isPending}
            style={styles.actionButton}
          />
        )}
        {showDisputeButton && (
          <Button
            title="申请仲裁"
            variant="danger"
            onPress={handleDispute}
            style={styles.actionButton}
          />
        )}
      </View>

      {/* Order Timeline */}
      <Card style={styles.timelineCard}>
        <Text style={[styles.sectionTitle, { color: colors.textPrimary }]}>
          订单进度
        </Text>
        <View style={styles.timeline}>
          <TimelineItem
            label="创建订单"
            time={new Date(order.createdAt).toLocaleString()}
            active
            colors={colors}
          />
          {order.paidAt && (
            <TimelineItem
              label="确认支付"
              time={new Date(order.paidAt).toLocaleString()}
              active
              colors={colors}
            />
          )}
          {order.releasedAt && (
            <TimelineItem
              label="放币完成"
              time={new Date(order.releasedAt).toLocaleString()}
              active
              isLast
              colors={colors}
            />
          )}
        </View>
      </Card>
    </ScrollView>
  );
}

function TimelineItem({
  label,
  time,
  active,
  isLast,
  colors,
}: {
  label: string;
  time: string;
  active?: boolean;
  isLast?: boolean;
  colors: any;
}) {
  return (
    <View style={styles.timelineItem}>
      <View style={styles.timelineDot}>
        <View
          style={[
            styles.dot,
            { backgroundColor: active ? Colors.success : colors.border },
          ]}
        />
        {!isLast && (
          <View
            style={[
              styles.line,
              { backgroundColor: active ? Colors.success : colors.border },
            ]}
          />
        )}
      </View>
      <View style={styles.timelineContent}>
        <Text style={[styles.timelineLabel, { color: colors.textPrimary }]}>
          {label}
        </Text>
        <Text style={[styles.timelineTime, { color: colors.textTertiary }]}>
          {time}
        </Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    padding: 16,
    paddingBottom: 40,
  },
  statusHeader: {
    alignItems: 'center',
    padding: 24,
    borderRadius: 16,
    marginBottom: 16,
  },
  statusText: {
    fontSize: 20,
    fontWeight: '700',
    marginTop: 8,
  },
  timerRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    marginTop: 8,
  },
  timerText: {
    fontSize: 16,
    fontWeight: '600',
  },
  orderCard: {
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 16,
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  infoLabel: {
    fontSize: 14,
  },
  infoValue: {
    fontSize: 14,
    fontWeight: '500',
  },
  copyRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
  },
  divider: {
    height: 1,
    backgroundColor: 'rgba(0,0,0,0.05)',
    marginVertical: 16,
  },
  amountRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  amountItem: {
    flex: 1,
  },
  amountLabel: {
    fontSize: 12,
    marginBottom: 4,
  },
  amountValue: {
    fontSize: 18,
    fontWeight: '700',
  },
  paymentCard: {
    marginBottom: 16,
  },
  warningBox: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    padding: 12,
    borderRadius: 8,
    marginTop: 12,
  },
  warningText: {
    flex: 1,
    fontSize: 13,
  },
  confirmCard: {
    marginBottom: 16,
  },
  txInput: {
    marginBottom: 16,
  },
  waitingCard: {
    alignItems: 'center',
    paddingVertical: 32,
    marginBottom: 16,
  },
  waitingTitle: {
    fontSize: 18,
    fontWeight: '600',
    marginTop: 12,
  },
  waitingSubtitle: {
    fontSize: 14,
    marginTop: 4,
  },
  actions: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 16,
  },
  actionButton: {
    flex: 1,
  },
  timelineCard: {
    marginBottom: 16,
  },
  timeline: {},
  timelineItem: {
    flexDirection: 'row',
  },
  timelineDot: {
    alignItems: 'center',
    marginRight: 12,
  },
  dot: {
    width: 10,
    height: 10,
    borderRadius: 5,
  },
  line: {
    width: 2,
    flex: 1,
    minHeight: 30,
  },
  timelineContent: {
    flex: 1,
    paddingBottom: 20,
  },
  timelineLabel: {
    fontSize: 14,
    fontWeight: '500',
  },
  timelineTime: {
    fontSize: 12,
    marginTop: 2,
  },
});
