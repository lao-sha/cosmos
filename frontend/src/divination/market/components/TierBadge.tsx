// frontend/src/divination/market/components/TierBadge.tsx

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { TIER_CONFIG } from '../constants/market.constants';

interface TierBadgeProps {
  tier: number;
  size?: 'small' | 'medium' | 'large';
  showIcon?: boolean;
}

export const TierBadge: React.FC<TierBadgeProps> = ({
  tier,
  size = 'small',
  showIcon = true,
}) => {
  const config = TIER_CONFIG[tier] || TIER_CONFIG[0];

  const sizeStyles = {
    small: { paddingH: 6, paddingV: 2, fontSize: 11, iconSize: 10 },
    medium: { paddingH: 8, paddingV: 3, fontSize: 12, iconSize: 12 },
    large: { paddingH: 10, paddingV: 4, fontSize: 14, iconSize: 14 },
  };

  const s = sizeStyles[size];

  return (
    <View
      style={[
        styles.badge,
        {
          backgroundColor: config.color + '20',
          paddingHorizontal: s.paddingH,
          paddingVertical: s.paddingV,
        },
      ]}
    >
      {showIcon && (
        <Ionicons
          name={config.icon as any}
          size={s.iconSize}
          color={config.color}
          style={styles.icon}
        />
      )}
      <Text style={[styles.text, { color: config.color, fontSize: s.fontSize }]}>
        {config.name}
      </Text>
    </View>
  );
};

const styles = StyleSheet.create({
  badge: {
    flexDirection: 'row',
    alignItems: 'center',
    borderRadius: 4,
  },
  icon: {
    marginRight: 3,
  },
  text: {
    fontWeight: '500',
  },
});
