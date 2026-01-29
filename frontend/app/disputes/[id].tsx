import { TransactionModal } from '@/src/components/TransactionModal';
import { UserAvatar } from '@/src/components/UserAvatar';
import { useTransaction } from '@/src/hooks/useTransaction';
import { useAuthStore } from '@/src/stores/auth';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useState } from 'react';
import {
    Pressable,
    ScrollView,
    StyleSheet,
    Text,
    TextInput,
    View
} from 'react-native';

const MOCK_DISPUTE = {
  id: '1',
  domain: 'otc',
  objectId: '12345',
  complaintType: 'payment_issue',
  status: 'submitted',
  complainant: {
    address: '5FHn...xPqA',
    name: '我',
  },
  respondent: {
    address: '5Grw...utQY',
    name: '张三',
  },
  amount: '100 USDT',
  detailsCid: 'QmXyz...',
  createdAt: '2025-01-27 14:30',
  updatedAt: '2025-01-27 14:30',
  deposit: '10 STAR',
  responseDeadline: '2025-01-30 14:30',
  evidences: [
    { id: 1, uploader: 'complainant', cid: 'QmAbc...', uploadedAt: '2025-01-27 14:30' },
  ],
};

const STATUS_FLOW = [
  { key: 'submitted', label: '已提交' },
  { key: 'responded', label: '已回应' },
  { key: 'mediating', label: '调解中' },
  { key: 'arbitrating', label: '仲裁中' },
  { key: 'resolved', label: '已结束' },
];

