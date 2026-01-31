import { useChainStore } from '@/src/stores/chain';
import { ApiPromise } from '@polkadot/api';

type AnyCodec = any;

export interface ChatSession {
  id: string;
  participantName: string;
  participantAvatar?: string;
  lastMessage: string;
  lastMessageTime: string;
  unreadCount: number;
  isOnline: boolean;
}

export interface ChatMessage {
  id: string;
  content: string;
  senderId: string;
  timestamp: string;
  isRead: boolean;
  contentCid?: string;
}

export interface DivinationProvider {
  id: string;
  name: string;
  avatar?: string;
  rating: number;
  orderCount: number;
  specialties: string[];
  price: number;
  isOnline: boolean;
  deposit: string;
}

export class ChainService {
  private api: ApiPromise | null = null;

  setApi(api: ApiPromise) {
    this.api = api;
  }

  private getApi(): ApiPromise {
    if (!this.api) {
      const { api } = useChainStore.getState();
      if (!api) {
        throw new Error('Chain not connected');
      }
      this.api = api;
    }
    return this.api;
  }

  async getChatSessions(accountId: string): Promise<ChatSession[]> {
    try {
      const api = this.getApi();
      const sessionIds = await api.query.chatCore.userSessions.keys(accountId);
      
      const sessions: ChatSession[] = [];
      for (const key of sessionIds) {
        const sessionId = key.args[1].toHex();
        const session = await api.query.chatCore.sessions(sessionId) as AnyCodec;
        
        if (session?.isSome) {
          const sessionData = session.unwrap();
          const unreadCount = await api.query.chatCore.unreadCount([accountId, sessionId]) as AnyCodec;
          
          sessions.push({
            id: sessionId,
            participantName: sessionData.participants[0].toString().slice(0, 8) + '...',
            lastMessage: '加密消息',
            lastMessageTime: sessionData.lastActive.toString(),
            unreadCount: unreadCount.toNumber(),
            isOnline: false,
          });
        }
      }
      return sessions;
    } catch (error) {
      console.error('Failed to get chat sessions:', error);
      return [];
    }
  }

  async getChatMessages(sessionId: string): Promise<ChatMessage[]> {
    try {
      const api = this.getApi();
      const messageIds = await api.query.chatCore.sessionMessages.keys(sessionId);
      
      const messages: ChatMessage[] = [];
      for (const key of messageIds) {
        const msgId = key.args[1].toString();
        const message = await api.query.chatCore.messages(msgId) as AnyCodec;
        
        if (message?.isSome) {
          const msgData = message.unwrap();
          messages.push({
            id: msgId,
            content: '加密消息 (CID: ' + msgData.contentCid.toHuman() + ')',
            senderId: msgData.sender.toString(),
            timestamp: msgData.sentAt.toString(),
            isRead: msgData.isRead.isTrue,
            contentCid: msgData.contentCid.toHuman() as string,
          });
        }
      }
      return messages;
    } catch (error) {
      console.error('Failed to get chat messages:', error);
      return [];
    }
  }

  async sendMessage(
    receiver: string,
    contentCid: string,
    msgType: number = 0,
    sessionId?: string
  ): Promise<string | null> {
    try {
      const api = this.getApi();
      const tx = api.tx.chatCore.sendMessage(
        receiver,
        contentCid,
        msgType,
        sessionId || null
      );
      return tx.toHex();
    } catch (error) {
      console.error('Failed to create send message tx:', error);
      return null;
    }
  }

  async getDivinationProviders(): Promise<DivinationProvider[]> {
    try {
      const api = this.getApi();
      const providerEntries = await api.query.divinationMarket.providers.entries();
      
      const providers: DivinationProvider[] = [];
      for (const [key, value] of providerEntries) {
        if ((value as AnyCodec)?.isSome) {
          const provider = (value as AnyCodec).unwrap();
          providers.push({
            id: key.args[0].toString(),
            name: provider.name?.toHuman() as string || '未知',
            rating: (provider.rating?.toNumber() || 0) / 10,
            orderCount: provider.totalOrders?.toNumber() || 0,
            specialties: provider.supportedTypes?.toHuman() as string[] || [],
            price: provider.minPrice?.toNumber() || 0,
            isOnline: provider.status?.toHuman() === 'Active',
            deposit: provider.deposit?.toString() || '0',
          });
        }
      }
      return providers;
    } catch (error) {
      console.error('Failed to get divination providers:', error);
      return [];
    }
  }

