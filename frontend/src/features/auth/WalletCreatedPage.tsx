/**
 * 星尘玄鉴 - 钱包创建成功页面
 * 最终步骤: 显示钱包创建成功
 * 主题色：金棕色 #B2955D
 */

import {
  View,
  Text,
  StyleSheet,
  Pressable,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

interface WalletCreatedPageProps {
  address: string;
  onComplete: () => void;
}

export default function WalletCreatedPage({
  address,
  onComplete,
}: WalletCreatedPageProps) {
  const router = useRouter();

  return (
    <View style={styles.container}>
      <View style={styles.content}>
        {/* 成功图标 */}
        <View style={styles.iconContainer}>
          <View style={styles.iconCircle}>
            <Ionicons name="checkmark" size={60} color="#FFF" />
          </View>
        </View>

        {/* 标题 */}
        <Text style={styles.title}>钱包创建成功</Text>
        <Text style={styles.subtitle}>
          恭喜！您的星尘钱包已成功创建并安全存储
        </Text>

        {/* 钱包地址 */}
        <View style={styles.addressBox}>
          <Text style={styles.addressLabel}>钱包地址</Text>
          <Text style={styles.addressText} numberOfLines={2}>
            {address}
          </Text>
        </View>

        {/* 提示信息 */}
        <View style={styles.tips}>
          <View style={styles.tipItem}>
            <View style={styles.tipIconCircle}>
              <Ionicons name="shield-checkmark" size={18} color="#27AE60" />
            </View>
            <View style={styles.tipContent}>
              <Text style={styles.tipTitle}>助记词已备份</Text>
              <Text style={styles.tipText}>请妥善保管您的助记词</Text>
            </View>
          </View>
          <View style={styles.tipItem}>
            <View style={styles.tipIconCircle}>
              <Ionicons name="lock-closed" size={18} color="#27AE60" />
            </View>
            <View style={styles.tipContent}>
              <Text style={styles.tipTitle}>密码保护</Text>
              <Text style={styles.tipText}>钱包已使用密码加密</Text>
            </View>
          </View>
          <View style={styles.tipItem}>
            <View style={styles.tipIconCircle}>
              <Ionicons name="wallet" size={18} color="#27AE60" />
            </View>
            <View style={styles.tipContent}>
              <Text style={styles.tipTitle}>准备就绪</Text>
              <Text style={styles.tipText}>可以开始使用星尘玄鉴</Text>
            </View>
          </View>
        </View>
      </View>

      {/* 完成按钮 */}
      <View style={styles.footer}>
        <Pressable style={styles.primaryButton} onPress={onComplete}>
          <Text style={styles.primaryButtonText}>开始使用</Text>
          <Ionicons name="arrow-forward" size={20} color="#FFF" />
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
  content: {
    flex: 1,
    padding: 16,
    paddingTop: 80,
    alignItems: 'center',
  },
  iconContainer: {
    marginBottom: 24,
  },
  iconCircle: {
    width: 100,
    height: 100,
    borderRadius: 50,
    backgroundColor: THEME_COLOR,
    justifyContent: 'center',
    alignItems: 'center',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.15,
    shadowRadius: 12,
    elevation: 6,
  },
  title: {
    fontSize: 24,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
    textAlign: 'center',
  },
  subtitle: {
    fontSize: 14,
    color: '#999',
    textAlign: 'center',
    marginBottom: 24,
    lineHeight: 22,
  },
  addressBox: {
    width: '100%',
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 24,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  addressLabel: {
    fontSize: 12,
    color: '#8B6914',
    marginBottom: 8,
  },
  addressText: {
    fontSize: 14,
    color: '#333',
    fontFamily: 'monospace',
    lineHeight: 22,
  },
  tips: {
    width: '100%',
    gap: 12,
  },
  tipItem: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 14,
    gap: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.04,
    shadowRadius: 4,
    elevation: 1,
  },
  tipIconCircle: {
    width: 36,
    height: 36,
    borderRadius: 18,
    backgroundColor: '#E8F5E9',
    justifyContent: 'center',
    alignItems: 'center',
  },
  tipContent: {
    flex: 1,
  },
  tipTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#333',
    marginBottom: 2,
  },
  tipText: {
    fontSize: 13,
    color: '#999',
  },
  footer: {
    padding: 16,
    paddingBottom: 40,
  },
  primaryButton: {
    backgroundColor: THEME_COLOR,
    paddingVertical: 16,
    borderRadius: 8,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 8,
  },
  primaryButtonText: {
    fontSize: 17,
    fontWeight: '600',
    color: '#FFF',
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
