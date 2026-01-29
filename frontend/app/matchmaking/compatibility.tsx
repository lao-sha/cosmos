import { TransactionModal } from '@/src/components/TransactionModal';
import { UserAvatar } from '@/src/components/UserAvatar';
import { useTransaction } from '@/src/hooks/useTransaction';
import { useAuthStore } from '@/src/stores/auth';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    Alert,
    Platform,
    Pressable,
    ScrollView,
    StyleSheet,
    Text,
    TextInput,
    View,
} from 'react-native';

interface BaziProfile {
  id: string;
  name: string;
  gender: 'male' | 'female';
  birthDate: string;
}

const MOCK_MY_BAZIS: BaziProfile[] = [
  { id: '1', name: '我的八字', gender: 'male', birthDate: '1990-05-15 08:30' },
  { id: '2', name: '备用八字', gender: 'male', birthDate: '1990-05-15 10:00' },
];

const MOCK_REQUESTS = [
  {
    id: '1',
    partyA: { name: '张三', address: '5Grw...utQY' },
    partyB: { name: '我', address: '5FHn...xPqA' },
    status: 'pending_authorization',
    createdAt: '2025-01-27',
  },
  {
    id: '2',
    partyA: { name: '我', address: '5FHn...xPqA' },
    partyB: { name: '李四', address: '5DAn...kQrB' },
    status: 'authorized',
    createdAt: '2025-01-26',
  },
  {
    id: '3',
    partyA: { name: '我', address: '5FHn...xPqA' },
    partyB: { name: '王五', address: '5Ck8...mNpC' },
    status: 'completed',
    createdAt: '2025-01-20',
  },
];

const STATUS_INFO: Record<string, { label: string; color: string }> = {
  pending_authorization: { label: '等待授权', color: '#f59e0b' },
  authorized: { label: '已授权', color: '#3b82f6' },
  rejected: { label: '已拒绝', color: '#ef4444' },
  completed: { label: '已完成', color: '#22c55e' },
  cancelled: { label: '已取消', color: '#6b7280' },
};

