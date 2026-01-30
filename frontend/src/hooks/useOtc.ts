import { otcService, OtcOrder, MakerInfo, OrderState } from '@/src/services/otc';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useCallback, useEffect, useState } from 'react';

export function useOtcOrder(orderId: number | null) {
  const { isConnected } = useChainStore();
  
  const [order, setOrder] = useState<OtcOrder | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchOrder = useCallback(async () => {
    if (orderId === null || !isConnected) {
      setOrder(null);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const result = await otcService.getOrder(orderId);
      setOrder(result);
    } catch (err: any) {
      setError(err.message || '获取订单失败');
      setOrder(null);
    } finally {
      setLoading(false);
    }
  }, [orderId, isConnected]);

  useEffect(() => {
    fetchOrder();
  }, [fetchOrder]);

  return {
    order,
    loading,
    error,
    refresh: fetchOrder,
  };
}

export function useBuyerOrders() {
  const { address, isLoggedIn } = useAuthStore();
  const { isConnected } = useChainStore();
  
  const [orderIds, setOrderIds] = useState<number[]>([]);
  const [orders, setOrders] = useState<OtcOrder[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchOrders = useCallback(async () => {
    if (!address || !isConnected) {
      setOrderIds([]);
      setOrders([]);
      return;
    }

    setLoading(true);
    try {
      const ids = await otcService.getBuyerOrders(address);
      setOrderIds(ids);
      
      const orderPromises = ids.slice(0, 20).map(id => otcService.getOrder(id));
      const orderResults = await Promise.all(orderPromises);
      setOrders(orderResults.filter((o): o is OtcOrder => o !== null));
    } catch (err) {
      console.error('获取订单失败:', err);
    } finally {
      setLoading(false);
    }
  }, [address, isConnected]);

  useEffect(() => {
    if (isLoggedIn && isConnected) {
      fetchOrders();
    }
  }, [isLoggedIn, isConnected, fetchOrders]);

  return {
    orderIds,
    orders,
    loading,
    refresh: fetchOrders,
  };
}

export function useMakerOrders(makerId: number | null) {
  const { isConnected } = useChainStore();
  
  const [orderIds, setOrderIds] = useState<number[]>([]);
  const [orders, setOrders] = useState<OtcOrder[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchOrders = useCallback(async () => {
    if (makerId === null || !isConnected) {
      setOrderIds([]);
      setOrders([]);
      return;
    }

    setLoading(true);
    try {
      const ids = await otcService.getMakerOrders(makerId);
      setOrderIds(ids);
      
      const orderPromises = ids.slice(0, 20).map(id => otcService.getOrder(id));
      const orderResults = await Promise.all(orderPromises);
      setOrders(orderResults.filter((o): o is OtcOrder => o !== null));
    } catch (err) {
      console.error('获取做市商订单失败:', err);
    } finally {
      setLoading(false);
    }
  }, [makerId, isConnected]);

  useEffect(() => {
    fetchOrders();
  }, [fetchOrders]);

  return {
    orderIds,
    orders,
    loading,
    refresh: fetchOrders,
  };
}

export function useActiveMakers() {
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
      const result = await otcService.getActiveMakers();
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

export function useFirstPurchaseStatus() {
  const { address, isLoggedIn } = useAuthStore();
  const { isConnected } = useChainStore();
  
  const [hasFirstPurchased, setHasFirstPurchased] = useState(false);
  const [loading, setLoading] = useState(false);

  const checkStatus = useCallback(async () => {
    if (!address || !isConnected) {
      setHasFirstPurchased(false);
      return;
    }

    setLoading(true);
    try {
      const result = await otcService.hasFirstPurchased(address);
      setHasFirstPurchased(result);
    } catch (err) {
      console.error('检查首购状态失败:', err);
    } finally {
      setLoading(false);
    }
  }, [address, isConnected]);

  useEffect(() => {
    if (isLoggedIn && isConnected) {
      checkStatus();
    }
  }, [isLoggedIn, isConnected, checkStatus]);

  return {
    hasFirstPurchased,
    canFirstPurchase: !hasFirstPurchased,
    loading,
    refresh: checkStatus,
  };
}

export function useCurrentPrice() {
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
      const result = await otcService.getCurrentPrice();
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
