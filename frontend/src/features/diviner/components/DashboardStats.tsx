/**
 * 仪表盘统计组件
 */

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { Provider, TIER_CONFIG } from '../types';
import { TierBadge } from './TierBadge';
import { StatusBadge } from './StatusBadge';

const THEME_COLOR = '#B2955D';

interface DashboardStatsProps {
  provider: Provider;
  pendingOrders: number;
  todayEarnings: bigint;
  monthlyEarnings: bigint;
  availableBalance: bigint;
}

export function DashboardStats({
  provider,
  pendingOrders,
  todayEarnings,
  monthlyEarnings,
  availableBalance,
}: DashboardStatsProps) {
  const formatDust = (amount: bigint) => (Number(amount) / 1e10).toFixed(2);
  const tierConfig = TIER_CONFIG[provider.tier];
  const completionRate = provider.totalOrders > 0
    ? ((provider.completedOrders / provider.totalOrders) * 100).toFixed(1)
    : '0.0';

  return (
    <View style={styles.container}>
      {/* 头部信息 */}
      <View style={styles.header}>
        <View style={styles.profileInfo}>
          <View style={styles.avatar}>
            <Text style={styles.avatarText}>{provider.name.slice(0, 1)}</Text>
          </View>
          <View style={styles.nameSection}>
            <Text style={styles.name}>{provider.name}</Text>
            <View style={styles.badges}>
              <TierBadge tier={provider.tier} size="small" />
              <StatusBadge status={provider.status} />
            </View>
          </View>
        </View>
        <View style={styles.feeInfo}>
          <Text style={styles.feeLabel}>平台费率</Text>
          <Text style={styles.feeValue}>{tierConfig.feeRate}%</Text>
        </View>
      </View>

      {/* 收益统计 */}
      <View style={styles.earningsSection}>
        <View style={styles.earningsMain}>
          <Text style={styles.earningsLabel}>可提现余额</Text>
          <Text style={styles.earningsValue}>{formatDust(availableBalance)} DUST</Text>
        </View>
        <View style={styles.earningsRow}>
          <View style={styles.earningsItem}>
            <Text style={styles.earningsItemLabel}>今日收益</Text>
            <Text style={styles.earningsItemValue}>{formatDust(todayEarnings)}</Text>
          </View>
          <View style={styles.earningsItem}>
            <Text style={styles.earningsItemLabel}>本月收益</Text>
            <Text style={styles.earningsItemValue}>{formatDust(monthlyEarnings)}</Text>
          </View>
          <View style={styles.earningsItem}>
            <Text style={styles.earningsItemLabel}>累计收益</Text>
            <Text style={styles.earningsItemValue}>{formatDust(provider.totalEarnings)}</Text>
          </View>
        </View>
      </View>

      {/* 业务统计 */}
      <View style={styles.statsSection}>
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{pendingOrders}</Text>
          <Text style={styles.statLabel}>待处理订单</Text>
        </View>
        <View style={styles.statDivider} />
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{provider.completedOrders}</Text>
          <Text style={styles.statLabel}>已完成订单</Text>
        </View>
        <View style={styles.statDivider} />
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{completionRate}%</Text>
          <Text style={styles.statLabel}>完成率</Text>
        </View>
        <View style={styles.statDivider} />
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{provider.averageRating.toFixed(1)}</Text>
          <Text style={styles.statLabel}>平均评分</Text>
        </View>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 16,
    paddingBottom: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  profileInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  avatar: {
    width: 48,
    height: 48,
    borderRadius: 24,
    backgroundColor: THEME_COLOR,
    justifyContent: 'center',
    alignItems: 'center',
  },
  avatarText: {
    fontSize: 20,
    color: '#FFF',
    fontWeight: '600',
  },
  nameSection: {
    gap: 6,
  },
  name: {
    fontSize: 18,
    fontWeight: '600',
    color: '#000',
  },
  badges: {
    flexDirection: 'row',
    gap: 8,
  },
  feeInfo: {
    alignItems: 'flex-end',
  },
  feeLabel: {
    fontSize: 12,
    color: '#999',
    marginBottom: 2,
  },
  feeValue: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME_COLOR,
  },
  earningsSection: {
    backgroundColor: '#FFF9F0',
    borderRadius: 8,
    padding: 16,
    marginBottom: 16,
  },
  earningsMain: {
    alignItems: 'center',
    marginBottom: 16,
  },
  earningsLabel: {
    fontSize: 14,
    color: '#666',
    marginBottom: 4,
  },
  earningsValue: {
    fontSize: 28,
    fontWeight: '700',
    color: THEME_COLOR,
  },
  earningsRow: {
    flexDirection: 'row',
    justifyContent: 'space-around',
  },
  earningsItem: {
    alignItems: 'center',
  },
  earningsItemLabel: {
    fontSize: 12,
    color: '#999',
    marginBottom: 4,
  },
  earningsItemValue: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  statsSection: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  statItem: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 18,
    fontWeight: '600',
    color: '#000',
    marginBottom: 4,
  },
  statLabel: {
    fontSize: 12,
    color: '#999',
  },
  statDivider: {
    width: 1,
    height: 30,
    backgroundColor: '#F0F0F0',
  },
});
