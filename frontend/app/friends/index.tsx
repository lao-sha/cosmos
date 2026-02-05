import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
  Alert,
  TextInput,
} from 'react-native';
import { useRouter } from 'expo-router';
import {
  Search,
  UserPlus,
  MessageCircle,
  MoreHorizontal,
  Circle,
  Bell,
} from 'lucide-react-native';
import * as Haptics from 'expo-haptics';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import {
  getFriends,
  getFriendRequests,
  acceptFriendRequest,
  rejectFriendRequest,
  removeFriend,
  type Friend,
  type FriendRequest,
} from '@/services/friends';
import { Card, Button } from '@/components/ui';
import { Colors } from '@/constants/colors';

type Tab = 'friends' | 'requests';

export default function FriendsScreen() {
  const colors = useColors();
  const router = useRouter();
  const { address, mnemonic } = useWalletStore();

  const [activeTab, setActiveTab] = useState<Tab>('friends');
  const [friends, setFriends] = useState<Friend[]>([]);
  const [requests, setRequests] = useState<FriendRequest[]>([]);
  const [searchText, setSearchText] = useState('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, [address]);

  const loadData = async () => {
    if (!address) return;
    setLoading(true);
    try {
      const [friendsData, requestsData] = await Promise.all([
        getFriends(address),
        getFriendRequests(address),
      ]);
      setFriends(friendsData);
      setRequests(requestsData);
    } catch (error) {
      console.error('Failed to load friends:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleAccept = async (requestId: string) => {
    if (!mnemonic) return;
    try {
      await acceptFriendRequest(requestId, mnemonic);
      Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
      loadData();
    } catch (error: any) {
      Alert.alert('操作失败', error.message);
    }
  };

  const handleReject = async (requestId: string) => {
    if (!mnemonic) return;
    try {
      await rejectFriendRequest(requestId, mnemonic);
      loadData();
    } catch (error: any) {
      Alert.alert('操作失败', error.message);
    }
  };

  const handleRemove = (friend: Friend) => {
    Alert.alert('删除好友', `确定要删除 ${friend.name} 吗？`, [
      { text: '取消', style: 'cancel' },
      {
        text: '删除',
        style: 'destructive',
        onPress: async () => {
          if (!mnemonic) return;
          try {
            await removeFriend(friend.address, mnemonic);
            loadData();
          } catch (error: any) {
            Alert.alert('操作失败', error.message);
          }
        },
      },
    ]);
  };

  const filteredFriends = friends.filter(
    (f) =>
      f.name.toLowerCase().includes(searchText.toLowerCase()) ||
      f.address.toLowerCase().includes(searchText.toLowerCase())
  );

  const renderFriend = ({ item }: { item: Friend }) => (
    <TouchableOpacity
      style={[styles.friendItem, { borderBottomColor: colors.border }]}
      onPress={() => router.push(`/chat/${item.id}`)}
      onLongPress={() => handleRemove(item)}
      activeOpacity={0.7}
    >
      <View style={styles.friendLeft}>
        <View style={[styles.avatar, { backgroundColor: Colors.primary + '20' }]}>
          <Text style={[styles.avatarText, { color: Colors.primary }]}>
            {item.name.charAt(0)}
          </Text>
        </View>
        <View style={styles.onlineIndicator}>
          <Circle
            size={10}
            fill={item.isOnline ? Colors.success : colors.textTertiary}
            color={item.isOnline ? Colors.success : colors.textTertiary}
          />
        </View>
      </View>
      <View style={styles.friendInfo}>
        <Text style={[styles.friendName, { color: colors.textPrimary }]}>
          {item.remark || item.name}
        </Text>
        <Text style={[styles.friendStatus, { color: colors.textTertiary }]}>
          {item.isOnline ? '在线' : item.lastSeen ? `${formatLastSeen(item.lastSeen)}` : '离线'}
        </Text>
      </View>
      <TouchableOpacity
        style={styles.chatButton}
        onPress={() => router.push(`/chat/${item.id}`)}
      >
        <MessageCircle size={22} color={Colors.primary} />
      </TouchableOpacity>
    </TouchableOpacity>
  );

  const renderRequest = ({ item }: { item: FriendRequest }) => (
    <Card style={styles.requestCard}>
      <View style={styles.requestHeader}>
        <View style={[styles.avatar, { backgroundColor: Colors.primary + '20' }]}>
          <Text style={[styles.avatarText, { color: Colors.primary }]}>
            {item.fromName.charAt(0)}
          </Text>
        </View>
        <View style={styles.requestInfo}>
          <Text style={[styles.requestName, { color: colors.textPrimary }]}>
            {item.fromName}
          </Text>
          <Text style={[styles.requestTime, { color: colors.textTertiary }]}>
            {formatTime(item.createdAt)}
          </Text>
        </View>
      </View>
      {item.message && (
        <Text style={[styles.requestMessage, { color: colors.textSecondary }]}>
          "{item.message}"
        </Text>
      )}
      <View style={styles.requestActions}>
        <Button
          title="拒绝"
          variant="outline"
          size="sm"
          onPress={() => handleReject(item.id)}
          style={styles.requestButton}
        />
        <Button
          title="接受"
          size="sm"
          onPress={() => handleAccept(item.id)}
          style={styles.requestButton}
        />
      </View>
    </Card>
  );

  return (
    <View style={[styles.container, { backgroundColor: colors.background }]}>
      {/* Search Bar */}
      <View style={styles.searchContainer}>
        <View style={[styles.searchBar, { backgroundColor: colors.surface }]}>
          <Search size={20} color={colors.textTertiary} />
          <TextInput
            style={[styles.searchInput, { color: colors.textPrimary }]}
            placeholder="搜索好友"
            placeholderTextColor={colors.textTertiary}
            value={searchText}
            onChangeText={setSearchText}
          />
        </View>
        <TouchableOpacity
          style={[styles.addButton, { backgroundColor: Colors.primary }]}
          onPress={() => router.push('/friends/add')}
        >
          <UserPlus size={22} color="#FFFFFF" />
        </TouchableOpacity>
      </View>

      {/* Tabs */}
      <View style={[styles.tabs, { borderBottomColor: colors.border }]}>
        <TouchableOpacity
          style={[
            styles.tab,
            activeTab === 'friends' && styles.tabActive,
          ]}
          onPress={() => setActiveTab('friends')}
        >
          <Text
            style={[
              styles.tabText,
              { color: activeTab === 'friends' ? Colors.primary : colors.textSecondary },
            ]}
          >
            好友 ({friends.length})
          </Text>
          {activeTab === 'friends' && (
            <View style={[styles.tabIndicator, { backgroundColor: Colors.primary }]} />
          )}
        </TouchableOpacity>
        <TouchableOpacity
          style={[
            styles.tab,
            activeTab === 'requests' && styles.tabActive,
          ]}
          onPress={() => setActiveTab('requests')}
        >
          <View style={styles.tabContent}>
            <Text
              style={[
                styles.tabText,
                { color: activeTab === 'requests' ? Colors.primary : colors.textSecondary },
              ]}
            >
              请求
            </Text>
            {requests.length > 0 && (
              <View style={[styles.badge, { backgroundColor: Colors.error }]}>
                <Text style={styles.badgeText}>{requests.length}</Text>
              </View>
            )}
          </View>
          {activeTab === 'requests' && (
            <View style={[styles.tabIndicator, { backgroundColor: Colors.primary }]} />
          )}
        </TouchableOpacity>
      </View>

      {/* Content */}
      {activeTab === 'friends' ? (
        <FlatList
          data={filteredFriends}
          renderItem={renderFriend}
          keyExtractor={(item) => item.id}
          contentContainerStyle={styles.list}
          ListEmptyComponent={
            <View style={styles.empty}>
              <Text style={[styles.emptyText, { color: colors.textSecondary }]}>
                {loading ? '加载中...' : searchText ? '未找到匹配的好友' : '暂无好友'}
              </Text>
              {!loading && !searchText && (
                <Button
                  title="添加好友"
                  onPress={() => router.push('/friends/add')}
                  style={styles.emptyButton}
                />
              )}
            </View>
          }
        />
      ) : (
        <FlatList
          data={requests}
          renderItem={renderRequest}
          keyExtractor={(item) => item.id}
          contentContainerStyle={styles.list}
          ListEmptyComponent={
            <View style={styles.empty}>
              <Text style={[styles.emptyText, { color: colors.textSecondary }]}>
                {loading ? '加载中...' : '暂无好友请求'}
              </Text>
            </View>
          }
        />
      )}
    </View>
  );
}

function formatLastSeen(timestamp: number): string {
  const diff = Date.now() - timestamp;
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return '刚刚在线';
  if (minutes < 60) return `${minutes}分钟前`;
  if (hours < 24) return `${hours}小时前`;
  return `${days}天前`;
}

function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleDateString();
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  searchContainer: {
    flexDirection: 'row',
    padding: 16,
    gap: 12,
  },
  searchBar: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 12,
    borderRadius: 12,
    height: 44,
  },
  searchInput: {
    flex: 1,
    marginLeft: 8,
    fontSize: 16,
  },
  addButton: {
    width: 44,
    height: 44,
    borderRadius: 12,
    alignItems: 'center',
    justifyContent: 'center',
  },
  tabs: {
    flexDirection: 'row',
    borderBottomWidth: 1,
  },
  tab: {
    flex: 1,
    alignItems: 'center',
    paddingVertical: 12,
    position: 'relative',
  },
  tabActive: {},
  tabContent: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  tabText: {
    fontSize: 15,
    fontWeight: '500',
  },
  tabIndicator: {
    position: 'absolute',
    bottom: 0,
    height: 2,
    width: 40,
    borderRadius: 1,
  },
  badge: {
    minWidth: 18,
    height: 18,
    borderRadius: 9,
    alignItems: 'center',
    justifyContent: 'center',
    marginLeft: 6,
    paddingHorizontal: 5,
  },
  badgeText: {
    color: '#FFFFFF',
    fontSize: 11,
    fontWeight: '600',
  },
  list: {
    padding: 16,
  },
  friendItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 12,
    borderBottomWidth: 1,
  },
  friendLeft: {
    position: 'relative',
  },
  avatar: {
    width: 48,
    height: 48,
    borderRadius: 24,
    alignItems: 'center',
    justifyContent: 'center',
  },
  avatarText: {
    fontSize: 18,
    fontWeight: '600',
  },
  onlineIndicator: {
    position: 'absolute',
    bottom: 0,
    right: 0,
    backgroundColor: '#FFFFFF',
    borderRadius: 6,
    padding: 2,
  },
  friendInfo: {
    flex: 1,
    marginLeft: 12,
  },
  friendName: {
    fontSize: 16,
    fontWeight: '500',
  },
  friendStatus: {
    fontSize: 13,
    marginTop: 2,
  },
  chatButton: {
    padding: 8,
  },
  requestCard: {
    marginBottom: 12,
  },
  requestHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8,
  },
  requestInfo: {
    flex: 1,
    marginLeft: 12,
  },
  requestName: {
    fontSize: 16,
    fontWeight: '500',
  },
  requestTime: {
    fontSize: 12,
    marginTop: 2,
  },
  requestMessage: {
    fontSize: 14,
    fontStyle: 'italic',
    marginBottom: 12,
  },
  requestActions: {
    flexDirection: 'row',
    gap: 12,
  },
  requestButton: {
    flex: 1,
  },
  empty: {
    alignItems: 'center',
    paddingTop: 60,
  },
  emptyText: {
    fontSize: 16,
    marginBottom: 16,
  },
  emptyButton: {},
});
