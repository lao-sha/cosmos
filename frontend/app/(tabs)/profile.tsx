import React from 'react';
import { View, Text, StyleSheet, ScrollView, TouchableOpacity } from 'react-native';
import { useRouter } from 'expo-router';
import {
  Shield,
  Users,
  CreditCard,
  Settings,
  ChevronRight,
  LogOut,
} from 'lucide-react-native';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import { Card } from '@/components/ui';
import { Colors } from '@/constants/colors';

export default function ProfileScreen() {
  const colors = useColors();
  const router = useRouter();
  const { address, name, isConnected, disconnect } = useWalletStore();

  const menuItems = [
    {
      id: 'kyc',
      title: 'KYC 认证',
      subtitle: '未认证',
      icon: Shield,
      color: Colors.kyc.none,
      route: '/settings/kyc',
    },
    {
      id: 'referral',
      title: '推荐邀请',
      subtitle: '邀请好友，获得奖励',
      icon: Users,
      color: Colors.primary,
      route: '/referral',
    },
    {
      id: 'membership',
      title: '会员等级',
      subtitle: '普通会员',
      icon: CreditCard,
      color: Colors.membership.normal,
      route: '/membership',
    },
    {
      id: 'settings',
      title: '设置',
      subtitle: '网络、安全、通知',
      icon: Settings,
      color: colors.textSecondary,
      route: '/settings',
    },
  ];

  return (
    <ScrollView
      style={[styles.container, { backgroundColor: colors.background }]}
      contentContainerStyle={styles.content}
    >
      {/* Profile Header */}
      <Card style={styles.profileCard}>
        <View style={styles.profileHeader}>
          <View style={[styles.avatar, { backgroundColor: Colors.primary }]}>
            <Text style={styles.avatarText}>
              {name?.charAt(0) || address?.charAt(0) || 'C'}
            </Text>
          </View>
          <View style={styles.profileInfo}>
            <Text style={[styles.profileName, { color: colors.textPrimary }]}>
              {name || '未命名钱包'}
            </Text>
            {address && (
              <Text style={[styles.profileAddress, { color: colors.textSecondary }]}>
                {address.slice(0, 8)}...{address.slice(-8)}
              </Text>
            )}
          </View>
        </View>
      </Card>

      {/* Menu Items */}
      <Card style={styles.menuCard}>
        {menuItems.map((item, index) => (
          <TouchableOpacity
            key={item.id}
            style={[
              styles.menuItem,
              index < menuItems.length - 1 && {
                borderBottomWidth: 1,
                borderBottomColor: colors.border,
              },
            ]}
            onPress={() => router.push(item.route as any)}
            activeOpacity={0.7}
          >
            <View style={[styles.menuIcon, { backgroundColor: item.color + '20' }]}>
              <item.icon size={20} color={item.color} />
            </View>
            <View style={styles.menuContent}>
              <Text style={[styles.menuTitle, { color: colors.textPrimary }]}>
                {item.title}
              </Text>
              <Text style={[styles.menuSubtitle, { color: colors.textSecondary }]}>
                {item.subtitle}
              </Text>
            </View>
            <ChevronRight size={20} color={colors.textTertiary} />
          </TouchableOpacity>
        ))}
      </Card>

      {/* Logout */}
      {isConnected && (
        <TouchableOpacity
          style={[styles.logoutButton, { borderColor: Colors.error }]}
          onPress={disconnect}
          activeOpacity={0.7}
        >
          <LogOut size={20} color={Colors.error} />
          <Text style={[styles.logoutText, { color: Colors.error }]}>
            退出登录
          </Text>
        </TouchableOpacity>
      )}

      {/* Version */}
      <Text style={[styles.version, { color: colors.textTertiary }]}>
        COSMOS v1.0.0
      </Text>
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
  profileCard: {
    marginBottom: 16,
  },
  profileHeader: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  avatar: {
    width: 64,
    height: 64,
    borderRadius: 32,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 16,
  },
  avatarText: {
    color: '#FFFFFF',
    fontSize: 28,
    fontWeight: '700',
  },
  profileInfo: {
    flex: 1,
  },
  profileName: {
    fontSize: 20,
    fontWeight: '700',
    marginBottom: 4,
  },
  profileAddress: {
    fontSize: 14,
  },
  menuCard: {
    marginBottom: 16,
    padding: 0,
  },
  menuItem: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 16,
  },
  menuIcon: {
    width: 40,
    height: 40,
    borderRadius: 12,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  menuContent: {
    flex: 1,
  },
  menuTitle: {
    fontSize: 16,
    fontWeight: '500',
    marginBottom: 2,
  },
  menuSubtitle: {
    fontSize: 13,
  },
  logoutButton: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 16,
    borderRadius: 12,
    borderWidth: 1,
    marginBottom: 24,
  },
  logoutText: {
    fontSize: 16,
    fontWeight: '500',
    marginLeft: 8,
  },
  version: {
    textAlign: 'center',
    fontSize: 12,
  },
});
