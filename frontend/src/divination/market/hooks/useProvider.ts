// frontend/src/divination/market/hooks/useProvider.ts

import { useState, useCallback, useEffect } from 'react';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { useWalletStore } from '@/stores/wallet.store';
import {
  Provider,
  ServicePackage,
  Order,
  ProviderDashboard,
  WithdrawalRequest,
} from '../types/market.types';

// API 端点
const WS_ENDPOINT = process.env.EXPO_PUBLIC_WS_ENDPOINT || 'ws://127.0.0.1:9944';

let apiInstance: ApiPromise | null = null;

async function getApi(): Promise<ApiPromise> {
  if (apiInstance && apiInstance.isConnected) {
    return apiInstance;
  }

  const provider = new WsProvider(WS_ENDPOINT);
  apiInstance = await ApiPromise.create({ provider });
  return apiInstance;
}

export function useProvider() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isProvider, setIsProvider] = useState(false);
  const [providerInfo, setProviderInfo] = useState<Provider | null>(null);
  const { address } = useWalletStore();

  /**
   * 检查当前用户是否是提供者
   */
  const checkIsProvider = useCallback(async (): Promise<boolean> => {
    if (!address) return false;

    try {
      const api = await getApi();

      // @ts-ignore
      const result = await api.query.divinationMarket.providers(address);
      const isRegistered = !result.isNone;

      setIsProvider(isRegistered);

      if (isRegistered) {
        const data = result.toJSON() as any;
        setProviderInfo({
          account: address,
          name: data.name || '',
          bio: data.bio || '',
          avatarCid: data.avatarCid,
          specialties: data.specialties || 0,
          supportedTypes: data.supportedTypes || 0,
          tier: data.tier || 0,
          status: data.status || 'Pending',
          totalRating: data.totalRating || 0,
          ratingCount: data.ratingCount || 0,
          completedOrders: data.completedOrders || 0,
          acceptsUrgent: data.acceptsUrgent || false,
          registeredAt: data.registeredAt || 0,
        });
      }

      return isRegistered;
    } catch (err) {
      console.error('Check provider error:', err);
      return false;
    }
  }, [address]);

  /**
   * 获取提供者余额
   */
  const getProviderBalance = useCallback(async (): Promise<bigint> => {
    if (!address) return 0n;

    try {
      const api = await getApi();

      // @ts-ignore
      const result = await api.query.divinationMarket.providerBalances(address);
      return BigInt(result.toString() || 0);
    } catch (err) {
      console.error('Get balance error:', err);
      return 0n;
    }
  }, [address]);

  /**
   * 获取提供者工作台数据
   */
  const getDashboard = useCallback(async (): Promise<ProviderDashboard | null> => {
    if (!address || !isProvider || !providerInfo) return null;

    try {
      setLoading(true);
      setError(null);

      const api = await getApi();

      // 获取余额
      const balance = await getProviderBalance();

      // 获取订单
      // @ts-ignore
      const orderEntries = await api.query.divinationMarket.orders.entries();

      const pendingOrders: Order[] = [];
      const activeOrders: Order[] = [];
      let todayNewOrders = 0;
      let todayCompleted = 0;
      let todayEarnings = 0n;

      const todayStart = new Date();
      todayStart.setHours(0, 0, 0, 0);
      const todayTimestamp = todayStart.getTime();

      for (const [key, value] of orderEntries) {
        const orderId = key.args[0].toNumber();
        const data = value.toJSON() as any;

        if (data.provider !== address) continue;

        const order: Order = {
          id: orderId,
          customer: data.customer,
          provider: data.provider,
          divinationType: data.divinationType || 0,
          hexagramId: data.hexagramId || 0,
          packageId: data.packageId || 0,
          questionCid: data.questionCid || '',
          answerCid: data.answerCid,
          status: data.status || 'PendingPayment',
          amount: BigInt(data.amount || 0),
          platformFee: BigInt(data.platformFee || 0),
          isUrgent: data.isUrgent || false,
          createdAt: data.createdAt || 0,
          acceptedAt: data.acceptedAt,
          completedAt: data.completedAt,
        };

        // 统计今日数据
        if (order.createdAt >= todayTimestamp) {
          todayNewOrders++;
        }
        if (order.completedAt && order.completedAt >= todayTimestamp) {
          todayCompleted++;
          todayEarnings += order.amount - order.platformFee;
        }

        // 分类订单
        if (order.status === 'Paid') {
          pendingOrders.push(order);
        } else if (order.status === 'Accepted') {
          activeOrders.push(order);
        }
      }

      return {
        provider: providerInfo,
        balance,
        todayStats: {
          newOrders: todayNewOrders,
          completed: todayCompleted,
          earnings: todayEarnings,
        },
        pendingOrders,
        activeOrders,
      };
    } catch (err) {
      const message = err instanceof Error ? err.message : '获取工作台数据失败';
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [address, isProvider, providerInfo, getProviderBalance]);

  /**
   * 获取我的套餐列表
   */
  const getMyPackages = useCallback(async (): Promise<ServicePackage[]> => {
    if (!address) return [];

    try {
      setLoading(true);
      setError(null);

      const api = await getApi();

      // @ts-ignore
      const entries = await api.query.divinationMarket.packages.entries(address);

      const packages: ServicePackage[] = entries.map(([key, value]: [any, any]) => {
        const [, packageId] = key.args;
        const data = value.toJSON() as any;

        return {
          id: packageId.toNumber(),
          divinationType: data.divinationType || 0,
          serviceType: data.serviceType || 0,
          name: data.name || '',
          description: data.description || '',
          price: BigInt(data.price || 0),
          duration: data.duration || 0,
          followUpCount: data.followUpCount || 0,
          urgentAvailable: data.urgentAvailable || false,
          urgentSurcharge: data.urgentSurcharge || 0,
          isActive: data.isActive !== false,
          salesCount: data.salesCount || 0,
        };
      });

      return packages;
    } catch (err) {
      const message = err instanceof Error ? err.message : '获取套餐列表失败';
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [address]);

  /**
   * 获取提现记录
   */
  const getWithdrawals = useCallback(async (): Promise<WithdrawalRequest[]> => {
    if (!address) return [];

    try {
      setLoading(true);
      setError(null);

      const api = await getApi();

      // @ts-ignore
      const entries = await api.query.divinationMarket.withdrawals.entries();

      const withdrawals: WithdrawalRequest[] = [];
      for (const [key, value] of entries) {
        const data = value.toJSON() as any;
        if (data.provider !== address) continue;

        withdrawals.push({
          id: key.args[0].toNumber(),
          provider: data.provider,
          amount: BigInt(data.amount || 0),
          status: data.status || 'Pending',
          createdAt: data.createdAt || 0,
          completedAt: data.completedAt,
        });
      }

      // 按时间倒序
      withdrawals.sort((a, b) => b.createdAt - a.createdAt);

      return withdrawals;
    } catch (err) {
      const message = err instanceof Error ? err.message : '获取提现记录失败';
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [address]);

  // 初始化时检查是否是提供者
  useEffect(() => {
    if (address) {
      checkIsProvider();
    } else {
      setIsProvider(false);
      setProviderInfo(null);
    }
  }, [address, checkIsProvider]);

  return {
    loading,
    error,
    isProvider,
    providerInfo,
    checkIsProvider,
    getProviderBalance,
    getDashboard,
    getMyPackages,
    getWithdrawals,
  };
}
