import { WalletService } from '@/src/lib/wallet';
import { useChainStore } from '@/src/stores/chain';
import { ApiPromise } from '@polkadot/api';
import { SubmittableExtrinsic } from '@polkadot/api/types';
import { KeyringPair } from '@polkadot/keyring/types';

export type TxStatus = 'pending' | 'signing' | 'broadcasting' | 'inBlock' | 'finalized' | 'failed';

export interface TxResult {
  success: boolean;
  status: TxStatus;
  blockHash?: string;
  txHash?: string;
  error?: string;
  events?: any[];
}

export interface TxCallbacks {
  onStatusChange?: (status: TxStatus) => void;
  onSuccess?: (result: TxResult) => void;
  onError?: (error: string) => void;
}

class TransactionService {
  private getApi(): ApiPromise {
    const { api } = useChainStore.getState();
    if (!api) {
      throw new Error('Chain not connected');
    }
    return api;
  }

  async getKeyPair(mnemonic: string): Promise<KeyringPair> {
    return await WalletService.getAccountFromMnemonic(mnemonic);
  }

  async signAndSend(
    tx: SubmittableExtrinsic<'promise'>,
    keyPair: KeyringPair,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    return new Promise((resolve) => {
      callbacks?.onStatusChange?.('signing');

      tx.signAndSend(keyPair, ({ status, events, dispatchError }) => {
        if (status.isReady) {
          callbacks?.onStatusChange?.('broadcasting');
        }

        if (status.isInBlock) {
          callbacks?.onStatusChange?.('inBlock');
          
          let hasError = false;
          let errorMessage = '';

          if (dispatchError) {
            hasError = true;
            if (dispatchError.isModule) {
              const api = this.getApi();
              const decoded = api.registry.findMetaError(dispatchError.asModule);
              errorMessage = `${decoded.section}.${decoded.name}: ${decoded.docs.join(' ')}`;
            } else {
              errorMessage = dispatchError.toString();
            }
          }

          events.forEach(({ event }) => {
            if (event.section === 'system' && event.method === 'ExtrinsicFailed') {
              hasError = true;
            }
          });

          if (hasError) {
            const result: TxResult = {
              success: false,
              status: 'failed',
              blockHash: status.asInBlock.toHex(),
              error: errorMessage || 'Transaction failed',
              events: events.map(e => e.toHuman()),
            };
            callbacks?.onError?.(result.error!);
            resolve(result);
          }
        }

        if (status.isFinalized) {
          callbacks?.onStatusChange?.('finalized');
          
          const result: TxResult = {
            success: true,
            status: 'finalized',
            blockHash: status.asFinalized.toHex(),
            txHash: tx.hash.toHex(),
            events: events.map(e => e.toHuman()),
          };
          callbacks?.onSuccess?.(result);
          resolve(result);
        }

        if (status.isDropped || status.isInvalid || status.isUsurped) {
          const result: TxResult = {
            success: false,
            status: 'failed',
            error: `Transaction ${status.type}`,
          };
          callbacks?.onError?.(result.error!);
          resolve(result);
        }
      }).catch((error) => {
        const result: TxResult = {
          success: false,
          status: 'failed',
          error: error.message || 'Unknown error',
        };
        callbacks?.onError?.(result.error!);
        resolve(result);
      });
    });
  }

  async transfer(
    mnemonic: string,
    to: string,
    amount: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      const tx = api.tx.balances.transferKeepAlive(to, amount);
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async sendChatMessage(
    mnemonic: string,
    receiver: string,
    contentCid: string,
    msgType: number = 0,
    sessionId?: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      const tx = (api.tx as any).chatCore.sendMessage(
        receiver,
        contentCid,
        msgType,
        sessionId || null
      );
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async createOtcOrder(
    mnemonic: string,
    orderType: 'Buy' | 'Sell',
    amount: string,
    price: string,
    currency: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      const tx = (api.tx as any).tradingOtc.createOrder(
        { [orderType]: null },
        amount,
        price,
        currency
      );
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async lockOtcOrder(
    mnemonic: string,
    orderId: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      const tx = (api.tx as any).tradingOtc.lockOrder(orderId);
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async createMatchmakingProfile(
    mnemonic: string,
    nickname: string,
    gender: 'Male' | 'Female',
    birthInfo?: { year: number; month: number; day: number; hour: number },
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      const tx = (api.tx as any).matchmakingProfile.createProfile(
        nickname,
        { [gender]: null },
        birthInfo || null
      );
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async createDivinationOrder(
    mnemonic: string,
    providerId: string,
    packageId: string,
    questionCid: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      const tx = (api.tx as any).divinationMarket.createOrder(
        providerId,
        packageId,
        questionCid
      );
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }
}

export const transactionService = new TransactionService();
