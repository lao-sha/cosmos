/**
 * 购买 DUST 首页
 * 显示当前价格、首购特惠、做市商列表
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { useTradingStore } from '@/stores/trading.store';
import {
  PriceDisplay,
  MakerCard,
  MakerOfflineWarning,
  TradingErrorBoundary,
} from '@/features/trading/components';
import { BottomNavBar } from '@/components/BottomNavBar';
import { PageHeader } from '@/components/PageHeader';
import type { Maker } from '@/stores/trading.store';

function BuyDustPageContent() {
  const router = useRouter();
  const {
    makers,
    loadingMakers,
    dustPrice,
    marketStats,
    isFirstPurchase,
    hasCompletedFirstPurchase,
    fetchMakers,
    fetchMarketStats,
    checkFirstPurchaseStatus,
    selectMaker,
  } = useTradingStore();

  const [showOfflineWarning, setShowOfflineWarning] = useState(false);
  const [pendingMaker, setPendingMaker] = useState<Maker | null>(null);

  useEffect(() => {
    // 初始化数据
    fetchMakers();
    fetchMarketStats();
    checkFirstPurchaseStatus();
  }, []);

  const handleFirstPurchase = () => {
    router.push('/wallet/buy-dust/first-purchase');
  };

  const handleSelectMaker = (makerId: number) => {
    const maker = makers.find(m => m.id === makerId);
    if (!maker) return;

    // 检查做市商是否离线
    if (maker.isOnline === false) {
      setPendingMaker(maker);
      setShowOfflineWarning(true);
      return;
    }

    // 正常流程
    proceedWithMaker(maker);
  };

  const proceedWithMaker = (maker: Maker) => {
    selectMaker(maker.id);
    if (isFirstPurchase && !hasCompletedFirstPurchase) {
      router.push('/wallet/buy-dust/first-purchase');
    } else {
      router.push('/wallet/buy-dust/order');
    }
  };

  const handleOfflineConfirm = () => {
    setShowOfflineWarning(false);
    if (pendingMaker) {
      proceedWithMaker(pendingMaker);
      setPendingMaker(null);
    }
  };

  const handleOfflineCancel = () => {
    setShowOfflineWarning(false);
    setPendingMaker(null);
  };

  // 计算在线做市商数量
  const onlineMakersCount = makers.filter(m => m.isOnline !== false).length;

  return (
    <View style={styles.wrapper}>
      {/* 页面头部 */}
      <PageHeader title="购买 DUST" />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 价格显示 */}
        <View style={styles.section}>
          <PriceDisplay
            price={marketStats?.weightedPrice || dustPrice || 0.10}
            priceChange24h={marketStats?.priceChange24h}
            label="💰 当前价格"
          />
        </View>

        {/* 首购特惠 */}
        {isFirstPurchase && !hasCompletedFirstPurchase && (
          <View style={styles.section}>
            <View style={styles.firstPurchaseCard}>
              <Text style={styles.firstPurchaseTitle}>🎁 首购特惠</Text>
              <Text style={styles.firstPurchaseDesc}>
                首次购买固定 10 USD
              </Text>
              <Text style={styles.firstPurchaseDesc}>
                享受新用户专属价格
              </Text>
              <TouchableOpacity
                style={styles.firstPurchaseButton}
                onPress={handleFirstPurchase}
              >
                <Text style={styles.firstPurchaseButtonText}>立即首购</Text>
              </TouchableOpacity>
            </View>
          </View>
        )}

        {/* 做市商列表 */}
        <View style={styles.section}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>
              {isFirstPurchase && !hasCompletedFirstPurchase
                ? '或选择做市商'
                : '选择做市商'}
            </Text>
            <Text style={styles.sectionSubtitle}>
              {onlineMakersCount} 位做市商在线
            </Text>
          </View>

          {loadingMakers ? (
            <View style={styles.loading}>
              <ActivityIndicator size="large" color="#B2955D" />
              <Text style={styles.loadingText}>加载中...</Text>
            </View>
          ) : makers.length === 0 ? (
            <View style={styles.empty}>
              <Text style={styles.emptyText}>暂无可用做市商</Text>
            </View>
          ) : (
            makers.map((maker) => (
              <MakerCard
                key={maker.id}
                maker={maker}
                onPress={() => handleSelectMaker(maker.id)}
              />
            ))
          )}
        </View>

        {/* 底部说明 */}
        <View style={styles.footer}>
          <Text style={styles.footerTitle}>💡 购买说明</Text>
          <Text style={styles.footerText}>• 首次购买固定 10 USD</Text>
          <Text style={styles.footerText}>• 普通订单 20-200 USD</Text>
          <Text style={styles.footerText}>• 支付方式：USDT (TRC20)</Text>
          <Text style={styles.footerText}>• 订单超时：30 分钟</Text>
        </View>
      </ScrollView>

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="profile" />

      {/* 做市商离线警告 */}
      {pendingMaker && (
        <MakerOfflineWarning
          visible={showOfflineWarning}
          maker={pendingMaker}
          onConfirm={handleOfflineConfirm}
          onCancel={handleOfflineCancel}
        />
      )}
    </View>
  );
}

export default function BuyDustPage() {
  return (
    <TradingErrorBoundary>
      <BuyDustPageContent />
    </TradingErrorBoundary>
  );
}

const styles = StyleSheet.create({
  wrapper: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  container: {
    flex: 1,
  },
  contentContainer: {
    paddingBottom: 20,
  },
  section: {
    padding: 16,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#000000',
  },
  sectionSubtitle: {
    fontSize: 14,
    color: '#666666',
  },
  firstPurchaseCard: {
    backgroundColor: '#FFF9F0',
    borderRadius: 12,
    padding: 20,
    borderWidth: 2,
    borderColor: '#B2955D',
  },
  firstPurchaseTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  firstPurchaseDesc: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 4,
  },
  firstPurchaseButton: {
    backgroundColor: '#B2955D',
    borderRadius: 8,
    paddingVertical: 12,
    alignItems: 'center',
    marginTop: 16,
  },
  firstPurchaseButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  loading: {
    alignItems: 'center',
    paddingVertical: 40,
  },
  loadingText: {
    fontSize: 14,
    color: '#666666',
    marginTop: 12,
  },
  empty: {
    alignItems: 'center',
    paddingVertical: 40,
  },
  emptyText: {
    fontSize: 14,
    color: '#999999',
  },
  footer: {
    padding: 16,
    paddingBottom: 32,
  },
  footerTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 12,
  },
  footerText: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 6,
  },
});
