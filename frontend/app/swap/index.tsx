import React, { useState, useMemo, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { ArrowDownUp, ChevronDown, Clock, Info } from 'lucide-react-native';
import * as Haptics from 'expo-haptics';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import { useBalance, formatBalance } from '@/hooks/useBalance';
import { getSwapQuotes, executeSwap, type SwapDirection, type SwapQuote } from '@/services/swap';
import { Button, Input, Card } from '@/components/ui';
import { Colors, Gradients } from '@/constants/colors';

export default function SwapScreen() {
  const colors = useColors();
  const router = useRouter();
  const { address, mnemonic, isConnected } = useWalletStore();
  const { data: balance } = useBalance();

  const [direction, setDirection] = useState<SwapDirection>('usdt_to_cos');
  const [inputAmount, setInputAmount] = useState('');
  const [quotes, setQuotes] = useState<SwapQuote[]>([]);
  const [selectedQuote, setSelectedQuote] = useState<SwapQuote | null>(null);
  const [loading, setLoading] = useState(false);
  const [swapping, setSwapping] = useState(false);

  const inputToken = direction === 'usdt_to_cos' ? 'USDT' : 'COS';
  const outputToken = direction === 'usdt_to_cos' ? 'COS' : 'USDT';

  useEffect(() => {
    const fetchQuotes = async () => {
      if (!inputAmount || parseFloat(inputAmount) <= 0) {
        setQuotes([]);
        setSelectedQuote(null);
        return;
      }

      setLoading(true);
      try {
        const decimals = direction === 'usdt_to_cos' ? 6 : 12;
        const amount = parseAmount(inputAmount, decimals);
        const result = await getSwapQuotes(direction, amount);
        setQuotes(result);
        if (result.length > 0) {
          setSelectedQuote(result[0]);
        }
      } catch (error) {
        console.error('Failed to get quotes:', error);
      } finally {
        setLoading(false);
      }
    };

    const timer = setTimeout(fetchQuotes, 500);
    return () => clearTimeout(timer);
  }, [inputAmount, direction]);

  const handleSwitch = () => {
    setDirection(d => d === 'usdt_to_cos' ? 'cos_to_usdt' : 'usdt_to_cos');
    setInputAmount('');
    setQuotes([]);
    setSelectedQuote(null);
    Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light);
  };

  const handleSwap = async () => {
    if (!isConnected || !mnemonic) {
      Alert.alert('提示', '请先解锁钱包');
      return;
    }
    if (!selectedQuote) {
      Alert.alert('提示', '请输入兑换金额');
      return;
    }

    Alert.alert(
      '确认兑换',
      `支付 ${formatAmount(selectedQuote.inputAmount, direction === 'usdt_to_cos' ? 6 : 12)} ${inputToken}\n获得 ≈${formatAmount(selectedQuote.outputAmount, direction === 'usdt_to_cos' ? 12 : 6)} ${outputToken}`,
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确认',
          onPress: async () => {
            setSwapping(true);
            try {
              await executeSwap(
                selectedQuote.makerId,
                direction,
                selectedQuote.inputAmount,
                mnemonic
              );
              Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
              Alert.alert('兑换成功', '资产已到账');
              setInputAmount('');
              setQuotes([]);
              setSelectedQuote(null);
            } catch (error: any) {
              Alert.alert('兑换失败', error.message);
            } finally {
              setSwapping(false);
            }
          },
        },
      ]
    );
  };

  return (
    <ScrollView
      style={[styles.container, { backgroundColor: colors.background }]}
      contentContainerStyle={styles.content}
    >
      <View style={styles.header}>
        <Text style={[styles.title, { color: colors.textPrimary }]}>兑换</Text>
        <TouchableOpacity
          style={styles.historyButton}
          onPress={() => router.push('/swap/history')}
        >
          <Clock size={20} color={Colors.primary} />
        </TouchableOpacity>
      </View>

      {/* Swap Card */}
      <Card style={styles.swapCard}>
        {/* Input */}
        <View style={styles.tokenSection}>
          <Text style={[styles.sectionLabel, { color: colors.textSecondary }]}>
            支付
          </Text>
          <View style={styles.inputRow}>
            <Input
              placeholder="0.00"
              value={inputAmount}
              onChangeText={setInputAmount}
              keyboardType="decimal-pad"
              containerStyle={styles.amountInput}
            />
            <View style={[styles.tokenBadge, { backgroundColor: colors.surface }]}>
              <Text style={[styles.tokenText, { color: colors.textPrimary }]}>
                {inputToken}
              </Text>
            </View>
          </View>
          <Text style={[styles.balanceText, { color: colors.textTertiary }]}>
            余额: {balance ? formatBalance(balance) : '0'} COS
          </Text>
        </View>

        {/* Switch Button */}
        <TouchableOpacity
          style={[styles.switchButton, { backgroundColor: Colors.primary }]}
          onPress={handleSwitch}
        >
          <ArrowDownUp size={20} color="#FFFFFF" />
        </TouchableOpacity>

        {/* Output */}
        <View style={styles.tokenSection}>
          <Text style={[styles.sectionLabel, { color: colors.textSecondary }]}>
            获得
          </Text>
          <View style={styles.outputRow}>
            <Text style={[styles.outputAmount, { color: colors.textPrimary }]}>
              {selectedQuote
                ? formatAmount(selectedQuote.outputAmount, direction === 'usdt_to_cos' ? 12 : 6)
                : '0.00'}
            </Text>
            <View style={[styles.tokenBadge, { backgroundColor: colors.surface }]}>
              <Text style={[styles.tokenText, { color: colors.textPrimary }]}>
                {outputToken}
              </Text>
            </View>
          </View>
        </View>
      </Card>

      {/* Quote Selection */}
      {quotes.length > 0 && (
        <Card style={styles.quotesCard}>
          <Text style={[styles.quotesTitle, { color: colors.textPrimary }]}>
            选择商家
          </Text>
          {quotes.map((quote) => (
            <TouchableOpacity
              key={quote.makerId}
              style={[
                styles.quoteItem,
                {
                  borderColor:
                    selectedQuote?.makerId === quote.makerId
                      ? Colors.primary
                      : colors.border,
                  backgroundColor:
                    selectedQuote?.makerId === quote.makerId
                      ? Colors.primary + '10'
                      : 'transparent',
                },
              ]}
              onPress={() => setSelectedQuote(quote)}
            >
              <View style={styles.quoteInfo}>
                <Text style={[styles.quoteMaker, { color: colors.textPrimary }]}>
                  {quote.makerName}
                </Text>
                <Text style={[styles.quotePrice, { color: colors.textSecondary }]}>
                  价格: {(Number(quote.price) / 10000).toFixed(4)} USDT
                </Text>
              </View>
              <View style={styles.quoteOutput}>
                <Text style={[styles.quoteAmount, { color: Colors.success }]}>
                  {formatAmount(quote.outputAmount, direction === 'usdt_to_cos' ? 12 : 6)}
                </Text>
                <Text style={[styles.quoteToken, { color: colors.textSecondary }]}>
                  {outputToken}
                </Text>
              </View>
            </TouchableOpacity>
          ))}
        </Card>
      )}

      {/* Info */}
      {selectedQuote && (
        <Card style={styles.infoCard}>
          <View style={styles.infoRow}>
            <Text style={[styles.infoLabel, { color: colors.textSecondary }]}>
              汇率
            </Text>
            <Text style={[styles.infoValue, { color: colors.textPrimary }]}>
              1 COS = {(Number(selectedQuote.price) / 10000).toFixed(4)} USDT
            </Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={[styles.infoLabel, { color: colors.textSecondary }]}>
              手续费
            </Text>
            <Text style={[styles.infoValue, { color: colors.textPrimary }]}>
              0.3%
            </Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={[styles.infoLabel, { color: colors.textSecondary }]}>
              滑点保护
            </Text>
            <Text style={[styles.infoValue, { color: colors.textPrimary }]}>
              {selectedQuote.slippage}%
            </Text>
          </View>
        </Card>
      )}

      {/* Submit */}
      <Button
        title={swapping ? '兑换中...' : '兑换'}
        onPress={handleSwap}
        loading={swapping}
        disabled={!selectedQuote || loading}
        style={styles.submitButton}
      />
    </ScrollView>
  );
}

