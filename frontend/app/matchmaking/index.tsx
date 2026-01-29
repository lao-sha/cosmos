import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useRouter } from 'expo-router';
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

interface MatchProfile {
  id: string;
  nickname: string;
  age: number;
  gender: 'male' | 'female';
  location: string;
  zodiac: string;
  baziScore?: number;
  bio: string;
  isVerified: boolean;
  photos: string[];
}

const MOCK_PROFILES: MatchProfile[] = [
  {
    id: '1',
    nickname: '星辰',
    age: 28,
    gender: 'female',
    location: '北京',
    zodiac: '双鱼座',
    baziScore: 92,
    bio: '喜欢阅读和旅行，希望找到志同道合的人',
    isVerified: true,
    photos: [],
  },
  {
    id: '2',
    nickname: '明月',
    age: 26,
    gender: 'female',
    location: '上海',
    zodiac: '天蝎座',
    baziScore: 88,
    bio: '热爱生活，期待美好的缘分',
    isVerified: true,
    photos: [],
  },
  {
    id: '3',
    nickname: '云飞',
    age: 30,
    gender: 'male',
    location: '深圳',
    zodiac: '狮子座',
    baziScore: 85,
    bio: '程序员一枚，工作之余喜欢运动',
    isVerified: false,
    photos: [],
  },
];

