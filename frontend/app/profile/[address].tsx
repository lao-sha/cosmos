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
    View,
} from 'react-native';

interface UserProfile {
  address: string;
  nickname?: string;
  bio?: string;
  avatar?: string;
  joinDate: string;
  isVerified: boolean;
  stats: {
    orders: number;
    reviews: number;
    friends: number;
  };
  badges: string[];
  membershipTier?: 'basic' | 'premium' | 'vip';
}

const MOCK_PROFILE: UserProfile = {
  address: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
  nickname: '星辰旅者',
  bio: '热爱传统文化，相信命运可以被理解和引导。',
  joinDate: '2024-01-01',
  isVerified: true,
  stats: {
    orders: 15,
    reviews: 12,
    friends: 28,
  },
  badges: ['早期用户', '活跃会员', '热心评价'],
  membershipTier: 'premium',
};

export default function UserProfileScreen() {
  const { address } = useLocalSearchParams<{ address: string }>();
  const router = useRouter();
  const { address: myAddress, isLoggedIn } = useAuthStore();

  const [profile] = useState<UserProfile>(MOCK_PROFILE);
  const [isFriend, setIsFriend] = useState(false);
  const [isBlocked, setIsBlocked] = useState(false);

  const isOwnProfile = address === myAddress;

  const showAlert = (title: string, msg: string) => {
    if (Platform.OS === 'web') {
      window.alert(`${title}\n\n${msg}`);
    } else {
      Alert.alert(title, msg);
    }
  };

  const handleAddFriend = () => {
    if (!isLoggedIn) {
      showAlert('提示', '请先登录');
      return;
    }
    setIsFriend(true);
    showAlert('成功', '好友请求已发送');
  };

  const handleRemoveFriend = () => {
    if (Platform.OS === 'web') {
      if (window.confirm('确定要删除好友吗？')) {
        setIsFriend(false);
      }
    } else {
      Alert.alert('确认', '确定要删除好友吗？', [
        { text: '取消', style: 'cancel' },
        { text: '删除', style: 'destructive', onPress: () => setIsFriend(false) },
      ]);
    }
  };

  const handleBlock = () => {
    if (Platform.OS === 'web') {
      if (window.confirm('确定要拉黑该用户吗？')) {
        setIsBlocked(true);
        setIsFriend(false);
      }
    } else {
      Alert.alert('确认', '确定要拉黑该用户吗？', [
        { text: '取消', style: 'cancel' },
        { text: '拉黑', style: 'destructive', onPress: () => {
          setIsBlocked(true);
          setIsFriend(false);
        }},
      ]);
    }
  };

  const handleUnblock = () => {
    setIsBlocked(false);
    showAlert('成功', '已解除拉黑');
  };

  const handleChat = () => {
    if (!isLoggedIn) {
      showAlert('提示', '请先登录');
      return;
    }
    router.push(`/chat/${address}`);
  };

  const getMembershipLabel = (tier?: string) => {
    switch (tier) {
      case 'basic': return '基础会员';
      case 'premium': return '高级会员';
      case 'vip': return 'VIP会员';
      default: return null;
    }
  };

  const getMembershipColor = (tier?: string) => {
    switch (tier) {
      case 'basic': return '#6b7280';
      case 'premium': return '#6D28D9';
      case 'vip': return '#d4af37';
      default: return '#6b7280';
    }
  };

  const truncateAddress = (addr: string) => {
    return `${addr.slice(0, 10)}...${addr.slice(-8)}`;
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>用户资料</Text>
        {!isOwnProfile && (
          <Pressable style={styles.moreButton}>
            <Text style={styles.moreText}>⋯</Text>
          </Pressable>
        )}
        {isOwnProfile && <View style={styles.headerRight} />}
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.profileCard}>
          <View style={styles.avatarSection}>
            <View style={styles.avatar}>
              <Text style={styles.avatarText}>
                {profile.nickname?.[0] || address?.[2] || '?'}
              </Text>
            </View>
            {profile.isVerified && (
              <View style={styles.verifiedBadge}>
                <Text style={styles.verifiedIcon}>✓</Text>
              </View>
            )}
          </View>

          <Text style={styles.nickname}>{profile.nickname || '未设置昵称'}</Text>
          
          {profile.membershipTier && (
            <View style={[styles.memberBadge, { backgroundColor: getMembershipColor(profile.membershipTier) + '20' }]}>
              <Text style={[styles.memberText, { color: getMembershipColor(profile.membershipTier) }]}>
                {getMembershipLabel(profile.membershipTier)}
              </Text>
            </View>
          )}

          <Text style={styles.addressText}>{truncateAddress(address || '')}</Text>

          {profile.bio && (
            <Text style={styles.bio}>{profile.bio}</Text>
          )}

          <View style={styles.statsRow}>
            <View style={styles.statItem}>
              <Text style={styles.statValue}>{profile.stats.orders}</Text>
              <Text style={styles.statLabel}>订单</Text>
            </View>
            <View style={styles.statDivider} />
            <View style={styles.statItem}>
              <Text style={styles.statValue}>{profile.stats.reviews}</Text>
              <Text style={styles.statLabel}>评价</Text>
            </View>
            <View style={styles.statDivider} />
            <View style={styles.statItem}>
              <Text style={styles.statValue}>{profile.stats.friends}</Text>
              <Text style={styles.statLabel}>好友</Text>
            </View>
          </View>
        </View>

        {profile.badges.length > 0 && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>徽章</Text>
            <View style={styles.badgesRow}>
              {profile.badges.map((badge, index) => (
                <View key={index} style={styles.badgeItem}>
                  <Text style={styles.badgeText}>{badge}</Text>
                </View>
              ))}
            </View>
          </View>
        )}

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>信息</Text>
          <View style={styles.infoCard}>
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>注册时间</Text>
              <Text style={styles.infoValue}>{profile.joinDate}</Text>
            </View>
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>身份验证</Text>
              <Text style={[styles.infoValue, { color: profile.isVerified ? '#16a34a' : '#9ca3af' }]}>
                {profile.isVerified ? '已验证' : '未验证'}
              </Text>
            </View>
          </View>
        </View>

        {!isOwnProfile && (
          <View style={styles.actionsSection}>
            {isBlocked ? (
              <Pressable style={styles.unblockButton} onPress={handleUnblock}>
                <Text style={styles.unblockButtonText}>解除拉黑</Text>
              </Pressable>
            ) : (
              <>
                <View style={styles.actionButtonsRow}>
                  <Pressable style={styles.chatButton} onPress={handleChat}>
                    <Text style={styles.chatButtonText}>💬 发消息</Text>
                  </Pressable>
                  {isFriend ? (
                    <Pressable style={styles.removeFriendButton} onPress={handleRemoveFriend}>
                      <Text style={styles.removeFriendText}>删除好友</Text>
                    </Pressable>
                  ) : (
                    <Pressable style={styles.addFriendButton} onPress={handleAddFriend}>
                      <Text style={styles.addFriendText}>+ 加好友</Text>
                    </Pressable>
                  )}
                </View>

                <Pressable style={styles.blockButton} onPress={handleBlock}>
                  <Text style={styles.blockButtonText}>拉黑用户</Text>
                </Pressable>
              </>
            )}
          </View>
        )}

        {isOwnProfile && (
          <View style={styles.actionsSection}>
            <Pressable 
              style={styles.editProfileButton}
              onPress={() => router.push('/matchmaking/profile' as any)}
            >
              <Text style={styles.editProfileText}>编辑资料</Text>
            </Pressable>
          </View>
        )}

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
    width: 50,
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
  moreButton: {
    padding: 4,
    width: 50,
    alignItems: 'flex-end',
  },
  moreText: {
    fontSize: 20,
    color: '#6b7280',
  },
  content: {
    flex: 1,
  },
  profileCard: {
    backgroundColor: '#fff',
    padding: 24,
    alignItems: 'center',
    marginBottom: 12,
  },
  avatarSection: {
    position: 'relative',
    marginBottom: 16,
  },
  avatar: {
    width: 88,
    height: 88,
    borderRadius: 44,
    backgroundColor: '#6D28D9',
    justifyContent: 'center',
    alignItems: 'center',
  },
  avatarText: {
    color: '#fff',
    fontSize: 36,
    fontWeight: '600',
  },
  verifiedBadge: {
    position: 'absolute',
    bottom: 0,
    right: 0,
    width: 24,
    height: 24,
    borderRadius: 12,
    backgroundColor: '#22c55e',
    justifyContent: 'center',
    alignItems: 'center',
    borderWidth: 2,
    borderColor: '#fff',
  },
  verifiedIcon: {
    color: '#fff',
    fontSize: 12,
    fontWeight: 'bold',
  },
  nickname: {
    fontSize: 22,
    fontWeight: '700',
    color: '#1f2937',
    marginBottom: 8,
  },
  memberBadge: {
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 12,
    marginBottom: 8,
  },
  memberText: {
    fontSize: 12,
    fontWeight: '600',
  },
  addressText: {
    fontSize: 13,
    color: '#9ca3af',
    fontFamily: 'monospace',
    marginBottom: 12,
  },
  bio: {
    fontSize: 14,
    color: '#6b7280',
    textAlign: 'center',
    lineHeight: 20,
    marginBottom: 16,
  },
  statsRow: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingTop: 16,
    borderTopWidth: 1,
    borderTopColor: '#f3f4f6',
    width: '100%',
    justifyContent: 'center',
  },
  statItem: {
    alignItems: 'center',
    paddingHorizontal: 24,
  },
  statValue: {
    fontSize: 20,
    fontWeight: '700',
    color: '#1f2937',
  },
  statLabel: {
    fontSize: 13,
    color: '#9ca3af',
    marginTop: 2,
  },
  statDivider: {
    width: 1,
    height: 24,
    backgroundColor: '#e5e7eb',
  },
  section: {
    backgroundColor: '#fff',
    padding: 16,
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 12,
  },
  badgesRow: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  badgeItem: {
    backgroundColor: '#f3e8ff',
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 14,
  },
  badgeText: {
    fontSize: 13,
    color: '#7c3aed',
  },
  infoCard: {},
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  infoLabel: {
    fontSize: 14,
    color: '#6b7280',
  },
  infoValue: {
    fontSize: 14,
    color: '#1f2937',
  },
  actionsSection: {
    padding: 16,
  },
  actionButtonsRow: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 12,
  },
  chatButton: {
    flex: 2,
    backgroundColor: '#6D28D9',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  chatButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#fff',
  },
  addFriendButton: {
    flex: 1,
    backgroundColor: '#f3f4f6',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  addFriendText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#4b5563',
  },
  removeFriendButton: {
    flex: 1,
    backgroundColor: '#fee2e2',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  removeFriendText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#dc2626',
  },
  blockButton: {
    paddingVertical: 14,
    alignItems: 'center',
  },
  blockButtonText: {
    fontSize: 14,
    color: '#9ca3af',
  },
  unblockButton: {
    backgroundColor: '#f3f4f6',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  unblockButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#4b5563',
  },
  editProfileButton: {
    backgroundColor: '#6D28D9',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  editProfileText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#fff',
  },
  bottomPadding: {
    height: 32,
  },
});
