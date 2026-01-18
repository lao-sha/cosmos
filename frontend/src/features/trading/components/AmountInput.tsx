/**
 * 金额输入组件
 */

import React from 'react';
import { View, Text, TextInput, StyleSheet, TouchableOpacity } from 'react-native';

interface AmountInputProps {
  value: string;
  onChangeText: (text: string) => void;
  min?: number;
  max?: number;
  label?: string;
  unit?: string;
  quickAmounts?: number[];
}

export const AmountInput: React.FC<AmountInputProps> = ({
  value,
  onChangeText,
  min = 20,
  max = 200,
  label = '购买金额',
  unit = 'USD',
  quickAmounts = [20, 50, 100, 200],
}) => {
  const handleQuickAmount = (amount: number) => {
    onChangeText(amount.toString());
  };

  return (
    <View style={styles.container}>
      <Text style={styles.label}>{label} ({unit})</Text>

      <View style={styles.inputContainer}>
        <TextInput
          style={styles.input}
          value={value}
          onChangeText={onChangeText}
          keyboardType="decimal-pad"
          placeholder={`${min} - ${max}`}
          placeholderTextColor="#999999"
        />
        <Text style={styles.unit}>{unit}</Text>
      </View>

      <Text style={styles.hint}>
        最小 {min} | 最大 {max}
      </Text>

      <View style={styles.quickAmounts}>
        {quickAmounts.map((amount) => (
          <TouchableOpacity
            key={amount}
            style={[
              styles.quickButton,
              value === amount.toString() && styles.quickButtonActive,
            ]}
            onPress={() => handleQuickAmount(amount)}
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
  label: {
    fontSize: 14,
    fontWeight: '500',
    color: '#000000',
    marginBottom: 8,
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
  unit: {
    fontSize: 14,
    color: '#666666',
    marginLeft: 8,
  },
  hint: {
    fontSize: 12,
    color: '#999999',
    marginTop: 4,
  },
  quickAmounts: {
    flexDirection: 'row',
    marginTop: 12,
    gap: 8,
  },
  quickButton: {
    flex: 1,
    paddingVertical: 8,
    paddingHorizontal: 16,
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
