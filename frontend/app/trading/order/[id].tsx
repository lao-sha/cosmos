import { TransactionModal } from '@/src/components/TransactionModal';
import { useTransaction } from '@/src/hooks/useTransaction';
import { useAuthStore } from '@/src/stores/auth';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useState } from 'react';
import {
    Pressable,
    ScrollView,
    StyleSheet,
    Text,
    View
} from 'react-native';

type OtcOrderStatus = 'open' | 'locked' | 'paid' | 'released' | 'disputed' | 'cancelled';

interface OtcOrderDetail {
  id: string;
  maker: string;
  taker?: string;
  side: 'buy' | 'sell';
  amount: string;
  price: string;
  currency: string;
  status: OtcOrderStatus;
  createdAt: string;
  lockedAt?: string;
  paidAt?: string;
  releasedAt?: string;
  paymentInfo?: {
    method: string;
    account: string;
    name: string;
  };
}

const STATUS_MAP: Record<OtcOrderStatus, { label: string; color: string; bg: string }> = {
  open: { label: '待接单', color: '#3b82f6', bg: '#dbeafe' },
  locked: { label: '已锁定', color: '#f59e0b', bg: '#fef3c7' },
  paid: { label: '已付款', color: '#6D28D9', bg: '#f3e8ff' },
  released: { label: '已完成', color: '#16a34a', bg: '#dcfce7' },
  disputed: { label: '申诉中', color: '#dc2626', bg: '#fee2e2' },
  cancelled: { label: '已取消', color: '#6b7280', bg: '#f3f4f6' },
};

const MOCK_ORDER: OtcOrderDetail = {
  id: 'OTC20240120001',
  maker: '5GrwvaEF...JMakT',
  side: 'sell',
  amount: '1000',
  price: '7.2',
  currency: 'CNY',
  status: 'locked',
  createdAt: '2024-01-20 10:30',
  lockedAt: '2024-01-20 11:15',
  paymentInfo: {
    method: '银行卡',
    account: '6222 **** **** 1234',
    name: '张**',
  },
};

