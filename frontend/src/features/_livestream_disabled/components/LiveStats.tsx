// frontend/src/features/livestream/components/LiveStats.tsx

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { Ionicons } from '@expo/vector-icons';

interface LiveStatsProps {
  viewerCount: number;
  earnings: string;
  duration: number; // 秒
}

export function LiveStats({ viewerCount, earnings, duration }: LiveStatsProps) {
  const formatDuration = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;

    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
    return `${minutes}:${secs.toString().padStart(2, '0')}`;
  };

  const formatViewers = (count: number): string => {
    if (count >= 10000) {
      return `${(count / 10000).toFixed(1)}万`;
    }
    if (count >= 1000) {
      return `${(count / 1000).toFixed(1)}k`;
    }
    return count.toString();
  };

  const formatEarnings = (value: string): string => {
    const num = Number(value);
    if (num >= 10000) {
      return `${(num / 10000).toFixed(1)}万`;
    }
    if (num >= 1000) {
      return `${(num / 1000).toFixed(1)}k`;
    }
    return value;
  };

  return (
    <View style={styles.container}>
      <View style={styles.statItem}>
        <Ionicons name="people" size={16} color="#FFF" />
        <Text style={styles.statValue}>{formatViewers(viewerCount)}</Text>
      </View>

      <View style={styles.divider} />

      <View style={styles.statItem}>
        <Ionicons name="gift" size={16} color="#FFD700" />
        <Text style={[styles.statValue, styles.earningsValue]}>
          {formatEarnings(earnings)} DUST
        </Text>
      </View>

      <View style={styles.divider} />

      <View style={styles.statItem}>
        <Ionicons name="time" size={16} color="#FFF" />
        <Text style={styles.statValue}>{formatDuration(duration)}</Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 12,
    paddingHorizontal: 20,
    backgroundColor: 'rgba(0,0,0,0.5)',
    gap: 16,
  },
  statItem: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
  },
  statValue: {
    color: '#FFF',
    fontSize: 14,
    fontWeight: '500',
  },
  earningsValue: {
    color: '#FFD700',
  },
  divider: {
    width: 1,
    height: 16,
    backgroundColor: 'rgba(255,255,255,0.3)',
  },
});
