import { Tabs } from 'expo-router';
import React from 'react';
import { Platform } from 'react-native';

import { IconSymbol } from '@/components/ui/icon-symbol';
import { Colors } from '@/constants/theme';
import { useColorScheme } from '@/hooks/use-color-scheme';

export default function TabLayout() {
  const colorScheme = useColorScheme();

  return (
    <Tabs
      screenOptions={{
        tabBarActiveTintColor: Colors[colorScheme ?? 'light'].tint,
        tabBarInactiveTintColor: '#888',
        headerShown: false,
        tabBarShowLabel: true,
        tabBarStyle: {
          height: Platform.OS === 'web' ? 80 : 65,
          paddingTop: 10,
          paddingBottom: Platform.OS === 'web' ? 16 : 10,
          borderTopWidth: 1,
          borderTopColor: '#e0e0e0',
          backgroundColor: '#fff',
        },
        tabBarLabelStyle: {
          fontSize: 11,
          fontWeight: '600',
          paddingBottom: 4,
        },
        tabBarIconStyle: {
          marginBottom: 0,
        },
        tabBarHideOnKeyboard: true,
      }}>
      <Tabs.Screen
        name="index"
        options={{
          title: '首页',
          tabBarIcon: ({ color }) => <IconSymbol size={18} name="house.fill" color={color} />,
        }}
      />
      <Tabs.Screen
        name="chat"
        options={{
          title: '聊天',
          tabBarIcon: ({ color }) => <IconSymbol size={18} name="bubble.left.fill" color={color} />,
        }}
      />
      <Tabs.Screen
        name="mall"
        options={{
          title: '商城',
          tabBarIcon: ({ color }) => <IconSymbol size={18} name="bag.fill" color={color} />,
        }}
      />
      <Tabs.Screen
        name="market"
        options={{
          title: '占卜',
          tabBarIcon: ({ color }) => <IconSymbol size={18} name="sparkles" color={color} />,
        }}
      />
      <Tabs.Screen
        name="matchmaking"
        options={{
          title: '合婚',
          tabBarIcon: ({ color }) => <IconSymbol size={18} name="heart.fill" color={color} />,
        }}
      />
      <Tabs.Screen
        name="profile"
        options={{
          title: '我的',
          tabBarIcon: ({ color }) => <IconSymbol size={18} name="person.fill" color={color} />,
        }}
      />
    </Tabs>
  );
}
