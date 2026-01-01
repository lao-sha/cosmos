/**
 * 星尘玄鉴 - 底部标签页布局
 * 主题色：金棕色 #B2955D
 */

import { Tabs } from 'expo-router';
import { Platform, View, StyleSheet } from 'react-native';
import { HomeOutline, CompassOutline, MessageOutline, UserOutline } from '@/components/TabIcons';

const THEME_COLOR = '#B2955D';

export default function TabsLayout() {
  return (
    <View style={styles.wrapper}>
      <Tabs
        screenOptions={{
          headerShown: false,
          tabBarActiveTintColor: THEME_COLOR,
          tabBarInactiveTintColor: '#999',
          tabBarStyle: {
            backgroundColor: '#FFF',
            borderTopColor: '#F0F0F0',
            borderTopWidth: 1,
            height: Platform.select({ ios: 88, android: 64, default: 64 }),
            paddingBottom: Platform.select({ ios: 24, android: 8, default: 8 }),
            paddingTop: 8,
            maxWidth: 414,
            width: '100%',
            alignSelf: 'center',
            position: 'absolute',
            bottom: 0,
            left: '50%',
            transform: [{ translateX: -207 }],
          },
          tabBarLabelStyle: {
            fontSize: 12,
            fontWeight: '500',
          },
        }}
      >
        <Tabs.Screen
          name="index"
          options={{
            title: '首页',
            tabBarIcon: ({ color, size }) => (
              <HomeOutline color={color} size={size} />
            ),
          }}
        />
        <Tabs.Screen
          name="divination"
          options={{
            title: '占卜',
            tabBarIcon: ({ color, size }) => (
              <CompassOutline color={color} size={size} />
            ),
          }}
        />
        <Tabs.Screen
          name="chat"
          options={{
            title: '消息',
            tabBarIcon: ({ color, size }) => (
              <MessageOutline color={color} size={size} />
            ),
          }}
        />
        <Tabs.Screen
          name="profile"
          options={{
            title: '我的',
            tabBarIcon: ({ color, size }) => (
              <UserOutline color={color} size={size} />
            ),
          }}
        />
      </Tabs>
    </View>
  );
}

const styles = StyleSheet.create({
  wrapper: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
});
