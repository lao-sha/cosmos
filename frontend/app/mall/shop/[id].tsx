import { useLocalSearchParams, useRouter } from 'expo-router';
import { Store, Star, ShoppingBag, Users, MessageCircle } from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
  Image,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import type { Shop, Product, ShopTokenConfig } from '@/src/types/sharemall';

export default function ShopDetailScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const router = useRouter();
  const [shop, setShop] = useState<Shop | null>(null);
  const [products, setProducts] = useState<Product[]>([]);
  const [tokenConfig, setTokenConfig] = useState<ShopTokenConfig | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  const shopId = parseInt(id || '0', 10);

  const loadData = async () => {
    if (!shopId) return;
    try {
      const [shopData, productList, config] = await Promise.all([
        sharemallService.getShop(shopId),
        sharemallService.getShopProducts(shopId),
        sharemallService.getTokenConfig(shopId),
      ]);
      setShop(shopData);
      setProducts(productList);
      setTokenConfig(config);
    } catch (error) {
      console.error('Failed to load shop:', error);
    }
  };

  useEffect(() => {
    loadData();
  }, [shopId]);

  const onRefresh = async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  };

  const formatPrice = (price: string) => {
    const num = BigInt(price);
    return (Number(num) / 1e10).toFixed(2);
  };

  if (!shop) {
    return (
      <View style={styles.loadingContainer}>
        <Text>加载中...</Text>
      </View>
    );
  }

  return (
    <ScrollView
      style={styles.container}
      refreshControl={<RefreshControl refreshing={refreshing} onRefresh={onRefresh} />}
    >
      {/* 店铺头部 */}
      <View style={styles.header}>
        <View style={styles.shopLogo}>
          {shop.logoCid ? (
            <Image
              source={{ uri: `https://ipfs.io/ipfs/${shop.logoCid}` }}
              style={styles.shopLogoImage}
            />
          ) : (
            <Store size={48} color="#999" />
          )}
        </View>
        <View style={styles.shopInfo}>
          <Text style={styles.shopName}>{shop.name}</Text>
          <View style={styles.ratingRow}>
            <Star size={16} color="#FFB300" fill="#FFB300" />
            <Text style={styles.ratingText}>{(shop.rating / 10).toFixed(1)}</Text>
            <Text style={styles.ratingCount}>({shop.ratingCount}评价)</Text>
          </View>
        </View>
      </View>

      {/* 店铺统计 */}
      <View style={styles.statsRow}>
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{shop.productCount}</Text>
          <Text style={styles.statLabel}>商品</Text>
        </View>
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{shop.totalOrders}</Text>
          <Text style={styles.statLabel}>订单</Text>
        </View>
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{formatPrice(shop.totalSales)}</Text>
          <Text style={styles.statLabel}>销售额</Text>
        </View>
      </View>

      {/* 快捷操作 */}
      <View style={styles.actions}>
        {tokenConfig?.enabled && (
          <TouchableOpacity
            style={styles.actionBtn}
            onPress={() => router.push(`/mall/market?shopId=${shopId}` as any)}
          >
            <ShoppingBag size={20} color="#1976D2" />
            <Text style={styles.actionText}>代币交易</Text>
          </TouchableOpacity>
        )}
        <TouchableOpacity
          style={styles.actionBtn}
          onPress={() => router.push(`/mall/shop/${shopId}/members` as any)}
        >
          <Users size={20} color="#388E3C" />
          <Text style={styles.actionText}>会员中心</Text>
        </TouchableOpacity>
        <TouchableOpacity style={styles.actionBtn}>
          <MessageCircle size={20} color="#F57C00" />
          <Text style={styles.actionText}>联系客服</Text>
        </TouchableOpacity>
      </View>

      {/* 商品列表 */}
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>店铺商品</Text>
        <View style={styles.productGrid}>
          {products.length > 0 ? (
            products.map((product) => (
              <TouchableOpacity
                key={product.id}
                style={styles.productCard}
                onPress={() => router.push(`/mall/product/${product.id}` as any)}
              >
                <View style={styles.productImage}>
                  {product.imagesCid ? (
                    <Image
                      source={{ uri: `https://ipfs.io/ipfs/${product.imagesCid}` }}
                      style={styles.productImageContent}
                    />
                  ) : (
                    <ShoppingBag size={32} color="#ccc" />
                  )}
                </View>
                <Text style={styles.productName} numberOfLines={2}>
                  商品 #{product.id}
                </Text>
                <View style={styles.productFooter}>
                  <Text style={styles.productPrice}>¥{formatPrice(product.price)}</Text>
                  <Text style={styles.productSold}>已售 {product.soldCount}</Text>
                </View>
              </TouchableOpacity>
            ))
          ) : (
            <View style={styles.emptyProducts}>
              <Text style={styles.emptyText}>暂无商品</Text>
            </View>
          )}
        </View>
      </View>
    </ScrollView>
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
  header: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    padding: 16,
    alignItems: 'center',
  },
  shopLogo: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: '#f5f5f5',
    alignItems: 'center',
    justifyContent: 'center',
    overflow: 'hidden',
  },
  shopLogoImage: {
    width: 80,
    height: 80,
  },
  shopInfo: {
    flex: 1,
    marginLeft: 16,
  },
  shopName: {
    fontSize: 20,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  ratingRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  ratingText: {
    fontSize: 14,
    color: '#FFB300',
    marginLeft: 4,
  },
  ratingCount: {
    fontSize: 13,
    color: '#999',
    marginLeft: 4,
  },
  statsRow: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    paddingVertical: 16,
    marginTop: 1,
  },
  statItem: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  statLabel: {
    fontSize: 12,
    color: '#999',
    marginTop: 4,
  },
  actions: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    marginTop: 8,
    paddingVertical: 12,
  },
  actionBtn: {
    flex: 1,
    alignItems: 'center',
    paddingVertical: 8,
  },
  actionText: {
    fontSize: 12,
    color: '#666',
    marginTop: 4,
  },
  section: {
    backgroundColor: '#fff',
    marginTop: 8,
    padding: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 12,
  },
  productGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    marginHorizontal: -6,
  },
  productCard: {
    width: '48%',
    marginHorizontal: '1%',
    marginBottom: 12,
    backgroundColor: '#fff',
    borderRadius: 8,
    overflow: 'hidden',
    borderWidth: 1,
    borderColor: '#eee',
  },
  productImage: {
    width: '100%',
    aspectRatio: 1,
    backgroundColor: '#f5f5f5',
    alignItems: 'center',
    justifyContent: 'center',
  },
  productImageContent: {
    width: '100%',
    height: '100%',
  },
  productName: {
    fontSize: 13,
    color: '#333',
    padding: 8,
    paddingBottom: 4,
    minHeight: 40,
  },
  productFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 8,
    paddingBottom: 8,
  },
  productPrice: {
    fontSize: 14,
    fontWeight: '600',
    color: '#E53935',
  },
  productSold: {
    fontSize: 11,
    color: '#999',
  },
  emptyProducts: {
    width: '100%',
    paddingVertical: 40,
    alignItems: 'center',
  },
  emptyText: {
    color: '#999',
    fontSize: 14,
  },
});
