/**
 * 星尘玄鉴 - 做市商服务层
 * 封装与 maker pallet 交互的 API
 */

import { ApiPromise } from '@polkadot/api';
import { getApi } from '@/lib/api';
import { signAndSend } from '@/lib/signer';

// ===== 类型定义 =====

/**
 * 申请状态
 */
export enum ApplicationStatus {
  DepositLocked = 'DepositLocked',
  PendingReview = 'PendingReview',
  Active = 'Active',
  Rejected = 'Rejected',
  Cancelled = 'Cancelled',
  Expired = 'Expired',
}

/**
 * 业务方向
 */
export enum Direction {
  Buy = 0,      // 仅买入 (Bridge)
  Sell = 1,     // 仅卖出 (OTC)
  BuyAndSell = 2, // 双向
}

/**
 * 提现状态
 */
export enum WithdrawalStatus {
  Pending = 'Pending',
  Executed = 'Executed',
  Cancelled = 'Cancelled',
}

/**
 * 做市商申请信息
 */
export interface MakerApplication {
  id: number;
  owner: string;
  deposit: bigint;
  status: ApplicationStatus;
  direction: Direction;
  tronAddress: string;
  publicCid: string;
  privateCid: string;
  buyPremiumBps: number;
  sellPremiumBps: number;
  minAmount: bigint;
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
  epayNo?: string;
  targetDepositUsd: number;
  lastPriceCheck: number;
  depositWarning: boolean;
}

/**
 * 提现请求
 */
export interface WithdrawalRequest {
  amount: bigint;
  requestedAt: number;
  executableAt: number;
  status: WithdrawalStatus;
}

/**
 * 扣除类型
 */
export type PenaltyType =
  | { type: 'OtcTimeout'; orderId: number; timeoutHours: number }
  | { type: 'BridgeTimeout'; swapId: number; timeoutHours: number }
  | { type: 'ArbitrationLoss'; caseId: number; lossAmount: number }
  | { type: 'LowCreditScore'; currentScore: number; daysBelowThreshold: number }
  | { type: 'MaliciousBehavior'; behaviorType: number; evidenceCid: string };

/**
 * 扣除记录
 */
export interface PenaltyRecord {
  id: number;
  makerId: number;
  penaltyType: PenaltyType;
  deductedAmount: bigint;
  usdValue: number;
  beneficiary?: string;
  deductedAt: number;
  appealed: boolean;
  appealResult?: boolean;
}

/**
 * 提交资料输入
 */
export interface MakerInfoInput {
  realName: string;
  idCardNumber: string;
  birthday: string;
  tronAddress: string;
  wechatId: string;
  epayNo?: string;
  epayKey?: string;
}

/**
 * 做市商服务类
 */
export class MakerService {
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

  // ===== 查询方法 =====

  /**
   * 通过账户获取做市商ID
   */
  async getMakerIdByAccount(account: string): Promise<number | null> {
    const api = this.getApi();

    try {
      const makerId = await api.query.maker.accountToMaker(account);
      if (makerId.isNone) {
        return null;
      }
      return makerId.unwrap().toNumber();
    } catch (error) {
      console.error('[MakerService] Get maker ID error:', error);
      return null;
    }
  }

  /**
   * 获取做市商详情
   */
  async getMakerApplication(makerId: number): Promise<MakerApplication | null> {
    const api = this.getApi();

    try {
      const app = await api.query.maker.makerApplications(makerId);
      if (app.isNone) {
        return null;
      }

      const data = app.unwrap();
      return this.parseMakerApplication(makerId, data);
    } catch (error) {
      console.error('[MakerService] Get maker application error:', error);
      return null;
    }
  }

  /**
   * 通过账户获取做市商详情
   */
  async getMakerByAccount(account: string): Promise<MakerApplication | null> {
    const makerId = await this.getMakerIdByAccount(account);
    if (makerId === null) {
      return null;
    }
    return this.getMakerApplication(makerId);
  }

