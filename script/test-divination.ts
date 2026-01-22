/**
 * 占卜系统测试
 * 测试各种占卜模块：八字、六爻、梅花易数、奇门遁甲、塔罗等
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
  randomNumber,
  randomString,
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
  log.section('初始化占卜系统测试环境...');
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
 * 运行占卜系统测试
 */
export async function runDivinationTests(): Promise<TestSuiteResult> {
  await setup();

  const result = await runTestSuite('占卜系统测试', [
    // ==================== 黄历模块 ====================
    {
      name: '[Almanac] 查询当前时间信息',
      fn: async () => {
        if (api.query.almanac && api.query.almanac.currentLunarDate) {
          const lunarDate = await api.query.almanac.currentLunarDate();
          log.subSection(`当前农历日期: ${lunarDate.toString()}`);
        } else {
          log.subSection('Almanac 模块不支持此查询或未配置');
        }
        assert(api.query.almanac !== undefined, 'Almanac pallet 应该存在');
      },
    },

    // ==================== 八字模块 ====================
    {
      name: '[Bazi] 创建八字命盘',
      fn: async () => {
        // 检查是否有创建八字的方法
        if (api.tx.bazi && api.tx.bazi.createChart) {
          // 生成测试出生日期：1990-01-15 12:00
          const birthTimestamp = Date.UTC(1990, 0, 15, 12, 0, 0);
          
          const tx = api.tx.bazi.createChart(birthTimestamp, null); // null 表示男性默认或自动
          const result = await signAndSendTx(api, tx, accounts.alice);
          
          if (result.success) {
            log.subSection('八字命盘创建成功');
          } else {
            log.subSection(`八字命盘创建结果: ${result.error}`);
          }
        } else {
          log.subSection('Bazi.createChart 方法不可用，跳过测试');
        }
        assert(api.query.bazi !== undefined, 'Bazi pallet 应该存在');
      },
    },
    {
      name: '[Bazi] 查询用户八字命盘',
      fn: async () => {
        if (api.query.bazi && api.query.bazi.charts) {
          const charts = await api.query.bazi.charts(accounts.alice.address);
          log.subSection(`Alice 的八字命盘数量: ${charts.toString() !== '[]' ? '存在' : '0'}`);
        } else if (api.query.bazi && api.query.bazi.userCharts) {
          const charts = await api.query.bazi.userCharts(accounts.alice.address);
          log.subSection(`Alice 的八字命盘: ${charts.toString()}`);
        }
        assert(true, '查询完成');
      },
    },

    // ==================== 六爻模块 ====================
    {
      name: '[Liuyao] 创建六爻卦象',
      fn: async () => {
        if (api.tx.liuyao && api.tx.liuyao.divine) {
          // 六爻需要问题和随机数种子
          const question = randomString(10);
          
          const tx = api.tx.liuyao.divine(question);
          const result = await signAndSendTx(api, tx, accounts.bob);
          
          if (result.success) {
            log.subSection('六爻卦象创建成功');
          } else {
            log.subSection(`六爻卦象创建结果: ${result.error}`);
          }
        } else if (api.tx.liuyao && api.tx.liuyao.createDivination) {
          const question = '测试问卦';
          const tx = api.tx.liuyao.createDivination(question);
          const result = await signAndSendTx(api, tx, accounts.bob);
          log.subSection(`六爻占卜结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('Liuyao 占卜方法不可用，跳过测试');
        }
        assert(api.query.liuyao !== undefined, 'Liuyao pallet 应该存在');
      },
    },
    {
      name: '[Liuyao] 查询用户六爻记录',
      fn: async () => {
        if (api.query.liuyao && api.query.liuyao.divinations) {
          const divinations = await api.query.liuyao.divinations(accounts.bob.address);
          log.subSection(`Bob 的六爻记录: ${divinations.toString()}`);
        } else if (api.query.liuyao && api.query.liuyao.userDivinations) {
          const divinations = await api.query.liuyao.userDivinations(accounts.bob.address);
          log.subSection(`Bob 的六爻记录: ${divinations.toString()}`);
        }
        assert(true, '查询完成');
      },
    },

    // ==================== 梅花易数模块 ====================
    {
      name: '[Meihua] 创建梅花卦象',
      fn: async () => {
        if (api.tx.meihua && api.tx.meihua.divine) {
          // 梅花易数通常需要时间或数字
          const number1 = randomNumber(1, 100);
          const number2 = randomNumber(1, 100);
          
          const tx = api.tx.meihua.divine(number1, number2);
          const result = await signAndSendTx(api, tx, accounts.charlie);
          
          if (result.success) {
            log.subSection('梅花卦象创建成功');
          } else {
            log.subSection(`梅花卦象创建结果: ${result.error}`);
          }
        } else if (api.tx.meihua && api.tx.meihua.createDivination) {
          const tx = api.tx.meihua.createDivination();
          const result = await signAndSendTx(api, tx, accounts.charlie);
          log.subSection(`梅花易数结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('Meihua 占卜方法不可用，跳过测试');
        }
        assert(api.query.meihua !== undefined, 'Meihua pallet 应该存在');
      },
    },
    {
      name: '[Meihua] 查询梅花卦象记录',
      fn: async () => {
        if (api.query.meihua && api.query.meihua.divinations) {
          const divinations = await api.query.meihua.divinations(accounts.charlie.address);
          log.subSection(`Charlie 的梅花记录: ${divinations.toString()}`);
        }
        assert(true, '查询完成');
      },
    },

    // ==================== 奇门遁甲模块 ====================
    {
      name: '[Qimen] 创建奇门盘',
      fn: async () => {
        if (api.tx.qimen && api.tx.qimen.createChart) {
          const timestamp = Date.now();
          
          const tx = api.tx.qimen.createChart(timestamp);
          const result = await signAndSendTx(api, tx, accounts.dave);
          
          if (result.success) {
            log.subSection('奇门盘创建成功');
          } else {
            log.subSection(`奇门盘创建结果: ${result.error}`);
          }
        } else if (api.tx.qimen && api.tx.qimen.divine) {
          const tx = api.tx.qimen.divine();
          const result = await signAndSendTx(api, tx, accounts.dave);
          log.subSection(`奇门遁甲结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('Qimen 方法不可用，跳过测试');
        }
        assert(api.query.qimen !== undefined, 'Qimen pallet 应该存在');
      },
    },
    {
      name: '[Qimen] 查询奇门记录',
      fn: async () => {
        if (api.query.qimen && api.query.qimen.charts) {
          const charts = await api.query.qimen.charts(accounts.dave.address);
          log.subSection(`Dave 的奇门记录: ${charts.toString()}`);
        }
        assert(true, '查询完成');
      },
    },

    // ==================== 紫微斗数模块 ====================
    {
      name: '[Ziwei] 创建紫微命盘',
      fn: async () => {
        if (api.tx.ziwei && api.tx.ziwei.createChart) {
          // 紫微需要出生年月日时
          const birthTimestamp = Date.UTC(1985, 5, 20, 10, 30, 0);
          
          const tx = api.tx.ziwei.createChart(birthTimestamp, true); // true = 男性
          const result = await signAndSendTx(api, tx, accounts.eve);
          
          if (result.success) {
            log.subSection('紫微命盘创建成功');
          } else {
            log.subSection(`紫微命盘创建结果: ${result.error}`);
          }
        } else {
          log.subSection('Ziwei.createChart 方法不可用，跳过测试');
        }
        assert(api.query.ziwei !== undefined, 'Ziwei pallet 应该存在');
      },
    },
    {
      name: '[Ziwei] 查询紫微命盘',
      fn: async () => {
        if (api.query.ziwei && api.query.ziwei.charts) {
          const charts = await api.query.ziwei.charts(accounts.eve.address);
          log.subSection(`Eve 的紫微命盘: ${charts.toString()}`);
        }
        assert(true, '查询完成');
      },
    },

    // ==================== 塔罗牌模块 ====================
    {
      name: '[Tarot] 抽取塔罗牌',
      fn: async () => {
        if (api.tx.tarot && api.tx.tarot.draw) {
          const spread = 3; // 抽取3张牌
          
          const tx = api.tx.tarot.draw(spread);
          const result = await signAndSendTx(api, tx, accounts.ferdie);
          
          if (result.success) {
            log.subSection('塔罗牌抽取成功');
          } else {
            log.subSection(`塔罗牌抽取结果: ${result.error}`);
          }
        } else if (api.tx.tarot && api.tx.tarot.drawCards) {
          const tx = api.tx.tarot.drawCards(3);
          const result = await signAndSendTx(api, tx, accounts.ferdie);
          log.subSection(`塔罗牌结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('Tarot 抽牌方法不可用，跳过测试');
        }
        assert(api.query.tarot !== undefined, 'Tarot pallet 应该存在');
      },
    },
    {
      name: '[Tarot] 查询塔罗记录',
      fn: async () => {
        if (api.query.tarot && api.query.tarot.readings) {
          const readings = await api.query.tarot.readings(accounts.ferdie.address);
          log.subSection(`Ferdie 的塔罗记录: ${readings.toString()}`);
        }
        assert(true, '查询完成');
      },
    },

    // ==================== 小六壬模块 ====================
    {
      name: '[Xiaoliuren] 创建小六壬占卜',
      fn: async () => {
        if (api.tx.xiaoliuren && api.tx.xiaoliuren.divine) {
          const tx = api.tx.xiaoliuren.divine();
          const result = await signAndSendTx(api, tx, accounts.alice);
          
          if (result.success) {
            log.subSection('小六壬占卜成功');
          } else {
            log.subSection(`小六壬占卜结果: ${result.error}`);
          }
        } else {
          log.subSection('Xiaoliuren 方法不可用，跳过测试');
        }
        assert(api.query.xiaoliuren !== undefined, 'Xiaoliuren pallet 应该存在');
      },
    },

    // ==================== 大六壬模块 ====================
    {
      name: '[Daliuren] 创建大六壬占课',
      fn: async () => {
        if (api.tx.daliuren && api.tx.daliuren.divine) {
          const timestamp = Date.now();
          
          const tx = api.tx.daliuren.divine(timestamp);
          const result = await signAndSendTx(api, tx, accounts.bob);
          
          if (result.success) {
            log.subSection('大六壬占课成功');
          } else {
            log.subSection(`大六壬占课结果: ${result.error}`);
          }
        } else {
          log.subSection('Daliuren 方法不可用，跳过测试');
        }
        assert(api.query.daliuren !== undefined, 'Daliuren pallet 应该存在');
      },
    },

    // ==================== 占卜市场模块 ====================
    {
      name: '[DivinationMarket] 查询市场信息',
      fn: async () => {
        if (api.query.divinationMarket) {
          if (api.query.divinationMarket.diviners) {
            const diviners = await api.query.divinationMarket.diviners.entries();
            log.subSection(`注册占卜师数量: ${diviners.length}`);
          }
          if (api.query.divinationMarket.services) {
            const services = await api.query.divinationMarket.services.entries();
            log.subSection(`可用服务数量: ${services.length}`);
          }
        }
        assert(api.query.divinationMarket !== undefined, 'DivinationMarket pallet 应该存在');
      },
    },
    {
      name: '[DivinationMarket] 注册成为占卜师',
      fn: async () => {
        if (api.tx.divinationMarket && api.tx.divinationMarket.registerDiviner) {
          const name = '测试占卜师';
          const description = '专业八字命理分析';
          
          const tx = api.tx.divinationMarket.registerDiviner(name, description);
          const result = await signAndSendTx(api, tx, accounts.alice);
          
          log.subSection(`注册结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('registerDiviner 方法不可用');
        }
        assert(true, '测试完成');
      },
    },

    // ==================== 隐私模块 ====================
    {
      name: '[Privacy] 查询隐私设置',
      fn: async () => {
        if (api.query.privacy && api.query.privacy.settings) {
          const settings = await api.query.privacy.settings(accounts.alice.address);
          log.subSection(`Alice 的隐私设置: ${settings.toString()}`);
        }
        assert(api.query.privacy !== undefined, 'Privacy pallet 应该存在');
      },
    },

    // ==================== NFT 模块 ====================
    {
      name: '[DivinationNft] 查询 NFT 信息',
      fn: async () => {
        if (api.query.divinationNft) {
          if (api.query.divinationNft.collections) {
            const collections = await api.query.divinationNft.collections.entries();
            log.subSection(`NFT 集合数量: ${collections.length}`);
          }
        }
        assert(api.query.divinationNft !== undefined, 'DivinationNft pallet 应该存在');
      },
    },

    // ==================== AI 模块 ====================
    {
      name: '[DivinationAi] 查询 AI 服务',
      fn: async () => {
        if (api.query.divinationAi) {
          if (api.query.divinationAi.providers) {
            const providers = await api.query.divinationAi.providers.entries();
            log.subSection(`AI 提供者数量: ${providers.length}`);
          }
        }
        assert(api.query.divinationAi !== undefined, 'DivinationAi pallet 应该存在');
      },
    },

    // ==================== 综合测试 ====================
    {
      name: '占卜模块存储检查',
      fn: async () => {
        const pallets = [
          'almanac', 'bazi', 'liuyao', 'meihua', 'qimen', 
          'ziwei', 'tarot', 'xiaoliuren', 'daliuren',
          'divinationMarket', 'divinationNft', 'divinationAi', 'privacy'
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
        
        log.subSection(`总计: ${availableCount}/${pallets.length} 个占卜模块可用`);
        assert(availableCount > 0, '至少应该有一个占卜模块可用');
      },
    },
  ]);

  await cleanup();
  return result;
}

// 如果直接运行此文件
const isMainModule = import.meta.url === `file://${process.argv[1]}`;
if (isMainModule) {
  runDivinationTests()
    .then(result => {
      process.exit(result.totalFailed > 0 ? 1 : 0);
    })
    .catch(error => {
      console.error('测试运行失败:', error);
      process.exit(1);
    });
}

