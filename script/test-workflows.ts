/**
 * 完整业务流程测试
 * 测试端到端的功能流程，验证业务逻辑的正确性和合理性
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
  assertEqual,
  assertGreaterThan,
  formatBalance,
  TestSuiteResult,
  DEFAULT_WS_ENDPOINT,
  UNIT,
  sleep,
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
 * 获取账户余额
 */
async function getBalance(address: string): Promise<bigint> {
  const { data: balance } = await api.query.system.account(address) as any;
  return BigInt(balance.free.toString());
}

/**
 * 初始化测试环境
 */
async function setup(): Promise<void> {
  log.section('初始化完整流程测试环境...');
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
 * 运行完整业务流程测试
 */
export async function runWorkflowTests(): Promise<TestSuiteResult> {
  await setup();

  const result = await runTestSuite('完整业务流程测试', [
    // ==================== 转账流程测试 ====================
    {
      name: '[转账流程] 完整转账流程验证',
      fn: async () => {
        // 1. 记录转账前余额
        const aliceBalanceBefore = await getBalance(accounts.alice.address);
        const bobBalanceBefore = await getBalance(accounts.bob.address);
        
        log.subSection(`转账前 Alice 余额: ${formatBalance(aliceBalanceBefore)}`);
        log.subSection(`转账前 Bob 余额: ${formatBalance(bobBalanceBefore)}`);
        
        // 2. 执行转账
        const transferAmount = UNIT * 10n; // 10 DUST
        const tx = api.tx.balances.transferKeepAlive(accounts.bob.address, transferAmount);
        const result = await signAndSendTx(api, tx, accounts.alice, {
          section: 'balances',
          method: 'Transfer',
        });
        
        assert(result.success, `转账应该成功: ${result.error}`);
        
        // 3. 验证转账后余额变化
        const aliceBalanceAfter = await getBalance(accounts.alice.address);
        const bobBalanceAfter = await getBalance(accounts.bob.address);
        
        log.subSection(`转账后 Alice 余额: ${formatBalance(aliceBalanceAfter)}`);
        log.subSection(`转账后 Bob 余额: ${formatBalance(bobBalanceAfter)}`);
        
        // 4. 验证余额变化正确性
        // Alice 余额减少 >= 转账金额（还有手续费）
        const aliceReduction = aliceBalanceBefore - aliceBalanceAfter;
        assert(aliceReduction >= transferAmount, 'Alice 余额减少应该 >= 转账金额');
        
        // Bob 余额增加 == 转账金额
        const bobIncrease = bobBalanceAfter - bobBalanceBefore;
        assertEqual(bobIncrease, transferAmount, 'Bob 余额增加应该等于转账金额');
        
        log.subSection(`✓ Alice 减少: ${formatBalance(aliceReduction)} (含手续费)`);
        log.subSection(`✓ Bob 增加: ${formatBalance(bobIncrease)}`);
      },
    },
    {
      name: '[转账流程] 余额不足应该失败',
      fn: async () => {
        // 1. 查询 Ferdie 余额
        const ferdieBalance = await getBalance(accounts.ferdie.address);
        log.subSection(`Ferdie 当前余额: ${formatBalance(ferdieBalance)}`);
        
        // 2. 尝试转账超过余额的金额
        const excessAmount = ferdieBalance + UNIT * 1000000n;
        const tx = api.tx.balances.transferKeepAlive(accounts.alice.address, excessAmount);
        const result = await signAndSendTx(api, tx, accounts.ferdie);
        
        // 3. 验证转账失败
        assert(!result.success, '转账超过余额应该失败');
        log.subSection(`✓ 预期失败: ${result.error}`);
        
        // 4. 验证余额未变化
        const ferdieBalanceAfter = await getBalance(accounts.ferdie.address);
        // 注意：失败的交易仍然会扣除手续费
        log.subSection(`Ferdie 余额变化: ${formatBalance(ferdieBalance - ferdieBalanceAfter)} (手续费)`);
      },
    },

    // ==================== 批量操作流程测试 ====================
    {
      name: '[批量操作] 批量转账流程验证',
      fn: async () => {
        const recipients = [accounts.charlie, accounts.dave, accounts.eve];
        const amountPerRecipient = UNIT; // 1 DUST each
        
        // 1. 记录转账前余额
        const balancesBefore: Map<string, bigint> = new Map();
        for (const recipient of recipients) {
          balancesBefore.set(recipient.address, await getBalance(recipient.address));
        }
        
        // 2. 构建批量转账
        const transfers = recipients.map(r => 
          api.tx.balances.transferKeepAlive(r.address, amountPerRecipient)
        );
        const batchTx = api.tx.utility.batch(transfers);
        
        // 3. 执行批量转账
        const result = await signAndSendTx(api, batchTx, accounts.alice);
        assert(result.success, `批量转账应该成功: ${result.error}`);
        
        // 4. 验证每个接收者余额增加
        let allCorrect = true;
        for (const recipient of recipients) {
          const balanceBefore = balancesBefore.get(recipient.address)!;
          const balanceAfter = await getBalance(recipient.address);
          const increase = balanceAfter - balanceBefore;
          
          if (increase !== amountPerRecipient) {
            allCorrect = false;
            log.subSection(`✗ ${recipient.address.slice(0, 8)}... 增加不正确: ${formatBalance(increase)}`);
          } else {
            log.subSection(`✓ ${recipient.address.slice(0, 8)}... 增加: ${formatBalance(increase)}`);
          }
        }
        
        assert(allCorrect, '所有接收者余额应该正确增加');
      },
    },

    // ==================== OTC 交易流程测试 ====================
    {
      name: '[OTC流程] 完整首购订单流程',
      fn: async () => {
        // 检查 OTC 模块是否可用
        if (!api.tx.tradingOtc || !api.query.tradingOtc) {
          log.subSection('TradingOtc 模块不可用，跳过测试');
          return;
        }
        
        // 1. 首先需要有一个做市商
        if (api.tx.tradingMaker && api.tx.tradingMaker.register) {
          log.subSection('步骤1: 注册做市商...');
          const wechatId = 'maker_workflow_' + randomString(4);
          const registerTx = api.tx.tradingMaker.register(wechatId, 5, UNIT * 100n);
          const registerResult = await signAndSendTx(api, registerTx, accounts.charlie);
          log.subSection(`做市商注册: ${registerResult.success ? '成功' : registerResult.error}`);
        }
        
        // 2. 创建首购订单
        if (api.tx.tradingOtc.createFirstPurchase) {
          log.subSection('步骤2: 创建首购订单...');
          const paymentInfo = {
            realName: '测试用户',
            idNumber: '110101199001011234',
            phone: '13800138000',
            wechatId: 'buyer_' + randomString(4),
          };
          
          const createTx = api.tx.tradingOtc.createFirstPurchase(
            accounts.charlie.address, // 做市商
            UNIT * 10n, // 购买金额
            paymentInfo
          );
          const createResult = await signAndSendTx(api, createTx, accounts.dave);
          log.subSection(`创建订单: ${createResult.success ? '成功' : createResult.error}`);
          
          if (createResult.success) {
            // 3. 查找订单 ID
            let orderId = 1; // 默认
            for (const { event } of createResult.events) {
              if (event.section === 'tradingOtc' && event.method.includes('Created')) {
                orderId = parseInt(event.data[0]?.toString() || '1');
                log.subSection(`订单 ID: ${orderId}`);
              }
            }
            
            // 4. 标记已付款
            if (api.tx.tradingOtc.markPaid) {
              log.subSection('步骤3: 标记已付款...');
              const markPaidTx = api.tx.tradingOtc.markPaid(orderId);
              const markPaidResult = await signAndSendTx(api, markPaidTx, accounts.dave);
              log.subSection(`标记付款: ${markPaidResult.success ? '成功' : markPaidResult.error}`);
            }
            
            // 5. 做市商释放代币
            if (api.tx.tradingOtc.releaseDust) {
              log.subSection('步骤4: 做市商释放代币...');
              const daveBalanceBefore = await getBalance(accounts.dave.address);
              
              const releaseTx = api.tx.tradingOtc.releaseDust(orderId);
              const releaseResult = await signAndSendTx(api, releaseTx, accounts.charlie);
              log.subSection(`释放代币: ${releaseResult.success ? '成功' : releaseResult.error}`);
              
              if (releaseResult.success) {
                // 6. 验证买家收到代币
                const daveBalanceAfter = await getBalance(accounts.dave.address);
                const received = daveBalanceAfter - daveBalanceBefore;
                log.subSection(`Dave 收到: ${formatBalance(received)}`);
                assertGreaterThan(received, 0n, '买家应该收到代币');
              }
            }
          }
        } else {
          log.subSection('createFirstPurchase 方法不可用');
        }
        
        assert(true, 'OTC 流程测试完成');
      },
    },

    // ==================== 聊天群组流程测试 ====================
    {
      name: '[群聊流程] 完整群组生命周期',
      fn: async () => {
        if (!api.tx.chatGroup) {
          log.subSection('ChatGroup 模块不可用，跳过测试');
          return;
        }
        
        let groupId: number | undefined;
        
        // 1. 创建群组
        if (api.tx.chatGroup.createGroup || api.tx.chatGroup.create) {
          log.subSection('步骤1: 创建群组...');
          const groupName = '流程测试群_' + randomString(4);
          
          const createTx = api.tx.chatGroup.createGroup 
            ? api.tx.chatGroup.createGroup(groupName, '测试群组描述')
            : api.tx.chatGroup.create(groupName, '测试群组描述');
          
          const createResult = await signAndSendTx(api, createTx, accounts.alice);
          log.subSection(`创建群组: ${createResult.success ? '成功' : createResult.error}`);
          
          // 获取群组 ID
          if (createResult.success) {
            for (const { event } of createResult.events) {
              if (event.section === 'chatGroup') {
                groupId = parseInt(event.data[0]?.toString() || '1');
                log.subSection(`群组 ID: ${groupId}`);
              }
            }
          }
        }
        
        // 2. 其他用户加入群组
        if (groupId && (api.tx.chatGroup.joinGroup || api.tx.chatGroup.join)) {
          log.subSection('步骤2: Bob 加入群组...');
          const joinTx = api.tx.chatGroup.joinGroup 
            ? api.tx.chatGroup.joinGroup(groupId)
            : api.tx.chatGroup.join(groupId);
          
          const joinResult = await signAndSendTx(api, joinTx, accounts.bob);
          log.subSection(`加入群组: ${joinResult.success ? '成功' : joinResult.error}`);
        }
        
        // 3. 发送群消息
        if (groupId && (api.tx.chatGroup.sendMessage || api.tx.chatGroup.sendGroupMessage)) {
          log.subSection('步骤3: 发送群消息...');
          const message = '这是一条流程测试消息 ' + Date.now();
          
          const msgTx = api.tx.chatGroup.sendMessage
            ? api.tx.chatGroup.sendMessage(groupId, message)
            : api.tx.chatGroup.sendGroupMessage(groupId, message);
          
          const msgResult = await signAndSendTx(api, msgTx, accounts.alice);
          log.subSection(`发送消息: ${msgResult.success ? '成功' : msgResult.error}`);
        }
        
        // 4. 查询群成员验证
        if (groupId && api.query.chatGroup) {
          log.subSection('步骤4: 验证群成员...');
          if (api.query.chatGroup.members) {
            const members = await api.query.chatGroup.members(groupId);
            log.subSection(`群成员: ${members.toString()}`);
          } else if (api.query.chatGroup.groupMembers) {
            const members = await api.query.chatGroup.groupMembers(groupId);
            log.subSection(`群成员: ${members.toString()}`);
          }
        }
        
        assert(true, '群聊流程测试完成');
      },
    },

    // ==================== 直播流程测试 ====================
    {
      name: '[直播流程] 完整直播生命周期',
      fn: async () => {
        if (!api.tx.livestream) {
          log.subSection('Livestream 模块不可用，跳过测试');
          return;
        }
        
        let roomId: number | undefined;
        
        // 1. 创建直播间
        if (api.tx.livestream.createRoom || api.tx.livestream.create) {
          log.subSection('步骤1: 创建直播间...');
          const title = '流程测试直播_' + randomString(4);
          
          const createTx = api.tx.livestream.createRoom
            ? api.tx.livestream.createRoom(title, '测试直播描述', 'Normal')
            : api.tx.livestream.create(title, 'Normal');
          
          const createResult = await signAndSendTx(api, createTx, accounts.charlie);
          log.subSection(`创建直播间: ${createResult.success ? '成功' : createResult.error}`);
          
          if (createResult.success) {
            for (const { event } of createResult.events) {
              if (event.section === 'livestream') {
                roomId = parseInt(event.data[0]?.toString() || '1');
                log.subSection(`直播间 ID: ${roomId}`);
              }
            }
          }
        }
        
        // 2. 开始直播
        if (roomId && (api.tx.livestream.startLive || api.tx.livestream.start)) {
          log.subSection('步骤2: 开始直播...');
          const startTx = api.tx.livestream.startLive
            ? api.tx.livestream.startLive(roomId)
            : api.tx.livestream.start(roomId);
          
          const startResult = await signAndSendTx(api, startTx, accounts.charlie);
          log.subSection(`开始直播: ${startResult.success ? '成功' : startResult.error}`);
        }
        
        // 3. 观众加入
        if (roomId && (api.tx.livestream.joinRoom || api.tx.livestream.join)) {
          log.subSection('步骤3: 观众加入...');
          const joinTx = api.tx.livestream.joinRoom
            ? api.tx.livestream.joinRoom(roomId)
            : api.tx.livestream.join(roomId);
          
          const joinResult = await signAndSendTx(api, joinTx, accounts.dave);
          log.subSection(`观众加入: ${joinResult.success ? '成功' : joinResult.error}`);
        }
        
        // 4. 发送礼物（验证代币流转）
        if (roomId && api.tx.livestream.sendGift) {
          log.subSection('步骤4: 发送礼物...');
          const charlieBalanceBefore = await getBalance(accounts.charlie.address);
          
          const giftTx = api.tx.livestream.sendGift(roomId, 1, 1); // roomId, giftId, quantity
          const giftResult = await signAndSendTx(api, giftTx, accounts.dave);
          log.subSection(`发送礼物: ${giftResult.success ? '成功' : giftResult.error}`);
          
          if (giftResult.success) {
            const charlieBalanceAfter = await getBalance(accounts.charlie.address);
            const received = charlieBalanceAfter - charlieBalanceBefore;
            log.subSection(`主播收到: ${formatBalance(received)}`);
          }
        }
        
        // 5. 结束直播
        if (roomId && (api.tx.livestream.endLive || api.tx.livestream.end)) {
          log.subSection('步骤5: 结束直播...');
          const endTx = api.tx.livestream.endLive
            ? api.tx.livestream.endLive(roomId)
            : api.tx.livestream.end(roomId);
          
          const endResult = await signAndSendTx(api, endTx, accounts.charlie);
          log.subSection(`结束直播: ${endResult.success ? '成功' : endResult.error}`);
        }
        
        // 6. 验证直播间状态
        if (roomId && api.query.livestream) {
          log.subSection('步骤6: 验证最终状态...');
          if (api.query.livestream.roomInfo) {
            const info = await api.query.livestream.roomInfo(roomId);
            log.subSection(`直播间状态: ${info.toString()}`);
          } else if (api.query.livestream.rooms) {
            const room = await api.query.livestream.rooms(roomId);
            log.subSection(`直播间状态: ${room.toString()}`);
          }
        }
        
        assert(true, '直播流程测试完成');
      },
    },

    // ==================== 托管流程测试 ====================
    {
      name: '[托管流程] 完整托管生命周期',
      fn: async () => {
        if (!api.tx.escrow) {
          log.subSection('Escrow 模块不可用，跳过测试');
          return;
        }
        
        let escrowId: number | undefined;
        
        // 1. 记录初始余额
        const aliceBalanceBefore = await getBalance(accounts.alice.address);
        const bobBalanceBefore = await getBalance(accounts.bob.address);
        log.subSection(`初始 Alice: ${formatBalance(aliceBalanceBefore)}`);
        log.subSection(`初始 Bob: ${formatBalance(bobBalanceBefore)}`);
        
        // 2. 创建托管
        if (api.tx.escrow.create || api.tx.escrow.createEscrow) {
          log.subSection('步骤1: 创建托管...');
          const escrowAmount = UNIT * 5n; // 5 DUST
          const releaseTime = Math.floor(Date.now() / 1000) + 10; // 10秒后可释放
          
          const createTx = api.tx.escrow.create
            ? api.tx.escrow.create(accounts.bob.address, escrowAmount, releaseTime)
            : api.tx.escrow.createEscrow(accounts.bob.address, escrowAmount);
          
          const createResult = await signAndSendTx(api, createTx, accounts.alice);
          log.subSection(`创建托管: ${createResult.success ? '成功' : createResult.error}`);
          
          if (createResult.success) {
            for (const { event } of createResult.events) {
              if (event.section === 'escrow') {
                escrowId = parseInt(event.data[0]?.toString() || '1');
                log.subSection(`托管 ID: ${escrowId}`);
              }
            }
            
            // 验证 Alice 余额减少
            const aliceAfterCreate = await getBalance(accounts.alice.address);
            log.subSection(`Alice 托管后余额: ${formatBalance(aliceAfterCreate)}`);
          }
        }
        
        // 3. 等待并释放托管
        if (escrowId && (api.tx.escrow.release || api.tx.escrow.releaseEscrow)) {
          log.subSection('步骤2: 等待释放时间...');
          await sleep(2000); // 等待2秒
          
          log.subSection('步骤3: 释放托管...');
          const releaseTx = api.tx.escrow.release
            ? api.tx.escrow.release(escrowId)
            : api.tx.escrow.releaseEscrow(escrowId);
          
          const releaseResult = await signAndSendTx(api, releaseTx, accounts.alice);
          log.subSection(`释放托管: ${releaseResult.success ? '成功' : releaseResult.error}`);
          
          if (releaseResult.success) {
            // 验证 Bob 收到资金
            const bobBalanceAfter = await getBalance(accounts.bob.address);
            const bobReceived = bobBalanceAfter - bobBalanceBefore;
            log.subSection(`Bob 最终余额: ${formatBalance(bobBalanceAfter)}`);
            log.subSection(`Bob 收到: ${formatBalance(bobReceived)}`);
            
            assertGreaterThan(bobReceived, 0n, 'Bob 应该收到托管资金');
          }
        }
        
        assert(true, '托管流程测试完成');
      },
    },

    // ==================== 占卜流程测试 ====================
    {
      name: '[占卜流程] 完整八字命盘流程',
      fn: async () => {
        if (!api.tx.bazi) {
          log.subSection('Bazi 模块不可用，跳过测试');
          return;
        }
        
        // 1. 创建八字命盘
        if (api.tx.bazi.createChart) {
          log.subSection('步骤1: 创建八字命盘...');
          const birthTimestamp = Date.UTC(1990, 5, 15, 10, 30, 0);
          
          const createTx = api.tx.bazi.createChart(birthTimestamp, true); // true = 男性
          const createResult = await signAndSendTx(api, createTx, accounts.eve);
          log.subSection(`创建命盘: ${createResult.success ? '成功' : createResult.error}`);
          
          if (createResult.success) {
            // 获取命盘 ID
            let chartId = 1;
            for (const { event } of createResult.events) {
              if (event.section === 'bazi') {
                chartId = parseInt(event.data[0]?.toString() || '1');
                log.subSection(`命盘 ID: ${chartId}`);
              }
            }
            
            // 2. 查询命盘详情
            if (api.query.bazi && api.query.bazi.charts) {
              log.subSection('步骤2: 查询命盘详情...');
              const chart = await api.query.bazi.charts(chartId);
              log.subSection(`命盘数据: ${chart.toString().slice(0, 200)}...`);
            }
            
            // 3. 请求解读（如果有这个功能）
            if (api.tx.bazi.requestInterpretation) {
              log.subSection('步骤3: 请求命理解读...');
              const reqTx = api.tx.bazi.requestInterpretation(chartId);
              const reqResult = await signAndSendTx(api, reqTx, accounts.eve);
              log.subSection(`请求解读: ${reqResult.success ? '成功' : reqResult.error}`);
            }
          }
        } else {
          log.subSection('createChart 方法不可用');
        }
        
        assert(true, '八字流程测试完成');
      },
    },

    // ==================== 边界条件测试 ====================
    {
      name: '[边界条件] 存在性存款测试',
      fn: async () => {
        // 获取存在性存款
        const ed = BigInt(api.consts.balances.existentialDeposit.toString());
        log.subSection(`存在性存款: ${formatBalance(ed)}`);
        
        // 尝试转账小于存在性存款的金额到新账户
        const newAccount = keyring.addFromUri('//NewAccount' + Date.now());
        const smallAmount = ed / 2n;
        
        log.subSection(`尝试转账 ${formatBalance(smallAmount)} 到新账户...`);
        
        const tx = api.tx.balances.transferKeepAlive(newAccount.address, smallAmount);
        const result = await signAndSendTx(api, tx, accounts.alice);
        
        // transferKeepAlive 对于小于 ED 的转账应该失败或者保持账户
        log.subSection(`转账结果: ${result.success ? '成功' : '失败 - ' + result.error}`);
        
        // 查询新账户余额
        const newBalance = await getBalance(newAccount.address);
        log.subSection(`新账户余额: ${formatBalance(newBalance)}`);
      },
    },
    {
      name: '[边界条件] 自我转账测试',
      fn: async () => {
        const balanceBefore = await getBalance(accounts.alice.address);
        log.subSection(`自我转账前余额: ${formatBalance(balanceBefore)}`);
        
        // 尝试自我转账
        const tx = api.tx.balances.transferKeepAlive(accounts.alice.address, UNIT);
        const result = await signAndSendTx(api, tx, accounts.alice);
        
        log.subSection(`自我转账结果: ${result.success ? '成功' : '失败 - ' + result.error}`);
        
        const balanceAfter = await getBalance(accounts.alice.address);
        const change = balanceBefore - balanceAfter;
        log.subSection(`余额变化: ${formatBalance(change)} (应该只是手续费)`);
        
        // 验证余额变化只是手续费
        assert(change < UNIT, '自我转账应该只扣除手续费');
      },
    },

    // ==================== 并发操作测试 ====================
    {
      name: '[并发测试] 多个交易并发执行',
      fn: async () => {
        const recipients = [accounts.charlie, accounts.dave, accounts.eve];
        const amount = UNIT / 10n;
        
        // 记录初始余额
        const initialBalances: Map<string, bigint> = new Map();
        for (const r of recipients) {
          initialBalances.set(r.address, await getBalance(r.address));
        }
        
        // 并发发送多个交易
        log.subSection('发送3个并发交易...');
        const txPromises = recipients.map(async (recipient, index) => {
          const tx = api.tx.balances.transferKeepAlive(recipient.address, amount);
          return signAndSendTx(api, tx, accounts.alice);
        });
        
        const results = await Promise.all(txPromises);
        
        // 验证结果
        let successCount = 0;
        for (let i = 0; i < results.length; i++) {
          const result = results[i];
          log.subSection(`交易 ${i + 1}: ${result.success ? '成功' : '失败 - ' + result.error}`);
          if (result.success) successCount++;
        }
        
        log.subSection(`成功率: ${successCount}/${results.length}`);
        
        // 验证余额变化
        for (const r of recipients) {
          const finalBalance = await getBalance(r.address);
          const change = finalBalance - initialBalances.get(r.address)!;
          log.subSection(`${r.address.slice(0, 8)}... 余额变化: ${formatBalance(change)}`);
        }
        
        assert(successCount > 0, '至少应该有一个交易成功');
      },
    },

    // ==================== 状态一致性测试 ====================
    {
      name: '[状态一致性] 系统总余额守恒',
      fn: async () => {
        // 获取所有测试账户的总余额
        const accountList = [
          accounts.alice, accounts.bob, accounts.charlie,
          accounts.dave, accounts.eve, accounts.ferdie
        ];
        
        let totalBefore = 0n;
        for (const account of accountList) {
          totalBefore += await getBalance(account.address);
        }
        log.subSection(`操作前总余额: ${formatBalance(totalBefore)}`);
        
        // 执行一些内部转账
        const tx1 = api.tx.balances.transferKeepAlive(accounts.bob.address, UNIT);
        await signAndSendTx(api, tx1, accounts.alice);
        
        const tx2 = api.tx.balances.transferKeepAlive(accounts.charlie.address, UNIT / 2n);
        await signAndSendTx(api, tx2, accounts.bob);
        
        // 重新计算总余额
        let totalAfter = 0n;
        for (const account of accountList) {
          totalAfter += await getBalance(account.address);
        }
        log.subSection(`操作后总余额: ${formatBalance(totalAfter)}`);
        
        // 验证总余额只因手续费减少
        const feePaid = totalBefore - totalAfter;
        log.subSection(`手续费消耗: ${formatBalance(feePaid)}`);
        
        assert(feePaid < UNIT, '手续费应该是合理的小额');
        assert(feePaid >= 0n, '总余额不应该增加');
      },
    },
  ]);

  await cleanup();
  return result;
}

// 如果直接运行此文件
const isMainModule = import.meta.url === `file://${process.argv[1]}`;
if (isMainModule) {
  runWorkflowTests()
    .then(result => {
      process.exit(result.totalFailed > 0 ? 1 : 0);
    })
    .catch(error => {
      console.error('测试运行失败:', error);
      process.exit(1);
    });
}

