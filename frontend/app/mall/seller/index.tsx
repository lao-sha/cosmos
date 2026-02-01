import { useRouter } from 'expo-router';
import {
  Store,
  Package,
  ShoppingCart,
  Coins,
  Users,
  Settings,
  Plus,
  TrendingUp,
  AlertCircle,
} from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import type { Shop, ShopFundInfo, MallOrder } from '@/src/types/sharemall';
import { useWalletStore } from '@/src/stores/wallet';

export default function SellerHomeScreen() {
  const router = useRouter();
  const { currentAccount } = useWalletStore();
  const [myShop, setMyShop] = useState<Shop | null>(null);
  const [fundInfo, setFundInfo] = useState<ShopFundInfo | null>(null);
  const [pendingOrders, setPendingOrders] = useState<MallOrder[]>([]);
  const [refreshing, setRefreshing] = useState(false);
  const [loading, setLoading] = useState(true);

  const loadData = async () => {
    if (!currentAccount) return;
    try {
      // 查找用户的店铺
      const shops = await sharemallService.getShopList(100);
      const userShop = shops.find((s) => s.owner === currentAccount.address);
      setMyShop(userShop || null);

      if (userShop) {
        const [fund, orders] = await Promise.all([
          sharemallService.getShopFundInfo(userShop.id),
          sharemallService.getShopOrders(userShop.id),
        ]);
        setFundInfo(fund);
        setPendingOrders(orders.filter((o) => o.status === 'Paid'));
      }
    } catch (error) {
      console.error('Failed to load seller data:', error);
    } finally {
      setLoading(false);
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

  const formatBalance = (balance: string) => {
    const num = BigInt(balance);
    return (Number(num) / 1e10).toFixed(2);
  };

  if (loading) {
    return (
      <View style={styles.loadingContainer}>
        <Text>加载中...</Text>
      </View>
    );
  }

  // 未创建店铺
  if (!myShop) {
    return (
      <View style={styles.noShopContainer}>
        <Store size={64} color="#ccc" />
        <Text style={styles.noShopTitle}>您还没有店铺</Text>
        <Text style={styles.noShopDesc}>创建店铺，开始您的电商之旅</Text>
        <TouchableOpacity
          style={styles.createBtn}
          onPress={() => router.push('/mall/seller/create' as any)}
        >
          <Plus size={20} color="#fff" />
          <Text style={styles.createBtnText}>创建店铺</Text>
        </TouchableOpacity>
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
    <ScrollView
      style={styles.container}
      refreshControl={<RefreshControl refreshing={refreshing} onRefresh={onRefresh} />}
    >
      {/* 店铺信息卡片 */}
      <View style={styles.shopCard}>
        <View style={styles.shopHeader}>
          <View style={styles.shopLogo}>
            <Store size={32} color="#1976D2" />
          </View>
          <View style={styles.shopInfo}>
            <Text style={styles.shopName}>{myShop.name}</Text>
            <View style={styles.shopStatus}>
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
          </View>
          <TouchableOpacity
            style={styles.settingsBtn}
            onPress={() => router.push('/mall/seller/settings' as any)}
          >
            <Settings size={20} color="#666" />
          </TouchableOpacity>
        </View>

        {/* 运营资金 */}
        <View style={styles.fundSection}>
          <View style={styles.fundHeader}>
            <Text style={styles.fundLabel}>运营资金</Text>
            <View style={[styles.healthBadge, { backgroundColor: fundHealthColor + '20' }]}>
              <Text style={[styles.healthText, { color: fundHealthColor }]}>
                {fundInfo?.health || 'Unknown'}
              </Text>
            </View>
          </View>
          <Text style={styles.fundBalance}>
            {fundInfo ? formatBalance(fundInfo.balance) : '0.00'} COS
          </Text>
          {fundInfo?.health !== 'Healthy' && (
            <View style={styles.fundWarning}>
              <AlertCircle size={14} color="#F57C00" />
              <Text style={styles.fundWarningText}>
                运营资金不足，请及时充值
              </Text>
            </View>
          )}
        </View>
      </View>

      {/* 数据概览 */}
      <View style={styles.statsCard}>
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{myShop.totalOrders}</Text>
          <Text style={styles.statLabel}>总订单</Text>
        </View>
        <View style={styles.statDivider} />
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{myShop.productCount}</Text>
          <Text style={styles.statLabel}>商品数</Text>
        </View>
        <View style={styles.statDivider} />
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{formatBalance(myShop.totalSales)}</Text>
          <Text style={styles.statLabel}>销售额</Text>
        </View>
      </View>

      {/* 待处理事项 */}
      {pendingOrders.length > 0 && (
        <TouchableOpacity
          style={styles.pendingCard}
          onPress={() => router.push('/mall/seller/orders' as any)}
        >
          <ShoppingCart size={20} color="#F57C00" />
          <Text style={styles.pendingText}>
            您有 {pendingOrders.length} 个订单待发货
          </Text>
          <Text style={styles.pendingArrow}>›</Text>
        </TouchableOpacity>
      )}

      {/* 功能入口 */}
      <View style={styles.menuSection}>
        <Text style={styles.menuTitle}>店铺管理</Text>
        <View style={styles.menuGrid}>
          <TouchableOpacity
            style={styles.menuItem}
            onPress={() => router.push('/mall/seller/products' as any)}
          >
            <View style={[styles.menuIcon, { backgroundColor: '#E3F2FD' }]}>
              <Package size={24} color="#1976D2" />
            </View>
            <Text style={styles.menuText}>商品管理</Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={styles.menuItem}
            onPress={() => router.push('/mall/seller/orders' as any)}
          >
            <View style={[styles.menuIcon, { backgroundColor: '#FFF3E0' }]}>
              <ShoppingCart size={24} color="#F57C00" />
            </View>
            <Text style={styles.menuText}>订单管理</Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={styles.menuItem}
            onPress={() => router.push('/mall/seller/token' as any)}
          >
            <View style={[styles.menuIcon, { backgroundColor: '#E8F5E9' }]}>
              <Coins size={24} color="#388E3C" />
            </View>
            <Text style={styles.menuText}>代币管理</Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={styles.menuItem}
            onPress={() => router.push('/mall/seller/members' as any)}
          >
            <View style={[styles.menuIcon, { backgroundColor: '#FCE4EC' }]}>
              <Users size={24} color="#C2185B" />
            </View>
            <Text style={styles.menuText}>会员管理</Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={styles.menuItem}
            onPress={() => router.push('/mall/seller/commission' as any)}
          >
            <View style={[styles.menuIcon, { backgroundColor: '#F3E5F5' }]}>
              <TrendingUp size={24} color="#7B1FA2" />
            </View>
            <Text style={styles.menuText}>返佣配置</Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={styles.menuItem}
            onPress={() => router.push('/mall/seller/settings' as any)}
          >
            <View style={[styles.menuIcon, { backgroundColor: '#ECEFF1' }]}>
              <Settings size={24} color="#546E7A" />
            </View>
            <Text style={styles.menuText}>店铺设置</Text>
          </TouchableOpacity>
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
  noShopContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 32,
  },
  noShopTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginTop: 16,
  },
  noShopDesc: {
    fontSize: 14,
    color: '#666',
    marginTop: 8,
    marginBottom: 24,
  },
  createBtn: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#1976D2',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 24,
  },
  createBtnText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '500',
    marginLeft: 8,
  },
  shopCard: {
    backgroundColor: '#fff',
    margin: 12,
    borderRadius: 12,
    padding: 16,
  },
  shopHeader: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  shopLogo: {
    width: 56,
    height: 56,
    borderRadius: 28,
    backgroundColor: '#E3F2FD',
    alignItems: 'center',
    justifyContent: 'center',
  },
  shopInfo: {
    flex: 1,
    marginLeft: 12,
  },
  shopName: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  shopStatus: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 4,
  },
  statusDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    marginRight: 6,
  },
  statusText: {
    fontSize: 13,
    color: '#666',
  },
  settingsBtn: {
    padding: 8,
  },
  fundSection: {
    marginTop: 16,
    paddingTop: 16,
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
  },
  fundHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  fundLabel: {
    fontSize: 13,
    color: '#666',
  },
  healthBadge: {
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 4,
  },
  healthText: {
    fontSize: 11,
  },
  fundBalance: {
    fontSize: 28,
    fontWeight: '700',
    color: '#333',
    marginTop: 8,
  },
  fundWarning: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 8,
  },
  fundWarningText: {
    fontSize: 12,
    color: '#F57C00',
    marginLeft: 6,
  },
  statsCard: {
    backgroundColor: '#fff',
    marginHorizontal: 12,
    borderRadius: 12,
    padding: 16,
    flexDirection: 'row',
  },
  statItem: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 20,
    fontWeight: '600',
    color: '#333',
  },
  statLabel: {
    fontSize: 12,
    color: '#999',
    marginTop: 4,
  },
  statDivider: {
    width: 1,
    backgroundColor: '#eee',
    marginVertical: 4,
  },
  pendingCard: {
    backgroundColor: '#FFF3E0',
    marginHorizontal: 12,
    marginTop: 12,
    borderRadius: 12,
    padding: 16,
    flexDirection: 'row',
    alignItems: 'center',
  },
  pendingText: {
    flex: 1,
    fontSize: 14,
    color: '#F57C00',
    marginLeft: 12,
  },
  pendingArrow: {
    fontSize: 20,
    color: '#F57C00',
  },
  menuSection: {
    backgroundColor: '#fff',
    margin: 12,
    borderRadius: 12,
    padding: 16,
  },
  menuTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginBottom: 16,
  },
  menuGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
  },
  menuItem: {
    width: '33.33%',
    alignItems: 'center',
    marginBottom: 20,
  },
  menuIcon: {
    width: 48,
    height: 48,
    borderRadius: 12,
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 8,
  },
  menuText: {
    fontSize: 12,
    color: '#333',
  },
});
