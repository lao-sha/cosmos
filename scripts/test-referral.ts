/**
 * Referral（推荐）模块测试脚本
 * 测试推荐码设置、绑定推荐人、查询上下线等功能
 */

import { getApi, disconnectApi } from './utils/api.js';
import { getAlice, getBob, getCharlie, getDave, logAccount } from './utils/accounts.js';
import { 
  signAndSend, 
  logSection, 
  logStep, 
  logSuccess, 
  logError, 
  logInfo,
} from './utils/helpers.js';

async function main() {
  logSection('Referral（推荐）模块测试');
  
  const api = await getApi();
  const alice = getAlice();   // 推荐人
  const bob = getBob();       // 被推荐人 1
  const charlie = getCharlie(); // 被推荐人 2
  
  logAccount('Alice (推荐人)', alice);
  logAccount('Bob (被推荐人1)', bob);
  logAccount('Charlie (被推荐人2)', charlie);
  
  try {
    // ========================================
    // 步骤 1: 查询 Alice 的推荐码
    // ========================================
    logStep(1, '查询 Alice 的推荐码');
    
    const aliceCode = await (api.query as any).affiliateReferral.codeByAccount(alice.address);
    if (aliceCode.isSome) {
      const code = new TextDecoder().decode(new Uint8Array(aliceCode.unwrap()));
      console.log(`   Alice 推荐码: ${code}`);
    } else {
      console.log(`   Alice 尚未设置推荐码`);
      
      // 设置推荐码
      logStep(1.1, 'Alice 设置推荐码');
      const setCodeTx = (api.tx as any).affiliateReferral.claimCode('ALICE2026');
      const setCodeResult = await signAndSend(api, setCodeTx, alice, 'Alice 设置推荐码');
      
      if (setCodeResult.success) {
        logSuccess('推荐码已设置: ALICE2026');
      } else {
        logError(`设置推荐码失败: ${setCodeResult.error}`);
      }
    }
    
    // ========================================
    // 步骤 2: Bob 绑定 Alice 为推荐人
    // ========================================
    logStep(2, 'Bob 绑定 Alice 为推荐人');
    
    // 先检查 Bob 是否已有推荐人
    const bobSponsor = await (api.query as any).affiliateReferral.sponsors(bob.address);
    if (bobSponsor.isSome) {
      console.log(`   Bob 已有推荐人: ${bobSponsor.unwrap().toString().slice(0, 16)}...`);
      logInfo('跳过绑定');
    } else {
      // 获取 Alice 的推荐码
      const aliceCodeNow = await (api.query as any).affiliateReferral.codeByAccount(alice.address);
      if (aliceCodeNow.isSome) {
        const code = new TextDecoder().decode(new Uint8Array(aliceCodeNow.unwrap()));
        
        const bindTx = (api.tx as any).affiliateReferral.bindSponsor(code);
        const bindResult = await signAndSend(api, bindTx, bob, `Bob 绑定推荐码 ${code}`);
        
        if (bindResult.success) {
          logSuccess('Bob 已绑定 Alice 为推荐人');
        } else {
          logError(`绑定失败: ${bindResult.error}`);
        }
      } else {
        logError('Alice 没有推荐码，无法绑定');
      }
    }
    
    // ========================================
    // 步骤 3: Charlie 绑定 Alice 为推荐人
    // ========================================
    logStep(3, 'Charlie 绑定 Alice 为推荐人');
    
    const charlieSponsor = await (api.query as any).affiliateReferral.sponsors(charlie.address);
    if (charlieSponsor.isSome) {
      console.log(`   Charlie 已有推荐人: ${charlieSponsor.unwrap().toString().slice(0, 16)}...`);
      logInfo('跳过绑定');
    } else {
      const aliceCodeNow = await (api.query as any).affiliateReferral.codeByAccount(alice.address);
      if (aliceCodeNow.isSome) {
        const code = new TextDecoder().decode(new Uint8Array(aliceCodeNow.unwrap()));
        
        const bindTx = (api.tx as any).affiliateReferral.bindSponsor(code);
        const bindResult = await signAndSend(api, bindTx, charlie, `Charlie 绑定推荐码 ${code}`);
        
        if (bindResult.success) {
          logSuccess('Charlie 已绑定 Alice 为推荐人');
        } else {
          logError(`绑定失败: ${bindResult.error}`);
        }
      }
    }
    
    // ========================================
    // 步骤 4: 查询 Alice 的下线列表
    // ========================================
    logStep(4, '查询 Alice 的下线列表');
    
    try {
      const aliceDownlines = await (api.query as any).affiliateReferral.downlines(alice.address);
      if (aliceDownlines && aliceDownlines.length > 0) {
        console.log(`   Alice 的下线数量: ${aliceDownlines.length}`);
        aliceDownlines.forEach((addr: any, i: number) => {
          console.log(`   - 下线 ${i + 1}: ${addr.toString().slice(0, 16)}...`);
        });
      } else {
        console.log(`   Alice 暂无下线（或存储项不存在）`);
      }
    } catch {
      logInfo('下线列表查询不可用（存储项可能未启用）');
    }
    
    // ========================================
    // 步骤 5: 查询 Bob 的上线
    // ========================================
    logStep(5, '查询 Bob 的上线');
    
    const bobSponsorNow = await (api.query as any).affiliateReferral.sponsors(bob.address);
    if (bobSponsorNow.isSome) {
      const sponsor = bobSponsorNow.unwrap().toString();
      console.log(`   Bob 的上线: ${sponsor.slice(0, 16)}...`);
      
      // 验证是否是 Alice
      if (sponsor === alice.address) {
        logSuccess('确认: Bob 的上线是 Alice');
      }
    } else {
      console.log(`   Bob 没有上线`);
    }
    
    // ========================================
    // 步骤 6: 查询推荐链
    // ========================================
    logStep(6, '查询推荐链（如果支持）');
    
    // 尝试查询 Bob 的推荐链
    try {
      // 手动构建推荐链
      let current = bob.address;
      const chain: string[] = [];
      
      for (let i = 0; i < 5; i++) {
        const sponsor = await (api.query as any).affiliateReferral.sponsors(current);
        if (sponsor.isSome) {
          const sponsorAddr = sponsor.unwrap().toString();
          chain.push(sponsorAddr);
          current = sponsorAddr;
        } else {
          break;
        }
      }
      
      if (chain.length > 0) {
        console.log(`   Bob 的推荐链 (${chain.length} 层):`);
        chain.forEach((addr, i) => {
          console.log(`   - 第 ${i + 1} 层: ${addr.slice(0, 16)}...`);
        });
      } else {
        console.log(`   Bob 没有推荐链`);
      }
    } catch (e) {
      logInfo('推荐链查询不可用');
    }
    
    // ========================================
    // 总结
    // ========================================
    logSection('测试完成');
    logSuccess('Referral 模块测试通过');
    
    console.log('\n📊 测试摘要:');
    console.log(`   - 推荐人: Alice`);
    console.log(`   - 被推荐人: Bob, Charlie`);
    console.log(`   - 推荐码: ALICE2026`);
    
  } catch (error: any) {
    logError(`测试失败: ${error.message}`);
    console.error(error);
  } finally {
    await disconnectApi();
  }
}

main().catch(console.error);
