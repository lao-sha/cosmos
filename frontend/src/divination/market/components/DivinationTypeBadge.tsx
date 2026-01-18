// frontend/src/divination/market/components/DivinationTypeBadge.tsx

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { getDivinationTypeName, getDivinationTypeColor } from '../utils/market.utils';
import { DivinationType } from '../types/market.types';

interface DivinationTypeBadgeProps {
  type: DivinationType | number;
  size?: 'small' | 'medium' | 'large';
}

export const DivinationTypeBadge: React.FC<DivinationTypeBadgeProps> = ({
  type,
  size = 'small',
}) => {
  const name = getDivinationTypeName(type);
  const color = getDivinationTypeColor(type);

  const sizeStyles = {
    small: { paddingH: 6, paddingV: 2, fontSize: 11 },
    medium: { paddingH: 8, paddingV: 3, fontSize: 12 },
    large: { paddingH: 10, paddingV: 4, fontSize: 14 },
  };

  const s = sizeStyles[size];

  return (
    <View
      style={[
        styles.badge,
        {
          backgroundColor: color + '20',
          paddingHorizontal: s.paddingH,
          paddingVertical: s.paddingV,
        },
      ]}
    >
      <Text style={[styles.text, { color, fontSize: s.fontSize }]}>{name}</Text>
    </View>
  );
};

const styles = StyleSheet.create({
  badge: {
    borderRadius: 4,
  },
  text: {
    fontWeight: '500',
  },
});
