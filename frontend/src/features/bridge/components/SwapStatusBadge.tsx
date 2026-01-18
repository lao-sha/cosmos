/**
 * 兑换状态徽章组件
 */

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { SwapStatus } from '../types';

interface SwapStatusBadgeProps {
  status: SwapStatus;
  size?: 'small' | 'medium' | 'large';
}

const statusConfig: Record<SwapStatus, { label: string; color: string; bgColor: string }> = {
  [SwapStatus.Pending]: {
    label: '处理中',
    color: '#FF9500',
    bgColor: '#FFF5E6',
  },
  [SwapStatus.Completed]: {
    label: '已完成',
    color: '#4CD964',
    bgColor: '#E8F8EB',
  },
  [SwapStatus.UserReported]: {
    label: '已举报',
    color: '#FF3B30',
    bgColor: '#FFE5E5',
  },
  [SwapStatus.Arbitrating]: {
    label: '仲裁中',
    color: '#5856D6',
    bgColor: '#EEEEFF',
  },
  [SwapStatus.ArbitrationApproved]: {
    label: '仲裁通过',
    color: '#4CD964',
    bgColor: '#E8F8EB',
  },
  [SwapStatus.ArbitrationRejected]: {
    label: '仲裁拒绝',
    color: '#FF3B30',
    bgColor: '#FFE5E5',
  },
  [SwapStatus.Refunded]: {
    label: '已退款',
    color: '#8E8E93',
    bgColor: '#F5F5F7',
  },
};

export const SwapStatusBadge: React.FC<SwapStatusBadgeProps> = ({
  status,
  size = 'medium',
}) => {
  const config = statusConfig[status];
  const sizeStyles = {
    small: { paddingH: 6, paddingV: 2, fontSize: 10 },
    medium: { paddingH: 10, paddingV: 4, fontSize: 12 },
    large: { paddingH: 14, paddingV: 6, fontSize: 14 },
  };
  const s = sizeStyles[size];

  return (
    <View
      style={[
        styles.badge,
        {
          backgroundColor: config.bgColor,
          paddingHorizontal: s.paddingH,
          paddingVertical: s.paddingV,
        },
      ]}
    >
      <Text style={[styles.text, { color: config.color, fontSize: s.fontSize }]}>
        {config.label}
      </Text>
    </View>
  );
};

const styles = StyleSheet.create({
  badge: {
    borderRadius: 4,
    alignSelf: 'flex-start',
  },
  text: {
    fontWeight: '500',
  },
});