  async getBalance(accountId: string): Promise<string> {
    try {
      const api = this.getApi();
      const account = await api.query.system.account(accountId) as AnyCodec;
      return account.data.free.toString();
    } catch (error) {
      console.error('Failed to get balance:', error);
      return '0';
    }
  }

  async getChainInfo(): Promise<{ chain: string; nodeName: string; nodeVersion: string }> {
    try {
      const api = this.getApi();
      const [chain, nodeName, nodeVersion] = await Promise.all([
        api.rpc.system.chain(),
        api.rpc.system.name(),
        api.rpc.system.version(),
      ]);
      return {
        chain: chain.toString(),
        nodeName: nodeName.toString(),
        nodeVersion: nodeVersion.toString(),
      };
    } catch (error) {
      console.error('Failed to get chain info:', error);
      return { chain: 'Unknown', nodeName: 'Unknown', nodeVersion: 'Unknown' };
    }
  }

  /**
   * 获取账户的转账记录
   * 通过查询最近区块的事件来获取转账记录
   * 注意：节点默认只保留最近 256 个区块的状态
   */
  async getTransferHistory(accountId: string, limit: number = 50): Promise<TransferRecord[]> {
    try {
      const api = this.getApi();
      const records: TransferRecord[] = [];
      
      // 获取当前区块号
      const currentHeader = await api.rpc.chain.getHeader();
      const currentBlock = currentHeader.number.toNumber();
      
      // 只扫描最近 200 个区块（节点默认保留 256 个区块状态）
      const blocksToScan = Math.min(200, currentBlock);
      
      for (let blockNum = currentBlock; blockNum > currentBlock - blocksToScan && records.length < limit; blockNum--) {
        try {
          const blockHash = await api.rpc.chain.getBlockHash(blockNum);
          const signedBlock = await api.rpc.chain.getBlock(blockHash);
          const apiAt = await api.at(blockHash);
          const events = await apiAt.query.system.events() as AnyCodec;
          
          // 遍历区块中的事件
          events.forEach((record: any, index: number) => {
            const { event } = record;
            
            // 检查是否是 Balances.Transfer 事件
            if (event.section === 'balances' && event.method === 'Transfer') {
              const [from, to, amount] = event.data;
              const fromAddr = from.toString();
              const toAddr = to.toString();
              const rawAmount = amount.toString();
              
              // 检查是否与当前账户相关
              if (fromAddr === accountId || toAddr === accountId) {
                const isIncoming = toAddr === accountId;
                // 链上精度：1 COS = 1e12 最小单位
                const cosAmount = (parseFloat(rawAmount) / 1e12).toFixed(4);
                
                // 获取交易哈希（从区块的 extrinsics 中）
                const extrinsicIndex = record.phase.isApplyExtrinsic 
                  ? record.phase.asApplyExtrinsic.toNumber() 
                  : 0;
                const extrinsic = signedBlock.block.extrinsics[extrinsicIndex];
                const txHash = extrinsic ? extrinsic.hash.toHex() : `0x${blockNum}-${index}`;
                
                records.push({
                  id: `${blockNum}-${index}`,
                  hash: txHash.slice(0, 10) + '...' + txHash.slice(-6),
                  type: isIncoming ? 'transfer_in' : 'transfer_out',
                  amount: cosAmount,
                  rawAmount,
                  counterparty: isIncoming 
                    ? fromAddr.slice(0, 6) + '...' + fromAddr.slice(-4)
                    : toAddr.slice(0, 6) + '...' + toAddr.slice(-4),
                  status: 'confirmed',
                  timestamp: `区块 #${blockNum}`,
                  blockNumber: blockNum,
                });
              }
            }
          });
        } catch (blockError) {
          // 区块状态已被修剪，停止扫描更早的区块
          break;
        }
      }
      
      return records.slice(0, limit);
    } catch (error) {
      console.error('Failed to get transfer history:', error);
      return [];
    }
  }

  async getBaziChart(chartId: number): Promise<FullBaziChartForApi | null> {
    try {
      const api = this.getApi();
      // Runtime API 返回 JSON 字符串（已经是 camelCase 格式）
      const result = await (api.call as any).baziChartApi.getFullBaziChart(chartId);
      
      if (result?.isSome) {
        const jsonStr = result.unwrap().toString();
        console.log('链端返回命盘 JSON:', jsonStr);
        const data = JSON.parse(jsonStr);
        // 链端返回的 JSON 已经是 camelCase，直接返回
        return data as FullBaziChartForApi;
      }
      return null;
    } catch (error) {
      console.error('Failed to get bazi chart:', error);
      return null;
    }
  }

