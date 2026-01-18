/**
 * 星尘玄鉴 - 密钥环服务
 * 管理账户的创建、导入和签名
 */

import { Keyring } from '@polkadot/keyring';
import {
  mnemonicGenerate,
  mnemonicValidate,
  cryptoWaitReady,
} from '@polkadot/util-crypto';
import type { KeyringPair } from '@polkadot/keyring/types';
import type { WalletAccount } from '@/features/wallet/types';

/**
 * 密钥环服务
 */
export class KeyringService {
  private keyring: Keyring | null = null;
  private pairs: Map<string, KeyringPair> = new Map();
  private mnemonic: string | null = null;
  private isInitialized = false;

  /**
   * 初始化密钥环
   */
  async init(): Promise<void> {
    if (this.isInitialized) return;

    await cryptoWaitReady();
    this.keyring = new Keyring({ type: 'sr25519', ss58Format: 42 });
    this.isInitialized = true;
  }

  /**
   * 确保已初始化
   */
  private async ensureInit(): Promise<void> {
    if (!this.isInitialized) {
      await this.init();
    }
  }

  /**
   * 生成新的助记词
   */
  generateMnemonic(words: 12 | 24 = 12): string {
    return mnemonicGenerate(words);
  }

  /**
   * 验证助记词
   */
  validateMnemonic(mnemonic: string): boolean {
    return mnemonicValidate(mnemonic);
  }

  /**
   * 从助记词创建钱包
   */
  async createFromMnemonic(
    mnemonic: string,
    accountName: string = '主账户'
  ): Promise<WalletAccount> {
    await this.ensureInit();

    if (!this.validateMnemonic(mnemonic)) {
      throw new Error('无效的助记词');
    }

    this.mnemonic = mnemonic;

    // 创建主账户
    const pair = this.keyring!.addFromMnemonic(mnemonic, { name: accountName });
    this.pairs.set(pair.address, pair);

    return {
      address: pair.address,
      name: accountName,
      type: 'sr25519',
      isPrimary: true,
      createdAt: Date.now(),
    };
  }

  /**
   * 派生新账户
   */
  async deriveAccount(
    accountName: string,
    derivePath: string = ''
  ): Promise<WalletAccount> {
    await this.ensureInit();

    if (!this.keyring || !this.mnemonic) {
      throw new Error('钱包未初始化');
    }

    // 自动生成派生路径
    const path = derivePath || `//${this.pairs.size}`;
    const fullPath = `${this.mnemonic}${path}`;

    const pair = this.keyring.addFromUri(fullPath, { name: accountName });
    this.pairs.set(pair.address, pair);

    return {
      address: pair.address,
      name: accountName,
      type: 'sr25519',
      isPrimary: false,
      createdAt: Date.now(),
      derivePath: path,
    };
  }

  /**
   * 恢复钱包（从存储的数据）
   */
  async restore(mnemonic: string, accounts: WalletAccount[]): Promise<void> {
    await this.ensureInit();

    this.mnemonic = mnemonic;
    this.pairs.clear();

    for (const account of accounts) {
      const uri = account.derivePath
        ? `${mnemonic}${account.derivePath}`
        : mnemonic;

      const pair = this.keyring!.addFromUri(uri, { name: account.name });
      this.pairs.set(pair.address, pair);
    }
  }

  /**
   * 获取密钥对
   */
  getPair(address: string): KeyringPair | undefined {
    return this.pairs.get(address);
  }

  /**
   * 获取所有地址
   */
  getAddresses(): string[] {
    return Array.from(this.pairs.keys());
  }

  /**
   * 签名消息
   */
  signMessage(address: string, message: Uint8Array): Uint8Array {
    const pair = this.pairs.get(address);
    if (!pair) {
      throw new Error('账户不存在');
    }
    return pair.sign(message);
  }

  /**
   * 签名交易
   */
  async signTransaction(
    address: string,
    tx: any // SubmittableExtrinsic
  ): Promise<string> {
    const pair = this.pairs.get(address);
    if (!pair) {
      throw new Error('账户不存在');
    }

    const signed = await tx.signAsync(pair);
    return signed.toHex();
  }

  /**
   * 获取助记词（用于备份）
   */
  getMnemonic(): string | null {
    return this.mnemonic;
  }

  /**
   * 锁定钱包（清除内存中的密钥）
   */
  lock(): void {
    this.pairs.clear();
    this.mnemonic = null;
  }

  /**
   * 检查是否已解锁
   */
  isUnlocked(): boolean {
    return this.mnemonic !== null && this.pairs.size > 0;
  }

  /**
   * 导出账户（JSON 格式）
   */
  exportAccount(address: string, password: string): string {
    const pair = this.pairs.get(address);
    if (!pair) {
      throw new Error('账户不存在');
    }
    return JSON.stringify(pair.toJson(password));
  }

  /**
   * 导入账户（JSON 格式）
   */
  async importAccount(json: string, password: string): Promise<WalletAccount> {
    await this.ensureInit();

    const pair = this.keyring!.addFromJson(JSON.parse(json));
    pair.unlock(password);
    this.pairs.set(pair.address, pair);

    return {
      address: pair.address,
      name: (pair.meta.name as string) || '导入账户',
      type: pair.type as 'sr25519' | 'ed25519',
      isPrimary: false,
      createdAt: Date.now(),
    };
  }

  /**
   * 从助记词派生地址（不保存到内存）
   */
  async deriveAddressFromMnemonic(mnemonic: string): Promise<string> {
    await this.ensureInit();

    if (!this.validateMnemonic(mnemonic)) {
      throw new Error('无效的助记词');
    }

    const pair = this.keyring!.createFromUri(mnemonic);
    return pair.address;
  }
}

// 单例
export const keyringService = new KeyringService();
