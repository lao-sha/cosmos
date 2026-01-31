import { useChainStore } from '@/src/stores/chain';
import { ApiPromise } from '@polkadot/api';

export enum SwapStatus {
  Pending = 0,
  AwaitingVerification = 1,
  Completed = 2,
  VerificationFailed = 3,
  UserReported = 4,
  Arbitrating = 5,
  ArbitrationApproved = 6,
  ArbitrationRejected = 7,
  Refunded = 8,
}

export interface SwapRecord {
  swapId: number;
  makerId: number;
  maker: string;
  user: string;
  cosAmount: string;
  usdtAmount: number;
  usdtAddress: string;
  createdAt: number;
  timeoutAt: number;
  trc20TxHash: string | null;
  completedAt: number | null;
  evidenceCid: string | null;
  status: SwapStatus;
  priceUsdt: number;
}

export interface SwapTimeInfo {
  swapId: number;
  makerId: number;
  user: string;
  cosAmount: string;
  usdtAmount: number;
  createdAtBlock: number;
  createdAtTimestamp: number;
  timeoutAtBlock: number;
  timeoutAtTimestamp: number;
  remainingSeconds: number;
  remainingReadable: string;
  status: number;
  isTimeout: boolean;
}

export interface MakerInfo {
  makerId: number;
  owner: string;
  tronAddress: string;
  sellPremiumBps: number;
  usersServed: number;
  servicePaused: boolean;
  maskedFullName: string;
  wechatId: string;
}

class SwapService {
  private getApi(): ApiPromise {
    const { api } = useChainStore.getState();
    if (!api) {
      throw new Error('Chain not connected');
    }
    return api;
  }

  private parseSwapStatus(status: any): SwapStatus {
    if (status.isPending) return SwapStatus.Pending;
    if (status.isAwaitingVerification) return SwapStatus.AwaitingVerification;
    if (status.isCompleted) return SwapStatus.Completed;
    if (status.isVerificationFailed) return SwapStatus.VerificationFailed;
    if (status.isUserReported) return SwapStatus.UserReported;
    if (status.isArbitrating) return SwapStatus.Arbitrating;
    if (status.isArbitrationApproved) return SwapStatus.ArbitrationApproved;
    if (status.isArbitrationRejected) return SwapStatus.ArbitrationRejected;
    if (status.isRefunded) return SwapStatus.Refunded;
    return SwapStatus.Pending;
  }

