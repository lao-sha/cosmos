import { useActiveMakers, useFirstPurchaseStatus } from '@/src/hooks/useOtc';
import { useCosPrice } from '@/src/hooks/usePricing';
import { otcService, MakerInfo } from '@/src/services/otc';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
  ActivityIndicator,
  Pressable,
  RefreshControl,
  ScrollView,
  StyleSheet,
  Text,
  View
} from 'react-native';

export default function OtcScreen() {
  const router = useRouter();
  const { isLoggedIn } = useAuthStore();
  const { isConnected } = useChainStore();
  const { makers, loading, refresh } = useActiveMakers();
  const { canFirstPurchase } = useFirstPurchaseStatus();
  const { priceFormatted } = useCosPrice();
  const [refreshing, setRefreshing] = useState(false);

  const handleRefresh = async () => {
    setRefreshing(true);
    await refresh();
    setRefreshing(false);
  };

  const handleSelectMaker = (maker: MakerInfo) => {
    router.push({
      pathname: '/otc/create',
      params: { makerId: maker.makerId.toString() },
    });
  };

  if (!isConnected) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>OTC 交易</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>🔌</Text>
          <Text style={styles.emptyTitle}>未连接网络</Text>
          <Text style={styles.emptySubtitle}>请先连接区块链网络</Text>
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
        <Text style={styles.headerTitle}>OTC 交易</Text>
        <Pressable 
          style={styles.ordersButton}
          onPress={() => router.push('/otc/orders')}
        >
          <Text style={styles.ordersButtonText}>我的订单</Text>
        </Pressable>
      </View>

      <ScrollView
        style={styles.content}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} />
        }
      >
        <View style={styles.priceCard}>
          <Text style={styles.priceLabel}>当前 COS 价格</Text>
          <Text style={styles.priceValue}>{priceFormatted}</Text>
        </View>

        {canFirstPurchase && isLoggedIn && (
          <View style={styles.firstPurchaseCard}>
            <Text style={styles.firstPurchaseIcon}>🎁</Text>
            <View style={styles.firstPurchaseContent}>
              <Text style={styles.firstPurchaseTitle}>首购优惠</Text>
              <Text style={styles.firstPurchaseDesc}>
                新用户首次购买享受固定 $10 USD 价值的 COS，免押金
              </Text>
            </View>
          </View>
        )}

        <View style={styles.sectionHeader}>
          <Text style={styles.sectionTitle}>选择做市商</Text>
          <Text style={styles.sectionCount}>{makers.length} 位在线</Text>
        </View>

        {loading && makers.length === 0 ? (
          <View style={styles.loadingContainer}>
            <ActivityIndicator size="large" color="#6D28D9" />
            <Text style={styles.loadingText}>加载中...</Text>
          </View>
        ) : makers.length === 0 ? (
          <View style={styles.emptyMakers}>
            <Text style={styles.emptyMakersIcon}>😔</Text>
            <Text style={styles.emptyMakersText}>暂无在线做市商</Text>
          </View>
        ) : (
          makers.map((maker) => (
            <Pressable
              key={maker.makerId}
              style={({ pressed }) => [
                styles.makerCard,
                pressed && styles.makerCardPressed,
              ]}
              onPress={() => handleSelectMaker(maker)}
            >
              <View style={styles.makerHeader}>
                <View style={styles.makerInfo}>
                  <Text style={styles.makerName}>{maker.maskedFullName || `做市商 #${maker.makerId}`}</Text>
                  <Text style={styles.makerStats}>
                    已服务 {maker.usersServed} 人
                  </Text>
                </View>
                <View style={styles.makerBadge}>
                  <Text style={styles.makerBadgeText}>在线</Text>
                </View>
              </View>
              
              <View style={styles.makerDetails}>
                <View style={styles.detailItem}>
                  <Text style={styles.detailLabel}>买入溢价</Text>
                  <Text style={styles.detailValue}>
                    {(maker.buyPremiumBps / 100).toFixed(2)}%
                  </Text>
                </View>
                <View style={styles.detailItem}>
                  <Text style={styles.detailLabel}>卖出溢价</Text>
                  <Text style={styles.detailValue}>
                    {(maker.sellPremiumBps / 100).toFixed(2)}%
                  </Text>
                </View>
                <View style={styles.detailItem}>
                  <Text style={styles.detailLabel}>微信</Text>
                  <Text style={styles.detailValue}>{maker.wechatId || '-'}</Text>
                </View>
              </View>

              <View style={styles.makerFooter}>
                <Text style={styles.tronAddress} numberOfLines={1}>
                  TRON: {maker.tronAddress}
                </Text>
              </View>
            </Pressable>
          ))
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
    backgroundColor: '#6D28D9',
    paddingTop: 50,
    paddingBottom: 16,
    paddingHorizontal: 16,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  backButton: {
    padding: 4,
  },
  backText: {
    color: '#fff',
    fontSize: 18,
  },
  headerTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#fff',
  },
  headerRight: {
    width: 70,
  },
  ordersButton: {
    paddingVertical: 6,
    paddingHorizontal: 12,
    backgroundColor: 'rgba(255,255,255,0.2)',
    borderRadius: 16,
  },
  ordersButtonText: {
    color: '#fff',
    fontSize: 13,
  },
  content: {
    flex: 1,
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
  priceCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  priceLabel: {
    fontSize: 12,
    color: '#6b7280',
    marginBottom: 4,
  },
  priceValue: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#10b981',
  },
  firstPurchaseCard: {
    backgroundColor: '#fef3c7',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
    flexDirection: 'row',
    alignItems: 'center',
  },
  firstPurchaseIcon: {
    fontSize: 32,
    marginRight: 12,
  },
  firstPurchaseContent: {
    flex: 1,
  },
  firstPurchaseTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#92400e',
    marginBottom: 4,
  },
  firstPurchaseDesc: {
    fontSize: 13,
    color: '#92400e',
    lineHeight: 18,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginHorizontal: 16,
    marginTop: 20,
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
  },
  sectionCount: {
    fontSize: 13,
    color: '#6b7280',
  },
  loadingContainer: {
    alignItems: 'center',
    paddingVertical: 40,
  },
  loadingText: {
    marginTop: 12,
    fontSize: 14,
    color: '#6b7280',
  },
  emptyMakers: {
    alignItems: 'center',
    paddingVertical: 40,
  },
  emptyMakersIcon: {
    fontSize: 48,
    marginBottom: 12,
  },
  emptyMakersText: {
    fontSize: 14,
    color: '#6b7280',
  },
  makerCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginBottom: 12,
    padding: 16,
    borderRadius: 12,
  },
  makerCardPressed: {
    backgroundColor: '#f9fafb',
  },
  makerHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 12,
  },
  makerInfo: {
    flex: 1,
  },
  makerName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 4,
  },
  makerStats: {
    fontSize: 12,
    color: '#6b7280',
  },
  makerBadge: {
    backgroundColor: '#d1fae5',
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
  },
  makerBadgeText: {
    fontSize: 12,
    color: '#059669',
    fontWeight: '500',
  },
  makerDetails: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 12,
    borderTopWidth: 1,
    borderBottomWidth: 1,
    borderColor: '#f3f4f6',
  },
  detailItem: {
    alignItems: 'center',
    flex: 1,
  },
  detailLabel: {
    fontSize: 11,
    color: '#9ca3af',
    marginBottom: 4,
  },
  detailValue: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1f2937',
  },
  makerFooter: {
    marginTop: 12,
  },
  tronAddress: {
    fontSize: 12,
    color: '#6b7280',
    fontFamily: 'monospace',
  },
  bottomPadding: {
    height: 40,
  },
});
