import { useLocalSearchParams, useRouter } from 'expo-router';
import { Package, MapPin, Coins, ChevronRight } from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  TextInput,
  Alert,
  Switch,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import { sharemallTxService } from '@/src/services/sharemall-tx';
import type { Product, Shop, ShopTokenConfig } from '@/src/types/sharemall';
import { useWalletStore } from '@/src/stores/wallet';

export default function CheckoutScreen() {
  const { productId, quantity: qtyParam } = useLocalSearchParams<{
    productId: string;
    quantity: string;
  }>();
  const router = useRouter();
  const { currentAccount } = useWalletStore();

  const [product, setProduct] = useState<Product | null>(null);
  const [shop, setShop] = useState<Shop | null>(null);
  const [tokenConfig, setTokenConfig] = useState<ShopTokenConfig | null>(null);
  const [tokenBalance, setTokenBalance] = useState('0');
  const [shippingCid, setShippingCid] = useState('');
  const [usePoints, setUsePoints] = useState(false);
  const [pointsAmount, setPointsAmount] = useState('');
  const [referrer, setReferrer] = useState('');
  const [loading, setLoading] = useState(false);

  const quantity = parseInt(qtyParam || '1', 10);

  const loadData = async () => {
    if (!productId) return;
    try {
      const productData = await sharemallService.getProduct(parseInt(productId, 10));
      setProduct(productData);

      if (productData) {
        const [shopData, config] = await Promise.all([
          sharemallService.getShop(productData.shopId),
          sharemallService.getTokenConfig(productData.shopId),
        ]);
        setShop(shopData);
        setTokenConfig(config);

        if (currentAccount && config?.enabled) {
          const balance = await sharemallService.getTokenBalance(
            productData.shopId,
            currentAccount.address
          );
          setTokenBalance(balance);
        }
      }
    } catch (error) {
      console.error('Failed to load checkout data:', error);
    }
  };

  useEffect(() => {
    loadData();
  }, [productId, currentAccount]);

  const formatPrice = (price: string) => {
    const num = BigInt(price);
    return (Number(num) / 1e10).toFixed(2);
  };

  const formatBalance = (balance: string) => {
    const num = BigInt(balance);
    return (Number(num) / 1e10).toFixed(2);
  };

  const calculateTotal = () => {
    if (!product) return { subtotal: '0', discount: '0', total: '0' };

    const subtotal = BigInt(product.price) * BigInt(quantity);
    let discount = BigInt(0);

    if (usePoints && pointsAmount && tokenConfig) {
      const points = BigInt(Math.floor(parseFloat(pointsAmount) * 1e10));
      const maxDiscount = (subtotal * BigInt(tokenConfig.exchangeRate)) / BigInt(10000);
      discount = points > maxDiscount ? maxDiscount : points;
    }

    const total = subtotal - discount;

    return {
      subtotal: subtotal.toString(),
      discount: discount.toString(),
      total: total.toString(),
    };
  };

  const handleSubmit = async () => {
    if (!currentAccount) {
      Alert.alert('提示', '请先登录钱包');
      return;
    }
    if (!product) return;

    if (product.category === 'Physical' && !shippingCid.trim()) {
      Alert.alert('提示', '请填写收货地址');
      return;
    }

    setLoading(true);
    try {
      const pointsValue = usePoints && pointsAmount
        ? (parseFloat(pointsAmount) * 1e10).toString()
        : '0';

      await sharemallTxService.placeOrder(
        product.id,
        quantity,
        shippingCid.trim() || undefined,
        usePoints,
        pointsValue,
        referrer.trim() || undefined
      );

      Alert.alert('成功', '订单已提交', [
        { text: '查看订单', onPress: () => router.replace('/mall/orders' as any) },
      ]);
    } catch (error: any) {
      Alert.alert('错误', error.message || '下单失败');
    } finally {
      setLoading(false);
    }
  };

  if (!product || !shop) {
    return (
      <View style={styles.loadingContainer}>
        <Text>加载中...</Text>
      </View>
    );
  }

  const { subtotal, discount, total } = calculateTotal();

  return (
    <View style={styles.container}>
      <ScrollView style={styles.scrollView}>
        {/* 商品信息 */}
        <View style={styles.section}>
          <View style={styles.productCard}>
            <View style={styles.productImage}>
              <Package size={32} color="#ccc" />
            </View>
            <View style={styles.productInfo}>
              <Text style={styles.productName}>商品 #{product.id}</Text>
              <Text style={styles.shopName}>{shop.name}</Text>
              <View style={styles.priceRow}>
                <Text style={styles.productPrice}>¥{formatPrice(product.price)}</Text>
                <Text style={styles.quantity}>x{quantity}</Text>
              </View>
            </View>
          </View>
        </View>

        {/* 收货地址 (实物商品) */}
        {product.category === 'Physical' && (
          <View style={styles.section}>
            <View style={styles.sectionHeader}>
              <MapPin size={18} color="#666" />
              <Text style={styles.sectionTitle}>收货地址</Text>
            </View>
            <TextInput
              style={styles.addressInput}
              placeholder="输入收货地址信息的 IPFS CID"
              value={shippingCid}
              onChangeText={setShippingCid}
              multiline
            />
          </View>
        )}

        {/* 积分抵扣 */}
        {tokenConfig?.enabled && (
          <View style={styles.section}>
            <View style={styles.sectionHeader}>
              <Coins size={18} color="#666" />
              <Text style={styles.sectionTitle}>积分抵扣</Text>
            </View>
            <View style={styles.pointsInfo}>
              <Text style={styles.pointsBalance}>
                可用积分: {formatBalance(tokenBalance)}
              </Text>
              <Switch value={usePoints} onValueChange={setUsePoints} />
            </View>
            {usePoints && (
              <TextInput
                style={styles.input}
                placeholder="输入使用的积分数量"
                keyboardType="decimal-pad"
                value={pointsAmount}
                onChangeText={setPointsAmount}
              />
            )}
          </View>
        )}

        {/* 推荐人 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>推荐人 (可选)</Text>
          <TextInput
            style={styles.input}
            placeholder="输入推荐人地址"
            value={referrer}
            onChangeText={setReferrer}
          />
          <Text style={styles.hint}>填写推荐人可获得额外积分奖励</Text>
        </View>

        {/* 订单金额 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>订单金额</Text>
          <View style={styles.amountRow}>
            <Text style={styles.amountLabel}>商品小计</Text>
            <Text style={styles.amountValue}>¥{formatPrice(subtotal)}</Text>
          </View>
          {BigInt(discount) > 0 && (
            <View style={styles.amountRow}>
              <Text style={styles.amountLabel}>积分抵扣</Text>
              <Text style={[styles.amountValue, styles.discountValue]}>
                -¥{formatPrice(discount)}
              </Text>
            </View>
          )}
          <View style={[styles.amountRow, styles.totalRow]}>
            <Text style={styles.totalLabel}>应付金额</Text>
            <Text style={styles.totalValue}>¥{formatPrice(total)}</Text>
          </View>
        </View>
      </ScrollView>

      {/* 底部提交 */}
      <View style={styles.bottomBar}>
        <View style={styles.totalInfo}>
          <Text style={styles.totalText}>合计:</Text>
          <Text style={styles.totalPrice}>¥{formatPrice(total)}</Text>
        </View>
        <TouchableOpacity
          style={[styles.submitBtn, loading && styles.submitBtnDisabled]}
          onPress={handleSubmit}
          disabled={loading}
        >
          <Text style={styles.submitBtnText}>
            {loading ? '提交中...' : '提交订单'}
          </Text>
        </TouchableOpacity>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  loadingContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  scrollView: {
    flex: 1,
  },
  section: {
    backgroundColor: '#fff',
    marginHorizontal: 12,
    marginTop: 12,
    borderRadius: 12,
    padding: 16,
  },
  sectionHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 14,
    fontWeight: '500',
    color: '#333',
    marginLeft: 6,
  },
  productCard: {
    flexDirection: 'row',
  },
  productImage: {
    width: 80,
    height: 80,
    borderRadius: 8,
    backgroundColor: '#f5f5f5',
    alignItems: 'center',
    justifyContent: 'center',
  },
  productInfo: {
    flex: 1,
    marginLeft: 12,
    justifyContent: 'center',
  },
  productName: {
    fontSize: 14,
    color: '#333',
    marginBottom: 4,
  },
  shopName: {
    fontSize: 12,
    color: '#999',
    marginBottom: 8,
  },
  priceRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  productPrice: {
    fontSize: 16,
    fontWeight: '600',
    color: '#E53935',
  },
  quantity: {
    fontSize: 14,
    color: '#666',
  },
  addressInput: {
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 12,
    fontSize: 14,
    minHeight: 60,
    textAlignVertical: 'top',
  },
  pointsInfo: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  pointsBalance: {
    fontSize: 13,
    color: '#666',
  },
  input: {
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 12,
    fontSize: 14,
  },
  hint: {
    fontSize: 12,
    color: '#999',
    marginTop: 6,
  },
  amountRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 8,
  },
  amountLabel: {
    fontSize: 13,
    color: '#666',
  },
  amountValue: {
    fontSize: 13,
    color: '#333',
  },
  discountValue: {
    color: '#388E3C',
  },
  totalRow: {
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
    marginTop: 8,
    paddingTop: 12,
  },
  totalLabel: {
    fontSize: 14,
    fontWeight: '500',
    color: '#333',
  },
  totalValue: {
    fontSize: 18,
    fontWeight: '700',
    color: '#E53935',
  },
  bottomBar: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    paddingVertical: 12,
    paddingHorizontal: 16,
    borderTopWidth: 1,
    borderTopColor: '#eee',
    alignItems: 'center',
  },
  totalInfo: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
  },
  totalText: {
    fontSize: 14,
    color: '#333',
  },
  totalPrice: {
    fontSize: 20,
    fontWeight: '700',
    color: '#E53935',
    marginLeft: 4,
  },
  submitBtn: {
    backgroundColor: '#E53935',
    borderRadius: 20,
    paddingHorizontal: 32,
    paddingVertical: 12,
  },
  submitBtnDisabled: {
    opacity: 0.6,
  },
  submitBtnText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
