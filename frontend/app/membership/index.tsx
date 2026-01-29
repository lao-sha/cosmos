import { CheckInCard } from '@/src/components/CheckInCard';
import { MembershipCard, MemberTier, SubscriptionDuration } from '@/src/components/MembershipCard';
import { TransactionModal } from '@/src/components/TransactionModal';
import { useTransaction } from '@/src/hooks/useTransaction';
import { useAuthStore } from '@/src/stores/auth';
import { useRouter } from 'expo-router';
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

const MOCK_MEMBER_INFO = {
  tier: 'basic' as MemberTier,
  expiresAt: '2025-03-15',
  cosBalance: 1250,
  streak: 5,
  lastCheckIn: new Date().toISOString().split('T')[0],
  totalRewards: 8500,
};

export default function MembershipScreen() {
  const router = useRouter();
  const { isLoggedIn } = useAuthStore();
  const { status, isLoading, transfer, reset } = useTransaction();

  const [duration, setDuration] = useState<SubscriptionDuration>('monthly');
  const [selectedTier, setSelectedTier] = useState<MemberTier | null>(null);
  const [modalVisible, setModalVisible] = useState(false);

  const memberInfo = MOCK_MEMBER_INFO;

  const handleSelectTier = (tier: MemberTier) => {
    if (!isLoggedIn) {
      if (Platform.OS === 'web') {
        alert('请先登录');
      } else {
        Alert.alert('提示', '请先登录');
      }
      return;
    }
    setSelectedTier(tier);
    setModalVisible(true);
    reset();
  };

  const handleConfirmSubscription = async () => {
    if (!selectedTier) return;
    // TODO: 实现会员订阅交易
    // 目前使用 transfer 模拟
    await transfer('5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY', '0.1');
  };

  const handleCheckIn = async () => {
    // TODO: 实现签到交易
  };

  const getMultiplier = (tier: MemberTier): number => {
    const multipliers: Record<MemberTier, number> = {
      free: 1,
      basic: 2,
      premium: 3,
      vip: 5,
    };
    return multipliers[tier];
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>会员中心</Text>
        <Pressable onPress={() => router.push('/membership/rewards')}>
          <Text style={styles.rewardsLink}>奖励记录</Text>
        </Pressable>
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {isLoggedIn && (
          <>
            <View style={styles.statusCard}>
              <View style={styles.statusHeader}>
                <View>
                  <Text style={styles.currentTierLabel}>当前会员</Text>
                  <Text style={styles.currentTier}>
                    {memberInfo.tier === 'free' ? '免费版' :
                     memberInfo.tier === 'basic' ? '基础版' :
                     memberInfo.tier === 'premium' ? '高级版' : 'VIP版'}
                  </Text>
                </View>
                <View style={styles.cosBadge}>
                  <Text style={styles.cosAmount}>{memberInfo.cosBalance}</Text>
                  <Text style={styles.cosLabel}>COS</Text>
                </View>
              </View>
              {memberInfo.tier !== 'free' && (
                <Text style={styles.expiresText}>
                  有效期至 {memberInfo.expiresAt}
                </Text>
              )}
            </View>

            <CheckInCard
              streak={memberInfo.streak}
              lastCheckIn={memberInfo.lastCheckIn}
              reward={10}
              multiplier={getMultiplier(memberInfo.tier)}
              onCheckIn={handleCheckIn}
            />
          </>
        )}

        <View style={styles.durationToggle}>
          <Pressable
            style={[styles.durationButton, duration === 'monthly' && styles.durationActive]}
            onPress={() => setDuration('monthly')}
          >
            <Text style={[styles.durationText, duration === 'monthly' && styles.durationTextActive]}>
              按月订阅
            </Text>
          </Pressable>
          <Pressable
            style={[styles.durationButton, duration === 'yearly' && styles.durationActive]}
            onPress={() => setDuration('yearly')}
          >
            <Text style={[styles.durationText, duration === 'yearly' && styles.durationTextActive]}>
              按年订阅
            </Text>
            <View style={styles.saveBadge}>
              <Text style={styles.saveText}>省17%</Text>
            </View>
          </Pressable>
        </View>

        <View style={styles.tiersSection}>
          <MembershipCard
            tier="free"
            duration={duration}
            currentTier={memberInfo.tier}
            onSelect={handleSelectTier}
          />
          <MembershipCard
            tier="basic"
            duration={duration}
            currentTier={memberInfo.tier}
            onSelect={handleSelectTier}
          />
          <MembershipCard
            tier="premium"
            duration={duration}
            currentTier={memberInfo.tier}
            onSelect={handleSelectTier}
          />
          <MembershipCard
            tier="vip"
            duration={duration}
            currentTier={memberInfo.tier}
            onSelect={handleSelectTier}
          />
        </View>

        <View style={styles.faq}>
          <Text style={styles.faqTitle}>常见问题</Text>
          <View style={styles.faqItem}>
            <Text style={styles.faqQuestion}>如何获取 COS 奖励？</Text>
            <Text style={styles.faqAnswer}>
              每日签到、邀请好友、完成任务等方式均可获得 COS 奖励。会员可享受签到奖励倍数加成。
            </Text>
          </View>
          <View style={styles.faqItem}>
            <Text style={styles.faqQuestion}>可以随时取消订阅吗？</Text>
            <Text style={styles.faqAnswer}>
              可以随时取消自动续费，取消后当前订阅期内仍可正常使用会员权益。
            </Text>
          </View>
          <View style={styles.faqItem}>
            <Text style={styles.faqQuestion}>如何升级会员？</Text>
            <Text style={styles.faqAnswer}>
              选择更高级别的套餐即可升级，系统将自动计算差价并延长有效期。
            </Text>
          </View>
        </View>
      </ScrollView>

      <TransactionModal
        visible={modalVisible}
        status={status}
        isLoading={isLoading}
        title={`订阅${selectedTier === 'basic' ? '基础版' : selectedTier === 'premium' ? '高级版' : 'VIP版'}`}
        description={`确认订阅${duration === 'monthly' ? '月度' : '年度'}会员？`}
        onConfirm={handleConfirmSubscription}
        onClose={() => {
          setModalVisible(false);
          reset();
        }}
      />
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
    backgroundColor: '#6D28D9',
    paddingTop: 50,
    paddingBottom: 16,
    paddingHorizontal: 16,
  },
  backButton: {
    padding: 4,
  },
  backText: {
    fontSize: 17,
    color: '#fff',
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#fff',
  },
  rewardsLink: {
    fontSize: 15,
    color: 'rgba(255,255,255,0.9)',
  },
  content: {
    flex: 1,
    padding: 16,
  },
  statusCard: {
    backgroundColor: '#6D28D9',
    borderRadius: 16,
    padding: 20,
    marginBottom: 16,
  },
  statusHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
  },
  currentTierLabel: {
    fontSize: 13,
    color: 'rgba(255,255,255,0.8)',
  },
  currentTier: {
    fontSize: 24,
    fontWeight: '700',
    color: '#fff',
    marginTop: 4,
  },
  cosBadge: {
    backgroundColor: 'rgba(255,255,255,0.2)',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 20,
    alignItems: 'center',
  },
  cosAmount: {
    fontSize: 20,
    fontWeight: '700',
    color: '#fff',
  },
  cosLabel: {
    fontSize: 11,
    color: 'rgba(255,255,255,0.8)',
  },
  expiresText: {
    fontSize: 13,
    color: 'rgba(255,255,255,0.7)',
    marginTop: 12,
  },
  durationToggle: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 4,
    marginBottom: 16,
  },
  durationButton: {
    flex: 1,
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
    flexDirection: 'row',
    justifyContent: 'center',
  },
  durationActive: {
    backgroundColor: '#6D28D9',
  },
  durationText: {
    fontSize: 15,
    fontWeight: '600',
    color: '#6b7280',
  },
  durationTextActive: {
    color: '#fff',
  },
  saveBadge: {
    backgroundColor: '#22c55e',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
    marginLeft: 6,
  },
  saveText: {
    fontSize: 10,
    fontWeight: '600',
    color: '#fff',
  },
  tiersSection: {
    marginBottom: 24,
  },
  faq: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 20,
    marginBottom: 32,
  },
  faqTitle: {
    fontSize: 18,
    fontWeight: '700',
    color: '#1f2937',
    marginBottom: 16,
  },
  faqItem: {
    marginBottom: 16,
  },
  faqQuestion: {
    fontSize: 15,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 4,
  },
  faqAnswer: {
    fontSize: 14,
    color: '#6b7280',
    lineHeight: 20,
  },
});
