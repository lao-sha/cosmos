/**
 * 分组列表项组件
 */

import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import type { ContactGroup } from '../types';

interface Props {
  group: ContactGroup;
  onPress: () => void;
  onLongPress?: () => void;
}

export function GroupItem({ group, onPress, onLongPress }: Props) {
  return (
    <TouchableOpacity
      style={styles.container}
      onPress={onPress}
      onLongPress={onLongPress}
    >
      <View style={styles.icon}>
        <Ionicons name="folder-outline" size={24} color="#007AFF" />
      </View>

      <View style={styles.info}>
        <Text style={styles.name} numberOfLines={1}>
          {group.name}
        </Text>
        <Text style={styles.count}>{group.memberCount} 位联系人</Text>
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
  icon: {
    width: 44,
    height: 44,
    borderRadius: 8,
    backgroundColor: '#E3F2FD',
    justifyContent: 'center',
    alignItems: 'center',
  },
  info: {
    flex: 1,
    marginLeft: 12,
  },
  name: {
    fontSize: 16,
    fontWeight: '500',
    color: '#333',
  },
  count: {
    fontSize: 13,
    color: '#999',
    marginTop: 2,
  },
});
