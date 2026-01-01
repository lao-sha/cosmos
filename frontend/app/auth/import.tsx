/**
 * 星尘玄鉴 - 导入钱包
 * 主题色：金棕色 #B2955D
 */

import { useState } from 'react';
import {
  View,
  Text,
  TextInput,
  StyleSheet,
  Pressable,
  ScrollView,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useWalletStore } from '@/stores';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

export default function ImportWalletPage() {
  const router = useRouter();
  const { importWallet, isLoading, error } = useWalletStore();

  const [mnemonic, setMnemonic] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');

  const handleImport = async () => {
    if (!mnemonic.trim()) {
      Alert.alert('错误', '请输入助记词');
      return;
    }

    if (password.length < 8) {
      Alert.alert('错误', '密码至少需要 8 位');
      return;
    }

    if (password !== confirmPassword) {
      Alert.alert('错误', '两次密码输入不一致');
      return;
    }

    try {
      await importWallet(mnemonic.trim(), password);
      router.replace('/(tabs)');
    } catch (err) {
      Alert.alert('导入失败', error || '未知错误');
    }
  };

  return (
    <View style={styles.wrapper}>
      <ScrollView style={styles.container} contentContainerStyle={styles.content} showsVerticalScrollIndicator={false}>
      {/* 顶部导航 */}
      <View style={styles.navBar}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Text style={styles.navTitle}>导入钱包</Text>
        <View style={styles.placeholder} />
      </View>

      <Text style={styles.subtitle}>输入您的助记词恢复钱包</Text>

      <View style={styles.form}>
        <View style={styles.formCard}>
          <Text style={styles.label}>助记词</Text>
          <TextInput
            style={[styles.input, styles.mnemonicInput]}
            placeholder="请输入 12 或 24 个单词，用空格分隔"
            placeholderTextColor="#999"
            multiline
            value={mnemonic}
            onChangeText={setMnemonic}
            autoCapitalize="none"
            autoCorrect={false}
          />

          <Text style={styles.label}>密码</Text>
          <TextInput
            style={styles.input}
            placeholder="至少 8 位"
            placeholderTextColor="#999"
            secureTextEntry
            value={password}
            onChangeText={setPassword}
            autoCapitalize="none"
          />

          <Text style={styles.label}>确认密码</Text>
          <TextInput
            style={styles.input}
            placeholder="再次输入密码"
            placeholderTextColor="#999"
            secureTextEntry
            value={confirmPassword}
            onChangeText={setConfirmPassword}
            autoCapitalize="none"
          />

          {error && <Text style={styles.error}>{error}</Text>}
        </View>

        <Pressable
          style={[styles.primaryButton, isLoading && styles.disabledButton]}
          onPress={handleImport}
          disabled={isLoading}
        >
          {isLoading ? (
            <ActivityIndicator color="#FFF" />
          ) : (
            <Text style={styles.primaryButtonText}>导入钱包</Text>
          )}
        </Pressable>

        <View style={styles.tips}>
          <View style={styles.tipItem}>
            <Ionicons name="shield-checkmark-outline" size={18} color="#27AE60" />
            <Text style={styles.tipText}>助记词将加密存储在本地</Text>
          </View>
          <View style={styles.tipItem}>
            <Ionicons name="lock-closed-outline" size={18} color="#27AE60" />
            <Text style={styles.tipText}>使用密码保护您的钱包</Text>
          </View>
        </View>
      </View>
    </ScrollView>

      {/* 底部导航 - 全局统一 */}
      <View style={styles.bottomNav}>
        <Pressable style={styles.bottomNavItem} onPress={() => router.push('/' as any)}>
          <Text style={styles.bottomNavIcon}>🏠</Text>
          <Text style={styles.bottomNavLabel}>首页</Text>
        </Pressable>
        <Pressable style={styles.bottomNavItem} onPress={() => router.push('/divination' as any)}>
          <Text style={styles.bottomNavIcon}>🧭</Text>
          <Text style={styles.bottomNavLabel}>占卜</Text>
        </Pressable>
        <Pressable style={styles.bottomNavItem} onPress={() => router.push('/chat' as any)}>
          <Text style={styles.bottomNavIcon}>💬</Text>
          <Text style={styles.bottomNavLabel}>消息</Text>
        </Pressable>
        <Pressable style={[styles.bottomNavItem, styles.bottomNavItemActive]} onPress={() => router.push('/profile' as any)}>
          <Text style={styles.bottomNavIcon}>👤</Text>
          <Text style={[styles.bottomNavLabel, styles.bottomNavLabelActive]}>我的</Text>
        </Pressable>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  wrapper: {
    flex: 1,
    backgroundColor: THEME_BG,
    maxWidth: 414,
    width: '100%',
    alignSelf: 'center',
  },
  container: {
    flex: 1,
    backgroundColor: THEME_BG,
  },
  content: {
    paddingBottom: 40,
  },
  navBar: {
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
  backButton: {
    padding: 4,
  },
  navTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#333',
  },
  placeholder: {
    width: 32,
  },
  subtitle: {
    fontSize: 14,
    color: '#999',
    paddingHorizontal: 16,
    paddingVertical: 16,
  },
  form: {
    paddingHorizontal: 16,
    gap: 16,
  },
  formCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    gap: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  label: {
    fontSize: 14,
    color: '#8B6914',
    marginBottom: -8,
  },
  input: {
    backgroundColor: '#FAFAFA',
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    padding: 14,
    fontSize: 15,
    color: '#333',
  },
  mnemonicInput: {
    minHeight: 100,
    textAlignVertical: 'top',
  },
  error: {
    fontSize: 14,
    color: '#E74C3C',
  },
  primaryButton: {
    backgroundColor: THEME_COLOR,
    paddingVertical: 16,
    borderRadius: 8,
    alignItems: 'center',
  },
  primaryButtonText: {
    fontSize: 17,
    fontWeight: '600',
    color: '#FFF',
  },
  disabledButton: {
    opacity: 0.6,
  },
  tips: {
    gap: 12,
    marginTop: 8,
  },
  tipItem: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 10,
  },
  tipText: {
    fontSize: 13,
    color: '#666',
  },
  bottomNav: {
    position: 'absolute',
    bottom: 0,
    left: '50%',
    transform: [{ translateX: -207 }],
    width: 414,
    flexDirection: 'row',
    justifyContent: 'space-around',
    alignItems: 'center',
    backgroundColor: '#FFF',
    paddingTop: 8,
    paddingBottom: 8,
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
  },
  bottomNavItem: {
    alignItems: 'center',
    paddingVertical: 4,
    flex: 1,
  },
  bottomNavItemActive: {},
  bottomNavIcon: {
    fontSize: 22,
    marginBottom: 2,
  },
  bottomNavLabel: {
    fontSize: 12,
    color: '#999',
    fontWeight: '500',
  },
  bottomNavLabelActive: {
    color: THEME_COLOR,
  },
});
