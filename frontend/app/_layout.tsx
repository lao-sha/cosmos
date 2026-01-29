import { Buffer } from 'buffer';
import 'fast-text-encoding';

if (typeof global.Buffer === 'undefined') {
  global.Buffer = Buffer;
}

import { DarkTheme, DefaultTheme, ThemeProvider } from '@react-navigation/native';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Stack } from 'expo-router';
import { StatusBar } from 'expo-status-bar';
import { useEffect } from 'react';
import 'react-native-reanimated';

import { useColorScheme } from '@/hooks/use-color-scheme';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useWalletStore } from '@/src/stores/wallet';

const queryClient = new QueryClient();

// 默认连接到本地节点或测试网
const RPC_ENDPOINT = 'ws://127.0.0.1:9944';

export const unstable_settings = {
  anchor: '(tabs)',
};

export default function RootLayout() {
  const colorScheme = useColorScheme();
  const initializeAuth = useAuthStore((state) => state.initialize);
  const connectChain = useChainStore((state) => state.connect);
  const initializeWallet = useWalletStore((state) => state.initialize);

  useEffect(() => {
    const init = async () => {
      try {
        // 初始化钱包（兼容旧版）
        await initializeAuth();
        // 初始化多账户系统
        await initializeWallet();
        // 连接区块链
        await connectChain(RPC_ENDPOINT);
        console.log('App initialized, wallet and chain connected');
      } catch (error) {
        console.error('Initialization error:', error);
      }
    };
    init();
  }, []);

  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider value={colorScheme === 'dark' ? DarkTheme : DefaultTheme}>
        <Stack>
          <Stack.Screen name="(tabs)" options={{ headerShown: false }} />
          <Stack.Screen name="modal" options={{ presentation: 'modal', title: 'Modal' }} />
        </Stack>
        <StatusBar style="auto" />
      </ThemeProvider>
    </QueryClientProvider>
  );
}
