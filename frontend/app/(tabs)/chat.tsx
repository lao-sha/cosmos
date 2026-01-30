import { ChatSession, useChatSessions } from '@/src/hooks/useChat';
import { useAuthStore } from '@/src/stores/auth';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    Alert,
    FlatList,
    Image,
    Platform,
    Pressable,
    StyleSheet,
    Text,
    View
} from 'react-native';

type TabType = 'private' | 'group';

interface GroupChat {
  id: string;
  name: string;
  avatar?: string;
  memberCount: number;
  lastMessage: string;
  lastMessageTime: string;
  unreadCount: number;
}

const MOCK_GROUPS: GroupChat[] = [
  {
    id: 'g1',
    name: '八字命理交流群',
    memberCount: 128,
    lastMessage: '今天有人排了个很有意思的盘',
    lastMessageTime: '10:30',
    unreadCount: 5,
  },
  {
    id: 'g2',
    name: '奇门遁甲研习社',
    memberCount: 56,
    lastMessage: '请教一下阳遁三局的值符怎么找',
    lastMessageTime: '昨天',
    unreadCount: 0,
  },
  {
    id: 'g3',
    name: '塔罗牌爱好者',
    memberCount: 89,
    lastMessage: '分享一个韦特塔罗的解读技巧',
    lastMessageTime: '周一',
    unreadCount: 12,
  },
  {
    id: 'g4',
    name: '紫微斗数学习群',
    memberCount: 45,
    lastMessage: '命宫有紫微贪狼是什么意思？',
    lastMessageTime: '周日',
    unreadCount: 0,
  },
];

