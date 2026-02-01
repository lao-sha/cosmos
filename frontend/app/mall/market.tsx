import { useLocalSearchParams, useRouter } from 'expo-router';
import { TrendingUp, TrendingDown, ArrowUpDown, RefreshCw } from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
  TextInput,
  Alert,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import { sharemallTxService } from '@/src/services/sharemall-tx';
import type { OrderBookDepth, MarketSummary, Shop, PriceLevel } from '@/src/types/sharemall';
import { useWalletStore } from '@/src/stores/wallet';

type OrderType = 'limit' | 'market';
type OrderSide = 'buy' | 'sell';

export default function MarketScreen() {
  const { shopId: shopIdParam } = useLocalSearchParams<{ shopId?: string }>();
  const router = useRouter();
  const { currentAccount } = useWalletStore();

  const [shops, setShops] = useState<Shop[]>([]);
  const [selectedShopId, setSelectedShopId] = useState<number | null>(
    shopIdParam ? parseInt(shopIdParam, 10) : null
  );
  const [orderBook, setOrderBook] = useState<OrderBookDepth>({ asks: [], bids: [] });
  const [marketSummary, setMarketSummary] = useState<MarketSummary | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  const [orderType, setOrderType] = useState<OrderType>('limit');
  const [orderSide, setOrderSide] = useState<OrderSide>('buy');
  const [price, setPrice] = useState('');
  const [amount, setAmount] = useState('');
  const [loading, setLoading] = useState(false);

  const loadShops = async () => {
    try {
      const list = await sharemallService.getShopList(50);
      // 只显示启用代币的店铺
      const tokenShops = [];
      for (const shop of list) {
        const config = await sharemallService.getTokenConfig(shop.id);
        if (config?.enabled) {
          tokenShops.push(shop);
        }
      }
      setShops(tokenShops);
      if (tokenShops.length > 0 && !selectedShopId) {
        setSelectedShopId(tokenShops[0].id);
      }
    } catch (error) {
      console.error('Failed to load shops:', error);
    }
  };

  const loadMarketData = async () => {
    if (!selectedShopId) return;
    try {
      const [book, summary] = await Promise.all([
        sharemallService.getOrderBookDepth(selectedShopId, 10),
        sharemallService.getMarketSummary(selectedShopId),
      ]);
      setOrderBook(book);
      setMarketSummary(summary);
    } catch (error) {
      console.error('Failed to load market data:', error);
    }
  };

  useEffect(() => {
    loadShops();
  }, []);

  useEffect(() => {
    if (selectedShopId) {
      loadMarketData();
    }
  }, [selectedShopId]);

  const onRefresh = async () => {
    setRefreshing(true);
    await loadMarketData();
    setRefreshing(false);
  };

  const formatPrice = (priceStr: string) => {
    const num = BigInt(priceStr);
    return (Number(num) / 1e10).toFixed(4);
  };

  const formatAmount = (amountStr: string) => {
    const num = BigInt(amountStr);
    return (Number(num) / 1e10).toFixed(2);
  };

  const handleSubmitOrder = async () => {
    if (!currentAccount) {
      Alert.alert('提示', '请先登录钱包');
      return;
    }
    if (!selectedShopId || !price || !amount) {
      Alert.alert('提示', '请填写完整信息');
      return;
    }

    setLoading(true);
    try {
      const priceValue = (parseFloat(price) * 1e10).toString();
      const amountValue = (parseFloat(amount) * 1e10).toString();

      if (orderType === 'limit') {
        if (orderSide === 'buy') {
          await sharemallTxService.placeBuyOrder(selectedShopId, amountValue, priceValue);
        } else {
          await sharemallTxService.placeSellOrder(selectedShopId, amountValue, priceValue);
        }
      } else {
        if (orderSide === 'buy') {
          await sharemallTxService.marketBuy(selectedShopId, priceValue, amountValue);
        } else {
          await sharemallTxService.marketSell(selectedShopId, amountValue, priceValue);
        }
      }

      Alert.alert('成功', '订单已提交');
      setPrice('');
      setAmount('');
      loadMarketData();
    } catch (error: any) {
      Alert.alert('错误', error.message || '订单提交失败');
    } finally {
      setLoading(false);
    }
  };

  const selectPrice = (priceLevel: PriceLevel) => {
    setPrice(formatPrice(priceLevel.price));
  };

  const selectedShop = shops.find((s) => s.id === selectedShopId);

  return (
    <ScrollView
      style={styles.container}
      refreshControl={<RefreshControl refreshing={refreshing} onRefresh={onRefresh} />}
    >
      {/* 店铺选择 */}
      <View style={styles.shopSelector}>
        <Text style={styles.selectorLabel}>选择店铺代币:</Text>
        <ScrollView horizontal showsHorizontalScrollIndicator={false}>
          {shops.map((shop) => (
            <TouchableOpacity
              key={shop.id}
              style={[
                styles.shopChip,
                selectedShopId === shop.id && styles.shopChipActive,
              ]}
              onPress={() => setSelectedShopId(shop.id)}
            >
              <Text
                style={[
                  styles.shopChipText,
                  selectedShopId === shop.id && styles.shopChipTextActive,
                ]}
              >
                {shop.name}
              </Text>
            </TouchableOpacity>
          ))}
        </ScrollView>
      </View>

      {selectedShopId && (
        <>
          {/* 市场概览 */}
          <View style={styles.summaryCard}>
            <View style={styles.summaryHeader}>
              <Text style={styles.tokenName}>{selectedShop?.name || '代币'}/COS</Text>
              <TouchableOpacity onPress={loadMarketData}>
                <RefreshCw size={18} color="#666" />
              </TouchableOpacity>
            </View>
            {marketSummary ? (
              <View style={styles.summaryContent}>
                <View style={styles.priceRow}>
                  <Text style={styles.lastPrice}>
                    {formatPrice(marketSummary.lastPrice)}
                  </Text>
                  <View
                    style={[
                      styles.changeTag,
                      {
                        backgroundColor:
                          BigInt(marketSummary.priceChange24h) >= 0
                            ? '#E8F5E9'
                            : '#FFEBEE',
                      },
                    ]}
                  >
                    {BigInt(marketSummary.priceChange24h) >= 0 ? (
                      <TrendingUp size={14} color="#388E3C" />
                    ) : (
                      <TrendingDown size={14} color="#D32F2F" />
                    )}
                    <Text
                      style={[
                        styles.changeText,
                        {
                          color:
                            BigInt(marketSummary.priceChange24h) >= 0
                              ? '#388E3C'
                              : '#D32F2F',
                        },
                      ]}
                    >
                      {formatPrice(marketSummary.priceChange24h)}%
                    </Text>
                  </View>
                </View>
                <View style={styles.statsRow}>
                  <View style={styles.statItem}>
                    <Text style={styles.statLabel}>24h 最高</Text>
                    <Text style={styles.statValue}>
                      {formatPrice(marketSummary.high24h)}
                    </Text>
                  </View>
                  <View style={styles.statItem}>
                    <Text style={styles.statLabel}>24h 最低</Text>
                    <Text style={styles.statValue}>
                      {formatPrice(marketSummary.low24h)}
                    </Text>
                  </View>
                  <View style={styles.statItem}>
                    <Text style={styles.statLabel}>24h 成交量</Text>
                    <Text style={styles.statValue}>
                      {formatAmount(marketSummary.volume24h)}
                    </Text>
                  </View>
                </View>
              </View>
            ) : (
              <Text style={styles.noData}>暂无市场数据</Text>
            )}
          </View>

          {/* 订单簿 */}
          <View style={styles.orderBookCard}>
            <Text style={styles.cardTitle}>订单簿</Text>
            <View style={styles.orderBookHeader}>
              <Text style={styles.obHeaderText}>价格 (COS)</Text>
              <Text style={styles.obHeaderText}>数量</Text>
              <Text style={styles.obHeaderText}>订单数</Text>
            </View>

            {/* 卖单 (红色) */}
            {orderBook.asks.slice().reverse().map((ask, index) => (
              <TouchableOpacity
                key={`ask-${index}`}
                style={styles.orderBookRow}
                onPress={() => selectPrice(ask)}
              >
                <Text style={[styles.obPrice, styles.askPrice]}>
                  {formatPrice(ask.price)}
                </Text>
                <Text style={styles.obAmount}>{formatAmount(ask.totalAmount)}</Text>
                <Text style={styles.obCount}>{ask.orderCount}</Text>
              </TouchableOpacity>
            ))}

            {orderBook.asks.length === 0 && orderBook.bids.length === 0 && (
              <Text style={styles.noOrders}>暂无挂单</Text>
            )}

            {/* 买单 (绿色) */}
            {orderBook.bids.map((bid, index) => (
              <TouchableOpacity
                key={`bid-${index}`}
                style={styles.orderBookRow}
                onPress={() => selectPrice(bid)}
              >
                <Text style={[styles.obPrice, styles.bidPrice]}>
                  {formatPrice(bid.price)}
                </Text>
                <Text style={styles.obAmount}>{formatAmount(bid.totalAmount)}</Text>
                <Text style={styles.obCount}>{bid.orderCount}</Text>
              </TouchableOpacity>
            ))}
          </View>

          {/* 下单表单 */}
          <View style={styles.orderForm}>
            <Text style={styles.cardTitle}>下单</Text>

            {/* 订单类型 */}
            <View style={styles.typeSelector}>
              <TouchableOpacity
                style={[styles.typeBtn, orderType === 'limit' && styles.typeBtnActive]}
                onPress={() => setOrderType('limit')}
              >
                <Text
                  style={[
                    styles.typeBtnText,
                    orderType === 'limit' && styles.typeBtnTextActive,
                  ]}
                >
                  限价单
                </Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={[styles.typeBtn, orderType === 'market' && styles.typeBtnActive]}
                onPress={() => setOrderType('market')}
              >
                <Text
                  style={[
                    styles.typeBtnText,
                    orderType === 'market' && styles.typeBtnTextActive,
                  ]}
                >
                  市价单
                </Text>
              </TouchableOpacity>
            </View>

            {/* 买卖方向 */}
            <View style={styles.sideSelector}>
              <TouchableOpacity
                style={[
                  styles.sideBtn,
                  styles.buyBtn,
                  orderSide === 'buy' && styles.buyBtnActive,
                ]}
                onPress={() => setOrderSide('buy')}
              >
                <Text
                  style={[
                    styles.sideBtnText,
                    orderSide === 'buy' && styles.buyBtnTextActive,
                  ]}
                >
                  买入
                </Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={[
                  styles.sideBtn,
                  styles.sellBtn,
                  orderSide === 'sell' && styles.sellBtnActive,
                ]}
                onPress={() => setOrderSide('sell')}
              >
                <Text
                  style={[
                    styles.sideBtnText,
                    orderSide === 'sell' && styles.sellBtnTextActive,
                  ]}
                >
                  卖出
                </Text>
              </TouchableOpacity>
            </View>

            {/* 价格输入 */}
            {orderType === 'limit' && (
              <View style={styles.inputGroup}>
                <Text style={styles.inputLabel}>价格 (COS)</Text>
                <TextInput
                  style={styles.input}
                  placeholder="输入价格"
                  keyboardType="decimal-pad"
                  value={price}
                  onChangeText={setPrice}
                />
              </View>
            )}

            {/* 数量输入 */}
            <View style={styles.inputGroup}>
              <Text style={styles.inputLabel}>
                {orderType === 'market' && orderSide === 'buy'
                  ? '最大花费 (COS)'
                  : '数量'}
              </Text>
              <TextInput
                style={styles.input}
                placeholder={
                  orderType === 'market' && orderSide === 'buy'
                    ? '输入最大花费'
                    : '输入数量'
                }
                keyboardType="decimal-pad"
                value={amount}
                onChangeText={setAmount}
              />
            </View>

            {/* 提交按钮 */}
            <TouchableOpacity
              style={[
                styles.submitBtn,
                orderSide === 'buy' ? styles.submitBuyBtn : styles.submitSellBtn,
                loading && styles.submitBtnDisabled,
              ]}
              onPress={handleSubmitOrder}
              disabled={loading}
            >
              <Text style={styles.submitBtnText}>
                {loading ? '提交中...' : orderSide === 'buy' ? '买入' : '卖出'}
              </Text>
            </TouchableOpacity>
          </View>
        </>
      )}

      {shops.length === 0 && (
        <View style={styles.emptyContainer}>
          <ArrowUpDown size={48} color="#ccc" />
          <Text style={styles.emptyText}>暂无可交易的店铺代币</Text>
        </View>
      )}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  shopSelector: {
    backgroundColor: '#fff',
    padding: 16,
  },
  selectorLabel: {
    fontSize: 14,
    color: '#666',
    marginBottom: 12,
  },
  shopChip: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 16,
    backgroundColor: '#f0f0f0',
    marginRight: 8,
  },
  shopChipActive: {
    backgroundColor: '#1976D2',
  },
  shopChipText: {
    fontSize: 13,
    color: '#666',
  },
  shopChipTextActive: {
    color: '#fff',
  },
  summaryCard: {
    backgroundColor: '#fff',
    marginTop: 8,
    padding: 16,
  },
  summaryHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  tokenName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  summaryContent: {},
  priceRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  lastPrice: {
    fontSize: 28,
    fontWeight: '700',
    color: '#333',
  },
  changeTag: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
    marginLeft: 12,
  },
  changeText: {
    fontSize: 13,
    marginLeft: 4,
  },
  statsRow: {
    flexDirection: 'row',
  },
  statItem: {
    flex: 1,
  },
  statLabel: {
    fontSize: 11,
    color: '#999',
    marginBottom: 4,
  },
  statValue: {
    fontSize: 13,
    color: '#333',
  },
  noData: {
    fontSize: 14,
    color: '#999',
    textAlign: 'center',
    paddingVertical: 20,
  },
  orderBookCard: {
    backgroundColor: '#fff',
    marginTop: 8,
    padding: 16,
  },
  cardTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginBottom: 12,
  },
  orderBookHeader: {
    flexDirection: 'row',
    paddingVertical: 8,
    borderBottomWidth: 1,
    borderBottomColor: '#eee',
  },
  obHeaderText: {
    flex: 1,
    fontSize: 11,
    color: '#999',
    textAlign: 'center',
  },
  orderBookRow: {
    flexDirection: 'row',
    paddingVertical: 8,
  },
  obPrice: {
    flex: 1,
    fontSize: 13,
    textAlign: 'center',
  },
  askPrice: {
    color: '#D32F2F',
  },
  bidPrice: {
    color: '#388E3C',
  },
  obAmount: {
    flex: 1,
    fontSize: 13,
    color: '#333',
    textAlign: 'center',
  },
  obCount: {
    flex: 1,
    fontSize: 13,
    color: '#999',
    textAlign: 'center',
  },
  noOrders: {
    fontSize: 13,
    color: '#999',
    textAlign: 'center',
    paddingVertical: 20,
  },
  orderForm: {
    backgroundColor: '#fff',
    marginTop: 8,
    padding: 16,
    marginBottom: 20,
  },
  typeSelector: {
    flexDirection: 'row',
    marginBottom: 12,
  },
  typeBtn: {
    flex: 1,
    paddingVertical: 10,
    alignItems: 'center',
    backgroundColor: '#f5f5f5',
    marginRight: 8,
    borderRadius: 8,
  },
  typeBtnActive: {
    backgroundColor: '#E3F2FD',
  },
  typeBtnText: {
    fontSize: 13,
    color: '#666',
  },
  typeBtnTextActive: {
    color: '#1976D2',
    fontWeight: '500',
  },
  sideSelector: {
    flexDirection: 'row',
    marginBottom: 16,
  },
  sideBtn: {
    flex: 1,
    paddingVertical: 12,
    alignItems: 'center',
    borderRadius: 8,
    marginRight: 8,
  },
  buyBtn: {
    backgroundColor: '#E8F5E9',
  },
  buyBtnActive: {
    backgroundColor: '#388E3C',
  },
  sellBtn: {
    backgroundColor: '#FFEBEE',
    marginRight: 0,
  },
  sellBtnActive: {
    backgroundColor: '#D32F2F',
  },
  sideBtnText: {
    fontSize: 14,
    fontWeight: '500',
  },
  buyBtnTextActive: {
    color: '#fff',
  },
  sellBtnTextActive: {
    color: '#fff',
  },
  inputGroup: {
    marginBottom: 12,
  },
  inputLabel: {
    fontSize: 12,
    color: '#666',
    marginBottom: 6,
  },
  input: {
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
    fontSize: 14,
  },
  submitBtn: {
    paddingVertical: 14,
    borderRadius: 8,
    alignItems: 'center',
    marginTop: 8,
  },
  submitBuyBtn: {
    backgroundColor: '#388E3C',
  },
  submitSellBtn: {
    backgroundColor: '#D32F2F',
  },
  submitBtnDisabled: {
    opacity: 0.6,
  },
  submitBtnText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  emptyContainer: {
    alignItems: 'center',
    paddingVertical: 60,
  },
  emptyText: {
    marginTop: 12,
    fontSize: 14,
    color: '#999',
  },
});
