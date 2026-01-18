/**
 * 提现进度组件
 */

import React, { useState, useEffect } from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { WithdrawalRequest, WithdrawalStatus, MakerService } from '@/services/maker.service';

interface WithdrawalProgressProps {
  request: WithdrawalRequest;
}

export function WithdrawalProgress({ request }: WithdrawalProgressProps) {
  const [timeLeft, setTimeLeft] = useState('');
  const [canExecute, setCanExecute] = useState(false);

  useEffect(() => {
    const updateCountdown = () => {
      const now = Math.floor(Date.now() / 1000);
      const remaining = request.executableAt - now;

      if (remaining <= 0) {
        setTimeLeft('可执行');
        setCanExecute(true);
        return;
      }

      setCanExecute(false);

      const days = Math.floor(remaining / 86400);
      const hours = Math.floor((remaining % 86400) / 3600);
      const minutes = Math.floor((remaining % 3600) / 60);
      const seconds = remaining % 60;

      if (days > 0) {
        setTimeLeft(`${days}天 ${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`);
      } else {
        setTimeLeft(`${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`);
      }
    };

    updateCountdown();
    const interval = setInterval(updateCountdown, 1000);

    return () => clearInterval(interval);
  }, [request.executableAt]);

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>提现申请</Text>
        <View style={[styles.statusBadge, canExecute ? styles.statusReady : styles.statusPending]}>
          <Text style={[styles.statusText, canExecute ? styles.statusTextReady : styles.statusTextPending]}>
            {canExecute ? '✅ 可执行' : '⏳ 冷却期中'}
          </Text>
        </View>
      </View>

      <View style={styles.amountContainer}>
        <Text style={styles.amountLabel}>提现金额</Text>
        <Text style={styles.amountValue}>
          {MakerService.formatDustAmount(request.amount)} DUST
        </Text>
      </View>

      <View style={styles.countdownContainer}>
        <Text style={styles.countdownLabel}>
          {canExecute ? '已可执行' : '冷却期倒计时'}
        </Text>
        <Text style={[styles.countdownValue, canExecute && styles.countdownReady]}>
          {timeLeft}
        </Text>
      </View>

      <View style={styles.timeline}>
        <View style={styles.timelineItem}>
          <View style={[styles.timelineDot, styles.timelineDotCompleted]} />
          <View style={styles.timelineContent}>
            <Text style={styles.timelineTitle}>申请已提交</Text>
            <Text style={styles.timelineTime}>{formatDate(request.requestedAt)}</Text>
          </View>
        </View>

        <View style={styles.timelineLine} />

        <View style={styles.timelineItem}>
          <View style={[styles.timelineDot, canExecute ? styles.timelineDotCompleted : styles.timelineDotPending]} />
          <View style={styles.timelineContent}>
            <Text style={styles.timelineTitle}>冷却期 (7天)</Text>
            <Text style={styles.timelineTime}>{canExecute ? '已完成' : '进行中'}</Text>
          </View>
        </View>

        <View style={styles.timelineLine} />

        <View style={styles.timelineItem}>
          <View style={[styles.timelineDot, canExecute ? styles.timelineDotReady : styles.timelineDotFuture]} />
          <View style={styles.timelineContent}>
            <Text style={styles.timelineTitle}>可执行提现</Text>
            <Text style={styles.timelineTime}>{formatDate(request.executableAt)}</Text>
          </View>
        </View>
      </View>
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
    marginBottom: 16,
  },
  title: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1C1C1E',
  },
  statusBadge: {
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 6,
  },
  statusPending: {
    backgroundColor: '#007AFF20',
  },
  statusReady: {
    backgroundColor: '#4CD96420',
  },
  statusText: {
    fontSize: 12,
    fontWeight: '500',
  },
  statusTextPending: {
    color: '#007AFF',
  },
  statusTextReady: {
    color: '#4CD964',
  },
  amountContainer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
    paddingBottom: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#F2F2F7',
  },
  amountLabel: {
    fontSize: 14,
    color: '#8E8E93',
  },
  amountValue: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1C1C1E',
  },
  countdownContainer: {
    alignItems: 'center',
    marginBottom: 20,
  },
  countdownLabel: {
    fontSize: 14,
    color: '#8E8E93',
    marginBottom: 8,
  },
  countdownValue: {
    fontSize: 32,
    fontWeight: '700',
    color: '#1C1C1E',
    fontVariant: ['tabular-nums'],
  },
  countdownReady: {
    color: '#4CD964',
  },
  timeline: {
    paddingLeft: 8,
  },
  timelineItem: {
    flexDirection: 'row',
    alignItems: 'flex-start',
  },
  timelineDot: {
    width: 12,
    height: 12,
    borderRadius: 6,
    marginRight: 12,
    marginTop: 4,
  },
  timelineDotCompleted: {
    backgroundColor: '#4CD964',
  },
  timelineDotPending: {
    backgroundColor: '#007AFF',
  },
  timelineDotReady: {
    backgroundColor: '#4CD964',
  },
  timelineDotFuture: {
    backgroundColor: '#E5E5EA',
  },
  timelineLine: {
    width: 2,
    height: 24,
    backgroundColor: '#E5E5EA',
    marginLeft: 5,
    marginVertical: 4,
  },
  timelineContent: {
    flex: 1,
  },
  timelineTitle: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1C1C1E',
    marginBottom: 2,
  },
  timelineTime: {
    fontSize: 12,
    color: '#8E8E93',
  },
});
