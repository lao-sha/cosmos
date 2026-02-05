import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
  RefreshControl,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Star, Clock, CheckCircle, Circle } from 'lucide-react-native';
import { useColors } from '@/hooks/useColors';
import { useMakers, formatCos, formatPrice } from '@/hooks/useOtc';
import { Card } from '@/components/ui';
import { Colors } from '@/constants/colors';
import type { Maker } from '@/services/otc';

export default function OtcScreen() {
  const colors = useColors();
  const router = useRouter();
  const { data: makers, isLoading, refetch } = useMakers();

  const renderMaker = ({ item }: { item: Maker }) => (
    <TouchableOpacity
      activeOpacity={0.8}
      onPress={() => router.push(`/otc/buy?makerId=${item.id}`)}
    >
      <Card style={styles.makerCard}>
        <View style={styles.makerHeader}>
          <View style={styles.makerInfo}>
            <View style={styles.nameRow}>
              <View
                style={[
                  styles.avatar,
                  { backgroundColor: Colors.primary + '20' },
                ]}
              >
                <Text style={[styles.avatarText, { color: Colors.primary }]}>
                  {item.name.charAt(0)}
                </Text>
              </View>
              <View>
                <Text style={[styles.makerName, { color: colors.textPrimary }]}>
                  {item.name}
                </Text>
                <View style={styles.statsRow}>
                  <Text style={[styles.statText, { color: colors.textSecondary }]}>
                    {item.completedOrders} 单
                  </Text>
                  <Text style={[styles.statDot, { color: colors.textTertiary }]}>
                    •
                  </Text>
                  <Text style={[styles.statText, { color: Colors.success }]}>
                    {item.completionRate}%
                  </Text>
                </View>
              </View>
            </View>
            <View style={styles.onlineStatus}>
              {item.isOnline ? (
                <CheckCircle size={16} color={Colors.success} />
              ) : (
                <Circle size={16} color={colors.textTertiary} />
              )}
              <Text
                style={[
                  styles.onlineText,
                  { color: item.isOnline ? Colors.success : colors.textTertiary },
                ]}
              >
                {item.isOnline ? '在线' : '离线'}
              </Text>
            </View>
          </View>
        </View>

        <View style={styles.makerBody}>
          <View style={styles.priceSection}>
            <Text style={[styles.priceLabel, { color: colors.textSecondary }]}>
              价格
            </Text>
            <Text style={[styles.priceValue, { color: colors.textPrimary }]}>
              {formatPrice(item.price)}{' '}
              <Text style={styles.priceUnit}>USDT</Text>
            </Text>
          </View>

          <View style={styles.limitSection}>
            <Text style={[styles.limitLabel, { color: colors.textSecondary }]}>
              限额
            </Text>
            <Text style={[styles.limitValue, { color: colors.textPrimary }]}>
              {formatCos(item.minAmount)} - {formatCos(item.maxAmount)} COS
            </Text>
          </View>
        </View>

        <View style={styles.makerFooter}>
          <View style={styles.paymentMethods}>
            {item.paymentMethods.includes('bank') && (
              <View style={[styles.paymentTag, { backgroundColor: '#3B82F620' }]}>
                <Text style={[styles.paymentText, { color: '#3B82F6' }]}>
                  银行卡
                </Text>
              </View>
            )}
            {item.paymentMethods.includes('alipay') && (
              <View style={[styles.paymentTag, { backgroundColor: '#1677FF20' }]}>
                <Text style={[styles.paymentText, { color: '#1677FF' }]}>
                  支付宝
                </Text>
              </View>
            )}
            {item.paymentMethods.includes('wechat') && (
              <View style={[styles.paymentTag, { backgroundColor: '#07C16020' }]}>
                <Text style={[styles.paymentText, { color: '#07C160' }]}>
                  微信
                </Text>
              </View>
            )}
          </View>
          <Text style={[styles.availableText, { color: colors.textSecondary }]}>
            可用: {formatCos(item.availableCos)} COS
          </Text>
        </View>
      </Card>
    </TouchableOpacity>
  );

  return (
    <View style={[styles.container, { backgroundColor: colors.background }]}>
      <View style={styles.header}>
        <Text style={[styles.title, { color: colors.textPrimary }]}>
          购买 COS
        </Text>
        <TouchableOpacity
          style={styles.ordersButton}
          onPress={() => router.push('/otc/orders')}
        >
          <Clock size={20} color={Colors.primary} />
          <Text style={[styles.ordersText, { color: Colors.primary }]}>
            我的订单
          </Text>
        </TouchableOpacity>
      </View>

      <FlatList
        data={makers}
        renderItem={renderMaker}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.list}
        refreshControl={
          <RefreshControl refreshing={isLoading} onRefresh={refetch} />
        }
        ListEmptyComponent={
          <View style={styles.empty}>
            <Text style={[styles.emptyText, { color: colors.textSecondary }]}>
              暂无做市商
            </Text>
          </View>
        }
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 16,
    paddingBottom: 8,
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
  },
  ordersButton: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  ordersText: {
    fontSize: 14,
    fontWeight: '500',
  },
  list: {
    padding: 16,
    paddingTop: 8,
    gap: 12,
  },
  makerCard: {
    marginBottom: 0,
  },
  makerHeader: {
    marginBottom: 12,
  },
  makerInfo: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  avatar: {
    width: 40,
    height: 40,
    borderRadius: 20,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  avatarText: {
    fontSize: 18,
    fontWeight: '700',
  },
  makerName: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 2,
  },
  statsRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  statText: {
    fontSize: 13,
  },
  statDot: {
    marginHorizontal: 6,
  },
  onlineStatus: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  onlineText: {
    fontSize: 12,
  },
  makerBody: {
    flexDirection: 'row',
    marginBottom: 12,
  },
  priceSection: {
    flex: 1,
  },
  priceLabel: {
    fontSize: 12,
    marginBottom: 4,
  },
  priceValue: {
    fontSize: 20,
    fontWeight: '700',
  },
  priceUnit: {
    fontSize: 14,
    fontWeight: '400',
  },
  limitSection: {
    flex: 1,
    alignItems: 'flex-end',
  },
  limitLabel: {
    fontSize: 12,
    marginBottom: 4,
  },
  limitValue: {
    fontSize: 14,
  },
  makerFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  paymentMethods: {
    flexDirection: 'row',
    gap: 6,
  },
  paymentTag: {
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
  },
  paymentText: {
    fontSize: 12,
    fontWeight: '500',
  },
  availableText: {
    fontSize: 12,
  },
  empty: {
    alignItems: 'center',
    paddingTop: 60,
  },
  emptyText: {
    fontSize: 16,
  },
});
