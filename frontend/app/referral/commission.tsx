import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  FlatList,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Wallet, TrendingUp, ArrowDownCircle, Filter } from 'lucide-react-native';
import * as Haptics from 'expo-haptics';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import {
  getReferralInfo,
  getCommissionRecords,
  withdrawCommission,
  REFERRAL_LEVELS,
  type ReferralInfo,
  type CommissionRecord,
} from '@/services/referral';
import { Button, Card, Input } from '@/components/ui';
import { Colors } from '@/constants/colors';

type FilterType = 'all' | 'trade' | 'referral' | 'bonus';

export default function CommissionScreen() {
  const colors = useColors();
  const router = useRouter();
  const { address, mnemonic } = useWalletStore();

  const [info, setInfo] = useState<ReferralInfo | null>(null);
  const [records, setRecords] = useState<CommissionRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<FilterType>('all');
  const [withdrawAmount, setWithdrawAmount] = useState('');
  const [withdrawing, setWithdrawing] = useState(false);

  useEffect(() => {
    loadData();
  }, [address]);

  const loadData = async () => {
    if (!address) return;
    setLoading(true);
    try {
      const [infoData, recordsData] = await Promise.all([
        getReferralInfo(address),
        getCommissionRecords(address),
      ]);
      setInfo(infoData);
      setRecords(recordsData);
    } catch (error) {
      console.error('Failed to load commission data:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleWithdraw = async () => {
    if (!mnemonic || !withdrawAmount) return;

    const amount = parseFloat(withdrawAmount);
    const pending = BigInt(info?.pendingEarnings || '0') / BigInt(1e12);
    
    if (amount <= 0 || amount > Number(pending)) {
      Alert.alert('提示', '请输入有效的提取金额');
      return;
    }

    setWithdrawing(true);
    try {
      const amountWei = (BigInt(Math.floor(amount * 1e12))).toString();
      await withdrawCommission(amountWei, mnemonic);
      Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
      Alert.alert('提取成功', `${amount} COS 已到账`);
      setWithdrawAmount('');
      loadData();
    } catch (error: any) {
      Alert.alert('提取失败', error.message);
    } finally {
      setWithdrawing(false);
    }
  };

  const formatAmount = (value: string): string => {
    const num = BigInt(value || '0') / BigInt(1e12);
    return num.toString();
  };

  const filteredRecords = records.filter(
    (r) => filter === 'all' || r.type === filter
  );

  const getTypeLabel = (type: string): string => {
    switch (type) {
      case 'trade': return '交易返佣';
      case 'referral': return '推荐奖励';
      case 'bonus': return '活动奖励';
      default: return type;
    }
  };

  const getTypeColor = (type: string): string => {
    switch (type) {
      case 'trade': return Colors.success;
      case 'referral': return Colors.primary;
      case 'bonus': return Colors.accent;
      default: return colors.textSecondary;
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

  return (
    <ScrollView
      style={[styles.container, { backgroundColor: colors.background }]}
      contentContainerStyle={styles.content}
    >
      {/* Stats */}
      <View style={styles.statsRow}>
        <Card style={styles.statCard}>
          <Wallet size={24} color={Colors.primary} />
          <Text style={[styles.statValue, { color: colors.textPrimary }]}>
            {formatAmount(info?.pendingEarnings || '0')}
          </Text>
          <Text style={[styles.statLabel, { color: colors.textSecondary }]}>
            待提取 (COS)
          </Text>
        </Card>
        <Card style={styles.statCard}>
          <TrendingUp size={24} color={Colors.success} />
          <Text style={[styles.statValue, { color: colors.textPrimary }]}>
            {formatAmount(info?.totalEarnings || '0')}
          </Text>
          <Text style={[styles.statLabel, { color: colors.textSecondary }]}>
            累计收益 (COS)
          </Text>
        </Card>
      </View>

      {/* Withdraw */}
      <Card style={styles.withdrawCard}>
        <Text style={[styles.cardTitle, { color: colors.textPrimary }]}>
          提取收益
        </Text>
        <View style={styles.withdrawInputRow}>
          <Input
            placeholder="输入提取金额"
            value={withdrawAmount}
            onChangeText={setWithdrawAmount}
            keyboardType="decimal-pad"
            containerStyle={styles.withdrawInput}
          />
          <TouchableOpacity
            style={[styles.maxButton, { borderColor: Colors.primary }]}
            onPress={() => setWithdrawAmount(formatAmount(info?.pendingEarnings || '0'))}
          >
            <Text style={[styles.maxText, { color: Colors.primary }]}>全部</Text>
          </TouchableOpacity>
        </View>
        <Button
          title="提取到钱包"
          onPress={handleWithdraw}
          loading={withdrawing}
          disabled={!withdrawAmount || parseFloat(withdrawAmount) <= 0}
        />
      </Card>

      {/* Commission Rates */}
      <Card style={styles.ratesCard}>
        <Text style={[styles.cardTitle, { color: colors.textPrimary }]}>
          返佣比例
        </Text>
        <View style={styles.ratesGrid}>
          {REFERRAL_LEVELS.slice(0, 10).map((level) => (
            <View key={level.level} style={styles.rateItem}>
              <Text style={[styles.rateLevel, { color: colors.textSecondary }]}>
                L{level.level}
              </Text>
              <Text style={[styles.rateValue, { color: Colors.primary }]}>
                {(level.rate * 100).toFixed(0)}%
              </Text>
            </View>
          ))}
        </View>
      </Card>

      {/* Records */}
      <Card style={styles.recordsCard}>
        <View style={styles.recordsHeader}>
          <Text style={[styles.cardTitle, { color: colors.textPrimary }]}>
            收益记录
          </Text>
          <View style={styles.filterTabs}>
            {(['all', 'trade', 'referral', 'bonus'] as FilterType[]).map((f) => (
              <TouchableOpacity
                key={f}
                style={[
                  styles.filterTab,
                  filter === f && { backgroundColor: Colors.primary + '15' },
                ]}
                onPress={() => setFilter(f)}
              >
                <Text
                  style={[
                    styles.filterText,
                    { color: filter === f ? Colors.primary : colors.textSecondary },
                  ]}
                >
                  {f === 'all' ? '全部' : getTypeLabel(f)}
                </Text>
              </TouchableOpacity>
            ))}
          </View>
        </View>

        {filteredRecords.length === 0 ? (
          <Text style={[styles.emptyText, { color: colors.textTertiary }]}>
            暂无记录
          </Text>
        ) : (
          filteredRecords.map((record) => (
            <View
              key={record.id}
              style={[styles.recordItem, { borderBottomColor: colors.border }]}
            >
              <View style={styles.recordLeft}>
                <View
                  style={[
                    styles.recordIcon,
                    { backgroundColor: getTypeColor(record.type) + '15' },
                  ]}
                >
                  <ArrowDownCircle size={18} color={getTypeColor(record.type)} />
                </View>
                <View>
                  <Text style={[styles.recordType, { color: colors.textPrimary }]}>
                    {getTypeLabel(record.type)}
                  </Text>
                  <Text style={[styles.recordFrom, { color: colors.textTertiary }]}>
                    来自 {record.fromName} (L{record.level})
                  </Text>
                </View>
              </View>
              <View style={styles.recordRight}>
                <Text style={[styles.recordAmount, { color: Colors.success }]}>
                  +{formatAmount(record.amount)} COS
                </Text>
                <Text style={[styles.recordTime, { color: colors.textTertiary }]}>
                  {new Date(record.createdAt).toLocaleDateString()}
                </Text>
              </View>
            </View>
          ))
        )}
      </Card>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    padding: 16,
    paddingBottom: 40,
  },
  statsRow: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 16,
  },
  statCard: {
    flex: 1,
    alignItems: 'center',
    paddingVertical: 20,
  },
  statValue: {
    fontSize: 24,
    fontWeight: '700',
    marginTop: 8,
  },
  statLabel: {
    fontSize: 12,
    marginTop: 4,
  },
  withdrawCard: {
    marginBottom: 16,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 12,
  },
  withdrawInputRow: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    marginBottom: 12,
  },
  withdrawInput: {
    flex: 1,
    marginBottom: 0,
    marginRight: 8,
  },
  maxButton: {
    paddingHorizontal: 16,
    paddingVertical: 14,
    borderRadius: 12,
    borderWidth: 1,
  },
  maxText: {
    fontSize: 14,
    fontWeight: '500',
  },
  ratesCard: {
    marginBottom: 16,
  },
  ratesGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
  },
  rateItem: {
    width: '20%',
    alignItems: 'center',
    paddingVertical: 8,
  },
  rateLevel: {
    fontSize: 12,
    marginBottom: 2,
  },
  rateValue: {
    fontSize: 14,
    fontWeight: '600',
  },
  recordsCard: {
    marginBottom: 16,
  },
  recordsHeader: {
    marginBottom: 12,
  },
  filterTabs: {
    flexDirection: 'row',
    marginTop: 12,
    gap: 8,
  },
  filterTab: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
  },
  filterText: {
    fontSize: 13,
    fontWeight: '500',
  },
  emptyText: {
    textAlign: 'center',
    fontSize: 14,
    paddingVertical: 20,
  },
  recordItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 12,
    borderBottomWidth: 1,
  },
  recordLeft: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  recordIcon: {
    width: 36,
    height: 36,
    borderRadius: 18,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  recordType: {
    fontSize: 14,
    fontWeight: '500',
  },
  recordFrom: {
    fontSize: 12,
    marginTop: 2,
  },
  recordRight: {
    alignItems: 'flex-end',
  },
  recordAmount: {
    fontSize: 15,
    fontWeight: '600',
  },
  recordTime: {
    fontSize: 11,
    marginTop: 2,
  },
});
