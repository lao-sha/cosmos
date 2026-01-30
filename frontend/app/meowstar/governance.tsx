import React, { useState } from 'react';
import { View, Text, ScrollView, TouchableOpacity, StyleSheet, Modal, TextInput, Alert } from 'react-native';
import { Vote, Clock, CheckCircle, XCircle, AlertCircle, Plus } from 'lucide-react-native';

interface Proposal {
  id: number;
  title: string;
  type: 'general' | 'parameter' | 'treasury' | 'emergency';
  proposer: string;
  status: 'active' | 'passed' | 'rejected' | 'executed';
  yesVotes: number;
  noVotes: number;
  totalVotes: number;
  quorum: number;
  endsIn: string;
  description: string;
}

const STATUS_CONFIG = {
  active: { color: '#4ECDC4', label: '投票中', icon: Clock },
  passed: { color: '#58D68D', label: '已通过', icon: CheckCircle },
  rejected: { color: '#FF6B6B', label: '已拒绝', icon: XCircle },
  executed: { color: '#BB8FCE', label: '已执行', icon: CheckCircle },
};

const TYPE_LABELS = {
  general: '普通提案',
  parameter: '参数修改',
  treasury: '国库支出',
  emergency: '紧急提案',
};

export default function GovernanceScreen() {
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [proposalTitle, setProposalTitle] = useState('');
  const [proposalDesc, setProposalDesc] = useState('');

  const userVotePower = 3750;
  const totalVotePower = 125000;

  const proposals: Proposal[] = [
    {
      id: 1,
      title: '增加战斗奖励池',
      type: 'treasury',
      proposer: '5Grw...aEF5',
      status: 'active',
      yesVotes: 45000,
      noVotes: 12000,
      totalVotes: 57000,
      quorum: 12500,
      endsIn: '2天 5小时',
      description: '提议从国库拨款 50,000 COS 用于增加战斗奖励池...',
    },
    {
      id: 2,
      title: '降低交易手续费至 2%',
      type: 'parameter',
      proposer: '5FHn...eVkZ',
      status: 'active',
      yesVotes: 28000,
      noVotes: 35000,
      totalVotes: 63000,
      quorum: 12500,
      endsIn: '5天 12小时',
      description: '建议将市场交易手续费从 2.5% 降低至 2%...',
    },
    {
      id: 3,
      title: '新增神话级宠物',
      type: 'general',
      proposer: '5DAn...kLmP',
      status: 'passed',
      yesVotes: 78000,
      noVotes: 15000,
      totalVotes: 93000,
      quorum: 12500,
      endsIn: '已结束',
      description: '提议新增 3 种神话级宠物...',
    },
  ];

  const handleVote = (proposalId: number, approve: boolean) => {
    Alert.alert(
      '确认投票',
      `确定要投 ${approve ? '赞成' : '反对'} 票吗？\n您的投票权重: ${userVotePower}`,
      [
        { text: '取消', style: 'cancel' },
        { 
          text: '确认', 
          onPress: () => Alert.alert('成功', '投票已提交') 
        },
      ]
    );
  };

  const handleCreateProposal = () => {
    if (!proposalTitle.trim()) {
      Alert.alert('提示', '请输入提案标题');
      return;
    }
    
    setShowCreateModal(false);
    setProposalTitle('');
    setProposalDesc('');
    Alert.alert('成功', '提案已创建，需支付 100 COS 押金');
  };

  return (
    <View style={styles.container}>
      {/* 投票权重 */}
      <View style={styles.powerCard}>
        <Vote size={24} color="#BB8FCE" />
        <View style={styles.powerInfo}>
          <Text style={styles.powerLabel}>我的投票权重</Text>
          <Text style={styles.powerValue}>{userVotePower.toLocaleString()}</Text>
        </View>
        <View style={styles.powerPercent}>
          <Text style={styles.percentText}>
            {((userVotePower / totalVotePower) * 100).toFixed(2)}%
          </Text>
          <Text style={styles.percentLabel}>占比</Text>
        </View>
      </View>

      {/* 提案列表 */}
      <ScrollView style={styles.proposalList}>
        {proposals.map((proposal) => {
          const StatusIcon = STATUS_CONFIG[proposal.status].icon;
          const yesPercent = (proposal.yesVotes / proposal.totalVotes) * 100;
          
          return (
            <View key={proposal.id} style={styles.proposalCard}>
              <View style={styles.proposalHeader}>
                <View style={[
                  styles.typeBadge,
                  { backgroundColor: proposal.type === 'treasury' ? '#F7DC6F20' : '#4ECDC420' }
                ]}>
                  <Text style={[
                    styles.typeText,
                    { color: proposal.type === 'treasury' ? '#F7DC6F' : '#4ECDC4' }
                  ]}>
                    {TYPE_LABELS[proposal.type]}
                  </Text>
                </View>
                <View style={[
                  styles.statusBadge,
                  { backgroundColor: STATUS_CONFIG[proposal.status].color + '20' }
                ]}>
                  <StatusIcon size={12} color={STATUS_CONFIG[proposal.status].color} />
                  <Text style={[
                    styles.statusText,
                    { color: STATUS_CONFIG[proposal.status].color }
                  ]}>
                    {STATUS_CONFIG[proposal.status].label}
                  </Text>
                </View>
              </View>

              <Text style={styles.proposalTitle}>{proposal.title}</Text>
              <Text style={styles.proposalDesc} numberOfLines={2}>
                {proposal.description}
              </Text>

              <View style={styles.voteProgress}>
                <View style={styles.progressBar}>
                  <View style={[styles.progressYes, { width: `${yesPercent}%` }]} />
                </View>
                <View style={styles.voteStats}>
                  <Text style={styles.voteYes}>
                    赞成 {(proposal.yesVotes / 1000).toFixed(1)}K ({yesPercent.toFixed(1)}%)
                  </Text>
                  <Text style={styles.voteNo}>
                    反对 {(proposal.noVotes / 1000).toFixed(1)}K ({(100 - yesPercent).toFixed(1)}%)
                  </Text>
                </View>
              </View>

              <View style={styles.proposalMeta}>
                <Text style={styles.metaText}>提案者: {proposal.proposer}</Text>
                <Text style={styles.metaText}>
                  {proposal.status === 'active' ? `剩余: ${proposal.endsIn}` : proposal.endsIn}
                </Text>
              </View>

              {proposal.status === 'active' && (
                <View style={styles.voteActions}>
                  <TouchableOpacity
                    style={styles.voteYesButton}
                    onPress={() => handleVote(proposal.id, true)}
                  >
                    <CheckCircle size={16} color="#fff" />
                    <Text style={styles.voteButtonText}>赞成</Text>
                  </TouchableOpacity>
                  <TouchableOpacity
                    style={styles.voteNoButton}
                    onPress={() => handleVote(proposal.id, false)}
                  >
                    <XCircle size={16} color="#fff" />
                    <Text style={styles.voteButtonText}>反对</Text>
                  </TouchableOpacity>
                </View>
              )}
            </View>
          );
        })}
      </ScrollView>

      {/* 创建提案按钮 */}
      <TouchableOpacity
        style={styles.fab}
        onPress={() => setShowCreateModal(true)}
      >
        <Plus size={28} color="#fff" />
      </TouchableOpacity>

      {/* 创建提案模态框 */}
      <Modal
        visible={showCreateModal}
        animationType="slide"
        transparent={true}
        onRequestClose={() => setShowCreateModal(false)}
      >
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <Text style={styles.modalTitle}>创建提案</Text>
            <Text style={styles.modalNote}>
              创建提案需要至少 {(10000).toLocaleString()} 投票权重，并支付 100 COS 押金
            </Text>

            <TextInput
              style={styles.input}
              placeholder="提案标题"
              placeholderTextColor="#666"
              value={proposalTitle}
              onChangeText={setProposalTitle}
            />

            <TextInput
              style={[styles.input, styles.textArea]}
              placeholder="提案描述..."
              placeholderTextColor="#666"
              multiline
              numberOfLines={4}
              value={proposalDesc}
              onChangeText={setProposalDesc}
            />

            <View style={styles.modalButtons}>
              <TouchableOpacity
                style={styles.cancelButton}
                onPress={() => setShowCreateModal(false)}
              >
                <Text style={styles.cancelButtonText}>取消</Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={styles.confirmButton}
                onPress={handleCreateProposal}
              >
                <Text style={styles.confirmButtonText}>创建</Text>
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
    backgroundColor: '#0f0f1a',
  },
  powerCard: {
    backgroundColor: '#1a1a2e',
    margin: 16,
    borderRadius: 16,
    padding: 20,
    flexDirection: 'row',
    alignItems: 'center',
  },
  powerInfo: {
    flex: 1,
    marginLeft: 16,
  },
  powerLabel: {
    fontSize: 12,
    color: '#888',
  },
  powerValue: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#fff',
  },
  powerPercent: {
    alignItems: 'flex-end',
  },
  percentText: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#BB8FCE',
  },
  percentLabel: {
    fontSize: 12,
    color: '#888',
  },
  proposalList: {
    flex: 1,
    padding: 16,
    paddingTop: 0,
  },
  proposalCard: {
    backgroundColor: '#1a1a2e',
    borderRadius: 16,
    padding: 16,
    marginBottom: 12,
  },
  proposalHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  typeBadge: {
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
  },
  typeText: {
    fontSize: 12,
    fontWeight: '600',
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
    fontSize: 18,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 8,
  },
  proposalDesc: {
    fontSize: 14,
    color: '#888',
    marginBottom: 16,
  },
  voteProgress: {
    marginBottom: 12,
  },
  progressBar: {
    height: 8,
    backgroundColor: '#FF6B6B40',
    borderRadius: 4,
    overflow: 'hidden',
  },
  progressYes: {
    height: '100%',
    backgroundColor: '#4ECDC4',
    borderRadius: 4,
  },
  voteStats: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginTop: 8,
  },
  voteYes: {
    fontSize: 12,
    color: '#4ECDC4',
  },
  voteNo: {
    fontSize: 12,
    color: '#FF6B6B',
  },
  proposalMeta: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  metaText: {
    fontSize: 12,
    color: '#666',
  },
  voteActions: {
    flexDirection: 'row',
  },
  voteYesButton: {
    flex: 1,
    flexDirection: 'row',
    backgroundColor: '#4ECDC4',
    borderRadius: 8,
    padding: 12,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 8,
  },
  voteNoButton: {
    flex: 1,
    flexDirection: 'row',
    backgroundColor: '#FF6B6B',
    borderRadius: 8,
    padding: 12,
    alignItems: 'center',
    justifyContent: 'center',
  },
  voteButtonText: {
    color: '#fff',
    fontWeight: 'bold',
    marginLeft: 8,
  },
  fab: {
    position: 'absolute',
    right: 20,
    bottom: 20,
    width: 60,
    height: 60,
    borderRadius: 30,
    backgroundColor: '#BB8FCE',
    justifyContent: 'center',
    alignItems: 'center',
    elevation: 4,
  },
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.7)',
    justifyContent: 'center',
    alignItems: 'center',
  },
  modalContent: {
    backgroundColor: '#1a1a2e',
    borderRadius: 20,
    padding: 24,
    width: '90%',
    maxWidth: 400,
  },
  modalTitle: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 8,
  },
  modalNote: {
    fontSize: 12,
    color: '#888',
    marginBottom: 20,
  },
  input: {
    backgroundColor: '#252540',
    borderRadius: 12,
    padding: 16,
    fontSize: 16,
    color: '#fff',
    marginBottom: 12,
  },
  textArea: {
    height: 100,
    textAlignVertical: 'top',
  },
  modalButtons: {
    flexDirection: 'row',
    marginTop: 8,
  },
  cancelButton: {
    flex: 1,
    padding: 14,
    borderRadius: 12,
    backgroundColor: '#333',
    marginRight: 8,
    alignItems: 'center',
  },
  cancelButtonText: {
    fontSize: 16,
    color: '#888',
  },
  confirmButton: {
    flex: 1,
    padding: 14,
    borderRadius: 12,
    backgroundColor: '#BB8FCE',
    marginLeft: 8,
    alignItems: 'center',
  },
  confirmButtonText: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#fff',
  },
});
