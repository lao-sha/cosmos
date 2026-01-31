import AsyncStorage from '@react-native-async-storage/async-storage';
import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';
import { AccountInfo, WalletService } from '../lib/wallet';
import { useAuthStore } from './auth';

interface WalletState {
  // 状态
  isInitialized: boolean;
  accounts: AccountInfo[];
  activeAccountId: string | null;
  
  // 计算属性
  currentAccount: AccountInfo | null;
  
  // 操作
  initialize: () => Promise<void>;
  createAccount: (name: string) => Promise<AccountInfo | null>;
  importAccount: (name: string, mnemonic: string) => Promise<AccountInfo | null>;
  switchAccount: (accountId: string) => Promise<void>;
  deleteAccount: (accountId: string) => Promise<boolean>;
  updateAccountName: (accountId: string, name: string) => Promise<boolean>;
  refreshAccounts: () => Promise<void>;
  reset: () => Promise<void>;
}

export const useWalletStore = create<WalletState>()(
  persist(
    (set, get) => ({
      isInitialized: false,
      accounts: [],
      activeAccountId: null,
      
      get currentAccount() {
        const state = get();
        if (!state.activeAccountId || state.accounts.length === 0) return null;
        return state.accounts.find(a => a.id === state.activeAccountId) || null;
      },

      initialize: async () => {
        await WalletService.init();
        const walletData = await WalletService.getWalletData();
        
        set({
          isInitialized: true,
          accounts: walletData.accounts,
          activeAccountId: walletData.activeAccountId,
        });
      },

      createAccount: async (name: string) => {
        const newAccount = await WalletService.createAccount(name);
        if (newAccount) {
          const accounts = await WalletService.getAccounts();
          set({ accounts });
        }
        return newAccount;
      },

      importAccount: async (name: string, mnemonic: string) => {
        const newAccount = await WalletService.importAccount(name, mnemonic);
        if (newAccount) {
          const accounts = await WalletService.getAccounts();
          set({ accounts });
        }
        return newAccount;
      },

      switchAccount: async (accountId: string) => {
        const { accounts } = get();
        const account = accounts.find(a => a.id === accountId);
        if (account) {
          await WalletService.saveActiveAccountId(accountId);
          set({ activeAccountId: accountId });
          // 同步更新 authStore 的地址
          useAuthStore.setState({ address: account.address });
        }
      },

      deleteAccount: async (accountId: string) => {
        const { activeAccountId, accounts } = get();
        const success = await WalletService.deleteAccount(accountId);
        
        if (success) {
          const newAccounts = await WalletService.getAccounts();
          
          // 如果删除的是当前账户，切换到第一个账户
          let newActiveId = activeAccountId;
          if (activeAccountId === accountId && newAccounts.length > 0) {
            newActiveId = newAccounts[0].id;
            await WalletService.saveActiveAccountId(newActiveId);
          }
          
          set({ 
            accounts: newAccounts,
            activeAccountId: newActiveId,
          });
        }
        
        return success;
      },

      updateAccountName: async (accountId: string, name: string) => {
        const success = await WalletService.updateAccountName(accountId, name);
        if (success) {
          const accounts = await WalletService.getAccounts();
          set({ accounts });
        }
        return success;
      },

      refreshAccounts: async () => {
        const walletData = await WalletService.getWalletData();
        set({
          accounts: walletData.accounts,
          activeAccountId: walletData.activeAccountId,
        });
      },

      reset: async () => {
        await WalletService.clearAllWalletData();
        set({
          isInitialized: false,
          accounts: [],
          activeAccountId: null,
        });
      },
    }),
    {
      name: 'wallet-storage',
      storage: createJSONStorage(() => AsyncStorage),
      partialize: (state) => ({
        activeAccountId: state.activeAccountId,
      }),
    }
  )
);

// 辅助 hook：获取当前账户
export const useCurrentAccount = () => {
  const { accounts, activeAccountId } = useWalletStore();
  if (!activeAccountId || accounts.length === 0) return null;
  return accounts.find(a => a.id === activeAccountId) || null;
};
