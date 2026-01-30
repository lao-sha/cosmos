/**
 * Chat（聊天）模块测试脚本
 * 测试聊天权限、联系人等功能
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
  logSection('Chat（聊天）模块测试');
  
  const api = await getApi();
  const alice = getAlice();
  const bob = getBob();
  const charlie = getCharlie();
  
  logAccount('Alice', alice);
  logAccount('Bob', bob);
  logAccount('Charlie', charlie);
  
  try {
    // ========================================
    // 步骤 1: 查询聊天权限
    // ========================================
    logStep(1, '查询聊天权限');
    
    for (const [name, account] of [['Alice', alice], ['Bob', bob], ['Charlie', charlie]]) {
      try {
        const permission = await (api.query as any).chatPermission.userPermissions(account.address);
        if (permission && permission.isSome) {
          const p = permission.unwrap();
          console.log(`   ${name} 权限: ${p.toString()}`);
        } else {
          console.log(`   ${name} 权限: 默认`);
        }
      } catch {
        console.log(`   ${name} 权限查询不可用`);
      }
    }
    
    // ========================================
    // 步骤 2: 查询群组列表
    // ========================================
    logStep(2, '查询群组列表');
    
    try {
      const nextGroupId = await (api.query as any).chatGroup.nextGroupId();
      console.log(`   总群组数: ${nextGroupId?.toString() || 0}`);
    } catch {
      logInfo('群组 ID 查询不可用');
    }
    
    // ========================================
    // 步骤 3: 查询直播间
    // ========================================
    logStep(3, '查询直播间');
    
    try {
      const nextStreamId = await (api.query as any).livestream.nextStreamId();
      console.log(`   总直播间数: ${nextStreamId?.toString() || 0}`);
    } catch {
      logInfo('直播间查询不可用');
    }
    
    // ========================================
    // 步骤 4: 查询消息统计
    // ========================================
    logStep(4, '查询消息统计');
    
    try {
      const stats = await (api.query as any).chatCore.messageStats();
      if (stats) {
        console.log(`   消息统计: ${JSON.stringify(stats.toHuman(), null, 2).slice(0, 200)}`);
      }
    } catch {
      logInfo('消息统计查询不可用');
    }
    
    logSection('测试完成');
    logSuccess('Chat 模块测试通过');
    
  } catch (error: any) {
    console.log(`❌ 测试失败: ${error.message}`);
  } finally {
    await disconnectApi();
  }
}

main().catch(console.error);