  async getUserBaziCharts(accountId: string): Promise<BaziChartSummary[]> {
    try {
      const api = this.getApi();
      const chartIds = await api.query.bazi.userCharts(accountId) as AnyCodec;
      
      const charts: BaziChartSummary[] = [];
      const ids = chartIds.toJSON() as number[] || [];
      
      for (const chartId of ids) {
        // 优先查询精简存储（新格式）
        let chartData = await (api.query.bazi as any).chartCompactById(chartId) as AnyCodec;
        
        // 如果精简存储没有，尝试旧格式
        if (!chartData || chartData.isNone) {
          chartData = await (api.query.bazi as any).chartById(chartId) as AnyCodec;
        }
        
        if (chartData?.isSome) {
          const chart = chartData.unwrap();
          charts.push({
            chartId,
            name: chart.name?.toHuman() as string || '',
            birthTime: chart.birthTime?.toHuman(),
            gender: chart.gender?.toHuman() as string,
            privacyMode: chart.privacyMode?.toHuman() as string,
            timestamp: chart.timestamp?.toNumber() || 0,
          });
        }
      }
      return charts;
    } catch (error) {
      console.error('Failed to get user bazi charts:', error);
      return [];
    }
  }

  private parseBaziChartResult(data: any): FullBaziChartForApi {
    const toHuman = (v: any) => v?.toHuman?.() ?? v;
    const toNumber = (v: any) => v?.toNumber?.() ?? (typeof v === 'number' ? v : 0);
    
    return {
      gender: toHuman(data.gender),
      birthYear: toNumber(data.birthYear),
      inputCalendarType: toHuman(data.inputCalendarType),
      sizhu: {
        yearZhu: this.parseZhu(data.sizhu?.yearZhu),
        monthZhu: this.parseZhu(data.sizhu?.monthZhu),
        dayZhu: this.parseZhu(data.sizhu?.dayZhu),
        hourZhu: this.parseZhu(data.sizhu?.hourZhu),
        rizhu: toHuman(data.sizhu?.rizhu),
      },
      kongwang: toHuman(data.kongwang),
      xingyun: toHuman(data.xingyun),
      shenshaList: (data.shenshaList || []).map((s: any) => toHuman(s)),
      wuxingStrength: toHuman(data.wuxingStrength),
      dayunList: (data.dayunList || []).map((d: any) => ({
        ganzhi: toHuman(d.ganzhi),
        tianganShishen: toHuman(d.tianganShishen),
        dizhiBenqiShishen: toHuman(d.dizhiBenqiShishen),
        startAge: toNumber(d.startAge),
        startYear: toNumber(d.startYear),
        startMonth: toNumber(d.startMonth),
        startDay: toNumber(d.startDay),
        endAge: toNumber(d.endAge),
        endYear: toNumber(d.endYear),
        changsheng: toHuman(d.changsheng),
        liunianList: (d.liunianList || []).map((l: any) => ({
          ganzhi: toHuman(l.ganzhi),
          tianganShishen: toHuman(l.tianganShishen),
          dizhiBenqiShishen: toHuman(l.dizhiBenqiShishen),
          year: toNumber(l.year),
          age: toNumber(l.age),
        })),
      })),
      qiyun: {
        ageYears: toNumber(data.qiyun?.ageYears),
        ageMonths: toNumber(data.qiyun?.ageMonths),
        ageDays: toNumber(data.qiyun?.ageDays),
        isShun: data.qiyun?.isShun?.isTrue ?? data.qiyun?.isShun ?? false,
        jiaoyunYear: toNumber(data.qiyun?.jiaoyunYear),
        jiaoyunMonth: toNumber(data.qiyun?.jiaoyunMonth),
        jiaoyunDay: toNumber(data.qiyun?.jiaoyunDay),
      },
      analysis: {
        geJu: toHuman(data.analysis?.geJu),
        qiangRuo: toHuman(data.analysis?.qiangRuo),
        yongShen: toHuman(data.analysis?.yongShen),
        yongShenType: toHuman(data.analysis?.yongShenType),
        xiShen: toHuman(data.analysis?.xiShen),
        jiShen: toHuman(data.analysis?.jiShen),
        score: toNumber(data.analysis?.score),
      },
    };
  }