  /**
   * 获取提现请求
   */
  async getWithdrawalRequest(makerId: number): Promise<WithdrawalRequest | null> {
    const api = this.getApi();

    try {
      const request = await api.query.maker.withdrawalRequests(makerId);
      if (request.isNone) {
        return null;
      }

      const data = request.unwrap();
      return {
        amount: data.amount.toBigInt(),
        requestedAt: data.requestedAt.toNumber(),
        executableAt: data.executableAt.toNumber(),
        status: data.status.toString() as WithdrawalStatus,
      };
    } catch (error) {
      console.error('[MakerService] Get withdrawal request error:', error);
      return null;
    }
  }

  /**
   * 获取做市商的扣除记录ID列表
   */
  async getMakerPenaltyIds(makerId: number): Promise<number[]> {
    const api = this.getApi();

    try {
      const penaltyIds = await api.query.maker.makerPenalties(makerId);
      return penaltyIds.map((id: any) => id.toNumber());
    } catch (error) {
      console.error('[MakerService] Get maker penalties error:', error);
      return [];
    }
  }

  /**
   * 获取扣除记录详情
   */
  async getPenaltyRecord(penaltyId: number): Promise<PenaltyRecord | null> {
    const api = this.getApi();

    try {
      const record = await api.query.maker.penaltyRecords(penaltyId);
      if (record.isNone) {
        return null;
      }

      const data = record.unwrap();
      return this.parsePenaltyRecord(penaltyId, data);
    } catch (error) {
      console.error('[MakerService] Get penalty record error:', error);
      return null;
    }
  }

  /**
   * 获取做市商的所有扣除记录
   */
  async getMakerPenalties(makerId: number): Promise<PenaltyRecord[]> {
    const penaltyIds = await this.getMakerPenaltyIds(makerId);
    const penalties: PenaltyRecord[] = [];

    for (const id of penaltyIds) {
      const record = await this.getPenaltyRecord(id);
      if (record) {
        penalties.push(record);
      }
    }

    // 按扣除时间倒序排序
    penalties.sort((a, b) => b.deductedAt - a.deductedAt);
    return penalties;
  }

  /**
   * 获取 DUST 价格 (USD)
   */
  async getDustPrice(): Promise<number> {
    const api = this.getApi();

    try {
      // 检查是否在冷启动期间
      const coldStartExited = await api.query.tradingPricing.coldStartExited();

      if (!coldStartExited.isTrue) {
        const defaultPrice = await api.query.tradingPricing.defaultPrice();
        const priceValue = defaultPrice.toNumber();
        return priceValue > 1000 ? priceValue / 1_000_000 : 0.1;
      }

      const stats = await api.query.tradingPricing.marketStats();
      return stats.weightedPrice.toNumber() / 1_000_000;
    } catch (error) {
      console.error('[MakerService] Get dust price error:', error);
      return 0.1;
    }
  }

  /**
   * 计算押金的 USD 价值
   */
  async calculateDepositUsdValue(depositAmount: bigint): Promise<number> {
    const dustPrice = await this.getDustPrice();
    // DUST 精度是 10^12
    const dustAmount = Number(depositAmount) / 1e12;
    return dustAmount * dustPrice;
  }

  // ===== 交易方法 =====

  /**
   * 锁定押金
   */
  async lockDeposit(
    accountAddress: string,
    onStatusChange?: (status: string) => void
  ): Promise<void> {
    const api = this.getApi();

    try {
      const tx = api.tx.maker.lockDeposit();
      await signAndSend(api, tx, accountAddress, onStatusChange);
      console.log('[MakerService] Deposit locked successfully');
    } catch (error) {
      console.error('[MakerService] Lock deposit error:', error);
      throw error;
    }
  }

  /**
   * 提交资料
   */
  async submitInfo(
    accountAddress: string,
    info: MakerInfoInput,
    onStatusChange?: (status: string) => void
  ): Promise<void> {
    const api = this.getApi();

    try {
      const tx = api.tx.maker.submitInfo(
        info.realName,
        info.idCardNumber,
        info.birthday,
        info.tronAddress,
        info.wechatId,
        info.epayNo || null,
        info.epayKey || null
      );
      await signAndSend(api, tx, accountAddress, onStatusChange);
      console.log('[MakerService] Info submitted successfully');
    } catch (error) {
      console.error('[MakerService] Submit info error:', error);
      throw error;
    }
  }

