/**
 * 占卜师状态徽章组件
 */

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { ProviderStatus, STATUS_CONFIG } from '../types';

interface StatusBadgeProps {
  status: ProviderStatus;
}

export function StatusBadge({ status }: StatusBadgeProps) {
  const config = STATUS_CONFIG[status];

  return (
    <View style={[styles.container, { backgroundColor: `${config.color}20` }]}>
      <View style={[styles.dot, { backgroundColor: config.color }]} />
      <Text style={[styles.label, { color: config.color }]}>{config.label}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
    gap: 6,
  },
  dot: {
    width: 6,
    height: 6,
    borderRadius: 3,
  },
  label: {
    fontSize: 12,
    fontWeight: '500',
  },
});
