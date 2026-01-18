/**
 * 占卜师公开资料页面
 * 展示占卜师的详细信息、服务套餐和评价
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  ActivityIndicator,
  RefreshControl,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import {
  TierBadge,
  StatusBadge,
  PackageCard,
  ReviewCard,
  Provider,
  ProviderStatus,
  ProviderTier,
  ServicePackage,
  Review,
  DivinationType,
  ServiceType,
  SPECIALTY_CONFIG,
  DIVINATION_TYPE_CONFIG,
} from '@/features/diviner';

const THEME_COLOR = '#B2955D';

// Mock 数据
const mockProvider: Provider = {
  account: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
  name: '玄机子',
  bio: '从业20年，专注事业财运分析，擅长梅花易数和八字命理。曾为多家企业提供决策咨询，帮助数千人解答人生困惑。',
  specialties: 0b0000_0111, // 事业、感情、财运
  supportedTypes: 0b0000_0011, // 梅花、八字
  status: ProviderStatus.Active,
  tier: ProviderTier.Senior,
  totalOrders: 256,
  completedOrders: 248,
  totalEarnings: BigInt(25680 * 1e10),
  averageRating: 4.9,
  ratingCount: 242,
  acceptsUrgent: true,
  registeredAt: Date.now() - 86400000 * 365,
};

const mockPackages: ServicePackage[] = [
  {
    id: 1,
    providerId: mockProvider.account,
    divinationType: DivinationType.Meihua,
    serviceType: ServiceType.TextReading,
    name: '梅花易数·文字详解',
    description: '根据您的问题起卦，提供详细的卦象分析和建议，包含体用关系、五行生克等深度解读。',
    price: BigInt(10 * 1e10),
    duration: 0,
    followUpCount: 3,
    urgentAvailable: true,
    urgentSurcharge: 5000,
    isActive: true,
    salesCount: 89,
  },
  {
    id: 2,
    providerId: mockProvider.account,
    divinationType: DivinationType.Bazi,
    serviceType: ServiceType.VoiceReading,
    name: '八字命理·语音解读',
    description: '根据您的出生时间排盘，通过语音详细讲解命盘格局、大运流年等。',
    price: BigInt(25 * 1e10),
    duration: 15,
    followUpCount: 5,
    urgentAvailable: false,
    urgentSurcharge: 0,
    isActive: true,
    salesCount: 45,
  },
];

const mockReviews: Review[] = [
  {
    orderId: 1001,
    customer: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
    provider: mockProvider.account,
    overallRating: 5,
    accuracyRating: 5,
    attitudeRating: 5,
    responseRating: 5,
    contentCid: 'QmReview1...',
    isAnonymous: false,
    createdAt: Date.now() - 86400000,
  },
  {
    orderId: 1002,
    customer: '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy',
    provider: mockProvider.account,
    overallRating: 5,
    accuracyRating: 4,
    attitudeRating: 5,
    responseRating: 5,
    isAnonymous: true,
    replyCid: 'QmReply1...',
    createdAt: Date.now() - 172800000,
  },
];

type TabType = 'packages' | 'reviews';

export default function ProviderDetailPage() {
  const router = useRouter();
  const { providerId } = useLocalSearchParams<{ providerId: string }>();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [provider, setProvider] = useState<Provider | null>(null);
  const [packages, setPackages] = useState<ServicePackage[]>([]);
  const [reviews, setReviews] = useState<Review[]>([]);
  const [activeTab, setActiveTab] = useState<TabType>('packages');

  const loadData = async () => {
    await new Promise(resolve => setTimeout(resolve, 500));
    // TODO: 根据 providerId 从链上加载数据
    setProvider(mockProvider);
    setPackages(mockPackages);
    setReviews(mockReviews);
  };

  useEffect(() => {
    loadData().finally(() => setLoading(false));
  }, [providerId]);

  const onRefresh = async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  };

  const handleSelectPackage = (packageId: number) => {
    // TODO: 跳转到下单页面
    router.push(`/market/order?packageId=${packageId}` as any);
  };

  // 获取擅长领域标签
  const getSpecialtyTags = (specialties: number) => {
    const tags: { label: string; icon: string }[] = [];
    Object.entries(SPECIALTY_CONFIG).forEach(([key, config]) => {
      if (specialties & Number(key)) {
        tags.push(config);
      }
    });
    return tags;
  };

  // 获取支持的占卜类型
  const getDivinationTypes = (types: number) => {
    const result: { label: string; icon: string }[] = [];
    Object.entries(DIVINATION_TYPE_CONFIG).forEach(([key, config]) => {
      if (types & (1 << Number(key))) {
        result.push(config);
      }
    });
    return result;
  };

  if (loading) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="占卜师详情" />
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
        </View>
        <BottomNavBar activeTab="market" />
      </View>
    );
  }

  if (!provider) {
    return (
      <View style={styles.wrapper}>
        <PageHeader title="占卜师详情" />
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>🔮</Text>
          <Text style={styles.emptyText}>占卜师不存在</Text>
        </View>
        <BottomNavBar activeTab="market" />
      </View>
    );
  }

  const specialtyTags = getSpecialtyTags(provider.specialties);
  const divinationTypes = getDivinationTypes(provider.supportedTypes);
  const completionRate = provider.totalOrders > 0
    ? ((provider.completedOrders / provider.totalOrders) * 100).toFixed(0)
    : '0';

  return (
    <View style={styles.wrapper}>
      <PageHeader title="占卜师详情" />

      <ScrollView
        style={styles.container}
        contentContainerStyle={styles.contentContainer}
        refreshControl={<RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor={THEME_COLOR} />}
      >
        {/* 头部信息 */}
        <View style={styles.headerCard}>
          <View style={styles.avatarContainer}>
            <Text style={styles.avatarText}>{provider.name.charAt(0)}</Text>
          </View>
          <Text style={styles.providerName}>{provider.name}</Text>
          <View style={styles.badgeRow}>
            <TierBadge tier={provider.tier} size="medium" />
            <StatusBadge status={provider.status} />
          </View>
          <Text style={styles.bio}>{provider.bio}</Text>
        </View>

        {/* 统计数据 */}
        <View style={styles.statsCard}>
          <View style={styles.statItem}>
            <Text style={styles.statValue}>{provider.averageRating.toFixed(1)}</Text>
            <Text style={styles.statLabel}>评分</Text>
          </View>
          <View style={styles.statDivider} />
          <View style={styles.statItem}>
            <Text style={styles.statValue}>{provider.completedOrders}</Text>
            <Text style={styles.statLabel}>完成订单</Text>
          </View>
          <View style={styles.statDivider} />
          <View style={styles.statItem}>
            <Text style={styles.statValue}>{completionRate}%</Text>
            <Text style={styles.statLabel}>完成率</Text>
          </View>
          <View style={styles.statDivider} />
          <View style={styles.statItem}>
            <Text style={styles.statValue}>{provider.ratingCount}</Text>
            <Text style={styles.statLabel}>评价数</Text>
          </View>
        </View>

        {/* 擅长领域 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>擅长领域</Text>
          <View style={styles.tagsContainer}>
            {specialtyTags.map((tag, index) => (
              <View key={index} style={styles.tag}>
                <Text style={styles.tagIcon}>{tag.icon}</Text>
                <Text style={styles.tagLabel}>{tag.label}</Text>
              </View>
            ))}
          </View>
        </View>

        {/* 占卜类型 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>支持的占卜类型</Text>
          <View style={styles.tagsContainer}>
            {divinationTypes.map((type, index) => (
              <View key={index} style={styles.tag}>
                <Text style={styles.tagIcon}>{type.icon}</Text>
                <Text style={styles.tagLabel}>{type.label}</Text>
              </View>
            ))}
          </View>
        </View>

        {/* Tab 切换 */}
        <View style={styles.tabContainer}>
          <Pressable
            style={[styles.tab, activeTab === 'packages' && styles.tabActive]}
            onPress={() => setActiveTab('packages')}
          >
            <Text style={[styles.tabText, activeTab === 'packages' && styles.tabTextActive]}>
              服务套餐 ({packages.length})
            </Text>
          </Pressable>
          <Pressable
            style={[styles.tab, activeTab === 'reviews' && styles.tabActive]}
            onPress={() => setActiveTab('reviews')}
          >
            <Text style={[styles.tabText, activeTab === 'reviews' && styles.tabTextActive]}>
              用户评价 ({reviews.length})
            </Text>
          </Pressable>
        </View>

        {/* 套餐列表 */}
        {activeTab === 'packages' && (
          <View style={styles.listSection}>
            {packages.length === 0 ? (
              <View style={styles.emptyList}>
                <Text style={styles.emptyListText}>暂无服务套餐</Text>
              </View>
            ) : (
              packages.filter(p => p.isActive).map(pkg => (
                <PackageCard
                  key={pkg.id}
                  package={pkg}
                  onSelect={() => handleSelectPackage(pkg.id)}
                />
              ))
            )}
          </View>
        )}

        {/* 评价列表 */}
        {activeTab === 'reviews' && (
          <View style={styles.listSection}>
            {reviews.length === 0 ? (
              <View style={styles.emptyList}>
                <Text style={styles.emptyListText}>暂无评价</Text>
              </View>
            ) : (
              reviews.map(review => (
                <ReviewCard key={review.orderId} review={review} />
              ))
            )}
          </View>
        )}
      </ScrollView>

      <BottomNavBar activeTab="market" />
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
  emptyIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  emptyText: {
    fontSize: 16,
    color: '#999',
  },
  headerCard: {
    backgroundColor: '#FFF',
    padding: 24,
    alignItems: 'center',
  },
  avatarContainer: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: THEME_COLOR,
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 12,
  },
  avatarText: {
    fontSize: 32,
    color: '#FFF',
    fontWeight: '600',
  },
  providerName: {
    fontSize: 22,
    fontWeight: '600',
    color: '#000',
    marginBottom: 8,
  },
  badgeRow: {
    flexDirection: 'row',
    gap: 8,
    marginBottom: 12,
  },
  bio: {
    fontSize: 14,
    color: '#666',
    textAlign: 'center',
    lineHeight: 22,
    paddingHorizontal: 16,
  },
  statsCard: {
    flexDirection: 'row',
    backgroundColor: '#FFF',
    marginTop: 1,
    padding: 16,
    justifyContent: 'space-around',
  },
  statItem: {
    alignItems: 'center',
  },
  statValue: {
    fontSize: 20,
    fontWeight: '600',
    color: THEME_COLOR,
  },
  statLabel: {
    fontSize: 12,
    color: '#999',
    marginTop: 4,
  },
  statDivider: {
    width: 1,
    backgroundColor: '#F0F0F0',
  },
  section: {
    padding: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000',
    marginBottom: 12,
  },
  tagsContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  tag: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FFF',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 8,
    gap: 4,
  },
  tagIcon: {
    fontSize: 16,
  },
  tagLabel: {
    fontSize: 13,
    color: '#333',
  },
  tabContainer: {
    flexDirection: 'row',
    backgroundColor: '#FFF',
    marginTop: 8,
  },
  tab: {
    flex: 1,
    paddingVertical: 14,
    alignItems: 'center',
    borderBottomWidth: 2,
    borderBottomColor: 'transparent',
  },
  tabActive: {
    borderBottomColor: THEME_COLOR,
  },
  tabText: {
    fontSize: 15,
    color: '#666',
  },
  tabTextActive: {
    color: THEME_COLOR,
    fontWeight: '600',
  },
  listSection: {
    padding: 16,
  },
  emptyList: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 32,
    alignItems: 'center',
  },
  emptyListText: {
    fontSize: 14,
    color: '#999',
  },
});
