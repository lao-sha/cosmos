import { useSwapMakers } from '@/src/hooks/useSwap';
import { useCosPrice } from '@/src/hooks/usePricing';
import { useTransaction } from '@/src/hooks/useTransaction';
import { swapService, MakerInfo } from '@/src/services/swap';
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

export default function SwapCreateScreen() {
  const router = useRouter();
  const params = useLocalSearchParams<{ makerId?: string }>();
  const { isLoggedIn, mnemonic, address } = useAuthStore();
  const { isConnected } = useChainStore();
  const { makers } = useSwapMakers();
  const { price, priceFormatted } = useCosPrice();
  const { createSwap, isTxLoading } = useTransaction();

  const [selectedMaker, setSelectedMaker] = useState<MakerInfo | null>(null);
  const [cosAmount, setCosAmount] = useState('');
  const [usdtAddress, setUsdtAddress] = useState('');

  useEffect(() => {
    if (params.makerId && makers.length > 0) {
      const maker = makers.find(m => m.makerId === parseInt(params.makerId!));
      if (maker) {
        setSelectedMaker(maker);
      }
    }
  }, [params.makerId, makers]);

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
    const usdt = cos * priceNum;
    return usdt.toFixed(2);
  };

  const validateTronAddress = (addr: string): boolean => {
    return addr.startsWith('T') && addr.length >= 34 && addr.length <= 42;
  };

  const handleCreateSwap = async () => {
    if (!mnemonic || !selectedMaker) {
      showAlert('错误', '请先登录并选择做市商');
      return;
    }

    if (!cosAmount || parseFloat(cosAmount) < 100) {
      showAlert('错误', '最小兑换金额为 100 COS');
      return;
    }

    if (!usdtAddress.trim()) {
      showAlert('错误', '请输入 USDT 收款地址');
      return;
    }

    if (!validateTronAddress(usdtAddress.trim())) {
      showAlert('错误', '请输入有效的 TRC20 地址（以 T 开头）');
      return;
    }

    // 链上精度：1 COS = 1e12 最小单位
    const cosAmountWei = BigInt(Math.floor(parseFloat(cosAmount) * 1e12)).toString();

    await createSwap(
      mnemonic,
      selectedMaker.makerId,
      cosAmountWei,
      usdtAddress.trim(),
      {
        onSuccess: () => {
          showAlert('成功', '兑换请求已创建，请等待做市商转账 USDT');
          router.replace('/swap/history');
        },
        onError: (error) => {
          showAlert('失败', error);
        },
      }
    );
  };

  if (!isLoggedIn || !isConnected) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>创建兑换</Text>
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
        <Text style={styles.headerTitle}>创建兑换</Text>
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
                卖出溢价: {(selectedMaker.sellPremiumBps / 100).toFixed(2)}%
              </Text>
              <Text style={styles.makerDetail}>
                微信: {selectedMaker.wechatId || '-'}
              </Text>
            </View>
          </View>
        )}

        <View style={styles.formCard}>
          <Text style={styles.cardTitle}>兑换信息</Text>

          <View style={styles.formGroup}>
            <Text style={styles.label}>兑换数量 (COS) *</Text>
            <TextInput
              style={styles.input}
              value={cosAmount}
              onChangeText={setCosAmount}
              placeholder="最小 100 COS"
              placeholderTextColor="#9ca3af"
              keyboardType="decimal-pad"
            />
            <Text style={styles.hint}>最小兑换金额：100 COS</Text>
          </View>

          <View style={styles.priceInfo}>
            <View style={styles.priceRow}>
              <Text style={styles.priceLabel}>当前价格</Text>
              <Text style={styles.priceValue}>{priceFormatted}</Text>
            </View>
            <View style={styles.priceRow}>
              <Text style={styles.priceLabel}>预计收到</Text>
              <Text style={styles.priceValueLarge}>${calculateUsdt()} USDT</Text>
            </View>
          </View>

          <View style={styles.formGroup}>
            <Text style={styles.label}>USDT 收款地址 (TRC20) *</Text>
            <TextInput
              style={styles.input}
              value={usdtAddress}
              onChangeText={setUsdtAddress}
              placeholder="T 开头的 TRC20 地址"
              placeholderTextColor="#9ca3af"
              autoCapitalize="none"
            />
            <Text style={styles.hint}>请确保地址正确，转账后无法撤回</Text>
          </View>
        </View>

        <View style={styles.noticeCard}>
          <Text style={styles.noticeTitle}>⚠️ 兑换须知</Text>
          <Text style={styles.noticeText}>
            1. 创建兑换后，您的 COS 将被锁定{'\n'}
            2. 做市商需在 24 小时内向您转账 USDT{'\n'}
            3. 系统将自动验证 TRC20 交易{'\n'}
            4. 验证成功后，COS 将释放给做市商{'\n'}
            5. 如做市商超时未转账，COS 将自动退还
          </Text>
        </View>

        <Pressable
          style={[styles.submitButton, isTxLoading && styles.buttonDisabled]}
          onPress={handleCreateSwap}
          disabled={isTxLoading || !selectedMaker}
        >
          {isTxLoading ? (
            <ActivityIndicator color="#fff" />
          ) : (
            <Text style={styles.submitButtonText}>确认兑换</Text>
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
    backgroundColor: '#ecfdf5',
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
    color: '#10b981',
  },
  noticeCard: {
    backgroundColor: '#fef3c7',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  noticeTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#92400e',
    marginBottom: 8,
  },
  noticeText: {
    fontSize: 13,
    color: '#92400e',
    lineHeight: 20,
  },
  submitButton: {
    backgroundColor: '#10b981',
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
