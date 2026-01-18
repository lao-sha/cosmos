// frontend/src/divination/market/hooks/useMarketApi.ts

import { useState, useCallback } from 'react';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { useWalletStore } from '@/stores/wallet.store';
import {
  Provider,
  ServicePackage,
  Order,
  Review,
  DivinationType,
  CreateOrderParams,
  SubmitReviewParams,
  ProviderFilterParams,
} from '../types/market.types';

// API 端点
const WS_ENDPOINT = process.env.EXPO_PUBLIC_WS_ENDPOINT || 'ws://127.0.0.1:9944';

let apiInstance: ApiPromise | null = null;
let apiInitPromise: Promise<ApiPromise> | null = null;

/**
 * 获取 API 实例（单例）
 */
async function getApi(): Promise<ApiPromise> {
  if (apiInstance && apiInstance.isConnected) {
    return apiInstance;
  }

  if (apiInitPromise) {
    return apiInitPromise;
  }

  apiInitPromise = (async () => {
    const provider = new WsProvider(WS_ENDPOINT);
    const api = await ApiPromise.create({ provider });
    apiInstance = api;
    return api;
  })();

  return apiInitPromise;
}

export function useMarketApi() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { address } = useWalletStore();

  /**
   * 获取所有提供者列表
   */
  const getProviders = useCallback(
    async (params?: ProviderFilterParams): Promise<Provider[]> => {
      try {
        setLoading(true);
        setError(null);

        const api = await getApi();

        // 查询所有提供者
        // @ts-ignore - 链上 runtime API
        const entries = await api.query.divinationMarket.providers.entries();

        const providers: Provider[] = entries.map(([key, value]: [any, any]) => {
          const account = key.args[0].toString();
          const data = value.toJSON() as any;

          return {
            account,
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
          };
        });

        // 过滤
        let filtered = providers.filter((p) => p.status === 'Active');

        if (params?.filterType !== undefined && params.filterType !== 'all') {
          filtered = filtered.filter(
            (p) => p.supportedTypes & (1 << (params.filterType as number))
          );
        }

        if (params?.keyword) {
          const keyword = params.keyword.toLowerCase();
          filtered = filtered.filter(
            (p) =>
              p.name.toLowerCase().includes(keyword) ||
              p.bio.toLowerCase().includes(keyword)
          );
        }

        // 排序
        if (params?.sortType === 'rating') {
          filtered.sort((a, b) => {
            const ratingA = a.ratingCount > 0 ? a.totalRating / a.ratingCount : 0;
            const ratingB = b.ratingCount > 0 ? b.totalRating / b.ratingCount : 0;
            return ratingB - ratingA;
          });
        } else if (params?.sortType === 'orders') {
          filtered.sort((a, b) => b.completedOrders - a.completedOrders);
        }

        return filtered;
      } catch (err) {
        const message = err instanceof Error ? err.message : '获取提供者列表失败';
        setError(message);
        throw err;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  /**
   * 获取单个提供者详情
   */
  const getProvider = useCallback(async (account: string): Promise<Provider | null> => {
    try {
      setLoading(true);
      setError(null);

      const api = await getApi();

      // @ts-ignore
      const result = await api.query.divinationMarket.providers(account);
      if (result.isNone) {
        return null;
      }

      const data = result.toJSON() as any;
      return {
        account,
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
      };
    } catch (err) {
      const message = err instanceof Error ? err.message : '获取提供者详情失败';
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * 获取提供者的套餐列表
   */
  const getProviderPackages = useCallback(
    async (providerAccount: string): Promise<ServicePackage[]> => {
      try {
        setLoading(true);
        setError(null);

        const api = await getApi();

        // @ts-ignore
        const entries = await api.query.divinationMarket.packages.entries(providerAccount);

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

        return packages.filter((p) => p.isActive);
      } catch (err) {
        const message = err instanceof Error ? err.message : '获取套餐列表失败';
        setError(message);
        throw err;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  /**
   * 获取提供者的评价列表
   */
  const getProviderReviews = useCallback(
    async (providerAccount: string): Promise<Review[]> => {
      try {
        setLoading(true);
        setError(null);

        const api = await getApi();

        // @ts-ignore
        const entries = await api.query.divinationMarket.reviews.entries();

        const reviews: Review[] = [];
        for (const [key, value] of entries) {
          const orderId = key.args[0].toNumber();
          const data = value.toJSON() as any;

          // 需要查询订单来确认是否属于该提供者
          // @ts-ignore
          const orderResult = await api.query.divinationMarket.orders(orderId);
          if (orderResult.isNone) continue;

          const orderData = orderResult.toJSON() as any;
          if (orderData.provider !== providerAccount) continue;

          reviews.push({
            orderId,
            customer: data.customer || '',
            overallRating: data.overallRating || 0,
            accuracyRating: data.accuracyRating || 0,
            attitudeRating: data.attitudeRating || 0,
            responseRating: data.responseRating || 0,
            contentCid: data.contentCid,
            isAnonymous: data.isAnonymous || false,
            replyCid: data.replyCid,
            createdAt: data.createdAt || 0,
            repliedAt: data.repliedAt,
          });
        }

        // 按时间倒序
        reviews.sort((a, b) => b.createdAt - a.createdAt);

        return reviews;
      } catch (err) {
        const message = err instanceof Error ? err.message : '获取评价列表失败';
        setError(message);
        throw err;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  return {
    loading,
    error,
    getProviders,
    getProvider,
    getProviderPackages,
    getProviderReviews,
  };
}
