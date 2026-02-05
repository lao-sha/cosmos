import { getApi } from './api';

export type CreditLevel = 'poor' | 'fair' | 'good' | 'excellent' | 'perfect';

export interface CreditInfo {
  score: number;
  level: CreditLevel;
  completedOrders: number;
  cancelledOrders: number;
  disputes: number;
  avgResponseTime: number; // seconds
  onTimeRate: number; // percentage
  lastUpdated: number;
}

export interface CreditHistory {
  id: string;
  type: 'increase' | 'decrease';
  amount: number;
  reason: string;
  createdAt: number;
}

export const CREDIT_LEVELS: { level: CreditLevel; min: number; max: number; name: string; color: string }[] = [
  { level: 'poor', min: 0, max: 349, name: '较差', color: '#EF4444' },
  { level: 'fair', min: 350, max: 549, name: '一般', color: '#F59E0B' },
  { level: 'good', min: 550, max: 699, name: '良好', color: '#3B82F6' },
  { level: 'excellent', min: 700, max: 849, name: '优秀', color: '#10B981' },
  { level: 'perfect', min: 850, max: 1000, name: '极佳', color: '#6366F1' },
];

export async function getCreditInfo(address: string): Promise<CreditInfo> {
  try {
    const api = await getApi();
    const data = await api.query.tradingCredit.creditScores(address);
    
    if (data.isEmpty) {
      return getDefaultCredit();
    }
    
    const info = data.toJSON() as any;
    return {
      score: info.score || 600,
      level: getScoreLevel(info.score || 600),
      completedOrders: info.completedOrders || 0,
      cancelledOrders: info.cancelledOrders || 0,
      disputes: info.disputes || 0,
      avgResponseTime: info.avgResponseTime || 0,
      onTimeRate: info.onTimeRate || 100,
      lastUpdated: info.lastUpdated || Date.now(),
    };
  } catch (error) {
    console.error('Failed to get credit info:', error);
    return getDefaultCredit();
  }
}

export async function getCreditHistory(address: string, limit = 50): Promise<CreditHistory[]> {
  try {
    const api = await getApi();
    const data = await api.query.tradingCredit.creditHistory(address);
    
    if (data.isEmpty) return [];
    
    return (data.toJSON() as any[]).slice(0, limit).map((item: any) => ({
      id: item.id,
      type: item.amount > 0 ? 'increase' : 'decrease',
      amount: Math.abs(item.amount),
      reason: item.reason,
      createdAt: item.createdAt,
    }));
  } catch (error) {
    return [];
  }
}

function getDefaultCredit(): CreditInfo {
  return {
    score: 600,
    level: 'good',
    completedOrders: 0,
    cancelledOrders: 0,
    disputes: 0,
    avgResponseTime: 0,
    onTimeRate: 100,
    lastUpdated: Date.now(),
  };
}

function getScoreLevel(score: number): CreditLevel {
  for (const level of CREDIT_LEVELS) {
    if (score >= level.min && score <= level.max) {
      return level.level;
    }
  }
  return 'good';
}

export function getLevelConfig(level: CreditLevel) {
  return CREDIT_LEVELS.find(l => l.level === level) || CREDIT_LEVELS[2];
}

export function getScoreColor(score: number): string {
  for (const level of CREDIT_LEVELS) {
    if (score >= level.min && score <= level.max) {
      return level.color;
    }
  }
  return '#3B82F6';
}

export const CREDIT_FACTORS = [
  { factor: '完成订单', impact: '+5~20', description: '成功完成交易' },
  { factor: '准时放币', impact: '+2~5', description: '在规定时间内完成' },
  { factor: '好评率', impact: '+1~10', description: '获得对方好评' },
  { factor: '取消订单', impact: '-10~30', description: '主动取消交易' },
  { factor: '超时未付', impact: '-20~50', description: '付款超时' },
  { factor: '争议败诉', impact: '-30~100', description: '仲裁判定违规' },
];
