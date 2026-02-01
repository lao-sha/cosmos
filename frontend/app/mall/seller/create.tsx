import { useRouter } from 'expo-router';
import { Store, Image as ImageIcon, FileText } from 'lucide-react-native';
import React, { useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  TextInput,
  Alert,
} from 'react-native';

import { sharemallTxService } from '@/src/services/sharemall-tx';
import { useWalletStore } from '@/src/stores/wallet';

export default function CreateShopScreen() {
  const router = useRouter();
  const { currentAccount } = useWalletStore();

  const [name, setName] = useState('');
  const [logoCid, setLogoCid] = useState('');
  const [descriptionCid, setDescriptionCid] = useState('');
  const [loading, setLoading] = useState(false);

  const handleCreate = async () => {
    if (!currentAccount) {
      Alert.alert('提示', '请先登录钱包');
      return;
    }
    if (!name.trim()) {
      Alert.alert('提示', '请输入店铺名称');
      return;
    }

    setLoading(true);
    try {
      await sharemallTxService.createShop(
        name.trim(),
        logoCid.trim() || undefined,
        descriptionCid.trim() || undefined
      );
      Alert.alert('成功', '店铺创建成功', [
        { text: '确定', onPress: () => router.replace('/mall/seller' as any) },
      ]);
    } catch (error: any) {
      Alert.alert('错误', error.message || '创建失败');
    } finally {
      setLoading(false);
    }
  };

  return (
    <ScrollView style={styles.container}>
      <View style={styles.header}>
        <View style={styles.iconContainer}>
          <Store size={48} color="#1976D2" />
        </View>
        <Text style={styles.title}>创建您的店铺</Text>
        <Text style={styles.subtitle}>
          创建店铺需要支付运营资金押金 (最低 10 COS)
        </Text>
      </View>

      <View style={styles.form}>
        <View style={styles.inputGroup}>
          <Text style={styles.label}>店铺名称 *</Text>
          <TextInput
            style={styles.input}
            placeholder="输入店铺名称"
            value={name}
            onChangeText={setName}
            maxLength={50}
          />
          <Text style={styles.hint}>{name.length}/50</Text>
        </View>

        <View style={styles.inputGroup}>
          <View style={styles.labelRow}>
            <ImageIcon size={16} color="#666" />
            <Text style={styles.label}>店铺 Logo (IPFS CID)</Text>
          </View>
          <TextInput
            style={styles.input}
            placeholder="输入 IPFS CID (可选)"
            value={logoCid}
            onChangeText={setLogoCid}
          />
          <Text style={styles.hint}>上传图片到 IPFS 后填入 CID</Text>
        </View>

        <View style={styles.inputGroup}>
          <View style={styles.labelRow}>
            <FileText size={16} color="#666" />
            <Text style={styles.label}>店铺简介 (IPFS CID)</Text>
          </View>
          <TextInput
            style={styles.input}
            placeholder="输入 IPFS CID (可选)"
            value={descriptionCid}
            onChangeText={setDescriptionCid}
          />
          <Text style={styles.hint}>上传简介文档到 IPFS 后填入 CID</Text>
        </View>

        <View style={styles.notice}>
          <Text style={styles.noticeTitle}>创建须知</Text>
          <Text style={styles.noticeText}>
            • 创建店铺需要支付运营资金押金{'\n'}
            • 运营资金用于支付平台手续费{'\n'}
            • 资金不足时店铺将被暂停{'\n'}
            • 关闭店铺可退回剩余资金
          </Text>
        </View>

        <TouchableOpacity
          style={[styles.submitBtn, loading && styles.submitBtnDisabled]}
          onPress={handleCreate}
          disabled={loading}
        >
          <Text style={styles.submitBtnText}>
            {loading ? '创建中...' : '创建店铺'}
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
    padding: 32,
    alignItems: 'center',
  },
  iconContainer: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: '#E3F2FD',
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 16,
  },
  title: {
    fontSize: 20,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 14,
    color: '#666',
    textAlign: 'center',
  },
  form: {
    backgroundColor: '#fff',
    marginTop: 8,
    padding: 16,
  },
  inputGroup: {
    marginBottom: 20,
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
    marginBottom: 8,
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
    marginTop: 4,
  },
  notice: {
    backgroundColor: '#FFF8E1',
    borderRadius: 8,
    padding: 16,
    marginBottom: 20,
  },
  noticeTitle: {
    fontSize: 14,
    fontWeight: '500',
    color: '#F57C00',
    marginBottom: 8,
  },
  noticeText: {
    fontSize: 13,
    color: '#666',
    lineHeight: 22,
  },
  submitBtn: {
    backgroundColor: '#1976D2',
    borderRadius: 8,
    paddingVertical: 14,
    alignItems: 'center',
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
