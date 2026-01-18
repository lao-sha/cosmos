/**
 * 星尘玄鉴 - 区块链连接管理（原生版本）
 * 使用真实的 @polkadot/api 连接
 */

import { ApiPromise, WsProvider } from '@polkadot/api';

// 默认节点地址 - 优先使用环境变量
const DEFAULT_ENDPOINT = process.env.EXPO_PUBLIC_WS_ENDPOINT || process.env.EXPO_PUBLIC_WS_PROVIDER || 'ws://127.0.0.1:9944';

// 全局 API 实例
let api: ApiPromise | null = null;
let provider: WsProvider | null = null;
let currentEndpoint: string = DEFAULT_ENDPOINT;

/**
 * 获取 API 实例
 */
export async function getApi(): Promise<ApiPromise> {
  if (api && api.isConnected) {
    return api;
  }

  try {
    console.log('[API] Connecting to:', currentEndpoint);

    provider = new WsProvider(currentEndpoint);
    api = await ApiPromise.create({ provider });

    // 监听连接状态
    api.on('connected', () => {
      console.log('[API] Connected to chain');
    });

    api.on('disconnected', () => {
      console.log('[API] Disconnected from chain');
    });

    api.on('error', (error) => {
      console.error('[API] Connection error:', error);
    });

    console.log('[API] Connected successfully');
    console.log('[API] Chain:', (await api.rpc.system.chain()).toString());
    console.log('[API] Node:', (await api.rpc.system.name()).toString());

    return api;
  } catch (error) {
    console.error('[API] Failed to connect:', error);
    throw error;
  }
}

/**
 * 监听网络变化
 */
export function setupNetworkListener(): void {
  // 在原生环境中可以使用 NetInfo 监听网络变化
  console.log('[Network] Network listener setup');
}

/**
 * 断开连接
 */
export async function disconnectApi(): Promise<void> {
  if (api) {
    await api.disconnect();
    api = null;
    provider = null;
    console.log('[API] Disconnected');
  }
}

/**
 * 检查连接状态
 */
export function isConnected(): boolean {
  return api?.isConnected ?? false;
}

/**
 * 获取当前连接的节点地址
 */
export function getEndpoint(): string {
  return currentEndpoint;
}

/**
 * 切换节点
 */
export async function switchEndpoint(endpoint: string): Promise<void> {
  currentEndpoint = endpoint;
  await disconnectApi();
  await getApi();
}

/**
 * 查询账户余额
 */
export async function getBalance(address: string): Promise<string> {
  const apiInstance = await getApi();
  const { data: { free } } = await apiInstance.query.system.account(address) as any;
  return free.toString();
}

/**
 * 订阅账户余额变化
 */
export async function subscribeBalance(
  address: string,
  callback: (balance: string) => void
): Promise<() => void> {
  const apiInstance = await getApi();
  const unsub = await apiInstance.query.system.account(address, ({ data: { free } }: any) => {
    callback(free.toString());
  });
  return unsub;
}
