import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
  Share,
} from 'react-native';
import { useRouter } from 'expo-router';
import {
  Copy,
  Share2,
  Users,
  TrendingUp,
  Gift,
  ChevronRight,
  CheckCircle,
} from 'lucide-react-native';
import * as Clipboard from 'expo-clipboard';
import * as Haptics from 'expo-haptics';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import {
  getReferralInfo,
  getTeamMembers,
  bindReferrer,
  REFERRAL_LEVELS,
  type ReferralInfo,
  type ReferralMember,
} from '@/services/referral';
import { Button, Card, Input } from '@/components/ui';
import { Colors } from '@/constants/colors';

export default function ReferralScreen() {
  const colors = useColors();
  const router = useRouter();
  const { address, mnemonic } = useWalletStore();

  const [info, setInfo] = useState<ReferralInfo | null>(null);
  const [members, setMembers] = useState<ReferralMember[]>([]);
  const [loading, setLoading] = useState(true);
  const [copied, setCopied] = useState(false);
  const [showBindModal, setShowBindModal] = useState(false);
  const [referralCode, setReferralCode] = useState('');
  const [binding, setBinding] = useState(false);

  useEffect(() => {
    loadData();
  }, [address]);

  const loadData = async () => {
    if (!address) return;
    setLoading(true);
    try {
      const [infoData, membersData] = await Promise.all([
        getReferralInfo(address),
        getTeamMembers(address),
      ]);
      setInfo(infoData);
      setMembers(membersData);
    } catch (error) {
      console.error('Failed to load referral data:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCopy = async () => {
    if (!info?.code) return;
    await Clipboard.setStringAsync(info.code);
    setCopied(true);
    Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleShare = async () => {
    if (!info?.code) return;
    try {
      await Share.share({
        message: `使用我的邀请码 ${info.code} 注册 COSMOS，一起赚取收益！\n下载链接: https://cosmos.app/download`,
      });
    } catch (error) {
      console.error('Share failed:', error);
    }
  };

  const handleBind = async () => {
    if (!referralCode.trim() || !mnemonic) return;

    setBinding(true);
    try {
      await bindReferrer(referralCode.trim(), mnemonic);
      Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
      Alert.alert('绑定成功', '推荐人已绑定');
      setShowBindModal(false);
      setReferralCode('');
      loadData();
    } catch (error: any) {
      Alert.alert('绑定失败', error.message);
    } finally {
      setBinding(false);
    }
  };

  const formatAmount = (value: string): string => {
    const num = BigInt(value || '0');
    const whole = num / BigInt(1e12);
    return whole.toString();
  };

  if (loading) {
    return (
      <View style={[styles.container, { backgroundColor: colors.background }]}>
        <Text style={{ color: colors.textSecondary, textAlign: 'center', marginTop: 100 }}>
          加载中...
        </Text>
      </View>
    );
  }

  return (
    <ScrollView
      style={[styles.container, { backgroundColor: colors.background }]}
      contentContainerStyle={styles.content}
    >
      {/* Invite Code Card */}
      <Card style={[styles.codeCard, { backgroundColor: Colors.primary }]}>
        <Text style={styles.codeLabel}>我的邀请码</Text>
        <Text style={styles.codeValue}>{info?.code || '---'}</Text>
        <View style={styles.codeActions}>
          <TouchableOpacity style={styles.codeButton} onPress={handleCopy}>
            {copied ? (
              <CheckCircle size={20} color="#FFFFFF" />
            ) : (
              <Copy size={20} color="#FFFFFF" />
            )}
            <Text style={styles.codeButtonText}>
              {copied ? '已复制' : '复制'}
            </Text>
          </TouchableOpacity>
          <TouchableOpacity style={styles.codeButton} onPress={handleShare}>
            <Share2 size={20} color="#FFFFFF" />
            <Text style={styles.codeButtonText}>分享</Text>
          </TouchableOpacity>
        </View>
      </Card>

      {/* Referrer */}
      {!info?.referrer ? (
        <Card style={styles.bindCard}>
          <Text style={[styles.bindTitle, { color: colors.textPrimary }]}>
            绑定推荐人
          </Text>
          <Text style={[styles.bindSubtitle, { color: colors.textSecondary }]}>
            绑定后可享受推荐人的返佣福利
          </Text>
          <Input
            placeholder="输入推荐人邀请码"
            value={referralCode}
            onChangeText={setReferralCode}
            containerStyle={styles.bindInput}
          />
          <Button
            title="绑定"
            onPress={handleBind}
            loading={binding}
            disabled={!referralCode.trim()}
          />
        </Card>
      ) : (
        <Card style={styles.referrerCard}>
          <View style={styles.referrerRow}>
            <Text style={[styles.referrerLabel, { color: colors.textSecondary }]}>
              我的推荐人
            </Text>
            <Text style={[styles.referrerName, { color: colors.textPrimary }]}>
              {info.referrerName || info.referrer?.slice(0, 8) + '...'}
            </Text>
          </View>
        </Card>
      )}

      {/* Stats */}
      <View style={styles.statsGrid}>
        <Card style={styles.statCard}>
          <Users size={24} color={Colors.primary} />
          <Text style={[styles.statValue, { color: colors.textPrimary }]}>
            {info?.totalReferrals || 0}
          </Text>
          <Text style={[styles.statLabel, { color: colors.textSecondary }]}>
            团队人数
          </Text>
        </Card>
        <Card style={styles.statCard}>
          <TrendingUp size={24} color={Colors.success} />
          <Text style={[styles.statValue, { color: colors.textPrimary }]}>
            {formatAmount(info?.totalEarnings || '0')}
          </Text>
          <Text style={[styles.statLabel, { color: colors.textSecondary }]}>
            累计收益
          </Text>
        </Card>
      </View>

      {/* Pending Earnings */}
      <Card style={styles.earningsCard}>
        <View style={styles.earningsHeader}>
          <Gift size={24} color={Colors.accent} />
          <View style={styles.earningsInfo}>
            <Text style={[styles.earningsLabel, { color: colors.textSecondary }]}>
              待提取收益
            </Text>
            <Text style={[styles.earningsValue, { color: colors.textPrimary }]}>
              {formatAmount(info?.pendingEarnings || '0')} COS
            </Text>
          </View>
        </View>
        <Button
          title="提取收益"
          onPress={() => router.push('/referral/withdraw')}
          disabled={BigInt(info?.pendingEarnings || '0') === BigInt(0)}
          style={styles.withdrawButton}
        />
      </Card>

      {/* Team Members */}
      <Card style={styles.teamCard}>
        <TouchableOpacity
          style={styles.teamHeader}
          onPress={() => router.push('/referral/team')}
        >
          <Text style={[styles.teamTitle, { color: colors.textPrimary }]}>
            我的团队
          </Text>
          <View style={styles.teamMore}>
            <Text style={[styles.teamCount, { color: colors.textSecondary }]}>
              {info?.directReferrals || 0} 直推
            </Text>
            <ChevronRight size={20} color={colors.textTertiary} />
          </View>
        </TouchableOpacity>

        {members.slice(0, 5).map((member, index) => (
          <View
            key={member.address}
            style={[
              styles.memberItem,
              index < Math.min(members.length, 5) - 1 && {
                borderBottomWidth: 1,
                borderBottomColor: colors.border,
              },
            ]}
          >
            <View style={[styles.memberAvatar, { backgroundColor: Colors.referral[`l${member.level}` as keyof typeof Colors.referral] + '20' || Colors.primary + '20' }]}>
              <Text style={[styles.memberAvatarText, { color: Colors.referral[`l${member.level}` as keyof typeof Colors.referral] || Colors.primary }]}>
                {member.name.charAt(0)}
              </Text>
            </View>
            <View style={styles.memberInfo}>
              <Text style={[styles.memberName, { color: colors.textPrimary }]}>
                {member.name}
              </Text>
              <Text style={[styles.memberLevel, { color: colors.textTertiary }]}>
                L{member.level} • {new Date(member.joinedAt).toLocaleDateString()}
              </Text>
            </View>
            <Text style={[styles.memberContribution, { color: Colors.success }]}>
              +{formatAmount(member.contribution)}
            </Text>
          </View>
        ))}

        {members.length === 0 && (
          <Text style={[styles.emptyText, { color: colors.textTertiary }]}>
            暂无团队成员，快去邀请好友吧！
          </Text>
        )}
      </Card>

      {/* Commission Rates */}
      <Card style={styles.ratesCard}>
        <Text style={[styles.ratesTitle, { color: colors.textPrimary }]}>
          返佣比例
        </Text>
        <View style={styles.ratesGrid}>
          {REFERRAL_LEVELS.slice(0, 6).map((level) => (
            <View key={level.level} style={styles.rateItem}>
              <Text style={[styles.rateLevel, { color: colors.textSecondary }]}>
                {level.label}
              </Text>
              <Text style={[styles.rateValue, { color: Colors.primary }]}>
                {(level.rate * 100).toFixed(0)}%
              </Text>
            </View>
          ))}
        </View>
        <TouchableOpacity onPress={() => router.push('/referral/rules')}>
          <Text style={[styles.viewRules, { color: Colors.primary }]}>
            查看完整规则 →
          </Text>
        </TouchableOpacity>
      </Card>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    padding: 16,
    paddingBottom: 40,
  },
  codeCard: {
    alignItems: 'center',
    paddingVertical: 24,
    marginBottom: 16,
  },
  codeLabel: {
    color: 'rgba(255,255,255,0.8)',
    fontSize: 14,
    marginBottom: 8,
  },
  codeValue: {
    color: '#FFFFFF',
    fontSize: 32,
    fontWeight: '700',
    letterSpacing: 2,
    marginBottom: 16,
  },
  codeActions: {
    flexDirection: 'row',
    gap: 24,
  },
  codeButton: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
  },
  codeButtonText: {
    color: '#FFFFFF',
    fontSize: 14,
    fontWeight: '500',
  },
  bindCard: {
    marginBottom: 16,
  },
  bindTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 4,
  },
  bindSubtitle: {
    fontSize: 13,
    marginBottom: 12,
  },
  bindInput: {
    marginBottom: 12,
  },
  referrerCard: {
    marginBottom: 16,
  },
  referrerRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  referrerLabel: {
    fontSize: 14,
  },
  referrerName: {
    fontSize: 14,
    fontWeight: '500',
  },
  statsGrid: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 16,
  },
  statCard: {
    flex: 1,
    alignItems: 'center',
    paddingVertical: 20,
  },
  statValue: {
    fontSize: 24,
    fontWeight: '700',
    marginTop: 8,
  },
  statLabel: {
    fontSize: 13,
    marginTop: 4,
  },
  earningsCard: {
    marginBottom: 16,
  },
  earningsHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 16,
  },
  earningsInfo: {
    marginLeft: 12,
  },
  earningsLabel: {
    fontSize: 13,
  },
  earningsValue: {
    fontSize: 20,
    fontWeight: '700',
  },
  withdrawButton: {},
  teamCard: {
    marginBottom: 16,
  },
  teamHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  teamTitle: {
    fontSize: 16,
    fontWeight: '600',
  },
  teamMore: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  teamCount: {
    fontSize: 13,
  },
  memberItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 12,
  },
  memberAvatar: {
    width: 40,
    height: 40,
    borderRadius: 20,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  memberAvatarText: {
    fontSize: 16,
    fontWeight: '600',
  },
  memberInfo: {
    flex: 1,
  },
  memberName: {
    fontSize: 15,
    fontWeight: '500',
  },
  memberLevel: {
    fontSize: 12,
    marginTop: 2,
  },
  memberContribution: {
    fontSize: 14,
    fontWeight: '600',
  },
  emptyText: {
    textAlign: 'center',
    fontSize: 14,
    paddingVertical: 20,
  },
  ratesCard: {
    marginBottom: 16,
  },
  ratesTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 12,
  },
  ratesGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
  },
  rateItem: {
    width: '33.33%',
    alignItems: 'center',
    paddingVertical: 12,
  },
  rateLevel: {
    fontSize: 12,
    marginBottom: 4,
  },
  rateValue: {
    fontSize: 18,
    fontWeight: '700',
  },
  viewRules: {
    textAlign: 'center',
    fontSize: 14,
    marginTop: 8,
  },
});
