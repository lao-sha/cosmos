import { useLocalSearchParams, useRouter } from 'expo-router';
import {
  Vote,
  Plus,
  Clock,
  CheckCircle,
  XCircle,
  AlertCircle,
  ThumbsUp,
  ThumbsDown,
  Minus,
} from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
  TextInput,
  Alert,
  Modal,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import { sharemallTxService } from '@/src/services/sharemall-tx';
import type { Shop, Proposal, ProposalStatus } from '@/src/types/sharemall';
import { useWalletStore } from '@/src/stores/wallet';

const STATUS_CONFIG: Record<ProposalStatus, { label: string; color: string; icon: any }> = {
  Created: { label: '待投票', color: '#F57C00', icon: Clock },
  Voting: { label: '投票中', color: '#1976D2', icon: Vote },
  Passed: { label: '已通过', color: '#388E3C', icon: CheckCircle },
  Failed: { label: '未通过', color: '#D32F2F', icon: XCircle },
  Queued: { label: '待执行', color: '#7B1FA2', icon: Clock },
  Executed: { label: '已执行', color: '#388E3C', icon: CheckCircle },
  Cancelled: { label: '已取消', color: '#757575', icon: XCircle },
  Expired: { label: '已过期', color: '#9E9E9E', icon: AlertCircle },
};

