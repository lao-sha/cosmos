/**
 * 订单详情页面
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
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import {
  Order,
  OrderStatus,
  ORDER_STATUS_CONFIG,
  DIVINATION_TYPE_CONFIG,
  SERVICE_TYPE_CONFIG,
  DivinationType,
  ServiceType,
} from '@/features/diviner';

const THEME_COLOR = '#B2955D';

export default function OrderDetailPage() {
  const router = useRouter();
  const { id } = useLocalSearchParams<{ id: string }>();
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [order, setOrder] = useState<Order | null>(null);
  const [answerText, setAnswerText] = useState('');

  useEffect(() => {
    // TODO: 从链上加载订单详情
    setTimeout(() => {
      setOrder({
        id: Number(id),
        customer: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
        provider: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        packageId: 1,
        divinationType: DivinationType.Meihua,
        questionCid: 'QmXxx...',
        totalAmount: BigInt(10 * 1e10),
        platformFee: BigInt(1.5 * 1e10),
        providerEarnings: BigInt(8.5 * 1e10),
        isUrgent: false,
        status: OrderStatus.Accepted,
        createdAt: Date.now() - 3600000,
        acceptedAt: Date.now() - 1800000,
        followUpsUsed: 0,
        followUpsTotal: 3,
      });
      setLoading(false);
    }, 500);
  }, [id]);

  const handleAccept = async () => {
    setSubmitting(true);
    try {
      await new Promise(resolve => setTimeout(resolve, 1000));
      setOrder(prev => prev ? { ...prev, status: OrderStatus.Accepted, acceptedAt: Date.now() } : null);
      Alert.alert('成功', '已接单');
    } catch (error) {
      Alert.alert('失败', '操作失败，请重试');
    } finally {
      setSubmitting(false);
    }
  };

  const handleReject = () => {
    Alert.alert('确认拒绝', '拒绝后订单将取消并退款给客户', [
      { text: '取消', style: 'cancel' },
      {
        text: '确认拒绝',
        style: 'destructive',
        onPress: async () => {
          setSubmitting(true);
          try {
            await new Promise(resolve => setTimeout(resolve, 1000));
            setOrder(prev => prev ? { ...prev, status: OrderStatus.Cancelled } : null);
            Alert.alert('成功', '已拒绝订单');
          } catch (error) {
            Alert.alert('失败', '操作失败，请重试');
          } finally {
            setSubmitting(false);
          }
        },
      },
    ]);
  };

  const handleSubmitAnswer = async () => {
    if (!answerText.trim()) {
      Alert.alert('提示', '请输入解读内容');
      return;
    }

    setSubmitting(true);
    try {
      // TODO: 上传到 IPFS 并提交到链上
      await new Promise(resolve => setTimeout(resolve, 1500));
      setOrder(prev => prev ? {
        ...prev,
        status: OrderStatus.Completed,
        answerCid: 'QmNewAnswer...',
        completedAt: Date.now(),
      } : null);
      Alert.alert('成功', '解读已提交，订单完成');
    } catch (error) {
      Alert.alert('失败', '提交失败，请重试');
    } finally {
      setSubmitting(false);
    }
  };

  if (loading) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="订单详情" />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  if (!order) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="订单详情" />
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyText}>订单不存在</Text>
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  const statusConfig = ORDER_STATUS_CONFIG[order.status];
  const divType = DIVINATION_TYPE_CONFIG[order.divinationType];
  const priceDisplay = (Number(order.totalAmount) / 1e10).toFixed(2);
  const feeDisplay = (Number(order.platformFee) / 1e10).toFixed(2);
  const earningsDisplay = (Number(order.providerEarnings) / 1e10).toFixed(2);

  const formatTime = (timestamp?: number) => {
    if (!timestamp) return '-';
    const date = new Date(timestamp);
    return `${date.getFullYear()}/${date.getMonth() + 1}/${date.getDate()} ${date.getHours()}:${String(date.getMinutes()).padStart(2, '0')}`;
  };

  return (
    <View style={styles.wrapper}>
      <PageHeader title="订单详情" />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 状态卡片 */}
        <View style={[styles.statusCard, { backgroundColor: `${statusConfig.color}10` }]}>
          <Text style={[styles.statusText, { color: statusConfig.color }]}>{statusConfig.label}</Text>
          {order.isUrgent && <Text style={styles.urgentTag}>⚡ 加急订单</Text>}
        </View>

        {/* 订单信息 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>订单信息</Text>
          <View style={styles.infoCard}>
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>订单编号</Text>
              <Text style={styles.infoValue}>#{order.id}</Text>
            </View>
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>占卜类型</Text>
              <Text style={styles.infoValue}>{divType?.icon} {divType?.label}</Text>
            </View>
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>创建时间</Text>
              <Text style={styles.infoValue}>{formatTime(order.createdAt)}</Text>
            </View>
            {order.acceptedAt && (
              <View style={styles.infoRow}>
                <Text style={styles.infoLabel}>接单时间</Text>
                <Text style={styles.infoValue}>{formatTime(order.acceptedAt)}</Text>
              </View>
            )}
            {order.completedAt && (
              <View style={styles.infoRow}>
                <Text style={styles.infoLabel}>完成时间</Text>
                <Text style={styles.infoValue}>{formatTime(order.completedAt)}</Text>
              </View>
            )}
          </View>
        </View>

        {/* 费用明细 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>费用明细</Text>
          <View style={styles.infoCard}>
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>订单金额</Text>
              <Text style={styles.infoValue}>{priceDisplay} DUST</Text>
            </View>
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>平台手续费</Text>
              <Text style={styles.infoValue}>-{feeDisplay} DUST</Text>
            </View>
            <View style={[styles.infoRow, styles.earningsRow]}>
              <Text style={styles.earningsLabel}>您的收益</Text>
              <Text style={styles.earningsValue}>{earningsDisplay} DUST</Text>
            </View>
          </View>
        </View>

        {/* 客户问题 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>客户问题</Text>
          <View style={styles.questionCard}>
            <Text style={styles.questionText}>问题内容加载中... (CID: {order.questionCid})</Text>
          </View>
        </View>

        {/* 追问信息 */}
        {order.followUpsTotal > 0 && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>追问</Text>
            <View style={styles.infoCard}>
              <Text style={styles.followUpText}>
                已使用 {order.followUpsUsed}/{order.followUpsTotal} 次追问
              </Text>
            </View>
          </View>
        )}

        {/* 提交解读 */}
        {order.status === OrderStatus.Accepted && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>提交解读</Text>
            <View style={styles.answerCard}>
              <TextInput
                style={styles.answerInput}
                value={answerText}
                onChangeText={setAnswerText}
                placeholder="请输入您的解读内容..."
                placeholderTextColor="#999"
                multiline
                numberOfLines={8}
                textAlignVertical="top"
              />
              <Pressable
                style={[styles.submitBtn, !answerText.trim() && styles.submitBtnDisabled]}
                onPress={handleSubmitAnswer}
                disabled={!answerText.trim() || submitting}
              >
                {submitting ? (
                  <ActivityIndicator color="#FFF" />
                ) : (
                  <Text style={styles.submitBtnText}>提交解读</Text>
                )}
              </Pressable>
            </View>
          </View>
        )}

        {/* 已完成的解读 */}
        {order.answerCid && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>解读内容</Text>
            <View style={styles.answerCard}>
              <Text style={styles.answerText}>解读内容加载中... (CID: {order.answerCid})</Text>
            </View>
          </View>
        )}

        {/* 操作按钮 */}
        {order.status === OrderStatus.Paid && (
          <View style={styles.actionSection}>
            <Pressable style={styles.rejectBtn} onPress={handleReject} disabled={submitting}>
              <Text style={styles.rejectBtnText}>拒绝订单</Text>
            </Pressable>
            <Pressable style={styles.acceptBtn} onPress={handleAccept} disabled={submitting}>
              {submitting ? (
                <ActivityIndicator color="#FFF" />
              ) : (
                <Text style={styles.acceptBtnText}>接受订单</Text>
              )}
            </Pressable>
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
    paddingBottom: 100,
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyText: {
    fontSize: 16,
    color: '#999',
  },
  statusCard: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 16,
    margin: 16,
    borderRadius: 12,
    gap: 12,
  },
  statusText: {
    fontSize: 18,
    fontWeight: '600',
  },
  urgentTag: {
    fontSize: 14,
    color: '#FF9500',
  },
  section: {
    paddingHorizontal: 16,
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000',
    marginBottom: 12,
  },
  infoCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 8,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  infoLabel: {
    fontSize: 14,
    color: '#666',
  },
  infoValue: {
    fontSize: 14,
    color: '#333',
    fontWeight: '500',
  },
  earningsRow: {
    borderBottomWidth: 0,
    paddingTop: 12,
  },
  earningsLabel: {
    fontSize: 14,
    color: THEME_COLOR,
    fontWeight: '500',
  },
  earningsValue: {
    fontSize: 16,
    color: THEME_COLOR,
    fontWeight: '600',
  },
  questionCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
  },
  questionText: {
    fontSize: 14,
    color: '#333',
    lineHeight: 22,
  },
  followUpText: {
    fontSize: 14,
    color: '#666',
  },
  answerCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
  },
  answerInput: {
    height: 160,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 8,
    padding: 12,
    fontSize: 14,
    color: '#333',
    backgroundColor: '#FAFAFA',
    marginBottom: 12,
  },
  answerText: {
    fontSize: 14,
    color: '#333',
    lineHeight: 22,
  },
  submitBtn: {
    height: 44,
    backgroundColor: THEME_COLOR,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  submitBtnDisabled: {
    opacity: 0.5,
  },
  submitBtnText: {
    fontSize: 16,
    color: '#FFF',
    fontWeight: '600',
  },
  actionSection: {
    flexDirection: 'row',
    padding: 16,
    gap: 12,
  },
  rejectBtn: {
    flex: 1,
    height: 48,
    borderWidth: 1,
    borderColor: '#FF3B30',
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  rejectBtnText: {
    fontSize: 16,
    color: '#FF3B30',
  },
  acceptBtn: {
    flex: 1,
    height: 48,
    backgroundColor: THEME_COLOR,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  acceptBtnText: {
    fontSize: 16,
    color: '#FFF',
    fontWeight: '600',
  },
});
