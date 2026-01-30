import { useAuthStore } from '@/src/stores/auth';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    FlatList,
    Pressable,
    StyleSheet,
    Text,
    View,
} from 'react-native';

type TxType = 'transfer_in' | 'transfer_out' | 'stake' | 'unstake' | 'reward' | 'fee';
type TxStatus = 'pending' | 'confirmed' | 'failed';

interface Transaction {
  id: string;
  hash: string;
  type: TxType;
  amount: string;
  counterparty?: string;
  status: TxStatus;
  timestamp: string;
  blockNumber?: number;
}

const MOCK_TRANSACTIONS: Transaction[] = [
  {
    id: '1',
    hash: '0xabc123...def456',
    type: 'transfer_out',
    amount: '10.5',
    counterparty: '5Grw...utQY',
    status: 'confirmed',
    timestamp: '2025-01-28 14:30',
    blockNumber: 123456,
  },
  {
    id: '2',
    hash: '0xdef456...abc789',
    type: 'transfer_in',
    amount: '25.0',
    counterparty: '5DAn...kQrB',
    status: 'confirmed',
    timestamp: '2025-01-27 10:15',
    blockNumber: 123400,
  },
  {
    id: '3',
    hash: '0x789abc...123def',
    type: 'reward',
    amount: '0.5',
    status: 'confirmed',
    timestamp: '2025-01-26 08:00',
    blockNumber: 123300,
  },
  {
    id: '4',
    hash: '0x456def...789abc',
    type: 'transfer_out',
    amount: '5.0',
    counterparty: '5Ck8...mNpC',
    status: 'pending',
    timestamp: '2025-01-28 15:00',
  },
  {
    id: '5',
    hash: '0x123abc...456def',
    type: 'stake',
    amount: '100.0',
    status: 'confirmed',
    timestamp: '2025-01-25 16:45',
    blockNumber: 123200,
  },
];

const TYPE_INFO: Record<TxType, { label: string; icon: string; color: string }> = {
  transfer_in: { label: '收款', icon: '↙️', color: '#22c55e' },
  transfer_out: { label: '转账', icon: '↗️', color: '#ef4444' },
  stake: { label: '质押', icon: '🔒', color: '#3b82f6' },
  unstake: { label: '解押', icon: '🔓', color: '#f59e0b' },
  reward: { label: '奖励', icon: '🎁', color: '#8b5cf6' },
  fee: { label: '手续费', icon: '💰', color: '#6b7280' },
};

const STATUS_INFO: Record<TxStatus, { label: string; color: string }> = {
  pending: { label: '处理中', color: '#f59e0b' },
  confirmed: { label: '已确认', color: '#22c55e' },
  failed: { label: '失败', color: '#ef4444' },
};

