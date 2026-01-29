import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    Alert,
    Platform,
    Pressable,
    ScrollView,
    StyleSheet,
    Switch,
    Text,
    View,
} from 'react-native';

interface SettingItem {
  id: string;
  title: string;
  subtitle?: string;
  type: 'link' | 'toggle' | 'action';
  value?: boolean;
  danger?: boolean;
  onPress?: () => void;
  onToggle?: (value: boolean) => void;
}

export default function SettingsScreen() {
  const router = useRouter();
  const { logout, isLoggedIn, address } = useAuthStore();
  const { isConnected } = useChainStore();
  
  const [notifications, setNotifications] = useState(true);
  const [biometric, setBiometric] = useState(false);
  const [darkMode, setDarkMode] = useState(false);

  const handleLogout = () => {
    const doLogout = () => {
      logout();
      router.replace('/(tabs)');
    };

    if (Platform.OS === 'web') {
      if (window.confirm('确定要退出登录吗？')) {
        doLogout();
      }
    } else {
      Alert.alert('确认', '确定要退出登录吗？', [
        { text: '取消', style: 'cancel' },
        { text: '退出', style: 'destructive', onPress: doLogout },
      ]);
    }
  };

  const settingSections = [
    {
      title: '账户',
      items: [
        {
          id: 'wallet',
          title: '钱包管理',
          subtitle: address ? `${address.slice(0, 8)}...${address.slice(-6)}` : '未登录',
          type: 'link' as const,
          onPress: () => router.push('/wallet'),
        },
        {
          id: 'security',
          title: '安全设置',
          type: 'link' as const,
          onPress: () => router.push('/settings/security'),
        },
      ],
    },
    {
      title: '通知',
      items: [
        {
          id: 'notifications',
          title: '推送通知',
          subtitle: '接收消息和订单提醒',
          type: 'toggle' as const,
          value: notifications,
          onToggle: setNotifications,
        },
      ],
    },
    {
      title: '隐私',
      items: [
        {
          id: 'privacy',
          title: '隐私设置',
          type: 'link' as const,
          onPress: () => router.push('/settings/privacy'),
        },
        {
          id: 'blocklist',
          title: '黑名单管理',
          type: 'link' as const,
          onPress: () => router.push('/friends'),
        },
      ],
    },
    {
      title: '显示',
      items: [
        {
          id: 'darkmode',
          title: '深色模式',
          type: 'toggle' as const,
          value: darkMode,
          onToggle: setDarkMode,
        },
        {
          id: 'biometric',
          title: '生物识别',
          subtitle: '使用指纹或面容解锁',
          type: 'toggle' as const,
          value: biometric,
          onToggle: setBiometric,
        },
      ],
    },
    {
      title: '关于',
      items: [
        {
          id: 'version',
          title: '版本',
          subtitle: '1.0.0',
          type: 'link' as const,
        },
        {
          id: 'terms',
          title: '服务条款',
          type: 'link' as const,
          onPress: () => router.push('/legal/terms'),
        },
        {
          id: 'privacy-policy',
          title: '隐私政策',
          type: 'link' as const,
          onPress: () => router.push('/legal/privacy'),
        },
      ],
    },
  ];

  const renderItem = (item: SettingItem) => (
    <Pressable
      key={item.id}
      style={({ pressed }) => [
        styles.settingItem,
        pressed && item.type !== 'toggle' && styles.itemPressed,
      ]}
      onPress={item.type !== 'toggle' ? item.onPress : undefined}
      disabled={item.type === 'toggle'}
    >
      <View style={styles.itemContent}>
        <Text style={[styles.itemTitle, item.danger && styles.dangerText]}>
          {item.title}
        </Text>
        {item.subtitle && (
          <Text style={styles.itemSubtitle}>{item.subtitle}</Text>
        )}
      </View>
      
      {item.type === 'toggle' && (
        <Switch
          value={item.value}
          onValueChange={item.onToggle}
          trackColor={{ false: '#e5e7eb', true: '#c4b5fd' }}
          thumbColor={item.value ? '#6D28D9' : '#f4f3f4'}
        />
      )}
      
      {item.type === 'link' && (
        <Text style={styles.chevron}>›</Text>
      )}
    </Pressable>
  );

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>设置</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.statusBar}>
          <View style={styles.statusItem}>
            <View style={[styles.statusDot, isConnected ? styles.connected : styles.disconnected]} />
            <Text style={styles.statusText}>
              {isConnected ? '链已连接' : '链未连接'}
            </Text>
          </View>
        </View>

        {settingSections.map((section) => (
          <View key={section.title} style={styles.section}>
            <Text style={styles.sectionTitle}>{section.title}</Text>
            <View style={styles.sectionContent}>
              {section.items.map(renderItem)}
            </View>
          </View>
        ))}

        {isLoggedIn && (
          <Pressable style={styles.logoutButton} onPress={handleLogout}>
            <Text style={styles.logoutText}>退出登录</Text>
          </Pressable>
        )}

        <View style={styles.footer}>
          <Text style={styles.footerText}>Cosmos © 2024</Text>
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
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#fff',
    paddingTop: 50,
    paddingBottom: 12,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  backButton: {
    padding: 4,
  },
  backText: {
    fontSize: 17,
    color: '#6D28D9',
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#1f2937',
  },
  headerRight: {
    width: 50,
  },
  content: {
    flex: 1,
  },
  statusBar: {
    backgroundColor: '#fff',
    padding: 16,
    marginBottom: 12,
  },
  statusItem: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  statusDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
  },
  connected: {
    backgroundColor: '#22c55e',
  },
  disconnected: {
    backgroundColor: '#ef4444',
  },
  statusText: {
    fontSize: 14,
    color: '#6b7280',
  },
  section: {
    marginBottom: 24,
  },
  sectionTitle: {
    fontSize: 13,
    fontWeight: '600',
    color: '#9ca3af',
    paddingHorizontal: 16,
    marginBottom: 8,
    textTransform: 'uppercase',
  },
  sectionContent: {
    backgroundColor: '#fff',
  },
  settingItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 14,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  itemPressed: {
    backgroundColor: '#f9fafb',
  },
  itemContent: {
    flex: 1,
  },
  itemTitle: {
    fontSize: 16,
    color: '#1f2937',
  },
  itemSubtitle: {
    fontSize: 13,
    color: '#9ca3af',
    marginTop: 2,
  },
  dangerText: {
    color: '#dc2626',
  },
  chevron: {
    fontSize: 20,
    color: '#d1d5db',
  },
  logoutButton: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 12,
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  logoutText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#dc2626',
  },
  footer: {
    padding: 32,
    alignItems: 'center',
  },
  footerText: {
    fontSize: 13,
    color: '#9ca3af',
  },
});
