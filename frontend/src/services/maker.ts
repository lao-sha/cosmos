import { useChainStore } from '@/src/stores/chain';
import { ApiPromise } from '@polkadot/api';

export enum ApplicationStatus {
  DepositLocked = 'DepositLocked',
  PendingReview = 'PendingReview',
  Active = 'Active',
  Rejected = 'Rejected',
  Cancelled = 'Cancelled',
  Expired = 'Expired',
}

export enum Direction {
  Buy = 0,
  Sell = 1,
  BuyAndSell = 2,
}

export enum WithdrawalStatus {
  Pending = 'Pending',
  Executed = 'Executed',
  Cancelled = 'Cancelled',
}

export interface MakerApplication {
  owner: string;
  deposit: string;
  status: ApplicationStatus;
  direction: Direction;
  tronAddress: string;
  publicCid: string;
  privateCid: string;
  buyPremiumBps: number;
  sellPremiumBps: number;
  minAmount: string;
  createdAt: number;
  infoDeadline: number;
  reviewDeadline: number;
  servicePaused: boolean;
  usersServed: number;
  maskedFullName: string;
  maskedIdCard: string;
  maskedBirthday: string;
  maskedPaymentInfo: string;
  wechatId: string;
  targetDepositUsd: number;
  lastPriceCheck: number;
  depositWarning: boolean;
}

export interface WithdrawalRequest {
  amount: string;
  requestedAt: number;
  executableAt: number;
  status: WithdrawalStatus;
}

export interface PenaltyRecord {
  makerId: number;
  penaltyType: string;
  deductedAmount: string;
  usdValue: number;
  beneficiary: string | null;
  deductedAt: number;
  appealed: boolean;
  appealResult: boolean | null;
}

export interface MakerInfo {
  makerId: number | null;
  application: MakerApplication | null;
  withdrawalRequest: WithdrawalRequest | null;
  penalties: number[];
  depositUsdValue: number | null;
  needsReplenishment: boolean;
}

class MakerService {
  private getApi(): ApiPromise {
    const { api } = useChainStore.getState();
    if (!api) {
      throw new Error('Chain not connected');
    }
    return api;
  }

  private parseStatus(status: any): ApplicationStatus {
    if (status.isDepositLocked) return ApplicationStatus.DepositLocked;
    if (status.isPendingReview) return ApplicationStatus.PendingReview;
    if (status.isActive) return ApplicationStatus.Active;
    if (status.isRejected) return ApplicationStatus.Rejected;
    if (status.isCancelled) return ApplicationStatus.Cancelled;
    if (status.isExpired) return ApplicationStatus.Expired;
    return ApplicationStatus.DepositLocked;
  }

  private parseDirection(direction: any): Direction {
    if (direction.isBuy) return Direction.Buy;
    if (direction.isSell) return Direction.Sell;
    if (direction.isBuyAndSell) return Direction.BuyAndSell;
    return Direction.BuyAndSell;
  }

  private parseWithdrawalStatus(status: any): WithdrawalStatus {
    if (status.isPending) return WithdrawalStatus.Pending;
    if (status.isExecuted) return WithdrawalStatus.Executed;
    if (status.isCancelled) return WithdrawalStatus.Cancelled;
    return WithdrawalStatus.Pending;
  }

