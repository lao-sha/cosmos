/**
 * 星尘玄鉴 - 占卜模块布局
 */

import { Stack } from 'expo-router';

export default function DivinationLayout() {
  return (
    <Stack
      screenOptions={{
        headerShown: false,
      }}
    >
      <Stack.Screen name="bazi" />
      <Stack.Screen name="bazi-detail" />
      <Stack.Screen name="ziwei" />
      <Stack.Screen name="qimen" />
      <Stack.Screen name="daliuren" />
      <Stack.Screen name="liuyao" />
      <Stack.Screen name="meihua" />
      <Stack.Screen name="tarot" />
      <Stack.Screen name="xiaoliuren" />
    </Stack>
  );
}
