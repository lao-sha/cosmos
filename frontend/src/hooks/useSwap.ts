import { swapService, SwapRecord, MakerInfo, SwapStatus } from '@/src/services/swap';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useCallback, useEffect, useState } from 'react';

export function useSwapRecord(swapId: number | null) {
  const { isConnected } = useChainStore();
  
  const [swap, setSwap] = useState<SwapRecord | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchSwap = useCallback(async () => {
    if (swapId === null || !isConnected) {
      setSwap(null);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const result = await swapService.getSwap(swapId);
      setSwap(result);
    } catch (err: any) {
      setError(err.message || '获取兑换记录失败');
      setSwap(null);
    } finally {
      setLoading(false);
    }
  }, [swapId, isConnected]);

  useEffect(() => {
    fetchSwap();
  }, [fetchSwap]);

  return {
    swap,
    loading,
    error,
    refresh: fetchSwap,
  };
}

export function useUserSwaps() {
  const { address, isLoggedIn } = useAuthStore();
  const { isConnected } = useChainStore();
  
  const [swapIds, setSwapIds] = useState<number[]>([]);
  const [swaps, setSwaps] = useState<SwapRecord[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchSwaps = useCallback(async () => {
    if (!address || !isConnected) {
      setSwapIds([]);
      setSwaps([]);
      return;
    }

    setLoading(true);
    try {
      const ids = await swapService.getUserSwaps(address);
      setSwapIds(ids);
      
      const swapPromises = ids.slice(0, 20).map(id => swapService.getSwap(id));
      const swapResults = await Promise.all(swapPromises);
      setSwaps(swapResults.filter((s): s is SwapRecord => s !== null));
    } catch (err) {
      console.error('获取兑换列表失败:', err);
    } finally {
      setLoading(false);
    }
  }, [address, isConnected]);

  useEffect(() => {
    if (isLoggedIn && isConnected) {
      fetchSwaps();
    }
  }, [isLoggedIn, isConnected, fetchSwaps]);

  return {
    swapIds,
    swaps,
    loading,
    refresh: fetchSwaps,
  };
}

export function useMakerSwaps(makerId: number | null) {
  const { isConnected } = useChainStore();
  
  const [swapIds, setSwapIds] = useState<number[]>([]);
  const [swaps, setSwaps] = useState<SwapRecord[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchSwaps = useCallback(async () => {
    if (makerId === null || !isConnected) {
      setSwapIds([]);
      setSwaps([]);
      return;
    }

    setLoading(true);
    try {
      const ids = await swapService.getMakerSwapList(makerId);
      setSwapIds(ids);
      
      const swapPromises = ids.slice(0, 20).map(id => swapService.getSwap(id));
      const swapResults = await Promise.all(swapPromises);
      setSwaps(swapResults.filter((s): s is SwapRecord => s !== null));
    } catch (err) {
      console.error('获取做市商兑换列表失败:', err);
    } finally {
      setLoading(false);
    }
  }, [makerId, isConnected]);

  useEffect(() => {
    fetchSwaps();
  }, [fetchSwaps]);

  return {
    swapIds,
    swaps,
    loading,
    refresh: fetchSwaps,
  };
}

export function useSwapMakers() {
  const { isConnected } = useChainStore();
  
  const [makers, setMakers] = useState<MakerInfo[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchMakers = useCallback(async () => {
    if (!isConnected) {
      setMakers([]);
      return;
    }

    setLoading(true);
    try {
      const result = await swapService.getActiveMakers();
      setMakers(result);
    } catch (err) {
      console.error('获取做市商列表失败:', err);
    } finally {
      setLoading(false);
    }
  }, [isConnected]);

  useEffect(() => {
    fetchMakers();
  }, [fetchMakers]);

  return {
    makers,
    loading,
    refresh: fetchMakers,
  };
}

export function useSwapPrice() {
  const { isConnected } = useChainStore();
  
  const [price, setPrice] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchPrice = useCallback(async () => {
    if (!isConnected) {
      setPrice(null);
      return;
    }

    setLoading(true);
    try {
      const result = await swapService.getCurrentPrice();
      setPrice(result);
    } catch (err) {
      console.error('获取价格失败:', err);
    } finally {
      setLoading(false);
    }
  }, [isConnected]);

  useEffect(() => {
    fetchPrice();
    const interval = setInterval(fetchPrice, 30000);
    return () => clearInterval(interval);
  }, [fetchPrice]);

  const priceFormatted = price ? `$${(Number(price) / 1e6).toFixed(4)}` : '-';

  return {
    price,
    priceFormatted,
    loading,
    refresh: fetchPrice,
  };
}
