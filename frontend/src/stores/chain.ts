import { ApiPromise, WsProvider } from '@polkadot/api';
import { create } from 'zustand';

interface ChainState {
  api: ApiPromise | null;
  isConnected: boolean;
  isReady: boolean;
  connect: (endpoint: string) => Promise<void>;
  disconnect: () => Promise<void>;
}

export const useChainStore = create<ChainState>((set, get) => ({
  api: null,
  isConnected: false,
  isReady: false,
  connect: async (endpoint: string) => {
    if (get().api) return;

    try {
      const provider = new WsProvider(endpoint);
      const api = await ApiPromise.create({ provider });
      
      api.on('connected', () => set({ isConnected: true }));
      api.on('disconnected', () => set({ isConnected: false }));
      
      await api.isReady;
      set({ api, isReady: true, isConnected: true });
    } catch (error) {
      console.error('Failed to connect to chain:', error);
    }
  },
  disconnect: async () => {
    const { api } = get();
    if (api) {
      await api.disconnect();
      set({ api: null, isConnected: false, isReady: false });
    }
  },
}));

