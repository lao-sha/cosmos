import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  getMakers,
  getMakerById,
  getOrders,
  getOrderById,
  createBuyOrder,
  confirmPayment,
  releaseCos,
  cancelOrder,
  type Maker,
  type OtcOrder,
} from '@/services/otc';
import { useWalletStore } from '@/stores/wallet';

export function useMakers() {
  return useQuery({
    queryKey: ['makers'],
    queryFn: getMakers,
    staleTime: 30_000,
  });
}

export function useMaker(makerId: string) {
  return useQuery({
    queryKey: ['maker', makerId],
    queryFn: () => getMakerById(makerId),
    enabled: !!makerId,
  });
}

export function useOrders() {
  const address = useWalletStore((state) => state.address);

  return useQuery({
    queryKey: ['orders', address],
    queryFn: () => getOrders(address || ''),
    enabled: !!address,
    refetchInterval: 10_000,
  });
}

export function useOrder(orderId: string) {
  return useQuery({
    queryKey: ['order', orderId],
    queryFn: () => getOrderById(orderId),
    enabled: !!orderId,
    refetchInterval: 5_000,
  });
}

export function useCreateBuyOrder() {
  const queryClient = useQueryClient();
  const mnemonic = useWalletStore((state) => state.mnemonic);

  return useMutation({
    mutationFn: async ({
      makerId,
      usdtAmount,
    }: {
      makerId: string;
      usdtAmount: string;
    }) => {
      if (!mnemonic) throw new Error('Wallet not unlocked');
      return createBuyOrder(makerId, usdtAmount, mnemonic);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['orders'] });
      queryClient.invalidateQueries({ queryKey: ['makers'] });
    },
  });
}

export function useConfirmPayment() {
  const queryClient = useQueryClient();
  const mnemonic = useWalletStore((state) => state.mnemonic);

  return useMutation({
    mutationFn: async ({
      orderId,
      txHash,
    }: {
      orderId: string;
      txHash: string;
    }) => {
      if (!mnemonic) throw new Error('Wallet not unlocked');
      return confirmPayment(orderId, txHash, mnemonic);
    },
    onSuccess: (_, { orderId }) => {
      queryClient.invalidateQueries({ queryKey: ['order', orderId] });
      queryClient.invalidateQueries({ queryKey: ['orders'] });
    },
  });
}

export function useReleaseCos() {
  const queryClient = useQueryClient();
  const mnemonic = useWalletStore((state) => state.mnemonic);

  return useMutation({
    mutationFn: async (orderId: string) => {
      if (!mnemonic) throw new Error('Wallet not unlocked');
      return releaseCos(orderId, mnemonic);
    },
    onSuccess: (_, orderId) => {
      queryClient.invalidateQueries({ queryKey: ['order', orderId] });
      queryClient.invalidateQueries({ queryKey: ['orders'] });
    },
  });
}

export function useCancelOrder() {
  const queryClient = useQueryClient();
  const mnemonic = useWalletStore((state) => state.mnemonic);

  return useMutation({
    mutationFn: async (orderId: string) => {
      if (!mnemonic) throw new Error('Wallet not unlocked');
      return cancelOrder(orderId, mnemonic);
    },
    onSuccess: (_, orderId) => {
      queryClient.invalidateQueries({ queryKey: ['order', orderId] });
      queryClient.invalidateQueries({ queryKey: ['orders'] });
    },
  });
}

// Utility functions
export function formatCos(amount: string, decimals = 12): string {
  const value = BigInt(amount);
  const divisor = BigInt(10 ** decimals);
  const whole = value / divisor;
  const fraction = value % divisor;
  const fractionStr = fraction.toString().padStart(decimals, '0').slice(0, 4);
  return `${whole}.${fractionStr}`;
}

export function formatUsdt(amount: string, decimals = 6): string {
  const value = BigInt(amount);
  const divisor = BigInt(10 ** decimals);
  const whole = value / divisor;
  const fraction = value % divisor;
  const fractionStr = fraction.toString().padStart(decimals, '0').slice(0, 2);
  return `${whole}.${fractionStr}`;
}

export function formatPrice(price: string): string {
  const value = Number(price) / 10000;
  return value.toFixed(4);
}

export function parseUsdt(amount: string, decimals = 6): string {
  const [whole, fraction = ''] = amount.split('.');
  const paddedFraction = fraction.padEnd(decimals, '0').slice(0, decimals);
  return BigInt(whole + paddedFraction).toString();
}
