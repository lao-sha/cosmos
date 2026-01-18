/**
 * 溢价滑块组件
 */

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import Slider from '@react-native-community/slider';

interface PremiumSliderProps {
  label: string;
  value: number; // 基点 (-500 ~ 500)
  onChange: (value: number) => void;
  disabled?: boolean;
}

export function PremiumSlider({ label, value, onChange, disabled = false }: PremiumSliderProps) {
  const percentage = value / 100;
  const isPositive = value >= 0;

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.label}>{label}</Text>
        <Text style={[styles.value, isPositive ? styles.valuePositive : styles.valueNegative]}>
          {isPositive ? '+' : ''}{percentage.toFixed(1)}%
        </Text>
      </View>

      <Slider
        style={styles.slider}
        minimumValue={-500}
        maximumValue={500}
        step={10}
        value={value}
        onValueChange={onChange}
        disabled={disabled}
        minimumTrackTintColor="#B2955D"
        maximumTrackTintColor="#E5E5EA"
        thumbTintColor="#B2955D"
      />

      <View style={styles.rangeLabels}>
        <Text style={styles.rangeLabel}>-5%</Text>
        <Text style={styles.rangeLabel}>0%</Text>
        <Text style={styles.rangeLabel}>+5%</Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    marginBottom: 16,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  label: {
    fontSize: 14,
    color: '#1C1C1E',
    fontWeight: '500',
  },
  value: {
    fontSize: 16,
    fontWeight: '600',
  },
  valuePositive: {
    color: '#4CD964',
  },
  valueNegative: {
    color: '#FF3B30',
  },
  slider: {
    width: '100%',
    height: 40,
  },
  rangeLabels: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingHorizontal: 4,
  },
  rangeLabel: {
    fontSize: 12,
    color: '#8E8E93',
  },
});
