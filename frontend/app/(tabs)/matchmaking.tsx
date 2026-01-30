import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { matchmakingService, MatchProfile } from '@/src/services/matchmaking';
import { useRouter } from 'expo-router';
import { useState, useEffect, useCallback } from 'react';
import {
    ActivityIndicator,
    Alert,
    Platform,
    Pressable,
    RefreshControl,
    ScrollView,
    StyleSheet,
    Text,
    View
} from 'react-native';

export default function MatchmakingScreen() {
  const router = useRouter();
  const { isLoggedIn, address } = useAuthStore();
  const { isConnected } = useChainStore();
  const [profiles, setProfiles] = useState<MatchProfile[]>([]);
  const [loading, setLoading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [profileCount, setProfileCount] = useState(0);

  const fetchProfiles = useCallback(async () => {
    if (!isConnected) {
      setProfiles([]);
      return;
    }

    try {
      const allProfiles = await matchmakingService.getAllProfiles();
      setProfiles(allProfiles);
      
      const count = await matchmakingService.getProfileCount();
      setProfileCount(count);
    } catch (error) {
      console.error('获取档案失败:', error);
    }
  }, [isConnected]);

  useEffect(() => {
    if (isConnected) {
      setLoading(true);
      fetchProfiles().finally(() => setLoading(false));
    }
  }, [isConnected, fetchProfiles]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await fetchProfiles();
    setRefreshing(false);
  }, [fetchProfiles]);

  const handleProfilePress = (profile: MatchProfile) => {
    router.push(`/matchmaking/profile?id=${profile.id}` as any);
  };

  const handleCompatibility = () => {
    router.push('/matchmaking/compatibility' as any);
  };

  const handleRecommendations = () => {
    router.push('/matchmaking/recommendations' as any);
  };

  const handleCreateProfile = () => {
    if (!isLoggedIn) {
      Alert.alert('提示', '请先登录后创建个人资料');
      return;
    }
    router.push('/matchmaking/profile' as any);
  };

  const renderProfileCard = (profile: MatchProfile) => (
    <Pressable
      key={profile.id}
      style={({ pressed }) => [styles.profileCard, pressed && styles.profileCardPressed]}
      onPress={() => handleProfilePress(profile)}
    >
      <View style={styles.profileAvatar}>
        <Text style={styles.avatarText}>{profile.nickname?.[0] || '?'}</Text>
      </View>
      <View style={styles.profileInfo}>
        <View style={styles.profileHeader}>
          <Text style={styles.profileName}>{profile.nickname}</Text>
          {profile.isVerified && <Text style={styles.verifiedBadge}>✓</Text>}
        </View>
        <Text style={styles.profileMeta}>
          {profile.age ? `${profile.age}岁` : '年龄未设置'} · {profile.location} · {profile.zodiac}
        </Text>
        <Text style={styles.profileBio} numberOfLines={1}>{profile.bio || '暂无简介'}</Text>
      </View>
      <View style={styles.scoreBox}>
        <Text style={styles.scoreValue}>{profile.completeness}</Text>
        <Text style={styles.scoreLabel}>完整度</Text>
      </View>
    </Pressable>
  );

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.headerTitle}>八字合婚</Text>
        <Text style={styles.headerSubtitle}>以命理为基，寻觅良缘</Text>
      </View>

      <ScrollView 
        style={styles.content} 
        showsVerticalScrollIndicator={false}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
      >
        <View style={styles.actionCards}>
          <Pressable style={styles.actionCard} onPress={handleCompatibility}>
            <Text style={styles.actionIcon}>💑</Text>
            <Text style={styles.actionTitle}>合婚测算</Text>
            <Text style={styles.actionDesc}>输入双方八字，测算姻缘契合度</Text>
          </Pressable>
          
          <Pressable style={styles.actionCard} onPress={handleRecommendations}>
            <Text style={styles.actionIcon}>💘</Text>
            <Text style={styles.actionTitle}>智能推荐</Text>
            <Text style={styles.actionDesc}>基于八字匹配的缘分推荐</Text>
          </Pressable>
        </View>

        <View style={styles.section}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>推荐缘分 ({profileCount})</Text>
            <Pressable onPress={handleRecommendations}>
              <Text style={styles.seeAllText}>查看更多 ›</Text>
            </Pressable>
          </View>
          
          {loading ? (
            <View style={styles.loadingContainer}>
              <ActivityIndicator size="large" color="#ec4899" />
              <Text style={styles.loadingText}>加载中...</Text>
            </View>
          ) : profiles.length === 0 ? (
            <View style={styles.emptyContainer}>
              <Text style={styles.emptyIcon}>💔</Text>
              <Text style={styles.emptyText}>暂无档案</Text>
              <Text style={styles.emptySubtext}>
                {isConnected ? '成为第一个创建档案的人吧！' : '请先连接钱包'}
              </Text>
            </View>
          ) : (
            profiles.map(renderProfileCard)
          )}
        </View>

        <View style={styles.createProfileCard}>
          <Text style={styles.createProfileIcon}>✨</Text>
          <Text style={styles.createProfileTitle}>完善个人资料</Text>
          <Text style={styles.createProfileDesc}>
            填写八字信息，获得更精准的缘分匹配
          </Text>
          <Pressable style={styles.createProfileButton} onPress={handleCreateProfile}>
            <Text style={styles.createProfileButtonText}>立即完善</Text>
          </Pressable>
        </View>

        <View style={styles.bottomPadding} />
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    backgroundColor: '#ec4899',
    paddingTop: 50,
    paddingBottom: 20,
    paddingHorizontal: 20,
  },
  headerTitle: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#fff',
  },
  headerSubtitle: {
    fontSize: 14,
    color: 'rgba(255,255,255,0.8)',
    marginTop: 4,
  },
  content: {
    flex: 1,
  },
  actionCards: {
    flexDirection: 'row',
    padding: 16,
    gap: 12,
  },
  actionCard: {
    flex: 1,
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 16,
    alignItems: 'center',
  },
  actionIcon: {
    fontSize: 32,
    marginBottom: 8,
  },
  actionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 4,
  },
  actionDesc: {
    fontSize: 12,
    color: '#6b7280',
    textAlign: 'center',
  },
  section: {
    paddingHorizontal: 16,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
  },
  seeAllText: {
    fontSize: 14,
    color: '#ec4899',
  },
  profileCard: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  profileCardPressed: {
    backgroundColor: '#f9fafb',
  },
  profileAvatar: {
    width: 56,
    height: 56,
    borderRadius: 28,
    backgroundColor: '#ec4899',
    alignItems: 'center',
    justifyContent: 'center',
  },
  avatarText: {
    color: '#fff',
    fontSize: 22,
    fontWeight: 'bold',
  },
  profileInfo: {
    flex: 1,
    marginLeft: 12,
  },
  profileHeader: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  profileName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
  },
  verifiedBadge: {
    marginLeft: 6,
    fontSize: 12,
    color: '#22c55e',
    fontWeight: 'bold',
  },
  profileMeta: {
    fontSize: 13,
    color: '#6b7280',
    marginTop: 2,
  },
  profileBio: {
    fontSize: 13,
    color: '#9ca3af',
    marginTop: 4,
  },
  scoreBox: {
    alignItems: 'center',
    backgroundColor: '#fdf2f8',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 8,
  },
  scoreValue: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#ec4899',
  },
  scoreLabel: {
    fontSize: 10,
    color: '#9ca3af',
  },
  createProfileCard: {
    backgroundColor: '#fff',
    margin: 16,
    padding: 24,
    borderRadius: 16,
    alignItems: 'center',
  },
  createProfileIcon: {
    fontSize: 40,
    marginBottom: 12,
  },
  createProfileTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 8,
  },
  createProfileDesc: {
    fontSize: 14,
    color: '#6b7280',
    textAlign: 'center',
    marginBottom: 16,
  },
  createProfileButton: {
    backgroundColor: '#ec4899',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 24,
  },
  createProfileButtonText: {
    color: '#fff',
    fontSize: 15,
    fontWeight: '600',
  },
  bottomPadding: {
    height: 40,
  },
  loadingContainer: {
    padding: 40,
    alignItems: 'center',
  },
  loadingText: {
    marginTop: 12,
    fontSize: 14,
    color: '#6b7280',
  },
  emptyContainer: {
    padding: 40,
    alignItems: 'center',
    backgroundColor: '#fff',
    borderRadius: 12,
    marginBottom: 12,
  },
  emptyIcon: {
    fontSize: 48,
    marginBottom: 12,
  },
  emptyText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 4,
  },
  emptySubtext: {
    fontSize: 14,
    color: '#6b7280',
  },
});
