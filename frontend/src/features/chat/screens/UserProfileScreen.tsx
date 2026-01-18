/**
 * 用户资料查看页面
 * 可以查看自己或他人的聊天资料
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  ActivityIndicator,
  Alert,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useUserStore } from '@/stores/user.store';
import { useContactsStore } from '@/stores/contacts.store';
import { useChatStore } from '@/stores/chat.store';
import type { ChatUserProfile, UserStatus } from '../types';

const THEME_COLOR = '#B2955D';

export function UserProfileScreen() {
  const router = useRouter();
  const params = useLocalSearchParams();
  const chatUserId = params.chatUserId ? Number(params.chatUserId) : null;
  const address = params.address as string | undefined;

  const { myProfile, myChatUserId, searchUser } = useUserStore();
  const { contacts, addContact, removeContact, blockUser } = useContactsStore();
  const { sessions } = useChatStore();

  const [profile, setProfile] = useState<ChatUserProfile | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isContact, setIsContact] = useState(false);

  const isMyProfile = chatUserId === myChatUserId;

  useEffect(() => {
    loadProfile();
  }, [chatUserId, address]);

  useEffect(() => {
    if (profile) {
      const contact = contacts.find(
        (c) => c.address === profile.accountId
      );
      setIsContact(!!contact);
    }
  }, [profile, contacts]);

  const loadProfile = async () => {
    setIsLoading(true);
    try {
      if (isMyProfile) {
        setProfile(myProfile);
      } else if (chatUserId) {
        const userProfile = await searchUser(chatUserId);
        setProfile(userProfile);
      }
    } catch (error) {
      Alert.alert('错误', '加载用户资料失败');
    } finally {
      setIsLoading(false);
    }
  };

  const handleSendMessage = () => {
    if (!profile) return;
    router.push(`/chat/${profile.accountId}`);
  };

  const handleAddContact = async () => {
    if (!profile) return;
    try {
      await addContact(profile.accountId, profile.nickname);
      Alert.alert('成功', '已添加到联系人');
    } catch (error) {
      Alert.alert('错误', '添加联系人失败');
    }
  };

  const handleRemoveContact = async () => {
    if (!profile) return;
    Alert.alert('删除联系人', '确定要删除此联系人吗？', [
      { text: '取消', style: 'cancel' },
      {
        text: '删除',
        style: 'destructive',
        onPress: async () => {
          try {
            await removeContact(profile.accountId);
            Alert.alert('成功', '已删除联系人');
          } catch (error) {
            Alert.alert('错误', '删除联系人失败');
          }
        },
      },
    ]);
  };

  const handleBlockUser = async () => {
    if (!profile) return;
    Alert.alert('拉黑用户', '确定要拉黑此用户吗？', [
      { text: '取消', style: 'cancel' },
      {
        text: '拉黑',
        style: 'destructive',
        onPress: async () => {
          try {
            await blockUser(profile.accountId);
            Alert.alert('成功', '已拉黑用户');
            router.back();
          } catch (error) {
            Alert.alert('错误', '拉黑用户失败');
          }
        },
      },
    ]);
  };

  const getStatusText = (status: UserStatus) => {
    switch (status) {
      case UserStatus.Online:
        return '在线';
      case UserStatus.Offline:
        return '离线';
      case UserStatus.Busy:
        return '忙碌';
      case UserStatus.Away:
        return '离开';
      case UserStatus.Invisible:
        return '隐身';
      default:
        return '未知';
    }
  };

  const getStatusColor = (status: UserStatus) => {
    switch (status) {
      case UserStatus.Online:
        return '#52c41a';
      case UserStatus.Busy:
        return '#faad14';
      case UserStatus.Away:
        return '#ff7a45';
      default:
        return '#999';
    }
  };

  if (isLoading) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backBtn} onPress={() => router.back()}>
            <Ionicons name="chevron-back" size={24} color="#333" />
          </Pressable>
          <Text style={styles.headerTitle}>用户资料</Text>
          <View style={styles.backBtn} />
        </View>
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
      </View>
    );
  }

  if (!profile) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backBtn} onPress={() => router.back()}>
            <Ionicons name="chevron-back" size={24} color="#333" />
          </Pressable>
          <Text style={styles.headerTitle}>用户资料</Text>
          <View style={styles.backBtn} />
        </View>
        <View style={styles.emptyContainer}>
          <Ionicons name="person-outline" size={64} color="#ccc" />
          <Text style={styles.emptyText}>用户不存在</Text>
        </View>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backBtn} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Text style={styles.headerTitle}>用户资料</Text>
        {isMyProfile && (
          <Pressable
            style={styles.editBtn}
            onPress={() => router.push('/profile/edit')}
          >
            <Text style={styles.editText}>编辑</Text>
          </Pressable>
        )}
        {!isMyProfile && <View style={styles.backBtn} />}
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 头像和基本信息 */}
        <View style={styles.profileHeader}>
          <View style={styles.avatarContainer}>
            <View style={styles.avatar}>
              <Text style={styles.avatarText}>
                {profile.nickname?.charAt(0) || '?'}
              </Text>
            </View>
            {profile.privacySettings.showOnlineStatus && (
              <View
                style={[
                  styles.statusDot,
                  { backgroundColor: getStatusColor(profile.status) },
                ]}
              />
            )}
          </View>

          <Text style={styles.nickname}>
            {profile.nickname || '未设置昵称'}
          </Text>

          <View style={styles.chatIdRow}>
            <Text style={styles.chatIdLabel}>聊天ID:</Text>
            <Text style={styles.chatId}>{profile.chatUserId}</Text>
          </View>

          {profile.signature && (
            <Text style={styles.signature}>{profile.signature}</Text>
          )}

          {profile.privacySettings.showOnlineStatus && (
            <View style={styles.statusBadge}>
              <View
                style={[
                  styles.statusIndicator,
                  { backgroundColor: getStatusColor(profile.status) },
                ]}
              />
              <Text style={styles.statusText}>
                {getStatusText(profile.status)}
              </Text>
            </View>
          )}
        </View>

        {/* 详细信息 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>详细信息</Text>
          <View style={styles.card}>
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>钱包地址</Text>
              <Text style={styles.infoValue} numberOfLines={1}>
                {profile.accountId.slice(0, 8)}...{profile.accountId.slice(-6)}
              </Text>
            </View>

            <View style={styles.divider} />

            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>注册时间</Text>
              <Text style={styles.infoValue}>
                {new Date(profile.createdAt * 1000).toLocaleDateString()}
              </Text>
            </View>

            {profile.privacySettings.showLastActive && (
              <>
                <View style={styles.divider} />
                <View style={styles.infoRow}>
                  <Text style={styles.infoLabel}>最后活跃</Text>
                  <Text style={styles.infoValue}>
                    {new Date(profile.lastActive * 1000).toLocaleString()}
                  </Text>
                </View>
              </>
            )}
          </View>
        </View>

        {/* 操作按钮 */}
        {!isMyProfile && (
          <View style={styles.actions}>
            <Pressable style={styles.primaryBtn} onPress={handleSendMessage}>
              <Ionicons name="chatbubble-outline" size={20} color="#FFF" />
              <Text style={styles.primaryBtnText}>发送消息</Text>
            </Pressable>

            {isContact ? (
              <Pressable style={styles.secondaryBtn} onPress={handleRemoveContact}>
                <Ionicons name="person-remove-outline" size={20} color={THEME_COLOR} />
                <Text style={styles.secondaryBtnText}>删除联系人</Text>
              </Pressable>
            ) : (
              <Pressable style={styles.secondaryBtn} onPress={handleAddContact}>
                <Ionicons name="person-add-outline" size={20} color={THEME_COLOR} />
                <Text style={styles.secondaryBtnText}>添加联系人</Text>
              </Pressable>
            )}

            <Pressable style={styles.dangerBtn} onPress={handleBlockUser}>
              <Ionicons name="ban-outline" size={20} color="#E74C3C" />
              <Text style={styles.dangerBtnText}>拉黑</Text>
            </Pressable>
          </View>
        )}

        <View style={{ height: 40 }} />
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingTop: 50,
    paddingHorizontal: 16,
    paddingBottom: 12,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  backBtn: {
    padding: 4,
    minWidth: 50,
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#333',
  },
  editBtn: {
    padding: 4,
  },
  editText: {
    fontSize: 16,
    color: THEME_COLOR,
    fontWeight: '600',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    gap: 16,
  },
  emptyText: {
    fontSize: 16,
    color: '#999',
  },
  content: {
    flex: 1,
  },
  profileHeader: {
    backgroundColor: '#FFF',
    alignItems: 'center',
    paddingVertical: 32,
    paddingHorizontal: 24,
    marginBottom: 16,
  },
  avatarContainer: {
    position: 'relative',
    marginBottom: 16,
  },
  avatar: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: THEME_COLOR,
    justifyContent: 'center',
    alignItems: 'center',
  },
  avatarText: {
    fontSize: 32,
    color: '#FFF',
    fontWeight: 'bold',
  },
  statusDot: {
    position: 'absolute',
    bottom: 2,
    right: 2,
    width: 16,
    height: 16,
    borderRadius: 8,
    borderWidth: 2,
    borderColor: '#FFF',
  },
  nickname: {
    fontSize: 22,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  chatIdRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    marginBottom: 12,
  },
  chatIdLabel: {
    fontSize: 14,
    color: '#999',
  },
  chatId: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME_COLOR,
    fontFamily: 'monospace',
  },
  signature: {
    fontSize: 14,
    color: '#666',
    textAlign: 'center',
    marginBottom: 12,
  },
  statusBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    paddingHorizontal: 12,
    paddingVertical: 6,
    backgroundColor: '#F5F5F5',
    borderRadius: 12,
  },
  statusIndicator: {
    width: 8,
    height: 8,
    borderRadius: 4,
  },
  statusText: {
    fontSize: 13,
    color: '#666',
  },
  section: {
    marginBottom: 16,
    paddingHorizontal: 16,
  },
  sectionTitle: {
    fontSize: 14,
    fontWeight: '500',
    color: '#666',
    marginBottom: 8,
  },
  card: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
  },
  infoRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 12,
  },
  infoLabel: {
    fontSize: 15,
    color: '#666',
  },
  infoValue: {
    fontSize: 15,
    color: '#333',
    fontWeight: '500',
    flex: 1,
    textAlign: 'right',
    marginLeft: 16,
  },
  divider: {
    height: 1,
    backgroundColor: '#F5F5F5',
  },
  actions: {
    paddingHorizontal: 16,
    gap: 12,
  },
  primaryBtn: {
    backgroundColor: THEME_COLOR,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 8,
    paddingVertical: 14,
    borderRadius: 8,
  },
  primaryBtnText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFF',
  },
  secondaryBtn: {
    backgroundColor: '#FFF',
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 8,
    paddingVertical: 14,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: THEME_COLOR,
  },
  secondaryBtnText: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME_COLOR,
  },
  dangerBtn: {
    backgroundColor: '#FFF',
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 8,
    paddingVertical: 14,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#FFEBEE',
  },
  dangerBtnText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#E74C3C',
  },
});
