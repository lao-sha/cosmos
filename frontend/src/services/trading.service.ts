/**
 * 星尘玄鉴 - Trading 服务层
 * 封装与区块链交互的 API
 */

import { ApiPromise } from '@polkadot/api';
import CryptoJS from 'crypto-js';
import { getApi } from '@/lib/api';
import { signAndSend, getCurrentSignerAddress } from '@/lib/signer';
import type { Maker, Order, MarketStats, BuyerCreditInfo, KycStatus } from '@/stores/trading.store';

/**
 * Trading Service
 * 提供与 pallet-trading 交互的方法
 */
export class TradingService {
  /**
   * 获取 API 实例
   */
  private getApi(): ApiPromise {
    try {
      return getApi();
    } catch (error) {
      throw new Error('API not initialized. Please initialize API first.');
    }
  }

  // ===== 做市商相关 =====

  /**
   * 获取所有活跃做市商
   */
  async getMakers(): Promise<Maker[]> {
    const api = this.getApi();

    try {
      // 查询所有做市商申请
      const entries = await api.query.maker.makerApplications.entries();

      const makers: Maker[] = [];

      for (const [key, value] of entries) {
        const makerId = key.args[0].toNumber();
        const app = value.unwrap();

        // 只返回已激活且未暂停的做市商
        if (app.status.isActive && !app.servicePaused.isTrue) {
          makers.push({
            id: makerId,
            owner: app.owner.toString(),
            tronAddress: app.tronAddress.toHuman() as string,
            buyPremiumBps: app.buyPremiumBps.toNumber(),
            sellPremiumBps: app.sellPremiumBps.toNumber(),
            minAmount: app.minAmount.toBigInt(),
            servicePaused: app.servicePaused.isTrue,
            usersServed: app.usersServed.toNumber(),
            maskedFullName: app.maskedFullName.toHuman() as string,
            wechatId: app.wechatId.toHuman() as string,
            rating: this.calculateMakerRating(app.usersServed.toNumber()),
          });
        }
      }

      // 按评分排序
      makers.sort((a, b) => b.rating - a.rating);

      return makers;
    } catch (error) {
      console.error('[TradingService] Get makers error:', error);
      throw error;
    }
  }

  /**
   * 获取做市商详情
   */
  async getMaker(makerId: number): Promise<Maker | null> {
    const api = this.getApi();

    try {
      const app = await api.query.maker.makerApplications(makerId);

      if (app.isNone) {
        return null;
      }

      const data = app.unwrap();

      return {
        id: makerId,
        owner: data.owner.toString(),
        tronAddress: data.tronAddress.toHuman() as string,
        buyPremiumBps: data.buyPremiumBps.toNumber(),
        sellPremiumBps: data.sellPremiumBps.toNumber(),
        minAmount: data.minAmount.toBigInt(),
        servicePaused: data.servicePaused.isTrue,
        usersServed: data.usersServed.toNumber(),
        maskedFullName: data.maskedFullName.toHuman() as string,
        wechatId: data.wechatId.toHuman() as string,
        rating: this.calculateMakerRating(data.usersServed.toNumber()),
      };
    } catch (error) {
      console.error('[TradingService] Get maker error:', error);
      throw error;
    }
  }

  /**
   * 计算做市商评分
   */
  private calculateMakerRating(usersServed: number): number {
    // 简单的评分算法：基于服务用户数
    if (usersServed >= 1000) return 5.0;
    if (usersServed >= 500) return 4.9;
    if (usersServed >= 200) return 4.8;
    if (usersServed >= 100) return 4.7;
    if (usersServed >= 50) return 4.6;
    return 4.5;
  }

  // ===== 订单相关 =====

  /**
   * 创建首购订单
   */
  async createFirstPurchase(
    accountAddress: string,
    makerId: number,
    paymentCommit: string,
    contactCommit: string,
    onStatusChange?: (status: string) => void
  ): Promise<number> {
    const api = this.getApi();

    try {
      const tx = api.tx.otcOrder.createFirstPurchase(
        makerId,
        paymentCommit,
        contactCommit
      );

      const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

      // 解析事件获取订单 ID
      for (const { event } of events) {
        if (api.events.otcOrder.FirstPurchaseCreated.is(event)) {
          const [orderId] = event.data;
          console.log('[Trading] First purchase created, order ID:', orderId.toString());
          return orderId.toNumber();
        }
      }

      throw new Error('Order ID not found in events');
    } catch (error) {
      console.error('[Trading] Create first purchase error:', error);
      throw error;
    }
  }

