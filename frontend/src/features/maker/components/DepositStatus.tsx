/**
 * 押金状态组件
 */

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { MakerService } from '@/services/maker.service';

interface DepositStatusProps {
  depositAmount: bigint;
  depositUsdValue: number;
  targetUsd?: number;
  thresholdUsd?: number;
  showProgress?: boolean;
}

const DEPOSIT_TARGET_USD = 1000;
const DEPOSIT_THRESHOLD_USD = 950;

export function DepositStatus({
  depositAmount,
  depositUsdValue,
  targetUsd = DEPOSIT_TARGET_USD,
  thresholdUsd = DEPOSIT_THRESHOLD_USD,
  showProgress = true,
}: DepositStatusProps) {
  const getStatus = () => {
    if (depositUsdValue >= targetUsd) {
      return { text: '正常', color: '#4CD964', icon: '✅' };
    }
    if (depositUsdValue >= thresholdUsd) {
      return { text: '接近阈值', color: '#FF9500', icon: '⚠️' };
    }
    return { text: '低于阈值', color: '#FF3B30', icon: '❌' };
  };

  const status = getStatus();
  const progressPercent = Math.min((depositUsdValue / targetUsd) * 100, 100);

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.label}>押金状态</Text>
        <View style={[styles.statusBadge, { backgroundColor: status.color + '20' }]}>
          <Text style={[styles.statusText, { color: status.color }]}>
            {status.icon} {status.text}
          </Text>
        </View>
      </View>

      <View style={styles.amountContainer}>
        <Text style={styles.dustAmount}>
          {MakerService.formatDustAmount(depositAmount)} DUST
        </Text>
        <Text style={styles.usdAmount}>
          ≈ ${MakerService.formatUsdAmount(depositUsdValue)} USD
        </Text>
      </View>

      {showProgress && (
        <View style={styles.progressContainer}>
          <View style={styles.progressBar}>
            <View
              style={[
                styles.progressFill,
                {
                  width: `${progressPercent}%`,
                  backgroundColor: status.color,
                },
              ]}
            />
            {/* 阈值标记 */}
            <View
              style={[
                styles.thresholdMarker,
                { left: `${(thresholdUsd / targetUsd) * 100}%` },
              ]}
            />
          </View>
          <View style={styles.progressLabels}>
            <Text style={styles.progressLabel}>$0</Text>
            <Text style={styles.progressLabel}>阈值: ${thresholdUsd}</Text>
            <Text style={styles.progressLabel}>目标: ${targetUsd}</Text>
          </View>
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  label: {
    fontSize: 14,
    color: '#8E8E93',
  },
  statusBadge: {
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
  },
  statusText: {
    fontSize: 12,
    fontWeight: '500',
  },
  amountContainer: {
    marginBottom: 16,
  },
  dustAmount: {
    fontSize: 24,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 4,
  },
  usdAmount: {
    fontSize: 14,
    color: '#8E8E93',
  },
  progressContainer: {
    marginTop: 8,
  },
  progressBar: {
    height: 8,
    backgroundColor: '#E5E5EA',
    borderRadius: 4,
    overflow: 'hidden',
    position: 'relative',
  },
  progressFill: {
    height: '100%',
    borderRadius: 4,
  },
  thresholdMarker: {
    position: 'absolute',
    top: 0,
    bottom: 0,
    width: 2,
    backgroundColor: '#FF9500',
  },
  progressLabels: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginTop: 4,
  },
  progressLabel: {
    fontSize: 10,
    color: '#8E8E93',
  },
});
