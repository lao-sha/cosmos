/**
 * 联系人列表页面
 */

import React, { useEffect, useCallback, useState } from 'react';
import {
  View,
  FlatList,
  StyleSheet,
  Text,
  RefreshControl,
  TouchableOpacity,
  SectionList,
  TextInput,
  Pressable,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { useContactsStore } from '@/stores/contacts.store';
import { ContactItem } from '../components/ContactItem';
import { GroupItem } from '../components/GroupItem';
import type { Contact, ContactGroup } from '../types';
import { FriendStatus } from '../types';

const THEME_COLOR = '#B2955D';

type ViewMode = 'all' | 'groups' | 'friends';

export function ContactListScreen() {
  const router = useRouter();
  const insets = useSafeAreaInsets();
  const {
    contacts,
    groups,
    stats,
    isLoading,
    loadContacts,
    loadGroups,
    refreshAll,
  } = useContactsStore();

  const [viewMode, setViewMode] = useState<ViewMode>('all');
  const [searchText, setSearchText] = useState('');

  useEffect(() => {
    refreshAll();
  }, []);

  const handleContactPress = useCallback(
    (contact: Contact) => {
      router.push(`/contacts/${contact.address}`);
    },
    [router]
  );

  const handleGroupPress = useCallback(
    (group: ContactGroup) => {
      router.push(`/contacts/group/${encodeURIComponent(group.name)}`);
    },
    [router]
  );

  const handleAddContact = useCallback(() => {
    router.push('/contacts/add');
  }, [router]);

  const handleFriendRequests = useCallback(() => {
    router.push('/contacts/requests');
  }, [router]);

  // 过滤联系人
  const filteredContacts = contacts.filter((c) => {
    // 搜索过滤
    if (searchText) {
      const search = searchText.toLowerCase();
      const name = (c.alias || c.profile?.nickname || c.address).toLowerCase();
      if (!name.includes(search)) return false;
    }

    // 视图模式过滤
    if (viewMode === 'friends') {
      return c.friendStatus === FriendStatus.Mutual;
    }

    return true;
  });

  // 按首字母分组
  const getSectionData = () => {
    const sections: { title: string; data: Contact[] }[] = [];
    const grouped: Record<string, Contact[]> = {};

    filteredContacts.forEach((contact) => {
      const name = contact.alias || contact.profile?.nickname || contact.address;
      const firstChar = name[0].toUpperCase();
      const key = /[A-Z]/.test(firstChar) ? firstChar : '#';

      if (!grouped[key]) {
        grouped[key] = [];
      }
      grouped[key].push(contact);
    });

    Object.keys(grouped)
      .sort()
      .forEach((key) => {
        sections.push({ title: key, data: grouped[key] });
      });

    return sections;
  };

  const renderSectionHeader = ({ section }: { section: { title: string } }) => (
    <View style={styles.sectionHeader}>
      <Text style={styles.sectionTitle}>{section.title}</Text>
    </View>
  );

  const renderContactItem = ({ item }: { item: Contact }) => (
    <ContactItem contact={item} onPress={() => handleContactPress(item)} />
  );

  const renderGroupItem = ({ item }: { item: ContactGroup }) => (
    <GroupItem group={item} onPress={() => handleGroupPress(item)} />
  );

  const renderEmpty = () => (
    <View style={styles.empty}>
      <Ionicons name="people-outline" size={64} color="#ccc" />
      <Text style={styles.emptyText}>暂无联系人</Text>
      <Text style={styles.emptySubtext}>添加联系人开始聊天吧</Text>
      <TouchableOpacity style={styles.addBtn} onPress={handleAddContact}>
        <Text style={styles.addBtnText}>添加联系人</Text>
      </TouchableOpacity>
    </View>
  );

  return (
    <View style={[styles.container, { paddingTop: insets.top }]}>
      {/* 头部 */}
      <View style={styles.header}>
        <Text style={styles.title}>通讯录</Text>
        <View style={styles.headerRight}>
          {stats && stats.pendingRequestCount > 0 && (
            <TouchableOpacity
              style={styles.requestBtn}
              onPress={handleFriendRequests}
            >
              <Ionicons name="person-add" size={22} color="#007AFF" />
              <View style={styles.requestBadge}>
                <Text style={styles.requestBadgeText}>
                  {stats.pendingRequestCount > 9 ? '9+' : stats.pendingRequestCount}
                </Text>
              </View>
            </TouchableOpacity>
          )}
          <TouchableOpacity style={styles.headerBtn} onPress={handleAddContact}>
            <Ionicons name="add" size={28} color="#007AFF" />
          </TouchableOpacity>
        </View>
      </View>

      {/* 搜索框 */}
      <View style={styles.searchContainer}>
        <View style={styles.searchBox}>
          <Ionicons name="search" size={18} color="#999" />
          <TextInput
            style={styles.searchInput}
            value={searchText}
            onChangeText={setSearchText}
            placeholder="搜索联系人"
            placeholderTextColor="#999"
          />
          {searchText.length > 0 && (
            <TouchableOpacity onPress={() => setSearchText('')}>
              <Ionicons name="close-circle" size={18} color="#999" />
            </TouchableOpacity>
          )}
        </View>
      </View>

      {/* 视图切换 */}
      <View style={styles.tabs}>
        <TouchableOpacity
          style={[styles.tab, viewMode === 'all' && styles.tabActive]}
          onPress={() => setViewMode('all')}
        >
          <Text
            style={[styles.tabText, viewMode === 'all' && styles.tabTextActive]}
          >
            全部 ({contacts.length})
          </Text>
        </TouchableOpacity>
        <TouchableOpacity
          style={[styles.tab, viewMode === 'friends' && styles.tabActive]}
          onPress={() => setViewMode('friends')}
        >
          <Text
            style={[
              styles.tabText,
              viewMode === 'friends' && styles.tabTextActive,
            ]}
          >
            好友 ({contacts.filter((c) => c.friendStatus === FriendStatus.Mutual).length})
          </Text>
        </TouchableOpacity>
        <TouchableOpacity
          style={[styles.tab, viewMode === 'groups' && styles.tabActive]}
          onPress={() => setViewMode('groups')}
        >
          <Text
            style={[
              styles.tabText,
              viewMode === 'groups' && styles.tabTextActive,
            ]}
          >
            分组 ({groups.length})
          </Text>
        </TouchableOpacity>
      </View>

      {/* 列表 */}
      {viewMode === 'groups' ? (
        <FlatList
          data={groups}
          renderItem={renderGroupItem}
          keyExtractor={(item) => item.name}
          ListEmptyComponent={
            <View style={styles.empty}>
              <Text style={styles.emptyText}>暂无分组</Text>
            </View>
          }
          refreshControl={
            <RefreshControl refreshing={isLoading} onRefresh={loadGroups} />
          }
          contentContainerStyle={{ paddingBottom: 80 }}
        />
      ) : (
        <SectionList
          sections={getSectionData()}
          renderItem={renderContactItem}
          renderSectionHeader={renderSectionHeader}
          keyExtractor={(item) => item.address}
          ListEmptyComponent={renderEmpty}
          refreshControl={
            <RefreshControl refreshing={isLoading} onRefresh={loadContacts} />
          }
          stickySectionHeadersEnabled
          contentContainerStyle={{ paddingBottom: 80 }}
        />
      )}

      {/* 底部导航栏 */}
      <View style={styles.bottomNav}>
        <Pressable style={styles.navItem} onPress={() => router.push('/(tabs)')}>
          <Ionicons name="home-outline" size={24} color="#999" />
          <Text style={styles.navLabel}>首页</Text>
        </Pressable>
        <Pressable style={styles.navItem} onPress={() => router.push('/(tabs)/divination')}>
          <Ionicons name="compass-outline" size={24} color="#999" />
          <Text style={styles.navLabel}>占卜</Text>
        </Pressable>
        <Pressable style={styles.navItem} onPress={() => router.push('/(tabs)/market')}>
          <Ionicons name="storefront-outline" size={24} color="#999" />
          <Text style={styles.navLabel}>市场</Text>
        </Pressable>
        <Pressable style={styles.navItem} onPress={() => router.push('/(tabs)/chat')}>
          <Ionicons name="chatbubble-outline" size={24} color="#999" />
          <Text style={styles.navLabel}>消息</Text>
        </Pressable>
        <Pressable style={styles.navItem} onPress={() => router.push('/(tabs)/profile')}>
          <Ionicons name="person-outline" size={24} color="#999" />
          <Text style={styles.navLabel}>我的</Text>
        </Pressable>
      </View>
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
    paddingHorizontal: 16,
    paddingVertical: 12,
    backgroundColor: '#fff',
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    color: '#333',
  },
  headerRight: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  headerBtn: {
    padding: 4,
  },
  requestBtn: {
    padding: 4,
    position: 'relative',
  },
  requestBadge: {
    position: 'absolute',
    top: 0,
    right: 0,
    minWidth: 16,
    height: 16,
    borderRadius: 8,
    backgroundColor: '#FF3B30',
    justifyContent: 'center',
    alignItems: 'center',
  },
  requestBadgeText: {
    color: '#fff',
    fontSize: 10,
    fontWeight: '600',
  },
  searchContainer: {
    padding: 12,
    backgroundColor: '#fff',
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#e0e0e0',
  },
  searchBox: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 12,
    height: 36,
  },
  searchInput: {
    flex: 1,
    marginLeft: 8,
    fontSize: 15,
    color: '#333',
  },
  tabs: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#e0e0e0',
  },
  tab: {
    flex: 1,
    paddingVertical: 12,
    alignItems: 'center',
  },
  tabActive: {
    borderBottomWidth: 2,
    borderBottomColor: '#007AFF',
  },
  tabText: {
    fontSize: 14,
    color: '#666',
  },
  tabTextActive: {
    color: '#007AFF',
    fontWeight: '600',
  },
  sectionHeader: {
    backgroundColor: '#f5f5f5',
    paddingHorizontal: 16,
    paddingVertical: 6,
  },
  sectionTitle: {
    fontSize: 13,
    fontWeight: '600',
    color: '#999',
  },
  empty: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingVertical: 60,
  },
  emptyText: {
    fontSize: 18,
    color: '#333',
    marginTop: 16,
    marginBottom: 8,
  },
  emptySubtext: {
    fontSize: 14,
    color: '#999',
    marginBottom: 24,
  },
  addBtn: {
    backgroundColor: '#007AFF',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  addBtnText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  bottomNav: {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
    flexDirection: 'row',
    backgroundColor: '#FFF',
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
    paddingTop: 8,
    paddingBottom: 24,
  },
  navItem: {
    flex: 1,
    alignItems: 'center',
    gap: 2,
  },
  navLabel: {
    fontSize: 11,
    color: '#999',
  },
});