  async getMakerId(address: string): Promise<number | null> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingMaker.accountToMaker(address);
      if (result.isSome) {
        return result.unwrap().toNumber();
      }
      return null;
    } catch (error) {
      console.error('获取做市商ID失败:', error);
      return null;
    }
  }

  async getMakerApplication(makerId: number): Promise<MakerApplication | null> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingMaker.makerApplications(makerId);
      if (result.isSome) {
        const app = result.unwrap();
        return {
          owner: app.owner.toString(),
          deposit: app.deposit.toString(),
          status: this.parseStatus(app.status),
          direction: this.parseDirection(app.direction),
          tronAddress: new TextDecoder().decode(new Uint8Array(app.tronAddress)),
          publicCid: new TextDecoder().decode(new Uint8Array(app.publicCid)),
          privateCid: new TextDecoder().decode(new Uint8Array(app.privateCid)),
          buyPremiumBps: app.buyPremiumBps.toNumber(),
          sellPremiumBps: app.sellPremiumBps.toNumber(),
          minAmount: app.minAmount.toString(),
          createdAt: app.createdAt.toNumber(),
          infoDeadline: app.infoDeadline.toNumber(),
          reviewDeadline: app.reviewDeadline.toNumber(),
          servicePaused: app.servicePaused.isTrue,
          usersServed: app.usersServed.toNumber(),
          maskedFullName: new TextDecoder().decode(new Uint8Array(app.maskedFullName)),
          maskedIdCard: new TextDecoder().decode(new Uint8Array(app.maskedIdCard)),
          maskedBirthday: new TextDecoder().decode(new Uint8Array(app.maskedBirthday)),
          maskedPaymentInfo: new TextDecoder().decode(new Uint8Array(app.maskedPaymentInfo)),
          wechatId: new TextDecoder().decode(new Uint8Array(app.wechatId)),
          targetDepositUsd: app.targetDepositUsd.toNumber(),
          lastPriceCheck: app.lastPriceCheck.toNumber(),
          depositWarning: app.depositWarning.isTrue,
        };
      }
      return null;
    } catch (error) {
      console.error('获取做市商申请信息失败:', error);
      return null;
    }
  }

  async getWithdrawalRequest(makerId: number): Promise<WithdrawalRequest | null> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingMaker.withdrawalRequests(makerId);
      if (result.isSome) {
        const req = result.unwrap();
        return {
          amount: req.amount.toString(),
          requestedAt: req.requestedAt.toNumber(),
          executableAt: req.executableAt.toNumber(),
          status: this.parseWithdrawalStatus(req.status),
        };
      }
      return null;
    } catch (error) {
      console.error('获取提现请求失败:', error);
      return null;
    }
  }

  async getMakerPenalties(makerId: number): Promise<number[]> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingMaker.makerPenalties(makerId);
      if (result && result.length > 0) {
        return result.map((id: any) => id.toNumber());
      }
      return [];
    } catch (error) {
      console.error('获取惩罚记录列表失败:', error);
      return [];
    }
  }

  async getPenaltyRecord(penaltyId: number): Promise<PenaltyRecord | null> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingMaker.penaltyRecords(penaltyId);
      if (result.isSome) {
        const record = result.unwrap();
        return {
          makerId: record.makerId.toNumber(),
          penaltyType: record.penaltyType.toString(),
          deductedAmount: record.deductedAmount.toString(),
          usdValue: record.usdValue.toNumber(),
          beneficiary: record.beneficiary.isSome ? record.beneficiary.unwrap().toString() : null,
          deductedAt: record.deductedAt.toNumber(),
          appealed: record.appealed.isTrue,
          appealResult: record.appealResult.isSome ? record.appealResult.unwrap().isTrue : null,
        };
      }
      return null;
    } catch (error) {
      console.error('获取惩罚记录失败:', error);
      return null;
    }
  }

  async getMakerInfo(address: string): Promise<MakerInfo> {
    const makerId = await this.getMakerId(address);
    
    if (makerId === null) {
      return {
        makerId: null,
        application: null,
        withdrawalRequest: null,
        penalties: [],
        depositUsdValue: null,
        needsReplenishment: false,
      };
    }

    const [application, withdrawalRequest, penalties] = await Promise.all([
      this.getMakerApplication(makerId),
      this.getWithdrawalRequest(makerId),
      this.getMakerPenalties(makerId),
    ]);

    return {
      makerId,
      application,
      withdrawalRequest,
      penalties,
      depositUsdValue: application?.targetDepositUsd || null,
      needsReplenishment: application?.depositWarning || false,
    };
  }

  async getNextMakerId(): Promise<number> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingMaker.nextMakerId();
      return result.toNumber();
    } catch (error) {
      console.error('获取下一个做市商ID失败:', error);
      return 0;
    }
  }

  async getDepositAmount(): Promise<string> {
    try {
      const api = this.getApi();
      const result = (api.consts as any).tradingMaker.makerDepositAmount;
      return result.toString();
    } catch (error) {
      console.error('获取押金金额失败:', error);
      return '1000000000000000000000'; // 默认 1000 COS
    }
  }

  async getWithdrawalCooldown(): Promise<number> {
    try {
      const api = this.getApi();
      const result = (api.consts as any).tradingMaker.withdrawalCooldown;
      return result.toNumber();
    } catch (error) {
      console.error('获取提现冷却期失败:', error);
      return 604800; // 默认 7 天（秒）
    }
  }

  getStatusText(status: ApplicationStatus): string {
    switch (status) {
      case ApplicationStatus.DepositLocked:
        return '押金已锁定';
      case ApplicationStatus.PendingReview:
        return '等待审核';
      case ApplicationStatus.Active:
        return '已激活';
      case ApplicationStatus.Rejected:
        return '已驳回';
      case ApplicationStatus.Cancelled:
        return '已取消';
      case ApplicationStatus.Expired:
        return '已过期';
      default:
        return '未知状态';
    }
  }

  getStatusColor(status: ApplicationStatus): string {
    switch (status) {
      case ApplicationStatus.DepositLocked:
        return '#f59e0b'; // 黄色
      case ApplicationStatus.PendingReview:
        return '#3b82f6'; // 蓝色
      case ApplicationStatus.Active:
        return '#10b981'; // 绿色
      case ApplicationStatus.Rejected:
        return '#ef4444'; // 红色
      case ApplicationStatus.Cancelled:
        return '#6b7280'; // 灰色
      case ApplicationStatus.Expired:
        return '#9ca3af'; // 浅灰色
      default:
        return '#6b7280';
    }
  }

  getDirectionText(direction: Direction): string {
    switch (direction) {
      case Direction.Buy:
        return '仅买入 (Bridge)';
      case Direction.Sell:
        return '仅卖出 (OTC)';
      case Direction.BuyAndSell:
        return '双向 (OTC + Bridge)';
      default:
        return '未知';
    }
  }

  formatDeposit(deposit: string): string {
    const value = BigInt(deposit);
    const cos = Number(value / BigInt(1e15)) / 1000;
    return `${cos.toFixed(2)} COS`;
  }

  formatUsdValue(usdValue: number): string {
    const usd = usdValue / 1e6;
    return `$${usd.toFixed(2)}`;
  }
}

export const makerService = new MakerService();
