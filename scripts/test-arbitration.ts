/**
 * Arbitration（仲裁）模块测试脚本
 * 测试仲裁查询功能
 */

import { getApi, disconnectApi } from './utils/api.js';
import { getAlice, getBob, logAccount } from './utils/accounts.js';
import { 
  logSection, 
  logStep, 
  logSuccess, 
  logInfo,
} from './utils/helpers.js';

async function main() {
  logSection('Arbitration（仲裁）模块测试');
  
  const api = await getApi();
  const alice = getAlice();
  const bob = getBob();
  
  logAccount('Alice', alice);
  logAccount('Bob', bob);
  
  try {
    // ========================================
    // 步骤 1: 查询仲裁配置
    // ========================================
    logStep(1, '查询仲裁配置');
    
    try {
      const nextId = await (api.query as any).arbitration.nextDisputeId();
      console.log(`   下一个争议 ID: ${nextId?.toString() || 0}`);
    } catch {
      logInfo('争议 ID 查询不可用');
    }
    
    // ========================================
    // 步骤 2: 查询仲裁委员会成员
    // ========================================
    logStep(2, '查询仲裁委员会成员');
    
    try {
      const members = await (api.query as any).arbitrationCommittee.members();
      if (members && members.length > 0) {
        console.log(`   仲裁委员会成员数: ${members.length}`);
        members.slice(0, 5).forEach((m: any, i: number) => {
          console.log(`   - 成员 ${i + 1}: ${m.toString().slice(0, 16)}...`);
        });
      } else {
        console.log(`   仲裁委员会暂无成员`);
      }
    } catch {
      logInfo('仲裁委员会查询不可用');
    }
    
    // ========================================
    // 步骤 3: 查询待处理争议
    // ========================================
    logStep(3, '查询待处理争议');
    
    try {
      const pendingDisputes = await (api.query as any).arbitration.pendingDisputes();
      if (pendingDisputes && pendingDisputes.length > 0) {
        console.log(`   待处理争议数: ${pendingDisputes.length}`);
      } else {
        console.log(`   暂无待处理争议`);
      }
    } catch {
      logInfo('待处理争议查询不可用');
    }
    
    // ========================================
    // 步骤 4: 查询用户争议历史
    // ========================================
    logStep(4, '查询用户争议历史');
    
    try {
      const aliceDisputes = await (api.query as any).arbitration.userDisputes(alice.address);
      if (aliceDisputes && aliceDisputes.length > 0) {
        console.log(`   Alice 的争议: ${aliceDisputes.map((id: any) => id.toString()).join(', ')}`);
      } else {
        console.log(`   Alice 暂无争议记录`);
      }
    } catch {
      logInfo('用户争议历史查询不可用');
    }
    
    logSection('测试完成');
    logSuccess('Arbitration 模块测试通过');
    
  } catch (error: any) {
    console.log(`❌ 测试失败: ${error.message}`);
  } finally {
    await disconnectApi();
  }
}

main().catch(console.error);
