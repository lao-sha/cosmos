/**
 * 倒计时组件
 */

import React, { useState, useEffect } from 'react';
import { Text, StyleSheet } from 'react-native';

interface CountdownTimerProps {
  expireAt: number; // 毫秒时间戳
  onExpire?: () => void;
  style?: any;
}

export const CountdownTimer: React.FC<CountdownTimerProps> = ({
  expireAt,
  onExpire,
  style,
}) => {
  const [timeLeft, setTimeLeft] = useState<string>('');
  const [isExpired, setIsExpired] = useState(false);

  useEffect(() => {
    const updateTimer = () => {
      const now = Date.now();
      const diff = expireAt - now;

      if (diff <= 0) {
        setTimeLeft('已超时');
        setIsExpired(true);
        if (onExpire && !isExpired) {
          onExpire();
        }
        return;
      }

      const minutes = Math.floor(diff / 60000);
      const seconds = Math.floor((diff % 60000) / 1000);
      setTimeLeft(`${minutes}:${seconds.toString().padStart(2, '0')}`);
    };

    updateTimer();
    const interval = setInterval(updateTimer, 1000);

    return () => clearInterval(interval);
  }, [expireAt, onExpire, isExpired]);

  return (
    <Text style={[styles.timer, isExpired && styles.expired, style]}>
      {isExpired ? '⏰ ' : '⏱️ '}
      {timeLeft}
    </Text>
  );
};

const styles = StyleSheet.create({
  timer: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FF9500',
  },
  expired: {
    color: '#FF3B30',
  },
});
