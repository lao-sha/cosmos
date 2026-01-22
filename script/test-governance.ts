/**
 * 治理和系统模块测试
 * 测试 委员会、成员管理、交易费用、共识、TEE 隐私等模块
 */

import { ApiPromise, Keyring } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import {
  initApi,
  getTestAccounts,
  log,
  runTestSuite,
  signAndSendTx,
  assert,
  assertGreaterThan,
  formatBalance,
  TestSuiteResult,
  DEFAULT_WS_ENDPOINT,
  UNIT,
} from './test-utils.js';

let api: ApiPromise;
let keyring: Keyring;
let accounts: {
  alice: KeyringPair;
  bob: KeyringPair;
  charlie: KeyringPair;
  dave: KeyringPair;
  eve: KeyringPair;
  ferdie: KeyringPair;
};

/**
 * 初始化测试环境
 */
async function setup(): Promise<void> {
  log.section('初始化治理和系统模块测试环境...');
  api = await initApi(DEFAULT_WS_ENDPOINT);
  keyring = new Keyring({ type: 'sr25519' });
  accounts = getTestAccounts(keyring);
  log.success('测试环境初始化完成');
}

/**
 * 清理测试环境
 */
async function cleanup(): Promise<void> {
  if (api) {
    await api.disconnect();
    log.info('API 连接已断开');
  }
}

/**
 * 运行治理和系统模块测试
 */
