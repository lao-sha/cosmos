import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { UserPlus, Search } from 'lucide-react-native';
import * as Haptics from 'expo-haptics';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import { sendFriendRequest } from '@/services/friends';
import { Button, Input, Card } from '@/components/ui';
import { Colors } from '@/constants/colors';

export default function AddFriendScreen() {
  const colors = useColors();
  const router = useRouter();
  const { mnemonic } = useWalletStore();

  const [searchText, setSearchText] = useState('');
  const [message, setMessage] = useState('');
  const [sending, setSending] = useState(false);

  const handleSend = async () => {
    if (!searchText.trim() || !mnemonic) return;

    setSending(true);
    try {
      await sendFriendRequest(searchText.trim(), message.trim(), mnemonic);
      Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
      Alert.alert('发送成功', '好友请求已发送', [
        { text: '确定', onPress: () => router.back() },
      ]);
    } catch (error: any) {
      Alert.alert('发送失败', error.message);
    } finally {
      setSending(false);
    }
  };

  return (
    <ScrollView
      style={[styles.container, { backgroundColor: colors.background }]}
      contentContainerStyle={styles.content}
      keyboardShouldPersistTaps="handled"
    >
      <View style={styles.iconContainer}>
        <UserPlus size={48} color={Colors.primary} />
      </View>
      <Text style={[styles.title, { color: colors.textPrimary }]}>
        添加好友
      </Text>
      <Text style={[styles.subtitle, { color: colors.textSecondary }]}>
        输入对方的钱包地址或用户ID
      </Text>

      <Card style={styles.card}>
        <Input
          label="钱包地址/用户ID"
          placeholder="输入钱包地址或用户ID"
          value={searchText}
          onChangeText={setSearchText}
          autoCapitalize="none"
        />
        <Input
          label="验证消息（可选）"
          placeholder="向对方介绍一下自己"
          value={message}
          onChangeText={setMessage}
          multiline
          numberOfLines={3}
          maxLength={100}
        />
        <Text style={[styles.hint, { color: colors.textTertiary }]}>
          {message.length}/100
        </Text>
      </Card>

      <Button
        title="发送好友请求"
        onPress={handleSend}
        loading={sending}
        disabled={!searchText.trim()}
        style={styles.button}
      />

      <Card style={[styles.tipsCard, { backgroundColor: colors.surface }]}>
        <Text style={[styles.tipsTitle, { color: colors.textPrimary }]}>
          💡 小提示
        </Text>
        <Text style={[styles.tipsText, { color: colors.textSecondary }]}>
          • 可以通过交易记录中的商家添加好友{'\n'}
          • 添加好友后可以直接发起聊天{'\n'}
          • 好友之间的交易更加安全可靠
        </Text>
      </Card>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    padding: 16,
  },
  iconContainer: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: 'rgba(99, 102, 241, 0.1)',
    alignItems: 'center',
    justifyContent: 'center',
    alignSelf: 'center',
    marginTop: 20,
    marginBottom: 20,
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    textAlign: 'center',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 14,
    textAlign: 'center',
    marginBottom: 24,
  },
  card: {
    marginBottom: 24,
  },
  hint: {
    fontSize: 12,
    textAlign: 'right',
    marginTop: -8,
  },
  button: {},
  tipsCard: {
    marginTop: 24,
  },
  tipsTitle: {
    fontSize: 14,
    fontWeight: '600',
    marginBottom: 8,
  },
  tipsText: {
    fontSize: 13,
    lineHeight: 22,
  },
});
