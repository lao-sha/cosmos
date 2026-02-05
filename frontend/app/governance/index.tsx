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
import { Vote, Plus, Clock, CheckCircle, XCircle } from 'lucide-react-native';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import { getProposals, type Proposal, type ProposalStatus } from '@/services/governance';
import { Card, Button } from '@/components/ui';
import { Colors } from '@/constants/colors';

const STATUS_CONFIG: Record<ProposalStatus, { label: string; color: string; icon: any }> = {
  proposing: { label: '提议中', color: Colors.governance.proposing, icon: Clock },
  voting: { label: '投票中', color: Colors.governance.voting, icon: Vote },
  passed: { label: '已通过', color: Colors.governance.passed, icon: CheckCircle },
  rejected: { label: '已否决', color: Colors.governance.rejected, icon: XCircle },
  executed: { label: '已执行', color: Colors.governance.executed, icon: CheckCircle },
  cancelled: { label: '已取消', color: '#9CA3AF', icon: XCircle },
};

export default function GovernanceScreen() {
  const colors = useColors();
  const router = useRouter();
  const { address } = useWalletStore();

  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<'all' | 'active' | 'ended'>('all');

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setLoading(true);
    try {
      const data = await getProposals();
      setProposals(data);
    } catch (error) {
      console.error('Failed to load proposals:', error);
    } finally {
      setLoading(false);
    }
  };

  const filteredProposals = proposals.filter((p) => {
    if (filter === 'active') return p.status === 'voting' || p.status === 'proposing';
    if (filter === 'ended') return p.status === 'passed' || p.status === 'rejected' || p.status === 'executed';
    return true;
  });

  const formatAmount = (value: string): string => {
    const num = BigInt(value || '0') / BigInt(1e12);
    return num.toLocaleString();
  };

  const getTimeLeft = (endTime: number): string => {
    const diff = endTime - Date.now();
    if (diff <= 0) return '已结束';
    const days = Math.floor(diff / (24 * 3600000));
    const hours = Math.floor((diff % (24 * 3600000)) / 3600000);
    if (days > 0) return `${days}天${hours}小时`;
    return `${hours}小时`;
  };

  const renderProposal = ({ item }: { item: Proposal }) => {
    const config = STATUS_CONFIG[item.status];
    const StatusIcon = config.icon;
    const totalVotes = BigInt(item.votesYes) + BigInt(item.votesNo) + BigInt(item.votesAbstain);
    const yesPercent = totalVotes > 0 ? Number((BigInt(item.votesYes) * BigInt(100)) / totalVotes) : 0;
    const noPercent = totalVotes > 0 ? Number((BigInt(item.votesNo) * BigInt(100)) / totalVotes) : 0;

    return (
      <TouchableOpacity
        activeOpacity={0.8}
        onPress={() => router.push(`/governance/${item.id}`)}
      >
        <Card style={styles.proposalCard}>
          <View style={styles.proposalHeader}>
            <View style={[styles.statusBadge, { backgroundColor: config.color + '20' }]}>
              <StatusIcon size={14} color={config.color} />
              <Text style={[styles.statusText, { color: config.color }]}>
                {config.label}
              </Text>
            </View>
            {item.status === 'voting' && (
              <Text style={[styles.timeLeft, { color: colors.textTertiary }]}>
                剩余 {getTimeLeft(item.endTime)}
              </Text>
            )}
          </View>

          <Text style={[styles.proposalTitle, { color: colors.textPrimary }]} numberOfLines={2}>
            {item.title}
          </Text>

          <Text style={[styles.proposalDesc, { color: colors.textSecondary }]} numberOfLines={2}>
            {item.description}
          </Text>

          <View style={styles.voteSection}>
            <View style={styles.voteBar}>
              <View style={[styles.voteYes, { width: `${yesPercent}%` }]} />
              <View style={[styles.voteNo, { width: `${noPercent}%` }]} />
            </View>
            <View style={styles.voteStats}>
              <Text style={[styles.voteText, { color: Colors.governance.voteYes }]}>
                赞成 {yesPercent.toFixed(0)}%
              </Text>
              <Text style={[styles.voteText, { color: Colors.governance.voteNo }]}>
                反对 {noPercent.toFixed(0)}%
              </Text>
            </View>
          </View>

          <View style={styles.proposalFooter}>
            <Text style={[styles.footerText, { color: colors.textTertiary }]}>
              {item.entityName}
            </Text>
            <Text style={[styles.footerText, { color: colors.textTertiary }]}>
              {formatAmount(item.totalVotes)} 票
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
        <Text style={[styles.title, { color: colors.textPrimary }]}>治理投票</Text>
        <TouchableOpacity
          style={[styles.createButton, { backgroundColor: Colors.primary }]}
          onPress={() => router.push('/governance/create')}
        >
          <Plus size={20} color="#FFFFFF" />
        </TouchableOpacity>
      </View>

      {/* Filter Tabs */}
      <View style={styles.filterTabs}>
        {(['all', 'active', 'ended'] as const).map((f) => (
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

      {/* Proposals List */}
      <FlatList
        data={filteredProposals}
        renderItem={renderProposal}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.list}
        refreshControl={
          <RefreshControl refreshing={loading} onRefresh={loadData} />
        }
        ListEmptyComponent={
          <View style={styles.empty}>
            <Vote size={48} color={colors.textTertiary} />
            <Text style={[styles.emptyText, { color: colors.textSecondary }]}>
              {loading ? '加载中...' : '暂无提案'}
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
  createButton: {
    width: 40,
    height: 40,
    borderRadius: 20,
    alignItems: 'center',
    justifyContent: 'center',
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
  proposalCard: {
    marginBottom: 12,
  },
  proposalHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  statusBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
    gap: 4,
  },
  statusText: {
    fontSize: 12,
    fontWeight: '500',
  },
  timeLeft: {
    fontSize: 12,
  },
  proposalTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 8,
    lineHeight: 22,
  },
  proposalDesc: {
    fontSize: 14,
    lineHeight: 20,
    marginBottom: 12,
  },
  voteSection: {
    marginBottom: 12,
  },
  voteBar: {
    height: 8,
    backgroundColor: '#E5E7EB',
    borderRadius: 4,
    flexDirection: 'row',
    overflow: 'hidden',
    marginBottom: 8,
  },
  voteYes: {
    height: '100%',
    backgroundColor: Colors.governance.voteYes,
  },
  voteNo: {
    height: '100%',
    backgroundColor: Colors.governance.voteNo,
  },
  voteStats: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  voteText: {
    fontSize: 13,
    fontWeight: '500',
  },
  proposalFooter: {
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
});
