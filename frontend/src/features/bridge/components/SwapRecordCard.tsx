/**
 * 兑换记录卡片组件
 */

import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import { SwapStatusBadge } from './SwapStatusBadge';
import { MakerSwapRecord, SwapStatus } from '../types';

interface SwapRecordCardProps {
  record: MakerSwapRecord;
  onPress?: () => void;
  onReport?: () => void;
}

export const SwapRecordCard: React.FC<SwapRecordCardProps> = ({
  record,
  onPress,
  onReport,
}) => {
  const formatDust = (amount: bigint): string => {
    return (Number(amount) / 1e12).toFixed(4);
  };

  const formatUsdt = (amount: number): string => {
    return (amount / 1e6).toFixed(2);
  };

  const formatTime = (blockNumber: number): string => {
    // 简化显示，实际应转换为时间
    return `区块 #${blockNumber}`;
  };

  const canReport = record.status === SwapStatus.Pending || record.status === SwapStatus.Completed;

  return (
    <TouchableOpacity
      style={styles.container}
      onPress={onPress}
      activeOpacity={0.7}
    >
      <View style={styles.header}>
        <Text style={styles.swapId}>兑换 #{record.swapId}</Text>
        <SwapStatusBadge status={record.status} />
      </View>

      <View style={styles.amountRow}>
        <View style={styles.amountItem}>
          <Text style={styles.amountLabel}>支付</Text>
          <Text style={styles.amountValue}>{formatDust(record.dustAmount)} DUST</Text>
        </View>
        <Text style={styles.arrow}>→</Text>
        <View style={styles.amountItem}>
          <Text style={styles.amountLabel}>获得</Text>
          <Text style={styles.amountValueGreen}>{formatUsdt(record.usdtAmount)} USDT</Text>
        </View>
      </View>

      <View style={styles.details}>
        <View style={styles.detailRow}>
          <Text style={styles.detailLabel}>做市商</Text>
          <Text style={styles.detailValue}>#{record.makerId}</Text>
        </View>
        <View style={styles.detailRow}>
          <Text style={styles.detailLabel}>价格</Text>
          <Text style={styles.detailValue}>{(record.priceUsdt / 1e6).toFixed(6)} USDT</Text>
        </View>
        <View style={styles.detailRow}>
          <Text style={styles.detailLabel}>创建时间</Text>
          <Text style={styles.detailValue}>{formatTime(record.createdAt)}</Text>
        </View>
      </View>

      {record.trc20TxHash && (
        <View style={styles.txHashRow}>
          <Text style={styles.txHashLabel}>交易哈希</Text>
          <Text style={styles.txHash} numberOfLines={1} ellipsizeMode="middle">
            {record.trc20TxHash}
          </Text>
        </View>
      )}

      {canReport && onReport && (
        <TouchableOpacity style={styles.reportButton} onPress={onReport}>
          <Text style={styles.reportText}>⚠️ 举报问题</Text>
        </TouchableOpacity>
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
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  swapId: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
  },
  amountRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    padding: 12,
    marginBottom: 12,
  },
  amountItem: {
    flex: 1,
    alignItems: 'center',
  },
  amountLabel: {
    fontSize: 12,
    color: '#999999',
    marginBottom: 4,
  },
  amountValue: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
  },
  amountValueGreen: {
    fontSize: 16,
    fontWeight: '600',
    color: '#4CD964',
  },
  arrow: {
    fontSize: 18,
    color: '#B2955D',
    marginHorizontal: 8,
  },
  details: {
    marginBottom: 12,
  },
  detailRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 4,
  },
  detailLabel: {
    fontSize: 14,
    color: '#666666',
  },
  detailValue: {
    fontSize: 14,
    color: '#000000',
  },
  txHashRow: {
    backgroundColor: '#F5F5F7',
    borderRadius: 6,
    padding: 8,
    marginBottom: 12,
  },
  txHashLabel: {
    fontSize: 12,
    color: '#999999',
    marginBottom: 4,
  },
  txHash: {
    fontSize: 12,
    color: '#666666',
    fontFamily: 'monospace',
  },
  reportButton: {
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
    paddingTop: 12,
    alignItems: 'center',
  },
  reportText: {
    fontSize: 14,
    color: '#FF3B30',
  },
});
