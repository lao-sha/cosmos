import { useRouter } from 'expo-router';
import { ShoppingCart, Trash2, Minus, Plus, Store } from 'lucide-react-native';
import React, { useState } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  Image,
  Alert,
} from 'react-native';

import type { CartItem } from '@/src/types/sharemall';

// 本地购物车状态 (实际应使用 zustand store)
const useCartStore = () => {
  const [items, setItems] = useState<CartItem[]>([]);

  const updateQuantity = (productId: number, quantity: number) => {
    if (quantity <= 0) {
      setItems(items.filter((item) => item.productId !== productId));
    } else {
      setItems(
        items.map((item) =>
          item.productId === productId ? { ...item, quantity } : item
        )
      );
    }
  };

  const removeItem = (productId: number) => {
    setItems(items.filter((item) => item.productId !== productId));
  };

  const clearCart = () => {
    setItems([]);
  };

  return { items, updateQuantity, removeItem, clearCart };
};

export default function CartScreen() {
  const router = useRouter();
  const { items, updateQuantity, removeItem, clearCart } = useCartStore();
  const [selectedItems, setSelectedItems] = useState<number[]>([]);

  const formatPrice = (price: string) => {
    const num = BigInt(price);
    return (Number(num) / 1e10).toFixed(2);
  };

  const toggleSelect = (productId: number) => {
    if (selectedItems.includes(productId)) {
      setSelectedItems(selectedItems.filter((id) => id !== productId));
    } else {
      setSelectedItems([...selectedItems, productId]);
    }
  };

  const selectAll = () => {
    if (selectedItems.length === items.length) {
      setSelectedItems([]);
    } else {
      setSelectedItems(items.map((item) => item.productId));
    }
  };

  const calculateTotal = () => {
    return items
      .filter((item) => selectedItems.includes(item.productId))
      .reduce((total, item) => {
        const price = item.product?.price || '0';
        return total + BigInt(price) * BigInt(item.quantity);
      }, BigInt(0));
  };

  const handleCheckout = () => {
    if (selectedItems.length === 0) {
      Alert.alert('提示', '请选择要结算的商品');
      return;
    }
    // 跳转到结算页
    const selectedProducts = items.filter((item) =>
      selectedItems.includes(item.productId)
    );
    if (selectedProducts.length === 1) {
      router.push({
        pathname: '/mall/checkout' as any,
        params: {
          productId: selectedProducts[0].productId,
          quantity: selectedProducts[0].quantity,
        },
      });
    } else {
      Alert.alert('提示', '暂不支持多商品结算，请分别购买');
    }
  };

  const handleRemove = (productId: number) => {
    Alert.alert('确认删除', '确定要从购物车中删除该商品吗？', [
      { text: '取消', style: 'cancel' },
      { text: '删除', style: 'destructive', onPress: () => removeItem(productId) },
    ]);
  };

  const renderCartItem = ({ item }: { item: CartItem }) => {
    const isSelected = selectedItems.includes(item.productId);

    return (
      <View style={styles.cartItem}>
        <TouchableOpacity
          style={[styles.checkbox, isSelected && styles.checkboxSelected]}
          onPress={() => toggleSelect(item.productId)}
        >
          {isSelected && <Text style={styles.checkmark}>✓</Text>}
        </TouchableOpacity>

        <View style={styles.productImage}>
          {item.product?.imagesCid ? (
            <Image
              source={{ uri: `https://ipfs.io/ipfs/${item.product.imagesCid}` }}
              style={styles.productImageContent}
            />
          ) : (
            <ShoppingCart size={24} color="#ccc" />
          )}
        </View>

        <View style={styles.productInfo}>
          <Text style={styles.productName} numberOfLines={2}>
            商品 #{item.productId}
          </Text>
          <Text style={styles.productPrice}>
            ¥{item.product ? formatPrice(item.product.price) : '0.00'}
          </Text>

          <View style={styles.quantityRow}>
            <View style={styles.quantityControl}>
              <TouchableOpacity
                style={styles.quantityBtn}
                onPress={() => updateQuantity(item.productId, item.quantity - 1)}
              >
                <Minus size={14} color="#666" />
              </TouchableOpacity>
              <Text style={styles.quantityValue}>{item.quantity}</Text>
              <TouchableOpacity
                style={styles.quantityBtn}
                onPress={() => updateQuantity(item.productId, item.quantity + 1)}
              >
                <Plus size={14} color="#666" />
              </TouchableOpacity>
            </View>
            <TouchableOpacity onPress={() => handleRemove(item.productId)}>
              <Trash2 size={18} color="#999" />
            </TouchableOpacity>
          </View>
        </View>
      </View>
    );
  };

  const total = calculateTotal();

  return (
    <View style={styles.container}>
      {items.length > 0 ? (
        <>
          <FlatList
            data={items}
            renderItem={renderCartItem}
            keyExtractor={(item) => item.productId.toString()}
            contentContainerStyle={styles.listContent}
          />

          <View style={styles.bottomBar}>
            <TouchableOpacity style={styles.selectAllBtn} onPress={selectAll}>
              <View
                style={[
                  styles.checkbox,
                  selectedItems.length === items.length && styles.checkboxSelected,
                ]}
              >
                {selectedItems.length === items.length && (
                  <Text style={styles.checkmark}>✓</Text>
                )}
              </View>
              <Text style={styles.selectAllText}>全选</Text>
            </TouchableOpacity>

            <View style={styles.totalInfo}>
              <Text style={styles.totalLabel}>合计:</Text>
              <Text style={styles.totalPrice}>¥{formatPrice(total.toString())}</Text>
            </View>

            <TouchableOpacity
              style={[
                styles.checkoutBtn,
                selectedItems.length === 0 && styles.checkoutBtnDisabled,
              ]}
              onPress={handleCheckout}
              disabled={selectedItems.length === 0}
            >
              <Text style={styles.checkoutBtnText}>
                结算({selectedItems.length})
              </Text>
            </TouchableOpacity>
          </View>
        </>
      ) : (
        <View style={styles.emptyContainer}>
          <ShoppingCart size={64} color="#ccc" />
          <Text style={styles.emptyText}>购物车是空的</Text>
          <TouchableOpacity
            style={styles.goShopBtn}
            onPress={() => router.push('/mall/shops' as any)}
          >
            <Text style={styles.goShopBtnText}>去逛逛</Text>
          </TouchableOpacity>
        </View>
      )}
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
  cartItem: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 12,
    marginBottom: 12,
    alignItems: 'flex-start',
  },
  checkbox: {
    width: 22,
    height: 22,
    borderRadius: 11,
    borderWidth: 2,
    borderColor: '#ddd',
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
    marginTop: 20,
  },
  checkboxSelected: {
    backgroundColor: '#E53935',
    borderColor: '#E53935',
  },
  checkmark: {
    color: '#fff',
    fontSize: 12,
    fontWeight: 'bold',
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
  },
  productName: {
    fontSize: 14,
    color: '#333',
    marginBottom: 6,
    lineHeight: 20,
  },
  productPrice: {
    fontSize: 16,
    fontWeight: '600',
    color: '#E53935',
    marginBottom: 8,
  },
  quantityRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  quantityControl: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  quantityBtn: {
    width: 28,
    height: 28,
    borderRadius: 4,
    backgroundColor: '#f5f5f5',
    alignItems: 'center',
    justifyContent: 'center',
  },
  quantityValue: {
    fontSize: 14,
    color: '#333',
    marginHorizontal: 12,
    minWidth: 20,
    textAlign: 'center',
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
  selectAllBtn: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  selectAllText: {
    fontSize: 14,
    color: '#333',
    marginLeft: 8,
  },
  totalInfo: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'flex-end',
    marginRight: 12,
  },
  totalLabel: {
    fontSize: 14,
    color: '#333',
  },
  totalPrice: {
    fontSize: 18,
    fontWeight: '700',
    color: '#E53935',
    marginLeft: 4,
  },
  checkoutBtn: {
    backgroundColor: '#E53935',
    borderRadius: 20,
    paddingHorizontal: 20,
    paddingVertical: 10,
  },
  checkoutBtnDisabled: {
    backgroundColor: '#ccc',
  },
  checkoutBtnText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
  },
  emptyContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  emptyText: {
    fontSize: 16,
    color: '#999',
    marginTop: 16,
    marginBottom: 24,
  },
  goShopBtn: {
    backgroundColor: '#E53935',
    borderRadius: 20,
    paddingHorizontal: 32,
    paddingVertical: 12,
  },
  goShopBtnText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
});
