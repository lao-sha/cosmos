/**
 * 聊天模块路由布局
 */

import { Stack } from 'expo-router';

export default function ChatLayout() {
  return (
    <Stack screenOptions={{ headerShown: false }}>
      <Stack.Screen name="index" options={{ title: '消息' }} />
      <Stack.Screen name="[sessionId]" options={{ title: '聊天' }} />
      <Stack.Screen name="new" options={{ title: '新建会话' }} />
    </Stack>
  );
}
