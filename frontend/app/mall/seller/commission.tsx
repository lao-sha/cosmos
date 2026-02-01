import { TrendingUp, Users, Layers, Award, Check } from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
  TextInput,
  Alert,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import type { Shop } from '@/src/types/sharemall';
import { useWalletStore } from '@/src/stores/wallet';

interface CommissionMode {
  id: string;
  name: string;
  description: string;
  icon: any;
  enabled: boolean;
}

export default function CommissionScreen() {
  const { currentAccount } = useWalletStore();
  const [myShop, setMyShop] = useState<Shop | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [loading, setLoading] = useState(false);

  const [modes, setModes] = useState<CommissionMode[]>([
    {
      id: 'direct',
      name: '直推奖励',
      description: '推荐新用户购买获得奖励',
      icon: Users,
      enabled: true,
    },
    {
      id: 'three_level',
      name: '三级分销',
      description: '三级推荐关系返佣',
      icon: Layers,
      enabled: false,
    },
    {
      id: 'team',
      name: '团队业绩',
      description: '团队销售业绩奖励',
      icon: TrendingUp,
      enabled: false,
    },
    {
      id: 'level',
      name: '等级奖励',
      description: '会员等级差额奖励',
      icon: Award,
      enabled: false,
    },
  ]);

  // 返佣比例配置
  const [directRate, setDirectRate] = useState('5');
  const [level1Rate, setLevel1Rate] = useState('3');
  const [level2Rate, setLevel2Rate] = useState('2');
  const [level3Rate, setLevel3Rate] = useState('1');

  const loadData = async () => {
    if (!currentAccount) return;
    try {
      const shops = await sharemallService.getShopList(100);
      const userShop = shops.find((s) => s.owner === currentAccount.address);
      setMyShop(userShop || null);

      // TODO: 从链上加载返佣配置
    } catch (error) {
      console.error('Failed to load commission config:', error);
    }
  };

  useEffect(() => {
    loadData();
  }, [currentAccount]);

  const onRefresh = async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  };

  const toggleMode = (modeId: string) => {
    setModes(
      modes.map((m) =>
        m.id === modeId ? { ...m, enabled: !m.enabled } : m
      )
    );
  };

  const handleSave = async () => {
    if (!myShop) return;

    setLoading(true);
    try {
      // TODO: 调用链上接口保存返佣配置
      Alert.alert('成功', '返佣配置已保存');
    } catch (error: any) {
      Alert.alert('错误', error.message || '保存失败');
    } finally {
      setLoading(false);
    }
  };

  return (
    <ScrollView
      style={styles.container}
      refreshControl={<RefreshControl refreshing={refreshing} onRefresh={onRefresh} />}
    >
      {/* 返佣模式选择 */}
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>返佣模式</Text>
        <Text style={styles.sectionHint}>可多选，多种模式叠加计算</Text>

        {modes.map((mode) => {
          const ModeIcon = mode.icon;
          return (
            <TouchableOpacity
              key={mode.id}
              style={[styles.modeCard, mode.enabled && styles.modeCardActive]}
              onPress={() => toggleMode(mode.id)}
            >
              <View
                style={[
                  styles.modeIcon,
                  { backgroundColor: mode.enabled ? '#E3F2FD' : '#f5f5f5' },
                ]}
              >
                <ModeIcon size={24} color={mode.enabled ? '#1976D2' : '#999'} />
              </View>
              <View style={styles.modeInfo}>
                <Text
                  style={[styles.modeName, mode.enabled && styles.modeNameActive]}
                >
                  {mode.name}
                </Text>
                <Text style={styles.modeDesc}>{mode.description}</Text>
              </View>
              <View
                style={[
                  styles.modeCheck,
                  mode.enabled && styles.modeCheckActive,
                ]}
              >
                {mode.enabled && <Check size={16} color="#fff" />}
              </View>
            </TouchableOpacity>
          );
        })}
      </View>

      {/* 直推奖励配置 */}
      {modes.find((m) => m.id === 'direct')?.enabled && (
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>直推奖励配置</Text>
          <View style={styles.inputGroup}>
            <Text style={styles.label}>直推返佣比例 (%)</Text>
            <TextInput
              style={styles.input}
              placeholder="5"
              keyboardType="decimal-pad"
              value={directRate}
              onChangeText={setDirectRate}
            />
          </View>
        </View>
      )}

      {/* 三级分销配置 */}
      {modes.find((m) => m.id === 'three_level')?.enabled && (
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>三级分销配置</Text>

          <View style={styles.inputGroup}>
            <Text style={styles.label}>一级返佣 (%)</Text>
            <TextInput
              style={styles.input}
              placeholder="3"
              keyboardType="decimal-pad"
              value={level1Rate}
              onChangeText={setLevel1Rate}
            />
          </View>

          <View style={styles.inputGroup}>
            <Text style={styles.label}>二级返佣 (%)</Text>
            <TextInput
              style={styles.input}
              placeholder="2"
              keyboardType="decimal-pad"
              value={level2Rate}
              onChangeText={setLevel2Rate}
            />
          </View>

          <View style={styles.inputGroup}>
            <Text style={styles.label}>三级返佣 (%)</Text>
            <TextInput
              style={styles.input}
              placeholder="1"
              keyboardType="decimal-pad"
              value={level3Rate}
              onChangeText={setLevel3Rate}
            />
          </View>
        </View>
      )}

      {/* 保存按钮 */}
      <View style={styles.section}>
        <TouchableOpacity
          style={[styles.saveBtn, loading && styles.saveBtnDisabled]}
          onPress={handleSave}
          disabled={loading}
        >
          <Text style={styles.saveBtnText}>
            {loading ? '保存中...' : '保存配置'}
          </Text>
        </TouchableOpacity>
      </View>

      {/* 说明 */}
      <View style={styles.notice}>
        <Text style={styles.noticeTitle}>返佣说明</Text>
        <Text style={styles.noticeText}>
          • 直推奖励：用户 A 推荐用户 B，B 购买后 A 获得返佣{'\n'}
          • 三级分销：支持三级推荐关系的返佣{'\n'}
          • 团队业绩：根据团队总销售额计算奖励{'\n'}
          • 等级奖励：高等级会员获得下级等级差额奖励{'\n'}
          • 返佣从店铺运营资金中扣除
        </Text>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  section: {
    backgroundColor: '#fff',
    margin: 12,
    marginBottom: 0,
    borderRadius: 12,
    padding: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 4,
  },
  sectionHint: {
    fontSize: 12,
    color: '#999',
    marginBottom: 16,
  },
  modeCard: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
    borderRadius: 8,
    backgroundColor: '#f9f9f9',
    marginBottom: 8,
  },
  modeCardActive: {
    backgroundColor: '#E3F2FD',
    borderWidth: 1,
    borderColor: '#1976D2',
  },
  modeIcon: {
    width: 48,
    height: 48,
    borderRadius: 24,
    alignItems: 'center',
    justifyContent: 'center',
  },
  modeInfo: {
    flex: 1,
    marginLeft: 12,
  },
  modeName: {
    fontSize: 14,
    fontWeight: '500',
    color: '#333',
  },
  modeNameActive: {
    color: '#1976D2',
  },
  modeDesc: {
    fontSize: 12,
    color: '#999',
    marginTop: 2,
  },
  modeCheck: {
    width: 24,
    height: 24,
    borderRadius: 12,
    borderWidth: 2,
    borderColor: '#ddd',
    alignItems: 'center',
    justifyContent: 'center',
  },
  modeCheckActive: {
    backgroundColor: '#1976D2',
    borderColor: '#1976D2',
  },
  inputGroup: {
    marginBottom: 12,
  },
  label: {
    fontSize: 14,
    color: '#333',
    marginBottom: 8,
  },
  input: {
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 12,
    fontSize: 14,
  },
  saveBtn: {
    backgroundColor: '#1976D2',
    borderRadius: 8,
    paddingVertical: 14,
    alignItems: 'center',
  },
  saveBtnDisabled: {
    opacity: 0.6,
  },
  saveBtnText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  notice: {
    margin: 12,
    padding: 16,
    backgroundColor: '#FFF8E1',
    borderRadius: 12,
  },
  noticeTitle: {
    fontSize: 14,
    fontWeight: '500',
    color: '#F57C00',
    marginBottom: 8,
  },
  noticeText: {
    fontSize: 13,
    color: '#666',
    lineHeight: 22,
  },
});
