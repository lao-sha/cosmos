import { useActiveMakers, useFirstPurchaseStatus } from '@/src/hooks/useOtc';
import { useCosPrice } from '@/src/hooks/usePricing';
import { useTransaction } from '@/src/hooks/useTransaction';
import { otcService, MakerInfo } from '@/src/services/otc';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useEffect, useState } from 'react';
import {
  ActivityIndicator,
  Platform,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View
} from 'react-native';
import { blake2AsHex } from '@polkadot/util-crypto';

export default function OtcCreateScreen() {
  const router = useRouter();
  const params = useLocalSearchParams<{ makerId?: string }>();
  const { isLoggedIn, mnemonic, address } = useAuthStore();
  const { isConnected } = useChainStore();
  const { makers } = useActiveMakers();
  const { canFirstPurchase, refresh: refreshFirstPurchase } = useFirstPurchaseStatus();
  const { price, priceFormatted } = useCosPrice();
  const { createOtcOrderNew, createFirstPurchase, isTxLoading } = useTransaction();

  const [selectedMaker, setSelectedMaker] = useState<MakerInfo | null>(null);
  const [orderType, setOrderType] = useState<'normal' | 'first'>('normal');
  const [cosAmount, setCosAmount] = useState('');
  const [contact, setContact] = useState('');

  useEffect(() => {
    if (params.makerId && makers.length > 0) {
      const maker = makers.find(m => m.makerId === parseInt(params.makerId!));
      if (maker) {
        setSelectedMaker(maker);
      }
    }
  }, [params.makerId, makers]);

  useEffect(() => {
    if (canFirstPurchase) {
      setOrderType('first');
    }
  }, [canFirstPurchase]);

  const showAlert = (title: string, message: string) => {
    if (Platform.OS === 'web') {
      window.alert(`${title}\n${message}`);
    } else {
      const { Alert } = require('react-native');
      Alert.alert(title, message);
    }
  };

  const calculateUsdt = () => {
    if (!cosAmount || !price) return '0.00';
    const cos = parseFloat(cosAmount);
    const priceNum = Number(price) / 1e6;
    return (cos * priceNum).toFixed(2);
  };

  const handleCreateOrder = async () => {
    if (!mnemonic || !selectedMaker) {
      showAlert('错误', '请先登录并选择做市商');
      return;
    }

    if (!contact.trim()) {
      showAlert('错误', '请输入联系方式');
      return;
    }

    const paymentCommit = blake2AsHex(`payment:${address}:${Date.now()}`);
    const contactCommit = blake2AsHex(`contact:${contact}:${Date.now()}`);

    if (orderType === 'first') {
      await createFirstPurchase(
        mnemonic,
        selectedMaker.makerId,
        paymentCommit,
        contactCommit,
        {
          onSuccess: () => {
            showAlert('成功', '首购订单已创建，请尽快完成付款');
            refreshFirstPurchase();
            router.replace('/otc/orders');
          },
          onError: (error) => {
            showAlert('失败', error);
          },
        }
      );
    } else {
      if (!cosAmount || parseFloat(cosAmount) <= 0) {
        showAlert('错误', '请输入有效的 COS 数量');
        return;
      }

      // 链上精度：1 COS = 1e12 最小单位
      const cosAmountWei = BigInt(Math.floor(parseFloat(cosAmount) * 1e12)).toString();

      await createOtcOrderNew(
        mnemonic,
        selectedMaker.makerId,
        cosAmountWei,
        paymentCommit,
        contactCommit,
        {
          onSuccess: () => {
            showAlert('成功', '订单已创建，请尽快完成付款');
            router.replace('/otc/orders');
          },
          onError: (error) => {
            showAlert('失败', error);
          },
        }
      );
    }
  };

  if (!isLoggedIn || !isConnected) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>创建订单</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>🔌</Text>
          <Text style={styles.emptyTitle}>请先登录并连接网络</Text>
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
        <Text style={styles.headerTitle}>创建订单</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content}>
        {selectedMaker && (
          <View style={styles.makerCard}>
            <Text style={styles.cardTitle}>做市商信息</Text>
            <View style={styles.makerInfo}>
              <Text style={styles.makerName}>
                {selectedMaker.maskedFullName || `做市商 #${selectedMaker.makerId}`}
              </Text>
              <Text style={styles.makerDetail}>
                微信: {selectedMaker.wechatId || '-'}
              </Text>
              <Text style={styles.makerDetail}>
                TRON: {selectedMaker.tronAddress}
              </Text>
            </View>
          </View>
        )}

        {canFirstPurchase && (
          <View style={styles.orderTypeCard}>
            <Text style={styles.cardTitle}>订单类型</Text>
            <View style={styles.typeButtons}>
              <Pressable
                style={[
                  styles.typeButton,
                  orderType === 'first' && styles.typeButtonActive,
                ]}
                onPress={() => setOrderType('first')}
              >
                <Text style={[
                  styles.typeButtonText,
                  orderType === 'first' && styles.typeButtonTextActive,
                ]}>
                  🎁 首购订单
                </Text>
                <Text style={styles.typeButtonDesc}>固定 $10 USD</Text>
              </Pressable>
              <Pressable
                style={[
                  styles.typeButton,
                  orderType === 'normal' && styles.typeButtonActive,
                ]}
                onPress={() => setOrderType('normal')}
              >
                <Text style={[
                  styles.typeButtonText,
                  orderType === 'normal' && styles.typeButtonTextActive,
                ]}>
                  📦 普通订单
                </Text>
                <Text style={styles.typeButtonDesc}>自定义金额</Text>
              </Pressable>
            </View>
          </View>
        )}

        <View style={styles.formCard}>
          <Text style={styles.cardTitle}>订单信息</Text>

          {orderType === 'normal' ? (
            <>
              <View style={styles.formGroup}>
                <Text style={styles.label}>购买数量 (COS)</Text>
                <TextInput
                  style={styles.input}
                  value={cosAmount}
                  onChangeText={setCosAmount}
                  placeholder="输入 COS 数量"
                  placeholderTextColor="#9ca3af"
                  keyboardType="decimal-pad"
                />
              </View>

              <View style={styles.priceInfo}>
                <View style={styles.priceRow}>
                  <Text style={styles.priceLabel}>当前价格</Text>
                  <Text style={styles.priceValue}>{priceFormatted}</Text>
                </View>
                <View style={styles.priceRow}>
                  <Text style={styles.priceLabel}>预计支付</Text>
                  <Text style={styles.priceValueLarge}>${calculateUsdt()} USDT</Text>
                </View>
              </View>
            </>
          ) : (
            <View style={styles.firstPurchaseInfo}>
              <Text style={styles.firstPurchaseTitle}>首购订单详情</Text>
              <View style={styles.priceRow}>
                <Text style={styles.priceLabel}>固定价值</Text>
                <Text style={styles.priceValueLarge}>$10.00 USDT</Text>
              </View>
              <View style={styles.priceRow}>
                <Text style={styles.priceLabel}>当前价格</Text>
                <Text style={styles.priceValue}>{priceFormatted}</Text>
              </View>
              <Text style={styles.firstPurchaseNote}>
                * COS 数量将根据当前价格自动计算
              </Text>
            </View>
          )}

          <View style={styles.formGroup}>
            <Text style={styles.label}>联系方式 *</Text>
            <TextInput
              style={styles.input}
              value={contact}
              onChangeText={setContact}
              placeholder="微信号或手机号"
              placeholderTextColor="#9ca3af"
            />
            <Text style={styles.hint}>用于做市商联系您确认付款</Text>
          </View>
        </View>

        <View style={styles.noticeCard}>
          <Text style={styles.noticeTitle}>⚠️ 交易须知</Text>
          <Text style={styles.noticeText}>
            1. 订单创建后请在 1 小时内完成付款{'\n'}
            2. 付款后请及时标记"已付款"{'\n'}
            3. 做市商确认收款后将释放 COS{'\n'}
            4. 如有争议可发起仲裁
          </Text>
        </View>

        <Pressable
          style={[styles.submitButton, isTxLoading && styles.buttonDisabled]}
          onPress={handleCreateOrder}
          disabled={isTxLoading || !selectedMaker}
        >
          {isTxLoading ? (
            <ActivityIndicator color="#fff" />
          ) : (
            <Text style={styles.submitButtonText}>
              {orderType === 'first' ? '创建首购订单' : '创建订单'}
            </Text>
          )}
        </Pressable>

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
  makerCard: {
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
  makerInfo: {
    backgroundColor: '#f9fafb',
    padding: 12,
    borderRadius: 8,
  },
  makerName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 8,
  },
  makerDetail: {
    fontSize: 13,
    color: '#6b7280',
    marginBottom: 4,
  },
  orderTypeCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  typeButtons: {
    flexDirection: 'row',
    gap: 12,
  },
  typeButton: {
    flex: 1,
    padding: 16,
    borderRadius: 10,
    borderWidth: 2,
    borderColor: '#e5e7eb',
    alignItems: 'center',
  },
  typeButtonActive: {
    borderColor: '#6D28D9',
    backgroundColor: '#f5f3ff',
  },
  typeButtonText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#6b7280',
    marginBottom: 4,
  },
  typeButtonTextActive: {
    color: '#6D28D9',
  },
  typeButtonDesc: {
    fontSize: 12,
    color: '#9ca3af',
  },
  formCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  formGroup: {
    marginBottom: 16,
  },
  label: {
    fontSize: 14,
    fontWeight: '500',
    color: '#374151',
    marginBottom: 6,
  },
  input: {
    backgroundColor: '#f9fafb',
    borderWidth: 1,
    borderColor: '#e5e7eb',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 12,
    fontSize: 16,
    color: '#1f2937',
  },
  hint: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 4,
  },
  priceInfo: {
    backgroundColor: '#f0fdf4',
    padding: 12,
    borderRadius: 8,
    marginBottom: 16,
  },
  priceRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 6,
  },
  priceLabel: {
    fontSize: 14,
    color: '#6b7280',
  },
  priceValue: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1f2937',
  },
  priceValueLarge: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#16a34a',
  },
  firstPurchaseInfo: {
    backgroundColor: '#fef3c7',
    padding: 12,
    borderRadius: 8,
    marginBottom: 16,
  },
  firstPurchaseTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#92400e',
    marginBottom: 8,
  },
  firstPurchaseNote: {
    fontSize: 12,
    color: '#92400e',
    marginTop: 8,
    fontStyle: 'italic',
  },
  noticeCard: {
    backgroundColor: '#eff6ff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  noticeTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1e40af',
    marginBottom: 8,
  },
  noticeText: {
    fontSize: 13,
    color: '#1e40af',
    lineHeight: 20,
  },
  submitButton: {
    backgroundColor: '#6D28D9',
    marginHorizontal: 16,
    marginTop: 24,
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  submitButtonText: {
    color: '#fff',
    fontSize: 18,
    fontWeight: '600',
  },
  buttonDisabled: {
    opacity: 0.6,
  },
  bottomPadding: {
    height: 40,
  },
});
