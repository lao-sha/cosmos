/**
 * 添加联系人页面
 */

import React, { useState, useCallback } from 'react';
import {
  View,
  Text,
  TextInput,
  StyleSheet,
  TouchableOpacity,
  ActivityIndicator,
  Alert,
  ScrollView,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { useContactsStore } from '@/stores/contacts.store';
import { getChatService } from '@/services/chat.service';
import type { ChatUserProfile, ChatUserId } from '@/features/chat/types';
import { getCidUrl } from '@/services/ipfs.service';

export function AddContactScreen() {
  const router = useRouter();
  const insets = useSafeAreaInsets();
  const { addContact, sendFriendRequest, groups } = useContactsStore();

  const [searchId, setSearchId] = useState('');
  const [isSearching, setIsSearching] = useState(false);
  const [result, setResult] = useState<ChatUserProfile | null>(null);
  const [notFound, setNotFound] = useState(false);

  const [alias, setAlias] = useState('');
  const [message, setMessage] = useState('');
  const [selectedGroups, setSelectedGroups] = useState<string[]>([]);
  const [isAdding, setIsAdding] = useState(false);

  const handleBack = useCallback(() => {
    router.back();
  }, [router]);

  const handleSearch = useCallback(async () => {
    const id = parseInt(searchId, 10);

    if (isNaN(id) || id < 10_000_000_000 || id > 99_999_999_999) {
      Alert.alert('提示', '请输入有效的 11 位聊天 ID');
      return;
    }

    setIsSearching(true);
    setNotFound(false);
    setResult(null);

    try {
      const chatService = getChatService();
      const profile = await chatService.getUserProfile(id as ChatUserId);
      if (profile) {
        setResult(profile);
      } else {
        setNotFound(true);
      }
    } catch (error) {
      Alert.alert('搜索失败', (error as Error).message);
    } finally {
      setIsSearching(false);
    }
  }, [searchId]);

  const handleAddContact = useCallback(async () => {
    if (!result) return;

    setIsAdding(true);
    try {
      // 添加联系人
      await addContact(result.accountId, alias || undefined, selectedGroups);

      // 发送好友申请
      if (message) {
        await sendFriendRequest(result.accountId, message);
      }

      Alert.alert('成功', '已添加联系人', [
        { text: '确定', onPress: () => router.back() },
      ]);
    } catch (error) {
      Alert.alert('失败', (error as Error).message);
    } finally {
      setIsAdding(false);
    }
  }, [result, alias, selectedGroups, message, addContact, sendFriendRequest, router]);

  const toggleGroup = (groupName: string) => {
    setSelectedGroups((prev) =>
      prev.includes(groupName)
        ? prev.filter((g) => g !== groupName)
        : [...prev, groupName]
    );
  };

  const avatarUri = result?.avatarCid ? getCidUrl(result.avatarCid) : null;

  return (
    <View style={[styles.container, { paddingTop: insets.top }]}>
      <View style={styles.header}>
        <TouchableOpacity style={styles.backBtn} onPress={handleBack}>
          <Ionicons name="chevron-back" size={28} color="#007AFF" />
        </TouchableOpacity>
        <Text style={styles.title}>添加联系人</Text>
        <View style={styles.placeholder} />
      </View>

      <ScrollView style={styles.content}>
        {/* 搜索框 */}
        <View style={styles.searchSection}>
          <Text style={styles.sectionTitle}>搜索用户</Text>
          <View style={styles.searchBox}>
            <TextInput
              style={styles.searchInput}
              value={searchId}
              onChangeText={setSearchId}
              placeholder="输入 11 位聊天 ID"
              keyboardType="number-pad"
              maxLength={11}
            />
            <TouchableOpacity
              style={[
                styles.searchBtn,
                (isSearching || searchId.length !== 11) &&
                  styles.searchBtnDisabled,
              ]}
              onPress={handleSearch}
              disabled={isSearching || searchId.length !== 11}
            >
              {isSearching ? (
                <ActivityIndicator color="#fff" size="small" />
              ) : (
                <Text style={styles.searchBtnText}>搜索</Text>
              )}
            </TouchableOpacity>
          </View>
        </View>

        {/* 搜索结果 */}
        {result && (
          <View style={styles.resultSection}>
            <View style={styles.profileCard}>
              <View style={styles.avatar}>
                <Text style={styles.avatarText}>
                  {(result.nickname || result.chatUserId.toString())
                    .slice(0, 2)
                    .toUpperCase()}
                </Text>
              </View>
              <View style={styles.profileInfo}>
                <Text style={styles.nickname}>
                  {result.nickname || `用户${result.chatUserId}`}
                </Text>
                <Text style={styles.chatId}>ID: {result.chatUserId}</Text>
              </View>
            </View>

            {/* 备注名 */}
            <View style={styles.inputGroup}>
              <Text style={styles.inputLabel}>备注名（可选）</Text>
              <TextInput
                style={styles.input}
                value={alias}
                onChangeText={setAlias}
                placeholder="给联系人设置备注名"
                maxLength={20}
              />
            </View>

            {/* 好友申请留言 */}
            <View style={styles.inputGroup}>
              <Text style={styles.inputLabel}>申请留言（可选）</Text>
              <TextInput
                style={[styles.input, styles.messageInput]}
                value={message}
                onChangeText={setMessage}
                placeholder="向对方介绍一下自己"
                maxLength={100}
                multiline
              />
            </View>

            {/* 分组选择 */}
            {groups.length > 0 && (
              <View style={styles.inputGroup}>
                <Text style={styles.inputLabel}>添加到分组</Text>
                <View style={styles.groupList}>
                  {groups.map((group) => (
                    <TouchableOpacity
                      key={group.name}
                      style={[
                        styles.groupTag,
                        selectedGroups.includes(group.name) &&
                          styles.groupTagSelected,
                      ]}
                      onPress={() => toggleGroup(group.name)}
                    >
                      <Text
                        style={[
                          styles.groupTagText,
                          selectedGroups.includes(group.name) &&
                            styles.groupTagTextSelected,
                        ]}
                      >
                        {group.name}
                      </Text>
                    </TouchableOpacity>
                  ))}
                </View>
              </View>
            )}

            {/* 添加按钮 */}
            <TouchableOpacity
              style={[styles.addBtn, isAdding && styles.addBtnDisabled]}
              onPress={handleAddContact}
              disabled={isAdding}
            >
              {isAdding ? (
                <ActivityIndicator color="#fff" />
              ) : (
                <Text style={styles.addBtnText}>添加联系人</Text>
              )}
            </TouchableOpacity>
          </View>
        )}

        {/* 未找到 */}
        {notFound && (
          <View style={styles.notFound}>
            <Ionicons name="person-outline" size={48} color="#ccc" />
            <Text style={styles.notFoundText}>未找到该用户</Text>
          </View>
        )}
      </ScrollView>
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
    paddingVertical: 10,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#e0e0e0',
  },
  backBtn: {
    padding: 8,
    marginLeft: 4,
  },
  title: {
    fontSize: 17,
    fontWeight: '600',
    color: '#333',
  },
  placeholder: {
    width: 44,
  },
  content: {
    flex: 1,
  },
  searchSection: {
    backgroundColor: '#fff',
    padding: 16,
    marginTop: 12,
  },
  sectionTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginBottom: 12,
  },
  searchBox: {
    flexDirection: 'row',
  },
  searchInput: {
    flex: 1,
    height: 44,
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 16,
    fontSize: 16,
  },
  searchBtn: {
    marginLeft: 12,
    backgroundColor: '#007AFF',
    paddingHorizontal: 20,
    borderRadius: 8,
    justifyContent: 'center',
  },
  searchBtnDisabled: {
    backgroundColor: '#ccc',
  },
  searchBtnText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '500',
  },
  resultSection: {
    backgroundColor: '#fff',
    marginTop: 12,
    padding: 16,
  },
  profileCard: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingBottom: 16,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#e0e0e0',
    marginBottom: 16,
  },
  avatar: {
    width: 56,
    height: 56,
    borderRadius: 28,
    backgroundColor: '#007AFF',
    justifyContent: 'center',
    alignItems: 'center',
  },
  avatarText: {
    color: '#fff',
    fontSize: 20,
    fontWeight: '600',
  },
  profileInfo: {
    marginLeft: 12,
  },
  nickname: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  chatId: {
    fontSize: 13,
    color: '#666',
    marginTop: 2,
  },
  inputGroup: {
    marginBottom: 16,
  },
  inputLabel: {
    fontSize: 14,
    color: '#666',
    marginBottom: 8,
  },
  input: {
    height: 44,
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 16,
    fontSize: 16,
  },
  messageInput: {
    height: 80,
    paddingTop: 12,
    textAlignVertical: 'top',
  },
  groupList: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  groupTag: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
    backgroundColor: '#f5f5f5',
  },
  groupTagSelected: {
    backgroundColor: '#007AFF',
  },
  groupTagText: {
    fontSize: 14,
    color: '#666',
  },
  groupTagTextSelected: {
    color: '#fff',
  },
  addBtn: {
    backgroundColor: '#007AFF',
    height: 48,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
    marginTop: 8,
  },
  addBtnDisabled: {
    backgroundColor: '#ccc',
  },
  addBtnText: {
    color: '#fff',
    fontSize: 17,
    fontWeight: '600',
  },
  notFound: {
    alignItems: 'center',
    marginTop: 60,
  },
  notFoundText: {
    fontSize: 16,
    color: '#666',
    marginTop: 12,
  },
});
