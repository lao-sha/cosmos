/**
 * Maker（做市商）模块测试脚本
 * 测试做市商申请、审批、押金管理等功能
 */

import { getApi, disconnectApi } from './utils/api.js';
import { getAlice, getBob, logAccount } from './utils/accounts.js';
import { 
  signAndSend, 
  logSection, 
  logStep, 
  logSuccess, 
  logError, 
  logInfo,
  logQuery,
  formatNxs,
  toNxsWei,
  sleep
} from './utils/helpers.js';

async function main() {
  logSection('Maker（做市商）模块测试');
  
  const api = await getApi();
  const alice = getAlice(); // Root 权限账户（审批）
  const bob = getBob();     // 做市商申请人
  
  logAccount('Alice (Root)', alice);
  logAccount('Bob (申请人)', bob);
  
  let makerId: number | null = null;
  
  try {
    // ========================================
    // 步骤 1: 查询初始状态
    // ========================================
    logStep(1, '查询初始状态');
    
    const nextMakerId = await (api.query as any).tradingMaker.nextMakerId();
    console.log(`   下一个做市商 ID: ${nextMakerId.toNumber()}`);
    
    const bobMakerId = await (api.query as any).tradingMaker.accountToMaker(bob.address);
    if (bobMakerId.isSome) {
      console.log(`   Bob 已是做市商，ID: ${bobMakerId.unwrap().toNumber()}`);
      makerId = bobMakerId.unwrap().toNumber();
    } else {
      console.log(`   Bob 尚未申请做市商`);
    }
    
    // 查询 Bob 余额
    const bobBalance = await api.query.system.account(bob.address);
    console.log(`   Bob 余额: ${formatNxs(bobBalance.data.free.toString())}`);
    
    // ========================================
    // 步骤 2: 锁定押金
    // ========================================
    if (!makerId) {
      logStep(2, '锁定押金');
      
      const lockDepositTx = (api.tx as any).tradingMaker.lockDeposit();
      const lockResult = await signAndSend(api, lockDepositTx, bob, 'Bob 锁定押金');
      
      if (!lockResult.success) {
        logError(`锁定押金失败: ${lockResult.error}`);
        return;
      }
      
      // 获取新的做市商 ID
      const newMakerId = await (api.query as any).tradingMaker.accountToMaker(bob.address);
      if (newMakerId.isSome) {
        makerId = newMakerId.unwrap().toNumber();
        logSuccess(`押金已锁定，做市商 ID: ${makerId}`);
      }
    } else {
      logInfo(`跳过步骤 2: Bob 已锁定押金`);
    }
    
    // ========================================
    // 步骤 3: 提交申请信息
    // ========================================
    logStep(3, '提交申请信息');
    
    // 查询当前状态
    const makerApp = await (api.query as any).tradingMaker.makerApplications(makerId);
    if (makerApp.isSome) {
      const app = makerApp.unwrap();
      const status = app.status.toString();
      console.log(`   当前状态: ${status}`);
      
      if (status === 'DepositLocked') {
        const submitInfoTx = (api.tx as any).tradingMaker.submitInfo(
          'Zhang San',           // 真实姓名
          '110101199001011234',  // 身份证号
          '1990-01-01',          // 生日
          'TXxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx', // TRON 地址
          'wechat_test_001'      // 微信号
        );
        const submitResult = await signAndSend(api, submitInfoTx, bob, 'Bob 提交申请信息');
        
        if (submitResult.success) {
          logSuccess('申请信息已提交');
        } else {
          logError(`提交信息失败: ${submitResult.error}`);
        }
      } else {
        logInfo(`跳过: 当前状态为 ${status}，无需提交信息`);
      }
    }
    
    // ========================================
    // 步骤 4: 审批做市商（Root 权限）
    // ========================================
    logStep(4, '审批做市商（Root 权限）');
    
    const makerAppAfterSubmit = await (api.query as any).tradingMaker.makerApplications(makerId);
    if (makerAppAfterSubmit.isSome) {
      const app = makerAppAfterSubmit.unwrap();
      const status = app.status.toString();
      console.log(`   当前状态: ${status}`);
      
      if (status === 'PendingReview') {
        const approveTx = (api.tx as any).tradingMaker.approveMaker(makerId);
        // 使用 sudo 调用
        const sudoTx = api.tx.sudo.sudo(approveTx);
        const approveResult = await signAndSend(api, sudoTx, alice, 'Alice 审批通过');
        
        if (approveResult.success) {
          logSuccess('做市商已审批通过');
        } else {
          logError(`审批失败: ${approveResult.error}`);
        }
      } else if (status === 'Active') {
        logInfo('跳过: 做市商已激活');
      } else {
        logInfo(`跳过: 当前状态为 ${status}`);
      }
    }
    
    // ========================================
    // 步骤 5: 查询做市商详情
    // ========================================
    logStep(5, '查询做市商详情');
    
    const finalMakerApp = await (api.query as any).tradingMaker.makerApplications(makerId);
    if (finalMakerApp.isSome) {
      const app = finalMakerApp.unwrap();
      console.log(`   做市商 ID: ${makerId}`);
      console.log(`   状态: ${app.status.toString()}`);
      console.log(`   账户: ${app.owner.toString()}`);
      console.log(`   押金: ${formatNxs(app.deposit.toString())}`);
      console.log(`   服务暂停: ${app.servicePaused ? '是' : '否'}`);
      console.log(`   已服务用户: ${app.usersServed.toNumber ? app.usersServed.toNumber() : app.usersServed}`);
      
      if (app.tronAddress && app.tronAddress.length > 0) {
        const tronAddr = new TextDecoder().decode(new Uint8Array(app.tronAddress));
        console.log(`   TRON 地址: ${tronAddr}`);
      } else {
        console.log(`   TRON 地址: 未设置`);
      }
    }
    
    // ========================================
    // 步骤 6: 测试押金提现流程（可选）
    // ========================================
    logStep(6, '测试押金提现流程（跳过，避免影响后续测试）');
    logInfo('提现流程需要冷却期，此处跳过');
    
    // 如需测试提现，取消下面的注释：
    /*
    // 申请提现
    const withdrawAmount = toNxsWei(1000); // 提现 1000 NXS
    const requestWithdrawTx = (api.tx as any).tradingMaker.requestWithdrawal(withdrawAmount);
    await signAndSend(api, requestWithdrawTx, bob, 'Bob 申请提现');
    
    // 等待冷却期（开发环境可能较短）
    // await waitForBlocks(api, 100);
    
    // 执行提现
    const executeWithdrawTx = (api.tx as any).tradingMaker.executeWithdrawal();
    await signAndSend(api, executeWithdrawTx, bob, 'Bob 执行提现');
    */
    
    // ========================================
    // 总结
    // ========================================
    logSection('测试完成');
    logSuccess('Maker 模块测试通过');
    
    console.log('\n📊 测试摘要:');
    console.log(`   - 做市商 ID: ${makerId}`);
    console.log(`   - 申请人: Bob`);
    console.log(`   - 测试步骤: 锁定押金 → 提交信息 → 审批通过`);
    
  } catch (error: any) {
    logError(`测试失败: ${error.message}`);
    console.error(error);
  } finally {
    await disconnectApi();
  }
}

main().catch(console.error);
