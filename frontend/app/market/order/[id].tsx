// frontend/app/market/order/[id].tsx

import React, { useEffect, useState, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  RefreshControl,
  SafeAreaView,
  StatusBar,
  Alert,
  TextInput,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { useWalletStore } from '@/stores/wallet.store';
import { useOrders, useMarketApi } from '@/divination/market/hooks';
import {
  Avatar,
  TierBadge,
  PriceDisplay,
  DivinationTypeBadge,
  OrderStatusBadge,
  OrderTimeline,
  LoadingSpinner,
  EmptyState,
} from '@/divination/market/components';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { Order, Provider, FollowUp } from '@/divination/market/types';
import { truncateAddress, formatDateTime } from '@/divination/market/utils/market.utils';
import { getIpfsUrl } from '@/divination/market/services/ipfs.service';

export default function OrderDetailScreen() {
  const router = useRouter();
  const { id } = useLocalSearchParams<{ id: string }>();
  const { address } = useWalletStore();
  const { getOrder, loading } = useOrders();
  const { getProvider } = useMarketApi();

  const [order, setOrder] = useState<Order | null>(null);
  const [provider, setProvider] = useState<Provider | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [followUpQuestion, setFollowUpQuestion] = useState('');
  const [submittingFollowUp, setSubmittingFollowUp] = useState(false);

  const loadData = useCallback(async () => {
    if (!id) return;

    try {
      const orderData = await getOrder(parseInt(id, 10));
      setOrder(orderData);

      if (orderData?.provider) {
        const providerData = await getProvider(orderData.provider);
        setProvider(providerData);
      }
    } catch (err) {
      console.error('Load order error:', err);
    }
  }, [id, getOrder, getProvider]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  }, [loadData]);

  const isCustomer = order?.customer === address;
  const isProvider = order?.provider === address;
  const canFollowUp =
    order?.status === 'Completed' &&
    order.followUps &&
    order.followUps.length < (order as any).followUpCount;

  const handleSubmitFollowUp = async () => {
    if (!followUpQuestion.trim()) {
      Alert.alert('提示', '请输入追问内容');
      return;
    }

    setSubmittingFollowUp(true);
    try {
      // TODO: 提交追问
      console.log('Submit follow up:', followUpQuestion);
      Alert.alert('成功', '追问已提交');
      setFollowUpQuestion('');
      loadData();
    } catch (err) {
      Alert.alert('失败', '提交追问失败');
    } finally {
      setSubmittingFollowUp(false);
    }
  };

  const handleReview = () => {
    router.push(`/market/review/create?orderId=${id}`);
  };

  if (loading && !order) {
    return (
      <SafeAreaView style={styles.container}>
        <LoadingSpinner text="加载中..." fullScreen />
      </SafeAreaView>
    );
  }

  if (!order) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.header}>
          <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
            <Ionicons name="arrow-back" size={24} color={THEME.text} />
          </TouchableOpacity>
          <Text style={styles.headerTitle}>订单详情</Text>
          <View style={styles.backBtn} />
        </View>
        <EmptyState
          icon="document-outline"
          title="订单不存在"
          actionText="返回"
          onAction={() => router.back()}
        />
      </SafeAreaView>
    );
  }

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="dark-content" backgroundColor={THEME.card} />

      {/* 顶部导航 */}
      <View style={styles.header}>
        <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
          <Ionicons name="arrow-back" size={24} color={THEME.text} />
        </TouchableOpacity>
        <Text style={styles.headerTitle}>订单详情</Text>
        <View style={styles.backBtn} />
      </View>

      <ScrollView
        style={styles.content}
        refreshControl={
          <RefreshControl
            refreshing={refreshing}
            onRefresh={onRefresh}
            colors={[THEME.primary]}
            tintColor={THEME.primary}
          />
        }
      >
        {/* 订单状态卡片 */}
        <View style={[styles.statusCard, SHADOWS.medium]}>
          <View style={styles.statusHeader}>
            <OrderStatusBadge status={order.status} size="medium" />
            {order.isUrgent && (
              <View style={styles.urgentTag}>
                <Ionicons name="flash" size={12} color={THEME.warning} />
                <Text style={styles.urgentText}>加急</Text>
              </View>
            )}
          </View>
          <Text style={styles.orderId}>订单号: {order.id}</Text>
        </View>

        {/* 解卦师/客户信息 */}
        <View style={[styles.section, SHADOWS.small]}>
          <Text style={styles.sectionTitle}>
            {isCustomer ? '解卦师' : '客户'}
          </Text>
          <View style={styles.personRow}>
            {isCustomer && provider ? (
              <>
                <Avatar
                  uri={provider.avatarCid ? getIpfsUrl(provider.avatarCid) : undefined}
                  name={provider.name}
                  size={44}
                />
                <View style={styles.personInfo}>
                  <View style={styles.nameRow}>
                    <Text style={styles.personName}>{provider.name}</Text>
                    <TierBadge tier={provider.tier} size="small" />
                  </View>
                  <Text style={styles.personOrders}>
                    已完成 {provider.completedOrders} 单
                  </Text>
                </View>
              </>
            ) : (
              <>
                <Avatar name="客" size={44} />
                <View style={styles.personInfo}>
                  <Text style={styles.personName}>
                    {truncateAddress(order.customer)}
                  </Text>
                </View>
              </>
            )}
          </View>
        </View>

        {/* 套餐信息 */}
        <View style={[styles.section, SHADOWS.small]}>
          <Text style={styles.sectionTitle}>服务信息</Text>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>占卜类型</Text>
            <DivinationTypeBadge type={order.divinationType} />
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>订单金额</Text>
            <PriceDisplay amount={order.amount} size="small" />
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>创建时间</Text>
            <Text style={styles.infoValue}>{formatDateTime(order.createdAt)}</Text>
          </View>
        </View>

        {/* 问题描述 */}
        <View style={[styles.section, SHADOWS.small]}>
          <Text style={styles.sectionTitle}>问题描述</Text>
          <View style={styles.questionBox}>
            <Text style={styles.questionText}>
              {order.question || '问题内容加密中...'}
            </Text>
          </View>
        </View>

        {/* 解读结果 */}
        {order.answer && (
          <View style={[styles.section, SHADOWS.small]}>
            <Text style={styles.sectionTitle}>解读结果</Text>
            <View style={styles.answerBox}>
              <Text style={styles.answerText}>{order.answer}</Text>
            </View>
          </View>
        )}

        {/* 追问列表 */}
        {order.followUps && order.followUps.length > 0 && (
          <View style={[styles.section, SHADOWS.small]}>
            <Text style={styles.sectionTitle}>
              追问记录 ({order.followUps.length})
            </Text>
            {order.followUps.map((followUp, index) => (
              <View key={index} style={styles.followUpItem}>
                <View style={styles.followUpQuestion}>
                  <Ionicons name="chatbubble-outline" size={14} color={THEME.info} />
                  <Text style={styles.followUpQuestionText}>
                    {followUp.question || '问题内容加密中...'}
                  </Text>
                </View>
                {followUp.answer && (
                  <View style={styles.followUpAnswer}>
                    <Ionicons name="chatbubble" size={14} color={THEME.primary} />
                    <Text style={styles.followUpAnswerText}>{followUp.answer}</Text>
                  </View>
                )}
              </View>
            ))}
          </View>
        )}

        {/* 追问输入 */}
        {isCustomer && canFollowUp && (
          <View style={[styles.section, SHADOWS.small]}>
            <Text style={styles.sectionTitle}>追问</Text>
            <TextInput
              style={styles.followUpInput}
              placeholder="输入您的追问..."
              placeholderTextColor={THEME.textTertiary}
              value={followUpQuestion}
              onChangeText={setFollowUpQuestion}
              multiline
              numberOfLines={3}
              textAlignVertical="top"
            />
            <TouchableOpacity
              style={[styles.followUpBtn, submittingFollowUp && styles.btnDisabled]}
              onPress={handleSubmitFollowUp}
              disabled={submittingFollowUp}
            >
              <Text style={styles.followUpBtnText}>
                {submittingFollowUp ? '提交中...' : '提交追问'}
              </Text>
            </TouchableOpacity>
          </View>
        )}

        {/* 订单时间线 */}
        <View style={[styles.section, SHADOWS.small]}>
          <Text style={styles.sectionTitle}>订单进度</Text>
          <OrderTimeline order={order} />
        </View>

        <View style={styles.bottomSpace} />
      </ScrollView>

      {/* 底部操作栏 */}
      {isCustomer && order.status === 'Completed' && (
        <View style={styles.footer}>
          <TouchableOpacity style={styles.reviewBtn} onPress={handleReview}>
            <Ionicons name="star-outline" size={20} color={THEME.textInverse} />
            <Text style={styles.reviewBtnText}>评价订单</Text>
          </TouchableOpacity>
        </View>
      )}
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME.background,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: THEME.card,
    paddingHorizontal: 8,
    paddingVertical: 10,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.border,
  },
  backBtn: {
    padding: 8,
    width: 40,
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: THEME.text,
  },
  content: {
    flex: 1,
    padding: 16,
  },
  statusCard: {
    backgroundColor: THEME.card,
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  statusHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 10,
  },
  urgentTag: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: THEME.warning + '15',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
    gap: 2,
  },
  urgentText: {
    fontSize: 11,
    color: THEME.warning,
    fontWeight: '500',
  },
  orderId: {
    fontSize: 12,
    color: THEME.textTertiary,
    marginTop: 8,
  },
  section: {
    backgroundColor: THEME.card,
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: THEME.text,
    marginBottom: 12,
  },
  personRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  personInfo: {
    flex: 1,
    marginLeft: 12,
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  personName: {
    fontSize: 15,
    fontWeight: '500',
    color: THEME.text,
  },
  personOrders: {
    fontSize: 12,
    color: THEME.textSecondary,
    marginTop: 4,
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 8,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.borderLight,
  },
  infoLabel: {
    fontSize: 14,
    color: THEME.textSecondary,
  },
  infoValue: {
    fontSize: 14,
    color: THEME.text,
  },
  questionBox: {
    backgroundColor: THEME.background,
    borderRadius: 8,
    padding: 12,
  },
  questionText: {
    fontSize: 14,
    color: THEME.text,
    lineHeight: 20,
  },
  answerBox: {
    backgroundColor: THEME.primary + '10',
    borderRadius: 8,
    padding: 12,
    borderLeftWidth: 3,
    borderLeftColor: THEME.primary,
  },
  answerText: {
    fontSize: 14,
    color: THEME.text,
    lineHeight: 22,
  },
  followUpItem: {
    marginBottom: 12,
    paddingBottom: 12,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.borderLight,
  },
  followUpQuestion: {
    flexDirection: 'row',
    gap: 8,
    marginBottom: 8,
  },
  followUpQuestionText: {
    flex: 1,
    fontSize: 13,
    color: THEME.info,
    lineHeight: 18,
  },
  followUpAnswer: {
    flexDirection: 'row',
    gap: 8,
    marginLeft: 22,
  },
  followUpAnswerText: {
    flex: 1,
    fontSize: 13,
    color: THEME.text,
    lineHeight: 18,
  },
  followUpInput: {
    backgroundColor: THEME.background,
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
    fontSize: 14,
    color: THEME.text,
    height: 80,
    marginBottom: 12,
  },
  followUpBtn: {
    backgroundColor: THEME.primary,
    borderRadius: 8,
    paddingVertical: 10,
    alignItems: 'center',
  },
  btnDisabled: {
    opacity: 0.6,
  },
  followUpBtnText: {
    fontSize: 14,
    fontWeight: '500',
    color: THEME.textInverse,
  },
  bottomSpace: {
    height: 80,
  },
  footer: {
    backgroundColor: THEME.card,
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: THEME.border,
  },
  reviewBtn: {
    flexDirection: 'row',
    backgroundColor: THEME.primary,
    borderRadius: 10,
    paddingVertical: 12,
    justifyContent: 'center',
    alignItems: 'center',
    gap: 6,
  },
  reviewBtnText: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.textInverse,
  },
});
