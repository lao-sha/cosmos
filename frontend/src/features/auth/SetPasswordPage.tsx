/**
 * 星尘玄鉴 - 设置密码页面
 * 步骤1: 用户设置钱包加密密码
 * 主题色：金棕色 #B2955D
 */

import { useState } from 'react';
import {
  View,
  Text,
  TextInput,
  StyleSheet,
  Pressable,
  ActivityIndicator,
  KeyboardAvoidingView,
  Platform,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

interface SetPasswordPageProps {
  onPasswordSet: (password: string) => void;
  onBack: () => void;
  isLoading?: boolean;
}

export default function SetPasswordPage({
  onPasswordSet,
  onBack,
  isLoading = false,
}: SetPasswordPageProps) {
  const router = useRouter();
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [error, setError] = useState('');

  const validatePassword = (): boolean => {
    if (!password) {
      setError('请输入密码');
      return false;
    }
    if (password.length < 8) {
      setError('密码至少需要 8 位');
      return false;
    }
    if (password !== confirmPassword) {
      setError('两次密码输入不一致');
      return false;
    }
    setError('');
    return true;
  };

  const handleContinue = () => {
    if (validatePassword()) {
      onPasswordSet(password);
    }
  };

  const passwordStrength = (): { level: number; text: string; color: string } => {
    if (!password) return { level: 0, text: '', color: '#666' };
    if (password.length < 8) return { level: 1, text: '弱', color: '#E74C3C' };
    if (password.length < 12) return { level: 2, text: '中', color: '#F39C12' };
    if (/[A-Z]/.test(password) && /[0-9]/.test(password) && /[^A-Za-z0-9]/.test(password)) {
      return { level: 4, text: '强', color: '#27AE60' };
    }
    return { level: 3, text: '较强', color: '#88BB44' };
  };

  const strength = passwordStrength();

  return (
    <KeyboardAvoidingView
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
    >
      <View style={styles.content}>
        {/* 返回按钮 */}
        <Pressable style={styles.backButton} onPress={onBack}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>

        {/* 标题 */}
        <Text style={styles.title}>设置密码</Text>
        <Text style={styles.subtitle}>
          密码用于加密您的钱包，请牢记密码
        </Text>

        {/* 表单卡片 */}
        <View style={styles.formCard}>
          {/* 密码输入 */}
          <View style={styles.inputContainer}>
            <Text style={styles.label}>密码</Text>
            <View style={styles.inputWrapper}>
              <TextInput
                style={styles.input}
                placeholder="至少 8 位字符"
                placeholderTextColor="#999"
                secureTextEntry={!showPassword}
                value={password}
                onChangeText={(text) => {
                  setPassword(text);
                  setError('');
                }}
                autoCapitalize="none"
                autoCorrect={false}
              />
              <Pressable
                style={styles.eyeButton}
                onPress={() => setShowPassword(!showPassword)}
              >
                <Ionicons
                  name={showPassword ? 'eye-off' : 'eye'}
                  size={20}
                  color="#999"
                />
              </Pressable>
            </View>

            {/* 密码强度指示器 */}
            {password.length > 0 && (
              <View style={styles.strengthContainer}>
                <View style={styles.strengthBars}>
                  {[1, 2, 3, 4].map((level) => (
                    <View
                      key={level}
                      style={[
                        styles.strengthBar,
                        { backgroundColor: level <= strength.level ? strength.color : '#E8E8E8' },
                      ]}
                    />
                  ))}
                </View>
                <Text style={[styles.strengthText, { color: strength.color }]}>
                  {strength.text}
                </Text>
              </View>
            )}
          </View>

          {/* 确认密码 */}
          <View style={styles.inputContainer}>
            <Text style={styles.label}>确认密码</Text>
            <View style={styles.inputWrapper}>
              <TextInput
                style={styles.input}
                placeholder="再次输入密码"
                placeholderTextColor="#999"
                secureTextEntry={!showConfirmPassword}
                value={confirmPassword}
                onChangeText={(text) => {
                  setConfirmPassword(text);
                  setError('');
                }}
                autoCapitalize="none"
                autoCorrect={false}
              />
              <Pressable
                style={styles.eyeButton}
                onPress={() => setShowConfirmPassword(!showConfirmPassword)}
              >
                <Ionicons
                  name={showConfirmPassword ? 'eye-off' : 'eye'}
                  size={20}
                  color="#999"
                />
              </Pressable>
            </View>
          </View>

          {/* 错误提示 */}
          {error ? <Text style={styles.error}>{error}</Text> : null}
        </View>

        {/* 提示信息 */}
        <View style={styles.tips}>
          <View style={styles.tipItem}>
            <Ionicons name="shield-checkmark-outline" size={18} color="#27AE60" />
            <Text style={styles.tipText}>密码仅存储在本地设备</Text>
          </View>
          <View style={styles.tipItem}>
            <Ionicons name="warning-outline" size={18} color="#F39C12" />
            <Text style={styles.tipText}>忘记密码需通过助记词恢复</Text>
          </View>
        </View>

        {/* 继续按钮 */}
        <Pressable
          style={[styles.primaryButton, isLoading && styles.disabledButton]}
          onPress={handleContinue}
          disabled={isLoading}
        >
          {isLoading ? (
            <ActivityIndicator color="#FFF" />
          ) : (
            <Text style={styles.primaryButtonText}>继续</Text>
          )}
        </Pressable>
      </View>

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
    </KeyboardAvoidingView>
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
  content: {
    flex: 1,
    padding: 16,
    paddingTop: 60,
  },
  backButton: {
    width: 40,
    height: 40,
    justifyContent: 'center',
    marginBottom: 16,
  },
  title: {
    fontSize: 24,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 14,
    color: '#999',
    marginBottom: 24,
  },
  formCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
    marginBottom: 24,
  },
  inputContainer: {
    marginBottom: 16,
  },
  label: {
    fontSize: 14,
    color: '#8B6914',
    marginBottom: 8,
  },
  inputWrapper: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FAFAFA',
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#E8E8E8',
  },
  input: {
    flex: 1,
    padding: 14,
    fontSize: 15,
    color: '#333',
  },
  eyeButton: {
    padding: 14,
  },
  strengthContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 8,
  },
  strengthBars: {
    flexDirection: 'row',
    flex: 1,
    gap: 4,
  },
  strengthBar: {
    flex: 1,
    height: 4,
    borderRadius: 2,
  },
  strengthText: {
    fontSize: 12,
    marginLeft: 8,
    width: 40,
  },
  error: {
    fontSize: 14,
    color: '#E74C3C',
  },
  tips: {
    marginBottom: 24,
    gap: 12,
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