export default function ChatScreen() {
  const router = useRouter();
  const { isLoggedIn } = useAuthStore();
  const { sessions, loading, error } = useChatSessions();
  const [activeTab, setActiveTab] = useState<TabType>('private');
  const [groups] = useState<GroupChat[]>(MOCK_GROUPS);

  const handleSessionPress = (session: ChatSession) => {
    if (!isLoggedIn) {
      if (Platform.OS === 'web') {
        window.alert('请先登录钱包');
      } else {
        Alert.alert('提示', '请先登录钱包');
      }
      return;
    }
    router.push(`/chat/${session.id}`);
  };

  const handleGroupPress = (group: GroupChat) => {
    if (!isLoggedIn) {
      if (Platform.OS === 'web') {
        window.alert('请先登录钱包');
      } else {
        Alert.alert('提示', '请先登录钱包');
      }
      return;
    }
    router.push(`/chat/group/${group.id}` as any);
  };

  const handleCreateGroup = () => {
    if (!isLoggedIn) {
      if (Platform.OS === 'web') {
        window.alert('请先登录钱包');
      } else {
        Alert.alert('提示', '请先登录钱包');
      }
      return;
    }
    router.push('/chat/group/create' as any);
  };

  const renderSession = ({ item }: { item: ChatSession }) => (
    <Pressable
      style={({ pressed }) => [styles.sessionItem, pressed && styles.sessionPressed]}
      onPress={() => handleSessionPress(item)}
    >
      <View style={styles.avatarContainer}>
        {item.participantAvatar ? (
          <Image source={{ uri: item.participantAvatar }} style={styles.avatar} />
        ) : (
          <View style={styles.avatarPlaceholder}>
            <Text style={styles.avatarText}>{item.participantName[0]}</Text>
          </View>
        )}
        {item.isOnline && <View style={styles.onlineIndicator} />}
      </View>
      
      <View style={styles.sessionContent}>
        <View style={styles.sessionHeader}>
          <Text style={styles.participantName}>{item.participantName}</Text>
          <Text style={styles.timeText}>{item.lastMessageTime}</Text>
        </View>
        <View style={styles.sessionFooter}>
          <Text style={styles.lastMessage} numberOfLines={1}>
            {item.lastMessage}
          </Text>
          {item.unreadCount > 0 && (
            <View style={styles.unreadBadge}>
              <Text style={styles.unreadText}>{item.unreadCount}</Text>
            </View>
          )}
        </View>
      </View>
    </Pressable>
  );

  const renderGroup = ({ item }: { item: GroupChat }) => (
    <Pressable
      style={({ pressed }) => [styles.sessionItem, pressed && styles.sessionPressed]}
      onPress={() => handleGroupPress(item)}
    >
      <View style={styles.avatarContainer}>
        {item.avatar ? (
          <Image source={{ uri: item.avatar }} style={styles.avatar} />
        ) : (
          <View style={[styles.avatarPlaceholder, styles.groupAvatar]}>
            <Text style={styles.avatarText}>👥</Text>
          </View>
        )}
      </View>
      
      <View style={styles.sessionContent}>
        <View style={styles.sessionHeader}>
          <View style={styles.groupNameRow}>
            <Text style={styles.participantName}>{item.name}</Text>
            <Text style={styles.memberCount}>({item.memberCount})</Text>
          </View>
          <Text style={styles.timeText}>{item.lastMessageTime}</Text>
        </View>
        <View style={styles.sessionFooter}>
          <Text style={styles.lastMessage} numberOfLines={1}>
            {item.lastMessage}
          </Text>
          {item.unreadCount > 0 && (
            <View style={styles.unreadBadge}>
              <Text style={styles.unreadText}>{item.unreadCount}</Text>
            </View>
          )}
        </View>
      </View>
    </Pressable>
  );

  if (!isLoggedIn) {
    return (
      <View style={styles.emptyContainer}>
        <Text style={styles.emptyIcon}>💬</Text>
        <Text style={styles.emptyTitle}>登录后查看消息</Text>
        <Text style={styles.emptySubtitle}>请先在"我的"页面创建或导入钱包</Text>
      </View>
    );
  }

  const renderPrivateList = () => (
    sessions.length > 0 ? (
      <FlatList
        data={sessions}
        renderItem={renderSession}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.listContainer}
      />
    ) : (
      <View style={styles.emptyContainer}>
        <Text style={styles.emptyIcon}>💬</Text>
        <Text style={styles.emptyTitle}>暂无私聊</Text>
        <Text style={styles.emptySubtitle}>开始与他人交流吧</Text>
      </View>
    )
  );

  const renderGroupList = () => (
    <View style={styles.groupListContainer}>
      <Pressable style={styles.createGroupButton} onPress={handleCreateGroup}>
        <Text style={styles.createGroupIcon}>➕</Text>
        <Text style={styles.createGroupText}>创建群聊</Text>
      </Pressable>
      
      {groups.length > 0 ? (
        <FlatList
          data={groups}
          renderItem={renderGroup}
          keyExtractor={(item) => item.id}
          contentContainerStyle={styles.listContainer}
        />
      ) : (
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>�</Text>
          <Text style={styles.emptyTitle}>暂无群聊</Text>
          <Text style={styles.emptySubtitle}>创建或加入一个群聊吧</Text>
        </View>
      )}
    </View>
  );

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.headerTitle}>消息</Text>
        <View style={styles.tabContainer}>
          <Pressable
            style={[styles.tab, activeTab === 'private' && styles.tabActive]}
            onPress={() => setActiveTab('private')}
          >
            <Text style={[styles.tabText, activeTab === 'private' && styles.tabTextActive]}>
              私聊
            </Text>
          </Pressable>
          <Pressable
            style={[styles.tab, activeTab === 'group' && styles.tabActive]}
            onPress={() => setActiveTab('group')}
          >
            <Text style={[styles.tabText, activeTab === 'group' && styles.tabTextActive]}>
              群聊
            </Text>
          </Pressable>
        </View>
      </View>
      
      {activeTab === 'private' ? renderPrivateList() : renderGroupList()}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    backgroundColor: '#fff',
    paddingTop: 50,
    paddingBottom: 12,
    paddingHorizontal: 20,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  headerTitle: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#1f2937',
    marginBottom: 12,
  },
  tabContainer: {
    flexDirection: 'row',
    backgroundColor: '#f3f4f6',
    borderRadius: 10,
    padding: 4,
  },
  tab: {
    flex: 1,
    paddingVertical: 8,
    alignItems: 'center',
    borderRadius: 8,
  },
  tabActive: {
    backgroundColor: '#fff',
  },
  tabText: {
    fontSize: 14,
    fontWeight: '500',
    color: '#6b7280',
  },
  tabTextActive: {
    color: '#6D28D9',
    fontWeight: '600',
  },
  groupListContainer: {
    flex: 1,
  },
  createGroupButton: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#fff',
    paddingHorizontal: 16,
    paddingVertical: 14,
    marginHorizontal: 12,
    marginTop: 12,
    marginBottom: 4,
    borderRadius: 12,
  },
  createGroupIcon: {
    fontSize: 20,
    marginRight: 12,
  },
  createGroupText: {
    fontSize: 16,
    fontWeight: '500',
    color: '#6D28D9',
  },
  groupAvatar: {
    backgroundColor: '#10b981',
  },
  groupNameRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  memberCount: {
    fontSize: 12,
    color: '#9ca3af',
    marginLeft: 4,
  },
  listContainer: {
    paddingVertical: 8,
  },
  sessionItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#fff',
    paddingHorizontal: 16,
    paddingVertical: 12,
    marginHorizontal: 12,
    marginVertical: 4,
    borderRadius: 12,
  },
  sessionPressed: {
    backgroundColor: '#f3f4f6',
  },
  avatarContainer: {
    position: 'relative',
    marginRight: 12,
  },
  avatar: {
    width: 50,
    height: 50,
    borderRadius: 25,
  },
  avatarPlaceholder: {
    width: 50,
    height: 50,
    borderRadius: 25,
    backgroundColor: '#6D28D9',
    alignItems: 'center',
    justifyContent: 'center',
  },
  avatarText: {
    color: '#fff',
    fontSize: 20,
    fontWeight: 'bold',
  },
  onlineIndicator: {
    position: 'absolute',
    bottom: 2,
    right: 2,
    width: 12,
    height: 12,
    borderRadius: 6,
    backgroundColor: '#22c55e',
    borderWidth: 2,
    borderColor: '#fff',
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
  participantName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
  },
  timeText: {
    fontSize: 12,
    color: '#9ca3af',
  },
  sessionFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  lastMessage: {
    flex: 1,
    fontSize: 14,
    color: '#6b7280',
    marginRight: 8,
  },
  unreadBadge: {
    backgroundColor: '#ef4444',
    borderRadius: 10,
    minWidth: 20,
    height: 20,
    alignItems: 'center',
    justifyContent: 'center',
    paddingHorizontal: 6,
  },
  unreadText: {
    color: '#fff',
    fontSize: 12,
    fontWeight: 'bold',
  },
  emptyContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
  },
  emptyIcon: {
    fontSize: 64,
    marginBottom: 16,
  },
  emptyTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#1f2937',
    marginBottom: 8,
  },
  emptySubtitle: {
    fontSize: 14,
    color: '#9ca3af',
    textAlign: 'center',
  },
});
