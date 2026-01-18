/**
 * 好友申请项组件
 */

import React from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  Image,
} from 'react-native';
import { useBlockTime, BlockTimeUtils } from '@/hooks/useBlockTime';
import type { FriendRequest } from '../types';
import { getCidUrl } from '@/services/ipfs.service';

interface Props {
  request: FriendRequest;
  onAccept: () => void;
  onReject: () => void;
  onPress?: () => void;
}

export function FriendRequestItem({
  request,
  onAccept,
  onReject,
  onPress,
}: Props) {
  const { currentBlock, blockTime } = useBlockTime();

  const timeAgo = BlockTimeUtils.formatRelative(
    request.requestedAt,
    currentBlock,
    blockTime
  );

  const blocksUntilExpiry = request.expiresAt - currentBlock;
  const isExpiringSoon = blocksUntilExpiry > 0 && blocksUntilExpiry < 14400; // 24小时内过期

  const avatarUri = request.profile?.avatarCid
    ? getCidUrl(request.profile.avatarCid)
    : null;
  const displayName =
    request.profile?.nickname ||
    (request.requesterChatId
      ? `ID: ${request.requesterChatId}`
      : `${request.requester.slice(0, 12)}...`);

  return (
    <TouchableOpacity
      style={styles.container}
      onPress={onPress}
      disabled={!onPress}
    >
      <View style={styles.avatar}>
        {avatarUri ? (
          <Image source={{ uri: avatarUri }} style={styles.avatarImage} />
        ) : (
          <Text style={styles.avatarText}>
            {displayName.slice(0, 2).toUpperCase()}
          </Text>
        )}
      </View>

      <View style={styles.content}>
        <Text style={styles.name} numberOfLines={1}>
          {displayName}
        </Text>

        {request.message && (
          <Text style={styles.message} numberOfLines={2}>
            {request.message}
          </Text>
        )}

        <Text style={[styles.time, isExpiringSoon && styles.expiring]}>
          {timeAgo}
          {isExpiringSoon && ' · 即将过期'}
          {request.isExpired && ' · 已过期'}
        </Text>
      </View>

      {!request.isExpired && (
        <View style={styles.actions}>
          <TouchableOpacity style={styles.acceptBtn} onPress={onAccept}>
            <Text style={styles.acceptText}>接受</Text>
          </TouchableOpacity>
          <TouchableOpacity style={styles.rejectBtn} onPress={onReject}>
            <Text style={styles.rejectText}>拒绝</Text>
          </TouchableOpacity>
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
    backgroundColor: '#FF9500',
    justifyContent: 'center',
    alignItems: 'center',
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
    marginLeft: 12,
  },
  name: {
    fontSize: 16,
    fontWeight: '500',
    color: '#333',
  },
  message: {
    fontSize: 14,
    color: '#666',
    marginTop: 4,
  },
  time: {
    fontSize: 12,
    color: '#999',
    marginTop: 4,
  },
  expiring: {
    color: '#FF9500',
  },
  actions: {
    flexDirection: 'row',
    gap: 8,
  },
  acceptBtn: {
    backgroundColor: '#007AFF',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 6,
  },
  acceptText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
  rejectBtn: {
    backgroundColor: '#f5f5f5',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 6,
  },
  rejectText: {
    color: '#666',
    fontSize: 14,
  },
});
