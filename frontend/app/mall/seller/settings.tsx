import { useRouter } from 'expo-router';
import {
  Store,
  Image as ImageIcon,
  FileText,
  Wallet,
  Power,
  AlertTriangle,
} from 'lucide-react-native';
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
import type { Shop, ShopFundInfo } from '@/src/types/sharemall';
import { useWalletStore } from '@/src/stores/wallet';

export default function ShopSettingsScreen() {
  const router = useRouter();
  const { currentAccount } = useWalletStore();

  const [myShop, setMyShop] = useState<Shop | null>(null);
  const [fundInfo, setFundInfo] = useState<ShopFundInfo | null>(null);
  const [name, setName] = useState('');
  const [logoCid, setLogoCid] = useState('');
  const [descriptionCid, setDescriptionCid] = useState('');
  const [depositAmount, setDepositAmount] = useState('');
  const [loading, setLoading] = useState(false);

  const loadData = async () => {
    if (!currentAccount) return;
    try {
      const shops = await sharemallService.getShopList(100);
      const userShop = shops.find((s) => s.owner === currentAccount.address);
      setMyShop(userShop || null);

      if (userShop) {
        setName(userShop.name);
        setLogoCid(userShop.logoCid || '');
        setDescriptionCid(userShop.descriptionCid || '');

        const fund = await sharemallService.getShopFundInfo(userShop.id);
        setFundInfo(fund);
      }
    } catch (error) {
      console.error('Failed to load shop:', error);
    }
  };

  useEffect(() => {
    loadData();
  }, [currentAccount]);

  const formatBalance = (balance: string) => {
    const num = BigInt(balance);
    return (Number(num) / 1e10).toFixed(2);
  };

  const handleUpdateShop = async () => {
    if (!myShop) return;

    setLoading(true);
    try {
      await sharemallTxService.updateShop(
        myShop.id,
        name.trim() || undefined,
        logoCid.trim() || undefined,
        descriptionCid.trim() || undefined
      );
      Alert.alert('成功', '店铺信息已更新');
      loadData();
    } catch (error: any) {
      Alert.alert('错误', error.message || '更新失败');
    } finally {
      setLoading(false);
    }
  };

  const handleDeposit = async () => {
    if (!myShop || !depositAmount) {
      Alert.alert('提示', '请输入充值金额');
      return;
    }

    setLoading(true);
    try {
      const amount = (parseFloat(depositAmount) * 1e10).toString();
      await sharemallTxService.depositShopFund(myShop.id, amount);
      Alert.alert('成功', '充值成功');
      setDepositAmount('');
      loadData();
    } catch (error: any) {
      Alert.alert('错误', error.message || '充值失败');
    } finally {
      setLoading(false);
    }
  };

  const handleSuspend = () => {
    if (!myShop) return;

    Alert.alert(
      '暂停营业',
      '暂停后店铺将不可见，确定要暂停吗？',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确定',
          onPress: async () => {
            try {
              await sharemallTxService.suspendShop(myShop.id);
              Alert.alert('成功', '店铺已暂停营业');
              loadData();
            } catch (error: any) {
              Alert.alert('错误', error.message || '操作失败');
            }
          },
        },
      ]
    );
  };

  const handleResume = async () => {
    if (!myShop) return;

    try {
      await sharemallTxService.resumeShop(myShop.id);
      Alert.alert('成功', '店铺已恢复营业');
      loadData();
    } catch (error: any) {
      Alert.alert('错误', error.message || '操作失败');
    }
  };

  if (!myShop) {
    return (
      <View style={styles.loadingContainer}>
        <Text>加载中...</Text>
      </View>
    );
  }

  const fundHealthColor =
    fundInfo?.health === 'Healthy'
      ? '#388E3C'
      : fundInfo?.health === 'Warning'
      ? '#F57C00'
      : '#D32F2F';

  return (
    <ScrollView style={styles.container}>
      {/* 基本信息 */}
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>基本信息</Text>

        <View style={styles.inputGroup}>
          <View style={styles.labelRow}>
            <Store size={16} color="#666" />
            <Text style={styles.label}>店铺名称</Text>
          </View>
          <TextInput
            style={styles.input}
            placeholder="输入店铺名称"
            value={name}
            onChangeText={setName}
          />
        </View>

        <View style={styles.inputGroup}>
          <View style={styles.labelRow}>
            <ImageIcon size={16} color="#666" />
            <Text style={styles.label}>店铺 Logo (IPFS CID)</Text>
          </View>
          <TextInput
            style={styles.input}
            placeholder="输入 IPFS CID"
            value={logoCid}
            onChangeText={setLogoCid}
          />
        </View>

        <View style={styles.inputGroup}>
          <View style={styles.labelRow}>
            <FileText size={16} color="#666" />
            <Text style={styles.label}>店铺简介 (IPFS CID)</Text>
          </View>
          <TextInput
            style={styles.input}
            placeholder="输入 IPFS CID"
            value={descriptionCid}
            onChangeText={setDescriptionCid}
          />
        </View>

        <TouchableOpacity
          style={[styles.updateBtn, loading && styles.btnDisabled]}
          onPress={handleUpdateShop}
          disabled={loading}
        >
          <Text style={styles.updateBtnText}>
            {loading ? '更新中...' : '更新信息'}
          </Text>
        </TouchableOpacity>
      </View>

      {/* 运营资金 */}
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>运营资金</Text>

        <View style={styles.fundCard}>
          <View style={styles.fundHeader}>
            <Wallet size={24} color="#1976D2" />
            <View style={styles.fundInfo}>
              <Text style={styles.fundBalance}>
                {fundInfo ? formatBalance(fundInfo.balance) : '0.00'} COS
              </Text>
              <View style={[styles.healthBadge, { backgroundColor: fundHealthColor + '20' }]}>
                <Text style={[styles.healthText, { color: fundHealthColor }]}>
                  {fundInfo?.health || 'Unknown'}
                </Text>
              </View>
            </View>
          </View>

          {fundInfo?.health !== 'Healthy' && (
            <View style={styles.fundWarning}>
              <AlertTriangle size={14} color="#F57C00" />
              <Text style={styles.fundWarningText}>
                运营资金不足，请及时充值以避免店铺被暂停
              </Text>
            </View>
          )}
        </View>

        <View style={styles.depositRow}>
          <TextInput
            style={[styles.input, { flex: 1, marginRight: 12 }]}
            placeholder="充值金额 (COS)"
            keyboardType="decimal-pad"
            value={depositAmount}
            onChangeText={setDepositAmount}
          />
          <TouchableOpacity
            style={[styles.depositBtn, loading && styles.btnDisabled]}
            onPress={handleDeposit}
            disabled={loading}
          >
            <Text style={styles.depositBtnText}>充值</Text>
          </TouchableOpacity>
        </View>
      </View>

      {/* 店铺状态 */}
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>店铺状态</Text>

        <View style={styles.statusCard}>
          <View style={styles.statusInfo}>
            <View
              style={[
                styles.statusDot,
                { backgroundColor: myShop.status === 'Active' ? '#388E3C' : '#F57C00' },
              ]}
            />
            <Text style={styles.statusText}>
              {myShop.status === 'Active' ? '营业中' : myShop.status}
            </Text>
          </View>

          {myShop.status === 'Active' ? (
            <TouchableOpacity style={styles.suspendBtn} onPress={handleSuspend}>
              <Power size={16} color="#F57C00" />
              <Text style={styles.suspendBtnText}>暂停营业</Text>
            </TouchableOpacity>
          ) : myShop.status === 'Suspended' ? (
            <TouchableOpacity style={styles.resumeBtn} onPress={handleResume}>
              <Power size={16} color="#388E3C" />
              <Text style={styles.resumeBtnText}>恢复营业</Text>
            </TouchableOpacity>
          ) : null}
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
  section: {
    backgroundColor: '#fff',
    margin: 12,
    marginBottom: 0,
    borderRadius: 12,
    padding: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 16,
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
    marginLeft: 6,
  },
  input: {
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 12,
    fontSize: 14,
  },
  updateBtn: {
    backgroundColor: '#1976D2',
    borderRadius: 8,
    paddingVertical: 12,
    alignItems: 'center',
  },
  updateBtnText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
  btnDisabled: {
    opacity: 0.6,
  },
  fundCard: {
    backgroundColor: '#f9f9f9',
    borderRadius: 8,
    padding: 16,
    marginBottom: 16,
  },
  fundHeader: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  fundInfo: {
    marginLeft: 12,
    flex: 1,
  },
  fundBalance: {
    fontSize: 24,
    fontWeight: '700',
    color: '#333',
  },
  healthBadge: {
    alignSelf: 'flex-start',
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 4,
    marginTop: 4,
  },
  healthText: {
    fontSize: 11,
  },
  fundWarning: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 12,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#eee',
  },
  fundWarningText: {
    fontSize: 12,
    color: '#F57C00',
    marginLeft: 8,
    flex: 1,
  },
  depositRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  depositBtn: {
    backgroundColor: '#388E3C',
    borderRadius: 8,
    paddingHorizontal: 20,
    paddingVertical: 12,
  },
  depositBtnText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
  statusCard: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    backgroundColor: '#f9f9f9',
    borderRadius: 8,
    padding: 16,
  },
  statusInfo: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  statusDot: {
    width: 10,
    height: 10,
    borderRadius: 5,
    marginRight: 8,
  },
  statusText: {
    fontSize: 14,
    color: '#333',
  },
  suspendBtn: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 16,
    backgroundColor: '#FFF3E0',
  },
  suspendBtnText: {
    fontSize: 13,
    color: '#F57C00',
    marginLeft: 4,
  },
  resumeBtn: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 16,
    backgroundColor: '#E8F5E9',
  },
  resumeBtnText: {
    fontSize: 13,
    color: '#388E3C',
    marginLeft: 4,
  },
});
