import { Buffer } from 'buffer';
import { Platform } from 'react-native';

if (typeof global.Buffer === 'undefined') {
  global.Buffer = Buffer;
}

import { Keyring } from '@polkadot/keyring';
import type { KeyringPair } from '@polkadot/keyring/types';
import { cryptoWaitReady, mnemonicGenerate, mnemonicValidate } from '@polkadot/util-crypto';

const MNEMONIC_KEY = 'cosmos_mnemonic';
const ACCOUNTS_KEY = 'cosmos_accounts';
const ACTIVE_ACCOUNT_KEY = 'cosmos_active_account';
const HD_PATH_PREFIX = '//cosmos//';

export interface AccountInfo {
  id: string;
  name: string;
  address: string;
  derivationIndex: number;
  isImported: boolean;
  createdAt: number;
}

export interface WalletData {
  accounts: AccountInfo[];
  activeAccountId: string | null;
}

const storage = {
  async setItem(key: string, value: string): Promise<void> {
    if (Platform.OS === 'web') {
      localStorage.setItem(key, value);
    } else {
      const SecureStore = await import('expo-secure-store');
      await SecureStore.setItemAsync(key, value, {
        keychainAccessible: SecureStore.WHEN_UNLOCKED,
      });
    }
  },

  async getItem(key: string): Promise<string | null> {
    if (Platform.OS === 'web') {
      return localStorage.getItem(key);
    } else {
      const SecureStore = await import('expo-secure-store');
      return await SecureStore.getItemAsync(key);
    }
  },

  async removeItem(key: string): Promise<void> {
    if (Platform.OS === 'web') {
      localStorage.removeItem(key);
    } else {
      const SecureStore = await import('expo-secure-store');
      await SecureStore.deleteItemAsync(key);
    }
  },
};

export class WalletService {
  private static keyring = new Keyring({ type: 'sr25519' });

  static async init() {
    await cryptoWaitReady();
  }

  /**
   * 生成新助记词
   */
  static generateMnemonic(): string {
    return mnemonicGenerate(12);
  }

  /**
   * 验证助记词
   */
  static validateMnemonic(mnemonic: string): boolean {
    return mnemonicValidate(mnemonic);
  }

  /**
   * 安全存储助记词
   */
  static async saveMnemonic(mnemonic: string) {
    await storage.setItem(MNEMONIC_KEY, mnemonic);
  }

  /**
   * 获取存储的助记词
   */
  static async getMnemonic(): Promise<string | null> {
    return await storage.getItem(MNEMONIC_KEY);
  }

  /**
   * 删除钱包（注销）
   */
  static async clearWallet() {
    await storage.removeItem(MNEMONIC_KEY);
  }

  /**
   * 从助记词派生账户（默认主账户）
   */
  static async getAccountFromMnemonic(mnemonic: string): Promise<KeyringPair> {
    await cryptoWaitReady();
    return this.keyring.addFromUri(mnemonic);
  }

  /**
   * 从助记词派生指定索引的账户
   */
  static async deriveAccount(mnemonic: string, index: number): Promise<KeyringPair> {
    await cryptoWaitReady();
    const derivationPath = `${mnemonic}${HD_PATH_PREFIX}${index}`;
    return this.keyring.addFromUri(derivationPath);
  }