  /**
   * 创建普通订单
   */
  async createOrder(
    accountAddress: string,
    makerId: number,
    dustAmount: bigint,
    paymentCommit: string,
    contactCommit: string,
    onStatusChange?: (status: string) => void
  ): Promise<number> {
    const api = this.getApi();

    try {
      const tx = api.tx.otcOrder.createOrder(
        makerId,
        dustAmount.toString(),
        paymentCommit,
        contactCommit
      );

      const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

      // 解析事件获取订单 ID
      for (const { event } of events) {
        if (api.events.otcOrder.OrderCreated.is(event)) {
          const [orderId] = event.data;
          console.log('[Trading] Order created, order ID:', orderId.toString());
          return orderId.toNumber();
        }
      }

      throw new Error('Order ID not found in events');
    } catch (error) {
      console.error('[Trading] Create order error:', error);
      throw error;
    }
  }

  /**
   * 标记已付款
   */
  async markPaid(
    accountAddress: string,
    orderId: number,
    tronTxHash?: string,
    onStatusChange?: (status: string) => void
  ): Promise<void> {
    const api = this.getApi();

    try {
      const tx = api.tx.otcOrder.markPaid(orderId, tronTxHash || null);
      await signAndSend(api, tx, accountAddress, onStatusChange);
      console.log('[Trading] Mark paid success');
    } catch (error) {
      console.error('[Trading] Mark paid error:', error);
      throw error;
    }
  }

  /**
   * 取消订单
   */
  async cancelOrder(
    accountAddress: string,
    orderId: number,
    onStatusChange?: (status: string) => void
  ): Promise<void> {
    const api = this.getApi();

    try {
      const tx = api.tx.otcOrder.cancelOrder(orderId);
      await signAndSend(api, tx, accountAddress, onStatusChange);
      console.log('[Trading] Cancel order success');
    } catch (error) {
      console.error('[Trading] Cancel order error:', error);
      throw error;
    }
  }

  /**
   * 申请仲裁
   */
  async dispute(
    accountAddress: string,
    orderId: number,
    reason: string,
    evidenceCid?: string,
    onStatusChange?: (status: string) => void
  ): Promise<void> {
    const api = this.getApi();

    try {
      // 生成证据哈希（如果有证据）
      const evidenceHash = evidenceCid
        ? '0x' + CryptoJS.SHA256(evidenceCid).toString()
        : null;

      const tx = api.tx.otcOrder.dispute(orderId, evidenceHash);
      await signAndSend(api, tx, accountAddress, onStatusChange);
      console.log('[Trading] Dispute submitted:', orderId);
    } catch (error) {
      console.error('[Trading] Dispute error:', error);
      throw error;
    }
  }

  /**
   * 获取订单详情
   */
  async getOrder(orderId: number): Promise<Order | null> {
    const api = this.getApi();

    try {
      const order = await api.query.otcOrder.orders(orderId);

      if (order.isNone) {
        return null;
      }

      const data = order.unwrap();

      return {
        id: orderId,
        makerId: data.makerId.toNumber(),
        maker: data.maker.toString(),
        taker: data.taker.toString(),
        price: data.price.toBigInt(),
        qty: data.qty.toBigInt(),
        amount: data.amount.toBigInt(),
        createdAt: data.createdAt.toNumber(),
        expireAt: data.expireAt.toNumber(),
        makerTronAddress: data.makerTronAddress.toHuman() as string,
        state: data.state.toString() as any,
        isFirstPurchase: data.isFirstPurchase.isTrue,
      };
    } catch (error) {
      console.error('[TradingService] Get order error:', error);
      throw error;
    }
  }

