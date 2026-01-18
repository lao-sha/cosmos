/**
 * 价格显示组件
 */

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';

interface PriceDisplayProps {
  price: number;
  priceChange24h?: number;
  label?: string;
}

export const PriceDisplay: React.FC<PriceDisplayProps> = ({
  price,
  priceChange24h,
  label = '当前价格',
}) => {
  const changeColor = priceChange24h && priceChange24h >= 0 ? '#4CD964' : '#FF3B30';
  const changePrefix = priceChange24h && priceChange24h >= 0 ? '+' : '';

  return (
    <View style={styles.container}>
      <Text style={styles.label}>{label}</Text>
      <View style={styles.priceRow}>
        <Text style={styles.price}>
          1 DUST = {price.toFixed(4)} USDT
        </Text>
        {priceChange24h !== undefined && (
          <View style={[styles.changeBadge, { backgroundColor: changeColor + '20' }]}>
            <Text style={[styles.changeText, { color: changeColor }]}>
              24h: {changePrefix}{priceChange24h.toFixed(2)}%
            </Text>
          </View>
        )}
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  label: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 8,
  },
  priceRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  price: {
    fontSize: 18,
    fontWeight: '600',
    color: '#000000',
  },
  changeBadge: {
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
  },
  changeText: {
    fontSize: 12,
    fontWeight: '500',
  },
});
