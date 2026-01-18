/**
 * 提现进度页面
 * 路径: /maker/deposit/withdraw/status
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
import { useMakerStore, selectCanExecuteWithdrawal } from '@/stores/maker.store';
import { WithdrawalProgress } from '@/features/maker/components';
import { PageHeader } from '@/components/PageHeader';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';

export default function WithdrawStatusPage() {
  const router = useRouter();
  const {
    withdrawalRequest,
    executeWithdrawal,
    cancelWithdrawal,
    isSubmitting,
    txStatus,
    error,
    clearError,
    fetchWithdrawalRequest,
  } = useMakerStore();

  const canExecute = useMakerStore(selectCanExecuteWithdrawal);

  const [showTxDialog, setShowTxDialog] = useState(false);

  useEffect(() => {
    fetchWithdrawalRequest();
  }, []);

  const handleExecute = async () => {
    Alert.alert(
      '执行提现',
      '确定要执行提现吗？押金将转入您的账户。',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确定执行',
          onPress: async () => {
            try {
              setShowTxDialog(true);
              await executeWithdrawal();
              setTimeout(() => {
                setShowTxDialog(false);
                router.replace('/maker/deposit');
              }, 1500);
            } catch (err) {
              // 错误已在 store 中处理
            }
          },
        },
      ]
    );
  };

  const handleCancel = async () => {
    Alert.alert(
      '取消提现',
      '确定要取消提现申请吗？',
      [
        { text: '再想想', style: 'cancel' },
        {
          text: '确定取消',
          style: 'destructive',
          onPress: async () => {
            try {
              setShowTxDialog(true);
              await cancelWithdrawal();
              setTimeout(() => {
                setShowTxDialog(false);
                router.replace('/maker/deposit');
              }, 1500);
            } catch (err) {
              // 错误已在 store 中处理
            }
          },
        },
      ]
    );
  };

  const handleCloseTxDialog = () => {
    setShowTxDialog(false);
    clearError();
  };

  if (!withdrawalRequest) {
    return (
      <View style={styles.container}>
        <PageHeader title="提现进度" showBack />
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyText}>暂无提现申请</Text>
          <TouchableOpacity
            style={styles.applyButton}
            onPress={() => router.replace('/maker/deposit/withdraw')}
          >
            <Text style={styles.applyButtonText}>申请提现</Text>
          </TouchableOpacity>
        </View>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <PageHeader title="提现进度" showBack />

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 提现进度组件 */}
        <WithdrawalProgress request={withdrawalRequest} />

        {/* 操作按钮 */}
        <View style={styles.actions}>
          {canExecute ? (
            <TouchableOpacity
              style={styles.executeButton}
              onPress={handleExecute}
              disabled={isSubmitting}
            >
              {isSubmitting ? (
                <ActivityIndicator color="#FFFFFF" />
              ) : (
                <Text style={styles.executeButtonText}>执行提现</Text>
              )}
            </TouchableOpacity>
          ) : null}

          <TouchableOpacity
            style={styles.cancelButton}
            onPress={handleCancel}
            disabled={isSubmitting}
          >
            {isSubmitting ? (
              <ActivityIndicator color="#FF3B30" />
            ) : (
              <Text style={styles.cancelButtonText}>取消提现</Text>
            )}
          </TouchableOpacity>
        </View>
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
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 24,
  },
  emptyText: {
    fontSize: 16,
    color: '#8E8E93',
    marginBottom: 16,
  },
  applyButton: {
    backgroundColor: '#B2955D',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  applyButtonText: {
    fontSize: 15,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  content: {
    flex: 1,
    padding: 16,
  },
  actions: {
    marginTop: 24,
    gap: 12,
  },
  executeButton: {
    backgroundColor: '#4CD964',
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
  },
  executeButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  cancelButton: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#FF3B30',
  },
  cancelButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FF3B30',
  },
});
