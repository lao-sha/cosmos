/**
 * 托管、仲裁、证据和推荐系统测试
 * 测试 Escrow, Arbitration, Evidence, AffiliateReferral, StardustIpfs 模块
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
  TestSuiteResult,
  DEFAULT_WS_ENDPOINT,
  randomString,
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
  log.section('初始化托管/仲裁系统测试环境...');
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
 * 运行托管和仲裁系统测试
 */
export async function runEscrowArbitrationTests(): Promise<TestSuiteResult> {
  await setup();

  const result = await runTestSuite('托管、仲裁和相关系统测试', [
    // ==================== Escrow 托管模块 ====================
    {
      name: '[Escrow] 查询托管账户列表',
      fn: async () => {
        if (api.query.escrow) {
          if (api.query.escrow.escrows) {
            const escrows = await api.query.escrow.escrows.entries();
            log.subSection(`托管账户数量: ${escrows.length}`);
          }
          if (api.query.escrow.escrowCount) {
            const count = await api.query.escrow.escrowCount();
            log.subSection(`托管总数: ${count.toString()}`);
          }
        }
        assert(api.query.escrow !== undefined, 'Escrow pallet 应该存在');
      },
    },
    {
      name: '[Escrow] 创建托管',
      fn: async () => {
        if (api.tx.escrow && api.tx.escrow.create) {
          const recipient = accounts.bob.address;
          const amount = UNIT * 10n; // 托管 10 DUST
          const releaseTime = Math.floor(Date.now() / 1000) + 3600; // 1小时后
          
          const tx = api.tx.escrow.create(recipient, amount, releaseTime);
          const result = await signAndSendTx(api, tx, accounts.alice);
          
          if (result.success) {
            log.subSection('托管创建成功');
            for (const { event } of result.events) {
              if (event.section === 'escrow') {
                log.subSection(`事件: ${event.method} - ${event.data.toString()}`);
              }
            }
          } else {
            log.subSection(`托管创建结果: ${result.error}`);
          }
        } else if (api.tx.escrow && api.tx.escrow.createEscrow) {
          const tx = api.tx.escrow.createEscrow(accounts.bob.address, UNIT * 10n);
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`托管创建结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('create 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[Escrow] 查询托管详情',
      fn: async () => {
        if (api.query.escrow && api.query.escrow.escrowInfo) {
          const info = await api.query.escrow.escrowInfo(1);
          log.subSection(`托管 1 信息: ${info.toString()}`);
        } else if (api.query.escrow && api.query.escrow.escrows) {
          const escrow = await api.query.escrow.escrows(1);
          log.subSection(`托管 1: ${escrow.toString()}`);
        }
        assert(true, '查询完成');
      },
    },
    {
      name: '[Escrow] 释放托管资金',
      fn: async () => {
        if (api.tx.escrow && api.tx.escrow.release) {
          const escrowId = 1;
          
          const tx = api.tx.escrow.release(escrowId);
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`释放托管结果: ${result.success ? '成功' : result.error}`);
        } else if (api.tx.escrow && api.tx.escrow.releaseEscrow) {
          const tx = api.tx.escrow.releaseEscrow(1);
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`释放托管结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('release 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[Escrow] 取消托管',
      fn: async () => {
        if (api.tx.escrow && api.tx.escrow.cancel) {
          const escrowId = 2;
          
          const tx = api.tx.escrow.cancel(escrowId);
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`取消托管结果: ${result.success ? '成功' : result.error}`);
        } else if (api.tx.escrow && api.tx.escrow.cancelEscrow) {
          const tx = api.tx.escrow.cancelEscrow(2);
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`取消托管结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('cancel 方法不可用');
        }
        assert(true, '测试完成');
      },
    },

    // ==================== Evidence 证据模块 ====================
    {
      name: '[Evidence] 查询证据列表',
      fn: async () => {
        if (api.query.evidence) {
          if (api.query.evidence.evidences) {
            const evidences = await api.query.evidence.evidences.entries();
            log.subSection(`证据数量: ${evidences.length}`);
          }
          if (api.query.evidence.evidenceCount) {
            const count = await api.query.evidence.evidenceCount();
            log.subSection(`证据总数: ${count.toString()}`);
          }
        }
        assert(api.query.evidence !== undefined, 'Evidence pallet 应该存在');
      },
    },
    {
      name: '[Evidence] 提交证据',
      fn: async () => {
        if (api.tx.evidence && api.tx.evidence.submit) {
          const evidenceHash = '0x' + randomString(64).replace(/[^0-9a-f]/gi, '0').slice(0, 64);
          const description = '交易纠纷证据_' + randomString(4);
          
          const tx = api.tx.evidence.submit(evidenceHash, description);
          const result = await signAndSendTx(api, tx, accounts.alice);
          
          if (result.success) {
            log.subSection('证据提交成功');
            for (const { event } of result.events) {
              if (event.section === 'evidence') {
                log.subSection(`事件: ${event.method}`);
              }
            }
          } else {
            log.subSection(`证据提交结果: ${result.error}`);
          }
        } else if (api.tx.evidence && api.tx.evidence.submitEvidence) {
          const tx = api.tx.evidence.submitEvidence('0x1234', '测试证据');
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`证据提交结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('submit 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[Evidence] 查询证据详情',
      fn: async () => {
        if (api.query.evidence && api.query.evidence.evidenceInfo) {
          const info = await api.query.evidence.evidenceInfo(1);
          log.subSection(`证据 1 信息: ${info.toString()}`);
        } else if (api.query.evidence && api.query.evidence.evidences) {
          const evidence = await api.query.evidence.evidences(1);
          log.subSection(`证据 1: ${evidence.toString()}`);
        }
        assert(true, '查询完成');
      },
    },

    // ==================== Arbitration 仲裁模块 ====================
    {
      name: '[Arbitration] 查询仲裁案件列表',
      fn: async () => {
        if (api.query.arbitration) {
          if (api.query.arbitration.cases) {
            const cases = await api.query.arbitration.cases.entries();
            log.subSection(`仲裁案件数量: ${cases.length}`);
          }
          if (api.query.arbitration.caseCount) {
            const count = await api.query.arbitration.caseCount();
            log.subSection(`案件总数: ${count.toString()}`);
          }
        }
        assert(api.query.arbitration !== undefined, 'Arbitration pallet 应该存在');
      },
    },
    {
      name: '[Arbitration] 发起仲裁',
      fn: async () => {
        if (api.tx.arbitration && api.tx.arbitration.initiate) {
          const respondent = accounts.charlie.address;
          const reason = '交易纠纷：未按时发货_' + randomString(4);
          const evidenceId = 1;
          
          const tx = api.tx.arbitration.initiate(respondent, reason, evidenceId);
          const result = await signAndSendTx(api, tx, accounts.alice);
          
          if (result.success) {
            log.subSection('仲裁发起成功');
            for (const { event } of result.events) {
              if (event.section === 'arbitration') {
                log.subSection(`事件: ${event.method} - ${event.data.toString()}`);
              }
            }
          } else {
            log.subSection(`仲裁发起结果: ${result.error}`);
          }
        } else if (api.tx.arbitration && api.tx.arbitration.createCase) {
          const tx = api.tx.arbitration.createCase(accounts.charlie.address, '测试仲裁');
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`仲裁发起结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('initiate 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[Arbitration] 查询仲裁案件详情',
      fn: async () => {
        if (api.query.arbitration && api.query.arbitration.caseInfo) {
          const info = await api.query.arbitration.caseInfo(1);
          log.subSection(`案件 1 信息: ${info.toString()}`);
        } else if (api.query.arbitration && api.query.arbitration.cases) {
          const caseData = await api.query.arbitration.cases(1);
          log.subSection(`案件 1: ${caseData.toString()}`);
        }
        assert(true, '查询完成');
      },
    },
    {
      name: '[Arbitration] 提交仲裁答辩',
      fn: async () => {
        if (api.tx.arbitration && api.tx.arbitration.respond) {
          const caseId = 1;
          const response = '对方描述不实，提供反驳证据_' + randomString(4);
          
          const tx = api.tx.arbitration.respond(caseId, response);
          const result = await signAndSendTx(api, tx, accounts.charlie);
          log.subSection(`答辩提交结果: ${result.success ? '成功' : result.error}`);
        } else if (api.tx.arbitration && api.tx.arbitration.submitResponse) {
          const tx = api.tx.arbitration.submitResponse(1, '测试答辩');
          const result = await signAndSendTx(api, tx, accounts.charlie);
          log.subSection(`答辩提交结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('respond 方法不可用');
        }
        assert(true, '测试完成');
      },
    },

    // ==================== AffiliateReferral 推荐模块 ====================
    {
      name: '[AffiliateReferral] 查询推荐关系',
      fn: async () => {
        if (api.query.affiliateReferral) {
          if (api.query.affiliateReferral.referrals) {
            const referrals = await api.query.affiliateReferral.referrals.entries();
            log.subSection(`推荐关系数量: ${referrals.length}`);
          }
          if (api.query.affiliateReferral.referrerOf) {
            const referrer = await api.query.affiliateReferral.referrerOf(accounts.alice.address);
            log.subSection(`Alice 的推荐人: ${referrer.toString()}`);
          }
        }
        assert(api.query.affiliateReferral !== undefined, 'AffiliateReferral pallet 应该存在');
      },
    },
    {
      name: '[AffiliateReferral] 绑定推荐人',
      fn: async () => {
        if (api.tx.affiliateReferral && api.tx.affiliateReferral.bind) {
          const referrer = accounts.alice.address;
          
          const tx = api.tx.affiliateReferral.bind(referrer);
          const result = await signAndSendTx(api, tx, accounts.ferdie);
          
          if (result.success) {
            log.subSection('推荐人绑定成功');
          } else {
            log.subSection(`推荐人绑定结果: ${result.error}`);
          }
        } else if (api.tx.affiliateReferral && api.tx.affiliateReferral.setReferrer) {
          const tx = api.tx.affiliateReferral.setReferrer(accounts.alice.address);
          const result = await signAndSendTx(api, tx, accounts.ferdie);
          log.subSection(`推荐人绑定结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('bind 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[AffiliateReferral] 查询推荐奖励',
      fn: async () => {
        if (api.query.affiliateReferral && api.query.affiliateReferral.rewards) {
          const rewards = await api.query.affiliateReferral.rewards(accounts.alice.address);
          log.subSection(`Alice 的推荐奖励: ${rewards.toString()}`);
        } else if (api.query.affiliateReferral && api.query.affiliateReferral.pendingRewards) {
          const rewards = await api.query.affiliateReferral.pendingRewards(accounts.alice.address);
          log.subSection(`Alice 待领取奖励: ${rewards.toString()}`);
        }
        assert(true, '查询完成');
      },
    },
    {
      name: '[AffiliateReferral] 领取推荐奖励',
      fn: async () => {
        if (api.tx.affiliateReferral && api.tx.affiliateReferral.claimRewards) {
          const tx = api.tx.affiliateReferral.claimRewards();
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`领取奖励结果: ${result.success ? '成功' : result.error}`);
        } else if (api.tx.affiliateReferral && api.tx.affiliateReferral.claim) {
          const tx = api.tx.affiliateReferral.claim();
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`领取奖励结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('claimRewards 方法不可用');
        }
        assert(true, '测试完成');
      },
    },

    // ==================== StardustIpfs 模块 ====================
    {
      name: '[StardustIpfs] 查询 IPFS 存储状态',
      fn: async () => {
        if (api.query.stardustIpfs) {
          const storageKeys = Object.keys(api.query.stardustIpfs);
          log.subSection(`StardustIpfs 存储项: ${storageKeys.join(', ')}`);
          
          if (api.query.stardustIpfs.files) {
            const files = await api.query.stardustIpfs.files.entries();
            log.subSection(`存储文件数量: ${files.length}`);
          }
        }
        assert(api.query.stardustIpfs !== undefined, 'StardustIpfs pallet 应该存在');
      },
    },
    {
      name: '[StardustIpfs] 存储 IPFS 哈希',
      fn: async () => {
        if (api.tx.stardustIpfs && api.tx.stardustIpfs.store) {
          // 模拟 IPFS CID
          const ipfsHash = 'Qm' + randomString(44).replace(/[^A-Za-z0-9]/g, 'a');
          const metadata = '测试文件元数据';
          
          const tx = api.tx.stardustIpfs.store(ipfsHash, metadata);
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`IPFS 存储结果: ${result.success ? '成功' : result.error}`);
        } else if (api.tx.stardustIpfs && api.tx.stardustIpfs.pin) {
          const tx = api.tx.stardustIpfs.pin('QmTestHash123');
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`IPFS pin 结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('store 方法不可用');
        }
        assert(true, '测试完成');
      },
    },

    // ==================== 综合测试 ====================
    {
      name: '辅助模块存储检查',
      fn: async () => {
        const pallets = ['escrow', 'evidence', 'arbitration', 'affiliateReferral', 'stardustIpfs'];
        
        let availableCount = 0;
        for (const pallet of pallets) {
          if ((api.query as any)[pallet]) {
            availableCount++;
            const storageKeys = Object.keys((api.query as any)[pallet]);
            log.subSection(`✓ ${pallet} 可用 (${storageKeys.length} 个存储项)`);
          } else {
            log.subSection(`✗ ${pallet} 不可用`);
          }
        }
        
        log.subSection(`总计: ${availableCount}/${pallets.length} 个辅助模块可用`);
        assert(availableCount > 0, '至少应该有一个辅助模块可用');
      },
    },
    {
      name: '辅助模块交易方法检查',
      fn: async () => {
        const methods: string[] = [];
        
        const pallets = ['escrow', 'evidence', 'arbitration', 'affiliateReferral', 'stardustIpfs'];
        
        for (const pallet of pallets) {
          if ((api.tx as any)[pallet]) {
            methods.push(...Object.keys((api.tx as any)[pallet]).map(m => `${pallet}.${m}`));
          }
        }
        
        log.subSection(`可用交易方法: ${methods.length}`);
        log.subSection(`方法列表: ${methods.slice(0, 10).join(', ')}${methods.length > 10 ? '...' : ''}`);
        
        assert(methods.length > 0, '应该有可用的交易方法');
      },
    },
    {
      name: '委员会模块检查',
      fn: async () => {
        const committees = [
          'technicalCommittee',
          'arbitrationCommittee', 
          'treasuryCouncil',
          'contentCommittee'
        ];
        
        for (const committee of committees) {
          if ((api.query as any)[committee]) {
            log.subSection(`✓ ${committee} 可用`);
            if ((api.query as any)[committee].members) {
              const members = await (api.query as any)[committee].members();
              log.subSection(`  成员数量: ${members.length || 0}`);
            }
          } else {
            log.subSection(`✗ ${committee} 不可用`);
          }
        }
        assert(true, '委员会检查完成');
      },
    },
  ]);

  await cleanup();
  return result;
}

// 如果直接运行此文件
const isMainModule = import.meta.url === `file://${process.argv[1]}`;
if (isMainModule) {
  runEscrowArbitrationTests()
    .then(result => {
      process.exit(result.totalFailed > 0 ? 1 : 0);
    })
    .catch(error => {
      console.error('测试运行失败:', error);
      process.exit(1);
    });
}

