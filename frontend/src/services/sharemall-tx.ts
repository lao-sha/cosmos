// ShareMall 交易服务 - 链上交易操作

import { useChainStore } from '@/src/stores/chain';
import { useWalletStore } from '@/src/stores/wallet';
import { WalletService } from '@/src/lib/wallet';
import { ApiPromise } from '@polkadot/api';

type AnyCodec = any;

export class ShareMallTxService {
  private getApi(): ApiPromise {
    const { api } = useChainStore.getState();
    if (!api) {
      throw new Error('Chain not connected');
    }
    return api;
  }

  private async signAndSend(tx: any): Promise<string> {
    const { currentAccount } = useWalletStore.getState();
    if (!currentAccount) {
      throw new Error('No account selected');
    }

    // 从钱包服务获取密钥对
    const mnemonic = await WalletService.getMnemonic();
    if (!mnemonic) {
      throw new Error('Wallet not initialized');
    }

    const pair = currentAccount.isImported 
      ? await WalletService.getAccountFromMnemonic(mnemonic)
      : await WalletService.deriveAccount(mnemonic, currentAccount.derivationIndex);

    return new Promise((resolve, reject) => {
      tx.signAndSend(pair, ({ status, dispatchError }: any) => {
        if (status.isInBlock) {
          if (dispatchError) {
            reject(new Error(dispatchError.toString()));
          } else {
            resolve(status.asInBlock.toString());
          }
        }
      }).catch(reject);
    });
  }

  // ==================== 店铺操作 ====================

