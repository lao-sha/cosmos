import { makerService, MakerInfo, ApplicationStatus } from '@/src/services/maker';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useCallback, useEffect, useState } from 'react';

export function useMaker() {
  const { address, isLoggedIn } = useAuthStore();
  const { isConnected } = useChainStore();
  
  const [makerInfo, setMakerInfo] = useState<MakerInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchMakerInfo = useCallback(async () => {
    if (!address || !isConnected) {
      setMakerInfo(null);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const info = await makerService.getMakerInfo(address);
      setMakerInfo(info);
    } catch (err: any) {
      setError(err.message || '获取做市商信息失败');
      setMakerInfo(null);
    } finally {
      setLoading(false);
    }
  }, [address, isConnected]);

  useEffect(() => {
    if (isLoggedIn && isConnected) {
      fetchMakerInfo();
    }
  }, [isLoggedIn, isConnected, fetchMakerInfo]);

  const isMaker = makerInfo?.makerId !== null;
  const isActiveMaker = makerInfo?.application?.status === ApplicationStatus.Active;
  const isPendingReview = makerInfo?.application?.status === ApplicationStatus.PendingReview;
  const isDepositLocked = makerInfo?.application?.status === ApplicationStatus.DepositLocked;
  const canApply = !isMaker;
  const canSubmitInfo = isDepositLocked;
  const canCancel = isDepositLocked || isPendingReview;

  return {
    makerInfo,
    loading,
    error,
    refresh: fetchMakerInfo,
    isMaker,
    isActiveMaker,
    isPendingReview,
    isDepositLocked,
    canApply,
    canSubmitInfo,
    canCancel,
  };
}

export function useMakerConstants() {
  const { isConnected } = useChainStore();
  
  const [depositAmount, setDepositAmount] = useState<string>('0');
  const [withdrawalCooldown, setWithdrawalCooldown] = useState<number>(604800);
  const [loading, setLoading] = useState(false);

  const fetchConstants = useCallback(async () => {
    if (!isConnected) return;

    setLoading(true);
    try {
      const [deposit, cooldown] = await Promise.all([
        makerService.getDepositAmount(),
        makerService.getWithdrawalCooldown(),
      ]);
      setDepositAmount(deposit);
      setWithdrawalCooldown(cooldown);
    } catch (err) {
      console.error('获取做市商常量失败:', err);
    } finally {
      setLoading(false);
    }
  }, [isConnected]);

  useEffect(() => {
    fetchConstants();
  }, [fetchConstants]);

  const depositAmountFormatted = makerService.formatDeposit(depositAmount);
  const cooldownDays = Math.ceil(withdrawalCooldown / 86400);

  return {
    depositAmount,
    depositAmountFormatted,
    withdrawalCooldown,
    cooldownDays,
    loading,
  };
}