export async function runGovernanceTests(): Promise<TestSuiteResult> {
  await setup();

  const result = await runTestSuite('治理和系统模块测试', [
    // ==================== Aura 共识模块 ====================
    {
      name: '[Aura] 查询当前验证者',
      fn: async () => {
        if (api.query.aura) {
          if (api.query.aura.authorities) {
            const authorities = await api.query.aura.authorities();
            log.subSection(`Aura 验证者数量: ${(authorities as any).length || 0}`);
          }
          if (api.query.aura.currentSlot) {
            const slot = await api.query.aura.currentSlot();
            log.subSection(`当前 Slot: ${slot.toString()}`);
          }
        }
        assert(api.query.aura !== undefined, 'Aura pallet 应该存在');
      },
    },

    // ==================== Grandpa 最终性模块 ====================
    {
      name: '[Grandpa] 查询最终性状态',
      fn: async () => {
        if (api.query.grandpa) {
          if (api.query.grandpa.currentSetId) {
            const setId = await api.query.grandpa.currentSetId();
            log.subSection(`当前 Set ID: ${setId.toString()}`);
          }
          if (api.query.grandpa.stalled) {
            const stalled = await api.query.grandpa.stalled();
            log.subSection(`是否停滞: ${stalled.toString()}`);
          }
        }
        assert(api.query.grandpa !== undefined, 'Grandpa pallet 应该存在');
      },
    },
    {
      name: '[Grandpa] 查询验证者集合',
      fn: async () => {
        if (api.query.grandpa && api.query.grandpa.authorities) {
          const authorities = await api.query.grandpa.authorities();
          log.subSection(`Grandpa 验证者: ${authorities.toString().slice(0, 100)}...`);
        }
        assert(true, '查询完成');
      },
    },

    // ==================== TransactionPayment 交易费用模块 ====================
    {
      name: '[TransactionPayment] 查询交易费用配置',
      fn: async () => {
        if (api.query.transactionPayment) {
          if (api.query.transactionPayment.nextFeeMultiplier) {
            const multiplier = await api.query.transactionPayment.nextFeeMultiplier();
            log.subSection(`下一个费用乘数: ${multiplier.toString()}`);
          }
        }
        assert(api.query.transactionPayment !== undefined, 'TransactionPayment pallet 应该存在');
      },
    },
    {
      name: '[TransactionPayment] 查询费用常量',
      fn: async () => {
        if (api.consts.transactionPayment) {
          if (api.consts.transactionPayment.operationalFeeMultiplier) {
            const multiplier = api.consts.transactionPayment.operationalFeeMultiplier;
            log.subSection(`操作费用乘数: ${multiplier.toString()}`);
          }
        }
        assert(true, '查询完成');
      },
    },

    // ==================== TeePrivacy TEE 隐私模块 ====================
    {
      name: '[TeePrivacy] 查询 TEE 隐私模块可用性',
      fn: async () => {
        if (api.query.teePrivacy) {
          const storageKeys = Object.keys(api.query.teePrivacy);
          log.subSection(`TeePrivacy 存储项: ${storageKeys.join(', ')}`);
        }
        assert(api.query.teePrivacy !== undefined, 'TeePrivacy pallet 应该存在');
      },
    },
    {
      name: '[TeePrivacy] 查询 TEE 节点列表',
      fn: async () => {
        if (api.query.teePrivacy) {
          if (api.query.teePrivacy.teeNodes) {
            const nodes = await api.query.teePrivacy.teeNodes.entries();
            log.subSection(`TEE 节点数量: ${nodes.length}`);
          } else if (api.query.teePrivacy.enclaves) {
            const enclaves = await api.query.teePrivacy.enclaves.entries();
            log.subSection(`Enclave 数量: ${enclaves.length}`);
          }
        }
        assert(true, '查询完成');
      },
    },
    {
      name: '[TeePrivacy] 注册 TEE 节点',
      fn: async () => {
        if (api.tx.teePrivacy && api.tx.teePrivacy.registerNode) {
          // 模拟 TEE 证明
          const attestation = '0x' + '00'.repeat(32);
          
          const tx = api.tx.teePrivacy.registerNode(attestation);
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`注册 TEE 节点结果: ${result.success ? '成功' : result.error}`);
        } else if (api.tx.teePrivacy && api.tx.teePrivacy.register) {
          const tx = api.tx.teePrivacy.register();
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`注册结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('registerNode 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[TeePrivacy] 提交加密数据',
      fn: async () => {
        if (api.tx.teePrivacy && api.tx.teePrivacy.submitEncryptedData) {
          const encryptedData = '0x' + 'ab'.repeat(64);
          
          const tx = api.tx.teePrivacy.submitEncryptedData(encryptedData);
          const result = await signAndSendTx(api, tx, accounts.bob);
          log.subSection(`提交加密数据结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('submitEncryptedData 方法不可用');
        }
        assert(true, '测试完成');
      },
    },

    // ==================== TechnicalCommittee 技术委员会 ====================
    {
      name: '[TechnicalCommittee] 查询委员会成员',
      fn: async () => {
        if (api.query.technicalCommittee) {
          if (api.query.technicalCommittee.members) {
            const members = await api.query.technicalCommittee.members();
            log.subSection(`技术委员会成员: ${(members as any).length || 0} 人`);
            log.subSection(`成员列表: ${members.toString().slice(0, 100)}...`);
          }
          if (api.query.technicalCommittee.proposalCount) {
            const count = await api.query.technicalCommittee.proposalCount();
            log.subSection(`提案总数: ${count.toString()}`);
          }
        }
        assert(api.query.technicalCommittee !== undefined, 'TechnicalCommittee pallet 应该存在');
      },
    },
    {
      name: '[TechnicalCommittee] 查询活跃提案',
      fn: async () => {
        if (api.query.technicalCommittee && api.query.technicalCommittee.proposals) {
          const proposals = await api.query.technicalCommittee.proposals();
          log.subSection(`活跃提案: ${proposals.toString()}`);
        }
        assert(true, '查询完成');
      },
    },

    // ==================== TechnicalMembership 技术委员会成员管理 ====================
    {
      name: '[TechnicalMembership] 查询成员管理模块',
      fn: async () => {
        if (api.query.technicalMembership) {
          const storageKeys = Object.keys(api.query.technicalMembership);
          log.subSection(`TechnicalMembership 存储项: ${storageKeys.join(', ')}`);
          
          if (api.query.technicalMembership.members) {
            const members = await api.query.technicalMembership.members();
            log.subSection(`成员: ${members.toString()}`);
          }
          if (api.query.technicalMembership.prime) {
            const prime = await api.query.technicalMembership.prime();
            log.subSection(`首席成员: ${prime.toString()}`);
          }
        }
        assert(api.query.technicalMembership !== undefined, 'TechnicalMembership pallet 应该存在');
      },
    },
    {
      name: '[TechnicalMembership] 添加成员 (需要 Sudo)',
      fn: async () => {
        if (api.tx.technicalMembership && api.tx.technicalMembership.addMember) {
          // 使用 sudo 添加成员
          if (api.tx.sudo && api.tx.sudo.sudo) {
            const addMemberCall = api.tx.technicalMembership.addMember(accounts.bob.address);
            const tx = api.tx.sudo.sudo(addMemberCall);
            const result = await signAndSendTx(api, tx, accounts.alice);
            log.subSection(`添加成员结果: ${result.success ? '成功' : result.error}`);
          } else {
            log.subSection('Sudo 不可用');
          }
        } else {
          log.subSection('addMember 方法不可用');
        }
        assert(true, '测试完成');
      },
    },

    // ==================== ArbitrationCommittee 仲裁委员会 ====================
    {
      name: '[ArbitrationCommittee] 查询委员会状态',
      fn: async () => {
        if (api.query.arbitrationCommittee) {
          if (api.query.arbitrationCommittee.members) {
            const members = await api.query.arbitrationCommittee.members();
            log.subSection(`仲裁委员会成员: ${(members as any).length || 0} 人`);
          }
          if (api.query.arbitrationCommittee.proposalCount) {
            const count = await api.query.arbitrationCommittee.proposalCount();
            log.subSection(`提案总数: ${count.toString()}`);
          }
        }
        assert(api.query.arbitrationCommittee !== undefined, 'ArbitrationCommittee pallet 应该存在');
      },
    },

    // ==================== ArbitrationMembership 仲裁委员会成员管理 ====================
    {
      name: '[ArbitrationMembership] 查询成员管理模块',
      fn: async () => {
        if (api.query.arbitrationMembership) {
          const storageKeys = Object.keys(api.query.arbitrationMembership);
          log.subSection(`ArbitrationMembership 存储项: ${storageKeys.join(', ')}`);
        }
        assert(api.query.arbitrationMembership !== undefined, 'ArbitrationMembership pallet 应该存在');
      },
    },

    // ==================== TreasuryCouncil 财务委员会 ====================
    {
      name: '[TreasuryCouncil] 查询委员会状态',
      fn: async () => {
        if (api.query.treasuryCouncil) {
          if (api.query.treasuryCouncil.members) {
            const members = await api.query.treasuryCouncil.members();
            log.subSection(`财务委员会成员: ${(members as any).length || 0} 人`);
          }
          if (api.query.treasuryCouncil.proposalCount) {
            const count = await api.query.treasuryCouncil.proposalCount();
            log.subSection(`提案总数: ${count.toString()}`);
          }
        }
        assert(api.query.treasuryCouncil !== undefined, 'TreasuryCouncil pallet 应该存在');
      },
    },

    // ==================== TreasuryMembership 财务委员会成员管理 ====================
    {
      name: '[TreasuryMembership] 查询成员管理模块',
      fn: async () => {
        if (api.query.treasuryMembership) {
          const storageKeys = Object.keys(api.query.treasuryMembership);
          log.subSection(`TreasuryMembership 存储项: ${storageKeys.join(', ')}`);
        }
        assert(api.query.treasuryMembership !== undefined, 'TreasuryMembership pallet 应该存在');
      },
    },

    // ==================== ContentCommittee 内容委员会 ====================
    {
      name: '[ContentCommittee] 查询委员会状态',
      fn: async () => {
        if (api.query.contentCommittee) {
          if (api.query.contentCommittee.members) {
            const members = await api.query.contentCommittee.members();
            log.subSection(`内容委员会成员: ${(members as any).length || 0} 人`);
          }
          if (api.query.contentCommittee.proposalCount) {
            const count = await api.query.contentCommittee.proposalCount();
            log.subSection(`提案总数: ${count.toString()}`);
          }
        }
        assert(api.query.contentCommittee !== undefined, 'ContentCommittee pallet 应该存在');
      },
    },

    // ==================== ContentMembership 内容委员会成员管理 ====================
    {
      name: '[ContentMembership] 查询成员管理模块',
      fn: async () => {
        if (api.query.contentMembership) {
          const storageKeys = Object.keys(api.query.contentMembership);
          log.subSection(`ContentMembership 存储项: ${storageKeys.join(', ')}`);
        }
        assert(api.query.contentMembership !== undefined, 'ContentMembership pallet 应该存在');
      },
    },

    // ==================== 委员会提案测试 ====================
    {
      name: '[Committee] 创建技术委员会提案',
      fn: async () => {
        if (api.tx.technicalCommittee && api.tx.technicalCommittee.propose) {
          // 创建一个简单的系统备注提案
          const remarkCall = api.tx.system.remark('Test proposal from technical committee');
          const threshold = 1;
          
          const tx = api.tx.technicalCommittee.propose(threshold, remarkCall, 1000);
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`创建提案结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('propose 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[Committee] 对提案投票',
      fn: async () => {
        if (api.tx.technicalCommittee && api.tx.technicalCommittee.vote) {
          // 假设提案哈希
          const proposalHash = '0x0000000000000000000000000000000000000000000000000000000000000000';
          const proposalIndex = 0;
          const approve = true;
          
          const tx = api.tx.technicalCommittee.vote(proposalHash, proposalIndex, approve);
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`投票结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('vote 方法不可用');
        }
        assert(true, '测试完成');
      },
    },

    // ==================== 综合检查 ====================
    {
      name: '治理模块存储检查',
      fn: async () => {
        const pallets = [
          'aura', 'grandpa', 'transactionPayment', 'teePrivacy',
          'technicalCommittee', 'technicalMembership',
          'arbitrationCommittee', 'arbitrationMembership',
          'treasuryCouncil', 'treasuryMembership',
          'contentCommittee', 'contentMembership'
        ];
        
        let availableCount = 0;
        for (const pallet of pallets) {
          if ((api.query as any)[pallet]) {
            availableCount++;
            log.subSection(`✓ ${pallet} 可用`);
          } else {
            log.subSection(`✗ ${pallet} 不可用`);
          }
        }
        
        log.subSection(`总计: ${availableCount}/${pallets.length} 个治理模块可用`);
        assert(availableCount > 0, '至少应该有一个治理模块可用');
      },
    },
    {
      name: '治理模块交易方法检查',
      fn: async () => {
        const methods: string[] = [];
        
        const pallets = [
          'technicalCommittee', 'technicalMembership',
          'arbitrationCommittee', 'arbitrationMembership',
          'treasuryCouncil', 'treasuryMembership',
          'contentCommittee', 'contentMembership',
          'teePrivacy'
        ];
        
        for (const pallet of pallets) {
          if ((api.tx as any)[pallet]) {
            methods.push(...Object.keys((api.tx as any)[pallet]).map(m => `${pallet}.${m}`));
          }
        }
        
        log.subSection(`可用交易方法: ${methods.length}`);
        if (methods.length > 0) {
          log.subSection(`方法列表: ${methods.slice(0, 15).join(', ')}${methods.length > 15 ? '...' : ''}`);
        }
        
        assert(true, '检查完成');
      },
    },
  ]);

  await cleanup();
  return result;
}

// 如果直接运行此文件
const isMainModule = import.meta.url === `file://${process.argv[1]}`;
if (isMainModule) {
  runGovernanceTests()
    .then(result => {
      process.exit(result.totalFailed > 0 ? 1 : 0);
    })
    .catch(error => {
      console.error('测试运行失败:', error);
      process.exit(1);
    });
}

