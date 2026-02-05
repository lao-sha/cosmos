import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
  Switch,
} from 'react-native';
import { useRouter } from 'expo-router';
import {
  TrendingUp,
  Wallet,
  Settings,
  Plus,
  Minus,
  Power,
  FileText,
} from 'lucide-react-native';
import * as Haptics from 'expo-haptics';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import {
  getMakerProfile,
  toggleStatus,
  type MakerProfile,
} from '@/services/maker';
import { Button, Card, Input } from '@/components/ui';
import { Colors, Shadows } from '@/constants/colors';

export default function MakerScreen() {
  const colors = useColors();
  const router = useRouter();
  const { address, mnemonic, isConnected } = useWalletStore();

  const [profile, setProfile] = useState<MakerProfile | null>(null);
  const [loading, setLoading] = useState(true);
  const [toggling, setToggling] = useState(false);

  useEffect(() => {
    loadProfile();
  }, [address]);

  const loadProfile = async () => {
    if (!address) return;
    setLoading(true);
    try {
      const data = await getMakerProfile(address);
      setProfile(data);
    } catch (error) {
      console.error('Failed to load maker profile:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleToggleStatus = async () => {
    if (!profile || !mnemonic) return;

    const newActive = profile.status !== 'active';
    setToggling(true);
    try {
      await toggleStatus(newActive, mnemonic);
      Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
      setProfile({ ...profile, status: newActive ? 'active' : 'paused' });
    } catch (error: any) {
      Alert.alert('操作失败', error.message);
    } finally {
      setToggling(false);
    }
  };

  if (loading) {
    return (
      <View style={[styles.container, { backgroundColor: colors.background }]}>
        <Text style={{ color: colors.textSecondary, textAlign: 'center', marginTop: 100 }}>
          加载中...
        </Text>
      </View>
    );
  }

  if (!profile) {
    return (
      <ScrollView
        style={[styles.container, { backgroundColor: colors.background }]}
        contentContainerStyle={styles.emptyContent}
      >
        <View style={[styles.emptyIcon, { backgroundColor: Colors.primary + '15' }]}>
          <TrendingUp size={48} color={Colors.primary} />
        </View>
        <Text style={[styles.emptyTitle, { color: colors.textPrimary }]}>
          成为做市商
        </Text>
        <Text style={[styles.emptySubtitle, { color: colors.textSecondary }]}>
          为平台提供流动性，赚取交易差价
        </Text>

        <Card style={styles.benefitsCard}>
          <Text style={[styles.benefitsTitle, { color: colors.textPrimary }]}>
            做市商权益
          </Text>
          <View style={styles.benefitItem}>
            <Text style={[styles.benefitDot, { color: Colors.success }]}>•</Text>
            <Text style={[styles.benefitText, { color: colors.textSecondary }]}>
              自定义买卖价格，赚取差价
            </Text>
          </View>
          <View style={styles.benefitItem}>
            <Text style={[styles.benefitDot, { color: Colors.success }]}>•</Text>
            <Text style={[styles.benefitText, { color: colors.textSecondary }]}>
              OTC 交易优先匹配
            </Text>
          </View>
          <View style={styles.benefitItem}>
            <Text style={[styles.benefitDot, { color: Colors.success }]}>•</Text>
            <Text style={[styles.benefitText, { color: colors.textSecondary }]}>
              专属做市商标识
            </Text>
          </View>
          <View style={styles.benefitItem}>
            <Text style={[styles.benefitDot, { color: Colors.success }]}>•</Text>
            <Text style={[styles.benefitText, { color: colors.textSecondary }]}>
              数据统计与分析
            </Text>
          </View>
        </Card>

        <Card style={styles.requireCard}>
          <Text style={[styles.requireTitle, { color: colors.textPrimary }]}>
            申请条件
          </Text>
          <View style={styles.requireItem}>
            <Text style={[styles.requireLabel, { color: colors.textSecondary }]}>
              KYC 等级
            </Text>
            <Text style={[styles.requireValue, { color: colors.textPrimary }]}>
              高级认证及以上
            </Text>
          </View>
          <View style={styles.requireItem}>
            <Text style={[styles.requireLabel, { color: colors.textSecondary }]}>
              保证金
            </Text>
            <Text style={[styles.requireValue, { color: colors.textPrimary }]}>
              ≥ 10,000 COS
            </Text>
          </View>
        </Card>

        <Button
          title="申请成为做市商"
          onPress={() => router.push('/maker/apply')}
          style={styles.applyButton}
        />
      </ScrollView>
    );
  }

  const isActive = profile.status === 'active';

  return (
    <ScrollView
      style={[styles.container, { backgroundColor: colors.background }]}
      contentContainerStyle={styles.content}
    >
      {/* Status Header */}
      <Card style={[styles.statusCard, { backgroundColor: isActive ? Colors.success + '15' : colors.surface }]}>
        <View style={styles.statusRow}>
          <View>
            <Text style={[styles.statusLabel, { color: colors.textSecondary }]}>
              当前状态
            </Text>
            <Text style={[styles.statusValue, { color: isActive ? Colors.success : Colors.warning }]}>
              {isActive ? '营业中' : '已暂停'}
            </Text>
          </View>
          <Switch
            value={isActive}
            onValueChange={handleToggleStatus}
            disabled={toggling}
            trackColor={{ false: colors.border, true: Colors.success }}
            thumbColor="#FFFFFF"
          />
        </View>
      </Card>

      {/* Stats */}
      <View style={styles.statsGrid}>
        <Card style={styles.statCard}>
          <Text style={[styles.statValue, { color: colors.textPrimary }]}>
            {profile.completedOrders}
          </Text>
          <Text style={[styles.statLabel, { color: colors.textSecondary }]}>
            完成订单
          </Text>
        </Card>
        <Card style={styles.statCard}>
          <Text style={[styles.statValue, { color: Colors.success }]}>
            {profile.completionRate}%
          </Text>
          <Text style={[styles.statLabel, { color: colors.textSecondary }]}>
            完成率
          </Text>
        </Card>
      </View>

      {/* Balances */}
      <Card style={styles.balanceCard}>
        <Text style={[styles.cardTitle, { color: colors.textPrimary }]}>
          资金池
        </Text>
        <View style={styles.balanceRow}>
          <View style={styles.balanceItem}>
            <Text style={[styles.balanceLabel, { color: colors.textSecondary }]}>
              COS 余额
            </Text>
            <Text style={[styles.balanceValue, { color: colors.textPrimary }]}>
              {formatAmount(profile.availableCos, 12)}
            </Text>
          </View>
          <View style={styles.balanceItem}>
            <Text style={[styles.balanceLabel, { color: colors.textSecondary }]}>
              USDT 余额
            </Text>
            <Text style={[styles.balanceValue, { color: colors.textPrimary }]}>
              {formatAmount(profile.availableUsdt, 6)}
            </Text>
          </View>
        </View>
        <View style={styles.balanceActions}>
          <TouchableOpacity
            style={[styles.balanceBtn, { backgroundColor: Colors.success + '15' }]}
            onPress={() => router.push('/maker/deposit')}
          >
            <Plus size={18} color={Colors.success} />
            <Text style={[styles.balanceBtnText, { color: Colors.success }]}>
              充值
            </Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={[styles.balanceBtn, { backgroundColor: Colors.warning + '15' }]}
            onPress={() => router.push('/maker/withdraw')}
          >
            <Minus size={18} color={Colors.warning} />
            <Text style={[styles.balanceBtnText, { color: Colors.warning }]}>
              提现
            </Text>
          </TouchableOpacity>
        </View>
      </Card>

      {/* Price Settings */}
      <Card style={styles.priceCard}>
        <View style={styles.priceHeader}>
          <Text style={[styles.cardTitle, { color: colors.textPrimary }]}>
            价格设置
          </Text>
          <TouchableOpacity onPress={() => router.push('/maker/settings')}>
            <Settings size={20} color={colors.textSecondary} />
          </TouchableOpacity>
        </View>
        <View style={styles.priceRow}>
          <Text style={[styles.priceLabel, { color: colors.textSecondary }]}>
            当前报价
          </Text>
          <Text style={[styles.priceValue, { color: Colors.primary }]}>
            {(Number(profile.price) / 10000).toFixed(4)} USDT/COS
          </Text>
        </View>
        <View style={styles.priceRow}>
          <Text style={[styles.priceLabel, { color: colors.textSecondary }]}>
            限额范围
          </Text>
          <Text style={[styles.priceValue, { color: colors.textPrimary }]}>
            {formatAmount(profile.minAmount, 12)} - {formatAmount(profile.maxAmount, 12)} COS
          </Text>
        </View>
      </Card>

      {/* Quick Actions */}
      <View style={styles.actions}>
        <TouchableOpacity
          style={[styles.actionItem, { backgroundColor: colors.surface }]}
          onPress={() => router.push('/maker/orders')}
        >
          <FileText size={24} color={Colors.primary} />
          <Text style={[styles.actionText, { color: colors.textPrimary }]}>
            订单管理
          </Text>
        </TouchableOpacity>
        <TouchableOpacity
          style={[styles.actionItem, { backgroundColor: colors.surface }]}
          onPress={() => router.push('/maker/stats')}
        >
          <TrendingUp size={24} color={Colors.primary} />
          <Text style={[styles.actionText, { color: colors.textPrimary }]}>
            数据统计
          </Text>
        </TouchableOpacity>
      </View>
    </ScrollView>
  );
}

function formatAmount(value: string, decimals: number): string {
  const bigValue = BigInt(value || '0');
  const divisor = BigInt(10 ** decimals);
  const whole = bigValue / divisor;
  const fraction = bigValue % divisor;
  const fractionStr = fraction.toString().padStart(decimals, '0').slice(0, 2);
  return `${whole}.${fractionStr}`;
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    padding: 16,
  },
  emptyContent: {
    padding: 16,
    paddingTop: 40,
    alignItems: 'center',
  },
  emptyIcon: {
    width: 100,
    height: 100,
    borderRadius: 50,
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 24,
  },
  emptyTitle: {
    fontSize: 24,
    fontWeight: '700',
    marginBottom: 8,
  },
  emptySubtitle: {
    fontSize: 16,
    textAlign: 'center',
    marginBottom: 24,
  },
  benefitsCard: {
    width: '100%',
    marginBottom: 16,
  },
  benefitsTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 12,
  },
  benefitItem: {
    flexDirection: 'row',
    marginBottom: 8,
  },
  benefitDot: {
    fontSize: 16,
    marginRight: 8,
  },
  benefitText: {
    fontSize: 14,
    flex: 1,
  },
  requireCard: {
    width: '100%',
    marginBottom: 24,
  },
  requireTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 12,
  },
  requireItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  requireLabel: {
    fontSize: 14,
  },
  requireValue: {
    fontSize: 14,
    fontWeight: '500',
  },
  applyButton: {
    width: '100%',
  },
  statusCard: {
    marginBottom: 16,
  },
  statusRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  statusLabel: {
    fontSize: 14,
    marginBottom: 4,
  },
  statusValue: {
    fontSize: 20,
    fontWeight: '700',
  },
  statsGrid: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 16,
  },
  statCard: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 24,
    fontWeight: '700',
    marginBottom: 4,
  },
  statLabel: {
    fontSize: 13,
  },
  balanceCard: {
    marginBottom: 16,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 12,
  },
  balanceRow: {
    flexDirection: 'row',
    marginBottom: 16,
  },
  balanceItem: {
    flex: 1,
  },
  balanceLabel: {
    fontSize: 13,
    marginBottom: 4,
  },
  balanceValue: {
    fontSize: 20,
    fontWeight: '700',
  },
  balanceActions: {
    flexDirection: 'row',
    gap: 12,
  },
  balanceBtn: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 12,
    borderRadius: 12,
    gap: 6,
  },
  balanceBtnText: {
    fontSize: 15,
    fontWeight: '600',
  },
  priceCard: {
    marginBottom: 16,
  },
  priceHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  priceRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  priceLabel: {
    fontSize: 14,
  },
  priceValue: {
    fontSize: 14,
    fontWeight: '600',
  },
  actions: {
    flexDirection: 'row',
    gap: 12,
  },
  actionItem: {
    flex: 1,
    alignItems: 'center',
    paddingVertical: 20,
    borderRadius: 16,
  },
  actionText: {
    fontSize: 14,
    fontWeight: '500',
    marginTop: 8,
  },
});
