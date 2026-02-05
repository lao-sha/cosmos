import { getApi } from './api';

export type ProposalStatus = 'proposing' | 'voting' | 'passed' | 'rejected' | 'executed' | 'cancelled';
export type VoteType = 'yes' | 'no' | 'abstain';

export interface Proposal {
  id: string;
  entityId: string;
  entityName: string;
  title: string;
  description: string;
  proposer: string;
  proposerName: string;
  status: ProposalStatus;
  votesYes: string;
  votesNo: string;
  votesAbstain: string;
  totalVotes: string;
  quorum: number;
  threshold: number;
  startTime: number;
  endTime: number;
  executedAt?: number;
  createdAt: number;
}

export interface VoteRecord {
  proposalId: string;
  voter: string;
  voteType: VoteType;
  amount: string;
  timestamp: number;
}

export async function getProposals(entityId?: string): Promise<Proposal[]> {
  try {
    const api = await getApi();
    const data = await api.query.entityGovernance.proposals.entries();
    
    return data
      .map(([key, value]: [any, any]) => {
        const id = key.args[0].toString();
        const p = value.toJSON() as any;
        return {
          id,
          entityId: p.entityId,
          entityName: p.entityName || '未知实体',
          title: p.title || '',
          description: p.description || '',
          proposer: p.proposer,
          proposerName: p.proposerName || '未知',
          status: p.status as ProposalStatus,
          votesYes: p.votesYes?.toString() || '0',
          votesNo: p.votesNo?.toString() || '0',
          votesAbstain: p.votesAbstain?.toString() || '0',
          totalVotes: p.totalVotes?.toString() || '0',
          quorum: p.quorum || 50,
          threshold: p.threshold || 50,
          startTime: p.startTime || Date.now(),
          endTime: p.endTime || Date.now() + 7 * 24 * 3600000,
          executedAt: p.executedAt,
          createdAt: p.createdAt || Date.now(),
        };
      })
      .filter((p) => !entityId || p.entityId === entityId);
  } catch (error) {
    console.error('Failed to get proposals:', error);
    return getMockProposals();
  }
}

export async function getProposalById(proposalId: string): Promise<Proposal | null> {
  try {
    const api = await getApi();
    const data = await api.query.entityGovernance.proposals(proposalId);
    
    if (data.isEmpty) return null;
    
    const p = data.toJSON() as any;
    return {
      id: proposalId,
      entityId: p.entityId,
      entityName: p.entityName || '未知实体',
      title: p.title || '',
      description: p.description || '',
      proposer: p.proposer,
      proposerName: p.proposerName || '未知',
      status: p.status as ProposalStatus,
      votesYes: p.votesYes?.toString() || '0',
      votesNo: p.votesNo?.toString() || '0',
      votesAbstain: p.votesAbstain?.toString() || '0',
      totalVotes: p.totalVotes?.toString() || '0',
      quorum: p.quorum || 50,
      threshold: p.threshold || 50,
      startTime: p.startTime || Date.now(),
      endTime: p.endTime || Date.now() + 7 * 24 * 3600000,
      executedAt: p.executedAt,
      createdAt: p.createdAt || Date.now(),
    };
  } catch (error) {
    return null;
  }
}

export async function createProposal(
  entityId: string,
  title: string,
  description: string,
  mnemonic: string
): Promise<string> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.entityGovernance
      .createProposal(entityId, title, description)
      .signAndSend(pair, ({ status, events, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) {
            reject(new Error(dispatchError.toString()));
          } else {
            const event = events.find(({ event }) => event.method === 'ProposalCreated');
            resolve(event?.event.data[0]?.toString() || status.asFinalized.toHex());
          }
        }
      })
      .catch(reject);
  });
}

export async function vote(
  proposalId: string,
  voteType: VoteType,
  amount: string,
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  const voteValue = { yes: 0, no: 1, abstain: 2 }[voteType];

  return new Promise((resolve, reject) => {
    api.tx.entityGovernance
      .vote(proposalId, voteValue, amount)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) reject(new Error(dispatchError.toString()));
          else resolve();
        }
      })
      .catch(reject);
  });
}

export async function getMyVote(proposalId: string, address: string): Promise<VoteRecord | null> {
  try {
    const api = await getApi();
    const data = await api.query.entityGovernance.votes(proposalId, address);
    
    if (data.isEmpty) return null;
    
    const v = data.toJSON() as any;
    return {
      proposalId,
      voter: address,
      voteType: (['yes', 'no', 'abstain'] as VoteType[])[v.voteType] || 'yes',
      amount: v.amount?.toString() || '0',
      timestamp: v.timestamp || Date.now(),
    };
  } catch (error) {
    return null;
  }
}

function getMockProposals(): Proposal[] {
  return [
    {
      id: '1',
      entityId: 'entity1',
      entityName: 'COSMOS DAO',
      title: '提议增加做市商最低保证金要求',
      description: '为了提高平台安全性，建议将做市商最低保证金从10,000 COS提高到20,000 COS。',
      proposer: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
      proposerName: '核心开发者',
      status: 'voting',
      votesYes: '150000000000000000',
      votesNo: '50000000000000000',
      votesAbstain: '10000000000000000',
      totalVotes: '210000000000000000',
      quorum: 50,
      threshold: 60,
      startTime: Date.now() - 3 * 24 * 3600000,
      endTime: Date.now() + 4 * 24 * 3600000,
      createdAt: Date.now() - 3 * 24 * 3600000,
    },
    {
      id: '2',
      entityId: 'entity1',
      entityName: 'COSMOS DAO',
      title: '新增平台手续费分红机制',
      description: '建议将平台交易手续费的30%用于回购销毁COS代币。',
      proposer: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
      proposerName: '社区成员',
      status: 'passed',
      votesYes: '200000000000000000',
      votesNo: '30000000000000000',
      votesAbstain: '20000000000000000',
      totalVotes: '250000000000000000',
      quorum: 50,
      threshold: 60,
      startTime: Date.now() - 10 * 24 * 3600000,
      endTime: Date.now() - 3 * 24 * 3600000,
      executedAt: Date.now() - 2 * 24 * 3600000,
      createdAt: Date.now() - 10 * 24 * 3600000,
    },
  ];
}
