/**
 * 星尘玄鉴 - 交易记录页面
 * 显示钱包交易历史
 * 主题色：金棕色 #B2955D
 */

import { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Pressable,
  ScrollView,
  RefreshControl,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useWalletStore } from '@/stores';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

interface Transaction {
  id: string;
  type: 'send' | 'receive';
  amount: string;
  address: string;
  timestamp: Date;
  status: 'pending' | 'confirmed' | 'failed';
  hash: string;
}

export default function TransactionsPage() {
  const router = useRouter();
  const { address } = useWalletStore();

  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);

  const loadTransactions = async () => {
    // TODO: 从链上获取真实交易记录
    await new Promise(resolve => setTimeout(resolve, 1000));

    // 模拟空交易记录
    setTransactions([]);
    setIsLoading(false);
  };

  const handleRefresh = async () => {
    setIsRefreshing(true);
    await loadTransactions();
    setIsRefreshing(false);
  };

  useEffect(() => {
    loadTransactions();
  }, []);

  const formatAddress = (addr: string) => {
    if (!addr || addr.length < 16) return addr;
    return `${addr.slice(0, 8)}...${addr.slice(-8)}`;
  };

  const formatDate = (date: Date) => {
    return date.toLocaleDateString('zh-CN', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getStatusColor = (status: Transaction['status']) => {
    switch (status) {
      case 'confirmed':
        return '#27AE60';
      case 'pending':
        return '#F39C12';
      case 'failed':
        return '#E74C3C';
      default:
        return '#999';
    }
  };

  const getStatusText = (status: Transaction['status']) => {
    switch (status) {
      case 'confirmed':
        return '已确认';
      case 'pending':
        return '处理中';
      case 'failed':
        return '失败';
      default:
        return '';
    }
  };

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.navBar}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Text style={styles.navTitle}>交易记录</Text>
        <View style={styles.placeholder} />
      </View>

      {/* 钱包地址 */}
      <View style={styles.addressCard}>
        <Text style={styles.addressLabel}>当前地址</Text>
        <Text style={styles.addressText}>{formatAddress(address || '')}</Text>
      </View>

      {isLoading ? (
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color={THEME_COLOR} />
          <Text style={styles.loadingText}>加载中...</Text>
        </View>
      ) : transactions.length === 0 ? (
        <ScrollView
          contentContainerStyle={styles.emptyContainer}
          refreshControl={
            <RefreshControl
              refreshing={isRefreshing}
              onRefresh={handleRefresh}
              tintColor={THEME_COLOR}
            />
          }
        >
          <View style={styles.emptyIconCircle}>
            <Ionicons name="receipt-outline" size={48} color={THEME_COLOR} />
          </View>
          <Text style={styles.emptyTitle}>暂无交易记录</Text>
          <Text style={styles.emptySubtitle}>
            您的交易记录将显示在这里
          </Text>
          <Pressable
            style={styles.refreshButton}
            onPress={handleRefresh}
          >
            <Ionicons name="refresh-outline" size={18} color={THEME_COLOR} />
            <Text style={styles.refreshButtonText}>刷新</Text>
          </Pressable>
        </ScrollView>
      ) : (
        <ScrollView
          style={styles.listContainer}
          contentContainerStyle={styles.listContent}
          refreshControl={
            <RefreshControl
              refreshing={isRefreshing}
              onRefresh={handleRefresh}
              tintColor={THEME_COLOR}
            />
          }
        >
          {transactions.map((tx) => (
            <Pressable key={tx.id} style={styles.txItem}>
              <View style={[styles.txIcon, tx.type === 'send' ? styles.txIconSend : styles.txIconReceive]}>
                <Ionicons
                  name={tx.type === 'send' ? 'arrow-up' : 'arrow-down'}
                  size={20}
                  color={tx.type === 'send' ? '#E74C3C' : '#27AE60'}
                />
              </View>
              <View style={styles.txInfo}>
                <Text style={styles.txType}>
                  {tx.type === 'send' ? '转出' : '转入'}
                </Text>
                <Text style={styles.txAddress}>{formatAddress(tx.address)}</Text>
              </View>
              <View style={styles.txRight}>
                <Text
                  style={[
                    styles.txAmount,
                    { color: tx.type === 'send' ? '#E74C3C' : '#27AE60' },
                  ]}
                >
                  {tx.type === 'send' ? '-' : '+'}{tx.amount} STAR
                </Text>
                <View style={styles.txMeta}>
                  <View
                    style={[
                      styles.statusDot,
                      { backgroundColor: getStatusColor(tx.status) },
                    ]}
                  />
                  <Text style={styles.txTime}>{formatDate(tx.timestamp)}</Text>
                </View>
              </View>
            </Pressable>
          ))}
        </ScrollView>
      )}

      {/* 底部导航 - 全局统一 */}
      <View style={styles.bottomNav}>
        <Pressable style={styles.bottomNavItem} onPress={() => router.push('/' as any)}>
          <Text style={styles.bottomNavIcon}>🏠</Text>
          <Text style={styles.bottomNavLabel}>首页</Text>
        </Pressable>
        <Pressable style={styles.bottomNavItem} onPress={() => router.push('/divination' as any)}>
          <Text style={styles.bottomNavIcon}>🧭</Text>
          <Text style={styles.bottomNavLabel}>占卜</Text>
        </Pressable>
        <Pressable style={styles.bottomNavItem} onPress={() => router.push('/chat' as any)}>
          <Text style={styles.bottomNavIcon}>💬</Text>
          <Text style={styles.bottomNavLabel}>消息</Text>
        </Pressable>
        <Pressable style={[styles.bottomNavItem, styles.bottomNavItemActive]} onPress={() => router.push('/profile' as any)}>
          <Text style={styles.bottomNavIcon}>👤</Text>
          <Text style={[styles.bottomNavLabel, styles.bottomNavLabelActive]}>我的</Text>
        </Pressable>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME_BG,
    maxWidth: 414,
    width: '100%',
    alignSelf: 'center',
  },
  navBar: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingTop: 50,
    paddingHorizontal: 16,
    paddingBottom: 12,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  backButton: {
    padding: 4,
  },
  navTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#333',
  },
  placeholder: {
    width: 32,
  },
  addressCard: {
    backgroundColor: '#FFF',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  addressLabel: {
    fontSize: 12,
    color: '#8B6914',
    marginBottom: 6,
  },
  addressText: {
    fontSize: 14,
    color: '#333',
    fontFamily: 'monospace',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    gap: 16,
  },
  loadingText: {
    fontSize: 15,
    color: '#999',
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 40,
  },
  emptyIconCircle: {
    width: 100,
    height: 100,
    borderRadius: 50,
    backgroundColor: '#FFF',
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 24,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 8,
    elevation: 3,
  },
  emptyTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  emptySubtitle: {
    fontSize: 14,
    color: '#999',
    textAlign: 'center',
    marginBottom: 24,
  },
  refreshButton: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    backgroundColor: '#FFF',
    paddingHorizontal: 20,
    paddingVertical: 10,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: THEME_COLOR,
  },
  refreshButtonText: {
    fontSize: 14,
    color: THEME_COLOR,
    fontWeight: '500',
  },
  listContainer: {
    flex: 1,
  },
  listContent: {
    padding: 16,
    paddingBottom: 40,
  },
  txItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 14,
    marginBottom: 10,
    gap: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.04,
    shadowRadius: 4,
    elevation: 1,
  },
  txIcon: {
    width: 40,
    height: 40,
    borderRadius: 20,
    justifyContent: 'center',
    alignItems: 'center',
  },
  txIconSend: {
    backgroundColor: '#FFEBEE',
  },
  txIconReceive: {
    backgroundColor: '#E8F5E9',
  },
  txInfo: {
    flex: 1,
  },
  txType: {
    fontSize: 15,
    fontWeight: '500',
    color: '#333',
    marginBottom: 4,
  },
  txAddress: {
    fontSize: 12,
    color: '#999',
    fontFamily: 'monospace',
  },
  txRight: {
    alignItems: 'flex-end',
  },
  txAmount: {
    fontSize: 15,
    fontWeight: '600',
    marginBottom: 4,
  },
  txMeta: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
  },
  statusDot: {
    width: 6,
    height: 6,
    borderRadius: 3,
  },
  txTime: {
    fontSize: 12,
    color: '#999',
  },
  bottomNav: {
    position: 'absolute',
    bottom: 0,
    left: '50%',
    transform: [{ translateX: -207 }],
    width: 414,
    flexDirection: 'row',
    justifyContent: 'space-around',
    alignItems: 'center',
    backgroundColor: '#FFF',
    paddingTop: 8,
    paddingBottom: 8,
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
  },
  bottomNavItem: {
    alignItems: 'center',
    paddingVertical: 4,
    flex: 1,
  },
  bottomNavItemActive: {},
  bottomNavIcon: {
    fontSize: 22,
    marginBottom: 2,
  },
  bottomNavLabel: {
    fontSize: 12,
    color: '#999',
    fontWeight: '500',
  },
  bottomNavLabelActive: {
    color: THEME_COLOR,
  },
});