  private parseZhu(zhu: any): ZhuForApi {
    const toHuman = (v: any) => v?.toHuman?.() ?? v;
    const toNumber = (v: any) => v?.toNumber?.() ?? (typeof v === 'number' ? v : 0);
    
    return {
      ganzhi: toHuman(zhu?.ganzhi),
      tianganShishen: toHuman(zhu?.tianganShishen),
      dizhiBenqiShishen: toHuman(zhu?.dizhiBenqiShishen),
      cangganList: (zhu?.cangganList || []).map((c: any) => ({
        gan: toHuman(c.gan),
        shishen: toHuman(c.shishen),
        cangganType: toHuman(c.cangganType),
        weight: toNumber(c.weight),
      })),
      nayin: toHuman(zhu?.nayin),
      changsheng: toHuman(zhu?.changsheng),
      zizuo: toHuman(zhu?.zizuo),
    };
  }

  /**
   * 临时排盘（免费、不存储到链上）
   * 调用链端 Runtime API: calculateBaziTempUnified
   * @param inputType 0=Solar, 1=Lunar, 2=SiZhu
   * @param params 参数数组 [year, month, day, hour, minute] 或 [year, month, day, hour, minute, longitude]
   * @param gender 0=Male, 1=Female
   * @param zishiMode 1=Traditional, 2=Modern
   */
  async calculateBaziTemp(
    inputType: 'solar' | 'lunar',
    params: {
      year: number;
      month: number;
      day: number;
      hour: number;
      minute: number;
      isLeapMonth?: boolean;
      longitude?: number;
    },
    gender: 'male' | 'female',
    zishiMode: 'modern' | 'traditional'
  ): Promise<FullBaziChartForApi | null> {
    try {
      const api = this.getApi();
      
      // 构建参数数组
      const inputTypeNum = inputType === 'solar' ? 0 : 1;
      let paramsArray: number[];
      
      if (inputType === 'solar') {
        paramsArray = [params.year, params.month, params.day, params.hour, params.minute];
        if (params.longitude !== undefined) {
          paramsArray.push(Math.round(params.longitude * 100000));
        }
      } else {
        paramsArray = [
          params.year, 
          params.month, 
          params.day, 
          params.isLeapMonth ? 1 : 0,
          params.hour, 
          params.minute
        ];
        if (params.longitude !== undefined) {
          paramsArray.push(Math.round(params.longitude * 100000));
        }
      }
      
      const genderNum = gender === 'male' ? 0 : 1;
      const zishiModeNum = zishiMode === 'traditional' ? 1 : 2;
      
      console.log('调用链端临时排盘 API:', { inputTypeNum, paramsArray, genderNum, zishiModeNum });
      
      // 调用 Runtime API
      const result = await (api.call as any).baziChartApi.calculateBaziTempUnified(
        inputTypeNum,
        paramsArray,
        genderNum,
        zishiModeNum
      );
      
      if (result?.isSome) {
        const jsonStr = result.unwrap().toHuman();
        console.log('链端临时排盘返回 JSON:', jsonStr);
        
        // 解析 JSON 字符串
        const parsed = JSON.parse(jsonStr);
        return this.parseJsonBaziChart(parsed);
      }
      
      console.log('链端临时排盘返回 None');
      return null;
    } catch (error) {
      console.error('临时排盘失败:', error);
      return null;
    }
  }

