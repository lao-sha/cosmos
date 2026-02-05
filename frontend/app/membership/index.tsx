import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Crown, Gift, Star, ChevronRight, Check } from 'lucide-react-native';
import { LinearGradient } from 'expo-linear-gradient';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import {
  getMembershipInfo,
  MEMBERSHIP_LEVELS,
  getLevelConfig,
  getNextLevel,
  type MembershipInfo,
  type MembershipLevel,
} from '@/services/membership';
import { Card } from '@/components/ui';
import { Colors, Shadows } from '@/constants/colors';

export default function MembershipScreen() {
  const colors = useColors();
  const router = useRouter();
  const { address } = useWalletStore();

  const [info, setInfo] = useState<MembershipInfo | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, [address]);

  const loadData = async () => {
    if (!address) return;
    setLoading(true);
    try {
      const data = await getMembershipInfo(address);
      setInfo(data);
    } catch (error) {
      console.error('Failed to load membership:', error);
    } finally {
      setLoading(false);
    }
  };

  if (loading || !info) {
    return (
      <View style={[styles.container, { backgroundColor: colors.background }]}>
        <Text style={{ color: colors.textSecondary, textAlign: 'center', marginTop: 100 }}>
          加载中...
        </Text>
      </View>
    );
  }

  const levelConfig = getLevelConfig(info.level);
  const nextLevel = getNextLevel(info.level);
  const progress = nextLevel 
    ? ((info.points - levelConfig.minPoints) / (nextLevel.minPoints - levelConfig.minPoints)) * 100
    : 100;

  const getGradientColors = (level: MembershipLevel): [string, string] => {
    switch (level) {
      case 'bronze': return ['#CD7F32', '#8B4513'];
      case 'silver': return ['#C0C0C0', '#808080'];
      case 'gold': return ['#FFD700', '#DAA520'];
      case 'platinum': return ['#E5E4E2', '#A9A9A9'];
      case 'diamond': return ['#B9F2FF', '#87CEEB'];
      case 'supreme': return ['#FFD700', '#FF6B6B'];
      default: return ['#6B7280', '#4B5563'];
    }
  };

  return (
    <ScrollView
      style={[styles.container, { backgroundColor: colors.background }]}
      contentContainerStyle={styles.content}
    >
      {/* Level Card */}
      <LinearGradient
        colors={getGradientColors(info.level)}
        start={{ x: 0, y: 0 }}
        end={{ x: 1, y: 1 }}
        style={[styles.levelCard, Shadows.lg]}
      >
        <View style={styles.levelHeader}>
          <Crown size={32} color="#FFFFFF" />
          <Text style={styles.levelName}>{levelConfig.name}</Text>
        </View>
        
        <View style={styles.pointsSection}>
          <Text style={styles.pointsLabel}>当前积分</Text>
          <Text style={styles.pointsValue}>{info.points.toLocaleString()}</Text>
        </View>

        {nextLevel && (
          <View style={styles.progressSection}>
            <View style={styles.progressHeader}>
              <Text style={styles.progressText}>
                距离 {nextLevel.name}
              </Text>
              <Text style={styles.progressText}>
                还需 {(nextLevel.minPoints - info.points).toLocaleString()} 积分
              </Text>
            </View>
            <View style={styles.progressBar}>
              <View style={[styles.progressFill, { width: `${Math.min(progress, 100)}%` }]} />
            </View>
          </View>
        )}
      </LinearGradient>

      {/* Current Benefits */}
      <Card style={styles.benefitsCard}>
        <View style={styles.sectionHeader}>
          <Gift size={20} color={Colors.primary} />
          <Text style={[styles.sectionTitle, { color: colors.textPrimary }]}>
            当前权益
          </Text>
        </View>
        {info.benefits.map((benefit, index) => (
          <View key={index} style={styles.benefitItem}>
            <Check size={18} color={Colors.success} />
            <Text style={[styles.benefitText, { color: colors.textPrimary }]}>
              {benefit}
            </Text>
          </View>
        ))}
        <View style={styles.discountRow}>
          <Text style={[styles.discountLabel, { color: colors.textSecondary }]}>
            手续费折扣
          </Text>
          <Text style={[styles.discountValue, { color: Colors.success }]}>
            {levelConfig.discountRate}% OFF
          </Text>
        </View>
        <View style={styles.discountRow}>
          <Text style={[styles.discountLabel, { color: colors.textSecondary }]}>
            日交易限额
          </Text>
          <Text style={[styles.discountValue, { color: colors.textPrimary }]}>
            {levelConfig.dailyLimit === 0 ? '无限制' : `${levelConfig.dailyLimit.toLocaleString()} USDT`}
          </Text>
        </View>
      </Card>

      {/* All Levels */}
      <Card style={styles.levelsCard}>
        <Text style={[styles.sectionTitle, { color: colors.textPrimary, marginBottom: 16 }]}>
          等级说明
        </Text>
        {MEMBERSHIP_LEVELS.map((level, index) => {
          const isCurrent = level.level === info.level;
          const isUnlocked = info.points >= level.minPoints;
          
          return (
            <TouchableOpacity
              key={level.level}
              style={[
                styles.levelItem,
                index < MEMBERSHIP_LEVELS.length - 1 && {
                  borderBottomWidth: 1,
                  borderBottomColor: colors.border,
                },
              ]}
              activeOpacity={0.7}
            >
              <View style={styles.levelItemLeft}>
                <View
                  style={[
                    styles.levelIcon,
                    { backgroundColor: level.color + '20' },
                  ]}
                >
                  <Star
                    size={18}
                    color={level.color}
                    fill={isUnlocked ? level.color : 'transparent'}
                  />
                </View>
                <View>
                  <View style={styles.levelNameRow}>
                    <Text style={[styles.levelItemName, { color: colors.textPrimary }]}>
                      {level.name}
                    </Text>
                    {isCurrent && (
                      <View style={[styles.currentBadge, { backgroundColor: Colors.primary }]}>
                        <Text style={styles.currentBadgeText}>当前</Text>
                      </View>
                    )}
                  </View>
                  <Text style={[styles.levelItemPoints, { color: colors.textTertiary }]}>
                    {level.minPoints.toLocaleString()} 积分
                  </Text>
                </View>
              </View>
              <View style={styles.levelItemRight}>
                <Text style={[styles.levelDiscount, { color: level.color }]}>
                  {level.discountRate}% OFF
                </Text>
                <ChevronRight size={18} color={colors.textTertiary} />
              </View>
            </TouchableOpacity>
          );
        })}
      </Card>

      {/* How to Earn */}
      <Card style={styles.earnCard}>
        <Text style={[styles.sectionTitle, { color: colors.textPrimary, marginBottom: 16 }]}>
          如何获取积分
        </Text>
        <View style={styles.earnItem}>
          <Text style={[styles.earnAction, { color: colors.textPrimary }]}>
            完成交易
          </Text>
          <Text style={[styles.earnPoints, { color: Colors.success }]}>
            +10~50 积分
          </Text>
        </View>
        <View style={styles.earnItem}>
          <Text style={[styles.earnAction, { color: colors.textPrimary }]}>
            邀请好友
          </Text>
          <Text style={[styles.earnPoints, { color: Colors.success }]}>
            +100 积分/人
          </Text>
        </View>
        <View style={styles.earnItem}>
          <Text style={[styles.earnAction, { color: colors.textPrimary }]}>
            完成 KYC
          </Text>
          <Text style={[styles.earnPoints, { color: Colors.success }]}>
            +200 积分
          </Text>
        </View>
        <View style={styles.earnItem}>
          <Text style={[styles.earnAction, { color: colors.textPrimary }]}>
            每日签到
          </Text>
          <Text style={[styles.earnPoints, { color: Colors.success }]}>
            +5 积分
          </Text>
        </View>
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
  levelCard: {
    borderRadius: 20,
    padding: 24,
    marginBottom: 16,
  },
  levelHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 20,
  },
  levelName: {
    color: '#FFFFFF',
    fontSize: 24,
    fontWeight: '700',
    marginLeft: 12,
  },
  pointsSection: {
    marginBottom: 20,
  },
  pointsLabel: {
    color: 'rgba(255,255,255,0.8)',
    fontSize: 14,
  },
  pointsValue: {
    color: '#FFFFFF',
    fontSize: 36,
    fontWeight: '700',
  },
  progressSection: {},
  progressHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  progressText: {
    color: 'rgba(255,255,255,0.8)',
    fontSize: 13,
  },
  progressBar: {
    height: 8,
    backgroundColor: 'rgba(255,255,255,0.3)',
    borderRadius: 4,
  },
  progressFill: {
    height: '100%',
    backgroundColor: '#FFFFFF',
    borderRadius: 4,
  },
  benefitsCard: {
    marginBottom: 16,
  },
  sectionHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginLeft: 8,
  },
  benefitItem: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  benefitText: {
    fontSize: 14,
    marginLeft: 10,
  },
  discountRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginTop: 12,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: 'rgba(0,0,0,0.05)',
  },
  discountLabel: {
    fontSize: 14,
  },
  discountValue: {
    fontSize: 14,
    fontWeight: '600',
  },
  levelsCard: {
    marginBottom: 16,
  },
  levelItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 14,
  },
  levelItemLeft: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  levelIcon: {
    width: 36,
    height: 36,
    borderRadius: 18,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  levelNameRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  levelItemName: {
    fontSize: 15,
    fontWeight: '500',
  },
  currentBadge: {
    marginLeft: 8,
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
  },
  currentBadgeText: {
    color: '#FFFFFF',
    fontSize: 10,
    fontWeight: '600',
  },
  levelItemPoints: {
    fontSize: 12,
    marginTop: 2,
  },
  levelItemRight: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  levelDiscount: {
    fontSize: 14,
    fontWeight: '600',
    marginRight: 8,
  },
  earnCard: {
    marginBottom: 16,
  },
  earnItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  earnAction: {
    fontSize: 14,
  },
  earnPoints: {
    fontSize: 14,
    fontWeight: '500',
  },
});
