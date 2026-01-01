/**
 * 星尘玄鉴 - 错误类定义
 * 提供统一的错误处理机制
 */

/**
 * 基础错误类
 */
export class StardustError extends Error {
  constructor(
    message: string,
    public code?: string,
    public cause?: unknown
  ) {
    super(message);
    this.name = this.constructor.name;
  }
}

/**
 * 钱包相关错误
 */
export class WalletError extends StardustError {
  constructor(message: string, cause?: unknown) {
    super(message, 'WALLET_ERROR', cause);
  }
}

/**
 * 认证错误（密码错误）
 */
export class AuthenticationError extends StardustError {
  constructor(message: string = '密码错误') {
    super(message, 'AUTH_ERROR');
  }
}

/**
 * 加密错误
 */
export class CryptoError extends StardustError {
  constructor(message: string, cause?: unknown) {
    super(message, 'CRYPTO_ERROR', cause);
  }
}

/**
 * 链连接错误
 */
export class APIConnectionError extends StardustError {
  constructor(message: string = '无法连接到区块链节点', cause?: unknown) {
    super(message, 'API_CONNECTION_ERROR', cause);
  }
}

/**
 * 交易错误
 */
export class TransactionError extends StardustError {
  constructor(message: string, cause?: unknown) {
    super(message, 'TRANSACTION_ERROR', cause);
  }
}

/**
 * 网络错误
 */
export class NetworkError extends StardustError {
  constructor(message: string = '网络连接失败', cause?: unknown) {
    super(message, 'NETWORK_ERROR', cause);
  }
}

/**
 * 占卜错误
 */
export class DivinationError extends StardustError {
  constructor(message: string, cause?: unknown) {
    super(message, 'DIVINATION_ERROR', cause);
  }
}
