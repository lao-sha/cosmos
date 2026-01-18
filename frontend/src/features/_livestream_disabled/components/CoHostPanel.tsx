// frontend/src/features/livestream/components/CoHostPanel.tsx

import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
  Image,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import type { CoHostRequest } from '../types';

interface CoHostPanelProps {
  requests: CoHostRequest[];
  onAccept: (address: string) => void;
  onReject: (address: string) => void;
}

export function CoHostPanel({ requests, onAccept, onReject }: CoHostPanelProps) {
  const renderRequest = ({ item }: { item: CoHostRequest }) => (
    <View style={styles.requestItem}>
      <Image
        source={{ uri: 'https://via.placeholder.com/40' }}
        style={styles.avatar}
      />
      <View style={styles.requestInfo}>
        <Text style={styles.requestName}>
          {item.name || item.address.slice(0, 8)}
        </Text>
        <Text style={styles.requestType}>
          申请{item.type === 'video' ? '视频' : '语音'}连麦
        </Text>
      </View>
      <View style={styles.actions}>
        <TouchableOpacity
          style={styles.acceptBtn}
          onPress={() => onAccept(item.address)}
        >
          <Ionicons name="checkmark" size={20} color="#FFF" />
        </TouchableOpacity>
        <TouchableOpacity
          style={styles.rejectBtn}
          onPress={() => onReject(item.address)}
        >
          <Ionicons name="close" size={20} color="#FFF" />
        </TouchableOpacity>
      </View>
    </View>
  );

  if (requests.length === 0) {
    return (
      <View style={styles.emptyContainer}>
        <Ionicons name="people-outline" size={48} color="#666" />
        <Text style={styles.emptyText}>暂无连麦申请</Text>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>连麦申请</Text>
        <View style={styles.badge}>
          <Text style={styles.badgeText}>{requests.length}</Text>
        </View>
      </View>
      <FlatList
        data={requests}
        renderItem={renderRequest}
        keyExtractor={(item) => item.address}
        style={styles.list}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#1A1A2E',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#333',
  },
  title: {
    color: '#FFF',
    fontSize: 16,
    fontWeight: '600',
  },
  badge: {
    backgroundColor: '#FF4757',
    borderRadius: 10,
    paddingHorizontal: 8,
    paddingVertical: 2,
    marginLeft: 8,
  },
  badgeText: {
    color: '#FFF',
    fontSize: 12,
    fontWeight: 'bold',
  },
  list: {
    flex: 1,
  },
  requestItem: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#333',
  },
  avatar: {
    width: 40,
    height: 40,
    borderRadius: 20,
  },
  requestInfo: {
    flex: 1,
    marginLeft: 12,
  },
  requestName: {
    color: '#FFF',
    fontSize: 14,
    fontWeight: '500',
  },
  requestType: {
    color: '#999',
    fontSize: 12,
    marginTop: 2,
  },
  actions: {
    flexDirection: 'row',
    gap: 8,
  },
  acceptBtn: {
    width: 36,
    height: 36,
    borderRadius: 18,
    backgroundColor: '#4CAF50',
    justifyContent: 'center',
    alignItems: 'center',
  },
  rejectBtn: {
    width: 36,
    height: 36,
    borderRadius: 18,
    backgroundColor: '#FF4757',
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 40,
  },
  emptyText: {
    color: '#666',
    fontSize: 14,
    marginTop: 12,
  },
});
