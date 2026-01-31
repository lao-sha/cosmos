import { chainService, BaziChartSummary } from '@/src/services/chain';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useRouter } from 'expo-router';
import { useCallback, useEffect, useState } from 'react';
import {
  ActivityIndicator,
  FlatList,
  Pressable,
  RefreshControl,
  StyleSheet,
  Text,
  View,
} from 'react-native';

// 占卜类型定义
type DivinationType = 'bazi' | 'qimen' | 'meihua' | 'liuyao' | 'ziwei' | 'tarot';

interface DivinationRecord {
  id: string;
  type: DivinationType;
  name: string;
  summary: string;
  timestamp: number;
  chartId?: number;
}

const DIVINATION_INFO: Record<DivinationType, { label: string; icon: string; color: string; route: string }> = {
  bazi: { label: '八字命盘', icon: '📅', color: '#dc2626', route: '/divination/bazi' },
  qimen: { label: '奇门遁甲', icon: '🔮', color: '#7c3aed', route: '/divination/qimen' },
  meihua: { label: '梅花易数', icon: '🌸', color: '#ec4899', route: '/divination/meihua' },
  liuyao: { label: '六爻占卜', icon: '☯️', color: '#0891b2', route: '/divination/liuyao' },
  ziwei: { label: '紫微斗数', icon: '⭐', color: '#7c3aed', route: '/divination/ziwei' },
  tarot: { label: '塔罗牌', icon: '🃏', color: '#f59e0b', route: '/divination/tarot' },
};

