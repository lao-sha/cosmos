// frontend/app/market/provider/[id].tsx

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
import { useRouter, useLocalSearchParams } from 'expo-router';
import { useMarketApi } from '@/divination/market/hooks';
import {
  Avatar,
  TierBadge,
  RatingStars,
  SpecialtyTags,
  PackageCard,
  ReviewCard,
  LoadingSpinner,
  EmptyState,
} from '@/divination/market/components';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { Provider, ServicePackage, Review } from '@/divination/market/types';
import {
  calculateAverageRating,
  getDivinationTypeNames,
} from '@/divination/market/utils/market.utils';
import { getIpfsUrl } from '@/divination/market/services/ipfs.service';

type TabType = 'packages' | 'reviews';

export default function ProviderDetailScreen() {
  const router = useRouter();
  const { id } = useLocalSearchParams<{ id: string }>();
  const { loading, getProvider, getProviderPackages, getProviderReviews } = useMarketApi();

  const [provider, setProvider] = useState<Provider | null>(null);
  const [packages, setPackages] = useState<ServicePackage[]>([]);
  const [reviews, setReviews] = useState<Review[]>([]);
  const [refreshing, setRefreshing] = useState(false);
  const [activeTab, setActiveTab] = useState<TabType>('packages');

  const loadData = useCallback(async () => {
    if (!id) return;

    try {
      const [providerData, packagesData, reviewsData] = await Promise.all([
        getProvider(id),
        getProviderPackages(id),
        getProviderReviews(id),
      ]);

      setProvider(providerData);
      setPackages(packagesData);
      setReviews(reviewsData);
    } catch (err) {
      console.error('Load provider data error:', err);
    }
  }, [id, getProvider, getProviderPackages, getProviderReviews]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  }, [loadData]);

  const handleSelectPackage = (pkg: ServicePackage) => {
    router.push({
      pathname: '/market/order/create',
      params: {
        providerId: id,
        packageId: pkg.id.toString(),
        divinationType: pkg.divinationType.toString(),
      },
    });
  };

  if (loading && !provider) {
    return (
      <SafeAreaView style={styles.container}>
        <LoadingSpinner text="加载中..." fullScreen />
      </SafeAreaView>
    );
  }

  if (!provider) {
    return (
      <SafeAreaView style={styles.container}>
        <View style={styles.header}>
          <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
            <Ionicons name="arrow-back" size={24} color={THEME.text} />
          </TouchableOpacity>
        </View>
        <EmptyState
          icon="person-outline"
          title="解卦师不存在"
          description="该解卦师可能已注销"
          actionText="返回"
          onAction={() => router.back()}
        />
      </SafeAreaView>
    );
  }

  const averageRating = calculateAverageRating(provider.totalRating, provider.ratingCount);
  const divinationTypes = getDivinationTypeNames(provider.supportedTypes);

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="dark-content" backgroundColor={THEME.card} />

      {/* 顶部导航 */}
      <View style={styles.header}>
        <TouchableOpacity onPress={() => router.back()} style={styles.backBtn}>
          <Ionicons name="arrow-back" size={24} color={THEME.text} />
        </TouchableOpacity>
        <Text style={styles.headerTitle}>解卦师详情</Text>
        <View style={styles.backBtn} />
      </View>

      <ScrollView
        style={styles.content}
        refreshControl={
          <RefreshControl
            refreshing={refreshing}
            onRefresh={onRefresh}
            colors={[THEME.primary]}
            tintColor={THEME.primary}
          />
        }
      >
        {/* 基本信息卡片 */}
        <View style={[styles.profileCard, SHADOWS.medium]}>
          <View style={styles.profileHeader}>
            <Avatar
              uri={provider.avatarCid ? getIpfsUrl(provider.avatarCid) : undefined}
              name={provider.name}
              size={72}
            />
            <View style={styles.profileInfo}>
              <View style={styles.nameRow}>
                <Text style={styles.name}>{provider.name}</Text>
                <TierBadge tier={provider.tier} size="medium" />
              </View>
              <View style={styles.statsRow}>
                <RatingStars rating={averageRating} size={14} count={provider.ratingCount} />
              </View>
              <View style={styles.statsRow}>
                <Text style={styles.statText}>
                  {provider.completedOrders} 单完成
                </Text>
                {provider.acceptsUrgent && (
                  <View style={styles.urgentTag}>
                    <Ionicons name="flash" size={12} color={THEME.warning} />
                    <Text style={styles.urgentTagText}>可加急</Text>
                  </View>
                )}
              </View>
            </View>
          </View>

          {/* 简介 */}
          <Text style={styles.bio}>{provider.bio}</Text>

          {/* 擅长领域 */}
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>擅长领域</Text>
            <SpecialtyTags specialties={provider.specialties} maxDisplay={10} size="medium" showIcon />
          </View>

          {/* 占卜类型 */}
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>占卜类型</Text>
            <Text style={styles.typesText}>{divinationTypes.join('、')}</Text>
          </View>
        </View>

        {/* Tab 切换 */}
        <View style={styles.tabs}>
          <TouchableOpacity
            style={[styles.tab, activeTab === 'packages' && styles.tabActive]}
            onPress={() => setActiveTab('packages')}
          >
            <Text style={[styles.tabText, activeTab === 'packages' && styles.tabTextActive]}>
              服务套餐 ({packages.length})
            </Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={[styles.tab, activeTab === 'reviews' && styles.tabActive]}
            onPress={() => setActiveTab('reviews')}
          >
            <Text style={[styles.tabText, activeTab === 'reviews' && styles.tabTextActive]}>
              用户评价 ({reviews.length})
            </Text>
          </TouchableOpacity>
        </View>

        {/* Tab 内容 */}
        <View style={styles.tabContent}>
          {activeTab === 'packages' ? (
            packages.length === 0 ? (
              <EmptyState icon="pricetag-outline" title="暂无套餐" description="该解卦师尚未创建服务套餐" />
            ) : (
              packages.map((pkg) => (
                <PackageCard
                  key={pkg.id}
                  package_={pkg}
                  onSelect={handleSelectPackage}
                />
              ))
            )
          ) : reviews.length === 0 ? (
            <EmptyState icon="chatbubble-outline" title="暂无评价" description="该解卦师尚未收到评价" />
          ) : (
            reviews.map((review) => (
              <ReviewCard key={review.orderId} review={review} />
            ))
          )}
        </View>
      </ScrollView>
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
    width: 40,
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: THEME.text,
  },
  content: {
    flex: 1,
  },
  profileCard: {
    backgroundColor: THEME.card,
    margin: 16,
    borderRadius: 12,
    padding: 16,
  },
  profileHeader: {
    flexDirection: 'row',
  },
  profileInfo: {
    flex: 1,
    marginLeft: 14,
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  name: {
    fontSize: 18,
    fontWeight: '600',
    color: THEME.text,
  },
  statsRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 6,
    gap: 10,
  },
  statText: {
    fontSize: 13,
    color: THEME.textSecondary,
  },
  urgentTag: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: THEME.warning + '15',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
    gap: 2,
  },
  urgentTagText: {
    fontSize: 11,
    color: THEME.warning,
    fontWeight: '500',
  },
  bio: {
    fontSize: 14,
    color: THEME.textSecondary,
    lineHeight: 20,
    marginTop: 14,
  },
  section: {
    marginTop: 16,
  },
  sectionTitle: {
    fontSize: 14,
    fontWeight: '500',
    color: THEME.text,
    marginBottom: 8,
  },
  typesText: {
    fontSize: 13,
    color: THEME.textSecondary,
    lineHeight: 18,
  },
  tabs: {
    flexDirection: 'row',
    backgroundColor: THEME.card,
    marginHorizontal: 16,
    borderRadius: 8,
    padding: 4,
  },
  tab: {
    flex: 1,
    paddingVertical: 10,
    alignItems: 'center',
    borderRadius: 6,
  },
  tabActive: {
    backgroundColor: THEME.primary,
  },
  tabText: {
    fontSize: 14,
    color: THEME.textSecondary,
  },
  tabTextActive: {
    color: THEME.textInverse,
    fontWeight: '500',
  },
  tabContent: {
    padding: 16,
  },
});
