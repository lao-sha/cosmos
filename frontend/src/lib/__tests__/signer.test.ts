/**
 * 移动端签名测试脚本
 * 用于验证签名功能是否正常工作
 */

import { initializeApi, getApi } from '@/lib/api';
import { initializeSigner, unlockWallet, signAndSendTransaction } from '@/lib/signer.native';
import { TradingService } from '@/services/trading.service';

/**
 * 测试签名器初始化
 */
export async function testSignerInitialization() {
  console.log('\n=== 测试 1: 签名器初始化 ===');

  try {
    await initializeSigner();
    console.log('✅ 签名器初始化成功');
    return true;
  } catch (error) {
    console.error('❌ 签名器初始化失败:', error);
    return false;
  }
}

/**
 * 测试钱包解锁
 */
export async function testWalletUnlock(password: string) {
  console.log('\n=== 测试 2: 钱包解锁 ===');

  try {
    const pair = await unlockWallet(password);
    console.log('✅ 钱包解锁成功');
    console.log('   地址:', pair.address);
    console.log('   公钥:', pair.publicKey.toString());
    return true;
  } catch (error) {
    console.error('❌ 钱包解锁失败:', error);
    return false;
  }
}

/**
 * 测试 API 连接
 */
export async function testApiConnection() {
  console.log('\n=== 测试 3: API 连接 ===');

  try {
    const api = await initializeApi();
    const chain = await api.rpc.system.chain();
    const version = await api.rpc.system.version();

    console.log('✅ API 连接成功');
    console.log('   链名称:', chain.toString());
    console.log('   节点版本:', version.toString());
    return true;
  } catch (error) {
    console.error('❌ API 连接失败:', error);
    return false;
  }
}

/**
 * 测试查询做市商
 */
export async function testQueryMakers() {
  console.log('\n=== 测试 4: 查询做市商 ===');

  try {
    const service = new TradingService();
    const makers = await service.getMakers();

    console.log('✅ 查询做市商成功');
    console.log('   做市商数量:', makers.length);

    if (makers.length > 0) {
      console.log('   第一个做市商:');
      console.log('     ID:', makers[0].id);
      console.log('     姓名:', makers[0].maskedFullName);
      console.log('     溢价:', makers[0].sellPremiumBps / 100, '%');
    }

    return true;
  } catch (error) {
    console.error('❌ 查询做市商失败:', error);
    return false;
  }
}

/**
 * 测试创建首购订单
 */
export async function testCreateFirstPurchase(
  password: string,
  makerId: number = 1
) {
  console.log('\n=== 测试 5: 创建首购订单 ===');

  try {
    // 1. 初始化
    await initializeApi();
    await initializeSigner();

    // 2. 解锁钱包
    const pair = await unlockWallet(password);
    console.log('✅ 钱包已解锁:', pair.address);

    // 3. 生成支付承诺
    const paymentCommit = TradingService.generatePaymentCommit(
      '张三',
      '110101199001011234',
      '13812345678'
    );
    const contactCommit = TradingService.generateContactCommit(
      'wechat_test',
      '13812345678'
    );

    console.log('✅ 支付承诺已生成');
    console.log('   Payment commit:', paymentCommit.slice(0, 20) + '...');
    console.log('   Contact commit:', contactCommit.slice(0, 20) + '...');

    // 4. 创建订单
    const service = new TradingService();
    const orderId = await service.createFirstPurchase(
      pair.address,
      makerId,
      paymentCommit,
      contactCommit,
      (status) => {
        console.log('   交易状态:', status);
      }
    );

    console.log('✅ 首购订单创建成功');
    console.log('   订单 ID:', orderId);

    return orderId;
  } catch (error) {
    console.error('❌ 创建首购订单失败:', error);
    if (error instanceof Error) {
      console.error('   错误信息:', error.message);
    }
    return null;
  }
}

/**
 * 测试查询订单
 */
