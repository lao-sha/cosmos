import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { WalletService } from '@/lib/wallet';

interface AuthState {
  isLoggedIn: boolean;
  address: string | null;
  mnemonic: string | null;
  login: (address: string, mnemonic: string) => void;
  logout: () => void;
  initialize: () => Promise<void>;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      isLoggedIn: false,
      address: null,
      mnemonic: null,
      login: (address, mnemonic) => {
        set({ isLoggedIn: true, address, mnemonic });
      },
      logout: async () => {
        await WalletService.clearWallet();
        set({ isLoggedIn: false, address: null, mnemonic: null });
      },
      initialize: async () => {
        await WalletService.init();
        const mnemonic = await WalletService.getMnemonic();
        if (mnemonic) {
          const account = await WalletService.getAccountFromMnemonic(mnemonic);
          set({ isLoggedIn: true, address: account.address, mnemonic });
        }
      },
    }),
    {
      name: 'auth-storage',
      storage: createJSONStorage(() => AsyncStorage),
      partialize: (state) => ({ 
        isLoggedIn: state.isLoggedIn, 
        address: state.address 
      }), // 助记词不序列化到持久化存储（AsyncStorage），只存在于内存和 SecureStore
    }
  )
);

