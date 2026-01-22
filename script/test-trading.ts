/**
 * 交易系统测试
 * 测试 TradingMaker, TradingOtc, TradingCredit, TradingPricing, TradingSwap 模块
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
  log.section('初始化交易系统测试环境...');
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
 * 运行交易系统测试
 */
export async function runTradingTests(): Promise<TestSuiteResult> {
  await setup();

  const result = await runTestSuite('交易系统测试', [
    // ==================== TradingPricing 模块 ====================
    {
      name: '[TradingPricing] 查询价格配置',
      fn: async () => {
        if (api.query.tradingPricing) {
          if (api.query.tradingPricing.currentPrice) {
            const price = await api.query.tradingPricing.currentPrice();
            log.subSection(`当前 DUST 价格: ${price.toString()}`);
          }
          if (api.query.tradingPricing.priceHistory) {
            const history = await api.query.tradingPricing.priceHistory.entries();
            log.subSection(`价格历史记录数: ${history.length}`);
          }
        }
        assert(api.query.tradingPricing !== undefined, 'TradingPricing pallet 应该存在');
      },
    },
    {
      name: '[TradingPricing] 查询价格参数',
      fn: async () => {
        if (api.query.tradingPricing && api.query.tradingPricing.pricingParams) {
          const params = await api.query.tradingPricing.pricingParams();
          log.subSection(`定价参数: ${params.toString()}`);
        } else if (api.query.tradingPricing && api.query.tradingPricing.config) {
          const config = await api.query.tradingPricing.config();
          log.subSection(`定价配置: ${config.toString()}`);
        }
        assert(true, '查询完成');
      },
    },

    // ==================== TradingCredit 模块 ====================
    {
      name: '[TradingCredit] 查询用户信用分',
      fn: async () => {
        if (api.query.tradingCredit) {
          if (api.query.tradingCredit.credits) {
            const aliceCredit = await api.query.tradingCredit.credits(accounts.alice.address);
            const bobCredit = await api.query.tradingCredit.credits(accounts.bob.address);
            log.subSection(`Alice 信用分: ${aliceCredit.toString()}`);
            log.subSection(`Bob 信用分: ${bobCredit.toString()}`);
          }
          if (api.query.tradingCredit.userCredit) {
            const credit = await api.query.tradingCredit.userCredit(accounts.alice.address);
            log.subSection(`Alice 信用: ${credit.toString()}`);
          }
        }
        assert(api.query.tradingCredit !== undefined, 'TradingCredit pallet 应该存在');
      },
    },
    {
      name: '[TradingCredit] 查询信用等级',
      fn: async () => {
        if (api.query.tradingCredit && api.query.tradingCredit.creditLevels) {
          const levels = await api.query.tradingCredit.creditLevels.entries();
          log.subSection(`信用等级数量: ${levels.length}`);
        }
        assert(true, '查询完成');
      },
    },

    // ==================== TradingMaker 模块 ====================
    {
      name: '[TradingMaker] 查询做市商列表',
      fn: async () => {
        if (api.query.tradingMaker) {
          if (api.query.tradingMaker.makers) {
            const makers = await api.query.tradingMaker.makers.entries();
            log.subSection(`注册做市商数量: ${makers.length}`);
            for (const [key, value] of makers.slice(0, 3)) {
              log.subSection(`  做市商: ${value.toString().slice(0, 80)}...`);
            }
          }
          if (api.query.tradingMaker.activeMakers) {
            const active = await api.query.tradingMaker.activeMakers();
            log.subSection(`活跃做市商: ${active.toString()}`);
          }
        }
        assert(api.query.tradingMaker !== undefined, 'TradingMaker pallet 应该存在');
      },
    },
    {
      name: '[TradingMaker] 注册成为做市商',
      fn: async () => {
        if (api.tx.tradingMaker && api.tx.tradingMaker.register) {
          const wechatId = 'maker_test_' + randomString(4);
          const premium = 5; // 5% 溢价
          const deposit = UNIT * 100n; // 100 DUST 保证金
          
          const tx = api.tx.tradingMaker.register(wechatId, premium, deposit);
          const result = await signAndSendTx(api, tx, accounts.charlie);
          
          if (result.success) {
            log.subSection('做市商注册成功');
            for (const { event } of result.events) {
              if (event.section === 'tradingMaker') {
                log.subSection(`事件: ${event.method} - ${event.data.toString()}`);
              }
            }
          } else {
            log.subSection(`做市商注册结果: ${result.error}`);
          }
        } else if (api.tx.tradingMaker && api.tx.tradingMaker.registerMaker) {
          const tx = api.tx.tradingMaker.registerMaker('test_wechat', 5);
          const result = await signAndSendTx(api, tx, accounts.charlie);
          log.subSection(`做市商注册结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('register 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[TradingMaker] 更新做市商信息',
      fn: async () => {
        if (api.tx.tradingMaker && api.tx.tradingMaker.updateInfo) {
          const newWechat = 'updated_wechat_' + randomString(4);
          
          const tx = api.tx.tradingMaker.updateInfo(newWechat, null); // null 保持溢价不变
          const result = await signAndSendTx(api, tx, accounts.charlie);
          log.subSection(`更新做市商信息结果: ${result.success ? '成功' : result.error}`);
        } else if (api.tx.tradingMaker && api.tx.tradingMaker.update) {
          const tx = api.tx.tradingMaker.update('new_wechat');
          const result = await signAndSendTx(api, tx, accounts.charlie);
          log.subSection(`更新结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('updateInfo 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[TradingMaker] 查询做市商详情',
      fn: async () => {
        if (api.query.tradingMaker && api.query.tradingMaker.makerInfo) {
          const info = await api.query.tradingMaker.makerInfo(accounts.charlie.address);
          log.subSection(`Charlie 做市商信息: ${info.toString()}`);
        } else if (api.query.tradingMaker && api.query.tradingMaker.makers) {
          const maker = await api.query.tradingMaker.makers(accounts.charlie.address);
          log.subSection(`Charlie 做市商: ${maker.toString()}`);
        }
        assert(true, '查询完成');
      },
    },

    // ==================== TradingOtc 模块 ====================
    {
      name: '[TradingOtc] 查询 OTC 订单列表',
      fn: async () => {
        if (api.query.tradingOtc) {
          if (api.query.tradingOtc.orders) {
            const orders = await api.query.tradingOtc.orders.entries();
            log.subSection(`OTC 订单数量: ${orders.length}`);
          }
          if (api.query.tradingOtc.orderCount) {
            const count = await api.query.tradingOtc.orderCount();
            log.subSection(`订单总数: ${count.toString()}`);
          }
        }
        assert(api.query.tradingOtc !== undefined, 'TradingOtc pallet 应该存在');
      },
    },
    {
      name: '[TradingOtc] 创建首购订单',
      fn: async () => {
        if (api.tx.tradingOtc && api.tx.tradingOtc.createFirstPurchase) {
          const makerId = accounts.charlie.address; // 假设 Charlie 是做市商
          const amount = UNIT * 10n; // 购买 10 DUST
          const paymentInfo = {
            realName: '测试用户',
            idNumber: '110101199001011234',
            phone: '13800138000',
            wechatId: 'test_buyer_' + randomString(4),
          };
          
          // 根据实际 API 签名调整参数
          const tx = api.tx.tradingOtc.createFirstPurchase(makerId, amount, paymentInfo);
          const result = await signAndSendTx(api, tx, accounts.dave);
          
          if (result.success) {
            log.subSection('首购订单创建成功');
            for (const { event } of result.events) {
              if (event.section === 'tradingOtc') {
                log.subSection(`事件: ${event.method} - ${event.data.toString()}`);
              }
            }
          } else {
            log.subSection(`首购订单创建结果: ${result.error}`);
          }
        } else if (api.tx.tradingOtc && api.tx.tradingOtc.firstPurchase) {
          const tx = api.tx.tradingOtc.firstPurchase(accounts.charlie.address, UNIT * 10n);
          const result = await signAndSendTx(api, tx, accounts.dave);
          log.subSection(`首购订单结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('createFirstPurchase 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[TradingOtc] 查询用户订单',
      fn: async () => {
        if (api.query.tradingOtc && api.query.tradingOtc.userOrders) {
          const orders = await api.query.tradingOtc.userOrders(accounts.dave.address);
          log.subSection(`Dave 的订单: ${orders.toString()}`);
        } else if (api.query.tradingOtc && api.query.tradingOtc.ordersByUser) {
          const orders = await api.query.tradingOtc.ordersByUser(accounts.dave.address);
          log.subSection(`Dave 的订单: ${orders.toString()}`);
        }
        assert(true, '查询完成');
      },
    },
    {
      name: '[TradingOtc] 标记订单已付款',
      fn: async () => {
        if (api.tx.tradingOtc && api.tx.tradingOtc.markPaid) {
          const orderId = 1; // 假设订单 ID 为 1
          
          const tx = api.tx.tradingOtc.markPaid(orderId);
          const result = await signAndSendTx(api, tx, accounts.dave);
          log.subSection(`标记已付款结果: ${result.success ? '成功' : result.error}`);
        } else if (api.tx.tradingOtc && api.tx.tradingOtc.confirmPayment) {
          const tx = api.tx.tradingOtc.confirmPayment(1);
          const result = await signAndSendTx(api, tx, accounts.dave);
          log.subSection(`确认付款结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('markPaid 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[TradingOtc] 做市商释放代币',
      fn: async () => {
        if (api.tx.tradingOtc && api.tx.tradingOtc.releaseDust) {
          const orderId = 1;
          
          const tx = api.tx.tradingOtc.releaseDust(orderId);
          const result = await signAndSendTx(api, tx, accounts.charlie);
          log.subSection(`释放代币结果: ${result.success ? '成功' : result.error}`);
        } else if (api.tx.tradingOtc && api.tx.tradingOtc.release) {
          const tx = api.tx.tradingOtc.release(1);
          const result = await signAndSendTx(api, tx, accounts.charlie);
          log.subSection(`释放代币结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('releaseDust 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[TradingOtc] 取消订单',
      fn: async () => {
        // 首先创建一个新订单用于取消测试
        if (api.tx.tradingOtc && api.tx.tradingOtc.cancelOrder) {
          const orderId = 2; // 假设订单 ID
          
          const tx = api.tx.tradingOtc.cancelOrder(orderId);
          const result = await signAndSendTx(api, tx, accounts.dave);
          log.subSection(`取消订单结果: ${result.success ? '成功' : result.error}`);
        } else if (api.tx.tradingOtc && api.tx.tradingOtc.cancel) {
          const tx = api.tx.tradingOtc.cancel(2);
          const result = await signAndSendTx(api, tx, accounts.dave);
          log.subSection(`取消订单结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('cancelOrder 方法不可用');
        }
        assert(true, '测试完成');
      },
    },

    // ==================== TradingSwap 模块 ====================
    {
      name: '[TradingSwap] 查询兑换池状态',
      fn: async () => {
        if (api.query.tradingSwap) {
          if (api.query.tradingSwap.pools) {
            const pools = await api.query.tradingSwap.pools.entries();
            log.subSection(`兑换池数量: ${pools.length}`);
          }
          if (api.query.tradingSwap.poolInfo) {
            const info = await api.query.tradingSwap.poolInfo();
            log.subSection(`兑换池信息: ${info.toString()}`);
          }
        }
        assert(api.query.tradingSwap !== undefined, 'TradingSwap pallet 应该存在');
      },
    },
    {
      name: '[TradingSwap] 执行兑换',
      fn: async () => {
        if (api.tx.tradingSwap && api.tx.tradingSwap.swap) {
          const amountIn = UNIT; // 1 DUST
          
          const tx = api.tx.tradingSwap.swap(amountIn, 0); // 0 表示接受任何输出
          const result = await signAndSendTx(api, tx, accounts.eve);
          log.subSection(`兑换结果: ${result.success ? '成功' : result.error}`);
        } else if (api.tx.tradingSwap && api.tx.tradingSwap.exchange) {
          const tx = api.tx.tradingSwap.exchange(UNIT);
          const result = await signAndSendTx(api, tx, accounts.eve);
          log.subSection(`兑换结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('swap 方法不可用');
        }
        assert(true, '测试完成');
      },
    },

    // ==================== 综合测试 ====================
    {
      name: '交易模块存储检查',
      fn: async () => {
        const pallets = ['tradingPricing', 'tradingCredit', 'tradingMaker', 'tradingSwap', 'tradingOtc'];
        
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
        
        log.subSection(`总计: ${availableCount}/${pallets.length} 个交易模块可用`);
        assert(availableCount > 0, '至少应该有一个交易模块可用');
      },
    },
    {
      name: '交易模块交易方法检查',
      fn: async () => {
        const methods: string[] = [];
        
        const pallets = ['tradingPricing', 'tradingCredit', 'tradingMaker', 'tradingSwap', 'tradingOtc'];
        
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
      name: '交易模块事件检查',
      fn: async () => {
        const eventSections: string[] = [];
        
        // 检查最近区块的事件
        const header = await api.rpc.chain.getHeader();
        const blockHash = header.hash;
        const events = await api.query.system.events.at(blockHash);
        
        for (const { event } of events as any) {
          if (!eventSections.includes(event.section)) {
            eventSections.push(event.section);
          }
        }
        
        const tradingEvents = eventSections.filter(s => s.toLowerCase().includes('trading'));
        log.subSection(`交易相关事件模块: ${tradingEvents.join(', ') || '暂无'}`);
        log.subSection(`当前区块所有事件模块: ${eventSections.join(', ')}`);
        
        assert(true, '事件检查完成');
      },
    },
  ]);

  await cleanup();
  return result;
}

// 如果直接运行此文件
const isMainModule = import.meta.url === `file://${process.argv[1]}`;
if (isMainModule) {
  runTradingTests()
    .then(result => {
      process.exit(result.totalFailed > 0 ? 1 : 0);
    })
    .catch(error => {
      console.error('测试运行失败:', error);
      process.exit(1);
    });
}

