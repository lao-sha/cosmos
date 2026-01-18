/**
 * 占卜服务 - 处理占卜结果的链上存储和查询
 */

import { ApiPromise } from '@polkadot/api';
import { getApi } from '@/lib/api';
import { signAndSend, getCurrentSignerAddress } from '@/lib/signer';
import { u8aToHex } from '@polkadot/util';

/**
 * 签名状态回调
 */
export type StatusCallback = (status: string) => void;

/**
 * 占卜类型枚举
 */
export enum DivinationType {
  Bazi = 'Bazi',           // 八字
  Ziwei = 'Ziwei',         // 紫微斗数
  Qimen = 'Qimen',         // 奇门遁甲
  Liuyao = 'Liuyao',       // 六爻
  Meihua = 'Meihua',       // 梅花易数
  Tarot = 'Tarot',         // 塔罗
  Daliuren = 'Daliuren',   // 大六壬
  Xiaoliuren = 'Xiaoliuren', // 小六壬
}

/**
 * 占卜记录
 */
export interface DivinationRecord {
  id: number;
  account: string;
  divinationType: DivinationType;
  resultCid: string;
  timestamp: number;
  blockNumber: number;
}

/**
 * 占卜服务类
 */
export class DivinationService {
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
   * 将占卜结果存储到链上
   * @param divinationType 占卜类型
   * @param resultData 占卜结果数据（将被序列化为JSON）
   * @param onStatusChange 状态变化回调
   * @returns 占卜记录ID
   */
  async storeDivinationResult(
    divinationType: DivinationType,
    resultData: any,
    onStatusChange?: StatusCallback
  ): Promise<number> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    // 将结果数据序列化为JSON
    const resultJson = JSON.stringify(resultData);
    const resultBytes = new TextEncoder().encode(resultJson);
    const resultCid = u8aToHex(resultBytes);

    onStatusChange?.('准备交易...');

    // 创建交易
    const tx = api.tx.divination.storeDivinationResult(
      divinationType,
      resultCid
    );

    onStatusChange?.('等待签名...');

    // 签名并发送交易
    const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

    // 从事件中提取占卜记录ID
    const divinationEvent = events.find(
      ({ event }: any) =>
        event.section === 'divination' &&
        event.method === 'DivinationStored'
    );

    if (!divinationEvent) {
      throw new Error('未找到占卜存储事件');
    }

    // 提取记录ID（假设事件数据格式为 [account, id, type, cid]）
    const recordId = divinationEvent.event.data[1].toString();

