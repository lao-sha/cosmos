/**
 * 运行所有测试脚本
 */

import { spawn } from 'child_process';
import { logSection, logSuccess, logError, logInfo } from './utils/helpers.js';

const tests = [
  { name: 'Pricing 模块', script: 'test-pricing.ts' },
  { name: 'Maker 模块', script: 'test-maker.ts' },
  { name: 'OTC 模块', script: 'test-otc.ts' },
  { name: 'Swap 模块', script: 'test-swap.ts' },
  { name: 'Referral 模块', script: 'test-referral.ts' },
];

async function runTest(name: string, script: string): Promise<boolean> {
  return new Promise((resolve) => {
    console.log(`\n🚀 运行 ${name} 测试...`);
    
    const child = spawn('npx', ['tsx', script], {
      stdio: 'inherit',
      shell: true,
    });
    
    child.on('close', (code) => {
      if (code === 0) {
        logSuccess(`${name} 测试通过`);
        resolve(true);
      } else {
        logError(`${name} 测试失败 (退出码: ${code})`);
        resolve(false);
      }
    });
    
    child.on('error', (err) => {
      logError(`${name} 测试出错: ${err.message}`);
      resolve(false);
    });
  });
}

async function main() {
  logSection('Nexus 链上功能测试');
  
  console.log('📋 测试列表:');
  tests.forEach((t, i) => {
    console.log(`   ${i + 1}. ${t.name} (${t.script})`);
  });
  
  const results: { name: string; passed: boolean }[] = [];
  
  for (const test of tests) {
    const passed = await runTest(test.name, test.script);
    results.push({ name: test.name, passed });
    
    // 测试间隔
    await new Promise(resolve => setTimeout(resolve, 2000));
  }
  
  // 输出总结
  logSection('测试总结');
  
  const passed = results.filter(r => r.passed).length;
  const failed = results.filter(r => !r.passed).length;
  
  console.log('\n📊 测试结果:');
  results.forEach(r => {
    const icon = r.passed ? '✅' : '❌';
    console.log(`   ${icon} ${r.name}`);
  });
  
  console.log(`\n📈 统计: ${passed} 通过, ${failed} 失败`);
  
  if (failed === 0) {
    logSuccess('所有测试通过！');
    process.exit(0);
  } else {
    logError(`${failed} 个测试失败`);
    process.exit(1);
  }
}

main().catch(console.error);
