import { ProviderData } from '@/src/components/ProviderCard';
import { useAuthStore } from '@/src/stores/auth';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useState } from 'react';
import {
    Alert,
    Platform,
    Pressable,
    ScrollView,
    StyleSheet,
    Text,
    View
} from 'react-native';

interface ServicePackage {
  id: string;
  name: string;
  description: string;
  price: string;
  duration: string;
  popular?: boolean;
}

interface Review {
  id: string;
  userId: string;
  userName: string;
  rating: number;
  content: string;
  createdAt: string;
  reply?: string;
}

const MOCK_PROVIDER: ProviderData = {
  id: '1',
  name: '玄明道长',
  specialties: ['八字命理', '紫微斗数', '六爻占卜', '梅花易数'],
  rating: 4.9,
  completedOrders: 1280,
  price: '¥88',
  responseTime: '5分钟',
  isOnline: true,
  description: '道家正一派传人，从业二十余年，擅长八字命理、紫微斗数。为人诚恳，断事精准，深受客户信赖。',
};

const MOCK_PACKAGES: ServicePackage[] = [
  {
    id: '1',
    name: '八字简批',
    description: '分析性格特点、大运走势',
    price: '¥88',
    duration: '24小时内',
  },
  {
    id: '2',
    name: '八字详批',
    description: '完整命盘分析、流年运势、事业财运、婚姻感情',
    price: '¥288',
    duration: '48小时内',
    popular: true,
  },
  {
    id: '3',
    name: '紫微斗数',
    description: '十二宫位详解、一生运势分析',
    price: '¥388',
    duration: '72小时内',
  },
  {
    id: '4',
    name: '六爻问事',
    description: '针对具体问题起卦分析',
    price: '¥168',
    duration: '12小时内',
  },
];

const MOCK_REVIEWS: Review[] = [
  {
    id: '1',
    userId: 'u1',
    userName: '缘***分',
    rating: 5,
    content: '大师分析得很准，性格特点完全说中了，对未来的建议也很实用。',
    createdAt: '2024-01-15',
    reply: '感谢认可，祝您诸事顺利。',
  },
  {
    id: '2',
    userId: 'u2',
    userName: '星***尘',
    rating: 5,
    content: '问了感情问题，大师耐心解答，给了很多中肯的建议。',
    createdAt: '2024-01-10',
  },
  {
    id: '3',
    userId: 'u3',
    userName: '月***光',
    rating: 4,
    content: '整体不错，分析很细致，就是等的时间稍微长了点。',
    createdAt: '2024-01-05',
  },
];