function parseAmount(value: string, decimals: number): string {
  const [whole, fraction = ''] = value.split('.');
  const paddedFraction = fraction.padEnd(decimals, '0').slice(0, decimals);
  return BigInt(whole + paddedFraction).toString();
}

function formatAmount(value: string, decimals: number): string {
  const bigValue = BigInt(value);
  const divisor = BigInt(10 ** decimals);
  const whole = bigValue / divisor;
  const fraction = bigValue % divisor;
  const fractionStr = fraction.toString().padStart(decimals, '0').slice(0, 4);
  return `${whole}.${fractionStr}`;
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    padding: 16,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
  },
  historyButton: {
    padding: 8,
  },
  swapCard: {
    marginBottom: 16,
    position: 'relative',
  },
  tokenSection: {
    marginBottom: 8,
  },
  sectionLabel: {
    fontSize: 14,
    marginBottom: 8,
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
  tokenBadge: {
    paddingHorizontal: 12,
    paddingVertical: 10,
    borderRadius: 8,
  },
  tokenText: {
    fontSize: 16,
    fontWeight: '600',
  },
  balanceText: {
    fontSize: 12,
    marginTop: 8,
  },
  switchButton: {
    position: 'absolute',
    top: '50%',
    left: '50%',
    marginLeft: -20,
    marginTop: -20,
    width: 40,
    height: 40,
    borderRadius: 20,
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1,
  },
  outputRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  outputAmount: {
    fontSize: 28,
    fontWeight: '700',
  },
  quotesCard: {
    marginBottom: 16,
  },
  quotesTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 12,
  },
  quoteItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 12,
    borderRadius: 12,
    borderWidth: 1.5,
    marginBottom: 8,
  },
  quoteInfo: {},
  quoteMaker: {
    fontSize: 15,
    fontWeight: '500',
    marginBottom: 2,
  },
  quotePrice: {
    fontSize: 13,
  },
  quoteOutput: {
    alignItems: 'flex-end',
  },
  quoteAmount: {
    fontSize: 18,
    fontWeight: '700',
  },
  quoteToken: {
    fontSize: 12,
  },
  infoCard: {
    marginBottom: 24,
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  infoLabel: {
    fontSize: 14,
  },
  infoValue: {
    fontSize: 14,
    fontWeight: '500',
  },
  submitButton: {},
});
