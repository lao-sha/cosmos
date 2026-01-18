/**
 * 通用页面头部组件
 * 包含返回按钮和页面标题
 */

import React from 'react';
import { View, Text, StyleSheet, Pressable, Platform, StatusBar } from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';

const THEME_COLOR = '#B2955D';
const STATUSBAR_HEIGHT = Platform.OS === 'android' ? StatusBar.currentHeight || 24 : 44;

interface PageHeaderProps {
  title: string;
  showBack?: boolean;
  onBack?: () => void;
  rightElement?: React.ReactNode;
}

export function PageHeader({ 
  title, 
  showBack = true, 
  onBack,
  rightElement 
}: PageHeaderProps) {
  const router = useRouter();

  const handleBack = () => {
    if (onBack) {
      onBack();
    } else {
      router.back();
    }
  };

  return (
    <View style={styles.container}>
      <StatusBar barStyle="dark-content" backgroundColor="#FFFFFF" />
      <View style={styles.header}>
        {/* 左侧返回按钮 */}
        <View style={styles.left}>
          {showBack && (
            <Pressable style={styles.backButton} onPress={handleBack}>
              <Ionicons name="chevron-back" size={24} color="#000000" />
            </Pressable>
          )}
        </View>

        {/* 中间标题 */}
        <View style={styles.center}>
          <Text style={styles.title} numberOfLines={1}>{title}</Text>
        </View>

        {/* 右侧元素 */}
        <View style={styles.right}>
          {rightElement}
        </View>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#FFFFFF',
    paddingTop: STATUSBAR_HEIGHT,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    height: 48,
    paddingHorizontal: 4,
  },
  left: {
    width: 48,
    alignItems: 'flex-start',
    justifyContent: 'center',
  },
  center: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  right: {
    width: 48,
    alignItems: 'flex-end',
    justifyContent: 'center',
  },
  backButton: {
    width: 40,
    height: 40,
    alignItems: 'center',
    justifyContent: 'center',
  },
  title: {
    fontSize: 17,
    fontWeight: '600',
    color: '#000000',
  },
});
