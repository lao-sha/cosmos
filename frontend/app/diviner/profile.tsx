/**
 * 占卜师资料编辑页面
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TextInput,
  Pressable,
  Switch,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import {
  SpecialtySelector,
  DivinationTypeSelector,
  StatusBadge,
  TierBadge,
  Provider,
  ProviderStatus,
  ProviderTier,
} from '@/features/diviner';

const THEME_COLOR = '#B2955D';

export default function DivinerProfilePage() {
  const router = useRouter();
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [provider, setProvider] = useState<Provider | null>(null);

  // 表单状态
  const [name, setName] = useState('');
  const [bio, setBio] = useState('');
  const [specialties, setSpecialties] = useState(0);
  const [supportedTypes, setSupportedTypes] = useState(0);
  const [acceptsUrgent, setAcceptsUrgent] = useState(false);

  useEffect(() => {
    // TODO: 从链上加载占卜师信息
    setTimeout(() => {
      const mockProvider: Provider = {
        account: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        name: '玄机子',
        bio: '从业20年，专注事业财运分析，擅长梅花易数和八字命理。',
        specialties: 0b0000_0101,
        supportedTypes: 0b0000_0011,
        status: ProviderStatus.Active,
        tier: ProviderTier.Certified,
        totalOrders: 156,
        completedOrders: 148,
        totalEarnings: BigInt(15680 * 1e10),
        averageRating: 4.8,
        ratingCount: 142,
        acceptsUrgent: true,
        registeredAt: Date.now() - 86400000 * 180,
      };
      setProvider(mockProvider);
      setName(mockProvider.name);
      setBio(mockProvider.bio);
      setSpecialties(mockProvider.specialties);
      setSupportedTypes(mockProvider.supportedTypes);
      setAcceptsUrgent(mockProvider.acceptsUrgent);
      setLoading(false);
    }, 500);
  }, []);

  const handleSave = async () => {
    if (!name.trim() || !bio.trim()) {
      Alert.alert('提示', '请填写完整信息');
      return;
    }
    if (specialties === 0) {
      Alert.alert('提示', '请至少选择一项擅长领域');
      return;
    }
    if (supportedTypes === 0) {
      Alert.alert('提示', '请至少选择一种占卜类型');
      return;
    }

    setSaving(true);
    try {
      // TODO: 调用链上更新方法
      await new Promise(resolve => setTimeout(resolve, 1500));
      Alert.alert('成功', '资料已更新');
    } catch (error: any) {
      Alert.alert('保存失败', error.message || '请稍后重试');
    } finally {
      setSaving(false);
    }
  };

  const handlePause = () => {
    Alert.alert('暂停接单', '暂停后您将不会收到新订单，确定要暂停吗？', [
      { text: '取消', style: 'cancel' },
      {
        text: '确定暂停',
        onPress: async () => {
          // TODO: 调用链上暂停方法
          setProvider(prev => prev ? { ...prev, status: ProviderStatus.Paused } : null);
          Alert.alert('成功', '已暂停接单');
        },
      },
    ]);
  };

  const handleResume = async () => {
    // TODO: 调用链上恢复方法
    setProvider(prev => prev ? { ...prev, status: ProviderStatus.Active } : null);
    Alert.alert('成功', '已恢复接单');
  };

  const handleDeactivate = () => {
    Alert.alert(
      '注销账户',
      '注销后您的占卜师身份将被移除，保证金将退还。此操作不可撤销，确定要注销吗？',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确定注销',
          style: 'destructive',
          onPress: async () => {
            // TODO: 调用链上注销方法
            Alert.alert('成功', '账户已注销，保证金已退还', [
              { text: '确定', onPress: () => router.replace('/diviner' as any) },
            ]);
          },
        },
      ]
    );
  };

  if (loading) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="编辑资料" />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  if (!provider) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="编辑资料" />
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyText}>未找到占卜师信息</Text>
        </View>
        <BottomNavBar activeTab="profile" />
      </View>
    );
  }

  return (
    <View style={styles.wrapper}>
      <PageHeader title="编辑资料" />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 状态信息 */}
        <View style={styles.statusCard}>
          <View style={styles.statusRow}>
            <TierBadge tier={provider.tier} />
            <StatusBadge status={provider.status} />
          </View>
          {provider.status === ProviderStatus.Active && (
            <Pressable style={styles.pauseBtn} onPress={handlePause}>
              <Text style={styles.pauseBtnText}>暂停接单</Text>
            </Pressable>
          )}
          {provider.status === ProviderStatus.Paused && (
            <Pressable style={styles.resumeBtn} onPress={handleResume}>
              <Text style={styles.resumeBtnText}>恢复接单</Text>
            </Pressable>
          )}
        </View>

        {/* 基本信息 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>基本信息</Text>
          <View style={styles.formCard}>
            <View style={styles.formItem}>
              <Text style={styles.label}>显示名称</Text>
              <TextInput
                style={styles.input}
                value={name}
                onChangeText={setName}
                placeholder="您的占卜师名称"
                placeholderTextColor="#999"
                maxLength={64}
              />
            </View>

            <View style={styles.formItem}>
              <Text style={styles.label}>个人简介</Text>
              <TextInput
                style={styles.textArea}
                value={bio}
                onChangeText={setBio}
                placeholder="介绍您的从业经历、擅长领域等"
                placeholderTextColor="#999"
                multiline
                numberOfLines={4}
                maxLength={256}
                textAlignVertical="top"
              />
              <Text style={styles.charCount}>{bio.length}/256</Text>
            </View>
          </View>
        </View>

        {/* 擅长领域 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>擅长领域</Text>
          <View style={styles.formCard}>
            <SpecialtySelector value={specialties} onChange={setSpecialties} />
          </View>
        </View>

        {/* 占卜类型 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>支持的占卜类型</Text>
          <View style={styles.formCard}>
            <DivinationTypeSelector value={supportedTypes} onChange={setSupportedTypes} />
          </View>
        </View>

        {/* 服务设置 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>服务设置</Text>
          <View style={styles.formCard}>
            <View style={styles.switchRow}>
              <View style={styles.switchLabel}>
                <Text style={styles.label}>接受加急订单</Text>
                <Text style={styles.switchHint}>开启后用户可选择加急服务</Text>
              </View>
              <Switch
                value={acceptsUrgent}
                onValueChange={setAcceptsUrgent}
                trackColor={{ false: '#E8E8E8', true: `${THEME_COLOR}80` }}
                thumbColor={acceptsUrgent ? THEME_COLOR : '#FFF'}
              />
            </View>
          </View>
        </View>

        {/* 保存按钮 */}
        <View style={styles.actionSection}>
          <Pressable
            style={[styles.saveBtn, saving && styles.saveBtnDisabled]}
            onPress={handleSave}
            disabled={saving}
          >
            {saving ? (
              <ActivityIndicator color="#FFF" />
            ) : (
              <Text style={styles.saveBtnText}>保存修改</Text>
            )}
          </Pressable>
        </View>

        {/* 危险操作 */}
        <View style={styles.dangerSection}>
          <Pressable style={styles.deactivateBtn} onPress={handleDeactivate}>
            <Text style={styles.deactivateBtnText}>注销占卜师账户</Text>
          </Pressable>
          <Text style={styles.dangerNote}>注销后保证金将退还，此操作不可撤销</Text>
        </View>
      </ScrollView>

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
    paddingBottom: 100,
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyText: {
    fontSize: 16,
    color: '#999',
  },
  statusCard: {
    backgroundColor: '#FFF',
    margin: 16,
    borderRadius: 12,
    padding: 16,
  },
  statusRow: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 12,
  },
  pauseBtn: {
    height: 40,
    borderWidth: 1,
    borderColor: '#FF9500',
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  pauseBtnText: {
    fontSize: 14,
    color: '#FF9500',
  },
  resumeBtn: {
    height: 40,
    backgroundColor: '#4CD964',
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  resumeBtnText: {
    fontSize: 14,
    color: '#FFF',
    fontWeight: '500',
  },
  section: {
    paddingHorizontal: 16,
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000',
    marginBottom: 12,
  },
  formCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
  },
  formItem: {
    marginBottom: 16,
  },
  label: {
    fontSize: 14,
    color: '#333',
    marginBottom: 8,
    fontWeight: '500',
  },
  input: {
    height: 44,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 8,
    paddingHorizontal: 12,
    fontSize: 14,
    color: '#333',
    backgroundColor: '#FAFAFA',
  },
  textArea: {
    height: 100,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
    fontSize: 14,
    color: '#333',
    backgroundColor: '#FAFAFA',
  },
  charCount: {
    fontSize: 12,
    color: '#999',
    textAlign: 'right',
    marginTop: 4,
  },
  switchRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  switchLabel: {
    flex: 1,
  },
  switchHint: {
    fontSize: 12,
    color: '#999',
    marginTop: 2,
  },
  actionSection: {
    paddingHorizontal: 16,
    marginBottom: 24,
  },
  saveBtn: {
    height: 52,
    backgroundColor: THEME_COLOR,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  saveBtnDisabled: {
    opacity: 0.5,
  },
  saveBtnText: {
    fontSize: 18,
    color: '#FFF',
    fontWeight: '600',
  },
  dangerSection: {
    paddingHorizontal: 16,
    alignItems: 'center',
    marginBottom: 24,
  },
  deactivateBtn: {
    paddingVertical: 12,
  },
  deactivateBtnText: {
    fontSize: 14,
    color: '#FF3B30',
  },
  dangerNote: {
    fontSize: 12,
    color: '#999',
    marginTop: 4,
  },
});
