import { getApi } from './api';

export interface ReferralInfo {
  code: string;
  referrer?: string;
  referrerName?: string;
  totalReferrals: number;
  directReferrals: number;
  totalEarnings: string;
  pendingEarnings: string;
  level: number;
}

export interface ReferralMember {
  address: string;
  name: string;
  level: number;
  joinedAt: number;
  contribution: string;
  isActive: boolean;
}

export interface CommissionRecord {
  id: string;
  fromAddress: string;
  fromName: string;
  amount: string;
  type: 'trade' | 'referral' | 'bonus';
  level: number;
  createdAt: number;
}

export async function getReferralInfo(address: string): Promise<ReferralInfo> {
  try {
    const api = await getApi();
    const data = await api.query.referral.referralInfo(address);
    
    if (data.isEmpty) {
      return {
        code: generateReferralCode(address),
        totalReferrals: 0,
        directReferrals: 0,
        totalEarnings: '0',
        pendingEarnings: '0',
        level: 1,
      };
    }
    
    const info = data.toJSON() as any;
    return {
      code: info.code || generateReferralCode(address),
      referrer: info.referrer,
      referrerName: info.referrerName,
      totalReferrals: info.totalReferrals || 0,
      directReferrals: info.directReferrals || 0,
      totalEarnings: info.totalEarnings?.toString() || '0',
      pendingEarnings: info.pendingEarnings?.toString() || '0',
      level: info.level || 1,
    };
  } catch (error) {
    console.error('Failed to get referral info:', error);
    return {
      code: generateReferralCode(address),
      totalReferrals: 0,
      directReferrals: 0,
      totalEarnings: '0',
      pendingEarnings: '0',
      level: 1,
    };
  }
}

export async function getTeamMembers(
  address: string,
  level?: number
): Promise<ReferralMember[]> {
  try {
    const api = await getApi();
    const data = await api.query.referral.teamMembers(address);
    
    if (data.isEmpty) return [];
    
    const members = data.toJSON() as any[];
    return members
      .filter((m: any) => !level || m.level === level)
      .map((m: any) => ({
        address: m.address,
        name: m.name || '用户',
        level: m.level || 1,
        joinedAt: m.joinedAt || Date.now(),
        contribution: m.contribution?.toString() || '0',
        isActive: m.isActive ?? true,
      }));
  } catch (error) {
    console.error('Failed to get team members:', error);
    return [];
  }
}

export async function getCommissionRecords(
  address: string,
  limit = 50
): Promise<CommissionRecord[]> {
  try {
    const api = await getApi();
    const data = await api.query.affiliate.commissionRecords(address);
    
    if (data.isEmpty) return [];
    
    const records = data.toJSON() as any[];
    return records.slice(0, limit).map((r: any) => ({
      id: r.id,
      fromAddress: r.fromAddress,
      fromName: r.fromName || '用户',
      amount: r.amount?.toString() || '0',
      type: r.type || 'trade',
      level: r.level || 1,
      createdAt: r.createdAt || Date.now(),
    }));
  } catch (error) {
    console.error('Failed to get commission records:', error);
    return [];
  }
}

export async function bindReferrer(
  referralCode: string,
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.referral
      .bindReferrer(referralCode)
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

export async function withdrawCommission(
  amount: string,
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.affiliate
      .withdrawCommission(amount)
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

function generateReferralCode(address: string): string {
  const hash = address.slice(-8).toUpperCase();
  return `COS${hash}`;
}

export const REFERRAL_LEVELS = [
  { level: 1, rate: 0.3, label: '直推' },
  { level: 2, rate: 0.2, label: '二级' },
  { level: 3, rate: 0.1, label: '三级' },
  { level: 4, rate: 0.05, label: '四级' },
  { level: 5, rate: 0.05, label: '五级' },
  { level: 6, rate: 0.03, label: '六级' },
  { level: 7, rate: 0.03, label: '七级' },
  { level: 8, rate: 0.02, label: '八级' },
  { level: 9, rate: 0.02, label: '九级' },
  { level: 10, rate: 0.01, label: '十级' },
];
