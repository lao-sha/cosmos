import { WalletService } from '@/src/lib/wallet';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
  ActivityIndicator,
  Alert,
  Platform,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  View
} from 'react-native';

export default function ProfileScreen() {
  const router = useRouter();
  const { isLoggedIn, address, login, logout } = useAuthStore();
  const { isConnected } = useChainStore();
  const [isCreating, setIsCreating] = useState(false);

  const handleCreateWallet = async () => {
    try {
      setIsCreating(true);
      console.log('Creating wallet...');
      
      // 初始化加密库
      await WalletService.init();
      
      // 生成助记词
      const newMnemonic = WalletService.generateMnemonic();
      console.log('Mnemonic generated');

      // 显示助记词
      const message = '请务必记录以下助记词，它是找回钱包的唯一凭证：\n\n' + newMnemonic;
      
      if (Platform.OS === 'web') {
        const confirmed = window.confirm(message + '\n\n点击"确定"表示已备份');
        if (confirmed) {
          await WalletService.saveMnemonic(newMnemonic);
          const account = await WalletService.getAccountFromMnemonic(newMnemonic);
          login(account.address, newMnemonic);
          window.alert('钱包创建成功！\n\n地址: ' + account.address);
        }
      } else {
        Alert.alert('备份助记词', message, [
          {
            text: '已备份',
            onPress: async () => {
              await WalletService.saveMnemonic(newMnemonic);
              const account = await WalletService.getAccountFromMnemonic(newMnemonic);
              login(account.address, newMnemonic);
            }
          }
        ]);
      }
    } catch (error) {
      console.error('Error creating wallet:', error);
      const errorMsg = '创建钱包失败: ' + (error as Error).message;
      if (Platform.OS === 'web') {
        window.alert(errorMsg);
      } else {
        Alert.alert('错误', errorMsg);
      }
    } finally {
      setIsCreating(false);
    }
  };

  const handleLogout = () => {
    const doLogout = async () => {
      await logout();
    };

    if (Platform.OS === 'web') {
      if (window.confirm('确定要注销钱包吗？')) {
        doLogout();
      }
    } else {
      Alert.alert('退出登录', '确定要注销钱包吗？', [
        { text: '取消', style: 'cancel' },
        { text: '确定', onPress: doLogout, style: 'destructive' }
      ]);
    }
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      {/* 头像区域 */}
      <View style={styles.avatarContainer}>
        <View style={styles.avatar}>
          <Text style={styles.avatarText}>{isLoggedIn ? 'S' : '?'}</Text>
        </View>
        <Text style={styles.title}>我的账户</Text>
        <View style={styles.statusRow}>
          <View style={[styles.statusDot, isConnected ? styles.statusOnline : styles.statusOffline]} />
          <Text style={styles.statusText}>
            {isConnected ? '链上已连接' : '链上未连接'}
          </Text>
        </View>
      </View>

      {isLoggedIn ? (
        /* 已登录状态 */
        <View style={styles.card}>
          <Text style={styles.cardTitle}>钱包地址</Text>
          <Text style={styles.addressText} selectable>{address}</Text>
          
          <Pressable 
            style={({ pressed }) => [styles.logoutButton, pressed && styles.buttonPressed]}
            onPress={handleLogout}
          >
            <Text style={styles.logoutButtonText}>注销钱包</Text>
          </Pressable>
        </View>
      ) : (
        /* 未登录状态 */
        <View style={styles.buttonGroup}>
          <Pressable 
            style={({ pressed }) => [styles.primaryButton, isCreating && styles.buttonDisabled, pressed && styles.buttonPressed]}
            onPress={handleCreateWallet}
            disabled={isCreating}
          >
            {isCreating ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <Text style={styles.primaryButtonText}>创建新钱包</Text>
            )}
          </Pressable>
          
          <Pressable 
            style={({ pressed }) => [styles.secondaryButton, pressed && styles.buttonPressed]}
            onPress={() => router.push('/wallet/import')}
          >
            <Text style={styles.secondaryButtonText}>导入助记词</Text>
          </Pressable>
        </View>
      )}

      {/* 快捷入口 */}
      {isLoggedIn && (
        <View style={styles.quickActions}>
          <Pressable 
            style={styles.quickAction}
            onPress={() => router.push('/wallet')}
          >
            <Text style={styles.quickActionIcon}>👛</Text>
            <Text style={styles.quickActionText}>钱包</Text>
          </Pressable>
          <Pressable 
            style={styles.quickAction}
            onPress={() => router.push('/membership')}
          >
            <Text style={styles.quickActionIcon}>⭐</Text>
            <Text style={styles.quickActionText}>会员</Text>
          </Pressable>
          <Pressable 
            style={styles.quickAction}
            onPress={() => router.push('/notifications')}
          >
            <Text style={styles.quickActionIcon}>🔔</Text>
            <Text style={styles.quickActionText}>通知</Text>
          </Pressable>
          <Pressable 
            style={styles.quickAction}
            onPress={() => router.push('/friends')}
          >
            <Text style={styles.quickActionIcon}>👥</Text>
            <Text style={styles.quickActionText}>好友</Text>
          </Pressable>
        </View>
      )}

      {/* 设置列表 */}
      <View style={styles.settingsList}>
        <Text style={styles.sectionTitle}>设置与服务</Text>
        <Pressable 
          style={({ pressed }) => [styles.settingItem, pressed && styles.settingItemPressed]}
          onPress={() => router.push('/settings')}
        >
          <Text style={styles.settingItemText}>⚙️ 系统设置</Text>
          <Text style={styles.settingItemArrow}>›</Text>
        </Pressable>
        <Pressable 
          style={({ pressed }) => [styles.settingItem, pressed && styles.settingItemPressed]}
          onPress={() => router.push('/settings/security')}
        >
          <Text style={styles.settingItemText}>🔒 安全中心</Text>
          <Text style={styles.settingItemArrow}>›</Text>
        </Pressable>
        <Pressable 
          style={({ pressed }) => [styles.settingItem, pressed && styles.settingItemPressed]}
          onPress={() => router.push('/settings/privacy')}
        >
          <Text style={styles.settingItemText}>🛡️ 隐私设置</Text>
          <Text style={styles.settingItemArrow}>›</Text>
        </Pressable>
        <Pressable 
          style={({ pressed }) => [styles.settingItem, pressed && styles.settingItemPressed]}
          onPress={() => router.push('/help')}
        >
          <Text style={styles.settingItemText}>❓ 帮助与反馈</Text>
          <Text style={styles.settingItemArrow}>›</Text>
        </Pressable>
        <Pressable 
          style={({ pressed }) => [styles.settingItem, pressed && styles.settingItemPressed]}
          onPress={() => router.push('/legal/terms')}
        >
          <Text style={styles.settingItemText}>📄 关于 Cosmos</Text>
          <Text style={styles.settingItemArrow}>›</Text>
        </Pressable>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flexGrow: 1,
    padding: 20,
    backgroundColor: '#f5f5f5',
  },
  avatarContainer: {
    alignItems: 'center',
    paddingVertical: 30,
  },
  avatar: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: '#6D28D9',
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 12,
  },
  avatarText: {
    color: '#fff',
    fontSize: 32,
    fontWeight: 'bold',
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#1f2937',
  },
  statusRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 8,
  },
  statusDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    marginRight: 6,
  },
  statusOnline: {
    backgroundColor: '#22c55e',
  },
  statusOffline: {
    backgroundColor: '#ef4444',
  },
  statusText: {
    fontSize: 12,
    color: '#9ca3af',
  },
  card: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 20,
    marginBottom: 20,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 8,
    elevation: 3,
  },
  cardTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#6b7280',
    marginBottom: 8,
  },
  addressText: {
    fontSize: 12,
    color: '#374151',
    marginBottom: 20,
    fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
  },
  buttonGroup: {
    width: '100%',
    gap: 12,
  },
  primaryButton: {
    backgroundColor: '#6D28D9',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  primaryButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: 'bold',
  },
  secondaryButton: {
    backgroundColor: '#e5e7eb',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  secondaryButtonText: {
    color: '#374151',
    fontSize: 16,
    fontWeight: 'bold',
  },
  buttonDisabled: {
    opacity: 0.6,
  },
  buttonPressed: {
    opacity: 0.7,
  },
  logoutButton: {
    backgroundColor: 'rgba(239, 68, 68, 0.1)',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  logoutButtonText: {
    color: '#ef4444',
    fontSize: 16,
    fontWeight: '600',
  },
  quickActions: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 16,
    marginBottom: 20,
  },
  quickAction: {
    alignItems: 'center',
  },
  quickActionIcon: {
    fontSize: 28,
    marginBottom: 6,
  },
  quickActionText: {
    fontSize: 12,
    color: '#6b7280',
  },
  settingsList: {
    marginTop: 10,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#374151',
    marginBottom: 12,
  },
  settingItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  settingItemText: {
    fontSize: 15,
    color: '#374151',
  },
  settingItemArrow: {
    fontSize: 20,
    color: '#9ca3af',
  },
  settingItemPressed: {
    backgroundColor: '#f3f4f6',
  },
});
