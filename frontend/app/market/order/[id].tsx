import { TransactionModal } from '@/src/components/TransactionModal';
import { useTransaction } from '@/src/hooks/useTransaction';
import { useAuthStore } from '@/src/stores/auth';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useState } from 'react';
import {
    Alert,
    Platform,
    Pressable,
    ScrollView,
    StyleSheet,
    Text,
    TextInput,
    View,
} from 'react-native';

type OrderStatus = 'pending' | 'paid' | 'processing' | 'completed' | 'disputed' | 'refunded';

interface DivinationOrder {
  id: string;
  providerId: string;
  providerName: string;
  packageName: string;
  price: string;
  status: OrderStatus;
  createdAt: string;
  question?: string;
  answer?: string;
  birthInfo?: {
    year: number;
    month: number;
    day: number;
    hour: number;
  };
}

const STATUS_MAP: Record<OrderStatus, { label: string; color: string; bg: string }> = {
  pending: { label: '待支付', color: '#f59e0b', bg: '#fef3c7' },
  paid: { label: '已支付', color: '#3b82f6', bg: '#dbeafe' },
  processing: { label: '分析中', color: '#6D28D9', bg: '#f3e8ff' },
  completed: { label: '已完成', color: '#16a34a', bg: '#dcfce7' },
  disputed: { label: '申诉中', color: '#dc2626', bg: '#fee2e2' },
  refunded: { label: '已退款', color: '#6b7280', bg: '#f3f4f6' },
};

const MOCK_ORDER: DivinationOrder = {
  id: '1',
  providerId: 'p1',
  providerName: '玄明道长',
  packageName: '八字详批',
  price: '¥288',
  status: 'pending',
  createdAt: '2024-01-20 14:30',
  birthInfo: {
    year: 1990,
    month: 6,
    day: 15,
    hour: 10,
  },
};

