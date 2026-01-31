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
      // 转换为链上精度：1 COS = 1e12 最小单位（12位小数）
      const amountInWei = BigInt(Math.floor(parseFloat(amount) * 1e12)).toString();
      const tx = api.tx.balances.transferKeepAlive(to, amountInWei);
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

  async createBaziChart(
    mnemonic: string,
    params: {
      name?: string;
      input: 
        | { Solar: { year: number; month: number; day: number; hour: number; minute: number } }
        | { Lunar: { year: number; month: number; day: number; hour: number; minute: number; is_leap_month: boolean } }
        | { SiZhu: { year_gan: number; year_zhi: number; month_gan: number; month_zhi: number; day_gan: number; day_zhi: number; hour_gan: number; hour_zhi: number } };
      gender: 'Male' | 'Female';
      zishi_mode: 'Modern' | 'Traditional';
      longitude?: number;
    },
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).bazi.createBaziChart(
        params.name || null,
        params.input,
        { [params.gender]: null },
        { [params.zishi_mode]: null },
        params.longitude ?? null
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

  async createQimenChart(
    mnemonic: string,
    params: {
      solar_year: number;
      solar_month: number;
      solar_day: number;
      hour: number;
      question_hash: number[];
      is_public: boolean;
      name?: string;
      gender?: number;
      birth_year?: number;
      question?: string;
      question_type?: number;
      pan_method: number;
    },
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).qimen.divineBySolarTime(
        params.solar_year,
        params.solar_month,
        params.solar_day,
        params.hour,
        params.question_hash,
        params.is_public,
        params.name || null,
        params.gender ?? null,
        params.birth_year ?? null,
        params.question || null,
        params.question_type ?? null,
        params.pan_method
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

  // ==================== 群聊相关 ====================

  async createGroup(
    mnemonic: string,
    params: {
      name: string;
      description?: string;
      encryption_mode: number;
      is_public: boolean;
    },
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).chatGroup.createGroup(
        params.name,
        params.description || null,
        params.encryption_mode,
        params.is_public
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

  async sendGroupMessage(
    mnemonic: string,
    groupId: number,
    content: string,
    messageType: number = 0,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).chatGroup.sendGroupMessage(
        groupId,
        content,
        messageType
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

  async joinGroup(
    mnemonic: string,
    groupId: number,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).chatGroup.joinGroup(groupId);
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async leaveGroup(
    mnemonic: string,
    groupId: number,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).chatGroup.leaveGroup(groupId);
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async disbandGroup(
    mnemonic: string,
    groupId: number,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).chatGroup.disbandGroup(groupId);
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  // ========================================
  // 做市商相关交易
  // ========================================

  async lockMakerDeposit(
    mnemonic: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingMaker.lockDeposit();
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async submitMakerInfo(
    mnemonic: string,
    realName: string,
    idCardNumber: string,
    birthday: string,
    tronAddress: string,
    wechatId: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingMaker.submitInfo(
        realName,
        idCardNumber,
        birthday,
        tronAddress,
        wechatId
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

  async cancelMaker(
    mnemonic: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingMaker.cancelMaker();
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async requestMakerWithdrawal(
    mnemonic: string,
    amount: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingMaker.requestWithdrawal(amount);
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async executeMakerWithdrawal(
    mnemonic: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingMaker.executeWithdrawal();
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async cancelMakerWithdrawal(
    mnemonic: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingMaker.cancelWithdrawal();
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async replenishMakerDeposit(
    mnemonic: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingMaker.replenishDeposit();
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async appealMakerPenalty(
    mnemonic: string,
    penaltyId: number,
    evidenceCid: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingMaker.appealPenalty(penaltyId, evidenceCid);
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  // ========================================
  // OTC 订单相关交易
  // ========================================

  async createOtcOrderNew(
    mnemonic: string,
    makerId: number,
    cosAmount: string,
    paymentCommit: string,
    contactCommit: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingOtc.createOrder(
        makerId,
        cosAmount,
        paymentCommit,
        contactCommit
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

  async createFirstPurchase(
    mnemonic: string,
    makerId: number,
    paymentCommit: string,
    contactCommit: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingOtc.createFirstPurchase(
        makerId,
        paymentCommit,
        contactCommit
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

  async markOtcPaid(
    mnemonic: string,
    orderId: number,
    tronTxHash?: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const hashParam = tronTxHash ? tronTxHash : null;
      const tx = (api.tx as any).tradingOtc.markPaid(orderId, hashParam);
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async releaseOtcCos(
    mnemonic: string,
    orderId: number,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingOtc.releaseCos(orderId);
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async cancelOtcOrder(
    mnemonic: string,
    orderId: number,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingOtc.cancelOrder(orderId);
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async disputeOtcOrder(
    mnemonic: string,
    orderId: number,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingOtc.disputeOrder(orderId);
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async initiateOtcDispute(
    mnemonic: string,
    orderId: number,
    evidenceCid: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingOtc.initiateDispute(orderId, evidenceCid);
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async respondOtcDispute(
    mnemonic: string,
    orderId: number,
    evidenceCid: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingOtc.respondDispute(orderId, evidenceCid);
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  // ========================================
  // Swap 兑换相关交易
  // ========================================

  async createSwap(
    mnemonic: string,
    makerId: number,
    cosAmount: string,
    usdtAddress: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingSwap.makerSwap(
        makerId,
        cosAmount,
        usdtAddress
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

  async markSwapComplete(
    mnemonic: string,
    swapId: number,
    trc20TxHash: string,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingSwap.markSwapComplete(swapId, trc20TxHash);
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async reportSwap(
    mnemonic: string,
    swapId: number,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingSwap.reportSwap(swapId);
      return await this.signAndSend(tx, keyPair, callbacks);
    } catch (error: any) {
      return {
        success: false,
        status: 'failed',
        error: error.message,
      };
    }
  }

  async handleSwapVerificationTimeout(
    mnemonic: string,
    swapId: number,
    callbacks?: TxCallbacks
  ): Promise<TxResult> {
    try {
      const api = this.getApi();
      const keyPair = await this.getKeyPair(mnemonic);
      
      const tx = (api.tx as any).tradingSwap.handleVerificationTimeout(swapId);
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
