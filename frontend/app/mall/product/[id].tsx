import { useLocalSearchParams, useRouter } from 'expo-router';
import { Store, Star, ShoppingCart, Minus, Plus, Heart } from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  Image,
  Alert,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import { sharemallTxService } from '@/src/services/sharemall-tx';
import type { Product, Shop, Review } from '@/src/types/sharemall';
import { useWalletStore } from '@/src/stores/wallet';

export default function ProductDetailScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const router = useRouter();
  const { currentAccount } = useWalletStore();
  
  const [product, setProduct] = useState<Product | null>(null);
  const [shop, setShop] = useState<Shop | null>(null);
  const [reviews, setReviews] = useState<Review[]>([]);
  const [quantity, setQuantity] = useState(1);
  const [loading, setLoading] = useState(false);

  const productId = parseInt(id || '0', 10);

  const loadData = async () => {
    if (!productId) return;
    try {
      const productData = await sharemallService.getProduct(productId);
      setProduct(productData);
      
      if (productData) {
        const [shopData, reviewList] = await Promise.all([
          sharemallService.getShop(productData.shopId),
          sharemallService.getProductReviews(productId),
        ]);
        setShop(shopData);
        setReviews(reviewList);
      }
    } catch (error) {
      console.error('Failed to load product:', error);
    }
  };

  useEffect(() => {
    loadData();
  }, [productId]);

  const formatPrice = (price: string) => {
    const num = BigInt(price);
    return (Number(num) / 1e10).toFixed(2);
  };

  const handleBuyNow = async () => {
    if (!currentAccount) {
      Alert.alert('提示', '请先登录钱包');
      return;
    }
    if (!product) return;

    router.push({
      pathname: '/mall/checkout' as any,
      params: {
        productId: product.id,
        quantity,
      },
    });
  };

  const handlePlaceOrder = async () => {
    if (!currentAccount) {
      Alert.alert('提示', '请先登录钱包');
      return;
    }
    if (!product) return;

    setLoading(true);
    try {
      await sharemallTxService.placeOrder(product.id, quantity);
      Alert.alert('成功', '下单成功', [
        { text: '查看订单', onPress: () => router.push('/mall/orders' as any) },
        { text: '继续购物', style: 'cancel' },
      ]);
    } catch (error: any) {
      Alert.alert('错误', error.message || '下单失败');
    } finally {
      setLoading(false);
    }
  };

  if (!product) {
    return (
      <View style={styles.loadingContainer}>
        <Text>加载中...</Text>
      </View>
    );
  }

  const totalPrice = BigInt(product.price) * BigInt(quantity);

  return (
    <View style={styles.container}>
      <ScrollView style={styles.scrollView}>
        {/* 商品图片 */}
        <View style={styles.imageContainer}>
          {product.imagesCid ? (
            <Image
              source={{ uri: `https://ipfs.io/ipfs/${product.imagesCid}` }}
              style={styles.productImage}
              resizeMode="cover"
            />
          ) : (
            <View style={styles.placeholderImage}>
              <ShoppingCart size={64} color="#ccc" />
            </View>
          )}
        </View>

        {/* 价格信息 */}
        <View style={styles.priceSection}>
          <Text style={styles.price}>¥{formatPrice(product.price)}</Text>
          <View style={styles.salesInfo}>
            <Text style={styles.salesText}>已售 {product.soldCount}</Text>
            <Text style={styles.stockText}>库存 {product.stock}</Text>
          </View>
        </View>

        {/* 商品标题 */}
        <View style={styles.titleSection}>
          <Text style={styles.productTitle}>商品 #{product.id}</Text>
          <TouchableOpacity style={styles.favoriteBtn}>
            <Heart size={20} color="#999" />
          </TouchableOpacity>
        </View>

        {/* 店铺信息 */}
        {shop && (
          <TouchableOpacity
            style={styles.shopSection}
            onPress={() => router.push(`/mall/shop/${shop.id}` as any)}
          >
            <View style={styles.shopLogo}>
              {shop.logoCid ? (
                <Image
                  source={{ uri: `https://ipfs.io/ipfs/${shop.logoCid}` }}
                  style={styles.shopLogoImage}
                />
              ) : (
                <Store size={24} color="#999" />
              )}
            </View>
            <View style={styles.shopInfo}>
              <Text style={styles.shopName}>{shop.name}</Text>
              <View style={styles.shopRating}>
                <Star size={12} color="#FFB300" fill="#FFB300" />
                <Text style={styles.ratingText}>{(shop.rating / 10).toFixed(1)}</Text>
              </View>
            </View>
            <Text style={styles.enterShop}>进店</Text>
          </TouchableOpacity>
        )}

        {/* 数量选择 */}
        <View style={styles.quantitySection}>
          <Text style={styles.quantityLabel}>购买数量</Text>
          <View style={styles.quantityControl}>
            <TouchableOpacity
              style={styles.quantityBtn}
              onPress={() => setQuantity(Math.max(1, quantity - 1))}
            >
              <Minus size={16} color="#666" />
            </TouchableOpacity>
            <Text style={styles.quantityValue}>{quantity}</Text>
            <TouchableOpacity
              style={styles.quantityBtn}
              onPress={() => setQuantity(Math.min(product.stock, quantity + 1))}
            >
              <Plus size={16} color="#666" />
            </TouchableOpacity>
          </View>
        </View>

        {/* 评价列表 */}
        <View style={styles.reviewSection}>
          <View style={styles.reviewHeader}>
            <Text style={styles.reviewTitle}>商品评价 ({reviews.length})</Text>
          </View>
          {reviews.length > 0 ? (
            reviews.slice(0, 3).map((review, index) => (
              <View key={index} style={styles.reviewItem}>
                <View style={styles.reviewUser}>
                  <View style={styles.reviewAvatar} />
                  <Text style={styles.reviewerName}>
                    {review.reviewer.slice(0, 8)}...
                  </Text>
                  <View style={styles.reviewRating}>
                    {[1, 2, 3, 4, 5].map((star) => (
                      <Star
                        key={star}
                        size={12}
                        color={star <= review.rating ? '#FFB300' : '#ddd'}
                        fill={star <= review.rating ? '#FFB300' : 'transparent'}
                      />
                    ))}
                  </View>
                </View>
                <Text style={styles.reviewContent}>
                  {review.contentCid ? '评价内容...' : '用户未填写评价内容'}
                </Text>
              </View>
            ))
          ) : (
            <Text style={styles.noReviews}>暂无评价</Text>
          )}
        </View>
      </ScrollView>

      {/* 底部操作栏 */}
      <View style={styles.bottomBar}>
        <TouchableOpacity
          style={styles.shopBtn}
          onPress={() => shop && router.push(`/mall/shop/${shop.id}` as any)}
        >
          <Store size={20} color="#666" />
          <Text style={styles.shopBtnText}>店铺</Text>
        </TouchableOpacity>
        <TouchableOpacity style={styles.cartBtn}>
          <ShoppingCart size={20} color="#666" />
          <Text style={styles.cartBtnText}>购物车</Text>
        </TouchableOpacity>
        <TouchableOpacity
          style={[styles.buyBtn, loading && styles.disabledBtn]}
          onPress={handleBuyNow}
          disabled={loading || product.stock === 0}
        >
          <Text style={styles.buyBtnText}>
            {product.stock === 0 ? '已售罄' : `立即购买 ¥${formatPrice(totalPrice.toString())}`}
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
  imageContainer: {
    width: '100%',
    aspectRatio: 1,
    backgroundColor: '#fff',
  },
  productImage: {
    width: '100%',
    height: '100%',
  },
  placeholderImage: {
    width: '100%',
    height: '100%',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#f5f5f5',
  },
  priceSection: {
    backgroundColor: '#fff',
    padding: 16,
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  price: {
    fontSize: 24,
    fontWeight: '700',
    color: '#E53935',
  },
  salesInfo: {
    flexDirection: 'row',
  },
  salesText: {
    fontSize: 12,
    color: '#999',
    marginRight: 12,
  },
  stockText: {
    fontSize: 12,
    color: '#999',
  },
  titleSection: {
    backgroundColor: '#fff',
    padding: 16,
    paddingTop: 0,
    flexDirection: 'row',
    alignItems: 'flex-start',
  },
  productTitle: {
    flex: 1,
    fontSize: 16,
    color: '#333',
    lineHeight: 22,
  },
  favoriteBtn: {
    padding: 4,
    marginLeft: 8,
  },
  shopSection: {
    backgroundColor: '#fff',
    marginTop: 8,
    padding: 16,
    flexDirection: 'row',
    alignItems: 'center',
  },
  shopLogo: {
    width: 48,
    height: 48,
    borderRadius: 24,
    backgroundColor: '#f5f5f5',
    alignItems: 'center',
    justifyContent: 'center',
    overflow: 'hidden',
  },
  shopLogoImage: {
    width: 48,
    height: 48,
  },
  shopInfo: {
    flex: 1,
    marginLeft: 12,
  },
  shopName: {
    fontSize: 14,
    fontWeight: '500',
    color: '#333',
  },
  shopRating: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 4,
  },
  ratingText: {
    fontSize: 12,
    color: '#FFB300',
    marginLeft: 4,
  },
  enterShop: {
    fontSize: 13,
    color: '#1976D2',
  },
  quantitySection: {
    backgroundColor: '#fff',
    marginTop: 8,
    padding: 16,
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  quantityLabel: {
    fontSize: 14,
    color: '#333',
  },
  quantityControl: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  quantityBtn: {
    width: 32,
    height: 32,
    borderRadius: 4,
    backgroundColor: '#f5f5f5',
    alignItems: 'center',
    justifyContent: 'center',
  },
  quantityValue: {
    fontSize: 16,
    fontWeight: '500',
    color: '#333',
    marginHorizontal: 16,
    minWidth: 24,
    textAlign: 'center',
  },
  reviewSection: {
    backgroundColor: '#fff',
    marginTop: 8,
    padding: 16,
  },
  reviewHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  reviewTitle: {
    fontSize: 14,
    fontWeight: '500',
    color: '#333',
  },
  reviewItem: {
    paddingVertical: 12,
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
  },
  reviewUser: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8,
  },
  reviewAvatar: {
    width: 24,
    height: 24,
    borderRadius: 12,
    backgroundColor: '#e0e0e0',
  },
  reviewerName: {
    fontSize: 12,
    color: '#666',
    marginLeft: 8,
    flex: 1,
  },
  reviewRating: {
    flexDirection: 'row',
  },
  reviewContent: {
    fontSize: 13,
    color: '#333',
    lineHeight: 20,
  },
  noReviews: {
    fontSize: 13,
    color: '#999',
    textAlign: 'center',
    paddingVertical: 20,
  },
  bottomBar: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    paddingVertical: 8,
    paddingHorizontal: 12,
    borderTopWidth: 1,
    borderTopColor: '#eee',
  },
  shopBtn: {
    alignItems: 'center',
    justifyContent: 'center',
    paddingHorizontal: 12,
  },
  shopBtnText: {
    fontSize: 10,
    color: '#666',
    marginTop: 2,
  },
  cartBtn: {
    alignItems: 'center',
    justifyContent: 'center',
    paddingHorizontal: 12,
    marginRight: 12,
  },
  cartBtnText: {
    fontSize: 10,
    color: '#666',
    marginTop: 2,
  },
  buyBtn: {
    flex: 1,
    backgroundColor: '#E53935',
    borderRadius: 20,
    paddingVertical: 12,
    alignItems: 'center',
  },
  buyBtnText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
  },
  disabledBtn: {
    backgroundColor: '#ccc',
  },
});
