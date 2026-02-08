/**
 * 做市商激活脚本
 * 完成做市商从 DepositLocked -> PendingReview -> Active 的完整流程
 * 
 * 使用方法:
 *   npx tsx activate-maker.ts [makerId]
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
  formatNxs,
} from './utils/helpers.js';

async function main() {
  const args = process.argv.slice(2);
  const targetMakerId = args[0] ? parseInt(args[0]) : 0;
  
  logSection('做市商激活工具');
  
  const api = await getApi();
  const alice = getAlice(); // Root 权限账户
  const bob = getBob();     // 做市商账户
  
  logAccount('Alice (Root)', alice);
  logAccount('Bob (做市商)', bob);
  
  try {
    // ========================================
    // 步骤 1: 查询做市商状态
    // ========================================
    logStep(1, `查询做市商 ID: ${targetMakerId}`);
    
    const makerApp = await (api.query as any).tradingMaker.makerApplications(targetMakerId);
    if (!makerApp.isSome) {
      logError(`做市商 ID ${targetMakerId} 不存在`);
      return;
    }
    
    let app = makerApp.unwrap();
    let status = app.status.toString();
    console.log(`   当前状态: ${status}`);
    console.log(`   账户: ${app.owner.toString()}`);
    console.log(`   押金: ${formatNxs(app.deposit.toString())}`);
    
    // ========================================
    // 步骤 2: 如果是 DepositLocked，提交资料
    // ========================================
    if (status === 'DepositLocked') {
      logStep(2, '提交做市商资料');
      
      // 检查调用者是否是做市商所有者
      const ownerAddress = app.owner.toString();
      if (ownerAddress !== bob.address) {
        logError(`做市商所有者不是 Bob，无法提交资料`);
        logInfo(`所有者: ${ownerAddress}`);
        logInfo(`Bob: ${bob.address}`);
        return;
      }
      
      const submitInfoTx = (api.tx as any).tradingMaker.submitInfo(
        'Zhang San',                                    // 真实姓名
        '110101199001011234',                           // 身份证号
        '1990-01-01',                                   // 生日
        'TYASr5UV6HEcXatwdFQfmLVUqQQQMUxHLS',           // TRON 地址 (34字符, Base58)
        'wechat_test_001'                               // 微信号
      );
      
      const submitResult = await signAndSend(api, submitInfoTx, bob, 'Bob 提交做市商资料');
      
      if (!submitResult.success) {
        logError(`提交资料失败: ${submitResult.error}`);
        return;
      }
      
      logSuccess('资料已提交');
      
      // 重新查询状态
      const updatedApp = await (api.query as any).tradingMaker.makerApplications(targetMakerId);
      app = updatedApp.unwrap();
      status = app.status.toString();
      console.log(`   更新后状态: ${status}`);
    } else {
      logInfo(`跳过步骤 2: 当前状态为 ${status}`);
    }
    
    // ========================================
    // 步骤 3: 如果是 PendingReview，审批通过
    // ========================================
    if (status === 'PendingReview') {
      logStep(3, '审批做市商');
      
      const approveTx = (api.tx as any).tradingMaker.approveMaker(targetMakerId);
      const sudoTx = api.tx.sudo.sudo(approveTx);
      
      const approveResult = await signAndSend(api, sudoTx, alice, 'Alice 审批做市商');
      
      if (!approveResult.success) {
        logError(`审批失败: ${approveResult.error}`);
        return;
      }
      
      logSuccess('审批通过');
      
      // 重新查询状态
      const finalApp = await (api.query as any).tradingMaker.makerApplications(targetMakerId);
      app = finalApp.unwrap();
      status = app.status.toString();
      console.log(`   最终状态: ${status}`);
    } else if (status === 'Active') {
      logInfo('做市商已激活，无需审批');
    } else {
      logInfo(`跳过步骤 3: 当前状态为 ${status}`);
    }
    
    // ========================================
    // 步骤 4: 显示最终状态
    // ========================================
    logStep(4, '最终状态');
    
    const finalMakerApp = await (api.query as any).tradingMaker.makerApplications(targetMakerId);
    if (finalMakerApp.isSome) {
      const finalApp = finalMakerApp.unwrap();
      console.log(`   做市商 ID: ${targetMakerId}`);
      console.log(`   状态: ${finalApp.status.toString()}`);
      console.log(`   押金: ${formatNxs(finalApp.deposit.toString())}`);
      console.log(`   服务暂停: ${finalApp.servicePaused ? '是' : '否'}`);
      
      if (finalApp.tronAddress && finalApp.tronAddress.length > 0) {
        const tronAddr = new TextDecoder().decode(new Uint8Array(finalApp.tronAddress));
        console.log(`   TRON 地址: ${tronAddr}`);
      }
      
      if (finalApp.status.isActive) {
        logSection('激活成功');
        logSuccess('🎉 做市商已激活，可以开始接单！');
      }
    }
    
  } catch (error: any) {
    logError(`执行失败: ${error.message}`);
    console.error(error);
  } finally {
    await disconnectApi();
  }
}

main().catch(console.error);