export default function DisputeDetailScreen() {
  const router = useRouter();
  const { id } = useLocalSearchParams<{ id: string }>();
  const { isLoggedIn, address } = useAuthStore();
  const { status, isLoading, reset } = useTransaction();

  const [modalVisible, setModalVisible] = useState(false);
  const [modalAction, setModalAction] = useState<'respond' | 'evidence' | 'escalate' | 'withdraw'>('respond');
  const [evidenceInput, setEvidenceInput] = useState('');
  const [responseInput, setResponseInput] = useState('');

  const dispute = MOCK_DISPUTE;
  const isComplainant = dispute.complainant.name === '我';
  const isRespondent = dispute.respondent.name === '我';
  const canRespond = isRespondent && dispute.status === 'submitted';
  const canAddEvidence = ['submitted', 'responded', 'mediating'].includes(dispute.status);
  const canEscalate = dispute.status === 'responded' || dispute.status === 'mediating';
  const canWithdraw = isComplainant && dispute.status === 'submitted';

  const getCurrentStep = () => {
    const idx = STATUS_FLOW.findIndex((s) => s.key === dispute.status);
    return idx >= 0 ? idx : 0;
  };

  const handleAction = (action: typeof modalAction) => {
    setModalAction(action);
    setModalVisible(true);
    reset();
  };

  const handleConfirm = async () => {
    switch (modalAction) {
      case 'respond':
        // TODO: 调用链上 respond_to_dispute
        break;
      case 'evidence':
        // TODO: 上传证据到 IPFS 并调用链上添加证据
        break;
      case 'escalate':
        // TODO: 调用链上 escalate_complaint
        break;
      case 'withdraw':
        // TODO: 调用链上撤销投诉
        break;
    }
  };

  const getModalTitle = () => {
    switch (modalAction) {
      case 'respond':
        return '回应投诉';
      case 'evidence':
        return '提交证据';
      case 'escalate':
        return '升级仲裁';
      case 'withdraw':
        return '撤销投诉';
    }
  };

  const getModalDescription = () => {
    switch (modalAction) {
      case 'respond':
        return '提交回应后将锁定押金，请确保提供充分的反驳证据。';
      case 'evidence':
        return '上传相关证据文件，支持图片、文档等。';
      case 'escalate':
        return '升级至仲裁委员会处理，将由专业仲裁员进行裁决。';
      case 'withdraw':
        return '确定撤销投诉吗？押金将按比例退还。';
    }
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>申诉详情</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.statusCard}>
          <Text style={styles.statusTitle}>处理进度</Text>
          <View style={styles.statusFlow}>
            {STATUS_FLOW.map((step, index) => {
              const currentStep = getCurrentStep();
              const isActive = index <= currentStep;
              const isCurrent = index === currentStep;

              return (
                <View key={step.key} style={styles.stepContainer}>
                  <View
                    style={[
                      styles.stepDot,
                      isActive && styles.stepDotActive,
                      isCurrent && styles.stepDotCurrent,
                    ]}
                  >
                    {isActive && <Text style={styles.stepCheck}>✓</Text>}
                  </View>
                  <Text
                    style={[
                      styles.stepLabel,
                      isActive && styles.stepLabelActive,
                    ]}
                  >
                    {step.label}
                  </Text>
                  {index < STATUS_FLOW.length - 1 && (
                    <View
                      style={[
                        styles.stepLine,
                        isActive && styles.stepLineActive,
                      ]}
                    />
                  )}
                </View>
              );
            })}
          </View>
        </View>

        <View style={styles.partiesCard}>
          <Text style={styles.cardTitle}>双方信息</Text>
          <View style={styles.partyRow}>
            <View style={styles.party}>
              <UserAvatar name={dispute.complainant.name} size="medium" />
              <Text style={styles.partyName}>{dispute.complainant.name}</Text>
              <Text style={styles.partyRole}>投诉方</Text>
              <Text style={styles.partyAddress}>{dispute.complainant.address}</Text>
            </View>
            <Text style={styles.vsText}>VS</Text>
            <View style={styles.party}>
              <UserAvatar name={dispute.respondent.name} size="medium" />
              <Text style={styles.partyName}>{dispute.respondent.name}</Text>
              <Text style={styles.partyRole}>被投诉方</Text>
              <Text style={styles.partyAddress}>{dispute.respondent.address}</Text>
            </View>
          </View>
        </View>

        <View style={styles.detailCard}>
          <Text style={styles.cardTitle}>投诉信息</Text>
          <View style={styles.detailRow}>
            <Text style={styles.detailLabel}>投诉类型</Text>
            <Text style={styles.detailValue}>付款问题</Text>
          </View>
          <View style={styles.detailRow}>
            <Text style={styles.detailLabel}>涉及金额</Text>
            <Text style={styles.detailValue}>{dispute.amount}</Text>
          </View>
          <View style={styles.detailRow}>
            <Text style={styles.detailLabel}>关联订单</Text>
            <Text style={styles.detailValue}>OTC #{dispute.objectId}</Text>
          </View>
          <View style={styles.detailRow}>
            <Text style={styles.detailLabel}>投诉押金</Text>
            <Text style={styles.detailValue}>{dispute.deposit}</Text>
          </View>
          <View style={styles.detailRow}>
            <Text style={styles.detailLabel}>创建时间</Text>
            <Text style={styles.detailValue}>{dispute.createdAt}</Text>
          </View>
          {dispute.status === 'submitted' && (
            <View style={styles.detailRow}>
              <Text style={styles.detailLabel}>回应截止</Text>
              <Text style={[styles.detailValue, { color: '#ef4444' }]}>
                {dispute.responseDeadline}
              </Text>
            </View>
          )}
        </View>

        <View style={styles.evidenceCard}>
          <Text style={styles.cardTitle}>证据材料</Text>
          {dispute.evidences.length > 0 ? (
            dispute.evidences.map((evidence) => (
              <View key={evidence.id} style={styles.evidenceItem}>
                <View style={styles.evidenceInfo}>
                  <Text style={styles.evidenceCid}>{evidence.cid}</Text>
                  <Text style={styles.evidenceMeta}>
                    {evidence.uploader === 'complainant' ? '投诉方' : '被投诉方'} · {evidence.uploadedAt}
                  </Text>
                </View>
                <Pressable style={styles.viewButton}>
                  <Text style={styles.viewButtonText}>查看</Text>
                </Pressable>
              </View>
            ))
          ) : (
            <Text style={styles.noEvidence}>暂无证据</Text>
          )}

          {canAddEvidence && (
            <View style={styles.addEvidenceRow}>
              <TextInput
                style={styles.evidenceInput}
                placeholder="输入 IPFS CID 或描述"
                placeholderTextColor="#9ca3af"
                value={evidenceInput}
                onChangeText={setEvidenceInput}
              />
              <Pressable
                style={styles.addButton}
                onPress={() => handleAction('evidence')}
              >
                <Text style={styles.addButtonText}>提交</Text>
              </Pressable>
            </View>
          )}
        </View>

        <View style={styles.actionsCard}>
          <Text style={styles.cardTitle}>可执行操作</Text>

          {canRespond && (
            <Pressable
              style={styles.actionButton}
              onPress={() => handleAction('respond')}
            >
              <View style={styles.actionContent}>
                <Text style={styles.actionIcon}>💬</Text>
                <View>
                  <Text style={styles.actionTitle}>回应投诉</Text>
                  <Text style={styles.actionDesc}>提交反驳证据并锁定押金</Text>
                </View>
              </View>
              <Text style={styles.actionArrow}>→</Text>
            </Pressable>
          )}

          {canEscalate && (
            <Pressable
              style={styles.actionButton}
              onPress={() => handleAction('escalate')}
            >
              <View style={styles.actionContent}>
                <Text style={styles.actionIcon}>⚖️</Text>
                <View>
                  <Text style={styles.actionTitle}>升级仲裁</Text>
                  <Text style={styles.actionDesc}>提交至仲裁委员会裁决</Text>
                </View>
              </View>
              <Text style={styles.actionArrow}>→</Text>
            </Pressable>
          )}

          {canWithdraw && (
            <Pressable
              style={[styles.actionButton, styles.dangerButton]}
              onPress={() => handleAction('withdraw')}
            >
              <View style={styles.actionContent}>
                <Text style={styles.actionIcon}>↩️</Text>
                <View>
                  <Text style={[styles.actionTitle, { color: '#dc2626' }]}>撤销投诉</Text>
                  <Text style={styles.actionDesc}>放弃投诉并退还部分押金</Text>
                </View>
              </View>
              <Text style={[styles.actionArrow, { color: '#dc2626' }]}>→</Text>
            </Pressable>
          )}

          {!canRespond && !canEscalate && !canWithdraw && (
            <Text style={styles.noAction}>当前状态无可执行操作</Text>
          )}
        </View>

        <View style={styles.helpCard}>
          <Text style={styles.helpTitle}>💡 申诉须知</Text>
          <Text style={styles.helpText}>
            • 被投诉方需在截止时间前回应，否则视为默认放弃{'\n'}
            • 双方都可以提交证据支持自己的主张{'\n'}
            • 调解失败可升级至仲裁委员会裁决{'\n'}
            • 仲裁结果将根据证据和规则做出公正判决
          </Text>
        </View>
      </ScrollView>

      <TransactionModal
        visible={modalVisible}
        status={status}
        isLoading={isLoading}
        title={getModalTitle()}
        description={getModalDescription()}
        onConfirm={handleConfirm}
        onClose={() => {
          setModalVisible(false);
          reset();
        }}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#fff',
    paddingTop: 50,
    paddingBottom: 12,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  backButton: {
    padding: 4,
  },
  backText: {
    fontSize: 17,
    color: '#6D28D9',
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#1f2937',
  },
  headerRight: {
    width: 50,
  },
  content: {
    flex: 1,
    padding: 16,
  },
  statusCard: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 20,
    marginBottom: 16,
  },
  statusTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 20,
  },
  statusFlow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  stepContainer: {
    flex: 1,
    alignItems: 'center',
    position: 'relative',
  },
  stepDot: {
    width: 28,
    height: 28,
    borderRadius: 14,
    backgroundColor: '#e5e7eb',
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 8,
  },
  stepDotActive: {
    backgroundColor: '#6D28D9',
  },
  stepDotCurrent: {
    backgroundColor: '#6D28D9',
    borderWidth: 3,
    borderColor: '#c4b5fd',
  },
  stepCheck: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '700',
  },
  stepLabel: {
    fontSize: 11,
    color: '#9ca3af',
    textAlign: 'center',
  },
  stepLabelActive: {
    color: '#6D28D9',
    fontWeight: '600',
  },
  stepLine: {
    position: 'absolute',
    top: 14,
    left: '60%',
    right: '-40%',
    height: 2,
    backgroundColor: '#e5e7eb',
    zIndex: -1,
  },
  stepLineActive: {
    backgroundColor: '#6D28D9',
  },
  partiesCard: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 20,
    marginBottom: 16,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 16,
  },
  partyRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-around',
  },
  party: {
    alignItems: 'center',
  },
  partyName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginTop: 8,
  },
  partyRole: {
    fontSize: 12,
    color: '#6b7280',
    marginTop: 2,
  },
  partyAddress: {
    fontSize: 11,
    color: '#9ca3af',
    marginTop: 4,
  },
  vsText: {
    fontSize: 18,
    fontWeight: '700',
    color: '#d1d5db',
  },
  detailCard: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 20,
    marginBottom: 16,
  },
  detailRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 10,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  detailLabel: {
    fontSize: 14,
    color: '#6b7280',
  },
  detailValue: {
    fontSize: 14,
    color: '#1f2937',
    fontWeight: '500',
  },
  evidenceCard: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 20,
    marginBottom: 16,
  },
  evidenceItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  evidenceInfo: {
    flex: 1,
  },
  evidenceCid: {
    fontSize: 14,
    color: '#1f2937',
    fontFamily: 'monospace',
  },
  evidenceMeta: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 2,
  },
  viewButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    backgroundColor: '#f3f4f6',
    borderRadius: 6,
  },
  viewButtonText: {
    fontSize: 13,
    color: '#6D28D9',
  },
  noEvidence: {
    fontSize: 14,
    color: '#9ca3af',
    textAlign: 'center',
    paddingVertical: 20,
  },
  addEvidenceRow: {
    flexDirection: 'row',
    marginTop: 16,
    gap: 8,
  },
  evidenceInput: {
    flex: 1,
    backgroundColor: '#f3f4f6',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
    fontSize: 14,
    color: '#1f2937',
  },
  addButton: {
    backgroundColor: '#6D28D9',
    paddingHorizontal: 16,
    borderRadius: 8,
    justifyContent: 'center',
  },
  addButtonText: {
    color: '#fff',
    fontWeight: '600',
  },
  actionsCard: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 20,
    marginBottom: 16,
  },
  actionButton: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 14,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  dangerButton: {
    borderBottomWidth: 0,
  },
  actionContent: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  actionIcon: {
    fontSize: 24,
  },
  actionTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#1f2937',
  },
  actionDesc: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 2,
  },
  actionArrow: {
    fontSize: 18,
    color: '#6D28D9',
  },
  noAction: {
    fontSize: 14,
    color: '#9ca3af',
    textAlign: 'center',
    paddingVertical: 20,
  },
  helpCard: {
    backgroundColor: '#fef3c7',
    borderRadius: 12,
    padding: 16,
    marginBottom: 32,
  },
  helpTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#92400e',
    marginBottom: 8,
  },
  helpText: {
    fontSize: 13,
    color: '#78350f',
    lineHeight: 20,
  },
});
