/**
 * 星尘玄鉴 - 个人中心
 * 根据钱包状态显示不同内容
 * 主题色：金棕色 #B2955D
 */

import { View, Text, StyleSheet, Pressable, Alert, ScrollView } from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useWalletStore } from '@/stores';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

export default function ProfilePage() {
  const router = useRouter();
  const { hasWallet, isLocked, address, lockWallet, deleteWallet } = useWalletStore();

  const handleLock = () => {
    lockWallet();
    router.replace('/auth/unlock');
  };

  const handleDelete = () => {
    Alert.alert(
      '删除钱包',
      '确定要删除钱包吗？此操作无法撤销，请确保已备份助记词。',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '删除',
          style: 'destructive',
          onPress: async () => {
            await deleteWallet();
          },
        },
      ]
    );
  };

  // 没有钱包时显示创建/导入入口
  if (!hasWallet) {
    return (
      <View style={styles.container}>
        <Text style={styles.title}>我的</Text>

        <View style={styles.welcomeSection}>
          <View style={styles.iconCircle}>
            <Ionicons name="wallet-outline" size={48} color={THEME_COLOR} />
          </View>
          <Text style={styles.welcomeTitle}>欢迎使用星尘玄鉴</Text>
          <Text style={styles.welcomeSubtitle}>
            创建或导入钱包以开始使用
          </Text>
        </View>

        <View style={styles.buttonGroup}>
          <Pressable
            style={styles.primaryButton}
            onPress={() => router.push('/auth/create')}
          >
            <Ionicons name="add-circle-outline" size={24} color="#000" />
            <Text style={styles.primaryButtonText}>创建钱包</Text>
          </Pressable>

          <Pressable
            style={styles.secondaryButton}
            onPress={() => router.push('/auth/import')}
          >
            <Ionicons name="download-outline" size={24} color="#D4A935" />
            <Text style={styles.secondaryButtonText}>导入钱包</Text>
          </Pressable>
        </View>

        <View style={styles.tips}>
          <View style={styles.tipItem}>
            <Ionicons name="shield-checkmark-outline" size={20} color="#44BB44" />
            <Text style={styles.tipText}>助记词安全存储在本地</Text>
          </View>
          <View style={styles.tipItem}>
            <Ionicons name="lock-closed-outline" size={20} color="#44BB44" />
            <Text style={styles.tipText}>使用密码加密保护</Text>
          </View>
          <View style={styles.tipItem}>
            <Ionicons name="globe-outline" size={20} color="#44BB44" />
            <Text style={styles.tipText}>支持 Substrate 区块链</Text>
          </View>
        </View>
      </View>
    );
  }

  // 钱包已锁定时跳转到解锁页面
  if (isLocked) {
    return (
      <View style={styles.container}>
        <Text style={styles.title}>我的</Text>

        <View style={styles.lockedSection}>
          <View style={styles.iconCircle}>
            <Ionicons name="lock-closed" size={48} color={THEME_COLOR} />
          </View>
          <Text style={styles.welcomeTitle}>钱包已锁定</Text>
          <Text style={styles.welcomeSubtitle}>
            请输入密码解锁钱包
          </Text>

          <Pressable
            style={styles.primaryButton}
            onPress={() => router.push('/auth/unlock')}
          >
            <Ionicons name="lock-open-outline" size={24} color="#000" />
            <Text style={styles.primaryButtonText}>解锁钱包</Text>
          </Pressable>
        </View>
      </View>
    );
  }

  // 钱包已解锁，显示钱包信息
  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.scrollContent}>
      <Text style={styles.title}>我的</Text>

      {/* 钱包卡片 */}
      <View style={styles.walletCard}>
        <View style={styles.walletHeader}>
          <View style={styles.walletIcon}>
            <Ionicons name="wallet" size={24} color={THEME_COLOR} />
          </View>
          <Text style={styles.walletLabel}>星尘钱包</Text>
          <View style={styles.statusBadge}>
            <View style={styles.statusDot} />
            <Text style={styles.statusText}>已连接</Text>
          </View>
        </View>

        <View style={styles.addressContainer}>
          <Text style={styles.addressLabel}>地址</Text>
          <Text style={styles.addressText} numberOfLines={1} ellipsizeMode="middle">
            {address}
          </Text>
        </View>

        <View style={styles.balanceContainer}>
          <Text style={styles.balanceLabel}>余额</Text>
          <Text style={styles.balanceText}>0.00 STAR</Text>
        </View>
      </View>

      {/* 功能列表 */}
      <View style={styles.menuSection}>
        <Text style={styles.menuTitle}>钱包管理</Text>

        <Pressable style={styles.menuItem} onPress={() => router.push('/wallet/transfer')}>
          <Ionicons name="swap-horizontal-outline" size={22} color="#888" />
          <Text style={styles.menuItemText}>转账</Text>
          <Ionicons name="chevron-forward" size={20} color="#666" />
        </Pressable>

        <Pressable style={styles.menuItem} onPress={() => router.push('/wallet/transactions')}>
          <Ionicons name="document-text-outline" size={22} color="#888" />
          <Text style={styles.menuItemText}>交易记录</Text>
          <Ionicons name="chevron-forward" size={20} color="#666" />
        </Pressable>

        <Pressable style={styles.menuItem} onPress={() => router.push('/wallet/export-mnemonic')}>
          <Ionicons name="key-outline" size={22} color="#888" />
          <Text style={styles.menuItemText}>导出助记词</Text>
          <Ionicons name="chevron-forward" size={20} color="#666" />
        </Pressable>
      </View>

      {/* 操作按钮 */}
      <View style={styles.actions}>
        <Pressable style={styles.actionButton} onPress={handleLock}>
          <Ionicons name="lock-closed-outline" size={20} color="#FFF" />
          <Text style={styles.actionButtonText}>锁定钱包</Text>
        </Pressable>

        <Pressable style={[styles.actionButton, styles.dangerButton]} onPress={handleDelete}>
          <Ionicons name="trash-outline" size={20} color="#FF4444" />
          <Text style={[styles.actionButtonText, styles.dangerButtonText]}>删除钱包</Text>
        </Pressable>
      </View>
    </ScrollView>
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
  scrollContent: {
    padding: 16,
    paddingBottom: 40,
  },
  title: {
    fontSize: 24,
    fontWeight: '600',
    color: '#333',
    marginTop: 50,
    marginBottom: 20,
    paddingHorizontal: 16,
  },
  // 欢迎页样式
  welcomeSection: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 24,
  },
  lockedSection: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 24,
  },
  iconCircle: {
    width: 100,
    height: 100,
    borderRadius: 50,
    backgroundColor: '#FFF',
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 24,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 8,
    elevation: 3,
  },
  welcomeTitle: {
    fontSize: 22,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  welcomeSubtitle: {
    fontSize: 15,
    color: '#999',
    marginBottom: 40,
  },
  buttonGroup: {
    paddingHorizontal: 24,
    gap: 12,
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
  secondaryButton: {
    backgroundColor: '#FFF',
    paddingVertical: 16,
    borderRadius: 8,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 8,
    borderWidth: 1,
    borderColor: THEME_COLOR,
  },
  secondaryButtonText: {
    fontSize: 17,
    fontWeight: '600',
    color: THEME_COLOR,
  },
  tips: {
    paddingHorizontal: 24,
    marginTop: 40,
    gap: 16,
  },
  tipItem: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  tipText: {
    fontSize: 14,
    color: '#666',
  },
  // 钱包卡片样式
  walletCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 20,
    marginBottom: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  walletHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 20,
  },
  walletIcon: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: THEME_BG,
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  walletLabel: {
    flex: 1,
    fontSize: 17,
    fontWeight: '600',
    color: '#333',
  },
  statusBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#E8F5E9',
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
    gap: 6,
  },
  statusDot: {
    width: 6,
    height: 6,
    borderRadius: 3,
    backgroundColor: '#27AE60',
  },
  statusText: {
    fontSize: 12,
    color: '#27AE60',
  },
  addressContainer: {
    marginBottom: 16,
  },
  addressLabel: {
    fontSize: 12,
    color: '#999',
    marginBottom: 4,
  },
  addressText: {
    fontSize: 14,
    color: '#333',
    fontFamily: 'monospace',
  },
  balanceContainer: {},
  balanceLabel: {
    fontSize: 12,
    color: '#999',
    marginBottom: 4,
  },
  balanceText: {
    fontSize: 24,
    fontWeight: 'bold',
    color: THEME_COLOR,
  },
  // 菜单样式
  menuSection: {
    marginBottom: 16,
  },
  menuTitle: {
    fontSize: 14,
    color: '#999',
    marginBottom: 12,
  },
  menuItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FFF',
    padding: 16,
    borderRadius: 8,
    marginBottom: 8,
    gap: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.04,
    shadowRadius: 4,
    elevation: 1,
  },
  menuItemText: {
    flex: 1,
    fontSize: 15,
    color: '#333',
  },
  // 操作按钮样式
  actions: {
    gap: 12,
  },
  actionButton: {
    backgroundColor: '#FFF',
    paddingVertical: 14,
    borderRadius: 8,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 8,
    borderWidth: 1,
    borderColor: '#E8E8E8',
  },
  actionButtonText: {
    fontSize: 15,
    color: '#666',
    fontWeight: '500',
  },
  dangerButton: {
    borderColor: '#FFEBEE',
    backgroundColor: '#FFF5F5',
  },
  dangerButtonText: {
    color: '#E74C3C',
  },
});
