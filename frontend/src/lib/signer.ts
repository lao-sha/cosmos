/**
 * 星尘玄鉴 - 统一签名接口
 * 只支持 React Native 原生平台
 */

import { ApiPromise } from '@polkadot/api';

// 直接使用移动端的签名方法
import {
  unlockWallet,
  isWalletUnlocked,
  lockWallet,
  signAndSendTransaction,
  getSignerAddress,
} from './signer.native';

/**
 * 签名结果
 */
export interface SignResult {
  blockHash: string;
  events: any[];
}

/**
 * 签名状态回调
 */
export type StatusCallback = (status: string) => void;

/**
 * 统一签名服务（仅 React Native）
 */
class UnifiedSigner {
  private static instance: UnifiedSigner;

  private constructor() {}

  static getInstance(): UnifiedSigner {
    if (!UnifiedSigner.instance) {
      UnifiedSigner.instance = new UnifiedSigner();
    }
    return UnifiedSigner.instance;
  }

  /**
   * 解锁钱包
   */
  async unlock(password: string): Promise<void> {
    await unlockWallet(password);
  }

  /**
   * 检查钱包是否已解锁
   */
  isUnlocked(): boolean {
    return isWalletUnlocked();
  }

  /**
   * 锁定钱包
   */
  lock(): void {
    lockWallet();
  }

  /**
   * 签名并发送交易
   */
  async signAndSend(
    api: ApiPromise,
    tx: any,
    accountAddress: string,
    onStatusChange?: StatusCallback
  ): Promise<SignResult> {
    if (!isWalletUnlocked()) {
      throw new Error('钱包已锁定，请先解锁');
    }

    return signAndSendTransaction(api, tx, onStatusChange);
  }

  /**
   * 获取当前账户地址
   */
  async getCurrentAddress(): Promise<string | null> {
    return getSignerAddress();
  }
}

// 导出单例
export const unifiedSigner = UnifiedSigner.getInstance();

// 导出便捷方法
export const unlockWalletForSigning = (password: string) => unifiedSigner.unlock(password);
export const isSignerUnlocked = () => unifiedSigner.isUnlocked();
export const lockSigner = () => unifiedSigner.lock();
export const signAndSend = (
  api: ApiPromise,
  tx: any,
  accountAddress: string,
  onStatusChange?: StatusCallback
) => unifiedSigner.signAndSend(api, tx, accountAddress, onStatusChange);
export const getCurrentSignerAddress = () => unifiedSigner.getCurrentAddress();
