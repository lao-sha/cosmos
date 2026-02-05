import { ApiPromise, WsProvider } from '@polkadot/api';

const WS_ENDPOINTS = {
  local: 'ws://127.0.0.1:9944',
  testnet: 'wss://testnet.cosmos.network',
  mainnet: 'wss://rpc.cosmos.network',
};

let api: ApiPromise | null = null;
let connectionPromise: Promise<ApiPromise> | null = null;

export async function getApi(
  network: keyof typeof WS_ENDPOINTS = 'local'
): Promise<ApiPromise> {
  if (api?.isConnected) {
    return api;
  }

  if (connectionPromise) {
    return connectionPromise;
  }

  connectionPromise = (async () => {
    const provider = new WsProvider(WS_ENDPOINTS[network]);
    api = await ApiPromise.create({ provider });
    return api;
  })();

  return connectionPromise;
}

export async function disconnectApi(): Promise<void> {
  if (api) {
    await api.disconnect();
    api = null;
    connectionPromise = null;
  }
}

export function isConnected(): boolean {
  return api?.isConnected ?? false;
}
