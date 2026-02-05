import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
  RefreshControl,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Rocket, Clock, CheckCircle, Users } from 'lucide-react-native';
import { LinearGradient } from 'expo-linear-gradient';
import { useColors } from '@/hooks/useColors';
import { getTokenSales, type TokenSale, type SaleStatus } from '@/services/tokensale';
import { Card } from '@/components/ui';
import { Colors, Shadows } from '@/constants/colors';

const STATUS_CONFIG: Record<SaleStatus, { label: string; color: string }> = {
  upcoming: { label: '即将开始', color: Colors.warning },
  active: { label: '进行中', color: Colors.success },
  ended: { label: '已结束', color: '#6B7280' },
  cancelled: { label: '已取消', color: Colors.error },
};

export default function TokenSaleScreen() {
  const colors = useColors();
  const router = useRouter();

  const [sales, setSales] = useState<TokenSale[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setLoading(true);
    try {
      const data = await getTokenSales();
      setSales(data);
    } catch (error) {
      console.error('Failed to load token sales:', error);
    } finally {
      setLoading(false);
    }
  };

  const formatAmount = (value: string, decimals = 12): string => {
    const num = BigInt(value || '0') / BigInt(10 ** decimals);
    return num.toLocaleString();
  };

  const getTimeText = (sale: TokenSale): string => {
    const now = Date.now();
    if (sale.status === 'upcoming') {
      const diff = sale.startTime - now;
      const days = Math.floor(diff / (24 * 3600000));
      const hours = Math.floor((diff % (24 * 3600000)) / 3600000);
      return `${days}天${hours}小时后开始`;
    }
    if (sale.status === 'active') {
      const diff = sale.endTime - now;
      const days = Math.floor(diff / (24 * 3600000));
      const hours = Math.floor((diff % (24 * 3600000)) / 3600000);
      return `剩余 ${days}天${hours}小时`;
    }
    return '';
  };

  const getProgress = (sale: TokenSale): number => {
    const sold = BigInt(sale.soldAmount);
    const total = BigInt(sale.saleAmount);
    if (total === BigInt(0)) return 0;
    return Number((sold * BigInt(100)) / total);
  };

  const renderSale = ({ item }: { item: TokenSale }) => {
    const config = STATUS_CONFIG[item.status];
    const progress = getProgress(item);

    return (
      <TouchableOpacity
        activeOpacity={0.8}
        onPress={() => router.push(`/token/${item.id}`)}
      >
        <Card style={styles.saleCard}>
          {/* Header */}
          <View style={styles.saleHeader}>
            <View style={styles.tokenInfo}>
              <LinearGradient
                colors={['#6366F1', '#8B5CF6']}
                style={styles.tokenIcon}
              >
                <Text style={styles.tokenSymbolIcon}>
                  {item.tokenSymbol.charAt(0)}
                </Text>
              </LinearGradient>
              <View>
                <Text style={[styles.tokenName, { color: colors.textPrimary }]}>
                  {item.tokenName}
                </Text>
                <Text style={[styles.tokenSymbol, { color: colors.textSecondary }]}>
                  ${item.tokenSymbol}
                </Text>
              </View>
            </View>
            <View style={[styles.statusBadge, { backgroundColor: config.color + '20' }]}>
              <Text style={[styles.statusText, { color: config.color }]}>
                {config.label}
              </Text>
            </View>
          </View>

          {/* Description */}
          <Text style={[styles.description, { color: colors.textSecondary }]} numberOfLines={2}>
            {item.description}
          </Text>

          {/* Progress */}
          <View style={styles.progressSection}>
            <View style={styles.progressHeader}>
              <Text style={[styles.progressLabel, { color: colors.textSecondary }]}>
                募集进度
              </Text>
              <Text style={[styles.progressValue, { color: colors.textPrimary }]}>
                {progress.toFixed(1)}%
              </Text>
            </View>
            <View style={[styles.progressBar, { backgroundColor: colors.border }]}>
              <View
                style={[
                  styles.progressFill,
                  {
                    width: `${Math.min(progress, 100)}%`,
                    backgroundColor: item.status === 'active' ? Colors.success : Colors.primary,
                  },
                ]}
              />
            </View>
            <View style={styles.progressStats}>
              <Text style={[styles.progressStat, { color: colors.textTertiary }]}>
                {formatAmount(item.soldAmount)} / {formatAmount(item.saleAmount)} {item.tokenSymbol}
              </Text>
            </View>
          </View>

          {/* Info Grid */}
          <View style={styles.infoGrid}>
            <View style={styles.infoItem}>
              <Text style={[styles.infoLabel, { color: colors.textTertiary }]}>
                价格
              </Text>
              <Text style={[styles.infoValue, { color: colors.textPrimary }]}>
                {(Number(item.price) / 10000).toFixed(4)} USDT
              </Text>
            </View>
            <View style={styles.infoItem}>
              <Text style={[styles.infoLabel, { color: colors.textTertiary }]}>
                参与人数
              </Text>
              <Text style={[styles.infoValue, { color: colors.textPrimary }]}>
                {item.participants.toLocaleString()}
              </Text>
            </View>
            <View style={styles.infoItem}>
              <Text style={[styles.infoLabel, { color: colors.textTertiary }]}>
                锁仓期
              </Text>
              <Text style={[styles.infoValue, { color: colors.textPrimary }]}>
                {item.vestingPeriod > 0 ? `${Math.floor(item.vestingPeriod / (24 * 3600000))}天` : '无'}
              </Text>
            </View>
          </View>

          {/* Footer */}
          {(item.status === 'upcoming' || item.status === 'active') && (
            <View style={styles.saleFooter}>
              <Clock size={14} color={colors.textTertiary} />
              <Text style={[styles.timeText, { color: colors.textTertiary }]}>
                {getTimeText(item)}
              </Text>
            </View>
          )}
        </Card>
      </TouchableOpacity>
    );
  };

  return (
    <View style={[styles.container, { backgroundColor: colors.background }]}>
      <View style={styles.header}>
        <Text style={[styles.title, { color: colors.textPrimary }]}>代币发售</Text>
        <Text style={[styles.subtitle, { color: colors.textSecondary }]}>
          参与项目早期投资
        </Text>
      </View>

      <FlatList
        data={sales}
        renderItem={renderSale}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.list}
        refreshControl={
          <RefreshControl refreshing={loading} onRefresh={loadData} />
        }
        ListEmptyComponent={
          <View style={styles.empty}>
            <Rocket size={48} color={colors.textTertiary} />
            <Text style={[styles.emptyText, { color: colors.textSecondary }]}>
              {loading ? '加载中...' : '暂无发售项目'}
            </Text>
          </View>
        }
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  header: {
    padding: 16,
    paddingBottom: 8,
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
  },
  subtitle: {
    fontSize: 14,
    marginTop: 4,
  },
  list: {
    padding: 16,
    paddingTop: 8,
  },
  saleCard: {
    marginBottom: 16,
  },
  saleHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  tokenInfo: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  tokenIcon: {
    width: 44,
    height: 44,
    borderRadius: 22,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  tokenSymbolIcon: {
    color: '#FFFFFF',
    fontSize: 20,
    fontWeight: '700',
  },
  tokenName: {
    fontSize: 16,
    fontWeight: '600',
  },
  tokenSymbol: {
    fontSize: 13,
    marginTop: 2,
  },
  statusBadge: {
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
  },
  statusText: {
    fontSize: 12,
    fontWeight: '500',
  },
  description: {
    fontSize: 14,
    lineHeight: 20,
    marginBottom: 16,
  },
  progressSection: {
    marginBottom: 16,
  },
  progressHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  progressLabel: {
    fontSize: 13,
  },
  progressValue: {
    fontSize: 13,
    fontWeight: '600',
  },
  progressBar: {
    height: 8,
    borderRadius: 4,
    overflow: 'hidden',
  },
  progressFill: {
    height: '100%',
    borderRadius: 4,
  },
  progressStats: {
    marginTop: 6,
  },
  progressStat: {
    fontSize: 12,
  },
  infoGrid: {
    flexDirection: 'row',
    marginBottom: 12,
  },
  infoItem: {
    flex: 1,
  },
  infoLabel: {
    fontSize: 12,
    marginBottom: 4,
  },
  infoValue: {
    fontSize: 14,
    fontWeight: '600',
  },
  saleFooter: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: 'rgba(0,0,0,0.05)',
  },
  timeText: {
    fontSize: 13,
  },
  empty: {
    alignItems: 'center',
    paddingTop: 60,
  },
  emptyText: {
    fontSize: 16,
    marginTop: 12,
  },
});