export default function OrderDetailScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const router = useRouter();
  const { isLoggedIn } = useAuthStore();
  const { createDivinationOrder, isLoading, status } = useTransaction();
  
  const [order, setOrder] = useState<DivinationOrder>(MOCK_ORDER);
  const [question, setQuestion] = useState('');
  const [showPayModal, setShowPayModal] = useState(false);

  const statusInfo = STATUS_MAP[order.status];

  const handlePay = async () => {
    const result = await createDivinationOrder(
      order.providerId,
      1,
      question || '八字命理分析'
    );
    
    if (result?.success) {
      setOrder(prev => ({ ...prev, status: 'paid' }));
      setShowPayModal(false);
    }
  };

  const handleChat = () => {
    router.push(`/chat/${order.providerId}`);
  };

  const handleDispute = () => {
    if (Platform.OS === 'web') {
      if (window.confirm('确定要发起申诉吗？')) {
        setOrder(prev => ({ ...prev, status: 'disputed' }));
      }
    } else {
      Alert.alert('确认', '确定要发起申诉吗？', [
        { text: '取消', style: 'cancel' },
        { text: '确认', onPress: () => setOrder(prev => ({ ...prev, status: 'disputed' })) },
      ]);
    }
  };

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
          <Text style={styles.orderTime}>{order.createdAt}</Text>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>服务信息</Text>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>占卜师</Text>
            <Text style={styles.infoValue}>{order.providerName}</Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>服务套餐</Text>
            <Text style={styles.infoValue}>{order.packageName}</Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>订单金额</Text>
            <Text style={styles.priceValue}>{order.price}</Text>
          </View>
        </View>

        {order.birthInfo && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>出生信息</Text>
            <View style={styles.birthInfoBox}>
              <Text style={styles.birthInfoText}>
                {order.birthInfo.year}年{order.birthInfo.month}月{order.birthInfo.day}日 {order.birthInfo.hour}时
              </Text>
            </View>
          </View>
        )}

        {order.status === 'pending' && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>咨询问题（可选）</Text>
            <TextInput
              style={styles.questionInput}
              placeholder="请描述您想咨询的具体问题..."
              value={question}
              onChangeText={setQuestion}
              multiline
              numberOfLines={4}
              textAlignVertical="top"
            />
          </View>
        )}

        {order.question && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>咨询问题</Text>
            <Text style={styles.questionText}>{order.question}</Text>
          </View>
        )}

        {order.answer && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>占卜结果</Text>
            <View style={styles.answerBox}>
              <Text style={styles.answerText}>{order.answer}</Text>
            </View>
          </View>
        )}

        {order.status === 'processing' && (
          <View style={styles.processingBox}>
            <Text style={styles.processingIcon}>🔮</Text>
            <Text style={styles.processingText}>占卜师正在为您分析命盘...</Text>
            <Text style={styles.processingHint}>预计24-48小时内完成</Text>
          </View>
        )}

        <View style={styles.bottomSpacer} />
      </ScrollView>

      <View style={styles.bottomBar}>
        {order.status === 'pending' && (
          <Pressable 
            style={styles.payButton}
            onPress={() => setShowPayModal(true)}
          >
            <Text style={styles.payButtonText}>立即支付 {order.price}</Text>
          </Pressable>
        )}

        {(order.status === 'paid' || order.status === 'processing') && (
          <>
            <Pressable style={styles.chatButton} onPress={handleChat}>
              <Text style={styles.chatButtonText}>💬 联系占卜师</Text>
            </Pressable>
            <Pressable style={styles.disputeButton} onPress={handleDispute}>
              <Text style={styles.disputeButtonText}>申诉</Text>
            </Pressable>
          </>
        )}

        {order.status === 'completed' && (
          <Pressable style={styles.reviewButton}>
            <Text style={styles.reviewButtonText}>评价服务</Text>
          </Pressable>
        )}
      </View>

      <TransactionModal
        visible={showPayModal}
        onClose={() => setShowPayModal(false)}
        title="确认支付"
        description={`${order.packageName} - ${order.providerName}`}
        amount={order.price}
        status={status}
        isLoading={isLoading}
        onConfirm={handlePay}
        confirmText="确认支付"
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
  orderTime: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 4,
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
    fontWeight: '500',
  },
  priceValue: {
    fontSize: 16,
    color: '#6D28D9',
    fontWeight: '700',
  },
  birthInfoBox: {
    backgroundColor: '#f9fafb',
    padding: 12,
    borderRadius: 8,
  },
  birthInfoText: {
    fontSize: 15,
    color: '#4b5563',
    textAlign: 'center',
  },
  questionInput: {
    backgroundColor: '#f9fafb',
    borderRadius: 8,
    padding: 12,
    fontSize: 14,
    color: '#1f2937',
    minHeight: 100,
  },
  questionText: {
    fontSize: 14,
    color: '#4b5563',
    lineHeight: 22,
  },
  answerBox: {
    backgroundColor: '#faf5ff',
    padding: 16,
    borderRadius: 12,
    borderLeftWidth: 3,
    borderLeftColor: '#6D28D9',
  },
  answerText: {
    fontSize: 14,
    color: '#4b5563',
    lineHeight: 24,
  },
  processingBox: {
    backgroundColor: '#fff',
    padding: 32,
    alignItems: 'center',
    marginBottom: 12,
  },
  processingIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  processingText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#6D28D9',
  },
  processingHint: {
    fontSize: 13,
    color: '#9ca3af',
    marginTop: 8,
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
  payButton: {
    flex: 1,
    backgroundColor: '#6D28D9',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  payButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#fff',
  },
  chatButton: {
    flex: 2,
    backgroundColor: '#6D28D9',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  chatButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#fff',
  },
  disputeButton: {
    flex: 1,
    backgroundColor: '#f3f4f6',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  disputeButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#6b7280',
  },
  reviewButton: {
    flex: 1,
    backgroundColor: '#6D28D9',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  reviewButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#fff',
  },
});
