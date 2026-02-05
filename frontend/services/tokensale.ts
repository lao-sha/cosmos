import { getApi } from './api';

export type SaleStatus = 'upcoming' | 'active' | 'ended' | 'cancelled';

export interface TokenSale {
  id: string;
  entityId: string;
  entityName: string;
  tokenSymbol: string;
  tokenName: string;
  description: string;
  totalSupply: string;
  saleAmount: string;
  soldAmount: string;
  price: string;
  minPurchase: string;
  maxPurchase: string;
  startTime: number;
  endTime: number;
  vestingPeriod: number;
  cliffPeriod: number;
  status: SaleStatus;
  participants: number;
  raised: string;
  softCap: string;
  hardCap: string;
}

export interface Participation {
  saleId: string;
  buyer: string;
  amount: string;
  tokenAmount: string;
  claimedAmount: string;
  purchasedAt: number;
  nextUnlockTime: number;
  unlockPercentage: number;
}

export async function getTokenSales(): Promise<TokenSale[]> {
  try {
    const api = await getApi();
    const data = await api.query.entitySale.sales.entries();
    
    return data.map(([key, value]: [any, any]) => {
      const id = key.args[0].toString();
      const s = value.toJSON() as any;
      return {
        id,
        entityId: s.entityId,
        entityName: s.entityName || '未知项目',
        tokenSymbol: s.tokenSymbol || 'TOKEN',
        tokenName: s.tokenName || '未知代币',
        description: s.description || '',
        totalSupply: s.totalSupply?.toString() || '0',
        saleAmount: s.saleAmount?.toString() || '0',
        soldAmount: s.soldAmount?.toString() || '0',
        price: s.price?.toString() || '0',
        minPurchase: s.minPurchase?.toString() || '0',
        maxPurchase: s.maxPurchase?.toString() || '0',
        startTime: s.startTime || Date.now(),
        endTime: s.endTime || Date.now() + 7 * 24 * 3600000,
        vestingPeriod: s.vestingPeriod || 0,
        cliffPeriod: s.cliffPeriod || 0,
        status: getSaleStatus(s.startTime, s.endTime, s.status),
        participants: s.participants || 0,
        raised: s.raised?.toString() || '0',
        softCap: s.softCap?.toString() || '0',
        hardCap: s.hardCap?.toString() || '0',
      };
    });
  } catch (error) {
    console.error('Failed to get token sales:', error);
    return getMockSales();
  }
}

export async function getSaleById(saleId: string): Promise<TokenSale | null> {
  try {
    const api = await getApi();
    const data = await api.query.entitySale.sales(saleId);
    
    if (data.isEmpty) return null;
    
    const s = data.toJSON() as any;
    return {
      id: saleId,
      entityId: s.entityId,
      entityName: s.entityName || '未知项目',
      tokenSymbol: s.tokenSymbol || 'TOKEN',
      tokenName: s.tokenName || '未知代币',
      description: s.description || '',
      totalSupply: s.totalSupply?.toString() || '0',
      saleAmount: s.saleAmount?.toString() || '0',
      soldAmount: s.soldAmount?.toString() || '0',
      price: s.price?.toString() || '0',
      minPurchase: s.minPurchase?.toString() || '0',
      maxPurchase: s.maxPurchase?.toString() || '0',
      startTime: s.startTime || Date.now(),
      endTime: s.endTime || Date.now() + 7 * 24 * 3600000,
      vestingPeriod: s.vestingPeriod || 0,
      cliffPeriod: s.cliffPeriod || 0,
      status: getSaleStatus(s.startTime, s.endTime, s.status),
      participants: s.participants || 0,
      raised: s.raised?.toString() || '0',
      softCap: s.softCap?.toString() || '0',
      hardCap: s.hardCap?.toString() || '0',
    };
  } catch (error) {
    return null;
  }
}

export async function participate(
  saleId: string,
  amount: string,
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.entitySale
      .participate(saleId, amount)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) reject(new Error(dispatchError.toString()));
          else resolve();
        }
      })
      .catch(reject);
  });
}

export async function claimTokens(saleId: string, mnemonic: string): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.entitySale
      .claimTokens(saleId)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) reject(new Error(dispatchError.toString()));
          else resolve();
        }
      })
      .catch(reject);
  });
}

export async function getMyParticipation(
  saleId: string,
  address: string
): Promise<Participation | null> {
  try {
    const api = await getApi();
    const data = await api.query.entitySale.participations(saleId, address);
    
    if (data.isEmpty) return null;
    
    const p = data.toJSON() as any;
    return {
      saleId,
      buyer: address,
      amount: p.amount?.toString() || '0',
      tokenAmount: p.tokenAmount?.toString() || '0',
      claimedAmount: p.claimedAmount?.toString() || '0',
      purchasedAt: p.purchasedAt || Date.now(),
      nextUnlockTime: p.nextUnlockTime || Date.now(),
      unlockPercentage: p.unlockPercentage || 0,
    };
  } catch (error) {
    return null;
  }
}

function getSaleStatus(startTime: number, endTime: number, status?: string): SaleStatus {
  if (status === 'cancelled') return 'cancelled';
  const now = Date.now();
  if (now < startTime) return 'upcoming';
  if (now > endTime) return 'ended';
  return 'active';
}

function getMockSales(): TokenSale[] {
  return [
    {
      id: '1',
      entityId: 'entity1',
      entityName: 'GameFi Project',
      tokenSymbol: 'GAME',
      tokenName: 'GameFi Token',
      description: '去中心化游戏平台代币，用于游戏内购买和治理投票。',
      totalSupply: '1000000000000000000000',
      saleAmount: '100000000000000000000',
      soldAmount: '75000000000000000000',
      price: '100000',
      minPurchase: '1000000000000000',
      maxPurchase: '10000000000000000',
      startTime: Date.now() - 2 * 24 * 3600000,
      endTime: Date.now() + 5 * 24 * 3600000,
      vestingPeriod: 180 * 24 * 3600000,
      cliffPeriod: 30 * 24 * 3600000,
      status: 'active',
      participants: 1234,
      raised: '75000000000',
      softCap: '50000000000',
      hardCap: '100000000000',
    },
    {
      id: '2',
      entityId: 'entity2',
      entityName: 'DeFi Protocol',
      tokenSymbol: 'DEFI',
      tokenName: 'DeFi Governance',
      description: '去中心化金融协议治理代币。',
      totalSupply: '500000000000000000000',
      saleAmount: '50000000000000000000',
      soldAmount: '0',
      price: '200000',
      minPurchase: '5000000000000000',
      maxPurchase: '50000000000000000',
      startTime: Date.now() + 3 * 24 * 3600000,
      endTime: Date.now() + 10 * 24 * 3600000,
      vestingPeriod: 365 * 24 * 3600000,
      cliffPeriod: 90 * 24 * 3600000,
      status: 'upcoming',
      participants: 0,
      raised: '0',
      softCap: '25000000000',
      hardCap: '50000000000',
    },
  ];
}
