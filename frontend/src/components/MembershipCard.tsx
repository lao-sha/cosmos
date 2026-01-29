import { Pressable, StyleSheet, Text, View } from 'react-native';

export type MemberTier = 'free' | 'basic' | 'premium' | 'vip';
export type SubscriptionDuration = 'monthly' | 'yearly';

interface TierInfo {
  name: string;
  price: { monthly: number; yearly: number };
  features: string[];
  color: string;
  recommended?: boolean;
}

const TIER_INFO: Record<MemberTier, TierInfo> = {
  free: {
    name: '免费版',
    price: { monthly: 0, yearly: 0 },
    features: ['基础八字排盘', '每日1次AI解读', '基础聊天功能'],
    color: '#6b7280',
  },
  basic: {
    name: '基础版',
    price: { monthly: 9.9, yearly: 99 },
    features: ['无限AI解读', '高级排盘功能', '优先客服支持', '每日签到2倍奖励'],
    color: '#3b82f6',
  },
  premium: {
    name: '高级版',
    price: { monthly: 29.9, yearly: 299 },
    features: ['所有基础版功能', '专属占卜师折扣', '合婚匹配功能', '每日签到3倍奖励'],
    color: '#8b5cf6',
    recommended: true,
  },
  vip: {
    name: 'VIP版',
    price: { monthly: 99.9, yearly: 999 },
    features: ['所有高级版功能', '专属1对1顾问', 'OTC交易手续费减免', '每日签到5倍奖励'],
    color: '#f59e0b',
  },
};

interface MembershipCardProps {
  tier: MemberTier;
  duration: SubscriptionDuration;
  currentTier?: MemberTier;
  onSelect?: (tier: MemberTier) => void;
}

export function MembershipCard({ tier, duration, currentTier, onSelect }: MembershipCardProps) {
  const info = TIER_INFO[tier];
  const price = info.price[duration];
  const isCurrent = tier === currentTier;
  const isUpgrade = currentTier && getTierLevel(tier) > getTierLevel(currentTier);

  return (
    <Pressable
      style={({ pressed }) => [
        styles.card,
        { borderColor: info.color },
        info.recommended && styles.recommended,
        isCurrent && styles.current,
        pressed && styles.pressed,
      ]}
      onPress={() => !isCurrent && onSelect?.(tier)}
      disabled={isCurrent}
    >
      {info.recommended && (
        <View style={[styles.badge, { backgroundColor: info.color }]}>
          <Text style={styles.badgeText}>推荐</Text>
        </View>
      )}

      <Text style={[styles.tierName, { color: info.color }]}>{info.name}</Text>

      <View style={styles.priceRow}>
        <Text style={styles.currency}>¥</Text>
        <Text style={styles.price}>{price}</Text>
        <Text style={styles.period}>/{duration === 'monthly' ? '月' : '年'}</Text>
      </View>

      {duration === 'yearly' && price > 0 && (
        <Text style={styles.savings}>
          省 ¥{(info.price.monthly * 12 - info.price.yearly).toFixed(0)}
        </Text>
      )}

      <View style={styles.divider} />

      <View style={styles.features}>
        {info.features.map((feature, index) => (
          <View key={index} style={styles.featureRow}>
            <Text style={styles.checkmark}>✓</Text>
            <Text style={styles.featureText}>{feature}</Text>
          </View>
        ))}
      </View>

      <View
        style={[
          styles.button,
          { backgroundColor: isCurrent ? '#e5e7eb' : info.color },
        ]}
      >
        <Text style={[styles.buttonText, isCurrent && styles.buttonTextDisabled]}>
          {isCurrent ? '当前套餐' : isUpgrade ? '升级' : '选择'}
        </Text>
      </View>
    </Pressable>
  );
}

function getTierLevel(tier: MemberTier): number {
  const levels: Record<MemberTier, number> = { free: 0, basic: 1, premium: 2, vip: 3 };
  return levels[tier];
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: '#fff',
    borderRadius: 16,
    borderWidth: 2,
    padding: 20,
    marginBottom: 16,
    position: 'relative',
  },
  recommended: {
    shadowColor: '#8b5cf6',
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.2,
    shadowRadius: 12,
    elevation: 8,
  },
  current: {
    opacity: 0.7,
  },
  pressed: {
    transform: [{ scale: 0.98 }],
  },
  badge: {
    position: 'absolute',
    top: -10,
    right: 16,
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 12,
  },
  badgeText: {
    color: '#fff',
    fontSize: 12,
    fontWeight: '600',
  },
  tierName: {
    fontSize: 20,
    fontWeight: '700',
    marginBottom: 8,
  },
  priceRow: {
    flexDirection: 'row',
    alignItems: 'baseline',
  },
  currency: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
  },
  price: {
    fontSize: 36,
    fontWeight: '700',
    color: '#1f2937',
  },
  period: {
    fontSize: 14,
    color: '#6b7280',
    marginLeft: 4,
  },
  savings: {
    fontSize: 13,
    color: '#22c55e',
    fontWeight: '500',
    marginTop: 4,
  },
  divider: {
    height: 1,
    backgroundColor: '#e5e7eb',
    marginVertical: 16,
  },
  features: {
    gap: 10,
    marginBottom: 16,
  },
  featureRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  checkmark: {
    color: '#22c55e',
    fontSize: 14,
    marginRight: 8,
  },
  featureText: {
    fontSize: 14,
    color: '#374151',
  },
  button: {
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
  },
  buttonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  buttonTextDisabled: {
    color: '#9ca3af',
  },
});
