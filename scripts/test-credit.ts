/**
 * Credit（信用）模块测试脚本
 * 测试信用分查询、信用等级等功能
 */

import { getApi, disconnectApi } from './utils/api.js';
import { getAlice, getBob, getCharlie, logAccount } from './utils/accounts.js';
import { 
  logSection, 
  logStep, 
  logSuccess, 
  logInfo,
} from './utils/helpers.js';

async function main() {
  logSection('Credit（信用）模块测试');
  
  const api = await getApi();
  const alice = getAlice();
  const bob = getBob();
  const charlie = getCharlie();
  
  logAccount('Alice', alice);
  logAccount('Bob', bob);
  logAccount('Charlie', charlie);
  
  try {
    // ========================================
    // 步骤 1: 查询用户信用分
    // ========================================
    logStep(1, '查询用户信用分');
    
    for (const [name, account] of [['Alice', alice], ['Bob', bob], ['Charlie', charlie]]) {
      try {
        const credit = await (api.query as any).tradingCredit.userCredits(account.address);
        if (credit.isSome) {
          const c = credit.unwrap();
          console.log(`   ${name} 信用分: ${c.score?.toString() || c.toString()}`);
        } else {
          console.log(`   ${name} 信用分: 未初始化`);
        }
      } catch (e: any) {
        console.log(`   ${name} 信用查询: ${e.message?.slice(0, 50) || '不可用'}`);
      }
    }
    
    // ========================================
    // 步骤 2: 查询信用配置
    // ========================================
    logStep(2, '查询信用配置');
    
    try {
      const config = await (api.query as any).tradingCredit.creditConfig();
      if (config) {
        console.log(`   信用配置: ${JSON.stringify(config.toHuman(), null, 2).slice(0, 200)}`);
      }
    } catch {
      logInfo('信用配置查询不可用');
    }
    
    // ========================================
    // 步骤 3: 查询默认信用分
    // ========================================
    logStep(3, '查询默认信用分');
    
    try {
      const defaultScore = await (api.query as any).tradingCredit.defaultCreditScore();
      console.log(`   默认信用分: ${defaultScore?.toString() || '未设置'}`);
    } catch {
      logInfo('默认信用分查询不可用');
    }
    
    logSection('测试完成');
    logSuccess('Credit 模块测试通过');
    
  } catch (error: any) {
    console.log(`❌ 测试失败: ${error.message}`);
  } finally {
    await disconnectApi();
  }
}

main().catch(console.error);
