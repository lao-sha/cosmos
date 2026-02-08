/**
 * CNY/USDT 汇率查询脚本
 * 查询链上存储的汇率数据和 OCW 更新状态
 */

import { getApi, disconnectApi, getCurrentBlock } from './utils/api.js';
import { logSection, logStep, logSuccess, logError, logInfo } from './utils/helpers.js';

async function main() {
  logSection('CNY/USDT 汇率查询');
  
  const api = await getApi();
  
  try {
    // ========================================
    // 步骤 1: 查询当前汇率数据
    // ========================================
    logStep(1, '查询当前汇率数据');
    
    const cnyUsdtRate = await (api.query as any).tradingPricing.cnyUsdtRate();
    const cnyRate = cnyUsdtRate.cnyRate.toNumber();
    const updatedAt = cnyUsdtRate.updatedAt.toNumber();
    
    if (cnyRate > 0) {
      const rateFormatted = (cnyRate / 1e6).toFixed(4);
      console.log(`   CNY/USDT 汇率: ¥${rateFormatted}`);
      console.log(`   原始值: ${cnyRate} (精度 10^6)`);
      console.log(`   更新时间戳: ${updatedAt}`);
      
      if (updatedAt > 0) {
        const updateDate = new Date(updatedAt * 1000);
        console.log(`   更新时间: ${updateDate.toLocaleString()}`);
        
        // 计算距今多久
        const now = Date.now();
        const diffMs = now - updatedAt * 1000;
        const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
        const diffMinutes = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60));
        console.log(`   距今: ${diffHours} 小时 ${diffMinutes} 分钟`);
      }
    } else {
      console.log(`   CNY/USDT 汇率: 未设置`);
      console.log(`   将使用默认值: ¥7.2000`);
    }
    
    // ========================================
    // 步骤 2: 查询上次更新区块
    // ========================================
    logStep(2, '查询上次更新区块');
    
    const lastRateUpdateBlock = await (api.query as any).tradingPricing.lastRateUpdateBlock();
    const lastUpdateBlock = lastRateUpdateBlock.toNumber();
    const currentBlock = await getCurrentBlock(api);
    
    console.log(`   上次更新区块: ${lastUpdateBlock}`);
    console.log(`   当前区块: ${currentBlock}`);
    console.log(`   区块差: ${currentBlock - lastUpdateBlock}`);
    
    // ========================================
    // 步骤 3: 查询更新间隔配置
    // ========================================
    logStep(3, '查询更新间隔配置');
    
    // 尝试获取常量配置
    try {
      const consts = (api.consts as any).tradingPricing;
      if (consts && consts.exchangeRateUpdateInterval) {
        const interval = consts.exchangeRateUpdateInterval.toNumber();
        console.log(`   更新间隔: ${interval} 区块`);
        console.log(`   约等于: ${(interval * 6 / 3600).toFixed(1)} 小时`);
        
        // 计算下次更新
        const nextUpdateBlock = lastUpdateBlock + interval;
        const blocksUntilUpdate = nextUpdateBlock - currentBlock;
        
        if (blocksUntilUpdate > 0) {
          console.log(`   下次更新区块: ${nextUpdateBlock}`);
          console.log(`   距下次更新: ${blocksUntilUpdate} 区块 (约 ${(blocksUntilUpdate * 6 / 60).toFixed(0)} 分钟)`);
        } else {
          console.log(`   ⚠️ 已超过更新间隔，OCW 应该会在下个区块更新`);
        }
      } else {
        console.log(`   更新间隔: 无法获取（使用默认 14400 区块 ≈ 24小时）`);
      }
    } catch (e) {
      console.log(`   更新间隔: 无法获取配置`);
    }
    
    // ========================================
    // 步骤 4: 汇率换算示例
    // ========================================
    logStep(4, '汇率换算示例');
    
    const rate = cnyRate > 0 ? cnyRate : 7_200_000; // 默认 7.2
    const rateValue = rate / 1e6;
    
    // USDT → CNY
    const usdtAmounts = [1, 10, 100, 1000];
    console.log('\n   USDT → CNY:');
    usdtAmounts.forEach(usdt => {
      const cny = usdt * rateValue;
      console.log(`   ${usdt} USDT = ¥${cny.toFixed(2)}`);
    });
    
    // CNY → USDT
    const cnyAmounts = [10, 100, 1000, 10000];
    console.log('\n   CNY → USDT:');
    cnyAmounts.forEach(cny => {
      const usdt = cny / rateValue;
      console.log(`   ¥${cny} = ${usdt.toFixed(2)} USDT`);
    });
    
    // ========================================
    // 步骤 5: NXS 价格换算
    // ========================================
    logStep(5, 'NXS 价格换算（结合当前 NXS 价格）');
    
    // 获取当前 NXS 价格
    const defaultPrice = await (api.query as any).tradingPricing.defaultPrice();
    const coldStartExited = await (api.query as any).tradingPricing.coldStartExited();
    
    let cosPrice: number;
    if (!coldStartExited.isTrue) {
      cosPrice = defaultPrice.toNumber();
    } else {
      const otcAgg = await (api.query as any).tradingPricing.otcPriceAggregate();
      const bridgeAgg = await (api.query as any).tradingPricing.bridgePriceAggregate();
      const totalNxs = BigInt(otcAgg.totalNxs.toString()) + BigInt(bridgeAgg.totalNxs.toString());
      const totalUsdt = BigInt(otcAgg.totalUsdt.toString()) + BigInt(bridgeAgg.totalUsdt.toString());
      
      if (totalNxs > 0n) {
        cosPrice = Number((totalUsdt * BigInt(1e12)) / totalNxs);
      } else {
        cosPrice = defaultPrice.toNumber();
      }
    }
    
    const cosPriceUsdt = cosPrice / 1e6;
    const cosPriceCny = cosPriceUsdt * rateValue;
    
    console.log(`\n   当前 NXS 价格:`);
    console.log(`   - USDT: $${cosPriceUsdt.toFixed(6)}`);
    console.log(`   - CNY:  ¥${cosPriceCny.toFixed(6)}`);
    
    // NXS 数量换算
    const nxsAmounts = [1000, 10000, 100000, 1000000];
    console.log('\n   NXS 价值换算:');
    nxsAmounts.forEach(cos => {
      const valueUsdt = cos * cosPriceUsdt;
      const valueCny = cos * cosPriceCny;
      console.log(`   ${cos.toLocaleString()} NXS = $${valueUsdt.toFixed(4)} USDT = ¥${valueCny.toFixed(4)}`);
    });
    
    // ========================================
    // 总结
    // ========================================
    logSection('查询完成');
    logSuccess('CNY/USDT 汇率查询完成');
    
    console.log('\n📊 汇率摘要:');
    console.log(`   - 当前汇率: ¥${rateValue.toFixed(4)} / USDT`);
    console.log(`   - 数据来源: ${cnyRate > 0 ? 'OCW 更新' : '默认值'}`);
    console.log(`   - NXS/USDT: $${cosPriceUsdt.toFixed(6)}`);
    console.log(`   - NXS/CNY:  ¥${cosPriceCny.toFixed(6)}`);
    
  } catch (error: any) {
    logError(`查询失败: ${error.message}`);
    console.error(error);
  } finally {
    await disconnectApi();
  }
}

main().catch(console.error);