  /**
   * 取消申请
   */
  async cancelMaker(
    accountAddress: string,
    onStatusChange?: (status: string) => void
  ): Promise<void> {
    const api = this.getApi();

    try {
      const tx = api.tx.maker.cancelMaker();
      await signAndSend(api, tx, accountAddress, onStatusChange);
      console.log('[MakerService] Maker cancelled successfully');
    } catch (error) {
      console.error('[MakerService] Cancel maker error:', error);
      throw error;
    }
  }

  /**
   * 补充押金
   */
  async replenishDeposit(
    accountAddress: string,
    onStatusChange?: (status: string) => void
  ): Promise<void> {
    const api = this.getApi();

    try {
      const tx = api.tx.maker.replenishDeposit();
      await signAndSend(api, tx, accountAddress, onStatusChange);
      console.log('[MakerService] Deposit replenished successfully');
    } catch (error) {
      console.error('[MakerService] Replenish deposit error:', error);
      throw error;
    }
  }

  /**
   * 申请提现
   */
  async requestWithdrawal(
    accountAddress: string,
    amount: bigint,
    onStatusChange?: (status: string) => void
  ): Promise<void> {
    const api = this.getApi();

    try {
      const tx = api.tx.maker.requestWithdrawal(amount.toString());
      await signAndSend(api, tx, accountAddress, onStatusChange);
      console.log('[MakerService] Withdrawal requested successfully');
    } catch (error) {
      console.error('[MakerService] Request withdrawal error:', error);
      throw error;
    }
  }

  /**
   * 执行提现
   */
  async executeWithdrawal(
    accountAddress: string,
    onStatusChange?: (status: string) => void
  ): Promise<void> {
    const api = this.getApi();

    try {
      const tx = api.tx.maker.executeWithdrawal();
      await signAndSend(api, tx, accountAddress, onStatusChange);
      console.log('[MakerService] Withdrawal executed successfully');
    } catch (error) {
      console.error('[MakerService] Execute withdrawal error:', error);
      throw error;
    }
  }

  /**
   * 取消提现
   */
  async cancelWithdrawal(
    accountAddress: string,
    onStatusChange?: (status: string) => void
  ): Promise<void> {
    const api = this.getApi();

    try {
      const tx = api.tx.maker.cancelWithdrawal();
      await signAndSend(api, tx, accountAddress, onStatusChange);
      console.log('[MakerService] Withdrawal cancelled successfully');
    } catch (error) {
      console.error('[MakerService] Cancel withdrawal error:', error);
      throw error;
    }
  }

  /**
   * 申诉扣除
   */
  async appealPenalty(
    accountAddress: string,
    penaltyId: number,
    evidenceCid: string,
    onStatusChange?: (status: string) => void
  ): Promise<void> {
    const api = this.getApi();

    try {
      const tx = api.tx.maker.appealPenalty(penaltyId, evidenceCid);
      await signAndSend(api, tx, accountAddress, onStatusChange);
      console.log('[MakerService] Penalty appealed successfully');
    } catch (error) {
      console.error('[MakerService] Appeal penalty error:', error);
      throw error;
    }
  }

  // ===== 订阅方法 =====

  /**
   * 订阅做市商状态变化
   */
  async subscribeToMaker(
    makerId: number,
    callback: (maker: MakerApplication | null) => void
  ): Promise<() => void> {
    const api = this.getApi();

    const unsub = await api.query.maker.makerApplications(makerId, (app: any) => {
      if (app.isSome) {
        callback(this.parseMakerApplication(makerId, app.unwrap()));
      } else {
        callback(null);
      }
    });

    return unsub;
  }

  /**
   * 订阅提现请求状态
   */
  async subscribeToWithdrawal(
    makerId: number,
    callback: (request: WithdrawalRequest | null) => void
  ): Promise<() => void> {
    const api = this.getApi();

    const unsub = await api.query.maker.withdrawalRequests(makerId, (request: any) => {
      if (request.isSome) {
        const data = request.unwrap();
        callback({
          amount: data.amount.toBigInt(),
          requestedAt: data.requestedAt.toNumber(),
          executableAt: data.executableAt.toNumber(),
          status: data.status.toString() as WithdrawalStatus,
        });
      } else {
        callback(null);
      }
    });

    return unsub;
  }

