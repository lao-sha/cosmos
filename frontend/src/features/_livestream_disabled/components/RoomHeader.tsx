// frontend/src/features/livestream/components/RoomHeader.tsx

import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity, Image } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useRouter } from 'expo-router';
import type { LiveRoom } from '../types';

interface RoomHeaderProps {
  room: LiveRoom;
  viewerCount: number;
  onShare?: () => void;
}

export function RoomHeader({ room, viewerCount, onShare }: RoomHeaderProps) {
  const router = useRouter();

  const formatViewers = (count: number): string => {
    if (count >= 10000) {
      return `${(count / 10000).toFixed(1)}万`;
    }
    if (count >= 1000) {
      return `${(count / 1000).toFixed(1)}k`;
    }
    return count.toString();
  };

  return (
    <View style={styles.container}>
      <TouchableOpacity style={styles.backBtn} onPress={() => router.back()}>
        <Ionicons name="chevron-back" size={24} color="#FFF" />
      </TouchableOpacity>

      <View style={styles.hostInfo}>
        <Image
          source={{ uri: room.hostAvatar || 'https://via.placeholder.com/40' }}
          style={styles.hostAvatar}
        />
        <View style={styles.hostTextContainer}>
          <Text style={styles.hostName} numberOfLines={1}>
            {room.hostName || room.host.slice(0, 8)}
          </Text>
          <View style={styles.statusRow}>
            <View style={styles.liveIndicator} />
            <Text style={styles.statusText}>直播中</Text>
          </View>
        </View>
      </View>

      <View style={styles.rightSection}>
        <View style={styles.viewerBadge}>
          <Ionicons name="people" size={14} color="#FFF" />
          <Text style={styles.viewerCount}>{formatViewers(viewerCount)}</Text>
        </View>

        {onShare && (
          <TouchableOpacity style={styles.shareBtn} onPress={onShare}>
            <Ionicons name="share-outline" size={20} color="#FFF" />
          </TouchableOpacity>
        )}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 12,
    paddingVertical: 10,
    backgroundColor: 'rgba(0,0,0,0.3)',
  },
  backBtn: {
    padding: 4,
    marginRight: 8,
  },
  hostInfo: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
  },
  hostAvatar: {
    width: 40,
    height: 40,
    borderRadius: 20,
    borderWidth: 2,
    borderColor: '#FF4757',
  },
  hostTextContainer: {
    marginLeft: 10,
    flex: 1,
  },
  hostName: {
    color: '#FFF',
    fontSize: 14,
    fontWeight: '600',
  },
  statusRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 2,
  },
  liveIndicator: {
    width: 6,
    height: 6,
    borderRadius: 3,
    backgroundColor: '#FF4757',
    marginRight: 4,
  },
  statusText: {
    color: '#FF4757',
    fontSize: 12,
  },
  rightSection: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  viewerBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: 'rgba(255,255,255,0.2)',
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 16,
    gap: 4,
  },
  viewerCount: {
    color: '#FFF',
    fontSize: 13,
    fontWeight: '500',
  },
  shareBtn: {
    padding: 8,
  },
});
