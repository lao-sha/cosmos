import { useMaker, useMakerConstants } from '@/src/hooks/useMaker';
import { makerService, ApplicationStatus } from '@/src/services/maker';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
  ActivityIndicator,
  Platform,
  Pressable,
  RefreshControl,
  ScrollView,
  StyleSheet,
  Text,
  View
} from 'react-native';

export default function MakerScreen() {
  const router = useRouter();
  const { isLoggedIn } = useAuthStore();
  const { isConnected } = useChainStore();
  const { makerInfo, loading, refresh, isMaker, isActiveMaker } = useMaker();
  const { depositAmountFormatted, cooldownDays } = useMakerConstants();
  const [refreshing, setRefreshing] = useState(false);

  const handleRefresh = async () => {
    setRefreshing(true);
    await refresh();
    setRefreshing(false);
  };

  if (!isLoggedIn) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>做市商</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>🏦</Text>
          <Text style={styles.emptyTitle}>请先登录</Text>
          <Text style={styles.emptySubtitle}>登录后查看做市商功能</Text>
        </View>
      </View>
    );
  }

  if (!isConnected) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>做市商</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>🔌</Text>
          <Text style={styles.emptyTitle}>未连接网络</Text>
          <Text style={styles.emptySubtitle}>请先连接区块链网络</Text>
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
        <Text style={styles.headerTitle}>做市商</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView
        style={styles.content}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} />
        }
      >
        {loading && !makerInfo ? (
          <View style={styles.loadingContainer}>
            <ActivityIndicator size="large" color="#6D28D9" />
            <Text style={styles.loadingText}>加载中...</Text>
          </View>
        ) : isMaker && makerInfo?.application ? (
          <>
            <View style={styles.statusCard}>
              <View style={styles.statusHeader}>
                <Text style={styles.statusTitle}>做市商状态</Text>
                <View style={[
                  styles.statusBadge,
                  { backgroundColor: makerService.getStatusColor(makerInfo.application.status) }
                ]}>
                  <Text style={styles.statusBadgeText}>
                    {makerService.getStatusText(makerInfo.application.status)}
                  </Text>
                </View>
              </View>
              
              <View style={styles.infoRow}>
                <Text style={styles.infoLabel}>做市商 ID</Text>
                <Text style={styles.infoValue}>#{makerInfo.makerId}</Text>
              </View>
              
              <View style={styles.infoRow}>
                <Text style={styles.infoLabel}>业务方向</Text>
                <Text style={styles.infoValue}>
                  {makerService.getDirectionText(makerInfo.application.direction)}
                </Text>
              </View>
              
              <View style={styles.infoRow}>
                <Text style={styles.infoLabel}>押金</Text>
                <Text style={styles.infoValue}>
                  {makerService.formatDeposit(makerInfo.application.deposit)}
                </Text>
              </View>

              {makerInfo.application.depositWarning && (
                <View style={styles.warningBox}>
                  <Text style={styles.warningText}>⚠️ 押金不足，请及时补充</Text>
                </View>
              )}
              
              <View style={styles.infoRow}>
                <Text style={styles.infoLabel}>已服务用户</Text>
                <Text style={styles.infoValue}>{makerInfo.application.usersServed} 人</Text>
              </View>

              {makerInfo.application.servicePaused && (
                <View style={styles.pausedBox}>
                  <Text style={styles.pausedText}>🔴 服务已暂停</Text>
                </View>
              )}
            </View>

            {isActiveMaker && (
              <View style={styles.actionCard}>
                <Text style={styles.cardTitle}>快捷操作</Text>
                <View style={styles.actionButtons}>
                  <Pressable
                    style={styles.actionButton}
                    onPress={() => router.push('/maker/manage')}
                  >
                    <Text style={styles.actionIcon}>⚙️</Text>
                    <Text style={styles.actionText}>管理</Text>
                  </Pressable>
                  <Pressable
                    style={styles.actionButton}
                    onPress={() => router.push('/maker/withdraw')}
                  >
                    <Text style={styles.actionIcon}>💰</Text>
                    <Text style={styles.actionText}>提现</Text>
                  </Pressable>
                  <Pressable
                    style={styles.actionButton}
                    onPress={() => router.push('/maker/penalties')}
                  >
                    <Text style={styles.actionIcon}>📋</Text>
                    <Text style={styles.actionText}>记录</Text>
                  </Pressable>
                </View>
              </View>
            )}

            {makerInfo.application.status === ApplicationStatus.DepositLocked && (
              <View style={styles.actionCard}>
                <Text style={styles.cardTitle}>下一步</Text>
                <Text style={styles.hintText}>
                  押金已锁定，请在 1 小时内提交资料完成申请
                </Text>
                <Pressable
                  style={styles.primaryButton}
                  onPress={() => router.push('/maker/apply')}
                >
                  <Text style={styles.primaryButtonText}>提交资料</Text>
                </Pressable>
              </View>
            )}

            {makerInfo.application.status === ApplicationStatus.PendingReview && (
              <View style={styles.infoCard}>
                <Text style={styles.cardTitle}>审核中</Text>
                <Text style={styles.hintText}>
                  您的申请正在审核中，请耐心等待治理委员会审批
                </Text>
              </View>
            )}

            {makerInfo.application.status === ApplicationStatus.Rejected && (
              <View style={styles.errorCard}>
                <Text style={styles.cardTitle}>申请被驳回</Text>
                <Text style={styles.hintText}>
                  您的做市商申请已被驳回，押金已退还
                </Text>
              </View>
            )}
          </>
        ) : (
          <>
            <View style={styles.welcomeCard}>
              <Text style={styles.welcomeIcon}>🏦</Text>
              <Text style={styles.welcomeTitle}>成为做市商</Text>
              <Text style={styles.welcomeSubtitle}>
                为 Cosmos 平台提供 OTC 和 Bridge 服务，赚取交易手续费
              </Text>
            </View>

            <View style={styles.benefitsCard}>
              <Text style={styles.cardTitle}>💎 做市商权益</Text>
              <View style={styles.benefitItem}>
                <Text style={styles.benefitIcon}>💰</Text>
                <View style={styles.benefitContent}>
                  <Text style={styles.benefitTitle}>交易手续费收入</Text>
                  <Text style={styles.benefitDesc}>每笔交易可获得溢价收益</Text>
                </View>
              </View>
              <View style={styles.benefitItem}>
                <Text style={styles.benefitIcon}>⭐</Text>
                <View style={styles.benefitContent}>
                  <Text style={styles.benefitTitle}>信用等级体系</Text>
                  <Text style={styles.benefitDesc}>优质服务可提升信用等级，降低押金要求</Text>
                </View>
              </View>
              <View style={styles.benefitItem}>
                <Text style={styles.benefitIcon}>🛡️</Text>
                <View style={styles.benefitContent}>
                  <Text style={styles.benefitTitle}>争议保护机制</Text>
                  <Text style={styles.benefitDesc}>完善的仲裁和申诉流程保障权益</Text>
                </View>
              </View>
            </View>

            <View style={styles.requirementsCard}>
              <Text style={styles.cardTitle}>📋 申请要求</Text>
              <View style={styles.requirementItem}>
                <Text style={styles.requirementDot}>•</Text>
                <Text style={styles.requirementText}>
                  锁定押金 {depositAmountFormatted}（约 1000 USD）
                </Text>
              </View>
              <View style={styles.requirementItem}>
                <Text style={styles.requirementDot}>•</Text>
                <Text style={styles.requirementText}>提供真实身份信息（脱敏展示）</Text>
              </View>
              <View style={styles.requirementItem}>
                <Text style={styles.requirementDot}>•</Text>
                <Text style={styles.requirementText}>提供 TRON 收款地址</Text>
              </View>
              <View style={styles.requirementItem}>
                <Text style={styles.requirementDot}>•</Text>
                <Text style={styles.requirementText}>通过治理委员会审核</Text>
              </View>
              <View style={styles.requirementItem}>
                <Text style={styles.requirementDot}>•</Text>
                <Text style={styles.requirementText}>
                  提现冷却期 {cooldownDays} 天
                </Text>
              </View>
            </View>

            <Pressable
              style={styles.applyButton}
              onPress={() => router.push('/maker/apply')}
            >
              <Text style={styles.applyButtonText}>立即申请</Text>
            </Pressable>
          </>
        )}

        <View style={styles.bottomPadding} />
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    backgroundColor: '#6D28D9',
    paddingTop: 50,
    paddingBottom: 16,
    paddingHorizontal: 16,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  backButton: {
    padding: 4,
  },
  backText: {
    color: '#fff',
    fontSize: 18,
  },
  headerTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#fff',
  },
  headerRight: {
    width: 40,
  },
  content: {
    flex: 1,
  },
  loadingContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    paddingTop: 100,
  },
  loadingText: {
    marginTop: 12,
    fontSize: 14,
    color: '#6b7280',
  },
  emptyContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
  },
  emptyIcon: {
    fontSize: 64,
    marginBottom: 16,
  },
  emptyTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#1f2937',
    marginBottom: 8,
  },
  emptySubtitle: {
    fontSize: 14,
    color: '#9ca3af',
  },
  statusCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  statusHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: 16,
  },
  statusTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
  },
  statusBadge: {
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 12,
  },
  statusBadgeText: {
    color: '#fff',
    fontSize: 12,
    fontWeight: '600',
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 10,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  infoLabel: {
    fontSize: 14,
    color: '#6b7280',
  },
  infoValue: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1f2937',
  },
  warningBox: {
    backgroundColor: '#fef3c7',
    padding: 12,
    borderRadius: 8,
    marginTop: 12,
  },
  warningText: {
    color: '#92400e',
    fontSize: 14,
  },
  pausedBox: {
    backgroundColor: '#fee2e2',
    padding: 12,
    borderRadius: 8,
    marginTop: 12,
  },
  pausedText: {
    color: '#991b1b',
    fontSize: 14,
  },
  actionCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 12,
  },
  actionButtons: {
    flexDirection: 'row',
    justifyContent: 'space-around',
  },
  actionButton: {
    alignItems: 'center',
    padding: 12,
  },
  actionIcon: {
    fontSize: 28,
    marginBottom: 4,
  },
  actionText: {
    fontSize: 12,
    color: '#6b7280',
  },
  hintText: {
    fontSize: 14,
    color: '#6b7280',
    marginBottom: 16,
    lineHeight: 20,
  },
  primaryButton: {
    backgroundColor: '#6D28D9',
    paddingVertical: 14,
    borderRadius: 10,
    alignItems: 'center',
  },
  primaryButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  infoCard: {
    backgroundColor: '#eff6ff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  errorCard: {
    backgroundColor: '#fef2f2',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  welcomeCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 24,
    borderRadius: 12,
    alignItems: 'center',
  },
  welcomeIcon: {
    fontSize: 64,
    marginBottom: 16,
  },
  welcomeTitle: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#1f2937',
    marginBottom: 8,
  },
  welcomeSubtitle: {
    fontSize: 14,
    color: '#6b7280',
    textAlign: 'center',
    lineHeight: 20,
  },
  benefitsCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  benefitItem: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    paddingVertical: 10,
  },
  benefitIcon: {
    fontSize: 24,
    marginRight: 12,
  },
  benefitContent: {
    flex: 1,
  },
  benefitTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 2,
  },
  benefitDesc: {
    fontSize: 12,
    color: '#6b7280',
  },
  requirementsCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  requirementItem: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    paddingVertical: 6,
  },
  requirementDot: {
    fontSize: 14,
    color: '#6D28D9',
    marginRight: 8,
    fontWeight: 'bold',
  },
  requirementText: {
    flex: 1,
    fontSize: 14,
    color: '#374151',
    lineHeight: 20,
  },
  applyButton: {
    backgroundColor: '#6D28D9',
    marginHorizontal: 16,
    marginTop: 24,
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  applyButtonText: {
    color: '#fff',
    fontSize: 18,
    fontWeight: '600',
  },
  bottomPadding: {
    height: 40,
  },
});
