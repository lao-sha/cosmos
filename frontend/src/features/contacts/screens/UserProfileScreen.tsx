/**
 * 用户资料页面
 */

import React, { useEffect, useCallback, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  Image,
  ScrollView,
  Alert,
  Switch,
  ActivityIndicator,
} from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { getChatService } from '@/services/chat.service';
import { useContactsStore } from '@/stores/contacts.store';
import { useChatStore } from '@/stores/chat.store';
import type { ChatUserProfile, ChatUserId, UserStatus } from '@/features/chat/types';
import { FriendStatus } from '@/features/contacts/types';
import { getCidUrl } from '@/services/ipfs.service';

const STATUS_TEXT: Record<UserStatus, string> = {
  [UserStatus.Online]: '在线',
  [UserStatus.Offline]: '离线',
  [UserStatus.Busy]: '忙碌',
  [UserStatus.Away]: '离开',
  [UserStatus.Invisible]: '隐身',
};

const STATUS_COLORS: Record<UserStatus, string> = {
  [UserStatus.Online]: '#34C759',
  [UserStatus.Offline]: '#8E8E93',
  [UserStatus.Busy]: '#FF3B30',
  [UserStatus.Away]: '#FF9500',
  [UserStatus.Invisible]: '#8E8E93',
};

export function UserProfileScreen() {
  const { address } = useLocalSearchParams<{ address: string }>();
  const router = useRouter();
  const insets = useSafeAreaInsets();

  const { contacts, addContact, removeContact, blockUser } = useContactsStore();
  const { blockUser: chatBlockUser } = useChatStore();

  const [profile, setProfile] = useState<ChatUserProfile | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const contact = contacts.find((c) => c.address === address);
  const isFriend = contact?.friendStatus === FriendStatus.Mutual;
  const isContact = !!contact;

  useEffect(() => {
    loadProfile();
  }, [address]);

  const loadProfile = async () => {
    if (!address) return;

    setIsLoading(true);
    try {
      const chatService = getChatService();
      // 先获取 ChatUserId
      // TODO: 需要从链上查询地址对应的 ChatUserId
      // 这里暂时使用模拟数据
      setProfile(null);
    } catch (error) {
      console.error('Failed to load profile:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleBack = useCallback(() => {
    router.back();
  }, [router]);

  const handleStartChat = useCallback(() => {
    router.push(`/chat/${address}`);
  }, [router, address]);

  const handleAddContact = useCallback(async () => {
    if (!address) return;

    try {
      await addContact(address);
      Alert.alert('成功', '已添加到联系人');
    } catch (error) {
      Alert.alert('失败', (error as Error).message);
    }
  }, [address, addContact]);

  const handleRemoveContact = useCallback(async () => {
    if (!address) return;

    Alert.alert('确认删除', '确定要删除该联系人吗？', [
      { text: '取消', style: 'cancel' },
      {
        text: '删除',
        style: 'destructive',
        onPress: async () => {
          try {
            await removeContact(address);
            Alert.alert('成功', '已删除联系人');
          } catch (error) {
            Alert.alert('失败', (error as Error).message);
          }
        },
      },
    ]);
  }, [address, removeContact]);

  const handleBlock = useCallback(async () => {
    if (!address) return;

    Alert.alert('确认拉黑', '拉黑后将无法收到对方的消息', [
      { text: '取消', style: 'cancel' },
      {
        text: '拉黑',
        style: 'destructive',
        onPress: async () => {
          try {
            await blockUser(address);
            await chatBlockUser(address);
            Alert.alert('成功', '已拉黑该用户');
            router.back();
          } catch (error) {
            Alert.alert('失败', (error as Error).message);
          }
        },
      },
    ]);
  }, [address, blockUser, chatBlockUser, router]);

  const avatarUri = profile?.avatarCid ? getCidUrl(profile.avatarCid) : null;
  const displayName =
    contact?.alias ||
    profile?.nickname ||
    (profile?.chatUserId ? `用户${profile.chatUserId}` : address?.slice(0, 12) + '...');

  if (isLoading) {
    return (
      <View style={[styles.container, styles.loading, { paddingTop: insets.top }]}>
        <ActivityIndicator size="large" color="#007AFF" />
      </View>
    );
  }

  return (
    <View style={[styles.container, { paddingTop: insets.top }]}>
      <View style={styles.header}>
        <TouchableOpacity style={styles.backBtn} onPress={handleBack}>
          <Ionicons name="chevron-back" size={28} color="#007AFF" />
        </TouchableOpacity>
        <Text style={styles.headerTitle}>用户资料</Text>
        <TouchableOpacity style={styles.moreBtn}>
          <Ionicons name="ellipsis-horizontal" size={24} color="#007AFF" />
        </TouchableOpacity>
      </View>

      <ScrollView style={styles.content}>
        {/* 头像和基本信息 */}
        <View style={styles.profileSection}>
          <View style={styles.avatarContainer}>
            {avatarUri ? (
              <Image source={{ uri: avatarUri }} style={styles.avatar} />
            ) : (
              <View style={[styles.avatar, styles.avatarPlaceholder]}>
                <Text style={styles.avatarText}>
                  {displayName.slice(0, 2).toUpperCase()}
                </Text>
              </View>
            )}
            {profile?.privacySettings?.showOnlineStatus && profile?.status && (
              <View
                style={[
                  styles.statusDot,
                  { backgroundColor: STATUS_COLORS[profile.status] },
                ]}
              />
            )}
          </View>

          <Text style={styles.nickname}>{displayName}</Text>

          {profile?.chatUserId && (
            <Text style={styles.chatId}>ID: {profile.chatUserId}</Text>
          )}

          {profile?.signature && (
            <Text style={styles.signature}>{profile.signature}</Text>
          )}

          {profile?.privacySettings?.showOnlineStatus && profile?.status && (
            <View style={styles.statusRow}>
              <View
                style={[
                  styles.statusIndicator,
                  { backgroundColor: STATUS_COLORS[profile.status] },
                ]}
              />
              <Text style={styles.statusText}>
                {STATUS_TEXT[profile.status]}
              </Text>
            </View>
          )}
        </View>

        {/* 操作按钮 */}
        <View style={styles.actionSection}>
          <TouchableOpacity style={styles.actionBtn} onPress={handleStartChat}>
            <Ionicons name="chatbubble" size={24} color="#fff" />
            <Text style={styles.actionBtnText}>发消息</Text>
          </TouchableOpacity>

          {!isContact ? (
            <TouchableOpacity
              style={[styles.actionBtn, styles.actionBtnSecondary]}
              onPress={handleAddContact}
            >
              <Ionicons name="person-add" size={24} color="#007AFF" />
              <Text style={[styles.actionBtnText, styles.actionBtnTextSecondary]}>
                添加联系人
              </Text>
            </TouchableOpacity>
          ) : (
            <TouchableOpacity
              style={[styles.actionBtn, styles.actionBtnSecondary]}
              onPress={() => router.push(`/contacts/edit/${address}`)}
            >
              <Ionicons name="create" size={24} color="#007AFF" />
              <Text style={[styles.actionBtnText, styles.actionBtnTextSecondary]}>
                编辑备注
              </Text>
            </TouchableOpacity>
          )}
        </View>

        {/* 详细信息 */}
        <View style={styles.infoSection}>
          <Text style={styles.sectionTitle}>详细信息</Text>

          <View style={styles.infoItem}>
            <Text style={styles.infoLabel}>地址</Text>
            <Text style={styles.infoValue} numberOfLines={1}>
              {address}
            </Text>
          </View>

          {contact && (
            <>
              <View style={styles.infoItem}>
                <Text style={styles.infoLabel}>好友状态</Text>
                <Text style={styles.infoValue}>
                  {isFriend ? '双向好友' : '单向添加'}
                </Text>
              </View>

              {contact.groups.length > 0 && (
                <View style={styles.infoItem}>
                  <Text style={styles.infoLabel}>所属分组</Text>
                  <Text style={styles.infoValue}>
                    {contact.groups.join(', ')}
                  </Text>
                </View>
              )}
            </>
          )}
        </View>

        {/* 危险操作 */}
        <View style={styles.dangerSection}>
          {isContact && (
            <TouchableOpacity
              style={styles.dangerBtn}
              onPress={handleRemoveContact}
            >
              <Ionicons name="person-remove" size={20} color="#FF3B30" />
              <Text style={styles.dangerBtnText}>删除联系人</Text>
            </TouchableOpacity>
          )}

          <TouchableOpacity style={styles.dangerBtn} onPress={handleBlock}>
            <Ionicons name="ban" size={20} color="#FF3B30" />
            <Text style={styles.dangerBtnText}>拉黑用户</Text>
          </TouchableOpacity>
        </View>
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  loading: {
    justifyContent: 'center',
    alignItems: 'center',
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
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#333',
  },
  moreBtn: {
    padding: 8,
    marginRight: 4,
  },
  content: {
    flex: 1,
  },
  profileSection: {
    backgroundColor: '#fff',
    alignItems: 'center',
    paddingVertical: 24,
    paddingHorizontal: 16,
  },
  avatarContainer: {
    position: 'relative',
  },
  avatar: {
    width: 80,
    height: 80,
    borderRadius: 40,
  },
  avatarPlaceholder: {
    backgroundColor: '#007AFF',
    justifyContent: 'center',
    alignItems: 'center',
  },
  avatarText: {
    color: '#fff',
    fontSize: 28,
    fontWeight: '600',
  },
  statusDot: {
    position: 'absolute',
    bottom: 2,
    right: 2,
    width: 16,
    height: 16,
    borderRadius: 8,
    borderWidth: 3,
    borderColor: '#fff',
  },
  nickname: {
    fontSize: 22,
    fontWeight: '600',
    color: '#333',
    marginTop: 12,
  },
  chatId: {
    fontSize: 14,
    color: '#666',
    marginTop: 4,
  },
  signature: {
    fontSize: 14,
    color: '#999',
    marginTop: 8,
    textAlign: 'center',
  },
  statusRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 8,
  },
  statusIndicator: {
    width: 8,
    height: 8,
    borderRadius: 4,
    marginRight: 6,
  },
  statusText: {
    fontSize: 13,
    color: '#666',
  },
  actionSection: {
    flexDirection: 'row',
    padding: 16,
    gap: 12,
    backgroundColor: '#fff',
    marginTop: 12,
  },
  actionBtn: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#007AFF',
    paddingVertical: 12,
    borderRadius: 8,
    gap: 8,
  },
  actionBtnSecondary: {
    backgroundColor: '#E3F2FD',
  },
  actionBtnText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  actionBtnTextSecondary: {
    color: '#007AFF',
  },
  infoSection: {
    backgroundColor: '#fff',
    marginTop: 12,
    padding: 16,
  },
  sectionTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#999',
    marginBottom: 12,
  },
  infoItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 12,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#e0e0e0',
  },
  infoLabel: {
    fontSize: 15,
    color: '#333',
  },
  infoValue: {
    fontSize: 15,
    color: '#666',
    flex: 1,
    textAlign: 'right',
    marginLeft: 16,
  },
  dangerSection: {
    backgroundColor: '#fff',
    marginTop: 12,
    marginBottom: 24,
  },
  dangerBtn: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 16,
    gap: 12,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#e0e0e0',
  },
  dangerBtnText: {
    fontSize: 16,
    color: '#FF3B30',
  },
});
