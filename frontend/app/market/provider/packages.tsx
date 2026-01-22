// frontend/app/market/provider/packages.tsx

import React, { useEffect, useState, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
  RefreshControl,
  SafeAreaView,
  StatusBar,
  Alert,
  Modal,
  TextInput,
  ScrollView,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useRouter } from 'expo-router';
import { useWalletStore } from '@/stores/wallet.store';
import { useMarketApi, useChainTransaction } from '@/divination/market/hooks';
import {
  PackageCard,
  LoadingSpinner,
  EmptyState,
} from '@/divination/market/components';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { ServicePackage, DivinationType } from '@/divination/market/types';
import { DIVINATION_TYPES } from '@/divination/market/constants/market.constants';

export default function PackageManagementScreen() {
  const router = useRouter();
  const { address } = useWalletStore();
  const { getProviderPackages, loading } = useMarketApi();
  const { createPackage, updatePackage, deletePackage, isProcessing } = useChainTransaction();

  const [packages, setPackages] = useState<ServicePackage[]>([]);
  const [refreshing, setRefreshing] = useState(false);
  const [isModalVisible, setIsModalVisible] = useState(false);
  const [editingPackage, setEditingPackage] = useState<ServicePackage | null>(null);

  // 表单状态
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [price, setPrice] = useState('');
  const [duration, setDuration] = useState('1'); // 默认1天
  const [maxFollowUps, setMaxFollowUps] = useState('2'); // 默认2次
  const [divinationType, setDivinationType] = useState<DivinationType>(0);

  const loadPackages = useCallback(async () => {
    if (!address) return;
    try {
      const result = await getProviderPackages(address);
      setPackages(result);
    } catch (err) {
      console.error('Load packages error:', err);
    }
  }, [address, getProviderPackages]);

  useEffect(() => {
    loadPackages();
  }, [loadPackages]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await loadPackages();
    setRefreshing(false);
  }, [loadPackages]);

  const resetForm = () => {
    setName('');
    setDescription('');
    setPrice('');
    setDuration('1');
    setDivinationType(0);
    setEditingPackage(null);
  };

  const handleAddPackage = () => {
    resetForm();
    setIsModalVisible(true);
  };

  const handleEditPackage = (pkg: ServicePackage) => {
    setEditingPackage(pkg);
    setName(pkg.name);
    setDescription(pkg.description);
    setPrice((Number(pkg.price) / 1000000).toString()); // 假设 DUST 有 6 位小数
    setDuration(pkg.duration.toString());
    setMaxFollowUps(pkg.followUpCount.toString());
    setDivinationType(pkg.divinationType);
    setIsModalVisible(true);
  };

  const handleTogglePackage = async (pkg: ServicePackage) => {
    await updatePackage(
      {
        packageId: pkg.id,
        isActive: !pkg.isActive,
      },
      {
        onSuccess: () => {
          Alert.alert('成功', `套餐已${pkg.isActive ? '下架' : '上架'}`);
          loadPackages();
        },
      }
    );
  };

  const handleDeletePackage = (pkg: ServicePackage) => {
    Alert.alert('删除套餐', `确定要删除套餐 "${pkg.name}" 吗？`, [
      { text: '取消', style: 'cancel' },
      {
        text: '删除',
        style: 'destructive',
        onPress: async () => {
          await deletePackage(pkg.id, {
            onSuccess: () => {
              Alert.alert('成功', '套餐已删除');
              loadPackages();
            },
          });
        },
      },
    ]);
  };

  const handleSubmit = async () => {
    if (!name.trim() || !description.trim() || !price || !duration) {
      Alert.alert('提示', '请填写完整信息');
      return;
    }

    const priceBigInt = BigInt(Math.floor(parseFloat(price) * 1000000));
    const durationNum = parseInt(duration, 10);
    const maxFollowUpsNum = parseInt(maxFollowUps, 10);

    try {
      if (editingPackage) {
        await updatePackage(
          {
            packageId: editingPackage.id,
            name,
            description,
            price: priceBigInt,
            deliveryDays: durationNum,
            maxFollowUps: maxFollowUpsNum,
          },
          {
            onSuccess: () => {
              Alert.alert('成功', '套餐已更新');
              setIsModalVisible(false);
              loadPackages();
            },
          }
        );
      } else {
        await createPackage(
          {
            name,
            description,
            divinationType,
            price: priceBigInt,
            deliveryDays: durationNum,
            maxFollowUps: maxFollowUpsNum,
          },
          {
            onSuccess: () => {
              Alert.alert('成功', '套餐已创建');
              setIsModalVisible(false);
              loadPackages();
            },
          }
        );
      }
    } catch (err) {
      Alert.alert('错误', '操作失败: ' + (err instanceof Error ? err.message : '未知错误'));
    }
  };

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="dark-content" backgroundColor={THEME.card} />

      {/* 顶部导航 */}
      <View style={styles.header}>
        <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
          <Ionicons name="arrow-back" size={24} color={THEME.text} />
        </TouchableOpacity>
        <Text style={styles.headerTitle}>套餐管理</Text>
        <TouchableOpacity onPress={handleAddPackage} style={styles.backBtn}>
          <Ionicons name="add" size={28} color={THEME.primary} />
        </TouchableOpacity>
      </View>

      {/* 套餐列表 */}
      <FlatList
        data={packages}
        keyExtractor={(item) => item.id.toString()}
        contentContainerStyle={styles.listContent}
        refreshControl={
          <RefreshControl
            refreshing={refreshing || isProcessing}
            onRefresh={onRefresh}
            colors={[THEME.primary]}
          />
        }
        renderItem={({ item }) => (
          <View style={styles.packageWrapper}>
            <PackageCard package_={item} />
            <View style={styles.packageActions}>
              <TouchableOpacity style={styles.actionBtn} onPress={() => handleTogglePackage(item)}>
                <Ionicons 
                  name={item.isActive ? "eye-off-outline" : "eye-outline"} 
                  size={18} 
                  color={item.isActive ? THEME.warning : THEME.success} 
                />
                <Text style={[styles.actionText, { color: item.isActive ? THEME.warning : THEME.success }]}>
                  {item.isActive ? '下架' : '上架'}
                </Text>
              </TouchableOpacity>
              <TouchableOpacity style={styles.actionBtn} onPress={() => handleEditPackage(item)}>
                <Ionicons name="create-outline" size={18} color={THEME.primary} />
                <Text style={styles.actionText}>编辑</Text>
              </TouchableOpacity>
              <TouchableOpacity style={styles.actionBtn} onPress={() => handleDeletePackage(item)}>
                <Ionicons name="trash-outline" size={18} color={THEME.error} />
                <Text style={[styles.actionText, { color: THEME.error }]}>删除</Text>
              </TouchableOpacity>
            </View>
          </View>
        )}
        ListEmptyComponent={
          loading ? (
            <LoadingSpinner text="加载中..." />
          ) : (
            <EmptyState
              icon="pricetag-outline"
              title="暂无套餐"
              description="点击右上角添加您的第一个占卜服务套餐"
              actionText="立即添加"
              onAction={handleAddPackage}
            />
          )
        }
      />

      {/* 编辑/新增 Modal */}
      <Modal visible={isModalVisible} animationType="slide" transparent>
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <View style={styles.modalHeader}>
              <Text style={styles.modalTitle}>{editingPackage ? '编辑套餐' : '新增套餐'}</Text>
              <TouchableOpacity onPress={() => setIsModalVisible(false)}>
                <Ionicons name="close" size={24} color={THEME.text} />
              </TouchableOpacity>
            </View>

            <ScrollView style={styles.form}>
              <Text style={styles.label}>套餐名称</Text>
              <TextInput
                style={styles.input}
                placeholder="例如：深度八字解析"
                value={name}
                onChangeText={setName}
              />

              <Text style={styles.label}>服务描述</Text>
              <TextInput
                style={[styles.input, styles.textArea]}
                placeholder="请详细说明服务内容..."
                value={description}
                onChangeText={setDescription}
                multiline
                numberOfLines={4}
                textAlignVertical="top"
              />

              <View style={styles.row}>
                <View style={styles.col}>
                  <Text style={styles.label}>价格 (DUST)</Text>
                  <TextInput
                    style={styles.input}
                    placeholder="0.00"
                    value={price}
                    onChangeText={setPrice}
                    keyboardType="decimal-pad"
                  />
                </View>
                <View style={styles.col}>
                  <Text style={styles.label}>预计交付 (天)</Text>
                  <TextInput
                    style={styles.input}
                    placeholder="1"
                    value={duration}
                    onChangeText={setDuration}
                    keyboardType="number-pad"
                  />
                </View>
                <View style={styles.col}>
                  <Text style={styles.label}>追问次数</Text>
                  <TextInput
                    style={styles.input}
                    placeholder="2"
                    value={maxFollowUps}
                    onChangeText={setMaxFollowUps}
                    keyboardType="number-pad"
                  />
                </View>
              </View>

              {!editingPackage && (
                <>
                  <Text style={styles.label}>占卜类型</Text>
                  <View style={styles.typeGrid}>
                    {DIVINATION_TYPES.map((type) => (
                      <TouchableOpacity
                        key={type.value}
                        style={[
                          styles.typeItem,
                          divinationType === type.value && styles.typeItemActive,
                        ]}
                        onPress={() => setDivinationType(type.value)}
                      >
                        <Text
                          style={[
                            styles.typeItemText,
                            divinationType === type.value && styles.typeItemTextActive,
                          ]}
                        >
                          {type.label}
                        </Text>
                      </TouchableOpacity>
                    ))}
                  </View>
                </>
              )}

              <TouchableOpacity
                style={[styles.submitBtn, isProcessing && styles.btnDisabled]}
                onPress={handleSubmit}
                disabled={isProcessing}
              >
                <Text style={styles.submitBtnText}>
                  {isProcessing ? '处理中...' : editingPackage ? '保存修改' : '立即创建'}
                </Text>
              </TouchableOpacity>
            </ScrollView>
          </View>
        </View>
      </Modal>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME.background,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: THEME.card,
    paddingHorizontal: 8,
    paddingVertical: 10,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.border,
  },
  backBtn: {
    padding: 8,
    width: 48,
    alignItems: 'center',
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: THEME.text,
  },
  listContent: {
    padding: 16,
  },
  packageWrapper: {
    marginBottom: 16,
  },
  packageActions: {
    flexDirection: 'row',
    backgroundColor: THEME.card,
    marginTop: -10,
    borderBottomLeftRadius: 10,
    borderBottomRightRadius: 10,
    paddingHorizontal: 16,
    paddingVertical: 12,
    justifyContent: 'flex-end',
    gap: 20,
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: THEME.borderLight,
  },
  actionBtn: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  actionText: {
    fontSize: 13,
    color: THEME.primary,
    fontWeight: '500',
  },
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.5)',
    justifyContent: 'flex-end',
  },
  modalContent: {
    backgroundColor: THEME.card,
    borderTopLeftRadius: 20,
    borderTopRightRadius: 20,
    padding: 20,
    maxHeight: '80%',
  },
  modalHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 20,
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: THEME.text,
  },
  form: {
    marginBottom: 20,
  },
  label: {
    fontSize: 14,
    color: THEME.textSecondary,
    marginBottom: 8,
  },
  input: {
    backgroundColor: THEME.background,
    borderRadius: 8,
    padding: 12,
    fontSize: 15,
    color: THEME.text,
    marginBottom: 16,
    borderWidth: 1,
    borderColor: THEME.border,
  },
  textArea: {
    height: 100,
  },
  row: {
    flexDirection: 'row',
    gap: 12,
  },
  col: {
    flex: 1,
  },
  typeGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    marginBottom: 20,
  },
  typeItem: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 15,
    backgroundColor: THEME.background,
    borderWidth: 1,
    borderColor: THEME.border,
  },
  typeItemActive: {
    backgroundColor: THEME.primary + '15',
    borderColor: THEME.primary,
  },
  typeItemText: {
    fontSize: 12,
    color: THEME.textSecondary,
  },
  typeItemTextActive: {
    color: THEME.primary,
    fontWeight: '500',
  },
  submitBtn: {
    backgroundColor: THEME.primary,
    borderRadius: 10,
    paddingVertical: 14,
    alignItems: 'center',
    marginTop: 10,
  },
  submitBtnText: {
    color: THEME.textInverse,
    fontSize: 16,
    fontWeight: '600',
  },
  btnDisabled: {
    opacity: 0.6,
  },
});