    return parseInt(recordId, 10);
  }

  /**
   * 查询用户的占卜历史记录
   * @param account 用户地址
   * @param divinationType 占卜类型（可选，不传则查询所有类型）
   * @returns 占卜记录列表
   */
  async getDivinationHistory(
    account: string,
    divinationType?: DivinationType
  ): Promise<DivinationRecord[]> {
    const api = this.getApi();

    try {
      // 查询链上存储
      const entries = await api.query.divination.divinationRecords.entries();

      const records: DivinationRecord[] = [];

      for (const [key, value] of entries) {
        const record = value.toJSON() as any;

        // 过滤用户和类型
        if (record.account === account) {
          if (!divinationType || record.divinationType === divinationType) {
            records.push({
              id: record.id,
              account: record.account,
              divinationType: record.divinationType,
              resultCid: record.resultCid,
              timestamp: record.timestamp,
              blockNumber: record.blockNumber,
            });
          }
        }
      }

      // 按时间倒序排序
      records.sort((a, b) => b.timestamp - a.timestamp);

      return records;
    } catch (error) {
      console.error('[DivinationService] Get history error:', error);
      throw error;
    }
  }

  /**
   * 查询单个占卜记录
   * @param recordId 记录ID
   * @returns 占卜记录
   */
  async getDivinationRecord(recordId: number): Promise<DivinationRecord | null> {
    const api = this.getApi();

    try {
      const record = await api.query.divination.divinationRecords(recordId);

      if (record.isEmpty) {
        return null;
      }

      const data = record.toJSON() as any;

      return {
        id: data.id,
        account: data.account,
        divinationType: data.divinationType,
        resultCid: data.resultCid,
        timestamp: data.timestamp,
        blockNumber: data.blockNumber,
      };
    } catch (error) {
      console.error('[DivinationService] Get record error:', error);
      throw error;
    }
  }

  /**
   * 解析占卜结果数据
   * @param resultCid 结果CID（十六进制字符串）
   * @returns 解析后的结果数据
   */
  parseResultData<T = any>(resultCid: string): T {
    // 移除 0x 前缀
    const hex = resultCid.startsWith('0x') ? resultCid.slice(2) : resultCid;

    // 将十六进制转换为字节数组
    const bytes = new Uint8Array(
      hex.match(/.{1,2}/g)?.map((byte) => parseInt(byte, 16)) || []
    );

    // 解码为字符串
    const json = new TextDecoder().decode(bytes);

    // 解析JSON
    return JSON.parse(json) as T;
  }

  /**
   * 获取占卜统计信息
   * @param account 用户地址
   * @returns 统计信息
   */
  async getDivinationStats(account: string): Promise<{
    total: number;
    byType: Record<DivinationType, number>;
  }> {
    const records = await this.getDivinationHistory(account);

    const stats = {
      total: records.length,
      byType: {} as Record<DivinationType, number>,
    };

    // 初始化计数器
    Object.values(DivinationType).forEach((type) => {
      stats.byType[type] = 0;
    });

    // 统计各类型数量
    records.forEach((record) => {
      stats.byType[record.divinationType]++;
    });

    return stats;
  }

  /**
   * 删除占卜记录（软删除，仅本地标记）
   * 注意：链上数据无法删除，此方法仅用于本地隐藏
   * @param recordId 记录ID
   */
  async markRecordAsDeleted(recordId: number): Promise<void> {
    // TODO: 实现本地存储标记
    // 可以使用 AsyncStorage 存储已删除的记录ID列表
    console.log('标记记录为已删除:', recordId);
  }

  /**
   * 临时计算八字（免费试算，不保存到链上）
   * @param birthYear 出生年份
   * @param birthMonth 出生月份 (1-12)
   * @param birthDay 出生日期 (1-31)
   * @param birthHour 出生小时 (0-23)
   * @param birthMinute 出生分钟 (0-59)
   * @param gender 性别 ('male' | 'female')
   * @param calendarType 日历类型 ('solar' | 'lunar')
   * @returns 八字命盘数据（JSON 格式）
   */
  async calculateBaziTemp(
    birthYear: number,
    birthMonth: number,
    birthDay: number,
    birthHour: number,
    birthMinute: number,
    gender: 'male' | 'female',
    calendarType: 'solar' | 'lunar' = 'solar'
  ): Promise<any> {
    const api = this.getApi();

    // 准备参数
    const inputType = calendarType === 'solar' ? 0 : 1; // 0=公历, 1=农历
    const params = [birthYear, birthMonth, birthDay, birthHour, birthMinute];
    const genderValue = gender === 'male' ? 0 : 1; // 0=男, 1=女
    const zishiMode = 2; // 2=现代派

    try {
      // 调用 Runtime API（免费，不上链）
      const result = await api.call.baziChartApi.calculateBaziTempUnified(
        inputType,
        params,
        genderValue,
        zishiMode
      );

      if (!result) {
        throw new Error('计算失败，请检查输入参数');
      }

      // 解析 JSON 字符串
      return JSON.parse(result.toString());
    } catch (error) {
      console.error('[DivinationService] Calculate bazi temp error:', error);
      throw error;
    }
  }

  /**
   * 创建八字命盘并保存到链上
   * @param name 命盘名称（可选）
   * @param birthYear 出生年份
   * @param birthMonth 出生月份 (1-12)
   * @param birthDay 出生日期 (1-31)
   * @param birthHour 出生小时 (0-23)
   * @param birthMinute 出生分钟 (0-59)
   * @param gender 性别 ('male' | 'female')
   * @param calendarType 日历类型 ('solar' | 'lunar')
   * @param onStatusChange 状态变化回调
   * @returns 命盘ID
   */
  async createBaziChart(
    name: string | null,
    birthYear: number,
    birthMonth: number,
    birthDay: number,
    birthHour: number,
    birthMinute: number,
    gender: 'male' | 'female',
    calendarType: 'solar' | 'lunar' = 'solar',
    onStatusChange?: StatusCallback
  ): Promise<number> {
    const api = this.getApi();
    const accountAddress = getCurrentSignerAddress();

    if (!accountAddress) {
      throw new Error('No signer address available. Please unlock wallet first.');
    }

    onStatusChange?.('准备交易...');

    // 构造输入类型
    let input;
    if (calendarType === 'solar') {
      input = {
        Solar: {
          year: birthYear,
          month: birthMonth,
          day: birthDay,
          hour: birthHour,
          minute: birthMinute,
        }
      };
    } else {
      input = {
        Lunar: {
          year: birthYear,
          month: birthMonth,
          day: birthDay,
          isLeapMonth: false,
          hour: birthHour,
          minute: birthMinute,
        }
      };
    }

    // 构造其他参数
    const nameParam = name ? new TextEncoder().encode(name) : null;
    const genderParam = gender === 'male' ? 'Male' : 'Female';
    const zishiModeParam = 'Modern';
    const longitudeParam = null; // 不使用真太阳时修正

    onStatusChange?.('等待签名...');

    // 创建交易
    const tx = api.tx.bazi.createBaziChart(
      nameParam,
      input,
      genderParam,
      zishiModeParam,
      longitudeParam
    );

    // 签名并发送交易
    const { events } = await signAndSend(api, tx, accountAddress, onStatusChange);

    // 从事件中提取命盘ID
    const baziEvent = events.find(
      ({ event }: any) =>
        event.section === 'bazi' &&
        event.method === 'BaziChartCreated'
    );

    if (!baziEvent) {
      throw new Error('未找到八字创建事件');
    }

    // 提取命盘ID（假设事件数据格式为 [owner, chart_id, birth_time]）
    const chartId = baziEvent.event.data[1].toString();

    return parseInt(chartId, 10);
  }
}

// 导出单例
export const divinationService = new DivinationService();
