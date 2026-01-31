import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { SubmittableExtrinsic } from '@polkadot/api/types';

export interface TxResult {
  success: boolean;
  blockHash?: string;
  txHash?: string;
  events?: any[];
  error?: string;
}

export async function signAndSend(
  api: ApiPromise,
  tx: SubmittableExtrinsic<'promise'>,
  signer: KeyringPair,
  description: string
): Promise<TxResult> {
  console.log(`\n📤 发送交易: ${description}`);
  
  return new Promise((resolve) => {
    tx.signAndSend(signer, ({ status, events, dispatchError }) => {
      if (status.isInBlock) {
        console.log(`📦 已入块: ${status.asInBlock.toHex()}`);
      }
      
      if (status.isFinalized) {
        const blockHash = status.asFinalized.toHex();
        console.log(`✅ 已确认: ${blockHash}`);
        
        if (dispatchError) {
          let errorMessage = 'Unknown error';
          if (dispatchError.isModule) {
            const decoded = api.registry.findMetaError(dispatchError.asModule);
            errorMessage = `${decoded.section}.${decoded.name}: ${decoded.docs.join(' ')}`;
          } else {
            errorMessage = dispatchError.toString();
          }
          console.log(`❌ 交易失败: ${errorMessage}`);
          resolve({
            success: false,
            blockHash,
            error: errorMessage,
          });
        } else {
          const relevantEvents = events
            .filter(({ event }) => 
              !event.section.includes('system') && 
              !event.section.includes('transactionPayment')
            )
            .map(({ event }) => ({
              section: event.section,
              method: event.method,
              data: event.data.toHuman(),
            }));
          
          if (relevantEvents.length > 0) {
            console.log('📋 事件:');
            relevantEvents.forEach(e => {
              console.log(`   - ${e.section}.${e.method}:`, e.data);
            });
          }
          
          resolve({
            success: true,
            blockHash,
            txHash: tx.hash.toHex(),
            events: relevantEvents,
          });
        }
      }
    }).catch((error) => {
      console.log(`❌ 发送失败: ${error.message}`);
      resolve({
        success: false,
        error: error.message,
      });
    });
  });
}

export function formatCos(amount: bigint | string | number): string {
  const value = BigInt(amount);
  // 链上精度：1 COS = 1e12 最小单位（12位小数）
  const cos = Number(value) / 1e12;
  return `${cos.toLocaleString()} COS`;
}

export function formatUsdt(amount: number): string {
  const usdt = amount / 1e6;
  if (usdt < 0.01 && usdt > 0) {
    return `$${usdt.toFixed(6)} USDT`;
  }
  return `$${usdt.toFixed(2)} USDT`;
}

export function toCosWei(cos: number): string {
  // 链上精度：1 COS = 1e12 最小单位（12位小数）
  return (BigInt(Math.floor(cos * 1e12))).toString();
}

export function toUsdtWei(usdt: number): string {
  return Math.floor(usdt * 1e6).toString();
}

export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function logSection(title: string): void {
  console.log('\n' + '='.repeat(60));
  console.log(`  ${title}`);
  console.log('='.repeat(60));
}

export function logStep(step: number, description: string): void {
  console.log(`\n📌 步骤 ${step}: ${description}`);
}

export function logSuccess(message: string): void {
  console.log(`✅ ${message}`);
}

export function logError(message: string): void {
  console.log(`❌ ${message}`);
}

export function logInfo(message: string): void {
  console.log(`ℹ️  ${message}`);
}

export function logQuery(name: string, value: any): void {
  console.log(`📊 ${name}:`, typeof value === 'object' ? JSON.stringify(value, null, 2) : value);
}