export default function GovernanceScreen() {
  const { shopId: shopIdParam } = useLocalSearchParams<{ shopId: string }>();
  const router = useRouter();
  const { currentAccount } = useWalletStore();

  const shopId = parseInt(shopIdParam || '0', 10);
  const [shop, setShop] = useState<Shop | null>(null);
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [tokenBalance, setTokenBalance] = useState('0');
  const [refreshing, setRefreshing] = useState(false);

  // 创建提案
  const [createModalVisible, setCreateModalVisible] = useState(false);
  const [titleCid, setTitleCid] = useState('');
  const [descriptionCid, setDescriptionCid] = useState('');

  // 投票
  const [voteModalVisible, setVoteModalVisible] = useState(false);
  const [selectedProposal, setSelectedProposal] = useState<Proposal | null>(null);
  const [voteAmount, setVoteAmount] = useState('');
  const [voteSupport, setVoteSupport] = useState<boolean | null>(null);

  const [loading, setLoading] = useState(false);

  const loadData = async () => {
    if (!shopId) return;
    try {
      const shopData = await sharemallService.getShop(shopId);
      setShop(shopData);

      // TODO: 从链上加载提案列表
      setProposals([]);

      if (currentAccount) {
        const balance = await sharemallService.getTokenBalance(shopId, currentAccount.address);
        setTokenBalance(balance);
      }
    } catch (error) {
      console.error('Failed to load governance data:', error);
    }
  };

  useEffect(() => {
    loadData();
  }, [shopId, currentAccount]);

  const onRefresh = async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
  };

  const formatBalance = (balance: string) => {
    const num = BigInt(balance);
    return (Number(num) / 1e10).toFixed(2);
  };

  const handleCreateProposal = async () => {
    if (!titleCid.trim() || !descriptionCid.trim()) {
      Alert.alert('提示', '请填写完整信息');
      return;
    }

    setLoading(true);
    try {
      await sharemallTxService.createProposal(shopId, titleCid.trim(), descriptionCid.trim());
      Alert.alert('成功', '提案已创建');
      setCreateModalVisible(false);
      setTitleCid('');
      setDescriptionCid('');
      loadData();
    } catch (error: any) {
      Alert.alert('错误', error.message || '创建失败');
    } finally {
      setLoading(false);
    }
  };

  const openVoteModal = (proposal: Proposal) => {
    setSelectedProposal(proposal);
    setVoteAmount('');
    setVoteSupport(null);
    setVoteModalVisible(true);
  };

  const handleVote = async () => {
    if (!selectedProposal || voteSupport === null || !voteAmount) {
      Alert.alert('提示', '请选择投票立场并输入投票数量');
      return;
    }

    setLoading(true);
    try {
      const amount = (parseFloat(voteAmount) * 1e10).toString();
      await sharemallTxService.vote(selectedProposal.id, voteSupport, amount);
      Alert.alert('成功', '投票成功');
      setVoteModalVisible(false);
      loadData();
    } catch (error: any) {
      Alert.alert('错误', error.message || '投票失败');
    } finally {
      setLoading(false);
    }
  };

  const renderProposalItem = ({ item }: { item: Proposal }) => {
    const statusConfig = STATUS_CONFIG[item.status];
    const StatusIcon = statusConfig.icon;

    const totalVotes = BigInt(item.forVotes) + BigInt(item.againstVotes) + BigInt(item.abstainVotes);
    const forPercent = totalVotes > 0 ? (Number(BigInt(item.forVotes) * BigInt(100) / totalVotes)) : 0;
    const againstPercent = totalVotes > 0 ? (Number(BigInt(item.againstVotes) * BigInt(100) / totalVotes)) : 0;

    return (
      <View style={styles.proposalCard}>
        <View style={styles.proposalHeader}>
          <Text style={styles.proposalId}>提案 #{item.id}</Text>
          <View style={[styles.statusBadge, { backgroundColor: statusConfig.color + '20' }]}>
            <StatusIcon size={12} color={statusConfig.color} />
            <Text style={[styles.statusText, { color: statusConfig.color }]}>
              {statusConfig.label}
            </Text>
          </View>
        </View>

        <Text style={styles.proposalTitle}>标题 CID: {item.titleCid}</Text>

        {/* 投票进度 */}
        <View style={styles.voteProgress}>
          <View style={styles.voteBar}>
            <View style={[styles.forBar, { width: `${forPercent}%` }]} />
            <View style={[styles.againstBar, { width: `${againstPercent}%` }]} />
          </View>
          <View style={styles.voteStats}>
            <View style={styles.voteStat}>
              <ThumbsUp size={14} color="#388E3C" />
              <Text style={styles.voteStatText}>{formatBalance(item.forVotes)}</Text>
            </View>
            <View style={styles.voteStat}>
              <ThumbsDown size={14} color="#D32F2F" />
              <Text style={styles.voteStatText}>{formatBalance(item.againstVotes)}</Text>
            </View>
            <View style={styles.voteStat}>
              <Minus size={14} color="#757575" />
              <Text style={styles.voteStatText}>{formatBalance(item.abstainVotes)}</Text>
            </View>
          </View>
        </View>

        {(item.status === 'Created' || item.status === 'Voting') && (
          <TouchableOpacity
            style={styles.voteBtn}
            onPress={() => openVoteModal(item)}
          >
            <Text style={styles.voteBtnText}>参与投票</Text>
          </TouchableOpacity>
        )}
      </View>
    );
  };

  return (
    <View style={styles.container}>
      {/* 头部信息 */}
      <View style={styles.header}>
        <View style={styles.shopInfo}>
          <Text style={styles.shopName}>{shop?.name || '店铺'} 治理</Text>
          <Text style={styles.tokenBalance}>
            我的投票权: {formatBalance(tokenBalance)} 代币
          </Text>
        </View>
        <TouchableOpacity
          style={styles.createBtn}
          onPress={() => setCreateModalVisible(true)}
        >
          <Plus size={20} color="#fff" />
          <Text style={styles.createBtnText}>创建提案</Text>
        </TouchableOpacity>
      </View>

      {/* 提案列表 */}
      <FlatList
        data={proposals}
        renderItem={renderProposalItem}
        keyExtractor={(item) => item.id.toString()}
        contentContainerStyle={styles.listContent}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
        ListEmptyComponent={
          <View style={styles.emptyContainer}>
            <Vote size={48} color="#ccc" />
            <Text style={styles.emptyText}>暂无提案</Text>
            <Text style={styles.emptyHint}>持有店铺代币可创建和参与提案投票</Text>
          </View>
        }
      />

      {/* 创建提案弹窗 */}
      <Modal visible={createModalVisible} transparent animationType="fade">
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <Text style={styles.modalTitle}>创建提案</Text>

            <Text style={styles.modalLabel}>提案标题 (IPFS CID)</Text>
            <TextInput
              style={styles.modalInput}
              placeholder="输入标题的 IPFS CID"
              value={titleCid}
              onChangeText={setTitleCid}
            />

            <Text style={styles.modalLabel}>提案描述 (IPFS CID)</Text>
            <TextInput
              style={[styles.modalInput, { minHeight: 80 }]}
              placeholder="输入描述的 IPFS CID"
              value={descriptionCid}
              onChangeText={setDescriptionCid}
              multiline
            />

            <View style={styles.modalActions}>
              <TouchableOpacity
                style={styles.modalCancelBtn}
                onPress={() => setCreateModalVisible(false)}
              >
                <Text style={styles.modalCancelText}>取消</Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={[styles.modalConfirmBtn, loading && styles.modalBtnDisabled]}
                onPress={handleCreateProposal}
                disabled={loading}
              >
                <Text style={styles.modalConfirmText}>
                  {loading ? '创建中...' : '创建提案'}
                </Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>
      </Modal>

      {/* 投票弹窗 */}
      <Modal visible={voteModalVisible} transparent animationType="fade">
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <Text style={styles.modalTitle}>参与投票</Text>
            <Text style={styles.modalSubtitle}>提案 #{selectedProposal?.id}</Text>

            <Text style={styles.modalLabel}>选择立场</Text>
            <View style={styles.voteOptions}>
              <TouchableOpacity
                style={[
                  styles.voteOption,
                  voteSupport === true && styles.voteOptionFor,
                ]}
                onPress={() => setVoteSupport(true)}
              >
                <ThumbsUp size={20} color={voteSupport === true ? '#fff' : '#388E3C'} />
                <Text
                  style={[
                    styles.voteOptionText,
                    voteSupport === true && styles.voteOptionTextActive,
                  ]}
                >
                  赞成
                </Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={[
                  styles.voteOption,
                  voteSupport === false && styles.voteOptionAgainst,
                ]}
                onPress={() => setVoteSupport(false)}
              >
                <ThumbsDown size={20} color={voteSupport === false ? '#fff' : '#D32F2F'} />
                <Text
                  style={[
                    styles.voteOptionText,
                    voteSupport === false && styles.voteOptionTextActive,
                  ]}
                >
                  反对
                </Text>
              </TouchableOpacity>
            </View>

            <Text style={styles.modalLabel}>投票数量</Text>
            <TextInput
              style={styles.modalInput}
              placeholder="输入投票代币数量"
              keyboardType="decimal-pad"
              value={voteAmount}
              onChangeText={setVoteAmount}
            />
            <Text style={styles.modalHint}>
              可用: {formatBalance(tokenBalance)} 代币
            </Text>

            <View style={styles.modalActions}>
              <TouchableOpacity
                style={styles.modalCancelBtn}
                onPress={() => setVoteModalVisible(false)}
              >
                <Text style={styles.modalCancelText}>取消</Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={[styles.modalConfirmBtn, loading && styles.modalBtnDisabled]}
                onPress={handleVote}
                disabled={loading}
              >
                <Text style={styles.modalConfirmText}>
                  {loading ? '投票中...' : '确认投票'}
                </Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>
      </Modal>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    backgroundColor: '#fff',
    padding: 16,
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  shopInfo: {},
  shopName: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  tokenBalance: {
    fontSize: 13,
    color: '#666',
    marginTop: 4,
  },
  createBtn: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#1976D2',
    paddingHorizontal: 16,
    paddingVertical: 10,
    borderRadius: 20,
  },
  createBtnText: {
    color: '#fff',
    fontSize: 14,
    marginLeft: 6,
  },
  listContent: {
    padding: 12,
  },
  proposalCard: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  proposalHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  proposalId: {
    fontSize: 14,
    fontWeight: '500',
    color: '#333',
  },
  statusBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
  },
  statusText: {
    fontSize: 12,
    marginLeft: 4,
  },
  proposalTitle: {
    fontSize: 13,
    color: '#666',
    marginBottom: 12,
  },
  voteProgress: {
    marginBottom: 12,
  },
  voteBar: {
    height: 8,
    backgroundColor: '#f0f0f0',
    borderRadius: 4,
    flexDirection: 'row',
    overflow: 'hidden',
  },
  forBar: {
    backgroundColor: '#388E3C',
    height: '100%',
  },
  againstBar: {
    backgroundColor: '#D32F2F',
    height: '100%',
  },
  voteStats: {
    flexDirection: 'row',
    marginTop: 8,
  },
  voteStat: {
    flexDirection: 'row',
    alignItems: 'center',
    marginRight: 16,
  },
  voteStatText: {
    fontSize: 12,
    color: '#666',
    marginLeft: 4,
  },
  voteBtn: {
    backgroundColor: '#1976D2',
    borderRadius: 8,
    paddingVertical: 10,
    alignItems: 'center',
  },
  voteBtnText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
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
    textAlign: 'center',
  },
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.5)',
    justifyContent: 'center',
    alignItems: 'center',
  },
  modalContent: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 20,
    width: '85%',
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    textAlign: 'center',
  },
  modalSubtitle: {
    fontSize: 13,
    color: '#666',
    textAlign: 'center',
    marginTop: 4,
    marginBottom: 20,
  },
  modalLabel: {
    fontSize: 14,
    color: '#666',
    marginBottom: 8,
  },
  modalInput: {
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 12,
    fontSize: 14,
    marginBottom: 16,
  },
  modalHint: {
    fontSize: 12,
    color: '#999',
    marginTop: -12,
    marginBottom: 16,
  },
  voteOptions: {
    flexDirection: 'row',
    marginBottom: 16,
  },
  voteOption: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 12,
    borderRadius: 8,
    backgroundColor: '#f5f5f5',
    marginRight: 8,
  },
  voteOptionFor: {
    backgroundColor: '#388E3C',
  },
  voteOptionAgainst: {
    backgroundColor: '#D32F2F',
    marginRight: 0,
  },
  voteOptionText: {
    fontSize: 14,
    color: '#666',
    marginLeft: 8,
  },
  voteOptionTextActive: {
    color: '#fff',
  },
  modalActions: {
    flexDirection: 'row',
    marginTop: 8,
  },
  modalCancelBtn: {
    flex: 1,
    paddingVertical: 12,
    alignItems: 'center',
    marginRight: 8,
    borderRadius: 8,
    backgroundColor: '#f5f5f5',
  },
  modalCancelText: {
    color: '#666',
    fontSize: 14,
  },
  modalConfirmBtn: {
    flex: 1,
    paddingVertical: 12,
    alignItems: 'center',
    marginLeft: 8,
    borderRadius: 8,
    backgroundColor: '#1976D2',
  },
  modalBtnDisabled: {
    opacity: 0.6,
  },
  modalConfirmText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
});
