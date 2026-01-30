import { referralService, ReferralInfo } from '@/src/services/referral';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useCallback, useEffect, useState } from 'react';

export function useReferral() {
  const { address, isLoggedIn } = useAuthStore();
  const { isConnected } = useChainStore();
  
  const [referralInfo, setReferralInfo] = useState<ReferralInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchReferralInfo = useCallback(async () => {
    if (!address || !isConnected) {
      setReferralInfo(null);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const info = await referralService.getReferralInfo(address);
      setReferralInfo(info);
    } catch (err: any) {
      setError(err.message || '获取推荐信息失败');
      setReferralInfo(null);
    } finally {
      setLoading(false);
    }
  }, [address, isConnected]);

  useEffect(() => {
    if (isLoggedIn && isConnected) {
      fetchReferralInfo();
    }
  }, [isLoggedIn, isConnected, fetchReferralInfo]);

  return {
    referralInfo,
    loading,
    error,
    refresh: fetchReferralInfo,
  };
}

export function useDownlines(address?: string) {
  const { isConnected } = useChainStore();
  
  const [downlines, setDownlines] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchDownlines = useCallback(async () => {
    if (!address || !isConnected) {
      setDownlines([]);
      return;
    }

    setLoading(true);
    try {
      const result = await referralService.getDownlines(address);
      setDownlines(result);
    } catch (err) {
      console.error('获取下线失败:', err);
      setDownlines([]);
    } finally {
      setLoading(false);
    }
  }, [address, isConnected]);

  useEffect(() => {
    fetchDownlines();
  }, [fetchDownlines]);

  return { downlines, loading, refresh: fetchDownlines };
}
