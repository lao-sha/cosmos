import { useAuthStore } from '@/src/stores/auth';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    FlatList,
    Pressable,
    StyleSheet,
    Text,
    View,
} from 'react-native';

type ComplaintStatus =
  | 'submitted'
  | 'responded'
  | 'mediating'
  | 'arbitrating'
  | 'resolved_complainant_win'
  | 'resolved_respondent_win'
  | 'resolved_settlement'
  | 'withdrawn';

type ComplaintType = 'service_quality' | 'payment_issue' | 'fraud' | 'harassment' | 'other';

interface Complaint {
  id: string;
  domain: string;
  objectId: string;
  complaintType: ComplaintType;
  status: ComplaintStatus;
  isComplainant: boolean;
  counterparty: string;
  createdAt: string;
  amount?: string;
}

const MOCK_COMPLAINTS: Complaint[] = [
  {
    id: '1',
    domain: 'otc',
    objectId: '12345',
    complaintType: 'payment_issue',
    status: 'submitted',
    isComplainant: true,
    counterparty: '5Grw...utQY',
    createdAt: '2025-01-27',
    amount: '100 USDT',
  },
  {
    id: '2',
    domain: 'divination',
    objectId: '67890',
    complaintType: 'service_quality',
    status: 'responded',
    isComplainant: false,
    counterparty: '5DAn...kQrB',
    createdAt: '2025-01-25',
  },
  {
    id: '3',
    domain: 'otc',
    objectId: '11111',
    complaintType: 'fraud',
    status: 'resolved_complainant_win',
    isComplainant: true,
    counterparty: '5Ck8...mNpC',
    createdAt: '2025-01-20',
    amount: '500 USDT',
  },
];

const STATUS_INFO: Record<ComplaintStatus, { label: string; color: string }> = {
  submitted: { label: '已提交', color: '#f59e0b' },
  responded: { label: '已回应', color: '#3b82f6' },
  mediating: { label: '调解中', color: '#8b5cf6' },
  arbitrating: { label: '仲裁中', color: '#ec4899' },
  resolved_complainant_win: { label: '投诉方胜', color: '#22c55e' },
  resolved_respondent_win: { label: '被投诉方胜', color: '#ef4444' },
  resolved_settlement: { label: '和解', color: '#06b6d4' },
  withdrawn: { label: '已撤销', color: '#6b7280' },
};

const TYPE_INFO: Record<ComplaintType, { label: string; icon: string }> = {
  service_quality: { label: '服务质量', icon: '⭐' },
  payment_issue: { label: '付款问题', icon: '💳' },
  fraud: { label: '欺诈', icon: '🚨' },
  harassment: { label: '骚扰', icon: '🚫' },
  other: { label: '其他', icon: '📝' },
};

const DOMAIN_INFO: Record<string, string> = {
  otc: 'OTC交易',
  divination: '占卜服务',
  matchmaking: '婚恋匹配',
};

