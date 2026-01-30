/**
 * 相亲档案创建脚本
 * 创建用户婚恋资料
 * 
 * 使用方法:
 *   npx tsx create-profile.ts <account> [nickname] [gender]
 *   
 * 示例:
 *   npx tsx create-profile.ts alice 小红 female
 *   npx tsx create-profile.ts bob 小明 male
 */

import { getApi, disconnectApi } from './utils/api.js';
import { getAlice, getBob, getCharlie, getDave, getEve, logAccount } from './utils/accounts.js';
import { 
  signAndSend,
  logSection, 
  logStep, 
  logSuccess, 
  logError,
  logInfo,
  formatCos,
} from './utils/helpers.js';

const ACCOUNTS: Record<string, () => any> = {
  alice: getAlice,
  bob: getBob,
  charlie: getCharlie,
  dave: getDave,
  eve: getEve,
};

// 性别映射
const GENDER_MAP: Record<string, string> = {
  male: 'Male',
  female: 'Female',
  m: 'Male',
  f: 'Female',
  男: 'Male',
  女: 'Female',
};

async function main() {
  const args = process.argv.slice(2);
  
  if (args.length < 1) {
    console.log('使用方法: npx tsx create-profile.ts <account> [nickname] [gender]');
    console.log('示例: npx tsx create-profile.ts alice 小红 female');
    console.log('\n可用账户: alice, bob, charlie, dave, eve');
    console.log('性别: male/m/男, female/f/女');
    return;
  }
  
  const accountName = args[0].toLowerCase();
  const nickname = args[1] || `User_${accountName}`;
  const genderInput = args[2]?.toLowerCase() || 'male';
  const gender = GENDER_MAP[genderInput] || 'Male';
  
  if (!ACCOUNTS[accountName]) {
    logError(`未知账户: ${accountName}`);
    console.log('可用账户: alice, bob, charlie, dave, eve');
    return;
  }
  
  logSection('相亲档案创建');
  
  const api = await getApi();
  const account = ACCOUNTS[accountName]();
  
  logAccount(`${accountName} (创建者)`, account);
  console.log(`   昵称: ${nickname}`);
  console.log(`   性别: ${gender}`);
  
  try {
    // ========================================
    // 步骤 1: 检查是否已有档案
    // ========================================
    logStep(1, '检查现有档案');
    
    const existingProfile = await (api.query as any).matchmakingProfile.profiles(account.address);
    if (existingProfile && existingProfile.isSome) {
      const profile = existingProfile.unwrap();
      const existingNickname = profile.nickname && profile.nickname.length > 0
        ? new TextDecoder().decode(new Uint8Array(profile.nickname))
        : '未设置';
      console.log(`   已有档案: ${existingNickname}`);
      logInfo('档案已存在，跳过创建');
      
      // 显示档案详情
      logStep(2, '档案详情');
      console.log(`   昵称: ${existingNickname}`);
      console.log(`   性别: ${profile.gender?.toString() || '未设置'}`);
      console.log(`   年龄: ${profile.age?.toString() || '未设置'}`);
      console.log(`   状态: ${profile.status?.toString() || '未知'}`);
      console.log(`   完整度: ${profile.completeness?.toString() || 0}%`);
      console.log(`   隐私模式: ${profile.privacyMode?.toString() || '未设置'}`);
      
      return;
    }
    
    console.log('   暂无档案，准备创建');
    
    // ========================================
    // 步骤 2: 查询保证金要求
    // ========================================
    logStep(2, '查询保证金要求');
    
    let depositAmount = '0';
    try {
      // 尝试查询保证金金额
      const depositConfig = await (api.query as any).matchmakingProfile.depositAmount();
      if (depositConfig) {
        depositAmount = depositConfig.toString();
        console.log(`   保证金: ${formatCos(depositAmount)}`);
      }
    } catch {
      console.log('   保证金: 50 USDT 等值 COS（默认）');
    }
    
    // 查询账户余额
    const balance = await api.query.system.account(account.address);
    console.log(`   账户余额: ${formatCos(balance.data.free.toString())}`);
    
    // ========================================
    // 步骤 3: 创建档案
    // ========================================
    logStep(3, '创建档案');
    
    // 构建创建参数
    const createTx = (api.tx as any).matchmakingProfile.createProfile(
      nickname,           // 昵称
      gender,             // 性别
      null,               // 年龄（可选）
      null,               // 出生日期（可选）
      null,               // 当前位置（可选）
      null                // 简介（可选）
    );
    
    const result = await signAndSend(api, createTx, account, `${accountName} 创建档案`);
    
    if (!result.success) {
      logError(`创建失败: ${result.error}`);
      return;
    }
    
    logSuccess('档案创建成功！');
    
    // ========================================
    // 步骤 4: 查询新档案
    // ========================================
    logStep(4, '查询新档案');
    
    const newProfile = await (api.query as any).matchmakingProfile.profiles(account.address);
    if (newProfile && newProfile.isSome) {
      const profile = newProfile.unwrap();
      const profileNickname = profile.nickname && profile.nickname.length > 0
        ? new TextDecoder().decode(new Uint8Array(profile.nickname))
        : '未设置';
      console.log(`   昵称: ${profileNickname}`);
      console.log(`   性别: ${profile.gender?.toString() || '未设置'}`);
      console.log(`   状态: ${profile.status?.toString() || '未知'}`);
      console.log(`   完整度: ${profile.completeness?.toString() || 0}%`);
    }
    
    // ========================================
    // 步骤 5: 查询档案统计
    // ========================================
    logStep(5, '查询档案统计');
    
    try {
      const profileCount = await (api.query as any).matchmakingProfile.profileCount();
      console.log(`   总档案数: ${profileCount?.toString() || 0}`);
    } catch {
      logInfo('档案统计查询不可用');
    }
    
    logSection('完成');
    
    console.log('\n📊 档案摘要:');
    console.log(`   - 账户: ${accountName}`);
    console.log(`   - 昵称: ${nickname}`);
    console.log(`   - 性别: ${gender}`);
    
  } catch (error: any) {
    logError(`执行失败: ${error.message}`);
    console.error(error);
  } finally {
    await disconnectApi();
  }
}

main().catch(console.error);
