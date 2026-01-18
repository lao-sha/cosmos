/**
 * 占卜类型选择器组件
 */

import React from 'react';
import { View, Text, StyleSheet, Pressable } from 'react-native';
import { DivinationType, DIVINATION_TYPE_CONFIG } from '../types';

const THEME_COLOR = '#B2955D';

interface DivinationTypeSelectorProps {
  value: number;
  onChange: (value: number) => void;
  disabled?: boolean;
}

export function DivinationTypeSelector({ value, onChange, disabled }: DivinationTypeSelectorProps) {
  const types = Object.entries(DIVINATION_TYPE_CONFIG) as [string, { label: string; icon: string }][];

  const toggleType = (type: number) => {
    if (disabled) return;
    const bit = 1 << type;
    if (value & bit) {
      onChange(value & ~bit);
    } else {
      onChange(value | bit);
    }
  };

  const isSelected = (type: number) => (value & (1 << type)) !== 0;

  return (
    <View style={styles.container}>
      <Text style={styles.title}>支持的占卜类型（至少选择1项）</Text>
      <View style={styles.grid}>
        {types.map(([key, config]) => {
          const type = Number(key);
          const selected = isSelected(type);
          return (
            <Pressable
              key={key}
              style={[styles.item, selected && styles.itemSelected, disabled && styles.itemDisabled]}
              onPress={() => toggleType(type)}
            >
              <Text style={styles.icon}>{config.icon}</Text>
              <Text style={[styles.label, selected && styles.labelSelected]}>{config.label}</Text>
            </Pressable>
          );
        })}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    marginBottom: 16,
  },
  title: {
    fontSize: 14,
    color: '#333',
    marginBottom: 12,
    fontWeight: '500',
  },
  grid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  item: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    backgroundColor: '#FFF',
    gap: 4,
  },
  itemSelected: {
    borderColor: THEME_COLOR,
    backgroundColor: `${THEME_COLOR}10`,
  },
  itemDisabled: {
    opacity: 0.5,
  },
  icon: {
    fontSize: 16,
  },
  label: {
    fontSize: 13,
    color: '#666',
  },
  labelSelected: {
    color: THEME_COLOR,
    fontWeight: '500',
  },
});
