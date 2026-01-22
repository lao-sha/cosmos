/**
 * 星尘玄鉴 - 移动端签名服务
 * 使用内置钱包进行交易签名
 */

import { ApiPromise } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { KeyringPair } from '@polkadot/keyring/types';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { retrieveEncryptedMnemonic, getCurrentAddress } from '@/lib/keystore';

/**
 * 移动端签名器
 */
class MobileSigner {
  private static instance: MobileSigner;
  private keyring: Keyring | null = null;
  private currentPair: KeyringPair | null = null;
  private isInitialized = false;

  private constructor() {}

  /**
   * 获取单例实例
   */
  static getInstance(): MobileSigner {
    if (!MobileSigner.instance) {
      MobileSigner.instance = new MobileSigner();
    }
    return MobileSigner.instance;
  }

  /**
   * 初始化签名器
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      return;
    }

    try {
      console.log('[MobileSigner] Initializing...');

      // 等待加密库准备就绪
      await cryptoWaitReady();

      // 创建 Keyring
      this.keyring = new Keyring({ type: 'sr25519' });

      this.isInitialized = true;
      console.log('[MobileSigner] Initialized');
    } catch (error) {
      console.error('[MobileSigner] Initialize error:', error);
      throw error;
    }
  }

  /**
   * 解锁钱包并加载密钥对
   */
  async unlockWallet(password: string): Promise<KeyringPair> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    if (!this.keyring) {
      throw new Error('Keyring not initialized');
    }

    try {
      console.log('[MobileSigner] Unlocking wallet...');

      // 获取当前地址
      const address = await getCurrentAddress();
      if (!address) {
        throw new Error('No wallet found');
      }

      // 解密助记词
      const mnemonic = await retrieveEncryptedMnemonic(password);

      // 从助记词创建密钥对
      this.currentPair = this.keyring.addFromMnemonic(mnemonic);

      console.log('[MobileSigner] Wallet unlocked:', this.currentPair.address);

      return this.currentPair;
    } catch (error) {
      console.error('[MobileSigner] Unlock error:', error);
      throw new Error('Failed to unlock wallet. Please check your password.');
    }
  }

  /**
   * 获取当前密钥对
   */
  getCurrentPair(): KeyringPair | null {
    return this.currentPair;
  }

  /**
   * 检查是否已解锁
   */
  isUnlocked(): boolean {
    return this.currentPair !== null;
  }

  /**
   * 锁定钱包
   */
  lock(): void {
    this.currentPair = null;
    console.log('[MobileSigner] Wallet locked');
  }

  /**
   * 签名并发送交易
   */
  async signAndSend(
    api: ApiPromise,
    tx: any,
    onStatusChange?: (status: string) => void
  ): Promise<{ blockHash: string; events: any[] }> {
    if (!this.currentPair) {
      throw new Error('Wallet is locked. Please unlock first.');
    }

    return new Promise((resolve, reject) => {
      tx.signAndSend(
        this.currentPair,
        ({ status, events, dispatchError }: any) => {
          // 更新状态
          if (status.isReady) {
            onStatusChange?.('准备中...');
          } else if (status.isBroadcast) {
            onStatusChange?.('广播中...');
          } else if (status.isInBlock) {
            onStatusChange?.('已打包...');
            console.log('[MobileSigner] Transaction in block:', status.asInBlock.toHex());

            // 检查错误
            if (dispatchError) {
              if (dispatchError.isModule) {
                const decoded = api.registry.findMetaError(dispatchError.asModule);
                const { docs, name, section } = decoded;
                reject(new Error(`${section}.${name}: ${docs.join(' ')}`));
              } else {
                reject(new Error(dispatchError.toString()));
              }
              return;
            }

            // 交易成功
            resolve({
              blockHash: status.asInBlock.toHex(),
              events,
            });
          } else if (status.isFinalized) {
            onStatusChange?.('已确认');
            console.log('[MobileSigner] Transaction finalized:', status.asFinalized.toHex());
          }
        }
      ).catch(reject);
    });
  }

  /**
   * 获取账户地址
   */
  getAddress(): string | null {
    return this.currentPair?.address || null;
  }
}

// 导出单例
export const mobileSigner = MobileSigner.getInstance();

// 导出便捷方法
export const initializeSigner = () => mobileSigner.initialize();
export const unlockWallet = (password: string) => mobileSigner.unlockWallet(password);
export const isWalletUnlocked = () => mobileSigner.isUnlocked();
export const lockWallet = () => mobileSigner.lock();
export const getCurrentPair = () => mobileSigner.getCurrentPair();
export const getSignerAddress = () => mobileSigner.getAddress();
export const signAndSendTransaction = (
  api: ApiPromise,
  tx: any,
  onStatusChange?: (status: string) => void
) => mobileSigner.signAndSend(api, tx, onStatusChange);
