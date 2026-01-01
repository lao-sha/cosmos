/**
 * 星尘玄鉴 - 区块链连接管理（Mock 版本）
 * Web 测试用
 */

// Mock 连接状态
let connected = false;

/**
 * 获取 API 实例（Mock）
 */
export async function getApi(): Promise<null> {
  console.log('[API] Mock getApi called');
  connected = true;
  return null;
}

/**
 * 监听网络变化
 */
export function setupNetworkListener(): void {
  console.log('[Network] Mock network listener setup');
}

/**
 * 断开连接
 */
export async function disconnectApi(): Promise<void> {
  console.log('[API] Mock disconnect');
  connected = false;
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
  return 'ws://mock:9944';
}
