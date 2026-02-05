import { useQuery } from '@tanstack/react-query';
import { getApi } from '@/services/api';
import { useWalletStore } from '@/stores/wallet';

export function useBalance(address?: string) {
  const walletAddress = useWalletStore((state) => state.address);
  const targetAddress = address || walletAddress;

  return useQuery({
    queryKey: ['balance', targetAddress],
    queryFn: async () => {
      if (!targetAddress) return '0';
      
      const api = await getApi();
      const { data } = await api.query.system.account(targetAddress);
      return data.free.toString();
    },
    enabled: !!targetAddress,
    refetchInterval: 12000,
  });
}

export function formatBalance(balance: string, decimals = 12): string {
  const value = BigInt(balance);
  const divisor = BigInt(10 ** decimals);
  const whole = value / divisor;
  const fraction = value % divisor;
  
  const fractionStr = fraction.toString().padStart(decimals, '0').slice(0, 4);
  return `${whole}.${fractionStr}`;
}
