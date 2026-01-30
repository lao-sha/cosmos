import { useChainStore } from '@/src/stores/chain';
import { ApiPromise } from '@polkadot/api';

export enum OrderState {
  Created = 0,
  PaidOrCommitted = 1,
  Released = 2,
  Refunded = 3,
  Canceled = 4,
  Disputed = 5,
  Closed = 6,
  Expired = 7,
}

export enum DepositStatus {
  None = 'None',
  Locked = 'Locked',
  Released = 'Released',
  Forfeited = 'Forfeited',
  PartiallyForfeited = 'PartiallyForfeited',
}

export enum DisputeStatus {
  WaitingMakerResponse = 'WaitingMakerResponse',
  WaitingArbitration = 'WaitingArbitration',
  BuyerWon = 'BuyerWon',
  MakerWon = 'MakerWon',
  Cancelled = 'Cancelled',
}

export interface OtcOrder {
  orderId: number;
  makerId: number;
  maker: string;
  taker: string;
  price: string;
  qty: string;
  amount: string;
  createdAt: number;
  expireAt: number;
  evidenceUntil: number;
  makerTronAddress: string;
  paymentCommit: string;
  contactCommit: string;
  state: OrderState;
  completedAt: number | null;
  isFirstPurchase: boolean;
  buyerDeposit: string;
  depositStatus: DepositStatus;
}

export interface Dispute {
  orderId: number;
  initiator: string;
  respondent: string;
  createdAt: number;
  responseDeadline: number;
  arbitrationDeadline: number;
  status: DisputeStatus;
  buyerEvidence: string | null;
  makerEvidence: string | null;
}

export interface MakerInfo {
  makerId: number;
  owner: string;
  tronAddress: string;
  buyPremiumBps: number;
  sellPremiumBps: number;
  usersServed: number;
  servicePaused: boolean;
  maskedFullName: string;
  wechatId: string;
}

class OtcService {
  private getApi(): ApiPromise {
    const { api } = useChainStore.getState();
    if (!api) {
      throw new Error('Chain not connected');
    }
    return api;
  }

  private parseOrderState(state: any): OrderState {
    if (state.isCreated) return OrderState.Created;
    if (state.isPaidOrCommitted) return OrderState.PaidOrCommitted;
    if (state.isReleased) return OrderState.Released;
    if (state.isRefunded) return OrderState.Refunded;
    if (state.isCanceled) return OrderState.Canceled;
    if (state.isDisputed) return OrderState.Disputed;
    if (state.isClosed) return OrderState.Closed;
    if (state.isExpired) return OrderState.Expired;
    return OrderState.Created;
  }

  private parseDepositStatus(status: any): DepositStatus {
    if (status.isNone) return DepositStatus.None;
    if (status.isLocked) return DepositStatus.Locked;
    if (status.isReleased) return DepositStatus.Released;
    if (status.isForfeited) return DepositStatus.Forfeited;
    if (status.isPartiallyForfeited) return DepositStatus.PartiallyForfeited;
    return DepositStatus.None;
  }

  private parseDisputeStatus(status: any): DisputeStatus {
    if (status.isWaitingMakerResponse) return DisputeStatus.WaitingMakerResponse;
    if (status.isWaitingArbitration) return DisputeStatus.WaitingArbitration;
    if (status.isBuyerWon) return DisputeStatus.BuyerWon;
    if (status.isMakerWon) return DisputeStatus.MakerWon;
    if (status.isCancelled) return DisputeStatus.Cancelled;
    return DisputeStatus.WaitingMakerResponse;
  }

