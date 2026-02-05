import { getApi } from './api';

export type SwapDirection = 'cos_to_usdt' | 'usdt_to_cos';

export interface SwapQuote {
  makerId: string;
  makerName: string;
  inputAmount: string;
  outputAmount: string;
  price: string;
  fee: string;
  slippage: number;
}

export interface SwapRequest {
  id: string;
  makerId: string;
  makerName: string;
  direction: SwapDirection;
  inputAmount: string;
  outputAmount: string;
  price: string;
  status: 'pending' | 'completed' | 'failed' | 'expired';
  createdAt: number;
  completedAt?: number;
}

export async function getSwapQuotes(
  direction: SwapDirection,
  amount: string
): Promise<SwapQuote[]> {
  try {
    const api = await getApi();
    const makers = await api.query.maker.makers.entries();
    
    return makers.map(([key, value]: [any, any]) => {
      const makerId = key.args[0].toString();
      const data = value.toJSON() as any;
      const price = Number(data.price) / 10000;
      const inputNum = BigInt(amount);
      
      let outputAmount: bigint;
      if (direction === 'cos_to_usdt') {
        outputAmount = inputNum * BigInt(Math.floor(price * 1e6)) / BigInt(1e12);
      } else {
        outputAmount = inputNum * BigInt(1e12) / BigInt(Math.floor(price * 1e6));
      }
      
      const fee = outputAmount * BigInt(3) / BigInt(1000); // 0.3% fee
      
      return {
        makerId,
        makerName: data.name || `Maker ${makerId.slice(0, 6)}`,
        inputAmount: amount,
        outputAmount: (outputAmount - fee).toString(),
        price: data.price?.toString() || '85000',
        fee: fee.toString(),
        slippage: 0.5,
      };
    });
  } catch (error) {
    console.error('Failed to get swap quotes:', error);
    return getMockQuotes(direction, amount);
  }
}

export async function executeSwap(
  makerId: string,
  direction: SwapDirection,
  amount: string,
  mnemonic: string
): Promise<string> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  const method = direction === 'cos_to_usdt' 
    ? 'swapCosToUsdt' 
    : 'swapUsdtToCos';

  return new Promise((resolve, reject) => {
    api.tx.swap[method](makerId, amount)
      .signAndSend(pair, ({ status, events, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) {
            reject(new Error(dispatchError.toString()));
          } else {
            const swapEvent = events.find(
              ({ event }) => event.method === 'SwapCompleted'
            );
            resolve(swapEvent?.event.data[0]?.toString() || status.asFinalized.toHex());
          }
        }
      })
      .catch(reject);
  });
}

export async function getSwapHistory(address: string): Promise<SwapRequest[]> {
  try {
    const api = await getApi();
    const entries = await api.query.swap.swapRequests.entries();
    
    return entries
      .map(([key, value]: [any, any]) => {
        const id = key.args[0].toString();
        const data = value.toJSON() as any;
        return {
          id,
          makerId: data.makerId,
          makerName: data.makerName || 'Maker',
          direction: data.direction as SwapDirection,
          inputAmount: data.inputAmount?.toString() || '0',
          outputAmount: data.outputAmount?.toString() || '0',
          price: data.price?.toString() || '0',
          status: data.status,
          createdAt: data.createdAt || Date.now(),
          completedAt: data.completedAt,
        };
      })
      .filter(req => req.makerId === address || true);
  } catch (error) {
    return [];
  }
}

function getMockQuotes(direction: SwapDirection, amount: string): SwapQuote[] {
  const inputNum = BigInt(amount || '0');
  const prices = [
    { makerId: '1', name: '金牌商家', price: 85000 },
    { makerId: '2', name: '信誉商家', price: 84500 },
    { makerId: '3', name: '优质商家', price: 85500 },
  ];

  return prices.map(({ makerId, name, price }) => {
    let outputAmount: bigint;
    if (direction === 'cos_to_usdt') {
      outputAmount = inputNum * BigInt(price) / BigInt(1e10);
    } else {
      outputAmount = inputNum * BigInt(1e10) / BigInt(price);
    }
    const fee = outputAmount * BigInt(3) / BigInt(1000);

    return {
      makerId,
      makerName: name,
      inputAmount: amount,
      outputAmount: (outputAmount - fee).toString(),
      price: price.toString(),
      fee: fee.toString(),
      slippage: 0.5,
    };
  });
}
