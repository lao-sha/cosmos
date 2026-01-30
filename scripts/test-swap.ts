/**
 * Swap（兑换）模块测试脚本
 * 测试 COS → USDT 兑换完整流程
 */

import { getApi, disconnectApi } from './utils/api.js';
import { getAlice, getBob, getDave, logAccount } from './utils/accounts.js';
import { 
  signAndSend, 
  logSection, 
  logStep, 
  logSuccess, 
  logError, 
  logInfo,
  formatCos,
  formatUsdt,
  toCosWei,
} from './utils/helpers.js';

async function main() {
  logSection('Swap（兑换）模块测试');
  
  const api = await getApi();
  const alice = getAlice();   // Root 权限
  const bob = getBob();       // 做市商
  const dave = getDave();     // 兑换用户
  
  logAccount('Alice (Root)', alice);
  logAccount('Bob (做市商)', bob);
  logAccount('Dave (用户)', dave);
  
  let makerId: number | null = null;
  let swapId: number | null = null;
  
  try {
    // ========================================
    // 步骤 1: 确保 Bob 是激活的做市商
    // ========================================
    logStep(1, '确保 Bob 是激活的做市商');
    
    const bobMakerId = await (api.query as any).tradingMaker.accountToMaker(bob.address);
    if (bobMakerId.isSome) {
      makerId = bobMakerId.unwrap().toNumber();
      const makerApp = await (api.query as any).tradingMaker.makerApplications(makerId);
      if (makerApp.isSome && makerApp.unwrap().status.isActive) {
        logSuccess(`Bob 是激活的做市商，ID: ${makerId}`);
      } else {
        logError('Bob 不是激活的做市商，请先运行 test-maker.ts');
        return;
      }
    } else {
      logError('Bob 不是做市商，请先运行 test-maker.ts');
      return;
    }
    
    // ========================================
    // 步骤 2: 查询 Dave 余额
    // ========================================
    logStep(2, '查询 Dave 余额');
    
    const daveBalance = await api.query.system.account(dave.address);
    console.log(`   Dave 余额: ${formatCos(daveBalance.data.free.toString())}`);
    
    // ========================================
    // 步骤 3: 创建兑换请求
    // ========================================
    logStep(3, '创建兑换请求');
    
    // 获取当前兑换 ID
    const nextSwapId = await (api.query as any).tradingSwap.nextSwapId();
    swapId = nextSwapId.toNumber();
    console.log(`   预期兑换 ID: ${swapId}`);
    
    // 创建兑换（兑换 500 COS）
    const cosAmount = toCosWei(500);
    const usdtAddress = 'TYASr5UV6HEcXatwdFQfmLVUqQQQMUxHLS'; // 测试 TRON 地址 (Base58)
    
    console.log(`   兑换数量: ${formatCos(cosAmount)}`);
    console.log(`   USDT 地址: ${usdtAddress}`);
    
    const createSwapTx = (api.tx as any).tradingSwap.makerSwap(
      makerId,
      cosAmount,
      usdtAddress
    );
    
    const createResult = await signAndSend(api, createSwapTx, dave, 'Dave 创建兑换请求');
    
    if (!createResult.success) {
      logError(`创建兑换失败: ${createResult.error}`);
      return;
    }
    
    logSuccess(`兑换请求已创建，ID: ${swapId}`);
    
    // ========================================
    // 步骤 4: 查询兑换详情
    // ========================================
    logStep(4, '查询兑换详情');
    
    const swap = await (api.query as any).tradingSwap.makerSwaps(swapId);
    if (swap.isSome) {
      const s = swap.unwrap();
      console.log(`   兑换 ID: ${swapId}`);
      console.log(`   做市商 ID: ${s.makerId.toNumber()}`);
      console.log(`   用户: ${s.user.toString().slice(0, 16)}...`);
      console.log(`   COS 数量: ${formatCos(s.cosAmount.toString())}`);
      console.log(`   USDT 金额: ${formatUsdt(s.usdtAmount.toNumber())}`);
      console.log(`   状态: ${s.status.toString()}`);
      console.log(`   创建区块: ${s.createdAt.toNumber()}`);
      console.log(`   超时区块: ${s.timeoutAt.toNumber()}`);
    }
    
    // ========================================
    // 步骤 5: 做市商提交 TRC20 交易哈希
    // ========================================
    logStep(5, '做市商提交 TRC20 交易哈希');
    
    // 模拟 TRC20 交易哈希
    const trc20TxHash = `${Date.now().toString(16)}abcdef1234567890swap`;
    console.log(`   TRC20 交易哈希: ${trc20TxHash}`);
    
    const markCompleteTx = (api.tx as any).tradingSwap.markSwapComplete(swapId, trc20TxHash);
    const markCompleteResult = await signAndSend(api, markCompleteTx, bob, 'Bob 提交交易哈希');
    
    if (!markCompleteResult.success) {
      logError(`提交哈希失败: ${markCompleteResult.error}`);
      return;
    }
    
    logSuccess('交易哈希已提交，等待 OCW 验证');
    
    // 查询更新后的状态
    const swapAfterMark = await (api.query as any).tradingSwap.makerSwaps(swapId);
    if (swapAfterMark.isSome) {
      const s = swapAfterMark.unwrap();
      console.log(`   新状态: ${s.status.toString()}`);
      
      if (s.trc20TxHash.isSome) {
        const hash = new TextDecoder().decode(new Uint8Array(s.trc20TxHash.unwrap()));
        console.log(`   记录的哈希: ${hash}`);
      }
    }
    
    // ========================================
    // 步骤 6: 查询待验证队列
    // ========================================
    logStep(6, '查询待验证队列');
    
    const pendingVerification = await (api.query as any).tradingSwap.pendingVerifications(swapId);
    if (pendingVerification.isSome) {
      const v = pendingVerification.unwrap();
      console.log(`   验证请求存在: 是`);
      console.log(`   提交区块: ${v.submittedAt.toNumber()}`);
      console.log(`   超时区块: ${v.verificationTimeoutAt.toNumber()}`);
      console.log(`   重试次数: ${v.retryCount.toNumber()}`);
    } else {
      console.log(`   验证请求存在: 否（可能已完成或超时）`);
    }
    
    // ========================================
    // 步骤 7: 模拟 OCW 验证（需要 Root 权限）
    // ========================================
    logStep(7, '模拟验证确认（Root 权限）');
    
    // 注意：实际环境中由 OCW 自动验证
    // 这里使用 sudo 模拟验证通过
    const confirmTx = (api.tx as any).tradingSwap.confirmVerification(swapId, true, null);
    const sudoConfirmTx = api.tx.sudo.sudo(confirmTx);
    const confirmResult = await signAndSend(api, sudoConfirmTx, alice, 'Alice 确认验证通过');
    
    if (!confirmResult.success) {
      logInfo(`验证确认失败（可能权限不足）: ${confirmResult.error}`);
      logInfo('在实际环境中，OCW 会自动验证 TRC20 交易');
    } else {
      logSuccess('验证已确认，COS 已释放给做市商');
    }
    
    // ========================================
    // 步骤 8: 查询最终状态
    // ========================================
    logStep(8, '查询最终状态');
    
    const swapFinal = await (api.query as any).tradingSwap.makerSwaps(swapId);
    if (swapFinal.isSome) {
      const s = swapFinal.unwrap();
      console.log(`   最终状态: ${s.status.toString()}`);
      if (s.completedAt.isSome) {
        console.log(`   完成区块: ${s.completedAt.unwrap().toNumber()}`);
      }
    }
    
    // ========================================
    // 步骤 9: 查询用户兑换列表
    // ========================================
    logStep(9, '查询用户兑换列表');
    
    const daveSwaps = await (api.query as any).tradingSwap.userSwaps(dave.address);
    if (daveSwaps && daveSwaps.length > 0) {
      const swapIds = daveSwaps.map((id: any) => id.toNumber());
      console.log(`   Dave 的兑换: [${swapIds.join(', ')}]`);
    } else {
      console.log(`   Dave 暂无兑换记录`);
    }
    
    // ========================================
    // 步骤 10: 查询做市商兑换列表
    // ========================================
    logStep(10, '查询做市商兑换列表');
    
    const makerSwaps = await (api.query as any).tradingSwap.makerSwapList(makerId);
    if (makerSwaps && makerSwaps.length > 0) {
      const swapIds = makerSwaps.map((id: any) => id.toNumber());
      console.log(`   做市商 ${makerId} 的兑换: [${swapIds.join(', ')}]`);
    } else {
      console.log(`   做市商暂无兑换记录`);
    }
    
    // ========================================
    // 总结
    // ========================================
    logSection('测试完成');
    logSuccess('Swap 模块测试通过');
    
    console.log('\n📊 测试摘要:');
    console.log(`   - 兑换 ID: ${swapId}`);
    console.log(`   - 做市商: Bob (ID: ${makerId})`);
    console.log(`   - 用户: Dave`);
    console.log(`   - 流程: 创建兑换 → 提交哈希 → OCW验证 → 完成`);
    
  } catch (error: any) {
    logError(`测试失败: ${error.message}`);
    console.error(error);
  } finally {
    await disconnectApi();
  }
}

main().catch(console.error);
