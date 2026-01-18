/**
 * 兑换历史记录页面
 */

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  ActivityIndicator,
  Alert,
  RefreshControl,
} from 'react-native';
import { useRouter } from 'expo-router';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { SwapRecordCard } from '@/features/bridge/components';
import { MakerSwapRecord, SwapStatus } from '@/features/bridge/types';

// 模拟兑换记录数据
const mockRecords: MakerSwapRecord[] = [
  {
    swapId: 1001,
    makerId: 1,
    maker: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    user: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
    dustAmount: BigInt(500_000_000_000_000),
    usdtAmount: 50_000_000,
    usdtAddress: 'TJYeasTPa6gpEEfYcPQgLHu9eGNj1FGrVK',
    createdAt: 12345678,
    timeoutAt: 12345978,
    trc20TxHash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
    completedAt: 12345700,
    status: SwapStatus.Completed,
    priceUsdt: 100_000,
  },
  {
    swapId: 1002,
    makerId: 2,
    maker: '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y',
    user: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
    dustAmount: BigInt(1000_000_000_000_000),
    usdtAmount: 100_000_000,
    usdtAddress: 'TN3W4H6rK2ce4vX9YnFQHwKENnHjoxb3m9',
    createdAt: 12345800,
    timeoutAt: 12346100,
    status: SwapStatus.Pending,
    priceUsdt: 100_000,
  },
  {
    swapId: 1003,
    makerId: 1,
    maker: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    user: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
    dustAmount: BigInt(200_000_000_000_000),
    usdtAmount: 20_000_000,
    usdtAddress: 'TJYeasTPa6gpEEfYcPQgLHu9eGNj1FGrVK',
    createdAt: 12344000,
    timeoutAt: 12344300,
    status: SwapStatus.Refunded,
    priceUsdt: 100_000,
  },
];

type FilterType = 'all' | 'pending' | 'completed' | 'refunded';

