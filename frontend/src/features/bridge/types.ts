/**
 * 桥接模块类型定义
 * DUST ↔ USDT 桥接服务
 */

/** 兑换状态 */
export enum SwapStatus {
  /** 待处理 */
  Pending = 'Pending',
  /** 已完成 */
  Completed = 'Completed',
  /** 用户举报 */
  UserReported = 'UserReported',
  /** 仲裁中 */
  Arbitrating = 'Arbitrating',
  /** 仲裁通过 */
  ArbitrationApproved = 'ArbitrationApproved',
  /** 仲裁拒绝 */
  ArbitrationRejected = 'ArbitrationRejected',
  /** 超时退款 */
  Refunded = 'Refunded',
}

/** 官方桥接兑换请求 */
export interface SwapRequest {
  id: number;
  user: string;
  dustAmount: bigint;
  tronAddress: string;
  completed: boolean;
  priceUsdt: number;
  createdAt: number;
  expireAt: number;
}

/** 做市商兑换记录 */
export interface MakerSwapRecord {
  swapId: number;
  makerId: number;
  maker: string;
  user: string;
  dustAmount: bigint;
  usdtAmount: number;
  usdtAddress: string;
  createdAt: number;
  timeoutAt: number;
  trc20TxHash?: string;
  completedAt?: number;
  evidenceCid?: string;
  status: SwapStatus;
  priceUsdt: number;
}

/** 桥接做市商信息 */
export interface BridgeMaker {
  id: number;
  account: string;
  tronAddress: string;
  isActive: boolean;
  rating: number;
  completedSwaps: number;
  avgResponseTime?: number;
  creditScore?: number;
  creditLevel?: string;
}

/** 创建兑换参数 */
export interface CreateSwapParams {
  dustAmount: bigint;
  tronAddress: string;
}

/** 创建做市商兑换参数 */
export interface CreateMakerSwapParams {
  makerId: number;
  dustAmount: bigint;
  usdtAddress: string;
}
