import { getApi } from './api';

export type KycLevel = 'none' | 'basic' | 'advanced' | 'enhanced';
export type KycStatus = 'none' | 'pending' | 'verified' | 'rejected' | 'expired';

export interface KycInfo {
  level: KycLevel;
  status: KycStatus;
  realName?: string;
  idNumber?: string;
  idType?: 'id_card' | 'passport';
  submittedAt?: number;
  verifiedAt?: number;
  rejectedReason?: string;
  expiresAt?: number;
}

export interface KycSubmission {
  realName: string;
  idNumber: string;
  idType: 'id_card' | 'passport';
  frontImageCid: string;
  backImageCid: string;
  selfieImageCid?: string;
}

export async function getKycInfo(address: string): Promise<KycInfo> {
  try {
    const api = await getApi();
    const data = await api.query.entityKyc.kycRecords(address);
    
    if (data.isEmpty) {
      return { level: 'none', status: 'none' };
    }
    
    const kyc = data.toJSON() as any;
    return {
      level: kyc.level as KycLevel,
      status: kyc.status as KycStatus,
      realName: kyc.realName,
      idNumber: kyc.idNumber ? maskIdNumber(kyc.idNumber) : undefined,
      idType: kyc.idType,
      submittedAt: kyc.submittedAt,
      verifiedAt: kyc.verifiedAt,
      rejectedReason: kyc.rejectedReason,
      expiresAt: kyc.expiresAt,
    };
  } catch (error) {
    console.error('Failed to get KYC info:', error);
    return { level: 'none', status: 'none' };
  }
}

export async function submitKyc(
  submission: KycSubmission,
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.entityKyc
      .submitKyc(
        submission.realName,
        submission.idNumber,
        submission.idType,
        submission.frontImageCid,
        submission.backImageCid,
        submission.selfieImageCid || null
      )
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

export async function uploadImage(
  imageUri: string,
  type: 'front' | 'back' | 'selfie'
): Promise<string> {
  // TODO: Implement IPFS upload
  // For now, return a mock CID
  return `Qm${type}${Date.now().toString(36)}`;
}

function maskIdNumber(idNumber: string): string {
  if (idNumber.length <= 6) return idNumber;
  const start = idNumber.slice(0, 3);
  const end = idNumber.slice(-3);
  const middle = '*'.repeat(idNumber.length - 6);
  return `${start}${middle}${end}`;
}

export const KYC_LEVEL_CONFIG = {
  none: {
    label: '未认证',
    limits: { daily: 0, single: 0 },
    features: [],
  },
  basic: {
    label: '基础认证',
    limits: { daily: 1000, single: 500 },
    features: ['小额交易', '基础功能'],
  },
  advanced: {
    label: '高级认证',
    limits: { daily: 10000, single: 5000 },
    features: ['大额交易', '做市商申请', '高级功能'],
  },
  enhanced: {
    label: '增强认证',
    limits: { daily: 100000, single: 50000 },
    features: ['无限额', '优先处理', 'VIP服务'],
  },
};