export default function CompatibilityScreen() {
  const router = useRouter();
  const { isLoggedIn, address } = useAuthStore();
  const { status, isLoading, reset } = useTransaction();

  const [activeTab, setActiveTab] = useState<'create' | 'requests'>('create');
  const [selectedMyBazi, setSelectedMyBazi] = useState<string>('');
  const [partnerAddress, setPartnerAddress] = useState('');
  const [partnerBaziId, setPartnerBaziId] = useState('');
  const [modalVisible, setModalVisible] = useState(false);

  const handleCreateRequest = () => {
    if (!isLoggedIn) {
      if (Platform.OS === 'web') {
        alert('请先登录');
      } else {
        Alert.alert('提示', '请先登录');
      }
      return;
    }

    if (!selectedMyBazi || !partnerAddress || !partnerBaziId) {
      const msg = '请填写完整信息';
      if (Platform.OS === 'web') {
        alert(msg);
      } else {
        Alert.alert('提示', msg);
      }
      return;
    }

    setModalVisible(true);
    reset();
  };

  const handleConfirmCreate = async () => {
    // TODO: 调用链上 create_request
  };

  const handleAuthorize = (requestId: string) => {
    // TODO: 调用链上 authorize_request
    console.log('Authorize request:', requestId);
  };

  const handleReject = (requestId: string) => {
    // TODO: 调用链上 reject_request
    console.log('Reject request:', requestId);
  };

  const handleViewReport = (requestId: string) => {
    router.push(`/matchmaking/report/${requestId}` as any);
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>合婚匹配</Text>
        <View style={styles.headerRight} />
      </View>

      <View style={styles.tabs}>
        <Pressable
          style={[styles.tab, activeTab === 'create' && styles.tabActive]}
          onPress={() => setActiveTab('create')}
        >
          <Text style={[styles.tabText, activeTab === 'create' && styles.tabTextActive]}>
            发起合婚
          </Text>
        </Pressable>
        <Pressable
          style={[styles.tab, activeTab === 'requests' && styles.tabActive]}
          onPress={() => setActiveTab('requests')}
        >
          <Text style={[styles.tabText, activeTab === 'requests' && styles.tabTextActive]}>
            我的请求
          </Text>
        </Pressable>
      </View>

      {activeTab === 'create' ? (
        <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>选择我的八字</Text>
            <View style={styles.baziList}>
              {MOCK_MY_BAZIS.map((bazi) => (
                <Pressable
                  key={bazi.id}
                  style={[
                    styles.baziItem,
                    selectedMyBazi === bazi.id && styles.baziItemSelected,
                  ]}
                  onPress={() => setSelectedMyBazi(bazi.id)}
                >
                  <View style={styles.baziInfo}>
                    <Text style={styles.baziName}>{bazi.name}</Text>
                    <Text style={styles.baziBirth}>{bazi.birthDate}</Text>
                  </View>
                  {selectedMyBazi === bazi.id && (
                    <Text style={styles.checkMark}>✓</Text>
                  )}
                </Pressable>
              ))}
            </View>
          </View>

          <View style={styles.section}>
            <Text style={styles.sectionTitle}>对方信息</Text>
            <View style={styles.inputGroup}>
              <Text style={styles.inputLabel}>对方钱包地址</Text>
              <TextInput
                style={styles.input}
                placeholder="输入对方的钱包地址"
                placeholderTextColor="#9ca3af"
                value={partnerAddress}
                onChangeText={setPartnerAddress}
              />
            </View>
            <View style={styles.inputGroup}>
              <Text style={styles.inputLabel}>对方八字 ID</Text>
              <TextInput
                style={styles.input}
                placeholder="输入对方的八字 ID"
                placeholderTextColor="#9ca3af"
                value={partnerBaziId}
                onChangeText={setPartnerBaziId}
                keyboardType="numeric"
              />
            </View>
          </View>

          <View style={styles.infoCard}>
            <Text style={styles.infoTitle}>💡 合婚说明</Text>
            <Text style={styles.infoText}>
              1. 发起合婚请求后，对方需要授权才能进行匹配分析{'\n'}
              2. 授权后系统将基于双方八字计算匹配度{'\n'}
              3. 报告包含日柱合婚、五行互补、性格匹配等多维度分析{'\n'}
              4. 合婚结果仅供参考，请理性看待
            </Text>
          </View>

          <Pressable
            style={[styles.submitButton, (!selectedMyBazi || !partnerAddress) && styles.submitButtonDisabled]}
            onPress={handleCreateRequest}
            disabled={!selectedMyBazi || !partnerAddress}
          >
            <Text style={styles.submitButtonText}>发起合婚请求</Text>
          </Pressable>
        </ScrollView>
      ) : (
        <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
          {MOCK_REQUESTS.map((request) => {
            const statusInfo = STATUS_INFO[request.status];
            const isMyRequest = request.partyA.name === '我';
            const needsAction = !isMyRequest && request.status === 'pending_authorization';

            return (
              <View key={request.id} style={styles.requestCard}>
                <View style={styles.requestHeader}>
                  <View style={styles.partiesRow}>
                    <View style={styles.party}>
                      <UserAvatar name={request.partyA.name} size="small" />
                      <Text style={styles.partyName}>{request.partyA.name}</Text>
                    </View>
                    <Text style={styles.matchIcon}>💕</Text>
                    <View style={styles.party}>
                      <UserAvatar name={request.partyB.name} size="small" />
                      <Text style={styles.partyName}>{request.partyB.name}</Text>
                    </View>
                  </View>
                  <View style={[styles.statusBadge, { backgroundColor: `${statusInfo.color}20` }]}>
                    <Text style={[styles.statusText, { color: statusInfo.color }]}>
                      {statusInfo.label}
                    </Text>
                  </View>
                </View>

                <Text style={styles.requestDate}>创建于 {request.createdAt}</Text>

                {needsAction && (
                  <View style={styles.actionRow}>
                    <Pressable
                      style={[styles.actionButton, styles.rejectButton]}
                      onPress={() => handleReject(request.id)}
                    >
                      <Text style={styles.rejectButtonText}>拒绝</Text>
                    </Pressable>
                    <Pressable
                      style={[styles.actionButton, styles.authorizeButton]}
                      onPress={() => handleAuthorize(request.id)}
                    >
                      <Text style={styles.authorizeButtonText}>授权</Text>
                    </Pressable>
                  </View>
                )}

                {request.status === 'completed' && (
                  <Pressable
                    style={styles.viewReportButton}
                    onPress={() => handleViewReport(request.id)}
                  >
                    <Text style={styles.viewReportText}>查看报告 →</Text>
                  </Pressable>
                )}
              </View>
            );
          })}
        </ScrollView>
      )}

      <TransactionModal
        visible={modalVisible}
        status={status}
        isLoading={isLoading}
        title="发起合婚请求"
        description="确认发起合婚请求？对方授权后将进行匹配分析。"
        onConfirm={handleConfirmCreate}
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
  tabs: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  tab: {
    flex: 1,
    paddingVertical: 14,
    alignItems: 'center',
    borderBottomWidth: 2,
    borderBottomColor: 'transparent',
  },
  tabActive: {
    borderBottomColor: '#6D28D9',
  },
  tabText: {
    fontSize: 15,
    color: '#6b7280',
  },
  tabTextActive: {
    color: '#6D28D9',
    fontWeight: '600',
  },
  content: {
    flex: 1,
    padding: 16,
  },
  section: {
    marginBottom: 24,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 12,
  },
  baziList: {
    gap: 8,
  },
  baziItem: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#fff',
    padding: 16,
    borderRadius: 12,
    borderWidth: 2,
    borderColor: 'transparent',
  },
  baziItemSelected: {
    borderColor: '#6D28D9',
    backgroundColor: '#f5f3ff',
  },
  baziInfo: {
    flex: 1,
  },
  baziName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
  },
  baziBirth: {
    fontSize: 13,
    color: '#6b7280',
    marginTop: 2,
  },
  checkMark: {
    fontSize: 20,
    color: '#6D28D9',
    fontWeight: '700',
  },
  inputGroup: {
    marginBottom: 16,
  },
  inputLabel: {
    fontSize: 14,
    color: '#374151',
    marginBottom: 8,
  },
  input: {
    backgroundColor: '#fff',
    borderRadius: 12,
    paddingHorizontal: 16,
    paddingVertical: 14,
    fontSize: 15,
    color: '#1f2937',
    borderWidth: 1,
    borderColor: '#e5e7eb',
  },
  infoCard: {
    backgroundColor: '#fef3c7',
    borderRadius: 12,
    padding: 16,
    marginBottom: 24,
  },
  infoTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#92400e',
    marginBottom: 8,
  },
  infoText: {
    fontSize: 13,
    color: '#78350f',
    lineHeight: 20,
  },
  submitButton: {
    backgroundColor: '#6D28D9',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
    marginBottom: 32,
  },
  submitButtonDisabled: {
    backgroundColor: '#d1d5db',
  },
  submitButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  requestCard: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 16,
    marginBottom: 12,
  },
  requestHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  partiesRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  party: {
    alignItems: 'center',
    gap: 4,
  },
  partyName: {
    fontSize: 13,
    color: '#374151',
  },
  matchIcon: {
    fontSize: 20,
  },
  statusBadge: {
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
  },
  statusText: {
    fontSize: 12,
    fontWeight: '600',
  },
  requestDate: {
    fontSize: 12,
    color: '#9ca3af',
  },
  actionRow: {
    flexDirection: 'row',
    gap: 12,
    marginTop: 16,
  },
  actionButton: {
    flex: 1,
    paddingVertical: 10,
    borderRadius: 8,
    alignItems: 'center',
  },
  rejectButton: {
    backgroundColor: '#fee2e2',
  },
  rejectButtonText: {
    color: '#dc2626',
    fontWeight: '600',
  },
  authorizeButton: {
    backgroundColor: '#6D28D9',
  },
  authorizeButtonText: {
    color: '#fff',
    fontWeight: '600',
  },
  viewReportButton: {
    marginTop: 12,
    alignItems: 'center',
  },
  viewReportText: {
    color: '#6D28D9',
    fontSize: 14,
    fontWeight: '600',
  },
});
