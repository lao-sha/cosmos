/**
 * 小六壬历史记录页面
 */

import React from 'react';
import { View, StyleSheet, Pressable, Text } from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { DivinationHistory } from '@/components/DivinationHistory';
import { BottomNavBar } from '@/components/BottomNavBar';
import { DivinationType, DivinationRecord } from '@/services/divination.service';

const THEME_COLOR = '#B2955D';

export default function XiaoliurenListPage() {
  const router = useRouter();

  const handleRecordPress = (record: DivinationRecord) => {
    // TODO: 导航到详情页
    console.log('查看小六壬记录:', record);
  };

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Text style={styles.headerTitle}>小六壬记录</Text>
        <View style={styles.backButton} />
      </View>

      {/* 历史记录列表 */}
      <DivinationHistory
        divinationType={DivinationType.Xiaoliuren}
        onRecordPress={handleRecordPress}
        emptyMessage="暂无小六壬记录"
      />

      <BottomNavBar activeTab="divination" />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingTop: 50,
    paddingHorizontal: 20,
    paddingBottom: 12,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  backButton: {
    padding: 4,
    width: 40,
  },
  headerTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
});
