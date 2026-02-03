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

  const createBaziChart = useCallback(async (params: {
    name?: string;
    input: 
      | { Solar: { year: number; month: number; day: number; hour: number; minute: number } }
      | { Lunar: { year: number; month: number; day: number; hour: number; minute: number; is_leap_month: boolean } };
    gender: 'Male' | 'Female';
    zishi_mode: 'Modern' | 'Traditional';
    longitude?: number;
  }) => {
    if (!checkReady() || !mnemonic) return null;

    setIsLoading(true);
    setError(null);
    setStatus('pending');

    const txResult = await transactionService.createBaziChart(
      mnemonic,
      params,
      {
        onStatusChange: setStatus,
        onSuccess: () => {
          showMessage('成功', '八字命盘已保存到链上');
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

  const deleteBaziChart = useCallback(async (chartId: number) => {
    if (!checkReady() || !mnemonic) return { success: false, error: '请先登录' } as TxResult;

    setIsLoading(true);
    setError(null);
    setStatus('pending');

    const txResult = await transactionService.deleteBaziChart(
      mnemonic,
      chartId,
      {
        onStatusChange: setStatus,
        onSuccess: () => {
          // 不在这里显示消息，由调用方处理
        },
        onError: (err) => {
          // 不在这里显示消息，由调用方处理
        },
      }
    );

    setResult(txResult);
    setIsLoading(false);
    if (!txResult.success) {
      setError(txResult.error || 'Unknown error');
    }
    return txResult;
  }, [mnemonic, checkReady]);

  const createQimenChart = useCallback(async (params: {
    solar_year: number;
    solar_month: number;
    solar_day: number;
    hour: number;
    question_hash: number[];
    is_public: boolean;
    name?: string;
    gender?: number;
    birth_year?: number;
    question?: string;
    question_type?: number;
    pan_method: number;
  }) => {
    if (!checkReady() || !mnemonic) return null;

    setIsLoading(true);
    setError(null);
    setStatus('pending');

    const txResult = await transactionService.createQimenChart(
      mnemonic,
      params,
      {
        onStatusChange: setStatus,
        onSuccess: () => {
          showMessage('成功', '奇门排盘已保存到链上');
        },
        onError: (err) => {
          showMessage('起局失败', err);
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

  // ==================== 群聊相关 ====================

  const createGroup = useCallback(async (params: {
    name: string;
    description?: string;
    encryption_mode: number;
    is_public: boolean;
  }) => {
    if (!checkReady() || !mnemonic) return null;

    setIsLoading(true);
    setError(null);
    setStatus('pending');

    const txResult = await transactionService.createGroup(
      mnemonic,
      params,
      {
        onStatusChange: setStatus,
        onSuccess: () => {
          showMessage('成功', '群聊创建成功');
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

  const sendGroupMessage = useCallback(async (
    groupId: number,
    content: string,
    messageType: number = 0
  ) => {
    if (!checkReady() || !mnemonic) return null;

    setIsLoading(true);
    setError(null);
    setStatus('pending');

    const txResult = await transactionService.sendGroupMessage(
      mnemonic,
      groupId,
      content,
      messageType,
      {
        onStatusChange: setStatus,
        onSuccess: () => {},
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

  const joinGroup = useCallback(async (groupId: number) => {
    if (!checkReady() || !mnemonic) return null;

    setIsLoading(true);
    setError(null);
    setStatus('pending');

    const txResult = await transactionService.joinGroup(mnemonic, groupId, {
      onStatusChange: setStatus,
      onSuccess: () => {
        showMessage('成功', '已加入群聊');
      },
      onError: (err) => {
        showMessage('加入失败', err);
      },
    });

    setResult(txResult);
    setIsLoading(false);
    if (!txResult.success) {
      setError(txResult.error || 'Unknown error');
    }
    return txResult;
  }, [mnemonic, checkReady, showMessage]);

  const leaveGroup = useCallback(async (groupId: number) => {
    if (!checkReady() || !mnemonic) return null;

    setIsLoading(true);
    setError(null);
    setStatus('pending');

    const txResult = await transactionService.leaveGroup(mnemonic, groupId, {
      onStatusChange: setStatus,
      onSuccess: () => {
        showMessage('成功', '已退出群聊');
      },
      onError: (err) => {
        showMessage('退出失败', err);
      },
    });

    setResult(txResult);
    setIsLoading(false);
    if (!txResult.success) {
      setError(txResult.error || 'Unknown error');
    }
    return txResult;
  }, [mnemonic, checkReady, showMessage]);

  const disbandGroup = useCallback(async (groupId: number) => {
    if (!checkReady() || !mnemonic) return null;

    setIsLoading(true);
    setError(null);
    setStatus('pending');

    const txResult = await transactionService.disbandGroup(mnemonic, groupId, {
      onStatusChange: setStatus,
      onSuccess: () => {
        showMessage('成功', '群聊已解散');
      },
      onError: (err) => {
        showMessage('解散失败', err);
      },
    });

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

  const lockMakerDeposit = useCallback(
    async (mnemonic: string, callbacks?: TxCallbacks) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.lockMakerDeposit(mnemonic, {
        ...callbacks,
        onStatusChange: (s) => {
          setStatus(s);
          callbacks?.onStatusChange?.(s);
        },
      });
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const submitMakerInfo = useCallback(
    async (
      mnemonic: string,
      realName: string,
      idCardNumber: string,
      birthday: string,
      tronAddress: string,
      wechatId: string,
      callbacks?: TxCallbacks
    ) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.submitMakerInfo(
        mnemonic,
        realName,
        idCardNumber,
        birthday,
        tronAddress,
        wechatId,
        {
          ...callbacks,
          onStatusChange: (s) => {
            setStatus(s);
            callbacks?.onStatusChange?.(s);
          },
        }
      );
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const cancelMaker = useCallback(
    async (mnemonic: string, callbacks?: TxCallbacks) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.cancelMaker(mnemonic, {
        ...callbacks,
        onStatusChange: (s) => {
          setStatus(s);
          callbacks?.onStatusChange?.(s);
        },
      });
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const requestMakerWithdrawal = useCallback(
    async (mnemonic: string, amount: string, callbacks?: TxCallbacks) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.requestMakerWithdrawal(mnemonic, amount, {
        ...callbacks,
        onStatusChange: (s) => {
          setStatus(s);
          callbacks?.onStatusChange?.(s);
        },
      });
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const executeMakerWithdrawal = useCallback(
    async (mnemonic: string, callbacks?: TxCallbacks) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.executeMakerWithdrawal(mnemonic, {
        ...callbacks,
        onStatusChange: (s) => {
          setStatus(s);
          callbacks?.onStatusChange?.(s);
        },
      });
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const cancelMakerWithdrawal = useCallback(
    async (mnemonic: string, callbacks?: TxCallbacks) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.cancelMakerWithdrawal(mnemonic, {
        ...callbacks,
        onStatusChange: (s) => {
          setStatus(s);
          callbacks?.onStatusChange?.(s);
        },
      });
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const replenishMakerDeposit = useCallback(
    async (mnemonic: string, callbacks?: TxCallbacks) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.replenishMakerDeposit(mnemonic, {
        ...callbacks,
        onStatusChange: (s) => {
          setStatus(s);
          callbacks?.onStatusChange?.(s);
        },
      });
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const appealMakerPenalty = useCallback(
    async (mnemonic: string, penaltyId: number, evidenceCid: string, callbacks?: TxCallbacks) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.appealMakerPenalty(mnemonic, penaltyId, evidenceCid, {
        ...callbacks,
        onStatusChange: (s) => {
          setStatus(s);
          callbacks?.onStatusChange?.(s);
        },
      });
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const createOtcOrderNew = useCallback(
    async (
      mnemonic: string,
      makerId: number,
      cosAmount: string,
      paymentCommit: string,
      contactCommit: string,
      callbacks?: TxCallbacks
    ) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.createOtcOrderNew(
        mnemonic,
        makerId,
        cosAmount,
        paymentCommit,
        contactCommit,
        {
          ...callbacks,
          onStatusChange: (s) => {
            setStatus(s);
            callbacks?.onStatusChange?.(s);
          },
        }
      );
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const createFirstPurchase = useCallback(
    async (
      mnemonic: string,
      makerId: number,
      paymentCommit: string,
      contactCommit: string,
      callbacks?: TxCallbacks
    ) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.createFirstPurchase(
        mnemonic,
        makerId,
        paymentCommit,
        contactCommit,
        {
          ...callbacks,
          onStatusChange: (s) => {
            setStatus(s);
            callbacks?.onStatusChange?.(s);
          },
        }
      );
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const markOtcPaid = useCallback(
    async (mnemonic: string, orderId: number, tronTxHash?: string, callbacks?: TxCallbacks) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.markOtcPaid(mnemonic, orderId, tronTxHash, {
        ...callbacks,
        onStatusChange: (s) => {
          setStatus(s);
          callbacks?.onStatusChange?.(s);
        },
      });
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const releaseOtcCos = useCallback(
    async (mnemonic: string, orderId: number, callbacks?: TxCallbacks) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.releaseOtcCos(mnemonic, orderId, {
        ...callbacks,
        onStatusChange: (s) => {
          setStatus(s);
          callbacks?.onStatusChange?.(s);
        },
      });
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const cancelOtcOrder = useCallback(
    async (mnemonic: string, orderId: number, callbacks?: TxCallbacks) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.cancelOtcOrder(mnemonic, orderId, {
        ...callbacks,
        onStatusChange: (s) => {
          setStatus(s);
          callbacks?.onStatusChange?.(s);
        },
      });
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const disputeOtcOrder = useCallback(
    async (mnemonic: string, orderId: number, callbacks?: TxCallbacks) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.disputeOtcOrder(mnemonic, orderId, {
        ...callbacks,
        onStatusChange: (s) => {
          setStatus(s);
          callbacks?.onStatusChange?.(s);
        },
      });
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const initiateOtcDispute = useCallback(
    async (mnemonic: string, orderId: number, evidenceCid: string, callbacks?: TxCallbacks) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.initiateOtcDispute(mnemonic, orderId, evidenceCid, {
        ...callbacks,
        onStatusChange: (s) => {
          setStatus(s);
          callbacks?.onStatusChange?.(s);
        },
      });
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const respondOtcDispute = useCallback(
    async (mnemonic: string, orderId: number, evidenceCid: string, callbacks?: TxCallbacks) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.respondOtcDispute(mnemonic, orderId, evidenceCid, {
        ...callbacks,
        onStatusChange: (s) => {
          setStatus(s);
          callbacks?.onStatusChange?.(s);
        },
      });
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const createSwap = useCallback(
    async (
      mnemonic: string,
      makerId: number,
      cosAmount: string,
      usdtAddress: string,
      callbacks?: TxCallbacks
    ) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.createSwap(
        mnemonic,
        makerId,
        cosAmount,
        usdtAddress,
        {
          ...callbacks,
          onStatusChange: (s) => {
            setStatus(s);
            callbacks?.onStatusChange?.(s);
          },
        }
      );
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const markSwapComplete = useCallback(
    async (mnemonic: string, swapId: number, trc20TxHash: string, callbacks?: TxCallbacks) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.markSwapComplete(mnemonic, swapId, trc20TxHash, {
        ...callbacks,
        onStatusChange: (s) => {
          setStatus(s);
          callbacks?.onStatusChange?.(s);
        },
      });
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const reportSwap = useCallback(
    async (mnemonic: string, swapId: number, callbacks?: TxCallbacks) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.reportSwap(mnemonic, swapId, {
        ...callbacks,
        onStatusChange: (s) => {
          setStatus(s);
          callbacks?.onStatusChange?.(s);
        },
      });
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  const handleSwapVerificationTimeout = useCallback(
    async (mnemonic: string, swapId: number, callbacks?: TxCallbacks) => {
      setIsLoading(true);
      setError(null);
      const result = await transactionService.handleSwapVerificationTimeout(mnemonic, swapId, {
        ...callbacks,
        onStatusChange: (s) => {
          setStatus(s);
          callbacks?.onStatusChange?.(s);
        },
      });
      setResult(result);
      setIsLoading(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
      return result;
    },
    []
  );

  return {
    status,
    isLoading,
    isTxLoading: isLoading,
    error,
    result,
    transfer,
    sendChatMessage,
    createOtcOrder,
    lockOtcOrder,
    createMatchmakingProfile,
    createDivinationOrder,
    createBaziChart,
    deleteBaziChart,
    createQimenChart,
    createGroup,
    sendGroupMessage,
    joinGroup,
    leaveGroup,
    disbandGroup,
    lockMakerDeposit,
    submitMakerInfo,
    cancelMaker,
    requestMakerWithdrawal,
    executeMakerWithdrawal,
    cancelMakerWithdrawal,
    replenishMakerDeposit,
    appealMakerPenalty,
    createOtcOrderNew,
    createFirstPurchase,
    markOtcPaid,
    releaseOtcCos,
    cancelOtcOrder,
    disputeOtcOrder,
    initiateOtcDispute,
    respondOtcDispute,
    createSwap,
    markSwapComplete,
    reportSwap,
    handleSwapVerificationTimeout,
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
