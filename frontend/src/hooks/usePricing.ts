import { pricingService, MarketStats, ExchangeRateData } from '@/src/services/pricing';
import { useChainStore } from '@/src/stores/chain';
import { useCallback, useEffect, useState } from 'react';

export function useCosPrice() {
  const { isConnected } = useChainStore();
  
  const [price, setPrice] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchPrice = useCallback(async () => {
    if (!isConnected) {
      setPrice(null);
      return;
    }

    setLoading(true);
    try {
      const result = await pricingService.getCosPrice();
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

  const priceFormatted = price !== null ? pricingService.formatPriceSimple(price) : '-';
  const priceUsd = price !== null ? price / 1e6 : 0;

  return {
    price,
    priceFormatted,
    priceUsd,
    loading,
    refresh: fetchPrice,
  };
}

export function useMarketStats() {
  const { isConnected } = useChainStore();
  
  const [stats, setStats] = useState<MarketStats | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchStats = useCallback(async () => {
    if (!isConnected) {
      setStats(null);
      return;
    }

    setLoading(true);
    try {
      const result = await pricingService.getMarketStats();
      setStats(result);
    } catch (err) {
      console.error('获取市场统计失败:', err);
    } finally {
      setLoading(false);
    }
  }, [isConnected]);

  useEffect(() => {
    fetchStats();
    const interval = setInterval(fetchStats, 60000);
    return () => clearInterval(interval);
  }, [fetchStats]);

  return {
    stats,
    loading,
    refresh: fetchStats,
  };
}

export function useCnyUsdtRate() {
  const { isConnected } = useChainStore();
  
  const [rate, setRate] = useState<ExchangeRateData | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchRate = useCallback(async () => {
    if (!isConnected) {
      setRate(null);
      return;
    }

    setLoading(true);
    try {
      const result = await pricingService.getCnyUsdtRate();
      setRate(result);
    } catch (err) {
      console.error('获取汇率失败:', err);
    } finally {
      setLoading(false);
    }
  }, [isConnected]);

  useEffect(() => {
    fetchRate();
  }, [fetchRate]);

  const rateFormatted = rate !== null ? `¥${(rate.cnyRate / 1e6).toFixed(4)}` : '-';

  return {
    rate,
    rateFormatted,
    loading,
    refresh: fetchRate,
  };
}
