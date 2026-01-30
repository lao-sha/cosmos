import { useSwapMakers } from '@/src/hooks/useSwap';
import { useCosPrice } from '@/src/hooks/usePricing';
import { swapService, MakerInfo } from '@/src/services/swap';
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

export default function SwapScreen() {
  const router = useRouter();
  const { isLoggedIn } = useAuthStore();
  const { isConnected } = useChainStore();
  const { makers, loading, refresh } = useSwapMakers();
  const { priceFormatted } = useCosPrice();
  const [refreshing, setRefreshing] = useState(false);

  const handleRefresh = async () => {
    setRefreshing(true);
    await refresh();
    setRefreshing(false);
  };

  const handleSelectMaker = (maker: MakerInfo) => {
    router.push({
      pathname: '/swap/create',
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
          <Text style={styles.headerTitle}>COS 兑换</Text>
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
        <Text style={styles.headerTitle}>COS 兑换</Text>
        <Pressable 
          style={styles.ordersButton}
          onPress={() => router.push('/swap/history')}
        >
          <Text style={styles.ordersButtonText}>兑换记录</Text>
        </Pressable>
      </View>

      <ScrollView
        style={styles.content}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} />
        }
      >
        <View style={styles.infoCard}>
          <View style={styles.infoHeader}>
            <Text style={styles.infoIcon}>💱</Text>
            <Text style={styles.infoTitle}>COS → USDT</Text>
          </View>
          <Text style={styles.infoDesc}>
            将您的 COS 代币兑换为 TRC20 USDT，由做市商提供服务
          </Text>
        </View>

        <View style={styles.priceCard}>
          <Text style={styles.priceLabel}>当前 COS 价格</Text>
          <Text style={styles.priceValue}>{priceFormatted}</Text>
        </View>

        <View style={styles.flowCard}>
          <Text style={styles.flowTitle}>兑换流程</Text>
          <View style={styles.flowSteps}>
            <View style={styles.flowStep}>
              <Text style={styles.flowStepNum}>1</Text>
              <Text style={styles.flowStepText}>选择做市商</Text>
            </View>
            <Text style={styles.flowArrow}>→</Text>
            <View style={styles.flowStep}>
              <Text style={styles.flowStepNum}>2</Text>
              <Text style={styles.flowStepText}>锁定 COS</Text>
            </View>
            <Text style={styles.flowArrow}>→</Text>
            <View style={styles.flowStep}>
              <Text style={styles.flowStepNum}>3</Text>
              <Text style={styles.flowStepText}>收取 USDT</Text>
            </View>
          </View>
        </View>

        <View style={styles.sectionHeader}>
          <Text style={styles.sectionTitle}>选择做市商</Text>
          <Text style={styles.sectionCount}>{makers.length} 位在线</Text>
        </View>

        {loading && makers.length === 0 ? (
          <View style={styles.loadingContainer}>
            <ActivityIndicator size="large" color="#10b981" />
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

        <View style={styles.noticeCard}>
          <Text style={styles.noticeTitle}>⚠️ 兑换须知</Text>
          <Text style={styles.noticeText}>
            1. 最小兑换金额：100 COS{'\n'}
            2. 做市商需在 24 小时内完成 USDT 转账{'\n'}
            3. 系统自动验证 TRC20 交易{'\n'}
            4. 如有问题可发起举报
          </Text>
        </View>

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
    backgroundColor: '#10b981',
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
  infoCard: {
    backgroundColor: '#ecfdf5',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  infoHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8,
  },
  infoIcon: {
    fontSize: 24,
    marginRight: 8,
  },
  infoTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#065f46',
  },
  infoDesc: {
    fontSize: 14,
    color: '#047857',
    lineHeight: 20,
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
  flowCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  flowTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 12,
    textAlign: 'center',
  },
  flowSteps: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
  },
  flowStep: {
    alignItems: 'center',
  },
  flowStepNum: {
    width: 24,
    height: 24,
    borderRadius: 12,
    backgroundColor: '#10b981',
    color: '#fff',
    textAlign: 'center',
    lineHeight: 24,
    fontSize: 12,
    fontWeight: '600',
    marginBottom: 4,
  },
  flowStepText: {
    fontSize: 12,
    color: '#6b7280',
  },
  flowArrow: {
    fontSize: 16,
    color: '#9ca3af',
    marginHorizontal: 12,
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
    justifyContent: 'space-around',
    paddingVertical: 12,
    borderTopWidth: 1,
    borderBottomWidth: 1,
    borderColor: '#f3f4f6',
  },
  detailItem: {
    alignItems: 'center',
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
  noticeCard: {
    backgroundColor: '#fef3c7',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  noticeTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#92400e',
    marginBottom: 8,
  },
  noticeText: {
    fontSize: 13,
    color: '#92400e',
    lineHeight: 20,
  },
  bottomPadding: {
    height: 40,
  },
});
