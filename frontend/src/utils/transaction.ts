/**
 * 交易工具函数
 * 包含错误重试机制、错误解析等
 */

import { ApiPromise } from '@polkadot/api';
import type { DispatchError } from '@polkadot/types/interfaces';

// ===== 类型定义 =====

export interface TxResult {
  success: boolean;
  txHash?: string;
  blockHash?: string;
  error?: TxError;
}

export interface TxError {
  code: string;
  message: string;
  isRetryable: boolean;
  userMessage: string;
}

// ===== 常量定义 =====

// 可重试的错误类型
const RETRYABLE_ERRORS = [
  'Priority is too low',
  'Transaction is outdated',
  'Unable to retrieve nonce',
  'Connection timeout',
  'WebSocket is not connected',
  'Disconnected from',
];

// 错误码到用户友好消息的映射
const ERROR_MESSAGES: Record<string, string> = {
  // OTC Order 错误
  'OtcOrder.MakerNotFound': '该做市商已下线，请选择其他做市商',
  'OtcOrder.MakerServicePaused': '做市商暂停服务，请选择其他做市商',
  'OtcOrder.InsufficientBalance': '做市商余额不足，请选择其他做市商',
  'OtcOrder.AmountTooLow': '金额不能低于最小限额',
  'OtcOrder.AmountTooHigh': '金额不能超过最大限额',
  'OtcOrder.OrderExpired': '订单已超时，请重新创建订单',
  'OtcOrder.AlreadyFirstPurchased': '您已完成首购，请使用普通购买',
  'OtcOrder.OrderNotFound': '订单不存在',
  'OtcOrder.NotOrderTaker': '您不是该订单的买家',
  'OtcOrder.InvalidOrderState': '订单状态不正确',
  'OtcOrder.AlreadyPaid': '订单已标记为已付款',
  'OtcOrder.AlreadyDisputed': '订单已在仲裁中',
  'OtcOrder.DisputeWindowClosed': '仲裁窗口已关闭',
  'OtcOrder.KycRequired': '需要完成身份认证',
  'OtcOrder.ConcurrentOrderLimit': '并发订单数已达上限',

  // Balances 错误
  'Balances.InsufficientBalance': '账户余额不足支付手续费',
  'Balances.ExistentialDeposit': '账户余额低于最小存款要求',

  // 通用错误
  'System.CallFiltered': '该操作暂时不可用',
  'Unknown': '操作失败，请稍后重试',
};

// ===== 工具函数 =====

/**
 * 解析链上错误
 */
export function parseDispatchError(
  api: ApiPromise,
  dispatchError: DispatchError
): TxError {
  if (dispatchError.isModule) {
    try {
      const decoded = api.registry.findMetaError(dispatchError.asModule);
      const code = `${decoded.section}.${decoded.name}`;
      const message = decoded.docs.join(' ');
      const userMessage = ERROR_MESSAGES[code] || message || '操作失败';

      return {
        code,
        message,
        isRetryable: false,
        userMessage,
      };
    } catch (e) {
      // 解析失败，返回通用错误
    }
  }

  const errorString = dispatchError.toString();
  const isRetryable = RETRYABLE_ERRORS.some(e => errorString.includes(e));

  return {
    code: 'Unknown',
    message: errorString,
    isRetryable,
    userMessage: ERROR_MESSAGES['Unknown'],
  };
}

/**
 * 检查错误是否可重试
 */
export function isRetryableError(error: Error | TxError): boolean {
  if ('isRetryable' in error) {
    return error.isRetryable;
  }

  const message = error.message || '';
  return RETRYABLE_ERRORS.some(e => message.includes(e));
}

/**
 * 延迟函数
 */
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * 指数退避延迟
 */
export function getRetryDelay(retryCount: number): number {
  const delays = [1000, 2000, 4000, 8000, 16000];
  return delays[Math.min(retryCount, delays.length - 1)];
}

/**
 * 带重试的异步操作
 */
export async function withRetry<T>(
  operation: () => Promise<T>,
  options: {
    maxRetries?: number;
    onRetry?: (error: Error, retryCount: number) => void;
    shouldRetry?: (error: Error) => boolean;
  } = {}
): Promise<T> {
  const {
    maxRetries = 3,
    onRetry,
    shouldRetry = isRetryableError,
  } = options;

  let lastError: Error | null = null;

  for (let i = 0; i <= maxRetries; i++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error as Error;

      if (i < maxRetries && shouldRetry(lastError)) {
        const delay = getRetryDelay(i);
        onRetry?.(lastError, i + 1);
        await sleep(delay);
      } else {
        break;
      }
    }
  }

  throw lastError;
}

/**
 * 格式化错误消息
 */
export function formatErrorMessage(error: Error | TxError | unknown): string {
  if (!error) {
    return '未知错误';
  }

  if (typeof error === 'object' && 'userMessage' in error) {
    return (error as TxError).userMessage;
  }

  if (error instanceof Error) {
    // 检查是否是已知错误
    for (const [code, message] of Object.entries(ERROR_MESSAGES)) {
      if (error.message.includes(code)) {
        return message;
      }
    }
    return error.message;
  }

  return String(error);
}

/**
 * 网络连接状态检查
 */
export interface ConnectionState {
  isConnected: boolean;
  isReconnecting: boolean;
  retryCount: number;
  lastError: string | null;
}

/**
 * 创建连接状态监听器
 */
export function createConnectionMonitor(
  api: ApiPromise,
  onStateChange: (state: ConnectionState) => void
): () => void {
  let state: ConnectionState = {
    isConnected: api.isConnected,
    isReconnecting: false,
    retryCount: 0,
    lastError: null,
  };

  const updateState = (updates: Partial<ConnectionState>) => {
    state = { ...state, ...updates };
    onStateChange(state);
  };

  const handleConnected = () => {
    updateState({
      isConnected: true,
      isReconnecting: false,
      retryCount: 0,
      lastError: null,
    });
  };

  const handleDisconnected = () => {
    updateState({
      isConnected: false,
      isReconnecting: true,
    });
  };

  const handleError = (error: Error) => {
    updateState({
      lastError: error.message,
      retryCount: state.retryCount + 1,
    });
  };

  api.on('connected', handleConnected);
  api.on('disconnected', handleDisconnected);
  api.on('error', handleError);

  // 返回清理函数
  return () => {
    api.off('connected', handleConnected);
    api.off('disconnected', handleDisconnected);
    api.off('error', handleError);
  };
}
