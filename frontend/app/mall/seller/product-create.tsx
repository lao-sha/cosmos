import { useRouter } from 'expo-router';
import { Package, Image as ImageIcon, FileText, DollarSign, Layers } from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  TextInput,
  Alert,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import { sharemallTxService } from '@/src/services/sharemall-tx';
import type { Shop, ProductCategory } from '@/src/types/sharemall';
import { useWalletStore } from '@/src/stores/wallet';

const CATEGORIES: { value: ProductCategory; label: string }[] = [
  { value: 'Digital', label: '数字商品' },
  { value: 'Physical', label: '实物商品' },
  { value: 'Service', label: '服务' },
  { value: 'Other', label: '其他' },
];

export default function ProductCreateScreen() {
  const router = useRouter();
  const { currentAccount } = useWalletStore();

  const [myShop, setMyShop] = useState<Shop | null>(null);
  const [nameCid, setNameCid] = useState('');
  const [imagesCid, setImagesCid] = useState('');
  const [detailCid, setDetailCid] = useState('');
  const [price, setPrice] = useState('');
  const [stock, setStock] = useState('');
  const [category, setCategory] = useState<ProductCategory>('Digital');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const loadShop = async () => {
      if (!currentAccount) return;
      const shops = await sharemallService.getShopList(100);
      const userShop = shops.find((s) => s.owner === currentAccount.address);
      setMyShop(userShop || null);
    };
    loadShop();
  }, [currentAccount]);

  const handleCreate = async () => {
    if (!currentAccount || !myShop) {
      Alert.alert('提示', '请先创建店铺');
      return;
    }
    if (!nameCid.trim() || !price || !stock) {
      Alert.alert('提示', '请填写必填项');
      return;
    }

    setLoading(true);
    try {
      const priceValue = (parseFloat(price) * 1e10).toString();
      const stockValue = parseInt(stock, 10);

      await sharemallTxService.createProduct(
        myShop.id,
        nameCid.trim(),
        imagesCid.trim() || 'default',
        detailCid.trim() || 'default',
        priceValue,
        stockValue,
        category
      );

      Alert.alert('成功', '商品创建成功', [
        { text: '继续添加', onPress: () => resetForm() },
        { text: '返回列表', onPress: () => router.back() },
      ]);
    } catch (error: any) {
      Alert.alert('错误', error.message || '创建失败');
    } finally {
      setLoading(false);
    }
  };

  const resetForm = () => {
    setNameCid('');
    setImagesCid('');
    setDetailCid('');
    setPrice('');
    setStock('');
    setCategory('Digital');
  };

  return (
    <ScrollView style={styles.container}>
      <View style={styles.header}>
        <View style={styles.iconContainer}>
          <Package size={40} color="#1976D2" />
        </View>
        <Text style={styles.title}>发布新商品</Text>
      </View>

      <View style={styles.form}>
        <View style={styles.inputGroup}>
          <View style={styles.labelRow}>
            <FileText size={16} color="#666" />
            <Text style={styles.label}>商品名称 (IPFS CID) *</Text>
          </View>
          <TextInput
            style={styles.input}
            placeholder="输入商品名称的 IPFS CID"
            value={nameCid}
            onChangeText={setNameCid}
          />
        </View>

        <View style={styles.inputGroup}>
          <View style={styles.labelRow}>
            <ImageIcon size={16} color="#666" />
            <Text style={styles.label}>商品图片 (IPFS CID)</Text>
          </View>
          <TextInput
            style={styles.input}
            placeholder="输入商品图片的 IPFS CID"
            value={imagesCid}
            onChangeText={setImagesCid}
          />
        </View>

        <View style={styles.inputGroup}>
          <View style={styles.labelRow}>
            <FileText size={16} color="#666" />
            <Text style={styles.label}>商品详情 (IPFS CID)</Text>
          </View>
          <TextInput
            style={styles.input}
            placeholder="输入商品详情的 IPFS CID"
            value={detailCid}
            onChangeText={setDetailCid}
          />
        </View>

        <View style={styles.row}>
          <View style={[styles.inputGroup, { flex: 1, marginRight: 8 }]}>
            <View style={styles.labelRow}>
              <DollarSign size={16} color="#666" />
              <Text style={styles.label}>价格 (COS) *</Text>
            </View>
            <TextInput
              style={styles.input}
              placeholder="0.00"
              keyboardType="decimal-pad"
              value={price}
              onChangeText={setPrice}
            />
          </View>

          <View style={[styles.inputGroup, { flex: 1, marginLeft: 8 }]}>
            <View style={styles.labelRow}>
              <Layers size={16} color="#666" />
              <Text style={styles.label}>库存 *</Text>
            </View>
            <TextInput
              style={styles.input}
              placeholder="0"
              keyboardType="number-pad"
              value={stock}
              onChangeText={setStock}
            />
          </View>
        </View>

        <View style={styles.inputGroup}>
          <Text style={styles.label}>商品分类</Text>
          <View style={styles.categoryGrid}>
            {CATEGORIES.map((cat) => (
              <TouchableOpacity
                key={cat.value}
                style={[
                  styles.categoryBtn,
                  category === cat.value && styles.categoryBtnActive,
                ]}
                onPress={() => setCategory(cat.value)}
              >
                <Text
                  style={[
                    styles.categoryText,
                    category === cat.value && styles.categoryTextActive,
                  ]}
                >
                  {cat.label}
                </Text>
              </TouchableOpacity>
            ))}
          </View>
        </View>

        <TouchableOpacity
          style={[styles.submitBtn, loading && styles.submitBtnDisabled]}
          onPress={handleCreate}
          disabled={loading}
        >
          <Text style={styles.submitBtnText}>
            {loading ? '发布中...' : '发布商品'}
          </Text>
        </TouchableOpacity>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    backgroundColor: '#fff',
    padding: 24,
    alignItems: 'center',
  },
  iconContainer: {
    width: 72,
    height: 72,
    borderRadius: 36,
    backgroundColor: '#E3F2FD',
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 12,
  },
  title: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  form: {
    backgroundColor: '#fff',
    marginTop: 8,
    padding: 16,
  },
  inputGroup: {
    marginBottom: 16,
  },
  labelRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8,
  },
  label: {
    fontSize: 14,
    color: '#333',
    fontWeight: '500',
    marginLeft: 6,
  },
  input: {
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 12,
    fontSize: 14,
  },
  row: {
    flexDirection: 'row',
  },
  categoryGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    marginTop: 8,
  },
  categoryBtn: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 16,
    backgroundColor: '#f5f5f5',
    marginRight: 8,
    marginBottom: 8,
  },
  categoryBtnActive: {
    backgroundColor: '#1976D2',
  },
  categoryText: {
    fontSize: 13,
    color: '#666',
  },
  categoryTextActive: {
    color: '#fff',
  },
  submitBtn: {
    backgroundColor: '#1976D2',
    borderRadius: 8,
    paddingVertical: 14,
    alignItems: 'center',
    marginTop: 8,
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
