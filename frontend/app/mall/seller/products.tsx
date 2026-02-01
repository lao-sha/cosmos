import { useRouter } from 'expo-router';
import { Package, Plus, Edit, Eye, EyeOff, Trash2 } from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
  Alert,
  Image,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import { sharemallTxService } from '@/src/services/sharemall-tx';
import type { Product, Shop } from '@/src/types/sharemall';
import { useWalletStore } from '@/src/stores/wallet';

export default function ProductsScreen() {
  const router = useRouter();
  const { currentAccount } = useWalletStore();
  const [products, setProducts] = useState<Product[]>([]);
  const [myShop, setMyShop] = useState<Shop | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  const loadData = async () => {
    if (!currentAccount) return;
    try {
      const shops = await sharemallService.getShopList(100);
      const userShop = shops.find((s) => s.owner === currentAccount.address);
      setMyShop(userShop || null);

      if (userShop) {
        const list = await sharemallService.getShopProducts(userShop.id);
        setProducts(list);
      }
    } catch (error) {
      console.error('Failed to load products:', error);
    }
  };

  useEffect(() => {
    loadData();
  }, [currentAccount]);

  const onRefresh = async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  };

  const formatPrice = (price: string) => {
    const num = BigInt(price);
    return (Number(num) / 1e10).toFixed(2);
  };

  const handleToggleStatus = async (product: Product) => {
    try {
      if (product.status === 'OnSale') {
        await sharemallTxService.unpublishProduct(product.id);
      } else {
        await sharemallTxService.publishProduct(product.id);
      }
      loadData();
    } catch (error: any) {
      Alert.alert('错误', error.message || '操作失败');
    }
  };

  const handleDelete = (product: Product) => {
    Alert.alert('确认删除', `确定要删除商品 #${product.id} 吗？`, [
      { text: '取消', style: 'cancel' },
      {
        text: '删除',
        style: 'destructive',
        onPress: async () => {
          try {
            await sharemallTxService.deleteProduct(product.id);
            loadData();
          } catch (error: any) {
            Alert.alert('错误', error.message || '删除失败');
          }
        },
      },
    ]);
  };

  const renderProductItem = ({ item }: { item: Product }) => (
    <View style={styles.productCard}>
      <View style={styles.productImage}>
        {item.imagesCid ? (
          <Image
            source={{ uri: `https://ipfs.io/ipfs/${item.imagesCid}` }}
            style={styles.productImageContent}
          />
        ) : (
          <Package size={32} color="#ccc" />
        )}
      </View>
      <View style={styles.productInfo}>
        <Text style={styles.productName}>商品 #{item.id}</Text>
        <Text style={styles.productPrice}>¥{formatPrice(item.price)}</Text>
        <View style={styles.productMeta}>
          <Text style={styles.metaText}>库存: {item.stock}</Text>
          <Text style={styles.metaText}>已售: {item.soldCount}</Text>
          <View
            style={[
              styles.statusBadge,
              {
                backgroundColor:
                  item.status === 'OnSale' ? '#E8F5E9' : '#ECEFF1',
              },
            ]}
          >
            <Text
              style={[
                styles.statusText,
                {
                  color: item.status === 'OnSale' ? '#388E3C' : '#757575',
                },
              ]}
            >
              {item.status === 'OnSale' ? '在售' : item.status}
            </Text>
          </View>
        </View>
      </View>
      <View style={styles.productActions}>
        <TouchableOpacity
          style={styles.actionBtn}
          onPress={() => handleToggleStatus(item)}
        >
          {item.status === 'OnSale' ? (
            <EyeOff size={18} color="#F57C00" />
          ) : (
            <Eye size={18} color="#388E3C" />
          )}
        </TouchableOpacity>
        <TouchableOpacity style={styles.actionBtn}>
          <Edit size={18} color="#1976D2" />
        </TouchableOpacity>
        <TouchableOpacity
          style={styles.actionBtn}
          onPress={() => handleDelete(item)}
        >
          <Trash2 size={18} color="#D32F2F" />
        </TouchableOpacity>
      </View>
    </View>
  );

  return (
    <View style={styles.container}>
      <FlatList
        data={products}
        renderItem={renderProductItem}
        keyExtractor={(item) => item.id.toString()}
        contentContainerStyle={styles.listContent}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
        ListEmptyComponent={
          <View style={styles.emptyContainer}>
            <Package size={48} color="#ccc" />
            <Text style={styles.emptyText}>暂无商品</Text>
            <Text style={styles.emptyHint}>点击右下角按钮发布商品</Text>
          </View>
        }
      />

      <TouchableOpacity
        style={styles.fab}
        onPress={() => router.push('/mall/seller/product-create' as any)}
      >
        <Plus size={24} color="#fff" />
      </TouchableOpacity>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  listContent: {
    padding: 12,
  },
  productCard: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 12,
    marginBottom: 12,
  },
  productImage: {
    width: 80,
    height: 80,
    borderRadius: 8,
    backgroundColor: '#f5f5f5',
    alignItems: 'center',
    justifyContent: 'center',
    overflow: 'hidden',
  },
  productImageContent: {
    width: 80,
    height: 80,
  },
  productInfo: {
    flex: 1,
    marginLeft: 12,
    justifyContent: 'center',
  },
  productName: {
    fontSize: 14,
    fontWeight: '500',
    color: '#333',
    marginBottom: 4,
  },
  productPrice: {
    fontSize: 16,
    fontWeight: '600',
    color: '#E53935',
    marginBottom: 6,
  },
  productMeta: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  metaText: {
    fontSize: 12,
    color: '#999',
    marginRight: 12,
  },
  statusBadge: {
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
  },
  statusText: {
    fontSize: 11,
  },
  productActions: {
    justifyContent: 'center',
  },
  actionBtn: {
    padding: 8,
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
  emptyHint: {
    marginTop: 4,
    fontSize: 12,
    color: '#bbb',
  },
  fab: {
    position: 'absolute',
    right: 20,
    bottom: 20,
    width: 56,
    height: 56,
    borderRadius: 28,
    backgroundColor: '#1976D2',
    alignItems: 'center',
    justifyContent: 'center',
    elevation: 4,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.25,
    shadowRadius: 4,
  },
});
