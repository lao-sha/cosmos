#!/usr/bin/env tsx
/**
 * Stardust 全功能测试入口
 * 运行所有模块的测试套件
 */

import {
  log,
  printFinalReport,
  TestSuiteResult,
  colors,
} from './test-utils.js';

import { runInfrastructureTests } from './test-infrastructure.js';
import { runBalanceTests } from './test-balances.js';
import { runDivinationTests } from './test-divination.js';
import { runChatTests } from './test-chat.js';
import { runTradingTests } from './test-trading.js';
import { runEscrowArbitrationTests } from './test-escrow-arbitration.js';
import { runGovernanceTests } from './test-governance.js';
import { runWorkflowTests } from './test-workflows.js';

// 测试套件配置
interface TestSuiteConfig {
  name: string;
  run: () => Promise<TestSuiteResult>;
  enabled: boolean;
}

const testSuites: TestSuiteConfig[] = [
  {
    name: '基础设施测试',
    run: runInfrastructureTests,
    enabled: true,
  },
  {
    name: '余额和转账测试',
    run: runBalanceTests,
    enabled: true,
  },
  {
    name: '占卜系统测试',
    run: runDivinationTests,
    enabled: true,
  },
  {
    name: '聊天/直播系统测试',
    run: runChatTests,
    enabled: true,
  },
  {
    name: '交易系统测试',
    run: runTradingTests,
    enabled: true,
  },
  {
    name: '托管/仲裁系统测试',
    run: runEscrowArbitrationTests,
    enabled: true,
  },
  {
    name: '治理和系统模块测试',
    run: runGovernanceTests,
    enabled: true,
  },
  {
    name: '完整业务流程测试',
    run: runWorkflowTests,
    enabled: true,
  },
];

/**
 * 打印欢迎信息
 */
function printWelcome(): void {
  console.log(`
${colors.cyan}${colors.bright}
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║              ✨ Stardust 全功能测试套件 ✨                    ║
║                                                               ║
║  测试模块:                                                    ║
║    • 基础设施 (节点连接、Pallet可用性)                        ║
║    • 余额和转账 (Balances)                                    ║
║    • 占卜系统 (八字、六爻、梅花、奇门、紫微、塔罗等)          ║
║    • 聊天/直播 (ChatCore、ChatGroup、Livestream)              ║
║    • 交易系统 (Maker、OTC、Swap、Pricing、Credit)             ║
║    • 托管/仲裁 (Escrow、Arbitration、Evidence、Referral)      ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
${colors.reset}
`);
}

/**
 * 打印使用说明
 */
function printUsage(): void {
  console.log(`
${colors.yellow}使用方法:${colors.reset}
  npx tsx run-all-tests.ts [选项]

${colors.yellow}选项:${colors.reset}
  --all                运行所有测试 (默认)
  --infrastructure     仅运行基础设施测试
  --balances           仅运行余额测试
  --divination         仅运行占卜系统测试
  --chat               仅运行聊天/直播测试
  --trading            仅运行交易系统测试
  --escrow             仅运行托管/仲裁测试
  --help               显示此帮助信息

${colors.yellow}示例:${colors.reset}
  npx tsx run-all-tests.ts                    # 运行所有测试
  npx tsx run-all-tests.ts --infrastructure   # 仅运行基础设施测试
  npx tsx run-all-tests.ts --divination       # 仅运行占卜测试
`);
}

/**
 * 解析命令行参数
 */
function parseArgs(): { suites: string[]; showHelp: boolean } {
  const args = process.argv.slice(2);
  
  if (args.includes('--help') || args.includes('-h')) {
    return { suites: [], showHelp: true };
  }

  const suiteFlags: { [key: string]: string } = {
    '--infrastructure': '基础设施测试',
    '--balances': '余额和转账测试',
    '--divination': '占卜系统测试',
    '--chat': '聊天/直播系统测试',
    '--trading': '交易系统测试',
    '--escrow': '托管/仲裁系统测试',
    '--governance': '治理和系统模块测试',
    '--workflow': '完整业务流程测试',
  };

  const selectedSuites: string[] = [];

  for (const arg of args) {
    if (suiteFlags[arg]) {
      selectedSuites.push(suiteFlags[arg]);
    }
  }

  // 如果没有指定或使用 --all，返回所有
  if (selectedSuites.length === 0 || args.includes('--all')) {
    return { suites: testSuites.map(s => s.name), showHelp: false };
  }

  return { suites: selectedSuites, showHelp: false };
}

/**
 * 主函数
 */
async function main(): Promise<void> {
  const startTime = Date.now();
  const { suites: selectedSuites, showHelp } = parseArgs();

  if (showHelp) {
    printUsage();
    process.exit(0);
  }

  printWelcome();

  console.log(`${colors.cyan}开始时间: ${new Date().toLocaleString()}${colors.reset}`);
  console.log(`${colors.cyan}选中的测试套件: ${selectedSuites.join(', ')}${colors.reset}\n`);

  const results: TestSuiteResult[] = [];
  let hasError = false;

  for (const suite of testSuites) {
    if (!selectedSuites.includes(suite.name) || !suite.enabled) {
      continue;
    }

    try {
      log.section(`正在运行: ${suite.name}`);
      const result = await suite.run();
      results.push(result);

      if (result.totalFailed > 0) {
        hasError = true;
      }
    } catch (error: any) {
      log.error(`测试套件 "${suite.name}" 运行失败: ${error.message}`);
      results.push({
        suiteName: suite.name,
        results: [{
          name: '套件执行',
          passed: false,
          message: error.message,
          duration: 0,
        }],
        totalPassed: 0,
        totalFailed: 1,
        totalDuration: 0,
      });
      hasError = true;
    }
  }

  // 打印最终报告
  printFinalReport(results);

  const totalTime = Date.now() - startTime;
  console.log(`\n${colors.cyan}总耗时: ${(totalTime / 1000).toFixed(2)} 秒${colors.reset}`);
  console.log(`${colors.cyan}结束时间: ${new Date().toLocaleString()}${colors.reset}\n`);

  // 根据测试结果退出
  process.exit(hasError ? 1 : 0);
}

// 运行主函数
main().catch(error => {
  console.error(`${colors.red}致命错误: ${error.message}${colors.reset}`);
  process.exit(1);
});