export default function ProviderDetailScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const router = useRouter();
  const { isLoggedIn } = useAuthStore();
  const [selectedPackage, setSelectedPackage] = useState<string | null>(null);

  const handleOrder = (pkg: ServicePackage) => {
    if (!isLoggedIn) {
      if (Platform.OS === 'web') {
        window.alert('请先登录钱包');
      } else {
        Alert.alert('提示', '请先登录钱包');
      }
      return;
    }
    
    setSelectedPackage(pkg.id);
    router.push(`/market/order/new?providerId=${id}&packageId=${pkg.id}`);
  };

  const handleChat = () => {
    if (!isLoggedIn) {
      if (Platform.OS === 'web') {
        window.alert('请先登录钱包');
      } else {
        Alert.alert('提示', '请先登录钱包');
      }
      return;
    }
    router.push(`/chat/${id}`);
  };

  const renderStars = (rating: number) => {
    return '★'.repeat(Math.floor(rating)) + '☆'.repeat(5 - Math.floor(rating));
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>占卜师详情</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.providerSection}>
          <View style={styles.avatarRow}>
            <View style={styles.avatar}>
              <Text style={styles.avatarText}>{MOCK_PROVIDER.name[0]}</Text>
            </View>
            <View style={styles.providerInfo}>
              <View style={styles.nameRow}>
                <Text style={styles.providerName}>{MOCK_PROVIDER.name}</Text>
                {MOCK_PROVIDER.isOnline && (
                  <View style={styles.onlineBadge}>
                    <Text style={styles.onlineText}>在线</Text>
                  </View>
                )}
              </View>
              <View style={styles.statsRow}>
                <Text style={styles.stars}>{renderStars(MOCK_PROVIDER.rating)}</Text>
                <Text style={styles.rating}>{MOCK_PROVIDER.rating}</Text>
                <Text style={styles.orders}>{MOCK_PROVIDER.completedOrders}单</Text>
              </View>
              <Text style={styles.responseTime}>⏱ 平均响应 {MOCK_PROVIDER.responseTime}</Text>
            </View>
          </View>
          
          <Text style={styles.description}>{MOCK_PROVIDER.description}</Text>
          
          <View style={styles.specialtiesRow}>
            {MOCK_PROVIDER.specialties.map((s, i) => (
              <View key={i} style={styles.specialtyTag}>
                <Text style={styles.specialtyText}>{s}</Text>
              </View>
            ))}
          </View>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>服务套餐</Text>
          {MOCK_PACKAGES.map((pkg) => (
            <Pressable
              key={pkg.id}
              style={({ pressed }) => [
                styles.packageCard,
                pkg.popular && styles.popularCard,
                pressed && styles.cardPressed,
              ]}
              onPress={() => handleOrder(pkg)}
            >
              {pkg.popular && (
                <View style={styles.popularBadge}>
                  <Text style={styles.popularText}>热门</Text>
                </View>
              )}
              <View style={styles.packageHeader}>
                <Text style={styles.packageName}>{pkg.name}</Text>
                <Text style={styles.packagePrice}>{pkg.price}</Text>
              </View>
              <Text style={styles.packageDesc}>{pkg.description}</Text>
              <Text style={styles.packageDuration}>预计 {pkg.duration} 完成</Text>
            </Pressable>
          ))}
        </View>

        <View style={styles.section}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>用户评价</Text>
            <Text style={styles.seeAll}>查看全部 &gt;</Text>
          </View>
          
          {MOCK_REVIEWS.map((review) => (
            <View key={review.id} style={styles.reviewCard}>
              <View style={styles.reviewHeader}>
                <Text style={styles.reviewUser}>{review.userName}</Text>
                <Text style={styles.reviewStars}>{renderStars(review.rating)}</Text>
                <Text style={styles.reviewDate}>{review.createdAt}</Text>
              </View>
              <Text style={styles.reviewContent}>{review.content}</Text>
              {review.reply && (
                <View style={styles.replyBox}>
                  <Text style={styles.replyLabel}>占卜师回复：</Text>
                  <Text style={styles.replyContent}>{review.reply}</Text>
                </View>
              )}
            </View>
          ))}
        </View>

        <View style={styles.bottomSpacer} />
      </ScrollView>

      <View style={styles.bottomBar}>
        <Pressable style={styles.chatButton} onPress={handleChat}>
          <Text style={styles.chatButtonText}>💬 咨询</Text>
        </Pressable>
        <Pressable 
          style={styles.orderButton}
          onPress={() => handleOrder(MOCK_PACKAGES[1])}
        >
          <Text style={styles.orderButtonText}>立即预约</Text>
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
  content: {
    flex: 1,
  },
  providerSection: {
    backgroundColor: '#fff',
    padding: 16,
    marginBottom: 12,
  },
  avatarRow: {
    flexDirection: 'row',
  },
  avatar: {
    width: 72,
    height: 72,
    borderRadius: 36,
    backgroundColor: '#6D28D9',
    justifyContent: 'center',
    alignItems: 'center',
  },
  avatarText: {
    color: '#fff',
    fontSize: 28,
    fontWeight: '600',
  },
  providerInfo: {
    flex: 1,
    marginLeft: 16,
    justifyContent: 'center',
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  providerName: {
    fontSize: 20,
    fontWeight: '700',
    color: '#1f2937',
  },
  onlineBadge: {
    backgroundColor: '#dcfce7',
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 10,
  },
  onlineText: {
    fontSize: 11,
    color: '#16a34a',
    fontWeight: '500',
  },
  statsRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 6,
  },
  stars: {
    color: '#f59e0b',
    fontSize: 14,
  },
  rating: {
    fontSize: 14,
    fontWeight: '600',
    color: '#f59e0b',
    marginLeft: 4,
  },
  orders: {
    fontSize: 13,
    color: '#9ca3af',
    marginLeft: 8,
  },
  responseTime: {
    fontSize: 13,
    color: '#6b7280',
    marginTop: 4,
  },
  description: {
    fontSize: 14,
    color: '#4b5563',
    lineHeight: 22,
    marginTop: 16,
  },
  specialtiesRow: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    marginTop: 12,
    gap: 8,
  },
  specialtyTag: {
    backgroundColor: '#f3e8ff',
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 14,
  },
  specialtyText: {
    fontSize: 13,
    color: '#7c3aed',
  },
  section: {
    backgroundColor: '#fff',
    padding: 16,
    marginBottom: 12,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 12,
  },
  seeAll: {
    fontSize: 14,
    color: '#6D28D9',
  },
  packageCard: {
    borderWidth: 1,
    borderColor: '#e5e7eb',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    position: 'relative',
  },
  popularCard: {
    borderColor: '#6D28D9',
    backgroundColor: '#faf5ff',
  },
  cardPressed: {
    opacity: 0.9,
  },
  popularBadge: {
    position: 'absolute',
    top: -8,
    right: 12,
    backgroundColor: '#6D28D9',
    paddingHorizontal: 10,
    paddingVertical: 3,
    borderRadius: 10,
  },
  popularText: {
    color: '#fff',
    fontSize: 11,
    fontWeight: '600',
  },
  packageHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  packageName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
  },
  packagePrice: {
    fontSize: 18,
    fontWeight: '700',
    color: '#6D28D9',
  },
  packageDesc: {
    fontSize: 14,
    color: '#6b7280',
    marginTop: 8,
    lineHeight: 20,
  },
  packageDuration: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 8,
  },
  reviewCard: {
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
    paddingBottom: 16,
    marginBottom: 16,
  },
  reviewHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  reviewUser: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1f2937',
  },
  reviewStars: {
    fontSize: 12,
    color: '#f59e0b',
  },
  reviewDate: {
    fontSize: 12,
    color: '#9ca3af',
    marginLeft: 'auto',
  },
  reviewContent: {
    fontSize: 14,
    color: '#4b5563',
    marginTop: 8,
    lineHeight: 20,
  },
  replyBox: {
    backgroundColor: '#f9fafb',
    padding: 12,
    borderRadius: 8,
    marginTop: 10,
  },
  replyLabel: {
    fontSize: 12,
    color: '#6D28D9',
    fontWeight: '500',
  },
  replyContent: {
    fontSize: 13,
    color: '#4b5563',
    marginTop: 4,
  },
  bottomSpacer: {
    height: 100,
  },
  bottomBar: {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
    flexDirection: 'row',
    backgroundColor: '#fff',
    paddingHorizontal: 16,
    paddingVertical: 12,
    paddingBottom: 30,
    borderTopWidth: 1,
    borderTopColor: '#e5e7eb',
    gap: 12,
  },
  chatButton: {
    flex: 1,
    backgroundColor: '#f3f4f6',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  chatButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#4b5563',
  },
  orderButton: {
    flex: 2,
    backgroundColor: '#6D28D9',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  orderButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#fff',
  },
});
