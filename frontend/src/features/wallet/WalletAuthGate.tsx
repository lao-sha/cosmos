/**
 * 星尘玄鉴 - 钱包认证入口
 * 根据钱包状态显示不同页面
 */

import React, { useEffect, useRef } from 'react';
import { View, ActivityIndicator, StyleSheet, Text } from 'react-native';
import { useWalletStore } from '@/stores/wallet.store';
import { useRouter, useSegments } from 'expo-router';

interface WalletAuthGateProps {
  children: React.ReactNode;
}

export function WalletAuthGate({ children }: WalletAuthGateProps) {
  const router = useRouter();
  const segments = useSegments();
  const { isReady, hasWallet, isLocked, initialize } = useWalletStore();
  const hasInitialized = useRef(false);
  const [initError, setInitError] = React.useState<string | null>(null);

  // 初始化钱包（只执行一次）
  useEffect(() => {
    if (!hasInitialized.current) {
      hasInitialized.current = true;
      console.log('[WalletAuthGate] Starting initialization...');
      
      // 添加超时保护
      const timeout = setTimeout(() => {
        console.error('[WalletAuthGate] Initialization timeout!');
        setInitError('初始化超时，请重启应用');
      }, 10000);
      
      initialize()
        .then(() => {
          console.log('[WalletAuthGate] Initialization success');
        })
        .catch((error) => {
          console.error('[WalletAuthGate] Initialization error:', error);
          setInitError('初始化失败: ' + (error?.message || '未知错误'));
        })
        .finally(() => {
          clearTimeout(timeout);
        });
    }
  }, [initialize]);

  // 根据状态导航
  useEffect(() => {
    if (!isReady) return;

    const inAuthGroup = segments[0] === 'auth';
    
    console.log('[WalletAuthGate] State:', { hasWallet, isLocked, inAuthGroup, segments });

    if (!hasWallet && !inAuthGroup) {
      // 没有钱包且不在认证页面，跳转到创建页面
      console.log('[WalletAuthGate] Navigating to /auth/create');
      router.replace('/auth/create');
    } else if (hasWallet && isLocked && !inAuthGroup) {
      // 钱包已锁定且不在认证页面，跳转到解锁页面
      console.log('[WalletAuthGate] Navigating to /auth/unlock');
      router.replace('/auth/unlock');
    } else if (hasWallet && !isLocked && inAuthGroup) {
      // 钱包已解锁但还在认证页面，跳转到主页
      console.log('[WalletAuthGate] Navigating to home');
      router.replace('/');
    }
  }, [isReady, hasWallet, isLocked, segments]);

  // 初始化错误
  if (initError) {
    return (
      <View style={styles.loadingContainer}>
        <Text style={styles.errorText}>⚠️ {initError}</Text>
        <Text style={styles.errorHint}>请检查控制台日志</Text>
      </View>
    );
  }

  // 加载中
  if (!isReady) {
    return (
      <View style={styles.loadingContainer}>
        <ActivityIndicator size="large" color="#8b5cf6" />
        <Text style={styles.loadingText}>正在初始化...</Text>
      </View>
    );
  }

  // 显示内容
  return <>{children}</>;
}

const styles = StyleSheet.create({
  loadingContainer: {
    flex: 1,
    backgroundColor: '#0a0a0f',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
  },
  loadingText: {
    color: '#8b5cf6',
    marginTop: 16,
    fontSize: 14,
  },
  errorText: {
    color: '#ef4444',
    fontSize: 16,
    textAlign: 'center',
    marginBottom: 8,
  },
  errorHint: {
    color: '#9ca3af',
    fontSize: 12,
    textAlign: 'center',
  },
});
