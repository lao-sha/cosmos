/**
 * 星尘玄鉴 - Polkadot API 配置与初始化 (React Native 版本)
 * 不依赖浏览器扩展
 */

import { ApiPromise, WsProvider } from '@polkadot/api';

// API 配置
const WS_PROVIDER_URL = process.env.EXPO_PUBLIC_WS_PROVIDER || process.env.EXPO_PUBLIC_WS_ENDPOINT || 'ws://127.0.0.1:9944';
const APP_NAME = 'Stardust';

/**
 * API 管理器
 */
class ApiManager {
  private static instance: ApiManager;
  private api: ApiPromise | null = null;
  private provider: WsProvider | null = null;
  private isInitializing = false;
  private initPromise: Promise<ApiPromise> | null = null;

  private constructor() {}

  /**
   * 获取单例实例
   */
  static getInstance(): ApiManager {
    if (!ApiManager.instance) {
      ApiManager.instance = new ApiManager();
    }
    return ApiManager.instance;
  }

  /**
   * 初始化 API
   */
  async initialize(): Promise<ApiPromise> {
    // 如果已经初始化，直接返回
    if (this.api) {
      return this.api;
    }

    // 如果正在初始化，等待初始化完成
    if (this.isInitializing && this.initPromise) {
      return this.initPromise;
    }

    // 开始初始化
    this.isInitializing = true;
    this.initPromise = this._doInitialize();

    try {
      this.api = await this.initPromise;
      return this.api;
    } finally {
      this.isInitializing = false;
      this.initPromise = null;
    }
  }

  /**
   * 执行初始化
   */
  private async _doInitialize(): Promise<ApiPromise> {
    console.log('[API] Initializing Polkadot API...');
    console.log('[API] Provider URL:', WS_PROVIDER_URL);

    try {
      // 创建 WebSocket Provider
      this.provider = new WsProvider(WS_PROVIDER_URL);

      // 创建 API 实例
      const api = await ApiPromise.create({
        provider: this.provider,
        types: {
          // 自定义类型定义（如果需要）
        },
      });

      // 等待 API 准备就绪
      await api.isReady;

      console.log('[API] Connected to chain:', await api.rpc.system.chain());
      console.log('[API] Node version:', await api.rpc.system.version());

      return api;
    } catch (error) {
      console.error('[API] Initialization failed:', error);
      throw new Error('Failed to connect to blockchain node');
    }
  }

  /**
   * 获取 API 实例
   */
  getApi(): ApiPromise {
    if (!this.api) {
      throw new Error('API not initialized. Call initialize() first.');
    }
    return this.api;
  }

  /**
   * 检查 API 是否已初始化
   */
  isInitialized(): boolean {
    return this.api !== null;
  }

  /**
   * 断开连接
   */
  async disconnect(): Promise<void> {
    if (this.api) {
      await this.api.disconnect();
      this.api = null;
    }
    if (this.provider) {
      await this.provider.disconnect();
      this.provider = null;
    }
    console.log('[API] Disconnected');
  }

  /**
   * 重新连接
   */
  async reconnect(): Promise<ApiPromise> {
    await this.disconnect();
    return this.initialize();
  }
}

// 导出单例实例
export const apiManager = ApiManager.getInstance();

// 导出便捷方法
export const initializeApi = () => apiManager.initialize();
export const getApi = () => apiManager.getApi();
export const isApiInitialized = () => apiManager.isInitialized();

// React Native 平台说明：
// - 不支持浏览器扩展
// - 账户管理通过 wallet.store.ts 处理
// - 交易签名通过 signer.native.ts 处理
