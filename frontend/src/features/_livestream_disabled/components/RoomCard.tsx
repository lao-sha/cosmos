// frontend/src/features/livestream/components/RoomCard.tsx

import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity, Image, Dimensions } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import type { LiveRoom } from '../types';
import { LiveRoomType } from '../types';

const { width: SCREEN_WIDTH } = Dimensions.get('window');
const CARD_WIDTH = (SCREEN_WIDTH - 36) / 2;
const IPFS_GATEWAY = process.env.EXPO_PUBLIC_IPFS_GATEWAY || 'https://ipfs.io/ipfs/';

interface RoomCardProps {
  room: LiveRoom;
  onPress: () => void;
}

export function RoomCard({ room, onPress }: RoomCardProps) {
  const formatViewers = (count: number): string => {
    if (count >= 10000) {
      return `${(count / 10000).toFixed(1)}万`;
    }
    if (count >= 1000) {
      return `${(count / 1000).toFixed(1)}k`;
    }
    return count.toString();
  };

  const coverUrl = room.coverCid
    ? `${IPFS_GATEWAY}${room.coverCid}`
    : 'https://via.placeholder.com/300x169/1A1A2E/666?text=LIVE';

  return (
    <TouchableOpacity style={styles.container} onPress={onPress} activeOpacity={0.8}>
      <View style={styles.coverContainer}>
        <Image source={{ uri: coverUrl }} style={styles.cover} />

        {/* 观众数 */}
        <View style={styles.viewerBadge}>
          <Ionicons name="people" size={12} color="#FFF" />
          <Text style={styles.viewerCount}>{formatViewers(room.currentViewers)}</Text>
        </View>

        {/* 付费标签 */}
        {room.roomType === LiveRoomType.Paid && (
          <View style={styles.paidBadge}>
            <Ionicons name="ticket" size={12} color="#FFD700" />
            <Text style={styles.paidText}>{room.ticketPrice}</Text>
          </View>
        )}

        {/* 连麦标签 */}
        {room.roomType === LiveRoomType.MultiHost && (
          <View style={styles.multiHostBadge}>
            <Ionicons name="people-circle" size={12} color="#FFF" />
            <Text style={styles.multiHostText}>连麦</Text>
          </View>
        )}

        {/* 直播中标签 */}
        <View style={styles.liveBadge}>
          <View style={styles.liveIndicator} />
          <Text style={styles.liveText}>直播中</Text>
        </View>
      </View>

      <View style={styles.infoContainer}>
        <View style={styles.hostRow}>
          <Image
            source={{ uri: room.hostAvatar || 'https://via.placeholder.com/32' }}
            style={styles.hostAvatar}
          />
          <Text style={styles.hostName} numberOfLines={1}>
            {room.hostName || room.host.slice(0, 8)}
          </Text>
        </View>
        <Text style={styles.title} numberOfLines={2}>
          {room.title}
        </Text>
      </View>
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  container: {
    width: CARD_WIDTH,
    marginBottom: 12,
    borderRadius: 12,
    backgroundColor: '#252540',
    overflow: 'hidden',
  },
  coverContainer: {
    width: '100%',
    aspectRatio: 16 / 9,
    position: 'relative',
  },
  cover: {
    width: '100%',
    height: '100%',
  },
  viewerBadge: {
    position: 'absolute',
    bottom: 8,
    left: 8,
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: 'rgba(0,0,0,0.6)',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 12,
    gap: 4,
  },
  viewerCount: {
    color: '#FFF',
    fontSize: 12,
  },
  paidBadge: {
    position: 'absolute',
    top: 8,
    right: 8,
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: 'rgba(0,0,0,0.6)',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 12,
    gap: 4,
  },
  paidText: {
    color: '#FFD700',
    fontSize: 12,
    fontWeight: 'bold',
  },
  multiHostBadge: {
    position: 'absolute',
    top: 8,
    right: 8,
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FF4757',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 12,
    gap: 4,
  },
  multiHostText: {
    color: '#FFF',
    fontSize: 12,
  },
  liveBadge: {
    position: 'absolute',
    top: 8,
    left: 8,
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FF4757',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 12,
    gap: 4,
  },
  liveIndicator: {
    width: 6,
    height: 6,
    borderRadius: 3,
    backgroundColor: '#FFF',
  },
  liveText: {
    color: '#FFF',
    fontSize: 10,
    fontWeight: 'bold',
  },
  infoContainer: {
    padding: 10,
  },
  hostRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 6,
  },
  hostAvatar: {
    width: 20,
    height: 20,
    borderRadius: 10,
    marginRight: 6,
  },
  hostName: {
    color: '#999',
    fontSize: 12,
    flex: 1,
  },
  title: {
    color: '#FFF',
    fontSize: 13,
    lineHeight: 18,
  },
});
