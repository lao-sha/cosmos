/**
 * 做市商状态卡片组件
 */

import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import { MakerApplication, ApplicationStatus, MakerService } from '@/services/maker.service';

interface MakerStatusCardProps {
  maker: MakerApplication;
  depositUsdValue: number;
  onPress?: () => void;
}

export function MakerStatusCard({ maker, depositUsdValue, onPress }: MakerStatusCardProps) {
  const getStatusConfig = (status: ApplicationStatus) => {
    const configs: Record<ApplicationStatus, { color: string; bgColor: string; icon: string }> = {
      [ApplicationStatus.DepositLocked]: { color: '#FF9500', bgColor: '#FF950020', icon: '🔒' },
      [ApplicationStatus.PendingReview]: { color: '#007AFF', bgColor: '#007AFF20', icon: '⏳' },
      [ApplicationStatus.Active]: { color: '#4CD964', bgColor: '#4CD96420', icon: '🟢' },
      [ApplicationStatus.Rejected]: { color: '#FF3B30', bgColor: '#FF3B3020', icon: '❌' },
      [ApplicationStatus.Cancelled]: { color: '#8E8E93', bgColor: '#8E8E9320', icon: '⭕' },
      [ApplicationStatus.Expired]: { color: '#8E8E93', bgColor: '#8E8E9320', icon: '⏰' },
    };
    return configs[status];
  };

  const statusConfig = getStatusConfig(maker.status);
  const isOnline = maker.status === ApplicationStatus.Active && !maker.servicePaused;

  return (
    <TouchableOpacity
      style={styles.container}
      onPress={onPress}
      activeOpacity={onPress ? 0.7 : 1}
    >
      <View style={styles.header}>
        <View style={styles.makerInfo}>
          <Text style={styles.makerId}>做市商 #{maker.id}</Text>
          <Text style={styles.makerName}>
            {maker.maskedFullName} | {isOnline ? '在线 🟢' : '离线 ⚪'}
          </Text>
        </View>
        <View style={[styles.statusBadge, { backgroundColor: statusConfig.bgColor }]}>
          <Text style={[styles.statusText, { color: statusConfig.color }]}>
            {statusConfig.icon} {MakerService.getStatusText(maker.status)}
          </Text>
        </View>
      </View>

      {maker.status === ApplicationStatus.Active && (
        <>
          <View style={styles.statsRow}>
            <View style={styles.statItem}>
              <Text style={styles.statValue}>⭐ 4.9</Text>
              <Text style={styles.statLabel}>评分</Text>
            </View>
            <View style={styles.statDivider} />
            <View style={styles.statItem}>
              <Text style={styles.statValue}>{maker.usersServed.toLocaleString()}</Text>
              <Text style={styles.statLabel}>已服务用户</Text>
            </View>
            <View style={styles.statDivider} />
            <View style={styles.statItem}>
              <Text style={styles.statValue}>${MakerService.formatUsdAmount(depositUsdValue)}</Text>
              <Text style={styles.statLabel}>押金价值</Text>
            </View>
          </View>

          {maker.depositWarning && (
            <View style={styles.warningBanner}>
              <Text style={styles.warningText}>
                ⚠️ 押金不足，请及时补充
              </Text>
            </View>
          )}
        </>
      )}
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.05,
    shadowRadius: 8,
    elevation: 2,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
  },
  makerInfo: {
    flex: 1,
  },
  makerId: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 4,
  },
  makerName: {
    fontSize: 14,
    color: '#8E8E93',
  },
  statusBadge: {
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 6,
  },
  statusText: {
    fontSize: 12,
    fontWeight: '500',
  },
  statsRow: {
    flexDirection: 'row',
    marginTop: 16,
    paddingTop: 16,
    borderTopWidth: 1,
    borderTopColor: '#F2F2F7',
  },
  statItem: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 4,
  },
  statLabel: {
    fontSize: 12,
    color: '#8E8E93',
  },
  statDivider: {
    width: 1,
    backgroundColor: '#F2F2F7',
    marginHorizontal: 8,
  },
  warningBanner: {
    marginTop: 12,
    padding: 10,
    backgroundColor: '#FF950020',
    borderRadius: 8,
  },
  warningText: {
    fontSize: 13,
    color: '#FF9500',
    textAlign: 'center',
  },
});
