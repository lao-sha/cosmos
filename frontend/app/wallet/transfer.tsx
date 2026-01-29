import { TransactionModal } from '@/src/components/TransactionModal';
import { useTransaction } from '@/src/hooks/useTransaction';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
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

export default function TransferScreen() {
  const router = useRouter();
  const { isLoggedIn, address } = useAuthStore();
  const { isConnected } = useChainStore();
  const { status, isLoading, error, transfer, reset } = useTransaction();

  const [recipient, setRecipient] = useState('');
  const [amount, setAmount] = useState('');
  const [modalVisible, setModalVisible] = useState(false);

  const isValidAddress = (addr: string) => {
    return addr.length >= 47 && addr.length <= 48 && addr.startsWith('5');
  };

  const canSubmit =
    isValidAddress(recipient) &&
    parseFloat(amount) > 0 &&
    recipient !== address;

  const handlePreview = () => {
    if (!canSubmit) {
      const msg = recipient === address
        ? '不能转账给自己'
        : !isValidAddress(recipient)
        ? '请输入有效的接收地址'
        : '请输入有效的转账金额';

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

  const handleConfirm = async () => {
    await transfer(recipient, amount);
  };

  const handleClose = () => {
    setModalVisible(false);
    if (status === 'finalized') {
      setRecipient('');
      setAmount('');
      router.back();
    }
    reset();
  };

  const handleMax = () => {
    // TODO: 获取实际余额并设置最大值
    setAmount('0');
  };

  const handleScan = () => {
    // TODO: 实现扫码功能
    const msg = '扫码功能开发中';
    if (Platform.OS === 'web') {
      alert(msg);
    } else {
      Alert.alert('提示', msg);
    }
  };

  if (!isLoggedIn) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>转账</Text>
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
        <Text style={styles.headerTitle}>转账</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView
        style={styles.content}
        contentContainerStyle={styles.contentContainer}
        showsVerticalScrollIndicator={false}
      >
        {!isConnected && (
          <View style={styles.warningBanner}>
            <Text style={styles.warningIcon}>⚠️</Text>
            <Text style={styles.warningText}>
              当前未连接到区块链网络，转账功能暂不可用
            </Text>
          </View>
        )}

        <View style={styles.formCard}>
          <View style={styles.inputGroup}>
            <View style={styles.labelRow}>
              <Text style={styles.label}>接收地址</Text>
              <Pressable onPress={handleScan}>
                <Text style={styles.scanButton}>📷 扫码</Text>
              </Pressable>
            </View>
            <TextInput
              style={styles.input}
              placeholder="输入或粘贴接收方钱包地址"
              placeholderTextColor="#9ca3af"
              value={recipient}
              onChangeText={setRecipient}
              autoCapitalize="none"
              autoCorrect={false}
            />
            {recipient.length > 0 && !isValidAddress(recipient) && (
              <Text style={styles.inputError}>地址格式不正确</Text>
            )}
            {recipient === address && (
              <Text style={styles.inputError}>不能转账给自己</Text>
            )}
          </View>

          <View style={styles.inputGroup}>
            <View style={styles.labelRow}>
              <Text style={styles.label}>转账金额</Text>
              <Pressable onPress={handleMax}>
                <Text style={styles.maxButton}>最大</Text>
              </Pressable>
            </View>
            <View style={styles.amountInputContainer}>
              <TextInput
                style={styles.amountInput}
                placeholder="0.00"
                placeholderTextColor="#9ca3af"
                value={amount}
                onChangeText={setAmount}
                keyboardType="decimal-pad"
              />
              <Text style={styles.amountUnit}>STAR</Text>
            </View>
            <Text style={styles.balanceHint}>可用余额: 0.00 STAR</Text>
          </View>
        </View>

        <View style={styles.feeCard}>
          <Text style={styles.feeTitle}>交易详情</Text>
          <View style={styles.feeRow}>
            <Text style={styles.feeLabel}>转账金额</Text>
            <Text style={styles.feeValue}>{amount || '0'} STAR</Text>
          </View>
          <View style={styles.feeRow}>
            <Text style={styles.feeLabel}>网络手续费</Text>
            <Text style={styles.feeValue}>~0.001 STAR</Text>
          </View>
          <View style={styles.divider} />
          <View style={styles.feeRow}>
            <Text style={styles.totalLabel}>总计</Text>
            <Text style={styles.totalValue}>
              {(parseFloat(amount || '0') + 0.001).toFixed(3)} STAR
            </Text>
          </View>
        </View>

        <View style={styles.infoCard}>
          <Text style={styles.infoTitle}>💡 转账须知</Text>
          <Text style={styles.infoText}>
            • 请仔细核对接收地址，转账不可撤销{'\n'}
            • 确保账户有足够余额支付手续费{'\n'}
            • 交易确认通常需要几秒钟
          </Text>
        </View>

        <Pressable
          style={[
            styles.submitButton,
            (!canSubmit || !isConnected) && styles.submitButtonDisabled,
          ]}
          onPress={handlePreview}
          disabled={!canSubmit || !isConnected}
        >
          <Text style={styles.submitButtonText}>预览转账</Text>
        </Pressable>
      </ScrollView>

      <TransactionModal
        visible={modalVisible}
        status={status}
        isLoading={isLoading}
        title="确认转账"
        description={`向 ${recipient.slice(0, 8)}...${recipient.slice(-6)} 转账 ${amount} STAR`}
        amount={`${amount} STAR`}
        onConfirm={handleConfirm}
        onClose={handleClose}
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
  },
  contentContainer: {
    padding: 16,
  },
  centerContent: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyText: {
    fontSize: 16,
    color: '#6b7280',
  },
  warningBanner: {
    flexDirection: 'row',
    backgroundColor: '#fef3c7',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    alignItems: 'center',
  },
  warningIcon: {
    fontSize: 20,
    marginRight: 12,
  },
  warningText: {
    flex: 1,
    fontSize: 13,
    color: '#92400e',
  },
  formCard: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 20,
    marginBottom: 16,
  },
  inputGroup: {
    marginBottom: 20,
  },
  labelRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  label: {
    fontSize: 14,
    fontWeight: '500',
    color: '#374151',
  },
  scanButton: {
    fontSize: 14,
    color: '#6D28D9',
  },
  maxButton: {
    fontSize: 14,
    color: '#6D28D9',
    fontWeight: '500',
  },
  input: {
    backgroundColor: '#f9fafb',
    borderRadius: 12,
    paddingHorizontal: 16,
    paddingVertical: 14,
    fontSize: 15,
    color: '#1f2937',
    borderWidth: 1,
    borderColor: '#e5e7eb',
  },
  inputError: {
    fontSize: 12,
    color: '#dc2626',
    marginTop: 6,
  },
  amountInputContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#f9fafb',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#e5e7eb',
    paddingRight: 16,
  },
  amountInput: {
    flex: 1,
    paddingHorizontal: 16,
    paddingVertical: 14,
    fontSize: 24,
    fontWeight: '600',
    color: '#1f2937',
  },
  amountUnit: {
    fontSize: 16,
    color: '#6b7280',
    fontWeight: '500',
  },
  balanceHint: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 6,
  },
  feeCard: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 20,
    marginBottom: 16,
  },
  feeTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 16,
  },
  feeRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  feeLabel: {
    fontSize: 14,
    color: '#6b7280',
  },
  feeValue: {
    fontSize: 14,
    color: '#1f2937',
  },
  divider: {
    height: 1,
    backgroundColor: '#e5e7eb',
    marginVertical: 12,
  },
  totalLabel: {
    fontSize: 15,
    fontWeight: '600',
    color: '#1f2937',
  },
  totalValue: {
    fontSize: 15,
    fontWeight: '600',
    color: '#6D28D9',
  },
  infoCard: {
    backgroundColor: '#eff6ff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 24,
  },
  infoTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1e40af',
    marginBottom: 8,
  },
  infoText: {
    fontSize: 13,
    color: '#1e3a8a',
    lineHeight: 20,
  },
  submitButton: {
    backgroundColor: '#6D28D9',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  submitButtonDisabled: {
    backgroundColor: '#d1d5db',
  },
  submitButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
