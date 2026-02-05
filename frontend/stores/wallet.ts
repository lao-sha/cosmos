import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { Keyring } from '@polkadot/keyring';
import { mnemonicGenerate, mnemonicValidate } from '@polkadot/util-crypto';
import * as SecureStore from 'expo-secure-store';

const SECURE_KEY = 'cosmos_wallet_secret';

interface WalletState {
  address: string | null;
  name: string;
  isConnected: boolean;
  isLocked: boolean;
  mnemonic: string | null;

  // Actions
  createWallet: (name: string, password: string) => Promise<string>;
  importWallet: (secret: string, name: string, password: string) => Promise<string>;
  connect: (address: string) => void;
  disconnect: () => void;
  lock: () => void;
  unlock: (password: string) => Promise<boolean>;
  getMnemonic: () => string | null;
}

export const useWalletStore = create<WalletState>()(
  persist(
    (set, get) => ({
      address: null,
      name: '',
      isConnected: false,
      isLocked: true,
      mnemonic: null,

      createWallet: async (name: string, password: string) => {
        const mnemonic = mnemonicGenerate();
        const keyring = new Keyring({ type: 'sr25519' });
        const pair = keyring.addFromMnemonic(mnemonic);

        // Store encrypted secret securely
        try {
          await SecureStore.setItemAsync(SECURE_KEY, mnemonic);
        } catch (e) {
          console.warn('SecureStore not available, using memory only');
        }

        set({
          address: pair.address,
          name,
          isConnected: true,
          isLocked: false,
          mnemonic,
        });

        return mnemonic;
      },

      importWallet: async (secret: string, name: string, password: string) => {
        const keyring = new Keyring({ type: 'sr25519' });
        let pair;

        // Try as mnemonic first
        if (mnemonicValidate(secret)) {
          pair = keyring.addFromMnemonic(secret);
        } else {
          // Try as private key (hex)
          const key = secret.startsWith('0x') ? secret : `0x${secret}`;
          pair = keyring.addFromUri(key);
        }

        // Store encrypted secret securely
        try {
          await SecureStore.setItemAsync(SECURE_KEY, secret);
        } catch (e) {
          console.warn('SecureStore not available, using memory only');
        }

        set({
          address: pair.address,
          name,
          isConnected: true,
          isLocked: false,
          mnemonic: mnemonicValidate(secret) ? secret : null,
        });

        return pair.address;
      },

      connect: (address: string) => {
        set({ address, isConnected: true });
      },

      disconnect: async () => {
        try {
          await SecureStore.deleteItemAsync(SECURE_KEY);
        } catch (e) {
          // Ignore
        }
        set({
          address: null,
          name: '',
          isConnected: false,
          isLocked: true,
          mnemonic: null,
        });
      },

      lock: () => {
        set({ isLocked: true, mnemonic: null });
      },

      unlock: async (password: string) => {
        // TODO: Verify password with stored encrypted key
        set({ isLocked: false });
        return true;
      },

      getMnemonic: () => {
        return get().mnemonic;
      },
    }),
    {
      name: 'wallet-storage',
      storage: createJSONStorage(() => AsyncStorage),
      partialize: (state) => ({
        address: state.address,
        name: state.name,
        isConnected: state.isConnected,
      }),
    }
  )
);
