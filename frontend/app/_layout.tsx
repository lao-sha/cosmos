/**
 * 星尘玄鉴 - 应用根布局
 * 主题色：金棕色 #B2955D
 */

import { Slot } from 'expo-router';
import { View, StyleSheet, Text } from 'react-native';
import { StatusBar } from 'expo-status-bar';
import React from 'react';

// 主题色
const THEME_BG = '#F5F5F7';
const DARK_BG = '#0a0a0f';

export default function RootLayout() {
  console.log('[RootLayout] Rendering...');
  
  // 添加错误边界
  const [error, setError] = React.useState<Error | null>(null);
  
  if (error) {
    return (
      <View style={styles.errorContainer}>
        <Text style={styles.errorText}>❌ 错误</Text>
        <Text style={styles.errorMessage}>{error.message}</Text>
      </View>
    );
  }
  
  return (
    <View style={styles.container}>
      <StatusBar style="light" />
      <View style={styles.content}>
        <Text style={styles.testText}>✅ 布局已加载</Text>
        <Slot />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: DARK_BG,
  },
  content: {
    flex: 1,
    backgroundColor: THEME_BG,
  },
  testText: {
    position: 'absolute',
    top: 50,
    left: 20,
    color: '#00ff00',
    fontSize: 16,
    zIndex: 999,
  },
  errorContainer: {
    flex: 1,
    backgroundColor: '#000',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  errorText: {
    color: '#ff0000',
    fontSize: 24,
    marginBottom: 10,
  },
  errorMessage: {
    color: '#fff',
    fontSize: 14,
    textAlign: 'center',
  },
});