export default function DisputesScreen() {
  const router = useRouter();
  const { isLoggedIn } = useAuthStore();

  const [filter, setFilter] = useState<'all' | 'active' | 'resolved'>('all');

  const filteredComplaints = MOCK_COMPLAINTS.filter((c) => {
    if (filter === 'all') return true;
    if (filter === 'active') {
      return ['submitted', 'responded', 'mediating', 'arbitrating'].includes(c.status);
    }
    return ['resolved_complainant_win', 'resolved_respondent_win', 'resolved_settlement', 'withdrawn'].includes(c.status);
  });

  const handleViewDetail = (id: string) => {
    router.push(`/disputes/${id}` as any);
  };

  const renderItem = ({ item }: { item: Complaint }) => {
    const statusInfo = STATUS_INFO[item.status];
    const typeInfo = TYPE_INFO[item.complaintType];

    return (
      <Pressable style={styles.card} onPress={() => handleViewDetail(item.id)}>
        <View style={styles.cardHeader}>
          <View style={styles.typeRow}>
            <Text style={styles.typeIcon}>{typeInfo.icon}</Text>
            <Text style={styles.typeLabel}>{typeInfo.label}</Text>
            <View style={styles.domainBadge}>
              <Text style={styles.domainText}>{DOMAIN_INFO[item.domain]}</Text>
            </View>
          </View>
          <View style={[styles.statusBadge, { backgroundColor: `${statusInfo.color}15` }]}>
            <Text style={[styles.statusText, { color: statusInfo.color }]}>
              {statusInfo.label}
            </Text>
          </View>
        </View>

        <View style={styles.cardContent}>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>角色</Text>
            <Text style={[styles.infoValue, { color: item.isComplainant ? '#f59e0b' : '#3b82f6' }]}>
              {item.isComplainant ? '投诉方' : '被投诉方'}
            </Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>对方</Text>
            <Text style={styles.infoValue}>{item.counterparty}</Text>
          </View>
          {item.amount && (
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>涉及金额</Text>
              <Text style={styles.infoValue}>{item.amount}</Text>
            </View>
          )}
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>创建时间</Text>
            <Text style={styles.infoValue}>{item.createdAt}</Text>
          </View>
        </View>

        <View style={styles.cardFooter}>
          <Text style={styles.viewDetail}>查看详情 →</Text>
        </View>
      </Pressable>
    );
  };

  if (!isLoggedIn) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>我的申诉</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.centerContent}>
          <Text style={styles.emptyText}>请先登录</Text>
        </View>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>我的申诉</Text>
        <View style={styles.headerRight} />
      </View>

      <View style={styles.filterRow}>
        {(['all', 'active', 'resolved'] as const).map((f) => (
          <Pressable
            key={f}
            style={[styles.filterButton, filter === f && styles.filterActive]}
            onPress={() => setFilter(f)}
          >
            <Text style={[styles.filterText, filter === f && styles.filterTextActive]}>
              {f === 'all' ? '全部' : f === 'active' ? '进行中' : '已结束'}
            </Text>
          </Pressable>
        ))}
      </View>

      <FlatList
        data={filteredComplaints}
        renderItem={renderItem}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.list}
        showsVerticalScrollIndicator={false}
        ListEmptyComponent={
          <View style={styles.centerContent}>
            <Text style={styles.emptyEmoji}>📋</Text>
            <Text style={styles.emptyText}>暂无申诉记录</Text>
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
  filterRow: {
    flexDirection: 'row',
    padding: 16,
    gap: 8,
  },
  filterButton: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 20,
    backgroundColor: '#fff',
  },
  filterActive: {
    backgroundColor: '#6D28D9',
  },
  filterText: {
    fontSize: 14,
    color: '#6b7280',
  },
  filterTextActive: {
    color: '#fff',
    fontWeight: '600',
  },
  list: {
    padding: 16,
    paddingTop: 0,
  },
  card: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 16,
    marginBottom: 12,
  },
  cardHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  typeRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  typeIcon: {
    fontSize: 18,
  },
  typeLabel: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
  },
  domainBadge: {
    backgroundColor: '#f3f4f6',
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 4,
  },
  domainText: {
    fontSize: 11,
    color: '#6b7280',
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
  cardContent: {
    gap: 8,
    paddingBottom: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  infoLabel: {
    fontSize: 14,
    color: '#6b7280',
  },
  infoValue: {
    fontSize: 14,
    color: '#1f2937',
  },
  cardFooter: {
    paddingTop: 12,
    alignItems: 'center',
  },
  viewDetail: {
    fontSize: 14,
    color: '#6D28D9',
    fontWeight: '500',
  },
  centerContent: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 40,
  },
  emptyEmoji: {
    fontSize: 48,
    marginBottom: 16,
  },
  emptyText: {
    fontSize: 16,
    color: '#6b7280',
  },
});