export async function testQueryOrder(orderId: number) {
  console.log('\n=== 测试 6: 查询订单 ===');

  try {
    const service = new TradingService();
    const order = await service.getOrder(orderId);

    if (!order) {
      console.error('❌ 订单不存在');
      return false;
    }

    console.log('✅ 查询订单成功');
    console.log('   订单 ID:', order.id);
    console.log('   做市商 ID:', order.makerId);
    console.log('   买家:', order.taker);
    console.log('   金额:', TradingService.formatUsdAmount(order.amount), 'USDT');
    console.log('   数量:', TradingService.formatDustAmount(order.qty), 'DUST');
    console.log('   状态:', order.state);
    console.log('   是否首购:', order.isFirstPurchase);

    return true;
  } catch (error) {
    console.error('❌ 查询订单失败:', error);
    return false;
  }
}

/**
 * 运行所有测试
 */
export async function runAllTests(password: string) {
  console.log('\n🧪 开始移动端签名测试...\n');

  const results = {
    signerInit: false,
    walletUnlock: false,
    apiConnection: false,
    queryMakers: false,
    createOrder: null as number | null,
    queryOrder: false,
  };

  // 测试 1: 签名器初始化
  results.signerInit = await testSignerInitialization();
  if (!results.signerInit) {
    console.log('\n❌ 测试失败：签名器初始化失败');
    return results;
  }

  // 测试 2: 钱包解锁
  results.walletUnlock = await testWalletUnlock(password);
  if (!results.walletUnlock) {
    console.log('\n❌ 测试失败：钱包解锁失败');
    return results;
  }

  // 测试 3: API 连接
  results.apiConnection = await testApiConnection();
  if (!results.apiConnection) {
    console.log('\n❌ 测试失败：API 连接失败');
    return results;
  }

  // 测试 4: 查询做市商
  results.queryMakers = await testQueryMakers();
  if (!results.queryMakers) {
    console.log('\n❌ 测试失败：查询做市商失败');
    return results;
  }

  // 测试 5: 创建首购订单
  results.createOrder = await testCreateFirstPurchase(password);
  if (!results.createOrder) {
    console.log('\n❌ 测试失败：创建订单失败');
    return results;
  }

  // 测试 6: 查询订单
  results.queryOrder = await testQueryOrder(results.createOrder);

  // 输出测试结果
  console.log('\n=== 测试结果汇总 ===');
  console.log('签名器初始化:', results.signerInit ? '✅' : '❌');
  console.log('钱包解锁:', results.walletUnlock ? '✅' : '❌');
  console.log('API 连接:', results.apiConnection ? '✅' : '❌');
  console.log('查询做市商:', results.queryMakers ? '✅' : '❌');
  console.log('创建订单:', results.createOrder ? `✅ (ID: ${results.createOrder})` : '❌');
  console.log('查询订单:', results.queryOrder ? '✅' : '❌');

  const allPassed = results.signerInit &&
                    results.walletUnlock &&
                    results.apiConnection &&
                    results.queryMakers &&
                    results.createOrder !== null &&
                    results.queryOrder;

  if (allPassed) {
    console.log('\n🎉 所有测试通过！');
  } else {
    console.log('\n❌ 部分测试失败');
  }

  return results;
}

/**
 * 快速测试（仅测试签名功能）
 */
export async function quickTest(password: string) {
  console.log('\n🚀 快速测试开始...\n');

  try {
    // 初始化
    await initializeApi();
    await initializeSigner();
    console.log('✅ 初始化完成');

    // 解锁
    const pair = await unlockWallet(password);
    console.log('✅ 钱包已解锁:', pair.address);

    // 创建简单交易（转账）
    const api = getApi();
    const tx = api.tx.balances.transfer(
      '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
      1000000000000 // 1 DUST
    );

    console.log('✅ 交易已创建');
    console.log('   开始签名...');

    const result = await signAndSendTransaction(
      api,
      tx,
      (status) => console.log('   状态:', status)
    );

    console.log('✅ 交易成功');
    console.log('   区块哈希:', result.blockHash);
    console.log('   事件数量:', result.events.length);

    console.log('\n🎉 快速测试通过！');
    return true;
  } catch (error) {
    console.error('\n❌ 快速测试失败:', error);
    return false;
  }
}
