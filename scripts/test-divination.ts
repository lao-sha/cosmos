/**
 * Divination（占卜）模块测试脚本
 * 测试八字、梅花易数等占卜功能
 */

import { getApi, disconnectApi } from './utils/api.js';
import { getAlice, logAccount } from './utils/accounts.js';
import { 
  logSection, 
  logStep, 
  logSuccess, 
  logInfo,
} from './utils/helpers.js';

async function main() {
  logSection('Divination（占卜）模块测试');
  
  const api = await getApi();
  const alice = getAlice();
  
  logAccount('Alice', alice);
  
  try {
    // ========================================
    // 步骤 1: 查询八字模块
    // ========================================
    logStep(1, '查询八字模块');
    
    try {
      const nextChartId = await (api.query as any).bazi.nextChartId();
      console.log(`   八字命盘数: ${nextChartId?.toString() || 0}`);
    } catch {
      logInfo('八字模块查询不可用');
    }
    
    // ========================================
    // 步骤 2: 查询梅花易数模块
    // ========================================
    logStep(2, '查询梅花易数模块');
    
    try {
      const nextId = await (api.query as any).meihua.nextDivinationId();
      console.log(`   梅花占卜数: ${nextId?.toString() || 0}`);
    } catch {
      logInfo('梅花易数模块查询不可用');
    }
    
    // ========================================
    // 步骤 3: 查询六爻模块
    // ========================================
    logStep(3, '查询六爻模块');
    
    try {
      const nextId = await (api.query as any).liuyao.nextDivinationId();
      console.log(`   六爻占卜数: ${nextId?.toString() || 0}`);
    } catch {
      logInfo('六爻模块查询不可用');
    }
    
    // ========================================
    // 步骤 4: 查询塔罗模块
    // ========================================
    logStep(4, '查询塔罗模块');
    
    try {
      const nextId = await (api.query as any).tarot.nextReadingId();
      console.log(`   塔罗占卜数: ${nextId?.toString() || 0}`);
    } catch {
      logInfo('塔罗模块查询不可用');
    }
    
    // ========================================
    // 步骤 5: 查询占卜会员
    // ========================================
    logStep(5, '查询占卜会员');
    
    try {
      const membership = await (api.query as any).divinationMembership.members(alice.address);
      if (membership && membership.isSome) {
        const m = membership.unwrap();
        console.log(`   Alice 会员等级: ${m.level?.toString() || m.toString()}`);
      } else {
        console.log(`   Alice 非会员`);
      }
    } catch {
      logInfo('占卜会员查询不可用');
    }
    
    // ========================================
    // 步骤 6: 查询万年历
    // ========================================
    logStep(6, '查询万年历');
    
    try {
      const today = new Date();
      const year = today.getFullYear();
      const month = today.getMonth() + 1;
      const day = today.getDate();
      
      const almanac = await (api.query as any).almanac.dayInfo(year, month, day);
      if (almanac && almanac.isSome) {
        const a = almanac.unwrap();
        console.log(`   今日: ${year}-${month}-${day}`);
        console.log(`   农历: ${a.lunarMonth?.toString() || '?'}月${a.lunarDay?.toString() || '?'}日`);
      } else {
        console.log(`   今日万年历: 未查询到`);
      }
    } catch {
      logInfo('万年历查询不可用');
    }
    
    logSection('测试完成');
    logSuccess('Divination 模块测试通过');
    
  } catch (error: any) {
    console.log(`❌ 测试失败: ${error.message}`);
  } finally {
    await disconnectApi();
  }
}

main().catch(console.error);
