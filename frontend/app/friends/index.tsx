import { UserAvatar } from '@/src/components/UserAvatar';
import { useAuthStore } from '@/src/stores/auth';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useState } from 'react';
import {
    Alert,
    FlatList,
    Platform,
    Pressable,
    StyleSheet,
    Text,
    TextInput,
    View,
} from 'react-native';

type TabType = 'friends' | 'blocked' | 'whitelist';

interface User {
  address: string;
  name: string;
  addedAt: string;
}

const MOCK_FRIENDS: User[] = [
  { address: '5Grw...utQY', name: '张三', addedAt: '2025-01-20' },
  { address: '5DAn...kQrB', name: '李四', addedAt: '2025-01-18' },
  { address: '5Ck8...mNpC', name: '王五', addedAt: '2025-01-15' },
];

const MOCK_BLOCKED: User[] = [
  { address: '5HpG...xYzA', name: '黑名单用户1', addedAt: '2025-01-22' },
];

const MOCK_WHITELIST: User[] = [
  { address: '5Grw...utQY', name: '张三', addedAt: '2025-01-20' },
];

export default function FriendsScreen() {
  const router = useRouter();
  const { tab } = useLocalSearchParams<{ tab?: string }>();
  const { isLoggedIn } = useAuthStore();

  const [activeTab, setActiveTab] = useState<TabType>((tab as TabType) || 'friends');
  const [searchQuery, setSearchQuery] = useState('');
  const [showAddModal, setShowAddModal] = useState(false);
  const [addAddress, setAddAddress] = useState('');

  const getData = () => {
    switch (activeTab) {
      case 'friends':
        return MOCK_FRIENDS;
      case 'blocked':
        return MOCK_BLOCKED;
      case 'whitelist':
        return MOCK_WHITELIST;
    }
  };

  const filteredData = getData().filter(
    (user) =>
      user.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      user.address.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const handleRemove = (user: User) => {
    const actions = {
      friends: '删除好友',
      blocked: '解除屏蔽',
      whitelist: '移出白名单',
    };

    const confirmRemove = () => {
      // TODO: 调用链上对应方法
      console.log(`${actions[activeTab]}: ${user.address}`);
    };

    if (Platform.OS === 'web') {
      if (window.confirm(`确定要${actions[activeTab]} ${user.name} 吗？`)) {
        confirmRemove();
      }
    } else {
      Alert.alert('确认', `确定要${actions[activeTab]} ${user.name} 吗？`, [
        { text: '取消', style: 'cancel' },
        { text: '确定', style: 'destructive', onPress: confirmRemove },
      ]);
    }
  };

  const handleAdd = () => {
    if (!addAddress.trim()) {
      const msg = '请输入用户地址';
      if (Platform.OS === 'web') {
        alert(msg);
      } else {
        Alert.alert('提示', msg);
      }
      return;
    }

    const actions = {
      friends: '添加好友',
      blocked: '加入黑名单',
      whitelist: '加入白名单',
    };

    // TODO: 调用链上对应方法
    console.log(`${actions[activeTab]}: ${addAddress}`);
    setAddAddress('');
    setShowAddModal(false);
  };

  const renderItem = ({ item }: { item: User }) => (
    <View style={styles.userItem}>
      <UserAvatar name={item.name} size="medium" />
      <View style={styles.userInfo}>
        <Text style={styles.userName}>{item.name}</Text>
        <Text style={styles.userAddress}>{item.address}</Text>
      </View>
      <Pressable
        style={styles.actionButton}
        onPress={() => handleRemove(item)}
      >
        <Text style={styles.actionText}>
          {activeTab === 'friends' ? '删除' : activeTab === 'blocked' ? '解除' : '移除'}
        </Text>
      </Pressable>
    </View>
  );

  const getEmptyText = () => {
    switch (activeTab) {
      case 'friends':
        return '暂无好友';
      case 'blocked':
        return '黑名单为空';
      case 'whitelist':
        return '白名单为空';
    }
  };

  const getAddText = () => {
    switch (activeTab) {
      case 'friends':
        return '添加好友';
      case 'blocked':
        return '添加黑名单';
      case 'whitelist':
        return '添加白名单';
    }
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>
          {activeTab === 'friends' ? '好友' : activeTab === 'blocked' ? '黑名单' : '白名单'}
        </Text>
        <Pressable style={styles.addButton} onPress={() => setShowAddModal(true)}>
          <Text style={styles.addText}>+ 添加</Text>
        </Pressable>
      </View>

      <View style={styles.tabs}>
        <Pressable
          style={[styles.tab, activeTab === 'friends' && styles.tabActive]}
          onPress={() => setActiveTab('friends')}
        >
          <Text style={[styles.tabText, activeTab === 'friends' && styles.tabTextActive]}>
            好友 ({MOCK_FRIENDS.length})
          </Text>
        </Pressable>
        <Pressable
          style={[styles.tab, activeTab === 'blocked' && styles.tabActive]}
          onPress={() => setActiveTab('blocked')}
        >
          <Text style={[styles.tabText, activeTab === 'blocked' && styles.tabTextActive]}>
            黑名单 ({MOCK_BLOCKED.length})
          </Text>
        </Pressable>
        <Pressable
          style={[styles.tab, activeTab === 'whitelist' && styles.tabActive]}
          onPress={() => setActiveTab('whitelist')}
        >
          <Text style={[styles.tabText, activeTab === 'whitelist' && styles.tabTextActive]}>
            白名单 ({MOCK_WHITELIST.length})
          </Text>
        </Pressable>
      </View>

      <View style={styles.searchContainer}>
        <TextInput
          style={styles.searchInput}
          placeholder="搜索用户..."
          placeholderTextColor="#9ca3af"
          value={searchQuery}
          onChangeText={setSearchQuery}
        />
      </View>

      <FlatList
        data={filteredData}
        renderItem={renderItem}
        keyExtractor={(item) => item.address}
        contentContainerStyle={styles.list}
        showsVerticalScrollIndicator={false}
        ListEmptyComponent={
          <View style={styles.empty}>
            <Text style={styles.emptyText}>{getEmptyText()}</Text>
          </View>
        }
      />

      {showAddModal && (
        <View style={styles.modalOverlay}>
          <View style={styles.modal}>
            <Text style={styles.modalTitle}>{getAddText()}</Text>
            <TextInput
              style={styles.modalInput}
              placeholder="输入用户钱包地址"
              placeholderTextColor="#9ca3af"
              value={addAddress}
              onChangeText={setAddAddress}
            />
            <View style={styles.modalActions}>
              <Pressable
                style={[styles.modalButton, styles.cancelButton]}
                onPress={() => {
                  setShowAddModal(false);
                  setAddAddress('');
                }}
              >
                <Text style={styles.cancelButtonText}>取消</Text>
              </Pressable>
              <Pressable
                style={[styles.modalButton, styles.confirmButton]}
                onPress={handleAdd}
              >
                <Text style={styles.confirmButtonText}>确定</Text>
              </Pressable>
            </View>
          </View>
        </View>
      )}
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
  addButton: {
    padding: 4,
  },
  addText: {
    fontSize: 15,
    color: '#6D28D9',
    fontWeight: '600',
  },
  tabs: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  tab: {
    flex: 1,
    paddingVertical: 12,
    alignItems: 'center',
    borderBottomWidth: 2,
    borderBottomColor: 'transparent',
  },
  tabActive: {
    borderBottomColor: '#6D28D9',
  },
  tabText: {
    fontSize: 14,
    color: '#6b7280',
  },
  tabTextActive: {
    color: '#6D28D9',
    fontWeight: '600',
  },
  searchContainer: {
    padding: 16,
    paddingBottom: 8,
  },
  searchInput: {
    backgroundColor: '#fff',
    borderRadius: 12,
    paddingHorizontal: 16,
    paddingVertical: 12,
    fontSize: 15,
    color: '#1f2937',
  },
  list: {
    padding: 16,
    paddingTop: 8,
  },
  userItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#fff',
    padding: 16,
    borderRadius: 12,
    marginBottom: 8,
  },
  userInfo: {
    flex: 1,
    marginLeft: 12,
  },
  userName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
  },
  userAddress: {
    fontSize: 13,
    color: '#6b7280',
    marginTop: 2,
  },
  actionButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 6,
    backgroundColor: '#fee2e2',
  },
  actionText: {
    fontSize: 13,
    color: '#dc2626',
    fontWeight: '500',
  },
  empty: {
    padding: 40,
    alignItems: 'center',
  },
  emptyText: {
    fontSize: 15,
    color: '#9ca3af',
  },
  modalOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0,0,0,0.5)',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 24,
  },
  modal: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 24,
    width: '100%',
    maxWidth: 400,
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 16,
    textAlign: 'center',
  },
  modalInput: {
    backgroundColor: '#f3f4f6',
    borderRadius: 12,
    paddingHorizontal: 16,
    paddingVertical: 14,
    fontSize: 15,
    color: '#1f2937',
    marginBottom: 20,
  },
  modalActions: {
    flexDirection: 'row',
    gap: 12,
  },
  modalButton: {
    flex: 1,
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
  },
  cancelButton: {
    backgroundColor: '#f3f4f6',
  },
  cancelButtonText: {
    color: '#6b7280',
    fontWeight: '600',
  },
  confirmButton: {
    backgroundColor: '#6D28D9',
  },
  confirmButtonText: {
    color: '#fff',
    fontWeight: '600',
  },
});
