import { useLocalSearchParams, useRouter } from 'expo-router';
import { Search, Store, ShoppingBag, X } from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  TextInput,
  Image,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import type { Shop, Product } from '@/src/types/sharemall';

type SearchTab = 'shops' | 'products';

export default function SearchScreen() {
  const { q } = useLocalSearchParams<{ q?: string }>();
  const router = useRouter();

  const [searchText, setSearchText] = useState(q || '');
  const [activeTab, setActiveTab] = useState<SearchTab>('products');
  const [shops, setShops] = useState<Shop[]>([]);
  const [products, setProducts] = useState<Product[]>([]);
  const [loading, setLoading] = useState(false);

  const doSearch = async (query: string) => {
    if (!query.trim()) {
      setShops([]);
      setProducts([]);
      return;
    }

    setLoading(true);
    try {
      const [shopList, productList] = await Promise.all([
        sharemallService.getShopList(50),
        sharemallService.getHotProducts(50),
      ]);

      // 简单的本地过滤 (实际应在链上实现搜索)
      const filteredShops = shopList.filter((shop) =>
        shop.name.toLowerCase().includes(query.toLowerCase())
      );
      const filteredProducts = productList.filter((product) =>
        product.id.toString().includes(query)
      );

      setShops(filteredShops);
      setProducts(filteredProducts);
    } catch (error) {
      console.error('Search failed:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (q) {
      doSearch(q);
    }
  }, [q]);

  const handleSearch = () => {
    doSearch(searchText);
  };

  const formatPrice = (price: string) => {
    const num = BigInt(price);
    return (Number(num) / 1e10).toFixed(2);
  };

  const renderShopItem = ({ item }: { item: Shop }) => (
    <TouchableOpacity
      style={styles.shopCard}
      onPress={() => router.push(`/mall/shop/${item.id}` as any)}
    >
      <View style={styles.shopLogo}>
        {item.logoCid ? (
          <Image
            source={{ uri: `https://ipfs.io/ipfs/${item.logoCid}` }}
            style={styles.shopLogoImage}
          />
        ) : (
          <Store size={32} color="#999" />
        )}
      </View>
      <View style={styles.shopInfo}>
        <Text style={styles.shopName}>{item.name}</Text>
        <Text style={styles.shopMeta}>
          {item.productCount} 件商品 · {item.totalOrders} 订单
        </Text>
      </View>
    </TouchableOpacity>
  );

  const renderProductItem = ({ item }: { item: Product }) => (
    <TouchableOpacity
      style={styles.productCard}
      onPress={() => router.push(`/mall/product/${item.id}` as any)}
    >
      <View style={styles.productImage}>
        {item.imagesCid ? (
          <Image
            source={{ uri: `https://ipfs.io/ipfs/${item.imagesCid}` }}
            style={styles.productImageContent}
          />
        ) : (
          <ShoppingBag size={32} color="#ccc" />
        )}
      </View>
      <Text style={styles.productName} numberOfLines={2}>
        商品 #{item.id}
      </Text>
      <View style={styles.productFooter}>
        <Text style={styles.productPrice}>¥{formatPrice(item.price)}</Text>
        <Text style={styles.productSold}>已售 {item.soldCount}</Text>
      </View>
    </TouchableOpacity>
  );

  const tabs: { key: SearchTab; label: string }[] = [
    { key: 'products', label: '商品' },
    { key: 'shops', label: '店铺' },
  ];

  return (
    <View style={styles.container}>
      {/* 搜索栏 */}
      <View style={styles.searchBar}>
        <View style={styles.searchInputContainer}>
          <Search size={18} color="#999" />
          <TextInput
            style={styles.searchInput}
            placeholder="搜索店铺或商品..."
            value={searchText}
            onChangeText={setSearchText}
            onSubmitEditing={handleSearch}
            returnKeyType="search"
          />
          {searchText.length > 0 && (
            <TouchableOpacity onPress={() => setSearchText('')}>
              <X size={18} color="#999" />
            </TouchableOpacity>
          )}
        </View>
        <TouchableOpacity style={styles.searchBtn} onPress={handleSearch}>
          <Text style={styles.searchBtnText}>搜索</Text>
        </TouchableOpacity>
      </View>

      {/* 标签栏 */}
      <View style={styles.tabBar}>
        {tabs.map((tab) => (
          <TouchableOpacity
            key={tab.key}
            style={[styles.tab, activeTab === tab.key && styles.activeTab]}
            onPress={() => setActiveTab(tab.key)}
          >
            <Text style={[styles.tabText, activeTab === tab.key && styles.activeTabText]}>
              {tab.label}
              {tab.key === 'products' && products.length > 0 && ` (${products.length})`}
              {tab.key === 'shops' && shops.length > 0 && ` (${shops.length})`}
            </Text>
          </TouchableOpacity>
        ))}
      </View>

      {/* 搜索结果 */}
      {activeTab === 'products' ? (
        <FlatList
          data={products}
          renderItem={renderProductItem}
          keyExtractor={(item) => item.id.toString()}
          numColumns={2}
          contentContainerStyle={styles.productGrid}
          ListEmptyComponent={
            <View style={styles.emptyContainer}>
              <ShoppingBag size={48} color="#ccc" />
              <Text style={styles.emptyText}>
                {searchText ? '未找到相关商品' : '输入关键词搜索商品'}
              </Text>
            </View>
          }
        />
      ) : (
        <FlatList
          data={shops}
          renderItem={renderShopItem}
          keyExtractor={(item) => item.id.toString()}
          contentContainerStyle={styles.listContent}
          ListEmptyComponent={
            <View style={styles.emptyContainer}>
              <Store size={48} color="#ccc" />
              <Text style={styles.emptyText}>
                {searchText ? '未找到相关店铺' : '输入关键词搜索店铺'}
              </Text>
            </View>
          }
        />
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  searchBar: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    padding: 12,
    alignItems: 'center',
  },
  searchInputContainer: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 12,
    marginRight: 12,
  },
  searchInput: {
    flex: 1,
    paddingVertical: 10,
    paddingHorizontal: 8,
    fontSize: 14,
  },
  searchBtn: {
    paddingHorizontal: 16,
    paddingVertical: 10,
  },
  searchBtnText: {
    color: '#1976D2',
    fontSize: 14,
    fontWeight: '500',
  },
  tabBar: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#eee',
  },
  tab: {
    flex: 1,
    paddingVertical: 14,
    alignItems: 'center',
  },
  activeTab: {
    borderBottomWidth: 2,
    borderBottomColor: '#1976D2',
  },
  tabText: {
    fontSize: 14,
    color: '#666',
  },
  activeTabText: {
    color: '#1976D2',
    fontWeight: '500',
  },
  listContent: {
    padding: 12,
  },
  shopCard: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  shopLogo: {
    width: 56,
    height: 56,
    borderRadius: 28,
    backgroundColor: '#f5f5f5',
    alignItems: 'center',
    justifyContent: 'center',
    overflow: 'hidden',
  },
  shopLogoImage: {
    width: 56,
    height: 56,
  },
  shopInfo: {
    flex: 1,
    marginLeft: 12,
    justifyContent: 'center',
  },
  shopName: {
    fontSize: 15,
    fontWeight: '500',
    color: '#333',
    marginBottom: 4,
  },
  shopMeta: {
    fontSize: 12,
    color: '#999',
  },
  productGrid: {
    padding: 12,
  },
  productCard: {
    width: '48%',
    marginHorizontal: '1%',
    marginBottom: 12,
    backgroundColor: '#fff',
    borderRadius: 8,
    overflow: 'hidden',
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
