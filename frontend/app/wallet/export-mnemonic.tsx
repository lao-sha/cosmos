/**
 * 星尘玄鉴 - 导出助记词页面
 * 需要密码验证后才能查看助记词
 * 主题色：金棕色 #B2955D
 */

import { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Pressable,
  TextInput,
  ScrollView,
  Alert,
  Platform,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import * as Clipboard from 'expo-clipboard';
import { retrieveEncryptedMnemonic } from '@/lib/keystore';
import { BottomNavBar } from '@/components/BottomNavBar';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

export default function ExportMnemonicPage() {
  const router = useRouter();
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [mnemonic, setMnemonic] = useState('');
  const [isVerified, setIsVerified] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [copied, setCopied] = useState(false);

  const handleVerify = async () => {
    if (!password) {
      Alert.alert('提示', '请输入密码');
      return;
    }

    setIsLoading(true);
    try {
      const storedMnemonic = await retrieveEncryptedMnemonic(password);
      setMnemonic(storedMnemonic);
      setIsVerified(true);
    } catch (error) {
      Alert.alert('验证失败', '密码错误，请重试');
    } finally {
      setIsLoading(false);
    }
  };

  const handleCopy = async () => {
    await Clipboard.setStringAsync(mnemonic);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const words = mnemonic ? mnemonic.split(' ') : [];

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.navBar}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Text style={styles.navTitle}>导出助记词</Text>
        <View style={styles.placeholder} />
      </View>

      <ScrollView style={styles.scrollView} contentContainerStyle={styles.content} showsVerticalScrollIndicator={false}>
        {!isVerified ? (
          <>
            {/* 警告提示 */}
            <View style={styles.warningBox}>
              <Ionicons name="warning-outline" size={20} color="#F39C12" />
              <Text style={styles.warningText}>
                助记词是恢复钱包的唯一凭证，请勿向任何人透露。确认周围无人窥屏后再进行操作。
              </Text>
            </View>

            <Text style={styles.subtitle}>请输入密码以验证身份</Text>

            {/* 密码输入卡片 */}
            <View style={styles.formCard}>
              <Text style={styles.inputLabel}>钱包密码</Text>
              <View style={styles.inputContainer}>
                <TextInput
                  style={styles.input}
                  placeholder="输入钱包密码"
                  placeholderTextColor="#999"
                  secureTextEntry={!showPassword}
                  value={password}
                  onChangeText={setPassword}
                  autoCapitalize="none"
                />
                <Pressable onPress={() => setShowPassword(!showPassword)} style={styles.eyeButton}>
                  <Ionicons
                    name={showPassword ? 'eye-off-outline' : 'eye-outline'}
                    size={20}
                    color="#999"
                  />
                </Pressable>
              </View>
            </View>

            {/* 验证按钮 */}
            <Pressable
              style={[styles.primaryButton, isLoading && styles.disabledButton]}
              onPress={handleVerify}
              disabled={isLoading}
            >
              {isLoading ? (
                <ActivityIndicator color="#FFF" />
              ) : (
                <Text style={styles.primaryButtonText}>验证密码</Text>
              )}
            </Pressable>
          </>
        ) : (
          <>
            {/* 严重警告 */}
            <View style={styles.dangerBox}>
              <Ionicons name="alert-circle" size={22} color="#E74C3C" />
              <Text style={styles.dangerText}>
                请勿截图！请勿以电子方式存储！建议手写在纸上并妥善保管。
              </Text>
            </View>

            {/* 助记词卡片 */}
            <View style={styles.mnemonicCard}>
              <View style={styles.mnemonicGrid}>
                {words.map((word, index) => (
                  <View key={index} style={styles.wordItem}>
                    <Text style={styles.wordIndex}>{index + 1}</Text>
                    <Text style={styles.wordText}>{word}</Text>
                  </View>
                ))}
              </View>

              {/* 复制按钮 */}
              <Pressable style={styles.copyButton} onPress={handleCopy}>
                <Ionicons
                  name={copied ? 'checkmark-circle' : 'copy-outline'}
                  size={18}
                  color={copied ? '#27AE60' : THEME_COLOR}
                />
                <Text style={[styles.copyText, copied && styles.copiedText]}>
                  {copied ? '已复制' : '复制助记词'}
                </Text>
              </Pressable>
            </View>

            {/* 返回按钮 */}
            <Pressable
              style={styles.secondaryButton}
              onPress={() => router.back()}
            >
              <Text style={styles.secondaryButtonText}>完成</Text>
            </Pressable>
          </>
        )}
      </ScrollView>

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="profile" />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME_BG,
    maxWidth: 414,
    width: '100%',
    alignSelf: 'center',
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
  scrollView: {
    flex: 1,
  },
  content: {
    padding: 16,
    paddingBottom: 40,
  },
  subtitle: {
    fontSize: 14,
    color: '#999',
    marginBottom: 16,
  },
  warningBox: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    backgroundColor: '#FFF8E6',
    padding: 14,
    borderRadius: 8,
    marginBottom: 20,
    gap: 10,
    borderWidth: 1,
    borderColor: '#F39C12',
  },
  warningText: {
    flex: 1,
    fontSize: 13,
    color: '#F39C12',
    lineHeight: 20,
  },
  dangerBox: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    backgroundColor: '#FFEBEE',
    padding: 14,
    borderRadius: 8,
    marginBottom: 16,
    gap: 10,
    borderWidth: 1,
    borderColor: '#E74C3C',
  },
  dangerText: {
    flex: 1,
    fontSize: 13,
    color: '#E74C3C',
    lineHeight: 20,
    fontWeight: '500',
  },
  formCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  inputLabel: {
    fontSize: 14,
    color: '#8B6914',
    marginBottom: 8,
  },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FAFAFA',
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    paddingHorizontal: 14,
    height: 48,
  },
  input: {
    flex: 1,
    fontSize: 15,
    color: '#333',
  },
  eyeButton: {
    padding: 6,
  },
  primaryButton: {
    backgroundColor: THEME_COLOR,
    paddingVertical: 16,
    borderRadius: 8,
    alignItems: 'center',
    marginBottom: 16,
  },
  primaryButtonText: {
    fontSize: 17,
    fontWeight: '600',
    color: '#FFF',
  },
  disabledButton: {
    opacity: 0.6,
  },
  secondaryButton: {
    backgroundColor: '#FFF',
    paddingVertical: 16,
    borderRadius: 8,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: THEME_COLOR,
  },
  secondaryButtonText: {
    fontSize: 17,
    fontWeight: '600',
    color: THEME_COLOR,
  },
  mnemonicCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  mnemonicGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 10,
    marginBottom: 16,
  },
  wordItem: {
    width: '30%',
    backgroundColor: '#FAFAFA',
    borderRadius: 8,
    padding: 10,
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    borderWidth: 1,
    borderColor: '#E8E8E8',
  },
  wordIndex: {
    fontSize: 12,
    color: '#999',
    width: 18,
  },
  wordText: {
    fontSize: 14,
    color: '#333',
    fontWeight: '500',
  },
  copyButton: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 6,
    paddingVertical: 10,
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
  },
  copyText: {
    fontSize: 14,
    color: THEME_COLOR,
  },
  copiedText: {
    color: '#27AE60',
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
