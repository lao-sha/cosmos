/**
 * 做市商卡片组件
 */

import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import type { Maker } from '@/stores/trading.store';

interface MakerCardProps {
  maker: Maker;
  onPress?: () => void;
  selected?: boolean;
}

export const MakerCard: React.FC<MakerCardProps> = ({
  maker,
  onPress,
  selected = false,
}) => {
  // 计算溢价显示
  const premiumText = maker.sellPremiumBps >= 0
    ? `+${(maker.sellPremiumBps / 100).toFixed(1)}%`
    : `${(maker.sellPremiumBps / 100).toFixed(1)}%`;

  // 在线状态颜色
  const statusColor = maker.isOnline ? '#4CD964' : '#8E8E93';

  return (
    <TouchableOpacity
      style={[styles.container, selected && styles.selected]}
      onPress={onPress}
      activeOpacity={0.7}
    >
      <View style={styles.header}>
        <View style={styles.makerInfo}>
          <Text style={styles.makerName}>👤 {maker.maskedFullName}</Text>
          <View style={styles.statusBadge}>
            <View style={[styles.statusDot, { backgroundColor: statusColor }]} />
            <Text style={styles.statusText}>
              {maker.isOnline ? '在线' : '离线'}
            </Text>
          </View>
        </View>
        <View style={styles.ratingContainer}>
          <Text style={styles.rating}>⭐ {maker.rating.toFixed(1)}</Text>
          <Text style={styles.usersServed}>{maker.usersServed}单</Text>
        </View>
      </View>

      <View style={styles.details}>
        <View style={styles.detailItem}>
          <Text style={styles.detailLabel}>溢价</Text>
          <Text style={[
            styles.detailValue,
            maker.sellPremiumBps >= 0 ? styles.premiumPositive : styles.premiumNegative
          ]}>
            {premiumText}
          </Text>
        </View>

        {maker.creditScore && (
          <View style={styles.detailItem}>
            <Text style={styles.detailLabel}>信用</Text>
            <Text style={styles.detailValue}>{maker.creditLevel}</Text>
          </View>
        )}

        {maker.avgResponseTime && (
          <View style={styles.detailItem}>
            <Text style={styles.detailLabel}>响应</Text>
            <Text style={styles.detailValue}>
              {Math.floor(maker.avgResponseTime / 60)}分钟
            </Text>
          </View>
        )}

        {maker.completionRate && (
          <View style={styles.detailItem}>
            <Text style={styles.detailLabel}>完成率</Text>
            <Text style={styles.detailValue}>
              {maker.completionRate.toFixed(1)}%
            </Text>
          </View>
        )}
      </View>

      <View style={styles.footer}>
        <Text style={styles.wechat}>微信: {maker.wechatId}</Text>
      </View>
    </TouchableOpacity>
  );
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    borderWidth: 2,
    borderColor: 'transparent',
  },
  selected: {
    borderColor: '#B2955D',
    backgroundColor: '#FFF9F0',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 12,
  },
  makerInfo: {
    flex: 1,
  },
  makerName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 4,
  },
  statusBadge: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  statusDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    marginRight: 4,
  },
  statusText: {
    fontSize: 12,
    color: '#666666',
  },
  ratingContainer: {
    alignItems: 'flex-end',
  },
  rating: {
    fontSize: 14,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 2,
  },
  usersServed: {
    fontSize: 12,
    color: '#666666',
  },
  details: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    marginBottom: 12,
  },
  detailItem: {
    marginRight: 16,
    marginBottom: 8,
  },
  detailLabel: {
    fontSize: 12,
    color: '#999999',
    marginBottom: 2,
  },
  detailValue: {
    fontSize: 14,
    fontWeight: '500',
    color: '#000000',
  },
  premiumPositive: {
    color: '#FF3B30',
  },
  premiumNegative: {
    color: '#4CD964',
  },
  footer: {
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
    paddingTop: 12,
  },
  wechat: {
    fontSize: 12,
    color: '#666666',
  },
});
