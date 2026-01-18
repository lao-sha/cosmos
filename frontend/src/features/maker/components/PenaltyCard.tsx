/**
 * 扣除记录卡片组件
 */

import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import { PenaltyRecord, MakerService } from '@/services/maker.service';

interface PenaltyCardProps {
  penalty: PenaltyRecord;
  onViewDetail?: () => void;
  onAppeal?: () => void;
}

export function PenaltyCard({ penalty, onViewDetail, onAppeal }: PenaltyCardProps) {
  const getTypeConfig = (type: string) => {
    const configs: Record<string, { color: string; icon: string }> = {
      OtcTimeout: { color: '#FF9500', icon: '⚠️' },
      BridgeTimeout: { color: '#FF9500', icon: '⚠️' },
      ArbitrationLoss: { color: '#FF3B30', icon: '❌' },
      LowCreditScore: { color: '#FF9500', icon: '📉' },
      MaliciousBehavior: { color: '#FF3B30', icon: '🚫' },
    };
    return configs[type] || { color: '#8E8E93', icon: '❓' };
  };

  const typeConfig = getTypeConfig(penalty.penaltyType.type);
  const typeText = MakerService.getPenaltyTypeText(penalty.penaltyType);

  const getAppealStatus = () => {
    if (!penalty.appealed) {
      return { text: '未申诉', color: '#8E8E93' };
    }
    if (penalty.appealResult === undefined) {
      return { text: '申诉中', color: '#007AFF' };
    }
    if (penalty.appealResult) {
      return { text: '申诉成功', color: '#4CD964' };
    }
    return { text: '申诉驳回', color: '#FF3B30' };
  };

  const appealStatus = getAppealStatus();

  // 计算申诉截止时间（假设7天）
  const appealDeadline = new Date((penalty.deductedAt + 7 * 24 * 3600) * 1000);
  const now = new Date();
  const canAppeal = !penalty.appealed && now < appealDeadline;
  const daysLeft = Math.ceil((appealDeadline.getTime() - now.getTime()) / (24 * 3600 * 1000));

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <View style={styles.typeContainer}>
          <Text style={[styles.typeIcon, { color: typeConfig.color }]}>
            {typeConfig.icon}
          </Text>
          <Text style={styles.typeText}>{typeText}</Text>
        </View>
        <View style={[styles.statusBadge, { backgroundColor: appealStatus.color + '20' }]}>
          <Text style={[styles.statusText, { color: appealStatus.color }]}>
            {appealStatus.text}
          </Text>
        </View>
      </View>

      <View style={styles.amountRow}>
        <Text style={styles.amountLabel}>扣除金额</Text>
        <View style={styles.amountValue}>
          <Text style={styles.dustAmount}>
            -{MakerService.formatDustAmount(penalty.deductedAmount)} DUST
          </Text>
          <Text style={styles.usdAmount}>
            (${MakerService.formatUsdAmount(penalty.usdValue)})
          </Text>
        </View>
      </View>

      <View style={styles.timeRow}>
        <Text style={styles.timeLabel}>扣除时间</Text>
        <Text style={styles.timeValue}>
          {new Date(penalty.deductedAt * 1000).toLocaleString('zh-CN')}
        </Text>
      </View>

      <View style={styles.actions}>
        <TouchableOpacity style={styles.detailButton} onPress={onViewDetail}>
          <Text style={styles.detailButtonText}>查看详情</Text>
        </TouchableOpacity>

        {canAppeal && (
          <TouchableOpacity style={styles.appealButton} onPress={onAppeal}>
            <Text style={styles.appealButtonText}>
              申诉 ({daysLeft}天后截止)
            </Text>
          </TouchableOpacity>
        )}
      </View>
    </View>
  );
}

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
  typeContainer: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  typeIcon: {
    fontSize: 16,
    marginRight: 6,
  },
  typeText: {
    fontSize: 15,
    fontWeight: '500',
    color: '#1C1C1E',
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
  amountRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  amountLabel: {
    fontSize: 14,
    color: '#8E8E93',
  },
  amountValue: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  dustAmount: {
    fontSize: 15,
    fontWeight: '600',
    color: '#FF3B30',
  },
  usdAmount: {
    fontSize: 13,
    color: '#8E8E93',
    marginLeft: 4,
  },
  timeRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  timeLabel: {
    fontSize: 14,
    color: '#8E8E93',
  },
  timeValue: {
    fontSize: 14,
    color: '#1C1C1E',
  },
  actions: {
    flexDirection: 'row',
    justifyContent: 'flex-end',
    gap: 12,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#F2F2F7',
  },
  detailButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
  },
  detailButtonText: {
    fontSize: 14,
    color: '#007AFF',
  },
  appealButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    backgroundColor: '#B2955D',
    borderRadius: 6,
  },
  appealButtonText: {
    fontSize: 14,
    color: '#FFFFFF',
    fontWeight: '500',
  },
});
