/**
 * 放币超时提醒组件
 */

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  Animated,
} from 'react-native';

interface ReleaseTimeoutAlertProps {
  paidAt: number;  // 付款时间戳（毫秒）
  onDispute: () => void;
  onContactMaker: () => void;
}

// 超时阈值
const WARNING_THRESHOLD = 30 * 60 * 1000;  // 30 分钟显示警告
const TIMEOUT_THRESHOLD = 2 * 60 * 60 * 1000;  // 2 小时可申请仲裁

type AlertLevel = 'normal' | 'warning' | 'timeout';

export const ReleaseTimeoutAlert: React.FC<ReleaseTimeoutAlertProps> = ({
  paidAt,
  onDispute,
  onContactMaker,
}) => {
  const [elapsed, setElapsed] = useState(0);
  const [alertLevel, setAlertLevel] = useState<AlertLevel>('normal');
  const [pulseAnim] = useState(new Animated.Value(1));

  useEffect(() => {
    const updateElapsed = () => {
      const now = Date.now();
      const diff = now - paidAt;
      setElapsed(diff);

      // 更新警告级别
      if (diff >= TIMEOUT_THRESHOLD) {
        setAlertLevel('timeout');
      } else if (diff >= WARNING_THRESHOLD) {
        setAlertLevel('warning');
      } else {
        setAlertLevel('normal');
      }
    };

    updateElapsed();
    const interval = setInterval(updateElapsed, 1000);

    return () => clearInterval(interval);
  }, [paidAt]);

  // 脉冲动画（超时时）
  useEffect(() => {
    if (alertLevel === 'timeout') {
      const pulse = Animated.loop(
        Animated.sequence([
          Animated.timing(pulseAnim, {
            toValue: 1.05,
            duration: 500,
            useNativeDriver: true,
          }),
          Animated.timing(pulseAnim, {
            toValue: 1,
            duration: 500,
            useNativeDriver: true,
          }),
        ])
      );
      pulse.start();
      return () => pulse.stop();
    }
  }, [alertLevel, pulseAnim]);

  // 格式化等待时间
  const formatElapsed = (ms: number): string => {
    const minutes = Math.floor(ms / 60000);
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;

    if (hours > 0) {
      return `${hours} 小时 ${mins} 分钟`;
    }
    return `${mins} 分钟`;
  };

  // 正常状态不显示
  if (alertLevel === 'normal') {
    return null;
  }

  // 警告状态
  if (alertLevel === 'warning') {
    return (
      <View style={styles.section}>
        <View style={styles.warningCard}>
          <Text style={styles.warningIcon}>⏰</Text>
          <View style={styles.warningContent}>
            <Text style={styles.warningTitle}>等待时间较长</Text>
            <Text style={styles.warningText}>
              已等待 {formatElapsed(elapsed)}，做市商可能较忙
            </Text>
          </View>
          <TouchableOpacity
            style={styles.contactSmallButton}
            onPress={onContactMaker}
          >
            <Text style={styles.contactSmallButtonText}>联系</Text>
          </TouchableOpacity>
        </View>
      </View>
    );
  }

  // 超时状态
  return (
    <View style={styles.section}>
      <Animated.View
        style={[
          styles.timeoutCard,
          { transform: [{ scale: pulseAnim }] },
        ]}
      >
        <Text style={styles.timeoutIcon}>⚠️</Text>
        <Text style={styles.timeoutTitle}>做市商响应超时</Text>
        <Text style={styles.timeoutText}>
          您已付款超过 {formatElapsed(elapsed)}，做市商仍未释放 DUST。
        </Text>

        <View style={styles.timeoutOptions}>
          <Text style={styles.optionsTitle}>您可以：</Text>
          <Text style={styles.optionItem}>1. 继续等待做市商处理</Text>
          <Text style={styles.optionItem}>2. 联系做市商催促</Text>
          <Text style={styles.optionItem}>3. 申请平台仲裁</Text>
        </View>

        <View style={styles.timeoutActions}>
          <TouchableOpacity
            style={styles.contactButton}
            onPress={onContactMaker}
          >
            <Text style={styles.contactButtonText}>联系做市商</Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={styles.disputeButton}
            onPress={onDispute}
          >
            <Text style={styles.disputeButtonText}>申请仲裁</Text>
          </TouchableOpacity>
        </View>
      </Animated.View>
    </View>
  );
};

const styles = StyleSheet.create({
  section: {
    padding: 16,
    paddingBottom: 0,
  },
  // 警告样式
  warningCard: {
    backgroundColor: '#FFF9F0',
    borderRadius: 12,
    padding: 16,
    flexDirection: 'row',
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#FFE0B2',
  },
  warningIcon: {
    fontSize: 24,
    marginRight: 12,
  },
  warningContent: {
    flex: 1,
  },
  warningTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#FF9500',
    marginBottom: 2,
  },
  warningText: {
    fontSize: 12,
    color: '#666666',
  },
  contactSmallButton: {
    backgroundColor: '#B2955D',
    borderRadius: 6,
    paddingHorizontal: 12,
    paddingVertical: 6,
  },
  contactSmallButtonText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  // 超时样式
  timeoutCard: {
    backgroundColor: '#FFF3F3',
    borderRadius: 12,
    padding: 20,
    borderWidth: 2,
    borderColor: '#FF3B30',
  },
  timeoutIcon: {
    fontSize: 40,
    textAlign: 'center',
    marginBottom: 12,
  },
  timeoutTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#FF3B30',
    textAlign: 'center',
    marginBottom: 8,
  },
  timeoutText: {
    fontSize: 14,
    color: '#666666',
    textAlign: 'center',
    lineHeight: 20,
    marginBottom: 16,
  },
  timeoutOptions: {
    backgroundColor: '#FFFFFF',
    borderRadius: 8,
    padding: 12,
    marginBottom: 16,
  },
  optionsTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  optionItem: {
    fontSize: 13,
    color: '#666666',
    marginBottom: 4,
  },
  timeoutActions: {
    flexDirection: 'row',
    gap: 12,
  },
  contactButton: {
    flex: 1,
    backgroundColor: '#B2955D',
    borderRadius: 10,
    paddingVertical: 12,
    alignItems: 'center',
  },
  contactButtonText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  disputeButton: {
    flex: 1,
    backgroundColor: '#FF3B30',
    borderRadius: 10,
    paddingVertical: 12,
    alignItems: 'center',
  },
  disputeButtonText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#FFFFFF',
  },
});
