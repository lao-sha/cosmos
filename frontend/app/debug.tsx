/**
 * 调试页面 - 用于诊断启动问题
 */

import React, { useEffect, useState } from 'react';
import { View, Text, StyleSheet, ScrollView, Button } from 'react-native';
import { useWalletStore } from '@/stores/wallet.store';
import * as SecureStore from 'expo-secure-store';

export default function DebugPage() {
  const [logs, setLogs] = useState<string[]>([]);
  const walletState = useWalletStore();

  const addLog = (message: string) => {
    setLogs(prev => [...prev, `[${new Date().toLocaleTimeString()}] ${message}`]);
  };

  useEffect(() => {
    addLog('Debug page mounted');
    addLog(`isReady: ${walletState.isReady}`);
    addLog(`hasWallet: ${walletState.hasWallet}`);
    addLog(`isLocked: ${walletState.isLocked}`);
    addLog(`address: ${walletState.address}`);
  }, []);

  const testSecureStore = async () => {
    try {
      addLog('Testing SecureStore...');
      await SecureStore.setItemAsync('test_key', 'test_value');
      const value = await SecureStore.getItemAsync('test_key');
      addLog(`SecureStore test: ${value === 'test_value' ? 'PASS' : 'FAIL'}`);
      await SecureStore.deleteItemAsync('test_key');
    } catch (error) {
      addLog(`SecureStore error: ${error}`);
    }
  };

  const testInitialize = async () => {
    try {
      addLog('Testing wallet initialize...');
      await walletState.initialize();
      addLog('Initialize complete');
    } catch (error) {
      addLog(`Initialize error: ${error}`);
    }
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>调试信息</Text>
      
      <View style={styles.stateBox}>
        <Text style={styles.stateText}>isReady: {String(walletState.isReady)}</Text>
        <Text style={styles.stateText}>hasWallet: {String(walletState.hasWallet)}</Text>
        <Text style={styles.stateText}>isLocked: {String(walletState.isLocked)}</Text>
        <Text style={styles.stateText}>address: {walletState.address || 'null'}</Text>
        <Text style={styles.stateText}>error: {walletState.error || 'null'}</Text>
      </View>

      <View style={styles.buttons}>
        <Button title="测试 SecureStore" onPress={testSecureStore} />
        <Button title="测试初始化" onPress={testInitialize} />
      </View>

      <ScrollView style={styles.logContainer}>
        {logs.map((log, index) => (
          <Text key={index} style={styles.logText}>{log}</Text>
        ))}
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0a0a0f',
    padding: 20,
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 20,
  },
  stateBox: {
    backgroundColor: '#1a1a2e',
    padding: 15,
    borderRadius: 8,
    marginBottom: 20,
  },
  stateText: {
    color: '#8b5cf6',
    fontSize: 14,
    marginBottom: 5,
  },
  buttons: {
    flexDirection: 'row',
    gap: 10,
    marginBottom: 20,
  },
  logContainer: {
    flex: 1,
    backgroundColor: '#1a1a2e',
    padding: 10,
    borderRadius: 8,
  },
  logText: {
    color: '#9ca3af',
    fontSize: 12,
    marginBottom: 3,
  },
});
