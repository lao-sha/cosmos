import { useEffect } from 'react';
import { Stack } from 'expo-router';
import { StatusBar } from 'expo-status-bar';
import { QueryClientProvider } from '@tanstack/react-query';
import { queryClient } from '@/lib/queryClient';
import { useColorScheme } from 'react-native';
import { Colors } from '@/constants/colors';

export default function RootLayout() {
  const colorScheme = useColorScheme();
  const isDark = colorScheme === 'dark';
  const theme = isDark ? Colors.dark : Colors.light;

  return (
    <QueryClientProvider client={queryClient}>
      <StatusBar style={isDark ? 'light' : 'dark'} />
      <Stack
        screenOptions={{
          headerStyle: {
            backgroundColor: theme.background,
          },
          headerTintColor: theme.textPrimary,
          headerTitleStyle: {
            fontWeight: '600',
          },
          contentStyle: {
            backgroundColor: theme.background,
          },
        }}
      >
        <Stack.Screen name="(tabs)" options={{ headerShown: false }} />
        <Stack.Screen
          name="wallet/create"
          options={{ title: '创建钱包', presentation: 'modal' }}
        />
        <Stack.Screen
          name="wallet/import"
          options={{ title: '导入钱包', presentation: 'modal' }}
        />
        <Stack.Screen
          name="otc/index"
          options={{ title: 'OTC 交易' }}
        />
        <Stack.Screen
          name="otc/buy"
          options={{ title: '购买 COS' }}
        />
        <Stack.Screen
          name="otc/orders"
          options={{ title: '我的订单' }}
        />
        <Stack.Screen
          name="otc/order/[id]"
          options={{ title: '订单详情' }}
        />
        <Stack.Screen
          name="swap/index"
          options={{ title: '兑换' }}
        />
        <Stack.Screen
          name="maker/index"
          options={{ title: '做市商' }}
        />
        <Stack.Screen
          name="maker/apply"
          options={{ title: '申请做市商' }}
        />
        <Stack.Screen
          name="settings/kyc"
          options={{ title: 'KYC 认证' }}
        />
        <Stack.Screen
          name="chat/[id]"
          options={{ title: '聊天' }}
        />
        <Stack.Screen
          name="referral/index"
          options={{ title: '推荐邀请' }}
        />
        <Stack.Screen
          name="friends/index"
          options={{ title: '联系人' }}
        />
        <Stack.Screen
          name="friends/add"
          options={{ title: '添加好友' }}
        />
        <Stack.Screen
          name="membership/index"
          options={{ title: '会员等级' }}
        />
        <Stack.Screen
          name="referral/commission"
          options={{ title: '返佣收益' }}
        />
        <Stack.Screen
          name="profile/credit"
          options={{ title: '信用分' }}
        />
        <Stack.Screen
          name="governance/index"
          options={{ title: '治理投票' }}
        />
        <Stack.Screen
          name="token/index"
          options={{ title: '代币发售' }}
        />
        <Stack.Screen
          name="disputes/index"
          options={{ title: '争议仲裁' }}
        />
        <Stack.Screen
          name="disputes/create"
          options={{ title: '发起争议' }}
        />
      </Stack>
    </QueryClientProvider>
  );
}
