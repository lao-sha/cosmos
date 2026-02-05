import { getApi } from './api';

export type MembershipLevel = 'normal' | 'bronze' | 'silver' | 'gold' | 'platinum' | 'diamond' | 'supreme';

export interface MembershipInfo {
  level: MembershipLevel;
  points: number;
  nextLevelPoints: number;
  totalSpent: string;
  joinedAt: number;
  expiresAt?: number;
  benefits: string[];
}

export interface LevelConfig {
  level: MembershipLevel;
  name: string;
  minPoints: number;
  color: string;
  benefits: string[];
  discountRate: number;
  dailyLimit: number;
}

export const MEMBERSHIP_LEVELS: LevelConfig[] = [
  {
    level: 'normal',
    name: '普通会员',
    minPoints: 0,
    color: '#6B7280',
    benefits: ['基础交易功能', '标准手续费'],
    discountRate: 0,
    dailyLimit: 1000,
  },
  {
    level: 'bronze',
    name: '青铜会员',
    minPoints: 100,
    color: '#CD7F32',
    benefits: ['手续费9.5折', '专属客服', '优先匹配'],
    discountRate: 5,
    dailyLimit: 5000,
  },
  {
    level: 'silver',
    name: '白银会员',
    minPoints: 500,
    color: '#C0C0C0',
    benefits: ['手续费9折', '专属客服', '优先匹配', '生日礼包'],
    discountRate: 10,
    dailyLimit: 10000,
  },
  {
    level: 'gold',
    name: '黄金会员',
    minPoints: 2000,
    color: '#FFD700',
    benefits: ['手续费8.5折', '1对1客服', '优先匹配', '生日礼包', '专属活动'],
    discountRate: 15,
    dailyLimit: 50000,
  },
  {
    level: 'platinum',
    name: '铂金会员',
    minPoints: 5000,
    color: '#E5E4E2',
    benefits: ['手续费8折', 'VIP客服', '最优匹配', '生日礼包', '专属活动', '做市商快速审核'],
    discountRate: 20,
    dailyLimit: 100000,
  },
  {
    level: 'diamond',
    name: '钻石会员',
    minPoints: 10000,
    color: '#B9F2FF',
    benefits: ['手续费7.5折', 'VIP客服', '最优匹配', '生日礼包', '专属活动', '做市商免审核', '专属徽章'],
    discountRate: 25,
    dailyLimit: 500000,
  },
  {
    level: 'supreme',
    name: '至尊会员',
    minPoints: 50000,
    color: '#FFD700',
    benefits: ['手续费7折', '私人客服', '最优匹配', '生日礼包', '专属活动', '做市商免审核', '专属徽章', '线下活动'],
    discountRate: 30,
    dailyLimit: 0, // Unlimited
  },
];

export async function getMembershipInfo(address: string): Promise<MembershipInfo> {
  try {
    const api = await getApi();
    const data = await api.query.entityMember.memberInfo(address);
    
    if (data.isEmpty) {
      return getDefaultMembership();
    }
    
    const info = data.toJSON() as any;
    const level = info.level as MembershipLevel;
    const levelConfig = MEMBERSHIP_LEVELS.find(l => l.level === level) || MEMBERSHIP_LEVELS[0];
    const nextLevel = MEMBERSHIP_LEVELS.find(l => l.minPoints > info.points);
    
    return {
      level,
      points: info.points || 0,
      nextLevelPoints: nextLevel?.minPoints || levelConfig.minPoints,
      totalSpent: info.totalSpent?.toString() || '0',
      joinedAt: info.joinedAt || Date.now(),
      expiresAt: info.expiresAt,
      benefits: levelConfig.benefits,
    };
  } catch (error) {
    console.error('Failed to get membership info:', error);
    return getDefaultMembership();
  }
}

export async function getMembershipHistory(address: string): Promise<any[]> {
  try {
    const api = await getApi();
    const data = await api.query.entityMember.pointsHistory(address);
    
    if (data.isEmpty) return [];
    
    return (data.toJSON() as any[]).map((item: any) => ({
      id: item.id,
      type: item.type,
      points: item.points,
      description: item.description,
      createdAt: item.createdAt,
    }));
  } catch (error) {
    return [];
  }
}

function getDefaultMembership(): MembershipInfo {
  return {
    level: 'normal',
    points: 0,
    nextLevelPoints: 100,
    totalSpent: '0',
    joinedAt: Date.now(),
    benefits: MEMBERSHIP_LEVELS[0].benefits,
  };
}

export function getLevelConfig(level: MembershipLevel): LevelConfig {
  return MEMBERSHIP_LEVELS.find(l => l.level === level) || MEMBERSHIP_LEVELS[0];
}

export function getNextLevel(currentLevel: MembershipLevel): LevelConfig | null {
  const currentIndex = MEMBERSHIP_LEVELS.findIndex(l => l.level === currentLevel);
  if (currentIndex < MEMBERSHIP_LEVELS.length - 1) {
    return MEMBERSHIP_LEVELS[currentIndex + 1];
  }
  return null;
}
