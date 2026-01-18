/**
 * 新建会话页面
 * 通过 ChatUserId 搜索用户并发起聊天
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
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { getChatService } from '@/services/chat.service';
import type { ChatUserProfile, ChatUserId } from '@/features/chat/types';
import { getCidUrl } from '@/services/ipfs.service';

export default function NewChatPage() {
  const router = useRouter();
  const insets = useSafeAreaInsets();

  const [searchId, setSearchId] = useState('');
  const [isSearching, setIsSearching] = useState(false);
  const [result, setResult] = useState<ChatUserProfile | null>(null);
  const [notFound, setNotFound] = useState(false);

  const handleSearch = useCallback(async () => {
    const id = parseInt(searchId, 10);

    // 验证 11 位数字
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

  const handleStartChat = useCallback(async () => {
    if (!result) return;

    try {
      // 直接跳转到聊天页面，会话会在发送第一条消息时自动创建
      router.replace(`/chat/${result.accountId}`);
    } catch (error) {
      Alert.alert('错误', '无法开始聊天');
    }
  }, [result, router]);

  const handleBack = useCallback(() => {
    router.back();
  }, [router]);

  const avatarUri = result?.avatarCid ? getCidUrl(result.avatarCid) : null;

  return (
    <View style={[styles.container, { paddingTop: insets.top }]}>
      <View style={styles.header}>
        <TouchableOpacity style={styles.backBtn} onPress={handleBack}>
          <Ionicons name="chevron-back" size={28} color="#007AFF" />
        </TouchableOpacity>
        <Text style={styles.title}>新建会话</Text>
        <View style={styles.placeholder} />
      </View>

      <View style={styles.searchBox}>
        <TextInput
          style={styles.input}
          value={searchId}
          onChangeText={setSearchId}
          placeholder="输入 11 位聊天 ID"
          keyboardType="number-pad"
          maxLength={11}
          autoFocus
        />
        <TouchableOpacity
          style={[
            styles.searchBtn,
            (isSearching || searchId.length !== 11) && styles.searchBtnDisabled,
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

      {result && (
        <View style={styles.resultContainer}>
          <View style={styles.profileCard}>
            <View style={styles.avatar}>
              {avatarUri ? (
                <View style={styles.avatarImage} />
              ) : (
                <Text style={styles.avatarText}>
                  {(result.nickname || result.chatUserId.toString())
                    .slice(0, 2)
                    .toUpperCase()}
                </Text>
              )}
            </View>

            <View style={styles.profileInfo}>
              <Text style={styles.nickname}>
                {result.nickname || `用户${result.chatUserId}`}
              </Text>
              <Text style={styles.chatId}>ID: {result.chatUserId}</Text>
              {result.signature && (
                <Text style={styles.signature} numberOfLines={1}>
                  {result.signature}
                </Text>
              )}
            </View>
          </View>

          <TouchableOpacity style={styles.chatBtn} onPress={handleStartChat}>
            <Text style={styles.chatBtnText}>发起聊天</Text>
          </TouchableOpacity>
        </View>
      )}

      {notFound && (
        <View style={styles.notFound}>
          <Ionicons name="person-outline" size={48} color="#ccc" />
          <Text style={styles.notFoundText}>未找到该用户</Text>
          <Text style={styles.notFoundSubtext}>请检查 ID 是否正确</Text>
        </View>
      )}

      <View style={styles.tips}>
        <Text style={styles.tipsTitle}>提示</Text>
        <Text style={styles.tipsText}>
          • 聊天 ID 是 11 位数字，类似 QQ 号{'\n'}
          • 可以在个人资料页查看自己的聊天 ID{'\n'}
          • 首次聊天需要对方允许陌生人消息
        </Text>
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
  searchBox: {
    flexDirection: 'row',
    padding: 16,
    backgroundColor: '#fff',
    marginTop: 12,
  },
  input: {
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
  resultContainer: {
    marginTop: 12,
    backgroundColor: '#fff',
  },
  profileCard: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 16,
  },
  avatar: {
    width: 56,
    height: 56,
    borderRadius: 28,
    backgroundColor: '#007AFF',
    justifyContent: 'center',
    alignItems: 'center',
  },
  avatarImage: {
    width: 56,
    height: 56,
    borderRadius: 28,
  },
  avatarText: {
    color: '#fff',
    fontSize: 20,
    fontWeight: '600',
  },
  profileInfo: {
    flex: 1,
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
  signature: {
    fontSize: 14,
    color: '#999',
    marginTop: 4,
  },
  chatBtn: {
    margin: 16,
    marginTop: 0,
    backgroundColor: '#007AFF',
    height: 44,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  chatBtnText: {
    color: '#fff',
    fontSize: 16,
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
  notFoundSubtext: {
    fontSize: 14,
    color: '#999',
    marginTop: 4,
  },
  tips: {
    margin: 16,
    padding: 16,
    backgroundColor: '#fff',
    borderRadius: 8,
  },
  tipsTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  tipsText: {
    fontSize: 13,
    color: '#666',
    lineHeight: 20,
  },
});
