import React, { useState, useMemo } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
  KeyboardAvoidingView,
  Platform,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { ArrowRight, Info } from 'lucide-react-native';
import * as Haptics from 'expo-haptics';
import { useColors } from '@/hooks/useColors';
import { useMaker, useCreateBuyOrder, formatCos, formatPrice, parseUsdt } from '@/hooks/useOtc';
import { useWalletStore } from '@/stores/wallet';
import { Button, Input, Card } from '@/components/ui';
import { Colors } from '@/constants/colors';

export default function BuyScreen() {
  const colors = useColors();
  const router = useRouter();
  const { makerId } = useLocalSearchParams<{ makerId: string }>();
  const { data: maker, isLoading: makerLoading } = useMaker(makerId || '');
  const { isConnected, mnemonic } = useWalletStore();
  const createOrder = useCreateBuyOrder();

  const [usdtInput, setUsdtInput] = useState('');
  const [selectedPayment, setSelectedPayment] = useState<string>('');

  const price = maker ? Number(formatPrice(maker.price)) : 0;
  const cosAmount = useMemo(() => {
    const usdt = parseFloat(usdtInput) || 0;
    if (!price) return 0;
    return usdt / price;
  }, [usdtInput, price]);

  const minUsdt = maker ? Number(formatCos(maker.minAmount)) * price : 0;
  const maxUsdt = maker ? Number(formatCos(maker.maxAmount)) * price : 0;

  const isValidAmount = useMemo(() => {
    const usdt = parseFloat(usdtInput) || 0;
    return usdt >= minUsdt && usdt <= maxUsdt;
  }, [usdtInput, minUsdt, maxUsdt]);

  const handleQuickAmount = (percent: number) => {
    const amount = (maxUsdt * percent).toFixed(2);
    setUsdtInput(amount);
  };

  const handleBuy = async () => {
    if (!isConnected || !mnemonic) {
      Alert.alert('提示', '请先解锁钱包');
      return;
    }
    if (!maker) return;
    if (!isValidAmount) {
      Alert.alert('提示', `金额需在 ${minUsdt.toFixed(2)} - ${maxUsdt.toFixed(2)} USDT 之间`);
      return;
    }
    if (!selectedPayment) {
      Alert.alert('提示', '请选择支付方式');
      return;
    }

    try {
      const usdtAmount = parseUsdt(usdtInput);
      const orderId = await createOrder.mutateAsync({
        makerId: maker.id,
        usdtAmount,
      });
      Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
      router.replace(`/otc/order/${orderId}`);
    } catch (error: any) {
      Alert.alert('创建订单失败', error.message);
    }
  };

  if (makerLoading || !maker) {
    return (
      <View style={[styles.container, { backgroundColor: colors.background }]}>
        <Text style={{ color: colors.textSecondary, textAlign: 'center', marginTop: 100 }}>
          加载中...
        </Text>
      </View>
    );
  }

  return (
    <KeyboardAvoidingView
      style={[styles.container, { backgroundColor: colors.background }]}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}
    >
      <ScrollView contentContainerStyle={styles.content}>
        {/* Maker Info */}
        <Card style={styles.makerCard}>
          <View style={styles.makerRow}>
            <View style={[styles.avatar, { backgroundColor: Colors.primary + '20' }]}>
              <Text style={[styles.avatarText, { color: Colors.primary }]}>
                {maker.name.charAt(0)}
              </Text>
            </View>
            <View style={styles.makerInfo}>
              <Text style={[styles.makerName, { color: colors.textPrimary }]}>
                {maker.name}
              </Text>
              <Text style={[styles.makerStats, { color: colors.textSecondary }]}>
                {maker.completedOrders} 单 • {maker.completionRate}% 完成率
              </Text>
            </View>
          </View>
          <View style={styles.priceRow}>
            <Text style={[styles.priceLabel, { color: colors.textSecondary }]}>
              单价
            </Text>
            <Text style={[styles.priceValue, { color: Colors.success }]}>
              {formatPrice(maker.price)} USDT/COS
            </Text>
          </View>
        </Card>

        {/* Amount Input */}
        <Card style={styles.amountCard}>
          <Text style={[styles.sectionTitle, { color: colors.textPrimary }]}>
            购买金额
          </Text>

          <View style={styles.inputRow}>
            <Input
              placeholder="输入金额"
              value={usdtInput}
              onChangeText={setUsdtInput}
              keyboardType="decimal-pad"
              containerStyle={styles.amountInput}
            />
            <Text style={[styles.inputUnit, { color: colors.textSecondary }]}>
              USDT
            </Text>
          </View>

          <View style={styles.quickAmounts}>
            {[0.25, 0.5, 0.75, 1].map((percent) => (
              <TouchableOpacity
                key={percent}
                style={[styles.quickButton, { borderColor: colors.border }]}
                onPress={() => handleQuickAmount(percent)}
              >
                <Text style={[styles.quickText, { color: colors.textPrimary }]}>
                  {percent * 100}%
                </Text>
              </TouchableOpacity>
            ))}
          </View>

          <View style={styles.limitInfo}>
            <Info size={14} color={colors.textTertiary} />
            <Text style={[styles.limitText, { color: colors.textTertiary }]}>
              限额: {minUsdt.toFixed(2)} - {maxUsdt.toFixed(2)} USDT
            </Text>
          </View>

          {/* Result */}
          <View style={[styles.resultBox, { backgroundColor: colors.surface }]}>
            <Text style={[styles.resultLabel, { color: colors.textSecondary }]}>
              预计获得
            </Text>
            <View style={styles.resultRow}>
              <Text style={[styles.resultValue, { color: colors.textPrimary }]}>
                ≈ {cosAmount.toFixed(4)}
              </Text>
              <Text style={[styles.resultUnit, { color: Colors.primary }]}>
                COS
              </Text>
            </View>
          </View>
        </Card>

        {/* Payment Method */}
        <Card style={styles.paymentCard}>
          <Text style={[styles.sectionTitle, { color: colors.textPrimary }]}>
            支付方式
          </Text>
          <View style={styles.paymentOptions}>
            {maker.paymentMethods.map((method) => {
              const isSelected = selectedPayment === method;
              const config = {
                bank: { label: '银行卡', color: '#3B82F6' },
                alipay: { label: '支付宝', color: '#1677FF' },
                wechat: { label: '微信', color: '#07C160' },
              }[method] || { label: method, color: colors.textSecondary };

              return (
                <TouchableOpacity
                  key={method}
                  style={[
                    styles.paymentOption,
                    {
                      borderColor: isSelected ? config.color : colors.border,
                      backgroundColor: isSelected ? config.color + '10' : 'transparent',
                    },
                  ]}
                  onPress={() => setSelectedPayment(method)}
                >
                  <View
                    style={[
                      styles.radio,
                      {
                        borderColor: isSelected ? config.color : colors.border,
                        backgroundColor: isSelected ? config.color : 'transparent',
                      },
                    ]}
                  >
                    {isSelected && <View style={styles.radioInner} />}
                  </View>
                  <Text
                    style={[
                      styles.paymentLabel,
                      { color: isSelected ? config.color : colors.textPrimary },
                    ]}
                  >
                    {config.label}
                  </Text>
                </TouchableOpacity>
              );
            })}
          </View>
        </Card>

        {/* Submit */}
        <Button
          title={`购买 ${cosAmount.toFixed(4)} COS`}
          onPress={handleBuy}
          loading={createOrder.isPending}
          disabled={!isValidAmount || !selectedPayment}
          style={styles.submitButton}
        />

        {/* Notice */}
        <View style={styles.notice}>
          <Text style={[styles.noticeText, { color: colors.textTertiary }]}>
            • 下单后请在15分钟内完成支付{'\n'}
            • 支付完成后点击"已支付"等待商家放币{'\n'}
            • 如有问题可发起争议申诉
          </Text>
        </View>
      </ScrollView>
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    padding: 16,
  },
  makerCard: {
    marginBottom: 16,
  },
  makerRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  avatar: {
    width: 44,
    height: 44,
    borderRadius: 22,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  avatarText: {
    fontSize: 18,
    fontWeight: '700',
  },
  makerInfo: {
    flex: 1,
  },
  makerName: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 2,
  },
  makerStats: {
    fontSize: 13,
  },
  priceRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: 'rgba(0,0,0,0.05)',
  },
  priceLabel: {
    fontSize: 14,
  },
  priceValue: {
    fontSize: 18,
    fontWeight: '700',
  },
  amountCard: {
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 12,
  },
  inputRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  amountInput: {
    flex: 1,
    marginBottom: 0,
    marginRight: 8,
  },
  inputUnit: {
    fontSize: 16,
    fontWeight: '500',
  },
  quickAmounts: {
    flexDirection: 'row',
    gap: 8,
    marginTop: 12,
  },
  quickButton: {
    flex: 1,
    paddingVertical: 8,
    borderRadius: 8,
    borderWidth: 1,
    alignItems: 'center',
  },
  quickText: {
    fontSize: 14,
    fontWeight: '500',
  },
  limitInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
    marginTop: 12,
  },
  limitText: {
    fontSize: 12,
  },
  resultBox: {
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  resultLabel: {
    fontSize: 14,
    marginBottom: 4,
  },
  resultRow: {
    flexDirection: 'row',
    alignItems: 'baseline',
  },
  resultValue: {
    fontSize: 28,
    fontWeight: '700',
  },
  resultUnit: {
    fontSize: 16,
    fontWeight: '600',
    marginLeft: 6,
  },
  paymentCard: {
    marginBottom: 24,
  },
  paymentOptions: {
    gap: 10,
  },
  paymentOption: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 14,
    borderRadius: 12,
    borderWidth: 1.5,
  },
  radio: {
    width: 20,
    height: 20,
    borderRadius: 10,
    borderWidth: 2,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  radioInner: {
    width: 10,
    height: 10,
    borderRadius: 5,
    backgroundColor: '#FFFFFF',
  },
  paymentLabel: {
    fontSize: 15,
    fontWeight: '500',
  },
  submitButton: {
    marginBottom: 16,
  },
  notice: {
    paddingHorizontal: 8,
  },
  noticeText: {
    fontSize: 12,
    lineHeight: 20,
  },
});
