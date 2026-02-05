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
import {
  Scale,
  Plus,
  Clock,
  FileText,
  AlertTriangle,
  CheckCircle,
} from 'lucide-react-native';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import {
  getDisputes,
  DISPUTE_STATUS_CONFIG,
  type Dispute,
  type DisputeStatus,
} from '@/services/dispute';
import { Card, Button } from '@/components/ui';
import { Colors } from '@/constants/colors';

export default function DisputesScreen() {
  const colors = useColors();
  const router = useRouter();
  const { address } = useWalletStore();

  const [disputes, setDisputes] = useState<Dispute[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<'all' | 'active' | 'resolved'>('all');

  useEffect(() => {
    loadData();
  }, [address]);

  const loadData = async () => {
    if (!address) return;
    setLoading(true);
    try {
      const data = await getDisputes(address);
      setDisputes(data);
    } catch (error) {
      console.error('Failed to load disputes:', error);
    } finally {
      setLoading(false);
    }
  };

  const filteredDisputes = disputes.filter((d) => {
    if (filter === 'active') {
      return d.status === 'pending' || d.status === 'evidence' || d.status === 'arbitrating';
    }
    if (filter === 'resolved') {
      return d.status === 'resolved' || d.status === 'appealed';
    }
    return true;
  });

  const formatAmount = (value: string): string => {
    const num = BigInt(value || '0') / BigInt(1e12);
    return num.toLocaleString();
  };

  const getTimeLeft = (deadline: number): string => {
    const diff = deadline - Date.now();
    if (diff <= 0) return '已截止';
    const hours = Math.floor(diff / 3600000);
    if (hours < 24) return `${hours}小时`;
    const days = Math.floor(hours / 24);
    return `${days}天${hours % 24}小时`;
  };

  const renderDispute = ({ item }: { item: Dispute }) => {
    const config = DISPUTE_STATUS_CONFIG[item.status];
    const isPlaintiff = item.plaintiff === address;

    return (
      <TouchableOpacity
        activeOpacity={0.8}
        onPress={() => router.push(`/disputes/${item.id}`)}
      >
        <Card style={styles.disputeCard}>
          <View style={styles.disputeHeader}>
            <View style={[styles.statusBadge, { backgroundColor: config.color + '20' }]}>
              <Text style={[styles.statusText, { color: config.color }]}>
                {config.label}
              </Text>
            </View>
            <View style={[styles.roleBadge, { backgroundColor: isPlaintiff ? Colors.primary + '20' : Colors.warning + '20' }]}>
              <Text style={[styles.roleText, { color: isPlaintiff ? Colors.primary : Colors.warning }]}>
                {isPlaintiff ? '我是原告' : '我是被告'}
              </Text>
            </View>
          </View>

          <Text style={[styles.disputeTitle, { color: colors.textPrimary }]} numberOfLines={2}>
            {item.description}
          </Text>

          <View style={styles.partiesRow}>
            <View style={styles.partyItem}>
              <Text style={[styles.partyLabel, { color: colors.textTertiary }]}>
                原告
              </Text>
              <Text style={[styles.partyName, { color: colors.textPrimary }]}>
                {item.plaintiffName}
              </Text>
            </View>
            <View style={styles.vsText}>
              <Text style={[styles.vs, { color: colors.textTertiary }]}>VS</Text>
            </View>
            <View style={styles.partyItem}>
              <Text style={[styles.partyLabel, { color: colors.textTertiary }]}>
                被告
              </Text>
              <Text style={[styles.partyName, { color: colors.textPrimary }]}>
                {item.defendantName}
              </Text>
            </View>
          </View>

          <View style={styles.depositRow}>
            <Text style={[styles.depositLabel, { color: colors.textSecondary }]}>
              押金池
            </Text>
            <Text style={[styles.depositValue, { color: Colors.primary }]}>
              {formatAmount(BigInt(item.plaintiffDeposit) + BigInt(item.defendantDeposit) + '')} COS
            </Text>
          </View>

          {item.status === 'evidence' && (
            <View style={[styles.deadlineRow, { backgroundColor: Colors.warning + '10' }]}>
              <Clock size={14} color={Colors.warning} />
              <Text style={[styles.deadlineText, { color: Colors.warning }]}>
                举证截止: {getTimeLeft(item.evidenceDeadline)}
              </Text>
            </View>
          )}

          <View style={styles.disputeFooter}>
            <Text style={[styles.footerText, { color: colors.textTertiary }]}>
              {item.domain.toUpperCase()} • {item.bizId.slice(0, 12)}...
            </Text>
            <Text style={[styles.footerText, { color: colors.textTertiary }]}>
              {new Date(item.createdAt).toLocaleDateString()}
            </Text>
          </View>
        </Card>
      </TouchableOpacity>
    );
  };

  return (
    <View style={[styles.container, { backgroundColor: colors.background }]}>
      {/* Header */}
      <View style={styles.header}>
        <View>
          <Text style={[styles.title, { color: colors.textPrimary }]}>争议仲裁</Text>
          <Text style={[styles.subtitle, { color: colors.textSecondary }]}>
            公平公正的纠纷解决
          </Text>
        </View>
        <TouchableOpacity
          style={[styles.createButton, { backgroundColor: Colors.primary }]}
          onPress={() => router.push('/disputes/create')}
        >
          <Plus size={20} color="#FFFFFF" />
        </TouchableOpacity>
      </View>

      {/* Info Card */}
      <Card style={[styles.infoCard, { backgroundColor: Colors.info + '10' }]}>
        <Scale size={20} color={Colors.info} />
        <Text style={[styles.infoText, { color: Colors.info }]}>
          平台提供双方押金仲裁机制，由专业仲裁员公正裁决
        </Text>
      </Card>

      {/* Filter Tabs */}
      <View style={styles.filterTabs}>
        {(['all', 'active', 'resolved'] as const).map((f) => (
          <TouchableOpacity
            key={f}
            style={[
              styles.filterTab,
              filter === f && { backgroundColor: Colors.primary + '15' },
            ]}
            onPress={() => setFilter(f)}
          >
            <Text
              style={[
                styles.filterText,
                { color: filter === f ? Colors.primary : colors.textSecondary },
              ]}
            >
              {f === 'all' ? '全部' : f === 'active' ? '进行中' : '已结束'}
            </Text>
          </TouchableOpacity>
        ))}
      </View>

      {/* Disputes List */}
      <FlatList
        data={filteredDisputes}
        renderItem={renderDispute}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.list}
        refreshControl={
          <RefreshControl refreshing={loading} onRefresh={loadData} />
        }
        ListEmptyComponent={
          <View style={styles.empty}>
            <Scale size={48} color={colors.textTertiary} />
            <Text style={[styles.emptyText, { color: colors.textSecondary }]}>
              {loading ? '加载中...' : '暂无争议记录'}
            </Text>
            <Text style={[styles.emptySubtext, { color: colors.textTertiary }]}>
              希望您永远不需要使用此功能
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
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
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
  createButton: {
    width: 40,
    height: 40,
    borderRadius: 20,
    alignItems: 'center',
    justifyContent: 'center',
  },
  infoCard: {
    flexDirection: 'row',
    alignItems: 'center',
    marginHorizontal: 16,
    marginBottom: 12,
    borderWidth: 0,
  },
  infoText: {
    flex: 1,
    fontSize: 13,
    marginLeft: 10,
  },
  filterTabs: {
    flexDirection: 'row',
    paddingHorizontal: 16,
    marginBottom: 8,
    gap: 8,
  },
  filterTab: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 20,
  },
  filterText: {
    fontSize: 14,
    fontWeight: '500',
  },
  list: {
    padding: 16,
    paddingTop: 8,
  },
  disputeCard: {
    marginBottom: 12,
  },
  disputeHeader: {
    flexDirection: 'row',
    gap: 8,
    marginBottom: 12,
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
  roleBadge: {
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
  },
  roleText: {
    fontSize: 12,
    fontWeight: '500',
  },
  disputeTitle: {
    fontSize: 15,
    fontWeight: '500',
    marginBottom: 12,
    lineHeight: 22,
  },
  partiesRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  partyItem: {
    flex: 1,
  },
  partyLabel: {
    fontSize: 12,
    marginBottom: 2,
  },
  partyName: {
    fontSize: 14,
    fontWeight: '500',
  },
  vsText: {
    paddingHorizontal: 12,
  },
  vs: {
    fontSize: 12,
    fontWeight: '600',
  },
  depositRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  depositLabel: {
    fontSize: 13,
  },
  depositValue: {
    fontSize: 14,
    fontWeight: '600',
  },
  deadlineRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    padding: 10,
    borderRadius: 8,
    marginBottom: 12,
  },
  deadlineText: {
    fontSize: 13,
    fontWeight: '500',
  },
  disputeFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: 'rgba(0,0,0,0.05)',
  },
  footerText: {
    fontSize: 12,
  },
  empty: {
    alignItems: 'center',
    paddingTop: 60,
  },
  emptyText: {
    fontSize: 16,
    marginTop: 12,
  },
  emptySubtext: {
    fontSize: 13,
    marginTop: 4,
  },
});
