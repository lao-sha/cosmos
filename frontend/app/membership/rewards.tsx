import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    FlatList,
    Pressable,
    StyleSheet,
    Text,
    View,
} from 'react-native';

type RewardTxType = 'checkin' | 'referral' | 'task' | 'bonus' | 'withdraw';

interface RewardTransaction {
  id: string;
  type: RewardTxType;
  amount: number;
  description: string;
  createdAt: string;
}

const MOCK_REWARDS: RewardTransaction[] = [
  { id: '1', type: 'checkin', amount: 20, description: '每日签到奖励 (2x)', createdAt: '2025-01-28 08:30' },
  { id: '2', type: 'task', amount: 50, description: '完成八字排盘任务', createdAt: '2025-01-27 15:20' },
  { id: '3', type: 'referral', amount: 100, description: '邀请好友注册奖励', createdAt: '2025-01-26 10:15' },
  { id: '4', type: 'checkin', amount: 20, description: '每日签到奖励 (2x)', createdAt: '2025-01-26 09:00' },
  { id: '5', type: 'bonus', amount: 70, description: '连续签到7天额外奖励', createdAt: '2025-01-25 08:45' },
  { id: '6', type: 'withdraw', amount: -500, description: '提现到钱包', createdAt: '2025-01-24 14:30' },
  { id: '7', type: 'checkin', amount: 20, description: '每日签到奖励 (2x)', createdAt: '2025-01-24 08:20' },
  { id: '8', type: 'task', amount: 30, description: '完成合婚测试任务', createdAt: '2025-01-23 16:40' },
];

const TYPE_INFO: Record<RewardTxType, { icon: string; color: string; label: string }> = {
  checkin: { icon: '📅', color: '#22c55e', label: '签到' },
  referral: { icon: '👥', color: '#3b82f6', label: '邀请' },
  task: { icon: '✅', color: '#8b5cf6', label: '任务' },
  bonus: { icon: '🎁', color: '#f59e0b', label: '奖励' },
  withdraw: { icon: '💸', color: '#ef4444', label: '提现' },
};

export default function RewardsScreen() {
  const router = useRouter();
  const [filter, setFilter] = useState<RewardTxType | 'all'>('all');

  const filteredRewards = filter === 'all'
    ? MOCK_REWARDS
    : MOCK_REWARDS.filter(r => r.type === filter);

  const totalEarned = MOCK_REWARDS
    .filter(r => r.amount > 0)
    .reduce((sum, r) => sum + r.amount, 0);

  const totalWithdrawn = Math.abs(
    MOCK_REWARDS
      .filter(r => r.amount < 0)
      .reduce((sum, r) => sum + r.amount, 0)
  );

  const renderItem = ({ item }: { item: RewardTransaction }) => {
    const info = TYPE_INFO[item.type];
    const isPositive = item.amount > 0;

    return (
      <View style={styles.txItem}>
        <View style={[styles.txIcon, { backgroundColor: `${info.color}20` }]}>
          <Text style={styles.txIconText}>{info.icon}</Text>
        </View>
        <View style={styles.txContent}>
          <Text style={styles.txDescription}>{item.description}</Text>
          <Text style={styles.txTime}>{item.createdAt}</Text>
        </View>
        <Text style={[styles.txAmount, { color: isPositive ? '#22c55e' : '#ef4444' }]}>
          {isPositive ? '+' : ''}{item.amount} COS
        </Text>
      </View>
    );
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>奖励记录</Text>
        <View style={styles.headerRight} />
      </View>

      <View style={styles.statsCard}>
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{totalEarned}</Text>
          <Text style={styles.statLabel}>累计获得</Text>
        </View>
        <View style={styles.statDivider} />
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{totalWithdrawn}</Text>
          <Text style={styles.statLabel}>累计提现</Text>
        </View>
        <View style={styles.statDivider} />
        <View style={styles.statItem}>
          <Text style={[styles.statValue, { color: '#6D28D9' }]}>
            {totalEarned - totalWithdrawn}
          </Text>
          <Text style={styles.statLabel}>当前余额</Text>
        </View>
      </View>

      <View style={styles.filterRow}>
        {(['all', 'checkin', 'task', 'referral', 'bonus', 'withdraw'] as const).map((type) => (
          <Pressable
            key={type}
            style={[styles.filterButton, filter === type && styles.filterActive]}
            onPress={() => setFilter(type)}
          >
            <Text style={[styles.filterText, filter === type && styles.filterTextActive]}>
              {type === 'all' ? '全部' : TYPE_INFO[type].label}
            </Text>
          </Pressable>
        ))}
      </View>

      <FlatList
        data={filteredRewards}
        renderItem={renderItem}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.list}
        showsVerticalScrollIndicator={false}
        ListEmptyComponent={
          <View style={styles.empty}>
            <Text style={styles.emptyText}>暂无记录</Text>
          </View>
        }
      />

      <View style={styles.footer}>
        <Pressable style={styles.withdrawButton}>
          <Text style={styles.withdrawButtonText}>提现到钱包</Text>
        </Pressable>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#fff',
    paddingTop: 50,
    paddingBottom: 12,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  backButton: {
    padding: 4,
  },
  backText: {
    fontSize: 17,
    color: '#6D28D9',
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#1f2937',
  },
  headerRight: {
    width: 50,
  },
  statsCard: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    margin: 16,
    borderRadius: 16,
    padding: 20,
  },
  statItem: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 24,
    fontWeight: '700',
    color: '#1f2937',
  },
  statLabel: {
    fontSize: 13,
    color: '#6b7280',
    marginTop: 4,
  },
  statDivider: {
    width: 1,
    backgroundColor: '#e5e7eb',
    marginVertical: 4,
  },
  filterRow: {
    flexDirection: 'row',
    paddingHorizontal: 16,
    marginBottom: 12,
    gap: 8,
  },
  filterButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
    backgroundColor: '#fff',
  },
  filterActive: {
    backgroundColor: '#6D28D9',
  },
  filterText: {
    fontSize: 13,
    color: '#6b7280',
  },
  filterTextActive: {
    color: '#fff',
    fontWeight: '600',
  },
  list: {
    padding: 16,
    paddingTop: 0,
  },
  txItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#fff',
    padding: 16,
    borderRadius: 12,
    marginBottom: 8,
  },
  txIcon: {
    width: 40,
    height: 40,
    borderRadius: 20,
    justifyContent: 'center',
    alignItems: 'center',
  },
  txIconText: {
    fontSize: 18,
  },
  txContent: {
    flex: 1,
    marginLeft: 12,
  },
  txDescription: {
    fontSize: 15,
    color: '#1f2937',
    fontWeight: '500',
  },
  txTime: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 2,
  },
  txAmount: {
    fontSize: 16,
    fontWeight: '600',
  },
  empty: {
    padding: 40,
    alignItems: 'center',
  },
  emptyText: {
    fontSize: 15,
    color: '#9ca3af',
  },
  footer: {
    padding: 16,
    backgroundColor: '#fff',
    borderTopWidth: 1,
    borderTopColor: '#e5e7eb',
  },
  withdrawButton: {
    backgroundColor: '#6D28D9',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  withdrawButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
