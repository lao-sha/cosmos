/**
 * 星尘玄鉴 - 认证页面布局
 */

import { Stack } from 'expo-router';

export default function AuthLayout() {
  return (
    <Stack
      screenOptions={{
        headerShown: false,
        contentStyle: {
          backgroundColor: '#0a0a0f',
        },
      }}
    />
  );
}
