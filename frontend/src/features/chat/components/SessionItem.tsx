/**
 * 会话列表项组件
 */

import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet, Image } from 'react-native';
import type { Session } from '../types';
import { useBlockTime } from '@/hooks/useBlockTime';
import { getCidUrl } from '@/services/ipfs.service';

interface Props {
  session: Session;
  onPress: () => void;
  onLongPress?: () => void;
}

/**
 * 将区块高度转换为相对时间显示
 */
function formatBlockTime(
  targetBlock: number,
  currentBlock: number,
  blockTime: number = 6000
): string {
  const blockDiff = currentBlock - targetBlock;
  const timeDiff = blockDiff * blockTime;

  if (timeDiff < 0) return '刚刚';
  if (timeDiff < 60000) return '刚刚';
  if (timeDiff < 3600000) return `${Math.floor(timeDiff / 60000)}分钟前`;
  if (timeDiff < 86400000) return `${Math.floor(timeDiff / 3600000)}小时前`;
  if (timeDiff < 604800000) return `${Math.floor(timeDiff / 86400000)}天前`;

  const date = new Date(Date.now() - timeDiff);
  return `${date.getMonth() + 1}/${date.getDate()}`;
}

export function SessionItem({ session, onPress, onLongPress }: Props) {
  const { currentBlock, blockTime } = useBlockTime();

  const displayName =
    session.peerAlias ||
    session.peerProfile?.nickname ||
    (session.peerChatId ? `ID: ${session.peerChatId}` : null) ||
    `${session.peerAddress.slice(0, 8)}...${session.peerAddress.slice(-4)}`;

  const avatarUri = session.peerProfile?.avatarCid
    ? getCidUrl(session.peerProfile.avatarCid)
    : null;

  const avatarText = (
    session.peerAlias ||
    session.peerProfile?.nickname ||
    session.peerAddress
  )
    .slice(0, 2)
    .toUpperCase();

  return (
    <TouchableOpacity
      style={styles.container}
      onPress={onPress}
      onLongPress={onLongPress}
    >
      <View style={styles.avatar}>
        {avatarUri ? (
          <Image source={{ uri: avatarUri }} style={styles.avatarImage} />
        ) : (
          <Text style={styles.avatarText}>{avatarText}</Text>
        )}
      </View>

      <View style={styles.content}>
        <View style={styles.header}>
          <Text style={styles.name} numberOfLines={1}>
            {displayName}
          </Text>
          <Text style={styles.time}>
            {formatBlockTime(session.lastActive, currentBlock, blockTime)}
          </Text>
        </View>

        <Text style={styles.preview} numberOfLines={1}>
          {session.lastMessage?.content || '暂无消息'}
        </Text>
      </View>

      {session.unreadCount > 0 && (
        <View style={styles.badge}>
          <Text style={styles.badgeText}>
            {session.unreadCount > 99 ? '99+' : session.unreadCount}
          </Text>
        </View>
      )}
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
    backgroundColor: '#fff',
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#e0e0e0',
  },
  avatar: {
    width: 48,
    height: 48,
    borderRadius: 24,
    backgroundColor: '#007AFF',
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
    overflow: 'hidden',
  },
  avatarImage: {
    width: 48,
    height: 48,
  },
  avatarText: {
    color: '#fff',
    fontSize: 18,
    fontWeight: '600',
  },
  content: {
    flex: 1,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 4,
  },
  name: {
    fontSize: 16,
    fontWeight: '500',
    color: '#333',
    flex: 1,
    marginRight: 8,
  },
  time: {
    fontSize: 12,
    color: '#999',
  },
  preview: {
    fontSize: 14,
    color: '#666',
  },
  badge: {
    minWidth: 20,
    height: 20,
    borderRadius: 10,
    backgroundColor: '#FF3B30',
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 6,
    marginLeft: 8,
  },
  badgeText: {
    color: '#fff',
    fontSize: 12,
    fontWeight: '600',
  },
});
