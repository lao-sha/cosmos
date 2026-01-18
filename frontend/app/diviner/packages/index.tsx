/**
 * 套餐管理列表页面
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  ActivityIndicator,
  RefreshControl,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { PackageCard, ServicePackage, DivinationType, ServiceType } from '@/features/diviner';

const THEME_COLOR = '#B2955D';
const MAX_PACKAGES = 10;

// Mock 数据
const mockPackages: ServicePackage[] = [
  {
    id: 1,
    providerId: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    divinationType: DivinationType.Meihua,
    serviceType: ServiceType.TextReading,
    name: '梅花易数·文字详解',
    description: '根据您的问题起卦，提供详细的卦象分析和建议，包含体用关系、五行生克等深度解读。',
    price: BigInt(10 * 1e10),
    duration: 0,
    followUpCount: 3,
    urgentAvailable: true,
    urgentSurcharge: 5000,
    isActive: true,
    salesCount: 89,
  },
  {
    id: 2,
    providerId: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    divinationType: DivinationType.Bazi,
    serviceType: ServiceType.VoiceReading,
    name: '八字命理·语音解读',
    description: '根据您的出生时间排盘，通过语音详细讲解命盘格局、大运流年等。',
    price: BigInt(25 * 1e10),
    duration: 15,
    followUpCount: 5,
    urgentAvailable: false,
    urgentSurcharge: 0,
    isActive: true,
    salesCount: 45,
  },
];

export default function PackagesListPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [packages, setPackages] = useState<ServicePackage[]>([]);

  const loadData = async () => {
    await new Promise(resolve => setTimeout(resolve, 500));
    setPackages(mockPackages);
  };

  useEffect(() => {
    loadData().finally(() => setLoading(false));
  }, []);

  const onRefresh = async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  };

  const handleCreate = () => {
    if (packages.length >= MAX_PACKAGES) {
      Alert.alert('提示', `套餐数量已达上限（${MAX_PACKAGES}个）`);
      return;
    }
    router.push('/diviner/packages/create' as any);
  };

  const handleEdit = (id: number) => {
    router.push(`/diviner/packages/${id}` as any);
  };

  const handleToggle = (id: number, isActive: boolean) => {
    // TODO: 调用链上方法切换状态
    setPackages(prev =>
      prev.map(p => (p.id === id ? { ...p, isActive } : p))
    );
  };

  const handleDelete = (id: number) => {
    Alert.alert('确认删除', '删除后无法恢复，确定要删除此套餐吗？', [
      { text: '取消', style: 'cancel' },
      {
        text: '删除',
        style: 'destructive',
        onPress: () => {
          // TODO: 调用链上方法删除
          setPackages(prev => prev.filter(p => p.id !== id));
        },
      },
    ]);
  };

  if (loading) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="套餐管理" />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  return (
    <View style={styles.wrapper}>
      <PageHeader title="套餐管理" />

      <ScrollView
        style={styles.container}
        contentContainerStyle={styles.contentContainer}
        refreshControl={<RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor={THEME_COLOR} />}
      >
        {/* 套餐数量提示 */}
        <View style={styles.countSection}>
          <Text style={styles.countText}>
            已创建 {packages.length}/{MAX_PACKAGES} 个套餐
          </Text>
        </View>

        {/* 套餐列表 */}
        <View style={styles.section}>
          {packages.length === 0 ? (
            <View style={styles.emptyContainer}>
              <Text style={styles.emptyIcon}>📦</Text>
              <Text style={styles.emptyText}>还没有创建套餐</Text>
              <Text style={styles.emptySubtext}>创建服务套餐，开始接单赚钱</Text>
            </View>
          ) : (
            packages.map(pkg => (
              <PackageCard
                key={pkg.id}
                package={pkg}
                editable
                onEdit={() => handleEdit(pkg.id)}
                onToggle={(isActive) => handleToggle(pkg.id, isActive)}
                onDelete={() => handleDelete(pkg.id)}
              />
            ))
          )}
        </View>
      </ScrollView>

      {/* 创建按钮 */}
      <View style={styles.footer}>
        <Pressable
          style={[styles.createBtn, packages.length >= MAX_PACKAGES && styles.createBtnDisabled]}
          onPress={handleCreate}
        >
          <Text style={styles.createBtnText}>+ 创建新套餐</Text>
        </Pressable>
      </View>

      <BottomNavBar activeTab="profile" />
    </View>
  );
}

const styles = StyleSheet.create({
  wrapper: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  container: {
    flex: 1,
  },
  contentContainer: {
    paddingBottom: 160,
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  countSection: {
    padding: 16,
    paddingBottom: 0,
  },
  countText: {
    fontSize: 14,
    color: '#666',
  },
  section: {
    padding: 16,
  },
  emptyContainer: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 48,
    alignItems: 'center',
  },
  emptyIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  emptyText: {
    fontSize: 16,
    color: '#333',
    marginBottom: 8,
  },
  emptySubtext: {
    fontSize: 14,
    color: '#999',
  },
  footer: {
    position: 'absolute',
    bottom: 80,
    left: 0,
    right: 0,
    padding: 16,
    backgroundColor: '#F5F5F7',
  },
  createBtn: {
    height: 52,
    backgroundColor: THEME_COLOR,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  createBtnDisabled: {
    opacity: 0.5,
  },
  createBtnText: {
    fontSize: 16,
    color: '#FFF',
    fontWeight: '600',
  },
});
