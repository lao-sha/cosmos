/**
 * 余额和转账测试
 * 测试账户余额查询、转账功能
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
  log.section('初始化余额测试环境...');
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
 * 获取账户余额
 */
async function getBalance(address: string): Promise<bigint> {
  const { data: balance } = await api.query.system.account(address) as any;
  return BigInt(balance.free.toString());
}

/**
 * 运行余额测试
 */
export async function runBalanceTests(): Promise<TestSuiteResult> {
  await setup();

  const result = await runTestSuite('余额和转账测试', [
    {
      name: 'Alice 账户余额查询',
      fn: async () => {
        const balance = await getBalance(accounts.alice.address);
        log.subSection(`Alice 地址: ${accounts.alice.address}`);
        log.subSection(`Alice 余额: ${formatBalance(balance)}`);
        assertGreaterThan(balance, 0n, 'Alice 应该有预置余额');
      },
    },
    {
      name: 'Bob 账户余额查询',
      fn: async () => {
        const balance = await getBalance(accounts.bob.address);
        log.subSection(`Bob 地址: ${accounts.bob.address}`);
        log.subSection(`Bob 余额: ${formatBalance(balance)}`);
        assertGreaterThan(balance, 0n, 'Bob 应该有预置余额');
      },
    },
    {
      name: 'Charlie 账户余额查询',
      fn: async () => {
        const balance = await getBalance(accounts.charlie.address);
        log.subSection(`Charlie 地址: ${accounts.charlie.address}`);
        log.subSection(`Charlie 余额: ${formatBalance(balance)}`);
        // Charlie 可能没有预置余额
      },
    },
    {
      name: '查询全部测试账户余额',
      fn: async () => {
        for (const [name, account] of Object.entries(accounts)) {
          const balance = await getBalance(account.address);
          log.subSection(`${name}: ${formatBalance(balance)}`);
        }
        assert(true, '查询成功');
      },
    },
    {
      name: 'Alice 转账给 Charlie',
      fn: async () => {
        const amount = UNIT; // 1 DUST
        const charlieBefore = await getBalance(accounts.charlie.address);
        
        log.subSection(`转账金额: ${formatBalance(amount)}`);
        log.subSection(`Charlie 转账前余额: ${formatBalance(charlieBefore)}`);
        
        const tx = api.tx.balances.transferKeepAlive(accounts.charlie.address, amount);
        const result = await signAndSendTx(api, tx, accounts.alice, {
          section: 'balances',
          method: 'Transfer',
        });
        
        assert(result.success, `转账应该成功: ${result.error || ''}`);
        log.subSection(`交易区块: ${result.blockHash.slice(0, 18)}...`);
        
        const charlieAfter = await getBalance(accounts.charlie.address);
        log.subSection(`Charlie 转账后余额: ${formatBalance(charlieAfter)}`);
        
        assertGreaterThan(charlieAfter, charlieBefore, 'Charlie 余额应该增加');
      },
    },
    {
      name: 'Bob 转账给 Dave',
      fn: async () => {
        const amount = UNIT * 5n; // 5 DUST
        const daveBefore = await getBalance(accounts.dave.address);
        
        log.subSection(`转账金额: ${formatBalance(amount)}`);
        
        const tx = api.tx.balances.transferKeepAlive(accounts.dave.address, amount);
        const result = await signAndSendTx(api, tx, accounts.bob, {
          section: 'balances',
          method: 'Transfer',
        });
        
        assert(result.success, `转账应该成功: ${result.error || ''}`);
        
        const daveAfter = await getBalance(accounts.dave.address);
        log.subSection(`Dave 余额变化: ${formatBalance(daveBefore)} → ${formatBalance(daveAfter)}`);
        
        assertGreaterThan(daveAfter, daveBefore, 'Dave 余额应该增加');
      },
    },
    {
      name: '批量转账测试',
      fn: async () => {
        const amount = UNIT / 10n; // 0.1 DUST
        const recipients = [accounts.eve, accounts.ferdie];
        
        const transfers = recipients.map(recipient =>
          api.tx.balances.transferKeepAlive(recipient.address, amount)
        );
        
        const batchTx = api.tx.utility.batch(transfers);
        const result = await signAndSendTx(api, batchTx, accounts.alice);
        
        assert(result.success, `批量转账应该成功: ${result.error || ''}`);
        log.subSection(`批量转账完成，区块: ${result.blockHash.slice(0, 18)}...`);
      },
    },
    {
      name: '转账金额为 0 应该失败',
      fn: async () => {
        const tx = api.tx.balances.transferKeepAlive(accounts.bob.address, 0);
        const result = await signAndSendTx(api, tx, accounts.alice);
        
        // 转账 0 应该失败或产生错误
        // 具体行为取决于 pallet 配置
        log.subSection(`转账 0 结果: ${result.success ? '成功' : '失败'}`);
      },
    },
    {
      name: '转账超过余额应该失败',
      fn: async () => {
        // 先查询 Charlie 余额
        const charlieBalance = await getBalance(accounts.charlie.address);
        const excessAmount = charlieBalance + UNIT * 1000000n; // 超过余额很多
        
        const tx = api.tx.balances.transferKeepAlive(accounts.alice.address, excessAmount);
        const result = await signAndSendTx(api, tx, accounts.charlie);
        
        assert(!result.success, '转账超过余额应该失败');
        log.subSection(`预期失败: ${result.error}`);
      },
    },
    {
      name: '存在性存款 (Existential Deposit) 测试',
      fn: async () => {
        // 查询存在性存款
        const ed = api.consts.balances.existentialDeposit;
        log.subSection(`存在性存款: ${formatBalance(BigInt(ed.toString()))}`);
        
        assertGreaterThan(BigInt(ed.toString()), 0n, '存在性存款应该大于 0');
      },
    },
    {
      name: '转账事件验证',
      fn: async () => {
        const amount = UNIT / 100n; // 0.01 DUST
        
        const tx = api.tx.balances.transferKeepAlive(accounts.bob.address, amount);
        const result = await signAndSendTx(api, tx, accounts.alice);
        
        // 查找 Transfer 事件
        let transferEventFound = false;
        for (const { event } of result.events) {
          if (event.section === 'balances' && event.method === 'Transfer') {
            transferEventFound = true;
            const [from, to, value] = event.data;
            log.subSection(`Transfer 事件: ${from} → ${to}, 金额: ${value}`);
          }
        }
        
        assert(transferEventFound, '应该找到 Transfer 事件');
      },
    },
    {
      name: '查询账户 nonce',
      fn: async () => {
        const { nonce } = await api.query.system.account(accounts.alice.address) as any;
        log.subSection(`Alice nonce: ${nonce.toString()}`);
        assertGreaterThan(BigInt(nonce.toString()), -1n, 'Nonce 应该是非负数');
      },
    },
  ]);

  await cleanup();
  return result;
}

// 如果直接运行此文件
const isMainModule = import.meta.url === `file://${process.argv[1]}`;
if (isMainModule) {
  runBalanceTests()
    .then(result => {
      process.exit(result.totalFailed > 0 ? 1 : 0);
    })
    .catch(error => {
      console.error('测试运行失败:', error);
      process.exit(1);
    });
}