  async getOrder(orderId: number): Promise<OtcOrder | null> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingOtc.orders(orderId);
      if (result.isSome) {
        const order = result.unwrap();
        return {
          orderId,
          makerId: order.makerId.toNumber(),
          maker: order.maker.toString(),
          taker: order.taker.toString(),
          price: order.price.toString(),
          qty: order.qty.toString(),
          amount: order.amount.toString(),
          createdAt: order.createdAt.toNumber(),
          expireAt: order.expireAt.toNumber(),
          evidenceUntil: order.evidenceUntil.toNumber(),
          makerTronAddress: new TextDecoder().decode(new Uint8Array(order.makerTronAddress)),
          paymentCommit: order.paymentCommit.toHex(),
          contactCommit: order.contactCommit.toHex(),
          state: this.parseOrderState(order.state),
          completedAt: order.completedAt.isSome ? order.completedAt.unwrap().toNumber() : null,
          isFirstPurchase: order.isFirstPurchase.isTrue,
          buyerDeposit: order.buyerDeposit.toString(),
          depositStatus: this.parseDepositStatus(order.depositStatus),
        };
      }
      return null;
    } catch (error) {
      console.error('获取订单失败:', error);
      return null;
    }
  }

  async getBuyerOrders(address: string): Promise<number[]> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingOtc.buyerOrders(address);
      if (result && result.length > 0) {
        return result.map((id: any) => id.toNumber());
      }
      return [];
    } catch (error) {
      console.error('获取买家订单列表失败:', error);
      return [];
    }
  }

  async getMakerOrders(makerId: number): Promise<number[]> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingOtc.makerOrders(makerId);
      if (result && result.length > 0) {
        return result.map((id: any) => id.toNumber());
      }
      return [];
    } catch (error) {
      console.error('获取做市商订单列表失败:', error);
      return [];
    }
  }

  async getDispute(orderId: number): Promise<Dispute | null> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingOtc.disputes(orderId);
      if (result.isSome) {
        const dispute = result.unwrap();
        return {
          orderId,
          initiator: dispute.initiator.toString(),
          respondent: dispute.respondent.toString(),
          createdAt: dispute.createdAt.toNumber(),
          responseDeadline: dispute.responseDeadline.toNumber(),
          arbitrationDeadline: dispute.arbitrationDeadline.toNumber(),
          status: this.parseDisputeStatus(dispute.status),
          buyerEvidence: dispute.buyerEvidence.isSome 
            ? new TextDecoder().decode(new Uint8Array(dispute.buyerEvidence.unwrap()))
            : null,
          makerEvidence: dispute.makerEvidence.isSome
            ? new TextDecoder().decode(new Uint8Array(dispute.makerEvidence.unwrap()))
            : null,
        };
      }
      return null;
    } catch (error) {
      console.error('获取争议记录失败:', error);
      return null;
    }
  }

  async hasFirstPurchased(address: string): Promise<boolean> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingOtc.hasFirstPurchased(address);
      return result.isTrue;
    } catch (error) {
      console.error('检查首购状态失败:', error);
      return false;
    }
  }

  async getNextOrderId(): Promise<number> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingOtc.nextOrderId();
      return result.toNumber();
    } catch (error) {
      console.error('获取下一个订单ID失败:', error);
      return 0;
    }
  }

  async getActiveMakers(): Promise<MakerInfo[]> {
    try {
      const api = this.getApi();
      const nextMakerId = await (api.query as any).tradingMaker.nextMakerId();
      const maxId = nextMakerId.toNumber();
      
      const makers: MakerInfo[] = [];
      for (let i = 0; i < maxId; i++) {
        const result = await (api.query as any).tradingMaker.makerApplications(i);
        if (result.isSome) {
          const app = result.unwrap();
          const isActive = app.status.isActive || app.status.toString() === 'Active';
          const isPaused = app.servicePaused === true || (app.servicePaused && app.servicePaused.isTrue);
          
          if (isActive && !isPaused) {
            makers.push({
              makerId: i,
              owner: app.owner.toString(),
              tronAddress: app.tronAddress && app.tronAddress.length > 0 
                ? new TextDecoder().decode(new Uint8Array(app.tronAddress))
                : '',
              buyPremiumBps: app.buyPremiumBps ? app.buyPremiumBps.toNumber() : 0,
              sellPremiumBps: app.sellPremiumBps ? app.sellPremiumBps.toNumber() : 0,
              usersServed: app.usersServed ? (app.usersServed.toNumber ? app.usersServed.toNumber() : app.usersServed) : 0,
              servicePaused: isPaused,
              maskedFullName: app.maskedFullName && app.maskedFullName.length > 0
                ? new TextDecoder().decode(new Uint8Array(app.maskedFullName))
                : '',
              wechatId: app.wechatId && app.wechatId.length > 0
                ? new TextDecoder().decode(new Uint8Array(app.wechatId))
                : '',
            });
          }
        }
      }
      return makers;
    } catch (error) {
      console.error('获取活跃做市商列表失败:', error);
      return [];
    }
  }

  async getCurrentPrice(): Promise<string | null> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingPricing.currentPrice();
      if (result.isSome) {
        return result.unwrap().toString();
      }
      return null;
    } catch (error) {
      console.error('获取当前价格失败:', error);
      return null;
    }
  }

  getStateText(state: OrderState): string {
    switch (state) {
      case OrderState.Created:
        return '待付款';
      case OrderState.PaidOrCommitted:
        return '已付款';
      case OrderState.Released:
        return '已完成';
      case OrderState.Refunded:
        return '已退款';
      case OrderState.Canceled:
        return '已取消';
      case OrderState.Disputed:
        return '争议中';
      case OrderState.Closed:
        return '已关闭';
      case OrderState.Expired:
        return '已过期';
      default:
        return '未知';
    }
  }

  getStateColor(state: OrderState): string {
    switch (state) {
      case OrderState.Created:
        return '#f59e0b'; // 黄色
      case OrderState.PaidOrCommitted:
        return '#3b82f6'; // 蓝色
      case OrderState.Released:
        return '#10b981'; // 绿色
      case OrderState.Refunded:
        return '#6b7280'; // 灰色
      case OrderState.Canceled:
        return '#6b7280'; // 灰色
      case OrderState.Disputed:
        return '#ef4444'; // 红色
      case OrderState.Closed:
        return '#9ca3af'; // 浅灰色
      case OrderState.Expired:
        return '#9ca3af'; // 浅灰色
      default:
        return '#6b7280';
    }
  }

  formatCos(amount: string): string {
    const value = BigInt(amount);
    const cos = Number(value / BigInt(1e15)) / 1000;
    return `${cos.toFixed(4)} COS`;
  }

  formatUsdt(amount: string): string {
    const value = BigInt(amount);
    const usdt = Number(value / BigInt(1e3)) / 1000;
    return `${usdt.toFixed(2)} USDT`;
  }

  formatTime(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }

  getRemainingTime(expireAt: number): string {
    const now = Math.floor(Date.now() / 1000);
    const remaining = expireAt - now;
    if (remaining <= 0) return '已过期';
    
    const minutes = Math.floor(remaining / 60);
    const seconds = remaining % 60;
    if (minutes >= 60) {
      const hours = Math.floor(minutes / 60);
      const mins = minutes % 60;
      return `${hours}小时${mins}分钟`;
    }
    return `${minutes}分${seconds}秒`;
  }

  isExpired(expireAt: number): boolean {
    const now = Math.floor(Date.now() / 1000);
    return now > expireAt;
  }
}

export const otcService = new OtcService();
