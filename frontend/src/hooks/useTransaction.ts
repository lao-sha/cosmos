import { transactionService, TxResult, TxStatus } from '@/src/services/transaction';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useCallback, useState } from 'react';
import { Alert, Platform } from 'react-native';

export type { TxResult, TxStatus };

export function useTransaction() {
  const { mnemonic, isLoggedIn } = useAuthStore();
  const { isConnected } = useChainStore();
  const [status, setStatus] = useState<TxStatus | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<TxResult | null>(null);

  const showMessage = useCallback((title: string, message: string) => {
    if (Platform.OS === 'web') {
      window.alert(`${title}\n${message}`);
    } else {
      Alert.alert(title, message);
    }
  }, []);

  const checkReady = useCallback(() => {
    if (!isLoggedIn || !mnemonic) {
      showMessage('错误', '请先登录钱包');
      return false;
    }
    if (!isConnected) {
      showMessage('错误', '链未连接');
      return false;
    }
    return true;
  }, [isLoggedIn, mnemonic, isConnected, showMessage]);

  const transfer = useCallback(async (to: string, amount: string) => {
    if (!checkReady() || !mnemonic) return null;

    setIsLoading(true);
    setError(null);
    setStatus('pending');

    const txResult = await transactionService.transfer(mnemonic, to, amount, {
      onStatusChange: setStatus,
      onSuccess: (res) => {
        showMessage('成功', `转账成功！\n区块: ${res.blockHash?.slice(0, 10)}...`);
      },
      onError: (err) => {
        showMessage('失败', err);
      },
    });

    setResult(txResult);
    setIsLoading(false);
    if (!txResult.success) {
      setError(txResult.error || 'Unknown error');
    }
    return txResult;
  }, [mnemonic, checkReady, showMessage]);

  const sendChatMessage = useCallback(async (
    receiver: string,
    contentCid: string,
    msgType?: number,
    sessionId?: string
  ) => {
    if (!checkReady() || !mnemonic) return null;

    setIsLoading(true);
    setError(null);
    setStatus('pending');

    const txResult = await transactionService.sendChatMessage(
      mnemonic,
      receiver,
      contentCid,
      msgType || 0,
      sessionId,
      {
        onStatusChange: setStatus,
        onSuccess: () => {
          showMessage('成功', '消息已发送到链上');
        },
        onError: (err) => {
          showMessage('发送失败', err);
        },
      }
    );

    setResult(txResult);
    setIsLoading(false);
    if (!txResult.success) {
      setError(txResult.error || 'Unknown error');
    }
    return txResult;
  }, [mnemonic, checkReady, showMessage]);

  const createOtcOrder = useCallback(async (
    orderType: 'Buy' | 'Sell',
    amount: string,
    price: string,
    currency: string = 'CNY'
  ) => {
    if (!checkReady() || !mnemonic) return null;

    setIsLoading(true);
    setError(null);
    setStatus('pending');

    const txResult = await transactionService.createOtcOrder(
      mnemonic,
      orderType,
      amount,
      price,
      currency,
      {
        onStatusChange: setStatus,
        onSuccess: () => {
          showMessage('成功', 'OTC 订单已创建');
        },
        onError: (err) => {
          showMessage('创建失败', err);
        },
      }
    );

    setResult(txResult);
    setIsLoading(false);
    if (!txResult.success) {
      setError(txResult.error || 'Unknown error');
    }
    return txResult;
  }, [mnemonic, checkReady, showMessage]);

  const lockOtcOrder = useCallback(async (orderId: string) => {
    if (!checkReady() || !mnemonic) return null;

    setIsLoading(true);
    setError(null);
    setStatus('pending');

    const txResult = await transactionService.lockOtcOrder(mnemonic, orderId, {
      onStatusChange: setStatus,
      onSuccess: () => {
        showMessage('成功', '订单已锁定，请按提示完成交易');
      },
      onError: (err) => {
        showMessage('锁定失败', err);
      },
    });

    setResult(txResult);
    setIsLoading(false);
    if (!txResult.success) {
      setError(txResult.error || 'Unknown error');
    }
    return txResult;
  }, [mnemonic, checkReady, showMessage]);

  const createMatchmakingProfile = useCallback(async (
    nickname: string,
    gender: 'Male' | 'Female',
    birthInfo?: { year: number; month: number; day: number; hour: number }
  ) => {
    if (!checkReady() || !mnemonic) return null;

    setIsLoading(true);
    setError(null);
    setStatus('pending');

    const txResult = await transactionService.createMatchmakingProfile(
      mnemonic,
      nickname,
      gender,
      birthInfo,
      {
        onStatusChange: setStatus,
        onSuccess: () => {
          showMessage('成功', '资料已保存到链上');
        },
        onError: (err) => {
          showMessage('保存失败', err);
        },
      }
    );

    setResult(txResult);
    setIsLoading(false);
    if (!txResult.success) {
      setError(txResult.error || 'Unknown error');
    }
    return txResult;
  }, [mnemonic, checkReady, showMessage]);

  const createDivinationOrder = useCallback(async (
    providerId: string,
    packageId: string,
    questionCid: string
  ) => {
    if (!checkReady() || !mnemonic) return null;

    setIsLoading(true);
    setError(null);
    setStatus('pending');

    const txResult = await transactionService.createDivinationOrder(
      mnemonic,
      providerId,
      packageId,
      questionCid,
      {
        onStatusChange: setStatus,
        onSuccess: () => {
          showMessage('成功', '占卜订单已创建');
        },
        onError: (err) => {
          showMessage('创建失败', err);
        },
      }
    );

    setResult(txResult);
    setIsLoading(false);
    if (!txResult.success) {
      setError(txResult.error || 'Unknown error');
    }
    return txResult;
  }, [mnemonic, checkReady, showMessage]);

  const reset = useCallback(() => {
    setStatus(null);
    setIsLoading(false);
    setError(null);
    setResult(null);
  }, []);

  return {
    status,
    isLoading,
    error,
    result,
    transfer,
    sendChatMessage,
    createOtcOrder,
    lockOtcOrder,
    createMatchmakingProfile,
    createDivinationOrder,
    reset,
  };
}

export const TX_STATUS_TEXT: Record<TxStatus, string> = {
  pending: '准备中...',
  signing: '签名中...',
  broadcasting: '广播中...',
  inBlock: '已入块...',
  finalized: '已确认',
  failed: '失败',
};
