import { getApi } from './api';

export type DisputeStatus = 'pending' | 'evidence' | 'arbitrating' | 'resolved' | 'appealed';
export type DisputeResult = 'plaintiff_wins' | 'defendant_wins' | 'settled' | 'cancelled';

export interface Dispute {
  id: string;
  domain: string;
  bizId: string;
  plaintiff: string;
  plaintiffName: string;
  defendant: string;
  defendantName: string;
  status: DisputeStatus;
  result?: DisputeResult;
  plaintiffDeposit: string;
  defendantDeposit: string;
  description: string;
  createdAt: number;
  evidenceDeadline: number;
  resolvedAt?: number;
}

export interface Evidence {
  id: string;
  disputeId: string;
  submitter: string;
  cid: string;
  description: string;
  accessMode: 'public' | 'arbitrator_only';
  submittedAt: number;
}

export interface Arbitrator {
  address: string;
  name: string;
  casesHandled: number;
  successRate: number;
  avgResolutionTime: number;
  isAvailable: boolean;
}

export async function getDisputes(address: string): Promise<Dispute[]> {
  try {
    const api = await getApi();
    const data = await api.query.arbitration.disputes.entries();
    
    return data
      .map(([key, value]: [any, any]) => {
        const id = key.args[0].toString();
        const d = value.toJSON() as any;
        return {
          id,
          domain: d.domain,
          bizId: d.bizId,
          plaintiff: d.plaintiff,
          plaintiffName: d.plaintiffName || '原告',
          defendant: d.defendant,
          defendantName: d.defendantName || '被告',
          status: d.status as DisputeStatus,
          result: d.result as DisputeResult,
          plaintiffDeposit: d.plaintiffDeposit?.toString() || '0',
          defendantDeposit: d.defendantDeposit?.toString() || '0',
          description: d.description || '',
          createdAt: d.createdAt || Date.now(),
          evidenceDeadline: d.evidenceDeadline || Date.now() + 3 * 24 * 3600000,
          resolvedAt: d.resolvedAt,
        };
      })
      .filter((d) => d.plaintiff === address || d.defendant === address);
  } catch (error) {
    console.error('Failed to get disputes:', error);
    return getMockDisputes();
  }
}

export async function getDisputeById(disputeId: string): Promise<Dispute | null> {
  try {
    const api = await getApi();
    const data = await api.query.arbitration.disputes(disputeId);
    
    if (data.isEmpty) return null;
    
    const d = data.toJSON() as any;
    return {
      id: disputeId,
      domain: d.domain,
      bizId: d.bizId,
      plaintiff: d.plaintiff,
      plaintiffName: d.plaintiffName || '原告',
      defendant: d.defendant,
      defendantName: d.defendantName || '被告',
      status: d.status as DisputeStatus,
      result: d.result as DisputeResult,
      plaintiffDeposit: d.plaintiffDeposit?.toString() || '0',
      defendantDeposit: d.defendantDeposit?.toString() || '0',
      description: d.description || '',
      createdAt: d.createdAt || Date.now(),
      evidenceDeadline: d.evidenceDeadline || Date.now() + 3 * 24 * 3600000,
      resolvedAt: d.resolvedAt,
    };
  } catch (error) {
    return null;
  }
}

export async function createDispute(
  domain: string,
  bizId: string,
  description: string,
  deposit: string,
  mnemonic: string
): Promise<string> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.arbitration
      .disputeWithTwoWayDeposit(domain, bizId, description, deposit)
      .signAndSend(pair, ({ status, events, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) {
            reject(new Error(dispatchError.toString()));
          } else {
            const event = events.find(({ event }) => event.method === 'DisputeCreated');
            resolve(event?.event.data[0]?.toString() || status.asFinalized.toHex());
          }
        }
      })
      .catch(reject);
  });
}

export async function submitEvidence(
  disputeId: string,
  cid: string,
  description: string,
  accessMode: 'public' | 'arbitrator_only',
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.evidence
      .commit(disputeId, cid, description, accessMode === 'public' ? 0 : 1)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) reject(new Error(dispatchError.toString()));
          else resolve();
        }
      })
      .catch(reject);
  });
}

export async function getEvidences(disputeId: string): Promise<Evidence[]> {
  try {
    const api = await getApi();
    const data = await api.query.evidence.evidences(disputeId);
    
    if (data.isEmpty) return [];
    
    return (data.toJSON() as any[]).map((e: any, index: number) => ({
      id: `${disputeId}-${index}`,
      disputeId,
      submitter: e.submitter,
      cid: e.cid,
      description: e.description || '',
      accessMode: e.accessMode === 0 ? 'public' : 'arbitrator_only',
      submittedAt: e.submittedAt || Date.now(),
    }));
  } catch (error) {
    return [];
  }
}

export async function respondToDispute(
  disputeId: string,
  deposit: string,
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.arbitration
      .respondToDispute(disputeId, deposit)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) reject(new Error(dispatchError.toString()));
          else resolve();
        }
      })
      .catch(reject);
  });
}

function getMockDisputes(): Dispute[] {
  return [
    {
      id: '1',
      domain: 'otc',
      bizId: 'order-123',
      plaintiff: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
      plaintiffName: '买家',
      defendant: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
      defendantName: '卖家',
      status: 'evidence',
      plaintiffDeposit: '1000000000000000',
      defendantDeposit: '1000000000000000',
      description: '已付款但卖家未放币，超过24小时',
      createdAt: Date.now() - 24 * 3600000,
      evidenceDeadline: Date.now() + 48 * 3600000,
    },
  ];
}

export const DISPUTE_STATUS_CONFIG = {
  pending: { label: '待响应', color: '#F59E0B' },
  evidence: { label: '举证中', color: '#3B82F6' },
  arbitrating: { label: '仲裁中', color: '#8B5CF6' },
  resolved: { label: '已解决', color: '#10B981' },
  appealed: { label: '已申诉', color: '#EF4444' },
};