export default function OtcOrderDetailScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const router = useRouter();
  const { address } = useAuthStore();
  const { lockOtcOrder, isLoading, status } = useTransaction();
  
  const [order, setOrder] = useState<OtcOrderDetail>(MOCK_ORDER);
  const [showConfirmModal, setShowConfirmModal] = useState(false);
  const [modalAction, setModalAction] = useState<'lock' | 'pay' | 'release' | 'dispute'>('lock');

  const statusInfo = STATUS_MAP[order.status];
  const isMaker = order.maker === address;
  const isTaker = order.taker === address;
  const totalAmount = (parseFloat(order.amount) * parseFloat(order.price)).toFixed(2);

  const handleAction = (action: typeof modalAction) => {
    setModalAction(action);
    setShowConfirmModal(true);
  };

  const handleConfirm = async () => {
    switch (modalAction) {
      case 'lock':
        const result = await lockOtcOrder(order.id);
        if (result?.success) {
          setOrder(prev => ({ ...prev, status: 'locked', taker: address || undefined }));
        }
        break;
      case 'pay':
        setOrder(prev => ({ ...prev, status: 'paid', paidAt: new Date().toLocaleString('zh-CN') }));
        break;
      case 'release':
        setOrder(prev => ({ ...prev, status: 'released', releasedAt: new Date().toLocaleString('zh-CN') }));
        break;
      case 'dispute':
        setOrder(prev => ({ ...prev, status: 'disputed' }));
        break;
    }
    setShowConfirmModal(false);
  };

  const getModalConfig = () => {
    switch (modalAction) {
      case 'lock':
        return {
          title: '确认接单',
          description: `您将${order.side === 'sell' ? '购买' : '出售'} ${order.amount} STAR`,
          amount: `¥${totalAmount}`,
          confirmText: '确认接单',
        };
      case 'pay':
        return {
          title: '确认付款',
          description: '请确认您已完成线下付款',
          amount: `¥${totalAmount}`,
          confirmText: '我已付款',
        };
      case 'release':
        return {
          title: '确认放币',
          description: '请确认您已收到付款，放币后不可撤回',
          amount: `${order.amount} STAR`,
          confirmText: '确认放币',
        };
      case 'dispute':
        return {
          title: '发起申诉',
          description: '申诉将由平台介入处理，请确保您有相关证据',
          confirmText: '确认申诉',
        };
      default:
        return { title: '', confirmText: '' };
    }
  };

  const modalConfig = getModalConfig();

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>订单详情</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.statusCard}>
          <View style={[styles.statusBadge, { backgroundColor: statusInfo.bg }]}>
            <Text style={[styles.statusText, { color: statusInfo.color }]}>
              {statusInfo.label}
            </Text>
          </View>
          <Text style={styles.orderId}>订单号: {order.id}</Text>
        </View>

        <View style={styles.tradeCard}>
          <View style={styles.tradeRow}>
            <Text style={styles.tradeLabel}>交易类型</Text>
            <Text style={[styles.tradeSide, order.side === 'sell' ? styles.sellText : styles.buyText]}>
              {order.side === 'sell' ? '出售' : '购买'}
            </Text>
          </View>
          <View style={styles.amountRow}>
            <Text style={styles.amountValue}>{order.amount}</Text>
            <Text style={styles.amountUnit}>STAR</Text>
          </View>
          <View style={styles.priceRow}>
            <Text style={styles.priceLabel}>单价</Text>
            <Text style={styles.priceValue}>¥{order.price}</Text>
          </View>
          <View style={styles.totalRow}>
            <Text style={styles.totalLabel}>总金额</Text>
            <Text style={styles.totalValue}>¥{totalAmount}</Text>
          </View>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>交易方信息</Text>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>挂单方</Text>
            <Text style={styles.infoValue}>{order.maker}</Text>
          </View>
          {order.taker && (
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>接单方</Text>
              <Text style={styles.infoValue}>{order.taker}</Text>
            </View>
          )}
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>创建时间</Text>
            <Text style={styles.infoValue}>{order.createdAt}</Text>
          </View>
          {order.lockedAt && (
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>锁定时间</Text>
              <Text style={styles.infoValue}>{order.lockedAt}</Text>
            </View>
          )}
        </View>

        {order.paymentInfo && order.status !== 'open' && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>收款信息</Text>
            <View style={styles.paymentCard}>
              <View style={styles.paymentRow}>
                <Text style={styles.paymentLabel}>收款方式</Text>
                <Text style={styles.paymentValue}>{order.paymentInfo.method}</Text>
              </View>
              <View style={styles.paymentRow}>
                <Text style={styles.paymentLabel}>收款账号</Text>
                <Text style={styles.paymentValue}>{order.paymentInfo.account}</Text>
              </View>
              <View style={styles.paymentRow}>
                <Text style={styles.paymentLabel}>收款人</Text>
                <Text style={styles.paymentValue}>{order.paymentInfo.name}</Text>
              </View>
            </View>
            <Text style={styles.paymentHint}>
              ⚠️ 请务必使用实名账户转账，并备注订单号
            </Text>
          </View>
        )}

        {order.status === 'locked' && (
          <View style={styles.timerBox}>
            <Text style={styles.timerIcon}>⏱</Text>
            <Text style={styles.timerText}>请在 15:00 内完成付款</Text>
            <Text style={styles.timerHint}>超时订单将自动取消</Text>
          </View>
        )}

        <View style={styles.bottomSpacer} />
      </ScrollView>

      <View style={styles.bottomBar}>
        {order.status === 'open' && !isMaker && (
          <Pressable 
            style={styles.primaryButton}
            onPress={() => handleAction('lock')}
          >
            <Text style={styles.primaryButtonText}>
              {order.side === 'sell' ? '我要购买' : '我要出售'}
            </Text>
          </Pressable>
        )}

        {order.status === 'locked' && isTaker && (
          <>
            <Pressable 
              style={styles.primaryButton}
              onPress={() => handleAction('pay')}
            >
              <Text style={styles.primaryButtonText}>我已付款</Text>
            </Pressable>
            <Pressable 
              style={styles.secondaryButton}
              onPress={() => handleAction('dispute')}
            >
              <Text style={styles.secondaryButtonText}>申诉</Text>
            </Pressable>
          </>
        )}

        {order.status === 'paid' && isMaker && (
          <>
            <Pressable 
              style={styles.primaryButton}
              onPress={() => handleAction('release')}
            >
              <Text style={styles.primaryButtonText}>确认放币</Text>
            </Pressable>
            <Pressable 
              style={styles.secondaryButton}
              onPress={() => handleAction('dispute')}
            >
              <Text style={styles.secondaryButtonText}>申诉</Text>
            </Pressable>
          </>
        )}

        {order.status === 'released' && (
          <View style={styles.completedBox}>
            <Text style={styles.completedIcon}>✅</Text>
            <Text style={styles.completedText}>交易已完成</Text>
          </View>
        )}
      </View>

      <TransactionModal
        visible={showConfirmModal}
        onClose={() => setShowConfirmModal(false)}
        title={modalConfig.title}
        description={modalConfig.description}
        amount={modalConfig.amount}
        status={status}
        isLoading={isLoading}
        onConfirm={handleConfirm}
        confirmText={modalConfig.confirmText}
      />
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
    padding: 4,
  },
  backText: {
    fontSize: 17,
    color: '#6D28D9',
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#1f2937',
  },
  headerRight: {
    width: 50,
  },
  content: {
    flex: 1,
  },
  statusCard: {
    backgroundColor: '#fff',
    padding: 20,
    alignItems: 'center',
    marginBottom: 12,
  },
  statusBadge: {
    paddingHorizontal: 16,
    paddingVertical: 6,
    borderRadius: 16,
    marginBottom: 12,
  },
  statusText: {
    fontSize: 15,
    fontWeight: '600',
  },
  orderId: {
    fontSize: 13,
    color: '#6b7280',
  },
  tradeCard: {
    backgroundColor: '#fff',
    padding: 20,
    marginBottom: 12,
  },
  tradeRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  tradeLabel: {
    fontSize: 14,
    color: '#6b7280',
  },
  tradeSide: {
    fontSize: 14,
    fontWeight: '600',
  },
  sellText: {
    color: '#dc2626',
  },
  buyText: {
    color: '#16a34a',
  },
  amountRow: {
    flexDirection: 'row',
    alignItems: 'baseline',
    justifyContent: 'center',
    marginBottom: 16,
  },
  amountValue: {
    fontSize: 36,
    fontWeight: '700',
    color: '#1f2937',
  },
  amountUnit: {
    fontSize: 18,
    color: '#6b7280',
    marginLeft: 8,
  },
  priceRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 8,
    borderTopWidth: 1,
    borderTopColor: '#f3f4f6',
  },
  priceLabel: {
    fontSize: 14,
    color: '#6b7280',
  },
  priceValue: {
    fontSize: 14,
    color: '#1f2937',
  },
  totalRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 8,
  },
  totalLabel: {
    fontSize: 14,
    color: '#6b7280',
  },
  totalValue: {
    fontSize: 18,
    fontWeight: '700',
    color: '#6D28D9',
  },
  section: {
    backgroundColor: '#fff',
    padding: 16,
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 12,
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
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
    color: '#1f2937',
  },
  paymentCard: {
    backgroundColor: '#f9fafb',
    borderRadius: 8,
    padding: 12,
    marginBottom: 8,
  },
  paymentRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 6,
  },
  paymentLabel: {
    fontSize: 13,
    color: '#6b7280',
  },
  paymentValue: {
    fontSize: 13,
    color: '#1f2937',
    fontWeight: '500',
  },
  paymentHint: {
    fontSize: 12,
    color: '#f59e0b',
  },
  timerBox: {
    backgroundColor: '#fff',
    padding: 24,
    alignItems: 'center',
    marginBottom: 12,
  },
  timerIcon: {
    fontSize: 32,
    marginBottom: 8,
  },
  timerText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#f59e0b',
  },
  timerHint: {
    fontSize: 13,
    color: '#9ca3af',
    marginTop: 4,
  },
  bottomSpacer: {
    height: 100,
  },
  bottomBar: {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
    flexDirection: 'row',
    backgroundColor: '#fff',
    paddingHorizontal: 16,
    paddingVertical: 12,
    paddingBottom: 30,
    borderTopWidth: 1,
    borderTopColor: '#e5e7eb',
    gap: 12,
  },
  primaryButton: {
    flex: 2,
    backgroundColor: '#6D28D9',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  primaryButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#fff',
  },
  secondaryButton: {
    flex: 1,
    backgroundColor: '#f3f4f6',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  secondaryButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#6b7280',
  },
  completedBox: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 8,
  },
  completedIcon: {
    fontSize: 20,
  },
  completedText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#16a34a',
  },
});
