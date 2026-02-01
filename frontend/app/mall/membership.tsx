import { useRouter } from 'expo-router';
import { Users, Crown, Star, TrendingUp, Store, ChevronRight } from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import type { Shop, MemberInfo, MemberLevel } from '@/src/types/sharemall';
import { useWalletStore } from '@/src/stores/wallet';

const LEVEL_CONFIG: Record<MemberLevel, { label: string; color: string; icon: any; bg: string }> = {
  Normal: { label: '普通会员', color: '#757575', icon: Users, bg: '#ECEFF1' },
  Silver: { label: '白银会员', color: '#9E9E9E', icon: Star, bg: '#F5F5F5' },
  Gold: { label: '黄金会员', color: '#FFB300', icon: Star, bg: '#FFF8E1' },
  Platinum: { label: '铂金会员', color: '#7B1FA2', icon: Crown, bg: '#F3E5F5' },
  Diamond: { label: '钻石会员', color: '#1976D2', icon: Crown, bg: '#E3F2FD' },
};

interface ShopMembership {
  shop: Shop;
  member: MemberInfo;
}

export default function MembershipScreen() {
  const router = useRouter();
  const { currentAccount } = useWalletStore();
  const [memberships, setMemberships] = useState<ShopMembership[]>([]);
  const [refreshing, setRefreshing] = useState(false);
  const [totalCommission, setTotalCommission] = useState(BigInt(0));

  const loadData = async () => {
    if (!currentAccount) return;
    try {
      const shops = await sharemallService.getShopList(50);
      const memberList: ShopMembership[] = [];
      let commission = BigInt(0);

      for (const shop of shops) {
        const member = await sharemallService.getMemberInfo(shop.id, currentAccount.address);
        if (member) {
          memberList.push({ shop, member });
          commission += BigInt(member.totalCommission);
        }
      }

      setMemberships(memberList);
      setTotalCommission(commission);
    } catch (error) {
      console.error('Failed to load memberships:', error);
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

  const renderMembershipItem = ({ item }: { item: ShopMembership }) => {
    const levelConfig = LEVEL_CONFIG[item.member.level];
    const LevelIcon = levelConfig.icon;

    return (
      <TouchableOpacity
        style={styles.memberCard}
        onPress={() => router.push(`/mall/shop/${item.shop.id}` as any)}
      >
        <View style={styles.cardHeader}>
          <View style={styles.shopInfo}>
            <View style={styles.shopIcon}>
              <Store size={20} color="#666" />
            </View>
            <Text style={styles.shopName}>{item.shop.name}</Text>
          </View>
          <View style={[styles.levelBadge, { backgroundColor: levelConfig.bg }]}>
            <LevelIcon size={14} color={levelConfig.color} />
            <Text style={[styles.levelText, { color: levelConfig.color }]}>
              {levelConfig.label}
            </Text>
          </View>
        </View>

        <View style={styles.statsRow}>
          <View style={styles.statItem}>
            <Text style={styles.statValue}>{formatBalance(item.member.totalSpent)}</Text>
            <Text style={styles.statLabel}>累计消费</Text>
          </View>
          <View style={styles.statDivider} />
          <View style={styles.statItem}>
            <Text style={styles.statValue}>{item.member.referralCount}</Text>
            <Text style={styles.statLabel}>推荐人数</Text>
          </View>
          <View style={styles.statDivider} />
          <View style={styles.statItem}>
            <Text style={[styles.statValue, { color: '#388E3C' }]}>
              {formatBalance(item.member.totalCommission)}
            </Text>
            <Text style={styles.statLabel}>返佣收益</Text>
          </View>
        </View>

        {item.member.referrer && (
          <View style={styles.referrerRow}>
            <Text style={styles.referrerLabel}>推荐人:</Text>
            <Text style={styles.referrerAddress}>
              {item.member.referrer.slice(0, 10)}...{item.member.referrer.slice(-6)}
            </Text>
          </View>
        )}
      </TouchableOpacity>
    );
  };

  return (
    <View style={styles.container}>
      {/* 总收益 */}
      <View style={styles.totalCard}>
        <View style={styles.totalHeader}>
          <TrendingUp size={24} color="#388E3C" />
          <Text style={styles.totalTitle}>返佣总收益</Text>
        </View>
        <Text style={styles.totalValue}>
          {formatBalance(totalCommission.toString())} COS
        </Text>
        <Text style={styles.totalHint}>
          已加入 {memberships.length} 个店铺会员
        </Text>
      </View>

      {/* 会员列表 */}
      <FlatList
        data={memberships}
        renderItem={renderMembershipItem}
        keyExtractor={(item) => item.shop.id.toString()}
        contentContainerStyle={styles.listContent}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
        ListEmptyComponent={
          <View style={styles.emptyContainer}>
            <Users size={48} color="#ccc" />
            <Text style={styles.emptyText}>暂无会员身份</Text>
            <Text style={styles.emptyHint}>在店铺购物后自动成为会员</Text>
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
  totalCard: {
    backgroundColor: '#fff',
    margin: 12,
    borderRadius: 12,
    padding: 20,
  },
  totalHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  totalTitle: {
    fontSize: 14,
    color: '#666',
    marginLeft: 8,
  },
  totalValue: {
    fontSize: 32,
    fontWeight: '700',
    color: '#388E3C',
  },
  totalHint: {
    fontSize: 12,
    color: '#999',
    marginTop: 8,
  },
  listContent: {
    paddingHorizontal: 12,
    paddingBottom: 12,
  },
  memberCard: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  cardHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  shopInfo: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  shopIcon: {
    width: 36,
    height: 36,
    borderRadius: 18,
    backgroundColor: '#f5f5f5',
    alignItems: 'center',
    justifyContent: 'center',
  },
  shopName: {
    fontSize: 15,
    fontWeight: '500',
    color: '#333',
    marginLeft: 10,
  },
  levelBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
  },
  levelText: {
    fontSize: 12,
    marginLeft: 4,
  },
  statsRow: {
    flexDirection: 'row',
    paddingVertical: 12,
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
  },
  statItem: {
    flex: 1,
    alignItems: 'center',
  },
  statDivider: {
    width: 1,
    backgroundColor: '#f0f0f0',
  },
  statValue: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  statLabel: {
    fontSize: 11,
    color: '#999',
    marginTop: 4,
  },
  referrerRow: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
  },
  referrerLabel: {
    fontSize: 12,
    color: '#999',
  },
  referrerAddress: {
    fontSize: 12,
    color: '#666',
    marginLeft: 4,
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
});
