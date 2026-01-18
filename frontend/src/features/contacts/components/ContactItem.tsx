/**
 * 联系人列表项组件
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
import { FriendStatus, type Contact } from '../types';
import { getCidUrl } from '@/services/ipfs.service';

interface Props {
  contact: Contact;
  onPress: () => void;
  onLongPress?: () => void;
}

const FRIEND_STATUS_CONFIG = {
  [FriendStatus.Mutual]: { text: '好友', color: '#34C759' },
  [FriendStatus.OneWay]: { text: '已添加', color: '#8E8E93' },
  [FriendStatus.Pending]: { text: '待确认', color: '#FF9500' },
};

export function ContactItem({ contact, onPress, onLongPress }: Props) {
  const statusConfig = FRIEND_STATUS_CONFIG[contact.friendStatus];
  const avatarUri = contact.profile?.avatarCid
    ? getCidUrl(contact.profile.avatarCid)
    : null;
  const displayName =
    contact.alias ||
    contact.profile?.nickname ||
    `${contact.address.slice(0, 8)}...`;

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
          <Text style={styles.avatarText}>
            {displayName.slice(0, 2).toUpperCase()}
          </Text>
        )}
      </View>

      <View style={styles.info}>
        <View style={styles.nameRow}>
          <Text style={styles.name} numberOfLines={1}>
            {displayName}
          </Text>
          <View
            style={[styles.friendBadge, { backgroundColor: statusConfig.color }]}
          >
            <Text style={styles.friendBadgeText}>{statusConfig.text}</Text>
          </View>
        </View>

        {contact.chatUserId && (
          <Text style={styles.chatId}>ID: {contact.chatUserId}</Text>
        )}

        {contact.groups.length > 0 && (
          <Text style={styles.groups} numberOfLines={1}>
            {contact.groups.join(', ')}
          </Text>
        )}
      </View>

      <Ionicons name="chevron-forward" size={20} color="#ccc" />
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
    backgroundColor: '#007AFF',
    justifyContent: 'center',
    alignItems: 'center',
    overflow: 'hidden',
  },
  avatarImage: {
    width: 44,
    height: 44,
  },
  avatarText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  info: {
    flex: 1,
    marginLeft: 12,
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  name: {
    fontSize: 16,
    fontWeight: '500',
    color: '#333',
    flex: 1,
  },
  friendBadge: {
    marginLeft: 8,
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
  },
  friendBadgeText: {
    fontSize: 10,
    color: '#fff',
  },
  chatId: {
    fontSize: 12,
    color: '#999',
    marginTop: 2,
  },
  groups: {
    fontSize: 13,
    color: '#666',
    marginTop: 2,
  },
});
