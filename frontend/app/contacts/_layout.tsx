/**
 * 通讯录模块路由布局
 */

import { Stack } from 'expo-router';

export default function ContactsLayout() {
  return (
    <Stack screenOptions={{ headerShown: false }}>
      <Stack.Screen name="index" options={{ title: '通讯录' }} />
      <Stack.Screen name="add" options={{ title: '添加联系人' }} />
      <Stack.Screen name="requests" options={{ title: '好友申请' }} />
      <Stack.Screen name="blacklist" options={{ title: '黑名单' }} />
      <Stack.Screen name="[address]" options={{ title: '联系人详情' }} />
    </Stack>
  );
}