export default function MatchmakingScreen() {
  const router = useRouter();
  const { isLoggedIn } = useAuthStore();
  const { isConnected } = useChainStore();
  const [profiles] = useState<MatchProfile[]>(MOCK_PROFILES);
  const [hasProfile, setHasProfile] = useState(false);

  const handleProfilePress = (profile: MatchProfile) => {
    if (!isLoggedIn) {
      showAlert('请先登录钱包');
      return;
    }
    if (!hasProfile) {
      showAlert('请先创建个人资料');
      return;
    }
    showAlert(`查看 ${profile.nickname} 的详细资料`);
  };

  const handleLike = (profile: MatchProfile) => {
    if (!isLoggedIn) {
      showAlert('请先登录钱包');
      return;
    }
    showAlert(`已向 ${profile.nickname} 发送好感`);
  };

  const showAlert = (msg: string) => {
    if (Platform.OS === 'web') {
      window.alert(msg);
    } else {
      Alert.alert('提示', msg);
    }
  };

  const renderProfileCard = (profile: MatchProfile) => (
    <Pressable
      key={profile.id}
      style={({ pressed }) => [styles.profileCard, pressed && styles.profileCardPressed]}
      onPress={() => handleProfilePress(profile)}
    >
      <View style={styles.avatarContainer}>
        <View style={[styles.avatar, profile.gender === 'female' ? styles.avatarFemale : styles.avatarMale]}>
          <Text style={styles.avatarText}>{profile.nickname[0]}</Text>
        </View>
        {profile.isVerified && (
          <View style={styles.verifiedBadge}>
            <Text style={styles.verifiedText}>✓</Text>
          </View>
        )}
      </View>

      <View style={styles.profileInfo}>
        <View style={styles.nameRow}>
          <Text style={styles.nickname}>{profile.nickname}</Text>
          <Text style={styles.age}>{profile.age}岁</Text>
          <Text style={styles.gender}>{profile.gender === 'female' ? '♀' : '♂'}</Text>
        </View>
        
        <View style={styles.tagsRow}>
          <View style={styles.tag}>
            <Text style={styles.tagText}>📍 {profile.location}</Text>
          </View>
          <View style={styles.tag}>
            <Text style={styles.tagText}>⭐ {profile.zodiac}</Text>
          </View>
        </View>

        <Text style={styles.bio} numberOfLines={2}>{profile.bio}</Text>

        {profile.baziScore && (
          <View style={styles.scoreRow}>
            <Text style={styles.scoreLabel}>八字匹配度</Text>
            <View style={styles.scoreBar}>
              <View style={[styles.scoreFill, { width: `${profile.baziScore}%` }]} />
            </View>
            <Text style={styles.scoreValue}>{profile.baziScore}%</Text>
          </View>
        )}
      </View>

      <Pressable
        style={({ pressed }) => [styles.likeButton, pressed && styles.likeButtonPressed]}
        onPress={() => handleLike(profile)}
      >
        <Text style={styles.likeIcon}>💕</Text>
      </Pressable>
    </Pressable>
  );

  if (!isLoggedIn) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>缘分匹配</Text>
          <View style={{ width: 60 }} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>💕</Text>
          <Text style={styles.emptyTitle}>登录后开启缘分之旅</Text>
          <Text style={styles.emptySubtitle}>请先在"我的"页面创建钱包</Text>
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
        <Text style={styles.headerTitle}>缘分匹配</Text>
        <Pressable
          style={styles.editButton}
          onPress={() => router.push('/matchmaking/profile' as any)}
        >
          <Text style={styles.editButtonText}>资料</Text>
        </Pressable>
      </View>

      <View style={styles.infoBar}>
        <Text style={styles.infoText}>
          {isConnected ? '🟢 链上数据' : '🔴 链未连接'}
        </Text>
        <Text style={styles.infoText}>
          {hasProfile ? '资料已创建' : '⚠️ 请先创建资料'}
        </Text>
      </View>

      {!hasProfile && (
        <Pressable
          style={styles.createProfileBanner}
          onPress={() => router.push('/matchmaking/profile' as any)}
        >
          <Text style={styles.bannerIcon}>👤</Text>
          <View style={styles.bannerContent}>
            <Text style={styles.bannerTitle}>创建个人资料</Text>
            <Text style={styles.bannerSubtitle}>完善资料后才能查看匹配</Text>
          </View>
          <Text style={styles.bannerArrow}>›</Text>
        </Pressable>
      )}

      <ScrollView contentContainerStyle={styles.profilesList}>
        <Text style={styles.sectionTitle}>今日推荐</Text>
        {profiles.map(renderProfileCard)}
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
    padding: 8,
    width: 60,
  },
  backText: {
    fontSize: 16,
    color: '#6D28D9',
  },
  headerTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
  },
  editButton: {
    padding: 8,
    width: 60,
    alignItems: 'flex-end',
  },
  editButtonText: {
    fontSize: 14,
    color: '#6D28D9',
    fontWeight: '500',
  },
  infoBar: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    backgroundColor: '#fff',
    paddingHorizontal: 16,
    paddingVertical: 10,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  infoText: {
    fontSize: 13,
    color: '#6b7280',
  },
  createProfileBanner: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#fef3c7',
    margin: 12,
    padding: 16,
    borderRadius: 12,
  },
  bannerIcon: {
    fontSize: 32,
    marginRight: 12,
  },
  bannerContent: {
    flex: 1,
  },
  bannerTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#92400e',
  },
  bannerSubtitle: {
    fontSize: 13,
    color: '#b45309',
    marginTop: 2,
  },
  bannerArrow: {
    fontSize: 24,
    color: '#92400e',
  },
  profilesList: {
    padding: 12,
    paddingBottom: 30,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 12,
  },
  profileCard: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  profileCardPressed: {
    backgroundColor: '#f9fafb',
  },
  avatarContainer: {
    position: 'relative',
    marginRight: 14,
  },
  avatar: {
    width: 60,
    height: 60,
    borderRadius: 30,
    alignItems: 'center',
    justifyContent: 'center',
  },
  avatarFemale: {
    backgroundColor: '#fce7f3',
  },
  avatarMale: {
    backgroundColor: '#dbeafe',
  },
  avatarText: {
    fontSize: 24,
    fontWeight: '600',
    color: '#374151',
  },
  verifiedBadge: {
    position: 'absolute',
    bottom: 0,
    right: 0,
    backgroundColor: '#22c55e',
    width: 18,
    height: 18,
    borderRadius: 9,
    alignItems: 'center',
    justifyContent: 'center',
    borderWidth: 2,
    borderColor: '#fff',
  },
  verifiedText: {
    color: '#fff',
    fontSize: 10,
    fontWeight: 'bold',
  },
  profileInfo: {
    flex: 1,
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 6,
  },
  nickname: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginRight: 8,
  },
  age: {
    fontSize: 13,
    color: '#6b7280',
    marginRight: 4,
  },
  gender: {
    fontSize: 14,
  },
  tagsRow: {
    flexDirection: 'row',
    gap: 8,
    marginBottom: 6,
  },
  tag: {
    backgroundColor: '#f3f4f6',
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 4,
  },
  tagText: {
    fontSize: 11,
    color: '#6b7280',
  },
  bio: {
    fontSize: 13,
    color: '#6b7280',
    marginBottom: 8,
  },
  scoreRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  scoreLabel: {
    fontSize: 11,
    color: '#9ca3af',
    marginRight: 8,
  },
  scoreBar: {
    flex: 1,
    height: 6,
    backgroundColor: '#e5e7eb',
    borderRadius: 3,
    marginRight: 8,
  },
  scoreFill: {
    height: '100%',
    backgroundColor: '#ec4899',
    borderRadius: 3,
  },
  scoreValue: {
    fontSize: 12,
    fontWeight: '600',
    color: '#ec4899',
    width: 36,
  },
  likeButton: {
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: '#fce7f3',
    alignItems: 'center',
    justifyContent: 'center',
    alignSelf: 'center',
  },
  likeButtonPressed: {
    backgroundColor: '#fbcfe8',
  },
  likeIcon: {
    fontSize: 20,
  },
  emptyContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
  },
  emptyIcon: {
    fontSize: 64,
    marginBottom: 16,
  },
  emptyTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#1f2937',
    marginBottom: 8,
  },
  emptySubtitle: {
    fontSize: 14,
    color: '#9ca3af',
  },
});
