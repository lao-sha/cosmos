/**
 * 星尘玄鉴 - 钱包状态管理
 * 使用 Zustand 管理钱包状态
 */

import { create } from 'zustand';
import {
  initializeCrypto,
  generateMnemonic,
  validateMnemonic,
  createKeyPairFromMnemonic,
  storeEncryptedMnemonic,
  retrieveEncryptedMnemonic,
  hasWallet as checkHasWallet,
  getStoredAddress,
  deleteWallet as removeWallet,
  WalletError,
  AuthenticationError,
} from '@/lib';

interface WalletState {
  // 状态
  isReady: boolean;
  hasWallet: boolean;
  isLocked: boolean;
  address: string | null;
  error: string | null;
  isLoading: boolean;

  // 操作方法
  initialize: () => Promise<void>;
  createWallet: (password: string) => Promise<string>;
  importWallet: (mnemonic: string, password: string) => Promise<void>;
  unlockWallet: (password: string) => Promise<void>;
  lockWallet: () => void;
  deleteWallet: () => Promise<void>;
  clearError: () => void;
}

export const useWalletStore = create<WalletState>()((set) => ({
  // 初始状态
  isReady: false,
  hasWallet: false,
  isLocked: true,
  address: null,
  error: null,
  isLoading: false,

  /**
   * 初始化钱包系统
   */
  initialize: async () => {
    try {
      set({ isLoading: true, error: null });

      await initializeCrypto();

      const hasWallet = await checkHasWallet();
      const address = hasWallet ? await getStoredAddress() : null;

      set({
        isReady: true,
        hasWallet,
        isLocked: hasWallet,
        address,
        error: null,
      });

      console.log('[Wallet] Initialized:', { hasWallet, address });
    } catch (error) {
      console.error('[Wallet] Initialize error:', error);
      set({
        isReady: true,
        hasWallet: false,
        isLocked: true,
        error: '初始化失败',
      });
    } finally {
      set({ isLoading: false });
    }
  },

  /**
   * 创建新钱包
   */
  createWallet: async (password: string) => {
    try {
      set({ isLoading: true, error: null });

      if (!password || password.length < 8) {
        throw new WalletError('密码至少需要 8 位');
      }

      const mnemonic = generateMnemonic();
      const pair = createKeyPairFromMnemonic(mnemonic);

      await storeEncryptedMnemonic(mnemonic, password, pair.address);

      set({
        hasWallet: true,
        isLocked: false,
        address: pair.address,
        error: null,
      });

      console.log('[Wallet] Created:', pair.address);
      return mnemonic;
    } catch (error) {
      const message = error instanceof Error ? error.message : '创建钱包失败';
      console.error('[Wallet] Create error:', error);
      set({ error: message });
      throw new WalletError(message, error);
    } finally {
      set({ isLoading: false });
    }
  },

  /**
   * 导入钱包
   */
  importWallet: async (mnemonic: string, password: string) => {
    try {
      set({ isLoading: true, error: null });

      if (!validateMnemonic(mnemonic)) {
        throw new WalletError('无效的助记词');
      }

      if (!password || password.length < 8) {
        throw new WalletError('密码至少需要 8 位');
      }

      const pair = createKeyPairFromMnemonic(mnemonic);

      await storeEncryptedMnemonic(mnemonic, password, pair.address);

      set({
        hasWallet: true,
        isLocked: false,
        address: pair.address,
        error: null,
      });

      console.log('[Wallet] Imported:', pair.address);
    } catch (error) {
      const message = error instanceof Error ? error.message : '导入钱包失败';
      console.error('[Wallet] Import error:', error);
      set({ error: message });
      throw new WalletError(message, error);
    } finally {
      set({ isLoading: false });
    }
  },

  /**
   * 解锁钱包
   */
  unlockWallet: async (password: string) => {
    try {
      set({ isLoading: true, error: null });

      const mnemonic = await retrieveEncryptedMnemonic(password);
      const pair = createKeyPairFromMnemonic(mnemonic);

      set({
        isLocked: false,
        address: pair.address,
        error: null,
      });

      console.log('[Wallet] Unlocked:', pair.address);
    } catch (error) {
      const message = error instanceof AuthenticationError ? '密码错误' : '解锁失败';
      console.error('[Wallet] Unlock error:', error);
      set({ error: message });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  /**
   * 锁定钱包
   */
  lockWallet: () => {
    set({
      isLocked: true,
      error: null,
    });
    console.log('[Wallet] Locked');
  },

  /**
   * 删除钱包
   */
  deleteWallet: async () => {
    try {
      set({ isLoading: true, error: null });

      await removeWallet();

      set({
        hasWallet: false,
        isLocked: true,
        address: null,
        error: null,
      });

      console.log('[Wallet] Deleted');
    } catch (error) {
      console.error('[Wallet] Delete error:', error);
      const message = '删除钱包失败';
      set({ error: message });
      throw new WalletError(message, error);
    } finally {
      set({ isLoading: false });
    }
  },

  /**
   * 清除错误
   */
  clearError: () => {
    set({ error: null });
  },
}));
