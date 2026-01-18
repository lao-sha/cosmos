/**
 * 星尘玄鉴 - 应用根布局
 * 主题色：金棕色 #B2955D
 */

import { Slot } from 'expo-router';
import { View, StyleSheet, Text } from 'react-native';
import { StatusBar } from 'expo-status-bar';
import { WalletAuthGate } from '@/features/wallet';

// 主题色
const THEME_BG = '#F5F5F7';
const DARK_BG = '#0a0a0f';

export default function RootLayout() {
  return (
    <View style={styles.container}>
      <StatusBar style="light" />
      <WalletAuthGate>
        <View style={styles.content}>
          <Slot />
        </View>
      </WalletAuthGate>
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
});
