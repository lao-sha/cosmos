import { Buffer } from 'buffer';

if (typeof global.Buffer === 'undefined') {
  global.Buffer = Buffer;
}

import { mnemonicGenerate, mnemonicValidate, cryptoWaitReady } from '@polkadot/util-crypto';
import { Keyring } from '@polkadot/keyring';
import * as SecureStore from 'expo-secure-store';

const MNEMONIC_KEY = 'cosmos_mnemonic';

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
    await SecureStore.setItemAsync(MNEMONIC_KEY, mnemonic, {
      keychainAccessible: SecureStore.WHEN_UNLOCKED,
    });
  }

  /**
   * 获取存储的助记词
   */
  static async getMnemonic(): Promise<string | null> {
    return await SecureStore.getItemAsync(MNEMONIC_KEY);
  }

  /**
   * 删除钱包（注销）
   */
  static async clearWallet() {
    await SecureStore.deleteItemAsync(MNEMONIC_KEY);
  }

  /**
   * 从助记词派生账户
   */
  static async getAccountFromMnemonic(mnemonic: string) {
    await cryptoWaitReady();
    return this.keyring.addFromUri(mnemonic);
  }
}

