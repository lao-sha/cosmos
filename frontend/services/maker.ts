import { getApi } from './api';

export type MakerStatus = 'active' | 'paused' | 'pending' | 'rejected';

export interface MakerProfile {
  id: string;
  address: string;
  name: string;
  status: MakerStatus;
  depositAmount: string;
  availableCos: string;
  availableUsdt: string;
  price: string;
  minAmount: string;
  maxAmount: string;
  completedOrders: number;
  completionRate: number;
  totalVolume: string;
  earnings: string;
  paymentMethods: string[];
  paymentInfo: {
    bank?: { bankName: string; accountNumber: string; accountName: string };
    alipay?: { account: string; name: string };
    wechat?: { account: string; name: string };
  };
  createdAt: number;
}

export interface MakerStats {
  todayOrders: number;
  todayVolume: string;
  pendingOrders: number;
  disputeRate: number;
}

export async function getMakerProfile(address: string): Promise<MakerProfile | null> {
  try {
    const api = await getApi();
    const data = await api.query.maker.makerByAddress(address);
    
    if (data.isEmpty) return null;
    
    const maker = data.toJSON() as any;
    return {
      id: maker.id,
      address,
      name: maker.name || '',
      status: maker.status as MakerStatus,
      depositAmount: maker.depositAmount?.toString() || '0',
      availableCos: maker.availableCos?.toString() || '0',
      availableUsdt: maker.availableUsdt?.toString() || '0',
      price: maker.price?.toString() || '85000',
      minAmount: maker.minAmount?.toString() || '0',
      maxAmount: maker.maxAmount?.toString() || '0',
      completedOrders: maker.completedOrders || 0,
      completionRate: maker.completionRate || 100,
      totalVolume: maker.totalVolume?.toString() || '0',
      earnings: maker.earnings?.toString() || '0',
      paymentMethods: maker.paymentMethods || [],
      paymentInfo: maker.paymentInfo || {},
      createdAt: maker.createdAt || Date.now(),
    };
  } catch (error) {
    console.error('Failed to get maker profile:', error);
    return null;
  }
}

export async function getMakerStats(makerId: string): Promise<MakerStats> {
  return {
    todayOrders: 0,
    todayVolume: '0',
    pendingOrders: 0,
    disputeRate: 0,
  };
}

export async function applyMaker(
  name: string,
  depositAmount: string,
  paymentMethods: string[],
  paymentInfo: any,
  mnemonic: string
): Promise<string> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.maker
      .apply(name, depositAmount, paymentMethods, paymentInfo)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) {
            reject(new Error(dispatchError.toString()));
          } else {
            resolve(status.asFinalized.toHex());
          }
        }
      })
      .catch(reject);
  });
}

export async function updatePrice(
  price: string,
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.maker
      .updatePrice(price)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) reject(new Error(dispatchError.toString()));
          else resolve();
        }
      })
      .catch(reject);
  });
}

export async function deposit(
  amount: string,
  assetType: 'cos' | 'usdt',
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  const method = assetType === 'cos' ? 'depositCos' : 'depositUsdt';

  return new Promise((resolve, reject) => {
    api.tx.maker[method](amount)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) reject(new Error(dispatchError.toString()));
          else resolve();
        }
      })
      .catch(reject);
  });
}

export async function withdraw(
  amount: string,
  assetType: 'cos' | 'usdt',
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  const method = assetType === 'cos' ? 'withdrawCos' : 'withdrawUsdt';

  return new Promise((resolve, reject) => {
    api.tx.maker[method](amount)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) reject(new Error(dispatchError.toString()));
          else resolve();
        }
      })
      .catch(reject);
  });
}

export async function toggleStatus(
  active: boolean,
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  const method = active ? 'activate' : 'pause';

  return new Promise((resolve, reject) => {
    api.tx.maker[method]()
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) reject(new Error(dispatchError.toString()));
          else resolve();
        }
      })
      .catch(reject);
  });
}