  // ===== 工具方法 =====

  /**
   * 解析做市商申请数据
   */
  private parseMakerApplication(makerId: number, data: any): MakerApplication {
    // 解析状态
    let status: ApplicationStatus;
    if (data.status.isDepositLocked) {
      status = ApplicationStatus.DepositLocked;
    } else if (data.status.isPendingReview) {
      status = ApplicationStatus.PendingReview;
    } else if (data.status.isActive) {
      status = ApplicationStatus.Active;
    } else if (data.status.isRejected) {
      status = ApplicationStatus.Rejected;
    } else if (data.status.isCancelled) {
      status = ApplicationStatus.Cancelled;
    } else {
      status = ApplicationStatus.Expired;
    }

    // 解析方向
    let direction: Direction;
    if (data.direction.isBuy) {
      direction = Direction.Buy;
    } else if (data.direction.isSell) {
      direction = Direction.Sell;
    } else {
      direction = Direction.BuyAndSell;
    }

    return {
      id: makerId,
      owner: data.owner.toString(),
      deposit: data.deposit.toBigInt(),
      status,
      direction,
      tronAddress: this.decodeBytes(data.tronAddress),
      publicCid: this.decodeBytes(data.publicCid),
      privateCid: this.decodeBytes(data.privateCid),
      buyPremiumBps: data.buyPremiumBps.toNumber(),
      sellPremiumBps: data.sellPremiumBps.toNumber(),
      minAmount: data.minAmount.toBigInt(),
      createdAt: data.createdAt.toNumber(),
      infoDeadline: data.infoDeadline.toNumber(),
      reviewDeadline: data.reviewDeadline.toNumber(),
      servicePaused: data.servicePaused.isTrue,
      usersServed: data.usersServed.toNumber(),
      maskedFullName: this.decodeBytes(data.maskedFullName),
      maskedIdCard: this.decodeBytes(data.maskedIdCard),
      maskedBirthday: this.decodeBytes(data.maskedBirthday),
      maskedPaymentInfo: this.decodeBytes(data.maskedPaymentInfo),
      wechatId: this.decodeBytes(data.wechatId),
      epayNo: data.epayNo.isSome ? this.decodeBytes(data.epayNo.unwrap()) : undefined,
      targetDepositUsd: data.targetDepositUsd.toNumber(),
      lastPriceCheck: data.lastPriceCheck.toNumber(),
      depositWarning: data.depositWarning.isTrue,
    };
  }

  /**
   * 解析扣除记录数据
   */
  private parsePenaltyRecord(penaltyId: number, data: any): PenaltyRecord {
    let penaltyType: PenaltyType;

    if (data.penaltyType.isOtcTimeout) {
      const inner = data.penaltyType.asOtcTimeout;
      penaltyType = {
        type: 'OtcTimeout',
        orderId: inner.orderId.toNumber(),
        timeoutHours: inner.timeoutHours.toNumber(),
      };
    } else if (data.penaltyType.isBridgeTimeout) {
      const inner = data.penaltyType.asBridgeTimeout;
      penaltyType = {
        type: 'BridgeTimeout',
        swapId: inner.swapId.toNumber(),
        timeoutHours: inner.timeoutHours.toNumber(),
      };
    } else if (data.penaltyType.isArbitrationLoss) {
      const inner = data.penaltyType.asArbitrationLoss;
      penaltyType = {
        type: 'ArbitrationLoss',
        caseId: inner.caseId.toNumber(),
        lossAmount: inner.lossAmount.toNumber(),
      };
    } else if (data.penaltyType.isLowCreditScore) {
      const inner = data.penaltyType.asLowCreditScore;
      penaltyType = {
        type: 'LowCreditScore',
        currentScore: inner.currentScore.toNumber(),
        daysBelowThreshold: inner.daysBelowThreshold.toNumber(),
      };
    } else {
      const inner = data.penaltyType.asMaliciousBehavior;
      penaltyType = {
        type: 'MaliciousBehavior',
        behaviorType: inner.behaviorType.toNumber(),
        evidenceCid: this.decodeBytes(inner.evidenceCid),
      };
    }

    return {
      id: penaltyId,
      makerId: data.makerId.toNumber(),
      penaltyType,
      deductedAmount: data.deductedAmount.toBigInt(),
      usdValue: data.usdValue.toNumber() / 1_000_000,
      beneficiary: data.beneficiary.isSome ? data.beneficiary.unwrap().toString() : undefined,
      deductedAt: data.deductedAt.toNumber(),
      appealed: data.appealed.isTrue,
      appealResult: data.appealResult.isSome ? data.appealResult.unwrap().isTrue : undefined,
    };
  }