  /**
   * 订阅订单状态
   */
  async subscribeToOrder(
    orderId: number,
    callback: (order: Order) => void
  ): Promise<() => void> {
    const api = this.getApi();

    const unsub = await api.query.otcOrder.orders(orderId, (order) => {
      if (order.isSome) {
        const data = order.unwrap();
        callback({
          id: orderId,
          makerId: data.makerId.toNumber(),
          maker: data.maker.toString(),
          taker: data.taker.toString(),
          price: data.price.toBigInt(),
          qty: data.qty.toBigInt(),
          amount: data.amount.toBigInt(),
          createdAt: data.createdAt.toNumber(),
          expireAt: data.expireAt.toNumber(),
          makerTronAddress: data.makerTronAddress.toHuman() as string,
          state: data.state.toString() as any,
          isFirstPurchase: data.isFirstPurchase.isTrue,
        });
      }
    });

    return unsub;
  }

  /**
   * 获取买家订单历史
   */
  async getOrderHistory(buyer: string): Promise<Order[]> {
    const api = this.getApi();

    try {
      const orderIds = await api.query.otcOrder.buyerOrders(buyer);
      const orders: Order[] = [];

      for (const orderId of orderIds) {
        const order = await this.getOrder(orderId.toNumber());
        if (order) {
          orders.push(order);
        }
      }

      // 按创建时间倒序排序
      orders.sort((a, b) => b.createdAt - a.createdAt);

      return orders;
    } catch (error) {
      console.error('[TradingService] Get order history error:', error);
      throw error;
    }
  }

  // ===== 价格相关 =====

  /**
   * 获取市场统计
   */
  async getMarketStats(): Promise<MarketStats> {
    const api = this.getApi();

    try {
      // 尝试获取市场统计
      const stats = await api.query.tradingPricing.marketStats();
      
      // 获取默认价格（用于冷启动期间）
      const defaultPrice = await api.query.tradingPricing.defaultPrice();
      const defaultPriceValue = defaultPrice.toNumber();
      
      // 检查是否在冷启动期间
      const coldStartExited = await api.query.tradingPricing.coldStartExited();
      
      let weightedPrice: number;
      
      if (!coldStartExited.isTrue) {
        // 冷启动期间，使用默认价格或配置的初始价格
        // 默认价格精度是 10^6，所以 100000 = 0.1 USDT
        weightedPrice = defaultPriceValue > 1 ? defaultPriceValue / 1_000_000 : 0.1;
      } else {
        weightedPrice = stats.weightedPrice.toNumber() / 1_000_000;
      }

      return {
        otcPrice: stats.otcPrice.toNumber() / 1_000_000 || weightedPrice,
        bridgePrice: stats.bridgePrice.toNumber() / 1_000_000 || weightedPrice,
        weightedPrice: weightedPrice,
        simpleAvgPrice: stats.simpleAvgPrice.toNumber() / 1_000_000 || weightedPrice,
        otcVolume: stats.otcVolume.toBigInt(),
        bridgeVolume: stats.bridgeVolume.toBigInt(),
        totalVolume: stats.totalVolume.toBigInt(),
      };
    } catch (error) {
      console.error('[TradingService] Get market stats error:', error);
      // 返回默认值
      return {
        otcPrice: 0.1,
        bridgePrice: 0.1,
        weightedPrice: 0.1,
        simpleAvgPrice: 0.1,
        otcVolume: BigInt(0),
        bridgeVolume: BigInt(0),
        totalVolume: BigInt(0),
      };
    }
  }

  /**
   * 获取 DUST 价格
   */
  async getDustPrice(): Promise<number> {
    try {
      const api = this.getApi();
      
      // 尝试获取加权市场价格
      const coldStartExited = await api.query.tradingPricing.coldStartExited();
      
      if (!coldStartExited.isTrue) {
        // 冷启动期间，获取默认价格
        const defaultPrice = await api.query.tradingPricing.defaultPrice();
        const priceValue = defaultPrice.toNumber();
        // 如果默认价格太小（如 1），使用 0.1 作为初始价格
        return priceValue > 1000 ? priceValue / 1_000_000 : 0.1;
      }
      
      const stats = await this.getMarketStats();
      return stats.weightedPrice;
    } catch (error) {
      console.error('[TradingService] Get dust price error:', error);
      return 0.1; // 默认价格
    }
  }

  // ===== 信用相关 =====

