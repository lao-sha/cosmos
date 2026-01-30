import { useMaker, useMakerConstants } from '@/src/hooks/useMaker';
import { useTransaction } from '@/src/hooks/useTransaction';
import { makerService, ApplicationStatus, WithdrawalStatus } from '@/src/services/maker';
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
  TextInput,
  View
} from 'react-native';

export default function MakerManageScreen() {
  const router = useRouter();
  const { isLoggedIn, mnemonic } = useAuthStore();
  const { isConnected } = useChainStore();
  const { makerInfo, loading, refresh, isActiveMaker } = useMaker();
  const { cooldownDays } = useMakerConstants();
  const {
    requestMakerWithdrawal,
    executeMakerWithdrawal,
    cancelMakerWithdrawal,
    replenishMakerDeposit,
    isTxLoading,
  } = useTransaction();

  const [refreshing, setRefreshing] = useState(false);
  const [withdrawAmount, setWithdrawAmount] = useState('');
  const [showWithdrawForm, setShowWithdrawForm] = useState(false);

  const showAlert = (title: string, message: string) => {
    if (Platform.OS === 'web') {
      window.alert(`${title}\n${message}`);
    } else {
      const { Alert } = require('react-native');
      Alert.alert(title, message);
    }
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    await refresh();
    setRefreshing(false);
  };

  const handleReplenish = async () => {
    if (!mnemonic) return;

    await replenishMakerDeposit(mnemonic, {
      onSuccess: () => {
        showAlert('成功', '押金已补充');
        refresh();
      },
      onError: (error) => {
        showAlert('失败', error);
      },
    });
  };

  const handleRequestWithdraw = async () => {
    if (!mnemonic) return;

    const amount = parseFloat(withdrawAmount);
    if (isNaN(amount) || amount <= 0) {
      showAlert('错误', '请输入有效的提现金额');
      return;
    }

    const amountInWei = BigInt(Math.floor(amount * 1e18)).toString();

    await requestMakerWithdrawal(mnemonic, amountInWei, {
      onSuccess: () => {
        showAlert('成功', `提现申请已提交，${cooldownDays} 天后可执行`);
        setWithdrawAmount('');
        setShowWithdrawForm(false);
        refresh();
      },
      onError: (error) => {
        showAlert('失败', error);
      },
    });
  };

  const handleExecuteWithdraw = async () => {
    if (!mnemonic) return;

    await executeMakerWithdrawal(mnemonic, {
      onSuccess: () => {
        showAlert('成功', '提现已执行');
        refresh();
      },
      onError: (error) => {
        showAlert('失败', error);
      },
    });
  };

  const handleCancelWithdraw = async () => {
    if (!mnemonic) return;

    const doCancel = async () => {
      await cancelMakerWithdrawal(mnemonic, {
        onSuccess: () => {
          showAlert('成功', '提现请求已取消');
          refresh();
        },
        onError: (error) => {
          showAlert('失败', error);
        },
      });
    };

    if (Platform.OS === 'web') {
      if (window.confirm('确定要取消提现请求吗？')) {
        doCancel();
      }
    } else {
      const { Alert } = require('react-native');
      Alert.alert('取消提现', '确定要取消提现请求吗？', [
        { text: '取消', style: 'cancel' },
        { text: '确定', onPress: doCancel },
      ]);
    }
  };

  if (!isLoggedIn || !isConnected) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>做市商管理</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>🔌</Text>
          <Text style={styles.emptyTitle}>请先登录并连接网络</Text>
        </View>
      </View>
    );
  }

  if (!isActiveMaker) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>做市商管理</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>🚫</Text>
          <Text style={styles.emptyTitle}>您还不是活跃做市商</Text>
          <Pressable
            style={styles.linkButton}
            onPress={() => router.replace('/maker')}
          >
            <Text style={styles.linkButtonText}>前往做市商中心</Text>
          </Pressable>
        </View>
      </View>
    );
  }

  const app = makerInfo?.application;
  const withdrawal = makerInfo?.withdrawalRequest;
  const canExecuteWithdraw = withdrawal?.status === WithdrawalStatus.Pending &&
    withdrawal.executableAt <= Math.floor(Date.now() / 1000);

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>做市商管理</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView
        style={styles.content}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} />
        }
      >
        {loading ? (
          <View style={styles.loadingContainer}>
            <ActivityIndicator size="large" color="#6D28D9" />
          </View>
        ) : (
          <>
            <View style={styles.card}>
              <Text style={styles.cardTitle}>💰 押金管理</Text>
              
              <View style={styles.depositBox}>
                <Text style={styles.depositLabel}>当前押金</Text>
                <Text style={styles.depositAmount}>
                  {app ? makerService.formatDeposit(app.deposit) : '0 COS'}
                </Text>
                <Text style={styles.depositUsd}>
                  ≈ {app ? makerService.formatUsdValue(app.targetDepositUsd) : '$0'}
                </Text>
              </View>

              {app?.depositWarning && (
                <View style={styles.warningBox}>
                  <Text style={styles.warningText}>
                    ⚠️ 押金价值不足，请及时补充以避免服务受限
                  </Text>
                  <Pressable
                    style={[styles.replenishButton, isTxLoading && styles.buttonDisabled]}
                    onPress={handleReplenish}
                    disabled={isTxLoading}
                  >
                    {isTxLoading ? (
                      <ActivityIndicator color="#fff" size="small" />
                    ) : (
                      <Text style={styles.replenishButtonText}>立即补充</Text>
                    )}
                  </Pressable>
                </View>
              )}
            </View>

            <View style={styles.card}>
              <Text style={styles.cardTitle}>💸 提现管理</Text>
              
              {withdrawal?.status === WithdrawalStatus.Pending ? (
                <View style={styles.withdrawalInfo}>
                  <View style={styles.infoRow}>
                    <Text style={styles.infoLabel}>提现金额</Text>
                    <Text style={styles.infoValue}>
                      {makerService.formatDeposit(withdrawal.amount)}
                    </Text>
                  </View>
                  <View style={styles.infoRow}>
                    <Text style={styles.infoLabel}>申请时间</Text>
                    <Text style={styles.infoValue}>
                      {new Date(withdrawal.requestedAt * 1000).toLocaleString()}
                    </Text>
                  </View>
                  <View style={styles.infoRow}>
                    <Text style={styles.infoLabel}>可执行时间</Text>
                    <Text style={styles.infoValue}>
                      {new Date(withdrawal.executableAt * 1000).toLocaleString()}
                    </Text>
                  </View>
                  
                  <View style={styles.withdrawalActions}>
                    {canExecuteWithdraw ? (
                      <Pressable
                        style={[styles.executeButton, isTxLoading && styles.buttonDisabled]}
                        onPress={handleExecuteWithdraw}
                        disabled={isTxLoading}
                      >
                        {isTxLoading ? (
                          <ActivityIndicator color="#fff" size="small" />
                        ) : (
                          <Text style={styles.executeButtonText}>执行提现</Text>
                        )}
                      </Pressable>
                    ) : (
                      <View style={styles.cooldownBox}>
                        <Text style={styles.cooldownText}>
                          ⏳ 冷却期中，请等待...
                        </Text>
                      </View>
                    )}
                    <Pressable
                      style={[styles.cancelWithdrawButton, isTxLoading && styles.buttonDisabled]}
                      onPress={handleCancelWithdraw}
                      disabled={isTxLoading}
                    >
                      <Text style={styles.cancelWithdrawButtonText}>取消提现</Text>
                    </Pressable>
                  </View>
                </View>
              ) : showWithdrawForm ? (
                <View style={styles.withdrawForm}>
                  <Text style={styles.formLabel}>提现金额 (COS)</Text>
                  <TextInput
                    style={styles.input}
                    value={withdrawAmount}
                    onChangeText={setWithdrawAmount}
                    placeholder="输入提现金额"
                    placeholderTextColor="#9ca3af"
                    keyboardType="decimal-pad"
                  />
                  <Text style={styles.formHint}>
                    提现需等待 {cooldownDays} 天冷却期
                  </Text>
                  <View style={styles.formActions}>
                    <Pressable
                      style={styles.formCancelButton}
                      onPress={() => {
                        setShowWithdrawForm(false);
                        setWithdrawAmount('');
                      }}
                    >
                      <Text style={styles.formCancelButtonText}>取消</Text>
                    </Pressable>
                    <Pressable
                      style={[styles.formSubmitButton, isTxLoading && styles.buttonDisabled]}
                      onPress={handleRequestWithdraw}
                      disabled={isTxLoading}
                    >
                      {isTxLoading ? (
                        <ActivityIndicator color="#fff" size="small" />
                      ) : (
                        <Text style={styles.formSubmitButtonText}>申请提现</Text>
                      )}
                    </Pressable>
                  </View>
                </View>
              ) : (
                <Pressable
                  style={styles.withdrawButton}
                  onPress={() => setShowWithdrawForm(true)}
                >
                  <Text style={styles.withdrawButtonText}>申请提现</Text>
                </Pressable>
              )}
            </View>

            <View style={styles.card}>
              <Text style={styles.cardTitle}>📊 服务信息</Text>
              
              <View style={styles.infoRow}>
                <Text style={styles.infoLabel}>业务方向</Text>
                <Text style={styles.infoValue}>
                  {app ? makerService.getDirectionText(app.direction) : '-'}
                </Text>
              </View>
              <View style={styles.infoRow}>
                <Text style={styles.infoLabel}>Buy 溢价</Text>
                <Text style={styles.infoValue}>
                  {app ? `${(app.buyPremiumBps / 100).toFixed(2)}%` : '-'}
                </Text>
              </View>
              <View style={styles.infoRow}>
                <Text style={styles.infoLabel}>Sell 溢价</Text>
                <Text style={styles.infoValue}>
                  {app ? `${(app.sellPremiumBps / 100).toFixed(2)}%` : '-'}
                </Text>
              </View>
              <View style={styles.infoRow}>
                <Text style={styles.infoLabel}>已服务用户</Text>
                <Text style={styles.infoValue}>{app?.usersServed || 0} 人</Text>
              </View>
              <View style={styles.infoRow}>
                <Text style={styles.infoLabel}>服务状态</Text>
                <Text style={[
                  styles.infoValue,
                  { color: app?.servicePaused ? '#ef4444' : '#10b981' }
                ]}>
                  {app?.servicePaused ? '已暂停' : '正常'}
                </Text>
              </View>
            </View>

            <View style={styles.card}>
              <Text style={styles.cardTitle}>👤 个人信息</Text>
              
              <View style={styles.infoRow}>
                <Text style={styles.infoLabel}>姓名（脱敏）</Text>
                <Text style={styles.infoValue}>{app?.maskedFullName || '-'}</Text>
              </View>
              <View style={styles.infoRow}>
                <Text style={styles.infoLabel}>TRON 地址</Text>
                <Text style={styles.infoValue} numberOfLines={1}>
                  {app?.tronAddress || '-'}
                </Text>
              </View>
              <View style={styles.infoRow}>
                <Text style={styles.infoLabel}>微信号</Text>
                <Text style={styles.infoValue}>{app?.wechatId || '-'}</Text>
              </View>
            </View>

            {makerInfo?.penalties && makerInfo.penalties.length > 0 && (
              <Pressable
                style={styles.penaltiesLink}
                onPress={() => router.push('/maker/penalties')}
              >
                <Text style={styles.penaltiesLinkText}>
                  📋 查看惩罚记录 ({makerInfo.penalties.length})
                </Text>
              </Pressable>
            )}

            <View style={styles.bottomPadding} />
          </>
        )}
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
  linkButton: {
    marginTop: 16,
    paddingVertical: 12,
    paddingHorizontal: 24,
    backgroundColor: '#6D28D9',
    borderRadius: 8,
  },
  linkButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  card: {
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
  depositBox: {
    backgroundColor: '#f0fdf4',
    padding: 16,
    borderRadius: 10,
    alignItems: 'center',
  },
  depositLabel: {
    fontSize: 12,
    color: '#6b7280',
    marginBottom: 4,
  },
  depositAmount: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#16a34a',
  },
  depositUsd: {
    fontSize: 14,
    color: '#6b7280',
    marginTop: 4,
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
    marginBottom: 12,
  },
  replenishButton: {
    backgroundColor: '#f59e0b',
    paddingVertical: 10,
    borderRadius: 8,
    alignItems: 'center',
  },
  replenishButtonText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
  },
  buttonDisabled: {
    opacity: 0.6,
  },
  withdrawalInfo: {
    backgroundColor: '#f9fafb',
    padding: 12,
    borderRadius: 8,
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 8,
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
    maxWidth: '60%',
  },
  withdrawalActions: {
    marginTop: 12,
    gap: 8,
  },
  executeButton: {
    backgroundColor: '#10b981',
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
  },
  executeButtonText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
  },
  cooldownBox: {
    backgroundColor: '#eff6ff',
    padding: 12,
    borderRadius: 8,
    alignItems: 'center',
  },
  cooldownText: {
    color: '#3b82f6',
    fontSize: 14,
  },
  cancelWithdrawButton: {
    backgroundColor: '#f3f4f6',
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
  },
  cancelWithdrawButtonText: {
    color: '#6b7280',
    fontSize: 14,
    fontWeight: '600',
  },
  withdrawButton: {
    backgroundColor: '#6D28D9',
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
  },
  withdrawButtonText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
  },
  withdrawForm: {
    backgroundColor: '#f9fafb',
    padding: 12,
    borderRadius: 8,
  },
  formLabel: {
    fontSize: 14,
    fontWeight: '500',
    color: '#374151',
    marginBottom: 6,
  },
  input: {
    backgroundColor: '#fff',
    borderWidth: 1,
    borderColor: '#e5e7eb',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 12,
    fontSize: 16,
    color: '#1f2937',
  },
  formHint: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 4,
  },
  formActions: {
    flexDirection: 'row',
    gap: 8,
    marginTop: 12,
  },
  formCancelButton: {
    flex: 1,
    backgroundColor: '#f3f4f6',
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
  },
  formCancelButtonText: {
    color: '#6b7280',
    fontSize: 14,
    fontWeight: '600',
  },
  formSubmitButton: {
    flex: 2,
    backgroundColor: '#6D28D9',
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
  },
  formSubmitButtonText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
  },
  penaltiesLink: {
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    backgroundColor: '#fef2f2',
    borderRadius: 12,
    alignItems: 'center',
  },
  penaltiesLinkText: {
    color: '#991b1b',
    fontSize: 14,
    fontWeight: '500',
  },
  bottomPadding: {
    height: 40,
  },
});
