/**
 * 星尘玄鉴 - 备份助记词页面
 * 步骤2: 显示助记词让用户抄写备份
 * 主题色：金棕色 #B2955D
 */

import { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Pressable,
  ScrollView,
  Alert,
  Platform,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import * as Clipboard from 'expo-clipboard';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

interface BackupMnemonicPageProps {
  mnemonic: string;
  address: string;
  onBackupComplete: () => void;
  onBack: () => void;
}

export default function BackupMnemonicPage({
  mnemonic,
  address,
  onBackupComplete,
  onBack,
}: BackupMnemonicPageProps) {
  const router = useRouter();
  const [confirmed, setConfirmed] = useState(false);
  const [copied, setCopied] = useState(false);

  const words = mnemonic.split(' ');

  const handleCopy = async () => {
    await Clipboard.setStringAsync(mnemonic);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleContinue = () => {
    if (!confirmed) {
      Alert.alert('提示', '请确认您已安全备份助记词');
      return;
    }
    onBackupComplete();
  };

  return (
    <View style={styles.wrapper}>
      <ScrollView style={styles.container} contentContainerStyle={styles.content} showsVerticalScrollIndicator={false}>
      {/* 返回按钮 */}
      <Pressable style={styles.backButton} onPress={onBack}>
        <Ionicons name="chevron-back" size={24} color="#333" />
      </Pressable>

      {/* 标题 */}
      <Text style={styles.title}>备份助记词</Text>
      <Text style={styles.subtitle}>
        请按顺序抄写以下 12 个单词，这是恢复钱包的唯一凭证
      </Text>

      {/* 警告提示 */}
      <View style={styles.warningBox}>
        <Ionicons name="warning-outline" size={20} color="#F39C12" />
        <Text style={styles.warningText}>
          请勿截图或以电子方式存储！建议手写在纸上并妥善保管
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

      {/* 钱包地址预览 */}
      <View style={styles.addressBox}>
        <Text style={styles.addressLabel}>钱包地址</Text>
        <Text style={styles.addressText} numberOfLines={1} ellipsizeMode="middle">
          {address}
        </Text>
      </View>

      {/* 确认勾选 */}
      <Pressable
        style={styles.confirmRow}
        onPress={() => setConfirmed(!confirmed)}
      >
        <View style={[styles.checkbox, confirmed && styles.checkboxChecked]}>
          {confirmed && <Ionicons name="checkmark" size={16} color="#FFF" />}
        </View>
        <Text style={styles.confirmText}>
          我已将助记词安全备份，并了解丢失后无法恢复
        </Text>
      </Pressable>

      {/* 继续按钮 */}
      <Pressable
        style={[styles.primaryButton, !confirmed && styles.disabledButton]}
        onPress={handleContinue}
      >
        <Text style={styles.primaryButtonText}>下一步：验证助记词</Text>
      </Pressable>
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
    padding: 16,
    paddingTop: 60,
    paddingBottom: 40,
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
    marginBottom: 20,
    lineHeight: 22,
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
  addressBox: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 20,
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
    fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
  },
  confirmRow: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    gap: 12,
    marginBottom: 20,
  },
  checkbox: {
    width: 22,
    height: 22,
    borderRadius: 6,
    borderWidth: 2,
    borderColor: '#D9D9D9',
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#FFF',
  },
  checkboxChecked: {
    backgroundColor: THEME_COLOR,
    borderColor: THEME_COLOR,
  },
  confirmText: {
    flex: 1,
    fontSize: 13,
    color: '#666',
    lineHeight: 20,
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