  /**
   * 获取买家信用信息
   */
  async getBuyerCredit(buyer: string): Promise<BuyerCreditInfo> {
    const api = this.getApi();

    try {
      const credit = await api.query.credit.buyerCredits(buyer);

      if (credit.isNone) {
        // 新用户默认信用
        return {
          riskScore: 500,
          level: '新用户',
          maxAmount: 10,
          concurrentOrders: 0,
          maxConcurrentOrders: 1,
          completedOrders: 0,
          trend: 'stable',
        };
      }

      const data = credit.unwrap();

      return {
        riskScore: data.riskScore.toNumber(),
        level: this.getCreditLevel(data.riskScore.toNumber()),
        maxAmount: data.maxAmount.toNumber() / 1_000_000,
        concurrentOrders: data.concurrentOrders.toNumber(),
        maxConcurrentOrders: data.maxConcurrentOrders.toNumber(),
        completedOrders: data.completedOrders.toNumber(),
        trend: 'stable', // TODO: 计算趋势
      };
    } catch (error) {
      console.error('[TradingService] Get buyer credit error:', error);
      throw error;
    }
  }

  /**
   * 获取信用等级
   */
  private getCreditLevel(riskScore: number): string {
    if (riskScore <= 200) return '高信任';
    if (riskScore <= 400) return '中信任';
    if (riskScore <= 600) return '低信任';
    return '极低信任';
  }

  /**
   * 检查是否完成首购
   */
  async hasCompletedFirstPurchase(buyer: string): Promise<boolean> {
    const api = this.getApi();

    try {
      const hasCompleted = await api.query.otcOrder.firstPurchaseCompleted(buyer);
      return hasCompleted.isTrue;
    } catch (error) {
      console.error('[TradingService] Check first purchase error:', error);
      return false;
    }
  }

  // ===== KYC 相关 =====

  /**
   * 检查 KYC 状态
   */
  async checkKycStatus(account: string): Promise<{
    status: KycStatus;
    failureReason: string | null;
  }> {
    const api = this.getApi();

    try {
      // 获取 KYC 配置
      const config = await api.query.otcOrder.kycConfig();

      // 如果 KYC 未启用
      if (!config.enabled.isTrue) {
        return {
          status: 'Skipped' as KycStatus,
          failureReason: null,
        };
      }

      // 检查是否为豁免账户
      const isExempt = await api.query.otcOrder.kycExemptAccounts(account);
      if (isExempt.isSome) {
        return {
          status: 'Exempted' as KycStatus,
          failureReason: null,
        };
      }

      // 检查身份认证
      const identity = await api.query.identity.identityOf(account);
      if (identity.isNone) {
        return {
          status: 'Failed' as KycStatus,
          failureReason: 'IdentityNotSet',
        };
      }

      // TODO: 检查认证等级

      return {
        status: 'Passed' as KycStatus,
        failureReason: null,
      };
    } catch (error) {
      console.error('[TradingService] Check KYC error:', error);
      throw error;
    }
  }

  // ===== 工具方法 =====

  /**
   * 生成支付承诺哈希
   */
  static generatePaymentCommit(
    realName: string,
    idCard: string,
    phone: string
  ): string {
    const data = `${realName}|${idCard}|${phone}`;
    return '0x' + CryptoJS.SHA256(data).toString();
  }

  /**
   * 生成联系方式承诺哈希
   */
  static generateContactCommit(wechat: string, phone: string): string {
    const data = `${wechat}|${phone}`;
    return '0x' + CryptoJS.SHA256(data).toString();
  }

  /**
   * 计算预计获得的 DUST 数量
   */
  static calculateDustAmount(
    usdAmount: number,
    dustPrice: number,
    premiumBps: number
  ): bigint {
    // 计算实际价格（含溢价）
    const actualPrice = dustPrice * (1 + premiumBps / 10000);
    // 计算 DUST 数量（精度 10^12）
    const dustAmount = (usdAmount / actualPrice) * 1e12;
    return BigInt(Math.floor(dustAmount));
  }

  /**
   * 格式化 DUST 数量
   */
  static formatDustAmount(amount: bigint): string {
    return (Number(amount) / 1e12).toFixed(4);
  }

  /**
   * 格式化 USD 金额
   */
  static formatUsdAmount(amount: bigint): string {
    return (Number(amount) / 1e6).toFixed(2);
  }
}

// 导出单例
export const tradingService = new TradingService();
