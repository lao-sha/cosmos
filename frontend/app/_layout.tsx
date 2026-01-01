/**
 * 星尘玄鉴 - 应用根布局
 * 主题色：金棕色 #B2955D
 */

import { Slot } from 'expo-router';
import { View, StyleSheet } from 'react-native';
import { StatusBar } from 'expo-status-bar';

// 主题色
const THEME_BG = '#F5F5F7';

export default function RootLayout() {
  return (
    <View style={styles.container}>
      <StatusBar style="dark" />
      <Slot />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME_BG,
  },
});