export default function HistoryScreen() {
  const router = useRouter();
  const { isLoggedIn, address } = useAuthStore();

  const [filter, setFilter] = useState<'all' | 'in' | 'out'>('all');

  const filteredTx = MOCK_TRANSACTIONS.filter((tx) => {
    if (filter === 'all') return true;
    if (filter === 'in') return ['transfer_in', 'reward'].includes(tx.type);
    return ['transfer_out', 'stake', 'fee'].includes(tx.type);
  });

  const handleViewDetail = (tx: Transaction) => {
    // TODO: 打开交易详情
    console.log('View tx:', tx.hash);
  };

  const renderItem = ({ item }: { item: Transaction }) => {
    const typeInfo = TYPE_INFO[item.type];
    const statusInfo = STATUS_INFO[item.status];
    const isIncoming = ['transfer_in', 'reward', 'unstake'].includes(item.type);

    return (
      <Pressable style={styles.txItem} onPress={() => handleViewDetail(item)}>
        <View style={[styles.txIcon, { backgroundColor: `${typeInfo.color}15` }]}>
          <Text style={styles.txIconText}>{typeInfo.icon}</Text>
        </View>

        <View style={styles.txContent}>
          <View style={styles.txHeader}>
            <Text style={styles.txType}>{typeInfo.label}</Text>
            <Text
              style={[
                styles.txAmount,
                { color: isIncoming ? '#22c55e' : '#1f2937' },
              ]}
            >
              {isIncoming ? '+' : '-'}{item.amount} COS
            </Text>
          </View>

          {item.counterparty && (
            <Text style={styles.txCounterparty}>
              {isIncoming ? '来自' : '发送至'} {item.counterparty}
            </Text>
          )}

          <View style={styles.txFooter}>
            <Text style={styles.txTime}>{item.timestamp}</Text>
            <View style={[styles.statusBadge, { backgroundColor: `${statusInfo.color}15` }]}>
              <Text style={[styles.statusText, { color: statusInfo.color }]}>
                {statusInfo.label}
              </Text>
            </View>
          </View>
        </View>
      </Pressable>
    );
  };

  if (!isLoggedIn) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>交易记录</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.centerContent}>
          <Text style={styles.emptyText}>请先登录</Text>
        </View>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>交易记录</Text>
        <View style={styles.headerRight} />
      </View>

      <View style={styles.filterRow}>
        {(['all', 'in', 'out'] as const).map((f) => (
          <Pressable
            key={f}
            style={[styles.filterButton, filter === f && styles.filterActive]}
            onPress={() => setFilter(f)}
          >
            <Text style={[styles.filterText, filter === f && styles.filterTextActive]}>
              {f === 'all' ? '全部' : f === 'in' ? '收入' : '支出'}
            </Text>
          </Pressable>
        ))}
      </View>

      <FlatList
        data={filteredTx}
        renderItem={renderItem}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.list}
        showsVerticalScrollIndicator={false}
        ListEmptyComponent={
          <View style={styles.emptyContainer}>
            <Text style={styles.emptyIcon}>📜</Text>
            <Text style={styles.emptyTitle}>暂无交易记录</Text>
            <Text style={styles.emptyDesc}>
              你的交易记录将显示在这里
            </Text>
          </View>
        }
        ListFooterComponent={
          filteredTx.length > 0 ? (
            <View style={styles.footer}>
              <Text style={styles.footerText}>
                仅显示最近的交易记录
              </Text>
            </View>
          ) : null
        }
      />
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
  filterRow: {
    flexDirection: 'row',
    padding: 16,
    gap: 8,
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  filterButton: {
    paddingHorizontal: 20,
    paddingVertical: 8,
    borderRadius: 20,
    backgroundColor: '#f3f4f6',
  },
  filterActive: {
    backgroundColor: '#6D28D9',
  },
  filterText: {
    fontSize: 14,
    color: '#6b7280',
  },
  filterTextActive: {
    color: '#fff',
    fontWeight: '600',
  },
  list: {
    padding: 16,
  },
  txItem: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 8,
  },
  txIcon: {
    width: 44,
    height: 44,
    borderRadius: 22,
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  txIconText: {
    fontSize: 20,
  },
  txContent: {
    flex: 1,
  },
  txHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  txType: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
  },
  txAmount: {
    fontSize: 16,
    fontWeight: '600',
  },
  txCounterparty: {
    fontSize: 13,
    color: '#6b7280',
    marginTop: 4,
  },
  txFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginTop: 8,
  },
  txTime: {
    fontSize: 12,
    color: '#9ca3af',
  },
  statusBadge: {
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 10,
  },
  statusText: {
    fontSize: 11,
    fontWeight: '500',
  },
  centerContent: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyContainer: {
    padding: 60,
    alignItems: 'center',
  },
  emptyIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  emptyTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 8,
  },
  emptyDesc: {
    fontSize: 14,
    color: '#6b7280',
  },
  emptyText: {
    fontSize: 16,
    color: '#6b7280',
  },
  footer: {
    padding: 20,
    alignItems: 'center',
  },
  footerText: {
    fontSize: 13,
    color: '#9ca3af',
  },
});