  async getSwap(swapId: number): Promise<SwapRecord | null> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingSwap.makerSwaps(swapId);
      if (result.isSome) {
        const record = result.unwrap();
        return {
          swapId,
          makerId: record.makerId.toNumber(),
          maker: record.maker.toString(),
          user: record.user.toString(),
          cosAmount: record.cosAmount.toString(),
          usdtAmount: record.usdtAmount.toNumber(),
          usdtAddress: new TextDecoder().decode(new Uint8Array(record.usdtAddress)),
          createdAt: record.createdAt.toNumber(),
          timeoutAt: record.timeoutAt.toNumber(),
          trc20TxHash: record.trc20TxHash.isSome 
            ? new TextDecoder().decode(new Uint8Array(record.trc20TxHash.unwrap()))
            : null,
          completedAt: record.completedAt.isSome ? record.completedAt.unwrap().toNumber() : null,
          evidenceCid: record.evidenceCid.isSome
            ? new TextDecoder().decode(new Uint8Array(record.evidenceCid.unwrap()))
            : null,
          status: this.parseSwapStatus(record.status),
          priceUsdt: record.priceUsdt.toNumber(),
        };
      }
      return null;
    } catch (error) {
      console.error('获取兑换记录失败:', error);
      return null;
    }
  }

  async getUserSwaps(address: string): Promise<number[]> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingSwap.userSwaps(address);
      if (result && result.length > 0) {
        return result.map((id: any) => id.toNumber());
      }
      return [];
    } catch (error) {
      console.error('获取用户兑换列表失败:', error);
      return [];
    }
  }

  async getMakerSwapList(makerId: number): Promise<number[]> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingSwap.makerSwapList(makerId);
      if (result && result.length > 0) {
        return result.map((id: any) => id.toNumber());
      }
      return [];
    } catch (error) {
      console.error('获取做市商兑换列表失败:', error);
      return [];
    }
  }

  async getNextSwapId(): Promise<number> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingSwap.nextSwapId();
      return result.toNumber();
    } catch (error) {
      console.error('获取下一个兑换ID失败:', error);
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

  getStatusText(status: SwapStatus): string {
    switch (status) {
      case SwapStatus.Pending:
        return '待处理';
      case SwapStatus.AwaitingVerification:
        return '验证中';
      case SwapStatus.Completed:
        return '已完成';
      case SwapStatus.VerificationFailed:
        return '验证失败';
      case SwapStatus.UserReported:
        return '已举报';
      case SwapStatus.Arbitrating:
        return '仲裁中';
      case SwapStatus.ArbitrationApproved:
        return '仲裁通过';
      case SwapStatus.ArbitrationRejected:
        return '仲裁拒绝';
      case SwapStatus.Refunded:
        return '已退款';
      default:
        return '未知';
    }
  }

  getStatusColor(status: SwapStatus): string {
    switch (status) {
      case SwapStatus.Pending:
        return '#f59e0b'; // 黄色
      case SwapStatus.AwaitingVerification:
        return '#3b82f6'; // 蓝色
      case SwapStatus.Completed:
        return '#10b981'; // 绿色
      case SwapStatus.VerificationFailed:
        return '#ef4444'; // 红色
      case SwapStatus.UserReported:
        return '#f97316'; // 橙色
      case SwapStatus.Arbitrating:
        return '#8b5cf6'; // 紫色
      case SwapStatus.ArbitrationApproved:
        return '#10b981'; // 绿色
      case SwapStatus.ArbitrationRejected:
        return '#ef4444'; // 红色
      case SwapStatus.Refunded:
        return '#6b7280'; // 灰色
      default:
        return '#6b7280';
    }
  }

  formatCos(amount: string): string {
    const value = BigInt(amount);
    const cos = Number(value / BigInt(1e15)) / 1000;
    return `${cos.toFixed(4)} COS`;
  }

  formatUsdt(amount: number): string {
    const usdt = amount / 1e6;
    return `${usdt.toFixed(2)} USDT`;
  }

  formatTime(blockNumber: number): string {
    const estimatedTimestamp = blockNumber * 6 * 1000;
    return new Date(estimatedTimestamp).toLocaleString();
  }

  getRemainingBlocks(timeoutAt: number, currentBlock: number): number {
    const remaining = timeoutAt - currentBlock;
    return remaining > 0 ? remaining : 0;
  }

  getRemainingTime(timeoutAt: number, currentBlock: number): string {
    const remainingBlocks = this.getRemainingBlocks(timeoutAt, currentBlock);
    if (remainingBlocks <= 0) return '已超时';
    
    const remainingSeconds = remainingBlocks * 6;
    const hours = Math.floor(remainingSeconds / 3600);
    const minutes = Math.floor((remainingSeconds % 3600) / 60);
    
    if (hours > 0) {
      return `${hours}小时${minutes}分钟`;
    }
    return `${minutes}分钟`;
  }

  isTimeout(timeoutAt: number, currentBlock: number): boolean {
    return currentBlock >= timeoutAt;
  }

  calculateUsdt(cosAmount: string, priceUsdt: number): number {
    const cos = BigInt(cosAmount);
    // 链上精度：1 COS = 1e12 最小单位
    const usdt = (Number(cos) * priceUsdt) / 1e12;
    return Math.floor(usdt);
  }
}

export const swapService = new SwapService();
