/**
 * 桥接做市商卡片组件
 */

import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import { BridgeMaker } from '../types';

interface BridgeMakerCardProps {
  maker: BridgeMaker;
  onPress?: () => void;
  selected?: boolean;
}

export const BridgeMakerCard: React.FC<BridgeMakerCardProps> = ({
  maker,
  onPress,
  selected = false,
}) => {
  const statusColor = maker.isActive ? '#4CD964' : '#8E8E93';

  const formatAddress = (address: string): string => {
    if (address.length <= 12) return address;
    return `${address.slice(0, 6)}...${address.slice(-6)}`;
  };

  return (
    <TouchableOpacity
      style={[styles.container, selected && styles.selected]}
      onPress={onPress}
      activeOpacity={0.7}
    >
      <View style={styles.header}>
        <View style={styles.makerInfo}>
          <Text style={styles.makerId}>做市商 #{maker.id}</Text>
          <View style={styles.statusBadge}>
            <View style={[styles.statusDot, { backgroundColor: statusColor }]} />
            <Text style={styles.statusText}>
              {maker.isActive ? '活跃' : '离线'}
            </Text>
          </View>
        </View>
        <View style={styles.ratingContainer}>
          <Text style={styles.rating}>⭐ {maker.rating.toFixed(1)}</Text>
          <Text style={styles.completedSwaps}>{maker.completedSwaps}单</Text>
        </View>
      </View>

      <View style={styles.details}>
        <View style={styles.detailItem}>
          <Text style={styles.detailLabel}>TRON 地址</Text>
          <Text style={styles.detailValue}>{formatAddress(maker.tronAddress)}</Text>
        </View>

        {maker.avgResponseTime && (
          <View style={styles.detailItem}>
            <Text style={styles.detailLabel}>平均响应</Text>
            <Text style={styles.detailValue}>
              {Math.floor(maker.avgResponseTime / 60)}分钟
            </Text>
          </View>
        )}

        {maker.creditLevel && (
          <View style={styles.detailItem}>
            <Text style={styles.detailLabel}>信用等级</Text>
            <Text style={styles.detailValue}>{maker.creditLevel}</Text>
          </View>
        )}
      </View>

      {selected && (
        <View style={styles.selectedIndicator}>
          <Text style={styles.selectedText}>✓ 已选择</Text>
        </View>
      )}
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
  makerId: {
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
  completedSwaps: {
    fontSize: 12,
    color: '#666666',
  },
  details: {
    flexDirection: 'row',
    flexWrap: 'wrap',
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
  selectedIndicator: {
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
    paddingTop: 12,
    marginTop: 4,
    alignItems: 'center',
  },
  selectedText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#B2955D',
  },
});
