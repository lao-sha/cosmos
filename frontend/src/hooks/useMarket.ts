import { useChainStore } from '@/src/stores/chain';
import { useCallback, useEffect, useState } from 'react';

export type DivinationType = 'meihua' | 'bazi' | 'liuyao' | 'qimen' | 'ziwei' | 'tarot';

export interface DivinationCategory {
  id: DivinationType;
  name: string;
  icon: string;
  description: string;
}

export interface Provider {
  id: string;
  name: string;
  avatar?: string;
  rating: number;
  orderCount: number;
  specialties: DivinationType[];
  price: number;
  isOnline: boolean;
}

export const CATEGORIES: DivinationCategory[] = [
  { id: 'meihua', name: '梅花易数', icon: '🌸', description: '以数起卦，灵活应变' },
  { id: 'bazi', name: '八字命理', icon: '📅', description: '四柱推命，知命改运' },
  { id: 'liuyao', name: '六爻占卜', icon: '🎲', description: '摇卦断事，趋吉避凶' },
  { id: 'qimen', name: '奇门遁甲', icon: '🧭', description: '时空择吉，运筹帷幄' },
  { id: 'ziwei', name: '紫微斗数', icon: '⭐', description: '帝王之学，命宫推演' },
  { id: 'tarot', name: '塔罗占卜', icon: '🃏', description: '西方神秘学，直觉引导' },
];

const MOCK_PROVIDERS: Provider[] = [
  {
    id: '1',
    name: '玄空大师',
    rating: 4.9,
    orderCount: 1280,
    specialties: ['bazi', 'ziwei'],
    price: 88,
    isOnline: true,
  },
  {
    id: '2',
    name: '易卦先生',
    rating: 4.8,
    orderCount: 856,
    specialties: ['meihua', 'liuyao'],
    price: 68,
    isOnline: true,
  },
  {
    id: '3',
    name: '紫霞仙子',
    rating: 4.7,
    orderCount: 632,
    specialties: ['tarot', 'ziwei'],
    price: 58,
    isOnline: false,
  },
];

export function useProviders(category?: DivinationType | null) {
  const { api, isConnected } = useChainStore();
  const [providers, setProviders] = useState<Provider[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchProviders = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      if (api && isConnected) {
        const entries = await (api.query as any).divinationMarket?.providers?.entries();
        if (entries && entries.length > 0) {
          const chainProviders: Provider[] = [];
          for (const [key, value] of entries) {
            if (value && value.isSome) {
              const data = value.unwrap();
              const specialties = (data.supportedTypes?.toHuman() as string[]) || [];
              chainProviders.push({
                id: key.args[0].toString(),
                name: (data.name?.toHuman() as string) || '未知大师',
                rating: (data.rating?.toNumber() || 45) / 10,
                orderCount: data.totalOrders?.toNumber() || 0,
                specialties: specialties as DivinationType[],
                // 链上精度：1 COS = 1e12 最小单位
                price: (data.minPrice?.toNumber() || 0) / 1e12,
                isOnline: data.status?.toHuman() === 'Active',
              });
            }
          }
          if (chainProviders.length > 0) {
            const filtered = category
              ? chainProviders.filter(p => p.specialties.includes(category))
              : chainProviders;
            setProviders(filtered);
            return;
          }
        }
      }
      
      const filtered = category
        ? MOCK_PROVIDERS.filter(p => p.specialties.includes(category))
        : MOCK_PROVIDERS;
      setProviders(filtered);
    } catch (err) {
      console.error('Failed to fetch providers:', err);
      setProviders(MOCK_PROVIDERS);
      setError('链上查询失败，使用模拟数据');
    } finally {
      setLoading(false);
    }
  }, [api, isConnected, category]);

  useEffect(() => {
    fetchProviders();
  }, [fetchProviders]);

  return { providers, loading, error, refetch: fetchProviders, categories: CATEGORIES };
}

export function useMarketStats() {
  const { api, isConnected } = useChainStore();
  const [stats, setStats] = useState({
    totalProviders: 0,
    totalOrders: 0,
    totalVolume: '0',
  });

  useEffect(() => {
    async function fetch() {
      try {
        if (api && isConnected) {
          const marketStats = await (api.query as any).divinationMarket?.marketStats?.();
          if (marketStats) {
            setStats({
              totalProviders: marketStats.totalProviders?.toNumber() || 3,
              totalOrders: marketStats.totalOrders?.toNumber() || 2768,
              totalVolume: marketStats.totalVolume?.toString() || '1000000',
            });
            return;
          }
        }
        setStats({ totalProviders: 3, totalOrders: 2768, totalVolume: '1000000' });
      } catch (err) {
        console.error('Failed to fetch market stats:', err);
        setStats({ totalProviders: 3, totalOrders: 2768, totalVolume: '1000000' });
      }
    }
    fetch();
  }, [api, isConnected]);

  return stats;
}
