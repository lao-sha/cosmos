import { UserAvatar } from '@/src/components/UserAvatar';
import { useAuthStore } from '@/src/stores/auth';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    ActivityIndicator,
    FlatList,
    Pressable,
    StyleSheet,
    Text,
    View,
} from 'react-native';

interface RecommendationResult {
  userId: string;
  name: string;
  age: number;
  location: string;
  matchScore: number;
  tags: string[];
  lastActive: string;
}

const MOCK_RECOMMENDATIONS: RecommendationResult[] = [
  {
    userId: '1',
    name: '小美',
    age: 26,
    location: '北京',
    matchScore: 92,
    tags: ['温柔', '爱旅行', '喜欢美食'],
    lastActive: '刚刚',
  },
  {
    userId: '2',
    name: '佳琪',
    age: 28,
    location: '上海',
    matchScore: 88,
    tags: ['知性', '爱读书', '瑜伽'],
    lastActive: '5分钟前',
  },
  {
    userId: '3',
    name: '思雨',
    age: 25,
    location: '杭州',
    matchScore: 85,
    tags: ['活泼', '音乐', '摄影'],
    lastActive: '1小时前',
  },
  {
    userId: '4',
    name: '雅婷',
    age: 27,
    location: '深圳',
    matchScore: 82,
    tags: ['独立', '健身', '电影'],
    lastActive: '3小时前',
  },
  {
    userId: '5',
    name: '晓雪',
    age: 24,
    location: '成都',
    matchScore: 78,
    tags: ['可爱', '美食', '宠物'],
    lastActive: '昨天',
  },
];

