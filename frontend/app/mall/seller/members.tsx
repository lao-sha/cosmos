import { Users, Crown, Star, TrendingUp } from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  FlatList,
  StyleSheet,
  RefreshControl,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import type { Shop, MemberInfo, MemberLevel } from '@/src/types/sharemall';
import { useWalletStore } from '@/src/stores/wallet';

const LEVEL_CONFIG: Record<MemberLevel, { label: string; color: string; icon: any }> = {
  Normal: { label: '普通会员', color: '#757575', icon: Users },
  Silver: { label: '白银会员', color: '#9E9E9E', icon: Star },
  Gold: { label: '黄金会员', color: '#FFB300', icon: Star },
  Platinum: { label: '铂金会员', color: '#7B1FA2', icon: Crown },
  Diamond: { label: '钻石会员', color: '#1976D2', icon: Crown },
};

export default function MembersScreen() {
  const { currentAccount } = useWalletStore();
  const [myShop, setMyShop] = useState<Shop | null>(null);
  const [members, setMembers] = useState<MemberInfo[]>([]);
  const [refreshing, setRefreshing] = useState(false);
  const [stats, setStats] = useState({
    total: 0,
    silver: 0,
    gold: 0,
    platinum: 0,
    diamond: 0,
  });

  const loadData = async () => {
    if (!currentAccount) return;
    try {
      const shops = await sharemallService.getShopList(100);
      const userShop = shops.find((s) => s.owner === currentAccount.address);
      setMyShop(userShop || null);

      // 模拟会员数据（实际应从链上查询）
      if (userShop) {
        // TODO: 实现会员列表查询
        setMembers([]);
        setStats({
          total: 0,
          silver: 0,
          gold: 0,
          platinum: 0,
          diamond: 0,
        });
      }
    } catch (error) {
      console.error('Failed to load members:', error);
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

  const renderMemberItem = ({ item }: { item: MemberInfo }) => {
    const levelConfig = LEVEL_CONFIG[item.level];
    const LevelIcon = levelConfig.icon;

    return (
      <View style={styles.memberCard}>
        <View style={styles.memberHeader}>
          <View style={styles.memberAvatar}>
            <Users size={20} color="#999" />
          </View>
          <View style={styles.memberInfo}>
            <Text style={styles.memberAddress}>
              {item.account.slice(0, 10)}...{item.account.slice(-6)}
            </Text>
            <View style={[styles.levelBadge, { backgroundColor: levelConfig.color + '20' }]}>
              <LevelIcon size={12} color={levelConfig.color} />
              <Text style={[styles.levelText, { color: levelConfig.color }]}>
                {levelConfig.label}
              </Text>
            </View>
          </View>
        </View>

        <View style={styles.memberStats}>
          <View style={styles.memberStat}>
            <Text style={styles.statValue}>{formatBalance(item.totalSpent)}</Text>
            <Text style={styles.statLabel}>消费金额</Text>
          </View>
          <View style={styles.memberStat}>
            <Text style={styles.statValue}>{item.referralCount}</Text>
            <Text style={styles.statLabel}>推荐人数</Text>
          </View>
          <View style={styles.memberStat}>
            <Text style={styles.statValue}>{formatBalance(item.totalCommission)}</Text>
            <Text style={styles.statLabel}>返佣金额</Text>
          </View>
        </View>
      </View>
    );
  };

  return (
    <View style={styles.container}>
      {/* 会员统计 */}
      <View style={styles.statsCard}>
        <View style={styles.totalStat}>
          <Users size={32} color="#1976D2" />
          <View style={styles.totalInfo}>
            <Text style={styles.totalValue}>{stats.total}</Text>
            <Text style={styles.totalLabel}>总会员数</Text>
          </View>
        </View>

        <View style={styles.levelStats}>
          <View style={styles.levelStat}>
            <View style={[styles.levelDot, { backgroundColor: '#9E9E9E' }]} />
            <Text style={styles.levelStatValue}>{stats.silver}</Text>
            <Text style={styles.levelStatLabel}>白银</Text>
          </View>
          <View style={styles.levelStat}>
            <View style={[styles.levelDot, { backgroundColor: '#FFB300' }]} />
            <Text style={styles.levelStatValue}>{stats.gold}</Text>
            <Text style={styles.levelStatLabel}>黄金</Text>
          </View>
          <View style={styles.levelStat}>
            <View style={[styles.levelDot, { backgroundColor: '#7B1FA2' }]} />
            <Text style={styles.levelStatValue}>{stats.platinum}</Text>
            <Text style={styles.levelStatLabel}>铂金</Text>
          </View>
          <View style={styles.levelStat}>
            <View style={[styles.levelDot, { backgroundColor: '#1976D2' }]} />
            <Text style={styles.levelStatValue}>{stats.diamond}</Text>
            <Text style={styles.levelStatLabel}>钻石</Text>
          </View>
        </View>
      </View>

      {/* 会员列表 */}
      <FlatList
        data={members}
        renderItem={renderMemberItem}
        keyExtractor={(item) => item.account}
        contentContainerStyle={styles.listContent}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
        ListEmptyComponent={
          <View style={styles.emptyContainer}>
            <Users size={48} color="#ccc" />
            <Text style={styles.emptyText}>暂无会员</Text>
            <Text style={styles.emptyHint}>用户购买商品后自动成为会员</Text>
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
  statsCard: {
    backgroundColor: '#fff',
    margin: 12,
    borderRadius: 12,
    padding: 16,
  },
  totalStat: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 16,
  },
  totalInfo: {
    marginLeft: 16,
  },
  totalValue: {
    fontSize: 28,
    fontWeight: '700',
    color: '#333',
  },
  totalLabel: {
    fontSize: 13,
    color: '#999',
  },
  levelStats: {
    flexDirection: 'row',
    paddingTop: 16,
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
  },
  levelStat: {
    flex: 1,
    alignItems: 'center',
  },
  levelDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    marginBottom: 4,
  },
  levelStatValue: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  levelStatLabel: {
    fontSize: 11,
    color: '#999',
    marginTop: 2,
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
  memberHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  memberAvatar: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#f5f5f5',
    alignItems: 'center',
    justifyContent: 'center',
  },
  memberInfo: {
    flex: 1,
    marginLeft: 12,
  },
  memberAddress: {
    fontSize: 14,
    color: '#333',
    marginBottom: 4,
  },
  levelBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    alignSelf: 'flex-start',
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 4,
  },
  levelText: {
    fontSize: 11,
    marginLeft: 4,
  },
  memberStats: {
    flexDirection: 'row',
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#f5f5f5',
  },
  memberStat: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
  },
  statLabel: {
    fontSize: 11,
    color: '#999',
    marginTop: 2,
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
