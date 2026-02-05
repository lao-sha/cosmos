import React from 'react';
import { View, Text, StyleSheet, ScrollView, TouchableOpacity } from 'react-native';
import { useRouter } from 'expo-router';
import { ArrowDownUp, Users, TrendingUp } from 'lucide-react-native';
import { useColors } from '@/hooks/useColors';
import { Card } from '@/components/ui';
import { Colors } from '@/constants/colors';

export default function TradeScreen() {
  const colors = useColors();
  const router = useRouter();

  const tradeOptions = [
    {
      id: 'otc',
      title: 'OTC 交易',
      subtitle: '与做市商直接交易 COS',
      icon: Users,
      color: Colors.trading.buy,
      route: '/otc',
    },
    {
      id: 'swap',
      title: 'Swap 兑换',
      subtitle: 'COS ↔ USDT 快速兑换',
      icon: ArrowDownUp,
      color: Colors.primary,
      route: '/swap',
    },
    {
      id: 'maker',
      title: '做市商',
      subtitle: '成为做市商，赚取差价',
      icon: TrendingUp,
      color: Colors.accent,
      route: '/maker',
    },
  ];

  return (
    <ScrollView
      style={[styles.container, { backgroundColor: colors.background }]}
      contentContainerStyle={styles.content}
    >
      <Text style={[styles.title, { color: colors.textPrimary }]}>交易</Text>
      <Text style={[styles.subtitle, { color: colors.textSecondary }]}>
        选择交易方式
      </Text>

      <View style={styles.options}>
        {tradeOptions.map((option) => (
          <TouchableOpacity
            key={option.id}
            onPress={() => router.push(option.route as any)}
            activeOpacity={0.8}
          >
            <Card style={styles.optionCard}>
              <View style={[styles.iconContainer, { backgroundColor: option.color + '20' }]}>
                <option.icon size={28} color={option.color} />
              </View>
              <View style={styles.optionInfo}>
                <Text style={[styles.optionTitle, { color: colors.textPrimary }]}>
                  {option.title}
                </Text>
                <Text style={[styles.optionSubtitle, { color: colors.textSecondary }]}>
                  {option.subtitle}
                </Text>
              </View>
            </Card>
          </TouchableOpacity>
        ))}
      </View>

      {/* Market Overview */}
      <Card style={styles.marketCard}>
        <Text style={[styles.sectionTitle, { color: colors.textPrimary }]}>
          市场行情
        </Text>
        <View style={styles.priceRow}>
          <Text style={[styles.priceLabel, { color: colors.textSecondary }]}>
            COS/USDT
          </Text>
          <View style={styles.priceInfo}>
            <Text style={[styles.priceValue, { color: colors.textPrimary }]}>
              0.0850
            </Text>
            <Text style={[styles.priceChange, { color: Colors.trading.buy }]}>
              +2.35%
            </Text>
          </View>
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
  },
  title: {
    fontSize: 28,
    fontWeight: '700',
    marginBottom: 4,
  },
  subtitle: {
    fontSize: 16,
    marginBottom: 24,
  },
  options: {
    gap: 12,
    marginBottom: 24,
  },
  optionCard: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  iconContainer: {
    width: 56,
    height: 56,
    borderRadius: 16,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 16,
  },
  optionInfo: {
    flex: 1,
  },
  optionTitle: {
    fontSize: 18,
    fontWeight: '600',
    marginBottom: 4,
  },
  optionSubtitle: {
    fontSize: 14,
  },
  marketCard: {},
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    marginBottom: 16,
  },
  priceRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  priceLabel: {
    fontSize: 16,
  },
  priceInfo: {
    alignItems: 'flex-end',
  },
  priceValue: {
    fontSize: 20,
    fontWeight: '700',
  },
  priceChange: {
    fontSize: 14,
    fontWeight: '500',
  },
});
