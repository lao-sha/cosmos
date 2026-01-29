import { StyleSheet, Text, View } from 'react-native';

type MemberTier = 'free' | 'basic' | 'premium' | 'vip';

interface MembershipBadgeProps {
  tier: MemberTier;
  size?: 'small' | 'medium';
}

const TIER_CONFIG: Record<MemberTier, { label: string; color: string; bg: string }> = {
  free: { label: '免费', color: '#6b7280', bg: '#f3f4f6' },
  basic: { label: '基础', color: '#3b82f6', bg: '#dbeafe' },
  premium: { label: '高级', color: '#8b5cf6', bg: '#ede9fe' },
  vip: { label: 'VIP', color: '#f59e0b', bg: '#fef3c7' },
};

export function MembershipBadge({ tier, size = 'small' }: MembershipBadgeProps) {
  const config = TIER_CONFIG[tier];
  const isSmall = size === 'small';

  return (
    <View
      style={[
        styles.badge,
        { backgroundColor: config.bg },
        isSmall ? styles.small : styles.medium,
      ]}
    >
      <Text
        style={[
          styles.text,
          { color: config.color },
          isSmall ? styles.textSmall : styles.textMedium,
        ]}
      >
        {config.label}
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  badge: {
    borderRadius: 4,
    alignSelf: 'flex-start',
  },
  small: {
    paddingHorizontal: 6,
    paddingVertical: 2,
  },
  medium: {
    paddingHorizontal: 10,
    paddingVertical: 4,
  },
  text: {
    fontWeight: '600',
  },
  textSmall: {
    fontSize: 10,
  },
  textMedium: {
    fontSize: 12,
  },
});
