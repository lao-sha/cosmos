/**
 * 星尘玄鉴 - 底部导航栏组件
 * 用于非 tabs 页面的底部导航
 * 与 tabs 布局保持一致的样式
 * 主题色：金棕色 #B2955D
 */

import React from 'react';
import { View, Text, StyleSheet, Pressable, Platform } from 'react-native';
import { useRouter, usePathname } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { HomeOutline, CompassOutline, MessageOutline, UserOutline } from './TabIcons';

const THEME_COLOR = '#B2955D';

interface NavItem {
  name: string;
  title: string;
  path: string;
  icon: (props: { color: string; size: number }) => React.ReactNode;
}

const navItems: NavItem[] = [
  {
    name: 'index',
    title: '首页',
    path: '/',
    icon: ({ color, size }) => <HomeOutline color={color} size={size} />,
  },
  {
    name: 'divination',
    title: '占卜',
    path: '/divination',
    icon: ({ color, size }) => <CompassOutline color={color} size={size} />,
  },
  {
    name: 'market',
    title: '市场',
    path: '/market',
    icon: ({ color, size }) => <Ionicons name="storefront-outline" size={size} color={color} />,
  },
  {
    name: 'chat',
    title: '消息',
    path: '/chat',
    icon: ({ color, size }) => <MessageOutline color={color} size={size} />,
  },
  {
    name: 'profile',
    title: '我的',
    path: '/profile',
    icon: ({ color, size }) => <UserOutline color={color} size={size} />,
  },
];

interface BottomNavBarProps {
  activeTab?: string;
}

export function BottomNavBar({ activeTab }: BottomNavBarProps) {
  const router = useRouter();
  const pathname = usePathname();

  // 根据当前路径判断激活状态
  const getActiveTab = () => {
    if (activeTab) return activeTab;
    if (pathname === '/' || pathname === '/index') return 'index';
    if (pathname.startsWith('/divination')) return 'divination';
    if (pathname.startsWith('/market')) return 'market';
    if (pathname.startsWith('/chat')) return 'chat';
    if (pathname.startsWith('/profile') || pathname.startsWith('/wallet')) return 'profile';
    return '';
  };

  const currentTab = getActiveTab();

  const handlePress = (item: NavItem) => {
    router.push(item.path as any);
  };

  return (
    <View style={styles.container}>
      {navItems.map((item) => {
        const isActive = currentTab === item.name;
        const color = isActive ? THEME_COLOR : '#999';

        return (
          <Pressable
            key={item.name}
            style={styles.navItem}
            onPress={() => handlePress(item)}
          >
            {item.icon({ color, size: 24 })}
            <Text style={[styles.navLabel, { color }]}>
              {item.title}
            </Text>
          </Pressable>
        );
      })}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    backgroundColor: '#FFF',
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
    height: Platform.select({ ios: 88, android: 60, default: 60 }),
    paddingBottom: Platform.select({ ios: 24, android: 8, default: 8 }),
    paddingTop: 8,
  },
  navItem: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  navLabel: {
    fontSize: 11,
    fontWeight: '500',
    marginTop: 2,
  },
});
