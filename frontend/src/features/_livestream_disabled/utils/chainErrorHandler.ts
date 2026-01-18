// frontend/src/features/livestream/utils/chainErrorHandler.ts

import { Alert } from 'react-native';

/**
 * 链上错误类型
 */
export enum ChainErrorType {
  InsufficientBalance = 'InsufficientBalance',
  RoomNotFound = 'RoomNotFound',
  NotRoomHost = 'NotRoomHost',
  RoomNotLive = 'RoomNotLive',
  AlreadyLive = 'AlreadyLive',
  TicketRequired = 'TicketRequired',
  AlreadyHasTicket = 'AlreadyHasTicket',
  ViewerBanned = 'ViewerBanned',
  GiftNotFound = 'GiftNotFound',
  GiftDisabled = 'GiftDisabled',
  InvalidQuantity = 'InvalidQuantity',
  CoHostLimitReached = 'CoHostLimitReached',
  NotCoHost = 'NotCoHost',
  NetworkError = 'NetworkError',
  SignatureError = 'SignatureError',
  Unknown = 'Unknown',
}

/**
 * 错误信息映射
 */
const ERROR_MESSAGES: Record<
  ChainErrorType,
  { title: string; message: string; action?: string }
> = {
  [ChainErrorType.InsufficientBalance]: {
    title: '余额不足',
    message: '您的 DUST 余额不足，请先充值',
    action: 'recharge',
  },
  [ChainErrorType.RoomNotFound]: {
    title: '直播间不存在',
    message: '该直播间已关闭或不存在',
  },
  [ChainErrorType.NotRoomHost]: {
    title: '权限不足',
    message: '您不是该直播间的主播',
  },
  [ChainErrorType.RoomNotLive]: {
    title: '直播未开始',
    message: '主播还未开始直播',
  },
  [ChainErrorType.AlreadyLive]: {
    title: '操作失败',
    message: '直播已经开始',
  },
  [ChainErrorType.TicketRequired]: {
    title: '需要门票',
    message: '这是付费直播，请先购买门票',
    action: 'buyTicket',
  },
  [ChainErrorType.AlreadyHasTicket]: {
    title: '已有门票',
    message: '您已经购买过门票了',
  },
  [ChainErrorType.ViewerBanned]: {
    title: '无法进入',
    message: '您已被主播禁止进入该直播间',
  },
  [ChainErrorType.GiftNotFound]: {
    title: '礼物不存在',
    message: '该礼物已下架',
  },
  [ChainErrorType.GiftDisabled]: {
    title: '礼物不可用',
    message: '该礼物暂时不可用',
  },
  [ChainErrorType.InvalidQuantity]: {
    title: '数量无效',
    message: '请输入有效的数量',
  },
  [ChainErrorType.CoHostLimitReached]: {
    title: '连麦人数已满',
    message: '当前直播间连麦人数已达上限',
  },
  [ChainErrorType.NotCoHost]: {
    title: '操作失败',
    message: '您不是当前连麦者',
  },
  [ChainErrorType.NetworkError]: {
    title: '网络错误',
    message: '网络连接失败，请检查网络后重试',
  },
  [ChainErrorType.SignatureError]: {
    title: '签名失败',
    message: '钱包签名失败，请重试',
  },
  [ChainErrorType.Unknown]: {
    title: '操作失败',
    message: '发生未知错误，请稍后重试',
  },
};

/**
 * 解析链上错误
 */
export function parseChainError(error: any): ChainErrorType {
  const errorString = error?.message || error?.toString() || '';

  if (
    errorString.includes('InsufficientBalance') ||
    errorString.includes('Arithmetic')
  ) {
    return ChainErrorType.InsufficientBalance;
  }
  if (errorString.includes('RoomNotFound')) {
    return ChainErrorType.RoomNotFound;
  }
  if (errorString.includes('NotRoomHost') || errorString.includes('NotHost')) {
    return ChainErrorType.NotRoomHost;
  }
  if (errorString.includes('RoomNotLive') || errorString.includes('NotLive')) {
    return ChainErrorType.RoomNotLive;
  }
  if (errorString.includes('AlreadyLive')) {
    return ChainErrorType.AlreadyLive;
  }
  if (
    errorString.includes('TicketRequired') ||
    errorString.includes('NoTicket')
  ) {
    return ChainErrorType.TicketRequired;
  }
  if (errorString.includes('AlreadyHasTicket')) {
    return ChainErrorType.AlreadyHasTicket;
  }
  if (
    errorString.includes('Banned') ||
    errorString.includes('Blacklisted')
  ) {
    return ChainErrorType.ViewerBanned;
  }
  if (errorString.includes('GiftNotFound')) {
    return ChainErrorType.GiftNotFound;
  }
  if (errorString.includes('GiftDisabled')) {
    return ChainErrorType.GiftDisabled;
  }
  if (
    errorString.includes('InvalidQuantity') ||
    errorString.includes('ZeroQuantity')
  ) {
    return ChainErrorType.InvalidQuantity;
  }
  if (errorString.includes('CoHostLimit')) {
    return ChainErrorType.CoHostLimitReached;
  }
  if (errorString.includes('NotCoHost')) {
    return ChainErrorType.NotCoHost;
  }
  if (
    errorString.includes('network') ||
    errorString.includes('timeout') ||
    errorString.includes('ECONNREFUSED')
  ) {
    return ChainErrorType.NetworkError;
  }
  if (errorString.includes('signature') || errorString.includes('sign')) {
    return ChainErrorType.SignatureError;
  }

  return ChainErrorType.Unknown;
}

/**
 * 获取错误信息
 */
export function getErrorInfo(errorType: ChainErrorType) {
  return ERROR_MESSAGES[errorType];
}

/**
 * 处理链上错误 (显示提示)
 */
export function handleChainError(
  error: any,
  options?: {
    onRecharge?: () => void;
    onBuyTicket?: () => void;
    onRetry?: () => void;
  }
): ChainErrorType {
  const errorType = parseChainError(error);
  const errorInfo = ERROR_MESSAGES[errorType];

  console.error('[ChainError]', errorType, error);

  // 根据错误类型显示不同的提示
  if (errorInfo.action === 'recharge' && options?.onRecharge) {
    Alert.alert(errorInfo.title, errorInfo.message, [
      { text: '取消', style: 'cancel' },
      { text: '去充值', onPress: options.onRecharge },
    ]);
  } else if (errorInfo.action === 'buyTicket' && options?.onBuyTicket) {
    Alert.alert(errorInfo.title, errorInfo.message, [
      { text: '取消', style: 'cancel' },
      { text: '购买门票', onPress: options.onBuyTicket },
    ]);
  } else if (errorType === ChainErrorType.NetworkError && options?.onRetry) {
    Alert.alert(errorInfo.title, errorInfo.message, [
      { text: '取消', style: 'cancel' },
      { text: '重试', onPress: options.onRetry },
    ]);
  } else {
    Alert.alert(errorInfo.title, errorInfo.message);
  }

  return errorType;
}

/**
 * 包装链上交易的错误处理
 */
export async function withChainErrorHandling<T>(
  operation: () => Promise<T>,
  options?: {
    onRecharge?: () => void;
    onBuyTicket?: () => void;
    onRetry?: () => void;
    loadingMessage?: string;
  }
): Promise<T | null> {
  try {
    const result = await operation();
    return result;
  } catch (error) {
    handleChainError(error, options);
    return null;
  }
}
