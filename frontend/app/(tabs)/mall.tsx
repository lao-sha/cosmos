import { useRouter } from 'expo-router';
import { ShoppingBag, Store, TrendingUp, ChevronRight, Star } from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
  Image,
  TextInput,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import type { Shop, Product } from '@/src/types/sharemall';

export default function MallScreen() {
  const router = useRouter();
  const [shops, setShops] = useState<Shop[]>([]);
  const [products, setProducts] = useState<Product[]>([]);
  const [refreshing, setRefreshing] = useState(false);
  const [searchText, setSearchText] = useState('');

  const loadData = async () => {
    try {
      const [shopList, productList] = await Promise.all([
        sharemallService.getShopList(10),
        sharemallService.getHotProducts(10),
      ]);
      setShops(shopList);
      setProducts(productList);
    } catch (error) {
      console.error('Failed to load mall data:', error);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const onRefresh = async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  };

  const formatPrice = (price: string) => {
    const num = BigInt(price);
    return (Number(num) / 1e10).toFixed(2);
  };

  return (
    <ScrollView
      style={styles.container}
      refreshControl={<RefreshControl refreshing={refreshing} onRefresh={onRefresh} />}
    >
      {/* 搜索栏 */}
      <View style={styles.searchContainer}>
        <TextInput
          style={styles.searchInput}
          placeholder="搜索店铺或商品..."
          value={searchText}
          onChangeText={setSearchText}
          onSubmitEditing={() => {
            if (searchText.trim()) {
              router.push(`/mall/search?q=${encodeURIComponent(searchText)}`);
            }
          }}
        />
      </View>

      {/* 快捷入口 */}
      <View style={styles.quickActions}>
        <TouchableOpacity
          style={styles.quickAction}
          onPress={() => router.push('/mall/shops')}
        >
          <View style={[styles.quickIcon, { backgroundColor: '#E3F2FD' }]}>
            <Store size={24} color="#1976D2" />
          </View>
          <Text style={styles.quickText}>全部店铺</Text>
        </TouchableOpacity>

        <TouchableOpacity
          style={styles.quickAction}
          onPress={() => router.push('/mall/orders')}
        >
          <View style={[styles.quickIcon, { backgroundColor: '#FFF3E0' }]}>
            <ShoppingBag size={24} color="#F57C00" />
          </View>
          <Text style={styles.quickText}>我的订单</Text>
        </TouchableOpacity>

        <TouchableOpacity
          style={styles.quickAction}
          onPress={() => router.push('/mall/market')}
        >
          <View style={[styles.quickIcon, { backgroundColor: '#E8F5E9' }]}>
            <TrendingUp size={24} color="#388E3C" />
          </View>
          <Text style={styles.quickText}>代币交易</Text>
        </TouchableOpacity>

        <TouchableOpacity
          style={styles.quickAction}
          onPress={() => router.push('/mall/seller')}
        >
          <View style={[styles.quickIcon, { backgroundColor: '#FCE4EC' }]}>
            <Store size={24} color="#C2185B" />
          </View>
          <Text style={styles.quickText}>卖家中心</Text>
        </TouchableOpacity>
      </View>

      {/* 热门店铺 */}
      <View style={styles.section}>
        <View style={styles.sectionHeader}>
          <Text style={styles.sectionTitle}>热门店铺</Text>
          <TouchableOpacity
            style={styles.moreBtn}
            onPress={() => router.push('/mall/shops')}
          >
            <Text style={styles.moreText}>查看更多</Text>
            <ChevronRight size={16} color="#666" />
          </TouchableOpacity>
        </View>

        <ScrollView horizontal showsHorizontalScrollIndicator={false}>
          {shops.length > 0 ? (
            shops.map((shop) => (
              <TouchableOpacity
                key={shop.id}
                style={styles.shopCard}
                onPress={() => router.push(`/mall/shop/${shop.id}`)}
              >
                <View style={styles.shopLogo}>
                  {shop.logoCid ? (
                    <Image
                      source={{ uri: `https://ipfs.io/ipfs/${shop.logoCid}` }}
                      style={styles.shopLogoImage}
                    />
                  ) : (
                    <Store size={32} color="#999" />
                  )}
                </View>
                <Text style={styles.shopName} numberOfLines={1}>
                  {shop.name}
                </Text>
                <View style={styles.shopRating}>
                  <Star size={12} color="#FFB300" fill="#FFB300" />
                  <Text style={styles.ratingText}>
                    {(shop.rating / 10).toFixed(1)}
                  </Text>
                  <Text style={styles.ratingCount}>({shop.ratingCount})</Text>
                </View>
              </TouchableOpacity>
            ))
          ) : (
            <View style={styles.emptyShops}>
              <Text style={styles.emptyText}>暂无店铺</Text>
            </View>
          )}
        </ScrollView>
      </View>

      {/* 热门商品 */}
      <View style={styles.section}>
        <View style={styles.sectionHeader}>
          <Text style={styles.sectionTitle}>热门商品</Text>
          <TouchableOpacity style={styles.moreBtn}>
            <Text style={styles.moreText}>查看更多</Text>
            <ChevronRight size={16} color="#666" />
          </TouchableOpacity>
        </View>

        <View style={styles.productGrid}>
          {products.length > 0 ? (
            products.map((product) => (
              <TouchableOpacity
                key={product.id}
                style={styles.productCard}
                onPress={() => router.push(`/mall/product/${product.id}`)}
              >
                <View style={styles.productImage}>
                  {product.imagesCid ? (
                    <Image
                      source={{ uri: `https://ipfs.io/ipfs/${product.imagesCid}` }}
                      style={styles.productImageContent}
                    />
                  ) : (
                    <ShoppingBag size={40} color="#ccc" />
                  )}
                </View>
                <Text style={styles.productName} numberOfLines={2}>
                  商品 #{product.id}
                </Text>
                <View style={styles.productFooter}>
                  <Text style={styles.productPrice}>
                    ¥{formatPrice(product.price)}
                  </Text>
                  <Text style={styles.productSold}>
                    已售 {product.soldCount}
                  </Text>
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
  searchContainer: {
    padding: 16,
    backgroundColor: '#fff',
  },
  searchInput: {
    backgroundColor: '#f0f0f0',
    borderRadius: 8,
    paddingHorizontal: 16,
    paddingVertical: 10,
    fontSize: 14,
  },
  quickActions: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    paddingVertical: 16,
    paddingHorizontal: 8,
    marginTop: 8,
  },
  quickAction: {
    flex: 1,
    alignItems: 'center',
  },
  quickIcon: {
    width: 48,
    height: 48,
    borderRadius: 24,
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 8,
  },
  quickText: {
    fontSize: 12,
    color: '#333',
  },
  section: {
    backgroundColor: '#fff',
    marginTop: 8,
    padding: 16,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  moreBtn: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  moreText: {
    fontSize: 12,
    color: '#666',
  },
  shopCard: {
    width: 100,
    marginRight: 12,
    alignItems: 'center',
  },
  shopLogo: {
    width: 64,
    height: 64,
    borderRadius: 32,
    backgroundColor: '#f5f5f5',
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 8,
    overflow: 'hidden',
  },
  shopLogoImage: {
    width: 64,
    height: 64,
  },
  shopName: {
    fontSize: 13,
    color: '#333',
    marginBottom: 4,
    textAlign: 'center',
  },
  shopRating: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  ratingText: {
    fontSize: 11,
    color: '#FFB300',
    marginLeft: 2,
  },
  ratingCount: {
    fontSize: 10,
    color: '#999',
    marginLeft: 2,
  },
  emptyShops: {
    width: '100%',
    paddingVertical: 20,
    alignItems: 'center',
  },
  emptyText: {
    color: '#999',
    fontSize: 14,
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
});
