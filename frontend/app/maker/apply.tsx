import { useMaker, useMakerConstants } from '@/src/hooks/useMaker';
import { useTransaction } from '@/src/hooks/useTransaction';
import { ApplicationStatus } from '@/src/services/maker';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
  ActivityIndicator,
  Platform,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View
} from 'react-native';

export default function MakerApplyScreen() {
  const router = useRouter();
  const { isLoggedIn, mnemonic } = useAuthStore();
  const { isConnected } = useChainStore();
  const { makerInfo, refresh, isMaker, isDepositLocked } = useMaker();
  const { depositAmountFormatted } = useMakerConstants();
  const {
    lockMakerDeposit,
    submitMakerInfo,
    cancelMaker,
    isTxLoading,
  } = useTransaction();

  const [step, setStep] = useState<'deposit' | 'info'>('deposit');
  const [realName, setRealName] = useState('');
  const [idCardNumber, setIdCardNumber] = useState('');
  const [birthday, setBirthday] = useState('');
  const [tronAddress, setTronAddress] = useState('');
  const [wechatId, setWechatId] = useState('');

  const showAlert = (title: string, message: string) => {
    if (Platform.OS === 'web') {
      window.alert(`${title}\n${message}`);
    } else {
      const { Alert } = require('react-native');
      Alert.alert(title, message);
    }
  };

  const handleLockDeposit = async () => {
    if (!mnemonic) {
      showAlert('错误', '请先登录');
      return;
    }

    const result = await lockMakerDeposit(mnemonic, {
      onSuccess: () => {
        showAlert('成功', '押金已锁定，请继续提交资料');
        refresh();
        setStep('info');
      },
      onError: (error) => {
        showAlert('失败', error);
      },
    });
  };

  const handleSubmitInfo = async () => {
    if (!mnemonic) {
      showAlert('错误', '请先登录');
      return;
    }

    if (!realName.trim()) {
      showAlert('错误', '请输入真实姓名');
      return;
    }
    if (!idCardNumber.trim()) {
      showAlert('错误', '请输入身份证号');
      return;
    }
    if (!birthday.trim()) {
      showAlert('错误', '请输入生日');
      return;
    }
    if (!tronAddress.trim() || !tronAddress.startsWith('T')) {
      showAlert('错误', '请输入有效的 TRON 地址（以 T 开头）');
      return;
    }
    if (!wechatId.trim()) {
      showAlert('错误', '请输入微信号');
      return;
    }

    const result = await submitMakerInfo(
      mnemonic,
      realName,
      idCardNumber,
      birthday,
      tronAddress,
      wechatId,
      {
        onSuccess: () => {
          showAlert('成功', '资料已提交，请等待审核');
          refresh();
          router.replace('/maker');
        },
        onError: (error) => {
          showAlert('失败', error);
        },
      }
    );
  };

  const handleCancel = async () => {
    if (!mnemonic) return;

    const doCancel = async () => {
      await cancelMaker(mnemonic, {
        onSuccess: () => {
          showAlert('成功', '申请已取消，押金已退还');
          refresh();
          router.replace('/maker');
        },
        onError: (error) => {
          showAlert('失败', error);
        },
      });
    };

    if (Platform.OS === 'web') {
      if (window.confirm('确定要取消申请吗？押金将退还到您的账户。')) {
        doCancel();
      }
    } else {
      const { Alert } = require('react-native');
      Alert.alert('取消申请', '确定要取消申请吗？押金将退还到您的账户。', [
        { text: '取消', style: 'cancel' },
        { text: '确定', onPress: doCancel, style: 'destructive' },
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
          <Text style={styles.headerTitle}>申请做市商</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>🔌</Text>
          <Text style={styles.emptyTitle}>请先登录并连接网络</Text>
        </View>
      </View>
    );
  }

  if (isMaker && makerInfo?.application?.status === ApplicationStatus.PendingReview) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>申请做市商</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>⏳</Text>
          <Text style={styles.emptyTitle}>申请审核中</Text>
          <Text style={styles.emptySubtitle}>请耐心等待治理委员会审批</Text>
        </View>
      </View>
    );
  }

  if (isMaker && makerInfo?.application?.status === ApplicationStatus.Active) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>申请做市商</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>✅</Text>
          <Text style={styles.emptyTitle}>您已是做市商</Text>
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

  const currentStep = isDepositLocked ? 'info' : step;

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>申请做市商</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content}>
        <View style={styles.stepsContainer}>
          <View style={styles.stepItem}>
            <View style={[
              styles.stepCircle,
              currentStep === 'deposit' ? styles.stepActive : styles.stepCompleted
            ]}>
              <Text style={styles.stepNumber}>
                {currentStep === 'info' ? '✓' : '1'}
              </Text>
            </View>
            <Text style={styles.stepText}>锁定押金</Text>
          </View>
          <View style={styles.stepLine} />
          <View style={styles.stepItem}>
            <View style={[
              styles.stepCircle,
              currentStep === 'info' ? styles.stepActive : styles.stepPending
            ]}>
              <Text style={styles.stepNumber}>2</Text>
            </View>
            <Text style={styles.stepText}>提交资料</Text>
          </View>
          <View style={styles.stepLine} />
          <View style={styles.stepItem}>
            <View style={[styles.stepCircle, styles.stepPending]}>
              <Text style={styles.stepNumber}>3</Text>
            </View>
            <Text style={styles.stepText}>等待审核</Text>
          </View>
        </View>

        {currentStep === 'deposit' ? (
          <View style={styles.card}>
            <Text style={styles.cardTitle}>💰 锁定押金</Text>
            <Text style={styles.cardDesc}>
              成为做市商需要锁定 {depositAmountFormatted} 作为押金，用于保障交易安全。
            </Text>
            
            <View style={styles.depositInfo}>
              <View style={styles.depositRow}>
                <Text style={styles.depositLabel}>押金金额</Text>
                <Text style={styles.depositValue}>{depositAmountFormatted}</Text>
              </View>
              <View style={styles.depositRow}>
                <Text style={styles.depositLabel}>目标价值</Text>
                <Text style={styles.depositValue}>≈ $1,000 USD</Text>
              </View>
            </View>

            <View style={styles.noticeBox}>
              <Text style={styles.noticeTitle}>⚠️ 注意事项</Text>
              <Text style={styles.noticeText}>
                • 押金锁定后需在 1 小时内提交资料{'\n'}
                • 审核通过前可随时取消并退还押金{'\n'}
                • 提现需等待 7 天冷却期{'\n'}
                • 违规行为将从押金中扣除罚款
              </Text>
            </View>

            <Pressable
              style={[styles.primaryButton, isTxLoading && styles.buttonDisabled]}
              onPress={handleLockDeposit}
              disabled={isTxLoading}
            >
              {isTxLoading ? (
                <ActivityIndicator color="#fff" />
              ) : (
                <Text style={styles.primaryButtonText}>锁定押金</Text>
              )}
            </Pressable>
          </View>
        ) : (
          <View style={styles.card}>
            <Text style={styles.cardTitle}>📝 提交资料</Text>
            <Text style={styles.cardDesc}>
              请填写真实信息，用于身份验证和交易保障。信息将脱敏展示给用户。
            </Text>

            <View style={styles.formGroup}>
              <Text style={styles.label}>真实姓名 *</Text>
              <TextInput
                style={styles.input}
                value={realName}
                onChangeText={setRealName}
                placeholder="请输入真实姓名"
                placeholderTextColor="#9ca3af"
              />
            </View>

            <View style={styles.formGroup}>
              <Text style={styles.label}>身份证号 *</Text>
              <TextInput
                style={styles.input}
                value={idCardNumber}
                onChangeText={setIdCardNumber}
                placeholder="请输入身份证号"
                placeholderTextColor="#9ca3af"
              />
            </View>

            <View style={styles.formGroup}>
              <Text style={styles.label}>生日 *</Text>
              <TextInput
                style={styles.input}
                value={birthday}
                onChangeText={setBirthday}
                placeholder="YYYY-MM-DD"
                placeholderTextColor="#9ca3af"
              />
            </View>

            <View style={styles.formGroup}>
              <Text style={styles.label}>TRON 收款地址 *</Text>
              <TextInput
                style={styles.input}
                value={tronAddress}
                onChangeText={setTronAddress}
                placeholder="T..."
                placeholderTextColor="#9ca3af"
                autoCapitalize="none"
              />
              <Text style={styles.hint}>用于接收 USDT 付款</Text>
            </View>

            <View style={styles.formGroup}>
              <Text style={styles.label}>微信号 *</Text>
              <TextInput
                style={styles.input}
                value={wechatId}
                onChangeText={setWechatId}
                placeholder="请输入微信号"
                placeholderTextColor="#9ca3af"
                autoCapitalize="none"
              />
              <Text style={styles.hint}>用于用户联系</Text>
            </View>

            <View style={styles.buttonRow}>
              <Pressable
                style={styles.cancelButton}
                onPress={handleCancel}
                disabled={isTxLoading}
              >
                <Text style={styles.cancelButtonText}>取消申请</Text>
              </Pressable>
              <Pressable
                style={[styles.submitButton, isTxLoading && styles.buttonDisabled]}
                onPress={handleSubmitInfo}
                disabled={isTxLoading}
              >
                {isTxLoading ? (
                  <ActivityIndicator color="#fff" />
                ) : (
                  <Text style={styles.submitButtonText}>提交资料</Text>
                )}
              </Pressable>
            </View>
          </View>
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
  stepsContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 24,
    paddingHorizontal: 16,
  },
  stepItem: {
    alignItems: 'center',
  },
  stepCircle: {
    width: 32,
    height: 32,
    borderRadius: 16,
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 4,
  },
  stepActive: {
    backgroundColor: '#6D28D9',
  },
  stepCompleted: {
    backgroundColor: '#10b981',
  },
  stepPending: {
    backgroundColor: '#d1d5db',
  },
  stepNumber: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
  },
  stepText: {
    fontSize: 12,
    color: '#6b7280',
  },
  stepLine: {
    width: 40,
    height: 2,
    backgroundColor: '#d1d5db',
    marginHorizontal: 8,
    marginBottom: 20,
  },
  card: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    padding: 16,
    borderRadius: 12,
  },
  cardTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 8,
  },
  cardDesc: {
    fontSize: 14,
    color: '#6b7280',
    marginBottom: 16,
    lineHeight: 20,
  },
  depositInfo: {
    backgroundColor: '#f9fafb',
    padding: 16,
    borderRadius: 8,
    marginBottom: 16,
  },
  depositRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 8,
  },
  depositLabel: {
    fontSize: 14,
    color: '#6b7280',
  },
  depositValue: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1f2937',
  },
  noticeBox: {
    backgroundColor: '#fef3c7',
    padding: 12,
    borderRadius: 8,
    marginBottom: 16,
  },
  noticeTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#92400e',
    marginBottom: 8,
  },
  noticeText: {
    fontSize: 13,
    color: '#92400e',
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
  buttonDisabled: {
    opacity: 0.6,
  },
  formGroup: {
    marginBottom: 16,
  },
  label: {
    fontSize: 14,
    fontWeight: '500',
    color: '#374151',
    marginBottom: 6,
  },
  input: {
    backgroundColor: '#f9fafb',
    borderWidth: 1,
    borderColor: '#e5e7eb',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 12,
    fontSize: 16,
    color: '#1f2937',
  },
  hint: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 4,
  },
  buttonRow: {
    flexDirection: 'row',
    gap: 12,
    marginTop: 8,
  },
  cancelButton: {
    flex: 1,
    backgroundColor: '#f3f4f6',
    paddingVertical: 14,
    borderRadius: 10,
    alignItems: 'center',
  },
  cancelButtonText: {
    color: '#6b7280',
    fontSize: 16,
    fontWeight: '600',
  },
  submitButton: {
    flex: 2,
    backgroundColor: '#6D28D9',
    paddingVertical: 14,
    borderRadius: 10,
    alignItems: 'center',
  },
  submitButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  bottomPadding: {
    height: 40,
  },
});
