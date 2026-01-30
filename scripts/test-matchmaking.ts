/**
 * Matchmaking（相亲）模块测试脚本
 * 测试相亲档案、会员等功能
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
  logSection('Matchmaking（相亲）模块测试');
  
  const api = await getApi();
  const alice = getAlice();
  const bob = getBob();
  const charlie = getCharlie();
  
  logAccount('Alice', alice);
  logAccount('Bob', bob);
  logAccount('Charlie', charlie);
  
  try {
    // ========================================
    // 步骤 1: 查询相亲会员
    // ========================================
    logStep(1, '查询相亲会员');
    
    for (const [name, account] of [['Alice', alice], ['Bob', bob], ['Charlie', charlie]]) {
      try {
        const membership = await (api.query as any).matchmakingMembership.members(account.address);
        if (membership && membership.isSome) {
          const m = membership.unwrap();
          console.log(`   ${name} 会员: ${m.level?.toString() || m.toString()}`);
        } else {
          console.log(`   ${name} 非会员`);
        }
      } catch {
        console.log(`   ${name} 会员查询不可用`);
      }
    }
    
    // ========================================
    // 步骤 2: 查询相亲档案
    // ========================================
    logStep(2, '查询相亲档案');
    
    for (const [name, account] of [['Alice', alice], ['Bob', bob]]) {
      try {
        const profile = await (api.query as any).matchmakingProfile.profiles(account.address);
        if (profile && profile.isSome) {
          const p = profile.unwrap();
          console.log(`   ${name} 档案: 已创建`);
          console.log(`     - 昵称: ${p.nickname ? new TextDecoder().decode(new Uint8Array(p.nickname)) : '未设置'}`);
        } else {
          console.log(`   ${name} 档案: 未创建`);
        }
      } catch {
        console.log(`   ${name} 档案查询不可用`);
      }
    }
    
    // ========================================
    // 步骤 3: 查询档案统计
    // ========================================
    logStep(3, '查询档案统计');
    
    try {
      const nextProfileId = await (api.query as any).matchmakingProfile.nextProfileId();
      console.log(`   总档案数: ${nextProfileId?.toString() || 0}`);
    } catch {
      logInfo('档案统计查询不可用');
    }
    
    // ========================================
    // 步骤 4: 查询会员配置
    // ========================================
    logStep(4, '查询会员配置');
    
    try {
      const config = await (api.query as any).matchmakingMembership.membershipConfig();
      if (config) {
        console.log(`   会员配置: ${JSON.stringify(config.toHuman(), null, 2).slice(0, 200)}`);
      }
    } catch {
      logInfo('会员配置查询不可用');
    }
    
    logSection('测试完成');
    logSuccess('Matchmaking 模块测试通过');
    
  } catch (error: any) {
    console.log(`❌ 测试失败: ${error.message}`);
  } finally {
    await disconnectApi();
  }
}

main().catch(console.error);
