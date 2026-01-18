/**
 * 占卜师等级徽章组件
 */

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { ProviderTier, TIER_CONFIG } from '../types';

interface TierBadgeProps {
  tier: ProviderTier;
  size?: 'small' | 'medium' | 'large';
  showLabel?: boolean;
}

export function TierBadge({ tier, size = 'medium', showLabel = true }: TierBadgeProps) {
  const config = TIER_CONFIG[tier];
  
  const sizeStyles = {
    small: { iconSize: 14, fontSize: 10, padding: 4 },
    medium: { iconSize: 18, fontSize: 12, padding: 6 },
    large: { iconSize: 24, fontSize: 14, padding: 8 },
  };
  
  const s = sizeStyles[size];

  return (
    <View style={[styles.container, { backgroundColor: `${config.color}20`, padding: s.padding }]}>
      <Text style={{ fontSize: s.iconSize }}>{config.icon}</Text>
      {showLabel && (
        <Text style={[styles.label, { color: config.color, fontSize: s.fontSize }]}>
          {config.label}
        </Text>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    borderRadius: 4,
    gap: 4,
  },
  label: {
    fontWeight: '500',
  },
});