export default function BridgeHistoryPage() {
  const router = useRouter();
  const [records, setRecords] = useState<MakerSwapRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [filter, setFilter] = useState<FilterType>('all');

  const fetchRecords = async () => {
    // TODO: 从链上获取用户兑换记录
    await new Promise(resolve => setTimeout(resolve, 500));
    setRecords(mockRecords);
    setLoading(false);
    setRefreshing(false);
  };

  useEffect(() => {
    fetchRecords();
  }, []);

  const handleRefresh = () => {
    setRefreshing(true);
    fetchRecords();
  };

  const handleReport = (swapId: number) => {
    Alert.alert(
      '举报兑换',
      '确定要举报此兑换吗？举报后将进入仲裁流程。',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确定举报',
          style: 'destructive',
          onPress: async () => {
            // TODO: 调用链上 bridge.report_swap() 方法
            Alert.alert('成功', '举报已提交，请等待仲裁处理');
          },
        },
      ]
    );
  };

  const handleViewDetail = (swapId: number) => {
    router.push(`/bridge/${swapId}` as any);
  };

  // 过滤记录
  const filteredRecords = records.filter(record => {
    switch (filter) {
      case 'pending':
        return record.status === SwapStatus.Pending;
      case 'completed':
        return record.status === SwapStatus.Completed;
      case 'refunded':
        return record.status === SwapStatus.Refunded;
      default:
        return true;
    }
  });

  const filters: { key: FilterType; label: string }[] = [
    { key: 'all', label: '全部' },
    { key: 'pending', label: '处理中' },
    { key: 'completed', label: '已完成' },
    { key: 'refunded', label: '已退款' },
  ];

  return (
    <View style={styles.wrapper}>
      <PageHeader title="兑换记录" />

      {/* 过滤器 */}
      <View style={styles.filterContainer}>
        <ScrollView horizontal showsHorizontalScrollIndicator={false}>
          {filters.map(f => (
            <TouchableOpacity
              key={f.key}
              style={[
                styles.filterButton,
                filter === f.key && styles.filterButtonActive,
              ]}
              onPress={() => setFilter(f.key)}
            >
              <Text
                style={[
                  styles.filterText,
                  filter === f.key && styles.filterTextActive,
                ]}
              >
                {f.label}
              </Text>
            </TouchableOpacity>
          ))}
        </ScrollView>
      </View>

      <ScrollView
        style={styles.container}
        contentContainerStyle={styles.contentContainer}
        refreshControl={
          <RefreshControl
            refreshing={refreshing}
            onRefresh={handleRefresh}
            colors={['#B2955D']}
          />
        }
      >
        {loading ? (
          <View style={styles.loading}>
            <ActivityIndicator size="large" color="#B2955D" />
            <Text style={styles.loadingText}>加载中...</Text>
          </View>
        ) : filteredRecords.length === 0 ? (
          <View style={styles.empty}>
            <Text style={styles.emptyEmoji}>📋</Text>
            <Text style={styles.emptyText}>暂无兑换记录</Text>
            <TouchableOpacity
              style={styles.emptyButton}
              onPress={() => router.push('/bridge' as any)}
            >
              <Text style={styles.emptyButtonText}>去兑换</Text>
            </TouchableOpacity>
          </View>
        ) : (
          <View style={styles.recordList}>
            {filteredRecords.map(record => (
              <SwapRecordCard
                key={record.swapId}
                record={record}
                onPress={() => handleViewDetail(record.swapId)}
                onReport={
                  record.status === SwapStatus.Pending ||
                  record.status === SwapStatus.Completed
                    ? () => handleReport(record.swapId)
                    : undefined
                }
              />
            ))}
          </View>
        )}

        {/* 统计信息 */}
        {!loading && records.length > 0 && (
          <View style={styles.stats}>
            <Text style={styles.statsTitle}>📊 统计</Text>
            <View style={styles.statsRow}>
              <View style={styles.statItem}>
                <Text style={styles.statValue}>{records.length}</Text>
                <Text style={styles.statLabel}>总兑换</Text>
              </View>
              <View style={styles.statItem}>
                <Text style={styles.statValue}>
                  {records.filter(r => r.status === SwapStatus.Completed).length}
                </Text>
                <Text style={styles.statLabel}>已完成</Text>
              </View>
              <View style={styles.statItem}>
                <Text style={styles.statValue}>
                  {records.filter(r => r.status === SwapStatus.Pending).length}
                </Text>
                <Text style={styles.statLabel}>处理中</Text>
              </View>
            </View>
          </View>
        )}
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
  filterContainer: {
    backgroundColor: '#FFFFFF',
    paddingVertical: 12,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#E5E5EA',
  },
  filterButton: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 16,
    backgroundColor: '#F5F5F7',
    marginRight: 8,
  },
  filterButtonActive: {
    backgroundColor: '#B2955D',
  },
  filterText: {
    fontSize: 14,
    color: '#666666',
  },
  filterTextActive: {
    color: '#FFFFFF',
    fontWeight: '500',
  },
  container: {
    flex: 1,
  },
  contentContainer: {
    padding: 16,
    paddingBottom: 20,
  },
  loading: {
    alignItems: 'center',
    paddingVertical: 60,
  },
  loadingText: {
    fontSize: 14,
    color: '#666666',
    marginTop: 12,
  },
  empty: {
    alignItems: 'center',
    paddingVertical: 60,
  },
  emptyEmoji: {
    fontSize: 48,
    marginBottom: 16,
  },
  emptyText: {
    fontSize: 16,
    color: '#999999',
    marginBottom: 16,
  },
  emptyButton: {
    backgroundColor: '#B2955D',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  emptyButtonText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  recordList: {
    marginBottom: 16,
  },
  stats: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
  },
  statsTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 12,
  },
  statsRow: {
    flexDirection: 'row',
    justifyContent: 'space-around',
  },
  statItem: {
    alignItems: 'center',
  },
  statValue: {
    fontSize: 24,
    fontWeight: '600',
    color: '#B2955D',
    marginBottom: 4,
  },
  statLabel: {
    fontSize: 12,
    color: '#666666',
  },
});
