import React from 'react';
import { View, Text, StyleSheet, FlatList, TouchableOpacity } from 'react-native';
import { useRouter } from 'expo-router';
import { useColors } from '@/hooks/useColors';
import { Colors } from '@/constants/colors';

interface Conversation {
  id: string;
  name: string;
  lastMessage: string;
  time: string;
  unread: number;
  avatar: string;
}

const mockConversations: Conversation[] = [
  {
    id: '1',
    name: '系统通知',
    lastMessage: '欢迎使用 COSMOS',
    time: '刚刚',
    unread: 1,
    avatar: 'S',
  },
];

export default function ChatScreen() {
  const colors = useColors();
  const router = useRouter();

  const renderItem = ({ item }: { item: Conversation }) => (
    <TouchableOpacity
      style={[styles.item, { borderBottomColor: colors.border }]}
      onPress={() => router.push(`/chat/${item.id}` as any)}
      activeOpacity={0.7}
    >
      <View style={[styles.avatar, { backgroundColor: Colors.primary }]}>
        <Text style={styles.avatarText}>{item.avatar}</Text>
      </View>
      <View style={styles.content}>
        <View style={styles.header}>
          <Text style={[styles.name, { color: colors.textPrimary }]}>
            {item.name}
          </Text>
          <Text style={[styles.time, { color: colors.textTertiary }]}>
            {item.time}
          </Text>
        </View>
        <View style={styles.footer}>
          <Text
            style={[styles.message, { color: colors.textSecondary }]}
            numberOfLines={1}
          >
            {item.lastMessage}
          </Text>
          {item.unread > 0 && (
            <View style={[styles.badge, { backgroundColor: Colors.error }]}>
              <Text style={styles.badgeText}>{item.unread}</Text>
            </View>
          )}
        </View>
      </View>
    </TouchableOpacity>
  );

  return (
    <View style={[styles.container, { backgroundColor: colors.background }]}>
      <FlatList
        data={mockConversations}
        renderItem={renderItem}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.list}
        ListEmptyComponent={
          <View style={styles.empty}>
            <Text style={[styles.emptyText, { color: colors.textSecondary }]}>
              暂无消息
            </Text>
          </View>
        }
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  list: {
    flexGrow: 1,
  },
  item: {
    flexDirection: 'row',
    padding: 16,
    borderBottomWidth: 1,
  },
  avatar: {
    width: 50,
    height: 50,
    borderRadius: 25,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  avatarText: {
    color: '#FFFFFF',
    fontSize: 20,
    fontWeight: '600',
  },
  content: {
    flex: 1,
    justifyContent: 'center',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 4,
  },
  name: {
    fontSize: 16,
    fontWeight: '600',
  },
  time: {
    fontSize: 12,
  },
  footer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  message: {
    fontSize: 14,
    flex: 1,
    marginRight: 8,
  },
  badge: {
    minWidth: 20,
    height: 20,
    borderRadius: 10,
    alignItems: 'center',
    justifyContent: 'center',
    paddingHorizontal: 6,
  },
  badgeText: {
    color: '#FFFFFF',
    fontSize: 12,
    fontWeight: '600',
  },
  empty: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    paddingTop: 100,
  },
  emptyText: {
    fontSize: 16,
  },
});
