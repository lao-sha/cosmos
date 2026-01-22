/**
 * 星尘玄鉴 - 区块链连接管理（Web 版本）
 * Web 环境下的模拟实现
 */

// 默认节点地址
const DEFAULT_ENDPOINT = 'ws://127.0.0.1:9944';

// 模拟 API 状态
let connected = false;
let currentEndpoint: string = DEFAULT_ENDPOINT;

// 模拟 API 对象
const mockApi = {
  isConnected: true,
  on: (event: string, callback: Function) => {},
  disconnect: async () => { connected = false; },
  rpc: {
    system: {
      chain: async () => ({ toString: () => 'Development' }),
      name: async () => ({ toString: () => 'Stardust Node' }),
    }
  },
  query: {
    system: {
      account: async (address: string, callback?: Function) => {
        const mockBalance = { data: { free: { toString: () => '1000000000000' } } };
        if (callback) {
          callback(mockBalance);
          return () => {};
        }
        return mockBalance;
      }
    }
  }
};

/**
 * 获取 API 实例
 */
export async function getApi(): Promise<typeof mockApi> {
  if (connected) {
    return mockApi;
  }

  console.log('[API-Web] Connecting to:', currentEndpoint);
  connected = true;
  console.log('[API-Web] Connected (mock mode for web)');

  return mockApi;
}

/**
 * 监听网络变化
 */
export function setupNetworkListener(): void {
  console.log('[Network-Web] Network listener setup (mock)');
}

/**
 * 断开连接
 */
export async function disconnectApi(): Promise<void> {
  connected = false;
  console.log('[API-Web] Disconnected');
}

/**
 * 检查连接状态
 */
export function isConnected(): boolean {
  return connected;
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
  console.log('[API-Web] Getting balance for:', address);
  return '1000000000000';
}

/**
 * 订阅账户余额变化
 */
export async function subscribeBalance(
  address: string,
  callback: (balance: string) => void
): Promise<() => void> {
  console.log('[API-Web] Subscribing to balance for:', address);
  callback('1000000000000');
  return () => {};
}
