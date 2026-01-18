/**
 * 兑换金额输入组件
 * 支持 DUST 输入和 USDT 预估显示
 */

import React from 'react';
import { View, Text, TextInput, StyleSheet, TouchableOpacity } from 'react-native';

interface SwapAmountInputProps {
  value: string;
  onChangeText: (text: string) => void;
  dustPrice: number;
  balance?: string;
  minAmount?: number;
  label?: string;
}

export const SwapAmountInput: React.FC<SwapAmountInputProps> = ({
  value,
  onChangeText,
  dustPrice,
  balance,
  minAmount = 10,
  label = '兑换数量',
}) => {
  const dustAmount = parseFloat(value) || 0;
  const usdtEstimate = dustAmount * dustPrice;

  const handleMax = () => {
    if (balance) {
      onChangeText(balance);
    }
  };

  const quickAmounts = [100, 500, 1000, 5000];

  return (
    <View style={styles.container}>
      <View style={styles.labelRow}>
        <Text style={styles.label}>{label} (DUST)</Text>
        {balance && (
          <Text style={styles.balance}>
            可用: {parseFloat(balance).toFixed(2)} DUST
          </Text>
        )}
      </View>

      <View style={styles.inputContainer}>
        <TextInput
          style={styles.input}
          value={value}
          onChangeText={onChangeText}
          keyboardType="decimal-pad"
          placeholder={`最小 ${minAmount} DUST`}
          placeholderTextColor="#999999"
        />
        <TouchableOpacity style={styles.maxButton} onPress={handleMax}>
          <Text style={styles.maxText}>MAX</Text>
        </TouchableOpacity>
      </View>

      <View style={styles.estimateRow}>
        <Text style={styles.estimateLabel}>预计获得</Text>
        <Text style={styles.estimateValue}>
          ≈ {usdtEstimate.toFixed(2)} USDT
        </Text>
      </View>

      <View style={styles.quickAmounts}>
        {quickAmounts.map((amount) => (
          <TouchableOpacity
            key={amount}
            style={[
              styles.quickButton,
              value === amount.toString() && styles.quickButtonActive,
            ]}
            onPress={() => onChangeText(amount.toString())}
          >
            <Text
              style={[
                styles.quickButtonText,
                value === amount.toString() && styles.quickButtonTextActive,
              ]}
            >
              {amount}
            </Text>
          </TouchableOpacity>
        ))}
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    marginBottom: 16,
  },
  labelRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  label: {
    fontSize: 14,
    fontWeight: '500',
    color: '#000000',
  },
  balance: {
    fontSize: 12,
    color: '#666666',
  },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    paddingHorizontal: 16,
    borderWidth: 1,
    borderColor: '#E5E5EA',
  },
  input: {
    flex: 1,
    fontSize: 18,
    fontWeight: '600',
    color: '#000000',
    paddingVertical: 12,
  },
  maxButton: {
    backgroundColor: '#B2955D',
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 4,
  },
  maxText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  estimateRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginTop: 8,
    paddingHorizontal: 4,
  },
  estimateLabel: {
    fontSize: 12,
    color: '#999999',
  },
  estimateValue: {
    fontSize: 14,
    fontWeight: '600',
    color: '#4CD964',
  },
  quickAmounts: {
    flexDirection: 'row',
    marginTop: 12,
    gap: 8,
  },
  quickButton: {
    flex: 1,
    paddingVertical: 8,
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#E5E5EA',
  },
  quickButtonActive: {
    backgroundColor: '#B2955D',
    borderColor: '#B2955D',
  },
  quickButtonText: {
    fontSize: 14,
    fontWeight: '500',
    color: '#666666',
  },
  quickButtonTextActive: {
    color: '#FFFFFF',
  },
});
