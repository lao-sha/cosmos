/**
 * 聊天头部组件
 */

import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet, Image } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { UserStatus } from '../types';
import type { ChatUserProfile } from '../types';
import { getCidUrl } from '@/services/ipfs.service';

interface Props {
  title: string;
  subtitle?: string;
  profile?: ChatUserProfile;
  onBack: () => void;
  onMore?: () => void;
  onProfilePress?: () => void;
}

const STATUS_COLORS: Record<string, string> = {
  'Online': '#34C759',
  'Offline': '#8E8E93',
  'Busy': '#FF3B30',
  'Away': '#FF9500',
  'Invisible': '#8E8E93',
};

const STATUS_TEXT: Record<string, string> = {
  'Online': '在线',
  'Offline': '离线',
  'Busy': '忙碌',
  'Away': '离开',
  'Invisible': '隐身',
};

export function ChatHeader({
  title,
  subtitle,
  profile,
  onBack,
  onMore,
  onProfilePress,
}: Props) {
  const insets = useSafeAreaInsets();

  const avatarUri = profile?.avatarCid ? getCidUrl(profile.avatarCid) : null;
  const avatarText = (profile?.nickname || title).slice(0, 2).toUpperCase();
  const statusColor = profile?.status
    ? STATUS_COLORS[profile.status]
    : undefined;
  const statusText = profile?.status ? STATUS_TEXT[profile.status] : subtitle;

  return (
    <View style={[styles.container, { paddingTop: insets.top }]}>
      <TouchableOpacity style={styles.backBtn} onPress={onBack}>
        <Ionicons name="chevron-back" size={28} color="#007AFF" />
      </TouchableOpacity>

      <TouchableOpacity
        style={styles.titleContainer}
        onPress={onProfilePress}
        disabled={!onProfilePress}
      >
        <View style={styles.avatar}>
          {avatarUri ? (
            <Image source={{ uri: avatarUri }} style={styles.avatarImage} />
          ) : (
            <Text style={styles.avatarText}>{avatarText}</Text>
          )}
          {statusColor && profile?.privacySettings?.showOnlineStatus && (
            <View
              style={[styles.statusDot, { backgroundColor: statusColor }]}
            />
          )}
        </View>

        <View style={styles.titleInfo}>
          <Text style={styles.title} numberOfLines={1}>
            {profile?.nickname || title}
          </Text>
          {statusText && (
            <Text style={styles.subtitle} numberOfLines={1}>
              {statusText}
            </Text>
          )}
        </View>
      </TouchableOpacity>

      {onMore && (
        <TouchableOpacity style={styles.moreBtn} onPress={onMore}>
          <Ionicons name="ellipsis-horizontal" size={24} color="#007AFF" />
        </TouchableOpacity>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#fff',
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#e0e0e0',
    paddingBottom: 10,
  },
  backBtn: {
    padding: 8,
    marginLeft: 4,
  },
  titleContainer: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 8,
  },
  avatar: {
    width: 36,
    height: 36,
    borderRadius: 18,
    backgroundColor: '#007AFF',
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 10,
    overflow: 'hidden',
  },
  avatarImage: {
    width: 36,
    height: 36,
  },
  avatarText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
  },
  statusDot: {
    position: 'absolute',
    bottom: 0,
    right: 0,
    width: 10,
    height: 10,
    borderRadius: 5,
    borderWidth: 2,
    borderColor: '#fff',
  },
  titleInfo: {
    flex: 1,
  },
  title: {
    fontSize: 17,
    fontWeight: '600',
    color: '#333',
  },
  subtitle: {
    fontSize: 12,
    color: '#999',
    marginTop: 2,
  },
  moreBtn: {
    padding: 8,
    marginRight: 4,
  },
});
