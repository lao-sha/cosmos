/**
 * 黑名单用户项组件
 */

import React from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  Image,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useBlockTime, BlockTimeUtils } from '@/hooks/useBlockTime';
import type { BlockedUser } from '../types';
import { getCidUrl } from '@/services/ipfs.service';

interface Props {
  blockedUser: BlockedUser;
  onUnblock: () => void;
  onPress?: () => void;
}

export function BlockedUserItem({ blockedUser, onUnblock, onPress }: Props) {
  const { currentBlock, blockTime } = useBlockTime();

  const blockedTimeAgo = BlockTimeUtils.formatRelative(
    blockedUser.blockedAt,
    currentBlock,
    blockTime
  );

  const avatarUri = blockedUser.profile?.avatarCid
    ? getCidUrl(blockedUser.profile.avatarCid)
    : null;
  const displayName =
    blockedUser.profile?.nickname ||
    (blockedUser.chatUserId
      ? `ID: ${blockedUser.chatUserId}`
      : `${blockedUser.address.slice(0, 12)}...`);

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
        <View style={styles.blockedBadge}>
          <Ionicons name="ban" size={12} color="#fff" />
        </View>
      </View>

      <View style={styles.content}>
        <Text style={styles.name} numberOfLines={1}>
          {displayName}
        </Text>

        {blockedUser.reason && (
          <Text style={styles.reason} numberOfLines={1}>
            原因: {blockedUser.reason}
          </Text>
        )}

        <Text style={styles.time}>拉黑于 {blockedTimeAgo}</Text>
      </View>

      <TouchableOpacity style={styles.unblockBtn} onPress={onUnblock}>
        <Text style={styles.unblockText}>解除</Text>
      </TouchableOpacity>
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
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: '#8E8E93',
    justifyContent: 'center',
    alignItems: 'center',
    overflow: 'hidden',
    position: 'relative',
  },
  avatarImage: {
    width: 44,
    height: 44,
    opacity: 0.5,
  },
  avatarText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  blockedBadge: {
    position: 'absolute',
    bottom: -2,
    right: -2,
    width: 18,
    height: 18,
    borderRadius: 9,
    backgroundColor: '#FF3B30',
    justifyContent: 'center',
    alignItems: 'center',
    borderWidth: 2,
    borderColor: '#fff',
  },
  content: {
    flex: 1,
    marginLeft: 12,
  },
  name: {
    fontSize: 16,
    fontWeight: '500',
    color: '#666',
  },
  reason: {
    fontSize: 13,
    color: '#999',
    marginTop: 2,
  },
  time: {
    fontSize: 12,
    color: '#999',
    marginTop: 2,
  },
  unblockBtn: {
    backgroundColor: '#f5f5f5',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 6,
  },
  unblockText: {
    color: '#007AFF',
    fontSize: 14,
    fontWeight: '500',
  },
});
