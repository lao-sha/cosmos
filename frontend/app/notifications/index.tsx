import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    FlatList,
    Pressable,
    StyleSheet,
    Text,
    View,
} from 'react-native';

type NotificationType = 'message' | 'order' | 'system' | 'reward';

interface Notification {
  id: string;
  type: NotificationType;
  title: string;
  content: string;
  isRead: boolean;
  createdAt: string;
  link?: string;
}

const MOCK_NOTIFICATIONS: Notification[] = [
  {
    id: '1',
    type: 'message',
    title: '新消息',
    content: '张三 向你发送了一条消息',
    isRead: false,
    createdAt: '5分钟前',
    link: '/chat/123',
  },
  {
    id: '2',
    type: 'order',
    title: '订单状态更新',
    content: '你的占卜订单 #12345 已完成',
    isRead: false,
    createdAt: '1小时前',
    link: '/market/order/12345',
  },
  {
    id: '3',
    type: 'reward',
    title: '签到奖励',
    content: '恭喜获得 20 COS 签到奖励',
    isRead: true,
    createdAt: '今天 08:30',
  },
  {
    id: '4',
    type: 'system',
    title: '系统公告',
    content: '平台将于今晚 00:00-02:00 进行系统维护',
    isRead: true,
    createdAt: '昨天',
  },
  {
    id: '5',
    type: 'order',
    title: 'OTC订单提醒',
    content: '请在 30 分钟内完成付款，否则订单将自动取消',
    isRead: true,
    createdAt: '昨天',
    link: '/trading/order/67890',
  },
];

const TYPE_INFO: Record<NotificationType, { icon: string; color: string }> = {
  message: { icon: '💬', color: '#3b82f6' },
  order: { icon: '📦', color: '#f59e0b' },
  system: { icon: '📢', color: '#6b7280' },
  reward: { icon: '🎁', color: '#22c55e' },
};

export default function NotificationsScreen() {
  const router = useRouter();
  const [filter, setFilter] = useState<'all' | NotificationType>('all');
  const [notifications, setNotifications] = useState(MOCK_NOTIFICATIONS);

  const filteredNotifications = filter === 'all'
    ? notifications
    : notifications.filter((n) => n.type === filter);

  const unreadCount = notifications.filter((n) => !n.isRead).length;

  const handlePress = (notification: Notification) => {
    setNotifications((prev) =>
      prev.map((n) => (n.id === notification.id ? { ...n, isRead: true } : n))
    );
    if (notification.link) {
      router.push(notification.link as any);
    }
  };

  const handleMarkAllRead = () => {
    setNotifications((prev) => prev.map((n) => ({ ...n, isRead: true })));
  };

  const renderItem = ({ item }: { item: Notification }) => {
    const typeInfo = TYPE_INFO[item.type];

    return (
      <Pressable
        style={[styles.notificationItem, !item.isRead && styles.unread]}
        onPress={() => handlePress(item)}
      >
        <View style={[styles.iconContainer, { backgroundColor: `${typeInfo.color}15` }]}>
          <Text style={styles.icon}>{typeInfo.icon}</Text>
        </View>
        <View style={styles.content}>
          <View style={styles.titleRow}>
            <Text style={[styles.title, !item.isRead && styles.titleUnread]}>
              {item.title}
            </Text>
            {!item.isRead && <View style={styles.unreadDot} />}
          </View>
          <Text style={styles.contentText} numberOfLines={2}>
            {item.content}
          </Text>
          <Text style={styles.time}>{item.createdAt}</Text>
        </View>
      </Pressable>
    );
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>
          通知 {unreadCount > 0 && `(${unreadCount})`}
        </Text>
        <Pressable style={styles.markReadButton} onPress={handleMarkAllRead}>
          <Text style={styles.markReadText}>全部已读</Text>
        </Pressable>
      </View>

      <View style={styles.filterRow}>
        {(['all', 'message', 'order', 'reward', 'system'] as const).map((f) => (
          <Pressable
            key={f}
            style={[styles.filterButton, filter === f && styles.filterActive]}
            onPress={() => setFilter(f)}
          >
            <Text style={[styles.filterText, filter === f && styles.filterTextActive]}>
              {f === 'all' ? '全部' : 
               f === 'message' ? '消息' : 
               f === 'order' ? '订单' : 
               f === 'reward' ? '奖励' : '系统'}
            </Text>
          </Pressable>
        ))}
      </View>

      <FlatList
        data={filteredNotifications}
        renderItem={renderItem}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.list}
        showsVerticalScrollIndicator={false}
        ListEmptyComponent={
          <View style={styles.empty}>
            <Text style={styles.emptyEmoji}>🔔</Text>
            <Text style={styles.emptyText}>暂无通知</Text>
          </View>
        }
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#fff',
    paddingTop: 50,
    paddingBottom: 12,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  backButton: {
    padding: 4,
  },
  backText: {
    fontSize: 17,
    color: '#6D28D9',
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#1f2937',
  },
  markReadButton: {
    padding: 4,
  },
  markReadText: {
    fontSize: 14,
    color: '#6D28D9',
  },
  filterRow: {
    flexDirection: 'row',
    padding: 12,
    gap: 8,
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  filterButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
    backgroundColor: '#f3f4f6',
  },
  filterActive: {
    backgroundColor: '#6D28D9',
  },
  filterText: {
    fontSize: 13,
    color: '#6b7280',
  },
  filterTextActive: {
    color: '#fff',
    fontWeight: '600',
  },
  list: {
    padding: 16,
  },
  notificationItem: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    padding: 16,
    borderRadius: 12,
    marginBottom: 8,
  },
  unread: {
    backgroundColor: '#f5f3ff',
  },
  iconContainer: {
    width: 44,
    height: 44,
    borderRadius: 22,
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  icon: {
    fontSize: 20,
  },
  content: {
    flex: 1,
  },
  titleRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  title: {
    fontSize: 15,
    fontWeight: '500',
    color: '#1f2937',
  },
  titleUnread: {
    fontWeight: '600',
  },
  unreadDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: '#6D28D9',
  },
  contentText: {
    fontSize: 14,
    color: '#6b7280',
    marginTop: 4,
    lineHeight: 20,
  },
  time: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 6,
  },
  empty: {
    padding: 60,
    alignItems: 'center',
  },
  emptyEmoji: {
    fontSize: 48,
    marginBottom: 16,
  },
  emptyText: {
    fontSize: 16,
    color: '#9ca3af',
  },
});