  /**
   * 解码字节数组为字符串
   */
  private decodeBytes(bytes: any): string {
    try {
      if (typeof bytes === 'string') {
        return bytes;
      }
      if (bytes.toHuman) {
        return bytes.toHuman() as string;
      }
      if (bytes.toUtf8) {
        return bytes.toUtf8();
      }
      return new TextDecoder().decode(new Uint8Array(bytes));
    } catch {
      return '';
    }
  }

  // ===== 静态工具方法 =====

  /**
   * 格式化 DUST 数量
   */
  static formatDustAmount(amount: bigint): string {
    return (Number(amount) / 1e12).toFixed(4);
  }

  /**
   * 格式化 USD 金额
   */
  static formatUsdAmount(amount: number): string {
    return amount.toFixed(2);
  }

  /**
   * 验证 TRON 地址格式
   */
  static isValidTronAddress(address: string): boolean {
    return /^T[A-Za-z1-9]{33}$/.test(address);
  }

  /**
   * 验证身份证号格式
   */
  static isValidIdCard(idCard: string): boolean {
    if (!/^\d{17}[\dXx]$/.test(idCard)) {
      return false;
    }
    // 校验码验证
    const weights = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
    const checkCodes = ['1', '0', 'X', '9', '8', '7', '6', '5', '4', '3', '2'];
    let sum = 0;
    for (let i = 0; i < 17; i++) {
      sum += parseInt(idCard[i]) * weights[i];
    }
    const checkCode = checkCodes[sum % 11];
    return idCard[17].toUpperCase() === checkCode;
  }

  /**
   * 脱敏姓名
   */
  static maskName(name: string): string {
    if (name.length <= 1) return name;
    if (name.length === 2) return name[0] + '*';
    return name[0] + '*'.repeat(name.length - 2) + name[name.length - 1];
  }

  /**
   * 脱敏身份证号
   */
  static maskIdCard(idCard: string): string {
    if (idCard.length < 8) return idCard;
    return idCard.slice(0, 4) + '**********' + idCard.slice(-4);
  }

  /**
   * 脱敏生日
   */
  static maskBirthday(birthday: string): string {
    // 格式: YYYY-MM-DD -> YYYY-**-**
    if (birthday.length !== 10) return birthday;
    return birthday.slice(0, 5) + '**-**';
  }

  /**
   * 获取申请状态显示文本
   */
  static getStatusText(status: ApplicationStatus): string {
    const texts: Record<ApplicationStatus, string> = {
      [ApplicationStatus.DepositLocked]: '押金已锁定',
      [ApplicationStatus.PendingReview]: '待审核',
      [ApplicationStatus.Active]: '已激活',
      [ApplicationStatus.Rejected]: '已驳回',
      [ApplicationStatus.Cancelled]: '已取消',
      [ApplicationStatus.Expired]: '已过期',
    };
    return texts[status] || '未知';
  }

  /**
   * 获取扣除类型显示文本
   */
  static getPenaltyTypeText(penaltyType: PenaltyType): string {
    switch (penaltyType.type) {
      case 'OtcTimeout':
        return 'OTC订单超时';
      case 'BridgeTimeout':
        return 'Bridge兑换超时';
      case 'ArbitrationLoss':
        return '争议仲裁败诉';
      case 'LowCreditScore':
        return '信用分过低';
      case 'MaliciousBehavior':
        return '恶意行为';
      default:
        return '未知';
    }
  }
}

// 导出单例
export const makerService = new MakerService();
