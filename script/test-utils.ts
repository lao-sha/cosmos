/**
 * Stardust 测试工具类
 * 提供通用的测试辅助函数
 */

import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { cryptoWaitReady } from '@polkadot/util-crypto';

// 颜色输出
export const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
};

// 默认配置
export const DEFAULT_WS_ENDPOINT = 'ws://localhost:9944';
export const UNIT = 1_000_000_000_000n; // 1 DUST = 10^12 units

// 测试结果接口
export interface TestResult {
  name: string;
  passed: boolean;
  message: string;
  duration: number;
}

// 测试套件结果
export interface TestSuiteResult {
  suiteName: string;
  results: TestResult[];
  totalPassed: number;
  totalFailed: number;
  totalDuration: number;
}

/**
 * 初始化 API 连接
 */
export async function initApi(endpoint: string = DEFAULT_WS_ENDPOINT): Promise<ApiPromise> {
  await cryptoWaitReady();
  const provider = new WsProvider(endpoint);
  const api = await ApiPromise.create({ provider });
  return api;
}

/**
 * 获取测试账户
 */
export function getTestAccounts(keyring: Keyring): {
  alice: KeyringPair;
  bob: KeyringPair;
  charlie: KeyringPair;
  dave: KeyringPair;
  eve: KeyringPair;
  ferdie: KeyringPair;
} {
  return {
    alice: keyring.addFromUri('//Alice'),
    bob: keyring.addFromUri('//Bob'),
    charlie: keyring.addFromUri('//Charlie'),
    dave: keyring.addFromUri('//Dave'),
    eve: keyring.addFromUri('//Eve'),
    ferdie: keyring.addFromUri('//Ferdie'),
  };
}

/**
 * 日志输出函数
 */
export const log = {
  info: (msg: string) => console.log(`${colors.blue}ℹ${colors.reset} ${msg}`),
  success: (msg: string) => console.log(`${colors.green}✓${colors.reset} ${msg}`),
  error: (msg: string) => console.log(`${colors.red}✗${colors.reset} ${msg}`),
  warn: (msg: string) => console.log(`${colors.yellow}⚠${colors.reset} ${msg}`),
  header: (msg: string) => console.log(`\n${colors.cyan}${colors.bright}═══════════════════════════════════════════════════════════════${colors.reset}`),
  section: (msg: string) => console.log(`\n${colors.magenta}${colors.bright}▶ ${msg}${colors.reset}`),
  subSection: (msg: string) => console.log(`  ${colors.yellow}→${colors.reset} ${msg}`),
};

/**
 * 等待交易完成
 */
export async function signAndSendTx(
  api: ApiPromise,
  tx: any,
  signer: KeyringPair,
  expectedEvent?: { section: string; method: string }
): Promise<{ blockHash: string; events: any[]; success: boolean; error?: string }> {
  return new Promise((resolve, reject) => {
    let unsub: () => void;

    tx.signAndSend(signer, { nonce: -1 }, ({ status, events, dispatchError }: any) => {
      if (status.isInBlock || status.isFinalized) {
        const blockHash = status.isInBlock ? status.asInBlock.toHex() : status.asFinalized.toHex();

        if (dispatchError) {
          let errorMessage = dispatchError.toString();
          if (dispatchError.isModule) {
            try {
              const decoded = api.registry.findMetaError(dispatchError.asModule);
              errorMessage = `${decoded.section}.${decoded.name}: ${decoded.docs.join(' ')}`;
            } catch (e) {
              // 使用原始错误信息
            }
          }
          if (unsub) unsub();
          resolve({ blockHash, events: events.toArray(), success: false, error: errorMessage });
          return;
        }

        // 检查是否有预期的事件
        let foundExpectedEvent = !expectedEvent;
        for (const { event } of events) {
          if (expectedEvent && event.section === expectedEvent.section && event.method === expectedEvent.method) {
            foundExpectedEvent = true;
          }
        }

        if (unsub) unsub();
        resolve({ blockHash, events: events.toArray(), success: foundExpectedEvent });
      }
    }).then((unsubFn: () => void) => {
      unsub = unsubFn;
    }).catch(reject);
  });
}

/**
 * 运行单个测试
 */
