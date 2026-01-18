/**
 * 星尘玄鉴 - 消息页（会话列表）
 * 主题色：金棕色 #B2955D
 */

import { View, Text, StyleSheet, ScrollView, Pressable } from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';

const THEME_COLOR = '#B2955D';
const THEME_BG = '#F5F5F7';

// 模拟会话数据
const MOCK_SESSIONS = [
  {
    id: '1',
    name: '玄机子',
    avatar: '🧙',
    lastMessage: '您的八字排盘已完成，请查看结果',
    time: '刚刚',
    unread: 2,
    online: true,
  },
  {
    id: '2',
    name: '易道人',
    avatar: '👴',
    lastMessage: '好的，我会尽快为您解答',
    time: '10分钟前',
    unread: 0,
    online: true,
  },
  {
    id: '3',
    name: '星月师',
    avatar: '🌙',
    lastMessage: '塔罗牌显示您近期会有好运',
    time: '1小时前',
    unread: 0,
    online: false,
  },
  {
    id: '4',
    name: '天机阁',
    avatar: '🏛️',
    lastMessage: '奇门遁甲排盘需要您提供具体时间',
    time: '昨天',
    unread: 1,
    online: false,
  },
];

export default function ChatPage() {
  const router = useRouter();

  const handleSessionPress = (sessionId: string) => {
    // TODO: 跳转到聊天详情页
    console.log('Open chat:', sessionId);
  };

  return (
    <View style={styles.container}>
      {/* 顶部标题 */}
      <View style={styles.header}>
        <Text style={styles.title}>消息</Text>
        <View style={styles.headerRight}>
          <Pressable 
            style={styles.headerButton}
            onPress={() => router.push('/contacts' as any)}
          >
            <Ionicons name="people-outline" size={24} color={THEME_COLOR} />
          </Pressable>
          <Pressable style={styles.headerButton}>
            <Ionicons name="create-outline" size={24} color={THEME_COLOR} />
          </Pressable>
        </View>
      </View>

      {/* 搜索栏 */}
      <View style={styles.searchSection}>
        <View style={styles.searchBar}>
          <Ionicons name="search" size={18} color="#999" />
          <Text style={styles.searchPlaceholder}>搜索聊天记录</Text>
        </View>
      </View>

      {/* 会话列表 */}
      <ScrollView 
        style={styles.sessionList}
        contentContainerStyle={styles.sessionListContent}
        showsVerticalScrollIndicator={false}
      >
        {MOCK_SESSIONS.map((session) => (
          <Pressable
            key={session.id}
            style={styles.sessionItem}
            onPress={() => handleSessionPress(session.id)}
          >
            <View style={styles.avatarContainer}>
              <Text style={styles.avatar}>{session.avatar}</Text>
              {session.online && <View style={styles.onlineDot} />}
            </View>
            
            <View style={styles.sessionContent}>
              <View style={styles.sessionHeader}>
                <Text style={styles.sessionName} numberOfLines={1}>
                  {session.name}
                </Text>
                <Text style={styles.sessionTime}>{session.time}</Text>
              </View>
              <View style={styles.sessionFooter}>
                <Text style={styles.lastMessage} numberOfLines={1}>
                  {session.lastMessage}
                </Text>
                {session.unread > 0 && (
                  <View style={styles.unreadBadge}>
                    <Text style={styles.unreadText}>
                      {session.unread > 99 ? '99+' : session.unread}
                    </Text>
                  </View>
                )}
              </View>
            </View>
          </Pressable>
        ))}
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME_BG,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingTop: 60,
    paddingHorizontal: 20,
    paddingBottom: 16,
    backgroundColor: '#FFF',
  },
  title: {
    fontSize: 28,
    fontWeight: '600',
    color: '#333',
  },
  addButton: {
    width: 40,
    height: 40,
    alignItems: 'center',
    justifyContent: 'center',
  },
  headerRight: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  headerButton: {
    width: 40,
    height: 40,
    alignItems: 'center',
    justifyContent: 'center',
  },
  searchSection: {
    paddingHorizontal: 16,
    paddingVertical: 12,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  searchBar: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
    gap: 8,
  },
  searchPlaceholder: {
    color: '#999',
    fontSize: 15,
  },
  sessionList: {
    flex: 1,
  },
  sessionListContent: {
    paddingBottom: 100,
  },
  sessionItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 14,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F5F5F7',
  },
  avatarContainer: {
    position: 'relative',
    marginRight: 12,
  },
  avatar: {
    fontSize: 36,
    width: 52,
    height: 52,
    textAlign: 'center',
    lineHeight: 52,
    backgroundColor: '#F5F5F7',
    borderRadius: 26,
    overflow: 'hidden',
  },
  onlineDot: {
    position: 'absolute',
    bottom: 2,
    right: 2,
    width: 12,
    height: 12,
    backgroundColor: '#4CD964',
    borderRadius: 6,
    borderWidth: 2,
    borderColor: '#FFF',
  },
  sessionContent: {
    flex: 1,
  },
  sessionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 4,
  },
  sessionName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    flex: 1,
    marginRight: 8,
  },
  sessionTime: {
    fontSize: 12,
    color: '#999',
  },
  sessionFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  lastMessage: {
    fontSize: 14,
    color: '#999',
    flex: 1,
    marginRight: 8,
  },
  unreadBadge: {
    backgroundColor: '#FF3B30',
    borderRadius: 10,
    minWidth: 20,
    height: 20,
    alignItems: 'center',
    justifyContent: 'center',
    paddingHorizontal: 6,
  },
  unreadText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#FFF',
  },
});
