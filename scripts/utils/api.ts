import { ApiPromise, WsProvider } from '@polkadot/api';

const DEFAULT_WS_URL = 'ws://127.0.0.1:9944';

let apiInstance: ApiPromise | null = null;

export async function getApi(wsUrl: string = DEFAULT_WS_URL): Promise<ApiPromise> {
  if (apiInstance && apiInstance.isConnected) {
    return apiInstance;
  }

  console.log(`🔗 连接到节点: ${wsUrl}`);
  const provider = new WsProvider(wsUrl);
  apiInstance = await ApiPromise.create({ provider });
  
  const chain = await apiInstance.rpc.system.chain();
  const version = await apiInstance.rpc.system.version();
  console.log(`✅ 已连接到 ${chain} (${version})`);
  
  return apiInstance;
}

export async function disconnectApi(): Promise<void> {
  if (apiInstance) {
    await apiInstance.disconnect();
    apiInstance = null;
    console.log('🔌 已断开连接');
  }
}

export async function getCurrentBlock(api: ApiPromise): Promise<number> {
  const header = await api.rpc.chain.getHeader();
  return header.number.toNumber();
}

export async function waitForBlocks(api: ApiPromise, blocks: number): Promise<void> {
  const startBlock = await getCurrentBlock(api);
  const targetBlock = startBlock + blocks;
  
  console.log(`⏳ 等待 ${blocks} 个区块 (当前: ${startBlock}, 目标: ${targetBlock})`);
  
  return new Promise((resolve) => {
    const unsub = api.rpc.chain.subscribeNewHeads((header) => {
      const currentBlock = header.number.toNumber();
      if (currentBlock >= targetBlock) {
        unsub.then(u => u());
        console.log(`✅ 已到达区块 ${currentBlock}`);
        resolve();
      }
    });
  });
}
