import { getApi } from './api';

export type OrderStatus = 
  | 'pending'      // 等待支付
  | 'paid'         // 已支付，等待放币
  | 'released'     // 已放币
  | 'cancelled'    // 已取消
  | 'disputed';    // 争议中

export type OrderType = 'buy' | 'sell';

export interface Maker {
  id: string;
  address: string;
  name: string;
  availableCos: string;
  minAmount: string;
  maxAmount: string;
  price: string;         // USDT per COS
  completedOrders: number;
  completionRate: number; // 0-100
  paymentMethods: string[];
  isOnline: boolean;
}

export interface OtcOrder {
  id: string;
  makerId: string;
  makerAddress: string;
  makerName: string;
  takerAddress: string;
  orderType: OrderType;
  cosAmount: string;
  usdtAmount: string;
  price: string;
  status: OrderStatus;
  paymentMethod: string;
  paymentInfo?: {
    bankName?: string;
    accountNumber?: string;
    accountName?: string;
  };
  txHash?: string;
  createdAt: number;
  paidAt?: number;
  releasedAt?: number;
  expiresAt: number;
}

export async function getMakers(): Promise<Maker[]> {
  try {
    const api = await getApi();
    const entries = await api.query.maker.makers.entries();
    
    return entries.map(([key, value]: [any, any]) => {
      const makerId = key.args[0].toString();
      const data = value.toJSON() as any;
      
      return {
        id: makerId,
        address: data.owner,
        name: data.name || `Maker ${makerId.slice(0, 6)}`,
        availableCos: data.availableCos?.toString() || '0',
        minAmount: data.minAmount?.toString() || '100000000000000',
        maxAmount: data.maxAmount?.toString() || '10000000000000000',
        price: data.price?.toString() || '85000',
        completedOrders: data.completedOrders || 0,
        completionRate: data.completionRate || 100,
        paymentMethods: data.paymentMethods || ['bank', 'alipay', 'wechat'],
        isOnline: data.isOnline ?? true,
      };
    });
  } catch (error) {
    console.error('Failed to get makers:', error);
    // Return mock data for development
    return getMockMakers();
  }
}

export async function getMakerById(makerId: string): Promise<Maker | null> {
  const makers = await getMakers();
  return makers.find(m => m.id === makerId) || null;
}

export async function createBuyOrder(
  makerId: string,
  usdtAmount: string,
  mnemonic: string
): Promise<string> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.otcOrder
      .createBuyOrder(makerId, usdtAmount)
      .signAndSend(pair, ({ status, events, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) {
            reject(new Error(dispatchError.toString()));
          } else {
            // Find OrderCreated event
            const orderEvent = events.find(
              ({ event }) => event.method === 'OrderCreated'
            );
            if (orderEvent) {
              const orderId = orderEvent.event.data[0].toString();
              resolve(orderId);
            } else {
              resolve(status.asFinalized.toHex());
            }
          }
        }
      })
      .catch(reject);
  });
}

export async function confirmPayment(
  orderId: string,
  txHash: string,
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.otcOrder
      .confirmPayment(orderId, txHash)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) {
            reject(new Error(dispatchError.toString()));
          } else {
            resolve();
          }
        }
      })
      .catch(reject);
  });
}

export async function releaseCos(
  orderId: string,
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.otcOrder
      .releaseCos(orderId)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) {
            reject(new Error(dispatchError.toString()));
          } else {
            resolve();
          }
        }
      })
      .catch(reject);
  });
}

export async function cancelOrder(
  orderId: string,
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.otcOrder
      .cancelOrder(orderId)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) {
            reject(new Error(dispatchError.toString()));
          } else {
            resolve();
          }
        }
      })
      .catch(reject);
  });
}

export async function getOrders(address: string): Promise<OtcOrder[]> {
  try {
    const api = await getApi();
    const entries = await api.query.otcOrder.orders.entries();
    
    return entries
      .map(([key, value]: [any, any]) => {
        const orderId = key.args[0].toString();
        const data = value.toJSON() as any;
        
        return {
          id: orderId,
          makerId: data.makerId,
          makerAddress: data.makerAddress,
          makerName: data.makerName || 'Maker',
          takerAddress: data.takerAddress,
          orderType: data.orderType as OrderType,
          cosAmount: data.cosAmount?.toString() || '0',
          usdtAmount: data.usdtAmount?.toString() || '0',
          price: data.price?.toString() || '0',
          status: data.status as OrderStatus,
          paymentMethod: data.paymentMethod || 'bank',
          paymentInfo: data.paymentInfo,
          txHash: data.txHash,
          createdAt: data.createdAt || Date.now(),
          paidAt: data.paidAt,
          releasedAt: data.releasedAt,
          expiresAt: data.expiresAt || Date.now() + 15 * 60 * 1000,
        };
      })
      .filter(order => 
        order.takerAddress === address || order.makerAddress === address
      );
  } catch (error) {
    console.error('Failed to get orders:', error);
    return [];
  }
}

export async function getOrderById(orderId: string): Promise<OtcOrder | null> {
  try {
    const api = await getApi();
    const data = await api.query.otcOrder.orders(orderId);
    
    if (data.isEmpty) return null;
    
    const order = data.toJSON() as any;
    return {
      id: orderId,
      makerId: order.makerId,
      makerAddress: order.makerAddress,
      makerName: order.makerName || 'Maker',
      takerAddress: order.takerAddress,
      orderType: order.orderType as OrderType,
      cosAmount: order.cosAmount?.toString() || '0',
      usdtAmount: order.usdtAmount?.toString() || '0',
      price: order.price?.toString() || '0',
      status: order.status as OrderStatus,
      paymentMethod: order.paymentMethod || 'bank',
      paymentInfo: order.paymentInfo,
      txHash: order.txHash,
      createdAt: order.createdAt || Date.now(),
      paidAt: order.paidAt,
      releasedAt: order.releasedAt,
      expiresAt: order.expiresAt || Date.now() + 15 * 60 * 1000,
    };
  } catch (error) {
    console.error('Failed to get order:', error);
    return null;
  }
}

// Mock data for development
function getMockMakers(): Maker[] {
  return [
    {
      id: '1',
      address: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
      name: '金牌商家',
      availableCos: '50000000000000000',
      minAmount: '100000000000000',
      maxAmount: '10000000000000000',
      price: '85000',
      completedOrders: 1234,
      completionRate: 99.8,
      paymentMethods: ['bank', 'alipay', 'wechat'],
      isOnline: true,
    },
    {
      id: '2',
      address: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
      name: '信誉商家',
      availableCos: '30000000000000000',
      minAmount: '500000000000000',
      maxAmount: '5000000000000000',
      price: '84500',
      completedOrders: 856,
      completionRate: 99.5,
      paymentMethods: ['bank', 'alipay'],
      isOnline: true,
    },
    {
      id: '3',
      address: '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y',
      name: '优质商家',
      availableCos: '20000000000000000',
      minAmount: '200000000000000',
      maxAmount: '8000000000000000',
      price: '85500',
      completedOrders: 421,
      completionRate: 98.9,
      paymentMethods: ['bank', 'wechat'],
      isOnline: false,
    },
  ];
}
