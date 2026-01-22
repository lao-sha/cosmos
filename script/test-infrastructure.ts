/**
 * 基础设施测试
 * 测试节点连接、系统模块、区块生成等基本功能
 */

import { ApiPromise, Keyring } from '@polkadot/api';
import {
  initApi,
  getTestAccounts,
  log,
  runTestSuite,
  assert,
  assertGreaterThan,
  TestSuiteResult,
  DEFAULT_WS_ENDPOINT,
} from './test-utils.js';

let api: ApiPromise;
let keyring: Keyring;

/**
 * 初始化测试环境
 */
async function setup(): Promise<void> {
  log.section('初始化基础设施测试环境...');
  api = await initApi(DEFAULT_WS_ENDPOINT);
  keyring = new Keyring({ type: 'sr25519' });
  log.success('API 连接成功');
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
 * 运行基础设施测试
 */
export async function runInfrastructureTests(): Promise<TestSuiteResult> {
  await setup();

  const result = await runTestSuite('基础设施测试', [
    {
      name: '节点连接测试',
      fn: async () => {
        assert(api.isConnected, 'API 应该已连接');
      },
    },
    {
      name: '链信息获取测试',
      fn: async () => {
        const chain = await api.rpc.system.chain();
        const version = await api.rpc.system.version();
        const name = await api.rpc.system.name();
        
        log.subSection(`链名称: ${chain.toString()}`);
        log.subSection(`节点版本: ${version.toString()}`);
        log.subSection(`节点名称: ${name.toString()}`);
        
        assert(chain.toString().length > 0, '链名称不应为空');
      },
    },
    {
      name: '运行时版本测试',
      fn: async () => {
        const runtimeVersion = api.runtimeVersion;
        
        log.subSection(`Spec 名称: ${runtimeVersion.specName.toString()}`);
        log.subSection(`Spec 版本: ${runtimeVersion.specVersion.toString()}`);
        log.subSection(`Impl 版本: ${runtimeVersion.implVersion.toString()}`);
        
        assert(runtimeVersion.specName.toString() === 'stardust', 'Spec 名称应为 stardust');
      },
    },
    {
      name: '区块生成测试',
      fn: async () => {
        const header = await api.rpc.chain.getHeader();
        const blockNumber = header.number.toNumber();
        
        log.subSection(`当前区块高度: ${blockNumber}`);
        
        assertGreaterThan(blockNumber, -1, '区块高度应该大于等于 0');
      },
    },
    {
      name: '最终区块测试',
      fn: async () => {
        const finalizedHash = await api.rpc.chain.getFinalizedHead();
        const finalizedHeader = await api.rpc.chain.getHeader(finalizedHash);
        
        log.subSection(`最终区块哈希: ${finalizedHash.toString().slice(0, 18)}...`);
        log.subSection(`最终区块高度: ${finalizedHeader.number.toNumber()}`);
        
        assert(finalizedHash.toString().length > 0, '最终区块哈希不应为空');
      },
    },
    {
      name: 'Pallet 可用性测试 - System',
      fn: async () => {
        const account = await api.query.system.account(getTestAccounts(keyring).alice.address);
        assert(account !== undefined, 'System pallet 应该可用');
      },
    },
    {
      name: 'Pallet 可用性测试 - Balances',
      fn: async () => {
        assert(api.query.balances !== undefined, 'Balances pallet 应该可用');
        assert(api.tx.balances !== undefined, 'Balances 交易应该可用');
      },
    },
    {
      name: 'Pallet 可用性测试 - Timestamp',
      fn: async () => {
        const now = await api.query.timestamp.now();
        log.subSection(`当前时间戳: ${now.toString()}`);
        assert(api.query.timestamp !== undefined, 'Timestamp pallet 应该可用');
      },
    },
    {
      name: 'Pallet 可用性测试 - Sudo',
      fn: async () => {
        assert(api.query.sudo !== undefined, 'Sudo pallet 应该可用');
        assert(api.tx.sudo !== undefined, 'Sudo 交易应该可用');
      },
    },
    {
      name: '占卜模块可用性测试 - Almanac',
      fn: async () => {
        assert(api.query.almanac !== undefined, 'Almanac pallet 应该可用');
      },
    },
    {
      name: '占卜模块可用性测试 - Meihua',
      fn: async () => {
        assert(api.query.meihua !== undefined, 'Meihua pallet 应该可用');
      },
    },
    {
      name: '占卜模块可用性测试 - Bazi',
      fn: async () => {
        assert(api.query.bazi !== undefined, 'Bazi pallet 应该可用');
      },
    },
    {
      name: '占卜模块可用性测试 - Liuyao',
      fn: async () => {
        assert(api.query.liuyao !== undefined, 'Liuyao pallet 应该可用');
      },
    },
    {
      name: '占卜模块可用性测试 - Qimen',
      fn: async () => {
        assert(api.query.qimen !== undefined, 'Qimen pallet 应该可用');
      },
    },
    {
      name: '占卜模块可用性测试 - Ziwei',
      fn: async () => {
        assert(api.query.ziwei !== undefined, 'Ziwei pallet 应该可用');
      },
    },
    {
      name: '占卜模块可用性测试 - Tarot',
      fn: async () => {
        assert(api.query.tarot !== undefined, 'Tarot pallet 应该可用');
      },
    },
    {
      name: '聊天模块可用性测试 - ChatCore',
      fn: async () => {
        assert(api.query.chatCore !== undefined, 'ChatCore pallet 应该可用');
      },
    },
    {
      name: '聊天模块可用性测试 - ChatGroup',
      fn: async () => {
        assert(api.query.chatGroup !== undefined, 'ChatGroup pallet 应该可用');
      },
    },
    {
      name: '聊天模块可用性测试 - Livestream',
      fn: async () => {
        assert(api.query.livestream !== undefined, 'Livestream pallet 应该可用');
      },
    },
    {
      name: '交易模块可用性测试 - TradingMaker',
      fn: async () => {
        assert(api.query.tradingMaker !== undefined, 'TradingMaker pallet 应该可用');
      },
    },
    {
      name: '交易模块可用性测试 - TradingOtc',
      fn: async () => {
        assert(api.query.tradingOtc !== undefined, 'TradingOtc pallet 应该可用');
      },
    },
    {
      name: '其他模块可用性测试 - Escrow',
      fn: async () => {
        assert(api.query.escrow !== undefined, 'Escrow pallet 应该可用');
      },
    },
    {
      name: '其他模块可用性测试 - Arbitration',
      fn: async () => {
        assert(api.query.arbitration !== undefined, 'Arbitration pallet 应该可用');
      },
    },
    {
      name: '其他模块可用性测试 - Evidence',
      fn: async () => {
        assert(api.query.evidence !== undefined, 'Evidence pallet 应该可用');
      },
    },
    {
      name: '节点健康检查',
      fn: async () => {
        const health = await api.rpc.system.health();
        log.subSection(`同步中: ${health.isSyncing.toString()}`);
        log.subSection(`连接的节点数: ${health.peers.toString()}`);
        log.subSection(`是否需要同步: ${health.shouldHavePeers.toString()}`);
        
        // 开发链通常不需要同步
        assert(health !== undefined, '健康检查应该返回有效数据');
      },
    },
  ]);

  await cleanup();
  return result;
}

// 如果直接运行此文件
const isMainModule = import.meta.url === `file://${process.argv[1]}`;
if (isMainModule) {
  runInfrastructureTests()
    .then(result => {
      process.exit(result.totalFailed > 0 ? 1 : 0);
    })
    .catch(error => {
      console.error('测试运行失败:', error);
      process.exit(1);
    });
}

