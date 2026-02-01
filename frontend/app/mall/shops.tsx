import { useRouter } from 'expo-router';
import { Store, Star, Search } from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
  Image,
  TextInput,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import type { Shop } from '@/src/types/sharemall';

export default function ShopsScreen() {
  const router = useRouter();
  const [shops, setShops] = useState<Shop[]>([]);
  const [filteredShops, setFilteredShops] = useState<Shop[]>([]);
  const [refreshing, setRefreshing] = useState(false);
  const [searchText, setSearchText] = useState('');

  const loadShops = async () => {
    try {
      const list = await sharemallService.getShopList(50);
      setShops(list);
      setFilteredShops(list);
    } catch (error) {
      console.error('Failed to load shops:', error);
    }
  };

  useEffect(() => {
    loadShops();
  }, []);

  useEffect(() => {
    if (searchText.trim()) {
      const filtered = shops.filter((shop) =>
        shop.name.toLowerCase().includes(searchText.toLowerCase())
      );
      setFilteredShops(filtered);
    } else {
      setFilteredShops(shops);
    }
  }, [searchText, shops]);

  const onRefresh = async () => {
    setRefreshing(true);
    await loadShops();
    setRefreshing(false);
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
          <Store size={40} color="#999" />
        )}
      </View>
      <View style={styles.shopInfo}>
        <Text style={styles.shopName}>{item.name}</Text>
        <View style={styles.shopMeta}>
          <View style={styles.ratingContainer}>
            <Star size={14} color="#FFB300" fill="#FFB300" />
            <Text style={styles.ratingText}>
              {(item.rating / 10).toFixed(1)}
            </Text>
            <Text style={styles.ratingCount}>({item.ratingCount}评价)</Text>
          </View>
          <Text style={styles.salesText}>销量 {item.totalOrders}</Text>
        </View>
        <Text style={styles.productCount}>{item.productCount} 件商品</Text>
      </View>
    </TouchableOpacity>
  );

  return (
    <View style={styles.container}>
      <View style={styles.searchContainer}>
        <Search size={18} color="#999" style={styles.searchIcon} />
        <TextInput
          style={styles.searchInput}
          placeholder="搜索店铺..."
          value={searchText}
          onChangeText={setSearchText}
        />
      </View>

      <FlatList
        data={filteredShops}
        renderItem={renderShopItem}
        keyExtractor={(item) => item.id.toString()}
        contentContainerStyle={styles.listContent}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
        ListEmptyComponent={
          <View style={styles.emptyContainer}>
            <Store size={48} color="#ccc" />
            <Text style={styles.emptyText}>暂无店铺</Text>
          </View>
        }
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  searchContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#fff',
    margin: 12,
    borderRadius: 8,
    paddingHorizontal: 12,
  },
  searchIcon: {
    marginRight: 8,
  },
  searchInput: {
    flex: 1,
    paddingVertical: 10,
    fontSize: 14,
  },
  listContent: {
    padding: 12,
    paddingTop: 0,
  },
  shopCard: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  shopLogo: {
    width: 72,
    height: 72,
    borderRadius: 36,
    backgroundColor: '#f5f5f5',
    alignItems: 'center',
    justifyContent: 'center',
    overflow: 'hidden',
  },
  shopLogoImage: {
    width: 72,
    height: 72,
  },
  shopInfo: {
    flex: 1,
    marginLeft: 16,
    justifyContent: 'center',
  },
  shopName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 6,
  },
  shopMeta: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 4,
  },
  ratingContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    marginRight: 16,
  },
  ratingText: {
    fontSize: 13,
    color: '#FFB300',
    marginLeft: 4,
  },
  ratingCount: {
    fontSize: 12,
    color: '#999',
    marginLeft: 4,
  },
  salesText: {
    fontSize: 12,
    color: '#666',
  },
  productCount: {
    fontSize: 12,
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