  async createShop(
    name: string,
    logoCid?: string,
    descriptionCid?: string,
    customerService?: string
  ): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallShop.createShop(
      name,
      logoCid || null,
      descriptionCid || null,
      customerService || null
    );
    return this.signAndSend(tx);
  }

  async updateShop(
    shopId: number,
    name?: string,
    logoCid?: string,
    descriptionCid?: string
  ): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallShop.updateShop(
      shopId,
      name || null,
      logoCid || null,
      descriptionCid || null
    );
    return this.signAndSend(tx);
  }

  async depositShopFund(shopId: number, amount: string): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallShop.depositFund(shopId, amount);
    return this.signAndSend(tx);
  }

  async suspendShop(shopId: number): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallShop.suspendShop(shopId);
    return this.signAndSend(tx);
  }

  async resumeShop(shopId: number): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallShop.resumeShop(shopId);
    return this.signAndSend(tx);
  }

  // ==================== 商品操作 ====================

  async createProduct(
    shopId: number,
    nameCid: string,
    imagesCid: string,
    detailCid: string,
    price: string,
    stock: number,
    category: string
  ): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallProduct.createProduct(
      shopId,
      nameCid,
      imagesCid,
      detailCid,
      price,
      stock,
      category
    );
    return this.signAndSend(tx);
  }

  async updateProduct(
    productId: number,
    nameCid?: string,
    imagesCid?: string,
    detailCid?: string,
    price?: string,
    stock?: number
  ): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallProduct.updateProduct(
      productId,
      nameCid || null,
      imagesCid || null,
      detailCid || null,
      price || null,
      stock || null
    );
    return this.signAndSend(tx);
  }

  async publishProduct(productId: number): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallProduct.publishProduct(productId);
    return this.signAndSend(tx);
  }

  async unpublishProduct(productId: number): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallProduct.unpublishProduct(productId);
    return this.signAndSend(tx);
  }

  async deleteProduct(productId: number): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallProduct.deleteProduct(productId);
    return this.signAndSend(tx);
  }

  // ==================== 订单操作 ====================

  async placeOrder(
    productId: number,
    quantity: number,
    shippingCid?: string,
    usePoints: boolean = false,
    pointsAmount: string = '0',
    referrer?: string
  ): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallOrder.placeOrder(
      productId,
      quantity,
      shippingCid || null,
      usePoints,
      pointsAmount,
      referrer || null
    );
    return this.signAndSend(tx);
  }

  async cancelOrder(orderId: number): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallOrder.cancelOrder(orderId);
    return this.signAndSend(tx);
  }

  async shipOrder(orderId: number, trackingCid: string): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallOrder.shipOrder(orderId, trackingCid);
    return this.signAndSend(tx);
  }

  async confirmReceipt(orderId: number): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallOrder.confirmReceipt(orderId);
    return this.signAndSend(tx);
  }

  async requestRefund(orderId: number, reasonCid: string): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallOrder.requestRefund(orderId, reasonCid);
    return this.signAndSend(tx);
  }

  async approveRefund(orderId: number): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallOrder.approveRefund(orderId);
    return this.signAndSend(tx);
  }

  // ==================== 评价操作 ====================

  async submitReview(orderId: number, rating: number, contentCid?: string): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallReview.submitReview(orderId, rating, contentCid || null);
    return this.signAndSend(tx);
  }

  // ==================== 代币操作 ====================

  async enableToken(
    shopId: number,
    rewardRate: number,
    exchangeRate: number,
    minRedeem: string,
    maxRedeemPerOrder: string,
    transferable: boolean
  ): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallToken.enableToken(
      shopId,
      rewardRate,
      exchangeRate,
      minRedeem,
      maxRedeemPerOrder,
      transferable
    );
    return this.signAndSend(tx);
  }

  async transferToken(shopId: number, to: string, amount: string): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallToken.transfer(shopId, to, amount);
    return this.signAndSend(tx);
  }

  // ==================== P2P 市场操作 ====================

  async placeSellOrder(
    shopId: number,
    tokenAmount: string,
    price: string
  ): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallMarket.placeSellOrder(shopId, tokenAmount, price);
    return this.signAndSend(tx);
  }

  async placeBuyOrder(
    shopId: number,
    tokenAmount: string,
    price: string
  ): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallMarket.placeBuyOrder(shopId, tokenAmount, price);
    return this.signAndSend(tx);
  }

  async takeOrder(orderId: number, amount: string): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallMarket.takeOrder(orderId, amount);
    return this.signAndSend(tx);
  }

  async marketBuy(
    shopId: number,
    maxCost: string,
    minReceive: string
  ): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallMarket.marketBuy(shopId, maxCost, minReceive);
    return this.signAndSend(tx);
  }

  async marketSell(
    shopId: number,
    tokenAmount: string,
    minReceive: string
  ): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallMarket.marketSell(shopId, tokenAmount, minReceive);
    return this.signAndSend(tx);
  }

  async cancelTradeOrder(orderId: number): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallMarket.cancelOrder(orderId);
    return this.signAndSend(tx);
  }

  async setInitialPrice(shopId: number, initialPrice: string): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallMarket.setInitialPrice(shopId, initialPrice);
    return this.signAndSend(tx);
  }

  // ==================== USDT 通道 ====================

  async placeUsdtSellOrder(
    shopId: number,
    tokenAmount: string,
    usdtPrice: number,
    tronAddress: string
  ): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallMarket.placeUsdtSellOrder(
      shopId,
      tokenAmount,
      usdtPrice,
      tronAddress
    );
    return this.signAndSend(tx);
  }

  async takeUsdtSellOrder(orderId: number, amount: string): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallMarket.takeUsdtSellOrder(orderId, amount);
    return this.signAndSend(tx);
  }

  async submitUsdtPayment(tradeId: number, txHash: string): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallMarket.submitUsdtPayment(tradeId, txHash);
    return this.signAndSend(tx);
  }

  // ==================== 会员操作 ====================

  async joinMember(shopId: number, referrer?: string): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallMember.join(shopId, referrer || null);
    return this.signAndSend(tx);
  }

  // ==================== 治理操作 ====================

  async createProposal(
    shopId: number,
    titleCid: string,
    descriptionCid: string
  ): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallGovernance.createProposal(shopId, titleCid, descriptionCid);
    return this.signAndSend(tx);
  }

  async vote(proposalId: number, support: boolean, amount: string): Promise<string> {
    const api = this.getApi();
    const tx = api.tx.sharemallGovernance.vote(proposalId, support, amount);
    return this.signAndSend(tx);
  }
}

export const sharemallTxService = new ShareMallTxService();
