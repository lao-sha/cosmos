/**
 * 锁定押金页面
 * 路径: /maker/apply/deposit
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  ActivityIndicator,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { useMakerStore } from '@/stores/maker.store';
import { useWalletStore } from '@/stores/wallet.store';
import { MakerService } from '@/services/maker.service';
import { PageHeader } from '@/components/PageHeader';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';

// 押金要求 (USD)
const DEPOSIT_REQUIRED_USD = 1000;

export default function DepositPage() {
  const router = useRouter();
  const { lockDeposit, dustPrice, fetchDustPrice, isSubmitting, txStatus, error, clearError } = useMakerStore();
  const { currentWallet, balance } = useWalletStore();

  const [showTxDialog, setShowTxDialog] = useState(false);

  useEffect(() => {
    fetchDustPrice();
  }, []);

  // 计算需要的 DUST 数量
  const requiredDust = dustPrice > 0 ? DEPOSIT_REQUIRED_USD / dustPrice : 0;
  const requiredDustBigInt = BigInt(Math.ceil(requiredDust * 1e12));

  // 检查余额是否充足
  const balanceBigInt = balance ? BigInt(balance) : BigInt(0);
  const isBalanceSufficient = balanceBigInt >= requiredDustBigInt;

  const handleLockDeposit = async () => {
    if (!isBalanceSufficient) {
      Alert.alert('余额不足', '您的账户余额不足以支付押金');
      return;
    }

    try {
      setShowTxDialog(true);
      await lockDeposit();
      // 成功后跳转到下一步
      setTimeout(() => {
        setShowTxDialog(false);
        router.replace('/maker/apply/info');
      }, 1500);
    } catch (err) {
      // 错误已在 store 中处理
    }
  };

  const handleCloseTxDialog = () => {
    setShowTxDialog(false);
    clearError();
  };

  return (
    <View style={styles.container}>
      <PageHeader title="申请做市商 (1/3)" showBack />

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <Text style={styles.stepTitle}>第一步：锁定押金</Text>

        {/* 押金要求 */}
        <View style={styles.card}>
          <Text style={styles.cardTitle}>押金要求</Text>
          <View style={styles.amountContainer}>
            <Text style={styles.usdAmount}>{DEPOSIT_REQUIRED_USD} USD</Text>
            <Text style={styles.dustAmount}>
              ≈ {requiredDust.toFixed(2)} DUST
            </Text>
            <Text style={styles.priceNote}>
              (按当前价格 {dustPrice.toFixed(4)} USD/DUST)
            </Text>
          </View>
        </View>

        {/* 账户余额 */}
        <View style={styles.card}>
          <Text style={styles.cardTitle}>您的余额</Text>
          <Text style={styles.balanceAmount}>
            {MakerService.formatDustAmount(balanceBigInt)} DUST
          </Text>
          <View style={[styles.balanceStatus, isBalanceSufficient ? styles.statusOk : styles.statusError]}>
            <Text style={[styles.balanceStatusText, isBalanceSufficient ? styles.statusTextOk : styles.statusTextError]}>
              {isBalanceSufficient ? '✅ 余额充足' : '❌ 余额不足'}
            </Text>
          </View>
        </View>

        {/* 押金说明 */}
        <View style={styles.infoCard}>
          <Text style={styles.infoIcon}>💡</Text>
          <Text style={styles.infoTitle}>押金说明</Text>
          <View style={styles.infoList}>
            <Text style={styles.infoItem}>• 押金将被锁定，不可交易</Text>
            <Text style={styles.infoItem}>• 价格波动时可能需要补充</Text>
            <Text style={styles.infoItem}>• 提现需要 7 天冷却期</Text>
            <Text style={styles.infoItem}>• 违规行为将扣除押金</Text>
          </View>
        </View>

        {/* 锁定按钮 */}
        <TouchableOpacity
          style={[styles.submitButton, (!isBalanceSufficient || isSubmitting) && styles.submitButtonDisabled]}
          onPress={handleLockDeposit}
          disabled={!isBalanceSufficient || isSubmitting}
        >
          {isSubmitting ? (
            <ActivityIndicator color="#FFFFFF" />
          ) : (
            <Text style={styles.submitButtonText}>锁定押金</Text>
          )}
        </TouchableOpacity>
      </ScrollView>

      {/* 交易状态弹窗 */}
      <TransactionStatusDialog
        visible={showTxDialog}
        status={txStatus || ''}
        error={error}
        onClose={handleCloseTxDialog}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  content: {
    flex: 1,
    padding: 16,
  },
  stepTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 20,
  },
  card: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  cardTitle: {
    fontSize: 14,
    color: '#8E8E93',
    marginBottom: 12,
  },
  amountContainer: {
    alignItems: 'center',
  },
  usdAmount: {
    fontSize: 28,
    fontWeight: '700',
    color: '#1C1C1E',
    marginBottom: 4,
  },
  dustAmount: {
    fontSize: 16,
    color: '#B2955D',
    fontWeight: '500',
    marginBottom: 4,
  },
  priceNote: {
    fontSize: 12,
    color: '#8E8E93',
  },
  balanceAmount: {
    fontSize: 24,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 12,
  },
  balanceStatus: {
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 8,
    alignSelf: 'flex-start',
  },
  statusOk: {
    backgroundColor: '#4CD96420',
  },
  statusError: {
    backgroundColor: '#FF3B3020',
  },
  balanceStatusText: {
    fontSize: 14,
    fontWeight: '500',
  },
  statusTextOk: {
    color: '#4CD964',
  },
  statusTextError: {
    color: '#FF3B30',
  },
  infoCard: {
    backgroundColor: '#FFF9E6',
    borderRadius: 12,
    padding: 16,
    marginBottom: 24,
  },
  infoIcon: {
    fontSize: 20,
    marginBottom: 8,
  },
  infoTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 12,
  },
  infoList: {
    gap: 6,
  },
  infoItem: {
    fontSize: 14,
    color: '#666666',
    lineHeight: 20,
  },
  submitButton: {
    backgroundColor: '#B2955D',
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    marginBottom: 32,
  },
  submitButtonDisabled: {
    backgroundColor: '#C9C9C9',
  },
  submitButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
});
