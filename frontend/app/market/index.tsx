// frontend/app/market/index.tsx

import React, { useEffect, useState, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  RefreshControl,
  SafeAreaView,
  StatusBar,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useRouter } from 'expo-router';
import { useMarketApi } from '@/divination/market/hooks';
import { ProviderCard } from '@/divination/market/components/ProviderCard';
import { LoadingSpinner, EmptyState } from '@/divination/market/components';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { DIVINATION_TYPES, SORT_OPTIONS } from '@/divination/market/constants/market.constants';
import { Provider, ProviderFilterParams } from '@/divination/market/types';
import { BottomNavBar } from '@/components/BottomNavBar';

export default function MarketScreen() {
  const router = useRouter();
  const { loading, error, getProviders } = useMarketApi();

  const [providers, setProviders] = useState<Provider[]>([]);
  const [refreshing, setRefreshing] = useState(false);
  const [filterType, setFilterType] = useState<'all' | number>('all');
  const [sortType, setSortType] = useState<'comprehensive' | 'rating' | 'orders' | 'price'>('comprehensive');

  const loadProviders = useCallback(async () => {
    try {
      const params: ProviderFilterParams = {
        filterType,
        sortType,
      };
      const result = await getProviders(params);
      setProviders(result);
    } catch (err) {
      console.error('Load providers error:', err);
    }
  }, [getProviders, filterType, sortType]);

  useEffect(() => {
    loadProviders();
  }, [loadProviders]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await loadProviders();
    setRefreshing(false);
  }, [loadProviders]);

  const handleSearch = () => {
    router.push('/market/search');
  };

  const handleMyOrders = () => {
    router.push('/market/order/list');
  };

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="dark-content" backgroundColor={THEME.background} />

      {/* 顶部导航栏 */}
      <View style={styles.header}>
        <View style={styles.headerLeft}>
          <Text style={styles.headerTitle}>占卜市场</Text>
          <TouchableOpacity
            style={styles.becomeDivinerBtn}
            onPress={() => router.push('/diviner')}
          >
            <Ionicons name="sparkles" size={14} color={THEME.primary} />
            <Text style={styles.becomeDivinerText}>成为占卜师</Text>
          </TouchableOpacity>
        </View>
        <View style={styles.headerActions}>
          <TouchableOpacity style={styles.headerBtn} onPress={handleSearch}>
            <Ionicons name="search-outline" size={22} color={THEME.text} />
          </TouchableOpacity>
          <TouchableOpacity style={styles.headerBtn} onPress={handleMyOrders}>
            <Ionicons name="document-text-outline" size={22} color={THEME.text} />
          </TouchableOpacity>
        </View>
      </View>

      {/* 筛选栏 */}
      <View style={styles.filterContainer}>
        <ScrollView
          horizontal
          showsHorizontalScrollIndicator={false}
          contentContainerStyle={styles.filterScroll}
        >
          <TouchableOpacity
            style={[
              styles.filterChip,
              filterType === 'all' && styles.filterChipActive,
            ]}
            onPress={() => setFilterType('all')}
          >
            <Text
              style={[
                styles.filterChipText,
                filterType === 'all' && styles.filterChipTextActive,
              ]}
            >
              全部
            </Text>
          </TouchableOpacity>
          {DIVINATION_TYPES.map((type) => (
            <TouchableOpacity
              key={type.id}
              style={[
                styles.filterChip,
                filterType === type.id && styles.filterChipActive,
              ]}
              onPress={() => setFilterType(type.id)}
            >
              <Text
                style={[
                  styles.filterChipText,
                  filterType === type.id && styles.filterChipTextActive,
                ]}
              >
                {type.name}
              </Text>
            </TouchableOpacity>
          ))}
        </ScrollView>
      </View>

      {/* 排序栏 */}
      <View style={styles.sortContainer}>
        {SORT_OPTIONS.map((option) => (
          <TouchableOpacity
            key={option.value}
            style={styles.sortItem}
            onPress={() => setSortType(option.value as typeof sortType)}
          >
            <Text
              style={[
                styles.sortText,
                sortType === option.value && styles.sortTextActive,
              ]}
            >
              {option.label}
            </Text>
            {sortType === option.value && (
              <View style={styles.sortIndicator} />
            )}
          </TouchableOpacity>
        ))}
      </View>

      {/* 提供者列表 */}
      <ScrollView
        style={styles.listContainer}
        contentContainerStyle={styles.listContent}
        refreshControl={
          <RefreshControl
            refreshing={refreshing}
            onRefresh={onRefresh}
            colors={[THEME.primary]}
            tintColor={THEME.primary}
          />
        }
      >
        {loading && providers.length === 0 ? (
          <LoadingSpinner text="加载中..." />
        ) : providers.length === 0 ? (
          <EmptyState
            icon="people-outline"
            title="暂无解卦师"
            description="当前分类下没有可用的解卦师"
          />
        ) : (
          providers.map((provider) => (
            <ProviderCard key={provider.account} provider={provider} />
          ))
        )}
      </ScrollView>

      {/* 底部悬浮按钮 - 成为解卦师 */}
      <TouchableOpacity
        style={[styles.fab, SHADOWS.large]}
        onPress={() => router.push('/market/provider/register')}
      >
        <Ionicons name="add" size={24} color={THEME.textInverse} />
        <Text style={styles.fabText}>成为解卦师</Text>
      </TouchableOpacity>

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="market" />
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
    paddingHorizontal: 16,
    paddingVertical: 12,
    backgroundColor: THEME.card,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.border,
  },
  headerLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  headerTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: THEME.text,
  },
  becomeDivinerBtn: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: `${THEME.primary}15`,
    paddingHorizontal: 10,
    paddingVertical: 5,
    borderRadius: 14,
    gap: 4,
  },
  becomeDivinerText: {
    fontSize: 12,
    color: THEME.primary,
    fontWeight: '500',
  },
  headerActions: {
    flexDirection: 'row',
    gap: 8,
  },
  headerBtn: {
    padding: 8,
  },
  filterContainer: {
    backgroundColor: THEME.card,
    paddingVertical: 8,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.border,
  },
  filterScroll: {
    paddingHorizontal: 12,
    gap: 8,
  },
  filterChip: {
    paddingHorizontal: 14,
    paddingVertical: 6,
    borderRadius: 16,
    backgroundColor: THEME.background,
    marginRight: 8,
  },
  filterChipActive: {
    backgroundColor: THEME.primary,
  },
  filterChipText: {
    fontSize: 13,
    color: THEME.textSecondary,
  },
  filterChipTextActive: {
    color: THEME.textInverse,
    fontWeight: '500',
  },
  sortContainer: {
    flexDirection: 'row',
    backgroundColor: THEME.card,
    paddingVertical: 10,
    paddingHorizontal: 16,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.border,
  },
  sortItem: {
    flex: 1,
    alignItems: 'center',
    position: 'relative',
  },
  sortText: {
    fontSize: 13,
    color: THEME.textSecondary,
  },
  sortTextActive: {
    color: THEME.primary,
    fontWeight: '500',
  },
  sortIndicator: {
    position: 'absolute',
    bottom: -10,
    width: 20,
    height: 2,
    backgroundColor: THEME.primary,
    borderRadius: 1,
  },
  listContainer: {
    flex: 1,
  },
  listContent: {
    padding: 16,
    paddingBottom: 140,
  },
  fab: {
    position: 'absolute',
    right: 16,
    bottom: 84,
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: THEME.primary,
    paddingVertical: 12,
    paddingHorizontal: 16,
    borderRadius: 24,
    gap: 6,
  },
  fabText: {
    fontSize: 14,
    fontWeight: '500',
    color: THEME.textInverse,
  },
});
