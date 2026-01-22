// frontend/src/divination/market/hooks/useChainTransaction.ts

import { useState, useCallback } from 'react';
import { Alert } from 'react-native';
import { useWalletStore } from '@/stores/wallet.store';
import {
  getSigner,
  TransactionResult,
  TransactionCallbacks,
  // Provider transactions
  registerProvider,
  updateProviderProfile,
  pauseProvider,
  resumeProvider,
  createPackage,
  updatePackage,
  deletePackage,
  // Order transactions
  createOrder,
  acceptOrder,
  rejectOrder,
  completeOrder,
  cancelOrder,
  requestRefund,
  submitReport,
  submitFollowUp,
  replyFollowUp,
  // Review transactions
  submitReview,
  replyReview,
  // Fund transactions
  withdraw,
  tip,
} from '../services/chain.service';
import type { DivinationType } from '../types';

export type TransactionStatus = 'idle' | 'signing' | 'broadcasting' | 'pending' | 'success' | 'error';

export interface TransactionState {
  status: TransactionStatus;
  txHash?: string;
  blockHash?: string;
  error?: string;
}

/**
 * 链上交易 Hook
 * 提供统一的交易状态管理和错误处理
 */
export function useChainTransaction() {
  const { address, isLocked } = useWalletStore();
  const [txState, setTxState] = useState<TransactionState>({ status: 'idle' });

  /**
   * 重置交易状态
   */
  const resetState = useCallback(() => {
    setTxState({ status: 'idle' });
  }, []);

  /**
   * 请求密码并执行交易
   */
  const executeTransaction = useCallback(
    async <T extends any[]>(
      txFn: (signer: any, ...args: T) => Promise<TransactionResult>,
      args: T,
      options?: {
        onSuccess?: (result: TransactionResult) => void;
        onError?: (error: Error) => void;
        requirePassword?: boolean;
      }
    ): Promise<TransactionResult | null> => {
      if (!address) {
        Alert.alert('错误', '请先连接钱包');
        return null;
      }

      return new Promise((resolve) => {
        // 弹出密码输入框
        Alert.prompt(
          '签名交易',
          '请输入钱包密码以签名交易',
          [
            {
              text: '取消',
              style: 'cancel',
              onPress: () => {
                setTxState({ status: 'idle' });
                resolve(null);
              },
            },
            {
              text: '确认',
              onPress: async (password?: string) => {
                if (!password) {
                  Alert.alert('错误', '请输入密码');
                  resolve(null);
                  return;
                }

                try {
                  setTxState({ status: 'signing' });

                  // 获取签名者
                  const signer = await getSigner(password, address);

                  setTxState({ status: 'broadcasting' });

                  // 执行交易
                  const result = await txFn(signer, ...args);

                  if (result.success) {
                    setTxState({
                      status: 'success',
                      txHash: result.txHash,
                      blockHash: result.blockHash,
                    });
                    options?.onSuccess?.(result);
                  } else {
                    setTxState({
                      status: 'error',
                      error: result.error,
                    });
                    options?.onError?.(new Error(result.error || '交易失败'));
                  }

                  resolve(result);
                } catch (error) {
                  const errorMessage = error instanceof Error ? error.message : '交易失败';
                  setTxState({ status: 'error', error: errorMessage });
                  options?.onError?.(error instanceof Error ? error : new Error(errorMessage));
                  resolve(null);
                }
              },
            },
          ],
          'secure-text'
        );
      });
    },
    [address]
  );

  // ==================== Provider 交易方法 ====================

  const doRegisterProvider = useCallback(
    (
      params: {
        name: string;
        bio: string;
        divinationTypes: DivinationType[];
        specialties: number[];
        avatarCid?: string;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => registerProvider(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doUpdateProviderProfile = useCallback(
    (
      params: {
        name?: string;
        bio?: string;
        avatarCid?: string;
        specialties?: number[];
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => updateProviderProfile(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doCreatePackage = useCallback(
    (
      params: {
        name: string;
        description: string;
        divinationType: DivinationType;
        price: bigint;
        deliveryDays: number;
        maxFollowUps: number;
        isActive?: boolean;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => createPackage(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doUpdatePackage = useCallback(
    (
      params: {
        packageId: number;
        name?: string;
        description?: string;
        price?: bigint;
        deliveryDays?: number;
        maxFollowUps?: number;
        isActive?: boolean;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => updatePackage(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doDeletePackage = useCallback(
    (
      packageId: number,
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => deletePackage(signer, packageId, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doPauseProvider = useCallback(
    (options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }) => {
      return executeTransaction(
        (signer) => pauseProvider(signer, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doResumeProvider = useCallback(
    (options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }) => {
      return executeTransaction(
        (signer) => resumeProvider(signer, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doSubmitReport = useCallback(
    (
      params: {
        provider: string;
        reportType: number;
        evidenceCid: string;
        description: string;
        relatedOrderId?: number;
        isAnonymous?: boolean;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => submitReport(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  // ==================== Order 交易方法 ====================

  const doCreateOrder = useCallback(
    (
      params: {
        providerId: string;
        packageId: number;
        questionCid: string;
        hexagramData?: string;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => createOrder(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doAcceptOrder = useCallback(
    (
      orderId: number,
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => acceptOrder(signer, orderId, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doRejectOrder = useCallback(
    (
      orderId: number,
      reason?: string,
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => rejectOrder(signer, orderId, reason, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doCompleteOrder = useCallback(
    (
      params: {
        orderId: number;
        resultCid: string;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => completeOrder(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doCancelOrder = useCallback(
    (
      orderId: number,
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => cancelOrder(signer, orderId, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doRequestRefund = useCallback(
    (
      orderId: number,
      reason: string,
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => requestRefund(signer, orderId, reason, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doSubmitFollowUp = useCallback(
    (
      params: {
        orderId: number;
        questionCid: string;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => submitFollowUp(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doReplyFollowUp = useCallback(
    (
      params: {
        orderId: number;
        followUpIndex: number;
        answerCid: string;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => replyFollowUp(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  // ==================== Review 交易方法 ====================

  const doSubmitReview = useCallback(
    (
      params: {
        orderId: number;
        ratings: {
          accuracy: number;
          attitude: number;
          speed: number;
          value: number;
        };
        contentCid?: string;
        isAnonymous?: boolean;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => submitReview(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doReplyReview = useCallback(
    (
      params: {
        reviewId: number;
        replyCid: string;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => replyReview(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  // ==================== 资金交易方法 ====================

  const doWithdraw = useCallback(
    (
      amount?: bigint,
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => withdraw(signer, amount, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  const doTip = useCallback(
    (
      params: {
        providerId: string;
        amount: bigint;
        orderId?: number;
      },
      options?: { onSuccess?: (result: TransactionResult) => void; onError?: (error: Error) => void }
    ) => {
      return executeTransaction(
        (signer) => tip(signer, params, createCallbacks()),
        [] as [],
        options
      );
    },
    [executeTransaction]
  );

  // ==================== 辅助函数 ====================

  /**
   * 创建交易回调
   */
  const createCallbacks = (): TransactionCallbacks => ({
    onBroadcast: () => setTxState((s) => ({ ...s, status: 'broadcasting' })),
    onInBlock: (blockHash) => setTxState((s) => ({ ...s, status: 'pending', blockHash })),
    onFinalized: (blockHash) => setTxState((s) => ({ ...s, status: 'success', blockHash })),
    onError: (error) => setTxState({ status: 'error', error: error.message }),
  });

  return {
    // 状态
    txState,
    isProcessing: ['signing', 'broadcasting', 'pending'].includes(txState.status),
    isSuccess: txState.status === 'success',
    isError: txState.status === 'error',

    // 操作
    resetState,

    // Provider 交易
    registerProvider: doRegisterProvider,
    updateProviderProfile: doUpdateProviderProfile,
    pauseProvider: doPauseProvider,
    resumeProvider: doResumeProvider,
    createPackage: doCreatePackage,
    updatePackage: doUpdatePackage,
    deletePackage: doDeletePackage,

    // Order 交易
    createOrder: doCreateOrder,
    acceptOrder: doAcceptOrder,
    rejectOrder: doRejectOrder,
    completeOrder: doCompleteOrder,
    cancelOrder: doCancelOrder,
    requestRefund: doRequestRefund,
    submitReport: doSubmitReport,
    submitFollowUp: doSubmitFollowUp,
    replyFollowUp: doReplyFollowUp,

    // Review 交易
    submitReview: doSubmitReview,
    replyReview: doReplyReview,

    // 资金交易
    withdraw: doWithdraw,
    tip: doTip,
  };
}