export default function RecommendationsScreen() {
  const router = useRouter();
  const { isLoggedIn } = useAuthStore();

  const [loading, setLoading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);

  const handleRefresh = async () => {
    setRefreshing(true);
    // TODO: 调用链上 refresh_recommendations
    await new Promise((resolve) => setTimeout(resolve, 1500));
    setRefreshing(false);
  };

  const handleViewProfile = (userId: string) => {
    router.push(`/matchmaking/user/${userId}` as any);
  };

  const handleSendRequest = (userId: string) => {
    router.push('/matchmaking/compatibility' as any);
  };

  const getScoreColor = (score: number) => {
    if (score >= 90) return '#22c55e';
    if (score >= 80) return '#3b82f6';
    if (score >= 70) return '#f59e0b';
    return '#6b7280';
  };

  const renderItem = ({ item, index }: { item: RecommendationResult; index: number }) => (
    <Pressable
      style={styles.card}
      onPress={() => handleViewProfile(item.userId)}
    >
      <View style={styles.rankBadge}>
        <Text style={styles.rankText}>{index + 1}</Text>
      </View>

      <View style={styles.cardHeader}>
        <UserAvatar name={item.name} size="large" isOnline={item.lastActive === '刚刚'} />
        <View style={styles.userInfo}>
          <Text style={styles.userName}>{item.name}</Text>
          <Text style={styles.userMeta}>{item.age}岁 · {item.location}</Text>
          <Text style={styles.lastActive}>{item.lastActive}活跃</Text>
        </View>
        <View style={[styles.scoreBadge, { backgroundColor: `${getScoreColor(item.matchScore)}15` }]}>
          <Text style={[styles.scoreValue, { color: getScoreColor(item.matchScore) }]}>
            {item.matchScore}
          </Text>
          <Text style={[styles.scoreLabel, { color: getScoreColor(item.matchScore) }]}>
            匹配度
          </Text>
        </View>
      </View>

      <View style={styles.tagsRow}>
        {item.tags.map((tag, i) => (
          <View key={i} style={styles.tag}>
            <Text style={styles.tagText}>{tag}</Text>
          </View>
        ))}
      </View>

      <View style={styles.cardActions}>
        <Pressable
          style={styles.secondaryButton}
          onPress={() => handleViewProfile(item.userId)}
        >
          <Text style={styles.secondaryButtonText}>查看资料</Text>
        </Pressable>
        <Pressable
          style={styles.primaryButton}
          onPress={() => handleSendRequest(item.userId)}
        >
          <Text style={styles.primaryButtonText}>发起合婚</Text>
        </Pressable>
      </View>
    </Pressable>
  );

  if (!isLoggedIn) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>推荐列表</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.centerContent}>
          <Text style={styles.emptyText}>请先登录查看推荐</Text>
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
        <Text style={styles.headerTitle}>推荐列表</Text>
        <Pressable style={styles.refreshButton} onPress={handleRefresh} disabled={refreshing}>
          <Text style={styles.refreshText}>{refreshing ? '刷新中' : '🔄 刷新'}</Text>
        </Pressable>
      </View>

      <View style={styles.infoBar}>
        <Text style={styles.infoText}>
          💡 推荐基于八字五行匹配度计算，每日更新
        </Text>
      </View>

      {loading ? (
        <View style={styles.centerContent}>
          <ActivityIndicator size="large" color="#6D28D9" />
          <Text style={styles.loadingText}>加载推荐中...</Text>
        </View>
      ) : (
        <FlatList
          data={MOCK_RECOMMENDATIONS}
          renderItem={renderItem}
          keyExtractor={(item) => item.userId}
          contentContainerStyle={styles.list}
          showsVerticalScrollIndicator={false}
          refreshing={refreshing}
          onRefresh={handleRefresh}
          ListEmptyComponent={
            <View style={styles.centerContent}>
              <Text style={styles.emptyEmoji}>🔮</Text>
              <Text style={styles.emptyText}>暂无推荐</Text>
              <Text style={styles.emptyHint}>完善你的资料可获得更多推荐</Text>
            </View>
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
    width: 60,
  },
  refreshButton: {
    padding: 4,
  },
  refreshText: {
    fontSize: 14,
    color: '#6D28D9',
  },
  infoBar: {
    backgroundColor: '#f5f3ff',
    paddingHorizontal: 16,
    paddingVertical: 10,
  },
  infoText: {
    fontSize: 13,
    color: '#6D28D9',
  },
  list: {
    padding: 16,
  },
  card: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 16,
    marginBottom: 16,
    position: 'relative',
  },
  rankBadge: {
    position: 'absolute',
    top: -8,
    left: -8,
    width: 28,
    height: 28,
    borderRadius: 14,
    backgroundColor: '#6D28D9',
    justifyContent: 'center',
    alignItems: 'center',
  },
  rankText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '700',
  },
  cardHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  userInfo: {
    flex: 1,
    marginLeft: 12,
  },
  userName: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
  },
  userMeta: {
    fontSize: 14,
    color: '#6b7280',
    marginTop: 2,
  },
  lastActive: {
    fontSize: 12,
    color: '#22c55e',
    marginTop: 2,
  },
  scoreBadge: {
    alignItems: 'center',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 12,
  },
  scoreValue: {
    fontSize: 24,
    fontWeight: '700',
  },
  scoreLabel: {
    fontSize: 11,
  },
  tagsRow: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    marginBottom: 16,
  },
  tag: {
    backgroundColor: '#f3f4f6',
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
  },
  tagText: {
    fontSize: 12,
    color: '#4b5563',
  },
  cardActions: {
    flexDirection: 'row',
    gap: 12,
  },
  secondaryButton: {
    flex: 1,
    paddingVertical: 10,
    borderRadius: 8,
    backgroundColor: '#f3f4f6',
    alignItems: 'center',
  },
  secondaryButtonText: {
    color: '#4b5563',
    fontWeight: '600',
  },
  primaryButton: {
    flex: 1,
    paddingVertical: 10,
    borderRadius: 8,
    backgroundColor: '#6D28D9',
    alignItems: 'center',
  },
  primaryButtonText: {
    color: '#fff',
    fontWeight: '600',
  },
  centerContent: {
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
  emptyEmoji: {
    fontSize: 48,
    marginBottom: 16,
  },
  emptyText: {
    fontSize: 16,
    color: '#6b7280',
  },
  emptyHint: {
    fontSize: 13,
    color: '#9ca3af',
    marginTop: 8,
  },
});
