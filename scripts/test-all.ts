/**
 * 综合测试脚本
 * 运行所有模块测试并汇总结果
 */

import { spawn } from 'child_process';
import { logSection, logSuccess, logError, logInfo } from './utils/helpers.js';

const TESTS = [
  { name: 'Pricing', script: 'test-pricing.ts', description: '价格查询' },
  { name: 'CNY Rate', script: 'test-cny-rate.ts', description: 'CNY/USDT 汇率' },
  { name: 'Maker', script: 'test-maker.ts', description: '做市商功能' },
  { name: 'OTC', script: 'test-otc.ts', description: 'OTC 交易' },
  { name: 'Swap', script: 'test-swap.ts', description: 'Swap 兑换' },
  { name: 'Referral', script: 'test-referral.ts', description: '推荐系统' },
  { name: 'Credit', script: 'test-credit.ts', description: '信用模块' },
  { name: 'Escrow', script: 'test-escrow.ts', description: '托管模块' },
  { name: 'Arbitration', script: 'test-arbitration.ts', description: '仲裁模块' },
  { name: 'Chat', script: 'test-chat.ts', description: '聊天模块' },
];

interface TestResult {
  name: string;
  description: string;
  success: boolean;
  duration: number;
  error?: string;
}

async function runTest(test: typeof TESTS[0]): Promise<TestResult> {
  const startTime = Date.now();
  
  return new Promise((resolve) => {
    const child = spawn('npx', ['tsx', test.script], {
      cwd: process.cwd(),
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    
    let stdout = '';
    let stderr = '';
    
    child.stdout?.on('data', (data) => {
      stdout += data.toString();
    });
    
    child.stderr?.on('data', (data) => {
      stderr += data.toString();
    });
    
    child.on('close', (code) => {
      const duration = Date.now() - startTime;
      const success = code === 0 && !stdout.includes('❌ 测试失败');
      
      resolve({
        name: test.name,
        description: test.description,
        success,
        duration,
        error: success ? undefined : stderr || '测试失败',
      });
    });
    
    child.on('error', (err) => {
      resolve({
        name: test.name,
        description: test.description,
        success: false,
        duration: Date.now() - startTime,
        error: err.message,
      });
    });
    
    // 超时处理 (OTC/Swap 需要更长时间)
    const timeout = ['OTC', 'Swap', 'Maker'].includes(test.name) ? 180000 : 60000;
    setTimeout(() => {
      child.kill();
      resolve({
        name: test.name,
        description: test.description,
        success: false,
        duration: timeout,
        error: '测试超时',
      });
    }, timeout);
  });
}

async function main() {
  logSection('Nexus 链综合测试');
  
  console.log(`\n📋 待测试模块: ${TESTS.length} 个\n`);
  
  const results: TestResult[] = [];
  
  for (const test of TESTS) {
    process.stdout.write(`⏳ 测试 ${test.name} (${test.description})...`);
    
    const result = await runTest(test);
    results.push(result);
    
    if (result.success) {
      console.log(` ✅ ${(result.duration / 1000).toFixed(1)}s`);
    } else {
      console.log(` ❌ 失败`);
    }
  }
  
  // 汇总结果
  logSection('测试结果汇总');
  
  const passed = results.filter(r => r.success).length;
  const failed = results.filter(r => !r.success).length;
  const totalTime = results.reduce((sum, r) => sum + r.duration, 0);
  
  console.log(`\n📊 统计:`);
  console.log(`   - 通过: ${passed}/${TESTS.length}`);
  console.log(`   - 失败: ${failed}/${TESTS.length}`);
  console.log(`   - 总耗时: ${(totalTime / 1000).toFixed(1)}s`);
  
  console.log(`\n📋 详细结果:`);
  console.log('   ' + '-'.repeat(60));
  console.log(`   ${'模块'.padEnd(15)} ${'描述'.padEnd(15)} ${'状态'.padEnd(8)} ${'耗时'}`);
  console.log('   ' + '-'.repeat(60));
  
  for (const result of results) {
    const status = result.success ? '✅ 通过' : '❌ 失败';
    const time = `${(result.duration / 1000).toFixed(1)}s`;
    console.log(`   ${result.name.padEnd(15)} ${result.description.padEnd(15)} ${status.padEnd(8)} ${time}`);
  }
  console.log('   ' + '-'.repeat(60));
  
  if (failed > 0) {
    console.log(`\n❌ 失败的测试:`);
    for (const result of results.filter(r => !r.success)) {
      console.log(`   - ${result.name}: ${result.error?.slice(0, 80) || '未知错误'}`);
    }
  }
  
  logSection(passed === TESTS.length ? '全部测试通过 🎉' : '部分测试失败');
  
  process.exit(failed > 0 ? 1 : 0);
}

main().catch(console.error);