export default function MyRecordsScreen() {
  const router = useRouter();
  const { isLoggedIn, address } = useAuthStore();
  const { isConnected } = useChainStore();

  const [activeTab, setActiveTab] = useState<'all' | DivinationType>('all');
  const [records, setRecords] = useState<DivinationRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);

  // 获取所有占卜记录
  const fetchRecords = useCallback(async () => {
    if (!address || !isConnected) return;

    setLoading(true);
    try {
      const allRecords: DivinationRecord[] = [];

      // 1. 获取八字命盘
      try {
        const baziCharts = await chainService.getUserBaziCharts(address);
        for (const chart of baziCharts) {
          allRecords.push({
            id: `bazi-${chart.chartId}`,
            type: 'bazi',
            name: chart.name || '未命名命盘',
            summary: chart.birthTime ? `出生时间: ${JSON.stringify(chart.birthTime)}` : '四柱命盘',
            timestamp: chart.timestamp || 0,
            chartId: chart.chartId,
          });
        }
      } catch (e) {
        console.log('获取八字记录失败:', e);
      }

      // 2. 获取奇门遁甲记录 (TODO: 实现 getUserQimenCharts)
      // try {
      //   const qimenCharts = await chainService.getUserQimenCharts(address);
      //   ...
      // } catch (e) {}

      // 3. 其他占卜模块... (按需添加)

      // 按时间倒序排列
      allRecords.sort((a, b) => b.timestamp - a.timestamp);
      setRecords(allRecords);
    } catch (error) {
      console.error('获取占卜记录失败:', error);
    } finally {
      setLoading(false);
    }
  }, [address, isConnected]);

  // 下拉刷新
  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await fetchRecords();
    setRefreshing(false);
  }, [fetchRecords]);

  // 初始加载
  useEffect(() => {
    if (isLoggedIn && isConnected) {
      fetchRecords();
    }
  }, [isLoggedIn, isConnected, fetchRecords]);

  // 筛选记录
  const filteredRecords = activeTab === 'all' 
    ? records 
    : records.filter(r => r.type === activeTab);

  // 查看详情
  const handleViewDetail = (record: DivinationRecord) => {
    if (record.type === 'bazi' && record.chartId !== undefined) {
      router.push(`/divination/bazi/${record.chartId}` as any);
    } else {
      console.log('查看详情:', record);
      // 其他占卜类型的详情页待实现
    }
  };

  // 渲染记录项
  const renderItem = ({ item }: { item: DivinationRecord }) => {
    const info = DIVINATION_INFO[item.type];
    return (
      <Pressable style={styles.recordItem} onPress={() => handleViewDetail(item)}>
        <View style={[styles.recordIcon, { backgroundColor: `${info.color}15` }]}>
          <Text style={styles.recordIconText}>{info.icon}</Text>
        </View>
        <View style={styles.recordContent}>
          <View style={styles.recordHeader}>
            <Text style={styles.recordName} numberOfLines={1}>{item.name}</Text>
            <View style={[styles.typeBadge, { backgroundColor: `${info.color}15` }]}>
              <Text style={[styles.typeText, { color: info.color }]}>{info.label}</Text>
            </View>
          </View>
          <Text style={styles.recordSummary} numberOfLines={1}>{item.summary}</Text>
          <Text style={styles.recordTime}>
            {item.timestamp > 0 ? `区块 #${item.timestamp}` : '时间未知'}
          </Text>
        </View>
        <Text style={styles.arrow}>›</Text>
      </Pressable>
    );
  };

  // 未登录状态
  if (!isLoggedIn) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>我的占卜</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.centerContent}>
          <Text style={styles.emptyIcon}>🔮</Text>
          <Text style={styles.emptyTitle}>请先登录</Text>
          <Text style={styles.emptyDesc}>登录后可查看您的占卜记录</Text>
          <Pressable style={styles.loginButton} onPress={() => router.push('/wallet' as any)}>
            <Text style={styles.loginButtonText}>前往登录</Text>
          </Pressable>
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
        <Text style={styles.headerTitle}>我的占卜</Text>
        <View style={styles.headerRight} />
      </View>

      {/* 分类标签 */}
      <View style={styles.tabContainer}>
        <FlatList
          horizontal
          showsHorizontalScrollIndicator={false}
          data={[
            { key: 'all', label: '全部', icon: '📋' },
            { key: 'bazi', label: '八字', icon: '📅' },
            { key: 'qimen', label: '奇门', icon: '🔮' },
            { key: 'meihua', label: '梅花', icon: '🌸' },
            { key: 'liuyao', label: '六爻', icon: '☯️' },
            { key: 'ziwei', label: '紫微', icon: '⭐' },
            { key: 'tarot', label: '塔罗', icon: '🃏' },
          ]}
          renderItem={({ item }) => (
            <Pressable
              style={[styles.tab, activeTab === item.key && styles.tabActive]}
              onPress={() => setActiveTab(item.key as any)}
            >
              <Text style={styles.tabIcon}>{item.icon}</Text>
              <Text style={[styles.tabText, activeTab === item.key && styles.tabTextActive]}>
                {item.label}
              </Text>
            </Pressable>
          )}
          keyExtractor={(item) => item.key}
          contentContainerStyle={styles.tabList}
        />
      </View>

      {/* 记录列表 */}
      {loading && records.length === 0 ? (
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color="#6D28D9" />
          <Text style={styles.loadingText}>加载中...</Text>
        </View>
      ) : (
        <FlatList
          data={filteredRecords}
          renderItem={renderItem}
          keyExtractor={(item) => item.id}
          contentContainerStyle={styles.list}
          showsVerticalScrollIndicator={false}
          refreshControl={
            <RefreshControl
              refreshing={refreshing}
              onRefresh={onRefresh}
              colors={['#6D28D9']}
              tintColor="#6D28D9"
            />
          }
          ListEmptyComponent={
            <View style={styles.emptyContainer}>
              <Text style={styles.emptyIcon}>📜</Text>
              <Text style={styles.emptyTitle}>暂无记录</Text>
              <Text style={styles.emptyDesc}>
                {activeTab === 'all' 
                  ? '您还没有任何占卜记录' 
                  : `您还没有${DIVINATION_INFO[activeTab as DivinationType]?.label || ''}记录`}
              </Text>
              <Pressable 
                style={styles.newButton} 
                onPress={() => router.push(
                  activeTab === 'all' 
                    ? '/(tabs)/market' as any
                    : DIVINATION_INFO[activeTab as DivinationType]?.route as any
                )}
              >
                <Text style={styles.newButtonText}>
                  {activeTab === 'all' ? '去占卜' : `新建${DIVINATION_INFO[activeTab as DivinationType]?.label || ''}`}
                </Text>
              </Pressable>
            </View>
          }
          ListFooterComponent={
            filteredRecords.length > 0 ? (
              <View style={styles.footer}>
                <Text style={styles.footerText}>
                  共 {filteredRecords.length} 条记录
                </Text>
              </View>
            ) : null
          }
        />
      )}
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
  tabContainer: {
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  tabList: {
    paddingHorizontal: 12,
    paddingVertical: 12,
    gap: 8,
  },
  tab: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 14,
    paddingVertical: 8,
    borderRadius: 20,
    backgroundColor: '#f3f4f6',
    marginRight: 8,
  },
  tabActive: {
    backgroundColor: '#6D28D9',
  },
  tabIcon: {
    fontSize: 14,
    marginRight: 4,
  },
  tabText: {
    fontSize: 13,
    color: '#6b7280',
  },
  tabTextActive: {
    color: '#fff',
    fontWeight: '600',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 40,
  },
  loadingText: {
    marginTop: 12,
    fontSize: 14,
    color: '#6b7280',
  },
  list: {
    padding: 16,
  },
  recordItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 8,
  },
  recordIcon: {
    width: 48,
    height: 48,
    borderRadius: 24,
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  recordIconText: {
    fontSize: 22,
  },
  recordContent: {
    flex: 1,
  },
  recordHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: 4,
  },
  recordName: {
    fontSize: 15,
    fontWeight: '600',
    color: '#1f2937',
    flex: 1,
    marginRight: 8,
  },
  typeBadge: {
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 10,
  },
  typeText: {
    fontSize: 11,
    fontWeight: '500',
  },
  recordSummary: {
    fontSize: 13,
    color: '#6b7280',
    marginBottom: 4,
  },
  recordTime: {
    fontSize: 11,
    color: '#9ca3af',
  },
  arrow: {
    fontSize: 20,
    color: '#9ca3af',
    marginLeft: 8,
  },
  centerContent: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 40,
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
    textAlign: 'center',
    marginBottom: 20,
  },
  loginButton: {
    backgroundColor: '#6D28D9',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  loginButtonText: {
    color: '#fff',
    fontSize: 15,
    fontWeight: '600',
  },
  newButton: {
    backgroundColor: '#6D28D9',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  newButtonText: {
    color: '#fff',
    fontSize: 15,
    fontWeight: '600',
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
