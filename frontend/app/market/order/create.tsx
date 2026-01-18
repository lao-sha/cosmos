// frontend/app/market/order/create.tsx

import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TextInput,
  TouchableOpacity,
  SafeAreaView,
  StatusBar,
  Alert,
  Switch,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { useWalletStore } from '@/stores/wallet.store';
import { useMarketApi, useOrders, useChainTransaction } from '@/divination/market/hooks';
import {
  Avatar,
  TierBadge,
  PriceDisplay,
  DivinationTypeBadge,
  LoadingSpinner,
  TransactionStatus,
} from '@/divination/market/components';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { Provider, ServicePackage, Hexagram } from '@/divination/market/types';
import {
  getServiceTypeName,
  getServiceTypeIcon,
  getDivinationTypeRoute,
  validateQuestion,
} from '@/divination/market/utils/market.utils';
import { getIpfsUrl, uploadToIpfs } from '@/divination/market/services';
import { encryptData } from '@/divination/market/services/e2e-encryption.service';

export default function CreateOrderScreen() {
  const router = useRouter();
  const params = useLocalSearchParams<{
    providerId: string;
    packageId: string;
    divinationType: string;
    hexagramId?: string;
  }>();

  const { address, isLocked } = useWalletStore();
  const { getProvider, getProviderPackages } = useMarketApi();
  const { previewCreateOrder, loading: orderLoading } = useOrders();
  const { createOrder, txState, isProcessing, resetState } = useChainTransaction();
  const [showTxStatus, setShowTxStatus] = useState(false);

  const [provider, setProvider] = useState<Provider | null>(null);
  const [selectedPackage, setSelectedPackage] = useState<ServicePackage | null>(null);
  const [selectedHexagram, setSelectedHexagram] = useState<Hexagram | null>(null);
  const [question, setQuestion] = useState('');
  const [isUrgent, setIsUrgent] = useState(false);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);

  // 加载数据
  const loadData = useCallback(async () => {
    if (!params.providerId) return;

    try {
      setLoading(true);
      const [providerData, packagesData] = await Promise.all([
        getProvider(params.providerId),
        getProviderPackages(params.providerId),
      ]);

      setProvider(providerData);

      if (params.packageId) {
        const pkg = packagesData.find((p) => p.id === parseInt(params.packageId, 10));
        setSelectedPackage(pkg || null);
      }
    } catch (err) {
      console.error('Load data error:', err);
    } finally {
      setLoading(false);
    }
  }, [params.providerId, params.packageId, getProvider, getProviderPackages]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  // 如果有传入 hexagramId，解析卦象
  useEffect(() => {
    if (params.hexagramId) {
      // TODO: 从本地获取卦象数据
      setSelectedHexagram({
        id: parseInt(params.hexagramId, 10),
        type: parseInt(params.divinationType || '0', 10),
        name: '待获取',
        createdAt: Date.now(),
      });
    }
  }, [params.hexagramId, params.divinationType]);

  // 计算价格
  const calculatePrice = () => {
    if (!selectedPackage) return 0n;
    let price = selectedPackage.price;
    if (isUrgent && selectedPackage.urgentAvailable) {
      price = price + (price * BigInt(selectedPackage.urgentSurcharge)) / 10000n;
    }
    return price;
  };

  // 选择卦象
  const handleSelectHexagram = () => {
    if (!selectedPackage) {
      Alert.alert('提示', '请先选择套餐');
      return;
    }
    const route = getDivinationTypeRoute(selectedPackage.divinationType);
    router.push({
      pathname: `/divination/${route}`,
      params: {
        selectMode: 'true',
        returnTo: `/market/order/create?providerId=${params.providerId}&packageId=${params.packageId}&divinationType=${params.divinationType}`,
      },
    });
  };

  // 提交订单
  const handleSubmit = async () => {
    if (!address || isLocked) {
      Alert.alert('提示', '请先解锁钱包');
      return;
    }

    if (!selectedPackage) {
      Alert.alert('提示', '请选择服务套餐');
      return;
    }

    if (!selectedHexagram) {
      Alert.alert('提示', '请先进行占卜获取卦象');
      return;
    }

    const questionValidation = validateQuestion(question);
    if (!questionValidation.valid) {
      Alert.alert('提示', questionValidation.message || '问题描述无效');
      return;
    }

    setSubmitting(true);
    setShowTxStatus(true);

    try {
      // 加密问题内容并上传到 IPFS
      const encryptedQuestion = await encryptData(question, provider!.address);
      const questionCid = await uploadToIpfs(encryptedQuestion);

      // 准备卦象数据
      const hexagramData = JSON.stringify({
        id: selectedHexagram.id,
        type: selectedHexagram.type,
        name: selectedHexagram.name,
      });

      // 调用链上交易创建订单
      const result = await createOrder(
        {
          providerId: params.providerId,
          packageId: selectedPackage.id,
          questionCid,
          hexagramData,
        },
        {
          onSuccess: () => {
            setTimeout(() => {
              setShowTxStatus(false);
              resetState();
              router.replace('/market/order/list');
            }, 1500);
          },
          onError: (error) => {
            console.error('Create order error:', error);
          },
        }
      );

      if (!result) {
        setShowTxStatus(false);
      }
    } catch (err) {
      console.error('Create order error:', err);
      Alert.alert('失败', '创建订单失败，请稍后重试');
      setShowTxStatus(false);
    } finally {
      setSubmitting(false);
    }
  };

  const handleTxStatusClose = () => {
    setShowTxStatus(false);
    resetState();
  };

  if (loading) {
    return (
      <SafeAreaView style={styles.container}>
        <LoadingSpinner text="加载中..." fullScreen />
      </SafeAreaView>
    );
  }

  if (!provider || !selectedPackage) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.header}>
          <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
            <Ionicons name="arrow-back" size={24} color={THEME.text} />
          </TouchableOpacity>
          <Text style={styles.headerTitle}>创建订单</Text>
          <View style={styles.backBtn} />
        </View>
        <View style={styles.centerContent}>
          <Ionicons name="alert-circle-outline" size={64} color={THEME.border} />
          <Text style={styles.emptyText}>套餐信息不存在</Text>
        </View>
      </SafeAreaView>
    );
  }

  const totalPrice = calculatePrice();

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="dark-content" backgroundColor={THEME.card} />

      {/* 顶部导航 */}
      <View style={styles.header}>
        <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
          <Ionicons name="arrow-back" size={24} color={THEME.text} />
        </TouchableOpacity>
        <Text style={styles.headerTitle}>创建订单</Text>
        <View style={styles.backBtn} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 解卦师信息 */}
        <View style={[styles.section, SHADOWS.small]}>
          <View style={styles.providerRow}>
            <Avatar
              uri={provider.avatarCid ? getIpfsUrl(provider.avatarCid) : undefined}
              name={provider.name}
              size={48}
            />
            <View style={styles.providerInfo}>
              <View style={styles.nameRow}>
                <Text style={styles.providerName}>{provider.name}</Text>
                <TierBadge tier={provider.tier} size="small" />
              </View>
              <Text style={styles.providerOrders}>
                已完成 {provider.completedOrders} 单
              </Text>
            </View>
          </View>
        </View>

        {/* 套餐信息 */}
        <View style={[styles.section, SHADOWS.small]}>
          <Text style={styles.sectionTitle}>服务套餐</Text>
          <View style={styles.packageInfo}>
            <View style={styles.packageHeader}>
              <DivinationTypeBadge type={selectedPackage.divinationType} />
              <View style={styles.serviceType}>
                <Ionicons
                  name={getServiceTypeIcon(selectedPackage.serviceType) as any}
                  size={14}
                  color={THEME.textSecondary}
                />
                <Text style={styles.serviceTypeText}>
                  {getServiceTypeName(selectedPackage.serviceType)}
                </Text>
              </View>
            </View>
            <Text style={styles.packageName}>{selectedPackage.name}</Text>
            <Text style={styles.packageDesc}>{selectedPackage.description}</Text>
            <View style={styles.packageDetails}>
              <Text style={styles.detailText}>
                时长: {selectedPackage.duration}分钟
              </Text>
              <Text style={styles.detailText}>
                追问: {selectedPackage.followUpCount}次
              </Text>
            </View>
          </View>
        </View>

        {/* 卦象选择 */}
        <View style={[styles.section, SHADOWS.small]}>
          <Text style={styles.sectionTitle}>关联卦象</Text>
          {selectedHexagram ? (
            <View style={styles.hexagramInfo}>
              <Text style={styles.hexagramName}>{selectedHexagram.name}</Text>
              <TouchableOpacity onPress={handleSelectHexagram}>
                <Text style={styles.changeText}>重新选择</Text>
              </TouchableOpacity>
            </View>
          ) : (
            <TouchableOpacity style={styles.selectHexagram} onPress={handleSelectHexagram}>
              <Ionicons name="add-circle-outline" size={24} color={THEME.primary} />
              <Text style={styles.selectText}>点击进行占卜获取卦象</Text>
            </TouchableOpacity>
          )}
        </View>

        {/* 问题描述 */}
        <View style={[styles.section, SHADOWS.small]}>
          <Text style={styles.sectionTitle}>问题描述</Text>
          <TextInput
            style={styles.textArea}
            placeholder="请详细描述您想咨询的问题（10-500字）"
            placeholderTextColor={THEME.textTertiary}
            value={question}
            onChangeText={setQuestion}
            multiline
            numberOfLines={6}
            textAlignVertical="top"
            maxLength={500}
          />
          <Text style={styles.charCount}>{question.length}/500</Text>
        </View>

        {/* 加急选项 */}
        {selectedPackage.urgentAvailable && (
          <View style={[styles.section, SHADOWS.small]}>
            <View style={styles.urgentRow}>
              <View style={styles.urgentInfo}>
                <View style={styles.urgentHeader}>
                  <Ionicons name="flash" size={18} color={THEME.warning} />
                  <Text style={styles.urgentLabel}>加急服务</Text>
                </View>
                <Text style={styles.urgentDesc}>
                  加急订单将优先处理，额外加收 {selectedPackage.urgentSurcharge / 100}%
                </Text>
              </View>
              <Switch
                value={isUrgent}
                onValueChange={setIsUrgent}
                trackColor={{ false: THEME.border, true: THEME.warning + '60' }}
                thumbColor={isUrgent ? THEME.warning : THEME.textTertiary}
              />
            </View>
          </View>
        )}

        {/* 价格汇总 */}
        <View style={[styles.priceSection, SHADOWS.small]}>
          <View style={styles.priceRow}>
            <Text style={styles.priceLabel}>套餐价格</Text>
            <PriceDisplay amount={selectedPackage.price} size="small" />
          </View>
          {isUrgent && (
            <View style={styles.priceRow}>
              <Text style={styles.priceLabel}>加急费用</Text>
              <PriceDisplay
                amount={totalPrice - selectedPackage.price}
                size="small"
                color={THEME.warning}
              />
            </View>
          )}
          <View style={[styles.priceRow, styles.totalRow]}>
            <Text style={styles.totalLabel}>应付总额</Text>
            <PriceDisplay amount={totalPrice} size="large" />
          </View>
        </View>

        {/* 提示 */}
        <View style={styles.notice}>
          <Ionicons name="shield-checkmark-outline" size={16} color={THEME.success} />
          <Text style={styles.noticeText}>
            您的问题将被加密存储，仅解卦师可见
          </Text>
        </View>

        <View style={styles.bottomSpace} />
      </ScrollView>

      {/* 底部提交按钮 */}
      <View style={styles.footer}>
        <View style={styles.footerPrice}>
          <Text style={styles.footerLabel}>应付</Text>
          <PriceDisplay amount={totalPrice} size="large" />
        </View>
        <TouchableOpacity
          style={[styles.submitBtn, submitting && styles.submitBtnDisabled]}
          onPress={handleSubmit}
          disabled={submitting}
        >
          {submitting ? (
            <LoadingSpinner size="small" color={THEME.textInverse} />
          ) : (
            <Text style={styles.submitBtnText}>确认下单</Text>
          )}
        </TouchableOpacity>
      </View>

      {/* 交易状态弹窗 */}
      <TransactionStatus
        visible={showTxStatus}
        state={txState}
        onClose={handleTxStatusClose}
        successMessage="订单创建成功！"
      />
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
  centerContent: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    gap: 16,
  },
  emptyText: {
    fontSize: 15,
    color: THEME.textSecondary,
  },
  content: {
    flex: 1,
    padding: 16,
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
  providerRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  providerInfo: {
    flex: 1,
    marginLeft: 12,
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  providerName: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.text,
  },
  providerOrders: {
    fontSize: 12,
    color: THEME.textSecondary,
    marginTop: 4,
  },
  packageInfo: {},
  packageHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 10,
    marginBottom: 10,
  },
  serviceType: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  serviceTypeText: {
    fontSize: 12,
    color: THEME.textSecondary,
  },
  packageName: {
    fontSize: 15,
    fontWeight: '500',
    color: THEME.text,
    marginBottom: 6,
  },
  packageDesc: {
    fontSize: 13,
    color: THEME.textSecondary,
    lineHeight: 18,
    marginBottom: 10,
  },
  packageDetails: {
    flexDirection: 'row',
    gap: 20,
  },
  detailText: {
    fontSize: 12,
    color: THEME.textTertiary,
  },
  hexagramInfo: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    backgroundColor: THEME.background,
    borderRadius: 8,
    padding: 12,
  },
  hexagramName: {
    fontSize: 15,
    color: THEME.text,
  },
  changeText: {
    fontSize: 13,
    color: THEME.primary,
  },
  selectHexagram: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: THEME.background,
    borderRadius: 8,
    padding: 24,
    borderWidth: 1,
    borderColor: THEME.primary,
    borderStyle: 'dashed',
    gap: 8,
  },
  selectText: {
    fontSize: 14,
    color: THEME.primary,
  },
  textArea: {
    backgroundColor: THEME.background,
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
    fontSize: 15,
    color: THEME.text,
    height: 120,
  },
  charCount: {
    fontSize: 11,
    color: THEME.textTertiary,
    textAlign: 'right',
    marginTop: 4,
  },
  urgentRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  urgentInfo: {
    flex: 1,
    marginRight: 16,
  },
  urgentHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    marginBottom: 4,
  },
  urgentLabel: {
    fontSize: 15,
    fontWeight: '500',
    color: THEME.text,
  },
  urgentDesc: {
    fontSize: 12,
    color: THEME.textTertiary,
  },
  priceSection: {
    backgroundColor: THEME.card,
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  priceRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 10,
  },
  priceLabel: {
    fontSize: 14,
    color: THEME.textSecondary,
  },
  totalRow: {
    marginTop: 6,
    paddingTop: 12,
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: THEME.border,
    marginBottom: 0,
  },
  totalLabel: {
    fontSize: 15,
    fontWeight: '600',
    color: THEME.text,
  },
  notice: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    marginBottom: 16,
  },
  noticeText: {
    fontSize: 12,
    color: THEME.success,
  },
  bottomSpace: {
    height: 100,
  },
  footer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: THEME.card,
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: THEME.border,
  },
  footerPrice: {
    flex: 1,
  },
  footerLabel: {
    fontSize: 12,
    color: THEME.textTertiary,
  },
  submitBtn: {
    backgroundColor: THEME.primary,
    borderRadius: 10,
    paddingVertical: 12,
    paddingHorizontal: 32,
  },
  submitBtnDisabled: {
    opacity: 0.6,
  },
  submitBtnText: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.textInverse,
  },
});
