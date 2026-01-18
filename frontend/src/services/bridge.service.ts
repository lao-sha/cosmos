/**
 * Bridge 服务 - 处理跨链桥接交易
 * 支持官方桥接和做市商桥接
 */

import { ApiPromise } from '@polkadot/api';
import { getApi } from '@/lib/api';
import { signAndSend, getCurrentSignerAddress } from '@/lib/signer';
import { u8aToHex, hexToU8a } from '@polkadot/util';

/**
 * 签名状态回调
 */
export type StatusCallback = (status: string) => void;

/**
 * 桥接类型
 */
export enum BridgeType {
  Official = 'Official',  // 官方桥接
  Maker = 'Maker',        // 做市商桥接
}

/**
 * 兑换记录
 */
export interface SwapRecord {
  id: number;
  account: string;
  bridgeType: BridgeType;
  dustAmount: bigint;
  usdtAmount: bigint;
  tronAddress: string;
  makerId?: number;
  status: 'Pending' | 'Completed' | 'Failed';
  timestamp: number;
  blockNumber: number;
}

/**
 * Bridge 服务类
 */
export class BridgeService {
  /**
   * 获取 API 实例
   */
  private getApi(): ApiPromise {
    try {
      return getApi();
    } catch (error) {
      throw new Error('API not initialized. Please initialize API first.');
    }
  }

  /**
   * 官方桥接：DUST → USDT
   * @param dustAmount DUST 数量（最小单位）
   * @param tronAddress TRON 地址
   * @param onStatusChange 状态变化回调
   * @returns 兑换记录ID
   */
  async officialSwap(
    dustAmount: bigint,
    tronAddress: string,
    onStatusChange?: StatusCallback
  ): Promise<number> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    // 验证 TRON 地址格式
    const tronRegex = /^T[A-Za-z1-9]{33}$/;
    if (!tronRegex.test(tronAddress)) {
      throw new Error('Invalid TRON address format');
    }

    onStatusChange?.('准备交易...');

    // 将 TRON 地址转换为字节数组
    const tronAddressBytes = new TextEncoder().encode(tronAddress);

    // 创建交易
    const tx = api.tx.bridge.officialSwap(
      dustAmount.toString(),
      u8aToHex(tronAddressBytes)
    );

    onStatusChange?.('等待签名...');

    // 签名并发送交易
    const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

    // 从事件中提取兑换记录ID
    const swapEvent = events.find(
      ({ event }: any) =>
        event.section === 'bridge' &&
        event.method === 'SwapCreated'
    );

    if (!swapEvent) {
      throw new Error('未找到兑换创建事件');
    }

    // 提取记录ID（假设事件数据格式为 [account, swapId, ...]）
    const swapId = swapEvent.event.data[1].toString();

    return parseInt(swapId, 10);
  }

  /**
   * 做市商桥接：DUST → USDT
   * @param makerId 做市商ID
   * @param dustAmount DUST 数量（最小单位）
   * @param tronAddress TRON 地址
   * @param onStatusChange 状态变化回调
   * @returns 兑换记录ID
   */
  async makerSwap(
    makerId: number,
    dustAmount: bigint,
    tronAddress: string,
    onStatusChange?: StatusCallback
  ): Promise<number> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    // 验证 TRON 地址格式
    const tronRegex = /^T[A-Za-z1-9]{33}$/;
    if (!tronRegex.test(tronAddress)) {
      throw new Error('Invalid TRON address format');
    }

    onStatusChange?.('准备交易...');

    // 将 TRON 地址转换为字节数组
    const tronAddressBytes = new TextEncoder().encode(tronAddress);

    // 创建交易
    const tx = api.tx.bridge.makerSwap(
      makerId,
      dustAmount.toString(),
      u8aToHex(tronAddressBytes)
    );

    onStatusChange?.('等待签名...');

    // 签名并发送交易
    const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

    // 从事件中提取兑换记录ID
    const swapEvent = events.find(
      ({ event }: any) =>
        event.section === 'bridge' &&
        event.method === 'SwapCreated'
    );

    if (!swapEvent) {
      throw new Error('未找到兑换创建事件');
    }

    // 提取记录ID
    const swapId = swapEvent.event.data[1].toString();

    return parseInt(swapId, 10);
  }

  /**
   * 查询用户的兑换历史记录
   * @param account 用户地址
   * @returns 兑换记录列表
   */
  async getSwapHistory(account: string): Promise<SwapRecord[]> {
    const api = this.getApi();

    try {
      // 查询链上存储
      const entries = await api.query.bridge.swapRecords.entries();

      const records: SwapRecord[] = [];

      for (const [key, value] of entries) {
        const record = value.toJSON() as any;

        // 过滤用户
        if (record.account === account) {
          records.push({
            id: record.id,
            account: record.account,
            bridgeType: record.bridgeType,
            dustAmount: BigInt(record.dustAmount),
            usdtAmount: BigInt(record.usdtAmount),
            tronAddress: record.tronAddress,
            makerId: record.makerId,
            status: record.status,
            timestamp: record.timestamp,
            blockNumber: record.blockNumber,
          });
        }
      }

      // 按时间倒序排序
      records.sort((a, b) => b.timestamp - a.timestamp);

      return records;
    } catch (error) {
      console.error('[BridgeService] Get history error:', error);
      throw error;
    }
  }

  /**
   * 查询单个兑换记录
   * @param swapId 兑换记录ID
   * @returns 兑换记录
   */
  async getSwapRecord(swapId: number): Promise<SwapRecord | null> {
    const api = this.getApi();

    try {
      const record = await api.query.bridge.swapRecords(swapId);

      if (record.isEmpty) {
        return null;
      }

      const data = record.toJSON() as any;

      return {
        id: data.id,
        account: data.account,
        bridgeType: data.bridgeType,
        dustAmount: BigInt(data.dustAmount),
        usdtAmount: BigInt(data.usdtAmount),
        tronAddress: data.tronAddress,
        makerId: data.makerId,
        status: data.status,
        timestamp: data.timestamp,
        blockNumber: data.blockNumber,
      };
    } catch (error) {
      console.error('[BridgeService] Get record error:', error);
      throw error;
    }
  }

  /**
   * 获取当前 DUST 价格
   * @returns DUST 价格（USDT）
   */
  async getDustPrice(): Promise<number> {
    const api = this.getApi();

    try {
      const price = await api.query.bridge.dustPrice();

      if (price.isEmpty) {
        return 0.10; // 默认价格
      }

      // 假设价格存储为固定精度的整数（例如：100 = 0.10 USDT）
      return price.toNumber() / 1000;
    } catch (error) {
      console.error('[BridgeService] Get price error:', error);
      return 0.10; // 返回默认价格
    }
  }

  /**
   * 获取用户的 DUST 余额
   * @param account 用户地址
   * @returns DUST 余额（最小单位）
   */
  async getDustBalance(account: string): Promise<bigint> {
    const api = this.getApi();

    try {
      const balance = await api.query.system.account(account);
      const data = balance.toJSON() as any;

      return BigInt(data.data.free);
    } catch (error) {
      console.error('[BridgeService] Get balance error:', error);
      throw error;
    }
  }
}

// 导出单例
export const bridgeService = new BridgeService();