  private parseJsonBaziChart(data: any): FullBaziChartForApi {
    return {
      gender: data.gender,
      birthYear: data.birth_year,
      inputCalendarType: data.input_calendar_type,
      sizhu: {
        yearZhu: this.parseJsonZhu(data.sizhu?.year_zhu),
        monthZhu: this.parseJsonZhu(data.sizhu?.month_zhu),
        dayZhu: this.parseJsonZhu(data.sizhu?.day_zhu),
        hourZhu: this.parseJsonZhu(data.sizhu?.hour_zhu),
        rizhu: data.sizhu?.rizhu,
      },
      kongwang: data.kongwang,
      xingyun: data.xingyun,
      shenshaList: data.shensha_list || [],
      wuxingStrength: data.wuxing_strength,
      dayunList: (data.dayun_list || []).map((d: any) => ({
        ganzhi: d.ganzhi,
        tianganShishen: d.tiangan_shishen,
        dizhiBenqiShishen: d.dizhi_benqi_shishen,
        startAge: d.start_age,
        startYear: d.start_year,
        startMonth: d.start_month,
        startDay: d.start_day,
        endAge: d.end_age,
        endYear: d.end_year,
        changsheng: d.changsheng,
        liunianList: (d.liunian_list || []).map((l: any) => ({
          ganzhi: l.ganzhi,
          tianganShishen: l.tiangan_shishen,
          dizhiBenqiShishen: l.dizhi_benqi_shishen,
          year: l.year,
          age: l.age,
        })),
      })),
      qiyun: {
        ageYears: data.qiyun?.age_years,
        ageMonths: data.qiyun?.age_months,
        ageDays: data.qiyun?.age_days,
        isShun: data.qiyun?.is_shun,
        jiaoyunYear: data.qiyun?.jiaoyun_year,
        jiaoyunMonth: data.qiyun?.jiaoyun_month,
        jiaoyunDay: data.qiyun?.jiaoyun_day,
      },
      analysis: {
        geJu: data.analysis?.ge_ju,
        qiangRuo: data.analysis?.qiang_ruo,
        yongShen: data.analysis?.yong_shen,
        yongShenType: data.analysis?.yong_shen_type,
        xiShen: data.analysis?.xi_shen,
        jiShen: data.analysis?.ji_shen,
        score: data.analysis?.score,
      },
    };
  }

  private parseJsonZhu(zhu: any): ZhuForApi {
    if (!zhu) {
      return {
        ganzhi: { gan: '', zhi: '' },
        tianganShishen: '',
        dizhiBenqiShishen: '',
        cangganList: [],
        nayin: '',
        changsheng: '',
        zizuo: '',
      };
    }
    return {
      ganzhi: zhu.ganzhi,
      tianganShishen: zhu.tiangan_shishen,
      dizhiBenqiShishen: zhu.dizhi_benqi_shishen,
      cangganList: (zhu.canggan_list || []).map((c: any) => ({
        gan: c.gan,
        shishen: c.shishen,
        cangganType: c.canggan_type,
        weight: c.weight,
      })),
      nayin: zhu.nayin,
      changsheng: zhu.changsheng,
      zizuo: zhu.zizuo,
    };
  }
}

export interface BaziChartSummary {
  chartId: number;
  name: string;
  birthTime: any;
  gender: string;
  privacyMode: string;
  timestamp: number;
}

export interface FullBaziChartForApi {
  gender: string;
  birthYear: number;
  inputCalendarType: string;
  sizhu: SiZhuForApi;
  kongwang: any;
  xingyun: any;
  shenshaList: any[];
  wuxingStrength: any;
  dayunList: DaYunForApi[];
  qiyun: QiYunForApi;
  analysis: AnalysisForApi;
}

export interface SiZhuForApi {
  yearZhu: ZhuForApi;
  monthZhu: ZhuForApi;
  dayZhu: ZhuForApi;
  hourZhu: ZhuForApi;
  rizhu: string;
}

export interface ZhuForApi {
  ganzhi: any;
  tianganShishen: string;
  dizhiBenqiShishen: string;
  cangganList: CangGanForApi[];
  nayin: string;
  changsheng: string;
  zizuo: string;
}

export interface CangGanForApi {
  gan: string;
  shishen: string;
  cangganType: string;
  weight: number;
}

export interface DaYunForApi {
  ganzhi: any;
  tianganShishen: string;
  dizhiBenqiShishen: string;
  startAge: number;
  startYear: number;
  startMonth: number;
  startDay: number;
  endAge: number;
  endYear: number;
  changsheng: string;
  liunianList: LiuNianForApi[];
}

export interface LiuNianForApi {
  ganzhi: any;
  tianganShishen: string;
  dizhiBenqiShishen: string;
  year: number;
  age: number;
}

export interface QiYunForApi {
  ageYears: number;
  ageMonths: number;
  ageDays: number;
  isShun: boolean;
  jiaoyunYear: number;
  jiaoyunMonth: number;
  jiaoyunDay: number;
}

export interface AnalysisForApi {
  geJu: string;
  qiangRuo: string;
  yongShen: string;
  yongShenType: string;
  xiShen: string;
  jiShen: string;
  score: number;
}

// 交易记录相关类型
export interface TransferRecord {
  id: string;
  hash: string;
  type: 'transfer_in' | 'transfer_out';
  amount: string;  // COS 单位（已转换）
  rawAmount: string;  // 链上原始值
  counterparty: string;
  status: 'confirmed';
  timestamp: string;
  blockNumber: number;
}

export const chainService = new ChainService();