  /**
   * 生成账户ID
   */
  static generateAccountId(): string {
    return `account_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
  }

  /**
   * 保存账户列表
   */
  static async saveAccounts(accounts: AccountInfo[]): Promise<void> {
    await storage.setItem(ACCOUNTS_KEY, JSON.stringify(accounts));
  }

  /**
   * 获取账户列表
   */
  static async getAccounts(): Promise<AccountInfo[]> {
    const data = await storage.getItem(ACCOUNTS_KEY);
    if (!data) return [];
    try {
      return JSON.parse(data);
    } catch {
      return [];
    }
  }

  /**
   * 保存当前激活账户ID
   */
  static async saveActiveAccountId(accountId: string): Promise<void> {
    await storage.setItem(ACTIVE_ACCOUNT_KEY, accountId);
  }

  /**
   * 获取当前激活账户ID
   */
  static async getActiveAccountId(): Promise<string | null> {
    return await storage.getItem(ACTIVE_ACCOUNT_KEY);
  }

  /**
   * 创建新账户
   */
  static async createAccount(name: string): Promise<AccountInfo | null> {
    const mnemonic = await this.getMnemonic();
    if (!mnemonic) return null;

    const accounts = await this.getAccounts();
    const nextIndex = accounts.length > 0 
      ? Math.max(...accounts.filter(a => !a.isImported).map(a => a.derivationIndex)) + 1 
      : 0;

    const keyPair = await this.deriveAccount(mnemonic, nextIndex);
    
    const newAccount: AccountInfo = {
      id: this.generateAccountId(),
      name: name || `账户 ${nextIndex + 1}`,
      address: keyPair.address,
      derivationIndex: nextIndex,
      isImported: false,
      createdAt: Date.now(),
    };

    accounts.push(newAccount);
    await this.saveAccounts(accounts);

    return newAccount;
  }

  /**
   * 导入账户（使用独立助记词）
   */
  static async importAccount(name: string, importedMnemonic: string): Promise<AccountInfo | null> {
    if (!this.validateMnemonic(importedMnemonic)) {
      return null;
    }

    await cryptoWaitReady();
    const keyPair = this.keyring.addFromUri(importedMnemonic);
    
    const accounts = await this.getAccounts();
    
    // 检查是否已存在相同地址
    if (accounts.some(a => a.address === keyPair.address)) {
      return null;
    }

    const newAccount: AccountInfo = {
      id: this.generateAccountId(),
      name: name || '导入账户',
      address: keyPair.address,
      derivationIndex: -1, // 导入账户不使用派生索引
      isImported: true,
      createdAt: Date.now(),
    };

    accounts.push(newAccount);
    await this.saveAccounts(accounts);

    // 存储导入账户的助记词（使用账户ID作为key）
    await storage.setItem(`mnemonic_${newAccount.id}`, importedMnemonic);

    return newAccount;
  }

  /**
   * 获取账户的 KeyPair
   */
  static async getKeyPairForAccount(account: AccountInfo): Promise<KeyringPair | null> {
    await cryptoWaitReady();

    if (account.isImported) {
      // 导入账户使用独立助记词
      const importedMnemonic = await storage.getItem(`mnemonic_${account.id}`);
      if (!importedMnemonic) return null;
      return this.keyring.addFromUri(importedMnemonic);
    } else {
      // HD派生账户
      const mnemonic = await this.getMnemonic();
      if (!mnemonic) return null;
      return this.deriveAccount(mnemonic, account.derivationIndex);
    }
  }

  /**
   * 删除账户
   */
  static async deleteAccount(accountId: string): Promise<boolean> {
    const accounts = await this.getAccounts();
    const account = accounts.find(a => a.id === accountId);
    
    if (!account) return false;
    
    // 主账户（index 0）不允许删除
    if (!account.isImported && account.derivationIndex === 0) {
      return false;
    }

    const newAccounts = accounts.filter(a => a.id !== accountId);
    await this.saveAccounts(newAccounts);

    // 如果是导入账户，删除其助记词
    if (account.isImported) {
      await storage.removeItem(`mnemonic_${accountId}`);
    }

    return true;
  }

  /**
   * 更新账户名称
   */
  static async updateAccountName(accountId: string, newName: string): Promise<boolean> {
    const accounts = await this.getAccounts();
    const account = accounts.find(a => a.id === accountId);
    
    if (!account) return false;
    
    account.name = newName;
    await this.saveAccounts(accounts);
    
    return true;
  }

  /**
   * 获取完整钱包数据
   */
  static async getWalletData(): Promise<WalletData> {
    const accounts = await this.getAccounts();
    const activeAccountId = await this.getActiveAccountId();
    
    return {
      accounts,
      activeAccountId,
    };
  }

  /**
   * 初始化主账户（首次创建钱包时调用）
   */
  static async initializePrimaryAccount(mnemonic: string): Promise<AccountInfo> {
    await cryptoWaitReady();
    
    // 使用派生路径创建主账户
    const keyPair = await this.deriveAccount(mnemonic, 0);
    
    const primaryAccount: AccountInfo = {
      id: this.generateAccountId(),
      name: '主账户',
      address: keyPair.address,
      derivationIndex: 0,
      isImported: false,
      createdAt: Date.now(),
    };

    await this.saveAccounts([primaryAccount]);
    await this.saveActiveAccountId(primaryAccount.id);

    return primaryAccount;
  }

  /**
   * 清除所有钱包数据
   */
  static async clearAllWalletData(): Promise<void> {
    const accounts = await this.getAccounts();
    
    // 删除所有导入账户的助记词
    for (const account of accounts) {
      if (account.isImported) {
        await storage.removeItem(`mnemonic_${account.id}`);
      }
    }

    await storage.removeItem(MNEMONIC_KEY);
    await storage.removeItem(ACCOUNTS_KEY);
    await storage.removeItem(ACTIVE_ACCOUNT_KEY);
  }
}