export async function runTest(
  name: string,
  testFn: () => Promise<void>
): Promise<TestResult> {
  const start = Date.now();
  try {
    await testFn();
    const duration = Date.now() - start;
    log.success(`${name} (${duration}ms)`);
    return { name, passed: true, message: 'OK', duration };
  } catch (error: any) {
    const duration = Date.now() - start;
    log.error(`${name}: ${error.message}`);
    return { name, passed: false, message: error.message, duration };
  }
}

/**
 * 运行测试套件
 */
export async function runTestSuite(
  suiteName: string,
  tests: Array<{ name: string; fn: () => Promise<void> }>
): Promise<TestSuiteResult> {
  log.header(suiteName);
  console.log(`${colors.cyan}${colors.bright}  ${suiteName}${colors.reset}`);
  log.header('');

  const results: TestResult[] = [];
  let totalPassed = 0;
  let totalFailed = 0;

  for (const test of tests) {
    const result = await runTest(test.name, test.fn);
    results.push(result);
    if (result.passed) {
      totalPassed++;
    } else {
      totalFailed++;
    }
  }

  const totalDuration = results.reduce((sum, r) => sum + r.duration, 0);

  console.log(`\n${colors.bright}测试结果:${colors.reset}`);
  console.log(`  ${colors.green}通过: ${totalPassed}${colors.reset}`);
  console.log(`  ${colors.red}失败: ${totalFailed}${colors.reset}`);
  console.log(`  总耗时: ${totalDuration}ms`);

  return { suiteName, results, totalPassed, totalFailed, totalDuration };
}

/**
 * 格式化余额显示
 */
export function formatBalance(balance: bigint): string {
  const dust = balance / UNIT;
  const remainder = balance % UNIT;
  if (remainder === 0n) {
    return `${dust} DUST`;
  }
  return `${dust}.${remainder.toString().padStart(12, '0').replace(/0+$/, '')} DUST`;
}

/**
 * 延迟函数
 */
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * 断言函数
 */
export function assert(condition: boolean, message: string): void {
  if (!condition) {
    throw new Error(message);
  }
}

/**
 * 断言相等
 */
export function assertEqual<T>(actual: T, expected: T, message?: string): void {
  if (actual !== expected) {
    throw new Error(message || `Expected ${expected}, but got ${actual}`);
  }
}

/**
 * 断言不相等
 */
export function assertNotEqual<T>(actual: T, notExpected: T, message?: string): void {
  if (actual === notExpected) {
    throw new Error(message || `Expected value to not be ${notExpected}`);
  }
}

/**
 * 断言大于
 */
export function assertGreaterThan(actual: bigint | number, expected: bigint | number, message?: string): void {
  if (actual <= expected) {
    throw new Error(message || `Expected ${actual} to be greater than ${expected}`);
  }
}

/**
 * 生成随机字符串
 */
export function randomString(length: number = 8): string {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  let result = '';
  for (let i = 0; i < length; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}

/**
 * 生成随机数字
 */
export function randomNumber(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

/**
 * 打印最终测试报告
 */
export function printFinalReport(suites: TestSuiteResult[]): void {
  console.log('\n');
  log.header('最终测试报告');
  console.log(`${colors.cyan}${colors.bright}  最终测试报告${colors.reset}`);
  log.header('');

  let totalPassed = 0;
  let totalFailed = 0;
  let totalDuration = 0;

  for (const suite of suites) {
    const status = suite.totalFailed === 0 ? colors.green + '✓' : colors.red + '✗';
    console.log(`${status}${colors.reset} ${suite.suiteName}: ${suite.totalPassed}/${suite.totalPassed + suite.totalFailed} 通过`);
    totalPassed += suite.totalPassed;
    totalFailed += suite.totalFailed;
    totalDuration += suite.totalDuration;
  }

  console.log('\n' + '─'.repeat(60));
  console.log(`${colors.bright}总计:${colors.reset}`);
  console.log(`  ${colors.green}通过: ${totalPassed}${colors.reset}`);
  console.log(`  ${colors.red}失败: ${totalFailed}${colors.reset}`);
  console.log(`  总耗时: ${totalDuration}ms`);

  if (totalFailed === 0) {
    console.log(`\n${colors.green}${colors.bright}🎉 所有测试通过！${colors.reset}`);
  } else {
    console.log(`\n${colors.red}${colors.bright}❌ 有 ${totalFailed} 个测试失败${colors.reset}`);
  }
}

